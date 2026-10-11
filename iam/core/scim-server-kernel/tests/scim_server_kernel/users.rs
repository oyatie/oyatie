use super::*;

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
