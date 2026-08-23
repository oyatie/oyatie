//! Identifier generation traits and the [`Fingerprint`] primitive.

use crate::error::{Error, Result};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// A short content fingerprint, used to deduplicate and version resources.
///
/// This is a self-contained, deterministic FNV-1a 64-bit hash rendered as a
/// fixed-width hex string. It is NOT cryptographically secure — it mirrors the
/// role of Talos resource "version" fingerprints used to detect change, not to
/// authenticate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fingerprint(u64);

impl Fingerprint {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0100_0000_01b3;

    /// Compute a fingerprint over arbitrary bytes.
    pub fn of(bytes: &[u8]) -> Self {
        let mut hash = Self::FNV_OFFSET;
        for &b in bytes {
            hash ^= u64::from(b);
            hash = hash.wrapping_mul(Self::FNV_PRIME);
        }
        Fingerprint(hash)
    }

    /// Compute a fingerprint over a string.
    pub fn of_str(s: &str) -> Self {
        Self::of(s.as_bytes())
    }

    /// The raw 64-bit value.
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Render as a zero-padded 16-char lowercase hex string.
    pub fn to_hex(&self) -> String {
        alloc::format!("{:016x}", self.0)
    }

    /// Parse a fingerprint from a 16-char hex string.
    pub fn from_hex(s: &str) -> Result<Self> {
        if s.len() != 16 {
            return Err(Error::parse("fingerprint hex must be 16 characters"));
        }
        let v = u64::from_str_radix(s, 16).map_err(|_| Error::parse("invalid fingerprint hex"))?;
        Ok(Fingerprint(v))
    }
}

impl fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Trait for components that generate unique identifiers.
///
/// Implementations may be monotonic counters, hash-based, or derived from
/// platform entropy. Kept abstract so subsystems can inject deterministic
/// generators in tests.
pub trait IdGenerator {
    /// Produce the next identifier.
    fn next_id(&mut self) -> String;
}

/// A simple deterministic, monotonic id generator with a fixed prefix.
///
/// Useful as a default and for tests. Produces `"<prefix>-<n>"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequentialIdGenerator {
    prefix: String,
    counter: u64,
}

impl SequentialIdGenerator {
    /// Create a generator starting at 0 with the given prefix.
    pub fn new(prefix: impl Into<String>) -> Self {
        SequentialIdGenerator {
            prefix: prefix.into(),
            counter: 0,
        }
    }

    /// Number of ids generated so far.
    pub fn count(&self) -> u64 {
        self.counter
    }

    /// Collect the next `n` ids.
    pub fn take(&mut self, n: usize) -> Vec<String> {
        (0..n).map(|_| self.next_id()).collect()
    }
}

impl IdGenerator for SequentialIdGenerator {
    fn next_id(&mut self) -> String {
        let id = alloc::format!("{}-{}", self.prefix, self.counter);
        self.counter += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_deterministic_and_distinct() {
        let a = Fingerprint::of_str("machine-config-v1");
        let b = Fingerprint::of_str("machine-config-v1");
        let c = Fingerprint::of_str("machine-config-v2");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn fingerprint_hex_roundtrip() {
        let f = Fingerprint::of_str("hello world");
        let hex = f.to_hex();
        assert_eq!(hex.len(), 16);
        assert_eq!(Fingerprint::from_hex(&hex).unwrap(), f);
        assert!(Fingerprint::from_hex("xyz").is_err());
        assert!(Fingerprint::from_hex("zzzzzzzzzzzzzzzz").is_err());
    }

    #[test]
    fn sequential_generator_is_monotonic() {
        let mut generator = SequentialIdGenerator::new("svc");
        assert_eq!(generator.next_id(), "svc-0");
        assert_eq!(generator.next_id(), "svc-1");
        assert_eq!(generator.count(), 2);
        let batch = generator.take(3);
        assert_eq!(batch, ["svc-2", "svc-3", "svc-4"]);
    }

    #[test]
    fn fnv_matches_known_vector() {
        // FNV-1a of empty input is the offset basis.
        assert_eq!(Fingerprint::of(b"").value(), 0xcbf2_9ce4_8422_2325);
    }
}
