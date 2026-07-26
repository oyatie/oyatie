//! The value half of a META key/value pair.

use std::fmt::Write as _;
use os_kernel::{Error, Result};

/// The maximum byte length of a single ADV1 value.
///
/// ADV1 encodes the value length as a big-endian `u16`, so a single tag's
/// payload cannot exceed 65535 bytes.
pub const MAX_VALUE_LEN: usize = u16::MAX as usize;

/// A stored META value: an opaque byte string that is usually UTF-8 text
/// (an image reference, a token, serialized config), but is not required to be.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetaValue {
    bytes: Vec<u8>,
}

impl MetaValue {
    /// Wraps raw bytes, enforcing the ADV1 length limit.
    pub fn new(bytes: impl Into<Vec<u8>>) -> Result<Self> {
        let bytes = bytes.into();
        if bytes.len() > MAX_VALUE_LEN {
            return Err(Error::invalid("meta value exceeds maximum ADV1 length"));
        }
        Ok(Self { bytes })
    }

    /// Convenience constructor for a UTF-8 string value.
    // Inherent `from_str` (rather than a `FromStr` impl) is deliberate: it keeps
    // the ergonomic `MetaValue::from_str("...")` call form used throughout the
    // crate, and a `FromStr` impl could not be `const` nor borrow-friendly here.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        Self::new(s.as_bytes().to_vec())
    }

    /// Borrows the raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Consumes the value, returning the owned bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Interprets the value as UTF-8, returning an error if it is not valid.
    pub fn as_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.bytes).map_err(|_| Error::parse("meta value is not valid UTF-8"))
    }

    /// Interprets the value as an owned UTF-8 string.
    pub fn to_string_lossy_checked(&self) -> Result<String> {
        self.as_str().map(String::from)
    }

    /// Number of bytes in the value.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Whether the value is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    // --- scalar codec --------------------------------------------------------
    //
    // Several META values in Talos are scalars stored as their decimal/text
    // representation (an install-step counter, a boolean flag, ...). These
    // helpers provide a small, well-tested key/value codec for those cases.

    /// Encodes an unsigned integer as its decimal text representation.
    pub fn from_u64(n: u64) -> Self {
        // Decimal of a u64 never exceeds 20 bytes, well under the limit.
        Self {
            bytes: n.to_string().into_bytes(),
        }
    }

    /// Parses the value as a decimal unsigned integer.
    pub fn as_u64(&self) -> Result<u64> {
        self.as_str()?
            .parse::<u64>()
            .map_err(|_| Error::parse("meta value is not a valid u64"))
    }

    /// Encodes a boolean as the text `"true"`/`"false"`.
    pub fn from_bool(b: bool) -> Self {
        Self::from_str(if b { "true" } else { "false" }).expect("ascii literal fits")
    }

    /// Parses the value as a boolean (`"true"`/`"false"`, case-insensitive).
    pub fn as_bool(&self) -> Result<bool> {
        match self.as_str()?.to_ascii_lowercase().as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            other => Err(Error::parse(format!(
                "meta value {other:?} is not a boolean"
            ))),
        }
    }

    /// Encodes raw bytes as a lowercase hex string value.
    pub fn from_hex_of(bytes: &[u8]) -> Self {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            let _ = write!(s, "{b:02x}");
        }
        Self::from_str(&s).expect("hex fits")
    }

    /// Decodes the value, treated as a hex string, back into raw bytes.
    pub fn decode_hex(&self) -> Result<Vec<u8>> {
        let s = self.as_str()?;
        if !s.len().is_multiple_of(2) {
            return Err(Error::parse("hex value has odd length"));
        }
        let mut out = Vec::with_capacity(s.len() / 2);
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let hi = hex_nibble(bytes[i])?;
            let lo = hex_nibble(bytes[i + 1])?;
            out.push((hi << 4) | lo);
            i += 2;
        }
        Ok(out)
    }
}

fn hex_nibble(b: u8) -> Result<u8> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(Error::parse("invalid hex digit")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_round_trips() {
        let v = MetaValue::from_str("ghcr.io/siderolabs/installer:v1.7.0").unwrap();
        assert_eq!(v.as_str().unwrap(), "ghcr.io/siderolabs/installer:v1.7.0");
        assert!(!v.is_empty());
    }

    #[test]
    fn rejects_oversized_value() {
        let big = vec![0u8; MAX_VALUE_LEN + 1];
        assert!(MetaValue::new(big).is_err());
        let ok = vec![0u8; MAX_VALUE_LEN];
        assert!(MetaValue::new(ok).is_ok());
    }

    #[test]
    fn non_utf8_is_rejected_as_str() {
        let v = MetaValue::new(vec![0xff, 0xfe, 0xfd]).unwrap();
        assert!(v.as_str().is_err());
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn u64_codec_round_trips() {
        for n in [0u64, 1, 42, u64::MAX] {
            let v = MetaValue::from_u64(n);
            assert_eq!(v.as_u64().unwrap(), n);
        }
        assert!(MetaValue::from_str("nan").unwrap().as_u64().is_err());
    }

    #[test]
    fn bool_codec_round_trips() {
        assert!(MetaValue::from_bool(true).as_bool().unwrap());
        assert!(!MetaValue::from_bool(false).as_bool().unwrap());
        assert!(MetaValue::from_str("TRUE").unwrap().as_bool().unwrap());
        assert!(!MetaValue::from_str("0").unwrap().as_bool().unwrap());
        assert!(MetaValue::from_str("maybe").unwrap().as_bool().is_err());
    }

    #[test]
    fn hex_codec_round_trips() {
        let raw = vec![0x00, 0xde, 0xad, 0xbe, 0xef, 0xff];
        let v = MetaValue::from_hex_of(&raw);
        assert_eq!(v.as_str().unwrap(), "00deadbeefff");
        assert_eq!(v.decode_hex().unwrap(), raw);
    }

    #[test]
    fn hex_codec_rejects_bad_input() {
        assert!(MetaValue::from_str("abc").unwrap().decode_hex().is_err());
        assert!(MetaValue::from_str("zz").unwrap().decode_hex().is_err());
        assert!(
            MetaValue::from_str("")
                .unwrap()
                .decode_hex()
                .unwrap()
                .is_empty()
        );
    }
}
