//! Trusted GitHub review-admission producer tests.
//!
//! Ported off `httpmock` onto the first-party `scripted-http-server` (ADR-0709 D-6
//! Rule 2). This file already contained BOTH styles — `scripted_http_server` (now
//! promoted into the shared crate) and httpmock — because neither alone covered it, so
//! it is the file that defined what the shared helper had to be able to do.
//!
//! The GitHub API is a set of ENDPOINTS rather than a fixed call sequence, so most tests
//! port onto content routing (`GitHubApi` below): one route per endpoint, which is
//! exactly the set of mocks it replaces. `final_head_change_posts_failure_without_any_success_status`
//! stays positional, because a PR head that MOVES between two reads of the same endpoint
//! is a sequence property, not a routing one.
//!
//! What the port makes stronger:
//!   * `mock_pull`'s four header matchers (Authorization / X-GitHub-Api-Version / Accept /
//!     User-Agent) become assertions on every recorded request. A matcher only selects a
//!     mock — if the adapter stopped sending `Authorization`, httpmock would have fallen
//!     through to "no matching mock" rather than reporting a missing credential.
//!   * `assert_hits(N)` becomes a count over the requests that actually arrived, and the
//!     TOCTOU guard (`pull.assert_hits(2)` — the head is re-read AFTER the reviews call
//!     and BEFORE any success status) additionally asserts the ORDER of those calls,
//!     which order-independent matchers could not express.
//!   * Every test asserts its total request count, so a stray call to an endpoint nobody
//!     scripted fails the test instead of 404-ing into silence.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::{
    collections::VecDeque,
    io::ErrorKind,
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
use scripted_http_server::{RecordedRequest, ScriptedResponse, ScriptedServer};
use serde_json::json;

const PR_NUMBER: u64 = 42;
const HEAD_SHA: &str = "abcdef1234567890abcdef1234567890abcdef12";
const OTHER_SHA: &str = "1234567890abcdef1234567890abcdef12345678";
const REVIEW_URL: &str = "https://github.com/jason931225/oyatie/pull/42#pullrequestreview-9001";
const EVALUATED_AT_UNIX_S: i64 = 1_700_000_000;

const PULL_PATH: &str = "/repos/jason931225/oyatie/pulls/42";
const REVIEWS_PATH: &str = "/repos/jason931225/oyatie/pulls/42/reviews";

fn status_path() -> String {
    format!("/repos/jason931225/oyatie/statuses/{HEAD_SHA}")
}

