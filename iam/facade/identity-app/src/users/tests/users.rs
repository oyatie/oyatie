use super::*;

#[tokio::test]
async fn user_lifecycle_and_tenant_isolation() {
    let fixture = fixture();
    let token = fixture.token.as_str();

    // Create -> 201 with a kernel-assigned id.
    let (status, created) = send(
        &fixture.router,
        "POST",
        "/scim/v2/ten_acme/Users",
        Some(token),
        Some(new_user_payload("amara@acme.example")),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().expect("assigned id").to_owned();

    // Get + list in the owning tenant.
    let (status, fetched) = send(
        &fixture.router,
        "GET",
        &format!("/scim/v2/ten_acme/Users/{id}"),
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["userName"], "amara@acme.example");

    let (status, listing) = send(
        &fixture.router,
        "GET",
        "/scim/v2/ten_acme/Users",
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listing["totalResults"].as_u64(), Some(1));

    // Cross-tenant read fails at the SCIM route guard before the store is
    // consulted: the token tenant must equal the path tenant.
    let (status, _) = send(
        &fixture.router,
        "GET",
        &format!("/scim/v2/ten_other/Users/{id}"),
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Existing SCIM provisioning may still delete an unbound user.
    let (status, _) = send(
        &fixture.router,
        "DELETE",
        &format!("/scim/v2/ten_acme/Users/{id}"),
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(
        &fixture.router,
        "GET",
        &format!("/scim/v2/ten_acme/Users/{id}"),
        Some(token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    assert_eq!(fixture.audit.len(), 6);
    let records = fixture.audit.records();
    let outcomes: Vec<&str> = records.iter().map(AuditRecord::outcome).collect();
    assert_eq!(
        outcomes,
        ["allow", "allow", "allow", "deny", "allow", "allow"]
    );
}

#[tokio::test]
async fn account_registration_wrapper_reuses_scim_create_and_guard() {
    let fixture = fixture();
    let payload = new_user_payload("register@acme.example");

    let (status, _) = send(
        &fixture.router,
        "POST",
        "/identity/v1/ten_acme/account-registrations",
        None,
        Some(payload.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = send(
        &fixture.router,
        "POST",
        "/identity/v1/ten_acme/account-registrations",
        Some(&fixture.wrong_scope_token),
        Some(payload.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, created) = send(
        &fixture.router,
        "POST",
        "/identity/v1/ten_acme/account-registrations",
        Some(&fixture.token),
        Some(payload),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["userName"], "register@acme.example");
    let id = created["id"].as_str().expect("assigned id").to_owned();

    let (status, fetched) = send(
        &fixture.router,
        "GET",
        &format!("/scim/v2/ten_acme/Users/{id}"),
        Some(&fixture.token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched["id"], id);

    let (status, _) = send(
        &fixture.router,
        "POST",
        "/identity/v1/ten_other/account-registrations",
        Some(&fixture.token),
        Some(new_user_payload("cross-tenant@acme.example")),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn ordinary_email_removal_does_not_require_native_admin() {
    let fixture = fixture();
    let mut body = new_user_payload("filtered@acme.example");
    body["emails"] = serde_json::json!([
        {"value":"roles@example.com"}, {"value":"nativeUserId@example.com"}
    ]);
    let (status, created) = send(
        &fixture.router,
        "POST",
        "/scim/v2/ten_acme/Users",
        Some(&fixture.token),
        Some(body),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let uri = format!(
        "/scim/v2/ten_acme/Users/{}",
        created["id"].as_str().expect("id")
    );
    for email in ["roles@example.com", "nativeUserId@example.com"] {
        let (status, _) = send(
            &fixture.router,
            "PATCH",
            &uri,
            Some(&fixture.token),
            Some(
                serde_json::json!({"schemas":[PatchOp::SCHEMA], "Operations":[{
                    "op":"remove", "path":format!("emails[value eq \"{email}\"]")
                }]}),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
    let (status, user) = send(&fixture.router, "GET", &uri, Some(&fixture.token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        user.get("emails")
            .is_none_or(|emails| emails.as_array().is_some_and(Vec::is_empty))
    );
}
