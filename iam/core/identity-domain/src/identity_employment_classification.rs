// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! Employment classification value object.
//!
//! This type is the merge-variant landing of the `employment_classification`
//! contract from `.omc/plans/milestones/M02b-substrate/phases/P03-identity/
//! impl-plan.md` (Concrete File Targets: `identity.employments` DDL +
//! `Employment` entity) into the existing `identity-domain` crate (kept
//! per `F-M02B-PLAN-LIVE-CRATE-RECONCILIATION`). It is additive — existing
//! types (`UserId`, `User`, `Principal`, `Token`, `StsCredential`) are
//! unchanged.
//!
//! Bominal ADR-0126 defines 8 Korean employment classification classes that
//! map to the `classification` column of `identity.employments` table. The
//! enum is the canonical Rust representation of those 8 wire values and is
//! intentionally exhaustive — all 8 classes must be handled.

use std::fmt;

/// Eight Korean employment classification classes per Bominal ADR-0126.
///
/// Each variant's `as_str()` value is the wire/SQL string that appears in the
/// `identity.employments.classification` column (see `V001__identity_init.sql`
/// CHECK constraint). Round-trip via `EmploymentClassification::from_wire()`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EmploymentClassification {
    /// 정규직 — full-time permanent employee
    Regular,
    /// 계약직 — fixed-term contract employee
    Contract,
    /// 단시간근로자 — part-time (reduced hours) employee
    PartTime,
    /// 파견 — dispatched (agency/temp) worker
    Dispatched,
    /// 도급 — contracted-out / outsourced worker
    Outsourced,
    /// 프리랜서 — freelancer / independent contractor
    Freelancer,
    /// 인턴 — intern
    Intern,
    /// 임원 — executive / officer
    Executive,
}

/// Error returned when a raw string does not match any classification variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownEmploymentClassification(pub String);

impl fmt::Display for UnknownEmploymentClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown employment classification: {:?}; expected one of \
             정규직|계약직|단시간근로자|파견|도급|프리랜서|인턴|임원",
            self.0
        )
    }
}

impl std::error::Error for UnknownEmploymentClassification {}

impl EmploymentClassification {
    /// Returns the canonical Korean wire string used in the SQL CHECK constraint.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Regular => "정규직",
            Self::Contract => "계약직",
            Self::PartTime => "단시간근로자",
            Self::Dispatched => "파견",
            Self::Outsourced => "도급",
            Self::Freelancer => "프리랜서",
            Self::Intern => "인턴",
            Self::Executive => "임원",
        }
    }

    /// Parse from the Korean wire string.
    pub fn from_wire(s: &str) -> Result<Self, UnknownEmploymentClassification> {
        match s {
            "정규직" => Ok(Self::Regular),
            "계약직" => Ok(Self::Contract),
            "단시간근로자" => Ok(Self::PartTime),
            "파견" => Ok(Self::Dispatched),
            "도급" => Ok(Self::Outsourced),
            "프리랜서" => Ok(Self::Freelancer),
            "인턴" => Ok(Self::Intern),
            "임원" => Ok(Self::Executive),
            other => Err(UnknownEmploymentClassification(other.to_string())),
        }
    }

    /// All 8 variants in ADR-0126 declaration order.
    pub const ALL: [Self; 8] = [
        Self::Regular,
        Self::Contract,
        Self::PartTime,
        Self::Dispatched,
        Self::Outsourced,
        Self::Freelancer,
        Self::Intern,
        Self::Executive,
    ];
}

impl fmt::Display for EmploymentClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_eight_adr_0126_classifications_compile_and_are_distinct() {
        let all = EmploymentClassification::ALL;
        assert_eq!(all.len(), 8);
        // All wire strings are distinct
        let mut seen = std::collections::HashSet::new();
        for variant in all {
            assert!(
                seen.insert(variant.as_str()),
                "duplicate wire string: {}",
                variant.as_str()
            );
        }
    }

    #[test]
    fn round_trip_from_str_for_every_variant() {
        for variant in EmploymentClassification::ALL {
            let wire = variant.as_str();
            let parsed = EmploymentClassification::from_wire(wire)
                .expect("all wire strings must round-trip");
            assert_eq!(parsed, variant, "round-trip failed for {wire}");
        }
    }

    #[test]
    fn from_str_rejects_unknown_value() {
        let err = EmploymentClassification::from_wire("unknown")
            .expect_err("unknown string must be rejected");
        assert_eq!(err.0, "unknown");
        let msg = err.to_string();
        assert!(msg.contains("unknown employment classification"));
        assert!(msg.contains("unknown"));
    }

    #[test]
    fn display_renders_korean_wire_string() {
        assert_eq!(EmploymentClassification::Regular.to_string(), "정규직");
        assert_eq!(EmploymentClassification::Executive.to_string(), "임원");
    }

    #[test]
    fn from_str_rejects_empty_string() {
        let err =
            EmploymentClassification::from_wire("").expect_err("empty string must be rejected");
        assert_eq!(err.0, "");
        assert!(
            err.to_string()
                .contains("unknown employment classification")
        );
    }

    #[test]
    fn from_str_rejects_english_equivalent() {
        // English labels must not accidentally match — only Korean wire strings are valid
        let err = EmploymentClassification::from_wire("Regular")
            .expect_err("English label must not match");
        assert_eq!(err.0, "Regular");
    }

    #[test]
    fn unknown_employment_classification_error_display_covers_all_variants() {
        let msg = UnknownEmploymentClassification("xyz".to_string()).to_string();
        assert!(msg.contains("xyz"));
        assert!(msg.contains("정규직"));
        assert!(msg.contains("임원"));
    }

    #[test]
    fn all_variants_covered_by_as_str_and_from_str_symmetry() {
        // Exhaustively verify the CHECK constraint values from V001__identity_init.sql
        let expected_wire_strings = [
            "정규직",
            "계약직",
            "단시간근로자",
            "파견",
            "도급",
            "프리랜서",
            "인턴",
            "임원",
        ];
        for wire in expected_wire_strings {
            let parsed = EmploymentClassification::from_wire(wire)
                .unwrap_or_else(|_| panic!("wire string {wire:?} must parse"));
            assert_eq!(parsed.as_str(), wire, "as_str must be identity for {wire}");
        }
    }
}
