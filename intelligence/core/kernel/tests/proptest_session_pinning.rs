//! Property tests for the pure session-pinning derivations: determinism,
//! wire-id precedence, no-raw-prompt leakage, and cache-key faithfulness.
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use intelligence_kernel::{Provider, derive_sticky_key, prompt_cache_key};
use proptest::prelude::*;

proptest! {
    // Same inputs always derive the same key (so replicas pin identically).
    #[test]
    fn derivation_is_deterministic(msg in ".{0,200}") {
        let a = derive_sticky_key(None, Some(&msg));
        let b = derive_sticky_key(None, Some(&msg));
        prop_assert_eq!(a, b);
    }

    // A non-blank wire session id always wins over the message fingerprint.
    #[test]
    fn non_blank_wire_id_takes_precedence(
        wire in "[!-~]{1,64}",
        msg in ".{0,200}",
    ) {
        let key = derive_sticky_key(Some(&wire), Some(&msg)).unwrap();
        prop_assert_eq!(key, format!("wsid:{}", wire.trim()));
    }

    // The fingerprint never embeds raw prompt content and is fixed-width.
    #[test]
    fn message_key_hides_prompt_and_is_fixed_width(msg in "[a-zA-Z0-9 ]{8,200}") {
        let key = derive_sticky_key(None, Some(&msg)).unwrap();
        prop_assert!(key.starts_with("sticky:"));
        prop_assert!(!key.contains(&msg));
        prop_assert_eq!(key.len(), "sticky:".len() + 16);
    }

    // Distinct (provider, model) pairs never share a cache slot for one session.
    #[test]
    fn cache_key_separates_model(
        wire in "[!-~]{1,32}",
        m1 in "[a-z0-9-]{1,30}",
        m2 in "[a-z0-9-]{1,30}",
    ) {
        let key = derive_sticky_key(Some(&wire), None).unwrap();
        let k1 = prompt_cache_key(Provider::Anthropic, &key, &m1);
        let k2 = prompt_cache_key(Provider::Anthropic, &key, &m2);
        prop_assert_eq!(k1 == k2, m1 == m2);
    }
}
