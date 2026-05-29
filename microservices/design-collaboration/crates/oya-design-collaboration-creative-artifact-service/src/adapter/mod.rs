pub mod http {
    use crate::domain::{CommentThreadId, DesignFileId, DesignerId, TenantId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        DesignCollaborationPorts, DesignCollaborationService, OpenDesignFileCommand,
        PromoteTokenCommand, ResolveCommentCommand, UsecaseReceipt,
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
    pub struct OpenDesignFileHttpRequest {
        pub tenant_id: String,
        pub design_file_id: String,
        pub owner_id: String,
        pub title: String,
    }

    pub struct DesignHttpHandler;

    impl DesignHttpHandler {
        pub fn routes() -> Vec<RouteDescriptor> {
            vec![
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/design-files",
                    capability: "design.file.open",
                },
                RouteDescriptor {
                    method: HttpMethod::Put,
                    path: "/v1/design-files/{design_file_id}/comments/{thread_id}/resolve",
                    capability: "design.comment.resolve",
                },
                RouteDescriptor {
                    method: HttpMethod::Post,
                    path: "/v1/design-files/{design_file_id}/tokens/promote",
                    capability: "design.token.promote",
                },
            ]
        }

        pub fn open_design_file(
            service: &mut DesignCollaborationService<impl DesignCollaborationPorts>,
            request: OpenDesignFileHttpRequest,
        ) -> ServiceResult<UsecaseReceipt> {
            service.open_design_file(OpenDesignFileCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                design_file_id: DesignFileId::parse(request.design_file_id)?,
                owner_id: DesignerId::parse(request.owner_id)?,
                title: request.title,
            })
        }

        pub fn resolve_comment(
            service: &mut DesignCollaborationService<impl DesignCollaborationPorts>,
            tenant_id: String,
            design_file_id: String,
            comment_thread_id: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.resolve_comment(ResolveCommentCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                design_file_id: DesignFileId::parse(design_file_id)?,
                comment_thread_id: CommentThreadId::parse(comment_thread_id)?,
            })
        }

        pub fn promote_token(
            service: &mut DesignCollaborationService<impl DesignCollaborationPorts>,
            tenant_id: String,
            design_file_id: String,
            promoted_by: String,
        ) -> ServiceResult<UsecaseReceipt> {
            service.promote_token(PromoteTokenCommand {
                tenant_id: TenantId::parse(tenant_id)?,
                design_file_id: DesignFileId::parse(design_file_id)?,
                promoted_by: DesignerId::parse(promoted_by)?,
            })
        }
    }
}

pub mod grpc {
    use crate::domain::{DesignFileId, DesignerId, TenantId};
    use crate::error::ServiceResult;
    use crate::usecase::{
        DesignCollaborationPorts, DesignCollaborationService, OpenDesignFileCommand, UsecaseReceipt,
    };

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct DesignFileGrpcRequest {
        pub tenant_id: String,
        pub design_file_id: String,
        pub owner_id: String,
        pub title: String,
        pub request_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct DesignFileGrpcResponse {
        pub tenant_id: String,
        pub design_file_id: String,
        pub status: String,
        pub audit_event: String,
    }

    pub struct DesignGrpcHandler;

    impl DesignGrpcHandler {
        pub fn open_design_file(
            service: &mut DesignCollaborationService<impl DesignCollaborationPorts>,
            request: DesignFileGrpcRequest,
        ) -> ServiceResult<DesignFileGrpcResponse> {
            let receipt = service.open_design_file(OpenDesignFileCommand {
                tenant_id: TenantId::parse(request.tenant_id)?,
                design_file_id: DesignFileId::parse(request.design_file_id)?,
                owner_id: DesignerId::parse(request.owner_id)?,
                title: request.title,
            })?;
            Ok(Self::response_from_receipt(receipt))
        }

        fn response_from_receipt(receipt: UsecaseReceipt) -> DesignFileGrpcResponse {
            DesignFileGrpcResponse {
                tenant_id: receipt.tenant_id.as_str().to_owned(),
                design_file_id: receipt.design_file_id.as_str().to_owned(),
                status: format!("{:?}", receipt.status),
                audit_event: format!("{:?}", receipt.audit_event),
            }
        }
    }
}

pub mod asyncapi {
    use crate::domain::{AuditEventKind, DesignFileId, TenantId};
    use crate::error::ServiceResult;

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct FileOpenedEvent {
        pub tenant_id: TenantId,
        pub design_file_id: DesignFileId,
        pub audit_event: AuditEventKind,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct CommentResolvedEvent {
        pub tenant_id: TenantId,
        pub design_file_id: DesignFileId,
        pub comment_thread_id: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct TokenPromotedEvent {
        pub tenant_id: TenantId,
        pub design_file_id: DesignFileId,
        pub token_ref: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    pub struct PublishedMessage {
        pub topic: String,
        pub payload_json: String,
    }

    pub struct DesignAsyncApiHandler;

    impl DesignAsyncApiHandler {
        pub fn file_opened(
            prefix: &str,
            event: FileOpenedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.file.opened"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn comment_resolved(
            prefix: &str,
            event: CommentResolvedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.comment.resolved"),
                payload_json: serde_json::to_string(&event)?,
            })
        }

        pub fn token_promoted(
            prefix: &str,
            event: TokenPromotedEvent,
        ) -> ServiceResult<PublishedMessage> {
            Ok(PublishedMessage {
                topic: format!("{prefix}.token.promoted"),
                payload_json: serde_json::to_string(&event)?,
            })
        }
    }
}

pub mod memory {
    use std::collections::BTreeMap;

    use crate::domain::{AuditEventKind, Capability, DesignArtifact, DesignFileId, TenantId};
    use crate::error::{ServiceError, ServiceResult};
    use crate::usecase::{AuditPublisher, DesignArtifactRepository, PolicyAuthorizer};

    #[derive(Clone, Debug, Default)]
    pub struct InMemoryDesignPorts {
        artifacts: BTreeMap<String, DesignArtifact>,
        audit_events: Vec<String>,
        denied_capabilities: Vec<Capability>,
    }

    impl InMemoryDesignPorts {
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

        fn key(tenant_id: &TenantId, design_file_id: &DesignFileId) -> String {
            format!("{}::{}", tenant_id.as_str(), design_file_id.as_str())
        }
    }

    impl DesignArtifactRepository for InMemoryDesignPorts {
        fn put_artifact(&mut self, artifact: DesignArtifact) -> ServiceResult<DesignArtifact> {
            let key = Self::key(&artifact.tenant_id, &artifact.design_file_id);
            self.artifacts.insert(key, artifact.clone());
            Ok(artifact)
        }

        fn get_artifact(
            &self,
            tenant_id: &TenantId,
            design_file_id: &DesignFileId,
        ) -> ServiceResult<Option<DesignArtifact>> {
            Ok(self
                .artifacts
                .get(&Self::key(tenant_id, design_file_id))
                .cloned())
        }
    }

    impl PolicyAuthorizer for InMemoryDesignPorts {
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

    impl AuditPublisher for InMemoryDesignPorts {
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
