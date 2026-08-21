//! D5 — Commit-status request body formatting tests.
//!
//! For each of the 5 required contexts, the [`GitHubStatusRequest::to_api_json`]
//! output must contain the correct `"state"` and `"context"` fields.
//! All 4 tests PASS at Stage-4 RED (pure data formatting, no I/O).

use ci_webhook_gateway_kernel::{
    CommitStatusContext, CommitStatusState, GitHubStatusRequest, JobStatus,
};

const OWNER: &str = "oyatie";
const REPO: &str = "oyatie";
const SHA: &str = "deadbeef01234567890abcdef01234567890abcdef";
const BUILD_URL: &str = "https://jenkins.oya.local/job/oyaCiLane/42/";

// ---------------------------------------------------------------------------
// D5-1: all 5 contexts produce correct context strings in the JSON body
// ---------------------------------------------------------------------------

#[test]
fn d5_all_five_contexts_produce_correct_context_strings() {
    let expected_pairs = [
        (CommitStatusContext::CargoFmt, "cargo-fmt"),
        (CommitStatusContext::CargoCheck, "cargo-check"),
        (CommitStatusContext::CargoClippy, "cargo-clippy"),
        (CommitStatusContext::CargoNextest, "cargo-nextest"),
        (CommitStatusContext::OyaPrReview, "oya-pr-review"),
    ];

    for (context, expected_str) in expected_pairs {
        let req = GitHubStatusRequest::from_job_outcome(
            OWNER,
            REPO,
            SHA,
            context,
            JobStatus::Success,
            Some(BUILD_URL),
        );
        let json = req.to_api_json();
        assert!(
            json.contains(&format!(r#""context":"{expected_str}""#)),
            "context {expected_str}: expected context string in JSON, got: {json}"
        );
        assert!(
            json.contains(r#""state":"success""#),
            "context {expected_str}: expected state=success, got: {json}"
        );
    }
}

// ---------------------------------------------------------------------------
// D5-2: failure job status maps all 5 contexts to state=failure
// ---------------------------------------------------------------------------

#[test]
fn d5_failure_job_status_maps_to_failure_state_for_all_contexts() {
    for context in CommitStatusContext::ALL {
        let req = GitHubStatusRequest::from_job_outcome(
            OWNER,
            REPO,
            SHA,
            context,
            JobStatus::Failure,
            None,
        );
        let json = req.to_api_json();
        assert!(
            json.contains(r#""state":"failure""#),
            "context {}: expected state=failure for Failure job, got: {json}",
            context.as_str()
        );
        // No target_url when build_url is None.
        assert!(
            !json.contains("target_url"),
            "context {}: expected no target_url when build_url is None, got: {json}",
            context.as_str()
        );
    }
}

// ---------------------------------------------------------------------------
// D5-3: pending status helper produces state=pending + target_url
// ---------------------------------------------------------------------------

#[test]
fn d5_pending_status_includes_target_url_and_pending_state() {
    let req = GitHubStatusRequest::pending(
        OWNER,
        REPO,
        SHA,
        CommitStatusContext::CargoNextest,
        Some(BUILD_URL),
    );
    let json = req.to_api_json();
    assert!(
        json.contains(r#""state":"pending""#),
        "pending helper should produce state=pending, got: {json}"
    );
    assert!(
        json.contains("target_url"),
        "pending helper should include target_url when provided, got: {json}"
    );
    assert!(
        json.contains(BUILD_URL),
        "pending helper should embed the build URL, got: {json}"
    );
}

// ---------------------------------------------------------------------------
// D5-4: oya-pr-review context produces correct state + context in JSON
// ---------------------------------------------------------------------------

#[test]
fn d5_oya_pr_review_context_formats_correctly() {
    let req = GitHubStatusRequest {
        owner: OWNER.to_owned(),
        repo: REPO.to_owned(),
        sha: SHA.to_owned(),
        state: CommitStatusState::Success,
        context: CommitStatusContext::OyaPrReview,
        description: "oya-pr-review — passed".to_owned(),
        target_url: Some(BUILD_URL.to_owned()),
    };
    let json = req.to_api_json();
    assert!(
        json.contains(r#""context":"oya-pr-review""#),
        "oya-pr-review context should appear in JSON, got: {json}"
    );
    assert!(
        json.contains(r#""state":"success""#),
        "oya-pr-review state should be success, got: {json}"
    );
    assert_eq!(req.owner, OWNER);
    assert_eq!(req.repo, REPO);
    assert_eq!(req.sha, SHA);
}
