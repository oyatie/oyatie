//! Tenant RBAC Postgres/RLS runtime evidence contract foundation.
//!
//! This control-plane crate defines the pre-cloud runtime evidence contract that
//! later durable Postgres adapters, migration runners, RLS verifiers, backup
//! restorers, and Oyatie cloud database integrations must satisfy before durable
//! Tenant RBAC storage is claimed. It binds the existing review-only
//! Postgres/RLS schema plan to runtime probe evidence for TLS verification,
//! migration digest application, tenant isolation, restrictive policies,
//! append-only/idempotency behavior, role hardening, backup restore, and PITR
//! rehearsal. It deliberately does not open a database connection, apply
//! migrations, verify live RLS behavior, attach a cloud database, persist
//! records, emit runtime audit-chain events, or claim durable storage readiness.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use iam_tenant_rbac_postgres_rls_storage::{
    TenantRbacPostgresRecordKind, TenantRbacPostgresRlsStorageError,
    tenant_rbac_postgres_rls_storage_plan, validate_tenant_rbac_postgres_rls_storage_plan,
};

const SCHEMA_VERSION: u32 = 1;
const MIN_PROBE_COUNT: usize = 11;
const PLAN_NAME: &str = "tenant-rbac-postgres-rls-runtime-evidence-plan";
const SERVICE_NAME: &str = "tenant-rbac";
const SCHEMA_NAME: &str = "tenant_rbac";
const RUNTIME_ROLE: &str = "tenant_rbac_runtime";
const TENANT_CONTEXT_SETTING: &str = "app.tenant_id";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PostgresRlsRuntimeProbeKind {
    TlsConnectionVerifyFull,
    MigrationDigestMatches,
    OwnerForceRls,
    BypassRlsRoleAbsent,
    SelectTenantIsolation,
    InsertWithCheckTenantIsolation,
    UpdateWithCheckTenantIsolation,
    DeleteForbidden,
    IdempotencyConflict,
    BackupRestoreRehearsal,
    PitrRehearsal,
}

impl PostgresRlsRuntimeProbeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TlsConnectionVerifyFull => "tls_connection_verify_full",
            Self::MigrationDigestMatches => "migration_digest_matches",
            Self::OwnerForceRls => "owner_force_rls",
            Self::BypassRlsRoleAbsent => "bypassrls_role_absent",
            Self::SelectTenantIsolation => "select_tenant_isolation",
            Self::InsertWithCheckTenantIsolation => "insert_with_check_tenant_isolation",
            Self::UpdateWithCheckTenantIsolation => "update_with_check_tenant_isolation",
            Self::DeleteForbidden => "delete_forbidden",
            Self::IdempotencyConflict => "idempotency_conflict",
            Self::BackupRestoreRehearsal => "backup_restore_rehearsal",
            Self::PitrRehearsal => "pitr_rehearsal",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresRlsRuntimeProbe {
    pub probe_id: &'static str,                    // data_class: PUBLIC
    pub table_name: &'static str,                  // data_class: PUBLIC
    pub record_kind: TenantRbacPostgresRecordKind, // data_class: PUBLIC
    pub probe_kind: PostgresRlsRuntimeProbeKind,   // data_class: PUBLIC
    pub official_doc_url: &'static str,            // data_class: PUBLIC
    pub source_migration_ref: &'static str,        // data_class: INTERNAL_ONLY
    pub runtime_check_ref: &'static str,           // data_class: INTERNAL_ONLY
    pub expected_evidence_ref: &'static str,       // data_class: INTERNAL_ONLY
    pub requires_tenant_a_context: bool,           // data_class: PUBLIC
    pub requires_tenant_b_context: bool,           // data_class: PUBLIC
    pub requires_non_owner_runtime_role: bool,     // data_class: PUBLIC
    pub requires_tls_verify_full: bool,            // data_class: PUBLIC
    pub destructive_operation_forbidden: bool,     // data_class: PUBLIC
    pub schema_version: u32,                       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRbacPostgresRlsRuntimeEvidencePlan {
    pub plan_name: &'static str,                     // data_class: PUBLIC
    pub service_name: &'static str,                  // data_class: PUBLIC
    pub schema_name: &'static str,                   // data_class: PUBLIC
    pub runtime_role: &'static str,                  // data_class: INTERNAL_ONLY
    pub tenant_context_setting: &'static str,        // data_class: INTERNAL_ONLY
    pub storage_plan_table_count: usize,             // data_class: PUBLIC
    pub probes: Vec<PostgresRlsRuntimeProbe>,        // data_class: PUBLIC
    pub official_docs_required: bool,                // data_class: PUBLIC
    pub tls_verify_full_required: bool,              // data_class: PUBLIC
    pub migration_digest_required: bool,             // data_class: PUBLIC
    pub migration_transaction_required: bool,        // data_class: PUBLIC
    pub role_matrix_required: bool,                  // data_class: PUBLIC
    pub rls_probe_matrix_required: bool,             // data_class: PUBLIC
    pub tenant_cross_read_denial_required: bool,     // data_class: PUBLIC
    pub tenant_cross_write_denial_required: bool,    // data_class: PUBLIC
    pub delete_forbidden_probe_required: bool,       // data_class: PUBLIC
    pub bypassrls_absence_required: bool,            // data_class: PUBLIC
    pub backup_restore_rehearsal_required: bool,     // data_class: PUBLIC
    pub pitr_rehearsal_required: bool,               // data_class: PUBLIC
    pub runtime_database_attached: bool,             // data_class: INTERNAL_ONLY
    pub postgres_connection_attached: bool,          // data_class: INTERNAL_ONLY
    pub migration_applied_attached: bool,            // data_class: INTERNAL_ONLY
    pub rls_runtime_verified_attached: bool,         // data_class: INTERNAL_ONLY
    pub durable_storage_runtime_attached: bool,      // data_class: INTERNAL_ONLY
    pub cloud_database_attached: bool,               // data_class: INTERNAL_ONLY
    pub production_backup_restore_attached: bool,    // data_class: INTERNAL_ONLY
    pub runtime_audit_chain_emission_attached: bool, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                         // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantRbacPostgresRlsRuntimeEvidenceError {
    StoragePlan(TenantRbacPostgresRlsStorageError),
    InvalidPlanName,
    InvalidServiceName,
    InvalidSchemaName,
    InvalidRuntimeRole,
    InvalidTenantContextSetting,
    InvalidTableCount,
    MissingProbes,
    DuplicateProbe(String),
    MissingStoragePlanTable(String),
    MissingProbeKind(PostgresRlsRuntimeProbeKind),
    InvalidProbeId,
    InvalidTableName,
    InvalidOfficialDocUrl,
    InvalidSourceMigrationRef,
    InvalidRuntimeCheckRef,
    InvalidExpectedEvidenceRef,
    MissingRequiredControl(&'static str),
    RuntimeAttachmentOverclaim,
}

pub fn tenant_rbac_postgres_rls_runtime_evidence_plan()
-> Result<TenantRbacPostgresRlsRuntimeEvidencePlan, TenantRbacPostgresRlsRuntimeEvidenceError> {
    let storage_plan = tenant_rbac_postgres_rls_storage_plan();
    validate_tenant_rbac_postgres_rls_storage_plan(&storage_plan)
        .map_err(TenantRbacPostgresRlsRuntimeEvidenceError::StoragePlan)?;

    Ok(TenantRbacPostgresRlsRuntimeEvidencePlan {
        plan_name: PLAN_NAME,
        service_name: SERVICE_NAME,
        schema_name: storage_plan.schema_name,
        runtime_role: storage_plan.runtime_role,
        tenant_context_setting: storage_plan.tenant_context_setting,
        storage_plan_table_count: storage_plan.tables.len(),
        probes: vec![
            probe(
                "tls-verify-full-tenant-rbac",
                "tenant_rbac_policy_admissions",
                TenantRbacPostgresRecordKind::PolicyAdmission,
                PostgresRlsRuntimeProbeKind::TlsConnectionVerifyFull,
                "https://www.postgresql.org/docs/current/libpq-ssl.html",
                "runtime-check/postgres-rls/connection/sslmode-verify-full",
            ),
            probe(
                "migration-digest-tenant-rbac-policy-admissions",
                "tenant_rbac_policy_admissions",
                TenantRbacPostgresRecordKind::PolicyAdmission,
                PostgresRlsRuntimeProbeKind::MigrationDigestMatches,
                "https://www.postgresql.org/docs/current/sql-altertable.html",
                "runtime-check/postgres-rls/migration/digest-match",
            ),
            probe(
                "owner-force-rls-group-close-rollups",
                "tenant_rbac_group_close_rollups",
                TenantRbacPostgresRecordKind::GroupCloseRollup,
                PostgresRlsRuntimeProbeKind::OwnerForceRls,
                "https://www.postgresql.org/docs/current/ddl-rowsecurity.html",
                "runtime-check/postgres-rls/group-close/force-rls",
            ),
            probe(
                "bypassrls-absent-runtime-role",
                "tenant_rbac_group_close_rollups",
                TenantRbacPostgresRecordKind::GroupCloseRollup,
                PostgresRlsRuntimeProbeKind::BypassRlsRoleAbsent,
                "https://www.postgresql.org/docs/current/ddl-rowsecurity.html",
                "runtime-check/postgres-rls/roles/bypassrls-absent",
            ),
            probe(
                "select-isolation-workflow-plans",
                "tenant_rbac_cross_service_workflow_plans",
                TenantRbacPostgresRecordKind::CrossServiceWorkflowPlan,
                PostgresRlsRuntimeProbeKind::SelectTenantIsolation,
                "https://www.postgresql.org/docs/current/sql-createpolicy.html",
                "runtime-check/postgres-rls/workflow/select-isolation",
            ),
            probe(
                "insert-with-check-workflow-plans",
                "tenant_rbac_cross_service_workflow_plans",
                TenantRbacPostgresRecordKind::CrossServiceWorkflowPlan,
                PostgresRlsRuntimeProbeKind::InsertWithCheckTenantIsolation,
                "https://www.postgresql.org/docs/current/sql-createpolicy.html",
                "runtime-check/postgres-rls/workflow/insert-with-check",
            ),
            probe(
                "update-with-check-incident-rollbacks",
                "tenant_rbac_incident_rollback_plans",
                TenantRbacPostgresRecordKind::IncidentRollbackPlan,
                PostgresRlsRuntimeProbeKind::UpdateWithCheckTenantIsolation,
                "https://www.postgresql.org/docs/current/sql-createpolicy.html",
                "runtime-check/postgres-rls/incident/update-with-check",
            ),
            probe(
                "delete-forbidden-incident-rollbacks",
                "tenant_rbac_incident_rollback_plans",
                TenantRbacPostgresRecordKind::IncidentRollbackPlan,
                PostgresRlsRuntimeProbeKind::DeleteForbidden,
                "https://www.postgresql.org/docs/current/sql-createpolicy.html",
                "runtime-check/postgres-rls/incident/delete-forbidden",
            ),
            probe(
                "idempotency-conflict-ops-commands",
                "tenant_rbac_ops_commands",
                TenantRbacPostgresRecordKind::OpsCommand,
                PostgresRlsRuntimeProbeKind::IdempotencyConflict,
                "https://www.postgresql.org/docs/current/ddl-constraints.html",
                "runtime-check/postgres-rls/ops/idempotency-conflict",
            ),
            probe(
                "backup-restore-rehearsal-tenant-rbac",
                "tenant_rbac_ops_commands",
                TenantRbacPostgresRecordKind::OpsCommand,
                PostgresRlsRuntimeProbeKind::BackupRestoreRehearsal,
                "https://www.postgresql.org/docs/current/backup.html",
                "runtime-check/postgres-rls/backup/restore-rehearsal",
            ),
            probe(
                "pitr-rehearsal-tenant-rbac",
                "tenant_rbac_policy_admissions",
                TenantRbacPostgresRecordKind::PolicyAdmission,
                PostgresRlsRuntimeProbeKind::PitrRehearsal,
                "https://www.postgresql.org/docs/current/continuous-archiving.html",
                "runtime-check/postgres-rls/backup/pitr-rehearsal",
            ),
        ],
        official_docs_required: true,
        tls_verify_full_required: true,
        migration_digest_required: true,
        migration_transaction_required: true,
        role_matrix_required: true,
        rls_probe_matrix_required: true,
        tenant_cross_read_denial_required: true,
        tenant_cross_write_denial_required: true,
        delete_forbidden_probe_required: true,
        bypassrls_absence_required: true,
        backup_restore_rehearsal_required: true,
        pitr_rehearsal_required: true,
        runtime_database_attached: false,
        postgres_connection_attached: false,
        migration_applied_attached: false,
        rls_runtime_verified_attached: false,
        durable_storage_runtime_attached: false,
        cloud_database_attached: false,
        production_backup_restore_attached: false,
        runtime_audit_chain_emission_attached: false,
        schema_version: SCHEMA_VERSION,
    })
}

pub fn validate_tenant_rbac_postgres_rls_runtime_evidence_plan(
    plan: &TenantRbacPostgresRlsRuntimeEvidencePlan,
) -> Result<(), TenantRbacPostgresRlsRuntimeEvidenceError> {
    let storage_plan = tenant_rbac_postgres_rls_storage_plan();
    validate_tenant_rbac_postgres_rls_storage_plan(&storage_plan)
        .map_err(TenantRbacPostgresRlsRuntimeEvidenceError::StoragePlan)?;

    validate_slug(
        plan.plan_name,
        TenantRbacPostgresRlsRuntimeEvidenceError::InvalidPlanName,
    )?;
    if plan.service_name != SERVICE_NAME {
        return Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidServiceName);
    }
    if plan.schema_name != SCHEMA_NAME {
        return Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidSchemaName);
    }
    if plan.runtime_role != RUNTIME_ROLE {
        return Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidRuntimeRole);
    }
    if plan.tenant_context_setting != TENANT_CONTEXT_SETTING {
        return Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidTenantContextSetting);
    }
    if plan.storage_plan_table_count != storage_plan.tables.len() {
        return Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidTableCount);
    }
    if plan.probes.len() < MIN_PROBE_COUNT {
        return Err(TenantRbacPostgresRlsRuntimeEvidenceError::MissingProbes);
    }

    let storage_tables = storage_plan
        .tables
        .iter()
        .map(|table| table.table_name)
        .collect::<BTreeSet<_>>();
    let mut seen_probes = BTreeSet::new();
    let mut seen_tables = BTreeSet::new();
    let mut seen_probe_kinds = BTreeSet::new();
    for probe in &plan.probes {
        validate_probe(probe, &storage_tables)?;
        if !seen_probes.insert(probe.probe_id) {
            return Err(TenantRbacPostgresRlsRuntimeEvidenceError::DuplicateProbe(
                probe.probe_id.to_owned(),
            ));
        }
        seen_tables.insert(probe.table_name);
        seen_probe_kinds.insert(probe.probe_kind);
    }

    for table_name in storage_tables {
        if !seen_tables.contains(table_name) {
            return Err(
                TenantRbacPostgresRlsRuntimeEvidenceError::MissingStoragePlanTable(
                    table_name.to_owned(),
                ),
            );
        }
    }
    for probe_kind in required_probe_kinds() {
        if !seen_probe_kinds.contains(&probe_kind) {
            return Err(TenantRbacPostgresRlsRuntimeEvidenceError::MissingProbeKind(
                probe_kind,
            ));
        }
    }

    require_control(plan.official_docs_required, "official_docs_required")?;
    require_control(plan.tls_verify_full_required, "tls_verify_full_required")?;
    require_control(plan.migration_digest_required, "migration_digest_required")?;
    require_control(
        plan.migration_transaction_required,
        "migration_transaction_required",
    )?;
    require_control(plan.role_matrix_required, "role_matrix_required")?;
    require_control(plan.rls_probe_matrix_required, "rls_probe_matrix_required")?;
    require_control(
        plan.tenant_cross_read_denial_required,
        "tenant_cross_read_denial_required",
    )?;
    require_control(
        plan.tenant_cross_write_denial_required,
        "tenant_cross_write_denial_required",
    )?;
    require_control(
        plan.delete_forbidden_probe_required,
        "delete_forbidden_probe_required",
    )?;
    require_control(
        plan.bypassrls_absence_required,
        "bypassrls_absence_required",
    )?;
    require_control(
        plan.backup_restore_rehearsal_required,
        "backup_restore_rehearsal_required",
    )?;
    require_control(plan.pitr_rehearsal_required, "pitr_rehearsal_required")?;

    if plan.runtime_database_attached
        || plan.postgres_connection_attached
        || plan.migration_applied_attached
        || plan.rls_runtime_verified_attached
        || plan.durable_storage_runtime_attached
        || plan.cloud_database_attached
        || plan.production_backup_restore_attached
        || plan.runtime_audit_chain_emission_attached
    {
        return Err(TenantRbacPostgresRlsRuntimeEvidenceError::RuntimeAttachmentOverclaim);
    }
    Ok(())
}

