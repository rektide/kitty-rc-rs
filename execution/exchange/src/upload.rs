//! Bounded immutable upload preparation, independent of sources and runtime.
use crate::{PrepareError, Prepared, prepare_request, remote_request_id};
use base64::{Engine, engine::general_purpose::STANDARD};
use kitty_rc_commands::{
    Mode,
    upload::{UploadCommand, UploadKey},
};
use kitty_rc_endpoint::Protection;
use kitty_rc_schema::PlainRequest;

/// Frozen metadata and stable random conversation IDs. No input body is buffered.
pub struct PreparedUpload {
    metadata: Vec<u8>,
    key: UploadKey,
    mode: Mode,
    reusable: bool,
    max_request: usize,
    frame_reservation: usize,
}
pub fn prepare_upload<C: UploadCommand>(
    command: &C,
    protection: &Protection,
    max_request: usize,
) -> Result<PreparedUpload, PrepareError> {
    let mut r = command.request();
    let key = command.upload_key();
    let mode = command.mode();
    if r.cmd != key.command()
        || r.stream.is_some()
        || r.stream_id.is_some()
        || r.r#async.is_some()
        || r.no_response.is_some()
        || !r
            .payload
            .as_ref()
            .is_some_and(|p| p.is_object() && p.get("data").is_none())
        || (key == UploadKey::Run
            && (mode != Mode::Deferred
                || !r.payload.as_ref().unwrap()["cmdline"]
                    .as_array()
                    .is_some_and(|a| !a.is_empty())))
    {
        return Err(PrepareError::UploadProfile);
    }
    r.stream_id = Some(remote_request_id());
    if mode == Mode::Deferred {
        r.r#async = Some(remote_request_id());
    }
    if mode == Mode::SubmissionOnly {
        r.no_response = Some(true);
    }
    // Bound static metadata first; no source polling or full-body preparation.
    let frozen = prepare_request(r.clone(), mode, false, protection, max_request)?;
    let metadata = frozen.plaintext().to_vec();
    r.stream = Some(true);
    r.payload.as_mut().unwrap()["data"] =
        serde_json::Value::String("A".repeat(key.raw_chunk_bytes().div_ceil(3) * 4));
    let probe = prepare_request(r, mode, false, protection, max_request)?;
    Ok(PreparedUpload {
        metadata,
        key,
        mode,
        reusable: command.reusable(),
        max_request,
        frame_reservation: probe.reservation_bytes(),
    })
}
impl PreparedUpload {
    pub fn key(&self) -> UploadKey {
        self.key
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn reusable(&self) -> bool {
        self.reusable
    }
    pub fn metadata(&self) -> &[u8] {
        &self.metadata
    }
    /// Conservative metadata tree/JSON copies + raw chunk + one protected frame
    /// and fixed executor future/read-buffer storage. Empty string arrays cost
    /// much more in serde's Value tree than their compact serialized form.
    /// Source-owned allocations are additional caller-declared reservations.
    pub fn reservation_bytes(&self) -> usize {
        self.metadata
            .capacity()
            .saturating_mul(32)
            .saturating_add(self.frame_reservation)
            .saturating_add(self.key.raw_chunk_bytes())
            .saturating_add(64 * 1024)
    }
    pub fn frame(
        &self,
        raw: &[u8],
        first: bool,
        protection: &Protection,
    ) -> Result<Prepared, PrepareError> {
        if raw.len() > self.key.raw_chunk_bytes() || (first && raw.is_empty()) {
            return Err(PrepareError::UploadProfile);
        }
        let mut r: PlainRequest = serde_json::from_slice(&self.metadata)?;
        r.stream = if first { Some(true) } else { None };
        r.payload.as_mut().unwrap()["data"] = serde_json::Value::String(STANDARD.encode(raw));
        prepare_request(r, self.mode, false, protection, self.max_request)
    }
}
