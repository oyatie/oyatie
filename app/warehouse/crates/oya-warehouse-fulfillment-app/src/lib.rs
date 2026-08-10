#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use crate::config::ServiceConfig;
pub use crate::domain::{
    Capability, Layer, ServiceCommand, ServiceEvent, ServiceInvariant, TenantId,
};
pub use crate::error::{Result, ServiceError, ServiceErrorKind};
pub use crate::usecase::{
    AuditPort, CommandEnvelope, CommandReceipt, EventPort, PolicyPort, RepositoryPort,
    ServiceInteractor, UsecaseContext,
};

pub const MICROSERVICE: &str = "warehouse";
pub const SERVICE_TITLE: &str = "Warehouse";
pub const PACKAGE_NAME: &str = "oya-warehouse-fulfillment-app";
pub const BOUNDED_CONTEXT: &str = "warehouse-fulfillment";
pub const OWNER_TEAM: &str = "axis-erp-warehouse + council-product";
pub const OPENAPI_CONTRACT: &str = "contracts/openapi-v1.yaml";
pub const ASYNCAPI_CONTRACT: &str = "contracts/asyncapi-v1.yaml";
pub const GRPC_CONTRACT: &str = "contracts/warehouse-v1.proto";
pub const PRIMARY_LAYER_ADR: &str = "ADR-0105";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceScaffold {
    pub microservice: &'static str,
    pub title: &'static str,
    pub package_name: &'static str,
    pub bounded_context: &'static str,
    pub owner_team: &'static str,
    pub contracts: ContractSet,
    pub layers: &'static [Layer],
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
        "ServiceInteractor::submit_command",
        "ServiceInteractor::reconcile",
        "ServiceInteractor::apply_governance_hold",
        "adapter::http::HttpHandler",
        "adapter::grpc::GrpcHandler",
        "adapter::asyncapi::AsyncApiHandler",
    ]
}

pub mod prelude {
    pub use crate::adapter::AdapterRegistry;
    pub use crate::config::ServiceConfig;
    pub use crate::domain::{Capability, ServiceCommand, ServiceEvent, TenantId};
    pub use crate::error::{Result, ServiceError, ServiceErrorKind};
    pub use crate::usecase::{CommandEnvelope, CommandReceipt, ServiceInteractor, UsecaseContext};
}
