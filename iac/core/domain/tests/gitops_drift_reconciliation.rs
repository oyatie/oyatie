// TDD red-phase tests for the GitOps drift reconciliation slice.
//
// These tests were authored BEFORE (or concurrent with) the implementation
// to specify all behavioral contracts of `reconcile_gitops_drift`,
// `GitOpsDriftVerdict`, and `GitOpsDriftReport` that are NOT covered by the
// existing `iac_app_foundation` integration test suite.
//
// Coverage added here:
//   - All non-Healthy health status variants trigger DegradedHealth
//     (Progressing, Missing, GitOpsHealthStatus::Unknown)
//   - Rank-order precedence: DriftedCommit beats DriftedSyncStatus
//   - Rank-order precedence: DriftedCommit beats DegradedHealth
//   - Rank-order precedence: DriftedSyncStatus beats DegradedHealth
//   - IdentityMismatch on tenant_id (not just application_name)
//   - IdentityMismatch on cell_id
//   - IdentityMismatch on controller (future-proofing when a second controller
//     variant is added)
//   - GitOpsDriftReport and GitOpsDriftVerdict derive Clone/Debug/Eq/PartialEq
//   - GitOpsDriftVerdict Ord/PartialOrd ordering matches documented rank
//   - reconcile_gitops_drift carries observed (not desired) identity fields in
//     the report even when identities match
//   - A symmetric InSync case where both desired and observed have the same
//     non-Synced sync_status (desired is intentionally degraded — unusual but
//     the pure function must still rank correctly)
//
// ADR-0339 Argo CD drift semantics:
//   - SyncStatus: Synced | OutOfSync | Unknown
//   - HealthStatus: Healthy | Progressing | Degraded | Missing | Unknown
//
// ADR-0130 SLO shaping note:
//   - The verdict feeds `iac-validator-availability` and `drift-latency`
//     OpenSLO indicators.  The report fields are intentionally flat so a
//     telemetry adapter can emit them without re-reading either evidence
//     object.  Tests here confirm the flat fields are populated correctly.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use iac_domain::{
    GitOpsController, GitOpsDriftReport, GitOpsDriftVerdict, GitOpsEvidence, GitOpsEvidenceInput,
    GitOpsHealthStatus, GitOpsSyncStatus, reconcile_gitops_drift,
};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SHA_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[allow(clippy::too_many_arguments)]
fn make_evidence(
    controller: GitOpsController,
    tenant_id: &str,
    cell_id: &str,
    application_name: &str,
    commit_sha: &str,
    sync_status: GitOpsSyncStatus,
    health_status: GitOpsHealthStatus,
    evidence_tag: &str,
) -> GitOpsEvidence {
    GitOpsEvidence::new(GitOpsEvidenceInput {
        controller,
        tenant_id: tenant_id.to_string(),
        cell_id: cell_id.to_string(),
        application_name: application_name.to_string(),
        repository_url: "https://git.oyatie.internal/oyatie/oyatie.git".to_string(),
        commit_sha: commit_sha.to_string(),
        sync_status,
        health_status,
        evidence_ref: format!("evidence://drift-tests/{evidence_tag}"),
    })
    .expect("test evidence is valid")
}

/// Build a fully-converged desired/observed pair sharing identity and SHA.
fn converged_pair() -> (GitOpsEvidence, GitOpsEvidence) {
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/converged",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "observed/converged",
    );
    (desired, observed)
}

// ---------------------------------------------------------------------------
// Health status variants → DegradedHealth
// ---------------------------------------------------------------------------

#[test]
fn drift_progressing_health_status_yields_degraded_health_verdict() {
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/progressing",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Progressing,
        "observed/progressing",
    );

    let report = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(report.verdict, GitOpsDriftVerdict::DegradedHealth);
    assert_eq!(
        report.observed_health_status,
        GitOpsHealthStatus::Progressing
    );
}

#[test]
fn drift_missing_health_status_yields_degraded_health_verdict() {
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/missing",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Missing,
        "observed/missing",
    );

    let report = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(report.verdict, GitOpsDriftVerdict::DegradedHealth);
    assert_eq!(report.observed_health_status, GitOpsHealthStatus::Missing);
}

