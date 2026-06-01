//! # oya-scm-forgejo
//!
//! Forgejo adapter implementing the [`Scm`] trait (ADR-0517).
//!
//! This crate is the **default/interim** SCM backend for the Oyatie CI stack.
//! All Forgejo-specific HTTP calls are isolated here; CI kernels depend only on
//! `oya-scm::Scm` and receive a `ForgejoScm` at construction time.
//!
//! ## Reuse of existing Forgejo client logic
//!
//! The gateway and tide crates each had bespoke Forgejo HTTP calls embedded
//! directly in their kernel/adapter code. `ForgejoScm` centralises that logic:
//!
//! - PR list / get → `GET /api/v1/repos/{owner}/{repo}/pulls[/{index}]`
//! - Combined status → `GET /api/v1/repos/{owner}/{repo}/commits/{sha}/statuses`
//! - Post status → `POST /api/v1/repos/{owner}/{repo}/statuses/{sha}`
//! - Reviews → `GET /api/v1/repos/{owner}/{repo}/pulls/{index}/reviews`
//! - Merge → `POST /api/v1/repos/{owner}/{repo}/pulls/{index}/merge`
//! - Ref resolution → `GET /api/v1/repos/{owner}/{repo}/git/refs/{ref}`
//! - Webhook parse → re-uses the same JSON shapes from `oya-ci-webhook-gateway-kernel`
//!   (the `RawPrPayload` logic is replicated here as neutral types, not re-exported)
//! - Branch protection → `GET /api/v1/repos/{owner}/{repo}/branch_protections`
//!
//! ## Authentication
//!
//! `ForgejoScm` accepts a [`ScmToken`] (API token) and sends it as
//! `Authorization: token <value>` on every request, matching Forgejo's token
//! auth scheme.
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path. HTTP errors map to
//! `ScmError::Transport` or `ScmError::UnexpectedResponse`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_scm::{
    BranchProtection, CombinedCommitStatus, CommitState, CommitStatus, CommitStatusEvent,
    MergeMethod, Mergeability, PostStatusRequest, PullRequest, PullRequestAction,
    PullRequestEvent, RepoCoords, Review, ReviewState, Scm, ScmError, ScmEvent, ScmToken,
    Result,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ForgejoScm — the adapter
// ---------------------------------------------------------------------------

/// Forgejo SCM adapter implementing `oya_scm::Scm`.
///
/// Constructed with a base URL and an API token. Uses `reqwest` blocking HTTP.
pub struct ForgejoScm {
    /// Base URL of the Forgejo instance, e.g. `"https://forgejo.example.com"`.
    base_url: String, // data_class: INTERNAL_ONLY
    /// Forgejo API token.
    token: ScmToken, // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client,
}

impl ForgejoScm {
    /// Construct a new `ForgejoScm`.
    ///
    /// `base_url` should have no trailing slash, e.g.
    /// `"https://forgejo.internal"`.
    pub fn new(base_url: impl Into<String>, token: ScmToken) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            token,
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Override the base URL (useful in tests pointing at a mock server).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_owned();
        self
    }

    // ---- URL helpers -------------------------------------------------------

    fn api(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    fn pulls_url(&self, repo: &RepoCoords) -> String {
        self.api(&format!("/repos/{}/{}/pulls", repo.owner, repo.name))
    }

    fn pull_url(&self, repo: &RepoCoords, number: u64) -> String {
        self.api(&format!("/repos/{}/{}/pulls/{}", repo.owner, repo.name, number))
    }

    fn statuses_url(&self, repo: &RepoCoords, sha: &str) -> String {
        self.api(&format!(
            "/repos/{}/{}/commits/{}/statuses",
            repo.owner, repo.name, sha
        ))
    }

    fn post_status_url(&self, repo: &RepoCoords, sha: &str) -> String {
        self.api(&format!(
            "/repos/{}/{}/statuses/{}",
            repo.owner, repo.name, sha
        ))
    }

    fn reviews_url(&self, repo: &RepoCoords, pr_number: u64) -> String {
        self.api(&format!(
            "/repos/{}/{}/pulls/{}/reviews",
            repo.owner, repo.name, pr_number
        ))
    }

    fn merge_url(&self, repo: &RepoCoords, pr_number: u64) -> String {
        self.api(&format!(
            "/repos/{}/{}/pulls/{}/merge",
            repo.owner, repo.name, pr_number
        ))
    }

    fn git_ref_url(&self, repo: &RepoCoords, git_ref: &str) -> String {
        self.api(&format!(
            "/repos/{}/{}/git/refs/{}",
            repo.owner, repo.name, git_ref
        ))
    }

    fn branch_protections_url(&self, repo: &RepoCoords) -> String {
        self.api(&format!(
            "/repos/{}/{}/branch_protections",
            repo.owner, repo.name
        ))
    }

    // ---- Auth header -------------------------------------------------------

    fn auth_header(&self) -> String {
        format!("token {}", self.token.as_str())
    }
}

