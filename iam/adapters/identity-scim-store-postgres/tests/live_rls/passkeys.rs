use super::*;

#[tokio::test]
#[ignore = "live postgres"]
async fn live_passkeys_are_tenant_scoped_single_use_and_retry_safe() {
    require_enabled();
    let setup_url = std::env::var(SETUP_URL_ENV).expect("SETUP url required when enabled");
    let app_url = std::env::var(APP_URL_ENV).expect("APP url required when enabled");
    let setup = pool(&setup_url).await;
    let app = pool(&app_url).await;
    let (app_role, rolsuper, rolbypassrls) = current_role_flags(&app).await;
    assert!(!rolsuper && !rolbypassrls);
    setup_schema(&setup, &app_role).await;
    iam_identity_scim_store_postgres::assert_rls_enforceable(&app)
        .await
        .unwrap();
    let store = PgPasskeyStore::from_pool(app.clone());
    let challenge = [7u8; 32];
    store
        .save_challenge(
            "ten_alpha",
            &challenge,
            "enroll",
            Some("usr_alice"),
            &serde_json::json!({"nonce": 1}),
            ChallengeWindow {
                issued_at_epoch_seconds: 100,
                expires_at_epoch_seconds: 200,
            },
        )
        .await
        .unwrap();
    assert!(
        store
            .take_challenge("ten_beta", &challenge, "enroll", 150)
            .await
            .unwrap()
            .is_none()
    );

    // A migration retry must not delete an outstanding invitation.
    for sql in split_migration_statements(PASSKEY_AUTHENTICATION_SQL) {
        sqlx::query(&sql).execute(&setup).await.unwrap();
    }
    let taken = store
        .take_challenge("ten_alpha", &challenge, "enroll", 150)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(taken.user_id.as_deref(), Some("usr_alice"));
    assert!(
        store
            .take_challenge("ten_alpha", &challenge, "enroll", 150)
            .await
            .unwrap()
            .is_none()
    );

    let id = [8u8; 32];
    let credential = serde_json::json!({"key": "opaque"});
    store
        .insert_credential("ten_alpha", "usr_alice", &id, &credential, 0)
        .await
        .unwrap();
    assert_eq!(
        store
            .insert_credential("ten_alpha", "usr_alice", &id, &credential, 0)
            .await,
        Err(PasskeyStoreError::Conflict)
    );
    assert!(
        store
            .credential("ten_beta", "usr_alice", &id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .credential("ten_alpha", "usr_bob", &id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .compare_and_set_counter("ten_alpha", "usr_alice", &id, &credential, 0, 1)
            .await
            .unwrap()
    );
    assert!(
        !store
            .compare_and_set_counter("ten_alpha", "usr_alice", &id, &credential, 0, 2)
            .await
            .unwrap()
    );

    let hash = [9u8; 32];
    store
        .create_session("ten_alpha", "usr_alice", &hash, 400)
        .await
        .unwrap();
    assert!(
        store
            .session("ten_beta", &hash, 300)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        store
            .session("ten_alpha", &hash, 400)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .session("ten_alpha", &hash, 300)
            .await
            .unwrap()
            .unwrap()
            .user_id,
        "usr_alice"
    );
    store.delete_session("ten_alpha", &hash).await.unwrap();
    assert!(
        store
            .session("ten_alpha", &hash, 300)
            .await
            .unwrap()
            .is_none()
    );

    // Without SET LOCAL tenant, even a privileged table SELECT sees no rows.
    let rows = sqlx::query("SELECT credential_id FROM identity_scim.identity_user_passkeys")
        .fetch_all(&app)
        .await
        .unwrap();
    assert!(rows.is_empty());
}
