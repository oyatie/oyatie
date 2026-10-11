use super::*;

#[tokio::test]
async fn native_bindings_require_scope_and_cedar_action_on_every_write_shape() {
    let fixture = fixture();
    let extension = User::OYATIE_SCHEMA;
    let mut payload = new_user_payload("bound@acme.example");
    payload[extension] = serde_json::json!({
        "nativeUserId": "usr_alice",
        "roles": ["foundry-operator"]
    });
    let users_uri = "/scim/v2/ten_acme/Users";
    for token in [&fixture.token, &fixture.cedar_denied_admin_token] {
        let (status, _) = send(
            &fixture.router,
            "POST",
            users_uri,
            Some(token),
            Some(payload.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    let (status, _) = send(
        &fixture.router,
        "POST",
        "/identity/v1/ten_acme/account-registrations",
        Some(&fixture.token),
        Some(payload.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, created) = send(
        &fixture.router,
        "POST",
        users_uri,
        Some(&fixture.admin_token),
        Some(payload),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created[extension]["nativeUserId"], "usr_alice");
    assert_eq!(created[extension]["roles"][0], "foundry-operator");
    let user_uri = format!("{users_uri}/{}", created["id"].as_str().expect("id"));

    let patch = |path: Option<&str>, value: serde_json::Value| {
        serde_json::json!({
            "schemas": [PatchOp::SCHEMA],
            "Operations": [{"op": "replace", "path": path, "value": value}]
        })
    };
    let native_id_path = format!("{extension}:nativeUserId");
    let roles_path = format!("{extension}:roles");
    for body in [
        patch(Some(&native_id_path), serde_json::json!("usr_bob")),
        patch(Some(&roles_path), serde_json::json!(["foundry-operator"])),
        patch(
            Some(extension),
            serde_json::json!({"nativeUserId":"usr_bob"}),
        ),
        patch(
            None,
            serde_json::json!({extension: {"nativeUserId":"usr_bob"}}),
        ),
    ] {
        let (status, _) = send(
            &fixture.router,
            "PATCH",
            &user_uri,
            Some(&fixture.token),
            Some(body),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    let (status, _) = send(
        &fixture.router,
        "PUT",
        &user_uri,
        Some(&fixture.token),
        Some(new_user_payload("bound@acme.example")),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = send(
        &fixture.router,
        "DELETE",
        &user_uri,
        Some(&fixture.token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = send(
        &fixture.router,
        "PATCH",
        &user_uri,
        Some(&fixture.admin_token),
        Some(patch(
            Some(extension),
            serde_json::json!({"nativeUserId":"usr_bob"}),
        )),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "whole-extension PATCH remains unsupported"
    );

    let (status, changed) = send(
        &fixture.router,
        "PATCH",
        &user_uri,
        Some(&fixture.admin_token),
        Some(patch(Some(&native_id_path), serde_json::json!("usr_bob"))),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(changed[extension]["nativeUserId"], "usr_bob");

    let (status, deactivated) = send(
        &fixture.router,
        "PATCH",
        &user_uri,
        Some(&fixture.token),
        Some(patch(Some("active"), serde_json::json!(false))),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(deactivated["active"], false);
    let (status, _) = send(
        &fixture.router,
        "PATCH",
        &user_uri,
        Some(&fixture.token),
        Some(patch(Some("active"), serde_json::json!(true))),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, reactivated) = send(
        &fixture.router,
        "PATCH",
        &user_uri,
        Some(&fixture.admin_token),
        Some(patch(Some("active"), serde_json::json!(true))),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reactivated["active"], true);
    let (status, _) = send(
        &fixture.router,
        "DELETE",
        &user_uri,
        Some(&fixture.admin_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}
