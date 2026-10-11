use super::*;

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live postgres"]
async fn live_durable_scim_store_persists_across_service_rebuild() {
    let app_url = require_live_app_url();

    // ONE signing fixture across both boots so the JWKS is identical: a token
    // minted now must validate against the freshly-rebuilt service.
    let fixture = SigningFixture::generate();

    // A unique userName per run so repeated live runs against the same database
    // do not collide on the durable UNIQUE (tenant_id, user_name).
    let user_name = format!("durable-{}@acme.example", std::process::id());
    let payload = serde_json::json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
        "userName": user_name,
        "displayName": "Durable Provisioned User",
        "active": true,
    });
    let token = fixture.mint("wl_provisioner", "scim.manage");

    // Service #1: provision the user (durable store, born in Postgres).
    let handle = boot_with_url(&fixture, Some(app_url.clone()), None).await;
    let base = format!("http://{}", handle.rest_addr);
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/scim/v2/ten_acme/Users"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .expect("scim create responds");
    assert_eq!(
        response.status().as_u16(),
        201,
        "durable SCIM create must succeed"
    );
    let created: serde_json::Value = response.json().await.expect("created json");
    let id = created["id"].as_str().expect("assigned id").to_owned();
    assert_eq!(created["userName"], user_name.as_str());
    handle.shutdown().await;

    // Service #2: a FRESH composition over the SAME backend. The earlier write
    // is only observable here if it was durably persisted (the in-memory store,
    // being per-process state, would 404).
    let rebuilt = boot_with_url(&fixture, Some(app_url), None).await;
    let base2 = format!("http://{}", rebuilt.rest_addr);

    let response = client
        .get(format!("{base2}/scim/v2/ten_acme/Users/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("scim get responds");
    assert_eq!(
        response.status().as_u16(),
        200,
        "durable SCIM user must survive a service rebuild"
    );
    let fetched: serde_json::Value = response.json().await.expect("fetched json");
    assert_eq!(fetched["userName"], user_name.as_str());

    // A fresh process must also be able to allocate a new ID without
    // overwriting or colliding with the first process's durable user.
    let second_name = format!("second-durable-{}@acme.example", std::process::id());
    let second = client
        .post(format!("{base2}/scim/v2/ten_acme/Users"))
        .bearer_auth(&token)
        .json(&serde_json::json!({
            "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
            "userName": second_name,
            "active": true,
        }))
        .send()
        .await
        .expect("second durable create responds");
    assert_eq!(second.status().as_u16(), 201);
    let second: serde_json::Value = second.json().await.expect("second created json");
    assert_ne!(second["id"], id);
    let first_again: serde_json::Value = client
        .get(format!("{base2}/scim/v2/ten_acme/Users/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("first user still responds")
        .json()
        .await
        .expect("first user still parses");
    assert_eq!(first_again["userName"], user_name.as_str());

    // Facade-layer SCIM PEP cross-tenant deny: the `ten_acme` token hitting a
    // `ten_other` path is denied 403 (tenant-mismatch) BEFORE the store is
    // consulted. This is the PEP guard, NOT a store-layer RLS test.
    let cross = client
        .get(format!("{base2}/scim/v2/ten_other/Users/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("cross-tenant scim responds");
    assert_eq!(
        cross.status().as_u16(),
        403,
        "cross-tenant SCIM access must be denied 403 by the PEP before the store"
    );

    rebuilt.shutdown().await;
}