pub fn postgres_rls_runtime_probe_doc_urls(
    plan: &TenantRbacPostgresRlsRuntimeEvidencePlan,
) -> Vec<&'static str> {
    plan.probes
        .iter()
        .map(|probe| probe.official_doc_url)
        .collect()
}

fn probe(
    probe_id: &'static str,
    table_name: &'static str,
    record_kind: TenantRbacPostgresRecordKind,
    probe_kind: PostgresRlsRuntimeProbeKind,
    official_doc_url: &'static str,
    runtime_check_ref: &'static str,
) -> PostgresRlsRuntimeProbe {
    let requires_tenant_a_context = matches!(
        probe_kind,
        PostgresRlsRuntimeProbeKind::OwnerForceRls
            | PostgresRlsRuntimeProbeKind::SelectTenantIsolation
            | PostgresRlsRuntimeProbeKind::InsertWithCheckTenantIsolation
            | PostgresRlsRuntimeProbeKind::UpdateWithCheckTenantIsolation
            | PostgresRlsRuntimeProbeKind::DeleteForbidden
            | PostgresRlsRuntimeProbeKind::IdempotencyConflict
    );
    let requires_tenant_b_context = matches!(
        probe_kind,
        PostgresRlsRuntimeProbeKind::OwnerForceRls
            | PostgresRlsRuntimeProbeKind::SelectTenantIsolation
            | PostgresRlsRuntimeProbeKind::InsertWithCheckTenantIsolation
            | PostgresRlsRuntimeProbeKind::UpdateWithCheckTenantIsolation
    );

    PostgresRlsRuntimeProbe {
        probe_id,
        table_name,
        record_kind,
        probe_kind,
        official_doc_url,
        source_migration_ref: "crates/tenant-rbac-postgres-rls-storage/src/lib.rs::render_tenant_rbac_postgres_rls_migration",
        runtime_check_ref,
        expected_evidence_ref: "evidence/postgres-rls-runtime/tenant-rbac/probe-results.jsonl",
        requires_tenant_a_context,
        requires_tenant_b_context,
        requires_non_owner_runtime_role: true,
        requires_tls_verify_full: probe_kind
            == PostgresRlsRuntimeProbeKind::TlsConnectionVerifyFull,
        destructive_operation_forbidden: true,
        schema_version: SCHEMA_VERSION,
    }
}

