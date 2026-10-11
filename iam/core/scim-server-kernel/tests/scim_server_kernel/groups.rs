use super::*;

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