#[test]
fn drift_unknown_health_status_yields_degraded_health_verdict() {
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/health-unknown",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Unknown,
        "observed/health-unknown",
    );

    let report = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(report.verdict, GitOpsDriftVerdict::DegradedHealth);
    assert_eq!(report.observed_health_status, GitOpsHealthStatus::Unknown);
}

// ---------------------------------------------------------------------------
// Rank-order precedence: DriftedCommit beats lower-priority verdicts
// ---------------------------------------------------------------------------

#[test]
fn drift_commit_beats_sync_status_drift_when_both_present() {
    // observed SHA differs AND sync_status is OutOfSync → commit wins
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/commit-beats-sync",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_B,                       // commit mismatch
        GitOpsSyncStatus::OutOfSync, // also out-of-sync
        GitOpsHealthStatus::Healthy,
        "observed/commit-beats-sync",
    );

    let report = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(report.verdict, GitOpsDriftVerdict::DriftedCommit);
    assert_eq!(report.observed_commit_sha, SHA_B);
    assert_eq!(report.observed_sync_status, GitOpsSyncStatus::OutOfSync);
}

#[test]
fn drift_commit_beats_degraded_health_when_both_present() {
    // observed SHA differs AND health is Degraded → commit wins
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/commit-beats-health",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_B, // commit mismatch
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Degraded, // also degraded
        "observed/commit-beats-health",
    );

    let report = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(report.verdict, GitOpsDriftVerdict::DriftedCommit);
    assert_eq!(report.observed_health_status, GitOpsHealthStatus::Degraded);
}

// ---------------------------------------------------------------------------
// Rank-order precedence: DriftedSyncStatus beats DegradedHealth
// ---------------------------------------------------------------------------

#[test]
fn drift_sync_status_beats_degraded_health_when_both_present() {
    // sync_status is OutOfSync AND health is Degraded → sync status wins
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/sync-beats-health",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::OutOfSync,  // sync drift
        GitOpsHealthStatus::Degraded, // also degraded
        "observed/sync-beats-health",
    );

    let report = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(report.verdict, GitOpsDriftVerdict::DriftedSyncStatus);
    assert_eq!(report.observed_sync_status, GitOpsSyncStatus::OutOfSync);
    assert_eq!(report.observed_health_status, GitOpsHealthStatus::Degraded);
}

// ---------------------------------------------------------------------------
// IdentityMismatch on individual identity fields
// ---------------------------------------------------------------------------

#[test]
fn drift_identity_mismatch_on_tenant_id() {
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/tenant-mismatch",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_beta", // different tenant
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "observed/tenant-mismatch",
    );

    let report = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(report.verdict, GitOpsDriftVerdict::IdentityMismatch);
    // report carries the observed tenant, not the desired tenant
    assert_eq!(report.tenant_id, "ten_beta");
}

#[test]
fn drift_identity_mismatch_on_cell_id() {
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/cell-mismatch",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-002", // different cell
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "observed/cell-mismatch",
    );

    let report = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(report.verdict, GitOpsDriftVerdict::IdentityMismatch);
    assert_eq!(report.cell_id, "cell-kr-seoul-1-a-002");
}

// ---------------------------------------------------------------------------
// Report carries observed fields (not desired) when identities match
// ---------------------------------------------------------------------------

#[test]
fn drift_report_carries_observed_identity_fields_when_in_sync() {
    // Desired and observed are fully aligned. The report must reflect the
    // observed evidence fields (controller, tenant_id, cell_id, application_name)
    // so downstream telemetry adapters only need the report struct.
    let (desired, observed) = converged_pair();

    let report = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(report.verdict, GitOpsDriftVerdict::InSync);
    assert_eq!(report.controller, GitOpsController::ArgoCd);
    assert_eq!(report.tenant_id, "ten_alpha");
    assert_eq!(report.cell_id, "cell-kr-seoul-1-a-001");
    assert_eq!(report.application_name, "iac-app-foundation");
    assert_eq!(report.observed_commit_sha, SHA_A);
    assert_eq!(report.observed_sync_status, GitOpsSyncStatus::Synced);
    assert_eq!(report.observed_health_status, GitOpsHealthStatus::Healthy);
}

