//! Backup-retention discipline check (ADR-0197 D-5).
//!
//! # Why this crate exists
//!
//! ADR-0197 D-5 establishes per-regulatory-pack retention floors:
//!
//! | Pack               | Floor                                |
//! |--------------------|--------------------------------------|
//! | generic            | 7 y annual                           |
//! | pack-primary       | 5 y annual                           |
//! | pack-secondary     | 7 y annual                           |
//! | pack-health        | 6 y annual                           |
//! | pack-financial     | 7 y annual                           |
//! | pack-public-sector | 7 y annual + replica policy evidence |
//!
//! This crate scans per-µservice backup declarations and reports
//! retention that fails to meet the pack's floor. Advisory mode this
//! batch; strict promotion follows when the per-µservice backlog
//! reaches zero.
//!
//! # Naming justification
//!
//! `oya-check-backup-retention-discipline` follows BNF v4.1:
//! `oya-check-<topic:backup-retention-discipline>`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use shared_backup_kernel::{RegulatoryPack, WorkloadClass};
use std::fmt;

/// A per-µservice backup retention declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionDeclaration {
    pub microservice: String,
    pub source_path: String,
    pub regulatory_pack: RegulatoryPack,
    pub workload_class: WorkloadClass,
    pub declared_retention_days: u32,
    /// Optional override authority (e.g. an ADR slug allowing a shorter
    /// retention for an explicit reason).
    pub override_adr: Option<String>,
}

/// One finding emitted by the advisory check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionFinding {
    pub microservice: String,
    pub source_path: String,
    pub pack: &'static str,
    pub declared_days: u32,
    pub floor_days: u32,
    pub severity: Severity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Severity {
    /// Declared retention meets the floor.
    Ok,
    /// Declared retention below floor; advisory.
    Advisory,
    /// Declared retention below floor AND no override ADR; would block
    /// strict mode.
    Blocking,
}

impl fmt::Display for RetentionFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}, pack={}): declared {}d vs floor {}d [{:?}]",
            self.microservice,
            self.source_path,
            self.pack,
            self.declared_days,
            self.floor_days,
            self.severity
        )
    }
}

/// Report emitted in advisory mode.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct RetentionReport {
    pub declarations_scanned: usize,
    pub findings_ok: usize,
    pub findings_advisory: usize,
    pub findings_blocking: usize,
    pub findings: Vec<RetentionFinding>,
}

impl RetentionReport {
    /// True iff there are no advisory or blocking findings (strict mode
    /// would pass).
    #[must_use]
    pub fn would_pass_strict(&self) -> bool {
        self.findings_advisory == 0 && self.findings_blocking == 0
    }
}

/// Validate declarations in advisory mode.
#[must_use]
pub fn validate_advisory<I>(declarations: I) -> RetentionReport
where
    I: IntoIterator<Item = RetentionDeclaration>,
{
    let mut report = RetentionReport::default();
    for d in declarations {
        report.declarations_scanned += 1;
        let floor = d.regulatory_pack.retention_floor_days();
        let pack_wire = d.regulatory_pack.wire_name();
        let severity = if d.declared_retention_days >= floor {
            Severity::Ok
        } else if d.override_adr.is_some() {
            Severity::Advisory
        } else {
            Severity::Blocking
        };
        match severity {
            Severity::Ok => report.findings_ok += 1,
            Severity::Advisory => report.findings_advisory += 1,
            Severity::Blocking => report.findings_blocking += 1,
        }
        if severity != Severity::Ok {
            report.findings.push(RetentionFinding {
                microservice: d.microservice,
                source_path: d.source_path,
                pack: pack_wire,
                declared_days: d.declared_retention_days,
                floor_days: floor,
                severity,
            });
        }
    }
    report
}

