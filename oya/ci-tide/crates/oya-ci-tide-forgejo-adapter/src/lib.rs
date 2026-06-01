//! # oya-ci-tide-forgejo-adapter
//!
//! Forgejo API client adapter for the oya-ci tide component.
//!
//! Implements [`ForgejoClient`] via reqwest blocking HTTP.
//!
//! ## Endpoints consumed
//!
//! - `GET /api/v1/repos/<owner>/<repo>/pulls?state=open&base=<branch>&limit=50`
//! - `GET /api/v1/repos/<owner>/<repo>/commits/<sha>/statuses?limit=50`
//! - `GET /api/v1/repos/<owner>/<repo>/pulls/<number>/reviews`
//! - `GET /api/v1/repos/<owner>/<repo>/pulls/<number>`
//! - `POST /api/v1/repos/<owner>/<repo>/pulls/<number>/merge`
//!
//! ## Authentication
//!
//! `Authorization: token <OYA_FORGEJO_TOKEN>` — token read from env at
//! construction time via [`ForgejoHttpClient::from_env`]. Never hardcoded.
//!
//! ## Pattern
//!
//! Lifted from `oya-ci-controller-forgejo-adapter`: same reqwest blocking
//! client, same `Authorization: token` header, same 200/201/204 acceptance,
//! same `KernelError::DownstreamTransport`-equivalent error mapping.
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_ci_tide_kernel::{
    CommitStatusState, ForgejoClient, MergeMethod, PullRequest, Result, Review, ReviewState,
    TideError,
};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// ForgejoHttpClient
// ---------------------------------------------------------------------------

/// Forgejo API client backed by reqwest blocking HTTP.
pub struct ForgejoHttpClient {
    api_base: String,    // data_class: INTERNAL_ONLY
    repo_owner: String,  // data_class: INTERNAL_ONLY
    repo_name: String,   // data_class: INTERNAL_ONLY
    token: String,       // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client,
}

impl ForgejoHttpClient {
    /// Construct with explicit values (used in tests and from config).
    pub fn new(api_base: &str, repo_owner: &str, repo_name: &str, token: &str) -> Self {
        Self {
            api_base: api_base.trim_end_matches('/').to_owned(),
            repo_owner: repo_owner.to_owned(),
            repo_name: repo_name.to_owned(),
            token: token.to_owned(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Construct from a [`TideConfig`] + resolved token.
    pub fn from_config(
        config: &oya_ci_tide_kernel::TideConfig,
        token: &str,
    ) -> Self {
        Self::new(
            &config.forgejo_base_url,
            &config.repo_owner,
            &config.repo_name,
            token,
        )
    }

    fn repo_url(&self) -> String {
        format!(
            "{}/api/v1/repos/{}/{}",
            self.api_base, self.repo_owner, self.repo_name
        )
    }

    fn auth_header(&self) -> String {
        format!("token {}", self.token)
    }
}

// ---------------------------------------------------------------------------
// Wire-format types (Forgejo JSON shapes)
// ---------------------------------------------------------------------------

/// Forgejo pull-request list item (projected fields only).
#[derive(Debug, Deserialize)]
struct ForgejoPr {
    number: u64,
    title: String,
    head: ForgejoPrRef,
    base: ForgejoPrRef,
    /// `null` while Forgejo is still computing, `true`/`false` otherwise.
    mergeable: Option<bool>,
    labels: Vec<ForgejoLabel>,
}

#[derive(Debug, Deserialize)]
struct ForgejoPrRef {
    sha: String,
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Debug, Deserialize)]
struct ForgejoLabel {
    name: String,
}

/// Forgejo commit-status item.
#[derive(Debug, Deserialize)]
struct ForgejoStatus {
    context: String,
    state: String,
}

/// Forgejo review item.
#[derive(Debug, Deserialize)]
struct ForgejoReview {
    user: ForgejoUser,
    state: String,
}

#[derive(Debug, Deserialize)]
struct ForgejoUser {
    login: String,
}

/// Forgejo merge request body (`POST /pulls/<n>/merge`).
#[derive(serde::Serialize)]
struct ForgejoMergeBody {
    #[serde(rename = "Do")]
    do_method: String,
    merge_message_field: String,
}

// ---------------------------------------------------------------------------
// ForgejoClient impl
// ---------------------------------------------------------------------------

impl ForgejoClient for ForgejoHttpClient {
    fn list_open_pulls(&self, base_branch: &str) -> Result<Vec<PullRequest>> {
        // Branch names in this codebase (e.g. "dev") contain only characters
        // that are safe in a query-string value; no percent-encoding needed.
        let url = format!(
            "{}/pulls?state=open&base={}&limit=50",
            self.repo_url(),
            base_branch
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| TideError::Downstream(format!("list_open_pulls GET: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(TideError::Downstream(format!(
                "list_open_pulls returned HTTP {status}"
            )));
        }

        let items: Vec<ForgejoPr> = resp
            .json()
            .map_err(|e| TideError::Downstream(format!("list_open_pulls decode: {e}")))?;

        Ok(items.into_iter().map(pr_from_wire).collect())
    }

    fn get_commit_status(
        &self,
        sha: &str,
        required_context: &str,
    ) -> Result<CommitStatusState> {
        let url = format!(
            "{}/commits/{}/statuses?limit=50",
            self.repo_url(),
            sha
        );
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| TideError::Downstream(format!("get_commit_status GET: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(TideError::Downstream(format!(
                "get_commit_status returned HTTP {status}"
            )));
        }

