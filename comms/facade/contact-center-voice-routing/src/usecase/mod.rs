use crate::domain::{ContactCenterCommand, ContactCenterEvent, TenantId, UsecaseActor};
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
    pub command: ContactCenterCommand,
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
    fn append(&self, event: &ContactCenterEvent) -> Result<()>;
}

pub trait EventPort {
    fn publish(&self, event: &ContactCenterEvent) -> Result<()>;
}

pub trait RepositoryPort {
    fn reserve_idempotency_key(&self, envelope: &CommandEnvelope) -> Result<()>;

    fn persist_command_receipt(&self, receipt: &CommandReceipt) -> Result<()>;
}

pub trait ClockPort {
    fn now_rfc3339(&self) -> Result<String>;
}

pub struct VoiceRoutingInteractor<P, A, E, R, C>
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

impl<P, A, E, R, C> VoiceRoutingInteractor<P, A, E, R, C>
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

    pub fn route_voice_contact(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                ContactCenterCommand::RouteVoiceContact { .. }
            ),
            "RouteVoiceContact",
        )?;
        self.handle(envelope)
    }

    pub fn rebalance_queue(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                ContactCenterCommand::RebalanceQueue { .. }
            ),
            "RebalanceQueue",
        )?;
        self.handle(envelope)
    }

    pub fn record_consent(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(envelope.command, ContactCenterCommand::RecordConsent { .. }),
            "RecordConsent",
        )?;
        self.handle(envelope)
    }

    pub fn sync_agent_state(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                ContactCenterCommand::SyncAgentState { .. }
            ),
            "SyncAgentState",
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

fn event_for(envelope: &CommandEnvelope) -> ContactCenterEvent {
    let tenant_id = envelope.context.actor.tenant_id.clone();
    match &envelope.command {
        ContactCenterCommand::RouteVoiceContact { route_id, .. } => {
            ContactCenterEvent::VoiceRouteAccepted {
                route_id: route_id.clone(),
                tenant_id,
            }
        }
        ContactCenterCommand::RebalanceQueue { queue_id } => {
            ContactCenterEvent::QueueRebalanceRequested {
                queue_id: queue_id.clone(),
                tenant_id,
            }
        }
        ContactCenterCommand::RecordConsent { route_id, .. } => {
            ContactCenterEvent::RecordingConsentCaptured {
                route_id: route_id.clone(),
                tenant_id,
            }
        }
        ContactCenterCommand::SyncAgentState { queue_id } => {
            ContactCenterEvent::AgentStateSyncRequested {
                queue_id: queue_id.clone(),
                tenant_id,
            }
        }
        ContactCenterCommand::ScheduleCallback { route_id } => {
            ContactCenterEvent::CallbackScheduled {
                route_id: route_id.clone(),
                tenant_id,
            }
        }
        ContactCenterCommand::EmergencyCallerBypass { route_id } => {
            ContactCenterEvent::EmergencyCallerBypassHeld {
                route_id: route_id.clone(),
                tenant_id,
            }
        }
    }
}

fn event_type(event: &ContactCenterEvent) -> &'static str {
    match event {
        ContactCenterEvent::VoiceRouteAccepted { .. } => "voice-route-accepted",
        ContactCenterEvent::QueueRebalanceRequested { .. } => "queue-rebalance-requested",
        ContactCenterEvent::RecordingConsentCaptured { .. } => "recording-consent-captured",
        ContactCenterEvent::AgentStateSyncRequested { .. } => "agent-state-sync-requested",
        ContactCenterEvent::CallbackScheduled { .. } => "callback-scheduled",
        ContactCenterEvent::EmergencyCallerBypassHeld { .. } => "emergency-caller-bypass-held",
    }
}
