use crate::domain::{
    AuditEventKind, Capability, EmployeeId, FeedbackVisibility, RatingBand, ReviewCycle,
    ReviewCycleId, ReviewCycleStatus, ReviewEvidence, TenantId,
};
use crate::error::{ServiceError, ServiceResult};

pub trait ReviewCycleRepository {
    fn put_review_cycle(&mut self, cycle: ReviewCycle) -> ServiceResult<ReviewCycle>;
    fn get_review_cycle(
        &self,
        tenant_id: &TenantId,
        review_cycle_id: &ReviewCycleId,
    ) -> ServiceResult<Option<ReviewCycle>>;
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

pub trait PerformancePorts: ReviewCycleRepository + PolicyAuthorizer + AuditPublisher {}

impl<T> PerformancePorts for T where T: ReviewCycleRepository + PolicyAuthorizer + AuditPublisher {}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OpenReviewCycleCommand {
    pub tenant_id: TenantId,
    pub review_cycle_id: ReviewCycleId,
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SubmitFeedbackCommand {
    pub tenant_id: TenantId,
    pub review_cycle_id: ReviewCycleId,
    pub subject_employee_id: EmployeeId,
    pub author_employee_id: EmployeeId,
    pub visibility: FeedbackVisibility,
    pub rating_band: Option<RatingBand>,
    pub narrative: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SealReviewEvidenceCommand {
    pub tenant_id: TenantId,
    pub review_cycle_id: ReviewCycleId,
    pub sealed_by: EmployeeId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UsecaseReceipt {
    pub tenant_id: TenantId,
    pub review_cycle_id: ReviewCycleId,
    pub audit_event: AuditEventKind,
    pub status: ReviewCycleStatus,
}

pub struct OpenReviewCycle;

impl OpenReviewCycle {
    pub fn execute(
        ports: &mut impl PerformancePorts,
        command: OpenReviewCycleCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::GoalCycleOpen)?;
        let cycle = ReviewCycle::new(
            command.tenant_id.clone(),
            command.review_cycle_id.clone(),
            command.title,
            ReviewCycleStatus::Draft,
        )
        .open()?;
        cycle.validate()?;
        let cycle = ports.put_review_cycle(cycle)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::ReviewCycleOpened,
            command.review_cycle_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: cycle.tenant_id,
            review_cycle_id: cycle.review_cycle_id,
            audit_event: AuditEventKind::ReviewCycleOpened,
            status: cycle.status,
        })
    }
}

pub struct SubmitFeedback;

impl SubmitFeedback {
    pub fn execute(
        ports: &mut impl PerformancePorts,
        command: SubmitFeedbackCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::ManagerFeedbackGate)?;
        let evidence = ReviewEvidence {
            tenant_id: command.tenant_id.clone(),
            review_cycle_id: command.review_cycle_id.clone(),
            subject_employee_id: command.subject_employee_id,
            author_employee_id: command.author_employee_id,
            visibility: command.visibility,
            rating_band: command.rating_band,
            narrative: command.narrative,
        };
        evidence.validate()?;
        let cycle = ports
            .get_review_cycle(&command.tenant_id, &command.review_cycle_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "review_cycle_repository",
            })?
            .record_feedback()?;
        let cycle = ports.put_review_cycle(cycle)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::FeedbackSubmitted,
            command.review_cycle_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: cycle.tenant_id,
            review_cycle_id: cycle.review_cycle_id,
            audit_event: AuditEventKind::FeedbackSubmitted,
            status: cycle.status,
        })
    }
}

pub struct SealReviewEvidence;

impl SealReviewEvidence {
    pub fn execute(
        ports: &mut impl PerformancePorts,
        command: SealReviewEvidenceCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::ReviewEvidenceSeal)?;
        let mut cycle = ports
            .get_review_cycle(&command.tenant_id, &command.review_cycle_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "review_cycle_repository",
            })?;
        if cycle.evidence_count == 0 {
            return Err(ServiceError::invariant(
                "evidence_before_seal",
                "at least one feedback evidence item is required before sealing",
            ));
        }
        cycle.status = ReviewCycleStatus::Sealed;
        let cycle = ports.put_review_cycle(cycle)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::EvidenceSealed,
            command.sealed_by.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: cycle.tenant_id,
            review_cycle_id: cycle.review_cycle_id,
            audit_event: AuditEventKind::EvidenceSealed,
            status: cycle.status,
        })
    }
}

pub struct PerformanceManagementService<P> {
    ports: P,
}

impl<P> PerformanceManagementService<P>
where
    P: PerformancePorts,
{
    pub fn new(ports: P) -> Self {
        Self { ports }
    }

    pub fn open_review_cycle(
        &mut self,
        command: OpenReviewCycleCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        OpenReviewCycle::execute(&mut self.ports, command)
    }

    pub fn submit_feedback(
        &mut self,
        command: SubmitFeedbackCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        SubmitFeedback::execute(&mut self.ports, command)
    }

    pub fn seal_review_evidence(
        &mut self,
        command: SealReviewEvidenceCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        SealReviewEvidence::execute(&mut self.ports, command)
    }

    pub fn into_ports(self) -> P {
        self.ports
    }
}
