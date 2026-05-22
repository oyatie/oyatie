use crate::domain::{
    ArtifactStatus, AuditEventKind, Capability, CommentThreadId, DesignArtifact, DesignFileId,
    DesignerId, TenantId,
};
use crate::error::{ServiceError, ServiceResult};

pub trait DesignArtifactRepository {
    fn put_artifact(&mut self, artifact: DesignArtifact) -> ServiceResult<DesignArtifact>;
    fn get_artifact(
        &self,
        tenant_id: &TenantId,
        design_file_id: &DesignFileId,
    ) -> ServiceResult<Option<DesignArtifact>>;
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

pub trait DesignCollaborationPorts:
    DesignArtifactRepository + PolicyAuthorizer + AuditPublisher
{
}

impl<T> DesignCollaborationPorts for T where
    T: DesignArtifactRepository + PolicyAuthorizer + AuditPublisher
{
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OpenDesignFileCommand {
    pub tenant_id: TenantId,
    pub design_file_id: DesignFileId,
    pub owner_id: DesignerId,
    pub title: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ResolveCommentCommand {
    pub tenant_id: TenantId,
    pub design_file_id: DesignFileId,
    pub comment_thread_id: CommentThreadId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PromoteTokenCommand {
    pub tenant_id: TenantId,
    pub design_file_id: DesignFileId,
    pub promoted_by: DesignerId,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UsecaseReceipt {
    pub tenant_id: TenantId,
    pub design_file_id: DesignFileId,
    pub audit_event: AuditEventKind,
    pub status: ArtifactStatus,
}

pub struct OpenDesignFile;

impl OpenDesignFile {
    pub fn execute(
        ports: &mut impl DesignCollaborationPorts,
        command: OpenDesignFileCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::FileOpen)?;
        let artifact = DesignArtifact::new(
            command.tenant_id.clone(),
            command.design_file_id.clone(),
            command.owner_id,
            command.title,
            ArtifactStatus::Draft,
        )
        .open()?;
        artifact.validate()?;
        let artifact = ports.put_artifact(artifact)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::FileOpened,
            command.design_file_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: artifact.tenant_id,
            design_file_id: artifact.design_file_id,
            audit_event: AuditEventKind::FileOpened,
            status: artifact.status,
        })
    }
}

pub struct ResolveComment;

impl ResolveComment {
    pub fn execute(
        ports: &mut impl DesignCollaborationPorts,
        command: ResolveCommentCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::CommentResolve)?;
        let artifact = ports
            .get_artifact(&command.tenant_id, &command.design_file_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "design_artifact_repository",
            })?
            .resolve_comment()?;
        let artifact = ports.put_artifact(artifact)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::CommentResolved,
            command.comment_thread_id.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: artifact.tenant_id,
            design_file_id: artifact.design_file_id,
            audit_event: AuditEventKind::CommentResolved,
            status: artifact.status,
        })
    }
}

pub struct PromoteToken;

impl PromoteToken {
    pub fn execute(
        ports: &mut impl DesignCollaborationPorts,
        command: PromoteTokenCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ports.authorize(&command.tenant_id, Capability::TokenPromote)?;
        let mut artifact = ports
            .get_artifact(&command.tenant_id, &command.design_file_id)?
            .ok_or(ServiceError::PortUnavailable {
                port: "design_artifact_repository",
            })?;
        artifact.status = ArtifactStatus::ReviewRequested;
        let artifact = artifact.promote_token()?;
        let artifact = ports.put_artifact(artifact)?;
        ports.publish_audit(
            &command.tenant_id,
            AuditEventKind::TokenPromoted,
            command.promoted_by.as_str(),
        )?;
        Ok(UsecaseReceipt {
            tenant_id: artifact.tenant_id,
            design_file_id: artifact.design_file_id,
            audit_event: AuditEventKind::TokenPromoted,
            status: artifact.status,
        })
    }
}

pub struct DesignCollaborationService<P> {
    ports: P,
}

impl<P> DesignCollaborationService<P>
where
    P: DesignCollaborationPorts,
{
    pub fn new(ports: P) -> Self {
        Self { ports }
    }

    pub fn open_design_file(
        &mut self,
        command: OpenDesignFileCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        OpenDesignFile::execute(&mut self.ports, command)
    }

    pub fn resolve_comment(
        &mut self,
        command: ResolveCommentCommand,
    ) -> ServiceResult<UsecaseReceipt> {
        ResolveComment::execute(&mut self.ports, command)
    }

    pub fn promote_token(&mut self, command: PromoteTokenCommand) -> ServiceResult<UsecaseReceipt> {
        PromoteToken::execute(&mut self.ports, command)
    }

    pub fn into_ports(self) -> P {
        self.ports
    }
}
