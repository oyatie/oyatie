use super::*;
use iam_identity_scim_store_postgres::{
    GOVERNED_TABLES, PASSKEY_CHALLENGES_TABLE, PASSKEYS_TABLE, SESSIONS_TABLE,
};

async fn records(setup: &PgPool, tables: &[&str]) -> Vec<String> {
    let mut snapshots = Vec::new();
    for table in tables {
        snapshots.push(
            sqlx::query_scalar(&format!(
                "SELECT coalesce(json_agg(row_to_json(r) ORDER BY row_to_json(r)::text)::text, '[]') FROM {table} r"
            ))
            .fetch_one(setup)
            .await
            .unwrap(),
        );
    }
    snapshots
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_passkey_schema_setup_preserves_records_and_refuses_runtime_ddl() {
    require_enabled();
    let setup = pool(&std::env::var(SETUP_URL_ENV).unwrap()).await;
    let app = pool(&std::env::var(APP_URL_ENV).unwrap()).await;
    let (app_role, rolsuper, rolbypassrls) = current_role_flags(&app).await;
    assert!(!rolsuper && !rolbypassrls);
    setup_schema(&setup, &app_role).await;
    let user_store = PgScimUserStore::from_pool(app.clone());
    let group_store = PgScimGroupStore::from_pool(app.clone());
    let passkeys = PgPasskeyStore::from_pool(app.clone());
    for tenant in ["ten_alpha", "ten_beta"] {
        let tenant_id = TenantId(tenant.to_owned());
        user_store
            .put(&bound_user("u1", "alice", "usr_alice", true), &tenant_id)
            .await
            .unwrap();
        group_store
            .put(&group("g1", "operators"), &tenant_id)
            .await
            .unwrap();
        passkeys
            .save_challenge(
                tenant,
                &[1; 32],
                "enroll",
                Some("usr_alice"),
                &serde_json::json!({"invitation": tenant}),
                ChallengeWindow {
                    issued_at_epoch_seconds: 100,
                    expires_at_epoch_seconds: 200,
                },
            )
            .await
            .unwrap();
        passkeys
            .insert_credential(
                tenant,
                "usr_alice",
                &[2; 32],
                &serde_json::json!({"credential": tenant}),
                17,
            )
            .await
            .unwrap();
        passkeys
            .create_session(tenant, "usr_alice", &[3; 32], 400)
            .await
            .unwrap();
    }
    let before = records(&setup, GOVERNED_TABLES).await;
    for table in &before {
        let rows: Vec<serde_json::Value> = serde_json::from_str(table).unwrap();
        assert_eq!(rows.len(), 2);
    }
    let mut setup_connection = setup.acquire().await.unwrap();
    provision_passkey_schema(&mut setup_connection)
        .await
        .unwrap();
    provision_passkey_schema(&mut setup_connection)
        .await
        .unwrap();
    assert_eq!(records(&setup, GOVERNED_TABLES).await, before);
    iam_identity_scim_store_postgres::assert_rls_enforceable(&app)
        .await
        .unwrap();

    let error = provision_passkey_schema(&mut app.acquire().await.unwrap())
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("42501")
    );
    assert_eq!(records(&setup, GOVERNED_TABLES).await, before);
    iam_identity_scim_store_postgres::assert_rls_enforceable(&app)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_passkey_schema_setup_rolls_back_partial_ddl_and_preserves_scim() {
    require_enabled();
    let setup = pool(&std::env::var(SETUP_URL_ENV).unwrap()).await;
    let app = pool(&std::env::var(APP_URL_ENV).unwrap()).await;
    let (app_role, rolsuper, rolbypassrls) = current_role_flags(&app).await;
    assert!(!rolsuper && !rolbypassrls);
    setup_schema(&setup, &app_role).await;
    let tenant = TenantId("ten_alpha".to_owned());
    PgScimUserStore::from_pool(app.clone())
        .put(&bound_user("u1", "alice", "usr_alice", true), &tenant)
        .await
        .unwrap();
    PgScimGroupStore::from_pool(app.clone())
        .put(&group("g1", "operators"), &tenant)
        .await
        .unwrap();
    let before = records(&setup, &[USERS_TABLE, GROUPS_TABLE]).await;
    // A late incompatible relation forces failure after two tables are created.
    for table in [PASSKEY_CHALLENGES_TABLE, PASSKEYS_TABLE, SESSIONS_TABLE] {
        sqlx::query(&format!("DROP TABLE {table}"))
            .execute(&setup)
            .await
            .unwrap();
    }
    sqlx::query(&format!("CREATE TABLE {SESSIONS_TABLE} (sentinel text)"))
        .execute(&setup)
        .await
        .unwrap();
    sqlx::query(&format!("INSERT INTO {SESSIONS_TABLE} VALUES ('keep me')"))
        .execute(&setup)
        .await
        .unwrap();
    let blocker_before = records(&setup, &[SESSIONS_TABLE]).await;
    assert!(
        iam_identity_scim_store_postgres::assert_rls_enforceable(&app)
            .await
            .is_err()
    );
    let mut setup_connection = setup.acquire().await.unwrap();
    let error = provision_passkey_schema(&mut setup_connection)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("42703")
    );
    for table in [PASSKEY_CHALLENGES_TABLE, PASSKEYS_TABLE] {
        let exists: bool = sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(table)
            .fetch_one(&mut *setup_connection)
            .await
            .unwrap();
        assert!(!exists, "partial setup leaked {table}");
    }
    assert_eq!(records(&setup, &[USERS_TABLE, GROUPS_TABLE]).await, before);
    assert_eq!(records(&setup, &[SESSIONS_TABLE]).await, blocker_before);
    assert!(
        iam_identity_scim_store_postgres::assert_rls_enforceable(&app)
            .await
            .is_err()
    );

    sqlx::query(&format!("DROP TABLE {SESSIONS_TABLE}"))
        .execute(&mut *setup_connection)
        .await
        .unwrap();
    provision_passkey_schema(&mut setup_connection)
        .await
        .unwrap();
    assert_eq!(records(&setup, &[USERS_TABLE, GROUPS_TABLE]).await, before);
    iam_identity_scim_store_postgres::assert_rls_enforceable(&app)
        .await
        .unwrap();
}