fn make_poster(server: &ScriptedServer) -> GitHubCommitStatusPoster {
    make_poster_at(server.base_url())
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

// ---------------------------------------------------------------------------
// A routed GitHub API — the port target for this file's httpmock mocks.
// ---------------------------------------------------------------------------

/// One endpoint's script. `responses` holds a queue; the LAST entry repeats forever, so
/// `on(..)` (a single response) answers every call to that endpoint exactly as a
/// standing httpmock mock did, while `on_sequence(..)` scripts an endpoint whose answer
/// CHANGES between calls.
struct RouteEntry {
    method: &'static str,
    path: String,
    query: Vec<(String, String)>,
    responses: VecDeque<ScriptedResponse>,
}

/// Routes are registered on a shared table so they can be added AFTER the server is
/// bound — the paginated tests need the server's own base URL inside a `Link` header,
/// which does not exist until the listener is up.
#[derive(Clone, Default)]
struct GitHubApi {
    routes: Arc<Mutex<Vec<RouteEntry>>>,
}

impl GitHubApi {
    fn new() -> Self {
        Self::default()
    }

    fn serve(&self) -> ScriptedServer {
        let routes = Arc::clone(&self.routes);
        ScriptedServer::start_with(move |request| {
            let mut routes = routes.lock().expect("route table poisoned");
            let matched = routes.iter_mut().find(|route| {
                route.method == request.method
                    && route.path == request.path()
                    && route.query.iter().all(|(name, value)| {
                        request.query_param(name).as_deref() == Some(value.as_str())
                    })
            });
            match matched {
                Some(route) if route.responses.len() > 1 => route
                    .responses
                    .pop_front()
                    .expect("checked non-empty above"),
                Some(route) => route
                    .responses
                    .front()
                    .cloned()
                    .expect("a route always holds at least one response"),
                // Recorded, and visible as a 404 rather than as silence.
                None => ScriptedResponse::status(404).text(format!(
                    "no GitHub route for {} {}",
                    request.method, request.target
                )),
            }
        })
    }

    fn on(&self, method: &'static str, path: impl Into<String>, response: ScriptedResponse) {
        self.register(
            method,
            path.into(),
            Vec::new(),
            VecDeque::from(vec![response]),
        );
    }

    fn on_query(
        &self,
        method: &'static str,
        path: impl Into<String>,
        query: &[(&str, &str)],
        response: ScriptedResponse,
    ) {
        self.register(
            method,
            path.into(),
            query
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                .collect(),
            VecDeque::from(vec![response]),
        );
    }

    fn register(
        &self,
        method: &'static str,
        path: String,
        query: Vec<(String, String)>,
        responses: VecDeque<ScriptedResponse>,
    ) {
        self.routes
            .lock()
            .expect("route table poisoned")
            .push(RouteEntry {
                method,
                path,
                query,
                responses,
            });
    }
}

fn json_ok(body: serde_json::Value) -> ScriptedResponse {
    ScriptedResponse::ok().json(&body)
}

/// The `pull` mock every test shared, as a route.
fn pull_body(author: &str, head_sha: &str) -> serde_json::Value {
    json!({
        "number": PR_NUMBER,
        "html_url": "https://github.com/jason931225/oyatie/pull/42",
        "user": github_user(author),
        "head": { "sha": head_sha }
    })
}

/// The single-APPROVED-review `reviews` mock every test shared, as a body.
fn reviews_body(reviewer: &str, commit_id: &str, evidence_url: &str) -> serde_json::Value {
    json!([{
        "id": 9001,
        "state": "APPROVED",
        "commit_id": commit_id,
        "html_url": evidence_url,
        "user": github_user(reviewer)
    }])
}

/// The port of `mock.assert_hits(n)`.
fn hits(server: &ScriptedServer, method: &str, path: &str) -> usize {
    server
        .requests()
        .iter()
        .filter(|request| request.method == method && request.path() == path)
        .count()
}

fn hits_page(server: &ScriptedServer, path: &str, page: &str) -> usize {
    server
        .requests()
        .iter()
        .filter(|request| {
            request.method == "GET"
                && request.path() == path
                && request.query_param("page").as_deref() == Some(page)
        })
        .count()
}

/// `mock_pull`'s four header matchers, as assertions over every recorded request.
fn assert_github_headers(server: &ScriptedServer) {
    for request in server.requests() {
        assert_eq!(
            request.header("authorization"),
            Some("Bearer test-token"),
            "every GitHub call must carry the app credential: {}",
            request.line()
        );
        assert_eq!(request.header("x-github-api-version"), Some("2022-11-28"));
        assert_eq!(
            request.header("accept"),
            Some("application/vnd.github+json")
        );
        assert_eq!(request.header("user-agent"), Some("oya-ci-controller"));
    }
}

/// The port of `json_body_partial(r#"{"state":"...","context":"oya-pr-review"}"#)`.
fn assert_status_state(request: &RecordedRequest, state: &str) {
    let body = request.json();
    assert_eq!(
        body["state"],
        json!(state),
        "posted status body: {}",
        request.body_string()
    );
    assert_eq!(body["context"], json!("oya-pr-review"));
}

/// Every status this run posted, in order.
fn posted_statuses(server: &ScriptedServer) -> Vec<RecordedRequest> {
    server
        .requests()
        .into_iter()
        .filter(|request| request.method == "POST" && request.path() == status_path())
        .collect()
}

/// The invariant no test may lose: a success status is NEVER posted on a rejected path.
fn assert_no_success_status_posted(server: &ScriptedServer) {
    for status in posted_statuses(server) {
        assert_ne!(
            status.json()["state"],
            json!("success"),
            "a rejected admission must never post a success status: {}",
            status.body_string()
        );
    }
}

/// Register the standard three endpoints: pull, single-review reviews page, status.
fn standard_api(
    author: &str,
    head_sha: &str,
    reviews: serde_json::Value,
) -> (GitHubApi, ScriptedServer) {
    let api = GitHubApi::new();
    api.on("GET", PULL_PATH, json_ok(pull_body(author, head_sha)));
    api.on_query(
        "GET",
        REVIEWS_PATH,
        &[("per_page", "100")],
        json_ok(reviews),
    );
    api.on("POST", status_path(), ScriptedResponse::status(201));
    let server = api.serve();
    (api, server)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn approved_distinct_reviewer_rechecks_head_before_posting_status() {
    let (_api, server) = standard_api(
        "change-author",
        HEAD_SHA,
        reviews_body("independent-reviewer", HEAD_SHA, REVIEW_URL),
    );

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

    // The TOCTOU guard. `pull.assert_hits(2)` said the head was read twice; this also
    // says WHERE the second read sits — after the reviews call and before the status —
    // which is the whole point of the re-read and which the order-independent matcher
    // could not express.
    assert_eq!(
        server.request_lines(),
        vec![
            format!("GET {PULL_PATH}"),
            format!("GET {REVIEWS_PATH}?per_page=100&page=1"),
            format!("GET {PULL_PATH}"),
            format!("POST {}", status_path()),
        ]
    );
    assert_github_headers(&server);

    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    // Was a `json_body(..)` exact matcher on the status mock.
    assert_eq!(
        statuses[0].json(),
        json!({
            "state": "success",
            "context": "oya-pr-review",
            "description": "oya-pr-review approved by independent-reviewer",
            "target_url": REVIEW_URL
        })
    );
}

#[test]
fn unchanged_digest_cannot_authorize_a_tampered_reviewer_allowlist() {
    let (_api, server) = standard_api(
        "change-author",
        HEAD_SHA,
        reviews_body("independent-reviewer", HEAD_SHA, REVIEW_URL),
    );
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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 3, "{:?}", server.request_lines());
}

#[test]
fn missing_durable_review_url_is_rejected_and_posts_failure() {
    let (_api, server) = standard_api(
        "change-author",
        HEAD_SHA,
        reviews_body("independent-reviewer", HEAD_SHA, ""),
    );

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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 3, "{:?}", server.request_lines());
}

