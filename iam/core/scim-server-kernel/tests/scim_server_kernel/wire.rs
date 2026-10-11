use super::*;

#[test]
fn scim_error_envelope_serializes_per_rfc_7644() {
    let e = ScimError::new(409, Some(ScimType::Uniqueness), "duplicate userName");
    let s = serde_json::to_string(&e).expect("ser");
    assert!(s.contains("\"status\":409"));
    assert!(s.contains("\"scimType\":\"uniqueness\""));
    assert!(s.contains("urn:ietf:params:scim:api:messages:2.0:Error"));
}

#[test]
fn new_user_deserializes_rfc_7644_camelcase_wire_shape() {
    // The shape real SCIM clients (Okta / Entra / Workspace) actually send:
    // camelCase members, extension content under its URN, absent members
    // omitted entirely (RFC 7644 §3.3).
    let okta_style = json!({
        "schemas": [
            "urn:ietf:params:scim:schemas:core:2.0:User",
            "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User"
        ],
        "userName": "amara@acme.example",
        "externalId": "00u1abcd",
        "displayName": "Amara A.",
        "active": true,
        "name": {"givenName": "Amara", "familyName": "A."},
        "emails": [{"value": "amara@acme.example", "type": "work", "primary": true}],
        "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User": {
            "employeeNumber": "E-1001",
            "department": "Platform"
        }
    });
    let parsed: NewUser = serde_json::from_value(okta_style).expect("camelCase parses");
    assert_eq!(parsed.user_name, "amara@acme.example");
    assert_eq!(parsed.external_id.as_deref(), Some("00u1abcd"));
    assert_eq!(parsed.display_name.as_deref(), Some("Amara A."));
    assert!(parsed.active);
    assert_eq!(
        parsed
            .enterprise
            .as_ref()
            .and_then(|e| e.employee_number.as_deref()),
        Some("E-1001")
    );

    // Backward-compat alias: the pre-RFC snake_case shape still parses.
    let snake_style = json!({
        "user_name": "amara@acme.example",
        "external_id": "00u1abcd",
        "display_name": "Amara A.",
        "active": true,
        "enterprise": {"employeeNumber": "E-1001"}
    });
    let parsed: NewUser = serde_json::from_value(snake_style).expect("snake_case alias parses");
    assert_eq!(parsed.user_name, "amara@acme.example");
    assert_eq!(parsed.display_name.as_deref(), Some("Amara A."));

    // NewGroup: camelCase + alias, members omittable.
    let group: NewGroup =
        serde_json::from_value(json!({"displayName": "platform-admins"})).expect("camelCase");
    assert_eq!(group.display_name, "platform-admins");
    let group: NewGroup =
        serde_json::from_value(json!({"display_name": "platform-admins"})).expect("alias");
    assert_eq!(group.display_name, "platform-admins");
}

#[tokio::test]
async fn user_response_serializes_display_name_as_camelcase() {
    let s = srv();
    let mut input = new_user("amara@acme.example");
    input.display_name = Some("Amara A.".into());
    let user = s
        .create_user(&tenant(), input, 1_700_000_000)
        .await
        .expect("create");
    let value = serde_json::to_value(&user).expect("serialize");
    assert_eq!(value["displayName"], "Amara A.");
    assert!(
        value.get("display_name").is_none(),
        "responses are RFC 7644 camelCase only"
    );
}
#[tokio::test]
async fn tenant_isolation_users_dont_leak_across_tenants() {
    let s = srv();
    let t1 = TenantId("tenant-1".into());
    let t2 = TenantId("tenant-2".into());
    s.create_user(&t1, new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    s.create_user(&t2, new_user("alice"), 1_700_000_000)
        .await
        .expect("c"); // same userName different tenant OK
    let r1 = s
        .list_users(&t1, &ListQuery::default())
        .await
        .expect("list");
    let r2 = s
        .list_users(&t2, &ListQuery::default())
        .await
        .expect("list");
    assert_eq!(r1.total_results, 1);
    assert_eq!(r2.total_results, 1);
    assert_ne!(r1.resources[0].id, r2.resources[0].id);
}
