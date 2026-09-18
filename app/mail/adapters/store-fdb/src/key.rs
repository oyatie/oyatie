//! Fixed-width key encoder. No tuple layer: every segment has a width fixed
//! by the caller, so byte order equals field order and every range bound is
//! prefix arithmetic. Compiled in both builds; no `libfdb_c` involved.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyError {
    /// The text is longer than the width it must fit.
    TooLong { width: usize, len: usize },
    /// Interior NUL would be indistinguishable from padding.
    Nul,
}

/// A key under construction. Segments append in order; the encoded bytes are
/// the key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key(Vec<u8>);

/// `[begin, end)` covering exactly the keys that start with one prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRange {
    pub begin: Vec<u8>,
    pub end: Vec<u8>,
}

/// Bytes of a versionstamp placeholder inside a key.
pub const VERSIONSTAMP_LEN: usize = 10;

impl Key {
    pub fn new(prefix: &[u8]) -> Self {
        Self(prefix.to_vec())
    }

    /// One-byte discriminator between record kinds under a shared prefix.
    pub fn tag(mut self, tag: u8) -> Self {
        self.0.push(tag);
        self
    }

    pub fn u32(mut self, value: u32) -> Self {
        self.0.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn u64(mut self, value: u64) -> Self {
        self.0.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// Order-preserving: the sign bit is flipped so negatives sort first.
    pub fn i64(self, value: i64) -> Self {
        self.u64((value as u64) ^ (1 << 63))
    }

    /// Raw bytes whose width the caller has already fixed (ids, hashes).
    pub fn raw(mut self, bytes: &[u8]) -> Self {
        self.0.extend_from_slice(bytes);
        self
    }

    /// UTF-8 text zero-padded to `width`; refuses longer text and interior
    /// NUL. Equal-width padding keeps lexical order equal to text order.
    pub fn text(mut self, width: usize, text: &str) -> Result<Self, KeyError> {
        let len = text.len();
        if len > width {
            return Err(KeyError::TooLong { width, len });
        }
        if text.bytes().any(|byte| byte == 0) {
            return Err(KeyError::Nul);
        }
        self.0.extend_from_slice(text.as_bytes());
        self.0.resize(self.0.len() + width - len, 0);
        Ok(self)
    }

    /// Appends the 10-byte placeholder `SET_VERSIONSTAMPED_KEY` fills at
    /// commit. The result is only usable as that mutation's key parameter.
    pub fn versionstamped(self) -> VersionstampedKey {
        let offset = self.0.len();
        let mut bytes = self.0;
        bytes.resize(offset + VERSIONSTAMP_LEN, 0);
        bytes.extend_from_slice(&(offset as u32).to_le_bytes());
        VersionstampedKey { bytes, offset }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    /// Every key that starts with this one, as a half-open range.
    pub fn range(&self) -> KeyRange {
        KeyRange {
            begin: self.0.clone(),
            end: strinc(&self.0),
        }
    }
}

/// Smallest byte string greater than every string with `prefix`. An empty or
/// all-`0xff` prefix has no such string below the system keyspace; `[0xff]`
/// is returned as the user-keyspace ceiling.
pub fn strinc(prefix: &[u8]) -> Vec<u8> {
    let trimmed = prefix.trim_ascii_end_matches_ff();
    match trimmed.split_last() {
        Some((last, head)) => {
            let mut end = head.to_vec();
            end.push(last + 1);
            end
        }
        None => vec![0xff],
    }
}

trait TrimFf {
    fn trim_ascii_end_matches_ff(&self) -> &[u8];
}

impl TrimFf for [u8] {
    fn trim_ascii_end_matches_ff(&self) -> &[u8] {
        let mut end = self.len();
        while end > 0 && self[end - 1] == 0xff {
            end -= 1;
        }
        &self[..end]
    }
}

pub fn read_u64(bytes: &[u8], at: usize) -> Option<u64> {
    let slice = bytes.get(at..at.checked_add(8)?)?;
    Some(u64::from_be_bytes(slice.try_into().ok()?))
}

pub fn read_i64(bytes: &[u8], at: usize) -> Option<i64> {
    read_u64(bytes, at).map(|raw| (raw ^ (1 << 63)) as i64)
}

/// Key parameter for `SET_VERSIONSTAMPED_KEY`: the key with a zeroed
/// 10-byte placeholder followed by the placeholder's little-endian offset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionstampedKey {
    bytes: Vec<u8>,
    offset: usize,
}

impl VersionstampedKey {
    pub fn atomic_param(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// The key as it will exist after commit, given the versionstamp FDB
    /// assigned.
    pub fn resolve(&self, versionstamp: [u8; VERSIONSTAMP_LEN]) -> Vec<u8> {
        let mut key = self.bytes[..self.offset].to_vec();
        key.extend_from_slice(&versionstamp);
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_sort_numerically() {
        let a = Key::new(b"m").u64(1).into_bytes();
        let b = Key::new(b"m").u64(256).into_bytes();
        assert!(a < b);
        let neg = Key::new(b"m").i64(-5).into_bytes();
        let zero = Key::new(b"m").i64(0).into_bytes();
        let pos = Key::new(b"m").i64(7).into_bytes();
        assert!(neg < zero && zero < pos);
        assert_eq!(read_i64(&neg, 1), Some(-5));
        assert_eq!(read_u64(&b, 1), Some(256));
        assert_eq!(read_u64(&b, 2), None);
    }

    #[test]
    fn text_is_padded_and_refused_beyond_width() {
        let key = Key::new(b"").text(4, "ab").unwrap().into_bytes();
        assert_eq!(key, b"ab\0\0");
        assert_eq!(
            Key::new(b"").text(1, "ab").err(),
            Some(KeyError::TooLong { width: 1, len: 2 })
        );
        assert_eq!(Key::new(b"").text(4, "a\0").err(), Some(KeyError::Nul));
    }

    #[test]
    fn range_end_is_the_next_prefix() {
        assert_eq!(strinc(b"ab"), b"ac");
        assert_eq!(strinc(b"a\xff\xff"), b"b");
        assert_eq!(strinc(b"\xff"), vec![0xff]);
        let range = Key::new(b"p").tag(1).range();
        assert_eq!(range.begin, b"p\x01");
        assert_eq!(range.end, b"p\x02");
    }

    #[test]
    fn versionstamp_param_carries_placeholder_and_offset() {
        let key = Key::new(b"audit").tag(2).versionstamped();
        let param = key.atomic_param();
        assert_eq!(key.offset(), 6);
        assert_eq!(&param[..6], b"audit\x02");
        assert_eq!(&param[6..16], &[0; 10]);
        assert_eq!(&param[16..], &6u32.to_le_bytes());
        let resolved = key.resolve([9; 10]);
        assert_eq!(&resolved[..6], b"audit\x02");
        assert_eq!(&resolved[6..], &[9; 10]);
    }
}
