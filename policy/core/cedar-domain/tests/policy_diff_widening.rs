// ADR-0083 Tier 3: integration tests legitimately use `.unwrap()` / `.expect()` / `panic!()`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

mod common;

use common::*;
use policy_cedar_domain::policy_diff::*;
use policy_cedar_domain::{PolicyEffect, PolicyRuleInput, PolicyScope, PolicyVersion};

/// ACCEPTANCE: adding an Allow rule to an empty prev widens the surface.
#[test]
fn added_allow_widens() {
    let prev = pv("1.0.0", vec![]);
    let next = pv("1.1.0", vec![allow_rule("editor", "doc.write", "docs:")]);

    let report = diff_policy_versions(&prev, &next);

    assert_eq!(report.prev_version, "1.0.0");
    assert_eq!(report.next_version, "1.1.0");
    assert!(
        report.has_widening(),
        "adding an Allow rule must widen; deltas={:?}",
        report.deltas
    );
    assert!(
        report
            .deltas
            .iter()
            .any(|d| matches!(d, RuleDelta::RuleAdded(r) if r.effect == PolicyEffect::Allow)),
        "expected RuleAdded(Allow) delta"
    );
}

// ── acceptance: removed-deny widens ──────────────────────────────────────

/// ACCEPTANCE: removing a Deny rule widens the effective allow surface.
#[test]
fn removed_deny_widens() {
    let deny = deny_rule("admin", "account.delete", "acct:");
    let prev = pv("1.0.0", vec![deny.clone()]);
    let next = pv("1.1.0", vec![]);

    let report = diff_policy_versions(&prev, &next);

    assert!(
        report.has_widening(),
        "removing a Deny rule must widen; deltas={:?}",
        report.deltas
    );
    assert!(
        report
            .deltas
            .iter()
            .any(|d| matches!(d, RuleDelta::RemovedDeny(_))),
        "expected RemovedDeny delta"
    );
}

// ── acceptance: narrowed resource prefix is NOT widening ──────────────────

/// ACCEPTANCE: making an Allow rule more specific (longer prefix) is narrowing,
/// not widening.
#[test]
fn narrowed_resource_prefix_not_widening() {
    // prev: Allow on "docs:" (broader)
    // next: Allow on "docs:private:" (narrower — longer prefix)
    let prev = pv("1.0.0", vec![allow_rule("editor", "doc.write", "docs:")]);
    let next = pv(
        "1.1.0",
        vec![allow_rule("editor", "doc.write", "docs:private:")],
    );

    let report = diff_policy_versions(&prev, &next);

    assert!(
        !report.has_widening(),
        "narrowing the resource_prefix must NOT widen; deltas={:?}",
        report.deltas
    );
    assert!(
        report
            .deltas
            .iter()
            .any(|d| matches!(d, RuleDelta::NarrowedAllow { .. })),
        "expected NarrowedAllow delta; deltas={:?}",
        report.deltas
    );
}

// ── acceptance: effect flip Deny→Allow widens ─────────────────────────────

/// ACCEPTANCE: flipping an existing rule from Deny to Allow widens the surface.
#[test]
fn effect_flip_deny_to_allow_widens() {
    let prev_rule = deny_rule("ops", "cluster.drain", "k8s:cluster:");
    let next_rule = allow_rule("ops", "cluster.drain", "k8s:cluster:");

    let prev = pv("1.0.0", vec![prev_rule]);
    let next = pv("1.1.0", vec![next_rule]);

    let report = diff_policy_versions(&prev, &next);

    assert!(
        report.has_widening(),
        "flipping Deny→Allow must widen; deltas={:?}",
        report.deltas
    );
    assert!(
        report.deltas.iter().any(|d| matches!(
            d,
            RuleDelta::EffectFlipped { next_rule, .. } if next_rule.effect == PolicyEffect::Allow
        )),
        "expected EffectFlipped delta with next Allow; deltas={:?}",
        report.deltas
    );
}

// ── acceptance: identical versions produce empty report ───────────────────

/// ACCEPTANCE: diffing a version against itself yields no deltas.
#[test]
fn identical_versions_empty_report() {
    let rules = vec![
        allow_rule("reader", "doc.read", "docs:"),
        deny_rule("guest", "admin.write", "admin:"),
    ];
    let v1 = pv("1.0.0", rules.clone());
    let v2 = pv("1.0.0", rules);

    let report = diff_policy_versions(&v1, &v2);

    assert!(
        report.deltas.is_empty(),
        "identical versions must yield empty deltas; got {:?}",
        report.deltas
    );
    assert!(!report.has_widening());
}

// ── acceptance: serde round-trip on ImpactReport ──────────────────────────

/// ACCEPTANCE: ImpactReport (including RuleDelta variants) round-trips through
/// serde_json without error.
#[test]
fn impact_report_serde_round_trip() {
    let prev = pv("1.0.0", vec![]);
    let next = pv(
        "1.1.0",
        vec![
            allow_rule("editor", "doc.write", "docs:"),
            deny_rule("guest", "admin.delete", "admin:"),
        ],
    );

    let report = diff_policy_versions(&prev, &next);
    assert!(report.has_widening());

    let json = serde_json::to_string(&report).expect("ImpactReport serializes");
    let roundtrip: ImpactReport = serde_json::from_str(&json).expect("ImpactReport deserializes");

    assert_eq!(report.prev_version, roundtrip.prev_version);
    assert_eq!(report.next_version, roundtrip.next_version);
    assert_eq!(report.deltas.len(), roundtrip.deltas.len());
    assert_eq!(roundtrip.has_widening(), report.has_widening());
}

// ── additional: broadened allow widens ───────────────────────────────────
