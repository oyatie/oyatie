use data_cloud_domain::{
    CloudDataError, DataBackupPolicy, DataTenantCellGuardrail, DataTenantCellGuardrailCreate,
    EngineShape, ManagedDataEngine, ManagedDataState, PostgresExtension, PostgresShape,
    ReplicationMode, SchemaMigrationPolicy,
};
use data_boundary_kernel::DataClass;
use network_residency::{
    PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
    ResidencyClass,
};

const TENANT: &str = "ten_acme";
const REGION: &str = "region-alpha1";
const CELL: &str = "cell-region-alpha1-a-primary";
const KMS_KEY: &str = "kms/region-alpha1/ten_acme/db-key";

fn residency_class() -> ResidencyClass {
    ResidencyClass::PerPack(Box::new(
        PerPackResidency::new(PerPackResidencyCreate {
            allowed_primary_regions: vec![REGION.to_string()],
            allowed_replica_regions: vec!["region-beta1".to_string()],
            forbidden_regions: vec!["region-gamma1".to_string()],
            regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                regulator_refs: vec!["regulator/global-data".to_string()],
                evidence_ref: "evidence/residency/global-data".to_string(),
            })
            .expect("regulator overlay fixture is valid"),
        })
        .expect("per-pack residency fixture is valid"),
    ))
}

fn migration_policy() -> SchemaMigrationPolicy {
    SchemaMigrationPolicy {
        forward_ref: "migration/cloud_data/forward".to_string(),
        backward_ref: "migration/cloud_data/backward".to_string(),
        synthetic_corpus_ref: "corpus/cloud_data/synthetic".to_string(),
        audit_chained: true,
        backward_compatible: true,
    }
}

fn backup_policy() -> DataBackupPolicy {
    DataBackupPolicy {
        pitr_days: 14,
        weekly_tenant_dump: true,
        retention_days: 90,
        kms_key_id: KMS_KEY.to_string(),
        object_store_ref: "object/region-alpha1/cloud-data-backup".to_string(),
        quarterly_dr_drill: true,
    }
}

fn postgres_shape() -> EngineShape {
    EngineShape::Postgres(PostgresShape {
        major_version: 16,
        extensions: vec![PostgresExtension::Citus],
        per_tenant_shards: 32,
        pgbouncer_per_tenant_pool: true,
    })
}

fn guardrail_create(engine: ManagedDataEngine) -> DataTenantCellGuardrailCreate {
    DataTenantCellGuardrailCreate {
        service_id: format!("data/{}/{REGION}/{TENANT}/primary", engine.token()),
        tenant_id: TENANT.to_string(),
        region: REGION.to_string(),
        primary_cell_id: CELL.to_string(),
        engine,
        table_refs: vec![
            "table/cloud_data/accounts".to_string(),
            "table/cloud_data/events".to_string(),
        ],
        tenant_partition_column: "tenant_id".to_string(),
        cell_partition_column: "cell_id".to_string(),
        citus_distribution_column: (engine == ManagedDataEngine::Citus)
            .then(|| "tenant_id".to_string()),
        citus_colocated_table_ref: (engine == ManagedDataEngine::Citus)
            .then(|| "table/cloud_data/accounts".to_string()),
        row_level_security_enabled: true,
        force_row_level_security: true,
        rls_policy_ref: "policy/cloud_data/tenant-cell".to_string(),
        migration: migration_policy(),
        backup: backup_policy(),
        restore_drill_evidence_ref: "restore/cloud_data/quarterly-drill".to_string(),
        evidence_refs: vec![
            "evidence/cloud_data/rls-policy-review".to_string(),
            "evidence/cloud_data/backup-restore-review".to_string(),
        ],
        residency: residency_class(),
        allowed_data_classes: vec![DataClass::InternalOnly, DataClass::PiiIdentifying],
        engine_shape: postgres_shape(),
        state: ManagedDataState::Provisioning,
    }
}

#[test]
fn postgres_guardrail_requires_tenant_cell_rls_migration_and_backup() {
    let guardrail = DataTenantCellGuardrail::new(guardrail_create(ManagedDataEngine::Postgres))
        .expect("postgres metadata guardrail should admit");

    assert_eq!(guardrail.engine.value, ManagedDataEngine::Postgres);
    assert_eq!(guardrail.tenant_partition_column.value, "tenant_id");
    assert_eq!(guardrail.cell_partition_column.value, "cell_id");
    assert!(guardrail.force_row_level_security.value);
    assert_eq!(guardrail.backup.value.pitr_days, 14);
}

#[test]
fn citus_guardrail_requires_tenant_distribution_and_colocation() {
    let guardrail = DataTenantCellGuardrail::new(guardrail_create(ManagedDataEngine::Citus))
        .expect("citus metadata guardrail should admit");

    assert_eq!(
        guardrail.citus_distribution_column.value.as_deref(),
        Some("tenant_id")
    );
    assert_eq!(
        guardrail.citus_colocated_table_ref.value.as_deref(),
        Some("table/cloud_data/accounts")
    );
}

#[test]
fn missing_force_rls_is_rejected() {
    let err = DataTenantCellGuardrail::new(DataTenantCellGuardrailCreate {
        force_row_level_security: false,
        ..guardrail_create(ManagedDataEngine::Postgres)
    })
    .expect_err("table owner bypass must be prevented with FORCE RLS");

    assert_eq!(err, CloudDataError::InvalidTenantIsolationPolicy);
}

#[test]
fn citus_distribution_must_match_tenant_column() {
    let err = DataTenantCellGuardrail::new(DataTenantCellGuardrailCreate {
        citus_distribution_column: Some("account_id".to_string()),
        ..guardrail_create(ManagedDataEngine::Citus)
    })
    .expect_err("Citus distribution column must be the tenant partition column");

    assert_eq!(err, CloudDataError::InvalidPartitioningPolicy);
}

#[test]
fn secret_like_evidence_references_are_rejected() {
    let err = DataTenantCellGuardrail::new(DataTenantCellGuardrailCreate {
        evidence_refs: vec!["evidence/cloud_data/raw-token".to_string()],
        ..guardrail_create(ManagedDataEngine::Postgres)
    })
    .expect_err("metadata evidence must not carry secret-like material");

    assert_eq!(err, CloudDataError::InvalidReference);
}

#[test]
fn evidence_references_are_required() {
    let err = DataTenantCellGuardrail::new(DataTenantCellGuardrailCreate {
        evidence_refs: vec![],
        ..guardrail_create(ManagedDataEngine::Postgres)
    })
    .expect_err("metadata guardrail needs at least one evidence ref");

    assert_eq!(err, CloudDataError::InvalidReference);
}
