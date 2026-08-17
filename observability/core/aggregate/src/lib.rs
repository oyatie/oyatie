//! Cloud observability aggregate kernel.
//!
//! This crate owns the cloud-facing observability contract for metrics, log
//! streams, traces, alerts, dashboards, per-region telemetry residency, and the
//! `cloud.observability.audit.read` CloudTrail-class read projection. It stays
//! adapter-free: VictoriaMetrics, Loki, Tempo, object export, and REST/gRPC API
//! crates consume these value objects through ports.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use audit_chain_domain::{AuditChain, AuditEvent, Plane};
use cell_region::{CellId, RegionCode};
use compute_resource::{CloudResourceError, ResourceId};
use iam_cloud_domain::IamRoleId;
use network_residency::{ResidencyClass, residency_class_allows_home_region_label};
use observability_domain::{TelemetryLogExposure, log_exposure_for_classification};
use oya_data_boundary_kernel::{
    Classified, DataClass, DataClassification, OperationalDataClass, PrivacyDataClass, Purpose,
};

const OBSERVABILITY_SCHEMA_VERSION: u32 = 1;
const AUDIT_RECORD_SCHEMA_VERSION: u32 = 1;
const TENANT_ID_PREFIX: &str = "ten_";
const METRIC_ID_PREFIX: &str = "metric_";
const LOG_STREAM_ID_PREFIX: &str = "log_";
const TRACE_ID_PREFIX: &str = "trace_";
const ALERT_ID_PREFIX: &str = "alert_";
const DASHBOARD_ID_PREFIX: &str = "dash_";
const AUDIT_CURSOR_PREFIX: &str = "cur/";
const IDEMPOTENCY_KEY_PREFIX: &str = "idem/";
const REGIONAL_PACK_PREFIX: &str = "oya-pack-";
const SHA256_PREFIX: &str = "sha256:";
const FNV_HASH_PREFIX: &str = "fnv1a64:";
const SIGNED_EXPORT_URI_PREFIX: &str = "s3+signed://";
pub const DEFAULT_AUDIT_READ_PAGE_SIZE: u16 = 100;
pub const MAX_AUDIT_READ_PAGE_SIZE: u16 = 10_000;
pub const MAX_AUDIT_READ_WINDOW_SECONDS: u64 = 31 * 24 * 60 * 60;
pub const MIN_AUDIT_RETENTION_DAYS: u16 = 365;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MetricId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct LogStreamId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TraceId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AlertId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DashboardId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AuditReadCursor {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AuditRecordId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ActorRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdempotencyKey {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DigestRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SignedAuditExportUri {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TelemetryKind {
    Metric,
    Log,
    Trace,
    Audit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LogStreamKind {
    ControlPlane,
    DataPlane,
    Security,
    AuditExport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TraceSamplingMode {
    AlwaysOn,
    RatioPermille(u16),
    ErrorOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Page,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DashboardVisibility {
    TenantPrivate,
    OperatorInternal,
    PublicStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ObservabilityResidencyState {
    Enforcing,
    Retiring,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AuditRecordClass {
    ControlPlaneMutation,
    DataPlaneSecurity,
    BillingAnalytics,
    Replication,
    CapacityOperations,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AuditReadScope {
    ControlPlaneMutations,
    AllTenantAudit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudAuditOperation {
    ResourceCreated,
    ResourceTerminated,
    IamRoleAssumed,
    IamPolicyChanged,
    RegionRegistered,
    KmsKeyUsed,
    CrossRegionReplication,
    NetworkFlowAnomaly,
    InvoiceIssued,
    DirectInterconnectProvisioned,
    CellRebalanced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudAuditTopic {
    CloudResourceCreated,
    CloudResourceTerminated,
    CloudIamAssume,
    CloudIamPolicy,
    CloudRegionRegister,
    CloudKmsUse,
    CloudReplication,
    CloudFlowAnomaly,
    CloudInvoice,
    CloudInterconnect,
    CloudCellRebalanced,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservabilityResidencyCreate {
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: PUBLIC
    pub regional_pack: String,              // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass,          // data_class: INTERNAL_ONLY
    pub metric_storage_region: String,      // data_class: PUBLIC
    pub log_storage_region: String,         // data_class: PUBLIC
    pub trace_storage_region: String,       // data_class: PUBLIC
    pub audit_storage_region: String,       // data_class: PUBLIC
    pub signed_audit_export_uri: String,    // data_class: INTERNAL_ONLY
    pub retention_days: u16,                // data_class: INTERNAL_ONLY
    pub state: ObservabilityResidencyState, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservabilityResidency {
    pub tenant_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,    // data_class: PUBLIC
    pub regional_pack: Classified<String>, // data_class: INTERNAL_ONLY
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub metric_storage_region: Classified<RegionCode>, // data_class: PUBLIC
    pub log_storage_region: Classified<RegionCode>, // data_class: PUBLIC
    pub trace_storage_region: Classified<RegionCode>, // data_class: PUBLIC
    pub audit_storage_region: Classified<RegionCode>, // data_class: PUBLIC
    pub signed_audit_export_uri: Classified<SignedAuditExportUri>, // data_class: INTERNAL_ONLY
    pub retention_days: Classified<u16>,   // data_class: INTERNAL_ONLY
    pub state: Classified<ObservabilityResidencyState>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetricCreate {
    pub id: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: PUBLIC
    pub cell_id: Option<String>,            // data_class: PUBLIC
    pub name: String,                       // data_class: PUBLIC
    pub kind: MetricKind,                   // data_class: PUBLIC
    pub unit: String,                       // data_class: PUBLIC
    pub source_resource_id: Option<String>, // data_class: INTERNAL_ONLY
    pub data_classes: Vec<DataClass>,       // data_class: INTERNAL_ONLY
    pub retention_days: u16,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metric {
    pub id: Classified<MetricId>,            // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub cell_id: Classified<Option<CellId>>, // data_class: PUBLIC
    pub name: Classified<String>,            // data_class: PUBLIC
    pub kind: Classified<MetricKind>,        // data_class: PUBLIC
    pub unit: Classified<String>,            // data_class: PUBLIC
    pub source_resource_id: Classified<Option<ResourceId>>, // data_class: INTERNAL_ONLY
    pub data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub retention_days: Classified<u16>,     // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogStreamCreate {
    pub id: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: PUBLIC
    pub cell_id: Option<String>,            // data_class: PUBLIC
    pub kind: LogStreamKind,                // data_class: PUBLIC
    pub source_resource_id: Option<String>, // data_class: INTERNAL_ONLY
    pub data_classes: Vec<DataClass>,       // data_class: INTERNAL_ONLY
    pub retention_days: u16,                // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogStream {
    pub id: Classified<LogStreamId>,         // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub cell_id: Classified<Option<CellId>>, // data_class: PUBLIC
    pub kind: Classified<LogStreamKind>,     // data_class: PUBLIC
    pub source_resource_id: Classified<Option<ResourceId>>, // data_class: INTERNAL_ONLY
    pub data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub retention_days: Classified<u16>,     // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceCreate {
    pub id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub region: String,                   // data_class: PUBLIC
    pub cell_id: Option<String>,          // data_class: PUBLIC
    pub sampling_mode: TraceSamplingMode, // data_class: INTERNAL_ONLY
    pub root_resource_id: Option<String>, // data_class: INTERNAL_ONLY
    pub data_classes: Vec<DataClass>,     // data_class: INTERNAL_ONLY
    pub retention_days: u16,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trace {
    pub id: Classified<TraceId>,             // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub cell_id: Classified<Option<CellId>>, // data_class: PUBLIC
    pub sampling_mode: Classified<TraceSamplingMode>, // data_class: INTERNAL_ONLY
    pub root_resource_id: Classified<Option<ResourceId>>, // data_class: INTERNAL_ONLY
    pub data_classes: Classified<Vec<PrivacyDataClass>>, // data_class: INTERNAL_ONLY
    pub retention_days: Classified<u16>,     // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlertCreate {
    pub id: String,                         // data_class: INTERNAL_ONLY
    pub tenant_id: String,                  // data_class: INTERNAL_ONLY
    pub region: String,                     // data_class: PUBLIC
    pub metric_id: String,                  // data_class: INTERNAL_ONLY
    pub severity: AlertSeverity,            // data_class: INTERNAL_ONLY
    pub expression_ref: String,             // data_class: INTERNAL_ONLY
    pub notify_principal_refs: Vec<String>, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,              // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alert {
    pub id: Classified<AlertId>,               // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,        // data_class: PUBLIC
    pub metric_id: Classified<MetricId>,       // data_class: INTERNAL_ONLY
    pub severity: Classified<AlertSeverity>,   // data_class: INTERNAL_ONLY
    pub expression_ref: Classified<DigestRef>, // data_class: INTERNAL_ONLY
    pub notify_principal_refs: Classified<Vec<ActorRef>>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,       // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DashboardCreate {
    pub id: String,                      // data_class: INTERNAL_ONLY
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub region: String,                  // data_class: PUBLIC
    pub title: String,                   // data_class: PUBLIC
    pub visibility: DashboardVisibility, // data_class: PUBLIC
    pub metric_ids: Vec<String>,         // data_class: INTERNAL_ONLY
    pub log_stream_ids: Vec<String>,     // data_class: INTERNAL_ONLY
    pub trace_ids: Vec<String>,          // data_class: INTERNAL_ONLY
    pub data_class: DataClass,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dashboard {
    pub id: Classified<DashboardId>,    // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub title: Classified<String>,      // data_class: PUBLIC
    pub visibility: Classified<DashboardVisibility>, // data_class: PUBLIC
    pub metric_ids: Classified<Vec<MetricId>>, // data_class: INTERNAL_ONLY
    pub log_stream_ids: Classified<Vec<LogStreamId>>, // data_class: INTERNAL_ONLY
    pub trace_ids: Classified<Vec<TraceId>>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudAuditEnvelopeCreate {
    pub event_sequence: u64,            // data_class: INTERNAL_ONLY
    pub topic: CloudAuditTopic,         // data_class: INTERNAL_ONLY
    pub operation: CloudAuditOperation, // data_class: INTERNAL_ONLY
    pub region: String,                 // data_class: PUBLIC
    pub cell_id: Option<String>,        // data_class: PUBLIC
    pub resource_id: Option<String>,    // data_class: INTERNAL_ONLY
    pub actor: String,                  // data_class: INTERNAL_ONLY
    pub iam_role: Option<String>,       // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub payload_hash: String,           // data_class: INTERNAL_ONLY
    pub idempotency_key: String,        // data_class: INTERNAL_ONLY
    pub signed_export_uri: String,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudAuditRecord {
    pub id: Classified<AuditRecordId>,  // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub cell_id: Classified<Option<CellId>>, // data_class: PUBLIC
    pub topic: Classified<CloudAuditTopic>, // data_class: INTERNAL_ONLY
    pub operation: Classified<CloudAuditOperation>, // data_class: INTERNAL_ONLY
    pub record_class: Classified<AuditRecordClass>, // data_class: INTERNAL_ONLY
    pub source_resource_id: Classified<Option<ResourceId>>, // data_class: INTERNAL_ONLY
    pub actor: Classified<ActorRef>,    // data_class: INTERNAL_ONLY
    pub iam_role: Classified<Option<IamRoleId>>, // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub chain_sequence: Classified<u64>, // data_class: INTERNAL_ONLY
    pub previous_hash: Classified<DigestRef>, // data_class: INTERNAL_ONLY
    pub hash: Classified<DigestRef>,    // data_class: INTERNAL_ONLY
    pub payload_hash: Classified<DigestRef>, // data_class: INTERNAL_ONLY
    pub idempotency_key: Classified<IdempotencyKey>, // data_class: INTERNAL_ONLY
    pub decision: Classified<String>,   // data_class: INTERNAL_ONLY
    pub purpose: Classified<Purpose>,   // data_class: INTERNAL_ONLY
    pub plane: Classified<Plane>,       // data_class: INTERNAL_ONLY
    pub data_classes_referenced: Classified<Vec<DataClassification>>, // data_class: INTERNAL_ONLY
    pub signed_export_uri: Classified<SignedAuditExportUri>, // data_class: INTERNAL_ONLY
    pub audit_marker: Classified<OperationalDataClass>, // data_class: AUDIT
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditReadRequest {
    pub tenant_id: String,            // data_class: INTERNAL_ONLY
    pub region: String,               // data_class: PUBLIC
    pub cell_id: Option<String>,      // data_class: PUBLIC
    pub scope: AuditReadScope,        // data_class: INTERNAL_ONLY
    pub start_epoch_seconds: u64,     // data_class: INTERNAL_ONLY
    pub end_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub topics: Vec<CloudAuditTopic>, // data_class: INTERNAL_ONLY
    pub actor: Option<String>,        // data_class: INTERNAL_ONLY
    pub resource_id: Option<String>,  // data_class: INTERNAL_ONLY
    pub cursor: Option<String>,       // data_class: INTERNAL_ONLY
    pub page_size: Option<u16>,       // data_class: INTERNAL_ONLY
    pub require_complete_chain: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditReadResult {
    pub records: Vec<CloudAuditRecord>, // data_class: INTERNAL_ONLY
    pub next_cursor: Option<AuditReadCursor>, // data_class: INTERNAL_ONLY
    pub chain_complete: bool,           // data_class: INTERNAL_ONLY
    pub high_watermark_sequence: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditReadSummary {
    pub total: u64,                                // data_class: INTERNAL_ONLY
    pub per_topic: BTreeMap<CloudAuditTopic, u64>, // data_class: INTERNAL_ONLY
    pub earliest_epoch_seconds: Option<u64>,       // data_class: INTERNAL_ONLY
    pub latest_epoch_seconds: Option<u64>,         // data_class: INTERNAL_ONLY
    pub chain_complete: bool,                      // data_class: INTERNAL_ONLY
    pub high_watermark_sequence: Option<u64>,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudObservabilityError {
    InvalidTenantId,
    InvalidRegion,
    InvalidCellId,
    InvalidRegionalPack,
    InvalidResidency,
    InvalidRetention,
    InvalidExportUri,
    InvalidMetricId,
    InvalidLogStreamId,
    InvalidTraceId,
    InvalidAlertId,
    InvalidDashboardId,
    InvalidMetricName,
    InvalidMetricUnit,
    InvalidExpressionRef,
    InvalidDashboardTitle,
    InvalidTelemetryClass,
    ForbiddenTelemetryClass,
    InvalidActorRef,
    InvalidRoleId,
    InvalidResourceId,
    ResourceTenantMismatch,
    ResourceRegionMismatch,
    InvalidAuditRecordId,
    InvalidAuditTopic,
    InvalidAuditOperation,
    AuditTopicOperationMismatch,
    AuditEnvelopeSequenceMismatch,
    InvalidAuditTimestamp,
    InvalidAuditHash,
    InvalidIdempotencyKey,
    InvalidReadWindow,
    InvalidPageSize,
    InvalidCursor,
    CursorTenantMismatch,
    CursorRegionMismatch,
    DuplicateEnvelopeSequence,
    MissingAuditEnvelope,
    DuplicateAuditRecord,
    DuplicateMetric,
    DuplicateLogStream,
    DuplicateTrace,
    DuplicateAlert,
    DuplicateDashboard,
    UnknownMetric,
    UnknownLogStream,
    UnknownTrace,
    IncompleteAuditChain,
    UnverifiedAuditChain,
    AuditChainMismatch,
    SourceAuditRejected,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudObservabilityCatalog {
    residencies: BTreeMap<(String, RegionCode), ObservabilityResidency>,
    metrics: BTreeMap<MetricId, Metric>,
    log_streams: BTreeMap<LogStreamId, LogStream>,
    traces: BTreeMap<TraceId, Trace>,
    alerts: BTreeMap<AlertId, Alert>,
    dashboards: BTreeMap<DashboardId, Dashboard>,
    audit_records: BTreeMap<AuditRecordId, CloudAuditRecord>,
    audit_sequences: BTreeSet<(String, RegionCode, u64)>,
    chain_verified: bool,
    high_watermark_sequence: Option<u64>,
}

impl CloudAuditTopic {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CloudResourceCreated => "oya.audit.cloud_resource_created",
            Self::CloudResourceTerminated => "oya.audit.cloud_resource_terminated",
            Self::CloudIamAssume => "oya.audit.cloud_iam_assume",
            Self::CloudIamPolicy => "oya.audit.cloud_iam_policy",
            Self::CloudRegionRegister => "oya.audit.cloud_region_register",
            Self::CloudKmsUse => "oya.audit.cloud_kms_use",
            Self::CloudReplication => "oya.audit.cloud_replication",
            Self::CloudFlowAnomaly => "oya.audit.cloud_flow_anomaly",
            Self::CloudInvoice => "oya.audit.cloud_invoice",
            Self::CloudInterconnect => "oya.audit.cloud_interconnect",
            Self::CloudCellRebalanced => "oya.audit.cloud_cell_rebalanced",
        }
    }

    pub const fn operation(self) -> CloudAuditOperation {
        match self {
            Self::CloudResourceCreated => CloudAuditOperation::ResourceCreated,
            Self::CloudResourceTerminated => CloudAuditOperation::ResourceTerminated,
            Self::CloudIamAssume => CloudAuditOperation::IamRoleAssumed,
            Self::CloudIamPolicy => CloudAuditOperation::IamPolicyChanged,
            Self::CloudRegionRegister => CloudAuditOperation::RegionRegistered,
            Self::CloudKmsUse => CloudAuditOperation::KmsKeyUsed,
            Self::CloudReplication => CloudAuditOperation::CrossRegionReplication,
            Self::CloudFlowAnomaly => CloudAuditOperation::NetworkFlowAnomaly,
            Self::CloudInvoice => CloudAuditOperation::InvoiceIssued,
            Self::CloudInterconnect => CloudAuditOperation::DirectInterconnectProvisioned,
            Self::CloudCellRebalanced => CloudAuditOperation::CellRebalanced,
        }
    }

    pub const fn record_class(self) -> AuditRecordClass {
        match self {
            Self::CloudResourceCreated
            | Self::CloudResourceTerminated
            | Self::CloudIamAssume
            | Self::CloudIamPolicy
            | Self::CloudRegionRegister
            | Self::CloudInterconnect => AuditRecordClass::ControlPlaneMutation,
            Self::CloudKmsUse | Self::CloudFlowAnomaly => AuditRecordClass::DataPlaneSecurity,
            Self::CloudInvoice => AuditRecordClass::BillingAnalytics,
            Self::CloudReplication => AuditRecordClass::Replication,
            Self::CloudCellRebalanced => AuditRecordClass::CapacityOperations,
        }
    }

    pub const fn is_control_plane_mutation(self) -> bool {
        matches!(self.record_class(), AuditRecordClass::ControlPlaneMutation)
    }
}

impl MetricId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        prefixed_id(
            value.into(),
            METRIC_ID_PREFIX,
            CloudObservabilityError::InvalidMetricId,
        )
        .map(|value| Self { value })
    }
}

impl LogStreamId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        prefixed_id(
            value.into(),
            LOG_STREAM_ID_PREFIX,
            CloudObservabilityError::InvalidLogStreamId,
        )
        .map(|value| Self { value })
    }
}

impl TraceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        prefixed_id(
            value.into(),
            TRACE_ID_PREFIX,
            CloudObservabilityError::InvalidTraceId,
        )
        .map(|value| Self { value })
    }
}

impl AlertId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        prefixed_id(
            value.into(),
            ALERT_ID_PREFIX,
            CloudObservabilityError::InvalidAlertId,
        )
        .map(|value| Self { value })
    }
}

impl DashboardId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        prefixed_id(
            value.into(),
            DASHBOARD_ID_PREFIX,
            CloudObservabilityError::InvalidDashboardId,
        )
        .map(|value| Self { value })
    }
}

impl AuditReadCursor {
    pub fn from_position(
        tenant_id: &str,
        region: &RegionCode,
        occurred_at_epoch_seconds: u64,
        chain_sequence: u64,
    ) -> Self {
        Self {
            value: format!(
                "{AUDIT_CURSOR_PREFIX}{tenant_id}/{}/{occurred_at_epoch_seconds}/{chain_sequence}",
                region.value
            ),
        }
    }

    fn parse(
        value: &str,
        tenant_id: &str,
        region: &RegionCode,
    ) -> Result<(u64, u64), CloudObservabilityError> {
        let Some(rest) = value.strip_prefix(AUDIT_CURSOR_PREFIX) else {
            return Err(CloudObservabilityError::InvalidCursor);
        };
        let parts: Vec<_> = rest.split('/').collect();
        if parts.len() != 4 {
            return Err(CloudObservabilityError::InvalidCursor);
        }
        if parts[0] != tenant_id {
            return Err(CloudObservabilityError::CursorTenantMismatch);
        }
        if parts[1] != region.value {
            return Err(CloudObservabilityError::CursorRegionMismatch);
        }
        let occurred_at = parts[2]
            .parse::<u64>()
            .map_err(|_| CloudObservabilityError::InvalidCursor)?;
        let sequence = parts[3]
            .parse::<u64>()
            .map_err(|_| CloudObservabilityError::InvalidCursor)?;
        Ok((occurred_at, sequence))
    }
}

impl AuditRecordId {
    fn for_event(event: &AuditEvent) -> Self {
        Self {
            value: format!("caud_{}_{}", event.tenant_id, event.sequence),
        }
    }

    pub fn new(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        prefixed_id(
            value.into(),
            "caud_",
            CloudObservabilityError::InvalidAuditRecordId,
        )
        .map(|value| Self { value })
    }
}

impl ActorRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        let value = value.into();
        let allowed = ["usr_", "sp_", "role_", "agent_", "sts_"];
        // ADR-0083 Tier 1: bind the matching prefix via `find` once instead of
        // checking `any` then re-finding with `.unwrap()`. If no prefix matches,
        // the `let-else` returns the canonical invalid-actor error.
        let Some(matched_prefix) = allowed.iter().find(|prefix| value.starts_with(*prefix)) else {
            return Err(CloudObservabilityError::InvalidActorRef);
        };
        if value.len() > matched_prefix.len() && is_ascii_token(&value) {
            Ok(Self { value })
        } else {
            Err(CloudObservabilityError::InvalidActorRef)
        }
    }
}

impl IdempotencyKey {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        let value = value.into();
        if value.starts_with(IDEMPOTENCY_KEY_PREFIX)
            && value.len() > IDEMPOTENCY_KEY_PREFIX.len()
            && value.len() <= 128
            && is_ascii_token_with_slash(&value)
        {
            Ok(Self { value })
        } else {
            Err(CloudObservabilityError::InvalidIdempotencyKey)
        }
    }
}

impl DigestRef {
    pub fn sha256(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        let value = value.into();
        if valid_sha256_ref(&value) {
            Ok(Self { value })
        } else {
            Err(CloudObservabilityError::InvalidAuditHash)
        }
    }

    fn audit_hash(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        let value = value.into();
        if value == "GENESIS" || valid_fnv_hash_ref(&value) || valid_sha256_ref(&value) {
            Ok(Self { value })
        } else {
            Err(CloudObservabilityError::InvalidAuditHash)
        }
    }
}

impl SignedAuditExportUri {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudObservabilityError> {
        let value = value.into();
        if value.starts_with(SIGNED_EXPORT_URI_PREFIX)
            && value.len() > SIGNED_EXPORT_URI_PREFIX.len()
            && !value.contains("..")
            && value.contains("?sig=")
        {
            Ok(Self { value })
        } else {
            Err(CloudObservabilityError::InvalidExportUri)
        }
    }
}

impl ObservabilityResidency {
    pub fn new(input: ObservabilityResidencyCreate) -> Result<Self, CloudObservabilityError> {
        validate_tenant_id(&input.tenant_id)?;
        let region = region_for(&input.region, &input.residency)?;
        validate_regional_pack(&input.regional_pack)?;
        let metric_storage_region = region_for(&input.metric_storage_region, &input.residency)?;
        let log_storage_region = region_for(&input.log_storage_region, &input.residency)?;
        let trace_storage_region = region_for(&input.trace_storage_region, &input.residency)?;
        let audit_storage_region = region_for(&input.audit_storage_region, &input.residency)?;
        validate_in_region(&region, &metric_storage_region)?;
        validate_in_region(&region, &log_storage_region)?;
        validate_in_region(&region, &trace_storage_region)?;
        validate_in_region(&region, &audit_storage_region)?;
        validate_retention(input.retention_days, MIN_AUDIT_RETENTION_DAYS)?;
        if input.state != ObservabilityResidencyState::Enforcing {
            return Err(CloudObservabilityError::InvalidResidency);
        }
        Ok(Self {
            tenant_id: internal(input.tenant_id),
            region: public(region),
            regional_pack: internal(input.regional_pack),
            residency: internal(input.residency),
            metric_storage_region: public(metric_storage_region),
            log_storage_region: public(log_storage_region),
            trace_storage_region: public(trace_storage_region),
            audit_storage_region: public(audit_storage_region),
            signed_audit_export_uri: internal(SignedAuditExportUri::new(
                input.signed_audit_export_uri,
            )?),
            retention_days: internal(input.retention_days),
            state: internal(input.state),
            schema_version: public(OBSERVABILITY_SCHEMA_VERSION),
        })
    }

    fn validate_tenant_region(
        &self,
        tenant_id: &str,
        region: &RegionCode,
    ) -> Result<(), CloudObservabilityError> {
        if self.tenant_id.value != tenant_id || self.region.value != *region {
            return Err(CloudObservabilityError::InvalidResidency);
        }
        if self.state.value != ObservabilityResidencyState::Enforcing {
            return Err(CloudObservabilityError::InvalidResidency);
        }
        Ok(())
    }

    pub fn storage_region_for(&self, kind: TelemetryKind) -> &RegionCode {
        match kind {
            TelemetryKind::Metric => &self.metric_storage_region.value,
            TelemetryKind::Log => &self.log_storage_region.value,
            TelemetryKind::Trace => &self.trace_storage_region.value,
            TelemetryKind::Audit => &self.audit_storage_region.value,
        }
    }
}

impl Metric {
    pub fn new(
        input: MetricCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Self, CloudObservabilityError> {
        let tenant_id = validated_tenant(input.tenant_id)?;
        let region = region_for(&input.region, &residency.residency.value)?;
        residency.validate_tenant_region(&tenant_id, &region)?;
        let cell_id = optional_cell_for(input.cell_id, &region)?;
        let source_resource_id =
            optional_resource_for(input.source_resource_id, &tenant_id, &region)?;
        let data_classes = telemetry_privacy_classes(input.data_classes)?;
        validate_metric_name(&input.name)?;
        validate_ascii_label(&input.unit, CloudObservabilityError::InvalidMetricUnit)?;
        validate_retention(input.retention_days, MIN_AUDIT_RETENTION_DAYS)?;
        Ok(Self {
            id: internal(MetricId::new(input.id)?),
            tenant_id: internal(tenant_id),
            region: public(region),
            cell_id: public(cell_id),
            name: public(input.name),
            kind: public(input.kind),
            unit: public(input.unit),
            source_resource_id: internal(source_resource_id),
            data_classes: internal(data_classes),
            retention_days: internal(input.retention_days),
            schema_version: public(OBSERVABILITY_SCHEMA_VERSION),
        })
    }
}

impl LogStream {
    pub fn new(
        input: LogStreamCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Self, CloudObservabilityError> {
        let tenant_id = validated_tenant(input.tenant_id)?;
        let region = region_for(&input.region, &residency.residency.value)?;
        residency.validate_tenant_region(&tenant_id, &region)?;
        let cell_id = optional_cell_for(input.cell_id, &region)?;
        let source_resource_id =
            optional_resource_for(input.source_resource_id, &tenant_id, &region)?;
        let data_classes = telemetry_privacy_classes(input.data_classes)?;
        validate_retention(input.retention_days, MIN_AUDIT_RETENTION_DAYS)?;
        Ok(Self {
            id: internal(LogStreamId::new(input.id)?),
            tenant_id: internal(tenant_id),
            region: public(region),
            cell_id: public(cell_id),
            kind: public(input.kind),
            source_resource_id: internal(source_resource_id),
            data_classes: internal(data_classes),
            retention_days: internal(input.retention_days),
            schema_version: public(OBSERVABILITY_SCHEMA_VERSION),
        })
    }
}

impl Trace {
    pub fn new(
        input: TraceCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Self, CloudObservabilityError> {
        let tenant_id = validated_tenant(input.tenant_id)?;
        let region = region_for(&input.region, &residency.residency.value)?;
        residency.validate_tenant_region(&tenant_id, &region)?;
        let cell_id = optional_cell_for(input.cell_id, &region)?;
        let root_resource_id = optional_resource_for(input.root_resource_id, &tenant_id, &region)?;
        let data_classes = telemetry_privacy_classes(input.data_classes)?;
        validate_sampling_mode(input.sampling_mode)?;
        validate_retention(input.retention_days, MIN_AUDIT_RETENTION_DAYS)?;
        Ok(Self {
            id: internal(TraceId::new(input.id)?),
            tenant_id: internal(tenant_id),
            region: public(region),
            cell_id: public(cell_id),
            sampling_mode: internal(input.sampling_mode),
            root_resource_id: internal(root_resource_id),
            data_classes: internal(data_classes),
            retention_days: internal(input.retention_days),
            schema_version: public(OBSERVABILITY_SCHEMA_VERSION),
        })
    }
}

impl Alert {
    pub fn new(
        input: AlertCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Self, CloudObservabilityError> {
        let tenant_id = validated_tenant(input.tenant_id)?;
        let region = region_for(&input.region, &residency.residency.value)?;
        residency.validate_tenant_region(&tenant_id, &region)?;
        let notify_principal_refs = input
            .notify_principal_refs
            .into_iter()
            .map(ActorRef::new)
            .collect::<Result<Vec<_>, _>>()?;
        if notify_principal_refs.is_empty() {
            return Err(CloudObservabilityError::InvalidActorRef);
        }
        Ok(Self {
            id: internal(AlertId::new(input.id)?),
            tenant_id: internal(tenant_id),
            region: public(region),
            metric_id: internal(MetricId::new(input.metric_id)?),
            severity: internal(input.severity),
            expression_ref: internal(DigestRef::sha256(input.expression_ref)?),
            notify_principal_refs: internal(notify_principal_refs),
            data_class: internal(privacy_class(input.data_class)?),
            schema_version: public(OBSERVABILITY_SCHEMA_VERSION),
        })
    }
}

impl Dashboard {
    pub fn new(
        input: DashboardCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Self, CloudObservabilityError> {
        let tenant_id = validated_tenant(input.tenant_id)?;
        let region = region_for(&input.region, &residency.residency.value)?;
        residency.validate_tenant_region(&tenant_id, &region)?;
        validate_dashboard_title(&input.title)?;
        let metric_ids = ids(input.metric_ids, MetricId::new)?;
        let log_stream_ids = ids(input.log_stream_ids, LogStreamId::new)?;
        let trace_ids = ids(input.trace_ids, TraceId::new)?;
        if metric_ids.is_empty() && log_stream_ids.is_empty() && trace_ids.is_empty() {
            return Err(CloudObservabilityError::UnknownMetric);
        }
        Ok(Self {
            id: internal(DashboardId::new(input.id)?),
            tenant_id: internal(tenant_id),
            region: public(region),
            title: public(input.title),
            visibility: public(input.visibility),
            metric_ids: internal(metric_ids),
            log_stream_ids: internal(log_stream_ids),
            trace_ids: internal(trace_ids),
            data_class: public(privacy_class(input.data_class)?),
            schema_version: public(OBSERVABILITY_SCHEMA_VERSION),
        })
    }
}

impl CloudAuditRecord {
    fn from_chain_event(
        event: &AuditEvent,
        envelope: CloudAuditEnvelopeCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Self, CloudObservabilityError> {
        validate_tenant_id(&event.tenant_id)?;
        let region = region_for(&envelope.region, &residency.residency.value)?;
        residency.validate_tenant_region(&event.tenant_id, &region)?;
        if envelope.event_sequence != event.sequence {
            return Err(CloudObservabilityError::AuditEnvelopeSequenceMismatch);
        }
        if envelope.topic.as_str() != event.surface {
            return Err(CloudObservabilityError::InvalidAuditTopic);
        }
        if envelope.topic.operation() != envelope.operation {
            return Err(CloudObservabilityError::AuditTopicOperationMismatch);
        }
        let cell_id = optional_cell_for(envelope.cell_id, &region)?;
        let source_resource_id =
            optional_resource_for(envelope.resource_id, &event.tenant_id, &region)?;
        let data_classes_referenced = audit_classifications(&event.data_classes)?;
        let occurred_at_epoch_seconds = envelope.occurred_at_epoch_seconds;
        if occurred_at_epoch_seconds == 0 {
            return Err(CloudObservabilityError::InvalidAuditTimestamp);
        }
        let signed_export_uri = SignedAuditExportUri::new(envelope.signed_export_uri)?;
        if signed_export_uri != residency.signed_audit_export_uri.value {
            return Err(CloudObservabilityError::InvalidExportUri);
        }
        Ok(Self {
            id: internal(AuditRecordId::for_event(event)),
            tenant_id: internal(event.tenant_id.clone()),
            region: public(region),
            cell_id: public(cell_id),
            topic: internal(envelope.topic),
            operation: internal(envelope.operation),
            record_class: internal(envelope.topic.record_class()),
            source_resource_id: internal(source_resource_id),
            actor: internal(ActorRef::new(envelope.actor)?),
            iam_role: internal(
                envelope
                    .iam_role
                    .map(IamRoleId::new)
                    .transpose()
                    .map_err(|_| CloudObservabilityError::InvalidRoleId)?,
            ),
            occurred_at_epoch_seconds: internal(occurred_at_epoch_seconds),
            chain_sequence: internal(event.sequence),
            previous_hash: internal(DigestRef::audit_hash(event.previous_hash.clone())?),
            hash: internal(DigestRef::audit_hash(event.hash.clone())?),
            payload_hash: internal(DigestRef::sha256(envelope.payload_hash)?),
            idempotency_key: internal(IdempotencyKey::new(envelope.idempotency_key)?),
            decision: internal(non_empty(
                event.decision.clone(),
                CloudObservabilityError::SourceAuditRejected,
            )?),
            purpose: internal(event.purpose),
            plane: internal(event.plane),
            data_classes_referenced: internal(data_classes_referenced),
            signed_export_uri: internal(signed_export_uri),
            audit_marker: Classified::new(OperationalDataClass::Audit, OperationalDataClass::Audit),
            schema_version: public(AUDIT_RECORD_SCHEMA_VERSION),
        })
    }
}

impl CloudObservabilityCatalog {
    pub fn register_residency(
        &mut self,
        input: ObservabilityResidencyCreate,
    ) -> Result<ObservabilityResidency, CloudObservabilityError> {
        let residency = ObservabilityResidency::new(input)?;
        let key = (
            residency.tenant_id.value.clone(),
            residency.region.value.clone(),
        );
        if self.residencies.insert(key, residency.clone()).is_some() {
            return Err(CloudObservabilityError::InvalidResidency);
        }
        Ok(residency)
    }

    pub fn add_metric(
        &mut self,
        input: MetricCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Metric, CloudObservabilityError> {
        let metric = Metric::new(input, residency)?;
        if self
            .metrics
            .insert(metric.id.value.clone(), metric.clone())
            .is_some()
        {
            return Err(CloudObservabilityError::DuplicateMetric);
        }
        Ok(metric)
    }

    pub fn add_log_stream(
        &mut self,
        input: LogStreamCreate,
        residency: &ObservabilityResidency,
    ) -> Result<LogStream, CloudObservabilityError> {
        let log_stream = LogStream::new(input, residency)?;
        if self
            .log_streams
            .insert(log_stream.id.value.clone(), log_stream.clone())
            .is_some()
        {
            return Err(CloudObservabilityError::DuplicateLogStream);
        }
        Ok(log_stream)
    }

    pub fn add_trace(
        &mut self,
        input: TraceCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Trace, CloudObservabilityError> {
        let trace = Trace::new(input, residency)?;
        if self
            .traces
            .insert(trace.id.value.clone(), trace.clone())
            .is_some()
        {
            return Err(CloudObservabilityError::DuplicateTrace);
        }
        Ok(trace)
    }

    pub fn add_alert(
        &mut self,
        input: AlertCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Alert, CloudObservabilityError> {
        let alert = Alert::new(input, residency)?;
        if !self.metrics.contains_key(&alert.metric_id.value) {
            return Err(CloudObservabilityError::UnknownMetric);
        }
        if self
            .alerts
            .insert(alert.id.value.clone(), alert.clone())
            .is_some()
        {
            return Err(CloudObservabilityError::DuplicateAlert);
        }
        Ok(alert)
    }

    pub fn add_dashboard(
        &mut self,
        input: DashboardCreate,
        residency: &ObservabilityResidency,
    ) -> Result<Dashboard, CloudObservabilityError> {
        let dashboard = Dashboard::new(input, residency)?;
        for metric_id in &dashboard.metric_ids.value {
            if !self.metrics.contains_key(metric_id) {
                return Err(CloudObservabilityError::UnknownMetric);
            }
        }
        for log_stream_id in &dashboard.log_stream_ids.value {
            if !self.log_streams.contains_key(log_stream_id) {
                return Err(CloudObservabilityError::UnknownLogStream);
            }
        }
        for trace_id in &dashboard.trace_ids.value {
            if !self.traces.contains_key(trace_id) {
                return Err(CloudObservabilityError::UnknownTrace);
            }
        }
        if self
            .dashboards
            .insert(dashboard.id.value.clone(), dashboard.clone())
            .is_some()
        {
            return Err(CloudObservabilityError::DuplicateDashboard);
        }
        Ok(dashboard)
    }

    pub fn ingest_verified_chain(
        &mut self,
        chain: &AuditChain,
        envelopes: Vec<CloudAuditEnvelopeCreate>,
        residency: &ObservabilityResidency,
    ) -> Result<Vec<CloudAuditRecord>, CloudObservabilityError> {
        if !chain.verify() {
            return Err(CloudObservabilityError::UnverifiedAuditChain);
        }
        let mut envelopes_by_sequence = BTreeMap::new();
        for envelope in envelopes {
            if envelopes_by_sequence
                .insert(envelope.event_sequence, envelope)
                .is_some()
            {
                return Err(CloudObservabilityError::DuplicateEnvelopeSequence);
            }
        }
        let mut records = Vec::with_capacity(chain.events().len());
        for event in chain.events() {
            let envelope = envelopes_by_sequence
                .remove(&event.sequence)
                .ok_or(CloudObservabilityError::MissingAuditEnvelope)?;
            let record = CloudAuditRecord::from_chain_event(event, envelope, residency)?;
            let sequence_key = (
                record.tenant_id.value.clone(),
                record.region.value.clone(),
                record.chain_sequence.value,
            );
            if self.audit_records.contains_key(&record.id.value)
                || self.audit_sequences.contains(&sequence_key)
            {
                return Err(CloudObservabilityError::DuplicateAuditRecord);
            }
            records.push(record);
        }
        if !envelopes_by_sequence.is_empty() {
            return Err(CloudObservabilityError::AuditChainMismatch);
        }
        for record in &records {
            self.audit_sequences.insert((
                record.tenant_id.value.clone(),
                record.region.value.clone(),
                record.chain_sequence.value,
            ));
            self.high_watermark_sequence = self
                .high_watermark_sequence
                .map(|existing| existing.max(record.chain_sequence.value))
                .or(Some(record.chain_sequence.value));
            self.audit_records
                .insert(record.id.value.clone(), record.clone());
        }
        self.chain_verified = true;
        Ok(records)
    }

    pub fn read_audit(
        &self,
        request: AuditReadRequest,
    ) -> Result<AuditReadResult, CloudObservabilityError> {
        let normalized = NormalizedAuditReadRequest::new(request)?;
        if normalized.require_complete_chain && !self.chain_verified {
            return Err(CloudObservabilityError::IncompleteAuditChain);
        }
        let mut records = self
            .audit_records
            .values()
            .filter(|record| normalized.matches(record))
            .cloned()
            .collect::<Vec<_>>();
        records.sort_by_key(|record| {
            (
                record.occurred_at_epoch_seconds.value,
                record.chain_sequence.value,
                record.id.value.value.clone(),
            )
        });
        if let Some((cursor_time, cursor_sequence)) = normalized.cursor_after {
            records.retain(|record| {
                (
                    record.occurred_at_epoch_seconds.value,
                    record.chain_sequence.value,
                ) > (cursor_time, cursor_sequence)
            });
        }
        let page_size = usize::from(normalized.page_size);
        let next_cursor = if records.len() > page_size {
            records.truncate(page_size);
            records.last().map(|last| {
                AuditReadCursor::from_position(
                    &normalized.tenant_id,
                    &normalized.region,
                    last.occurred_at_epoch_seconds.value,
                    last.chain_sequence.value,
                )
            })
        } else {
            None
        };
        Ok(AuditReadResult {
            records,
            next_cursor,
            chain_complete: self.chain_verified,
            high_watermark_sequence: self.high_watermark_sequence,
        })
    }

    pub fn summarize_audit(
        &self,
        request: AuditReadRequest,
    ) -> Result<AuditReadSummary, CloudObservabilityError> {
        let normalized = NormalizedAuditReadRequest::new(request)?;
        if normalized.require_complete_chain && !self.chain_verified {
            return Err(CloudObservabilityError::IncompleteAuditChain);
        }
        let mut total: u64 = 0;
        let mut per_topic: BTreeMap<CloudAuditTopic, u64> = BTreeMap::new();
        let mut earliest: Option<u64> = None;
        let mut latest: Option<u64> = None;
        for record in self
            .audit_records
            .values()
            .filter(|r| normalized.matches(r))
        {
            total += 1;
            *per_topic.entry(record.topic.value).or_insert(0) += 1;
            let ts = record.occurred_at_epoch_seconds.value;
            earliest = Some(earliest.map_or(ts, |e| e.min(ts)));
            latest = Some(latest.map_or(ts, |l| l.max(ts)));
        }
        Ok(AuditReadSummary {
            total,
            per_topic,
            earliest_epoch_seconds: earliest,
            latest_epoch_seconds: latest,
            chain_complete: self.chain_verified,
            high_watermark_sequence: self.high_watermark_sequence,
        })
    }

    pub fn metrics(&self) -> impl Iterator<Item = &Metric> {
        self.metrics.values()
    }

    pub fn audit_records(&self) -> impl Iterator<Item = &CloudAuditRecord> {
        self.audit_records.values()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NormalizedAuditReadRequest {
    tenant_id: String,
    region: RegionCode,
    cell_id: Option<CellId>,
    scope: AuditReadScope,
    start_epoch_seconds: u64,
    end_epoch_seconds: u64,
    topics: BTreeSet<CloudAuditTopic>,
    actor: Option<ActorRef>,
    resource_id: Option<ResourceId>,
    cursor_after: Option<(u64, u64)>,
    page_size: u16,
    require_complete_chain: bool,
}

impl NormalizedAuditReadRequest {
    fn new(request: AuditReadRequest) -> Result<Self, CloudObservabilityError> {
        let tenant_id = validated_tenant(request.tenant_id)?;
        let region =
            RegionCode::new(request.region).map_err(|_| CloudObservabilityError::InvalidRegion)?;
        if request.start_epoch_seconds >= request.end_epoch_seconds {
            return Err(CloudObservabilityError::InvalidReadWindow);
        }
        if request.end_epoch_seconds - request.start_epoch_seconds > MAX_AUDIT_READ_WINDOW_SECONDS {
            return Err(CloudObservabilityError::InvalidReadWindow);
        }
        let page_size = request.page_size.unwrap_or(DEFAULT_AUDIT_READ_PAGE_SIZE);
        if page_size == 0 || page_size > MAX_AUDIT_READ_PAGE_SIZE {
            return Err(CloudObservabilityError::InvalidPageSize);
        }
        let cell_id = optional_cell_for(request.cell_id, &region)?;
        let topics = request.topics.into_iter().collect::<BTreeSet<_>>();
        if request.scope == AuditReadScope::ControlPlaneMutations
            && topics
                .iter()
                .any(|topic| !topic.is_control_plane_mutation())
        {
            return Err(CloudObservabilityError::InvalidAuditTopic);
        }
        let actor = request.actor.map(ActorRef::new).transpose()?;
        let resource_id = optional_resource_for(request.resource_id, &tenant_id, &region)?;
        let cursor_after = request
            .cursor
            .as_deref()
            .map(|cursor| AuditReadCursor::parse(cursor, &tenant_id, &region))
            .transpose()?;
        Ok(Self {
            tenant_id,
            region,
            cell_id,
            scope: request.scope,
            start_epoch_seconds: request.start_epoch_seconds,
            end_epoch_seconds: request.end_epoch_seconds,
            topics,
            actor,
            resource_id,
            cursor_after,
            page_size,
            require_complete_chain: request.require_complete_chain,
        })
    }

    fn matches(&self, record: &CloudAuditRecord) -> bool {
        if record.tenant_id.value != self.tenant_id || record.region.value != self.region {
            return false;
        }
        if let Some(cell_id) = &self.cell_id
            && record.cell_id.value.as_ref() != Some(cell_id)
        {
            return false;
        }
        if record.occurred_at_epoch_seconds.value < self.start_epoch_seconds
            || record.occurred_at_epoch_seconds.value >= self.end_epoch_seconds
        {
            return false;
        }
        if self.scope == AuditReadScope::ControlPlaneMutations
            && !record.topic.value.is_control_plane_mutation()
        {
            return false;
        }
        if !self.topics.is_empty() && !self.topics.contains(&record.topic.value) {
            return false;
        }
        if let Some(actor) = &self.actor
            && &record.actor.value != actor
        {
            return false;
        }
        if let Some(resource_id) = &self.resource_id
            && record.source_resource_id.value.as_ref() != Some(resource_id)
        {
            return false;
        }
        true
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn prefixed_id(
    value: String,
    prefix: &str,
    error: CloudObservabilityError,
) -> Result<String, CloudObservabilityError> {
    if value.starts_with(prefix) && value.len() > prefix.len() && is_ascii_token(&value) {
        Ok(value)
    } else {
        Err(error)
    }
}

fn validated_tenant(value: String) -> Result<String, CloudObservabilityError> {
    validate_tenant_id(&value)?;
    Ok(value)
}

fn validate_tenant_id(value: &str) -> Result<(), CloudObservabilityError> {
    if value.starts_with(TENANT_ID_PREFIX)
        && value.len() > TENANT_ID_PREFIX.len()
        && is_ascii_token(value)
    {
        Ok(())
    } else {
        Err(CloudObservabilityError::InvalidTenantId)
    }
}

fn validate_regional_pack(value: &str) -> Result<(), CloudObservabilityError> {
    if value.starts_with(REGIONAL_PACK_PREFIX)
        && value.len() > REGIONAL_PACK_PREFIX.len()
        && is_ascii_token(value)
    {
        Ok(())
    } else {
        Err(CloudObservabilityError::InvalidRegionalPack)
    }
}

fn region_for(
    value: &str,
    residency: &ResidencyClass,
) -> Result<RegionCode, CloudObservabilityError> {
    let region =
        RegionCode::new(value.to_string()).map_err(|_| CloudObservabilityError::InvalidRegion)?;
    if residency_class_allows_home_region_label(residency, &region.value) {
        Ok(region)
    } else {
        Err(CloudObservabilityError::InvalidResidency)
    }
}

fn validate_in_region(
    home: &RegionCode,
    storage: &RegionCode,
) -> Result<(), CloudObservabilityError> {
    if home == storage {
        Ok(())
    } else {
        Err(CloudObservabilityError::InvalidResidency)
    }
}

fn optional_cell_for(
    value: Option<String>,
    region: &RegionCode,
) -> Result<Option<CellId>, CloudObservabilityError> {
    value
        .map(|value| {
            let cell_id = CellId::new(value).map_err(|_| CloudObservabilityError::InvalidCellId)?;
            let expected_prefix = format!("cell-{}-", region.value);
            if cell_id.value.starts_with(&expected_prefix) {
                Ok(cell_id)
            } else {
                Err(CloudObservabilityError::InvalidCellId)
            }
        })
        .transpose()
}

fn optional_resource_for(
    value: Option<String>,
    tenant_id: &str,
    region: &RegionCode,
) -> Result<Option<ResourceId>, CloudObservabilityError> {
    value
        .map(|value| {
            let resource = ResourceId::new(value).map_err(map_resource_error)?;
            if resource.tenant_id().map_err(map_resource_error)? != tenant_id {
                return Err(CloudObservabilityError::ResourceTenantMismatch);
            }
            if resource.region().map_err(map_resource_error)? != *region {
                return Err(CloudObservabilityError::ResourceRegionMismatch);
            }
            Ok(resource)
        })
        .transpose()
}

fn map_resource_error(error: CloudResourceError) -> CloudObservabilityError {
    match error {
        CloudResourceError::InvalidResourceId => CloudObservabilityError::InvalidResourceId,
        CloudResourceError::ResourceIdTenantMismatch => {
            CloudObservabilityError::ResourceTenantMismatch
        }
        CloudResourceError::ResourceIdRegionMismatch => {
            CloudObservabilityError::ResourceRegionMismatch
        }
        _ => CloudObservabilityError::InvalidResourceId,
    }
}

fn privacy_class(data_class: DataClass) -> Result<PrivacyDataClass, CloudObservabilityError> {
    PrivacyDataClass::new(data_class).map_err(|_| CloudObservabilityError::InvalidTelemetryClass)
}

fn telemetry_privacy_classes(
    input: Vec<DataClass>,
) -> Result<Vec<PrivacyDataClass>, CloudObservabilityError> {
    if input.is_empty() {
        return Err(CloudObservabilityError::InvalidTelemetryClass);
    }
    let mut seen = BTreeSet::new();
    let mut output = Vec::with_capacity(input.len());
    for data_class in input {
        let classification = DataClassification::from(data_class);
        if matches!(
            log_exposure_for_classification(classification),
            TelemetryLogExposure::Forbid
        ) {
            return Err(CloudObservabilityError::ForbiddenTelemetryClass);
        }
        let data_class = privacy_class(data_class)?;
        if seen.insert(data_class) {
            output.push(data_class);
        }
    }
    Ok(output)
}

fn audit_classifications(
    input: &[DataClass],
) -> Result<Vec<DataClassification>, CloudObservabilityError> {
    if input.is_empty() {
        return Err(CloudObservabilityError::InvalidTelemetryClass);
    }
    let mut seen = BTreeSet::new();
    let mut output = Vec::with_capacity(input.len());
    for data_class in input {
        let classification = DataClassification::from(*data_class);
        if matches!(
            classification,
            DataClassification::Operational(OperationalDataClass::Secret)
        ) {
            return Err(CloudObservabilityError::ForbiddenTelemetryClass);
        }
        if seen.insert(classification) {
            output.push(classification);
        }
    }
    Ok(output)
}

fn validate_metric_name(value: &str) -> Result<(), CloudObservabilityError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if valid {
        Ok(())
    } else {
        Err(CloudObservabilityError::InvalidMetricName)
    }
}

fn validate_ascii_label(
    value: &str,
    error: CloudObservabilityError,
) -> Result<(), CloudObservabilityError> {
    if !value.trim().is_empty() && value.len() <= 64 && is_ascii_token_with_slash(value) {
        Ok(())
    } else {
        Err(error)
    }
}

fn validate_dashboard_title(value: &str) -> Result<(), CloudObservabilityError> {
    if !value.trim().is_empty() && value.len() <= 120 && !value.contains('\n') {
        Ok(())
    } else {
        Err(CloudObservabilityError::InvalidDashboardTitle)
    }
}

fn validate_sampling_mode(value: TraceSamplingMode) -> Result<(), CloudObservabilityError> {
    match value {
        TraceSamplingMode::RatioPermille(value) if value > 1000 => {
            Err(CloudObservabilityError::InvalidTelemetryClass)
        }
        _ => Ok(()),
    }
}

fn validate_retention(value: u16, minimum: u16) -> Result<(), CloudObservabilityError> {
    if value >= minimum {
        Ok(())
    } else {
        Err(CloudObservabilityError::InvalidRetention)
    }
}

fn ids<T, F>(values: Vec<String>, parse: F) -> Result<Vec<T>, CloudObservabilityError>
where
    T: Clone + Ord,
    F: Fn(String) -> Result<T, CloudObservabilityError>,
{
    let mut seen = BTreeSet::new();
    let mut ids = Vec::with_capacity(values.len());
    for value in values {
        let id = parse(value)?;
        if seen.insert(id.clone()) {
            ids.push(id);
        }
    }
    Ok(ids)
}

fn non_empty(
    value: String,
    error: CloudObservabilityError,
) -> Result<String, CloudObservabilityError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

fn valid_sha256_ref(value: &str) -> bool {
    value
        .strip_prefix(SHA256_PREFIX)
        .is_some_and(|rest| rest.len() == 64 && rest.chars().all(|c| c.is_ascii_hexdigit()))
}

fn valid_fnv_hash_ref(value: &str) -> bool {
    value
        .strip_prefix(FNV_HASH_PREFIX)
        .is_some_and(|rest| rest.len() == 16 && rest.chars().all(|c| c.is_ascii_hexdigit()))
}

fn is_ascii_token(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':'))
}

fn is_ascii_token_with_slash(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '/'))
}

#[cfg(test)]
mod tests {
    use audit_chain_domain::Plane;
    use network_residency::{
        PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
    };
    use oya_data_boundary_kernel::{OperationalDataClass, Purpose};

    use super::*;

    const TENANT: &str = "ten_alpha";
    const REGION: &str = "region-alpha1";
    const CELL: &str = "cell-region-alpha1-a-001";
    const SIGNED_EXPORT: &str = "s3+signed://region-alpha1/ten_alpha/audit?sig=abc123";
    const RESOURCE_ID: &str = "oya:cloud:region-alpha1:ten_alpha:instance:vm-a";
    const HASH_A: &str = "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
    const HASH_B: &str = "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

    fn residency_class() -> ResidencyClass {
        ResidencyClass::PerPack(Box::new(
            PerPackResidency::new(PerPackResidencyCreate {
                allowed_primary_regions: vec!["region-alpha1".to_string()],
                allowed_replica_regions: vec!["region-beta1".to_string()],
                forbidden_regions: vec!["region-gamma1".to_string()],
                regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                    regulator_refs: vec!["regulator/cloud-observability".to_string()],
                    evidence_ref: "evidence/residency/cloud-observability".to_string(),
                })
                .expect("regulator overlay fixture is valid"),
            })
            .expect("per-pack residency fixture is valid"),
        ))
    }

    fn residency() -> ObservabilityResidency {
        ObservabilityResidency::new(ObservabilityResidencyCreate {
            tenant_id: TENANT.to_string(),
            region: REGION.to_string(),
            regional_pack: "oya-pack-alpha".to_string(),
            residency: residency_class(),
            metric_storage_region: REGION.to_string(),
            log_storage_region: REGION.to_string(),
            trace_storage_region: REGION.to_string(),
            audit_storage_region: REGION.to_string(),
            signed_audit_export_uri: SIGNED_EXPORT.to_string(),
            retention_days: 2555,
            state: ObservabilityResidencyState::Enforcing,
        })
        .expect("valid residency")
    }

    fn chain() -> AuditChain {
        let mut chain = AuditChain::default();
        chain
            .append_classifications(
                TENANT,
                CloudAuditTopic::CloudResourceCreated.as_str(),
                Plane::Control,
                Purpose::CoreService,
                [DataClass::InternalOnly, DataClass::Public, DataClass::Audit],
                "ALLOW",
            )
            .expect("test fixture: append CloudResourceCreated must succeed for valid tenant");
        chain
            .append_classifications(
                TENANT,
                CloudAuditTopic::CloudIamPolicy.as_str(),
                Plane::Control,
                Purpose::CoreService,
                [DataClass::InternalOnly, DataClass::Audit],
                "ALLOW",
            )
            .expect("test fixture: append CloudIamPolicy must succeed for valid tenant");
        chain
            .append_classifications(
                TENANT,
                CloudAuditTopic::CloudKmsUse.as_str(),
                Plane::Data,
                Purpose::CoreService,
                [DataClass::InternalOnly, DataClass::Audit],
                "ALLOW",
            )
            .expect("test fixture: append CloudKmsUse must succeed for valid tenant");
        assert!(chain.verify());
        chain
    }

    fn envelopes() -> Vec<CloudAuditEnvelopeCreate> {
        vec![
            CloudAuditEnvelopeCreate {
                event_sequence: 0,
                topic: CloudAuditTopic::CloudResourceCreated,
                operation: CloudAuditOperation::ResourceCreated,
                region: REGION.to_string(),
                cell_id: Some(CELL.to_string()),
                resource_id: Some(RESOURCE_ID.to_string()),
                actor: "usr_admin".to_string(),
                iam_role: Some("role_cloud_admin".to_string()),
                occurred_at_epoch_seconds: 1_000,
                payload_hash: HASH_A.to_string(),
                idempotency_key: "idem/create-vm-a".to_string(),
                signed_export_uri: SIGNED_EXPORT.to_string(),
            },
            CloudAuditEnvelopeCreate {
                event_sequence: 1,
                topic: CloudAuditTopic::CloudIamPolicy,
                operation: CloudAuditOperation::IamPolicyChanged,
                region: REGION.to_string(),
                cell_id: Some(CELL.to_string()),
                resource_id: None,
                actor: "sp_foundry".to_string(),
                iam_role: Some("role_cloud_admin".to_string()),
                occurred_at_epoch_seconds: 1_010,
                payload_hash: HASH_B.to_string(),
                idempotency_key: "idem/iam-policy".to_string(),
                signed_export_uri: SIGNED_EXPORT.to_string(),
            },
            CloudAuditEnvelopeCreate {
                event_sequence: 2,
                topic: CloudAuditTopic::CloudKmsUse,
                operation: CloudAuditOperation::KmsKeyUsed,
                region: REGION.to_string(),
                cell_id: Some(CELL.to_string()),
                resource_id: None,
                actor: "role_cloud_admin".to_string(),
                iam_role: Some("role_cloud_admin".to_string()),
                occurred_at_epoch_seconds: 1_020,
                payload_hash: HASH_A.to_string(),
                idempotency_key: "idem/kms-use".to_string(),
                signed_export_uri: SIGNED_EXPORT.to_string(),
            },
        ]
    }

    fn metric_create() -> MetricCreate {
        MetricCreate {
            id: "metric_cpu".to_string(),
            tenant_id: TENANT.to_string(),
            region: REGION.to_string(),
            cell_id: Some(CELL.to_string()),
            name: "compute.cpu.utilization".to_string(),
            kind: MetricKind::Gauge,
            unit: "percent".to_string(),
            source_resource_id: Some(RESOURCE_ID.to_string()),
            data_classes: vec![DataClass::InternalOnly, DataClass::Usage],
            retention_days: 400,
        }
    }

    #[test]
    fn residency_requires_every_observability_store_in_home_region() {
        let valid = residency();
        assert_eq!(valid.storage_region_for(TelemetryKind::Audit).value, REGION);

        let err = ObservabilityResidency::new(ObservabilityResidencyCreate {
            trace_storage_region: "region-gamma1".to_string(),
            ..ObservabilityResidencyCreate {
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                regional_pack: "oya-pack-alpha".to_string(),
                residency: residency_class(),
                metric_storage_region: REGION.to_string(),
                log_storage_region: REGION.to_string(),
                trace_storage_region: REGION.to_string(),
                audit_storage_region: REGION.to_string(),
                signed_audit_export_uri: SIGNED_EXPORT.to_string(),
                retention_days: 2555,
                state: ObservabilityResidencyState::Enforcing,
            }
        })
        .unwrap_err();
        assert_eq!(err, CloudObservabilityError::InvalidResidency);
    }

    #[test]
    fn ingests_verified_chain_and_reads_control_plane_audit_with_cursor() {
        let residency = residency();
        let mut catalog = CloudObservabilityCatalog::default();
        let records = catalog
            .ingest_verified_chain(&chain(), envelopes(), &residency)
            .expect("verified audit projection");
        assert_eq!(records.len(), 3);
        assert_eq!(
            records[0].record_class.value,
            AuditRecordClass::ControlPlaneMutation
        );
        assert_eq!(
            records[0].audit_marker.data_class,
            DataClassification::Operational(OperationalDataClass::Audit)
        );

        let first_page = catalog
            .read_audit(AuditReadRequest {
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: Some(CELL.to_string()),
                scope: AuditReadScope::ControlPlaneMutations,
                start_epoch_seconds: 900,
                end_epoch_seconds: 1_100,
                topics: Vec::new(),
                actor: None,
                resource_id: None,
                cursor: None,
                page_size: Some(1),
                require_complete_chain: true,
            })
            .expect("read first page");
        assert_eq!(first_page.records.len(), 1);
        assert_eq!(
            first_page.records[0].operation.value,
            CloudAuditOperation::ResourceCreated
        );
        assert!(first_page.next_cursor.is_some());
        assert!(first_page.chain_complete);

        let second_page = catalog
            .read_audit(AuditReadRequest {
                cursor: first_page.next_cursor.map(|cursor| cursor.value),
                page_size: Some(10),
                ..AuditReadRequest {
                    tenant_id: TENANT.to_string(),
                    region: REGION.to_string(),
                    cell_id: Some(CELL.to_string()),
                    scope: AuditReadScope::ControlPlaneMutations,
                    start_epoch_seconds: 900,
                    end_epoch_seconds: 1_100,
                    topics: Vec::new(),
                    actor: None,
                    resource_id: None,
                    cursor: None,
                    page_size: None,
                    require_complete_chain: true,
                }
            })
            .expect("read second page");
        assert_eq!(second_page.records.len(), 1);
        assert_eq!(
            second_page.records[0].operation.value,
            CloudAuditOperation::IamPolicyChanged
        );
    }

    #[test]
    fn control_plane_read_excludes_data_plane_security_events() {
        let residency = residency();
        let mut catalog = CloudObservabilityCatalog::default();
        catalog
            .ingest_verified_chain(&chain(), envelopes(), &residency)
            .expect("ingest");

        let control = catalog
            .read_audit(AuditReadRequest {
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: None,
                scope: AuditReadScope::ControlPlaneMutations,
                start_epoch_seconds: 900,
                end_epoch_seconds: 1_100,
                topics: Vec::new(),
                actor: None,
                resource_id: None,
                cursor: None,
                page_size: Some(100),
                require_complete_chain: true,
            })
            .expect("control read");
        assert_eq!(control.records.len(), 2);
        assert!(
            control
                .records
                .iter()
                .all(|record| record.topic.value.is_control_plane_mutation())
        );

        let err = catalog
            .read_audit(AuditReadRequest {
                topics: vec![CloudAuditTopic::CloudKmsUse],
                ..AuditReadRequest {
                    tenant_id: TENANT.to_string(),
                    region: REGION.to_string(),
                    cell_id: None,
                    scope: AuditReadScope::ControlPlaneMutations,
                    start_epoch_seconds: 900,
                    end_epoch_seconds: 1_100,
                    topics: Vec::new(),
                    actor: None,
                    resource_id: None,
                    cursor: None,
                    page_size: Some(100),
                    require_complete_chain: true,
                }
            })
            .unwrap_err();
        assert_eq!(err, CloudObservabilityError::InvalidAuditTopic);
    }

    #[test]
    fn rejects_forged_audit_envelope_topic_sequence_resource_and_export() {
        let residency = residency();
        let mut wrong_sequence = envelopes();
        wrong_sequence[0].event_sequence = 99;
        assert_eq!(
            CloudObservabilityCatalog::default()
                .ingest_verified_chain(&chain(), wrong_sequence, &residency)
                .unwrap_err(),
            CloudObservabilityError::MissingAuditEnvelope
        );

        let mut wrong_topic = envelopes();
        wrong_topic[0].topic = CloudAuditTopic::CloudIamPolicy;
        assert_eq!(
            CloudObservabilityCatalog::default()
                .ingest_verified_chain(&chain(), wrong_topic, &residency)
                .unwrap_err(),
            CloudObservabilityError::InvalidAuditTopic
        );

        let mut wrong_resource = envelopes();
        wrong_resource[0].resource_id =
            Some("oya:cloud:region-alpha1:ten_other:instance:vm-a".to_string());
        assert_eq!(
            CloudObservabilityCatalog::default()
                .ingest_verified_chain(&chain(), wrong_resource, &residency)
                .unwrap_err(),
            CloudObservabilityError::ResourceTenantMismatch
        );

        let mut wrong_export = envelopes();
        wrong_export[0].signed_export_uri =
            "s3+signed://region-alpha1/ten_alpha/audit?sig=other".to_string();
        assert_eq!(
            CloudObservabilityCatalog::default()
                .ingest_verified_chain(&chain(), wrong_export, &residency)
                .unwrap_err(),
            CloudObservabilityError::InvalidExportUri
        );
    }

    #[test]
    fn rejects_duplicate_audit_records_and_cursor_tenant_crossing() {
        let residency = residency();
        let mut catalog = CloudObservabilityCatalog::default();
        catalog
            .ingest_verified_chain(&chain(), envelopes(), &residency)
            .expect("ingest");
        assert_eq!(
            catalog
                .ingest_verified_chain(&chain(), envelopes(), &residency)
                .unwrap_err(),
            CloudObservabilityError::DuplicateAuditRecord
        );

        let cursor = AuditReadCursor::from_position(
            "ten_other",
            &RegionCode::new(REGION).expect("region"),
            1_000,
            0,
        );
        assert_eq!(
            catalog
                .read_audit(AuditReadRequest {
                    tenant_id: TENANT.to_string(),
                    region: REGION.to_string(),
                    cell_id: None,
                    scope: AuditReadScope::AllTenantAudit,
                    start_epoch_seconds: 900,
                    end_epoch_seconds: 1_100,
                    topics: Vec::new(),
                    actor: None,
                    resource_id: None,
                    cursor: Some(cursor.value),
                    page_size: Some(100),
                    require_complete_chain: true,
                })
                .unwrap_err(),
            CloudObservabilityError::CursorTenantMismatch
        );
    }

    #[test]
    fn creates_metric_log_trace_alert_and_dashboard_with_catalog_refs() {
        let residency = residency();
        let mut catalog = CloudObservabilityCatalog::default();
        let metric = catalog
            .add_metric(metric_create(), &residency)
            .expect("metric");
        let log_stream = catalog
            .add_log_stream(
                LogStreamCreate {
                    id: "log_control".to_string(),
                    tenant_id: TENANT.to_string(),
                    region: REGION.to_string(),
                    cell_id: Some(CELL.to_string()),
                    kind: LogStreamKind::ControlPlane,
                    source_resource_id: Some(RESOURCE_ID.to_string()),
                    data_classes: vec![DataClass::InternalOnly],
                    retention_days: 400,
                },
                &residency,
            )
            .expect("log stream");
        let trace = catalog
            .add_trace(
                TraceCreate {
                    id: "trace_api".to_string(),
                    tenant_id: TENANT.to_string(),
                    region: REGION.to_string(),
                    cell_id: Some(CELL.to_string()),
                    sampling_mode: TraceSamplingMode::RatioPermille(100),
                    root_resource_id: Some(RESOURCE_ID.to_string()),
                    data_classes: vec![DataClass::InternalOnly],
                    retention_days: 400,
                },
                &residency,
            )
            .expect("trace");
        let alert = catalog
            .add_alert(
                AlertCreate {
                    id: "alert_cpu".to_string(),
                    tenant_id: TENANT.to_string(),
                    region: REGION.to_string(),
                    metric_id: metric.id.value.value.clone(),
                    severity: AlertSeverity::Critical,
                    expression_ref: HASH_A.to_string(),
                    notify_principal_refs: vec!["usr_oncall".to_string()],
                    data_class: DataClass::InternalOnly,
                },
                &residency,
            )
            .expect("alert");
        let dashboard = catalog
            .add_dashboard(
                DashboardCreate {
                    id: "dash_ops".to_string(),
                    tenant_id: TENANT.to_string(),
                    region: REGION.to_string(),
                    title: "Alpha Operations".to_string(),
                    visibility: DashboardVisibility::TenantPrivate,
                    metric_ids: vec![metric.id.value.value.clone()],
                    log_stream_ids: vec![log_stream.id.value.value.clone()],
                    trace_ids: vec![trace.id.value.value.clone()],
                    data_class: DataClass::InternalOnly,
                },
                &residency,
            )
            .expect("dashboard");

        assert_eq!(alert.metric_id.value, metric.id.value);
        assert_eq!(dashboard.metric_ids.value, vec![metric.id.value]);
        assert_eq!(dashboard.log_stream_ids.value, vec![log_stream.id.value]);
        assert_eq!(dashboard.trace_ids.value, vec![trace.id.value]);
    }

    #[test]
    fn rejects_forbidden_telemetry_classes_and_invalid_read_windows() {
        let residency = residency();
        assert_eq!(
            Metric::new(
                MetricCreate {
                    data_classes: vec![DataClass::Secret],
                    ..metric_create()
                },
                &residency,
            )
            .unwrap_err(),
            CloudObservabilityError::ForbiddenTelemetryClass
        );

        let mut secret_chain = AuditChain::default();
        secret_chain
            .append_classifications(
                TENANT,
                CloudAuditTopic::CloudResourceCreated.as_str(),
                Plane::Control,
                Purpose::CoreService,
                [DataClass::Secret],
                "ALLOW",
            )
            .expect("test fixture: append secret_chain must succeed for valid tenant");
        assert_eq!(
            CloudObservabilityCatalog::default()
                .ingest_verified_chain(&secret_chain, vec![envelopes().remove(0)], &residency)
                .unwrap_err(),
            CloudObservabilityError::ForbiddenTelemetryClass
        );

        assert_eq!(
            CloudObservabilityCatalog::default()
                .read_audit(AuditReadRequest {
                    tenant_id: TENANT.to_string(),
                    region: REGION.to_string(),
                    cell_id: None,
                    scope: AuditReadScope::AllTenantAudit,
                    start_epoch_seconds: 2_000,
                    end_epoch_seconds: 1_000,
                    topics: Vec::new(),
                    actor: None,
                    resource_id: None,
                    cursor: None,
                    page_size: Some(100),
                    require_complete_chain: false,
                })
                .unwrap_err(),
            CloudObservabilityError::InvalidReadWindow
        );
    }

    #[test]
    fn summarize_audit_empty_catalog_returns_zero_totals() {
        // No records ingested — every field is zero/None.
        let catalog = CloudObservabilityCatalog::default();
        let summary = catalog
            .summarize_audit(AuditReadRequest {
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: None,
                scope: AuditReadScope::AllTenantAudit,
                start_epoch_seconds: 1_000,
                end_epoch_seconds: 1_000 + MAX_AUDIT_READ_WINDOW_SECONDS,
                topics: Vec::new(),
                actor: None,
                resource_id: None,
                cursor: None,
                page_size: None,
                require_complete_chain: false,
            })
            .expect("summarize on empty catalog");
        assert_eq!(summary.total, 0);
        assert!(summary.per_topic.is_empty());
        assert_eq!(summary.earliest_epoch_seconds, None);
        assert_eq!(summary.latest_epoch_seconds, None);
        assert!(!summary.chain_complete);
        assert_eq!(summary.high_watermark_sequence, None);
    }

    #[test]
    fn summarize_audit_multi_topic_aggregates_correctly() {
        // chain() has: CloudResourceCreated(t=1000), CloudIamPolicy(t=1010), CloudKmsUse(t=1020)
        let residency = residency();
        let mut catalog = CloudObservabilityCatalog::default();
        catalog
            .ingest_verified_chain(&chain(), envelopes(), &residency)
            .expect("ingest");

        let summary = catalog
            .summarize_audit(AuditReadRequest {
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: None,
                scope: AuditReadScope::AllTenantAudit,
                start_epoch_seconds: 900,
                end_epoch_seconds: 1_100,
                topics: Vec::new(),
                actor: None,
                resource_id: None,
                cursor: None,
                page_size: None,
                require_complete_chain: false,
            })
            .expect("summarize all-tenant");

        assert_eq!(summary.total, 3);
        assert_eq!(summary.per_topic.values().sum::<u64>(), 3);
        assert_eq!(summary.per_topic[&CloudAuditTopic::CloudResourceCreated], 1);
        assert_eq!(summary.per_topic[&CloudAuditTopic::CloudIamPolicy], 1);
        assert_eq!(summary.per_topic[&CloudAuditTopic::CloudKmsUse], 1);
        assert_eq!(summary.earliest_epoch_seconds, Some(1_000));
        assert_eq!(summary.latest_epoch_seconds, Some(1_020));
        assert!(summary.chain_complete);
    }

    #[test]
    fn summarize_audit_scope_control_plane_excludes_data_plane_security() {
        // scope=ControlPlaneMutations must exclude CloudKmsUse (DataPlaneSecurity).
        let residency = residency();
        let mut catalog = CloudObservabilityCatalog::default();
        catalog
            .ingest_verified_chain(&chain(), envelopes(), &residency)
            .expect("ingest");

        let summary = catalog
            .summarize_audit(AuditReadRequest {
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: None,
                scope: AuditReadScope::ControlPlaneMutations,
                start_epoch_seconds: 900,
                end_epoch_seconds: 1_100,
                topics: Vec::new(),
                actor: None,
                resource_id: None,
                cursor: None,
                page_size: None,
                require_complete_chain: false,
            })
            .expect("summarize control-plane");

        assert_eq!(summary.total, 2);
        assert_eq!(summary.per_topic.values().sum::<u64>(), 2);
        assert!(
            !summary
                .per_topic
                .contains_key(&CloudAuditTopic::CloudKmsUse)
        );
        assert!(
            summary
                .per_topic
                .keys()
                .all(|t| t.is_control_plane_mutation())
        );
        assert_eq!(summary.earliest_epoch_seconds, Some(1_000));
        assert_eq!(summary.latest_epoch_seconds, Some(1_010));
    }

    #[test]
    fn summarize_audit_incomplete_chain_rejected() {
        // require_complete_chain on a catalog where no chain has been ingested → IncompleteAuditChain.
        let catalog = CloudObservabilityCatalog::default();
        let err = catalog
            .summarize_audit(AuditReadRequest {
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: None,
                scope: AuditReadScope::AllTenantAudit,
                start_epoch_seconds: 900,
                end_epoch_seconds: 1_100,
                topics: Vec::new(),
                actor: None,
                resource_id: None,
                cursor: None,
                page_size: None,
                require_complete_chain: true,
            })
            .unwrap_err();
        assert_eq!(err, CloudObservabilityError::IncompleteAuditChain);
    }

    #[test]
    fn summarize_audit_invalid_window_rejected() {
        let catalog = CloudObservabilityCatalog::default();
        let err = catalog
            .summarize_audit(AuditReadRequest {
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: None,
                scope: AuditReadScope::AllTenantAudit,
                start_epoch_seconds: 2_000,
                end_epoch_seconds: 1_000,
                topics: Vec::new(),
                actor: None,
                resource_id: None,
                cursor: None,
                page_size: None,
                require_complete_chain: false,
            })
            .unwrap_err();
        assert_eq!(err, CloudObservabilityError::InvalidReadWindow);
    }

    #[test]
    fn summarize_audit_invalid_topic_for_scope_rejected() {
        // ControlPlaneMutations scope + non-control-plane topic → InvalidAuditTopic.
        let catalog = CloudObservabilityCatalog::default();
        let err = catalog
            .summarize_audit(AuditReadRequest {
                tenant_id: TENANT.to_string(),
                region: REGION.to_string(),
                cell_id: None,
                scope: AuditReadScope::ControlPlaneMutations,
                start_epoch_seconds: 900,
                end_epoch_seconds: 1_100,
                topics: vec![CloudAuditTopic::CloudKmsUse],
                actor: None,
                resource_id: None,
                cursor: None,
                page_size: None,
                require_complete_chain: false,
            })
            .unwrap_err();
        assert_eq!(err, CloudObservabilityError::InvalidAuditTopic);
    }

    #[test]
    fn rejects_unknown_catalog_refs_and_duplicates() {
        let residency = residency();
        let mut catalog = CloudObservabilityCatalog::default();
        let metric = catalog
            .add_metric(metric_create(), &residency)
            .expect("metric");
        assert_eq!(
            catalog.add_metric(metric_create(), &residency).unwrap_err(),
            CloudObservabilityError::DuplicateMetric
        );
        assert_eq!(
            catalog
                .add_alert(
                    AlertCreate {
                        id: "alert_unknown".to_string(),
                        tenant_id: TENANT.to_string(),
                        region: REGION.to_string(),
                        metric_id: "metric_missing".to_string(),
                        severity: AlertSeverity::Critical,
                        expression_ref: HASH_A.to_string(),
                        notify_principal_refs: vec!["usr_oncall".to_string()],
                        data_class: DataClass::InternalOnly,
                    },
                    &residency,
                )
                .unwrap_err(),
            CloudObservabilityError::UnknownMetric
        );
        assert_eq!(metric.name.value, "compute.cpu.utilization");
    }
}
