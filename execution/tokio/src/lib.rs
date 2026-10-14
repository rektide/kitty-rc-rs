//! Exclusive finite socket driving. First slice retires every connection,
//! including successful exchanges: no uncorrelated late reply can be reused.
use kitty_rc_commands::{Command, Mode};
use kitty_rc_endpoint::{Address, Endpoint};
use kitty_rc_exchange::{Completion, Evidence, Failure, Outcome, Prepared, prepare};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    time::Instant,
};

trait Socket: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Socket for T {}

pub struct Session {
    endpoint: Endpoint,
}
impl Session {
    pub fn new(endpoint: Endpoint) -> Self {
        Self { endpoint }
    }
    /// Dropping this borrowed future drops its exclusively owned socket.
    pub async fn execute<C: Command>(
        &mut self,
        command: C,
        deadline: Instant,
        max_request: usize,
        max_response: usize,
    ) -> Result<Completion<C::Output>, Failure> {
        let timestamp = timestamp_ns().map_err(Failure::InvalidReply)?;
        let id = format!("direct-{timestamp}");
        let prepared = prepare(
            &command,
            &self.endpoint.protection,
            id,
            timestamp,
            max_request,
        )
        .map_err(Failure::InvalidReply)?;
        drive(
            &self.endpoint.address,
            prepared,
            deadline,
            max_response,
            Evidence::default(),
        )
        .await
        .decode::<C>()
    }
}
pub fn timestamp_ns() -> Result<u64, String> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos()
        .try_into()
        .map_err(|_| "timestamp overflow".into())
}

pub async fn drive(
    address: &Address,
    prepared: Prepared,
    deadline: Instant,
    max_response: usize,
    evidence: Evidence,
) -> Outcome {
    let work = async {
        if Instant::now() >= deadline {
            return Err("deadline elapsed before connect".into());
        }
        let mut stream: Box<dyn Socket> = match address {
            Address::Unix(path) => Box::new(
                tokio::net::UnixStream::connect(path)
                    .await
                    .map_err(|e| e.to_string())?,
            ),
            Address::Tcp(address) => Box::new(
                tokio::net::TcpStream::connect(address)
                    .await
                    .map_err(|e| e.to_string())?,
            ),
        };
        // Arm uncertainty before the first cancelable write poll.
        if Instant::now() >= deadline {
            return Err("deadline elapsed before write".into());
        }
        evidence.write_started();
        stream
            .write_all(&prepared.bytes)
            .await
            .map_err(|e| e.to_string())?;
        evidence.outbound_complete();
        if prepared.mode == Mode::SubmissionOnly {
            return Ok(Outcome::Submitted);
        }
        let mut decoder = kitty_rc_frame::FrameDecoder::with_limits(max_response, max_response, 1);
        let mut buf = [0u8; 8192];
        loop {
            let n = stream.read(&mut buf).await.map_err(|e| e.to_string())?;
            if n == 0 {
                decoder.finish().map_err(|e| e.to_string())?;
                return Err("EOF before terminal reply".into());
            }
            decoder.extend(&buf[..n]).map_err(|e| e.to_string())?;
            if let Some(payload) = decoder.next_frame().map_err(|e| e.to_string())? {
                if !decoder.is_idle() {
                    return Err("unexplained surplus after terminal reply".into());
                }
                return Ok(Outcome::Reply(payload));
            }
        }
    };
    let result: Result<Outcome, String> = match tokio::time::timeout_at(deadline, work).await {
        Ok(r) => r,
        Err(_) => Err("deadline elapsed".into()),
    };
    result.unwrap_or_else(|reason| Outcome::Interrupted {
        progress: evidence.progress(),
        reason,
    })
}
