//! Trusted GitHub review-admission producer tests.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{
    io::{BufRead, BufReader, ErrorKind, Read, Write},
    net::TcpListener,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

use ci_controller_github_adapter::GitHubCommitStatusPoster;
use ci_controller_kernel::{
    CommitState, CommitStatusPoster, GitHubAccountType, GitHubPrincipal, KernelError,
    ReviewAdmissionPolicy, ReviewAdmissionProducer, ReviewVerdict,
};
use httpmock::Mock;
use httpmock::prelude::*;
use serde_json::json;

const PR_NUMBER: u64 = 42;
const HEAD_SHA: &str = "abcdef1234567890abcdef1234567890abcdef12";
const OTHER_SHA: &str = "1234567890abcdef1234567890abcdef12345678";
const REVIEW_URL: &str = "https://github.com/jason931225/oyatie/pull/42#pullrequestreview-9001";
const EVALUATED_AT_UNIX_S: i64 = 1_700_000_000;

fn make_poster(server: &MockServer) -> GitHubCommitStatusPoster {
    make_poster_at(&server.base_url())
}

fn make_poster_at(api_base: &str) -> GitHubCommitStatusPoster {
    GitHubCommitStatusPoster::new("jason931225", "oyatie", "test-token").with_api_base(api_base)
}

fn review_policy(reviewers: &[&str]) -> ReviewAdmissionPolicy {
    let mut policy = ReviewAdmissionPolicy {
        policy_ref: "repo://review-policy/rust-reviewers".to_owned(),
        version: "2026-07-21".to_owned(),
        sha256_digest: String::new(),
        issuer: "test-controller".to_owned(),
        effective_at_unix_s: EVALUATED_AT_UNIX_S - 1,
        expires_at_unix_s: EVALUATED_AT_UNIX_S + 1,
        revoked: false,
        eligible_reviewers: reviewers.iter().map(|login| principal(login)).collect(),
    };
    policy.sha256_digest = policy.canonical_sha256();
    policy
}

fn producer() -> ReviewAdmissionProducer {
    ReviewAdmissionProducer {
        github_app_id: 1,
        workload_identity: "test://oya-ci-controller/review-admission".to_owned(),
    }
}

fn principal(login: &str) -> GitHubPrincipal {
    let id = match login.to_ascii_lowercase().as_str() {
        "change-author" | "same-user" => 1,
        "independent-reviewer" => 2,
        "drive-by-reviewer" => 3,
        "designated-reviewer" => 4,
        _ => 99,
    };
    GitHubPrincipal {
        id,
        account_type: GitHubAccountType::User,
        login: login.to_owned(),
    }
}

fn github_user(login: &str) -> serde_json::Value {
    let principal = principal(login);
    json!({
        "id": principal.id,
        "login": principal.login,
        "type": "User"
    })
}

fn scripted_http_server(
    responses: Vec<String>,
) -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind scripted HTTP server");
    let address = listener.local_addr().expect("read scripted server address");
    let requests = Arc::new(Mutex::new(Vec::new()));
    let observed_requests = Arc::clone(&requests);
    let handle = thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().expect("accept scripted HTTP request");
            let mut reader = BufReader::new(&mut stream);
            let mut request_line = String::new();
            reader
                .read_line(&mut request_line)
                .expect("read scripted request line");
            let mut content_length = 0;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).expect("read scripted header");
                if header == "\r\n" {
                    break;
                }
                if let Some(value) = header.strip_prefix("content-length:") {
                    content_length = value.trim().parse().expect("parse content length");
                }
            }
            let mut body = vec![0; content_length];
            reader
                .read_exact(&mut body)
                .expect("read scripted request body");
            observed_requests
                .lock()
                .expect("lock scripted request trace")
                .push(format!(
                    "{}{}",
                    request_line.trim_end(),
                    String::from_utf8_lossy(&body)
                ));
            reader
                .get_mut()
                .write_all(response.as_bytes())
                .expect("write scripted response");
        }
    });
    (format!("http://{address}"), requests, handle)
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
            "user": github_user(author),
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
            "user": github_user(reviewer)
        }]));
    })
}

