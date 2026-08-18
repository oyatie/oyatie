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
pub struct GrpcHandler;
impl GrpcHandler {
    pub fn methods() -> Vec<GrpcMethod> {
        vec![
            GrpcMethod {
                service: "oyatie.supply_chain_planning.v1.SupplyChainPlanningService",
                method: "SubmitCommand",
                request: "SubmitCommandRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.supply_chain_planning.v1.SupplyChainPlanningService",
                method: "Reconcile",
                request: "ReconcileRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.supply_chain_planning.v1.SupplyChainPlanningService",
                method: "ApplyGovernanceHold",
                request: "ApplyGovernanceHoldRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.supply_chain_planning.v1.SupplyChainPlanningService",
                method: "ExportEvidence",
                request: "ExportEvidenceRequest",
                response: "CommandReceipt",
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
            "scaffold requires command and read gRPC methods",
        ));
    }
    Ok(())
}
