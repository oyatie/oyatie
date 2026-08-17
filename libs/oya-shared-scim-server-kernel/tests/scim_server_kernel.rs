#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Integration tests for `oya-shared-scim-server-kernel`.

use oya_shared_scim_server_kernel::{
    CounterIdGen, Email, FilterExpr, Group, GroupMembership, InMemoryGroupStore, InMemoryUserStore,
    ListQuery, NewGroup, NewUser, PatchOp, PatchOpKind, PatchOperation, ReferenceScimServer,
    ScimError, ScimId, ScimServer, ScimType, TenantId, User, UserName, parse_filter,
};
use serde_json::json;

fn srv() -> ReferenceScimServer<InMemoryUserStore, InMemoryGroupStore, CounterIdGen> {
    ReferenceScimServer::new(
        InMemoryUserStore::default(),
        InMemoryGroupStore::default(),
        CounterIdGen::new(),
        "https://identity.oyatie.com/scim/v2",
    )
}

fn tenant() -> TenantId {
    TenantId("tenant-acme".into())
}

fn new_user(name: &str) -> NewUser {
    NewUser {
        user_name: name.into(),
        external_id: Some(format!("ext-{name}")),
        name: Some(UserName {
            given_name: Some("First".into()),
            family_name: Some("Last".into()),
            ..Default::default()
        }),
        display_name: Some(name.into()),
        active: true,
        emails: vec![Email {
            value: format!("{name}@example.com"),
            r#type: Some("work".into()),
            primary: Some(true),
            display: None,
        }],
        enterprise: None,
        oyatie: None,
    }
}

#[tokio::test]
async fn create_then_get_user_roundtrips() {
    let s = srv();
    let u = s
        .create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("create");
    assert_eq!(u.user_name, "alice");
    let got = s.get_user(&tenant(), &u.id).await.expect("get");
    assert_eq!(got.id, u.id);
    assert!(got.meta.location.contains("/Users/"));
    assert!(got.meta.version.starts_with("W/\""));
    assert!(got.schemas.contains(&User::CORE_SCHEMA.to_owned()));
}

#[tokio::test]
async fn create_user_username_uniqueness() {
    let s = srv();
    s.create_user(&tenant(), new_user("bob"), 1_700_000_000)
        .await
        .expect("create1");
    let err = s
        .create_user(&tenant(), new_user("bob"), 1_700_000_001)
        .await
        .unwrap_err();
    assert_eq!(err.status, 409);
    assert_eq!(err.scim_type, Some(ScimType::Uniqueness));
}

#[tokio::test]
async fn create_user_requires_username() {
    let s = srv();
    let mut nu = new_user("");
    nu.user_name = String::new();
    let err = s
        .create_user(&tenant(), nu, 1_700_000_000)
        .await
        .unwrap_err();
    assert_eq!(err.status, 400);
    assert_eq!(err.scim_type, Some(ScimType::InvalidValue));
}

#[tokio::test]
async fn list_users_paginates() {
    let s = srv();
    for i in 0..5 {
        s.create_user(
            &tenant(),
            new_user(&format!("user{i}")),
            1_700_000_000 + i as i64,
        )
        .await
        .expect("create");
    }
    let page1 = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 2,
                filter: None,
            },
        )
        .await
        .expect("list");
    assert_eq!(page1.total_results, 5);
    assert_eq!(page1.resources.len(), 2);
    assert_eq!(page1.start_index, 1);
    let page2 = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 3,
                items_per_page: 2,
                filter: None,
            },
        )
        .await
        .expect("list");
    assert_eq!(page2.resources.len(), 2);
    assert_eq!(page2.start_index, 3);
}

#[tokio::test]
async fn list_users_caps_items_per_page() {
    let mut s = srv();
    s.max_items_per_page = 50;
    let r = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 9_999,
                filter: None,
            },
        )
        .await
        .expect("list");
    assert!(r.items_per_page <= 50);
}