#[test]
fn approved_distinct_reviewer_rechecks_head_before_posting_status() {
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
        .produce_review_admission_status(
            PR_NUMBER,
            HEAD_SHA,
            &review_policy(&["independent-reviewer"]),
            &producer(),
            EVALUATED_AT_UNIX_S,
        )
        .expect("distinct approved review should be admitted");

    assert_eq!(packet.pr_number, PR_NUMBER);
    assert_eq!(packet.head_sha, HEAD_SHA);
    assert_eq!(packet.author, principal("change-author"));
    assert_eq!(packet.reviewer, principal("independent-reviewer"));
    assert_eq!(
        packet.reviewer_eligibility_policy_ref,
        "repo://review-policy/rust-reviewers"
    );
    assert_eq!(packet.reviewer_eligibility_policy_version, "2026-07-21");
    assert_eq!(
        packet.reviewer_eligibility_policy_sha256,
        review_policy(&["independent-reviewer"]).sha256_digest
    );
    assert_eq!(packet.policy_evaluated_at_unix_s, EVALUATED_AT_UNIX_S);
    assert_eq!(
        packet.reviewer_eligibility_policy_effective_at_unix_s,
        EVALUATED_AT_UNIX_S - 1
    );
    assert_eq!(
        packet.reviewer_eligibility_policy_expires_at_unix_s,
        EVALUATED_AT_UNIX_S + 1
    );
    assert!(!packet.reviewer_eligibility_policy_revoked);
    assert_eq!(packet.verdict, ReviewVerdict::Approved);
    assert_eq!(packet.evidence_url, REVIEW_URL);
    pull.assert_hits(2);
    reviews.assert();
    status.assert();
}

#[test]
fn unchanged_digest_cannot_authorize_a_tampered_reviewer_allowlist() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = mock_reviews(&server, "independent-reviewer", HEAD_SHA, REVIEW_URL);
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });
    let mut policy = review_policy(&["independent-reviewer"]);
    policy
        .eligible_reviewers
        .insert(principal("drive-by-reviewer"));

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &policy,
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(&result, Err(KernelError::InvalidInput(message)) if message.contains("digest")),
        "expected policy-digest integrity failure, got {result:?}"
    );
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

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("evidence URL"))
    );
    pull.assert();
    reviews.assert();
    status.assert();
}

#[test]
fn evidence_url_must_bind_the_configured_repository_pr_and_review() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = mock_reviews(
        &server,
        "independent-reviewer",
        HEAD_SHA,
        "https://github.com/other/repository/pull/42#pullrequestreview-9001",
    );
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("repository, PR, and review"))
    );
    pull.assert();
    reviews.assert();
    status.assert();
}

#[test]
fn malformed_eligible_approval_does_not_veto_another_valid_eligible_approval() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100");
        then.status(200).json_body(json!([
            {
                "id": 9002,
                "state": "APPROVED",
                "commit_id": HEAD_SHA,
                "html_url": "https://github.com/other/repository/pull/42#pullrequestreview-9002",
                "user": github_user("independent-reviewer")
            },
            {
                "id": 9001,
                "state": "APPROVED",
                "commit_id": HEAD_SHA,
                "html_url": REVIEW_URL,
                "user": github_user("designated-reviewer")
            }
        ]));
    });
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body(json!({
                "state": "success",
                "context": "oya-pr-review",
                "description": "oya-pr-review approved by designated-reviewer",
                "target_url": REVIEW_URL
            }));
        then.status(201);
    });

    let packet = make_poster(&server)
        .produce_review_admission_status(
            PR_NUMBER,
            HEAD_SHA,
            &review_policy(&["independent-reviewer", "designated-reviewer"]),
            &producer(),
            EVALUATED_AT_UNIX_S,
        )
        .expect("another eligible reviewer with durable evidence should be selected");

    assert_eq!(packet.reviewer, principal("designated-reviewer"));
    assert_eq!(packet.evidence_url, REVIEW_URL);
    pull.assert_hits(2);
    reviews.assert();
    status.assert();
}

