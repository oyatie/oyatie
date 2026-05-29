pub mod asyncapi;
pub mod grpc;
pub mod http;

use crate::error::Result;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AdapterRegistry {
    pub http_routes: Vec<http::HttpRoute>,
    pub grpc_methods: Vec<grpc::GrpcMethod>,
    pub asyncapi_channels: Vec<asyncapi::AsyncApiChannel>,
}

impl AdapterRegistry {
    pub fn scaffolded() -> Self {
        Self {
            http_routes: http::HttpHandler::routes(),
            grpc_methods: grpc::GrpcHandler::methods(),
            asyncapi_channels: asyncapi::AsyncApiHandler::channels(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        http::validate_routes(&self.http_routes)?;
        grpc::validate_methods(&self.grpc_methods)?;
        asyncapi::validate_channels(&self.asyncapi_channels)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterSurface {
    HttpRest,
    Grpc,
    AsyncApi,
}
