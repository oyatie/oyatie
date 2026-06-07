//! GitHub commit-status poster adapter tests (the `oya-ci-required` producer).
//!
//! Lifts the httpmock pattern from the proven
//! oya-ci-webhook-gateway-github-adapter D5 tests. Asserts the success path
//! (201 -> Ok), non-2xx -> `KernelError::DownstreamTransport`, and the exact
//! request body (`{state,context,description,target_url}`) + GitHub headers
//! (Authorization Bearer, X-GitHub-Api-Version, Accept, User-Agent).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use httpmock::prelude::*;
use oya_ci_controller_github_adapter::GitHubCommitStatusPoster;
use oya_ci_controller_kernel::{CommitState, CommitStatusPoster, KernelError};

const TEST_SHA: &str = "abcdef1234567890abcdef1234567890abcdef12";

fn make_poster(server: &MockServer) -> GitHubCommitStatusPoster {
    GitHubCommitStatusPoster::new("jason931225", "oyatie", "ghp_test_token")
        .with_api_base(&server.base_url())
}

/// Success path — 201 Created -> Ok, with the correct body and headers.
#[test]
fn success_path_posts_correct_body_and_headers() {
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{TEST_SHA}"))
            .header("Authorization", "Bearer ghp_test_token")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "oya-ci-controller")
            .json_body_partial(
                r#"{"state":"success","context":"oya-ci-required","description":"gate passed","target_url":"https://ci.example.com/run/42"}"#,
            );
        then.status(201);
    });

    let poster = make_poster(&server);
    let result = poster.post(
        TEST_SHA,
        CommitState::Success,
        "oya-ci-required",
        "gate passed",
        Some("https://ci.example.com/run/42"),
    );

    assert!(result.is_ok(), "expected Ok on 201, got {result:?}");
    m.assert();
}

/// Pending state with no target_url omits the `target_url` field entirely.
#[test]
fn pending_without_target_url_omits_field() {
    let server = MockServer::start();

    let m = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{TEST_SHA}"))
            .json_body_partial(r#"{"state":"pending","context":"oya-ci-required"}"#);
        then.status(201);
    });

    let poster = make_poster(&server);
    poster
        .post(
            TEST_SHA,
            CommitState::Pending,
            "oya-ci-required",
            "running trusted required gate",
            None,
        )
        .unwrap();

    m.assert();
}

/// Non-2xx response maps to `KernelError::DownstreamTransport`.
#[test]
fn non_2xx_maps_to_downstream_transport() {
    for status in [401_u16, 404, 422, 500] {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/repos/jason931225/oyatie/statuses/{TEST_SHA}"));
            then.status(status);
        });

        let poster = make_poster(&server);
        let result = poster.post(
            TEST_SHA,
            CommitState::Failure,
            "oya-ci-required",
            "gate failed",
            None,
        );

        m.assert();
        match result {
            Err(KernelError::DownstreamTransport(msg)) => {
                assert!(
                    msg.contains(&status.to_string()),
                    "HTTP {status} error should carry the status code, got: {msg}"
                );
            }
            other => panic!("HTTP {status} should map to DownstreamTransport, got {other:?}"),
        }
    }
}