#[test]
fn evidence_url_must_bind_the_configured_repository_pr_and_review() {
    let (_api, server) = standard_api(
        "change-author",
        HEAD_SHA,
        reviews_body(
            "independent-reviewer",
            HEAD_SHA,
            "https://github.com/other/repository/pull/42#pullrequestreview-9001",
        ),
    );

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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 3, "{:?}", server.request_lines());
}

#[test]
fn malformed_eligible_approval_does_not_veto_another_valid_eligible_approval() {
    let (_api, server) = standard_api(
        "change-author",
        HEAD_SHA,
        json!([
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
        ]),
    );

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
    // Two pull reads bracketing the reviews call: the TOCTOU guard runs on the success
    // path here too.
    assert_eq!(hits(&server, "GET", PULL_PATH), 2);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_eq!(
        statuses[0].json(),
        json!({
            "state": "success",
            "context": "oya-pr-review",
            "description": "oya-pr-review approved by designated-reviewer",
            "target_url": REVIEW_URL
        })
    );
    assert_eq!(server.request_count(), 4, "{:?}", server.request_lines());
}

#[test]
fn policy_receipt_and_producer_identity_fail_closed_when_incomplete_or_invalid() {
    let (_api, server) = standard_api(
        "change-author",
        HEAD_SHA,
        reviews_body("independent-reviewer", HEAD_SHA, REVIEW_URL),
    );

    // Each entry is (label, policy) so a failure names WHICH fail-closed case broke —
    // the httpmock original pushed unlabelled policies into a Vec, so one silently
    // passing case was indistinguishable from another.
    let mut policies: Vec<(&str, ReviewAdmissionPolicy)> = Vec::new();
    let mut missing_version = review_policy(&["independent-reviewer"]);
    missing_version.version.clear();
    policies.push(("blank version", missing_version));
    let mut bad_digest = review_policy(&["independent-reviewer"]);
    bad_digest.sha256_digest = "not-a-digest".to_owned();
    policies.push(("non-digest sha256", bad_digest));
    let mut missing_issuer = review_policy(&["independent-reviewer"]);
    missing_issuer.issuer.clear();
    policies.push(("blank issuer", missing_issuer));
    let mut future = review_policy(&["independent-reviewer"]);
    future.effective_at_unix_s = EVALUATED_AT_UNIX_S + 1;
    policies.push(("not yet effective", future));
    let mut expired = review_policy(&["independent-reviewer"]);
    expired.expires_at_unix_s = EVALUATED_AT_UNIX_S;
    policies.push(("expired", expired));
    let mut revoked = review_policy(&["independent-reviewer"]);
    revoked.revoked = true;
    policies.push(("revoked", revoked));

    let policy_case_count = policies.len();
    for (index, (label, policy)) in policies.into_iter().enumerate() {
        let result = make_poster(&server).produce_review_admission_status(
            PR_NUMBER,
            HEAD_SHA,
            &policy,
            &producer(),
            EVALUATED_AT_UNIX_S,
        );
        assert!(
            matches!(&result, Err(KernelError::InvalidInput(message)) if message.contains("policy")),
            "policy case '{label}' must fail closed, got {result:?}"
        );
        // Each case must have driven its OWN pull/reviews/status round trip. Asserting
        // inside the loop is what stops six of the seven cases from going unchecked if
        // the loop ever stops iterating.
        assert_eq!(
            server.request_count(),
            (index + 1) * 3,
            "policy case '{label}' did not drive a full pull/reviews/status round trip: {:?}",
            server.request_lines()
        );
        assert_no_success_status_posted(&server);
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
        matches!(&result, Err(KernelError::InvalidInput(message)) if message.contains("producer workload")),
        "a zero-id / blank-workload producer must fail closed, got {result:?}"
    );

    let iterations = policy_case_count + 1;
    assert_eq!(iterations, 7);
    assert_eq!(hits(&server, "GET", PULL_PATH), iterations);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), iterations);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), iterations);
    for status in &statuses {
        assert_status_state(status, "failure");
    }
    assert_eq!(server.request_count(), iterations * 3);
}