// ---------------------------------------------------------------------------
// Forgejo API response shapes (serde)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ForgejoPull {
    number: u64,
    title: String,
    state: String,
    draft: Option<bool>,
    head: ForgejoRef,
    base: ForgejoRef,
    mergeable: Option<bool>,
}

#[derive(Deserialize)]
struct ForgejoRef {
    #[serde(rename = "ref")]
    ref_name: String,
    sha: String,
    repo: Option<ForgejoRepo>,
}

#[derive(Deserialize)]
struct ForgejoRepo {
    owner: ForgejoUser,
    name: String,
}

#[derive(Deserialize)]
struct ForgejoUser {
    login: String,
}

#[derive(Deserialize)]
struct ForgejoStatus {
    context: String,
    state: String,
    description: Option<String>,
    target_url: Option<String>,
}

#[derive(Serialize)]
struct ForgejoPostStatusBody<'a> {
    context: &'a str,
    state: &'a str,
    description: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_url: Option<&'a str>,
}

#[derive(Deserialize)]
struct ForgejoReview {
    id: u64,
    user: ForgejoUser,
    #[serde(rename = "type")]
    review_type: String,
}

#[derive(Serialize)]
struct ForgejoMergeBody<'a> {
    #[serde(rename = "Do")]
    do_action: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    merge_message_field: Option<&'a str>,
}

#[derive(Deserialize)]
struct ForgejoGitRef {
    #[serde(rename = "ref")]
    ref_name: String,
    object: ForgejoGitObject,
}

#[derive(Deserialize)]
struct ForgejoGitObject {
    sha: String,
}

#[derive(Deserialize)]
struct ForgejoBranchProtection {
    branch_name: Option<String>,
    require_signed_commits: Option<bool>,
    enable_push: Option<bool>,
    required_approvals: Option<i64>,
    status_check_contexts: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Helper: map Forgejo state string → CommitState
// ---------------------------------------------------------------------------

fn map_forgejo_state(s: &str) -> CommitState {
    match s {
        "pending" => CommitState::Pending,
        "success" => CommitState::Success,
        "failure" => CommitState::Failure,
        _ => CommitState::Error,
    }
}

// ---------------------------------------------------------------------------
// Helper: map ForgejoPull → PullRequest
// ---------------------------------------------------------------------------

fn map_pull(raw: ForgejoPull) -> PullRequest {
    let mergeability = match raw.mergeable {
        Some(true) => Mergeability::Mergeable,
        Some(false) => Mergeability::Conflicted,
        None => Mergeability::Unknown,
    };

    let (owner, repo_name) = raw
        .head
        .repo
        .as_ref()
        .map(|r| (r.owner.login.clone(), r.name.clone()))
        .unwrap_or_default();

    PullRequest {
        number: raw.number,
        title: raw.title,
        state: raw.state,
        draft: raw.draft.unwrap_or(false),
        head_sha: raw.head.sha,
        head_ref: raw.head.ref_name,
        base_sha: raw.base.sha,
        base_ref: raw.base.ref_name,
        mergeability,
        repo: RepoCoords::new(owner, repo_name),
    }
}

// ---------------------------------------------------------------------------
// Scm impl for ForgejoScm
// ---------------------------------------------------------------------------

impl Scm for ForgejoScm {
    fn list_open_pulls(&self, repo: &RepoCoords, base_ref: &str) -> Result<Vec<PullRequest>> {
        let url = format!("{}?state=open&base={}&limit=50", self.pulls_url(repo), base_ref);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| ScmError::Transport(format!("list_open_pulls: {e}")))?;

