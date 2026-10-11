use super::*;

fn assert_guc_first_then_scoped(statements: &[&str]) {
    assert!(!statements.is_empty(), "plan must not be empty");
    assert_eq!(
        statements[0], SET_LOCAL_TENANT_SQL,
        "the tenant GUC set_config MUST be the first statement"
    );
    for stmt in &statements[1..] {
        assert_ne!(
            *stmt, SET_LOCAL_TENANT_SQL,
            "only the leading statement sets the GUC"
        );
    }
}

#[test]
fn guc_set_statement_is_canonical_oyatie_tenant_id() {
    // The fix for the GUC divergence: this adapter uses the canonical GUC,
    // NOT app.tenant_id.
    assert_eq!(
        SET_LOCAL_TENANT_SQL,
        "SELECT set_config('oyatie.tenant_id', $1, true)"
    );
}

#[test]
fn user_plans_set_guc_before_statement() {
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, LIST_USERS_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, GET_USER_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, FIND_USER_BY_NAME_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, FIND_NATIVE_USER_MEMBERSHIPS_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, INSERT_USER_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, LOCK_USER_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, UPDATE_USER_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, DELETE_USER_SQL]);
}

#[test]
fn native_membership_decode_refuses_missing_or_inconsistent_records() {
    let payload = serde_json::json!({
        "schemas": [User::CORE_SCHEMA, User::OYATIE_SCHEMA],
        "id": "urn:oya:scim:1",
        "userName": "alice@example.com",
        "active": true,
        "urn:oyatie:scim:extension:2.0:User": {
            "nativeUserId": "usr_alice",
            "roles": ["foundry-operator"]
        },
        "meta": {
            "resourceType": "User", "created": "2026-01-01T00:00:00Z",
            "lastModified": "2026-01-01T00:00:00Z", "location": "https://example.com/Users/1",
            "version": "W/\"1\""
        }
    });
    let membership = membership_from_payload(payload.clone(), "ten_acme", "usr_alice", true)
        .expect("exact native binding");
    assert_eq!(membership.roles, vec!["foundry-operator"]);
    assert!(membership.active);
    assert_eq!(
        membership_from_payload(payload.clone(), "ten_acme", "usr_other", true),
        Err(MembershipStoreError::Corrupt)
    );
    assert_eq!(
        membership_from_payload(payload.clone(), "ten_acme", "usr_alice", false),
        Err(MembershipStoreError::Corrupt)
    );
    assert_eq!(
        membership_from_payload(serde_json::json!({}), "ten_acme", "usr_alice", true),
        Err(MembershipStoreError::Corrupt)
    );
    let mut malformed = payload;
    malformed[User::OYATIE_SCHEMA]["roles"] = serde_json::json!("operator");
    assert_eq!(
        membership_from_payload(malformed, "ten_acme", "usr_alice", true),
        Err(MembershipStoreError::Corrupt)
    );
}

#[test]
fn group_plans_set_guc_before_statement() {
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, LIST_GROUPS_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, GET_GROUP_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, UPSERT_GROUP_SQL]);
    assert_guc_first_then_scoped(&[SET_LOCAL_TENANT_SQL, DELETE_GROUP_SQL]);
}

#[test]
fn every_scoped_statement_filters_by_tenant_id() {
    for sql in [
        LIST_USERS_SQL,
        GET_USER_SQL,
        FIND_USER_BY_NAME_SQL,
        FIND_NATIVE_USER_MEMBERSHIPS_SQL,
        INSERT_USER_SQL,
        LOCK_USER_SQL,
        UPDATE_USER_SQL,
        DELETE_USER_SQL,
        LIST_GROUPS_SQL,
        GET_GROUP_SQL,
        UPSERT_GROUP_SQL,
        DELETE_GROUP_SQL,
    ] {
        assert!(
            sql.contains("tenant_id"),
            "scoped statement must reference tenant_id: {sql}"
        );
    }
}

#[test]
fn connect_rejects_empty_database_url() {
    assert_eq!(
        validate_database_url("   "),
        Err(PgScimConnectError::MissingDatabaseUrl)
    );
    assert_eq!(
        validate_database_url("postgres://u:p@localhost/db?sslmode=require"),
        Ok(())
    );
}

