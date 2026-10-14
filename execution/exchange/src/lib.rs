//! Preparation and evidence vocabulary independent of a transport/runtime.
use kitty_rc_commands::{Command, Mode};
use kitty_rc_endpoint::Protection;
use kitty_rc_schema::{ResponseEnvelope, ResponseKind};
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Progress {
    NotWritten,
    WriteStarted,
    OutboundComplete,
}
#[derive(Clone, Default)]
pub struct Evidence(Arc<AtomicU8>);
impl Evidence {
    pub fn progress(&self) -> Progress {
        match self.0.load(Ordering::SeqCst) {
            0 => Progress::NotWritten,
            1 => Progress::WriteStarted,
            _ => Progress::OutboundComplete,
        }
    }
    pub fn write_started(&self) {
        self.0.store(1, Ordering::SeqCst);
    }
    pub fn outbound_complete(&self) {
        self.0.store(2, Ordering::SeqCst);
    }
}

/// Native result retained independently of any typed waiter.
#[derive(Debug)]
pub enum Outcome {
    Reply(Vec<u8>),
    Submitted,
    Interrupted { progress: Progress, reason: String },
}
impl Outcome {
    pub fn unresolved(&self) -> bool {
        if let Self::Reply(bytes) = self {
            // Structural validation without allocating success data before
            // custody/shutdown. Typed data is interpreted by the sole taker.
            #[derive(serde::Deserialize)]
            struct Header {
                ok: bool,
                stream: Option<bool>,
                data: Option<serde::de::IgnoredAny>,
                error: Option<String>,
            }
            return match serde_json::from_slice::<Header>(bytes) {
                Ok(h) => {
                    let _ = (h.ok, h.data, h.error);
                    h.stream == Some(true)
                }
                Err(_) => true,
            };
        }
        matches!(
            self,
            Self::Interrupted {
                progress: Progress::WriteStarted | Progress::OutboundComplete,
                ..
            }
        )
    }
    pub fn retained_bytes(&self) -> usize {
        match self {
            Self::Reply(b) => b.len(),
            Self::Interrupted { reason, .. } => reason.len(),
            Self::Submitted => 0,
        }
    }
    pub fn decode<C: Command>(self) -> Result<Completion<C::Output>, Failure> {
        match self {
            Self::Submitted => Ok(Completion::Submitted),
            Self::Interrupted { progress, reason } => {
                Err(Failure::Interrupted { progress, reason })
            }
            Self::Reply(bytes) => {
                let envelope: ResponseEnvelope = match serde_json::from_slice(&bytes) {
                    Ok(envelope) => envelope,
                    Err(e) => {
                        return Err(Failure::MalformedReply {
                            error: e.to_string(),
                            native: bytes,
                        });
                    }
                };
                let kind = match envelope.into_kind() {
                    Ok(kind) => kind,
                    Err(e) => {
                        return Err(Failure::MalformedReply {
                            error: e.to_string(),
                            native: bytes,
                        });
                    }
                };
                match kind {
                    ResponseKind::Failure { error } => Err(Failure::Remote {
                        error: error.unwrap_or_default(),
                        native: bytes,
                    }),
                    ResponseKind::StreamAcknowledged => Err(Failure::MalformedReply {
                        error: "unexpected stream acknowledgment".into(),
                        native: bytes,
                    }),
                    ResponseKind::Success { data } => C::decode(data)
                        .map(Completion::Reply)
                        .map_err(|error| Failure::TypedDecode {
                            error,
                            native: bytes,
                        }),
                }
            }
        }
    }
}
#[derive(Debug)]
pub enum Completion<T> {
    Reply(T),
    Submitted,
}
#[derive(Debug)]
pub enum Failure {
    Remote { error: String, native: Vec<u8> },
    TypedDecode { error: String, native: Vec<u8> },
    MalformedReply { error: String, native: Vec<u8> },
    InvalidReply(String),
    Interrupted { progress: Progress, reason: String },
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Failure {}

/// Transport accepts only prepared bytes, never a command to encrypt again.
pub struct Prepared {
    pub bytes: Vec<u8>,
    pub mode: Mode,
}
pub fn prepare<C: Command>(
    command: &C,
    protection: &Protection,
    async_id: String,
    timestamp_ns: u64,
    max_request: usize,
) -> Result<Prepared, String> {
    let mut r = command.request();
    if r.stream.is_some() || r.stream_id.is_some() || r.r#async.is_some() || r.no_response.is_some()
    {
        return Err("finite command supplied executor-owned envelope flags".into());
    }
    let mode = command.mode();
    if mode == Mode::Deferred {
        r.r#async = Some(async_id);
    }
    if mode == Mode::SubmissionOnly {
        r.no_response = Some(true);
    }
    let plain = r.to_json_bytes().map_err(|e| e.to_string())?;
    if plain.len() > max_request {
        return Err("request byte limit exceeded".into());
    }
    let bytes = match protection {
        Protection::Plain => plain,
        Protection::Encrypted { key, password, .. } => serde_json::to_vec(
            &kitty_rc_crypto::encrypt_request(
                &r,
                password,
                timestamp_ns,
                key,
                &mut rand_core::OsRng,
            )
            .map_err(|_| "request protection failed")?,
        )
        .map_err(|e| e.to_string())?,
    };
    if bytes.len() > max_request {
        return Err("protected request byte limit exceeded".into());
    }
    Ok(Prepared {
        bytes: kitty_rc_frame::frame(&bytes),
        mode,
    })
}
