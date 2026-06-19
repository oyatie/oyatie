#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod adapter;
pub mod config;
pub mod domain;
pub mod error;
pub mod usecase;

pub use crate::config::ServiceConfig;
pub use crate::domain::{
    BoundedContext, Capability, ContactCenterCommand, ContactCenterEvent, ContactCenterInvariant,
    TenantId,
};
pub use crate::error::{Result, ServiceError, ServiceErrorKind};
pub use crate::usecase::{
    AuditPort, CommandEnvelope, CommandReceipt, EventPort, PolicyPort, RepositoryPort,
    UsecaseContext, VoiceRoutingInteractor,
};

pub const MICROSERVICE: &str = "contact-center";
pub const SERVICE_TITLE: &str = "Contact Center";
pub const PACKAGE_NAME: &str = "comms-contact-center-voice-routing";
pub const BOUNDED_CONTEXT: &str = "voice-routing";
pub const OWNER_TEAM: &str = "axis-contact-center + council-product";
pub const OPENAPI_CONTRACT: &str = "contracts/openapi-v1.yaml";
pub const ASYNCAPI_CONTRACT: &str = "contracts/asyncapi-v1.yaml";
pub const GRPC_CONTRACT: &str = "contracts/contact-center-v1.proto";
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
        "VoiceRoutingInteractor::route_voice_contact",
        "VoiceRoutingInteractor::rebalance_queue",
        "VoiceRoutingInteractor::record_consent",
        "VoiceRoutingInteractor::sync_agent_state",
        "adapter::http::ContactCenterHttpHandler",
        "adapter::grpc::ContactCenterGrpcHandler",
        "adapter::asyncapi::ContactCenterAsyncApiHandler",
    ]
}

pub mod prelude {
    pub use crate::adapter::AdapterRegistry;
    pub use crate::config::ServiceConfig;
    pub use crate::domain::{BoundedContext, Capability, ContactCenterCommand, TenantId};
    pub use crate::error::{Result, ServiceError, ServiceErrorKind};
    pub use crate::usecase::{
        CommandEnvelope, CommandReceipt, UsecaseContext, VoiceRoutingInteractor,
    };
}
