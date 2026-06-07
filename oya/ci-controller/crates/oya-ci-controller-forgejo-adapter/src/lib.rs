// DELETION-TAGGED: Forgejo is dropped (D-FORGE); bridge impl retained until the Forgejo-eradication lane removes it.
//! # oya-ci-controller-forgejo-adapter
//!
//! Forgejo commit-status poster adapter for the oya-ci controller.
//!
//! Implements [`CommitStatusPoster`] via reqwest blocking HTTP.
//!
//! ## Endpoint
//!
//! `POST http://forgejo.oya-forge.svc.cluster.local:3000/api/v1/repos/oya-admin/oyatie/statuses/<sha>`
//!
//! ## Headers
//!
//! - `Authorization: token <FORGEJO_CI_TOKEN>`
//! - `Content-Type: application/json`
//!
//! Accepts 200 or 201. Any other status -> `KernelError::DownstreamTransport`.
//!
//! Lifted from Jenkinsfile-oya-ci-gate:131-144 `postForgejoStatus`.
//!
//! ## ADR-0083 Tier-3
//!
//! No `unwrap`/`expect`/`panic` on the request path.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use oya_ci_controller_kernel::{CommitState, CommitStatusPoster, KernelError, Result};
use serde::Serialize;

/// Default Forgejo API base URL (in-cluster).
const DEFAULT_FORGEJO_BASE: &str =
    "http://forgejo.oya-forge.svc.cluster.local:3000";

/// Default repository (the gating forge of record, ADR-0363).
const DEFAULT_REPO_OWNER: &str = "oya-admin";
const DEFAULT_REPO_NAME: &str = "oyatie";

/// Forgejo commit-status poster backed by reqwest blocking.
pub struct ForgejoCommitStatusPoster {
    api_base: String,     // data_class: INTERNAL_ONLY
    repo_owner: String,   // data_class: INTERNAL_ONLY
    repo_name: String,    // data_class: INTERNAL_ONLY
    token: String,        // data_class: INTERNAL_ONLY
    client: reqwest::blocking::Client,
}

impl ForgejoCommitStatusPoster {
    /// Construct with the given Forgejo base URL and token.
    pub fn new(api_base: &str, repo_owner: &str, repo_name: &str, token: &str) -> Self {
        Self {
            api_base: api_base.trim_end_matches('/').to_owned(),
            repo_owner: repo_owner.to_owned(),
            repo_name: repo_name.to_owned(),
            token: token.to_owned(),
            client: reqwest::blocking::Client::new(),
        }
    }

    /// Construct with in-cluster defaults (forgejo.oya-forge, oya-admin/oyatie).
    pub fn with_defaults(token: &str) -> Self {
        Self::new(DEFAULT_FORGEJO_BASE, DEFAULT_REPO_OWNER, DEFAULT_REPO_NAME, token)
    }

    fn statuses_url(&self, sha: &str) -> String {
        format!(
            "{}/api/v1/repos/{}/{}/statuses/{}",
            self.api_base, self.repo_owner, self.repo_name, sha
        )
    }
}

impl CommitStatusPoster for ForgejoCommitStatusPoster {
    fn post(
        &self,
        sha: &str,
        state: CommitState,
        context: &str,
        description: &str,
        target_url: Option<&str>,
    ) -> Result<()> {
        // Truncate description to 240 Unicode scalar values (Forgejo / GitHub
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

        let body = ForgejoStatusBody {
            state: state.as_str().to_owned(),
            context: context.to_owned(),
            description: description.to_owned(),
            target_url: target_url.map(ToOwned::to_owned),
        };

        let resp = self
            .client
            .post(self.statuses_url(sha))
            .header("Authorization", format!("token {}", self.token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| {
                KernelError::DownstreamTransport(format!("forgejo status post: {e}"))
            })?;

        let status = resp.status();
        // Accept 200 OK or 201 Created (mirrors Jenkinsfile postForgejoStatus acceptance).
        if status.as_u16() == 200 || status.as_u16() == 201 {
            return Ok(());
        }

        Err(KernelError::DownstreamTransport(format!(
            "forgejo status returned HTTP {status}"
        )))
    }
}

/// JSON body for `POST /api/v1/repos/<owner>/<repo>/statuses/<sha>`.
#[derive(Serialize)]
struct ForgejoStatusBody {
    state: String,
    context: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_url: Option<String>,
}