#[test]
fn policy_receipt_and_producer_identity_fail_closed_when_incomplete_or_invalid() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = mock_reviews(&server, "independent-reviewer", HEAD_SHA, REVIEW_URL);
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });
    let mut policies = Vec::new();
    let mut missing_version = review_policy(&["independent-reviewer"]);
    missing_version.version.clear();
    policies.push(missing_version);
    let mut bad_digest = review_policy(&["independent-reviewer"]);
    bad_digest.sha256_digest = "not-a-digest".to_owned();
    policies.push(bad_digest);
    let mut missing_issuer = review_policy(&["independent-reviewer"]);
    missing_issuer.issuer.clear();
    policies.push(missing_issuer);
    let mut future = review_policy(&["independent-reviewer"]);
    future.effective_at_unix_s = EVALUATED_AT_UNIX_S + 1;
    policies.push(future);
    let mut expired = review_policy(&["independent-reviewer"]);
    expired.expires_at_unix_s = EVALUATED_AT_UNIX_S;
    policies.push(expired);
    let mut revoked = review_policy(&["independent-reviewer"]);
    revoked.revoked = true;
    policies.push(revoked);

    for policy in policies {
        let result = make_poster(&server).produce_review_admission_status(
            PR_NUMBER,
            HEAD_SHA,
            &policy,
            &producer(),
            EVALUATED_AT_UNIX_S,
        );
        assert!(
            matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("policy"))
        );
    }
    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &ReviewAdmissionProducer {
            github_app_id: 0,
            workload_identity: String::new(),
        },
        EVALUATED_AT_UNIX_S,
    );
    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("producer workload"))
    );

    pull.assert_hits(7);
    reviews.assert_hits(7);
    status.assert_hits(7);
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

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["same-user"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(&result, Err(KernelError::InvalidInput(message)) if message.contains("distinct")),
        "expected immutable author/reviewer distinction failure, got {result:?}"
    );
    pull.assert();
    reviews.assert();
    status.assert();
}

#[test]
fn mutable_login_cannot_substitute_for_an_eligible_immutable_principal() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100");
        then.status(200).json_body(json!([{
            "id": 9001,
            "state": "APPROVED",
            "commit_id": HEAD_SHA,
            "html_url": REVIEW_URL,
            "user": { "id": 77, "login": "independent-reviewer", "type": "Bot" }
        }]));
    });
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("eligible"))
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

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("head SHA"))
    );
    pull.assert();
    status.assert();
}

#[test]
fn final_head_change_posts_failure_without_any_success_status() {
    let first_pull = json!({
        "number": PR_NUMBER,
        "user": github_user("change-author"),
        "head": { "sha": HEAD_SHA }
    });
    let reviews = json!([{
        "id": 9001,
        "state": "APPROVED",
        "commit_id": HEAD_SHA,
        "html_url": REVIEW_URL,
        "user": github_user("independent-reviewer")
    }]);
    let moved_pull = json!({
        "number": PR_NUMBER,
        "user": github_user("change-author"),
        "head": { "sha": OTHER_SHA }
    });
    let responses = vec![first_pull, reviews, moved_pull]
        .into_iter()
        .map(|body| {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.to_string().len()
            )
        })
        .chain(std::iter::once(
            "HTTP/1.1 201 Created\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_owned(),
        ))
        .collect();
    let (api_base, requests, server) = scripted_http_server(responses);

    let result = make_poster_at(&api_base).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );
    server.join().expect("scripted server completes");
    let requests = requests.lock().expect("lock scripted request trace");

    assert!(
        matches!(&result, Err(KernelError::InvalidInput(message)) if message.contains("final PR-head readback")),
        "expected final-head mismatch failure, got {result:?}"
    );
    assert_eq!(requests.len(), 4);
    assert!(requests[0].starts_with("GET /repos/jason931225/oyatie/pulls/42 "));
    assert!(
        requests[1]
            .starts_with("GET /repos/jason931225/oyatie/pulls/42/reviews?per_page=100&page=1 ")
    );
    assert!(requests[2].starts_with("GET /repos/jason931225/oyatie/pulls/42 "));
    assert!(requests[3].starts_with(&format!(
        "POST /repos/jason931225/oyatie/statuses/{HEAD_SHA} "
    )));
    assert!(requests[3].contains("\"state\":\"failure\""));
    assert!(
        !requests
            .iter()
            .any(|request| request.contains("\"state\":\"success\""))
    );
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
                "user": github_user("independent-reviewer")
            },
            {
                "id": 9002,
                "state": "CHANGES_REQUESTED",
                "commit_id": HEAD_SHA,
                "html_url": "https://github.com/jason931225/oyatie/pull/42#pullrequestreview-9002",
                "user": github_user("independent-reviewer")
            }
        ]));
    });
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("APPROVED"))
    );
    pull.assert();
    reviews.assert();
    status.assert();
}

