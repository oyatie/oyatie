mod boundaries;
mod compatibility;
mod concurrency;
mod fixtures;
mod integrity;
mod recovery;

use compute_k8s_api::CloudComputeK8sOperationLookup;
use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract,
};
use shared_resource_provider_contract_kernel::OperationState;
use sqlx::PgPool;

use crate::support::{setup_schema, tenant_count};
use fixtures::*;

pub(crate) async fn assert_acceptance(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    app_url: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    let repository = PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .unwrap();
    let first = accept(&repository, pending_request("first", "durable-key"))
        .await
        .unwrap();
    assert_eq!(first.status_code(), 202);
    assert_eq!(first.operation.receipt.request_id, "first");
    assert_eq!(
        first.operation.receipt.intent.resource_id,
        "oyatie:cloud:region-home:ten_alpha:k8s:pending"
    );
    assert_eq!(first.operation.state, OperationState::Accepted);
    assert_eq!(
        tenant_count(
            app,
            "ten_alpha",
            "SELECT count(*)::bigint FROM compute_k8s_lifecycle.operations"
        )
        .await,
        1
    );
    assert_eq!(
        tenant_count(
            app,
            "ten_alpha",
            "SELECT count(*)::bigint FROM compute_k8s_lifecycle.clusters"
        )
        .await,
        0
    );
    let reopened = PgK8sLifecycleRepository::connect(app_url, runtime_contract)
        .await
        .unwrap();
    let recovered = read(&reopened, operation_read_request("durable-key"))
        .await
        .unwrap();
    assert_eq!(
        recovered,
        CloudComputeK8sOperationLookup::Found(first.operation.clone())
    );
    let replay = accept(&reopened, pending_request("retry", "durable-key"))
        .await
        .unwrap();
    assert_eq!(replay, first);
    concurrency::assert_concurrency(&repository, app).await;
    compatibility::assert_compatibility(&repository, app).await;
    boundaries::assert_boundaries(&repository).await;
    recovery::assert_recovery(setup, app, app_url, runtime_contract, &repository).await;
    integrity::assert_integrity(setup, &repository).await;
}