        if resp.status().as_u16() == 401 {
            return Err(ScmError::Unauthorized);
        }
        if !resp.status().is_success() {
            return Err(ScmError::Transport(format!(
                "list_open_pulls: HTTP {}",
                resp.status()
            )));
        }

        let pulls: Vec<ForgejoPull> = resp
            .json()
            .map_err(|e| ScmError::UnexpectedResponse(format!("list_open_pulls parse: {e}")))?;

        Ok(pulls.into_iter().map(map_pull).collect())
    }

    fn get_pull(&self, repo: &RepoCoords, number: u64) -> Result<PullRequest> {
        let url = self.pull_url(repo, number);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| ScmError::Transport(format!("get_pull: {e}")))?;

        if resp.status().as_u16() == 404 {
            return Err(ScmError::NotFound(format!("pull #{number}")));
        }
        if resp.status().as_u16() == 401 {
            return Err(ScmError::Unauthorized);
        }
        if !resp.status().is_success() {
            return Err(ScmError::Transport(format!(
                "get_pull: HTTP {}",
                resp.status()
            )));
        }

        let raw: ForgejoPull = resp
            .json()
            .map_err(|e| ScmError::UnexpectedResponse(format!("get_pull parse: {e}")))?;

        Ok(map_pull(raw))
    }

    fn get_combined_commit_status(
        &self,
        repo: &RepoCoords,
        sha: &str,
    ) -> Result<CombinedCommitStatus> {
        let url = format!("{}?limit=50", self.statuses_url(repo, sha));
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| ScmError::Transport(format!("get_combined_commit_status: {e}")))?;

        if resp.status().as_u16() == 404 {
            return Err(ScmError::NotFound(format!("statuses for {sha}")));
        }
        if !resp.status().is_success() {
            return Err(ScmError::Transport(format!(
                "get_combined_commit_status: HTTP {}",
                resp.status()
            )));
        }

        let raw_statuses: Vec<ForgejoStatus> = resp.json().map_err(|e| {
            ScmError::UnexpectedResponse(format!("get_combined_commit_status parse: {e}"))
        })?;

        let statuses: Vec<CommitStatus> = raw_statuses
            .iter()
            .map(|s| CommitStatus {
                context: s.context.clone(),
                state: map_forgejo_state(&s.state),
                description: s.description.clone().unwrap_or_default(),
                target_url: s.target_url.clone(),
            })
            .collect();

        let aggregate = CommitState::aggregate(statuses.iter().map(|s| s.state));

        Ok(CombinedCommitStatus {
            state: aggregate,
            statuses,
        })
    }

    fn post_commit_status(&self, repo: &RepoCoords, request: &PostStatusRequest) -> Result<()> {
        let url = self.post_status_url(repo, &request.sha);
        let body = ForgejoPostStatusBody {
            context: &request.context,
            state: request.state.as_str(),
            description: &request.description,
            target_url: request.target_url.as_deref(),
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ScmError::Transport(format!("post_commit_status: {e}")))?;

        if resp.status().as_u16() == 401 {
            return Err(ScmError::Unauthorized);
        }
        if !resp.status().is_success() {
            return Err(ScmError::Transport(format!(
                "post_commit_status: HTTP {}",
                resp.status()
            )));
        }

        Ok(())
    }

    fn list_reviews(&self, repo: &RepoCoords, pr_number: u64) -> Result<Vec<Review>> {
        let url = self.reviews_url(repo, pr_number);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| ScmError::Transport(format!("list_reviews: {e}")))?;

        if resp.status().as_u16() == 404 {
            return Err(ScmError::NotFound(format!("reviews for PR #{pr_number}")));
        }
        if !resp.status().is_success() {
            return Err(ScmError::Transport(format!(
                "list_reviews: HTTP {}",
                resp.status()
            )));
        }

        let raw_reviews: Vec<ForgejoReview> = resp
            .json()
            .map_err(|e| ScmError::UnexpectedResponse(format!("list_reviews parse: {e}")))?;

        let reviews = raw_reviews
            .into_iter()
            .map(|r| Review {
                id: r.id,
                reviewer: r.user.login,
                state: map_review_type(&r.review_type),
            })
            .collect();

        Ok(reviews)
    }

    fn merge_pull(
        &self,
        repo: &RepoCoords,
        pr_number: u64,
        method: MergeMethod,
        commit_title: Option<&str>,
    ) -> Result<()> {
        let do_action = match method {
            MergeMethod::Merge => "merge",
            MergeMethod::Squash => "squash",
            MergeMethod::Rebase => "rebase",
        };

        let url = self.merge_url(repo, pr_number);
        let body = ForgejoMergeBody {
            do_action,
            merge_message_field: commit_title,
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ScmError::Transport(format!("merge_pull: {e}")))?;

        match resp.status().as_u16() {
            200 | 204 => Ok(()),
            401 => Err(ScmError::Unauthorized),
            404 => Err(ScmError::NotFound(format!("pull #{pr_number}"))),
            405 | 409 | 422 => {
                let body_text = resp.text().unwrap_or_default();
                Err(ScmError::MergeRejected(format!(
                    "merge rejected for PR #{pr_number}: {body_text}"
                )))
            }
            code => Err(ScmError::Transport(format!(
                "merge_pull: HTTP {code}"
            ))),
        }
    }

    fn fetch_ref(&self, repo: &RepoCoords, git_ref: &str) -> Result<String> {
        let url = self.git_ref_url(repo, git_ref);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| ScmError::Transport(format!("fetch_ref: {e}")))?;

        if resp.status().as_u16() == 404 {
            return Err(ScmError::NotFound(format!("ref {git_ref}")));
        }
        if !resp.status().is_success() {
            return Err(ScmError::Transport(format!(
                "fetch_ref: HTTP {}",
                resp.status()
            )));
        }

        // Forgejo returns an array of matching refs.
        let refs: Vec<ForgejoGitRef> = resp
            .json()
            .map_err(|e| ScmError::UnexpectedResponse(format!("fetch_ref parse: {e}")))?;

        refs.into_iter()
            .find(|r| r.ref_name == git_ref || r.ref_name.ends_with(&format!("/{git_ref}")))
            .map(|r| r.object.sha)
            .ok_or_else(|| ScmError::NotFound(format!("ref {git_ref}")))
    }

    fn webhook_event_from_bytes(
        &self,
        event_type: &str,
        body: &[u8],
        delivery_id: &str,
    ) -> Result<ScmEvent> {
        parse_forgejo_webhook(event_type, body, delivery_id)
    }

    fn get_branch_protection(
        &self,
        repo: &RepoCoords,
        branch: &str,
    ) -> Result<BranchProtection> {
        let url = self.branch_protections_url(repo);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| ScmError::Transport(format!("get_branch_protection: {e}")))?;

        if !resp.status().is_success() {
            return Err(ScmError::Transport(format!(
                "get_branch_protection: HTTP {}",
                resp.status()
            )));
        }

        let protections: Vec<ForgejoBranchProtection> = resp.json().map_err(|e| {
            ScmError::UnexpectedResponse(format!("get_branch_protection parse: {e}"))
        })?;

        protections
            .into_iter()
            .find(|p| {
                p.branch_name
                    .as_deref()
                    .map(|b| b == branch)
                    .unwrap_or(false)
            })
            .map(|p| BranchProtection {
                branch: p.branch_name.unwrap_or_default(),
                require_review: p.required_approvals.unwrap_or(0) > 0,
                required_status_contexts: p.status_check_contexts.unwrap_or_default(),
                block_force_push: !p.enable_push.unwrap_or(true),
            })
            .ok_or_else(|| ScmError::NotFound(format!("branch-protection for {branch}")))
    }
}

