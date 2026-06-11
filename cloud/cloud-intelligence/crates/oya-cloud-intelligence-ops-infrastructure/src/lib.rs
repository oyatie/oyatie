use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReadOnlyOpsSurface {
    pub read_only: bool,
    pub views: Vec<String>,
    pub mutation_routes: Vec<String>,
}

impl ReadOnlyOpsSurface {
    pub fn default_cloud_dashboard() -> Self {
        Self {
            read_only: true,
            views: vec![
                "status".to_string(),
                "accounts".to_string(),
                "backends".to_string(),
                "canaries".to_string(),
                "circuits".to_string(),
            ],
            mutation_routes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiagnosticSnapshot {
    pub tenant_id: String,
    credential_handle_redacted: String,
}

impl DiagnosticSnapshot {
    pub fn new(tenant_id: &str, _credential_handle: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            credential_handle_redacted: "redacted".to_string(),
        }
    }

    pub fn render_redacted_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminClientSurface {
    pub grpc_methods: Vec<String>,
    pub rest_routes: Vec<String>,
    pub includes_binary_entrypoint: bool,
}

impl AdminClientSurface {
    pub fn generated_contract_shape() -> Self {
        Self {
            grpc_methods: vec![
                "RefreshProviderPool".to_string(),
                "GetGatewayStatus".to_string(),
                "RegisterSubscriptionSeat".to_string(),
            ],
            rest_routes: vec![
                "/admin/v1/status".to_string(),
                "/admin/v1/accounts".to_string(),
                "/admin/v1/analytics".to_string(),
            ],
            includes_binary_entrypoint: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageEvent {
    pub event_type: &'static str,
    pub tenant_id: String,
    pub route_id: String,
    pub account_id: String,
    pub model: String,
    pub payload_redacted: bool,
    pub payload_summary: String,
}

impl UsageEvent {
    pub fn llm_usage(
        tenant_id: &str,
        route_id: &str,
        account_id: &str,
        model: &str,
        raw_payload: &str,
    ) -> Self {
        Self {
            event_type: "llm.usage.v1",
            tenant_id: tenant_id.to_string(),
            route_id: route_id.to_string(),
            account_id: account_id.to_string(),
            model: model.to_string(),
            payload_redacted: true,
            payload_summary: format!("payload_len={}", raw_payload.len()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub tenant_id: String,
    pub reason: String,
    pub is_tripped: bool,
    pub retry_after_seconds: Option<u64>,
    pub resumed_by: Option<String>,
}

impl CircuitBreakerState {
    pub fn tripped(tenant_id: &str, reason: &str, retry_after_seconds: u64) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            reason: reason.to_string(),
            is_tripped: true,
            retry_after_seconds: Some(retry_after_seconds),
            resumed_by: None,
        }
    }

    pub fn admin_resume(mut self, principal: &str) -> Self {
        self.is_tripped = false;
        self.retry_after_seconds = None;
        self.resumed_by = Some(principal.to_string());
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReadOnlyMcpToolset {
    pub read_only: bool,
    pub tools: Vec<String>,
}

impl ReadOnlyMcpToolset {
    pub fn default_ops_tools() -> Self {
        Self {
            read_only: true,
            tools: vec![
                "doctor".to_string(),
                "status".to_string(),
                "accounts".to_string(),
                "backends".to_string(),
                "fingerprint".to_string(),
            ],
        }
    }
}
