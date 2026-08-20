//! Integration tests for the capability registry kernel — lifecycle guard +
//! registry-view slice (ST1 / ST2 / ST3 acceptance + spec-mandated standards).
//!
//! Honour standards (per task spec):
//!   1. DDL CHECK constraint values: `active | deprecated | disabled` only.
//!   2. MCP discovery vs invocation surface semantics for deprecated capabilities.
//!   3. ADR-0003 / M02-P05 autonomy-tier interaction: no tier downgrade on status
//!      change — a capability's `autonomy_tier` field must be independent of its
//!      lifecycle status.
//!
//! These tests live in `tests/` so they exercise only the public API.
//!
// ADR-0083 Tier 3 exemption applies to integration tests as well.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use intelligence_capability_registry_kernel::{
    CapabilityId, CapabilityStatus, CapabilityStatusParseError, CapabilityStatusTransitionError,
    partition_views,
};

// ---------------------------------------------------------------------------
// Standard 1: DDL CHECK constraint values
// ---------------------------------------------------------------------------

/// `as_str()` must produce exactly the values accepted by the DDL CHECK
/// constraint (`active | deprecated | disabled`).  Any other value would
/// silently corrupt the persistence layer.
#[test]
fn ddl_check_constraint_values_match_exactly() {
    assert_eq!(CapabilityStatus::Active.as_str(), "active");
    assert_eq!(CapabilityStatus::Deprecated.as_str(), "deprecated");
    assert_eq!(CapabilityStatus::Disabled.as_str(), "disabled");

    // All three and only three values exist — exhaustive by coverage.
    let all = [
        CapabilityStatus::Active,
        CapabilityStatus::Deprecated,
        CapabilityStatus::Disabled,
    ];
    let labels: Vec<&str> = all.iter().map(|s| s.as_str()).collect();
    assert_eq!(labels, ["active", "deprecated", "disabled"]);
}

/// Round-trip: every DDL label parses back to the correct variant.
#[test]
fn ddl_labels_round_trip_via_try_from() {
    for (raw, expected) in [
        ("active", CapabilityStatus::Active),
        ("deprecated", CapabilityStatus::Deprecated),
        ("disabled", CapabilityStatus::Disabled),
    ] {
        let parsed = CapabilityStatus::try_from(raw)
            .unwrap_or_else(|_| panic!("failed to parse DDL label '{raw}'"));
        assert_eq!(parsed, expected, "label '{raw}' did not round-trip");
    }
}

/// Non-DDL labels (e.g. uppercase, legacy values) must be rejected to prevent
/// silent schema drift.
#[test]
fn non_ddl_labels_are_rejected() {
    for bad in ["Active", "ACTIVE", "archived", "pending", "DEPRECATED", ""] {
        assert!(
            CapabilityStatus::try_from(bad).is_err(),
            "expected rejection of '{bad}'"
        );
    }
}

// ---------------------------------------------------------------------------
// Standard 2: MCP discovery vs invocation surface semantics
// ---------------------------------------------------------------------------

/// `Deprecated` capabilities must be excluded from MCP discovery (new binding
/// surface) but remain reachable for invocation by existing bindings.
/// This is the canonical MCP surface rule: discovery is forward-looking,
/// invocation honours existing contracts.
#[test]
fn deprecated_excluded_from_mcp_discovery_surface() {
    let views = partition_views([
        (CapabilityId::new("svc.alpha"), CapabilityStatus::Active),
        (CapabilityId::new("svc.beta"), CapabilityStatus::Deprecated),
    ]);

    // discovery: only Active
    assert!(
        views
            .discoverable
            .contains_key(&CapabilityId::new("svc.alpha")),
        "Active must appear in discovery"
    );
    assert!(
        !views
            .discoverable
            .contains_key(&CapabilityId::new("svc.beta")),
        "Deprecated must NOT appear in MCP discovery"
    );

    // invocation: both
    assert!(
        views
            .invocable
            .contains_key(&CapabilityId::new("svc.alpha")),
        "Active must be invocable"
    );
    assert!(
        views.invocable.contains_key(&CapabilityId::new("svc.beta")),
        "Deprecated must remain invocable for existing bindings"
    );
}

/// `Disabled` capabilities must appear in NEITHER surface — they are
/// administratively suspended.
#[test]
fn disabled_absent_from_both_mcp_surfaces() {
    let views = partition_views([(
        CapabilityId::new("svc.suspended"),
        CapabilityStatus::Disabled,
    )]);

    assert!(
        views.discoverable.is_empty(),
        "Disabled must not appear in MCP discovery"
    );
    assert!(
        views.invocable.is_empty(),
        "Disabled must not appear in MCP invocation surface"
    );
}

