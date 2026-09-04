//! Exact change-path selectors for independently qualified live-Postgres cells.

use super::layout::CARGO_CONFIG_PATHS;

const LIVE_POSTGRES_OTHER_EXACT_PATHS: &[&str] = &[
    ".config/nextest.toml",
    ".github/workflows/live-postgres.yml",
    ".github/workflows/postsubmit.yml",
    ".github/workflows/presubmit.yml",
    "Cargo.lock",
    "Cargo.toml",
    "rust-toolchain.toml",
];

pub const BACKBONE_LIVE_POSTGRES_PATH_PREFIXES: &[&str] = &[
    "audit/adapters/file/",
    "audit/core/chain-domain/",
    "compute/core/resource-provider-contract-kernel/",
    "data/adapters/postgres-command-sqlx/",
    "data/core/data-boundary-kernel/",
    "data/core/postgres-command-kernel/",
    "data/core/ulid-id-kernel/",
    "iam/adapters/identity-scim-store-postgres/",
    "iam/adapters/identity-workload-authz-cedar/",
    "iam/adapters/identity-workload-oidc/",
    "iam/core/identity-oidc-issuer-kernel/",
    "iam/core/identity-workload-domain/",
    "iam/core/platform-contracts-kernel/",
    "iam/core/scim-server-kernel/",
    "iam/facade/identity-service/",
    "iam/facade/identity-workload-app/",
    "iam/facade/identity-workload-rest/",
    "iam/ports/identity-workload-api/",
    "policy/adapters/pdp-cedar/",
    "policy/core/pdp-kernel/",
    "tenancy/adapters/tenant-lifecycle-authz-pdp/",
    "tenancy/adapters/tenant-lifecycle-store-inmemory/",
    "tenancy/adapters/tenant-lifecycle-store-postgres/",
    "tenancy/core/tenant-lifecycle-domain/",
    "tenancy/core/tenant-lifecycle-kernel/",
    "tenancy/core/tenant-lifecycle-usecase/",
    "tenancy/facade/tenant-lifecycle-app/",
    "tenancy/ports/tenant-lifecycle-authz/",
];

pub const COMPUTE_LIFECYCLE_LIVE_POSTGRES_PATH_PREFIXES: &[&str] = &[
    "cell/core/region/",
    "cell/core/routing/",
    "cell/ports/location/",
    "compute/adapters/k8s-lifecycle-repository-postgres/",
    "compute/core/domain/",
    "compute/core/resource/",
    "compute/ports/k8s-api/",
    "data/adapters/postgres-command-sqlx/",
    "data/core/data-boundary-kernel/",
    "data/core/postgres-command-kernel/",
    "iam/core/domain-control/",
    "iam/core/identity-domain/",
    "network/core/domain/",
    "network/core/residency/",
];

pub const LIVE_POSTGRES_SELECTOR_PATH_PREFIXES: &[&str] = &[
    "pipeline/adapters/draft/repository-git/",
    "pipeline/core/admission/",
    "pipeline/core/workspace-members-kernel/",
    "pipeline/facade/change-gates-app/",
    "pipeline/ports/draft/repository/",
];

fn hits_prefix(path: &str, prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(prefix))
}

fn hits_shared_input(path: &str) -> bool {
    CARGO_CONFIG_PATHS.contains(&path) || LIVE_POSTGRES_OTHER_EXACT_PATHS.contains(&path)
}

fn hits_selector(path: &str) -> bool {
    hits_prefix(path, LIVE_POSTGRES_SELECTOR_PATH_PREFIXES)
}

pub fn hits_backbone_postgres_path(path: &str) -> bool {
    hits_shared_input(path)
        || hits_selector(path)
        || hits_prefix(path, BACKBONE_LIVE_POSTGRES_PATH_PREFIXES)
}

pub fn hits_compute_lifecycle_postgres_path(path: &str) -> bool {
    hits_shared_input(path)
        || hits_selector(path)
        || hits_prefix(path, COMPUTE_LIFECYCLE_LIVE_POSTGRES_PATH_PREFIXES)
}

pub fn live_postgres_exact_paths() -> impl Iterator<Item = &'static str> {
    CARGO_CONFIG_PATHS
        .iter()
        .chain(LIVE_POSTGRES_OTHER_EXACT_PATHS)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_local_paths_select_only_the_affected_cell() {
        let backbone = "iam/adapters/identity-scim-store-postgres/src/lib.rs";
        assert!(hits_backbone_postgres_path(backbone));
        assert!(!hits_compute_lifecycle_postgres_path(backbone));

        let compute = "compute/adapters/k8s-lifecycle-repository-postgres/src/lib.rs";
        assert!(!hits_backbone_postgres_path(compute));
        assert!(hits_compute_lifecycle_postgres_path(compute));
    }

    #[test]
    fn shared_packages_select_both_cells() {
        let path = "data/core/data-boundary-kernel/src/lib.rs";
        assert!(hits_backbone_postgres_path(path));
        assert!(hits_compute_lifecycle_postgres_path(path));
    }

    #[test]
    fn selector_and_exact_inputs_select_both_cells() {
        for path in LIVE_POSTGRES_SELECTOR_PATH_PREFIXES
            .iter()
            .copied()
            .chain(live_postgres_exact_paths())
        {
            assert!(hits_backbone_postgres_path(path), "backbone omitted {path}");
            assert!(
                hits_compute_lifecycle_postgres_path(path),
                "Compute omitted {path}"
            );
        }
    }
}
