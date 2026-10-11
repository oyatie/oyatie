use super::*;

/// AC-W-13 on the SCIM surface: a SCIM ALLOW emits exactly one audit
/// record carrying the right principal/action/resource/outcome — and no
/// token material.
#[tokio::test]
async fn scim_allow_emits_exactly_one_audit_record_with_target() {
    let fixture = fixture();
    let (status, _) = send(
        &fixture.router,
        "GET",
        "/scim/v2/ten_acme/Users",
        Some(&fixture.token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fixture.audit.len(), 1);
    let record = &fixture.audit.records()[0];
    assert_eq!(record.event(), AuditEvent::Authorize);
    assert_eq!(record.workload_id(), Some("wl_provisioner"));
    assert_eq!(record.outcome(), "allow");
    assert_eq!(record.detail(), Some("permit-scim-manage"));
    assert_eq!(record.action(), Some(SCIM_MANAGE_ACTION));
    assert_eq!(record.resource_type(), Some(SCIM_TENANT_RESOURCE_TYPE));
    assert_eq!(record.resource_id(), Some("ten_acme"));
    // The bearer credential must never reach the audit chain: no segment
    // of the JWT may appear anywhere in the sealed record.
    let rendered = format!("{record:?}");
    assert!(!rendered.contains(&fixture.token));
    for segment in fixture.token.split('.') {
        assert!(
            !rendered.contains(segment),
            "audit record leaked token material"
        );
    }
}

/// AC-W-13 on the SCIM surface: a SCIM policy DENY (registered, scoped,
/// tenant-bound principal that Cedar default-denies) emits exactly one
/// audit record carrying the right principal/action/resource/outcome.
#[tokio::test]
async fn scim_cedar_deny_emits_exactly_one_audit_record_with_target() {
    let fixture = fixture();
    let (status, _) = send(
        &fixture.router,
        "GET",
        "/scim/v2/ten_other/Users",
        Some(&fixture.cedar_denied_token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(fixture.audit.len(), 1);
    let record = &fixture.audit.records()[0];
    assert_eq!(record.event(), AuditEvent::Authorize);
    assert_eq!(record.workload_id(), Some("wl_other_provisioner"));
    assert_eq!(record.outcome(), "deny");
    assert_eq!(record.detail(), Some("default-deny"));
    assert_eq!(record.action(), Some(SCIM_MANAGE_ACTION));
    assert_eq!(record.resource_type(), Some(SCIM_TENANT_RESOURCE_TYPE));
    assert_eq!(record.resource_id(), Some("ten_other"));
    let rendered = format!("{record:?}");
    for segment in fixture.cedar_denied_token.split('.') {
        assert!(
            !rendered.contains(segment),
            "audit record leaked token material"
        );
    }
}
