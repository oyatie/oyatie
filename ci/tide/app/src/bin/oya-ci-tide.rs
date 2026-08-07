//! # oya-ci-tide binary
//!
//! Entry point for the tide merge-queue service (ADR-0513 Phase 2).
//!
//! ## Bootstrap
//!
//! 1. Initialise `tracing-subscriber` (JSON, env-filter).
//! 2. Load [`TideConfig`] from the process environment.
//! 3. Resolve `OYA_GITHUB_TOKEN` from env — fail-fast if absent.
//! 4. Build [`GitHubHttpClient`] from config + token.
//! 5. Hand off to [`TideRunner::run_loop`] (blocking).
//!
//! ## Safety
//!
//! `dry_run` defaults to `true`; the service merges nothing unless
//! `OYA_TIDE_DRY_RUN=false` is explicitly set.

use ci_tide_app::TideRunner;
use ci_tide_github_adapter::GitHubHttpClient;
use ci_tide_kernel::{ENV_GITHUB_TOKEN, TideConfig};
use std::sync::Arc;
use tracing::info;

fn main() {
    // Initialise structured logging (JSON lines, env-filter for log level).
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load config from the environment.
    let config = TideConfig::from_env();

    info!(
        base_branch = %config.base_branch,
        required_status_context = %config.required_status_context,
        min_approvals = config.approval_policy.min_approvals,
        poll_interval_secs = config.poll_interval_secs,
        merge_method = %config.merge_method.as_str(),
        dry_run = config.dry_run,
        "oya-ci-tide: starting"
    );

    // Resolve the GitHub token. Fail-fast: a missing token means we cannot
    // call any GitHub API, so there is no point running the loop.
    let token = std::env::var(ENV_GITHUB_TOKEN).unwrap_or_else(|_| {
        eprintln!(
            "FATAL: {ENV_GITHUB_TOKEN} is not set. \
             Provision the token via the deploy substrate (OpenBao + ESO) and retry."
        );
        std::process::exit(1);
    });

    if token.trim().is_empty() {
        eprintln!(
            "FATAL: {ENV_GITHUB_TOKEN} is set but empty. \
             Provision a valid token and retry."
        );
        std::process::exit(1);
    }

    // Build the HTTP client.
    let client = Arc::new(GitHubHttpClient::from_config(&config, &token));

    // Run the tide loop (blocks forever).
    let runner = TideRunner::new(config, client);
    runner.run_loop();
}