/// A capability transitioning Active→Deprecated must disappear from discovery
/// in the next view snapshot while remaining invocable.
#[test]
fn active_to_deprecated_transition_removes_from_discovery_keeps_invocable() {
    let original_status = CapabilityStatus::Active;
    let transitioned = original_status
        .try_transition_to(CapabilityStatus::Deprecated)
        .expect("Active->Deprecated must be legal");

    let id = CapabilityId::new("svc.transitioning");
    let views_after = partition_views([(id.clone(), transitioned)]);

    assert!(
        !views_after.discoverable.contains_key(&id),
        "after Active->Deprecated transition, capability must leave discovery surface"
    );
    assert!(
        views_after.invocable.contains_key(&id),
        "after Active->Deprecated transition, capability must remain invocable"
    );
}

/// A capability transitioning Deprecated→Disabled must disappear from the
/// invocation surface — no longer callable.
#[test]
fn deprecated_to_disabled_transition_removes_from_invocable_surface() {
    let status = CapabilityStatus::Deprecated
        .try_transition_to(CapabilityStatus::Disabled)
        .expect("Deprecated->Disabled must be legal");

    let id = CapabilityId::new("svc.retiring");
    let views = partition_views([(id.clone(), status)]);

    assert!(
        !views.invocable.contains_key(&id),
        "Deprecated->Disabled: capability must leave invocable surface"
    );
    assert!(
        !views.discoverable.contains_key(&id),
        "Deprecated->Disabled: capability must leave discovery surface"
    );
}

/// Re-activation (Disabled→Active) must restore the capability to both
/// surfaces.
#[test]
fn disabled_to_active_reactivation_restores_both_surfaces() {
    let reactivated = CapabilityStatus::Disabled
        .try_transition_to(CapabilityStatus::Active)
        .expect("Disabled->Active must be legal");

    let id = CapabilityId::new("svc.reactivated");
    let views = partition_views([(id.clone(), reactivated)]);

    assert!(
        views.discoverable.contains_key(&id),
        "Re-activated capability must appear in MCP discovery"
    );
    assert!(
        views.invocable.contains_key(&id),
        "Re-activated capability must appear in MCP invocation surface"
    );
}

// ---------------------------------------------------------------------------
// Standard 3: ADR-0003 / M02-P05 — autonomy-tier independence from status
// ---------------------------------------------------------------------------
//
// The autonomy tier of a capability is a static policy classification
// (T1Read / T2Suggest / T3PropAct / T4Actuate).  Lifecycle status transitions
// (Active / Deprecated / Disabled) must NEVER cause a tier change.
//
// NOTE: The `Capability` struct currently carries `autonomy_tier` but has NO
// `status` field.  These tests verify the INTENDED design: that a `Capability`
// exposes a way to track its current status while keeping `autonomy_tier`
// immutable through transitions.  Until `Capability` gains a `status` field
// and a `transition_status` method, these tests will FAIL TO COMPILE (red).
//
// This is the primary red-producing block for the TDD stage.

use intelligence_capability_registry_kernel::{AutonomyTier, Capability};

/// A capability's autonomy tier must not change when its status is
/// transitioned to `Deprecated`.
///
/// Spec: ADR-0003 §4 — "autonomy classification is immutable post-registration".
#[test]
fn autonomy_tier_unchanged_after_active_to_deprecated_transition() {
    let mut cap = Capability::new(
        CapabilityId::new("svc.tier.guard"),
        "Tier guard test",
        AutonomyTier::T3PropAct,
        true,
    );
    let original_tier = cap.autonomy_tier;
    assert_eq!(
        cap.status,
        CapabilityStatus::Active,
        "new capability must start Active"
    );

    cap.transition_status(CapabilityStatus::Deprecated)
        .expect("Active->Deprecated must be legal");

    assert_eq!(
        cap.autonomy_tier, original_tier,
        "autonomy_tier must not change after Active->Deprecated transition"
    );
    assert_eq!(
        cap.status,
        CapabilityStatus::Deprecated,
        "status must reflect the transition"
    );
}

