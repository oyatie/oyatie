use crate::domain::{
    AuditEventKind, Capability, DeadLetterBatchId, IngestRun, PipelineId, PipelineStatus, SourceId,
    TenantId, TransformId,
};
use crate::error::{ServiceError, ServiceResult};

pub trait PipelineRepository {
    fn put_ingest_run(&mut self, run: IngestRun) -> ServiceResult<IngestRun>;
    fn get_ingest_run(
        &self,
        tenant_id: &TenantId,
        pipeline_id: &PipelineId,
    ) -> ServiceResult<Option<IngestRun>>;
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

pub trait DataPipelinePorts: PipelineRepository + PolicyAuthorizer + AuditPublisher {}

impl<T> DataPipelinePorts for T where T: PipelineRepository + PolicyAuthorizer + AuditPublisher {}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct StartIngestRunCommand {
    pub tenant_id: TenantId,
    pub pipeline_id: PipelineId,
    pub source_id: SourceId,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RecordLineageCommand {
    pub tenant_id: TenantId,
    pub pipeline_id: PipelineId,
    pub transform_id: TransformId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ApproveDeadLetterReplayCommand {
    pub tenant_id: TenantId,
    pub pipeline_id: PipelineId,
    pub dead_letter_batch_id: DeadLetterBatchId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UsecaseReceipt {
    pub tenant_id: TenantId,
    pub pipeline_id: PipelineId,
    pub audit_event: AuditEventKind,
    pub status: PipelineStatus,
}

pub struct StartIngestRun;

impl StartIngestRun {
    pub fn execute(
        ports: &mut impl DataPipelinePorts,
        command: StartIngestRunCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::IngestStart)?;
        let run = IngestRun::new(
            command.tenant_id.clone(),
            command.pipeline_id.clone(),
            command.source_id,
            command.name,
            PipelineStatus::Draft,
        )
        .start()?;
        run.validate()?;
        let run = ports.put_ingest_run(run)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::IngestRunStarted,
            command.pipeline_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: run.tenant_id,
            pipeline_id: run.pipeline_id,
            audit_event: AuditEventKind::IngestRunStarted,
            status: run.status,
        })
    }
}

pub struct RecordLineage;

impl RecordLineage {
    pub fn execute(
        ports: &mut impl DataPipelinePorts,
        command: RecordLineageCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::LineageRecord)?;
        let run = ports
            .get_ingest_run(&command.tenant_id, &command.pipeline_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "pipeline_repository",
            })?
            .record_lineage()?;
        let run = ports.put_ingest_run(run)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::LineageCaptured,
            command.transform_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: run.tenant_id,
            pipeline_id: run.pipeline_id,
            audit_event: AuditEventKind::LineageCaptured,
            status: run.status,
        })
    }
}

pub struct ApproveDeadLetterReplay;

impl ApproveDeadLetterReplay {
    pub fn execute(
        ports: &mut impl DataPipelinePorts,
        command: ApproveDeadLetterReplayCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::DeadLetterReplayApprove)?;
        let mut run = ports
            .get_ingest_run(&command.tenant_id, &command.pipeline_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "pipeline_repository",
            })?;
        run.status = PipelineStatus::ReplayPending;
        let run = run.approve_replay()?;
        let run = ports.put_ingest_run(run)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::DeadLetterReplayApproved,
            command.dead_letter_batch_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: run.tenant_id,
            pipeline_id: run.pipeline_id,
            audit_event: AuditEventKind::DeadLetterReplayApproved,
            status: run.status,
        })
    }
}

pub struct DataPipelineService<P> {
    ports: P,
}

impl<P> DataPipelineService<P>
where
    P: DataPipelinePorts,
{
    pub fn new(ports: P) -> Self {
        Self { ports }
    }

    pub fn start_ingest_run(
        &mut self,
        command: StartIngestRunCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        StartIngestRun::execute(&mut self.ports, command)
    }

    pub fn record_lineage(
        &mut self,
        command: RecordLineageCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        RecordLineage::execute(&mut self.ports, command)
    }

    pub fn approve_dead_letter_replay(
        &mut self,
        command: ApproveDeadLetterReplayCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ApproveDeadLetterReplay::execute(&mut self.ports, command)
    }

    pub fn into_ports(self) -> P {
        self.ports
    }
}
