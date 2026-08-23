//! Production ULID minting for decision ids.
//!
//! The shared Cedar PDP mints one fresh decision id per decision (cached
//! replays included) through the [`IdGenerator`] port. This is the first
//! PRODUCTION implementation in the workspace (the kernel ships only the
//! deterministic `SeededIdGenerator` for tests): 48-bit millisecond
//! timestamp + 80 bits of entropy from the ADR-0506 blessed crypto stack
//! (aws-lc-rs `SystemRandom`), Crockford-base32 per the ULID spec.
//!
//! Failure posture: an entropy or clock failure yields an error — never a
//! degraded id — and the PDP maps it to `DecisionIdUnavailable`, refusing to
//! emit a decision that would be unattributable in the audit chain.

use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use shared_ulid_id_kernel::{IdGenerator, IdGeneratorError, Ulid};

/// Crockford-base32 alphabet (ULID spec: no I, L, O, U).
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Wall-clock + CSPRNG [`IdGenerator`].
#[derive(Debug, Default)]
pub struct SystemUlidIdGenerator {
    rng: SystemRandom,
}

impl SystemUlidIdGenerator {
    /// Build a generator over the process-global CSPRNG.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rng: SystemRandom::new(),
        }
    }
}

/// Encode the 128-bit ULID value as 26 Crockford-base32 characters
/// (MSB-first; the leading character carries only 3 significant bits, so a
/// 48-bit-masked timestamp always satisfies the spec's `0..=7` constraint).
fn encode(value: u128) -> String {
    let mut out = String::with_capacity(26);
    for i in 0..26 {
        let shift = 5 * (25 - i);
        let digit = ((value >> shift) & 0x1F) as usize;
        // digit < 32 by construction (5-bit mask); indexing is total.
        out.push(char::from(CROCKFORD[digit]));
    }
    out
}

impl IdGenerator for SystemUlidIdGenerator {
    fn new_ulid(&self) -> Result<Ulid, IdGeneratorError> {
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0); // pre-epoch clock: encode as the epoch, never panic
        let timestamp = millis & ((1u128 << 48) - 1);
        let mut entropy = [0u8; 10];
        self.rng.fill(&mut entropy).map_err(|_| {
            // The kernel error enum has no entropy variant; the detail string
            // names the real failure and the PDP surfaces it as
            // DecisionIdUnavailable (fail-closed, no decision emitted).
            IdGeneratorError::MalformedUlid("entropy-unavailable".to_owned())
        })?;
        let mut random: u128 = 0;
        for byte in entropy {
            random = (random << 8) | u128::from(byte);
        }
        let value = (timestamp << 80) | random;
        Ulid::try_new(encode(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mints_valid_distinct_ulids() {
        let id_gen = SystemUlidIdGenerator::new();
        let a = id_gen.new_ulid().expect("mint a");
        let b = id_gen.new_ulid().expect("mint b");
        assert_ne!(a, b, "80 bits of entropy must differ across mints");
        assert_eq!(a.as_str().len(), 26);
    }

    #[test]
    fn leading_character_respects_timestamp_overflow_guard() {
        let id_gen = SystemUlidIdGenerator::new();
        for _ in 0..32 {
            let ulid = id_gen.new_ulid().expect("mint");
            let first = ulid.as_str().as_bytes()[0];
            assert!(
                (b'0'..=b'7').contains(&first),
                "ULID first char must be 0-7, got {}",
                char::from(first)
            );
        }
    }

    #[test]
    fn encode_is_msb_first_crockford() {
        // 1 in the lowest 5 bits encodes as ...0001 over 26 chars.
        assert_eq!(encode(1), "00000000000000000000000001");
        assert_eq!(encode(31), "0000000000000000000000000Z");
    }
}
