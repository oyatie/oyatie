use super::*;

#[tokio::test]
#[ignore = "live postgres"]
async fn live_rls_denies_cross_tenant_read_and_write() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let store = PgScimUserStore::from_pool(app.clone());
    let alpha = TenantId("alpha".to_owned());
    let beta = TenantId("beta".to_owned());
    store.put(&user("a-1", "anna"), &alpha).await.unwrap();
    store.put(&user("b-1", "bob"), &beta).await.unwrap();

    // Cross-tenant read: tenant alpha cannot see tenant beta's row.
    assert_eq!(store.get(&alpha, &ScimId("b-1".to_owned())).await, None);

    // Cross-tenant write: under alpha's GUC, inserting a beta-scoped row is
    // denied by the PERMISSIVE policy's
    // `WITH CHECK (tenant_id = current_setting('oyatie.tenant_id'))` — the row's
    // tenant_id ('beta') does not equal the session GUC ('alpha'). The
    // RESTRICTIVE require_tenant_guc policy is NOT what denies this case: the GUC
    // IS set (to 'alpha'), so the restrictive WITH CHECK is satisfied; it is the
    // permissive tenant-equality WITH CHECK that rejects the wrong-tenant row.
    let mut tx = app.begin().await.unwrap();
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("alpha")
        .execute(&mut *tx)
        .await
        .unwrap();
    let cross = sqlx::query(&format!(
        "INSERT INTO {USERS_TABLE} (tenant_id, scim_id, user_name, external_id, active, payload_json, schema_version, updated_at) VALUES ('beta', 'x-1', 'x', '', true, '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        cross.is_err(),
        "cross-tenant INSERT under tenant alpha's GUC must be denied by RLS"
    );
    let _ = tx.rollback().await;
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_rls_denies_cross_tenant_on_groups() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let store = PgScimGroupStore::from_pool(app.clone());
    let alpha = TenantId("alpha".to_owned());
    let beta = TenantId("beta".to_owned());
    store.put(&group("ga-1", "alphas"), &alpha).await.unwrap();
    store.put(&group("gb-1", "betas"), &beta).await.unwrap();

    // Each tenant lists only its own group.
    let alpha_groups = store.list(&alpha).await;
    assert_eq!(alpha_groups.len(), 1);
    assert_eq!(alpha_groups[0].id.0, "ga-1");

    // Cross-tenant read: tenant alpha cannot see tenant beta's group row.
    assert_eq!(store.get(&alpha, &ScimId("gb-1".to_owned())).await, None);

    // Cross-tenant write: under alpha's GUC, inserting a beta-scoped group row
    // is rejected by the PERMISSIVE WITH CHECK (tenant_id = GUC) — the row's
    // tenant_id ('beta') does not equal the session GUC ('alpha').
    let mut tx = app.begin().await.unwrap();
    sqlx::query(SET_LOCAL_TENANT_SQL)
        .bind("alpha")
        .execute(&mut *tx)
        .await
        .unwrap();
    let cross = sqlx::query(&format!(
        "INSERT INTO {GROUPS_TABLE} (tenant_id, scim_id, display_name, payload_json, schema_version, updated_at) VALUES ('beta', 'gx-1', 'x', '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        cross.is_err(),
        "cross-tenant groups INSERT under alpha's GUC must be denied by the permissive WITH CHECK"
    );
    let _ = tx.rollback().await;
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_rls_unset_guc_denies_all_access() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    // Seed a users row and a groups row (each via their own tenant GUC) so the
    // no-GUC SELECTs have something they MUST be denied from seeing.
    let user_store = PgScimUserStore::from_pool(app.clone());
    let group_store = PgScimGroupStore::from_pool(app.clone());
    let gamma = TenantId("gamma".to_owned());
    user_store
        .put(&user("u-g", "ugamma"), &gamma)
        .await
        .unwrap();
    group_store
        .put(&group("g-g", "ggamma"), &gamma)
        .await
        .unwrap();

    // With NO per-tx GUC set, the RESTRICTIVE require_tenant_guc policy hard-
    // denies access: current_setting('oyatie.tenant_id', true) is NULL, so the
    // restrictive USING/WITH CHECK fails and intersects to deny-all. Each table
    // must return 0 rows on SELECT — proving a missing SET_LOCAL_TENANT_SQL can
    // never fall back to an open scan.
    for (table, select_sql) in [
        (
            USERS_TABLE,
            format!("SELECT count(*)::bigint FROM {USERS_TABLE}"),
        ),
        (
            GROUPS_TABLE,
            format!("SELECT count(*)::bigint FROM {GROUPS_TABLE}"),
        ),
    ] {
        let mut tx = app.begin().await.unwrap();
        // Deliberately do NOT set the tenant GUC.
        let count: i64 = sqlx::query_scalar(&select_sql)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        let _ = tx.rollback().await;
        assert_eq!(
            count, 0,
            "no GUC set: SELECT on {table} must return 0 rows (restrictive deny-all)"
        );
    }

    // INSERT with no GUC set is rejected by the restrictive WITH CHECK.
    let mut tx = app.begin().await.unwrap();
    let insert = sqlx::query(&format!(
        "INSERT INTO {USERS_TABLE} (tenant_id, scim_id, user_name, external_id, active, payload_json, schema_version, updated_at) VALUES ('gamma', 'u-x', 'x', NULL, true, '{{}}'::jsonb, 1, now())"
    ))
    .execute(&mut *tx)
    .await;
    assert!(
        insert.is_err(),
        "no GUC set: INSERT must be denied by the restrictive require_tenant_guc WITH CHECK"
    );
    let _ = tx.rollback().await;
}
