//! Architecture-plane taxonomy for the M02 plane-verification phase
//! (P21-architecture-planes-green, IP-001-plane-verification).
//!
//! Defines the closed 9-plane enum from ADR-0224..ADR-0231 and a
//! `PlaneVerdict` that records the L4/L5 proof-ladder status for each
//! plane.  Pure `std`-only; no I/O, no panics outside `cfg(test)`.
//!
//! The 9 planes are:
//!   Data (ADR-0224) | Identity (ADR-0225) | Policy (ADR-0226) |
//!   Audit (ADR-0227) | Integration (ADR-0228) | Observability (ADR-0229) |
//!   Security (ADR-0230) | Scalability (ADR-0231) | Reliability (ADR-0231)
//!
//! Naming justification:
//! - `ArchitecturePlane` — noun phrase matching the phase name
//!   `P21-architecture-planes-green`; `Architecture` + `Plane` follows
//!   the `NodeKind` / `EdgeKind` naming convention in the parent kernel.
//! - `PlaneVerdict` — `Verdict` matches the proof-ladder terminology used
//!   in ADR-0223 (Proof Ladder); `Plane` prefix scopes it to this module.
//! - `ProofLevel` — `Level` is the noun ADR-0223 uses ("L4 / L5 level");
//!   `Proof` qualifies it to distinguish from other potential `Level` types.

use std::fmt;

/// Closed enum of the 9 M02 architecture planes.
///
/// The discriminants are assigned in ADR order (ADR-0224..ADR-0231).
/// Scalability and Reliability both cite ADR-0231; `Scalability` is
/// assigned the lower discriminant since it appears first in the ADR body.
///
/// `non_exhaustive` is intentionally NOT applied here: the 9-plane set is
/// defined as closed by ADR-0224..ADR-0231.  Any future plane requires a
/// new ADR and a version bump.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArchitecturePlane {
    /// Plane 1 — tenant-bound tables with `tenant_id` + RLS (ADR-0224).
    Data,
    /// Plane 2 — Tenant ≠ Org ≠ User ≠ Person ≠ Employee separation (ADR-0225).
    Identity,
    /// Plane 3 — Cedar engine + per-tenant rule packs (ADR-0226).
    Policy,
    /// Plane 4 — Merkle-sealed Ed25519 audit segments (ADR-0227).
    Audit,
    /// Plane 5 — Workflow + Ontology as the only cross-product adapters (ADR-0228).
    Integration,
    /// Plane 6 — OTel traces + metrics on all µservice boundaries (ADR-0229).
    Observability,
    /// Plane 7 — Secrets via `oya-secrets-kernel`; no plaintext credentials (ADR-0230).
    Security,
    /// Plane 8 — Statelessness + shardability + cell architecture (ADR-0231).
    Scalability,
    /// Plane 9 — Outbox pattern + RTO/RPO documented (ADR-0231).
    Reliability,
}

impl ArchitecturePlane {
    /// All 9 planes in ADR order. Callers that must iterate all planes
    /// should use this rather than hard-coding their own array.
    pub const ALL: [ArchitecturePlane; 9] = [
        ArchitecturePlane::Data,
        ArchitecturePlane::Identity,
        ArchitecturePlane::Policy,
        ArchitecturePlane::Audit,
        ArchitecturePlane::Integration,
        ArchitecturePlane::Observability,
        ArchitecturePlane::Security,
        ArchitecturePlane::Scalability,
        ArchitecturePlane::Reliability,
    ];