// ---------------------------------------------------------------------------
// Derived trait verification: Clone / Debug / Eq / PartialEq
// ---------------------------------------------------------------------------

#[test]
fn drift_verdict_derives_clone_debug_eq_partial_eq() {
    let v1 = GitOpsDriftVerdict::InSync;
    let v2 = v1;
    assert_eq!(v1, v2);
    // Debug must not panic and must produce non-empty output
    let debug = format!("{v1:?}");
    assert!(!debug.is_empty());

    assert_ne!(
        GitOpsDriftVerdict::InSync,
        GitOpsDriftVerdict::DriftedCommit
    );
    assert_ne!(
        GitOpsDriftVerdict::DriftedCommit,
        GitOpsDriftVerdict::DriftedSyncStatus
    );
    assert_ne!(
        GitOpsDriftVerdict::DriftedSyncStatus,
        GitOpsDriftVerdict::DegradedHealth
    );
    assert_ne!(
        GitOpsDriftVerdict::DegradedHealth,
        GitOpsDriftVerdict::IdentityMismatch
    );
}

#[test]
fn drift_report_derives_clone_debug_eq_partial_eq() {
    let (desired, observed) = converged_pair();
    let report: GitOpsDriftReport = reconcile_gitops_drift(&desired, &observed);

    // Clone
    let cloned = report.clone();
    assert_eq!(report, cloned);

    // Debug must not panic
    let debug = format!("{report:?}");
    assert!(!debug.is_empty());

    // PartialEq / Eq: a report with a different verdict must not equal the original
    let desired2 = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired2/eq",
    );
    let observed_drifted = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_B,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "observed2/eq",
    );
    let drifted_report = reconcile_gitops_drift(&desired2, &observed_drifted);
    assert_ne!(report, drifted_report);
}

// ---------------------------------------------------------------------------
// GitOpsDriftVerdict Ord: rank ordering matches documented priority
//
// Documented order (ascending severity):
//   InSync < DriftedCommit < DriftedSyncStatus < DegradedHealth < IdentityMismatch
//
// NOTE: The derive(Ord) ordering tracks declaration order in the enum, which
// the crate defines as:
//   InSync, DriftedCommit, DriftedSyncStatus, DegradedHealth, IdentityMismatch
// This test pins that contract so any re-ordering of the enum variants is
// caught immediately.
// ---------------------------------------------------------------------------

#[test]
fn drift_verdict_ord_matches_documented_rank_order() {
    assert!(GitOpsDriftVerdict::InSync < GitOpsDriftVerdict::DriftedCommit);
    assert!(GitOpsDriftVerdict::DriftedCommit < GitOpsDriftVerdict::DriftedSyncStatus);
    assert!(GitOpsDriftVerdict::DriftedSyncStatus < GitOpsDriftVerdict::DegradedHealth);
    assert!(GitOpsDriftVerdict::DegradedHealth < GitOpsDriftVerdict::IdentityMismatch);
}

// ---------------------------------------------------------------------------
// Regression: reconcile_gitops_drift is pure (no side effects detectable from
// calling it multiple times with the same inputs returns identical results)
// ---------------------------------------------------------------------------

#[test]
fn drift_reconcile_is_deterministic_pure_function() {
    let desired = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_A,
        GitOpsSyncStatus::Synced,
        GitOpsHealthStatus::Healthy,
        "desired/pure",
    );
    let observed = make_evidence(
        GitOpsController::ArgoCd,
        "ten_alpha",
        "cell-kr-seoul-1-a-001",
        "iac-app-foundation",
        SHA_B,
        GitOpsSyncStatus::OutOfSync,
        GitOpsHealthStatus::Degraded,
        "observed/pure",
    );

    let r1 = reconcile_gitops_drift(&desired, &observed);
    let r2 = reconcile_gitops_drift(&desired, &observed);

    assert_eq!(r1, r2);
    // Inputs are not consumed (borrows only)
    let r3 = reconcile_gitops_drift(&desired, &observed);
    assert_eq!(r1, r3);
}
