//! Preparation and evidence vocabulary independent of a transport/runtime.
mod response;
mod upload;
use kitty_rc_commands::{Command, Mode};
use kitty_rc_endpoint::Address;
use kitty_rc_endpoint::Protection;
use kitty_rc_schema::{ResponseEnvelope, ResponseKind};
pub use response::{ResponseHead, response_head};
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};
pub use upload::{PreparedUpload, prepare_upload};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Progress {
    NotWritten,
    WriteStarted,
    OutboundComplete,
}
#[derive(Clone, Default)]
pub struct Evidence {
    progress: Arc<AtomicU8>,
    phase: Arc<AtomicU8>,
    upload: Arc<AtomicU8>,
}
impl Evidence {
    pub fn stream_acknowledged(&self) -> bool {
        self.upload.load(Ordering::SeqCst) & 1 != 0
    }
    pub fn acknowledge_stream(&self) {
        self.upload.fetch_or(1, Ordering::SeqCst);
    }
    /// Locally written EOF marker, not proof of accumulator cleanup.
    pub fn input_complete(&self) -> bool {
        self.upload.load(Ordering::SeqCst) & 2 != 0
    }
    pub fn complete_input(&self) {
        self.upload.fetch_or(2, Ordering::SeqCst);
    }
    pub fn progress(&self) -> Progress {
        match self.progress.load(Ordering::SeqCst) {
            0 => Progress::NotWritten,
            1 => Progress::WriteStarted,
            _ => Progress::OutboundComplete,
        }
    }
    pub fn write_started(&self) {
        self.progress.store(1, Ordering::SeqCst);
    }
    pub fn outbound_complete(&self) {
        self.progress.store(2, Ordering::SeqCst);
    }
    pub fn set_phase(&self, phase: Phase) {
        self.phase.store(phase as u8, Ordering::SeqCst);
    }
    pub fn phase(&self) -> Phase {
        match self.phase.load(Ordering::SeqCst) {
            1 => Phase::Prepare,
            2 => Phase::Connect,
            3 => Phase::IdleCheck,
            4 => Phase::Write,
            5 => Phase::Read,
            6 => Phase::Source,
            7 => Phase::Handshake,
            8 => Phase::Upload,
            9 => Phase::EndInput,
            _ => Phase::Queue,
        }
    }
}

