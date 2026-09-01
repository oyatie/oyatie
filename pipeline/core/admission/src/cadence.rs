//! Typed protected-event gates over one batch of changed repository paths.

use super::layout::CARGO_CONFIG_PATHS;

pub const LIVE_POSTGRES_PATH_PREFIXES: &[&str] = &[
    "tenancy/adapters/tenant-lifecycle-store-postgres/",
    "iam/adapters/identity-scim-store-postgres/",
    "iam/facade/identity-service/",
    "tenancy/facade/tenant-lifecycle-app/",
    ".github/workflows/presubmit.yml",
    ".github/workflows/live-postgres.yml",
    ".github/workflows/postsubmit.yml",
    ".config/nextest.toml",
];

pub const LIVE_POSTGRES_CRATES: &[&str] = &[
    "tenancy-tenant-lifecycle-store-postgres",
    "identity-scim-store-postgres",
    "iam-identity-service",
    "tenancy-tenant-lifecycle-app",
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
    "change-gates",
    "clippy",
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
    "buck2-weekly-smoke.yml",
    "license-weekly-advisory.yml",
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
    live_postgres: bool,
    reindeer_source_qualification: bool,
}

impl PresubmitChangeGates {
    pub const fn live_postgres(self) -> bool {
        self.live_postgres
    }

    pub const fn reindeer_source_qualification(self) -> bool {
        self.reindeer_source_qualification
    }
}

pub fn hits_live_postgres_path(path: &str) -> bool {
    LIVE_POSTGRES_PATH_PREFIXES
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(prefix))
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
        gates.live_postgres |= hits_live_postgres_path(path);
        gates.reindeer_source_qualification |= hits_reindeer_qualification_path(path);
        if gates.live_postgres && gates.reindeer_source_qualification {
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
            changed_paths.iter().copied().any(hits_live_postgres_path)
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
mod tests {
    use super::*;

    #[test]
    fn docs_pr_does_not_pay_postgres() {
        assert!(!live_postgres_required(
            CadenceEvent::PullRequest,
            &["docs/decisions/ADR-0719-eac-serving-control-north-star.md"]
        ));
        assert!(!live_postgres_required(
            CadenceEvent::MergeGroup,
            &["README.md", "docs/AGENTS.md"]
        ));
    }

    #[test]
    fn adapter_change_pays_postgres_on_pr_and_queue() {
        let paths = ["iam/adapters/identity-scim-store-postgres/src/lib.rs"];
        assert!(live_postgres_required(CadenceEvent::PullRequest, &paths));
        assert!(live_postgres_required(CadenceEvent::MergeGroup, &paths));
    }

    #[test]
    fn workflow_change_pays_postgres() {
        assert!(live_postgres_required(
            CadenceEvent::PullRequest,
            &[".github/workflows/live-postgres.yml"]
        ));
    }

    #[test]
    fn provider_changes_pay_real_reindeer_qualification() {
        let path = "build/dependency-declarations/adapters/generation-reindeer/src/items/provider_source.rs";
        assert!(reindeer_source_qualification_required(
            CadenceEvent::PullRequest,
            &[path]
        ));
        assert!(reindeer_source_qualification_required(
            CadenceEvent::MergeGroup,
            &[path]
        ));
    }

    #[test]
    fn qualification_inputs_pay_on_both_protected_events() {
        for path in reindeer_qualification_exact_paths() {
            for event in [CadenceEvent::PullRequest, CadenceEvent::MergeGroup] {
                assert!(
                    presubmit_change_gates(event, [path]).reindeer_source_qualification(),
                    "{event:?} omitted {path}"
                );
            }
        }
        assert!(!reindeer_source_qualification_required(
            CadenceEvent::PostsubmitPush,
            &["build/dependency-declarations/adapters/generation-reindeer/src/lib.rs"]
        ));
    }

    #[test]
    fn postsubmit_and_dispatch_always_pay() {
        assert!(live_postgres_required(CadenceEvent::PostsubmitPush, &[]));
        assert!(live_postgres_required(CadenceEvent::WorkflowDispatch, &[]));
    }

    #[test]
    fn crates_cover_the_four_live_packages() {
        assert_eq!(LIVE_POSTGRES_CRATES.len(), 4);
    }

    #[test]
    fn every_postgres_prefix_pays_on_both_protected_events() {
        for prefix in LIVE_POSTGRES_PATH_PREFIXES {
            for event in [CadenceEvent::PullRequest, CadenceEvent::MergeGroup] {
                assert!(
                    presubmit_change_gates(event, [*prefix]).live_postgres(),
                    "{event:?} omitted {prefix}"
                );
            }
        }
    }

    #[test]
    fn live_job_names_every_live_crate() {
        let y = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.github/workflows/live-postgres.yml"),
        )
        .expect("live-postgres.yml");
        for crate_name in LIVE_POSTGRES_CRATES {
            assert!(
                y.contains(crate_name),
                "live-postgres.yml missing crate {crate_name}"
            );
        }
        assert!(y.contains("--no-tests=error"));
        assert!(y.contains("--run-ignored only"));
    }

    #[test]
    fn one_path_traversal_produces_both_gate_outputs() {
        use std::cell::Cell;

        let visits = Cell::new(0);
        let paths = [
            "iam/adapters/identity-scim-store-postgres/src/lib.rs",
            "Cargo.lock",
            "docs/not-visited.md",
        ];
        let gates = presubmit_change_gates(
            CadenceEvent::PullRequest,
            paths.into_iter().inspect(|_| {
                visits.set(visits.get() + 1);
            }),
        );

        assert!(gates.live_postgres());
        assert!(gates.reindeer_source_qualification());
        assert_eq!(visits.get(), 2);
    }
}
