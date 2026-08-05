use crate::domain::{Capability, IdempotencyKey, PrincipalId, ResourceId, TenantId};
use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
                service: "oyatie.quality_management.v1.QualityManagementService",
                method: "MutateInspectionPlan",
                request: "InspectionPlanCommand",
                response: "InspectionPlanResult",
            },
            GrpcMethod {
                service: "oyatie.quality_management.v1.QualityManagementService",
                method: "MutateInspectionLot",
                request: "InspectionLotCommand",
                response: "InspectionLotResult",
            },
            GrpcMethod {
                service: "oyatie.quality_management.v1.QualityManagementService",
                method: "MutateCertificateOfAnalysis",
                request: "CertificateOfAnalysisCommand",
                response: "CertificateOfAnalysisResult",
            },
            GrpcMethod {
                service: "oyatie.quality_management.v1.QualityManagementService",
                method: "MutateQualityNotification",
                request: "QualityNotificationCommand",
                response: "QualityNotificationResult",
            },
            GrpcMethod {
                service: "oyatie.quality_management.v1.QualityManagementService",
                method: "MutateQualityHold",
                request: "QualityHoldCommand",
                response: "QualityHoldResult",
            },
            GrpcMethod {
                service: "oyatie.quality_management.v1.QualityManagementService",
                method: "MutateAuditEvidence",
                request: "AuditEvidenceCommand",
                response: "AuditEvidenceResult",
            },
        ]
    }

    pub fn handle(request: GrpcRequest) -> Result<GrpcResponse> {
        let tenant_id = TenantId::new(request.tenant_id)?;
        let capability = capability_from_method(&request.method)?;
        PrincipalId::new(required_payload_str(&request.payload_json, "principal_id")?)?;
        IdempotencyKey::new(required_payload_str(
            &request.payload_json,
            "idempotency_key",
        )?)?;
        ResourceId::new(required_payload_str(&request.payload_json, "resource_id")?)?;

        Ok(GrpcResponse {
            accepted: true,
            payload_json: json!({
                "accepted": true,
                "tenant_id": tenant_id.as_str(),
                "capability": capability_wire_name(capability),
                "audit_event_type": "command-accepted",
                "runtime_deployed": false,
                "durable_persistence": false
            }),
        })
    }
}

fn required_payload_str<'a>(payload: &'a Value, field: &'static str) -> Result<&'a str> {
    payload.get(field).and_then(Value::as_str).ok_or_else(|| {
        ServiceError::validation(
            field,
            format!("{field} is required for gRPC command fixture"),
        )
    })
}

fn capability_from_method(method: &str) -> Result<Capability> {
    match method {
        "MutateInspectionPlan" => Ok(Capability::InspectionPlan),
        "MutateInspectionLot" => Ok(Capability::InspectionLot),
        "MutateCertificateOfAnalysis" => Ok(Capability::CertificateAnalysis),
        "MutateQualityNotification" => Ok(Capability::QualityNotification),
        "MutateQualityHold" => Ok(Capability::QualityHold),
        "MutateAuditEvidence" => Ok(Capability::AuditEvidence),
        _ => Err(ServiceError::validation(
            "method",
            "unknown quality-management gRPC method fixture",
        )),
    }
}

fn capability_wire_name(capability: Capability) -> &'static str {
    match capability {
        Capability::InspectionPlan => "inspection-plan",
        Capability::InspectionLot => "inspection-lot",
        Capability::QualityNotification => "quality-notification",
        Capability::QualityHold => "quality-hold",
        Capability::CertificateAnalysis => "certificate-analysis",
        Capability::AuditEvidence => "audit-evidence",
    }
}

pub fn validate_methods(methods: &[GrpcMethod]) -> Result<()> {
    if methods.len() < 6 {
        return Err(ServiceError::validation(
            "grpc_methods",
            "scaffold requires all six Quality Management gRPC methods",
        ));
    }
    for method in [
        "MutateInspectionPlan",
        "MutateInspectionLot",
        "MutateCertificateOfAnalysis",
        "MutateQualityNotification",
        "MutateQualityHold",
        "MutateAuditEvidence",
    ] {
        if !methods.iter().any(|candidate| candidate.method == method) {
            return Err(ServiceError::validation(
                "grpc_methods",
                format!("missing gRPC method fixture {method}"),
            ));
        }
    }
    Ok(())
}