#[test]
fn author_cannot_satisfy_review_admission_and_failure_is_posted() {
    let (_api, server) = standard_api(
        "same-user",
        HEAD_SHA,
        reviews_body("SAME-USER", HEAD_SHA, REVIEW_URL),
    );

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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 3, "{:?}", server.request_lines());
}

#[test]
fn mutable_login_cannot_substitute_for_an_eligible_immutable_principal() {
    let (_api, server) = standard_api(
        "change-author",
        HEAD_SHA,
        json!([{
            "id": 9001,
            "state": "APPROVED",
            "commit_id": HEAD_SHA,
            "html_url": REVIEW_URL,
            "user": { "id": 77, "login": "independent-reviewer", "type": "Bot" }
        }]),
    );

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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 3, "{:?}", server.request_lines());
}

#[test]
fn pull_head_mismatch_is_rejected_before_review_evidence_can_pass() {
    let api = GitHubApi::new();
    api.on(
        "GET",
        PULL_PATH,
        json_ok(pull_body("change-author", OTHER_SHA)),
    );
    // Deliberately routed so that a reviews fetch would SUCCEED — the assertion below
    // proves the adapter never makes it, rather than proving a 404 stopped it.
    api.on_query(
        "GET",
        REVIEWS_PATH,
        &[("per_page", "100")],
        json_ok(reviews_body("independent-reviewer", OTHER_SHA, REVIEW_URL)),
    );
    api.on("POST", status_path(), ScriptedResponse::status(201));
    let server = api.serve();

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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(
        hits(&server, "GET", REVIEWS_PATH),
        0,
        "a head mismatch must be rejected BEFORE reviews are fetched: {:?}",
        server.request_lines()
    );
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 2, "{:?}", server.request_lines());
}

