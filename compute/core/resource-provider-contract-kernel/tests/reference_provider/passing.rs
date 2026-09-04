use shared_resource_provider_contract_kernel::conformance::{
    check_async_delete_operation, check_create_idempotency, check_idempotent_put,
    check_operation_ledger_semantics, check_read_after_write, check_stable_pagination,
    run_all_checks,
};

use super::fixture::ReferenceFixture;

#[tokio::test]
async fn reference_provider_passes_idempotent_put() {
    check_idempotent_put(&ReferenceFixture).await.unwrap();
}

#[tokio::test]
async fn reference_provider_passes_create_idempotency() {
    check_create_idempotency(&ReferenceFixture).await.unwrap();
}

#[tokio::test]
async fn reference_provider_passes_read_after_write() {
    check_read_after_write(&ReferenceFixture).await.unwrap();
}

#[tokio::test]
async fn reference_provider_passes_stable_pagination() {
    check_stable_pagination(&ReferenceFixture).await.unwrap();
}

#[tokio::test]
async fn reference_provider_passes_async_delete_operation() {
    check_async_delete_operation(&ReferenceFixture)
        .await
        .unwrap();
}

#[tokio::test]
async fn reference_provider_passes_operation_ledger_semantics() {
    check_operation_ledger_semantics(&ReferenceFixture)
        .await
        .unwrap();
}

#[tokio::test]
async fn reference_provider_passes_the_full_contract() {
    let violations = run_all_checks(&ReferenceFixture).await;
    assert!(violations.is_empty(), "{violations:#?}");
}
