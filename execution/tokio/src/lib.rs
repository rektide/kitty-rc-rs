//! Single-owner reusable sessions. Cancelable exchange futures take the socket
//! out of the session before I/O and return it only at a verified clean boundary.
use kitty_rc_commands::{Command, Mode};
use kitty_rc_endpoint::{Address, Endpoint};
use kitty_rc_exchange::{
    Completion, Evidence, ExecutionError, Failure, Outcome, Phase, Prepared, prepare,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    time::Instant,
};

trait Socket: AsyncRead + AsyncWrite + Unpin + Send {
    fn try_read(&self, bytes: &mut [u8]) -> std::io::Result<usize>;
}
impl Socket for tokio::net::UnixStream {
    fn try_read(&self, b: &mut [u8]) -> std::io::Result<usize> {
        self.try_read(b)
    }
}
impl Socket for tokio::net::TcpStream {
    fn try_read(&self, b: &mut [u8]) -> std::io::Result<usize> {
        self.try_read(b)
    }
}
static NEXT_CONNECTION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
/// Local transport lifetime only, never a server identity or remote request ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionToken(u64);
struct Connection {
    socket: Box<dyn Socket>,
    decoder: kitty_rc_frame::FrameDecoder,
    token: ConnectionToken,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionOptions {
    pub deadline: Instant,
    pub max_request: usize,
    pub max_response: usize,
}
pub struct Session {
    endpoint: Endpoint,
    connection: Option<Connection>,
}
impl Session {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            connection: None,
        }
    }
    pub fn is_connected(&self) -> bool {
        // Transport ownership, not a liveness probe or server identity claim.
        self.connection.is_some()
    }
    pub fn connection_token(&self) -> Option<ConnectionToken> {
        self.connection.as_ref().map(|c| c.token)
    }
    pub async fn execute<C: Command>(
        &mut self,
        command: C,
        deadline: Instant,
        max_request: usize,
        max_response: usize,
    ) -> Result<Completion<C::Output>, Failure> {
        self.execute_with_options(
            command,
            SessionOptions {
                deadline,
                max_request,
                max_response,
            },
        )
        .await
    }
    pub async fn execute_with_options<C: Command>(
        &mut self,
        command: C,
        options: SessionOptions,
    ) -> Result<Completion<C::Output>, Failure> {
        let prepared = prepare(&command, &self.endpoint.protection, options.max_request)
            .map_err(Failure::Prepare)?;
        self.execute_prepared(
            prepared,
            options.deadline,
            options.max_response,
            Evidence::default(),
        )
        .await
        .decode::<C>()
    }
    pub async fn execute_prepared(
        &mut self,
        prepared: Prepared,
        deadline: Instant,
        max_response: usize,
        evidence: Evidence,
    ) -> Outcome {
        // Before any await: cancellation owns/drops this transport, never leaves
        // a partly consumed socket sitting in the borrowed session.
        let mut connection = self.connection.take();
        let reused = connection.is_some();
        let address = &self.endpoint.address;
        let reusable = prepared.reusable();
        let mode = prepared.mode();
        let work = async {
            evidence.set_phase(Phase::Connect);
            if Instant::now() >= deadline {
                return Err(ExecutionError::Deadline {
                    phase: Phase::Connect,
                });
            }
            if connection.is_none() {
                let socket: Box<dyn Socket> = match address {
                    Address::Unix(path) => Box::new(
                        tokio::net::UnixStream::connect(path)
                            .await
                            .map_err(|source| ExecutionError::Io {
                                endpoint: address.clone(),
                                phase: Phase::Connect,
                                source,
                            })?,
                    ),
                    Address::Tcp(addr) => Box::new(
                        tokio::net::TcpStream::connect(addr)
                            .await
                            .map_err(|source| ExecutionError::Io {
                                endpoint: address.clone(),
                                phase: Phase::Connect,
                                source,
                            })?,
                    ),
                };
                let id = NEXT_CONNECTION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                assert!(id != u64::MAX, "connection token exhausted");
                connection = Some(Connection {
                    socket,
                    decoder: kitty_rc_frame::FrameDecoder::with_limits(
                        max_response,
                        max_response,
                        1,
                    ),
                    token: ConnectionToken(id),
                });
            }
            let conn = connection.as_mut().unwrap();
            if reused {
                evidence.set_phase(Phase::IdleCheck);
                match conn.socket.try_read(&mut [0; 1]) {
                    Ok(0) => return Err(ExecutionError::IdleClosed),
                    Ok(_) => return Err(ExecutionError::Surplus),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(source) => {
                        return Err(ExecutionError::Io {
                            endpoint: address.clone(),
                            phase: Phase::IdleCheck,
                            source,
                        });
                    }
                }
                conn.decoder.reconfigure(max_response, max_response, 1)?;
            }
            // Fresh first-send protection after queue/connect, exactly once.
            evidence.set_phase(Phase::Prepare);
            let bytes = prepared.protect(&self.endpoint.protection)?;
            if Instant::now() >= deadline {
                return Err(ExecutionError::Deadline {
                    phase: Phase::Prepare,
                });
            }
            evidence.write_started();
            evidence.set_phase(Phase::Write);
            conn.socket
                .write_all(&bytes)
                .await
                .map_err(|source| ExecutionError::Io {
                    endpoint: address.clone(),
                    phase: Phase::Write,
                    source,
                })?;
            evidence.outbound_complete();
            if mode == Mode::SubmissionOnly {
                return Ok(Outcome::Submitted);
            }
            evidence.set_phase(Phase::Read);
            let mut buf = [0; 8192];
            loop {
                let n = conn
                    .socket
                    .read(&mut buf)
                    .await
                    .map_err(|source| ExecutionError::Io {
                        endpoint: address.clone(),
                        phase: Phase::Read,
                        source,
                    })?;
                if n == 0 {
                    conn.decoder.finish()?;
                    return Err(ExecutionError::Eof);
                }
                conn.decoder.extend(&buf[..n])?;
                if let Some(payload) = conn.decoder.next_frame()? {
                    if !conn.decoder.is_idle() {
                        return Err(ExecutionError::Surplus);
                    }
                    return Ok(Outcome::Reply(payload));
                }
            }
        };
        let result = match tokio::time::timeout_at(deadline, work).await {
            Ok(result) => result,
            Err(_) => Err(ExecutionError::Deadline {
                phase: evidence.phase(),
            }),
        };
        match result {
            Ok(outcome) => {
                if reusable && matches!(outcome, Outcome::Reply(_)) && !outcome.unresolved() {
                    self.connection = connection;
                }
                outcome
            }
            Err(reason) => Outcome::Interrupted {
                progress: evidence.progress(),
                reason,
            },
        }
    }
}

pub async fn drive(
    address: &Address,
    prepared: Prepared,
    deadline: Instant,
    max_response: usize,
    evidence: Evidence,
) -> Outcome {
    Session::new(Endpoint {
        address: address.clone(),
        protection: kitty_rc_endpoint::Protection::Plain,
    })
    .execute_prepared(prepared, deadline, max_response, evidence)
    .await
}
