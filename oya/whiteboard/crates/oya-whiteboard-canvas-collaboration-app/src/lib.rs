#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use crate::config::ServiceConfig;
pub use crate::domain::{
    BoundedContext, Capability, TenantId, WhiteboardCommand, WhiteboardEvent, WhiteboardInvariant,
};
pub use crate::error::{Result, ServiceError, ServiceErrorKind};
pub use crate::usecase::{
    AuditPort, CanvasCollaborationInteractor, CommandEnvelope, CommandReceipt, EventPort,
    PolicyPort, RepositoryPort, UsecaseContext,
};

pub const MICROSERVICE: &str = "whiteboard";
pub const SERVICE_TITLE: &str = "Whiteboard";
pub const PACKAGE_NAME: &str = "oya-whiteboard-canvas-collaboration-app";
pub const BOUNDED_CONTEXT: &str = "canvas-collaboration";
pub const OWNER_TEAM: &str = "axis-whiteboard + council-product";
pub const OPENAPI_CONTRACT: &str = "contracts/openapi-v1.yaml";
pub const ASYNCAPI_CONTRACT: &str = "contracts/asyncapi-v1.yaml";
pub const GRPC_CONTRACT: &str = "contracts/whiteboard-v1.proto";
pub const PRIMARY_LAYER_ADR: &str = "ADR-0105";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceScaffold {
    pub microservice: &'static str,
    pub title: &'static str,
    pub package_name: &'static str,
    pub bounded_context: &'static str,
    pub owner_team: &'static str,
    pub contracts: ContractSet,
    pub layers: &'static [domain::Layer],
    pub capabilities: &'static [Capability],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractSet {
    pub openapi: &'static str,
    pub asyncapi: &'static str,
    pub grpc: &'static str,
}

pub fn scaffold() -> ServiceScaffold {
    ServiceScaffold {
        microservice: MICROSERVICE,
        title: SERVICE_TITLE,
        package_name: PACKAGE_NAME,
        bounded_context: BOUNDED_CONTEXT,
        owner_team: OWNER_TEAM,
        contracts: ContractSet {
            openapi: OPENAPI_CONTRACT,
            asyncapi: ASYNCAPI_CONTRACT,
            grpc: GRPC_CONTRACT,
        },
        layers: domain::LAYERS,
        capabilities: domain::CAPABILITIES,
    }
}

pub fn public_api_surface() -> Vec<&'static str> {
    vec![
        "CanvasCollaborationInteractor::open_board",
        "CanvasCollaborationInteractor::append_canvas_op",
        "CanvasCollaborationInteractor::render_export",
        "CanvasCollaborationInteractor::sync_presence",
        "adapter::http::WhiteboardHttpHandler",
        "adapter::grpc::WhiteboardGrpcHandler",
        "adapter::asyncapi::WhiteboardAsyncApiHandler",
    ]
}

pub mod prelude {
    pub use crate::adapter::AdapterRegistry;
    pub use crate::config::ServiceConfig;
    pub use crate::domain::{BoundedContext, Capability, TenantId, WhiteboardCommand};
    pub use crate::error::{Result, ServiceError, ServiceErrorKind};
    pub use crate::usecase::{
        CanvasCollaborationInteractor, CommandEnvelope, CommandReceipt, UsecaseContext,
    };
}