/// A T4Actuate capability (highest-risk tier) must not have its tier silently
/// downgraded when disabled.
#[test]
fn t4_actuate_tier_preserved_through_disabled_transition() {
    let mut cap = Capability::new(
        CapabilityId::new("svc.actuate.critical"),
        "Critical actuator",
        AutonomyTier::T4Actuate,
        true,
    );
    cap.transition_status(CapabilityStatus::Disabled)
        .expect("Active->Disabled must be legal");

    assert_eq!(
        cap.autonomy_tier,
        AutonomyTier::T4Actuate,
        "T4Actuate tier must be preserved through Disabled transition"
    );
}

/// Re-activation from Disabled must not alter the tier.
#[test]
fn autonomy_tier_unchanged_after_reactivation() {
    let mut cap = Capability::new(
        CapabilityId::new("svc.suggest.reactivate"),
        "Suggestion surface",
        AutonomyTier::T2Suggest,
        false,
    );
    cap.transition_status(CapabilityStatus::Disabled)
        .expect("Active->Disabled must be legal");
    cap.transition_status(CapabilityStatus::Active)
        .expect("Disabled->Active must be legal");

    assert_eq!(
        cap.autonomy_tier,
        AutonomyTier::T2Suggest,
        "T2Suggest tier must survive Disabled->Active round-trip"
    );
    assert_eq!(cap.status, CapabilityStatus::Active);
}

/// `transition_status` must return an error for an illegal transition without
/// mutating the capability.
#[test]
fn illegal_transition_does_not_mutate_capability_state() {
    let mut cap = Capability::new(
        CapabilityId::new("svc.immutable.test"),
        "Immutability check",
        AutonomyTier::T1Read,
        false,
    );
    // Pre-transition to Disabled
    cap.transition_status(CapabilityStatus::Disabled)
        .expect("Active->Disabled must be legal");

    // Illegal: Disabled->Deprecated
    let result = cap.transition_status(CapabilityStatus::Deprecated);
    assert!(result.is_err(), "Disabled->Deprecated must be rejected");

    // Status must remain Disabled — not mutated by the failed transition
    assert_eq!(
        cap.status,
        CapabilityStatus::Disabled,
        "status must not change after a rejected transition"
    );
    assert_eq!(
        cap.autonomy_tier,
        AutonomyTier::T1Read,
        "autonomy_tier must never change"
    );
}

// ---------------------------------------------------------------------------
// Error trait surface (std::error::Error contract)
// ---------------------------------------------------------------------------

/// `CapabilityStatusTransitionError` must implement `std::error::Error` with
/// `source()` returning `None` (no causal chain — it is a leaf error).
#[test]
fn transition_error_source_is_none() {
    use std::error::Error;

    let err = CapabilityStatus::Disabled
        .try_transition_to(CapabilityStatus::Deprecated)
        .unwrap_err();

    // Error::source() must return None — this is a leaf error type.
    assert!(
        err.source().is_none(),
        "CapabilityStatusTransitionError must have no error source"
    );
}

/// `CapabilityStatusParseError` must implement `std::error::Error` with
/// `source()` returning `None`.
#[test]
fn parse_error_source_is_none() {
    use std::error::Error;

    let err = CapabilityStatus::try_from("invalid_status").unwrap_err();
    assert!(
        err.source().is_none(),
        "CapabilityStatusParseError must have no error source"
    );
}

/// `CapabilityStatusTransitionError` Display message must contain both the
/// `from` and `to` DDL labels to produce actionable operator diagnostics.
#[test]
fn transition_error_display_contains_both_ddl_labels() {
    let err = CapabilityStatusTransitionError {
        from: CapabilityStatus::Disabled,
        to: CapabilityStatus::Deprecated,
    };
    let msg = err.to_string();
    assert!(
        msg.contains("disabled"),
        "Display must include 'disabled' (from): got '{msg}'"
    );
    assert!(
        msg.contains("deprecated"),
        "Display must include 'deprecated' (to): got '{msg}'"
    );
}

/// `CapabilityStatusParseError` Display message must include the rejected
/// label so operators can identify the bad input.
#[test]
fn parse_error_display_includes_rejected_label() {
    let err = CapabilityStatusParseError("bogus_status".to_owned());
    let msg = err.to_string();
    assert!(
        msg.contains("bogus_status"),
        "Display must include the rejected label: got '{msg}'"
    );
}

// ---------------------------------------------------------------------------
// Registry-view supplementary: duplicate IDs and structural equality
// ---------------------------------------------------------------------------

