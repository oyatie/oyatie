//! Which event pays live Postgres, and which paths trip the gate.
//! GHA bash in presubmit.yml must match `live_postgres_required`.

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CadenceEvent {
    PullRequest,
    MergeGroup,
    WorkflowDispatch,
    PostsubmitPush,
}

pub fn hits_live_postgres_path(path: &str) -> bool {
    LIVE_POSTGRES_PATH_PREFIXES
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(prefix))
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
    fn postsubmit_and_dispatch_always_pay() {
        assert!(live_postgres_required(CadenceEvent::PostsubmitPush, &[]));
        assert!(live_postgres_required(CadenceEvent::WorkflowDispatch, &[]));
    }

    #[test]
    fn crates_cover_the_four_live_packages() {
        assert_eq!(LIVE_POSTGRES_CRATES.len(), 4);
    }

    #[test]
    fn presubmit_pg_gate_lists_every_prefix() {
        let y = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.github/workflows/presubmit.yml"),
        )
        .expect("presubmit.yml");
        for prefix in LIVE_POSTGRES_PATH_PREFIXES {
            let as_grep = prefix.replace('.', r"\.");
            assert!(
                y.contains(prefix) || y.contains(&as_grep),
                "presubmit pg-gate missing {prefix}"
            );
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
}
