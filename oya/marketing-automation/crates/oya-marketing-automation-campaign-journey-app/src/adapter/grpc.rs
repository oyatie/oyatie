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

pub struct MarketingAutomationGrpcHandler;

impl MarketingAutomationGrpcHandler {
    pub fn methods() -> Vec<GrpcMethod> {
        vec![
            GrpcMethod {
                service: "oyatie.marketing_automation.v1.CampaignJourneyService",
                method: "LaunchJourney",
                request: "LaunchJourneyRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.marketing_automation.v1.CampaignJourneyService",
                method: "SyncSegment",
                request: "SyncSegmentRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.marketing_automation.v1.CampaignJourneyService",
                method: "EnforceSuppression",
                request: "EnforceSuppressionRequest",
                response: "CommandReceipt",
            },
            GrpcMethod {
                service: "oyatie.marketing_automation.v1.CampaignJourneyService",
                method: "ReadJourneyState",
                request: "ReadJourneyStateRequest",
                response: "JourneyStateView",
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
            "marketing automation scaffold requires command and read gRPC methods",
        ));
    }
    Ok(())
}
