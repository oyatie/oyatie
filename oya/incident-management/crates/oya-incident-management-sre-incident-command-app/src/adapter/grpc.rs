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

pub struct IncidentManagementGrpcHandler;

impl IncidentManagementGrpcHandler {
    pub fn methods() -> Vec<GrpcMethod> {
        vec![
            GrpcMethod {
                service: "oyatie.incident_management.v1.IncidentCommandService",
                method: "DispatchPage",
                request: "DispatchPageRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.incident_management.v1.IncidentCommandService",
                method: "EvaluateEscalation",
                request: "EvaluateEscalationRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.incident_management.v1.IncidentCommandService",
                method: "OpenIncidentRoom",
                request: "OpenIncidentRoomRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.incident_management.v1.IncidentCommandService",
                method: "ReadIncidentState",
                request: "ReadIncidentStateRequest",
                response: "IncidentStateView",
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
            "incident management scaffold requires command and read gRPC methods",
        ));
    }
    Ok(())
}
