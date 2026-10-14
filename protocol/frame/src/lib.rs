//! Incremental DCS framing for the kitty remote-control protocol.
//!
//! Every RC message on the wire — requests and responses alike — is a DCS
//! sequence: [`PREFIX`] `payload` [`SUFFIX`], where the payload is a UTF-8
//! JSON document. Because JSON serialization escapes control characters, the
//! payload itself can never contain the ESC byte, so the first SUFFIX after a
//! PREFIX always closes the frame.
//!
//! [`FrameDecoder`] is the persistent owner of framing state for one byte
//! stream:
//!
//! - frames split across any number of reads are reassembled;
//! - frames coalesced into one read are handed out in order, and surplus bytes
//!   after a complete frame are retained for the next call rather than
//!   discarded;
//! - payload size is bounded as bytes arrive, so a stream that never closes a
//!   frame cannot grow the buffer without limit;
//! - any unexpected byte outside a frame poisons the decoder: a dedicated RC
//!   byte stream has no defined junk, and ambiguity must retire the
//!   connection rather than become the next exchange's response.
//!
//! The decoder is byte-level by design; UTF-8 and JSON decoding belong to
//! `kitty-rc-schema`.

use thiserror::Error;

/// DCS introducer used by the RC protocol: `ESC P @kitty-cmd`.
pub const PREFIX: &[u8] = b"\x1bP@kitty-cmd";
/// String terminator that closes every RC frame: `ESC \`.
pub const SUFFIX: &[u8] = b"\x1b\\";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FrameError {
    /// A single frame's payload exceeded the configured limit. Reported as
    /// bytes arrive, before unbounded allocation.
    #[error("frame payload exceeds limit of {limit} bytes (at least {actual} bytes)")]
    PayloadTooLarge { limit: usize, actual: usize },

    /// Bytes that are not part of any frame appeared where a frame prefix was
    /// required. The stream is ambiguous; the decoder is poisoned.
    #[error("unexpected {found:?} where a frame prefix was required")]
    UnexpectedBytes { found: Vec<u8> },

    /// The stream ended with an incomplete frame (or a partial prefix) still
    /// buffered.
    #[error("stream ended mid-frame with {pending} buffered bytes")]
    Truncated { pending: usize },

    #[error("decoder retention budget exhausted")]
    RetentionFull,
}

/// Encode one complete frame around `payload`.
pub fn frame(payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(PREFIX.len() + payload.len() + SUFFIX.len());
    bytes.extend_from_slice(PREFIX);
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(SUFFIX);
    bytes
}

/// Incremental decoder for a stream of RC frames.
///
/// Feed transport reads with [`extend`](Self::extend) as they complete, take
/// complete frames out with [`next_frame`](Self::next_frame) in arrival order,
/// and confirm a clean end of stream with [`finish`](Self::finish).
///
/// Once any method returns an error, the decoder is poisoned: every later call
/// returns the same error, because the byte stream's framing can no longer be
/// trusted. The connection owning it must be retired.
#[derive(Debug)]
pub struct FrameDecoder {
    max_payload: usize,
    buf: Vec<u8>,
    prefix: usize,
    in_frame: bool,
    pending_escape: bool,
    ready: std::collections::VecDeque<Vec<u8>>,
    retained: usize,
    max_retained: usize,
    max_frames: usize,
    poisoned: Option<FrameError>,
}

impl FrameDecoder {
    /// Create a decoder that accepts payloads of at most `max_payload` bytes.
    pub fn new(max_payload: usize) -> Self {
        Self::with_limits(max_payload, max_payload.saturating_mul(2), 2)
    }

    /// Bound queued payload bytes and frame count as well as each payload.
    /// Exhaustion poisons the stream; drain frames between transport reads.
    pub fn with_limits(max_payload: usize, max_retained: usize, max_frames: usize) -> Self {
        Self {
            max_payload,
            buf: Vec::new(),
            prefix: 0,
            in_frame: false,
            pending_escape: false,
            ready: std::collections::VecDeque::new(),
            retained: 0,
            max_retained,
            max_frames,
            poisoned: None,
        }
    }

