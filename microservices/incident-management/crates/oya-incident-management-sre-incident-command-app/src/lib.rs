#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use crate::config::ServiceConfig;
pub use crate::domain::{
    BoundedContext, Capability, IncidentManagementCommand, IncidentManagementEvent,
    IncidentManagementInvariant, TenantId,
};
pub use crate::error::{Result, ServiceError, ServiceErrorKind};
pub use crate::usecase::{
    AuditPort, CommandEnvelope, CommandReceipt, EventPort, IncidentCommandInteractor, PolicyPort,
    RepositoryPort, UsecaseContext,
};

pub const MICROSERVICE: &str = "incident-management";
pub const SERVICE_TITLE: &str = "Incident Management";
pub const PACKAGE_NAME: &str = "oya-incident-management-sre-incident-command-app";
pub const BOUNDED_CONTEXT: &str = "sre-incident-command";
pub const OWNER_TEAM: &str = "axis-incident-management + council-product";
pub const OPENAPI_CONTRACT: &str = "contracts/openapi-v1.yaml";
pub const ASYNCAPI_CONTRACT: &str = "contracts/asyncapi-v1.yaml";
pub const GRPC_CONTRACT: &str = "contracts/incident-management-v1.proto";
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
        "IncidentCommandInteractor::dispatch_page",
        "IncidentCommandInteractor::evaluate_escalation",
        "IncidentCommandInteractor::open_incident_room",
        "IncidentCommandInteractor::sync_status_page",
        "adapter::http::IncidentManagementHttpHandler",
        "adapter::grpc::IncidentManagementGrpcHandler",
        "adapter::asyncapi::IncidentManagementAsyncApiHandler",
    ]
}

pub mod prelude {
    pub use crate::adapter::AdapterRegistry;
    pub use crate::config::ServiceConfig;
    pub use crate::domain::{BoundedContext, Capability, IncidentManagementCommand, TenantId};
    pub use crate::error::{Result, ServiceError, ServiceErrorKind};
    pub use crate::usecase::{
        CommandEnvelope, CommandReceipt, IncidentCommandInteractor, UsecaseContext,
    };
}
