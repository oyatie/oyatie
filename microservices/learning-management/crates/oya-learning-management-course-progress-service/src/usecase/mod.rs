use crate::domain::{
    AuditEventKind, Capability, CourseId, EnrollmentId, EnrollmentStatus, LearnerId, LearningPath,
    TenantId,
};
use crate::error::{ServiceError, ServiceResult};

pub trait LearningPathRepository {
    fn put_learning_path(&mut self, path: LearningPath) -> ServiceResult<LearningPath>;
    fn get_learning_path(
        &self,
        tenant_id: &TenantId,
        enrollment_id: &EnrollmentId,
    ) -> ServiceResult<Option<LearningPath>>;
}

pub trait PolicyAuthorizer {
    fn authorize(&self, tenant_id: &TenantId, capability: Capability) -> ServiceResult<()>;
}

pub trait AuditPublisher {
    fn publish_audit(
        &mut self,
        tenant_id: &TenantId,
        event_kind: AuditEventKind,
        subject: &str,
    ) -> ServiceResult<()>;
}

pub trait LearningPorts: LearningPathRepository + PolicyAuthorizer + AuditPublisher {}

impl<T> LearningPorts for T where T: LearningPathRepository + PolicyAuthorizer + AuditPublisher {}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OpenEnrollmentCommand {
    pub tenant_id: TenantId,
    pub enrollment_id: EnrollmentId,
    pub course_id: CourseId,
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RecordProgressCommand {
    pub tenant_id: TenantId,
    pub enrollment_id: EnrollmentId,
    pub learner_id: LearnerId,
    pub progress_percent: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SealCourseCompletionCommand {
    pub tenant_id: TenantId,
    pub enrollment_id: EnrollmentId,
    pub sealed_by: LearnerId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UsecaseReceipt {
    pub tenant_id: TenantId,
    pub enrollment_id: EnrollmentId,
    pub audit_event: AuditEventKind,
    pub status: EnrollmentStatus,
}

pub struct OpenEnrollment;

impl OpenEnrollment {
    pub fn execute(
        ports: &mut impl LearningPorts,
        command: OpenEnrollmentCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::CourseAssign)?;
        let path = LearningPath::new(
            command.tenant_id.clone(),
            command.enrollment_id.clone(),
            command.course_id,
            command.title,
            EnrollmentStatus::Draft,
        )
        .assign()?;
        path.validate()?;
        let path = ports.put_learning_path(path)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::EnrollmentOpened,
            command.enrollment_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: path.tenant_id,
            enrollment_id: path.enrollment_id,
            audit_event: AuditEventKind::EnrollmentOpened,
            status: path.status,
        })
    }
}

pub struct RecordProgress;

impl RecordProgress {
    pub fn execute(
        ports: &mut impl LearningPorts,
        command: RecordProgressCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::ProgressRecord)?;
        let path = ports
            .get_learning_path(&command.tenant_id, &command.enrollment_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "learning_path_repository",
            })?
            .record_progress(command.progress_percent)?;
        let path = ports.put_learning_path(path)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::ProgressRecorded,
            command.learner_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: path.tenant_id,
            enrollment_id: path.enrollment_id,
            audit_event: AuditEventKind::ProgressRecorded,
            status: path.status,
        })
    }
}

pub struct SealCourseCompletion;

impl SealCourseCompletion {
    pub fn execute(
        ports: &mut impl LearningPorts,
        command: SealCourseCompletionCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::CertificateSeal)?;
        let path = ports
            .get_learning_path(&command.tenant_id, &command.enrollment_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "learning_path_repository",
            })?
            .seal_certificate()?;
        let path = ports.put_learning_path(path)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::CertificateSealed,
            command.sealed_by.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: path.tenant_id,
            enrollment_id: path.enrollment_id,
            audit_event: AuditEventKind::CertificateSealed,
            status: path.status,
        })
    }
}

pub struct LearningManagementService<P> {
    ports: P,
}

impl<P> LearningManagementService<P>
where
    P: LearningPorts,
{
    pub fn new(ports: P) -> Self {
        Self { ports }
    }

    pub fn open_enrollment(
        &mut self,
        command: OpenEnrollmentCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        OpenEnrollment::execute(&mut self.ports, command)
    }

    pub fn record_progress(
        &mut self,
        command: RecordProgressCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        RecordProgress::execute(&mut self.ports, command)
    }

    pub fn seal_course_completion(
        &mut self,
        command: SealCourseCompletionCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        SealCourseCompletion::execute(&mut self.ports, command)
    }

    pub fn into_ports(self) -> P {
        self.ports
    }
}