    /// Feed bytes from one transport read. Complete frames found in the new
    /// bytes become available immediately.
    pub fn extend(&mut self, bytes: &[u8]) -> Result<(), FrameError> {
        if let Some(error) = &self.poisoned {
            return Err(error.clone());
        }
        for &byte in bytes {
            if !self.in_frame {
                if byte != PREFIX[self.prefix] {
                    return self.poison(FrameError::UnexpectedBytes { found: vec![byte] });
                }
                self.prefix += 1;
                if self.prefix == PREFIX.len() {
                    self.prefix = 0;
                    self.in_frame = true;
                }
            } else if self.pending_escape {
                if byte != b'\\' {
                    return self.poison(FrameError::UnexpectedBytes { found: vec![0x1b, byte] });
                }
                if self.ready.len() == self.max_frames {
                    return self.poison(FrameError::RetentionFull);
                }
                self.ready.push_back(std::mem::take(&mut self.buf));
                self.in_frame = false;
                self.pending_escape = false;
            } else if byte == 0x1b {
                self.pending_escape = true;
            } else {
                if self.buf.len() == self.max_payload {
                    return self.poison(FrameError::PayloadTooLarge { limit: self.max_payload, actual: self.buf.len() + 1 });
                }
                if self.retained == self.max_retained {
                    return self.poison(FrameError::RetentionFull);
                }
                self.buf.push(byte);
                self.retained += 1;
            }
        }
        Ok(())
    }

    /// Take the next complete frame's payload, in arrival order. `Ok(None)`
    /// means more bytes are needed.
    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>, FrameError> {
        if let Some(error) = &self.poisoned {
            return Err(error.clone());
        }
        let result = self.ready.pop_front();
        if let Some(payload) = &result { self.retained -= payload.len(); }
        Ok(result)
    }

    /// Confirm the stream ended cleanly: no partial frame or partial prefix
    /// may remain buffered.
    pub fn finish(&mut self) -> Result<(), FrameError> {
        if let Some(error) = &self.poisoned {
            return Err(error.clone());
        }
        if !self.in_frame && self.prefix == 0 {
            Ok(())
        } else {
            self.poison(FrameError::Truncated {
                pending: self.buf.len() + self.prefix + usize::from(self.pending_escape),
            })
        }
    }

    /// True when no partial frame is buffered (all received bytes have been
    /// consumed into complete frames).
    pub fn is_idle(&self) -> bool {
        !self.in_frame && self.prefix == 0 && self.ready.is_empty() && self.poisoned.is_none()
    }

