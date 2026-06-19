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

pub struct ContactCenterHttpHandler;

impl ContactCenterHttpHandler {
    pub fn routes() -> Vec<HttpRoute> {
        vec![
            HttpRoute {
                method: "POST",
                path: "/v1/contact-center/voice-routes:decide",
                capability: "voice-route",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/contact-center/queues:rebalance",
                capability: "queue-rebalance",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/contact-center/recording-consents:capture",
                capability: "recording-consent",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/contact-center/agent-state:sync",
                capability: "agent-state-sync",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/contact-center/callbacks:schedule",
                capability: "callback-schedule",
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
            "contact center scaffold requires at least five REST routes",
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
