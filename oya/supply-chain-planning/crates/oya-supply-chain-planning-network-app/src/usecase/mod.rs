use crate::domain::{ServiceCommand, ServiceEvent, TenantId, UsecaseActor};
use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UsecaseContext {
    pub actor: UsecaseActor,
    pub source: String,
    pub data_residency_pack: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CommandEnvelope {
    pub context: UsecaseContext,
    pub command: ServiceCommand,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CommandReceipt {
    pub accepted: bool,
    pub tenant_id: TenantId,
    pub capability: String,
    pub audit_event_type: String,
}
pub trait PolicyPort {
    fn authorize(&self, envelope: &CommandEnvelope) -> Result<()>;
}
pub trait AuditPort {
    fn append(&self, event: &ServiceEvent) -> Result<()>;
}
pub trait EventPort {
    fn publish(&self, event: &ServiceEvent) -> Result<()>;
}
pub trait RepositoryPort {
    fn reserve_idempotency_key(&self, envelope: &CommandEnvelope) -> Result<()>;
    fn persist_command_receipt(&self, receipt: &CommandReceipt) -> Result<()>;
}
pub trait ClockPort {
    fn now_rfc3339(&self) -> Result<String>;
}
pub struct ServiceInteractor<P, A, E, R, C>
where
    P: PolicyPort,
    A: AuditPort,
    E: EventPort,
    R: RepositoryPort,
    C: ClockPort,
{
    policy: P,
    audit: A,
    events: E,
    repository: R,
    clock: C,
}
impl<P, A, E, R, C> ServiceInteractor<P, A, E, R, C>
where
    P: PolicyPort,
    A: AuditPort,
    E: EventPort,
    R: RepositoryPort,
    C: ClockPort,
{
    pub fn new(policy: P, audit: A, events: E, repository: R, clock: C) -> Self {
        Self {
            policy,
            audit,
            events,
            repository,
            clock,
        }
    }
    pub fn submit_command(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        self.handle(envelope)
    }
    pub fn reconcile(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(&envelope.command, ServiceCommand::Reconcile { .. }),
            "Reconcile",
        )?;
        self.handle(envelope)
    }
    pub fn apply_governance_hold(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                &envelope.command,
                ServiceCommand::ApplyGovernanceHold { .. }
            ),
            "ApplyGovernanceHold",
        )?;
        self.handle(envelope)
    }
    pub fn export_evidence(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(&envelope.command, ServiceCommand::ExportEvidence { .. }),
            "ExportEvidence",
        )?;
        self.handle(envelope)
    }
    fn handle(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        self.policy.authorize(&envelope)?;
        self.repository.reserve_idempotency_key(&envelope)?;
        let event = event_for(&envelope);
        self.audit.append(&event)?;
        self.events.publish(&event)?;
        let _observed_at = self.clock.now_rfc3339()?;
        let receipt = CommandReceipt {
            accepted: true,
            tenant_id: envelope.context.actor.tenant_id.clone(),
            capability: format!("{:?}", envelope.command.capability()),
            audit_event_type: event_type(&event).to_string(),
        };
        self.repository.persist_command_receipt(&receipt)?;
        Ok(receipt)
    }
}
fn ensure_command(
    envelope: &CommandEnvelope,
    matches_expected: bool,
    expected: &str,
) -> Result<()> {
    if matches_expected {
        Ok(())
    } else {
        Err(ServiceError::validation(
            "command",
            format!(
                "expected {expected}, got {:?}",
                envelope.command.capability()
            ),
        ))
    }
}
fn event_for(envelope: &CommandEnvelope) -> ServiceEvent {
    let tenant_id = envelope.context.actor.tenant_id.clone();
    let capability = envelope.command.capability();
    match &envelope.command {
        ServiceCommand::Submit { .. } => ServiceEvent::CommandAccepted {
            capability,
            tenant_id,
        },
        ServiceCommand::Reconcile { .. } => ServiceEvent::ReconciliationQueued {
            capability,
            tenant_id,
        },
        ServiceCommand::ApplyGovernanceHold { .. } => ServiceEvent::GovernanceHoldApplied {
            capability,
            tenant_id,
        },
        ServiceCommand::ExportEvidence { .. } => ServiceEvent::EvidenceExportQueued {
            capability,
            tenant_id,
        },
    }
}
fn event_type(event: &ServiceEvent) -> &'static str {
    match event {
        ServiceEvent::CommandAccepted { .. } => "command-accepted",
        ServiceEvent::ReconciliationQueued { .. } => "reconciliation-queued",
        ServiceEvent::GovernanceHoldApplied { .. } => "governance-hold-applied",
        ServiceEvent::EvidenceExportQueued { .. } => "evidence-export-queued",
    }
}