// ---------------------------------------------------------------------------
// Review type mapping
// ---------------------------------------------------------------------------

fn map_review_type(t: &str) -> ReviewState {
    match t {
        "APPROVED" => ReviewState::Approved,
        "REQUEST_CHANGES" => ReviewState::ChangesRequested,
        "COMMENT" => ReviewState::Commented,
        "DISMISSED" => ReviewState::Dismissed,
        _ => ReviewState::Commented,
    }
}

// ---------------------------------------------------------------------------
// Webhook parser — Forgejo-specific JSON shapes
// ---------------------------------------------------------------------------

/// Parses Forgejo webhook bytes into a neutral `ScmEvent`.
///
/// This function centralises the JSON parsing logic previously scattered across
/// `oya-ci-webhook-gateway-kernel` and downstream adapter crates.
/// The raw serde shapes are private to this module; only `ScmEvent` crosses
/// the public boundary.
fn parse_forgejo_webhook(
    event_type: &str,
    body: &[u8],
    delivery_id: &str,
) -> Result<ScmEvent> {
    match event_type {
        "pull_request" => parse_pr_event(body, delivery_id),
        "status" => parse_status_event(body),
        "ping" => {
            let ping: serde_json::Value = serde_json::from_slice(body)
                .unwrap_or(serde_json::Value::Null);
            let message = ping
                .get("zen")
                .and_then(|v| v.as_str())
                .unwrap_or("ping")
                .to_owned();
            Ok(ScmEvent::Ping { message })
        }
        other => Ok(ScmEvent::Other {
            event_type: other.to_owned(),
        }),
    }
}