#[tokio::test]
async fn patch_user_replaces_active() {
    let s = srv();
    let u = s
        .create_user(&tenant(), new_user("carol"), 1_700_000_000)
        .await
        .expect("create");
    let patched = s
        .patch_user(
            &tenant(),
            &u.id,
            &PatchOp {
                schemas: vec![PatchOp::SCHEMA.to_owned()],
                operations: vec![PatchOperation {
                    op: PatchOpKind::Replace,
                    path: Some("active".into()),
                    value: Some(json!(false)),
                }],
            },
            1_700_000_100,
        )
        .await
        .expect("patch");
    assert!(!patched.active);
}

#[tokio::test]
async fn patch_user_unknown_path_returns_invalidpath() {
    let s = srv();
    let u = s
        .create_user(&tenant(), new_user("dave"), 1_700_000_000)
        .await
        .expect("create");
    let err = s
        .patch_user(
            &tenant(),
            &u.id,
            &PatchOp {
                schemas: vec![PatchOp::SCHEMA.to_owned()],
                operations: vec![PatchOperation {
                    op: PatchOpKind::Replace,
                    path: Some("nosuchthing".into()),
                    value: Some(json!("x")),
                }],
            },
            1_700_000_100,
        )
        .await
        .unwrap_err();
    assert_eq!(err.scim_type, Some(ScimType::InvalidPath));
}

#[tokio::test]
async fn delete_user_then_get_404s() {
    let s = srv();
    let u = s
        .create_user(&tenant(), new_user("eve"), 1_700_000_000)
        .await
        .expect("create");
    s.delete_user(&tenant(), &u.id).await.expect("delete");
    let err = s.get_user(&tenant(), &u.id).await.unwrap_err();
    assert_eq!(err.status, 404);
}

#[tokio::test]
async fn delete_unknown_user_404s() {
    let s = srv();
    let err = s
        .delete_user(&tenant(), &ScimId("missing".into()))
        .await
        .unwrap_err();
    assert_eq!(err.status, 404);
}

#[tokio::test]
async fn create_group_then_patch_members() {
    let s = srv();
    let g = s
        .create_group(
            &tenant(),
            NewGroup {
                display_name: "engineers".into(),
                members: vec![],
            },
            1_700_000_000,
        )
        .await
        .expect("create");
    let alice = s
        .create_user(&tenant(), new_user("alice"), 1_700_000_001)
        .await
        .expect("u");
    let patched = s
        .patch_group(
            &tenant(),
            &g.id,
            &PatchOp {
                schemas: vec![PatchOp::SCHEMA.to_owned()],
                operations: vec![PatchOperation {
                    op: PatchOpKind::Add,
                    path: Some("members".into()),
                    value: Some(json!([{"value": alice.id.0, "display": "alice"}])),
                }],
            },
            1_700_000_100,
        )
        .await
        .expect("patch");
    assert_eq!(patched.members.len(), 1);
    assert_eq!(patched.members[0].value, alice.id);
}

#[tokio::test]
async fn replace_user_preserves_created_timestamp() {
    let s = srv();
    let u = s
        .create_user(&tenant(), new_user("frank"), 1_700_000_000)
        .await
        .expect("create");
    let original_created = u.meta.created.clone();
    let mut nu = new_user("frank");
    nu.display_name = Some("Frank Renamed".into());
    let replaced = s
        .replace_user(&tenant(), &u.id, nu, 1_700_000_500)
        .await
        .expect("replace");
    assert_eq!(replaced.meta.created, original_created);
    assert_ne!(replaced.meta.last_modified, original_created);
    assert_eq!(replaced.display_name.as_deref(), Some("Frank Renamed"));
}

