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

/// Maximum payload retained for diagnostics when a stream goes wrong.
const MAX_DIAGNOSTIC_BYTES: usize = 32;

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
    /// Buffered bytes not yet consumed into complete frames.
    buf: Vec<u8>,
    /// Byte index where the payload of the frame currently being scanned
    /// starts; `None` when idle (expecting a PREFIX).
    frame_start: Option<usize>,
    /// Complete payloads awaiting `next_frame`, in arrival order.
    ready: std::collections::VecDeque<Vec<u8>>,
    /// Number of buffered bytes already consumed by complete frames; drained
    /// from `buf` at the end of each scan.
    consumed: usize,
    poisoned: Option<FrameError>,
}

impl FrameDecoder {
    /// Create a decoder that accepts payloads of at most `max_payload` bytes.
    pub fn new(max_payload: usize) -> Self {
        Self {
            max_payload,
            buf: Vec::new(),
            frame_start: None,
            ready: std::collections::VecDeque::new(),
            consumed: 0,
            poisoned: None,
        }
    }

    /// Feed bytes from one transport read. Complete frames found in the new
    /// bytes become available immediately.
    pub fn extend(&mut self, bytes: &[u8]) -> Result<(), FrameError> {
        if let Some(error) = &self.poisoned {
            return Err(error.clone());
        }
        self.buf.extend_from_slice(bytes);
        self.scan()
    }

    /// Take the next complete frame's payload, in arrival order. `Ok(None)`
    /// means more bytes are needed.
    pub fn next_frame(&mut self) -> Result<Option<Vec<u8>>, FrameError> {
        if let Some(error) = &self.poisoned {
            return Err(error.clone());
        }
        Ok(self.ready.pop_front())
    }

    /// Confirm the stream ended cleanly: no partial frame or partial prefix
    /// may remain buffered.
    pub fn finish(&mut self) -> Result<(), FrameError> {
        if let Some(error) = &self.poisoned {
            return Err(error.clone());
        }
        if self.buf.is_empty() && self.frame_start.is_none() {
            Ok(())
        } else {
            Err(FrameError::Truncated {
                pending: self.buf.len(),
            })
        }
    }

    /// True when no partial frame is buffered (all received bytes have been
    /// consumed into complete frames).
    pub fn is_idle(&self) -> bool {
        self.frame_start.is_none() && self.buf.is_empty()
    }

    fn scan(&mut self) -> Result<(), FrameError> {
        loop {
            if let Some(start) = self.frame_start {
                // Inside a frame: search for the terminator.
                let hay = &self.buf[start..];
                match find_subslice(hay, SUFFIX) {
                    Some(index) => {
                        if index > self.max_payload {
                            return self.poison(FrameError::PayloadTooLarge {
                                limit: self.max_payload,
                                actual: index,
                            });
                        }
                        let payload = self.buf[start..start + index].to_vec();
                        self.ready.push_back(payload);
                        self.consumed = start + index + SUFFIX.len();
                        self.frame_start = None;
                    }
                    None => {
                        let pending = self.buf.len() - start;
                        if pending > self.max_payload {
                            return self.poison(FrameError::PayloadTooLarge {
                                limit: self.max_payload,
                                actual: pending,
                            });
                        }
                        break;
                    }
                }
            } else {
                // Idle: a frame prefix must start here.
                let remaining = &self.buf[self.consumed..];
                if remaining.is_empty() {
                    break;
                }
                if remaining.starts_with(PREFIX) {
                    self.frame_start = Some(self.consumed + PREFIX.len());
                } else {
                    let matched = common_prefix_len(PREFIX, remaining);
                    if matched == remaining.len() {
                        // A proper prefix of PREFIX; wait for more bytes.
                        break;
                    }
                    let found: Vec<u8> = remaining
                        [matched..(matched + MAX_DIAGNOSTIC_BYTES).min(remaining.len())]
                        .to_vec();
                    return self.poison(FrameError::UnexpectedBytes { found });
                }
            }
        }

        // Drop bytes already folded into complete frames, keeping later
        // indices consistent.
        if self.consumed > 0 {
            self.buf.drain(..self.consumed);
            if let Some(start) = self.frame_start.as_mut() {
                *start -= self.consumed;
            }
            self.consumed = 0;
        }
        Ok(())
    }

    fn poison(&mut self, error: FrameError) -> Result<(), FrameError> {
        self.poisoned = Some(error.clone());
        Err(error)
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn common_prefix_len(expected: &[u8], actual: &[u8]) -> usize {
    expected
        .iter()
        .zip(actual.iter())
        .take_while(|(a, b)| a == b)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

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
            FrameError::PayloadTooLarge { limit: 8, actual } if actual >= 100
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
