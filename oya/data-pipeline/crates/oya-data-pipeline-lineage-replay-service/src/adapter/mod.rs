pub mod http {
    use crate::domain::{DeadLetterBatchId, PipelineId, SourceId, TenantId, TransformId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        ApproveDeadLetterReplayCommand, DataPipelinePorts, DataPipelineService,
        RecordLineageCommand, StartIngestRunCommand, UsecaseReceipt,
    };

    #[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub enum HttpMethod {
        Get,
        Post,
        Put,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct RouteDescriptor {
        pub method: HttpMethod,
        pub path: &'static str,
        pub capability: &'static str,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct StartIngestRunHttpRequest {
        pub tenant_id: String,
        pub pipeline_id: String,
        pub source_id: String,
        pub name: String,
    }

    pub struct DataPipelineHttpHandler;

    impl DataPipelineHttpHandler {
        pub fn routes() -> Vec<RouteDescriptor> {
            vec![
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/ingest-runs",
                    capability: "pipeline.ingest.start",
                },
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/pipelines/{pipeline_id}/lineage",
                    capability: "pipeline.lineage.record",
                },
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/pipelines/{pipeline_id}/dead-letter/{batch_id}/approve",
                    capability: "pipeline.dead_letter.replay.approve",
                },
            ]
        }

        pub fn start_ingest_run(
            service: &mut DataPipelineService<impl DataPipelinePorts>,
            request: StartIngestRunHttpRequest,
        ) -> ServiceResult<UsecaseReceipt> {
            service.start_ingest_run(StartIngestRunCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                pipeline_id: PipelineId::parse(request.pipeline_id)?,
                source_id: SourceId::parse(request.source_id)?,
                name: request.name,
            })
        }

        pub fn record_lineage(
            service: &mut DataPipelineService<impl DataPipelinePorts>,
            tenant_id: String,
            pipeline_id: String,
            transform_id: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.record_lineage(RecordLineageCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                pipeline_id: PipelineId::parse(pipeline_id)?,
                transform_id: TransformId::parse(transform_id)?,
            })
        }

        pub fn approve_dead_letter_replay(
            service: &mut DataPipelineService<impl DataPipelinePorts>,
            tenant_id: String,
            pipeline_id: String,
            dead_letter_batch_id: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.approve_dead_letter_replay(ApproveDeadLetterReplayCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                pipeline_id: PipelineId::parse(pipeline_id)?,
                dead_letter_batch_id: DeadLetterBatchId::parse(dead_letter_batch_id)?,
            })
        }
    }
}

pub mod grpc {
    use crate::domain::{PipelineId, SourceId, TenantId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        DataPipelinePorts, DataPipelineService, StartIngestRunCommand, UsecaseReceipt,
    };

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct PipelineGrpcRequest {
        pub tenant_id: String,
        pub pipeline_id: String,
        pub source_id: String,
        pub name: String,
        pub request_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct PipelineGrpcResponse {
        pub tenant_id: String,
        pub pipeline_id: String,
        pub status: String,
        pub audit_event: String,
    }

    pub struct DataPipelineGrpcHandler;

    impl DataPipelineGrpcHandler {
        pub fn start_ingest_run(
            service: &mut DataPipelineService<impl DataPipelinePorts>,
            request: PipelineGrpcRequest,
        ) -> ServiceResult<PipelineGrpcResponse> {
            let receipt = service.start_ingest_run(StartIngestRunCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                pipeline_id: PipelineId::parse(request.pipeline_id)?,
                source_id: SourceId::parse(request.source_id)?,
                name: request.name,
            })?;
            Ok(Self::response_from_receipt(receipt))
        }

        fn response_from_receipt(receipt: UsecaseReceipt) -> PipelineGrpcResponse {
            PipelineGrpcResponse {
                tenant_id: receipt.tenant_id.as_str().to_owned(),
                pipeline_id: receipt.pipeline_id.as_str().to_owned(),
                status: format!("{:?}", receipt.status),
                audit_event: format!("{:?}", receipt.audit_event),
            }
        }
    }
}

pub mod asyncapi {
    use crate::domain::{AuditEventKind, PipelineId, TenantId};
    use crate::error::ServiceResult;

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct IngestRunStartedEvent {
        pub tenant_id: TenantId,
        pub pipeline_id: PipelineId,
        pub audit_event: AuditEventKind,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct LineageCapturedEvent {
        pub tenant_id: TenantId,
        pub pipeline_id: PipelineId,
        pub transform_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct DeadLetterReplayApprovedEvent {
        pub tenant_id: TenantId,
        pub pipeline_id: PipelineId,
        pub batch_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct PublishedMessage {
        pub topic: String,
        pub payload_json: String,
    }

    pub struct DataPipelineAsyncApiHandler;

    impl DataPipelineAsyncApiHandler {
        pub fn ingest_run_started(
            prefix: &str,
            event: IngestRunStartedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.ingest_run.started"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn lineage_captured(
            prefix: &str,
            event: LineageCapturedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.lineage.captured"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn dead_letter_replay_approved(
            prefix: &str,
            event: DeadLetterReplayApprovedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.dead_letter.replay_approved"),
                payload_json: serde_json::to_string(&event)?,
            })
        }
    }
}

pub mod memory {
    use std::collections::BTreeMap;

    use crate::domain::{AuditEventKind, Capability, IngestRun, PipelineId, TenantId};
    use crate::error::{ServiceError, ServiceResult};
    use crate::usecase::{AuditPublisher, PipelineRepository, PolicyAuthorizer};

    #[derive(Clone, Debug, Default)]
    pub struct InMemoryDataPipelinePorts {
        runs: BTreeMap<String, IngestRun>,
        audit_events: Vec<String>,
        denied_capabilities: Vec<Capability>,
    }

    impl InMemoryDataPipelinePorts {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn deny(mut self, capability: Capability) -> Self {
            self.denied_capabilities.push(capability);
            self
        }

        pub fn audit_events(&self) -> &[String] {
            &self.audit_events
        }

        fn key(tenant_id: &TenantId, pipeline_id: &PipelineId) -> String {
            format!("{}::{}", tenant_id.as_str(), pipeline_id.as_str())
        }
    }

    impl PipelineRepository for InMemoryDataPipelinePorts {
        fn put_ingest_run(&mut self, run: IngestRun) -> ServiceResult<IngestRun> {
            let key = Self::key(&run.tenant_id, &run.pipeline_id);
            self.runs.insert(key, run.clone());
            Ok(run)
        }

        fn get_ingest_run(
            &self,
            tenant_id: &TenantId,
            pipeline_id: &PipelineId,
        ) -> ServiceResult<Option<IngestRun>> {
            Ok(self.runs.get(&Self::key(tenant_id, pipeline_id)).cloned())
        }
    }

    impl PolicyAuthorizer for InMemoryDataPipelinePorts {
        fn authorize(&self, _tenant_id: &TenantId, capability: Capability) -> ServiceResult<()> {
            if self.denied_capabilities.contains(&capability) {
                Err(ServiceError::policy_denied(
                    capability.action_slug(),
                    "capability denied by in-memory policy",
                ))
            } else {
                Ok(())
            }
        }
    }

    impl AuditPublisher for InMemoryDataPipelinePorts {
        fn publish_audit(
            &mut self,
            tenant_id: &TenantId,
            event_kind: AuditEventKind,
            subject: &str,
        ) -> ServiceResult<()> {
            self.audit_events
                .push(format!("{}::{event_kind:?}::{subject}", tenant_id.as_str()));
            Ok(())
        }
    }
}
