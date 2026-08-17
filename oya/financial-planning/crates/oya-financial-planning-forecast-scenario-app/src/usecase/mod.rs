use crate::domain::{FinancialPlanningCommand, FinancialPlanningEvent, TenantId, UsecaseActor};
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
    pub command: FinancialPlanningCommand,
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
    fn append(&self, event: &FinancialPlanningEvent) -> Result<()>;
}

pub trait EventPort {
    fn publish(&self, event: &FinancialPlanningEvent) -> Result<()>;
}

pub trait RepositoryPort {
    fn reserve_idempotency_key(&self, envelope: &CommandEnvelope) -> Result<()>;

    fn persist_command_receipt(&self, receipt: &CommandReceipt) -> Result<()>;
}

pub trait ClockPort {
    fn now_rfc3339(&self) -> Result<String>;
}

pub struct ForecastScenarioInteractor<P, A, E, R, C>
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

impl<P, A, E, R, C> ForecastScenarioInteractor<P, A, E, R, C>
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

    pub fn open_forecast_version(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                FinancialPlanningCommand::OpenForecastVersion { .. }
            ),
            "OpenForecastVersion",
        )?;
        self.handle(envelope)
    }

    pub fn import_driver_model(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                FinancialPlanningCommand::ImportDriverModel { .. }
            ),
            "ImportDriverModel",
        )?;
        self.handle(envelope)
    }

    pub fn recalculate_scenario(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                FinancialPlanningCommand::RecalculateScenario { .. }
            ),
            "RecalculateScenario",
        )?;
        self.handle(envelope)
    }

    pub fn explain_variance(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(
                envelope.command,
                FinancialPlanningCommand::ExplainVariance { .. }
            ),
            "ExplainVariance",
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

fn event_for(envelope: &CommandEnvelope) -> FinancialPlanningEvent {
    let tenant_id = envelope.context.actor.tenant_id.clone();
    match &envelope.command {
        FinancialPlanningCommand::OpenForecastVersion {
            forecast_version_id,
            ..
        } => FinancialPlanningEvent::ForecastVersionOpened {
            forecast_version_id: forecast_version_id.clone(),
            tenant_id,
        },
        FinancialPlanningCommand::ImportDriverModel {
            forecast_version_id,
        } => FinancialPlanningEvent::DriverModelImportQueued {
            forecast_version_id: forecast_version_id.clone(),
            tenant_id,
        },
        FinancialPlanningCommand::RecalculateScenario { scenario_id } => {
            FinancialPlanningEvent::ScenarioRecalculateQueued {
                scenario_id: scenario_id.clone(),
                tenant_id,
            }
        }
        FinancialPlanningCommand::ExplainVariance {
            forecast_version_id,
        } => FinancialPlanningEvent::VarianceExplanationQueued {
            forecast_version_id: forecast_version_id.clone(),
            tenant_id,
        },
        FinancialPlanningCommand::CloseConsolidation { budget_cycle_id } => {
            FinancialPlanningEvent::ConsolidationCloseQueued {
                budget_cycle_id: budget_cycle_id.clone(),
                tenant_id,
            }
        }
        FinancialPlanningCommand::SealBoardReport {
            forecast_version_id,
        } => FinancialPlanningEvent::BoardReportSealQueued {
            forecast_version_id: forecast_version_id.clone(),
            tenant_id,
        },
    }
}

fn event_type(event: &FinancialPlanningEvent) -> &'static str {
    match event {
        FinancialPlanningEvent::ForecastVersionOpened { .. } => "forecast-version-opened",
        FinancialPlanningEvent::DriverModelImportQueued { .. } => "driver-model-import-queued",
        FinancialPlanningEvent::ScenarioRecalculateQueued { .. } => "scenario-recalculate-queued",
        FinancialPlanningEvent::VarianceExplanationQueued { .. } => "variance-explanation-queued",
        FinancialPlanningEvent::ConsolidationCloseQueued { .. } => "consolidation-close-queued",
        FinancialPlanningEvent::BoardReportSealQueued { .. } => "board-report-seal-queued",
    }
}
