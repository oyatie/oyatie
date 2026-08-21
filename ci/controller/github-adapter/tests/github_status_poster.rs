//! GitHub commit-status poster adapter tests (the `oya-ci-required` producer).
//!
//! Asserts the success path (201 -> Ok), non-2xx -> `KernelError::DownstreamTransport`,
//! and the exact request body (`{state,context,description,target_url}`) + GitHub headers
//! (Authorization Bearer, X-GitHub-Api-Version, Accept, User-Agent).
//!
//! Ported off `httpmock` onto the first-party `scripted-http-server` (ADR-0709 D-6
//! Rule 2). The body assertions get STRONGER in the port: `json_body_partial` only
//! checked that the named fields were present with those values and said nothing about
//! any other field, whereas the recorded body is compared to a whole `json!` value, so
//! an extra or renamed field now fails. `pending_without_target_url_omits_field` in
//! particular could not previously observe the omission it is named for — a partial
//! match cannot assert ABSENCE — and now does.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ci_controller_github_adapter::GitHubCommitStatusPoster;
use ci_controller_kernel::{CommitState, CommitStatusPoster, KernelError};
use scripted_http_server::{ScriptedResponse, ScriptedServer};
use serde_json::json;

const TEST_SHA: &str = "abcdef1234567890abcdef1234567890abcdef12";

fn make_poster(server: &ScriptedServer) -> GitHubCommitStatusPoster {
    GitHubCommitStatusPoster::new("jason931225", "oyatie", "ghp_test_token")
        .with_api_base(server.base_url())
}

/// Success path — 201 Created -> Ok, with the correct body and headers.
#[test]
fn success_path_posts_correct_body_and_headers() {
    let server = ScriptedServer::start(vec![ScriptedResponse::status(201)]);

    let poster = make_poster(&server);
    let result = poster.post(
        TEST_SHA,
        CommitState::Success,
        "oya-ci-required",
        "gate passed",
        Some("https://ci.example.com/run/42"),
    );

    assert!(result.is_ok(), "expected Ok on 201, got {result:?}");

    let requests = server.requests();
    assert_eq!(requests.len(), 1, "requests: {:?}", server.request_lines());
    let request = &requests[0];
    assert_eq!(request.method, "POST");
    assert_eq!(
        request.path(),
        format!("/repos/jason931225/oyatie/statuses/{TEST_SHA}")
    );
    assert_eq!(
        request.header("authorization"),
        Some("Bearer ghp_test_token")
    );
    assert_eq!(request.header("x-github-api-version"), Some("2022-11-28"));
    assert_eq!(
        request.header("accept"),
        Some("application/vnd.github+json")
    );
    assert_eq!(request.header("user-agent"), Some("oya-ci-controller"));
    assert_eq!(
        request.json(),
        json!({
            "state": "success",
            "context": "oya-ci-required",
            "description": "gate passed",
            "target_url": "https://ci.example.com/run/42"
        })
    );
}

/// Pending state with no target_url omits the `target_url` field entirely.
#[test]
fn pending_without_target_url_omits_field() {
    let server = ScriptedServer::start(vec![ScriptedResponse::status(201)]);

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

    let requests = server.requests();
    assert_eq!(requests.len(), 1, "requests: {:?}", server.request_lines());
    assert_eq!(
        requests[0].path(),
        format!("/repos/jason931225/oyatie/statuses/{TEST_SHA}")
    );
    let body = requests[0].json();
    // Whole-value equality is what actually asserts the omission this test is named
    // for; `json_body_partial` never could.
    assert_eq!(
        body,
        json!({
            "state": "pending",
            "context": "oya-ci-required",
            "description": "running trusted required gate"
        })
    );
    assert!(
        body.get("target_url").is_none(),
        "target_url must be absent, not null: {body}"
    );
}

/// Non-2xx response maps to `KernelError::DownstreamTransport`.
#[test]
fn non_2xx_maps_to_downstream_transport() {
    for status in [401_u16, 404, 422, 500] {
        let server = ScriptedServer::start(vec![ScriptedResponse::status(status)]);

        let poster = make_poster(&server);
        let result = poster.post(
            TEST_SHA,
            CommitState::Failure,
            "oya-ci-required",
            "gate failed",
            None,
        );

        let requests = server.requests();
        assert_eq!(
            requests.len(),
            1,
            "HTTP {status}: requests: {:?}",
            server.request_lines()
        );
        assert_eq!(requests[0].method, "POST");
        assert_eq!(
            requests[0].path(),
            format!("/repos/jason931225/oyatie/statuses/{TEST_SHA}")
        );

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
