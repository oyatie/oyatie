use super::*;

#[tokio::test]
async fn atomic_store_preserves_legacy_writes_and_refuses_stale_protected_changes() {
    let server = srv();
    let tenant = tenant();
    let created = server
        .create_user(&tenant, new_user("legacy"), 1_700_000_000)
        .await
        .unwrap();
    let mut ordinary = created.clone();
    ordinary.display_name = Some("Updated".into());
    server
        .users
        .compare_and_put(&created, &ordinary, &tenant, false)
        .await
        .unwrap();

    let stale = ordinary.clone();
    let mut bound = ordinary.clone();
    bound.oyatie = Some(OyatieExtension {
        native_user_id: Some("usr_alice".into()),
        roles: Some(vec!["foundry-operator".into()]),
        ..Default::default()
    });
    bound.schemas.push(User::OYATIE_SCHEMA.into());
    assert_eq!(
        server
            .users
            .compare_and_put(&ordinary, &bound, &tenant, false)
            .await,
        Err(ScimStoreError::ProtectedUserChange)
    );
    server
        .users
        .compare_and_put(&ordinary, &bound, &tenant, true)
        .await
        .unwrap();
    let mut stale_ordinary = stale.clone();
    stale_ordinary.display_name = Some("Stale writer".into());
    assert_eq!(
        server
            .users
            .compare_and_put(&stale, &stale_ordinary, &tenant, false)
            .await,
        Err(ScimStoreError::Conflict),
        "a normal writer that read before the bind cannot overwrite it"
    );
    assert_eq!(
        server
            .users
            .compare_and_delete(&stale, &tenant, false)
            .await,
        Err(ScimStoreError::Conflict)
    );
    assert_eq!(
        server
            .users
            .compare_and_delete(&bound, &tenant, false)
            .await,
        Err(ScimStoreError::ProtectedUserChange)
    );

    let mut deactivated = bound.clone();
    deactivated.active = false;
    server
        .users
        .compare_and_put(&bound, &deactivated, &tenant, false)
        .await
        .unwrap();
    assert_eq!(
        server
            .users
            .compare_and_put(&deactivated, &bound, &tenant, false)
            .await,
        Err(ScimStoreError::ProtectedUserChange)
    );
    server
        .users
        .compare_and_put(&deactivated, &bound, &tenant, true)
        .await
        .unwrap();
    server
        .users
        .compare_and_delete(&bound, &tenant, true)
        .await
        .unwrap();

    let unbound = server
        .create_user(&tenant, new_user("legacy-two"), 1_700_000_001)
        .await
        .unwrap();
    let mut inactive = unbound.clone();
    inactive.active = false;
    server
        .users
        .compare_and_put(&unbound, &inactive, &tenant, false)
        .await
        .unwrap();
    server
        .users
        .compare_and_put(&inactive, &unbound, &tenant, false)
        .await
        .unwrap();
    server
        .users
        .compare_and_delete(&unbound, &tenant, false)
        .await
        .unwrap();
}

#[tokio::test]
async fn native_binding_validates_create_replace_and_patch_before_persisting() {
    let server = srv();
    let mut input = new_user("bound");
    input.oyatie = Some(OyatieExtension {
        native_user_id: Some("usr_alice".into()),
        roles: Some(vec!["foundry-operator".into()]),
        ..Default::default()
    });
    assert_eq!(
        server
            .create_user(&tenant(), input.clone(), 1_700_000_000)
            .await
            .unwrap_err()
            .status,
        403,
    );
    let created = server
        .create_user_with_authority(&tenant(), input.clone(), 1_700_000_000, true)
        .await
        .expect("valid binding");
    assert!(created.schemas.contains(&User::OYATIE_SCHEMA.to_owned()));
    assert_eq!(
        created.oyatie.as_ref().unwrap().native_user_id.as_deref(),
        Some("usr_alice")
    );

    for (native_user_id, roles) in [
        (Some("external-subject"), Some(vec!["foundry-operator"])),
        (Some("usr_alice"), Some(vec![""])),
        (
            Some("usr_alice"),
            Some(vec!["foundry-operator", "foundry-operator"]),
        ),
        (None, Some(vec!["foundry-operator"])),
    ] {
        let mut invalid = input.clone();
        invalid.oyatie.as_mut().unwrap().native_user_id = native_user_id.map(str::to_owned);
        invalid.oyatie.as_mut().unwrap().roles =
            roles.map(|roles| roles.into_iter().map(str::to_owned).collect());
        assert_eq!(
            server
                .replace_user(&tenant(), &created.id, invalid, 1_700_000_100)
                .await
                .unwrap_err()
                .scim_type,
            Some(ScimType::InvalidValue)
        );
    }

    let patch = |path: &str, value: serde_json::Value| PatchOp {
        schemas: vec![PatchOp::SCHEMA.to_owned()],
        operations: vec![PatchOperation {
            op: PatchOpKind::Replace,
            path: Some(path.to_owned()),
            value: Some(value),
        }],
    };
    let bad = server
        .patch_user(
            &tenant(),
            &created.id,
            &patch("urn:oyatie:scim:extension:2.0:User:roles", json!([""])),
            1_700_000_200,
        )
        .await
        .unwrap_err();
    assert_eq!(bad.scim_type, Some(ScimType::InvalidValue));
    let still_bound = server
        .get_user(&tenant(), &created.id)
        .await
        .expect("unchanged");
    assert_eq!(
        still_bound.oyatie.as_ref().unwrap().roles.as_ref().unwrap(),
        &["foundry-operator"]
    );

    let renamed = server
        .patch_user_with_authority(
            &tenant(),
            &created.id,
            &patch(
                "urn:oyatie:scim:extension:2.0:User:nativeUserId",
                json!("usr_bob"),
            ),
            1_700_000_300,
            true,
        )
        .await
        .expect("rebind");
    assert_eq!(
        renamed.oyatie.as_ref().unwrap().native_user_id.as_deref(),
        Some("usr_bob")
    );
}
