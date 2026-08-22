use iam_tenant_rbac_postgres_rls_runtime_evidence::{
    PostgresRlsRuntimeProbeKind, TenantRbacPostgresRlsRuntimeEvidenceError,
    postgres_rls_runtime_probe_doc_urls, tenant_rbac_postgres_rls_runtime_evidence_plan,
    validate_tenant_rbac_postgres_rls_runtime_evidence_plan,
};

#[test]
fn postgres_rls_runtime_evidence_plan_validates_controls_and_nonclaims() {
    let plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    validate_tenant_rbac_postgres_rls_runtime_evidence_plan(&plan)
        .expect("runtime evidence plan validates");

    assert_eq!(
        plan.plan_name,
        "tenant-rbac-postgres-rls-runtime-evidence-plan"
    );
    assert_eq!(plan.service_name, "tenant-rbac");
    assert_eq!(plan.schema_name, "tenant_rbac");
    assert_eq!(plan.runtime_role, "tenant_rbac_runtime");
    assert_eq!(plan.tenant_context_setting, "app.tenant_id");
    assert_eq!(plan.storage_plan_table_count, 5);
    assert_eq!(plan.probes.len(), 11);
    assert!(plan.official_docs_required);
    assert!(plan.tls_verify_full_required);
    assert!(plan.migration_digest_required);
    assert!(plan.migration_transaction_required);
    assert!(plan.role_matrix_required);
    assert!(plan.rls_probe_matrix_required);
    assert!(plan.tenant_cross_read_denial_required);
    assert!(plan.tenant_cross_write_denial_required);
    assert!(plan.delete_forbidden_probe_required);
    assert!(plan.bypassrls_absence_required);
    assert!(plan.backup_restore_rehearsal_required);
    assert!(plan.pitr_rehearsal_required);
    assert!(!plan.runtime_database_attached);
    assert!(!plan.postgres_connection_attached);
    assert!(!plan.migration_applied_attached);
    assert!(!plan.rls_runtime_verified_attached);
    assert!(!plan.durable_storage_runtime_attached);
    assert!(!plan.cloud_database_attached);
    assert!(!plan.production_backup_restore_attached);
    assert!(!plan.runtime_audit_chain_emission_attached);
}

#[test]
fn postgres_rls_runtime_evidence_plan_covers_tables_and_required_probe_kinds() {
    let plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    let tables = plan
        .probes
        .iter()
        .map(|probe| probe.table_name)
        .collect::<std::collections::BTreeSet<_>>();

    for table in [
        "tenant_rbac_policy_admissions",
        "tenant_rbac_group_close_rollups",
        "tenant_rbac_cross_service_workflow_plans",
        "tenant_rbac_incident_rollback_plans",
        "tenant_rbac_ops_commands",
    ] {
        assert!(tables.contains(table), "missing table {table}");
    }
    for probe_kind in [
        PostgresRlsRuntimeProbeKind::TlsConnectionVerifyFull,
        PostgresRlsRuntimeProbeKind::MigrationDigestMatches,
        PostgresRlsRuntimeProbeKind::OwnerForceRls,
        PostgresRlsRuntimeProbeKind::BypassRlsRoleAbsent,
        PostgresRlsRuntimeProbeKind::SelectTenantIsolation,
        PostgresRlsRuntimeProbeKind::InsertWithCheckTenantIsolation,
        PostgresRlsRuntimeProbeKind::UpdateWithCheckTenantIsolation,
        PostgresRlsRuntimeProbeKind::DeleteForbidden,
        PostgresRlsRuntimeProbeKind::IdempotencyConflict,
        PostgresRlsRuntimeProbeKind::BackupRestoreRehearsal,
        PostgresRlsRuntimeProbeKind::PitrRehearsal,
    ] {
        assert!(
            plan.probes
                .iter()
                .any(|probe| probe.probe_kind == probe_kind),
            "missing {probe_kind:?}"
        );
    }
}

