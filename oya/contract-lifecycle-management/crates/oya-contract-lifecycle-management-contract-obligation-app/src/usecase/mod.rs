use crate::domain::{ContractLifecycleCommand, ContractLifecycleEvent, TenantId, UsecaseActor};
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
    pub command: ContractLifecycleCommand,
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
    fn append(&self, event: &ContractLifecycleEvent) -> Result<()>;
}

pub trait EventPort {
    fn publish(&self, event: &ContractLifecycleEvent) -> Result<()>;
}

pub trait RepositoryPort {
    fn reserve_idempotency_key(&self, envelope: &CommandEnvelope) -> Result<()>;

    fn persist_command_receipt(&self, receipt: &CommandReceipt) -> Result<()>;
}

pub trait ClockPort {
    fn now_rfc3339(&self) -> Result<String>;
}

pub struct ContractObligationInteractor<P, A, E, R, C>
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

impl<P, A, E, R, C> ContractObligationInteractor<P, A, E, R, C>
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

    pub fn handle(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
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

    pub fn create_contract_draft(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                ContractLifecycleCommand::CreateContractDraft { .. }
            ),
            "CreateContractDraft",
        )?;
        self.handle(envelope)
    }

    pub fn evaluate_clause_policy(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                ContractLifecycleCommand::EvaluateClausePolicy { .. }
            ),
            "EvaluateClausePolicy",
        )?;
        self.handle(envelope)
    }

    pub fn route_approval(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                ContractLifecycleCommand::RouteApproval { .. }
            ),
            "RouteApproval",
        )?;
        self.handle(envelope)
    }

    pub fn track_obligation(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                ContractLifecycleCommand::TrackObligation { .. }
            ),
            "TrackObligation",
        )?;
        self.handle(envelope)
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

fn event_for(envelope: &CommandEnvelope) -> ContractLifecycleEvent {
    let tenant_id = envelope.context.actor.tenant_id.clone();
    match &envelope.command {
        ContractLifecycleCommand::CreateContractDraft { contract_id, .. } => {
            ContractLifecycleEvent::ContractDraftCreated {
                contract_id: contract_id.clone(),
                tenant_id,
            }
        }
        ContractLifecycleCommand::EvaluateClausePolicy { contract_id, .. } => {
            ContractLifecycleEvent::ClausePolicyEvaluationQueued {
                contract_id: contract_id.clone(),
                tenant_id,
            }
        }
        ContractLifecycleCommand::RouteApproval { contract_id } => {
            ContractLifecycleEvent::ApprovalRouteQueued {
                contract_id: contract_id.clone(),
                tenant_id,
            }
        }
        ContractLifecycleCommand::TrackObligation { obligation_id, .. } => {
            ContractLifecycleEvent::ObligationTrackQueued {
                obligation_id: obligation_id.clone(),
                tenant_id,
            }
        }
        ContractLifecycleCommand::ScoreRenewalRisk { contract_id } => {
            ContractLifecycleEvent::RenewalRiskScoreQueued {
                contract_id: contract_id.clone(),
                tenant_id,
            }
        }
        ContractLifecycleCommand::BindDealSetContract { contract_id } => {
            ContractLifecycleEvent::DealSetContractBindQueued {
                contract_id: contract_id.clone(),
                tenant_id,
            }
        }
    }
}

fn event_type(event: &ContractLifecycleEvent) -> &'static str {
    match event {
        ContractLifecycleEvent::ContractDraftCreated { .. } => "contract-draft-created",
        ContractLifecycleEvent::ClausePolicyEvaluationQueued { .. } => {
            "clause-policy-evaluation-queued"
        }
        ContractLifecycleEvent::ApprovalRouteQueued { .. } => "approval-route-queued",
        ContractLifecycleEvent::ObligationTrackQueued { .. } => "obligation-track-queued",
        ContractLifecycleEvent::RenewalRiskScoreQueued { .. } => "renewal-risk-score-queued",
        ContractLifecycleEvent::DealSetContractBindQueued { .. } => "dealset-contract-bind-queued",
    }
}
