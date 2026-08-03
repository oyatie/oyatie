//! Cloud managed data-service kernel.
//!
//! This crate owns the provider-neutral control contract for managed Postgres,
//! Citus, pgvector, Valkey-compatible cache, Kafka, ClickHouse, and gated stable
//! expansion engines. The kernel validates topology and evidence; adapters own
//! engine I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use secrets_kms_domain::KmsKeyId;
use cell_region::{AzCode, CellId, RegionCode};
use compute_resource::{DatabaseEngine, QueueEngine, ResourceId, ResourceKind};
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use network_residency::{ResidencyClass, residency_class_allows_home_region_label};

const CLOUD_DATA_SCHEMA_VERSION: u32 = 1;
const BACKUP_EVIDENCE_SCHEMA_VERSION: u32 = 1;
const SERVICE_ID_PREFIX: &str = "data";
const BACKUP_EVIDENCE_ID_PREFIX: &str = "backup";
const TENANT_ID_PREFIX: &str = "ten_";
const REF_MIGRATION_PREFIX: &str = "migration";
const REF_CORPUS_PREFIX: &str = "corpus";
const REF_OBJECT_STORE_PREFIX: &str = "object";
const REF_RESIDENCY_POLICY_PREFIX: &str = "residency";
const REF_SCHEMA_REGISTRY_PREFIX: &str = "schema";
const REF_LICENSE_PREFIX: &str = "license";
const REF_ADR_GATE_PREFIX: &str = "adr-gate";
const REF_TABLE_PREFIX: &str = "table";
const REF_POLICY_PREFIX: &str = "policy";
const REF_RESTORE_PREFIX: &str = "restore";
const REF_EVIDENCE_PREFIX: &str = "evidence";
const TENANT_PARTITION_COLUMN: &str = "tenant_id";
const CELL_PARTITION_COLUMN: &str = "cell_id";
const MIN_REPLICATED_AZ_COUNT: usize = 3;
const POSTGRES_MAJOR_VERSION: u16 = 16;
const MIN_KAFKA_OUTBOX_POLL_MS: u16 = 100;
const MAX_KAFKA_OUTBOX_POLL_MS: u16 = 500;
const DEFAULT_PITR_DAYS: u16 = 14;
const MIN_BACKUP_RETENTION_DAYS: u16 = 30;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ManagedDataServiceId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BackupEvidenceId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ManagedDataEngine {
    Postgres,
    Citus,
    PgVector,
    Valkey,
    Kafka,
    ClickHouse,
    Cassandra,
    Iceberg,
    Milvus,
    Temporal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ManagedDataTier {
    Oltp,
    Cache,
    Stream,
    Olap,
    Lakehouse,
    Vector,
    Workflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ManagedDataState {
    Provisioning,
    Ready,
    Draining,
    Deleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PostgresExtension {
    Citus,
    PgVector,
    TimescaleDb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ValkeyDistribution {
    LegacyBsdPre74,
    Valkey,
    Garnet,
    ForbiddenSsplRsal74OrLater,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KafkaPartitionKey {
    TenantCell,
    TenantOnly,
    Random,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ForkTrackingCadence {
    PerReleaseWithQuarterlyReview,
    QuarterlyOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StableExpansionAdr {
    Adr0035,
    Adr0045,
    Adr0047,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GateDecision {
    Proposed,
    Accepted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ReplicationMode {
    ThreeAz,
    ThreeAzWithCrossRegionReadMirror,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PostgresShape {
    pub major_version: u16,                 // data_class: PUBLIC
    pub extensions: Vec<PostgresExtension>, // data_class: PUBLIC
    pub per_tenant_shards: u16,             // data_class: INTERNAL_ONLY
    pub pgbouncer_per_tenant_pool: bool,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkeyShape {
    pub distribution: ValkeyDistribution, // data_class: PUBLIC
    pub major_version: u16,               // data_class: PUBLIC
    pub replica_count: u16,               // data_class: INTERNAL_ONLY
    pub max_ttl_seconds: Option<u64>,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaShape {
    pub broker_count: u16,                // data_class: INTERNAL_ONLY
    pub partition_key: KafkaPartitionKey, // data_class: PUBLIC
    pub cloudevents_envelope: bool,       // data_class: PUBLIC
    pub protobuf_payloads: bool,          // data_class: PUBLIC
    pub schema_registry_ref: String,      // data_class: INTERNAL_ONLY
    pub outbox_poll_min_ms: u16,          // data_class: INTERNAL_ONLY
    pub outbox_poll_max_ms: u16,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClickHouseShape {
    pub apache2_fork_attestation_ref: String, // data_class: INTERNAL_ONLY
    pub fork_tracking: ForkTrackingCadence,   // data_class: PUBLIC
    pub per_tenant_database: bool,            // data_class: INTERNAL_ONLY
    pub materialized_view_refs: Vec<String>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StableExpansionGate {
    pub adr: StableExpansionAdr, // data_class: PUBLIC
    pub decision: GateDecision,  // data_class: PUBLIC
    pub evidence_ref: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EngineShape {
    Postgres(PostgresShape),
    Valkey(ValkeyShape),
    Kafka(KafkaShape),
    ClickHouse(ClickHouseShape),
    StableExpansion(StableExpansionGate),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataReplicationPolicy {
    pub mode: ReplicationMode,                // data_class: PUBLIC
    pub cross_region: Option<String>,         // data_class: INTERNAL_ONLY
    pub residency_policy_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataBackupPolicy {
    pub pitr_days: u16,           // data_class: INTERNAL_ONLY
    pub weekly_tenant_dump: bool, // data_class: INTERNAL_ONLY
    pub retention_days: u16,      // data_class: INTERNAL_ONLY
    pub kms_key_id: String,       // data_class: INTERNAL_ONLY
    pub object_store_ref: String, // data_class: INTERNAL_ONLY
    pub quarterly_dr_drill: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaMigrationPolicy {
    pub forward_ref: String,          // data_class: INTERNAL_ONLY
    pub backward_ref: String,         // data_class: INTERNAL_ONLY
    pub synthetic_corpus_ref: String, // data_class: INTERNAL_ONLY
    pub audit_chained: bool,          // data_class: INTERNAL_ONLY
    pub backward_compatible: bool,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedDataServiceCreate {
    pub id: String,                           // data_class: INTERNAL_ONLY
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: PUBLIC
    pub primary_cell_id: String,              // data_class: PUBLIC
    pub azs: Vec<String>,                     // data_class: PUBLIC
    pub resource_id: String,                  // data_class: INTERNAL_ONLY
    pub engine: ManagedDataEngine,            // data_class: PUBLIC
    pub state: ManagedDataState,              // data_class: PUBLIC
    pub residency: ResidencyClass,            // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Vec<DataClass>, // data_class: PUBLIC
    pub replication: DataReplicationPolicy,   // data_class: INTERNAL_ONLY
    pub backup: DataBackupPolicy,             // data_class: INTERNAL_ONLY
    pub migration: SchemaMigrationPolicy,     // data_class: INTERNAL_ONLY
    pub engine_shape: EngineShape,            // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManagedDataService {
    pub id: Classified<ManagedDataServiceId>,
    pub tenant_id: Classified<String>,
    pub region: Classified<RegionCode>,
    pub primary_cell_id: Classified<CellId>,
    pub azs: Classified<Vec<AzCode>>,
    pub resource_id: Classified<ResourceId>,
    pub engine: Classified<ManagedDataEngine>,
    pub tier: Classified<ManagedDataTier>,
    pub state: Classified<ManagedDataState>,
    pub residency: Classified<ResidencyClass>,
    pub allowed_data_classes: Classified<Vec<PrivacyDataClass>>,
    pub replication: Classified<TypedDataReplicationPolicy>,
    pub backup: Classified<TypedDataBackupPolicy>,
    pub migration: Classified<SchemaMigrationPolicy>,
    pub engine_shape: Classified<EngineShape>,
    pub created_at_epoch_seconds: Classified<u64>,
    pub updated_at_epoch_seconds: Classified<u64>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedDataReplicationPolicy {
    pub mode: ReplicationMode,                // data_class: PUBLIC
    pub cross_region: Option<RegionCode>,     // data_class: INTERNAL_ONLY
    pub residency_policy_ref: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedDataBackupPolicy {
    pub pitr_days: u16,           // data_class: INTERNAL_ONLY
    pub weekly_tenant_dump: bool, // data_class: INTERNAL_ONLY
    pub retention_days: u16,      // data_class: INTERNAL_ONLY
    pub kms_key_id: KmsKeyId,     // data_class: INTERNAL_ONLY
    pub object_store_ref: String, // data_class: INTERNAL_ONLY
    pub quarterly_dr_drill: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupEvidenceCreate {
    pub id: String,                       // data_class: INTERNAL_ONLY
    pub service_id: String,               // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub region: String,                   // data_class: PUBLIC
    pub kms_key_id: String,               // data_class: INTERNAL_ONLY
    pub covered_start_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub covered_end_epoch_seconds: u64,   // data_class: INTERNAL_ONLY
    pub completed_at_epoch_seconds: u64,  // data_class: INTERNAL_ONLY
    pub bytes_written: u64,               // data_class: INTERNAL_ONLY
    pub restore_drill_verified: bool,     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackupEvidence {
    pub id: Classified<BackupEvidenceId>,
    pub service_id: Classified<ManagedDataServiceId>,
    pub tenant_id: Classified<String>,
    pub region: Classified<RegionCode>,
    pub kms_key_id: Classified<KmsKeyId>,
    pub covered_start_epoch_seconds: Classified<u64>,
    pub covered_end_epoch_seconds: Classified<u64>,
    pub completed_at_epoch_seconds: Classified<u64>,
    pub bytes_written: Classified<u64>,
    pub restore_drill_verified: Classified<bool>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataTenantCellGuardrailCreate {
    pub service_id: String,                        // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                            // data_class: PUBLIC
    pub primary_cell_id: String,                   // data_class: PUBLIC
    pub engine: ManagedDataEngine,                 // data_class: PUBLIC
    pub table_refs: Vec<String>,                   // data_class: INTERNAL_ONLY
    pub tenant_partition_column: String,           // data_class: INTERNAL_ONLY
    pub cell_partition_column: String,             // data_class: INTERNAL_ONLY
    pub citus_distribution_column: Option<String>, // data_class: INTERNAL_ONLY
    pub citus_colocated_table_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub row_level_security_enabled: bool,          // data_class: INTERNAL_ONLY
    pub force_row_level_security: bool,            // data_class: INTERNAL_ONLY
    pub rls_policy_ref: String,                    // data_class: INTERNAL_ONLY
    pub migration: SchemaMigrationPolicy,          // data_class: INTERNAL_ONLY
    pub backup: DataBackupPolicy,                  // data_class: INTERNAL_ONLY
    pub restore_drill_evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,                // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass,                 // data_class: INTERNAL_ONLY
    pub allowed_data_classes: Vec<DataClass>,      // data_class: PUBLIC
    pub engine_shape: EngineShape,                 // data_class: INTERNAL_ONLY
    pub state: ManagedDataState,                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataTenantCellGuardrail {
    pub service_id: Classified<ManagedDataServiceId>,
    pub tenant_id: Classified<String>,
    pub region: Classified<RegionCode>,
    pub primary_cell_id: Classified<CellId>,
    pub engine: Classified<ManagedDataEngine>,
    pub table_refs: Classified<Vec<String>>,
    pub tenant_partition_column: Classified<String>,
    pub cell_partition_column: Classified<String>,
    pub citus_distribution_column: Classified<Option<String>>,
    pub citus_colocated_table_ref: Classified<Option<String>>,
    pub row_level_security_enabled: Classified<bool>,
    pub force_row_level_security: Classified<bool>,
    pub rls_policy_ref: Classified<String>,
    pub migration: Classified<SchemaMigrationPolicy>,
    pub backup: Classified<TypedDataBackupPolicy>,
    pub restore_drill_evidence_ref: Classified<String>,
    pub evidence_refs: Classified<Vec<String>>,
    pub residency: Classified<ResidencyClass>,
    pub allowed_data_classes: Classified<Vec<PrivacyDataClass>>,
    pub engine_shape: Classified<EngineShape>,
    pub state: Classified<ManagedDataState>,
    pub schema_version: Classified<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudDataError {
    InvalidServiceId,
    InvalidBackupEvidenceId,
    InvalidTenantId,
    InvalidRegion,
    InvalidAz,
    InvalidCellId,
    InvalidResourceId,
    InvalidKmsKeyId,
    InvalidInitialState,
    InvalidStateTransition,
    InvalidTimeOrder,
    InvalidDataClass,
    InvalidReplicationPolicy,
    InvalidBackupPolicy,
    InvalidMigrationPolicy,
    InvalidEngineShape,
    InvalidPartitioningPolicy,
    InvalidTenantIsolationPolicy,
    InvalidLicensePosture,
    InvalidExpansionGate,
    InvalidReference,
    ResidencyRegionMismatch,
    CrossRegionPolicyRequired,
    ServiceIdMismatch,
    ResourceKindMismatch,
    TenantMismatch,
    RegionMismatch,
    CellRegionMismatch,
    DuplicateService,
    DuplicateBackupEvidence,
    UnknownService,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudDataCatalog {
    services: BTreeMap<ManagedDataServiceId, ManagedDataService>,
    backups: BTreeMap<BackupEvidenceId, BackupEvidence>,
}

impl ManagedDataEngine {
    pub const fn token(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Citus => "citus",
            Self::PgVector => "pgvector",
            Self::Valkey => "valkey",
            Self::Kafka => "kafka",
            Self::ClickHouse => "clickhouse",
            Self::Cassandra => "cassandra",
            Self::Iceberg => "iceberg",
            Self::Milvus => "milvus",
            Self::Temporal => "temporal",
        }
    }

    pub const fn tier(self) -> ManagedDataTier {
        match self {
            Self::Postgres | Self::Citus | Self::PgVector | Self::Cassandra => {
                ManagedDataTier::Oltp
            }
            Self::Valkey => ManagedDataTier::Cache,
            Self::Kafka => ManagedDataTier::Stream,
            Self::ClickHouse => ManagedDataTier::Olap,
            Self::Iceberg => ManagedDataTier::Lakehouse,
            Self::Milvus => ManagedDataTier::Vector,
            Self::Temporal => ManagedDataTier::Workflow,
        }
    }

    pub const fn resource_kind(self) -> ResourceKind {
        match self {
            Self::Postgres => ResourceKind::Database(DatabaseEngine::Postgres),
            Self::Citus => ResourceKind::Database(DatabaseEngine::Citus),
            Self::PgVector => ResourceKind::Database(DatabaseEngine::PgVector),
            Self::Valkey => ResourceKind::Database(DatabaseEngine::Valkey),
            Self::Kafka => ResourceKind::QueueOrStream(QueueEngine::Kafka),
            Self::ClickHouse => ResourceKind::Database(DatabaseEngine::ClickHouse),
            Self::Cassandra => ResourceKind::Database(DatabaseEngine::Cassandra),
            Self::Iceberg => ResourceKind::Database(DatabaseEngine::Iceberg),
            Self::Milvus => ResourceKind::Database(DatabaseEngine::Milvus),
            Self::Temporal => ResourceKind::Database(DatabaseEngine::Temporal),
        }
    }

    pub const fn stable_gate_adr(self) -> Option<StableExpansionAdr> {
        match self {
            Self::Cassandra | Self::Iceberg => Some(StableExpansionAdr::Adr0045),
            Self::Milvus => Some(StableExpansionAdr::Adr0047),
            Self::Temporal => Some(StableExpansionAdr::Adr0035),
            Self::Postgres
            | Self::Citus
            | Self::PgVector
            | Self::Valkey
            | Self::Kafka
            | Self::ClickHouse => None,
        }
    }
}

impl ManagedDataServiceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDataError> {
        let value = value.into();
        parse_service_id(&value)?;
        Ok(Self { value })
    }

    pub fn engine(&self) -> Result<ManagedDataEngine, CloudDataError> {
        Ok(parse_service_id(&self.value)?.engine)
    }

    pub fn region(&self) -> Result<RegionCode, CloudDataError> {
        Ok(parse_service_id(&self.value)?.region)
    }

    pub fn tenant_id(&self) -> Result<String, CloudDataError> {
        Ok(parse_service_id(&self.value)?.tenant_id)
    }
}

impl BackupEvidenceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudDataError> {
        let value = value.into();
        validate_path(
            &value,
            BACKUP_EVIDENCE_ID_PREFIX,
            6,
            CloudDataError::InvalidBackupEvidenceId,
        )?;
        Ok(Self { value })
    }
}

impl ManagedDataService {
    pub fn new(input: ManagedDataServiceCreate) -> Result<Self, CloudDataError> {
        if input.state != ManagedDataState::Provisioning {
            return Err(CloudDataError::InvalidInitialState);
        }
        validate_positive_time(input.created_at_epoch_seconds)?;
        validate_tenant_id(&input.tenant_id)?;
        let id = ManagedDataServiceId::new(input.id)?;
        let id_parts = parse_service_id(&id.value)?;
        let region = RegionCode::new(input.region).map_err(|_| CloudDataError::InvalidRegion)?;
        let cell_id =
            CellId::new(input.primary_cell_id).map_err(|_| CloudDataError::InvalidCellId)?;
        let azs = typed_azs(&input.azs, &region)?;
        validate_cell_region(&cell_id, &region)?;
        validate_service_id_matches(&id_parts, input.engine, &region, &input.tenant_id)?;
        validate_residency(&input.residency, &region)?;
        let resource_id =
            ResourceId::new(input.resource_id).map_err(|_| CloudDataError::InvalidResourceId)?;
        validate_resource_matches(&resource_id, input.engine, &region, &input.tenant_id)?;
        let allowed_data_classes = typed_privacy_classes(&input.allowed_data_classes)?;
        let replication = typed_replication(input.replication, &region, &azs)?;
        let backup = typed_backup(input.backup, &region, &input.tenant_id)?;
        validate_migration_policy(&input.migration)?;
        validate_engine_shape(input.engine, &input.engine_shape)?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            primary_cell_id: public(cell_id),
            azs: public(azs),
            resource_id: internal(resource_id),
            engine: public(input.engine),
            tier: public(input.engine.tier()),
            state: public(input.state),
            residency: internal(input.residency),
            allowed_data_classes: public(allowed_data_classes),
            replication: internal(replication),
            backup: internal(backup),
            migration: internal(input.migration),
            engine_shape: internal(input.engine_shape),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(CLOUD_DATA_SCHEMA_VERSION),
        })
    }

    pub fn transition(
        &self,
        next_state: ManagedDataState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudDataError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !managed_data_transition_allowed(self.state.value, next_state) {
            return Err(CloudDataError::InvalidStateTransition);
        }
        let mut next = self.clone();
        next.state = public(next_state);
        next.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(next)
    }
}

impl BackupEvidence {
    pub fn new(
        service: &ManagedDataService,
        input: BackupEvidenceCreate,
    ) -> Result<Self, CloudDataError> {
        validate_time_order(
            input.covered_start_epoch_seconds,
            input.covered_end_epoch_seconds,
        )?;
        validate_time_order(
            input.covered_end_epoch_seconds,
            input.completed_at_epoch_seconds,
        )?;
        if input.bytes_written == 0 {
            return Err(CloudDataError::InvalidBackupPolicy);
        }
        validate_tenant_id(&input.tenant_id)?;
        let id = BackupEvidenceId::new(input.id)?;
        let service_id = ManagedDataServiceId::new(input.service_id)?;
        let region = RegionCode::new(input.region).map_err(|_| CloudDataError::InvalidRegion)?;
        let kms_key_id =
            KmsKeyId::new(input.kms_key_id).map_err(|_| CloudDataError::InvalidKmsKeyId)?;
        validate_backup_child_id(&id.value, &service.id.value.value)?;
        if service_id != service.id.value {
            return Err(CloudDataError::ServiceIdMismatch);
        }
        if input.tenant_id != service.tenant_id.value {
            return Err(CloudDataError::TenantMismatch);
        }
        if region != service.region.value {
            return Err(CloudDataError::RegionMismatch);
        }
        validate_kms_matches(&kms_key_id, &region, &input.tenant_id)?;
        if kms_key_id != service.backup.value.kms_key_id {
            return Err(CloudDataError::InvalidKmsKeyId);
        }
        if service.backup.value.quarterly_dr_drill && !input.restore_drill_verified {
            return Err(CloudDataError::InvalidBackupPolicy);
        }
        Ok(Self {
            id: internal(id),
            service_id: internal(service_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            kms_key_id: internal(kms_key_id),
            covered_start_epoch_seconds: internal(input.covered_start_epoch_seconds),
            covered_end_epoch_seconds: internal(input.covered_end_epoch_seconds),
            completed_at_epoch_seconds: internal(input.completed_at_epoch_seconds),
            bytes_written: internal(input.bytes_written),
            restore_drill_verified: internal(input.restore_drill_verified),
            schema_version: public(BACKUP_EVIDENCE_SCHEMA_VERSION),
        })
    }
}

impl DataTenantCellGuardrail {
    pub fn new(input: DataTenantCellGuardrailCreate) -> Result<Self, CloudDataError> {
        let DataTenantCellGuardrailCreate {
            service_id,
            tenant_id,
            region,
            primary_cell_id,
            engine,
            table_refs,
            tenant_partition_column,
            cell_partition_column,
            citus_distribution_column,
            citus_colocated_table_ref,
            row_level_security_enabled,
            force_row_level_security,
            rls_policy_ref,
            migration,
            backup,
            restore_drill_evidence_ref,
            evidence_refs,
            residency,
            allowed_data_classes,
            engine_shape,
            state,
        } = input;

        if state != ManagedDataState::Provisioning {
            return Err(CloudDataError::InvalidInitialState);
        }
        validate_tenant_id(&tenant_id)?;
        let id = ManagedDataServiceId::new(service_id)?;
        let id_parts = parse_service_id(&id.value)?;
        let region = RegionCode::new(region).map_err(|_| CloudDataError::InvalidRegion)?;
        let cell_id = CellId::new(primary_cell_id).map_err(|_| CloudDataError::InvalidCellId)?;
        validate_cell_region(&cell_id, &region)?;
        validate_service_id_matches(&id_parts, engine, &region, &tenant_id)?;
        validate_residency(&residency, &region)?;
        let allowed_data_classes = typed_privacy_classes(&allowed_data_classes)?;
        validate_engine_shape(engine, &engine_shape)?;
        let table_refs = validate_table_refs(&table_refs)?;
        validate_tenant_cell_partitioning(
            engine,
            &tenant_partition_column,
            &cell_partition_column,
            citus_distribution_column.as_deref(),
            citus_colocated_table_ref.as_deref(),
            &table_refs,
        )?;
        validate_row_level_security(row_level_security_enabled, force_row_level_security)?;
        validate_metadata_ref_path(&rls_policy_ref, REF_POLICY_PREFIX)?;
        validate_migration_policy(&migration)?;
        let backup = typed_backup(backup, &region, &tenant_id)?;
        validate_metadata_ref_path(&restore_drill_evidence_ref, REF_RESTORE_PREFIX)?;
        if evidence_refs.is_empty() {
            return Err(CloudDataError::InvalidReference);
        }
        let evidence_refs = unique_metadata_refs(&evidence_refs, REF_EVIDENCE_PREFIX)?;

        Ok(Self {
            service_id: internal(id),
            tenant_id: internal(tenant_id),
            region: public(region),
            primary_cell_id: public(cell_id),
            engine: public(engine),
            table_refs: internal(table_refs),
            tenant_partition_column: internal(tenant_partition_column),
            cell_partition_column: internal(cell_partition_column),
            citus_distribution_column: internal(citus_distribution_column),
            citus_colocated_table_ref: internal(citus_colocated_table_ref),
            row_level_security_enabled: internal(row_level_security_enabled),
            force_row_level_security: internal(force_row_level_security),
            rls_policy_ref: internal(rls_policy_ref),
            migration: internal(migration),
            backup: internal(backup),
            restore_drill_evidence_ref: internal(restore_drill_evidence_ref),
            evidence_refs: internal(evidence_refs),
            residency: internal(residency),
            allowed_data_classes: public(allowed_data_classes),
            engine_shape: internal(engine_shape),
            state: public(state),
            schema_version: public(CLOUD_DATA_SCHEMA_VERSION),
        })
    }
}

impl CloudDataCatalog {
    pub fn create_service(
        &mut self,
        input: ManagedDataServiceCreate,
    ) -> Result<ManagedDataService, CloudDataError> {
        let service = ManagedDataService::new(input)?;
        if self.services.contains_key(&service.id.value) {
            return Err(CloudDataError::DuplicateService);
        }
        self.services
            .insert(service.id.value.clone(), service.clone());
        Ok(service)
    }

    pub fn transition_service(
        &mut self,
        service_id: &ManagedDataServiceId,
        next_state: ManagedDataState,
        updated_at_epoch_seconds: u64,
    ) -> Result<ManagedDataService, CloudDataError> {
        let service = self
            .services
            .get(service_id)
            .ok_or(CloudDataError::UnknownService)?;
        let next = service.transition(next_state, updated_at_epoch_seconds)?;
        self.services.insert(service_id.clone(), next.clone());
        Ok(next)
    }

    pub fn record_backup(
        &mut self,
        input: BackupEvidenceCreate,
    ) -> Result<BackupEvidence, CloudDataError> {
        let service_id = ManagedDataServiceId::new(input.service_id.clone())?;
        let service = self
            .services
            .get(&service_id)
            .ok_or(CloudDataError::UnknownService)?;
        let evidence = BackupEvidence::new(service, input)?;
        if self.backups.contains_key(&evidence.id.value) {
            return Err(CloudDataError::DuplicateBackupEvidence);
        }
        self.backups
            .insert(evidence.id.value.clone(), evidence.clone());
        Ok(evidence)
    }

    pub fn services(&self) -> impl Iterator<Item = &ManagedDataService> {
        self.services.values()
    }

    pub fn backups(&self) -> impl Iterator<Item = &BackupEvidence> {
        self.backups.values()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ServiceIdParts {
    engine: ManagedDataEngine, // data_class: PUBLIC
    region: RegionCode,        // data_class: PUBLIC
    tenant_id: String,         // data_class: INTERNAL_ONLY
    name: String,              // data_class: INTERNAL_ONLY
}

fn parse_service_id(value: &str) -> Result<ServiceIdParts, CloudDataError> {
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() != 5 || parts[0] != SERVICE_ID_PREFIX || parts.iter().any(|part| part.is_empty())
    {
        return Err(CloudDataError::InvalidServiceId);
    }
    let engine = parse_engine(parts[1])?;
    let region = RegionCode::new(parts[2]).map_err(|_| CloudDataError::InvalidServiceId)?;
    validate_tenant_id(parts[3]).map_err(|_| CloudDataError::InvalidServiceId)?;
    validate_segment(parts[4]).map_err(|_| CloudDataError::InvalidServiceId)?;
    Ok(ServiceIdParts {
        engine,
        region,
        tenant_id: parts[3].to_string(),
        name: parts[4].to_string(),
    })
}

fn parse_engine(value: &str) -> Result<ManagedDataEngine, CloudDataError> {
    match value {
        "postgres" => Ok(ManagedDataEngine::Postgres),
        "citus" => Ok(ManagedDataEngine::Citus),
        "pgvector" => Ok(ManagedDataEngine::PgVector),
        "valkey" => Ok(ManagedDataEngine::Valkey),
        "kafka" => Ok(ManagedDataEngine::Kafka),
        "clickhouse" => Ok(ManagedDataEngine::ClickHouse),
        "cassandra" => Ok(ManagedDataEngine::Cassandra),
        "iceberg" => Ok(ManagedDataEngine::Iceberg),
        "milvus" => Ok(ManagedDataEngine::Milvus),
        "temporal" => Ok(ManagedDataEngine::Temporal),
        _ => Err(CloudDataError::InvalidServiceId),
    }
}

fn validate_service_id_matches(
    parts: &ServiceIdParts,
    engine: ManagedDataEngine,
    region: &RegionCode,
    tenant_id: &str,
) -> Result<(), CloudDataError> {
    if parts.engine != engine {
        return Err(CloudDataError::ServiceIdMismatch);
    }
    if &parts.region != region {
        return Err(CloudDataError::RegionMismatch);
    }
    if parts.tenant_id != tenant_id {
        return Err(CloudDataError::TenantMismatch);
    }
    Ok(())
}

fn validate_resource_matches(
    resource_id: &ResourceId,
    engine: ManagedDataEngine,
    region: &RegionCode,
    tenant_id: &str,
) -> Result<(), CloudDataError> {
    if resource_id
        .region()
        .map_err(|_| CloudDataError::InvalidResourceId)?
        != *region
    {
        return Err(CloudDataError::RegionMismatch);
    }
    if resource_id
        .tenant_id()
        .map_err(|_| CloudDataError::InvalidResourceId)?
        != tenant_id
    {
        return Err(CloudDataError::TenantMismatch);
    }
    if resource_id
        .kind_label()
        .map_err(|_| CloudDataError::InvalidResourceId)?
        != engine.resource_kind().type_label()
    {
        return Err(CloudDataError::ResourceKindMismatch);
    }
    Ok(())
}

fn typed_azs(values: &[String], region: &RegionCode) -> Result<Vec<AzCode>, CloudDataError> {
    if values.len() < MIN_REPLICATED_AZ_COUNT {
        return Err(CloudDataError::InvalidReplicationPolicy);
    }
    let mut seen = BTreeSet::new();
    let mut azs = Vec::with_capacity(values.len());
    for value in values {
        let az = AzCode::new(value.clone()).map_err(|_| CloudDataError::InvalidAz)?;
        validate_az_region(&az, region)?;
        if !seen.insert(az.clone()) {
            return Err(CloudDataError::InvalidReplicationPolicy);
        }
        azs.push(az);
    }
    Ok(azs)
}

fn validate_az_region(az: &AzCode, region: &RegionCode) -> Result<(), CloudDataError> {
    let prefix = format!("{}-", region.value);
    if az.value.starts_with(&prefix) && az.value.len() > prefix.len() {
        Ok(())
    } else {
        Err(CloudDataError::RegionMismatch)
    }
}

fn validate_cell_region(cell_id: &CellId, region: &RegionCode) -> Result<(), CloudDataError> {
    let prefix = format!("cell-{}-", region.value);
    if cell_id.value.starts_with(&prefix) {
        Ok(())
    } else {
        Err(CloudDataError::CellRegionMismatch)
    }
}

fn typed_privacy_classes(values: &[DataClass]) -> Result<Vec<PrivacyDataClass>, CloudDataError> {
    if values.is_empty() {
        return Err(CloudDataError::InvalidDataClass);
    }
    let mut seen = BTreeSet::new();
    let mut typed = Vec::with_capacity(values.len());
    for data_class in values {
        let privacy =
            PrivacyDataClass::new(*data_class).map_err(|_| CloudDataError::InvalidDataClass)?;
        if !seen.insert(privacy) {
            return Err(CloudDataError::InvalidDataClass);
        }
        typed.push(privacy);
    }
    Ok(typed)
}

fn typed_replication(
    input: DataReplicationPolicy,
    region: &RegionCode,
    azs: &[AzCode],
) -> Result<TypedDataReplicationPolicy, CloudDataError> {
    if azs.len() < MIN_REPLICATED_AZ_COUNT {
        return Err(CloudDataError::InvalidReplicationPolicy);
    }
    match input.mode {
        ReplicationMode::ThreeAz => {
            if input.cross_region.is_some() || input.residency_policy_ref.is_some() {
                return Err(CloudDataError::InvalidReplicationPolicy);
            }
            Ok(TypedDataReplicationPolicy {
                mode: input.mode,
                cross_region: None,
                residency_policy_ref: None,
            })
        }
        ReplicationMode::ThreeAzWithCrossRegionReadMirror => {
            let cross_region = input
                .cross_region
                .ok_or(CloudDataError::CrossRegionPolicyRequired)
                .and_then(|value| {
                    RegionCode::new(value).map_err(|_| CloudDataError::InvalidRegion)
                })?;
            if cross_region == *region {
                return Err(CloudDataError::InvalidReplicationPolicy);
            }
            let policy_ref = input
                .residency_policy_ref
                .ok_or(CloudDataError::CrossRegionPolicyRequired)?;
            validate_ref_path(&policy_ref, REF_RESIDENCY_POLICY_PREFIX)?;
            Ok(TypedDataReplicationPolicy {
                mode: input.mode,
                cross_region: Some(cross_region),
                residency_policy_ref: Some(policy_ref),
            })
        }
    }
}

fn typed_backup(
    input: DataBackupPolicy,
    region: &RegionCode,
    tenant_id: &str,
) -> Result<TypedDataBackupPolicy, CloudDataError> {
    if input.pitr_days < DEFAULT_PITR_DAYS
        || !input.weekly_tenant_dump
        || input.retention_days < MIN_BACKUP_RETENTION_DAYS
        || !input.quarterly_dr_drill
    {
        return Err(CloudDataError::InvalidBackupPolicy);
    }
    let kms_key_id =
        KmsKeyId::new(input.kms_key_id).map_err(|_| CloudDataError::InvalidKmsKeyId)?;
    validate_kms_matches(&kms_key_id, region, tenant_id)?;
    validate_ref_path(&input.object_store_ref, REF_OBJECT_STORE_PREFIX)?;
    Ok(TypedDataBackupPolicy {
        pitr_days: input.pitr_days,
        weekly_tenant_dump: input.weekly_tenant_dump,
        retention_days: input.retention_days,
        kms_key_id,
        object_store_ref: input.object_store_ref,
        quarterly_dr_drill: input.quarterly_dr_drill,
    })
}

fn validate_kms_matches(
    kms_key_id: &KmsKeyId,
    region: &RegionCode,
    tenant_id: &str,
) -> Result<(), CloudDataError> {
    if kms_key_id
        .region()
        .map_err(|_| CloudDataError::InvalidKmsKeyId)?
        != *region
    {
        return Err(CloudDataError::RegionMismatch);
    }
    if kms_key_id
        .tenant_id()
        .map_err(|_| CloudDataError::InvalidKmsKeyId)?
        != tenant_id
    {
        return Err(CloudDataError::TenantMismatch);
    }
    Ok(())
}

fn validate_migration_policy(input: &SchemaMigrationPolicy) -> Result<(), CloudDataError> {
    validate_ref_path(&input.forward_ref, REF_MIGRATION_PREFIX)?;
    validate_ref_path(&input.backward_ref, REF_MIGRATION_PREFIX)?;
    validate_ref_path(&input.synthetic_corpus_ref, REF_CORPUS_PREFIX)?;
    if !input.audit_chained || !input.backward_compatible {
        return Err(CloudDataError::InvalidMigrationPolicy);
    }
    Ok(())
}

fn validate_engine_shape(
    engine: ManagedDataEngine,
    shape: &EngineShape,
) -> Result<(), CloudDataError> {
    match (engine, shape) {
        (ManagedDataEngine::Postgres, EngineShape::Postgres(postgres))
        | (ManagedDataEngine::Citus, EngineShape::Postgres(postgres))
        | (ManagedDataEngine::PgVector, EngineShape::Postgres(postgres)) => {
            validate_postgres_shape(engine, postgres)
        }
        (ManagedDataEngine::Valkey, EngineShape::Valkey(valkey)) => validate_valkey_shape(valkey),
        (ManagedDataEngine::Kafka, EngineShape::Kafka(kafka)) => validate_kafka_shape(kafka),
        (ManagedDataEngine::ClickHouse, EngineShape::ClickHouse(clickhouse)) => {
            validate_clickhouse_shape(clickhouse)
        }
        (
            ManagedDataEngine::Cassandra
            | ManagedDataEngine::Iceberg
            | ManagedDataEngine::Milvus
            | ManagedDataEngine::Temporal,
            EngineShape::StableExpansion(gate),
        ) => validate_stable_expansion_gate(engine, gate),
        _ => Err(CloudDataError::InvalidEngineShape),
    }
}

fn validate_postgres_shape(
    engine: ManagedDataEngine,
    shape: &PostgresShape,
) -> Result<(), CloudDataError> {
    if shape.major_version != POSTGRES_MAJOR_VERSION
        || !shape.pgbouncer_per_tenant_pool
        || !matches!(shape.per_tenant_shards, 32 | 64)
    {
        return Err(CloudDataError::InvalidEngineShape);
    }
    let extension_set: BTreeSet<PostgresExtension> = shape.extensions.iter().copied().collect();
    if extension_set.len() != shape.extensions.len()
        || !extension_set.contains(&PostgresExtension::Citus)
    {
        return Err(CloudDataError::InvalidEngineShape);
    }
    if engine == ManagedDataEngine::PgVector
        && !extension_set.contains(&PostgresExtension::PgVector)
    {
        return Err(CloudDataError::InvalidEngineShape);
    }
    Ok(())
}

fn validate_valkey_shape(shape: &ValkeyShape) -> Result<(), CloudDataError> {
    if shape.replica_count < 3 {
        return Err(CloudDataError::InvalidEngineShape);
    }
    if shape.distribution == ValkeyDistribution::ForbiddenSsplRsal74OrLater {
        return Err(CloudDataError::InvalidLicensePosture);
    }
    if shape.major_version == 0 || shape.max_ttl_seconds == Some(0) {
        return Err(CloudDataError::InvalidEngineShape);
    }
    Ok(())
}

fn validate_kafka_shape(shape: &KafkaShape) -> Result<(), CloudDataError> {
    if shape.broker_count < 3
        || shape.partition_key != KafkaPartitionKey::TenantCell
        || !shape.cloudevents_envelope
        || !shape.protobuf_payloads
        || shape.outbox_poll_min_ms < MIN_KAFKA_OUTBOX_POLL_MS
        || shape.outbox_poll_max_ms > MAX_KAFKA_OUTBOX_POLL_MS
        || shape.outbox_poll_min_ms > shape.outbox_poll_max_ms
    {
        return Err(CloudDataError::InvalidEngineShape);
    }
    validate_ref_path(&shape.schema_registry_ref, REF_SCHEMA_REGISTRY_PREFIX)?;
    Ok(())
}

fn validate_clickhouse_shape(shape: &ClickHouseShape) -> Result<(), CloudDataError> {
    validate_ref_path(&shape.apache2_fork_attestation_ref, REF_LICENSE_PREFIX)?;
    if shape.fork_tracking != ForkTrackingCadence::PerReleaseWithQuarterlyReview
        || !shape.per_tenant_database
        || shape.materialized_view_refs.is_empty()
    {
        return Err(CloudDataError::InvalidLicensePosture);
    }
    unique_refs(&shape.materialized_view_refs, "view")?;
    Ok(())
}

fn validate_stable_expansion_gate(
    engine: ManagedDataEngine,
    gate: &StableExpansionGate,
) -> Result<(), CloudDataError> {
    validate_ref_path(&gate.evidence_ref, REF_ADR_GATE_PREFIX)?;
    if engine.stable_gate_adr() != Some(gate.adr) || gate.decision != GateDecision::Accepted {
        return Err(CloudDataError::InvalidExpansionGate);
    }
    Ok(())
}

fn validate_table_refs(values: &[String]) -> Result<Vec<String>, CloudDataError> {
    if values.is_empty() {
        return Err(CloudDataError::InvalidPartitioningPolicy);
    }
    unique_metadata_refs(values, REF_TABLE_PREFIX)
}

fn validate_tenant_cell_partitioning(
    engine: ManagedDataEngine,
    tenant_partition_column: &str,
    cell_partition_column: &str,
    citus_distribution_column: Option<&str>,
    citus_colocated_table_ref: Option<&str>,
    table_refs: &[String],
) -> Result<(), CloudDataError> {
    validate_column_name(tenant_partition_column)?;
    validate_column_name(cell_partition_column)?;
    if tenant_partition_column != TENANT_PARTITION_COLUMN
        || cell_partition_column != CELL_PARTITION_COLUMN
    {
        return Err(CloudDataError::InvalidPartitioningPolicy);
    }

    match engine {
        ManagedDataEngine::Postgres => {
            if citus_distribution_column.is_some() || citus_colocated_table_ref.is_some() {
                return Err(CloudDataError::InvalidPartitioningPolicy);
            }
            Ok(())
        }
        ManagedDataEngine::Citus => {
            let distribution_column =
                citus_distribution_column.ok_or(CloudDataError::InvalidPartitioningPolicy)?;
            if distribution_column != tenant_partition_column {
                return Err(CloudDataError::InvalidPartitioningPolicy);
            }
            let colocated_ref =
                citus_colocated_table_ref.ok_or(CloudDataError::InvalidPartitioningPolicy)?;
            validate_metadata_ref_path(colocated_ref, REF_TABLE_PREFIX)?;
            if !table_refs
                .iter()
                .any(|table_ref| table_ref == colocated_ref)
            {
                return Err(CloudDataError::InvalidPartitioningPolicy);
            }
            Ok(())
        }
        ManagedDataEngine::PgVector
        | ManagedDataEngine::Valkey
        | ManagedDataEngine::Kafka
        | ManagedDataEngine::ClickHouse
        | ManagedDataEngine::Cassandra
        | ManagedDataEngine::Iceberg
        | ManagedDataEngine::Milvus
        | ManagedDataEngine::Temporal => Err(CloudDataError::InvalidEngineShape),
    }
}

fn validate_column_name(value: &str) -> Result<(), CloudDataError> {
    validate_segment(value).map_err(|_| CloudDataError::InvalidPartitioningPolicy)
}

fn validate_row_level_security(enabled: bool, force_enabled: bool) -> Result<(), CloudDataError> {
    if enabled && force_enabled {
        Ok(())
    } else {
        Err(CloudDataError::InvalidTenantIsolationPolicy)
    }
}

fn validate_residency(
    residency: &ResidencyClass,
    region: &RegionCode,
) -> Result<(), CloudDataError> {
    if residency_class_allows_home_region_label(residency, &region.value) {
        Ok(())
    } else {
        Err(CloudDataError::ResidencyRegionMismatch)
    }
}

fn managed_data_transition_allowed(current: ManagedDataState, next: ManagedDataState) -> bool {
    matches!(
        (current, next),
        (ManagedDataState::Provisioning, ManagedDataState::Ready)
            | (ManagedDataState::Ready, ManagedDataState::Draining)
            | (ManagedDataState::Draining, ManagedDataState::Deleted)
    )
}

fn validate_backup_child_id(id: &str, service_id: &str) -> Result<(), CloudDataError> {
    let required = format!("{BACKUP_EVIDENCE_ID_PREFIX}/{service_id}/");
    if id.starts_with(&required) {
        Ok(())
    } else {
        Err(CloudDataError::InvalidBackupEvidenceId)
    }
}

fn validate_ref_path(value: &str, prefix: &str) -> Result<(), CloudDataError> {
    validate_path(value, prefix, 3, CloudDataError::InvalidReference)
}

fn validate_metadata_ref_path(value: &str, prefix: &str) -> Result<(), CloudDataError> {
    if contains_secret_like_reference(value) {
        return Err(CloudDataError::InvalidReference);
    }
    validate_ref_path(value, prefix)
}

fn validate_path(
    value: &str,
    prefix: &str,
    min_segments: usize,
    error: CloudDataError,
) -> Result<(), CloudDataError> {
    let parts: Vec<&str> = value.split('/').collect();
    if parts.len() < min_segments || parts.first().copied() != Some(prefix) {
        return Err(error);
    }
    for part in parts.iter().skip(1) {
        validate_segment(part).map_err(|_| error.clone())?;
    }
    Ok(())
}

fn unique_refs(values: &[String], prefix: &str) -> Result<(), CloudDataError> {
    let mut seen = BTreeSet::new();
    for value in values {
        validate_ref_path(value, prefix)?;
        if !seen.insert(value.clone()) {
            return Err(CloudDataError::InvalidReference);
        }
    }
    Ok(())
}

fn unique_metadata_refs(values: &[String], prefix: &str) -> Result<Vec<String>, CloudDataError> {
    let mut seen = BTreeSet::new();
    let mut refs = Vec::with_capacity(values.len());
    for value in values {
        validate_metadata_ref_path(value, prefix)?;
        if !seen.insert(value.clone()) {
            return Err(CloudDataError::InvalidReference);
        }
        refs.push(value.clone());
    }
    Ok(refs)
}

fn contains_secret_like_reference(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    ["secret", "token", "password", "credential", "private_key"]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn validate_segment(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
    {
        Err(())
    } else {
        Ok(())
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudDataError> {
    let Some(suffix) = value.strip_prefix(TENANT_ID_PREFIX) else {
        return Err(CloudDataError::InvalidTenantId);
    };
    validate_segment(suffix).map_err(|_| CloudDataError::InvalidTenantId)?;
    if suffix.is_empty() {
        return Err(CloudDataError::InvalidTenantId);
    }
    Ok(())
}

fn validate_positive_time(value: u64) -> Result<(), CloudDataError> {
    if value == 0 {
        Err(CloudDataError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_time_order(start: u64, end: u64) -> Result<(), CloudDataError> {
    if start == 0 || end <= start {
        Err(CloudDataError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_residency::{
        PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
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

    fn azs() -> Vec<String> {
        vec![
            "region-alpha1-a".to_string(),
            "region-alpha1-b".to_string(),
            "region-alpha1-c".to_string(),
        ]
    }

    fn replication() -> DataReplicationPolicy {
        DataReplicationPolicy {
            mode: ReplicationMode::ThreeAz,
            cross_region: None,
            residency_policy_ref: None,
        }
    }

    fn backup() -> DataBackupPolicy {
        DataBackupPolicy {
            pitr_days: 14,
            weekly_tenant_dump: true,
            retention_days: 90,
            kms_key_id: KMS_KEY.to_string(),
            object_store_ref: "object/region-alpha1/db-backup".to_string(),
            quarterly_dr_drill: true,
        }
    }

    fn migration() -> SchemaMigrationPolicy {
        SchemaMigrationPolicy {
            forward_ref: "migration/db/forward".to_string(),
            backward_ref: "migration/db/backward".to_string(),
            synthetic_corpus_ref: "corpus/db/synthetic-10k".to_string(),
            audit_chained: true,
            backward_compatible: true,
        }
    }

    fn postgres_shape(extra: Vec<PostgresExtension>) -> EngineShape {
        let mut extensions = vec![PostgresExtension::Citus];
        extensions.extend(extra);
        EngineShape::Postgres(PostgresShape {
            major_version: 16,
            extensions,
            per_tenant_shards: 32,
            pgbouncer_per_tenant_pool: true,
        })
    }

    fn service_create(engine: ManagedDataEngine, shape: EngineShape) -> ManagedDataServiceCreate {
        ManagedDataServiceCreate {
            id: format!("data/{}/{REGION}/{TENANT}/primary", engine.token()),
            tenant_id: TENANT.to_string(),
            region: REGION.to_string(),
            primary_cell_id: CELL.to_string(),
            azs: azs(),
            resource_id: format!(
                "oya:cloud:{REGION}:{TENANT}:{}:primary",
                engine.resource_kind().type_label()
            ),
            engine,
            state: ManagedDataState::Provisioning,
            residency: residency_class(),
            allowed_data_classes: vec![DataClass::InternalOnly, DataClass::PiiIdentifying],
            replication: replication(),
            backup: backup(),
            migration: migration(),
            engine_shape: shape,
            created_at_epoch_seconds: 1,
        }
    }

    #[test]
    fn creates_postgres_family_service_with_canonical_topology() {
        let service = ManagedDataService::new(service_create(
            ManagedDataEngine::PgVector,
            postgres_shape(vec![
                PostgresExtension::PgVector,
                PostgresExtension::TimescaleDb,
            ]),
        ))
        .expect("pgvector service");
        assert_eq!(service.engine.value, ManagedDataEngine::PgVector);
        assert_eq!(service.tier.value, ManagedDataTier::Oltp);
        assert_eq!(service.azs.value.len(), 3);
        assert_eq!(service.backup.value.kms_key_id.value, KMS_KEY);
    }

    #[test]
    fn rejects_forged_state_location_resource_and_data_class_drift() {
        assert_eq!(
            ManagedDataService::new(ManagedDataServiceCreate {
                state: ManagedDataState::Ready,
                ..service_create(ManagedDataEngine::Postgres, postgres_shape(vec![]))
            })
            .expect_err("ready state is not caller-controlled"),
            CloudDataError::InvalidInitialState
        );
        assert_eq!(
            ManagedDataService::new(ManagedDataServiceCreate {
                azs: vec!["region-alpha1-a".to_string(), "region-alpha1-b".to_string()],
                ..service_create(ManagedDataEngine::Postgres, postgres_shape(vec![]))
            })
            .expect_err("three AZs are required"),
            CloudDataError::InvalidReplicationPolicy
        );
        assert_eq!(
            ManagedDataService::new(ManagedDataServiceCreate {
                resource_id: format!("oya:cloud:{REGION}:{TENANT}:database:primary"),
                ..service_create(
                    ManagedDataEngine::Kafka,
                    EngineShape::Kafka(valid_kafka_shape())
                )
            })
            .expect_err("Kafka is queue-stream resource kind"),
            CloudDataError::ResourceKindMismatch
        );
        assert_eq!(
            ManagedDataService::new(ManagedDataServiceCreate {
                allowed_data_classes: vec![DataClass::Secret],
                ..service_create(ManagedDataEngine::Postgres, postgres_shape(vec![]))
            })
            .expect_err("operational labels are rejected"),
            CloudDataError::InvalidDataClass
        );
    }

    #[test]
    fn valkey_license_posture_is_enforced() {
        let service = ManagedDataService::new(service_create(
            ManagedDataEngine::Valkey,
            EngineShape::Valkey(ValkeyShape {
                distribution: ValkeyDistribution::Valkey,
                major_version: 8,
                replica_count: 3,
                max_ttl_seconds: Some(86_400),
            }),
        ))
        .expect("license-clean Valkey-compatible service");
        assert_eq!(service.tier.value, ManagedDataTier::Cache);
        assert_eq!(
            ManagedDataService::new(service_create(
                ManagedDataEngine::Valkey,
                EngineShape::Valkey(ValkeyShape {
                    distribution: ValkeyDistribution::ForbiddenSsplRsal74OrLater,
                    major_version: 8,
                    replica_count: 2,
                    max_ttl_seconds: None,
                }),
            ))
            .expect_err("Valkey cache requires replicated topology"),
            CloudDataError::InvalidEngineShape
        );
    }

    #[test]
    fn kafka_requires_tenant_cell_partitioning_schema_registry_and_outbox_window() {
        let service = ManagedDataService::new(service_create(
            ManagedDataEngine::Kafka,
            EngineShape::Kafka(valid_kafka_shape()),
        ))
        .expect("kafka service");
        assert_eq!(service.tier.value, ManagedDataTier::Stream);
        assert_eq!(
            ManagedDataService::new(service_create(
                ManagedDataEngine::Kafka,
                EngineShape::Kafka(KafkaShape {
                    partition_key: KafkaPartitionKey::TenantOnly,
                    ..valid_kafka_shape()
                }),
            ))
            .expect_err("partition key must include cell"),
            CloudDataError::InvalidEngineShape
        );
    }

    #[test]
    fn clickhouse_requires_fork_attestation_and_per_tenant_database() {
        let service = ManagedDataService::new(service_create(
            ManagedDataEngine::ClickHouse,
            EngineShape::ClickHouse(valid_clickhouse_shape()),
        ))
        .expect("clickhouse service");
        assert_eq!(service.tier.value, ManagedDataTier::Olap);
        assert_eq!(
            ManagedDataService::new(service_create(
                ManagedDataEngine::ClickHouse,
                EngineShape::ClickHouse(ClickHouseShape {
                    apache2_fork_attestation_ref: "bad".to_string(),
                    ..valid_clickhouse_shape()
                }),
            ))
            .expect_err("license evidence is required"),
            CloudDataError::InvalidReference
        );
    }

    #[test]
    fn stable_expansion_engines_require_accepted_matching_gate() {
        let service = ManagedDataService::new(service_create(
            ManagedDataEngine::Iceberg,
            EngineShape::StableExpansion(StableExpansionGate {
                adr: StableExpansionAdr::Adr0045,
                decision: GateDecision::Accepted,
                evidence_ref: "adr-gate/0045/iceberg".to_string(),
            }),
        ))
        .expect("accepted gated engine");
        assert_eq!(service.tier.value, ManagedDataTier::Lakehouse);
        assert_eq!(
            ManagedDataService::new(service_create(
                ManagedDataEngine::Milvus,
                EngineShape::StableExpansion(StableExpansionGate {
                    adr: StableExpansionAdr::Adr0045,
                    decision: GateDecision::Accepted,
                    evidence_ref: "adr-gate/0047/milvus".to_string(),
                }),
            ))
            .expect_err("Milvus is gated by ADR-0047"),
            CloudDataError::InvalidExpansionGate
        );
        assert_eq!(
            ManagedDataService::new(service_create(
                ManagedDataEngine::Temporal,
                EngineShape::StableExpansion(StableExpansionGate {
                    adr: StableExpansionAdr::Adr0035,
                    decision: GateDecision::Proposed,
                    evidence_ref: "adr-gate/0035/temporal".to_string(),
                }),
            ))
            .expect_err("proposed gate is not enough"),
            CloudDataError::InvalidExpansionGate
        );
    }

    #[test]
    fn cross_region_mirror_requires_explicit_residency_policy() {
        assert_eq!(
            ManagedDataService::new(ManagedDataServiceCreate {
                replication: DataReplicationPolicy {
                    mode: ReplicationMode::ThreeAzWithCrossRegionReadMirror,
                    cross_region: Some("region-beta1".to_string()),
                    residency_policy_ref: None,
                },
                ..service_create(ManagedDataEngine::Postgres, postgres_shape(vec![]))
            })
            .expect_err("cross-region mirror needs policy evidence"),
            CloudDataError::CrossRegionPolicyRequired
        );
        let service = ManagedDataService::new(ManagedDataServiceCreate {
            replication: DataReplicationPolicy {
                mode: ReplicationMode::ThreeAzWithCrossRegionReadMirror,
                cross_region: Some("region-beta1".to_string()),
                residency_policy_ref: Some("residency/global/read-mirror".to_string()),
            },
            ..service_create(ManagedDataEngine::Postgres, postgres_shape(vec![]))
        })
        .expect("explicitly governed mirror");
        assert_eq!(
            service.replication.value.mode,
            ReplicationMode::ThreeAzWithCrossRegionReadMirror
        );
    }

    #[test]
    fn catalog_records_backups_idempotently_against_service_contract() {
        let mut catalog = CloudDataCatalog::default();
        let service = catalog
            .create_service(service_create(
                ManagedDataEngine::Postgres,
                postgres_shape(vec![]),
            ))
            .expect("service");
        catalog
            .transition_service(&service.id.value, ManagedDataState::Ready, 2)
            .expect("ready");
        let evidence = backup_evidence(&service);
        catalog.record_backup(evidence.clone()).expect("backup");
        assert_eq!(catalog.backups().count(), 1);
        assert_eq!(
            catalog
                .record_backup(evidence)
                .expect_err("duplicate backup id rejected"),
            CloudDataError::DuplicateBackupEvidence
        );
        assert_eq!(
            BackupEvidence::new(
                &service,
                BackupEvidenceCreate {
                    kms_key_id: "kms/region-beta1/ten_acme/db-key".to_string(),
                    ..backup_evidence(&service)
                },
            )
            .expect_err("backup key must match service"),
            CloudDataError::RegionMismatch
        );
    }

    fn valid_kafka_shape() -> KafkaShape {
        KafkaShape {
            broker_count: 3,
            partition_key: KafkaPartitionKey::TenantCell,
            cloudevents_envelope: true,
            protobuf_payloads: true,
            schema_registry_ref: "schema/cloud/kafka".to_string(),
            outbox_poll_min_ms: 100,
            outbox_poll_max_ms: 500,
        }
    }

    fn valid_clickhouse_shape() -> ClickHouseShape {
        ClickHouseShape {
            apache2_fork_attestation_ref: "license/clickhouse/apache2".to_string(),
            fork_tracking: ForkTrackingCadence::PerReleaseWithQuarterlyReview,
            per_tenant_database: true,
            materialized_view_refs: vec!["view/cloud/finops".to_string()],
        }
    }

    fn backup_evidence(service: &ManagedDataService) -> BackupEvidenceCreate {
        BackupEvidenceCreate {
            id: format!("backup/{}/day-1", service.id.value.value),
            service_id: service.id.value.value.clone(),
            tenant_id: TENANT.to_string(),
            region: REGION.to_string(),
            kms_key_id: KMS_KEY.to_string(),
            covered_start_epoch_seconds: 10,
            covered_end_epoch_seconds: 20,
            completed_at_epoch_seconds: 30,
            bytes_written: 4096,
            restore_drill_verified: true,
        }
    }
}
