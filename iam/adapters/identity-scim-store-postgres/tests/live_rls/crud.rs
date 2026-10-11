use super::*;

#[tokio::test]
#[ignore = "live postgres"]
async fn live_app_role_has_no_bypassrls() {
    require_enabled();
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let app = pool(&app_url).await;
    let (name, rolsuper, rolbypassrls) = current_role_flags(&app).await;
    assert!(
        !rolsuper && !rolbypassrls,
        "app role {name} must NOT carry rolsuper/rolbypassrls (RLS would be skipped)"
    );
}

#[tokio::test]
#[ignore = "live postgres"]
async fn live_scim_tenant_scoped_crud_and_uniqueness() {
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

    store.put(&user("u-1", "alice"), &alpha).await.unwrap();
    // Same userName under a DIFFERENT tenant is allowed (per-tenant uniqueness).
    store.put(&user("u-2", "alice"), &beta).await.unwrap();

    assert_eq!(
        store.get(&alpha, &ScimId("u-1".to_owned())).await,
        Some(user("u-1", "alice"))
    );
    assert_eq!(
        store
            .find_by_user_name(&alpha, "alice")
            .await
            .map(|u| u.id.0),
        Some("u-1".to_owned())
    );
    // Tenant alpha lists only its own user.
    let alpha_users = store.list(&alpha).await;
    assert_eq!(alpha_users.len(), 1);
    assert_eq!(alpha_users[0].id.0, "u-1");
}
