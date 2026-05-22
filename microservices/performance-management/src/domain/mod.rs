use crate::error::{ServiceError, ServiceResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ArchitectureLayer {
    Kernel,
    Domain,
    Usecase,
    App,
    Adapter,
    Infrastructure,
    Cli,
    Rest,
    Grpc,
    Graphql,
    Worker,
    Sdk,
    Api,
}

impl ArchitectureLayer {
    pub const fn all() -> [Self; 13] {
        [
            Self::Kernel,
            Self::Domain,
            Self::Usecase,
            Self::App,
            Self::Adapter,
            Self::Infrastructure,
            Self::Cli,
            Self::Rest,
            Self::Grpc,
            Self::Graphql,
            Self::Worker,
            Self::Sdk,
            Self::Api,
        ]
    }

    pub const fn slug(&self) -> &'static str {
        match self {
            Self::Kernel => "kernel",
            Self::Domain => "domain",
            Self::Usecase => "usecase",
            Self::App => "app",
            Self::Adapter => "adapter",
            Self::Infrastructure => "infrastructure",
            Self::Cli => "cli",
            Self::Rest => "rest",
            Self::Grpc => "grpc",
            Self::Graphql => "graphql",
            Self::Worker => "worker",
            Self::Sdk => "sdk",
            Self::Api => "api",
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TenantId(String);

impl TenantId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("tenant_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ReviewCycleId(String);

impl ReviewCycleId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("review_cycle_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EmployeeId(String);

impl EmployeeId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("employee_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CalibrationCohortId(String);

impl CalibrationCohortId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("calibration_cohort_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ReviewCycleStatus {
    Draft,
    Open,
    FeedbackLocked,
    Calibrating,
    Calibrated,
    Sealed,
    Cancelled,
}

impl ReviewCycleStatus {
    pub const fn allows_feedback(&self) -> bool {
        matches!(self, Self::Open)
    }

    pub const fn allows_calibration(&self) -> bool {
        matches!(self, Self::FeedbackLocked | Self::Calibrating)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum RatingBand {
    Exceptional,
    Exceeds,
    Meets,
    GrowthNeeded,
    AtRisk,
}

impl RatingBand {
    pub const fn sort_weight(&self) -> u8 {
        match self {
            Self::Exceptional => 5,
            Self::Exceeds => 4,
            Self::Meets => 3,
            Self::GrowthNeeded => 2,
            Self::AtRisk => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum FeedbackVisibility {
    EmployeeVisible,
    ManagerOnly,
    CalibrationPanel,
    HrRestricted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DataClass {
    WorkforceConfidential,
    ManagerPrivate,
    AggregateAnalytics,
    AuditEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AuditEventKind {
    ReviewCycleOpened,
    FeedbackSubmitted,
    RatingChanged,
    CalibrationCompleted,
    EvidenceSealed,
    LaborOverlayExported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Capability {
    GoalCycleOpen,
    ManagerFeedbackGate,
    CalibrationRun,
    ReviewEvidenceSeal,
    LaborOverlayExport,
    EngagementPulse,
}

impl Capability {
    pub const fn action_slug(&self) -> &'static str {
        match self {
            Self::GoalCycleOpen => "performance.goal_cycle.open",
            Self::ManagerFeedbackGate => "performance.feedback.submit",
            Self::CalibrationRun => "performance.calibration.run",
            Self::ReviewEvidenceSeal => "performance.evidence.seal",
            Self::LaborOverlayExport => "performance.labor_overlay.export",
            Self::EngagementPulse => "performance.engagement_pulse.emit",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CapabilityContract {
    pub capability: Capability,
    pub command_name: &'static str,
    pub result_name: &'static str,
    pub required_layer: ArchitectureLayer,
    pub data_class: DataClass,
    pub audit_event: AuditEventKind,
    pub idempotency_key: &'static str,
}

pub const CAPABILITY_CONTRACTS: &[CapabilityContract] = &[
    CapabilityContract {
        capability: Capability::GoalCycleOpen,
        command_name: "OpenReviewCycleCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Usecase,
        data_class: DataClass::WorkforceConfidential,
        audit_event: AuditEventKind::ReviewCycleOpened,
        idempotency_key: "tenant_id + review_cycle_id",
    },
    CapabilityContract {
        capability: Capability::ManagerFeedbackGate,
        command_name: "SubmitFeedbackCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Rest,
        data_class: DataClass::ManagerPrivate,
        audit_event: AuditEventKind::FeedbackSubmitted,
        idempotency_key: "tenant_id + review_cycle_id + author_employee_id",
    },
    CapabilityContract {
        capability: Capability::CalibrationRun,
        command_name: "RunCalibrationCommand",
        result_name: "CalibrationRun",
        required_layer: ArchitectureLayer::Worker,
        data_class: DataClass::AggregateAnalytics,
        audit_event: AuditEventKind::CalibrationCompleted,
        idempotency_key: "tenant_id + review_cycle_id + cohort_id",
    },
    CapabilityContract {
        capability: Capability::ReviewEvidenceSeal,
        command_name: "SealReviewEvidenceCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Api,
        data_class: DataClass::AuditEvidence,
        audit_event: AuditEventKind::EvidenceSealed,
        idempotency_key: "tenant_id + review_cycle_id + sealed_by",
    },
    CapabilityContract {
        capability: Capability::LaborOverlayExport,
        command_name: "ExportLaborOverlayCommand",
        result_name: "LaborOverlayExportReceipt",
        required_layer: ArchitectureLayer::Adapter,
        data_class: DataClass::AuditEvidence,
        audit_event: AuditEventKind::LaborOverlayExported,
        idempotency_key: "tenant_id + review_cycle_id + export_window",
    },
    CapabilityContract {
        capability: Capability::EngagementPulse,
        command_name: "EmitEngagementPulseCommand",
        result_name: "EngagementPulseReceipt",
        required_layer: ArchitectureLayer::Worker,
        data_class: DataClass::AggregateAnalytics,
        audit_event: AuditEventKind::RatingChanged,
        idempotency_key: "tenant_id + pulse_window + cohort_id",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AdapterSurface {
    pub layer: ArchitectureLayer,
    pub protocol: &'static str,
    pub handler: &'static str,
    pub contract_path: &'static str,
}

pub const ADAPTER_SURFACES: &[AdapterSurface] = &[
    AdapterSurface {
        layer: ArchitectureLayer::Rest,
        protocol: "HTTP",
        handler: "PerformanceHttpHandler::open_review_cycle",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1review-cycles",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Rest,
        protocol: "HTTP",
        handler: "PerformanceHttpHandler::submit_feedback",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1review-cycles~1{id}~1feedback",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Grpc,
        protocol: "gRPC",
        handler: "PerformanceGrpcHandler::open_review_cycle",
        contract_path: "contracts/performance-management-v1.proto",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "PerformanceAsyncApiHandler::review_cycle_opened",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/review_cycle_opened",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "PerformanceAsyncApiHandler::feedback_submitted",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/feedback_submitted",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Worker,
        protocol: "AsyncAPI",
        handler: "PerformanceAsyncApiHandler::calibration_completed",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/calibration_completed",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OperationalCheckpoint {
    pub name: &'static str,
    pub invariant: &'static str,
    pub expected_evidence: &'static str,
}

pub const OPERATIONAL_CHECKPOINTS: &[OperationalCheckpoint] = &[
    OperationalCheckpoint {
        name: "cycle-open-policy",
        invariant: "policy_checked",
        expected_evidence: "Cedar decision for performance.goal_cycle.open",
    },
    OperationalCheckpoint {
        name: "feedback-author-subject-split",
        invariant: "feedback_author_subject_separation",
        expected_evidence: "distinct employee ids in ReviewEvidence",
    },
    OperationalCheckpoint {
        name: "calibration-participant-floor",
        invariant: "calibration_participant_count",
        expected_evidence: "participant_count > 0 before distribution lock",
    },
    OperationalCheckpoint {
        name: "evidence-before-seal",
        invariant: "evidence_before_seal",
        expected_evidence: "review cycle evidence_count > 0",
    },
    OperationalCheckpoint {
        name: "labor-overlay-residency",
        invariant: "region_bound",
        expected_evidence: "residency pack id on export request",
    },
    OperationalCheckpoint {
        name: "audit-chain-emission",
        invariant: "audit_emitted",
        expected_evidence: "EVT-PERFORMANCE-MANAGEMENT-*",
    },
];

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PerformancePolicy {
    pub capability: Capability,
    pub required_layer: ArchitectureLayer,
    pub data_class: DataClass,
    pub audit_event: AuditEventKind,
}

impl PerformancePolicy {
    pub fn baseline() -> Vec<Self> {
        vec![
            Self {
                capability: Capability::GoalCycleOpen,
                required_layer: ArchitectureLayer::Usecase,
                data_class: DataClass::WorkforceConfidential,
                audit_event: AuditEventKind::ReviewCycleOpened,
            },
            Self {
                capability: Capability::ManagerFeedbackGate,
                required_layer: ArchitectureLayer::Rest,
                data_class: DataClass::ManagerPrivate,
                audit_event: AuditEventKind::FeedbackSubmitted,
            },
            Self {
                capability: Capability::CalibrationRun,
                required_layer: ArchitectureLayer::Worker,
                data_class: DataClass::AggregateAnalytics,
                audit_event: AuditEventKind::CalibrationCompleted,
            },
            Self {
                capability: Capability::ReviewEvidenceSeal,
                required_layer: ArchitectureLayer::Api,
                data_class: DataClass::AuditEvidence,
                audit_event: AuditEventKind::EvidenceSealed,
            },
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ReviewEvidence {
    pub tenant_id: TenantId,
    pub review_cycle_id: ReviewCycleId,
    pub subject_employee_id: EmployeeId,
    pub author_employee_id: EmployeeId,
    pub visibility: FeedbackVisibility,
    pub rating_band: Option<RatingBand>,
    pub narrative: String,
}

impl ReviewEvidence {
    pub fn validate(&self) -> ServiceResult<()> {
        ensure_same_tenant(&self.tenant_id, &self.tenant_id)?;
        if self.narrative.trim().len() < 8 {
            return Err(ServiceError::invariant(
                "feedback_narrative_minimum",
                "feedback narrative must be at least 8 characters",
            ));
        }
        if self.subject_employee_id == self.author_employee_id {
            return Err(ServiceError::invariant(
                "feedback_author_subject_separation",
                "author and subject must be distinct employees",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct GoalAlignmentSnapshot {
    pub tenant_id: TenantId,
    pub employee_id: EmployeeId,
    pub review_cycle_id: ReviewCycleId,
    pub aligned_goal_count: u16,
    pub blocked_goal_count: u16,
}

impl GoalAlignmentSnapshot {
    pub fn alignment_ratio(&self) -> f32 {
        let total = self.aligned_goal_count + self.blocked_goal_count;
        if total == 0 {
            0.0
        } else {
            f32::from(self.aligned_goal_count) / f32::from(total)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CalibrationRun {
    pub tenant_id: TenantId,
    pub review_cycle_id: ReviewCycleId,
    pub cohort_id: CalibrationCohortId,
    pub status: ReviewCycleStatus,
    pub participant_count: u32,
    pub distribution_locked: bool,
}

impl CalibrationRun {
    pub fn validate(&self) -> ServiceResult<()> {
        if self.participant_count == 0 {
            return Err(ServiceError::invariant(
                "calibration_participant_count",
                "calibration requires at least one participant",
            ));
        }
        if self.distribution_locked && !matches!(self.status, ReviewCycleStatus::Calibrated) {
            return Err(ServiceError::invariant(
                "locked_distribution_status",
                "locked distribution requires calibrated status",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ReviewCycle {
    pub tenant_id: TenantId,
    pub review_cycle_id: ReviewCycleId,
    pub title: String,
    pub status: ReviewCycleStatus,
    pub evidence_count: u32,
}

impl ReviewCycle {
    pub fn new(
        tenant_id: TenantId,
        review_cycle_id: ReviewCycleId,
        title: String,
        status: ReviewCycleStatus,
    ) -> Self {
        Self {
            tenant_id,
            review_cycle_id,
            title,
            status,
            evidence_count: 0,
        }
    }

    pub fn open(mut self) -> ServiceResult<Self> {
        if !matches!(self.status, ReviewCycleStatus::Draft) {
            return Err(ServiceError::invariant(
                "review_cycle_open_transition",
                "only draft cycles can be opened",
            ));
        }
        self.status = ReviewCycleStatus::Open;
        Ok(self)
    }

    pub fn record_feedback(mut self) -> ServiceResult<Self> {
        if !self.status.allows_feedback() {
            return Err(ServiceError::invariant(
                "feedback_status_gate",
                "cycle must be open before feedback submission",
            ));
        }
        self.evidence_count += 1;
        Ok(self)
    }

    pub fn validate(&self) -> ServiceResult<()> {
        validate_identifier("tenant_id", self.tenant_id.as_str())?;
        validate_identifier("review_cycle_id", self.review_cycle_id.as_str())?;
        if self.title.trim().is_empty() {
            return Err(ServiceError::missing_field("title"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DomainInvariant {
    pub name: &'static str,
    pub layer: ArchitectureLayer,
    pub statement: String,
}

impl DomainInvariant {
    pub fn tenant_scoped(statement: impl Into<String>) -> Self {
        Self {
            name: "tenant_scoped",
            layer: ArchitectureLayer::Kernel,
            statement: statement.into(),
        }
    }

    pub fn policy_checked(statement: impl Into<String>) -> Self {
        Self {
            name: "policy_checked",
            layer: ArchitectureLayer::Usecase,
            statement: statement.into(),
        }
    }

    pub fn audit_emitted(statement: impl Into<String>) -> Self {
        Self {
            name: "audit_emitted",
            layer: ArchitectureLayer::Api,
            statement: statement.into(),
        }
    }

    pub fn data_classified(statement: impl Into<String>) -> Self {
        Self {
            name: "data_classified",
            layer: ArchitectureLayer::Domain,
            statement: statement.into(),
        }
    }

    pub fn region_bound(statement: impl Into<String>) -> Self {
        Self {
            name: "region_bound",
            layer: ArchitectureLayer::Infrastructure,
            statement: statement.into(),
        }
    }
}

pub fn ensure_same_tenant(left: &TenantId, right: &TenantId) -> ServiceResult<()> {
    if left == right {
        Ok(())
    } else {
        Err(ServiceError::invariant(
            "tenant_scope_match",
            "cross-tenant review data is not allowed",
        ))
    }
}

pub fn validate_identifier(field: &'static str, value: &str) -> ServiceResult<()> {
    let valid = !value.trim().is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err(ServiceError::invalid_identifier(field, value))
    }
}
