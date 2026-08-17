//! Deterministic percentage bucketing.
//!
//! Stickiness contract: the bucket is a pure function of `(flag_key, salt, targeting_key)`. The
//! same triple ALWAYS yields the same basis-point bucket, on any machine, with no clock, no RNG,
//! and no allocation beyond the inputs. This is what makes percentage rollouts sticky and
//! reproducible across processes, regions, and replays.
//!
//! We use FNV-1a (64-bit) implemented inline rather than `std::hash::DefaultHasher`, because the
//! standard hasher is explicitly NOT guaranteed stable across Rust versions or platforms, which
//! would silently re-bucket every subject on a toolchain bump. FNV-1a is fixed by definition.

use crate::model::TOTAL_BASIS_POINTS;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64-bit hash over a byte slice. Stable by specification.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Compute the deterministic bucket for a subject, in basis points `0..TOTAL_BASIS_POINTS`
/// (i.e. `0..=9999`).
///
/// The hash domain is the NUL-joined `(flag_key, salt, targeting_key)`. NUL-joining (rather than
/// plain concatenation) prevents boundary collisions such as `("ab", "", "c")` vs `("a", "", "bc")`
/// mapping to the same byte string.
pub fn bucket_basis_points(flag_key: &str, salt: &str, targeting_key: &str) -> u32 {
    let mut domain = Vec::with_capacity(flag_key.len() + salt.len() + targeting_key.len() + 2);
    domain.extend_from_slice(flag_key.as_bytes());
    domain.push(0);
    domain.extend_from_slice(salt.as_bytes());
    domain.push(0);
    domain.extend_from_slice(targeting_key.as_bytes());

    let hash = fnv1a_64(&domain);
    // Map the 64-bit hash uniformly into [0, TOTAL_BASIS_POINTS). The modulo bias across a
    // 64-bit space against a 10_000 divisor is below 1 part in 1.8e15 — negligible.
    (hash % u64::from(TOTAL_BASIS_POINTS)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_is_in_range() {
        for i in 0..1000 {
            let key = format!("subject-{i}");
            let bp = bucket_basis_points("flag.x", "", &key);
            assert!(bp < TOTAL_BASIS_POINTS, "bucket {bp} out of range");
        }
    }

    #[test]
    fn bucket_is_deterministic() {
        let a = bucket_basis_points("flag.x", "salt", "user-42");
        let b = bucket_basis_points("flag.x", "salt", "user-42");
        assert_eq!(a, b);
    }

    #[test]
    fn flag_key_changes_bucket() {
        // The SAME subject buckets independently per flag (no cross-flag correlation by default).
        let a = bucket_basis_points("flag.a", "", "user-42");
        let b = bucket_basis_points("flag.b", "", "user-42");
        assert_ne!(a, b, "distinct flags should not lock-step the same subject");
    }

    #[test]
    fn salt_changes_bucket() {
        let a = bucket_basis_points("flag.x", "exp-1", "user-42");
        let b = bucket_basis_points("flag.x", "exp-2", "user-42");
        assert_ne!(
            a, b,
            "distinct salts should decorrelate the same subject on the same flag"
        );
    }

    #[test]
    fn nul_join_avoids_boundary_collision() {
        let a = bucket_basis_points("ab", "", "c");
        let b = bucket_basis_points("a", "", "bc");
        // Not a hard guarantee for ALL inputs, but the documented boundary case must differ.
        assert_ne!(a, b);
    }

    #[test]
    fn distribution_is_roughly_uniform() {
        // Coarse smoke test: ~50% should land below the 5000bp line over many subjects.
        let mut below = 0u32;
        let n = 20_000u32;
        for i in 0..n {
            let key = format!("u{i}");
            if bucket_basis_points("flag.dist", "", &key) < TOTAL_BASIS_POINTS / 2 {
                below += 1;
            }
        }
        let ratio = f64::from(below) / f64::from(n);
        assert!(
            (0.45..=0.55).contains(&ratio),
            "skewed distribution: {ratio}"
        );
    }

    #[test]
    fn fnv1a_known_vector() {
        // FNV-1a 64-bit of empty input is the offset basis (canonical reference vector).
        assert_eq!(fnv1a_64(b""), FNV_OFFSET_BASIS);
        // FNV-1a 64-bit of "a" == 0xaf63dc4c8601ec8c (canonical reference vector).
        assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
    }
}
