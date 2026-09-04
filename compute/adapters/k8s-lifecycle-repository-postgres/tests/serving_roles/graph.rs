use compute_k8s_lifecycle_repository_postgres::{
    CLUSTERS_TABLE, PgK8sLifecycleConnectError, PgK8sLifecycleRepository,
    PgK8sLifecycleRuntimeContract, RUNTIME_ROLE,
};
use sqlx::PgPool;

use crate::support::{quote_identifier, setup_schema};

const ROGUE_DIRECT_ROLE: &str = "compute_k8s_rogue_direct";
const ROGUE_DELEGATE_ROLE: &str = "compute_k8s_rogue_delegate";
const ROGUE_TRANSITIVE_ROLE: &str = "compute_k8s_rogue_transitive";
const GREEN_ROLE: &str = "compute_k8s_serving_green";

pub(super) async fn assert_membership_graph_is_exact(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    undeclared_direct_member_is_refused_and_remediated(
        setup,
        app,
        app_role,
        runtime_contract,
        ROGUE_DIRECT_ROLE,
        false,
    )
    .await;
    undeclared_direct_member_is_refused_and_remediated(
        setup,
        app,
        app_role,
        runtime_contract,
        ROGUE_DELEGATE_ROLE,
        true,
    )
    .await;
    transitive_member_is_refused_and_remediated(setup, app, app_role, runtime_contract).await;
    serving_edge_options_are_exact(setup, app, app_role, runtime_contract).await;
    declared_blue_green_members_are_admitted(setup, app, app_role).await;
}

async fn undeclared_direct_member_is_refused_and_remediated(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
    rogue_role: &str,
    admin_option: bool,
) {
    setup_schema(setup, app_role).await;
    recreate_role(setup, rogue_role).await;
    let options = if admin_option {
        " WITH ADMIN OPTION, SET FALSE"
    } else {
        " WITH SET FALSE"
    };
    sqlx::query(&format!(
        "GRANT {RUNTIME_ROLE} TO {}{options}",
        quote_identifier(rogue_role)
    ))
    .execute(setup)
    .await
    .expect("grant lifecycle access to undeclared role");

    assert!(can_read_lifecycle_table(setup, rogue_role).await);
    assert_graph_mismatch(app, runtime_contract).await;

    sqlx::query(&format!(
        "REVOKE {RUNTIME_ROLE} FROM {}",
        quote_identifier(rogue_role)
    ))
    .execute(setup)
    .await
    .expect("revoke undeclared lifecycle membership");
    assert!(!can_read_lifecycle_table(setup, rogue_role).await);
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("runtime accepts the remediated membership graph");
    drop_role(setup, rogue_role).await;
}

async fn transitive_member_is_refused_and_remediated(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    recreate_role(setup, ROGUE_TRANSITIVE_ROLE).await;
    sqlx::query(&format!(
        "GRANT {} TO {}",
        quote_identifier(app_role),
        quote_identifier(ROGUE_TRANSITIVE_ROLE)
    ))
    .execute(setup)
    .await
    .expect("grant serving principal transitively");

    assert!(can_read_lifecycle_table(setup, ROGUE_TRANSITIVE_ROLE).await);
    assert_graph_mismatch(app, runtime_contract).await;

    sqlx::query(&format!(
        "REVOKE {} FROM {}",
        quote_identifier(app_role),
        quote_identifier(ROGUE_TRANSITIVE_ROLE)
    ))
    .execute(setup)
    .await
    .expect("revoke transitive serving principal");
    assert!(!can_read_lifecycle_table(setup, ROGUE_TRANSITIVE_ROLE).await);
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("runtime accepts remediated transitive membership");
    drop_role(setup, ROGUE_TRANSITIVE_ROLE).await;
}

async fn serving_edge_options_are_exact(
    setup: &PgPool,
    app: &PgPool,
    app_role: &str,
    runtime_contract: &PgK8sLifecycleRuntimeContract,
) {
    setup_schema(setup, app_role).await;
    let role = quote_identifier(app_role);
    sqlx::query(&format!(
        "GRANT {RUNTIME_ROLE} TO {role} WITH INHERIT TRUE, SET TRUE"
    ))
    .execute(setup)
    .await
    .expect("inject unnecessary role-switch authority");
    assert_graph_mismatch(app, runtime_contract).await;

    sqlx::query(&format!(
        "GRANT {RUNTIME_ROLE} TO {role} WITH INHERIT FALSE, SET FALSE"
    ))
    .execute(setup)
    .await
    .expect("inject non-inheriting lifecycle edge");
    assert_graph_mismatch(app, runtime_contract).await;

    sqlx::query(&format!(
        "GRANT {RUNTIME_ROLE} TO {role} WITH INHERIT TRUE, SET FALSE"
    ))
    .execute(setup)
    .await
    .expect("restore least-authority lifecycle edge");
    PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract)
        .await
        .expect("runtime accepts the exact membership options");
}

async fn declared_blue_green_members_are_admitted(setup: &PgPool, app: &PgPool, app_role: &str) {
    setup_schema(setup, app_role).await;
    recreate_role(setup, GREEN_ROLE).await;
    sqlx::query(&format!(
        "GRANT {RUNTIME_ROLE} TO {} WITH SET FALSE",
        quote_identifier(GREEN_ROLE)
    ))
    .execute(setup)
    .await
    .expect("grant declared green serving principal");
    let rotation = PgK8sLifecycleRuntimeContract::new([app_role, GREEN_ROLE])
        .expect("blue-green contract is valid");
    PgK8sLifecycleRepository::from_pool(app.clone(), &rotation)
        .await
        .expect("declared blue-green membership graph is admitted");

    sqlx::query(&format!(
        "REVOKE {RUNTIME_ROLE} FROM {}",
        quote_identifier(GREEN_ROLE)
    ))
    .execute(setup)
    .await
    .expect("retire green serving principal");
    drop_role(setup, GREEN_ROLE).await;
}

async fn assert_graph_mismatch(app: &PgPool, runtime_contract: &PgK8sLifecycleRuntimeContract) {
    assert!(matches!(
        PgK8sLifecycleRepository::from_pool(app.clone(), runtime_contract).await,
        Err(PgK8sLifecycleConnectError::ServingRoleGraphMismatch)
    ));
}

async fn can_read_lifecycle_table(setup: &PgPool, role: &str) -> bool {
    let mut transaction = setup.begin().await.expect("begin access probe");
    sqlx::query(&format!("SET LOCAL ROLE {}", quote_identifier(role)))
        .execute(&mut *transaction)
        .await
        .expect("assume probed role");
    sqlx::query("SELECT set_config('oyatie.tenant_id', 'ten_probe', true)")
        .execute(&mut *transaction)
        .await
        .expect("set probe tenant");
    let result =
        sqlx::query_scalar::<_, i64>(&format!("SELECT count(*)::bigint FROM {CLUSTERS_TABLE}"))
            .fetch_one(&mut *transaction)
            .await;
    transaction.rollback().await.expect("rollback access probe");
    result.is_ok()
}

async fn recreate_role(setup: &PgPool, role: &str) {
    drop_role(setup, role).await;
    sqlx::query(&format!(
        "CREATE ROLE {} NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS INHERIT",
        quote_identifier(role)
    ))
    .execute(setup)
    .await
    .expect("create isolated serving-role probe");
}

async fn drop_role(setup: &PgPool, role: &str) {
    sqlx::query(&format!("DROP ROLE IF EXISTS {}", quote_identifier(role)))
        .execute(setup)
        .await
        .expect("drop serving-role probe");
}
