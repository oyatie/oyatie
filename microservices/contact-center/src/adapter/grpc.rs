use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GrpcMethod {
    pub service: &'static str,
    pub method: &'static str,
    pub request: &'static str,
    pub response: &'static str,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GrpcRequest {
    pub tenant_id: String,
    pub method: String,
    pub payload_json: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GrpcResponse {
    pub accepted: bool,
    pub payload_json: serde_json::Value,
}

pub struct ContactCenterGrpcHandler;

impl ContactCenterGrpcHandler {
    pub fn methods() -> Vec<GrpcMethod> {
        vec![
            GrpcMethod {
                service: "oyatie.contact_center.v1.VoiceRoutingService",
                method: "RouteVoiceContact",
                request: "RouteVoiceContactRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.contact_center.v1.VoiceRoutingService",
                method: "RebalanceQueue",
                request: "RebalanceQueueRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.contact_center.v1.VoiceRoutingService",
                method: "RecordConsent",
                request: "RecordConsentRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.contact_center.v1.VoiceRoutingService",
                method: "ReadQueueState",
                request: "ReadQueueStateRequest",
                response: "QueueStateView",
            },
        ]
    }

    pub fn handle(_request: GrpcRequest) -> Result<GrpcResponse> {
        Err(ServiceError::contract_stub("grpc"))
    }
}

pub fn validate_methods(methods: &[GrpcMethod]) -> Result<()> {
    if methods.len() < 4 {
        return Err(ServiceError::validation(
            "grpc_methods",
            "contact center scaffold requires command and read gRPC methods",
        ));
    }
    Ok(())
}
