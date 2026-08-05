use crate::domain::{Capability, IdempotencyKey, PrincipalId, RequestId, ResourceId, TenantId};
use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HttpRoute {
    pub method: &'static str,
    pub path: &'static str,
    pub capability: &'static str,
    pub idempotent: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HttpRequest {
    pub tenant_id: String,
    pub principal_id: String,
    pub request_id: String,
    pub idempotency_key: String,
    pub body: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HttpResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

pub struct HttpHandler;

impl HttpHandler {
    pub fn routes() -> Vec<HttpRoute> {
        vec![
            HttpRoute {
                method: "POST",
                path: "/v1/quality-management/inspection-plan",
                capability: "inspection-plan",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/quality-management/inspection-lot",
                capability: "inspection-lot",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/quality-management/certificate-of-analysis",
                capability: "certificate-analysis",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/quality-management/quality-notification",
                capability: "quality-notification",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/quality-management/quality-hold",
                capability: "quality-hold",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/quality-management/audit-evidence",
                capability: "audit-evidence",
                idempotent: true,
            },
        ]
    }

    pub fn handle(request: HttpRequest) -> Result<HttpResponse> {
        let tenant_id = TenantId::new(request.tenant_id)?;
        PrincipalId::new(request.principal_id)?;
        RequestId::new(request.request_id)?;
        IdempotencyKey::new(request.idempotency_key)?;
        let capability = capability_from_value(required_body_str(&request.body, "capability")?)?;
        ResourceId::new(required_body_str(&request.body, "resource_id")?)?;

        Ok(HttpResponse {
            status: 202,
            body: accepted_body(tenant_id.as_str(), capability),
        })
    }
}

fn required_body_str<'a>(body: &'a Value, field: &'static str) -> Result<&'a str> {
    body.get(field).and_then(Value::as_str).ok_or_else(|| {
        ServiceError::validation(
            field,
            format!("{field} is required for HTTP command fixture"),
        )
    })
}

fn capability_from_value(value: &str) -> Result<Capability> {
    match value {
        "inspection-plan" => Ok(Capability::InspectionPlan),
        "inspection-lot" => Ok(Capability::InspectionLot),
        "quality-notification" => Ok(Capability::QualityNotification),
        "quality-hold" => Ok(Capability::QualityHold),
        "certificate-analysis" | "certificate-of-analysis" => Ok(Capability::CertificateAnalysis),
        "audit-evidence" => Ok(Capability::AuditEvidence),
        _ => Err(ServiceError::validation(
            "capability",
            "unknown quality-management HTTP capability fixture",
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

fn accepted_body(tenant_id: &str, capability: Capability) -> Value {
    json!({
        "accepted": true,
        "tenant_id": tenant_id,
        "capability": capability_wire_name(capability),
        "audit_event_type": "command-accepted",
        "runtime_deployed": false,
        "durable_persistence": false
    })
}

pub fn validate_routes(routes: &[HttpRoute]) -> Result<()> {
    if routes.len() < 6 {
        return Err(ServiceError::validation(
            "http_routes",
            "scaffold requires all six Quality Management REST routes",
        ));
    }
    if routes.iter().any(|route| !route.path.starts_with("/v1/")) {
        return Err(ServiceError::validation(
            "http_routes",
            "all REST routes must be versioned under /v1",
        ));
    }
    for (path, capability) in [
        ("/v1/quality-management/inspection-plan", "inspection-plan"),
        ("/v1/quality-management/inspection-lot", "inspection-lot"),
        (
            "/v1/quality-management/certificate-of-analysis",
            "certificate-analysis",
        ),
        (
            "/v1/quality-management/quality-notification",
            "quality-notification",
        ),
        ("/v1/quality-management/quality-hold", "quality-hold"),
        ("/v1/quality-management/audit-evidence", "audit-evidence"),
    ] {
        if !routes.iter().any(|route| {
            route.method == "POST"
                && route.path == path
                && route.capability == capability
                && route.idempotent
        }) {
            return Err(ServiceError::validation(
                "http_routes",
                format!("missing OpenAPI fixture route for {capability}"),
            ));
        }
    }
    Ok(())
}