// ---- PR event shapes -------------------------------------------------------

#[derive(Deserialize)]
struct RawPrWebhook {
    action: String,
    #[serde(default)]
    number: Option<u64>,
    pull_request: RawPrDetail,
    repository: Option<RawWebhookRepo>,
}

#[derive(Deserialize)]
struct RawPrDetail {
    #[serde(default)]
    number: Option<u64>,
    title: Option<String>,
    state: Option<String>,
    draft: Option<bool>,
    head: RawWebhookRef,
    base: RawWebhookRef,
    mergeable: Option<bool>,
}

#[derive(Deserialize)]
struct RawWebhookRef {
    #[serde(rename = "ref")]
    ref_name: String,
    sha: String,
}

#[derive(Deserialize)]
struct RawWebhookRepo {
    owner: Option<RawWebhookUser>,
    name: Option<String>,
    full_name: Option<String>,
}

#[derive(Deserialize)]
struct RawWebhookUser {
    login: Option<String>,
}

fn parse_pr_event(body: &[u8], delivery_id: &str) -> Result<ScmEvent> {
    let raw: RawPrWebhook = serde_json::from_slice(body)
        .map_err(|e| ScmError::WebhookParse(format!("pull_request: {e}")))?;

    let action = match raw.action.as_str() {
        "opened" => PullRequestAction::Opened,
        "reopened" => PullRequestAction::Reopened,
        "synchronized" | "synchronize" => PullRequestAction::Synchronized,
        "closed" => PullRequestAction::Closed,
        "ready_for_review" => PullRequestAction::ReadyForReview,
        other => {
            // Unknown action — emit as Other rather than error, so the
            // gateway can log-and-ignore without crashing.
            return Ok(ScmEvent::Other {
                event_type: format!("pull_request:{other}"),
            });
        }
    };

    let pr_number = raw
        .pull_request
        .number
        .or(raw.number)
        .ok_or_else(|| ScmError::WebhookParse("missing pull_request.number".into()))?;

    let mergeability = match raw.pull_request.mergeable {
        Some(true) => Mergeability::Mergeable,
        Some(false) => Mergeability::Conflicted,
        None => Mergeability::Unknown,
    };

    // Derive repo coords from the repository field, falling back to empty.
    let repo = raw
        .repository
        .as_ref()
        .map(|r| {
            let owner = r
                .owner
                .as_ref()
                .and_then(|u| u.login.clone())
                .unwrap_or_default();
            let name = r.name.clone().unwrap_or_default();
            RepoCoords::new(owner, name)
        })
        .unwrap_or_else(|| RepoCoords::new("", ""));

    let pull = PullRequest {
        number: pr_number,
        title: raw.pull_request.title.unwrap_or_default(),
        state: raw.pull_request.state.unwrap_or_else(|| "open".into()),
        draft: raw.pull_request.draft.unwrap_or(false),
        head_sha: raw.pull_request.head.sha,
        head_ref: raw.pull_request.head.ref_name,
        base_sha: raw.pull_request.base.sha,
        base_ref: raw.pull_request.base.ref_name,
        mergeability,
        repo,
    };

    Ok(ScmEvent::PullRequest(PullRequestEvent {
        action,
        pull,
        delivery_id: delivery_id.to_owned(),
    }))
}

