#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use crate::config::ServiceConfig;
pub use crate::domain::{
    BoundedContext, Capability, ContractLifecycleCommand, ContractLifecycleEvent,
    ContractLifecycleInvariant, TenantId,
};
pub use crate::error::{Result, ServiceError, ServiceErrorKind};
pub use crate::usecase::{
    AuditPort, CommandEnvelope, CommandReceipt, ContractObligationInteractor, EventPort,
    PolicyPort, RepositoryPort, UsecaseContext,
};

pub const MICROSERVICE: &str = "contract-lifecycle-management";
pub const SERVICE_TITLE: &str = "Contract Lifecycle Management";
pub const PACKAGE_NAME: &str = "oya-contract-lifecycle-management-contract-obligation-app";
pub const BOUNDED_CONTEXT: &str = "contract-obligation";
pub const OWNER_TEAM: &str = "axis-contract-lifecycle-management + council-product";
pub const OPENAPI_CONTRACT: &str = "contracts/openapi-v1.yaml";
pub const ASYNCAPI_CONTRACT: &str = "contracts/asyncapi-v1.yaml";
pub const GRPC_CONTRACT: &str = "contracts/contract-lifecycle-management-v1.proto";
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
        "ContractObligationInteractor::create_contract_draft",
        "ContractObligationInteractor::evaluate_clause_policy",
        "ContractObligationInteractor::route_approval",
        "ContractObligationInteractor::track_obligation",
        "adapter::http::ContractLifecycleHttpHandler",
        "adapter::grpc::ContractLifecycleGrpcHandler",
        "adapter::asyncapi::ContractLifecycleAsyncApiHandler",
    ]
}

pub mod prelude {
    pub use crate::adapter::AdapterRegistry;
    pub use crate::config::ServiceConfig;
    pub use crate::domain::{BoundedContext, Capability, ContractLifecycleCommand, TenantId};
    pub use crate::error::{Result, ServiceError, ServiceErrorKind};
    pub use crate::usecase::{
        CommandEnvelope, CommandReceipt, ContractObligationInteractor, UsecaseContext,
    };
}
