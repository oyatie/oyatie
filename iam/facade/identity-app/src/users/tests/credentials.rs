use super::*;

#[tokio::test]
async fn refuses_missing_and_invalid_credentials_with_401() {
    let fixture = fixture();
    let (status, body) = send(
        &fixture.router,
        "GET",
        "/scim/v2/ten_acme/Users",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["schemas"][0], ScimError::SCHEMA);
    // No credential presented -> no decision rendered -> no audit record.
    assert!(fixture.audit.is_empty());

    let (status, _) = send(
        &fixture.router,
        "GET",
        "/scim/v2/ten_acme/Users",
        Some("not-a-jwt"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // An invalid credential IS audited: one token-rejected record.
    assert_eq!(fixture.audit.len(), 1);
    let record = &fixture.audit.records()[0];
    assert_eq!(record.event(), AuditEvent::Authorize);
    assert_eq!(record.workload_id(), None);
    assert_eq!(record.outcome(), "token-rejected");
    assert_eq!(record.action(), Some(SCIM_MANAGE_ACTION));
    assert_eq!(record.resource_type(), Some(SCIM_TENANT_RESOURCE_TYPE));
    assert_eq!(record.resource_id(), Some("ten_acme"));
}

#[tokio::test]
async fn refuses_missing_scope_with_403() {
    let fixture = fixture();
    let (status, _) = send(
        &fixture.router,
        "GET",
        "/scim/v2/ten_acme/Users",
        Some(&fixture.wrong_scope_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // Exactly one deny record for the scope refusal.
    assert_eq!(fixture.audit.len(), 1);
    let record = &fixture.audit.records()[0];
    assert_eq!(record.outcome(), "deny");
    assert_eq!(record.detail(), Some("scope-missing"));
    assert_eq!(record.workload_id(), Some("wl_provisioner"));
}

#[tokio::test]
async fn refuses_unregistered_or_cross_tenant_scim_principals() {
    let fixture = fixture();
    let (status, _) = send(
        &fixture.router,
        "GET",
        "/scim/v2/ten_acme/Users",
        Some(&fixture.unknown_principal_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // The PEP hot path audited the fail-closed unknown-principal deny.
    assert_eq!(fixture.audit.len(), 1);
    assert_eq!(fixture.audit.records()[0].outcome(), "deny");
    assert_eq!(
        fixture.audit.records()[0].detail(),
        Some("principal-unknown")
    );

    let (status, _) = send(
        &fixture.router,
        "GET",
        "/scim/v2/ten_other/Users",
        Some(&fixture.token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // One additional deny record for the tenant-bind refusal.
    assert_eq!(fixture.audit.len(), 2);
    let record = &fixture.audit.records()[1];
    assert_eq!(record.outcome(), "deny");
    assert_eq!(record.detail(), Some("tenant-mismatch"));
    assert_eq!(record.workload_id(), Some("wl_provisioner"));
    assert_eq!(record.resource_id(), Some("ten_other"));
}