// ---- Status event shapes ---------------------------------------------------

#[derive(Deserialize)]
struct RawStatusWebhook {
    sha: String,
    context: Option<String>,
    state: Option<String>,
    repository: Option<RawWebhookRepo>,
}

fn parse_status_event(body: &[u8]) -> Result<ScmEvent> {
    let raw: RawStatusWebhook = serde_json::from_slice(body)
        .map_err(|e| ScmError::WebhookParse(format!("status: {e}")))?;

    let repo = raw
        .repository
        .as_ref()
        .map(|r| {
            let owner = r
                .owner
                .as_ref()
                .and_then(|u| u.login.clone())
                .unwrap_or_default();
            let name = r.name.clone().unwrap_or_default();
            RepoCoords::new(owner, name)
        })
        .unwrap_or_else(|| RepoCoords::new("", ""));

    Ok(ScmEvent::CommitStatus(CommitStatusEvent {
        sha: raw.sha,
        context: raw.context.unwrap_or_default(),
        state: map_forgejo_state(raw.state.as_deref().unwrap_or("error")),
        repo,
    }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- webhook parser tests (no network) ---------------------------------

    #[test]
    fn parse_ping_event() {
        let body = br#"{"zen":"Keep it simple"}"#;
        let event = parse_forgejo_webhook("ping", body, "delivery-1").unwrap();
        assert!(matches!(event, ScmEvent::Ping { .. }));
        if let ScmEvent::Ping { message } = event {
            assert_eq!(message, "Keep it simple");
        }
    }

    #[test]
    fn parse_unknown_event_type_returns_other() {
        let event =
            parse_forgejo_webhook("wiki", b"{}", "d1").unwrap();
        assert!(matches!(event, ScmEvent::Other { .. }));
    }

    #[test]
    fn parse_pr_opened_event() {
        let body = serde_json::json!({
            "action": "opened",
            "number": 42,
            "pull_request": {
                "number": 42,
                "title": "feat: my feature",
                "state": "open",
                "draft": false,
                "head": { "ref": "feat/my-feature", "sha": "abc123" },
                "base": { "ref": "dev", "sha": "base000" },
                "mergeable": true,
            },
            "repository": {
                "owner": { "login": "oyatie" },
                "name": "oyatie",
                "full_name": "oyatie/oyatie"
            }
        })
        .to_string();

        let event = parse_forgejo_webhook("pull_request", body.as_bytes(), "del-42").unwrap();
        let ScmEvent::PullRequest(pr_event) = event else {
            panic!("expected PullRequest event");
        };
        assert_eq!(pr_event.pull.number, 42);
        assert_eq!(pr_event.pull.head_sha, "abc123");
        assert_eq!(pr_event.pull.base_ref, "dev");
        assert!(matches!(pr_event.action, PullRequestAction::Opened));
        assert_eq!(pr_event.delivery_id, "del-42");
        assert_eq!(pr_event.pull.mergeability, Mergeability::Mergeable);
    }

    #[test]
    fn parse_pr_synchronized_event() {
        let body = serde_json::json!({
            "action": "synchronized",
            "number": 7,
            "pull_request": {
                "number": 7,
                "head": { "ref": "feat/b", "sha": "deadbeef" },
                "base": { "ref": "dev", "sha": "base111" },
            }
        })
        .to_string();

        let event = parse_forgejo_webhook("pull_request", body.as_bytes(), "d7").unwrap();
        let ScmEvent::PullRequest(pr_event) = event else {
            panic!("expected PullRequest");
        };
        assert!(matches!(pr_event.action, PullRequestAction::Synchronized));
    }

    #[test]
    fn parse_pr_unknown_action_produces_other() {
        let body = serde_json::json!({
            "action": "labeled",
            "number": 1,
            "pull_request": {
                "number": 1,
                "head": { "ref": "b", "sha": "s" },
                "base": { "ref": "dev", "sha": "b" }
            }
        })
        .to_string();

        let event = parse_forgejo_webhook("pull_request", body.as_bytes(), "d1").unwrap();
        assert!(matches!(event, ScmEvent::Other { .. }));
    }

    #[test]
    fn parse_status_event_success() {
        let body = serde_json::json!({
            "sha": "abc123",
            "context": "cargo-nextest",
            "state": "success",
            "repository": {
                "owner": { "login": "oyatie" },
                "name": "oyatie"
            }
        })
        .to_string();

        let event = parse_forgejo_webhook("status", body.as_bytes(), "d1").unwrap();
        let ScmEvent::CommitStatus(cs) = event else {
            panic!("expected CommitStatus");
        };
        assert_eq!(cs.sha, "abc123");
        assert_eq!(cs.context, "cargo-nextest");
        assert_eq!(cs.state, CommitState::Success);
    }

    #[test]
    fn map_forgejo_state_covers_all_states() {
        assert_eq!(map_forgejo_state("pending"), CommitState::Pending);
        assert_eq!(map_forgejo_state("success"), CommitState::Success);
        assert_eq!(map_forgejo_state("failure"), CommitState::Failure);
        assert_eq!(map_forgejo_state("unknown"), CommitState::Error);
    }

    #[test]
    fn map_review_type_covers_all_types() {
        assert_eq!(map_review_type("APPROVED"), ReviewState::Approved);
        assert_eq!(
            map_review_type("REQUEST_CHANGES"),
            ReviewState::ChangesRequested
        );
        assert_eq!(map_review_type("COMMENT"), ReviewState::Commented);
        assert_eq!(map_review_type("DISMISSED"), ReviewState::Dismissed);
        assert_eq!(map_review_type("UNKNOWN"), ReviewState::Commented);
    }

    #[test]
    fn forgejo_scm_api_url_no_double_slash() {
        let scm = ForgejoScm::new("https://forgejo.example.com/", ScmToken::new("tok"));
        let repo = RepoCoords::new("oyatie", "oyatie");
        let url = scm.pulls_url(&repo);
        assert!(!url.contains("//api"), "url should not have double slash: {url}");
        assert_eq!(
            url,
            "https://forgejo.example.com/api/v1/repos/oyatie/oyatie/pulls"
        );
    }
}
