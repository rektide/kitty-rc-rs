use super::*;
use kitty_rc_exchange::{PreparedUpload, ResponseHead, response_head};

/// Source ownership is transferred once, never cloned or replayed.
pub struct UploadSource {
    reader: Box<dyn AsyncRead + Unpin + Send>,
    retained_bytes: usize,
}
impl UploadSource {
    /// The caller must account for all memory retained by this reader (including
    /// hidden buffers/captured state). Future allocations must fit that declared
    /// bound, including owned error causes it can return. The registry retains
    /// this reservation with a producer error until native take/release. It
    /// cannot introspect arbitrary reader implementations or hard-preempt them.
    pub fn reader<R: AsyncRead + Unpin + Send + 'static>(reader: R, retained_bytes: usize) -> Self {
        Self {
            reader: Box::new(reader),
            retained_bytes: retained_bytes.saturating_add(std::mem::size_of::<R>()),
        }
    }
    pub fn bytes(bytes: Vec<u8>) -> Self {
        let retained = bytes.capacity();
        Self::reader(std::io::Cursor::new(bytes), retained)
    }
    pub fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}
#[derive(Debug, Clone, Copy)]
pub struct UploadOptions {
    pub session: SessionOptions,
    /// Cumulative raw source limit; never implies a body-sized allocation.
    pub max_upload: u64,
}

