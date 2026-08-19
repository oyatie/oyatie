//! Pending-relocation retraction: a `pending_relocations` row retracts the absorb
//! claim it names so the destination deriver does not assert a forbidden home.
//!
//! Cases: a pending row retracts the named absorb; a non-pending row does not;
//! a row naming a different capability does not; a malformed `from` is ignored.

use crate::{derive_destination, CapabilityPlacement};

/// A `pending_relocations` row retracts the absorb claim it names. ADR-0615 §2 Q13 rules
/// that `oya/governance` must NOT fold into `compliance`, while the absorb entry stays in
/// place so the membership lint does not orphan the path -- so reading `absorbs_current_dirs`
/// alone makes the deriver assert a destination the authority forbids.
#[test]
fn a_pending_relocation_retracts_the_absorb_it_names() {
    let registry = serde_json::json!({
        "capabilities": [
            {"name": "compliance", "absorbs_current_dirs": ["oya/governance"]},
            {"name": "iam", "absorbs_current_dirs": ["oya/identity"]}
        ],
        "meta_directories": [{"dir": "governance/"}],
        "pending_relocations": [{
            "from": "compliance.absorbs_current_dirs[oya/governance]",
            "to": "governance/ (authority) + ci/gateway/cell (SLOs)",
            "ruled_by": "ADR-0615 §2 Q13",
            "pending_relocation": true
        }]
    });
    let placement = CapabilityPlacement::from_registry_value(&registry);

    // Retracted: no destination at all, so the path lands in `unclassified` rather than
    // being given a home the layout authority forbids.
    assert_eq!(
        derive_destination("oya/governance/slos/a.yaml", &placement, &None),
        None,
        "a retracted absorb must yield no destination, not the forbidden one"
    );
    // Unaffected sibling claims still resolve.
    assert_eq!(
        derive_destination("oya/identity/src/lib.rs", &placement, &None).as_deref(),
        Some("iam/")
    );
}

/// Only rows that actually say `pending_relocation: true` retract. A row describing a
/// completed or proposed relocation must not silently delete a live claim.
#[test]
fn a_relocation_row_not_marked_pending_does_not_retract() {
    let placement = CapabilityPlacement::from_registry_value(&serde_json::json!({
        "capabilities": [{"name": "compliance", "absorbs_current_dirs": ["oya/governance"]}],
        "pending_relocations": [{
            "from": "compliance.absorbs_current_dirs[oya/governance]",
            "pending_relocation": false
        }]
    }));
    assert_eq!(
        derive_destination("oya/governance/x.yaml", &placement, &None).as_deref(),
        Some("compliance/")
    );
}

/// The retraction is guarded on the CLAIMING capability, so a stale or mistyped row cannot
/// delete a different capability's live claim over the same directory.
#[test]
fn a_relocation_naming_a_different_capability_does_not_retract() {
    let placement = CapabilityPlacement::from_registry_value(&serde_json::json!({
        "capabilities": [{"name": "iam", "absorbs_current_dirs": ["oya/governance"]}],
        "pending_relocations": [{
            "from": "compliance.absorbs_current_dirs[oya/governance]",
            "pending_relocation": true
        }]
    }));
    assert_eq!(
        derive_destination("oya/governance/x.yaml", &placement, &None).as_deref(),
        Some("iam/"),
        "a row naming compliance must not retract iam's claim"
    );
}

/// A malformed `from` is ignored rather than panicking or retracting something adjacent.
#[test]
fn malformed_pending_relocation_rows_are_ignored() {
    for from in [
        "compliance",
        "compliance.absorbs_current_dirs[oya/governance",
        "absorbs_current_dirs[oya/governance]",
        "",
    ] {
        let placement = CapabilityPlacement::from_registry_value(&serde_json::json!({
            "capabilities": [
                {"name": "compliance", "absorbs_current_dirs": ["oya/governance"]}
            ],
            "pending_relocations": [{"from": from, "pending_relocation": true}]
        }));
        assert_eq!(
            derive_destination("oya/governance/x.yaml", &placement, &None).as_deref(),
            Some("compliance/"),
            "malformed from={from:?} must not retract"
        );
    }
}