/// Validate in strict mode — panics until promoted out of advisory.
pub fn validate_strict<I>(_declarations: I) -> !
where
    I: IntoIterator<Item = RetentionDeclaration>,
{
    unimplemented!(
        "strict mode pending fleet migration; tracked in registry/placeholder-debt/adr-follow-ups.yaml#adr-0197-retention-strict"
    )
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn decl(
        microservice: &str,
        pack: RegulatoryPack,
        days: u32,
        override_adr: Option<&str>,
    ) -> RetentionDeclaration {
        RetentionDeclaration {
            microservice: microservice.to_string(),
            source_path: format!("microservices/{microservice}/manifest.json"),
            regulatory_pack: pack,
            workload_class: WorkloadClass::App,
            declared_retention_days: days,
            override_adr: override_adr.map(String::from),
        }
    }

    #[test]
    fn meeting_generic_floor_is_ok() {
        let r = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::Generic,
            2_555,
            None,
        )));
        assert_eq!(r.findings_ok, 1);
        assert_eq!(r.findings_blocking, 0);
        assert!(r.would_pass_strict());
    }

    #[test]
    fn below_generic_floor_no_override_is_blocking() {
        let r = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::Generic,
            90,
            None,
        )));
        assert_eq!(r.findings_blocking, 1);
        assert_eq!(r.findings[0].declared_days, 90);
        assert_eq!(r.findings[0].floor_days, 2_555);
        assert!(!r.would_pass_strict());
    }

    #[test]
    fn below_floor_with_override_is_advisory_only() {
        let r = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::Generic,
            90,
            Some("ADR-XXXX"),
        )));
        assert_eq!(r.findings_blocking, 0);
        assert_eq!(r.findings_advisory, 1);
        assert!(!r.would_pass_strict());
    }

    #[test]
    fn pack_primary_floor_is_5y() {
        let r = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::PackPrimary,
            1_825,
            None,
        )));
        assert_eq!(r.findings_ok, 1);
        let r2 = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::PackPrimary,
            1_824,
            None,
        )));
        assert_eq!(r2.findings_blocking, 1);
    }

    #[test]
    fn pack_health_floor_is_6y() {
        let r = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::PackHealth,
            2_190,
            None,
        )));
        assert_eq!(r.findings_ok, 1);
        let r2 = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::PackHealth,
            1_825,
            None,
        )));
        assert_eq!(r2.findings_blocking, 1);
    }

    #[test]
    fn pack_secondary_floor_is_7y() {
        let r = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::PackSecondary,
            2_555,
            None,
        )));
        assert_eq!(r.findings_ok, 1);
        let r2 = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::PackSecondary,
            2_190,
            None,
        )));
        assert_eq!(r2.findings_blocking, 1);
    }

    #[test]
    fn mixed_declarations_aggregate_correctly() {
        let ds = vec![
            decl("a", RegulatoryPack::Generic, 2_555, None), // ok
            decl("b", RegulatoryPack::PackPrimary, 1_825, None), // ok
            decl("c", RegulatoryPack::PackHealth, 1_000, None), // blocking
            decl("d", RegulatoryPack::PackSecondary, 100, Some("ADR-OVR-1")), // advisory
        ];
        let r = validate_advisory(ds);
        assert_eq!(r.declarations_scanned, 4);
        assert_eq!(r.findings_ok, 2);
        assert_eq!(r.findings_advisory, 1);
        assert_eq!(r.findings_blocking, 1);
        assert!(!r.would_pass_strict());
    }

    #[test]
    fn empty_input_passes_strict() {
        let r = validate_advisory(std::iter::empty::<RetentionDeclaration>());
        assert_eq!(r.declarations_scanned, 0);
        assert!(r.would_pass_strict());
    }

    #[test]
    fn finding_display_carries_pack_and_floor() {
        let r = validate_advisory(std::iter::once(decl(
            "drive",
            RegulatoryPack::PackPrimary,
            1_000,
            None,
        )));
        let s = format!("{}", r.findings[0]);
        assert!(s.contains("pack-primary"));
        assert!(s.contains("1825"));
        assert!(s.contains("drive"));
    }
}
