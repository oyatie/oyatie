#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use check_a11y_discipline::{
    A11yGapReason, ClientStack, ClientStackManifest, WcagTarget, check,
};
use std::collections::BTreeSet;

fn mf(ms: &str, stack: ClientStack, runners: &[&str]) -> ClientStackManifest {
    ClientStackManifest {
        microservice: ms.into(),
        stack,
        wcag_target: WcagTarget::AA,
        declared_runners: runners.iter().map(|s| (*s).to_owned()).collect(),
    }
}

#[test]
fn winui3_requires_accessibility_insights() {
    let r = check(&[mf(
        "ops-portal",
        ClientStack::WinUi3,
        &["accessibility-insights-for-windows"],
    )]);
    assert!(r.gaps.is_empty());
}

#[test]
fn gtk4_requires_at_spi_runner() {
    let r = check(&[mf(
        "calendar",
        ClientStack::Gtk4,
        &["at-spi-conformance-test"],
    )]);
    assert!(r.gaps.is_empty());
}

#[test]
fn wcag_aaa_target_is_accepted() {
    let manifest = ClientStackManifest {
        microservice: "healthcare-portal".into(),
        stack: ClientStack::SvelteKit,
        wcag_target: WcagTarget::AAA,
        declared_runners: BTreeSet::from(["pa11y".to_string(), "axe-core-playwright".to_string()]),
    };
    let r = check(&[manifest]);
    assert!(r.gaps.is_empty());
}

#[test]
fn unknown_runner_is_flagged_per_stack() {
    let r = check(&[
        mf("ms-a", ClientStack::Leptos, &["axe-core-playwright"]),
        mf("ms-b", ClientStack::Leptos, &["wrong-runner"]),
    ]);
    let unknowns: Vec<_> = r
        .gaps
        .iter()
        .filter(|g| matches!(g.reason, A11yGapReason::UnknownRunner { .. }))
        .collect();
    assert_eq!(unknowns.len(), 1);
}

#[test]
fn mobile_compose_stack_uses_accessibility_scanner() {
    let r = check(&[mf(
        "workflow-studio",
        ClientStack::Compose,
        &["android-accessibility-scanner-ci"],
    )]);
    assert!(r.gaps.is_empty());
}