#[test]
fn final_head_change_posts_failure_without_any_success_status() {
    // Positional, not routed: the property under test is that the SAME endpoint answers
    // differently on its second read, which is a sequence, not a route.
    let server = ScriptedServer::start(vec![
        json_ok(json!({
            "number": PR_NUMBER,
            "user": github_user("change-author"),
            "head": { "sha": HEAD_SHA }
        })),
        json_ok(json!([{
            "id": 9001,
            "state": "APPROVED",
            "commit_id": HEAD_SHA,
            "html_url": REVIEW_URL,
            "user": github_user("independent-reviewer")
        }])),
        json_ok(json!({
            "number": PR_NUMBER,
            "user": github_user("change-author"),
            "head": { "sha": OTHER_SHA }
        })),
        ScriptedResponse::status(201),
    ]);

    let result = make_poster_at(server.base_url()).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(&result, Err(KernelError::InvalidInput(message)) if message.contains("final PR-head readback")),
        "expected final-head mismatch failure, got {result:?}"
    );

    let requests = server.requests();
    assert_eq!(requests.len(), 4, "{:?}", server.request_lines());
    assert_eq!(
        server.request_lines(),
        vec![
            format!("GET {PULL_PATH}"),
            format!("GET {REVIEWS_PATH}?per_page=100&page=1"),
            format!("GET {PULL_PATH}"),
            format!("POST {}", status_path()),
        ]
    );
    assert_status_state(&requests[3], "failure");
    assert_no_success_status_posted(&server);
    assert!(
        !requests
            .iter()
            .any(|request| request.body_string().contains("\"state\":\"success\"")),
        "no request body on this path may carry a success state"
    );
}

#[test]
fn newer_changes_requested_verdict_supersedes_older_approval() {
    let (_api, server) = standard_api(
        "change-author",
        HEAD_SHA,
        json!([
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
        ]),
    );

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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 3, "{:?}", server.request_lines());
}

#[test]
fn distinct_but_unauthorized_reviewer_cannot_satisfy_admission() {
    let (_api, server) = standard_api(
        "change-author",
        HEAD_SHA,
        reviews_body("drive-by-reviewer", HEAD_SHA, REVIEW_URL),
    );

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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 3, "{:?}", server.request_lines());
}

/// 100 APPROVED reviews on page one, one CHANGES_REQUESTED on page two.
fn hundred_approvals() -> serde_json::Value {
    json!(
        (1..=100)
            .map(|id| {
                json!({
                    "id": id,
                    "state": "APPROVED",
                    "commit_id": HEAD_SHA,
                    "html_url": format!("https://github.com/jason931225/oyatie/pull/42#pullrequestreview-{id}"),
                    "user": github_user("independent-reviewer")
                })
            })
            .collect::<Vec<_>>()
    )
}

fn page_two_changes_requested() -> serde_json::Value {
    json!([{
        "id": 101,
        "state": "CHANGES_REQUESTED",
        "commit_id": HEAD_SHA,
        "html_url": "https://github.com/jason931225/oyatie/pull/42#pullrequestreview-101",
        "user": github_user("independent-reviewer")
    }])
}

/// Both pagination tests differ only in the shape of the `Link` header, so they share
/// everything else. `link` is built from the server's own base URL, which is why routes
/// are registered AFTER `serve()`.
fn assert_next_relation_is_followed(link_for: fn(&str) -> String) {
    let api = GitHubApi::new();
    let server = api.serve();

    api.on(
        "GET",
        PULL_PATH,
        json_ok(pull_body("change-author", HEAD_SHA)),
    );
    api.on_query(
        "GET",
        REVIEWS_PATH,
        &[("per_page", "100"), ("page", "1")],
        json_ok(hundred_approvals()).header("Link", link_for(server.base_url())),
    );
    api.on_query(
        "GET",
        REVIEWS_PATH,
        &[("per_page", "100"), ("page", "2")],
        json_ok(page_two_changes_requested()),
    );
    api.on("POST", status_path(), ScriptedResponse::status(201));

    let result = make_poster(&server).produce_review_admission_status(
        PR_NUMBER,
        HEAD_SHA,
        &review_policy(&["independent-reviewer"]),
        &producer(),
        EVALUATED_AT_UNIX_S,
    );

    assert!(
        matches!(&result, Err(KernelError::InvalidInput(message)) if message.contains("APPROVED")),
        "page two's CHANGES_REQUESTED must supersede all 100 page-one approvals: {result:?}"
    );
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(hits_page(&server, REVIEWS_PATH, "1"), 1);
    assert_eq!(
        hits_page(&server, REVIEWS_PATH, "2"),
        1,
        "the rel=next relation was not followed: {:?}",
        server.request_lines()
    );
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 4, "{:?}", server.request_lines());
}

#[test]
fn page_two_changes_requested_supersedes_page_one_approval() {
    assert_next_relation_is_followed(|base_url| {
        format!("<{base_url}{REVIEWS_PATH}?per_page=100&page=2>; rel=\"next\"")
    });
}

