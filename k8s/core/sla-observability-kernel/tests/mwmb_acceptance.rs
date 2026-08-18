//! Acceptance tests for `summarize_burn_rate_alert` — RED stage.
//!
//! These tests verify behaviours that are **specified** by the kernel contract
//! but are not yet implemented.  They are expected to FAIL until the
//! corresponding implementation is added.
//!
//! Contract reference: `docs/specs/task-sla-multiwindow-burnrate-alerting.md`
//! — "Unknown/malformed inputs fail closed" and the `SlaKernelError` variant
//!   `InvalidClusterIdentity` which `summarize_sla` enforces but
//!   `summarize_burn_rate_alert` / `window_burn_rate` currently do not.

use k8s_sla_observability_kernel::{
    BurnRatePolicy, ObservedControlPlaneStatus, SlaKernelError, SlaObservation, SlaPolicy,
    summarize_burn_rate_alert,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn valid_obs(tenant_id: &str, cluster_name: &str) -> SlaObservation {
    SlaObservation::new(
        tenant_id,
        cluster_name,
        ObservedControlPlaneStatus::Active,
        1_000,
        1_000,
        None,
    )
}

fn good() -> SlaObservation {
    valid_obs("ten_acme", "prod-a")
}

// ---------------------------------------------------------------------------
// Identity validation — RED tests
//
// `summarize_sla` rejects blank/whitespace `tenant_id` or `cluster_name` with
// `SlaKernelError::InvalidClusterIdentity`.  `summarize_burn_rate_alert` must
// enforce the same invariant for both the fast and slow window observations so
// that callers cannot silently produce alert verdicts for unidentifiable clusters.
//
// Current state: `window_burn_rate` validates only sample counts, so all four
// tests below return `Ok(...)` instead of `Err(InvalidClusterIdentity)` and
// therefore FAIL (RED).
// ---------------------------------------------------------------------------

/// Empty `tenant_id` in the fast window must be rejected with
/// `InvalidClusterIdentity` — mirrors `summarize_sla` fail-closed contract.
#[test]
fn blank_tenant_id_in_fast_window_fails_closed() {
    let fast = valid_obs("", "prod-a");
    let slow = good();
    assert_eq!(
        summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default()
        )
        .unwrap_err(),
        SlaKernelError::InvalidClusterIdentity,
    );
}

/// Empty `cluster_name` in the slow window must be rejected with
/// `InvalidClusterIdentity`.
#[test]
fn blank_cluster_name_in_slow_window_fails_closed() {
    let fast = good();
    let slow = valid_obs("ten_acme", "");
    assert_eq!(
        summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default()
        )
        .unwrap_err(),
        SlaKernelError::InvalidClusterIdentity,
    );
}

/// Whitespace-only `tenant_id` in the fast window must be rejected — the
/// existing `summarize_sla` guard uses `.trim().is_empty()`, so the same
/// normalisation must apply in `summarize_burn_rate_alert`.
#[test]
fn whitespace_only_tenant_id_in_fast_window_fails_closed() {
    let fast = valid_obs("   ", "prod-a");
    let slow = good();
    assert_eq!(
        summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default()
        )
        .unwrap_err(),
        SlaKernelError::InvalidClusterIdentity,
    );
}

/// When both windows carry blank identity the fast window is evaluated first;
/// the function must still return `InvalidClusterIdentity` (not `Ok`).
#[test]
fn blank_identity_in_both_windows_returns_invalid_cluster_identity() {
    let fast = valid_obs("", "");
    let slow = valid_obs("", "");
    assert_eq!(
        summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default()
        )
        .unwrap_err(),
        SlaKernelError::InvalidClusterIdentity,
    );
}

/// Empty `tenant_id` in the slow window must also be rejected — the check must
/// cover both windows, not only the fast one.
#[test]
fn blank_tenant_id_in_slow_window_fails_closed() {
    let fast = good();
    let slow = valid_obs("", "prod-a");
    assert_eq!(
        summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default()
        )
        .unwrap_err(),
        SlaKernelError::InvalidClusterIdentity,
    );
}

/// Whitespace-only `cluster_name` in the slow window must be rejected.
#[test]
fn whitespace_only_cluster_name_in_slow_window_fails_closed() {
    let fast = good();
    let slow = valid_obs("ten_acme", "\t");
    assert_eq!(
        summarize_burn_rate_alert(
            &fast,
            &slow,
            SlaPolicy::default(),
            BurnRatePolicy::default()
        )
        .unwrap_err(),
        SlaKernelError::InvalidClusterIdentity,
    );
}
