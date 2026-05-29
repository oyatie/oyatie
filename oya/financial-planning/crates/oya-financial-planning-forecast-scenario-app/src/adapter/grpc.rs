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

pub struct FinancialPlanningGrpcHandler;

impl FinancialPlanningGrpcHandler {
    pub fn methods() -> Vec<GrpcMethod> {
        vec![
            GrpcMethod {
                service: "oyatie.financial_planning.v1.ForecastScenarioService",
                method: "OpenForecastVersion",
                request: "OpenForecastVersionRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.financial_planning.v1.ForecastScenarioService",
                method: "RecalculateScenario",
                request: "RecalculateScenarioRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.financial_planning.v1.ForecastScenarioService",
                method: "ExplainVariance",
                request: "ExplainVarianceRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.financial_planning.v1.ForecastScenarioService",
                method: "ReadForecastState",
                request: "ReadForecastStateRequest",
                response: "ForecastStateView",
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
            "financial planning scaffold requires command and read gRPC methods",
        ));
    }
    Ok(())
}