#[test]
fn postgres_rls_runtime_evidence_plan_preserves_official_docs_and_ref_boundaries() {
    let plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    let docs = postgres_rls_runtime_probe_doc_urls(&plan);

    assert!(docs.contains(&"https://www.postgresql.org/docs/current/ddl-rowsecurity.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/sql-createpolicy.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/libpq-ssl.html"));
    assert!(docs.contains(&"https://www.postgresql.org/docs/current/continuous-archiving.html"));
    assert!(plan.probes.iter().all(|probe| {
        probe
            .source_migration_ref
            .starts_with("crates/tenant-rbac-postgres-rls-storage/")
    }));
    assert!(plan.probes.iter().all(|probe| {
        probe
            .runtime_check_ref
            .starts_with("runtime-check/postgres-rls/")
    }));
    assert!(plan.probes.iter().all(|probe| {
        probe
            .expected_evidence_ref
            .starts_with("evidence/postgres-rls-runtime/")
    }));
    assert!(
        plan.probes
            .iter()
            .all(|probe| probe.requires_non_owner_runtime_role)
    );
    assert!(
        plan.probes
            .iter()
            .all(|probe| probe.destructive_operation_forbidden)
    );
    assert!(plan.probes.iter().any(|probe| {
        probe.probe_kind == PostgresRlsRuntimeProbeKind::TlsConnectionVerifyFull
            && probe.requires_tls_verify_full
    }));
    assert!(plan.probes.iter().all(|probe| {
        probe.requires_tls_verify_full
            == (probe.probe_kind == PostgresRlsRuntimeProbeKind::TlsConnectionVerifyFull)
    }));

    let tenant_context_flags = plan
        .probes
        .iter()
        .map(|probe| {
            (
                probe.probe_id,
                (
                    probe.requires_tenant_a_context,
                    probe.requires_tenant_b_context,
                ),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        tenant_context_flags["owner-force-rls-group-close-rollups"],
        (true, true)
    );
    assert_eq!(
        tenant_context_flags["select-isolation-workflow-plans"],
        (true, true)
    );
    assert_eq!(
        tenant_context_flags["insert-with-check-workflow-plans"],
        (true, true)
    );
    assert_eq!(
        tenant_context_flags["update-with-check-incident-rollbacks"],
        (true, true)
    );
    assert_eq!(
        tenant_context_flags["delete-forbidden-incident-rollbacks"],
        (true, false)
    );
    assert_eq!(
        tenant_context_flags["idempotency-conflict-ops-commands"],
        (true, false)
    );
}

#[test]
fn postgres_rls_runtime_evidence_plan_rejects_missing_probe_duplicate_and_doc_drift() {
    let mut plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    plan.probes.truncate(2);
    assert_eq!(
        validate_tenant_rbac_postgres_rls_runtime_evidence_plan(&plan),
        Err(TenantRbacPostgresRlsRuntimeEvidenceError::MissingProbes)
    );

    let mut plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    plan.probes[1].probe_id = plan.probes[0].probe_id;
    assert_eq!(
        validate_tenant_rbac_postgres_rls_runtime_evidence_plan(&plan),
        Err(TenantRbacPostgresRlsRuntimeEvidenceError::DuplicateProbe(
            "tls-verify-full-tenant-rbac".to_owned()
        ))
    );

    let mut plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    plan.probes[0].official_doc_url = "https://example.com/postgres";
    assert_eq!(
        validate_tenant_rbac_postgres_rls_runtime_evidence_plan(&plan),
        Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidOfficialDocUrl)
    );

    let mut plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    plan.probes[0].table_name = "tenant_rbac_missing_table";
    assert_eq!(
        validate_tenant_rbac_postgres_rls_runtime_evidence_plan(&plan),
        Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidTableName)
    );
}

#[test]
fn postgres_rls_runtime_evidence_plan_rejects_unsafe_refs_missing_controls_and_runtime_overclaims()
{
    let mut plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    plan.probes[0].expected_evidence_ref = "evidence/postgres-rls-runtime/secret-api-key";
    assert_eq!(
        validate_tenant_rbac_postgres_rls_runtime_evidence_plan(&plan),
        Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidExpectedEvidenceRef)
    );

    let mut plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    plan.tls_verify_full_required = false;
    assert_eq!(
        validate_tenant_rbac_postgres_rls_runtime_evidence_plan(&plan),
        Err(
            TenantRbacPostgresRlsRuntimeEvidenceError::MissingRequiredControl(
                "tls_verify_full_required"
            )
        )
    );

    let mut plan = tenant_rbac_postgres_rls_runtime_evidence_plan().expect("plan builds");
    plan.rls_runtime_verified_attached = true;
    assert_eq!(
        validate_tenant_rbac_postgres_rls_runtime_evidence_plan(&plan),
        Err(TenantRbacPostgresRlsRuntimeEvidenceError::RuntimeAttachmentOverclaim)
    );
}
