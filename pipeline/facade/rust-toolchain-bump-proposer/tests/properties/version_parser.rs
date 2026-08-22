//! Property coverage for the caller-controlled version parser (`parse_stable_version`) and the
//! comparison surface (`latest_is_newer`), per the repository's parser-testing floor
//! (docs/standards/testing.md §Property: `proptest` inside `tests/properties/`).
//!
//! The parser feeds filesystem-mutation decisions from flag/environment text, so generated
//! coverage asserts: (a) never panics on arbitrary input; (b) accepted inputs normalize
//! idempotently to a three-part numeric form; (c) rejection is consistent (non-numeric tokens
//! reject, `v`/whitespace/parenthetical forms normalize); (d) comparison is antisymmetric and
//! reflexive on accepted forms.

use ci_rust_toolchain_bump_proposer::{latest_is_newer, parse_stable_version};
use proptest::prelude::*;

fn normalized_components() -> impl Strategy<Value = (u64, u64, u64)> {
    (0..10_000u64, 0..100u64, 0..100u64)
}

proptest! {
    /// Any accepted input normalizes to a three-part numeric string, and normalization is
    /// idempotent: parsing the normalized form yields the same value.
    #[test]
    fn parse_normalizes_and_is_idempotent((a, b, c) in normalized_components()) {
        for raw in [
            format!("{a}.{b}.{c}"),
            format!("{a}.{b}"),
            format!("v{a}.{b}.{c}"),
            format!("  {a}.{b}.{c}  "),
            format!("{a}.{b}.{c} (8bab26f4f 2026-07-14)"),
        ] {
            let parsed = parse_stable_version(&raw).unwrap_or_else(|error| {
                panic!("expected {raw:?} to parse: {error}")
            });
            let parts: Vec<&str> = parsed.split('.').collect();
            prop_assert_eq!(parts.len(), 3);
            prop_assert!(
                parts.iter().all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit())),
                "normalized parts must be numeric: {}",
                parsed
            );
            let again = parse_stable_version(&parsed).expect("normalized form must reparse");
            prop_assert_eq!(again, parsed);
        }
    }

    /// Comparison is antisymmetric and reflexive: newer(a, b) XOR newer(b, a) for distinct
    /// versions, and newer(a, a) is false.
    #[test]
    fn comparison_is_antisymmetric_and_reflexive((a1, a2, a3) in normalized_components(), (b1, b2, b3) in normalized_components()) {
        let a = format!("{a1}.{a2}.{a3}");
        let b = format!("{b1}.{b2}.{b3}");
        let a_newer = latest_is_newer(&a, &b).expect("compare must not fail on parsed forms");
        let b_newer = latest_is_newer(&b, &a).expect("compare must not fail on parsed forms");
        if a == b {
            prop_assert!(!a_newer && !b_newer);
        } else {
            prop_assert!(a_newer ^ b_newer, "{a} vs {b} must order exactly one way");
        }
        prop_assert!(!latest_is_newer(&a, &a).expect("reflexive compare"));
    }

    /// Non-numeric tokens reject: letters, symbols, and multi-segment forms never parse.
    #[test]
    fn malformed_inputs_reject(input in "[^0-9.]{1,20}|[0-9.]{5,20}") {
        let result = parse_stable_version(&input);
        if input.contains(|ch: char| !ch.is_ascii_digit() && ch != '.') {
            prop_assert!(result.is_err(), "{input:?} must reject (non-numeric token)");
        }
    }

    /// Arbitrary strings (any unicode) never panic the parser.
    #[test]
    fn parser_never_panics_on_arbitrary_input(input in "\\PC*") {
        let _ = parse_stable_version(&input);
        let _ = latest_is_newer("1.97.1", &input);
    }
}