#[test]
fn validate_tenant_rejects_empty_or_blank() {
    assert!(matches!(
        validate_tenant(&TenantId(String::new())),
        Err(ScimStoreError::Corrupt { .. })
    ));
    assert!(matches!(
        validate_tenant(&TenantId("   ".to_owned())),
        Err(ScimStoreError::Corrupt { .. })
    ));
    assert_eq!(validate_tenant(&TenantId("acme".to_owned())), Ok(()));
}

// --- RlsEnforceabilityError -> PgScimConnectError mapping -----------------
// The DB-free role/forced predicate decisions now live in the shared kernel
// (shared-postgres-command-kernel); this adapter keeps only the THIN
// mapping that preserves its fail-closed connect-error contract.

#[test]
fn rls_error_mapping_preserves_fail_closed_contract() {
    assert_eq!(
        PgScimConnectError::from(RlsEnforceabilityError::Unenforceable {
            role: "pg_superuser".to_owned()
        }),
        PgScimConnectError::RlsUnenforceable {
            role: "pg_superuser".to_owned()
        }
    );
    assert_eq!(
        PgScimConnectError::from(RlsEnforceabilityError::RoleMismatch {
            role: "some_role".to_owned(),
            expected: RUNTIME_ROLE.to_owned(),
        }),
        PgScimConnectError::RlsRoleMismatch {
            role: "some_role".to_owned(),
            expected: RUNTIME_ROLE.to_owned(),
        }
    );
    // A SET ROLE switch and a probe failure both preserve today's `Sqlx`
    // surface (not the structured RLS variants).
    assert!(matches!(
        PgScimConnectError::from(RlsEnforceabilityError::RoleSwitchInEffect {
            session_role: "a".to_owned(),
            current_role: "b".to_owned(),
        }),
        PgScimConnectError::Sqlx(_)
    ));
    assert!(matches!(
        PgScimConnectError::from(RlsEnforceabilityError::ProbeFailed {
            detail: "role does not exist".to_owned()
        }),
        PgScimConnectError::Sqlx(_)
    ));
    // The #799 FORCE-RLS hardening: a not-forced or missing governed table
    // maps to the new RlsNotForcedOnTable variant.
    assert_eq!(
        PgScimConnectError::from(RlsEnforceabilityError::RlsNotForced {
            table: USERS_TABLE.to_owned(),
            row_security: true,
            force_row_security: false,
        }),
        PgScimConnectError::RlsNotForcedOnTable {
            table: USERS_TABLE.to_owned()
        }
    );
    assert_eq!(
        PgScimConnectError::from(RlsEnforceabilityError::GovernedTableMissing {
            table: GROUPS_TABLE.to_owned()
        }),
        PgScimConnectError::RlsNotForcedOnTable {
            table: GROUPS_TABLE.to_owned()
        }
    );
}

#[test]
fn runtime_role_matches_migration_policy_subject() {
    // The const MUST equal the `TO <role>` subject in
    // migrations/0001_identity_scim_store.sql — the guard checks membership
    // in exactly this role, so a drift would silently let a non-policy role
    // pass the guard while the policies never apply to it.
    assert_eq!(RUNTIME_ROLE, "identity_scim_runtime");
}

#[test]
fn governed_tables_match_migration_force_rls() {
    // The boot guard verifies ENABLE+FORCE RLS for exactly GOVERNED_TABLES.
    // If a dev FORCE-RLS'es a new table in the migration but forgets to add
    // it here (or vice versa), tenant isolation would be silently unverified
    // at boot. Asserting the SAME list the guard passes EXACTLY equals the
    // migration's FORCE'd-table set makes that drift impossible. DB-free —
    // pure string comparison, runs in the always-on unit lane.
    use shared_postgres_command_kernel::force_rls_tables;
    let mut from_migration =
        force_rls_tables(include_str!("../migrations/0001_identity_scim_store.sql"));
    from_migration.extend(force_rls_tables(PASSKEY_AUTHENTICATION_SQL));
    from_migration.sort();
    let mut governed: Vec<String> = GOVERNED_TABLES.iter().map(|t| (*t).to_owned()).collect();
    governed.sort();
    assert_eq!(
        governed, from_migration,
        "GOVERNED_TABLES must EXACTLY match the tables \
         migrations/0001_identity_scim_store.sql FORCE-RLS'es"
    );
}