/// Native result retained independently of any typed waiter.
#[derive(Debug)]
pub enum Outcome {
    Reply(Vec<u8>),
    Submitted,
    Interrupted {
        progress: Progress,
        reason: ExecutionError,
    },
}
impl Outcome {
    pub fn unresolved(&self) -> bool {
        if let Self::Reply(bytes) = self {
            // Structural validation without allocating success data before
            // custody/shutdown. Typed data is interpreted by the sole taker.
            return match response_head(bytes) {
                Ok(head) => head == ResponseHead::StreamAcknowledged,
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
            Self::Interrupted {
                reason: ExecutionError::UploadResponse { native, .. },
                ..
            } => native.capacity().saturating_add(256),
            Self::Interrupted { .. } => 256,
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
                            error: kitty_rc_schema::SchemaError::Json(e),
                            native: bytes,
                        });
                    }
                };
                let kind = match envelope.into_kind() {
                    Ok(kind) => kind,
                    Err(e) => {
                        return Err(Failure::MalformedReply {
                            error: e,
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
                        error: kitty_rc_schema::SchemaError::InvalidEnvelope(
                            "unexpected stream acknowledgment".into(),
                        ),
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
    Remote {
        error: String,
        native: Vec<u8>,
    },
    TypedDecode {
        error: kitty_rc_commands::DecodeError,
        native: Vec<u8>,
    },
    MalformedReply {
        error: kitty_rc_schema::SchemaError,
        native: Vec<u8>,
    },
    Unavailable(Box<dyn std::error::Error + Send + Sync>),
    Prepare(PrepareError),
    Interrupted {
        progress: Progress,
        reason: ExecutionError,
    },
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text: &dyn std::fmt::Display = match self {
            Self::Remote { error, .. } => error,
            Self::TypedDecode { error, .. } => error,
            Self::MalformedReply { error, .. } => error,
            Self::Unavailable(error) => error,
            Self::Prepare(error) => error,
            Self::Interrupted { reason, .. } => reason,
        };
        struct Bounded<'a, 'b> {
            f: &'a mut std::fmt::Formatter<'b>,
            left: usize,
        }
        impl std::fmt::Write for Bounded<'_, '_> {
            fn write_str(&mut self, value: &str) -> std::fmt::Result {
                let mut end = value.len().min(self.left);
                while !value.is_char_boundary(end) {
                    end -= 1;
                }
                self.left -= end;
                self.f.write_str(&value[..end])
            }
        }
        let mut bounded = Bounded { f, left: 256 };
        let category = match self {
            Self::Remote { .. } => "remote error",
            Self::TypedDecode { .. } => "typed decode",
            Self::MalformedReply { .. } => "malformed reply",
            Self::Unavailable(_) => "custody unavailable",
            Self::Prepare(_) => "prepare",
            Self::Interrupted { progress, .. } => {
                return std::fmt::write(
                    &mut bounded,
                    format_args!("interrupted ({progress:?}): {text}"),
                );
            }
        };
        std::fmt::write(&mut bounded, format_args!("{category}: {text}"))
    }
}
impl std::error::Error for Failure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Prepare(e) => Some(e),
            Self::Unavailable(e) => Some(e.as_ref()),
            Self::Interrupted { reason, .. } => Some(reason),
            Self::TypedDecode { error, .. } => Some(error),
            Self::MalformedReply { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    Queue = 0,
    Prepare = 1,
    Connect = 2,
    IdleCheck = 3,
    Write = 4,
    Read = 5,
    Source = 6,
    Handshake = 7,
    Upload = 8,
    EndInput = 9,
}
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("upload source failed: {0}")]
    Source(#[source] std::io::Error),
    #[error("upload exceeds configured total byte limit")]
    UploadLimit,
    #[error("input does not begin with a PNG signature")]
    InvalidPng,
    #[error("invalid upload response during {phase:?}")]
    UploadResponse { phase: Phase, native: Vec<u8> },
    #[error("{phase:?} I/O at {endpoint:?}: {source}")]
    Io {
        endpoint: Address,
        phase: Phase,
        #[source]
        source: std::io::Error,
    },
    #[error("deadline elapsed during {phase:?}")]
    Deadline { phase: Phase },
    #[error("explicit operation cancellation during {phase:?}")]
    Cancelled { phase: Phase },
    #[error("driver stopped during {phase:?}; no replay")]
    DriverStopped { phase: Phase },
    #[error("framing error: {0}")]
    Framing(#[from] kitty_rc_frame::FrameError),
    #[error("unexplained surplus after terminal reply")]
    Surplus,
    #[error("EOF before terminal reply")]
    Eof,
    #[error("idle peer closed before write")]
    IdleClosed,
    #[error("preparation failed: {0}")]
    Prepare(#[from] PrepareError),
}
#[derive(Debug, thiserror::Error)]
pub enum PrepareError {
    #[error("upload requires the dedicated bounded-source entrance")]
    UploadEntrance,
    #[error("invalid upload metadata or response mode")]
    UploadProfile,
    #[error("finite command supplied executor-owned envelope flags")]
    Flags,
    #[error("request or protected expansion exceeds byte limit")]
    Limit,
    #[error("JSON preparation failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("schema preparation failed: {0}")]
    Schema(#[from] kitty_rc_schema::SchemaError),
    #[error("request protection failed: {0}")]
    Crypto(#[from] kitty_rc_crypto::CryptoError),
    #[error("clock before Unix epoch: {0}")]
    Clock(#[from] std::time::SystemTimeError),
    #[error("timestamp overflow")]
    TimestampOverflow,
}

/// Frozen plaintext for a consuming first-send protection boundary. Already
/// protected envelopes cannot be constructed as Prepared values.
pub struct Prepared {
    /// Immutable plaintext; no password/timestamp injected until first send.
    bytes: Vec<u8>,
    mode: Mode,
    encoded_bound: usize,
    reusable: bool,
    scratch_factor: usize,
}
pub fn prepare<C: Command>(
    command: &C,
    protection: &Protection,
    max_request: usize,
) -> Result<Prepared, PrepareError> {
    if command.is_upload() {
        return Err(PrepareError::UploadEntrance);
    }
    let mut r = command.request();
    // A raw/custom finite constructor cannot disguise an accumulator upload,
    // especially run's empty-input path, as an ordinary one-frame command.
    if r.cmd == "run"
        || (matches!(r.cmd.as_str(), "set-background-image" | "set-window-logo")
            && !r
                .payload
                .as_ref()
                .and_then(|p| p.get("data"))
                .and_then(serde_json::Value::as_str)
                .is_some_and(|data| {
                    data == "-" || (r.cmd == "set-background-image" && data.starts_with("index:"))
                }))
    {
        return Err(PrepareError::UploadEntrance);
    }
    if r.stream.is_some() || r.stream_id.is_some() || r.r#async.is_some() || r.no_response.is_some()
    {
        return Err(PrepareError::Flags);
    }
    let mode = command.mode();
    if mode == Mode::Deferred {
        r.r#async = Some(remote_request_id());
    }
    if mode == Mode::SubmissionOnly {
        r.no_response = Some(true);
    }
    prepare_request(r, mode, command.reusable(), protection, max_request)
}

pub(crate) fn prepare_request(
    r: kitty_rc_schema::PlainRequest,
    mode: Mode,
    reusable: bool,
    protection: &Protection,
    max_request: usize,
) -> Result<Prepared, PrepareError> {
    // A bounded writer rejects oversized input without first building an
    // oversized serialized copy. Caller-supplied Command::request is synchronous.
    struct Limited {
        bytes: Vec<u8>,
        limit: usize,
        full: bool,
    }
    impl std::io::Write for Limited {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
                self.full = true;
                return Err(std::io::Error::other("request limit"));
            }
            if self.bytes.len() + bytes.len() > self.bytes.capacity() {
                let desired = (self.bytes.capacity().max(64).saturating_mul(2))
                    .max(self.bytes.len() + bytes.len())
                    .min(self.limit);
                self.bytes.reserve_exact(desired - self.bytes.len());
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Limited {
        bytes: Vec::new(),
        limit: max_request,
        full: false,
    };
    if let Err(error) = serde_json::to_writer(&mut writer, &r) {
        return Err(if writer.full {
            PrepareError::Limit
        } else {
            PrepareError::Json(error)
        });
    }
    let plain = writer.bytes;
    // JSON escaping is at most six bytes per password byte. Base85 expansion
    // plus fixed nonce/key/tag/envelope fields is reserved before acceptance.
    let encoded_bound = match protection {
        Protection::Plain => plain.len(),
        Protection::Encrypted { password, .. } => plain
            .len()
            .saturating_add(password.len().saturating_mul(6))
            .saturating_add(128)
            .div_ceil(4)
            .saturating_mul(5)
            .saturating_add(256),
    };
    if encoded_bound > max_request {
        return Err(PrepareError::Limit);
    }
    Ok(Prepared {
        bytes: plain,
        mode,
        reusable,
        scratch_factor: if matches!(protection, Protection::Plain) {
            3
        } else {
            8
        },
        encoded_bound: encoded_bound
            .saturating_add(kitty_rc_frame::PREFIX.len() + kitty_rc_frame::SUFFIX.len()),
    })
}

impl Prepared {
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn reusable(&self) -> bool {
        self.reusable
    }
    pub fn plaintext(&self) -> &[u8] {
        &self.bytes
    }
    pub fn reservation_bytes(&self) -> usize {
        self.bytes
            .capacity()
            .saturating_add(self.encoded_bound.saturating_mul(self.scratch_factor))
    }
    /// One first-send protection boundary, never a retry/re-encryption path.
    pub fn protect(self, protection: &Protection) -> Result<Vec<u8>, PrepareError> {
        let payload = match protection {
            Protection::Plain => self.bytes,
            Protection::Encrypted { key, password, .. } => {
                let request = serde_json::from_slice(&self.bytes)?;
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_nanos()
                    .try_into()
                    .map_err(|_| PrepareError::TimestampOverflow)?;
                serde_json::to_vec(&kitty_rc_crypto::encrypt_request(
                    &request,
                    password,
                    timestamp,
                    key,
                    &mut rand_core::OsRng,
                )?)?
            }
        };
        let framed = kitty_rc_frame::frame(&payload);
        if framed.len() > self.encoded_bound {
            return Err(PrepareError::Limit);
        }
        Ok(framed)
    }
}

/// Server-global async identity, independent of process-local operation IDs.
/// Kitty evicts the oldest registration after 32 concurrent async requests;
/// randomness avoids collisions, not eviction caused by other clients.
pub fn remote_request_id() -> String {
    use rand_core::RngCore;
    let mut bytes = [0u8; 32];
    rand_core::OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn independent_client_id_sequences_do_not_restart() {
        let first: Vec<_> = (0..64).map(|_| super::remote_request_id()).collect();
        let second: Vec<_> = (0..64).map(|_| super::remote_request_id()).collect();
        assert!(
            first
                .iter()
                .all(|id| id.len() == 64 && !second.contains(id))
        );
    }

    #[test]
    fn plaintext_freeze_reserves_protection_expansion_without_sealing() {
        let mut point = [0; 32];
        point[0] = 9;
        let key = kitty_rc_crypto::ServerPublicKey::parse(&format!("1:{}", base85::encode(&point)))
            .unwrap();
        let protection = kitty_rc_endpoint::Protection::Encrypted {
            key,
            password: "\0\u{0001}\"\\🦀".into(),
            key_origin: "test".into(),
        };
        let prepared =
            super::prepare(&kitty_rc_commands::Ls::default(), &protection, 4096).unwrap();
        let frozen: serde_json::Value = serde_json::from_slice(prepared.plaintext()).unwrap();
        assert!(frozen.get("password").is_none());
        assert!(frozen.get("timestamp").is_none());
        assert!(frozen.get("encrypted").is_none());
        let bound = prepared.encoded_bound;
        assert!(prepared.reservation_bytes() >= bound);
        let framed = prepared.protect(&protection).unwrap();
        assert!(framed.len() <= bound);
        let envelope: serde_json::Value = serde_json::from_slice(
            &framed[kitty_rc_frame::PREFIX.len()..framed.len() - kitty_rc_frame::SUFFIX.len()],
        )
        .unwrap();
        assert!(envelope.get("encrypted").is_some());
        assert!(envelope.get("cmd").is_none());
    }

    #[test]
    fn custom_decoder_keeps_trait_object_cause_and_native_evidence() {
        struct Custom;
        impl kitty_rc_commands::Command for Custom {
            type Output = Box<u8>;
            fn request(&self) -> kitty_rc_schema::PlainRequest {
                kitty_rc_schema::PlainRequest::new("custom")
            }
            fn class(&self) -> kitty_rc_commands::Class {
                kitty_rc_commands::Class::Short
            }
            fn decode(
                _: Option<serde_json::Value>,
            ) -> Result<Self::Output, kitty_rc_commands::DecodeError> {
                Err(kitty_rc_commands::DecodeError::Custom(Box::new(
                    std::io::Error::from(std::io::ErrorKind::PermissionDenied),
                )))
            }
        }
        let failure = super::Outcome::Reply(b"{\"ok\":true}".to_vec())
            .decode::<Custom>()
            .unwrap_err();
        let cause = std::error::Error::source(&failure)
            .unwrap()
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap();
        assert_eq!(cause.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(
            matches!(failure, super::Failure::TypedDecode { native, .. } if native == b"{\"ok\":true}")
        );
    }

    #[test]
    fn default_diagnostics_are_bounded_without_truncating_native_evidence() {
        let error = "🦀".repeat(1024);
        let failure = super::Failure::Remote {
            error: error.clone(),
            native: error.as_bytes().to_vec(),
        };
        let display = failure.to_string();
        assert!(display.len() <= 256);
        assert!(display.starts_with("remote error:"));
        assert!(
            matches!(failure, super::Failure::Remote { native, error } if native.len() == 4096 && error.len() == 4096)
        );
    }
}
