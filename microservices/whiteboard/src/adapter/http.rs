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

pub struct WhiteboardHttpHandler;

impl WhiteboardHttpHandler {
    pub fn routes() -> Vec<HttpRoute> {
        vec![
            HttpRoute {
                method: "POST",
                path: "/v1/whiteboard/boards:open",
                capability: "board-open",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/whiteboard/canvas-ops:append",
                capability: "canvas-op-append",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/whiteboard/exports:render",
                capability: "export-render",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/whiteboard/history:snapshot",
                capability: "history-snapshot",
                idempotent: true,
            },
            HttpRoute {
                method: "POST",
                path: "/v1/whiteboard/presence:sync",
                capability: "presence-sync",
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
            "whiteboard scaffold requires at least five REST routes",
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
