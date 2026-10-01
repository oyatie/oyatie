//! Typed protected-event gates over one batch of changed repository paths.

use super::layout::CARGO_CONFIG_PATHS;
use super::live_postgres::{hits_backbone_postgres_path, hits_compute_lifecycle_postgres_path};

pub const LIVE_POSTGRES_CRATES: &[&str] = &[
    "compute-k8s-lifecycle-repository-postgres",
    "tenancy-tenant-lifecycle-store-postgres",
    "identity-scim-store-postgres",
    "iam-identity-app",
    "tenancy-tenant-lifecycle-app",
];

/// Occupants of the reusable live-Postgres workflow (sorted).
pub const LIVE_POSTGRES_JOBS: &[&str] = &[
    "compute-lifecycle-postgres",
    "live-postgres",
    "live-postgres-verdict",
];

const REINDEER_QUALIFICATION_OTHER_EXACT_PATHS: &[&str] = &[
    ".config/nextest.toml",
    ".github/workflows/presubmit.yml",
    "Cargo.lock",
    "Cargo.toml",
    "reindeer.toml",
    "rust-toolchain.toml",
];

pub const REINDEER_QUALIFICATION_PATH_PREFIXES: &[&str] = &[
    "build/dependency-declarations/adapters/generation-reindeer/",
    "build/dependency-declarations/core/reconcile/",
    "build/dependency-declarations/ports/generation/",
    "build/dependency-declarations/ports/publication/",
    "pipeline/adapters/draft/repository-git/",
    "pipeline/core/admission/",
    "pipeline/core/workspace-members-kernel/",
    "pipeline/facade/change-gates-app/",
    "pipeline/ports/draft/repository/",
];

/// Occupants of the presubmit workflow (sorted).
pub const PRESUBMIT_JOBS: &[&str] = &[
    "build-cache-qualification",
    "change-gates",
    "clippy",
    "commit-signing",
    "deny",
    "layout",
    "lint",
    "live-postgres",
    "occupancy",
    "presubmit",
    "reindeer-source-qualification",
    "test",
];

/// Occupants of the postsubmit workflow (sorted).
pub const POSTSUBMIT_JOBS: &[&str] = &["live-postgres", "postsubmit", "test"];

/// Occupants of `.github/workflows/` (sorted).
pub const WORKFLOW_FILES: &[&str] = &[
    "buck2-graph-honesty.yml",
    "build-cache-qualification.yml",
    "commit-signing.yml",
    "dependency-denial.yml",
    "license-weekly-advisory.yml",
    "live-fdb.yml",
    "live-postgres.yml",
    "nightly.yml",
    "postsubmit.yml",
    "presubmit.yml",
    "promotion-predecessor.yml",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CadenceEvent {
    PullRequest,
    MergeGroup,
    WorkflowDispatch,
    PostsubmitPush,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PresubmitChangeGates {
    backbone_postgres: bool,
    compute_lifecycle_postgres: bool,
    reindeer_source_qualification: bool,
}

impl PresubmitChangeGates {
    pub const fn backbone_postgres(self) -> bool {
        self.backbone_postgres
    }

    pub const fn compute_lifecycle_postgres(self) -> bool {
        self.compute_lifecycle_postgres
    }

    pub const fn live_postgres(self) -> bool {
        self.backbone_postgres || self.compute_lifecycle_postgres
    }

    pub const fn reindeer_source_qualification(self) -> bool {
        self.reindeer_source_qualification
    }
}

pub fn hits_reindeer_qualification_path(path: &str) -> bool {
    CARGO_CONFIG_PATHS.contains(&path)
        || REINDEER_QUALIFICATION_OTHER_EXACT_PATHS.contains(&path)
        || REINDEER_QUALIFICATION_PATH_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix))
}

pub fn reindeer_qualification_exact_paths() -> impl Iterator<Item = &'static str> {
    CARGO_CONFIG_PATHS
        .iter()
        .chain(REINDEER_QUALIFICATION_OTHER_EXACT_PATHS)
        .copied()
}

pub fn presubmit_change_gates<'a>(
    event: CadenceEvent,
    changed_paths: impl IntoIterator<Item = &'a str>,
) -> PresubmitChangeGates {
    if !matches!(event, CadenceEvent::PullRequest | CadenceEvent::MergeGroup) {
        return PresubmitChangeGates::default();
    }
    let mut gates = PresubmitChangeGates::default();
    for path in changed_paths {
        gates.backbone_postgres |= hits_backbone_postgres_path(path);
        gates.compute_lifecycle_postgres |= hits_compute_lifecycle_postgres_path(path);
        gates.reindeer_source_qualification |= hits_reindeer_qualification_path(path);
        if gates.backbone_postgres
            && gates.compute_lifecycle_postgres
            && gates.reindeer_source_qualification
        {
            break;
        }
    }
    gates
}

/// Fail-closed: unknown events are not represented. Dispatch and postsubmit
/// always run live Postgres (that is the unique proof of those cadences).
/// PR and merge_group run it only when a live path changed.
pub fn live_postgres_required(event: CadenceEvent, changed_paths: &[&str]) -> bool {
    match event {
        CadenceEvent::WorkflowDispatch | CadenceEvent::PostsubmitPush => true,
        CadenceEvent::PullRequest | CadenceEvent::MergeGroup => {
            presubmit_change_gates(event, changed_paths.iter().copied()).live_postgres()
        }
    }
}

pub fn backbone_postgres_required(event: CadenceEvent, changed_paths: &[&str]) -> bool {
    match event {
        CadenceEvent::WorkflowDispatch | CadenceEvent::PostsubmitPush => true,
        CadenceEvent::PullRequest | CadenceEvent::MergeGroup => {
            presubmit_change_gates(event, changed_paths.iter().copied()).backbone_postgres()
        }
    }
}

pub fn compute_lifecycle_postgres_required(event: CadenceEvent, changed_paths: &[&str]) -> bool {
    match event {
        CadenceEvent::WorkflowDispatch | CadenceEvent::PostsubmitPush => true,
        CadenceEvent::PullRequest | CadenceEvent::MergeGroup => {
            presubmit_change_gates(event, changed_paths.iter().copied())
                .compute_lifecycle_postgres()
        }
    }
}

pub fn reindeer_source_qualification_required(event: CadenceEvent, changed_paths: &[&str]) -> bool {
    matches!(event, CadenceEvent::PullRequest | CadenceEvent::MergeGroup)
        && changed_paths
            .iter()
            .copied()
            .any(hits_reindeer_qualification_path)
}

#[cfg(test)]
mod tests;