    /// Short kebab-case identifier, consistent with the gate validate
    /// `plane-class` lane naming convention and the evidence artifact
    /// section anchors in `docs/architecture/plane-verification-M02.md`.
    pub fn id(self) -> &'static str {
        match self {
            ArchitecturePlane::Data => "data",
            ArchitecturePlane::Identity => "identity",
            ArchitecturePlane::Policy => "policy",
            ArchitecturePlane::Audit => "audit",
            ArchitecturePlane::Integration => "integration",
            ArchitecturePlane::Observability => "observability",
            ArchitecturePlane::Security => "security",
            ArchitecturePlane::Scalability => "scalability",
            ArchitecturePlane::Reliability => "reliability",
        }
    }

    /// Human-readable label used in the evidence artifact headings.
    pub fn label(self) -> &'static str {
        match self {
            ArchitecturePlane::Data => "Data Plane",
            ArchitecturePlane::Identity => "Identity Plane",
            ArchitecturePlane::Policy => "Policy Plane",
            ArchitecturePlane::Audit => "Audit Plane",
            ArchitecturePlane::Integration => "Integration Plane",
            ArchitecturePlane::Observability => "Observability Plane",
            ArchitecturePlane::Security => "Security Plane",
            ArchitecturePlane::Scalability => "Scalability Plane",
            ArchitecturePlane::Reliability => "Reliability Plane",
        }
    }

    /// Primary governing ADR number (decimal). Where two planes share an
    /// ADR (Scalability + Reliability → ADR-0231) the shared number is
    /// returned for both.
    pub fn governing_adr(self) -> u16 {
        match self {
            ArchitecturePlane::Data => 224,
            ArchitecturePlane::Identity => 225,
            ArchitecturePlane::Policy => 226,
            ArchitecturePlane::Audit => 227,
            ArchitecturePlane::Integration => 228,
            ArchitecturePlane::Observability => 229,
            ArchitecturePlane::Security => 230,
            ArchitecturePlane::Scalability => 231,
            ArchitecturePlane::Reliability => 231,
        }
    }

    /// Parse from the short kebab-case `id()` representation.
    ///
    /// # Errors
    ///
    /// Returns `Err(UnknownPlane)` when the input does not match any of
    /// the 9 canonical plane ids.
    pub fn from_id(s: &str) -> Result<Self, UnknownPlane> {
        match s {
            "data" => Ok(ArchitecturePlane::Data),
            "identity" => Ok(ArchitecturePlane::Identity),
            "policy" => Ok(ArchitecturePlane::Policy),
            "audit" => Ok(ArchitecturePlane::Audit),
            "integration" => Ok(ArchitecturePlane::Integration),
            "observability" => Ok(ArchitecturePlane::Observability),
            "security" => Ok(ArchitecturePlane::Security),
            "scalability" => Ok(ArchitecturePlane::Scalability),
            "reliability" => Ok(ArchitecturePlane::Reliability),
            other => Err(UnknownPlane {
                input: other.into(),
            }),
        }
    }
}

impl fmt::Display for ArchitecturePlane {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

/// Error returned when an unrecognised plane id string is parsed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownPlane {
    /// The unrecognised input string.  `data_class: INTERNAL_ONLY`
    pub input: String,
}

impl fmt::Display for UnknownPlane {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unknown architecture plane id `{}`; expected one of: {}",
            self.input,
            ArchitecturePlane::ALL
                .iter()
                .map(|p| p.id())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl std::error::Error for UnknownPlane {}

/// Proof-ladder level per ADR-0223.
///
/// Only L4 and L5 are modelled here because P21 only asserts L4/L5
/// verdicts.  Lower levels (L0..L3) are pre-gate and not tracked in the
/// plane-verification evidence artifact.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProofLevel {
    /// L4 — evidence produced by automated CI checks; reproducible.
    L4,
    /// L5 — L4 + live-system verification (deployment, integration test).
    L5,
}

impl ProofLevel {
    /// Short label matching the evidence artifact notation.
    pub fn label(self) -> &'static str {
        match self {
            ProofLevel::L4 => "L4",
            ProofLevel::L5 => "L5",
        }
    }
}

impl fmt::Display for ProofLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

/// Recorded verdict for a single architecture plane in the M02
/// plane-verification evidence artifact (`plane-verification-M02.md`).
///
/// A `PlaneVerdict` captures whether the plane reached L4 in the current
/// milestone and whether L5 has been verified.  The optional
/// `l5_deferred_to` field records the milestone slug where L5 is
/// planned (e.g. `"M03"`, `"M04"`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaneVerdict {
    /// Which plane this verdict covers.  `data_class: INTERNAL_ONLY`
    pub plane: ArchitecturePlane,
    /// Highest proof level reached at assessment time.  `data_class: INTERNAL_ONLY`
    pub reached: ProofLevel,
    /// Milestone slug where the next proof level is deferred, if any.
    /// `None` means no outstanding deferral.  `data_class: INTERNAL_ONLY`
    pub l5_deferred_to: Option<&'static str>,
}

impl PlaneVerdict {
    /// Returns `true` when L4 has been reached (always true for any
    /// `PlaneVerdict` since L4 is the minimum bar for the evidence
    /// artifact).
    #[must_use]
    pub fn is_l4(&self) -> bool {
        self.reached >= ProofLevel::L4
    }

    /// Returns `true` when L5 has been reached.
    #[must_use]
    pub fn is_l5(&self) -> bool {
        self.reached >= ProofLevel::L5
    }

    /// Returns `true` when L5 is pending in a future milestone.
    #[must_use]
    pub fn has_l5_deferral(&self) -> bool {
        self.l5_deferred_to.is_some()
    }
}

