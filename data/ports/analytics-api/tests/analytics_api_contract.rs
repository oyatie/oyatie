// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use data_analytics_api::{
    ANALYTICS_ASYNCAPI_CONTRACT, ANALYTICS_OPENAPI_CONTRACT, ANALYTICS_PROTO_CONTRACT, ApiError,
};
use data_analytics_usecase::UseCaseError;
use oya_shared_olap_client_kernel::KernelError;

#[test]
fn analytics_api_contract_runtime_constants_are_covered() {
    assert_eq!(
        ANALYTICS_OPENAPI_CONTRACT,
        "data/analytics/contracts/openapi-v1.yaml"
    );
    assert_eq!(
        ANALYTICS_ASYNCAPI_CONTRACT,
        "data/analytics/contracts/asyncapi-v1.yaml"
    );
    assert_eq!(
        ANALYTICS_PROTO_CONTRACT,
        "data/analytics/contracts/analytics.proto"
    );
}

/// Walk up from a start dir to the repo root (buck2-safe; avoid sole reliance on
/// `CARGO_MANIFEST_DIR`, which is absent or non-source under buck).
fn repo_root() -> PathBuf {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(manifest) = option_env!("CARGO_MANIFEST_DIR") {
        candidates.push(PathBuf::from(manifest));
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    for start in candidates {
        let mut dir = start;
        for _ in 0..12 {
            if dir.join("data/analytics/contracts/openapi-v1.yaml").is_file()
                && dir.join("specs/root-hub-pointers.json").is_file()
            {
                return dir;
            }
            if !dir.pop() {
                break;
            }
        }
    }
    panic!("could not locate repo root containing data/analytics contracts");
}

/// Contract paths must resolve under the data/ capability root (Wave-2 hygiene).
#[test]
fn analytics_api_contract_files_exist_under_data_root() {
    let root = repo_root();
    for rel in [
        ANALYTICS_OPENAPI_CONTRACT,
        ANALYTICS_ASYNCAPI_CONTRACT,
        ANALYTICS_PROTO_CONTRACT,
    ] {
        let abs = root.join(rel);
        assert!(
            abs.is_file(),
            "missing analytics contract under data/ root: {rel} (resolved {abs:?})"
        );
    }
    // Silence unused import if Path is only used via PathBuf above in some editions.
    let _: &Path = root.as_path();
}

#[test]
fn api_error_maps_cross_tenant_kernel_error_to_forbidden() {
    let err = ApiError::from(UseCaseError::Kernel(KernelError::CrossTenantAccessDenied));
    assert!(matches!(err, ApiError::Forbidden(_)));
}