    fn poison(&mut self, error: FrameError) -> Result<(), FrameError> {
        self.poisoned = Some(error.clone());
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_limit_split_suffix_and_poisoned_eof() {
        let mut d = FrameDecoder::new(3);
        d.extend(PREFIX).unwrap();
        d.extend(b"abc\x1b").unwrap();
        d.extend(b"\\").unwrap();
        assert!(!d.is_idle());
        assert_eq!(d.next_frame().unwrap().unwrap(), b"abc");
        assert!(d.is_idle());
        d.extend(PREFIX).unwrap();
        let error = d.finish().unwrap_err();
        assert_eq!(d.extend(b"x").unwrap_err(), error);
        assert_eq!(d.next_frame().unwrap_err(), error);
    }

    #[test]
    fn huge_read_and_empty_frame_flood_are_bounded() {
        let mut d = FrameDecoder::with_limits(8, 8, 1);
        d.extend(&frame(b"12345678")).unwrap();
        assert_eq!(d.extend(&frame(b"x")), Err(FrameError::RetentionFull));
        assert_eq!(d.retained, 8);
        let mut d = FrameDecoder::with_limits(8, 8, 1);
        d.extend(&frame(b"")).unwrap();
        assert_eq!(d.extend(&frame(b"")), Err(FrameError::RetentionFull));
    }

    const PAYLOAD_A: &[u8] = br#"{"cmd":"ls","version":[0,43,1]}"#;
    const PAYLOAD_B: &[u8] = b"{\"ok\":true,\"data\":\"windows\"}";

    fn decoder() -> FrameDecoder {
        FrameDecoder::new(1024)
    }

    #[test]
    fn encodes_documented_frame_shape() {
        let bytes = frame(PAYLOAD_A);
        assert!(bytes.starts_with(PREFIX));
        assert!(bytes.ends_with(SUFFIX));
        assert_eq!(&bytes[PREFIX.len()..bytes.len() - SUFFIX.len()], PAYLOAD_A);
    }

    #[test]
    fn single_frame_single_read() {
        let mut decoder = decoder();
        decoder.extend(&frame(PAYLOAD_A)).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), Some(PAYLOAD_A.to_vec()));
        assert_eq!(decoder.next_frame().unwrap(), None);
        decoder.finish().unwrap();
    }

    /// Every possible split point must reassemble to the same payload.
    #[test]
    fn every_split_point() {
        let bytes = frame(PAYLOAD_A);
        for split in 0..bytes.len() {
            let mut decoder = decoder();
            decoder.extend(&bytes[..split]).unwrap();
            assert_eq!(decoder.next_frame().unwrap(), None, "split {split}");
            decoder.extend(&bytes[split..]).unwrap();
            assert_eq!(
                decoder.next_frame().unwrap(),
                Some(PAYLOAD_A.to_vec()),
                "split {split}"
            );
            decoder.finish().unwrap();
        }
    }

    /// Two frames coalesced into one read come out in order, exactly once.
    #[test]
    fn coalesced_frames_in_order() {
        let mut decoder = decoder();
        let mut bytes = frame(PAYLOAD_A);
        bytes.extend_from_slice(&frame(PAYLOAD_B));
        decoder.extend(&bytes).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), Some(PAYLOAD_A.to_vec()));
        assert_eq!(decoder.next_frame().unwrap(), Some(PAYLOAD_B.to_vec()));
        assert_eq!(decoder.next_frame().unwrap(), None);
        decoder.finish().unwrap();
    }

    /// A complete frame followed by a partial one: the complete frame is
    /// delivered, the partial one is neither lost nor fabricated, and EOF
    /// reports truncation.
    #[test]
    fn complete_then_partial_reports_truncation() {
        let mut decoder = decoder();
        let mut bytes = frame(PAYLOAD_A);
        bytes.extend_from_slice(&frame(PAYLOAD_B)[..7]);
        decoder.extend(&bytes).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), Some(PAYLOAD_A.to_vec()));
        assert_eq!(decoder.next_frame().unwrap(), None);
        assert!(matches!(
            decoder.finish(),
            Err(FrameError::Truncated { .. })
        ));
    }

    /// A partial prefix (just the ESC byte) waits for more bytes instead of
    /// being treated as junk or as a terminator fragment.
    #[test]
    fn partial_prefix_waits() {
        let mut decoder = decoder();
        decoder.extend(&[0x1b]).unwrap();
        // The lone ESC is a proper prefix of PREFIX: buffered, not junk, and
        // no frame can be produced yet.
        assert!(!decoder.is_idle());
        assert_eq!(decoder.next_frame().unwrap(), None);

        let mut rest = b"P@kitty-cmd".to_vec();
        rest.extend_from_slice(br#"{"ok":true}"#);
        rest.extend_from_slice(&[0x1b, b'\\']);
        decoder.extend(&rest).unwrap();
        assert_eq!(
            decoder.next_frame().unwrap(),
            Some(br#"{"ok":true}"#.to_vec())
        );
        decoder.finish().unwrap();
    }

    #[test]
    fn junk_between_frames_poisons() {
        let mut decoder = decoder();
        decoder.extend(&frame(PAYLOAD_A)).unwrap();
        let error = decoder.extend(b"junk").unwrap_err();
        assert!(matches!(error, FrameError::UnexpectedBytes { .. }));
        // Poisoned: later calls keep failing, frames are not fabricated.
        assert!(decoder.next_frame().is_err());
        assert!(decoder.extend(&frame(PAYLOAD_B)).is_err());
        assert!(decoder.finish().is_err());
    }

    /// An oversized frame is rejected as bytes arrive, without needing the
    /// terminator, and the decoder stays bounded.
    #[test]
    fn oversized_frame_rejected_before_termination() {
        let mut decoder = FrameDecoder::new(8);
        let mut bytes = PREFIX.to_vec();
        bytes.extend_from_slice(&[b'x'; 100]);
        let error = decoder.extend(&bytes).unwrap_err();
        assert!(matches!(
            error,
            FrameError::PayloadTooLarge { limit: 8, actual: 9 }
        ));
        assert!(decoder.next_frame().is_err());
    }

    #[test]
    fn empty_payload_frame_is_valid() {
        let mut decoder = decoder();
        decoder.extend(&frame(b"")).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), Some(Vec::new()));
        decoder.finish().unwrap();
    }

    /// Payloads may contain any byte sequence except the ESC byte (JSON never
    /// emits it raw); binary-adjacent bytes pass through unchanged.
    #[test]
    fn high_bytes_pass_through() {
        let payload: &[u8] = &[0xff, 0xfe, b'{', b'}', 0x80];
        let mut decoder = decoder();
        decoder.extend(&frame(payload)).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), Some(payload.to_vec()));
    }

    /// Interleaved delivery pattern mirroring a real exchange: request write,
    /// response split over three reads, clean EOF.
    #[test]
    fn interleaved_reads_pattern() {
        let mut decoder = decoder();
        let response = frame(PAYLOAD_B);
        let third = response.len() / 3;
        decoder.extend(&response[..third]).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), None);
        decoder.extend(&response[third..2 * third]).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), None);
        decoder.extend(&response[2 * third..]).unwrap();
        assert_eq!(decoder.next_frame().unwrap(), Some(PAYLOAD_B.to_vec()));
        decoder.finish().unwrap();
    }
}