#[tokio::test]
async fn filter_eq_matches_username() {
    let s = srv();
    s.create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    s.create_user(&tenant(), new_user("bob"), 1_700_000_001)
        .await
        .expect("c");
    let r = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 100,
                filter: Some(r#"userName eq "alice""#.into()),
            },
        )
        .await
        .expect("list");
    assert_eq!(r.total_results, 1);
    assert_eq!(r.resources[0].user_name, "alice");
}

#[tokio::test]
async fn filter_co_matches_substring() {
    let s = srv();
    s.create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    s.create_user(&tenant(), new_user("bob"), 1_700_000_001)
        .await
        .expect("c");
    let r = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 100,
                filter: Some(r#"userName co "li""#.into()),
            },
        )
        .await
        .expect("list");
    assert_eq!(r.total_results, 1);
}

#[tokio::test]
async fn filter_and_combines_conditions() {
    let s = srv();
    s.create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    s.create_user(&tenant(), new_user("alex"), 1_700_000_001)
        .await
        .expect("c");
    s.create_user(&tenant(), new_user("bob"), 1_700_000_002)
        .await
        .expect("c");
    let r = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 100,
                filter: Some(r#"userName sw "al" and active eq "true""#.into()),
            },
        )
        .await
        .expect("list");
    assert_eq!(r.total_results, 2);
}

#[tokio::test]
async fn filter_pr_present() {
    let s = srv();
    s.create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    let r = s
        .list_users(
            &tenant(),
            &ListQuery {
                start_index: 1,
                items_per_page: 100,
                filter: Some("userName pr".into()),
            },
        )
        .await
        .expect("list");
    assert_eq!(r.total_results, 1);
}

#[test]
fn filter_parser_rejects_garbage() {
    let err = parse_filter(r#"userName == "alice""#).unwrap_err();
    assert_eq!(err.scim_type, Some(ScimType::InvalidFilter));
}

#[test]
fn filter_parser_handles_or() {
    let e = parse_filter(r#"userName eq "a" or userName eq "b""#).expect("parse");
    assert!(matches!(e, FilterExpr::Or(_, _)));
}

#[test]
fn scim_error_envelope_serializes_per_rfc_7644() {
    let e = ScimError::new(409, Some(ScimType::Uniqueness), "duplicate userName");
    let s = serde_json::to_string(&e).expect("ser");
    assert!(s.contains("\"status\":409"));
    assert!(s.contains("\"scimType\":\"uniqueness\""));
    assert!(s.contains("urn:ietf:params:scim:api:messages:2.0:Error"));
}

#[tokio::test]
async fn group_create_requires_displayname() {
    let s = srv();
    let err = s
        .create_group(
            &tenant(),
            NewGroup {
                display_name: String::new(),
                members: vec![],
            },
            1_700_000_000,
        )
        .await
        .unwrap_err();
    assert_eq!(err.status, 400);
}

#[tokio::test]
async fn patch_remove_email_by_value() {
    let s = srv();
    let u = s
        .create_user(&tenant(), new_user("alice"), 1_700_000_000)
        .await
        .expect("c");
    let p = s
        .patch_user(
            &tenant(),
            &u.id,
            &PatchOp {
                schemas: vec![PatchOp::SCHEMA.to_owned()],
                operations: vec![PatchOperation {
                    op: PatchOpKind::Remove,
                    path: Some(r#"emails[value eq "alice@example.com"]"#.into()),
                    value: None,
                }],
            },
            1_700_000_100,
        )
        .await
        .expect("patch");
    assert!(p.emails.is_empty());
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

#[tokio::test]
async fn group_membership_query_does_not_break_users() {
    let s = srv();
    let g = s
        .create_group(
            &tenant(),
            NewGroup {
                display_name: "team".into(),
                members: vec![GroupMembership {
                    value: ScimId("u1".into()),
                    display: None,
                    r#ref: None,
                }],
            },
            1_700_000_000,
        )
        .await
        .expect("g");
    let _: Group = s.get_group(&tenant(), &g.id).await.expect("get");
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
