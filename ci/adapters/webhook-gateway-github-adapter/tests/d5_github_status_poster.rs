//! D5 GitHub commit-status poster adapter tests (ADR-0387).
//!
//! 5 tests: one per CommitStatusContext, each asserting the correct JSON body
//! shape (state, context, description fields) is posted to the right endpoint.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use httpmock::prelude::*;
use ci_webhook_gateway_github_adapter::GitHubStatusPoster;
use oya_ci_webhook_gateway_kernel::{
    CommitStatusContext, CommitStatusPoster, CommitStatusState, GitHubStatusRequest,
};

fn make_request(context: CommitStatusContext, state: CommitStatusState) -> GitHubStatusRequest {
    GitHubStatusRequest {
        owner: "oyatie".to_owned(),
        repo: "oyatie".to_owned(),
        sha: "deadbeef1234".to_owned(),
        state,
        context,
        description: format!("{} — {}", context.as_str(), state.as_str()),
        target_url: Some("https://jenkins.example.com/job/oyaCiLane/42/".to_owned()),
    }
}

fn make_poster(server: &MockServer) -> GitHubStatusPoster {
    GitHubStatusPoster::new("oyatie", "oyatie", "ghp_test_token").with_api_base(&server.base_url())
}

/// Test 1 — cargo-fmt context posts with correct JSON body.
#[test]
fn cargo_fmt_posts_correct_body() {
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/repos/oyatie/oyatie/statuses/deadbeef1234")
            .header("Authorization", "Bearer ghp_test_token")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json_body_partial(r#"{"state":"success","context":"cargo-fmt"}"#);
        then.status(201);
    });

    let poster = make_poster(&server);
    poster
        .post(&make_request(
            CommitStatusContext::CargoFmt,
            CommitStatusState::Success,
        ))
        .unwrap();

    m.assert();
}

/// Test 2 — cargo-check context posts with correct JSON body.
#[test]
fn cargo_check_posts_correct_body() {
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/repos/oyatie/oyatie/statuses/deadbeef1234")
            .json_body_partial(r#"{"state":"pending","context":"cargo-check"}"#);
        then.status(201);
    });

    let poster = make_poster(&server);
    poster
        .post(&make_request(
            CommitStatusContext::CargoCheck,
            CommitStatusState::Pending,
        ))
        .unwrap();

    m.assert();
}

/// Test 3 — cargo-clippy context posts with correct JSON body.
#[test]
fn cargo_clippy_posts_correct_body() {
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/repos/oyatie/oyatie/statuses/deadbeef1234")
            .json_body_partial(r#"{"state":"failure","context":"cargo-clippy"}"#);
        then.status(201);
    });

    let poster = make_poster(&server);
    poster
        .post(&make_request(
            CommitStatusContext::CargoClippy,
            CommitStatusState::Failure,
        ))
        .unwrap();

    m.assert();
}

/// Test 4 — cargo-nextest context posts with correct JSON body.
#[test]
fn cargo_nextest_posts_correct_body() {
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/repos/oyatie/oyatie/statuses/deadbeef1234")
            .json_body_partial(r#"{"state":"success","context":"cargo-nextest"}"#);
        then.status(201);
    });

    let poster = make_poster(&server);
    poster
        .post(&make_request(
            CommitStatusContext::CargoNextest,
            CommitStatusState::Success,
        ))
        .unwrap();

    m.assert();
}

/// Test 5 — oya-pr-review context posts with correct JSON body.
#[test]
fn oya_pr_review_posts_correct_body() {
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path("/repos/oyatie/oyatie/statuses/deadbeef1234")
            .json_body_partial(r#"{"state":"success","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let poster = make_poster(&server);
    poster
        .post(&make_request(
            CommitStatusContext::OyaPrReview,
            CommitStatusState::Success,
        ))
        .unwrap();

    m.assert();
}