#[test]
fn distinct_but_unauthorized_reviewer_cannot_satisfy_admission() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = mock_reviews(&server, "drive-by-reviewer", HEAD_SHA, REVIEW_URL);
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["designated-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("eligible"))
    );
    pull.assert();
    reviews.assert();
    status.assert();
}

#[test]
fn page_two_changes_requested_supersedes_page_one_approval() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let page_one_reviews = (1..=100)
        .map(|id| {
            json!({
                "id": id,
                "state": "APPROVED",
                "commit_id": HEAD_SHA,
                "html_url": format!("https://github.com/jason931225/oyatie/pull/42#pullrequestreview-{id}"),
                "user": github_user("independent-reviewer")
            })
        })
        .collect::<Vec<_>>();
    let page_one = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100")
            .query_param("page", "1");
        then.status(200)
            .header(
                "Link",
                format!(
                    "<{}/repos/jason931225/oyatie/pulls/42/reviews?per_page=100&page=2>; rel=\"next\"",
                    server.base_url()
                ),
            )
            .json_body(json!(page_one_reviews));
    });
    let page_two = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100")
            .query_param("page", "2");
        then.status(200).json_body(json!([{
            "id": 101,
            "state": "CHANGES_REQUESTED",
            "commit_id": HEAD_SHA,
            "html_url": "https://github.com/jason931225/oyatie/pull/42#pullrequestreview-101",
            "user": github_user("independent-reviewer")
        }]));
    });
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("APPROVED"))
    );
    pull.assert();
    page_one.assert();
    page_two.assert();
    status.assert();
}

#[test]
fn spaced_next_relation_is_followed_even_when_title_mentions_next() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let page_one_reviews = (1..=100)
        .map(|id| {
            json!({
                "id": id,
                "state": "APPROVED",
                "commit_id": HEAD_SHA,
                "html_url": format!("https://github.com/jason931225/oyatie/pull/42#pullrequestreview-{id}"),
                "user": github_user("independent-reviewer")
            })
        })
        .collect::<Vec<_>>();
    let page_one = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100")
            .query_param("page", "1");
        then.status(200)
            .header(
                "Link",
                format!(
                    "<{}/repos/jason931225/oyatie/pulls/42/reviews?per_page=100&page=2>; title=\"rel=\\\"next\\\"\"; rel = \"next\"",
                    server.base_url()
                ),
            )
            .json_body(json!(page_one_reviews));
    });
    let page_two = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100")
            .query_param("page", "2");
        then.status(200).json_body(json!([{
            "id": 101,
            "state": "CHANGES_REQUESTED",
            "commit_id": HEAD_SHA,
            "html_url": "https://github.com/jason931225/oyatie/pull/42#pullrequestreview-101",
            "user": github_user("independent-reviewer")
        }]));
    });
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("APPROVED"))
    );
    pull.assert();
    page_one.assert();
    page_two.assert();
    status.assert();
}

