use crate::error::{Result, ServiceError};
use serde::{Deserialize, Serialize};

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
                path: "/v1/treasury/cash-positions:rollup",
                capability: "cash-position",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/treasury/bank-accounts:bind",
                capability: "bank-account",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/treasury/fx-exposures:measure",
                capability: "fx-exposure",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/treasury/hedge-designations:approve",
                capability: "hedge-designation",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/treasury/payments:execute",
                capability: "payment-execution",
                idempotent: true,
            },
        ]
    }
    pub fn handle(_request: HttpRequest) -> Result<HttpResponse> {
        Err(ServiceError::contract_stub("http"))
    }
}
pub fn validate_routes(routes: &[HttpRoute]) -> Result<()> {
    if routes.len() < 5 {
        return Err(ServiceError::validation(
            "http_routes",
            "scaffold requires at least five REST routes",
        ));
    }
    if routes.iter().any(|route| !route.path.starts_with("/v1/")) {
        return Err(ServiceError::validation(
            "http_routes",
            "all REST routes must be versioned under /v1",
        ));
    }
    Ok(())
}
