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
    Worker,
    Sdk,
    Api,
}

impl ArchitectureLayer {
    pub const fn all() -> [Self; 12] {
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
pub struct CourseId(String);

impl CourseId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("course_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LearnerId(String);

impl LearnerId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("learner_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EnrollmentId(String);

impl EnrollmentId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn parse(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        validate_identifier("enrollment_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum EnrollmentStatus {
    Draft,
    Assigned,
    InProgress,
    AssessmentLocked,
    Completed,
    Certified,
    Cancelled,
}

impl EnrollmentStatus {
    pub const fn allows_progress(&self) -> bool {
        matches!(self, Self::Assigned | Self::InProgress)
    }

    pub const fn allows_certificate(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AssessmentBand {
    Mastered,
    Proficient,
    Developing,
    NeedsRetake,
}

impl AssessmentBand {
    pub const fn is_passing(&self) -> bool {
        matches!(self, Self::Mastered | Self::Proficient)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum EvidenceVisibility {
    LearnerVisible,
    ManagerVisible,
    InstructorOnly,
    ComplianceOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum DataClass {
    LearnerConfidential,
    AssessmentEvidence,
    AggregateAnalytics,
    ComplianceCertificate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum AuditEventKind {
    EnrollmentOpened,
    ProgressRecorded,
    AssessmentSubmitted,
    CourseCompleted,
    CertificateSealed,
    RemediationAssigned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Capability {
    CourseAssign,
    ProgressRecord,
    AssessmentSubmit,
    CourseComplete,
    CertificateSeal,
    RemediationAssign,
}

impl Capability {
    pub const fn action_slug(&self) -> &'static str {
        match self {
            Self::CourseAssign => "learning.course.assign",
            Self::ProgressRecord => "learning.progress.record",
            Self::AssessmentSubmit => "learning.assessment.submit",
            Self::CourseComplete => "learning.course.complete",
            Self::CertificateSeal => "learning.certificate.seal",
            Self::RemediationAssign => "learning.remediation.assign",
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
        capability: Capability::CourseAssign,
        command_name: "OpenEnrollmentCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Usecase,
        data_class: DataClass::LearnerConfidential,
        audit_event: AuditEventKind::EnrollmentOpened,
        idempotency_key: "tenant_id + enrollment_id",
    },
    CapabilityContract {
        capability: Capability::ProgressRecord,
        command_name: "RecordProgressCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Rest,
        data_class: DataClass::AssessmentEvidence,
        audit_event: AuditEventKind::ProgressRecorded,
        idempotency_key: "tenant_id + enrollment_id + learner_id + progress_percent",
    },
    CapabilityContract {
        capability: Capability::AssessmentSubmit,
        command_name: "SubmitAssessmentCommand",
        result_name: "AssessmentReceipt",
        required_layer: ArchitectureLayer::Api,
        data_class: DataClass::AssessmentEvidence,
        audit_event: AuditEventKind::AssessmentSubmitted,
        idempotency_key: "tenant_id + enrollment_id + assessment_attempt",
    },
    CapabilityContract {
        capability: Capability::CourseComplete,
        command_name: "CompleteCourseCommand",
        result_name: "CourseCompletionReceipt",
        required_layer: ArchitectureLayer::Worker,
        data_class: DataClass::ComplianceCertificate,
        audit_event: AuditEventKind::CourseCompleted,
        idempotency_key: "tenant_id + enrollment_id + course_id",
    },
    CapabilityContract {
        capability: Capability::CertificateSeal,
        command_name: "SealCourseCompletionCommand",
        result_name: "UsecaseReceipt",
        required_layer: ArchitectureLayer::Api,
        data_class: DataClass::ComplianceCertificate,
        audit_event: AuditEventKind::CertificateSealed,
        idempotency_key: "tenant_id + enrollment_id + sealed_by",
    },
    CapabilityContract {
        capability: Capability::RemediationAssign,
        command_name: "AssignRemediationCommand",
        result_name: "RemediationReceipt",
        required_layer: ArchitectureLayer::Usecase,
        data_class: DataClass::LearnerConfidential,
        audit_event: AuditEventKind::RemediationAssigned,
        idempotency_key: "tenant_id + learner_id + course_id + remediation_plan",
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
        handler: "LearningHttpHandler::open_enrollment",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1enrollments",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Rest,
        protocol: "HTTP",
        handler: "LearningHttpHandler::record_progress",
        contract_path: "contracts/openapi-v1.yaml#/paths/~1v1~1enrollments~1{id}~1progress",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Grpc,
        protocol: "gRPC",
        handler: "LearningGrpcHandler::open_enrollment",
        contract_path: "contracts/learning-management-v1.proto",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "LearningAsyncApiHandler::enrollment_opened",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/enrollment_opened",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "LearningAsyncApiHandler::progress_recorded",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/progress_recorded",
    },
    AdapterSurface {
        layer: ArchitectureLayer::Api,
        protocol: "AsyncAPI",
        handler: "LearningAsyncApiHandler::course_completed",
        contract_path: "contracts/asyncapi-v1.yaml#/channels/course_completed",
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
        name: "course-assignment-policy",
        invariant: "policy_checked",
        expected_evidence: "Cedar decision for learning.course.assign",
    },
    OperationalCheckpoint {
        name: "progress-monotonic-percent",
        invariant: "progress_monotonic_percent",
        expected_evidence: "new progress >= stored progress",
    },
    OperationalCheckpoint {
        name: "assessment-passing-band",
        invariant: "assessment_completion_gate",
        expected_evidence: "AssessmentBand::is_passing before certificate",
    },
    OperationalCheckpoint {
        name: "certificate-after-completion",
        invariant: "certificate_completion_gate",
        expected_evidence: "EnrollmentStatus::Completed before seal",
    },
    OperationalCheckpoint {
        name: "learner-record-residency",
        invariant: "region_bound",
        expected_evidence: "residency pack id on learning record",
    },
    OperationalCheckpoint {
        name: "audit-chain-emission",
        invariant: "audit_emitted",
        expected_evidence: "EVT-LEARNING-MANAGEMENT-*",
    },
];

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LearningPolicy {
    pub capability: Capability,
    pub required_layer: ArchitectureLayer,
    pub data_class: DataClass,
    pub audit_event: AuditEventKind,
}

impl LearningPolicy {
    pub fn baseline() -> Vec<Self> {
        vec![
            Self {
                capability: Capability::CourseAssign,
                required_layer: ArchitectureLayer::Usecase,
                data_class: DataClass::LearnerConfidential,
                audit_event: AuditEventKind::EnrollmentOpened,
            },
            Self {
                capability: Capability::ProgressRecord,
                required_layer: ArchitectureLayer::Rest,
                data_class: DataClass::AssessmentEvidence,
                audit_event: AuditEventKind::ProgressRecorded,
            },
            Self {
                capability: Capability::CertificateSeal,
                required_layer: ArchitectureLayer::Api,
                data_class: DataClass::ComplianceCertificate,
                audit_event: AuditEventKind::CertificateSealed,
            },
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CourseEvidence {
    pub tenant_id: TenantId,
    pub enrollment_id: EnrollmentId,
    pub course_id: CourseId,
    pub learner_id: LearnerId,
    pub visibility: EvidenceVisibility,
    pub assessment_band: Option<AssessmentBand>,
    pub note: String,
}

impl CourseEvidence {
    pub fn validate(&self) -> ServiceResult<()> {
        if self.note.trim().len() < 4 {
            return Err(ServiceError::invariant(
                "course_evidence_note_minimum",
                "course evidence note must be at least 4 characters",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ProgressSnapshot {
    pub tenant_id: TenantId,
    pub enrollment_id: EnrollmentId,
    pub learner_id: LearnerId,
    pub completed_units: u16,
    pub total_units: u16,
}

impl ProgressSnapshot {
    pub fn completion_ratio(&self) -> f32 {
        if self.total_units == 0 {
            0.0
        } else {
            f32::from(self.completed_units) / f32::from(self.total_units)
        }
    }

    pub fn validate(&self) -> ServiceResult<()> {
        if self.completed_units > self.total_units {
            return Err(ServiceError::invariant(
                "progress_monotonic_bounds",
                "completed units cannot exceed total units",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct LearningPath {
    pub tenant_id: TenantId,
    pub enrollment_id: EnrollmentId,
    pub course_id: CourseId,
    pub title: String,
    pub status: EnrollmentStatus,
    pub progress_percent: u8,
}

impl LearningPath {
    pub fn new(
        tenant_id: TenantId,
        enrollment_id: EnrollmentId,
        course_id: CourseId,
        title: String,
        status: EnrollmentStatus,
    ) -> Self {
        Self {
            tenant_id,
            enrollment_id,
            course_id,
            title,
            status,
            progress_percent: 0,
        }
    }

    pub fn assign(mut self) -> ServiceResult<Self> {
        if !matches!(self.status, EnrollmentStatus::Draft) {
            return Err(ServiceError::invariant(
                "learning_path_assign_transition",
                "only draft learning paths can be assigned",
            ));
        }
        self.status = EnrollmentStatus::Assigned;
        Ok(self)
    }

    pub fn record_progress(mut self, progress_percent: u8) -> ServiceResult<Self> {
        if !self.status.allows_progress() {
            return Err(ServiceError::invariant(
                "progress_status_gate",
                "learning path must be assigned or in progress",
            ));
        }
        if progress_percent < self.progress_percent || progress_percent > 100 {
            return Err(ServiceError::invariant(
                "progress_monotonic_percent",
                "progress percent must be monotonic and <= 100",
            ));
        }
        self.progress_percent = progress_percent;
        self.status = if progress_percent == 100 {
            EnrollmentStatus::Completed
        } else {
            EnrollmentStatus::InProgress
        };
        Ok(self)
    }

    pub fn seal_certificate(mut self) -> ServiceResult<Self> {
        if !self.status.allows_certificate() {
            return Err(ServiceError::invariant(
                "certificate_completion_gate",
                "only completed courses can be certified",
            ));
        }
        self.status = EnrollmentStatus::Certified;
        Ok(self)
    }

    pub fn validate(&self) -> ServiceResult<()> {
        validate_identifier("tenant_id", self.tenant_id.as_str())?;
        validate_identifier("enrollment_id", self.enrollment_id.as_str())?;
        validate_identifier("course_id", self.course_id.as_str())?;
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

    pub fn progress_monotonic(statement: impl Into<String>) -> Self {
        Self {
            name: "progress_monotonic",
            layer: ArchitectureLayer::Domain,
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
            "cross-tenant learning records are not allowed",
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