fn validate_probe(
    probe: &PostgresRlsRuntimeProbe,
    storage_tables: &BTreeSet<&'static str>,
) -> Result<(), TenantRbacPostgresRlsRuntimeEvidenceError> {
    validate_slug(
        probe.probe_id,
        TenantRbacPostgresRlsRuntimeEvidenceError::InvalidProbeId,
    )?;
    if !storage_tables.contains(probe.table_name) {
        return Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidTableName);
    }
    validate_doc_url(probe.official_doc_url)?;
    validate_prefixed_ref(
        probe.source_migration_ref,
        "crates/tenant-rbac-postgres-rls-storage/",
        TenantRbacPostgresRlsRuntimeEvidenceError::InvalidSourceMigrationRef,
    )?;
    validate_prefixed_ref(
        probe.runtime_check_ref,
        "runtime-check/postgres-rls/",
        TenantRbacPostgresRlsRuntimeEvidenceError::InvalidRuntimeCheckRef,
    )?;
    validate_prefixed_ref(
        probe.expected_evidence_ref,
        "evidence/postgres-rls-runtime/",
        TenantRbacPostgresRlsRuntimeEvidenceError::InvalidExpectedEvidenceRef,
    )?;
    require_control(
        probe.requires_non_owner_runtime_role,
        "requires_non_owner_runtime_role",
    )?;
    require_control(
        probe.destructive_operation_forbidden,
        "destructive_operation_forbidden",
    )?;
    if probe.probe_kind == PostgresRlsRuntimeProbeKind::TlsConnectionVerifyFull {
        require_control(
            probe.requires_tls_verify_full,
            "probe_requires_tls_verify_full",
        )?;
    }
    if matches!(
        probe.probe_kind,
        PostgresRlsRuntimeProbeKind::SelectTenantIsolation
            | PostgresRlsRuntimeProbeKind::InsertWithCheckTenantIsolation
            | PostgresRlsRuntimeProbeKind::UpdateWithCheckTenantIsolation
    ) {
        require_control(
            probe.requires_tenant_a_context,
            "probe_requires_tenant_a_context",
        )?;
        require_control(
            probe.requires_tenant_b_context,
            "probe_requires_tenant_b_context",
        )?;
    }
    Ok(())
}

