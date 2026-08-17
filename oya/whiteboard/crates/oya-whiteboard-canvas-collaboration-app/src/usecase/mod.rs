use crate::domain::{TenantId, UsecaseActor, WhiteboardCommand, WhiteboardEvent};
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
    pub command: WhiteboardCommand,
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
    fn append(&self, event: &WhiteboardEvent) -> Result<()>;
}

pub trait EventPort {
    fn publish(&self, event: &WhiteboardEvent) -> Result<()>;
}

pub trait RepositoryPort {
    fn reserve_idempotency_key(&self, envelope: &CommandEnvelope) -> Result<()>;

    fn persist_command_receipt(&self, receipt: &CommandReceipt) -> Result<()>;
}

pub trait ClockPort {
    fn now_rfc3339(&self) -> Result<String>;
}

pub struct CanvasCollaborationInteractor<P, A, E, R, C>
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

impl<P, A, E, R, C> CanvasCollaborationInteractor<P, A, E, R, C>
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

    pub fn open_board(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(envelope.command, WhiteboardCommand::OpenBoard { .. }),
            "OpenBoard",
        )?;
        self.handle(envelope)
    }

    pub fn append_canvas_op(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(envelope.command, WhiteboardCommand::AppendCanvasOp { .. }),
            "AppendCanvasOp",
        )?;
        self.handle(envelope)
    }

    pub fn render_export(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(envelope.command, WhiteboardCommand::RenderExport { .. }),
            "RenderExport",
        )?;
        self.handle(envelope)
    }

    pub fn sync_presence(&self, envelope: CommandEnvelope) -> Result<CommandReceipt> {
        ensure_command(
            &envelope,
            matches!(envelope.command, WhiteboardCommand::SyncPresence { .. }),
            "SyncPresence",
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

fn event_for(envelope: &CommandEnvelope) -> WhiteboardEvent {
    let tenant_id = envelope.context.actor.tenant_id.clone();
    match &envelope.command {
        WhiteboardCommand::OpenBoard { board_id } => WhiteboardEvent::BoardOpened {
            board_id: board_id.clone(),
            tenant_id,
        },
        WhiteboardCommand::AppendCanvasOp { board_id, .. } => {
            WhiteboardEvent::CanvasOpAppendQueued {
                board_id: board_id.clone(),
                tenant_id,
            }
        }
        WhiteboardCommand::RenderExport { board_id } => WhiteboardEvent::ExportRenderQueued {
            board_id: board_id.clone(),
            tenant_id,
        },
        WhiteboardCommand::SnapshotHistory { board_id } => WhiteboardEvent::HistorySnapshotQueued {
            board_id: board_id.clone(),
            tenant_id,
        },
        WhiteboardCommand::SyncPresence { board_id } => WhiteboardEvent::PresenceSyncRequested {
            board_id: board_id.clone(),
            tenant_id,
        },
        WhiteboardCommand::InstallTemplate { template_id, .. } => {
            WhiteboardEvent::TemplateInstallQueued {
                template_id: template_id.clone(),
                tenant_id,
            }
        }
    }
}

fn event_type(event: &WhiteboardEvent) -> &'static str {
    match event {
        WhiteboardEvent::BoardOpened { .. } => "board-opened",
        WhiteboardEvent::CanvasOpAppendQueued { .. } => "canvas-op-append-queued",
        WhiteboardEvent::ExportRenderQueued { .. } => "export-render-queued",
        WhiteboardEvent::HistorySnapshotQueued { .. } => "history-snapshot-queued",
        WhiteboardEvent::PresenceSyncRequested { .. } => "presence-sync-requested",
        WhiteboardEvent::TemplateInstallQueued { .. } => "template-install-queued",
    }
}
