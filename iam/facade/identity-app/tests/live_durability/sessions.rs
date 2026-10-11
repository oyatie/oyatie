use super::*;

/// The served session endpoint resolves the durable token and current native
/// membership on each request. Logout removes the token; deprovisioning also
/// refuses an otherwise unexpired token.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "live postgres"]
async fn live_passkey_session_is_tenant_scoped_revocable_and_membership_current() {
    let app_url = require_live_app_url();
    let fixture = SigningFixture::generate();
    let handle = boot_with_url(
        &fixture,
        Some(app_url.clone()),
        Some("https://identity.example.test"),
    )
    .await;
    let base = format!("http://{}", handle.rest_addr);
    let client = reqwest::Client::new();
    let user_name = format!("session-{}@acme.example", uuid::Uuid::new_v4());
    let created = client
        .post(format!("{base}/scim/v2/ten_acme/Users"))
        .bearer_auth(fixture.mint("wl_provisioner", "scim.manage"))
        .json(&serde_json::json!({
            "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
            "userName": user_name,
            "active": true,
        }))
        .send()
        .await
        .expect("create session user responds");
    assert_eq!(created.status().as_u16(), 201);
    let unbound: User = created.json().await.expect("created user parses");
    let native_user_id = format!("usr_session_{}", uuid::Uuid::new_v4());
    let mut bound = unbound.clone();
    bound.schemas.push(User::OYATIE_SCHEMA.to_owned());
    bound.oyatie = Some(OyatieExtension {
        native_user_id: Some(native_user_id.clone()),
        roles: Some(vec!["foundry-operator".to_owned()]),
        ..Default::default()
    });
    let pool = sqlx::PgPool::connect(&app_url)
        .await
        .expect("app role connects");
    let users = PgScimUserStore::from_pool(pool.clone());
    let tenant = TenantId("ten_acme".to_owned());
    users
        .compare_and_put(&unbound, &bound, &tenant, true)
        .await
        .expect("test setup binds native identity");
    let sessions = PgPasskeyStore::from_pool(pool);
    let expires_at = epoch_now() + 3600;

    let first_token = format!("v1.{}", b64url(&[9u8; 32]));
    let first_hash = digest(&SHA256, first_token.as_bytes());
    sessions
        .create_session("ten_acme", &native_user_id, first_hash.as_ref(), expires_at)
        .await
        .expect("test setup creates session");
    let cookie = format!("__Host-oya_session={first_token}");
    let current = client
        .get(format!("{base}/identity/v1/ten_acme/session"))
        .header("cookie", &cookie)
        .send()
        .await
        .expect("current session responds");
    assert_eq!(current.status().as_u16(), 200);
    let current: serde_json::Value = current.json().await.expect("session view parses");
    assert_eq!(current["tenantId"], "ten_acme");
    assert_eq!(current["userId"], native_user_id);
    assert_eq!(current["roles"], serde_json::json!(["foundry-operator"]));
    let wrong_tenant = client
        .get(format!("{base}/identity/v1/ten_other/session"))
        .header("cookie", &cookie)
        .send()
        .await
        .expect("wrong-tenant session responds");
    assert_eq!(wrong_tenant.status().as_u16(), 401);
    let logout = client
        .post(format!("{base}/identity/v1/ten_acme/session:logout"))
        .header("cookie", &cookie)
        .header("origin", "https://identity.example.test")
        .send()
        .await
        .expect("logout responds");
    assert_eq!(logout.status().as_u16(), 204);
    assert!(
        sessions
            .session("ten_acme", first_hash.as_ref(), epoch_now())
            .await
            .expect("session read")
            .is_none()
    );

    let second_token = format!("v1.{}", b64url(&[10u8; 32]));
    let second_hash = digest(&SHA256, second_token.as_bytes());
    sessions
        .create_session(
            "ten_acme",
            &native_user_id,
            second_hash.as_ref(),
            expires_at,
        )
        .await
        .expect("test setup creates second session");
    let mut inactive = bound.clone();
    inactive.active = false;
    users
        .compare_and_put(&bound, &inactive, &tenant, true)
        .await
        .expect("test setup deactivates user");
    let denied = client
        .get(format!("{base}/identity/v1/ten_acme/session"))
        .header("cookie", format!("__Host-oya_session={second_token}"))
        .send()
        .await
        .expect("inactive session responds");
    assert_eq!(denied.status().as_u16(), 401);
    handle.shutdown().await;
}