fn required_probe_kinds() -> [PostgresRlsRuntimeProbeKind; 11] {
    [
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
    ]
}

fn validate_doc_url(url: &str) -> Result<(), TenantRbacPostgresRlsRuntimeEvidenceError> {
    if is_unsafe_ref(url)
        || ![
            "https://www.postgresql.org/docs/current/ddl-rowsecurity.html",
            "https://www.postgresql.org/docs/current/sql-createpolicy.html",
            "https://www.postgresql.org/docs/current/sql-altertable.html",
            "https://www.postgresql.org/docs/current/ddl-constraints.html",
            "https://www.postgresql.org/docs/current/backup.html",
            "https://www.postgresql.org/docs/current/continuous-archiving.html",
            "https://www.postgresql.org/docs/current/libpq-ssl.html",
        ]
        .contains(&url)
    {
        return Err(TenantRbacPostgresRlsRuntimeEvidenceError::InvalidOfficialDocUrl);
    }
    Ok(())
}

fn validate_slug(
    value: &str,
    error: TenantRbacPostgresRlsRuntimeEvidenceError,
) -> Result<(), TenantRbacPostgresRlsRuntimeEvidenceError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.contains("--")
        || value
            .chars()
            .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'))
    {
        return Err(error);
    }
    Ok(())
}

fn validate_prefixed_ref(
    value: &str,
    prefix: &str,
    error: TenantRbacPostgresRlsRuntimeEvidenceError,
) -> Result<(), TenantRbacPostgresRlsRuntimeEvidenceError> {
    if value.len() <= prefix.len() || !value.starts_with(prefix) || is_unsafe_ref(value) {
        return Err(error);
    }
    Ok(())
}

fn require_control(
    value: bool,
    control: &'static str,
) -> Result<(), TenantRbacPostgresRlsRuntimeEvidenceError> {
    if value {
        Ok(())
    } else {
        Err(TenantRbacPostgresRlsRuntimeEvidenceError::MissingRequiredControl(control))
    }
}

fn is_unsafe_ref(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.trim() != value
        || value.contains("..")
        || value.chars().any(char::is_whitespace)
        || lower.contains("password")
        || lower.contains("secret")
        || lower.contains("credential=")
        || lower.contains("private_key")
        || lower.contains("api_key")
        || lower.contains("bearer")
}