impl Session {
    /// Dedicated consuming conversation: first chunk, handshake, remaining chunks,
    /// exactly one EOF marker, and a terminal result (async for run).
    pub async fn upload_prepared(
        &mut self,
        prepared: PreparedUpload,
        mut source: UploadSource,
        deadline: Instant,
        max_response: usize,
        max_upload: u64,
        evidence: Evidence,
    ) -> Outcome {
        self.last_evidence = Some(evidence.clone());
        let mut connection = self.connection.take();
        let reused = connection.is_some();
        let address = &self.endpoint.address;
        if Instant::now() >= deadline {
            return Outcome::Interrupted {
                progress: evidence.progress(),
                reason: ExecutionError::Deadline {
                    phase: Phase::Source,
                },
            };
        }
        let work = async {
            // Bounded lookahead. Empty PNG is invalid locally; empty run still
            // sends its stream identity + async EOF, without fictitious handshake.
            evidence.set_phase(Phase::Source);
            let mut raw = vec![0; prepared.key().raw_chunk_bytes()];
            let mut n = source
                .reader
                .read(&mut raw)
                .await
                .map_err(ExecutionError::Source)?;
            if prepared.key().requires_png() {
                while n < 8 {
                    let more = source
                        .reader
                        .read(&mut raw[n..8])
                        .await
                        .map_err(ExecutionError::Source)?;
                    if more == 0 {
                        return Err(ExecutionError::InvalidPng);
                    }
                    n += more;
                }
                if &raw[..8] != b"\x89PNG\r\n\x1a\n" {
                    return Err(ExecutionError::InvalidPng);
                }
            }
            let mut total = n as u64;
            if total > max_upload {
                return Err(ExecutionError::UploadLimit);
            }
            evidence.set_phase(Phase::Connect);
            if connection.is_none() {
                let socket: Box<dyn Socket> = match address {
                    Address::Unix(path) => Box::new(
                        tokio::net::UnixStream::connect(path)
                            .await
                            .map_err(|source| io_error(address, Phase::Connect, source))?,
                    ),
                    Address::Tcp(addr) => Box::new(
                        tokio::net::TcpStream::connect(addr)
                            .await
                            .map_err(|source| io_error(address, Phase::Connect, source))?,
                    ),
                };
                let id = NEXT_CONNECTION.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                assert!(id != u64::MAX, "connection token exhausted");
                connection = Some(Connection {
                    socket,
                    decoder: kitty_rc_frame::FrameDecoder::with_limits(
                        max_response,
                        max_response,
                        2,
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
                    Err(e) => return Err(io_error(address, Phase::IdleCheck, e)),
                }
                conn.decoder.reconfigure(max_response, max_response, 2)?;
            }
            if n > 0 {
                write_frame(
                    conn,
                    &prepared,
                    &raw[..n],
                    true,
                    &self.endpoint,
                    evidence.clone(),
                    deadline,
                )
                .await?;
                evidence.set_phase(Phase::Handshake);
                let ack = read_frame_with_surplus(conn, address, true).await?;
                match kind(&ack) {
                    Some(ResponseHead::StreamAcknowledged) => evidence.acknowledge_stream(),
                    Some(ResponseHead::Failure) => return Ok(Outcome::Reply(ack)),
                    _ => {
                        return Err(ExecutionError::UploadResponse {
                            phase: Phase::Handshake,
                            native: ack,
                        });
                    }
                }
                // Demand is stopped until handshake. No source read races the
                // first ack. Subsequent source reads/writes observe early replies.
                loop {
                    evidence.set_phase(Phase::Source);
                    let next = tokio::select! {
                        biased;
                        reply = read_frame(conn,address) => return early(reply?),
                        n = source.reader.read(&mut raw) => n.map_err(ExecutionError::Source)?,
                    };
                    n = next;
                    if n == 0 {
                        break;
                    }
                    total = total
                        .checked_add(n as u64)
                        .ok_or(ExecutionError::UploadLimit)?;
                    if total > max_upload {
                        return Err(ExecutionError::UploadLimit);
                    }
                    if let Some(reply) = write_frame(
                        conn,
                        &prepared,
                        &raw[..n],
                        false,
                        &self.endpoint,
                        evidence.clone(),
                        deadline,
                    )
                    .await?
                    {
                        return early(reply);
                    }
                }
            }
            // Producer errors above deliberately never execute truncated stdin.
            if let Some(reply) = write_frame(
                conn,
                &prepared,
                &[],
                false,
                &self.endpoint,
                evidence.clone(),
                deadline,
            )
            .await?
            {
                return early(reply);
            }
            evidence.complete_input();
            evidence.outbound_complete();
            if prepared.mode() == Mode::SubmissionOnly {
                return Ok(Outcome::Submitted);
            }
            evidence.set_phase(Phase::Read);
            let reply = read_frame(conn, address).await?;
            match kind(&reply) {
                Some(ResponseHead::Success | ResponseHead::Failure) => Ok(Outcome::Reply(reply)),
                _ => Err(ExecutionError::UploadResponse {
                    phase: Phase::Read,
                    native: reply,
                }),
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
                if prepared.reusable()
                    && evidence.input_complete()
                    && matches!(&outcome,Outcome::Reply(bytes) if matches!(kind(bytes),Some(ResponseHead::Success)))
                {
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
fn io_error(address: &Address, phase: Phase, source: std::io::Error) -> ExecutionError {
    ExecutionError::Io {
        endpoint: address.clone(),
        phase,
        source,
    }
}
fn kind(bytes: &[u8]) -> Option<ResponseHead> {
    response_head(bytes).ok()
}
fn early(bytes: Vec<u8>) -> Result<Outcome, ExecutionError> {
    if matches!(kind(&bytes), Some(ResponseHead::Failure)) {
        Ok(Outcome::Reply(bytes))
    } else {
        Err(ExecutionError::UploadResponse {
            phase: Phase::Upload,
            native: bytes,
        })
    }
}
async fn read_frame(conn: &mut Connection, address: &Address) -> Result<Vec<u8>, ExecutionError> {
    read_frame_with_surplus(conn, address, false).await
}
async fn read_frame_with_surplus(
    conn: &mut Connection,
    address: &Address,
    allow_following: bool,
) -> Result<Vec<u8>, ExecutionError> {
    // Persistent decoder belongs to connection, so canceling this future retains
    // partial inbound data. The transport itself retires if exchange is dropped.
    let mut bytes = [0; 8192];
    loop {
        if let Some(reply) = conn.decoder.next_frame()? {
            if !allow_following && !conn.decoder.is_idle() {
                return Err(ExecutionError::Surplus);
            }
            return Ok(reply);
        }
        let n = conn
            .socket
            .read(&mut bytes)
            .await
            .map_err(|e| io_error(address, Phase::Read, e))?;
        if n == 0 {
            conn.decoder.finish()?;
            return Err(ExecutionError::Eof);
        }
        conn.decoder.extend(&bytes[..n])?;
    }
}
async fn write_frame(
    conn: &mut Connection,
    prepared: &PreparedUpload,
    raw: &[u8],
    first: bool,
    endpoint: &Endpoint,
    evidence: Evidence,
    deadline: Instant,
) -> Result<Option<Vec<u8>>, ExecutionError> {
    let observe = !first && evidence.stream_acknowledged();
    evidence.set_phase(Phase::Prepare);
    let bytes = prepared
        .frame(raw, first, &endpoint.protection)?
        .protect(&endpoint.protection)?;
    if Instant::now() >= deadline {
        return Err(ExecutionError::Deadline {
            phase: Phase::Prepare,
        });
    }
    let phase = if raw.is_empty() {
        Phase::EndInput
    } else {
        Phase::Upload
    };
    evidence.set_phase(phase);
    let mut offset = 0;
    // Split read/write borrows: one owner, bounded inbound decoder and one frame.
    let (mut read, mut write) = tokio::io::split(&mut conn.socket);
    let decoder = &mut conn.decoder;
    let mut inbound = [0; 8192];
    loop {
        if observe && let Some(reply) = decoder.next_frame()? {
            if !decoder.is_idle() {
                return Err(ExecutionError::Surplus);
            }
            return Ok(Some(reply));
        }
        if offset == bytes.len() {
            return Ok(None);
        }
        tokio::select! {
            biased;
            result = read.read(&mut inbound), if observe => {
                let n = result.map_err(|e|io_error(&endpoint.address,Phase::Read,e))?;
                if n == 0 { decoder.finish()?; return Err(ExecutionError::Eof); }
                decoder.extend(&inbound[..n])?;
            },
            result = async { evidence.write_started(); write.write(&bytes[offset..]).await } => {
                let n = result.map_err(|e|io_error(&endpoint.address,phase,e))?;
                if n == 0 { return Err(io_error(&endpoint.address,phase,std::io::ErrorKind::WriteZero.into())); }
                offset += n;
            },
        }
    }
}
