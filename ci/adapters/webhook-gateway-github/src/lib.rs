//! # ci-webhook-gateway-github
//!
//! GitHub commit-status poster adapter for the CI webhook gateway (ADR-0387 D5).
//!
//! Implements [`CommitStatusPoster`] via reqwest blocking HTTP.
//!
//! ## Endpoint
//!
//! `POST https://api.github.com/repos/<owner>/<repo>/statuses/<sha>`
//!
//! ## Headers
//!
//! - `Authorization: Bearer <token>`
//! - `X-GitHub-Api-Version: 2022-11-28`
//! - `Accept: application/vnd.github+json`
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path. HTTP errors map to
//! [`KernelError::DownstreamTransport`].

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use ci_webhook_gateway_kernel::{CommitStatusPoster, GitHubStatusRequest, KernelError, Result};
use serde::Serialize;

/// GitHub Statuses API version header value.
const GITHUB_API_VERSION: &str = "2022-11-28";
/// Default GitHub API base URL.
const GITHUB_API_BASE: &str = "https://api.github.com";

/// GitHub commit-status poster backed by reqwest blocking.
pub struct GitHubStatusPoster {
    repo_owner: String,   // data_class: INTERNAL_ONLY
    repo_name: String,    // data_class: INTERNAL_ONLY
    github_token: String, // data_class: INTERNAL_ONLY
    api_base: String,     // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client,
}

impl GitHubStatusPoster {
    /// Construct with the given repository coordinates and GitHub token.
    pub fn new(repo_owner: &str, repo_name: &str, github_token: &str) -> Self {
        Self {
            repo_owner: repo_owner.to_owned(),
            repo_name: repo_name.to_owned(),
            github_token: github_token.to_owned(),
            api_base: GITHUB_API_BASE.to_owned(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Override the API base URL (useful in tests to point at a local test server).
    pub fn with_api_base(mut self, base: &str) -> Self {
        self.api_base = base.trim_end_matches('/').to_owned();
        self
    }

    fn statuses_url(&self, sha: &str) -> String {
        format!(
            "{}/repos/{}/{}/statuses/{}",
            self.api_base, self.repo_owner, self.repo_name, sha
        )
    }
}

impl CommitStatusPoster for GitHubStatusPoster {
    fn post(&self, request: &GitHubStatusRequest) -> Result<()> {
        let body = GitHubStatusBody {
            state: request.state.as_str().to_owned(),
            context: request.context.as_str().to_owned(),
            description: request.description.clone(),
            target_url: request.target_url.clone(),
        };

        let resp = self
            .client
            .post(self.statuses_url(&request.sha))
            .bearer_auth(&self.github_token)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .header("Accept", "application/vnd.github+json")
            .json(&body)
            .send()
            .map_err(|e| KernelError::DownstreamTransport(format!("github status post: {e}")))?;

        if !resp.status().is_success() {
            return Err(KernelError::DownstreamTransport(format!(
                "github status returned HTTP {}",
                resp.status()
            )));
        }

        Ok(())
    }
}

/// JSON body for `POST /repos/<owner>/<repo>/statuses/<sha>`.
#[derive(Serialize)]
struct GitHubStatusBody {
    state: String,
    context: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_url: Option<String>,
}
