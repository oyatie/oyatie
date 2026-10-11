use super::*;

#[tokio::test]
#[ignore = "live postgres"]
async fn live_native_membership_lookup_is_exact_tenant_fallible_and_ambiguous() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, _, _) = current_role_flags(&app).await;
    setup_schema(&setup, &app_role).await;

    let store = PgScimUserStore::from_pool(app);
    let alpha = TenantId("ten_alpha".into());
    let beta = TenantId("ten_beta".into());
    store
        .put(&bound_user("a-1", "alice-a", "usr_alice", true), &alpha)
        .await
        .unwrap();
    store
        .put(&bound_user("b-1", "alice-b", "usr_alice", true), &beta)
        .await
        .unwrap();
    store
        .put(&bound_user("a-2", "bob-a", "usr_bob", false), &alpha)
        .await
        .unwrap();

    let alpha_alice = store.candidates("ten_alpha", "usr_alice").await.unwrap();
    assert_eq!(alpha_alice.len(), 1);
    assert!(alpha_alice[0].active);
    assert_eq!(alpha_alice[0].tenant_id, "ten_alpha");
    assert_eq!(alpha_alice[0].roles, vec!["foundry-operator"]);
    assert_eq!(
        store
            .candidates("ten_alpha", "usr_absent")
            .await
            .unwrap()
            .len(),
        0
    );
    let inactive = store.candidates("ten_alpha", "usr_bob").await.unwrap();
    assert_eq!(inactive.len(), 1);
    assert!(!inactive[0].active);

    store
        .put(&bound_user("a-3", "alice-two", "usr_alice", true), &alpha)
        .await
        .unwrap();
    assert_eq!(
        store
            .candidates("ten_alpha", "usr_alice")
            .await
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        store
            .candidates("ten_beta", "usr_alice")
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(store.candidates(" ", "usr_alice").await.is_err());
}