        let statuses: Vec<ForgejoStatus> = resp
            .json()
            .map_err(|e| TideError::Downstream(format!("get_commit_status decode: {e}")))?;

        // Find the most-recent entry for the required context. Forgejo returns
        // statuses newest-first, so the first match is authoritative.
        let found = statuses.iter().find(|s| s.context == required_context);
        Ok(match found {
            Some(s) => CommitStatusState::from_str(&s.state),
            None => CommitStatusState::Missing,
        })
    }

    fn list_reviews(&self, pr_number: u64) -> Result<Vec<Review>> {
        let url = format!("{}/pulls/{}/reviews", self.repo_url(), pr_number);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| TideError::Downstream(format!("list_reviews GET: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(TideError::Downstream(format!(
                "list_reviews returned HTTP {status}"
            )));
        }

        let items: Vec<ForgejoReview> = resp
            .json()
            .map_err(|e| TideError::Downstream(format!("list_reviews decode: {e}")))?;

        Ok(items
            .into_iter()
            .map(|r| Review {
                reviewer: r.user.login,
                state: ReviewState::from_str(&r.state),
            })
            .collect())
    }

    fn get_pull(&self, pr_number: u64) -> Result<PullRequest> {
        let url = format!("{}/pulls/{}", self.repo_url(), pr_number);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .map_err(|e| TideError::Downstream(format!("get_pull GET: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            return Err(TideError::Downstream(format!(
                "get_pull returned HTTP {status}"
            )));
        }

        let pr: ForgejoPr = resp
            .json()
            .map_err(|e| TideError::Downstream(format!("get_pull decode: {e}")))?;

        Ok(pr_from_wire(pr))
    }

    fn merge_pull(&self, pr_number: u64, method: MergeMethod) -> Result<()> {
        let url = format!("{}/pulls/{}/merge", self.repo_url(), pr_number);
        let body = ForgejoMergeBody {
            do_method: method.as_str().to_owned(),
            merge_message_field: String::new(),
        };
        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| TideError::Downstream(format!("merge_pull POST: {e}")))?;

        let status = resp.status();
        // Forgejo returns 200 OK or 204 No Content on success.
        if status.as_u16() == 200 || status.as_u16() == 204 {
            return Ok(());
        }

        Err(TideError::Downstream(format!(
            "merge_pull returned HTTP {status}"
        )))
    }
}

// ---------------------------------------------------------------------------
// Wire → domain projection
// ---------------------------------------------------------------------------

fn pr_from_wire(pr: ForgejoPr) -> PullRequest {
    PullRequest {
        number: pr.number,
        title: pr.title,
        head_sha: pr.head.sha,
        base_ref: pr.base.ref_name,
        mergeable: pr.mergeable,
        labels: pr.labels.into_iter().map(|l| l.name).collect(),
    }
}
