//! D5 GitHub commit-status poster adapter tests (ADR-0387).
//!
//! 5 tests: one per CommitStatusContext, each asserting the correct JSON body
//! shape (state, context, description fields) is posted to the right endpoint.
//!
//! Ported off `httpmock` onto the first-party `scripted-http-server` (ADR-0709 D-6
//! Rule 2). Each `json_body_partial` becomes whole-value equality against a `json!`
//! literal, which is strictly stronger: the partial matcher checked only that `state`
//! and `context` were present with those values, so `description` and `target_url` — the
//! two fields `make_request` actually computes — went unasserted in all five tests.
//! They are asserted here.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ci_webhook_gateway_github::GitHubStatusPoster;
use ci_webhook_gateway_kernel::{
    CommitStatusContext, CommitStatusPoster, CommitStatusState, GitHubStatusRequest,
};
use scripted_http_server::{ScriptedResponse, ScriptedServer};
use serde_json::json;

const TARGET_URL: &str = "https://jenkins.example.com/job/oyaCiLane/42/";

fn make_request(context: CommitStatusContext, state: CommitStatusState) -> GitHubStatusRequest {
    GitHubStatusRequest {
        owner: "oyatie".to_owned(),
        repo: "oyatie".to_owned(),
        sha: "deadbeef1234".to_owned(),
        state,
        context,
        description: format!("{} — {}", context.as_str(), state.as_str()),
        target_url: Some(TARGET_URL.to_owned()),
    }
}

fn make_poster(server: &ScriptedServer) -> GitHubStatusPoster {
    GitHubStatusPoster::new("oyatie", "oyatie", "ghp_test_token").with_api_base(server.base_url())
}

/// Drive one context/state pair and assert the whole posted body, not a fragment.
fn assert_posts_body(context: CommitStatusContext, state: CommitStatusState) {
    let server = ScriptedServer::start(vec![ScriptedResponse::status(201)]);

    let poster = make_poster(&server);
    poster.post(&make_request(context, state)).unwrap();

    let requests = server.requests();
    assert_eq!(requests.len(), 1, "requests: {:?}", server.request_lines());
    let request = &requests[0];
    assert_eq!(request.method, "POST");
    assert_eq!(request.path(), "/repos/oyatie/oyatie/statuses/deadbeef1234");
    assert_eq!(
        request.header("authorization"),
        Some("Bearer ghp_test_token")
    );
    assert_eq!(request.header("x-github-api-version"), Some("2022-11-28"));
    assert_eq!(
        request.json(),
        json!({
            "state": state.as_str(),
            "context": context.as_str(),
            "description": format!("{} — {}", context.as_str(), state.as_str()),
            "target_url": TARGET_URL,
        })
    );
}

/// Test 1 — cargo-fmt context posts with correct JSON body.
#[test]
fn cargo_fmt_posts_correct_body() {
    assert_posts_body(CommitStatusContext::CargoFmt, CommitStatusState::Success);
}

/// Test 2 — cargo-check context posts with correct JSON body.
#[test]
fn cargo_check_posts_correct_body() {
    assert_posts_body(CommitStatusContext::CargoCheck, CommitStatusState::Pending);
}

/// Test 3 — cargo-clippy context posts with correct JSON body.
#[test]
fn cargo_clippy_posts_correct_body() {
    assert_posts_body(CommitStatusContext::CargoClippy, CommitStatusState::Failure);
}

/// Test 4 — cargo-nextest context posts with correct JSON body.
#[test]
fn cargo_nextest_posts_correct_body() {
    assert_posts_body(
        CommitStatusContext::CargoNextest,
        CommitStatusState::Success,
    );
}

/// Test 5 — pr-review context posts with correct JSON body.
#[test]
fn pr_review_posts_correct_body() {
    assert_posts_body(CommitStatusContext::OyaPrReview, CommitStatusState::Success);
}
