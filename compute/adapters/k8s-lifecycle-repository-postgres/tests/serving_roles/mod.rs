mod authority;
mod graph;

use compute_k8s_lifecycle_repository_postgres::PgK8sLifecycleRuntimeContract;
use sqlx::PgPool;

pub(crate) async fn assert_serving_role_contract(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    graph::assert_membership_graph_is_exact(setup, app, app_role, runtime_contract).await;
    authority::assert_authority_is_lifecycle_scoped(setup, app, app_role, runtime_contract).await;
}