impl fmt::Display for PlaneVerdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} — {} ✓",
            self.plane.label(),
            self.reached.label()
        )?;
        if let Some(milestone) = self.l5_deferred_to {
            write!(formatter, " (L5 deferred to {milestone})")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
    // `panic!()` to assert invariants under the `cfg(test)` exemption.

    #[test]
    fn all_planes_count_is_nine() {
        assert_eq!(ArchitecturePlane::ALL.len(), 9);
    }

    #[test]
    fn plane_ids_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for plane in ArchitecturePlane::ALL {
            let id = plane.id();
            assert!(seen.insert(id), "duplicate plane id: {id}");
        }
    }

    #[test]
    fn plane_labels_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for plane in ArchitecturePlane::ALL {
            let label = plane.label();
            assert!(seen.insert(label), "duplicate plane label: {label}");
        }
    }

    #[test]
    fn from_id_round_trips_all_planes() {
        for plane in ArchitecturePlane::ALL {
            let parsed = ArchitecturePlane::from_id(plane.id()).unwrap();
            assert_eq!(parsed, plane);
        }
    }

    #[test]
    fn from_id_rejects_unknown() {
        let result = ArchitecturePlane::from_id("unknown-plane");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.input, "unknown-plane");
        // Display must mention the bad input and enumerate candidates.
        let msg = err.to_string();
        assert!(msg.contains("unknown-plane"), "msg={msg}");
        assert!(msg.contains("data"), "msg={msg}");
    }

    #[test]
    fn governing_adr_data_plane_is_0224() {
        assert_eq!(ArchitecturePlane::Data.governing_adr(), 224);
    }

    #[test]
    fn governing_adr_scalability_and_reliability_share_0231() {
        assert_eq!(ArchitecturePlane::Scalability.governing_adr(), 231);
        assert_eq!(ArchitecturePlane::Reliability.governing_adr(), 231);
    }

    #[test]
    fn governing_adrs_are_in_224_to_231_range() {
        for plane in ArchitecturePlane::ALL {
            let adr = plane.governing_adr();
            assert!(
                (224..=231).contains(&adr),
                "plane {}: ADR-{adr} out of expected range 224..=231",
                plane.id()
            );
        }
    }

    #[test]
    fn display_contains_label() {
        for plane in ArchitecturePlane::ALL {
            let s = plane.to_string();
            assert_eq!(s, plane.label());
        }
    }

    #[test]
    fn proof_level_ordering_l4_lt_l5() {
        assert!(ProofLevel::L4 < ProofLevel::L5);
    }

    #[test]
    fn proof_level_labels_round_trip() {
        assert_eq!(ProofLevel::L4.label(), "L4");
        assert_eq!(ProofLevel::L5.label(), "L5");
    }

    #[test]
    fn plane_verdict_is_l4_always_true() {
        let v = PlaneVerdict {
            plane: ArchitecturePlane::Data,
            reached: ProofLevel::L4,
            l5_deferred_to: Some("M03"),
        };
        assert!(v.is_l4());
        assert!(!v.is_l5());
        assert!(v.has_l5_deferral());
    }

    #[test]
    fn plane_verdict_is_l5_when_l5_reached() {
        let v = PlaneVerdict {
            plane: ArchitecturePlane::Identity,
            reached: ProofLevel::L5,
            l5_deferred_to: None,
        };
        assert!(v.is_l4());
        assert!(v.is_l5());
        assert!(!v.has_l5_deferral());
    }

    #[test]
    fn plane_verdict_display_includes_plane_and_level() {
        let v = PlaneVerdict {
            plane: ArchitecturePlane::Policy,
            reached: ProofLevel::L4,
            l5_deferred_to: None,
        };
        let s = v.to_string();
        assert!(s.contains("Policy Plane"), "display: {s}");
        assert!(s.contains("L4"), "display: {s}");
    }

    #[test]
    fn plane_verdict_display_includes_deferral_milestone() {
        let v = PlaneVerdict {
            plane: ArchitecturePlane::Security,
            reached: ProofLevel::L4,
            l5_deferred_to: Some("M03"),
        };
        let s = v.to_string();
        assert!(s.contains("M03"), "display: {s}");
    }

    #[test]
    fn all_nine_plane_verdicts_are_constructible() {
        // Smoke-test that all 9 planes can be used as PlaneVerdict.plane.
        let verdicts: Vec<PlaneVerdict> = ArchitecturePlane::ALL
            .iter()
            .map(|&plane| PlaneVerdict {
                plane,
                reached: ProofLevel::L4,
                l5_deferred_to: None,
            })
            .collect();
        assert_eq!(verdicts.len(), 9);
        assert!(verdicts.iter().all(|v| v.is_l4()));
        assert!(verdicts.iter().all(|v| !v.is_l5()));
    }
}
