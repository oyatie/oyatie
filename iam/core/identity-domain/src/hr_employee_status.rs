// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

//! HR employee lifecycle status value object.
//!
//! This type is the merge-variant landing of the `employee_status`
//! contract from `.omc/plans/milestones/M07-first-tenant/phases/P01-hr/
//! impl-plan.md` (Concrete File Targets: `hr.employees` DDL column
//! `status hr.employee_status`) into the existing `identity-domain`
//! crate (merge-variant execution per user directive 2026-05-17,
//! `execution_variant: merge-into-existing-crates`). It is additive —
//! existing types (`UserId`, `User`, `Principal`, `Token`, `StsCredential`,
//! `EmploymentClassification`) are unchanged.
//!
//! The three variants map exactly to the `hr.employee_status` Postgres ENUM
//! defined in the P01-hr DDL (`'active' | 'terminated' | 'on_leave'`).
//! Round-trip via [`EmployeeStatus::from_wire`] / [`EmployeeStatus::as_str`].
//!
//! Statute: 대한민국.노동.근로기준법 §17 (record retention 3yr), §42
//! (ADR-0126 / Bominal ADR-0125 domain naming canon)

use std::fmt;

/// Three-state employee lifecycle status per the P01-hr HR µservice schema.
///
/// Each variant's [`as_str`](EmployeeStatus::as_str) value is the wire/SQL
/// string that appears in the `hr.employees.status` column (see
/// `migrations/hr/001_hr_schema.sql`). Round-trip via
/// [`EmployeeStatus::from_wire`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EmployeeStatus {
    /// 재직중 — employee is currently active
    Active,
    /// 퇴직 — employment has been terminated
    Terminated,
    /// 휴직중 — employee is on approved leave
    OnLeave,
}

/// Error returned when a raw string does not match any [`EmployeeStatus`] variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownEmployeeStatus(pub String);

impl fmt::Display for UnknownEmployeeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown employee status: {:?}; expected one of active|terminated|on_leave",
            self.0
        )
    }
}

impl std::error::Error for UnknownEmployeeStatus {}

impl EmployeeStatus {
    /// Returns the canonical SQL wire string used in the `hr.employees.status` column.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Terminated => "terminated",
            Self::OnLeave => "on_leave",
        }
    }

    /// Parse from the SQL wire string.
    pub fn from_wire(s: &str) -> Result<Self, UnknownEmployeeStatus> {
        match s {
            "active" => Ok(Self::Active),
            "terminated" => Ok(Self::Terminated),
            "on_leave" => Ok(Self::OnLeave),
            other => Err(UnknownEmployeeStatus(other.to_string())),
        }
    }

    /// Returns the Korean display label for this status.
    pub const fn as_korean(self) -> &'static str {
        match self {
            Self::Active => "재직중",
            Self::Terminated => "퇴직",
            Self::OnLeave => "휴직중",
        }
    }

    /// `true` iff the employee may accrue leave or hold active employment terms.
    ///
    /// Terminated employees cannot; OnLeave employees retain accrual rights
    /// per 근로기준법 §60.
    pub const fn is_leave_accrual_eligible(self) -> bool {
        !matches!(self, Self::Terminated)
    }

    /// All 3 variants in DDL declaration order (`active`, `terminated`, `on_leave`).
    pub const ALL: [Self; 3] = [Self::Active, Self::Terminated, Self::OnLeave];
}

impl fmt::Display for EmployeeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_three_variants_compile_and_are_distinct() {
        let all = EmployeeStatus::ALL;
        assert_eq!(all.len(), 3);
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
    fn round_trip_from_wire_for_every_variant() {
        for variant in EmployeeStatus::ALL {
            let wire = variant.as_str();
            let parsed = EmployeeStatus::from_wire(wire).expect("all wire strings must round-trip");
            assert_eq!(parsed, variant, "round-trip failed for {wire}");
        }
    }

    #[test]
    fn from_wire_rejects_unknown_value() {
        let err =
            EmployeeStatus::from_wire("suspended").expect_err("unknown string must be rejected");
        assert_eq!(err.0, "suspended");
        let msg = err.to_string();
        assert!(msg.contains("unknown employee status"));
        assert!(msg.contains("suspended"));
    }

    #[test]
    fn from_wire_rejects_empty_string() {
        let err = EmployeeStatus::from_wire("").expect_err("empty string must be rejected");
        assert_eq!(err.0, "");
        assert!(err.to_string().contains("unknown employee status"));
    }

    #[test]
    fn display_renders_wire_string() {
        assert_eq!(EmployeeStatus::Active.to_string(), "active");
        assert_eq!(EmployeeStatus::Terminated.to_string(), "terminated");
        assert_eq!(EmployeeStatus::OnLeave.to_string(), "on_leave");
    }

    #[test]
    fn korean_labels_are_non_empty_and_distinct() {
        let labels: Vec<&str> = EmployeeStatus::ALL.iter().map(|v| v.as_korean()).collect();
        let unique: std::collections::HashSet<_> = labels.iter().copied().collect();
        assert_eq!(unique.len(), 3, "Korean labels must be distinct");
        for label in &labels {
            assert!(!label.is_empty(), "Korean label must not be empty");
        }
    }

    #[test]
    fn leave_accrual_eligibility_contract() {
        assert!(EmployeeStatus::Active.is_leave_accrual_eligible());
        assert!(EmployeeStatus::OnLeave.is_leave_accrual_eligible());
        assert!(!EmployeeStatus::Terminated.is_leave_accrual_eligible());
    }

    #[test]
    fn unknown_employee_status_error_display_covers_expected_values() {
        let msg = UnknownEmployeeStatus("xyz".to_string()).to_string();
        assert!(msg.contains("xyz"));
        assert!(msg.contains("active"));
        assert!(msg.contains("terminated"));
        assert!(msg.contains("on_leave"));
    }

    #[test]
    fn wire_strings_match_hr_ddl_enum_values() {
        // Exhaustively verify the CHECK values from migrations/hr/001_hr_schema.sql
        // CREATE TYPE hr.employee_status AS ENUM ('active', 'terminated', 'on_leave')
        let ddl_values = ["active", "terminated", "on_leave"];
        for wire in ddl_values {
            let parsed = EmployeeStatus::from_wire(wire)
                .unwrap_or_else(|_| panic!("DDL wire value {wire:?} must parse"));
            assert_eq!(parsed.as_str(), wire, "as_str must be identity for {wire}");
        }
    }
}
