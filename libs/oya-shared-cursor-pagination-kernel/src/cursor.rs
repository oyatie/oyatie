//! Base64-URL cursor encode/decode and scope_hash helpers (std-only).
//!
//! Cursor payload format: `{offset_u64}:{scope_hash_u64}` encoded as
//! UTF-8 bytes then base64-URL (RFC 4648 §5, no padding).

use crate::PaginationError;

// RFC 4648 §5 base64-URL alphabet.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Encode raw bytes to a base64-URL string (no padding).
#[must_use]
pub fn encode(bytes: &[u8]) -> String {
    let mut out = Vec::with_capacity((bytes.len() * 4).div_ceil(3));
    let mut iter = bytes.chunks(3);
    while let Some(chunk) = iter.next() {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let combined = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((combined >> 18) & 0x3f) as usize]);
        out.push(ALPHABET[((combined >> 12) & 0x3f) as usize]);
        if chunk.len() > 1 {
            out.push(ALPHABET[((combined >> 6) & 0x3f) as usize]);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(combined & 0x3f) as usize]);
        }
    }
    // SAFETY: all bytes are from ALPHABET which is pure ASCII.
    String::from_utf8(out).expect("base64-URL output is always valid UTF-8")
}

/// Decode a base64-URL string (no padding) to bytes.
///
/// # Errors
/// Returns `PaginationError::CursorMalformed` on invalid input.
pub fn decode(s: &str) -> Result<Vec<u8>, PaginationError> {
    // Build decode table: 0xFF = invalid.
    let mut table = [0xFFu8; 128];
    for (i, &c) in ALPHABET.iter().enumerate() {
        table[c as usize] = i as u8;
    }

    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::with_capacity((bytes.len() * 3).div_ceil(4));
    let mut i = 0;
    while i < bytes.len() {
        let remaining = bytes.len() - i;
        let c0 = decode_char(bytes[i], &table, s)?;
        if remaining == 1 {
            // A single base64-URL char encodes 6 bits — not a valid trailing group.
            return Err(PaginationError::CursorMalformed(s.to_owned()));
        }
        let c1 = decode_char(bytes[i + 1], &table, s)?;
        out.push((c0 << 2) | (c1 >> 4));
        if remaining == 2 {
            i += 2;
            continue;
        }
        let c2 = decode_char(bytes[i + 2], &table, s)?;
        out.push(((c1 & 0x0f) << 4) | (c2 >> 2));
        if remaining == 3 {
            i += 3;
            continue;
        }
        let c3 = decode_char(bytes[i + 3], &table, s)?;
        out.push(((c2 & 0x03) << 6) | c3);
        i += 4;
    }
    Ok(out)
}

fn decode_char(b: u8, table: &[u8; 128], raw: &str) -> Result<u8, PaginationError> {
    if b as usize >= 128 {
        return Err(PaginationError::CursorMalformed(raw.to_owned()));
    }
    let v = table[b as usize];
    if v == 0xFF {
        return Err(PaginationError::CursorMalformed(raw.to_owned()));
    }
    Ok(v)
}

/// Produce a stable `u64` scope identifier from a string.
///
/// Uses `std::hash::DefaultHasher` — deterministic within a single binary
/// invocation, which is sufficient for in-memory testing and sufficient for
/// cursor-scope binding within a single request (cursors are ephemeral).
#[must_use]
pub fn scope_hash(scope: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    // Use FNV-1a inline to guarantee cross-run determinism (DefaultHasher
    // is NOT guaranteed stable across runs/versions).
    let mut hash: u64 = 14_695_981_039_346_656_037;
    for byte in scope.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

/// Internal cursor payload.
#[derive(Debug, Eq, PartialEq)]
pub struct CursorPayload {
    /// Zero-based offset into the (filter-applied) item list.
    pub offset: u64,
    /// FNV-1a hash of the canonical filter string.
    pub scope: u64,
}

impl CursorPayload {
    /// Encode to an opaque `Cursor`.
    #[must_use]
    pub fn to_cursor(&self) -> crate::Cursor {
        let s = format!("{}:{}", self.offset, self.scope);
        crate::Cursor(encode(s.as_bytes()))
    }

    /// Decode from a `Cursor`.
    ///
    /// # Errors
    /// Returns `PaginationError::CursorMalformed` if the cursor cannot be
    /// decoded or does not match the expected payload format.
    pub fn from_cursor(c: &crate::Cursor) -> Result<Self, PaginationError> {
        let bytes = decode(&c.0)?;
        let s =
            String::from_utf8(bytes).map_err(|_| PaginationError::CursorMalformed(c.0.clone()))?;
        let mut parts = s.splitn(2, ':');
        let offset_str = parts
            .next()
            .ok_or_else(|| PaginationError::CursorMalformed(c.0.clone()))?;
        let scope_str = parts
            .next()
            .ok_or_else(|| PaginationError::CursorMalformed(c.0.clone()))?;
        let offset = offset_str
            .parse::<u64>()
            .map_err(|_| PaginationError::CursorMalformed(c.0.clone()))?;
        let scope = scope_str
            .parse::<u64>()
            .map_err(|_| PaginationError::CursorMalformed(c.0.clone()))?;
        Ok(CursorPayload { offset, scope })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_empty() {
        let encoded = encode(b"");
        assert_eq!(encoded, "");
        assert_eq!(decode(&encoded).unwrap(), b"");
    }

    #[test]
    fn encode_decode_roundtrip_short() {
        let input = b"hello";
        let encoded = encode(input);
        assert_eq!(decode(&encoded).unwrap(), input);
    }

    #[test]
    fn encode_decode_roundtrip_payload() {
        let input = b"42:12345678901234567";
        let encoded = encode(input);
        // Must use URL-safe alphabet only.
        assert!(
            encoded
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
        assert_eq!(decode(&encoded).unwrap(), input);
    }

    #[test]
    fn decode_invalid_returns_error() {
        assert!(matches!(
            decode("!!!"),
            Err(PaginationError::CursorMalformed(_))
        ));
    }

    #[test]
    fn cursor_payload_roundtrip() {
        let p = CursorPayload {
            offset: 42,
            scope: 99_999_999_999,
        };
        let c = p.to_cursor();
        let p2 = CursorPayload::from_cursor(&c).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn scope_hash_is_deterministic() {
        assert_eq!(scope_hash("filter-a"), scope_hash("filter-a"));
        assert_ne!(scope_hash("filter-a"), scope_hash("filter-b"));
    }
}
