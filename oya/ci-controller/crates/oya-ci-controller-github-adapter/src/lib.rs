//! # oya-ci-controller-github-adapter
//!
//! GitHub commit-status poster for the oya-ci controller (the `oya-ci-required`
//! producer).
//!
//! Implements [`CommitStatusPoster`] via reqwest blocking HTTP. The HTTP shape
//! is lifted from the proven oya-ci-webhook-gateway-github-adapter (ADR-0387 D5).
//! Forge-of-record = GitHub (D2/D-FORGE; GitHub dropped).
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
//! - `User-Agent: oya-ci-controller` (GitHub rejects UA-less requests)
//!
//! Accepts any 2xx (`is_success`; GitHub returns 201 Created). Any other status
//! -> `KernelError::DownstreamTransport`.
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_ci_controller_kernel::{CommitState, CommitStatusPoster, KernelError, Result};
use serde::Serialize;

/// GitHub Statuses API version header value.
const GITHUB_API_VERSION: &str = "2022-11-28";
/// Default GitHub API base URL.
const GITHUB_API_BASE: &str = "https://api.github.com";
/// Forge of record (D2): GitHub interim until the Sapling-inspired bespoke SCM.
const DEFAULT_REPO_OWNER: &str = "jason931225";
const DEFAULT_REPO_NAME: &str = "oyatie";

/// GitHub commit-status poster backed by reqwest blocking.
pub struct GitHubCommitStatusPoster {
    repo_owner: String,   // data_class: INTERNAL_ONLY
    repo_name: String,    // data_class: INTERNAL_ONLY
    github_token: String, // data_class: INTERNAL_ONLY  (controller crier token ONLY; never to runner)
    api_base: String,     // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client,
}

impl GitHubCommitStatusPoster {
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

    /// Construct with the forge-of-record defaults (jason931225/oyatie).
    pub fn with_defaults(token: &str) -> Self {
        Self::new(DEFAULT_REPO_OWNER, DEFAULT_REPO_NAME, token)
    }

    /// Override the API base URL (useful in tests to point at httpmock).
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

impl CommitStatusPoster for GitHubCommitStatusPoster {
    fn post(
        &self,
        sha: &str,
        state: CommitState,
        context: &str,
        description: &str,
        target_url: Option<&str>,
    ) -> Result<()> {
        // Truncate description to 240 Unicode scalar values (GitHub / GitHub
        // limit). Byte-slicing is unsafe on multibyte boundaries; use char
        // indices to find the correct byte offset.
        let description: &str = if description.chars().count() > 240 {
            let byte_end = description
                .char_indices()
                .nth(240)
                .map(|(i, _)| i)
                .unwrap_or(description.len());
            &description[..byte_end]
        } else {
            description
        };

        let body = GitHubStatusBody {
            state: state.as_str().to_owned(),
            context: context.to_owned(),
            description: description.to_owned(),
            target_url: target_url.map(ToOwned::to_owned),
        };

        let resp = self
            .client
            .post(self.statuses_url(sha))
            .bearer_auth(&self.github_token)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "oya-ci-controller")
            .json(&body)
            .send()
            .map_err(|e| KernelError::DownstreamTransport(format!("github status post: {e}")))?;

        // GitHub returns 201 Created on success.
        if resp.status().is_success() {
            return Ok(());
        }

        Err(KernelError::DownstreamTransport(format!(
            "github status returned HTTP {}",
            resp.status()
        )))
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