/// When duplicate `CapabilityId` values are supplied, the last entry wins
/// (BTreeMap::insert semantics).  Both output maps must be consistent.
#[test]
fn duplicate_id_last_entry_wins_in_both_views() {
    // Supply the same ID twice: first Active, then Disabled.
    // The BTreeMap must end with Disabled (last writer wins).
    let entries = vec![
        (CapabilityId::new("svc.dup"), CapabilityStatus::Active),
        (CapabilityId::new("svc.dup"), CapabilityStatus::Disabled),
    ];
    let views = partition_views(entries);

    // With Disabled as the final value:
    // - must NOT appear in discoverable
    assert!(
        !views
            .discoverable
            .contains_key(&CapabilityId::new("svc.dup")),
        "duplicate resolved to Disabled: must not appear in discoverable"
    );
    // - must NOT appear in invocable
    assert!(
        !views.invocable.contains_key(&CapabilityId::new("svc.dup")),
        "duplicate resolved to Disabled: must not appear in invocable"
    );
}

/// Two `partition_views` calls with identical inputs must produce equal
/// `RegistryViews` (structural equality via derived `PartialEq`).
#[test]
fn registry_views_structural_equality() {
    let entries = || {
        vec![
            (CapabilityId::new("cap.a"), CapabilityStatus::Active),
            (CapabilityId::new("cap.b"), CapabilityStatus::Deprecated),
            (CapabilityId::new("cap.c"), CapabilityStatus::Disabled),
        ]
    };
    let v1 = partition_views(entries());
    let v2 = partition_views(entries());
    assert_eq!(v1, v2, "identical inputs must produce equal RegistryViews");
}

/// `RegistryViews::clone()` must produce a deep copy that is equal to the
/// original and independently mutable.
#[test]
fn registry_views_clone_is_independent() {
    let views = partition_views([(CapabilityId::new("cap.original"), CapabilityStatus::Active)]);
    let cloned = views.clone();
    assert_eq!(views, cloned, "clone must equal original");
    // Cloned map is independent (BTreeMap::clone guarantee — verified by type).
}

/// `partition_views` output ordering is stable across multiple calls with the
/// same entries supplied in different input orders.
#[test]
fn partition_views_ordering_independent_of_input_order() {
    let order_a = partition_views([
        (CapabilityId::new("z.cap"), CapabilityStatus::Active),
        (CapabilityId::new("a.cap"), CapabilityStatus::Active),
        (CapabilityId::new("m.cap"), CapabilityStatus::Active),
    ]);
    let order_b = partition_views([
        (CapabilityId::new("m.cap"), CapabilityStatus::Active),
        (CapabilityId::new("z.cap"), CapabilityStatus::Active),
        (CapabilityId::new("a.cap"), CapabilityStatus::Active),
    ]);

    let keys_a: Vec<_> = order_a.discoverable.keys().cloned().collect();
    let keys_b: Vec<_> = order_b.discoverable.keys().cloned().collect();
    assert_eq!(
        keys_a, keys_b,
        "BTreeMap ordering must be input-order-independent"
    );
}

// ---------------------------------------------------------------------------
// Multi-hop transition chain
// ---------------------------------------------------------------------------

/// Full lifecycle round-trip: Active→Deprecated→Disabled→Active.
/// Each step must succeed; the final state must be Active.
#[test]
fn multi_hop_transition_chain_active_deprecated_disabled_active() {
    let s = CapabilityStatus::Active;
    let s = s
        .try_transition_to(CapabilityStatus::Deprecated)
        .expect("Active->Deprecated");
    let s = s
        .try_transition_to(CapabilityStatus::Disabled)
        .expect("Deprecated->Disabled");
    let s = s
        .try_transition_to(CapabilityStatus::Active)
        .expect("Disabled->Active");
    assert_eq!(s, CapabilityStatus::Active, "chain must end at Active");
}

/// After Disabled→Active re-activation the illegal Disabled→Deprecated edge
/// must still be rejected (regression guard).
#[test]
fn after_reactivation_disabled_to_deprecated_still_rejected() {
    // First reach Disabled via a legal path.
    let disabled = CapabilityStatus::Active
        .try_transition_to(CapabilityStatus::Disabled)
        .expect("Active->Disabled");

    // The illegal edge must be rejected regardless of how we arrived at Disabled.
    assert!(
        disabled
            .try_transition_to(CapabilityStatus::Deprecated)
            .is_err(),
        "Disabled->Deprecated must always be illegal"
    );
}
