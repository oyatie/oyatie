#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use adapter::{
    asyncapi::{CommentResolvedEvent, DesignAsyncApiHandler, FileOpenedEvent, TokenPromotedEvent},
    grpc::{DesignFileGrpcRequest, DesignFileGrpcResponse, DesignGrpcHandler},
    http::{DesignHttpHandler, HttpMethod, RouteDescriptor},
};
pub use config::{RuntimeProfile, ServiceConfig};
pub use domain::{
    ArchitectureLayer, ArtifactStatus, AuditEventKind, Capability, CommentThreadId, DataClass,
    DesignArtifact, DesignFileId, DesignPolicy, DesignerId, DomainInvariant, HandoffFormat,
    PermissionScope, TenantId, VersionId,
};
pub use error::{ServiceError, ServiceResult};
pub use usecase::{
    DesignCollaborationPorts, DesignCollaborationService, OpenDesignFile, OpenDesignFileCommand,
    PromoteToken, PromoteTokenCommand, ResolveComment, ResolveCommentCommand,
};

pub const MICROSERVICE: &str = "design-collaboration";
pub const BOUNDED_CONTEXT: &str = "creative-artifact";
pub const PRIMARY_CAPABILITY: &str = "artifact-version-collaboration";
pub const PRIMARY_ADR: &str = "ADR-0105";
pub const USECASE_RENAME_ADR: &str = "ADR-0106";
pub const OPENAPI_CONTRACT: &str = "microservices/design-collaboration/contracts/openapi-v1.yaml";
pub const GRPC_CONTRACT: &str =
    "microservices/design-collaboration/contracts/design-collaboration-v1.proto";
pub const ASYNCAPI_CONTRACT: &str = "microservices/design-collaboration/contracts/asyncapi-v1.yaml";

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ServiceDescriptor {
    pub microservice: &'static str,
    pub bounded_context: &'static str,
    pub primary_capability: &'static str,
    pub architecture_layers: Vec<&'static str>,
    pub contract_paths: Vec<&'static str>,
}

impl ServiceDescriptor {
    pub fn layer_count(&self) -> usize {
        self.architecture_layers.len()
    }

    pub fn contract_count(&self) -> usize {
        self.contract_paths.len()
    }

    pub fn includes_layer(&self, layer: ArchitectureLayer) -> bool {
        self.architecture_layers.contains(&layer.slug())
    }
}

pub fn descriptor() -> ServiceDescriptor {
    ServiceDescriptor {
        microservice: MICROSERVICE,
        bounded_context: BOUNDED_CONTEXT,
        primary_capability: PRIMARY_CAPABILITY,
        architecture_layers: ArchitectureLayer::all()
            .iter()
            .map(ArchitectureLayer::slug)
            .collect(),
        contract_paths: vec![OPENAPI_CONTRACT, GRPC_CONTRACT, ASYNCAPI_CONTRACT],
    }
}

pub fn default_design_artifact() -> DesignArtifact {
    DesignArtifact::new(
        TenantId::new("tenant-demo"),
        DesignFileId::new("design-file-demo"),
        DesignerId::new("designer-demo"),
        "Brand refresh file".to_owned(),
        ArtifactStatus::Draft,
    )
}

pub fn default_http_routes() -> Vec<RouteDescriptor> {
    DesignHttpHandler::routes()
}

pub fn default_domain_invariants() -> Vec<DomainInvariant> {
    vec![
        DomainInvariant::tenant_scoped("design files remain tenant scoped"),
        DomainInvariant::policy_checked("handoff export checks artifact entitlement"),
        DomainInvariant::audit_emitted("token promotion emits immutable evidence"),
        DomainInvariant::data_classified("creative artifacts declare collaboration data class"),
        DomainInvariant::region_bound("asset previews honor residency pack boundaries"),
        DomainInvariant::version_monotonic("file versions only advance by append"),
    ]
}

pub fn validate_scaffold() -> ServiceResult<()> {
    let descriptor = descriptor();
    if descriptor.layer_count() != 13 {
        return Err(ServiceError::InvariantViolation {
            invariant: "adr_0105_layer_count",
            details: format!("expected 13 layers, found {}", descriptor.layer_count()),
        });
    }
    if descriptor.contract_count() != 3 {
        return Err(ServiceError::InvariantViolation {
            invariant: "contract_surface_count",
            details: format!(
                "expected 3 contracts, found {}",
                descriptor.contract_count()
            ),
        });
    }
    Ok(())
}
