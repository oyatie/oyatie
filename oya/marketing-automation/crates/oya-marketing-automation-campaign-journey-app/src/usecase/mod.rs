use crate::domain::{MarketingAutomationCommand, MarketingAutomationEvent, TenantId, UsecaseActor};
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
    pub command: MarketingAutomationCommand,
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
    fn append(&self, event: &MarketingAutomationEvent) -> Result<()>;
}

pub trait EventPort {
    fn publish(&self, event: &MarketingAutomationEvent) -> Result<()>;
}

pub trait RepositoryPort {
    fn reserve_idempotency_key(&self, envelope: &CommandEnvelope) -> Result<()>;

    fn persist_command_receipt(&self, receipt: &CommandReceipt) -> Result<()>;
}

pub trait ClockPort {
    fn now_rfc3339(&self) -> Result<String>;
}

pub struct CampaignJourneyInteractor<P, A, E, R, C>
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

impl<P, A, E, R, C> CampaignJourneyInteractor<P, A, E, R, C>
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

    pub fn launch_journey(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                MarketingAutomationCommand::LaunchJourney { .. }
            ),
            "LaunchJourney",
        )?;
        self.handle(envelope)
    }

    pub fn enforce_suppression(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                MarketingAutomationCommand::EnforceSuppression { .. }
            ),
            "EnforceSuppression",
        )?;
        self.handle(envelope)
    }

    pub fn sync_segment(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                MarketingAutomationCommand::SyncSegment { .. }
            ),
            "SyncSegment",
        )?;
        self.handle(envelope)
    }

    pub fn rollup_attribution(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                MarketingAutomationCommand::RollupAttribution { .. }
            ),
            "RollupAttribution",
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

fn event_for(envelope: &CommandEnvelope) -> MarketingAutomationEvent {
    let tenant_id = envelope.context.actor.tenant_id.clone();
    match &envelope.command {
        MarketingAutomationCommand::LaunchJourney { journey_id, .. } => {
            MarketingAutomationEvent::JourneyLaunchAccepted {
                journey_id: journey_id.clone(),
                tenant_id,
            }
        }
        MarketingAutomationCommand::EnforceSuppression { segment_id, .. } => {
            MarketingAutomationEvent::SuppressionApplied {
                segment_id: segment_id.clone(),
                tenant_id,
            }
        }
        MarketingAutomationCommand::SyncSegment { segment_id } => {
            MarketingAutomationEvent::SegmentSyncRequested {
                segment_id: segment_id.clone(),
                tenant_id,
            }
        }
        MarketingAutomationCommand::RollupAttribution { journey_id } => {
            MarketingAutomationEvent::AttributionRollupQueued {
                journey_id: journey_id.clone(),
                tenant_id,
            }
        }
        MarketingAutomationCommand::ExportConsent { consent_ref } => {
            MarketingAutomationEvent::ConsentExportQueued {
                consent_ref: consent_ref.clone(),
                tenant_id,
            }
        }
        MarketingAutomationCommand::LicenseMarketplaceAudience { segment_id } => {
            MarketingAutomationEvent::MarketplaceAudienceLicenseHeld {
                segment_id: segment_id.clone(),
                tenant_id,
            }
        }
    }
}

fn event_type(event: &MarketingAutomationEvent) -> &'static str {
    match event {
        MarketingAutomationEvent::JourneyLaunchAccepted { .. } => "journey-launch-accepted",
        MarketingAutomationEvent::SuppressionApplied { .. } => "suppression-applied",
        MarketingAutomationEvent::SegmentSyncRequested { .. } => "segment-sync-requested",
        MarketingAutomationEvent::AttributionRollupQueued { .. } => "attribution-rollup-queued",
        MarketingAutomationEvent::ConsentExportQueued { .. } => "consent-export-queued",
        MarketingAutomationEvent::MarketplaceAudienceLicenseHeld { .. } => {
            "marketplace-audience-license-held"
        }
    }
}