#[test]
fn spaced_next_relation_is_followed_even_when_title_mentions_next() {
    assert_next_relation_is_followed(|base_url| {
        format!(
            "<{base_url}{REVIEWS_PATH}?per_page=100&page=2>; title=\"rel=\\\"next\\\"\"; rel = \"next\""
        )
    });
}

#[test]
fn title_parameter_cannot_advertise_a_next_page() {
    let api = GitHubApi::new();
    api.on(
        "GET",
        PULL_PATH,
        json_ok(pull_body("change-author", HEAD_SHA)),
    );
    api.on_query(
        "GET",
        REVIEWS_PATH,
        &[("per_page", "100"), ("page", "1")],
        json_ok(reviews_body("independent-reviewer", HEAD_SHA, REVIEW_URL)).header(
            "Link",
            "<https://example.test/reviews?page=2>; title=\"rel=\\\"next\\\"\"",
        ),
    );
    api.on("POST", status_path(), ScriptedResponse::status(201));
    let server = api.serve();

    make_poster(&server)
        .produce_review_admission_status(
            PR_NUMBER,
            HEAD_SHA,
            &review_policy(&["independent-reviewer"]),
            &producer(),
            EVALUATED_AT_UNIX_S,
        )
        .expect("a title parameter is not a pagination relation");

    assert_eq!(hits(&server, "GET", PULL_PATH), 2);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    assert_eq!(
        hits_page(&server, REVIEWS_PATH, "2"),
        0,
        "a title parameter must never advertise a next page: {:?}",
        server.request_lines()
    );
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "success");
    assert_eq!(server.request_count(), 4, "{:?}", server.request_lines());
}

#[test]
fn more_than_one_page_without_next_link_fails_closed() {
    let over_a_page = json!(
        (1..=101)
            .map(|id| {
                json!({
                    "id": id,
                    "state": "APPROVED",
                    "commit_id": HEAD_SHA,
                    "html_url": format!("https://github.com/jason931225/oyatie/pull/42#pullrequestreview-{id}"),
                    "user": github_user("independent-reviewer")
                })
            })
            .collect::<Vec<_>>()
    );

    let api = GitHubApi::new();
    api.on(
        "GET",
        PULL_PATH,
        json_ok(pull_body("change-author", HEAD_SHA)),
    );
    api.on_query(
        "GET",
        REVIEWS_PATH,
        &[("per_page", "100"), ("page", "1")],
        json_ok(over_a_page),
    );
    api.on("POST", status_path(), ScriptedResponse::status(201));
    let server = api.serve();

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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(hits(&server, "GET", REVIEWS_PATH), 1);
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "error");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 3, "{:?}", server.request_lines());
}

#[test]
fn mismatched_pr_number_does_not_fetch_reviews() {
    let api = GitHubApi::new();
    api.on(
        "GET",
        PULL_PATH,
        json_ok(json!({
            "number": 99,
            "html_url": "https://github.com/jason931225/oyatie/pull/99",
            "user": github_user("change-author"),
            "head": { "sha": HEAD_SHA }
        })),
    );
    // Routed to SUCCEED, so the assertion below proves the adapter chose not to call it.
    api.on_query(
        "GET",
        REVIEWS_PATH,
        &[("per_page", "100")],
        json_ok(json!([])),
    );
    api.on("POST", status_path(), ScriptedResponse::status(201));
    let server = api.serve();

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
    assert_eq!(hits(&server, "GET", PULL_PATH), 1);
    assert_eq!(
        hits(&server, "GET", REVIEWS_PATH),
        0,
        "a PR-number mismatch must never fetch reviews: {:?}",
        server.request_lines()
    );
    assert!(
        !server
            .request_lines()
            .iter()
            .any(|line| line.contains("/reviews")),
        "no request may touch the reviews endpoint: {:?}",
        server.request_lines()
    );
    let statuses = posted_statuses(&server);
    assert_eq!(statuses.len(), 1);
    assert_status_state(&statuses[0], "failure");
    assert_no_success_status_posted(&server);
    assert_eq!(server.request_count(), 2, "{:?}", server.request_lines());
}

#[test]
fn configured_request_timeout_stops_a_stalled_blocking_github_call() {
    // Hand-rolled rather than scripted: this test needs a peer that ACCEPTS and then
    // never answers, which is the one behaviour a response-scripted server cannot offer.
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
