//! Trusted GitHub review-admission producer tests.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use httpmock::Mock;
use httpmock::prelude::*;
use oya_ci_controller_github_adapter::GitHubCommitStatusPoster;
use oya_ci_controller_kernel::{KernelError, ReviewVerdict};
use serde_json::json;

const PR_NUMBER: u64 = 42;
const HEAD_SHA: &str = "abcdef1234567890abcdef1234567890abcdef12";
const OTHER_SHA: &str = "1234567890abcdef1234567890abcdef12345678";
const REVIEW_URL: &str = "https://github.com/jason931225/oyatie/pull/42#pullrequestreview-9001";

fn make_poster(server: &MockServer) -> GitHubCommitStatusPoster {
    GitHubCommitStatusPoster::new("jason931225", "oyatie", "test-token")
        .with_api_base(&server.base_url())
}

fn mock_pull<'a>(server: &'a MockServer, author: &str, head_sha: &str) -> Mock<'a> {
    server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42")
            .header("Authorization", "Bearer test-token")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "oya-ci-controller");
        then.status(200).json_body(json!({
            "number": PR_NUMBER,
            "html_url": "https://github.com/jason931225/oyatie/pull/42",
            "user": { "login": author },
            "head": { "sha": head_sha }
        }));
    })
}

fn mock_reviews<'a>(
    server: &'a MockServer,
    reviewer: &str,
    commit_id: &str,
    evidence_url: &str,
) -> Mock<'a> {
    server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100");
        then.status(200).json_body(json!([{
            "id": 9001,
            "state": "APPROVED",
            "commit_id": commit_id,
            "html_url": evidence_url,
            "user": { "login": reviewer }
        }]));
    })
}

#[test]
fn approved_distinct_reviewer_posts_head_bound_status_with_durable_evidence() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = mock_reviews(&server, "independent-reviewer", HEAD_SHA, REVIEW_URL);
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body(json!({
                "state": "success",
                "context": "oya-pr-review",
                "description": "oya-pr-review approved by independent-reviewer",
                "target_url": REVIEW_URL
            }));
        then.status(201);
    });

    let packet = make_poster(&server)
        .produce_review_admission_status(PR_NUMBER, HEAD_SHA)
        .expect("distinct approved review should be admitted");

    assert_eq!(packet.pr_number, PR_NUMBER);
    assert_eq!(packet.head_sha, HEAD_SHA);
    assert_eq!(packet.author_login, "change-author");
    assert_eq!(packet.reviewer_login, "independent-reviewer");
    assert_eq!(packet.verdict, ReviewVerdict::Approved);
    assert_eq!(packet.evidence_url, REVIEW_URL);
    pull.assert();
    reviews.assert();
    status.assert();
}

#[test]
fn missing_durable_review_url_is_rejected_and_posts_failure() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = mock_reviews(&server, "independent-reviewer", HEAD_SHA, "");
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(PR_NUMBER, HEAD_SHA);

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("evidence URL"))
    );
    pull.assert();
    reviews.assert();
    status.assert();
}

#[test]
fn author_cannot_satisfy_review_admission_and_failure_is_posted() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "same-user", HEAD_SHA);
    let reviews = mock_reviews(&server, "SAME-USER", HEAD_SHA, REVIEW_URL);
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(PR_NUMBER, HEAD_SHA);

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("distinct"))
    );
    pull.assert();
    reviews.assert();
    status.assert();
}

#[test]
fn pull_head_mismatch_is_rejected_before_review_evidence_can_pass() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", OTHER_SHA);
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(PR_NUMBER, HEAD_SHA);

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("head SHA"))
    );
    pull.assert();
    status.assert();
}

#[test]
fn newer_changes_requested_verdict_supersedes_older_approval() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100");
        then.status(200).json_body(json!([
            {
                "id": 9001,
                "state": "APPROVED",
                "commit_id": HEAD_SHA,
                "html_url": REVIEW_URL,
                "user": { "login": "independent-reviewer" }
            },
            {
                "id": 9002,
                "state": "CHANGES_REQUESTED",
                "commit_id": HEAD_SHA,
                "html_url": "https://github.com/jason931225/oyatie/pull/42#pullrequestreview-9002",
                "user": { "login": "independent-reviewer" }
            }
        ]));
    });
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(PR_NUMBER, HEAD_SHA);

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("APPROVED"))
    );
    pull.assert();
    reviews.assert();
    status.assert();
}
