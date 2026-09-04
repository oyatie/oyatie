use compute_k8s_lifecycle_repository_postgres::{
    PgK8sLifecycleConnectError, PgK8sLifecycleRepository, PgK8sLifecycleRuntimeContract,
    PgK8sLifecycleSchemaError,
};
use sqlx::PgPool;

use crate::support::setup_schema;

pub(super) async fn assert_relation_properties(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    let mut refusals = Vec::new();
    for (change, restore) in [
        (
            "ALTER TABLE compute_k8s_lifecycle.operations SET (fillfactor = 70)",
            "ALTER TABLE compute_k8s_lifecycle.operations RESET (fillfactor)",
        ),
        (
            "CREATE TABLE public.lifecycle_inheritance_child () INHERITS (compute_k8s_lifecycle.operations)",
            "DROP TABLE public.lifecycle_inheritance_child",
        ),
    ] {
        setup_schema(setup, app_role).await;
        sqlx::query(change)
            .execute(setup)
            .await
            .expect("ordinary relation configuration change");
        refusals.push(matches!(
            PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
            Err(PgK8sLifecycleConnectError::Schema(
                PgK8sLifecycleSchemaError::NamespaceContract
            ))
        ));
        sqlx::query(restore)
            .execute(setup)
            .await
            .expect("restore relation configuration");
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
            .await
            .expect("restored relation contract admits startup");
    }
    assert_eq!(
        refusals,
        [true, true],
        "storage options and inheritance must each be refused"
    );
}