#[test]
fn title_parameter_cannot_advertise_a_next_page() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100")
            .query_param("page", "1");
        then.status(200)
            .header(
                "Link",
                "<https://example.test/reviews?page=2>; title=\"rel=\\\"next\\\"\"",
            )
            .json_body(json!([{
                "id": 9001,
                "state": "APPROVED",
                "commit_id": HEAD_SHA,
                "html_url": REVIEW_URL,
                "user": github_user("independent-reviewer")
            }]));
    });
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"success","context":"oya-pr-review"}"#);
        then.status(201);
    });

    make_poster(&server)
        .produce_review_admission_status(
            PR_NUMBER,
            HEAD_SHA,
            &review_policy(&["independent-reviewer"]),
            &producer(),
            EVALUATED_AT_UNIX_S,
        )
        .expect("a title parameter is not a pagination relation");

    pull.assert_hits(2);
    reviews.assert();
    status.assert();
}

#[test]
fn more_than_one_page_without_next_link_fails_closed() {
    let server = MockServer::start();
    let pull = mock_pull(&server, "change-author", HEAD_SHA);
    let reviews = (1..=101)
        .map(|id| {
            json!({
                "id": id,
                "state": "APPROVED",
                "commit_id": HEAD_SHA,
                "html_url": format!("https://github.com/jason931225/oyatie/pull/42#pullrequestreview-{id}"),
                "user": github_user("independent-reviewer")
            })
        })
        .collect::<Vec<_>>();
    let review_request = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews")
            .query_param("per_page", "100")
            .query_param("page", "1");
        then.status(200).json_body(json!(reviews));
    });
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"error","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::DownstreamTransport(message)) if message.contains("completeness"))
    );
    pull.assert();
    review_request.assert();
    status.assert();
}

#[test]
fn mismatched_pr_number_does_not_fetch_reviews() {
    let server = MockServer::start();
    let pull = server.mock(|when, then| {
        when.method(GET).path("/repos/jason931225/oyatie/pulls/42");
        then.status(200).json_body(json!({
            "number": 99,
            "html_url": "https://github.com/jason931225/oyatie/pull/99",
            "user": github_user("change-author"),
            "head": { "sha": HEAD_SHA }
        }));
    });
    let reviews = server.mock(|when, then| {
        when.method(GET)
            .path("/repos/jason931225/oyatie/pulls/42/reviews");
        then.status(200).json_body(json!([]));
    });
    let status = server.mock(|when, then| {
        when.method(POST)
            .path(format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}"))
            .json_body_partial(r#"{"state":"failure","context":"oya-pr-review"}"#);
        then.status(201);
    });

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(result, Err(KernelError::InvalidInput(message)) if message.contains("PR mismatch"))
    );
    pull.assert();
    reviews.assert_hits(0);
    status.assert();
}

#[test]
fn configured_request_timeout_stops_a_stalled_blocking_github_call() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind stalled HTTP server");
    let address = listener.local_addr().expect("read stalled server address");
    listener
        .set_nonblocking(true)
        .expect("configure bounded stalled HTTP server");
    let (stop_tx, stop_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let accept_deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match listener.accept() {
                Ok((_stream, _)) => {
                    let _ = stop_rx.recv_timeout(Duration::from_secs(2));
                    return;
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    if stop_rx.try_recv().is_ok() || Instant::now() >= accept_deadline {
                        return;
                    }
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept stalled HTTP request: {error}"),
            }
        }
    });
    let poster = GitHubCommitStatusPoster::new("jason931225", "oyatie", "test-token")
        .with_api_base(&format!("http://{address}"))
        .with_request_timeout(Duration::from_millis(100));

    let started = Instant::now();
    let result = poster.post(
        HEAD_SHA,
        CommitState::Failure,
        "oya-pr-review",
        "timeout regression",
        None,
    );
    let elapsed = started.elapsed();
    let _ = stop_tx.send(());
    server.join().expect("join stalled HTTP server");

    assert!(
        matches!(result, Err(KernelError::DownstreamTransport(_))),
        "a stalled GitHub request must fail closed, got {result:?}"
    );
    assert!(
        elapsed < Duration::from_secs(1),
        "configured request timeout was not enforced: {elapsed:?}"
    );
}
