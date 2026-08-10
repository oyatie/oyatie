// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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

#[test]
fn api_error_maps_cross_tenant_kernel_error_to_forbidden() {
    let err = ApiError::from(UseCaseError::Kernel(KernelError::CrossTenantAccessDenied));
    assert!(matches!(err, ApiError::Forbidden(_)));
}
