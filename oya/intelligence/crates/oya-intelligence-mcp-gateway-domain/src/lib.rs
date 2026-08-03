//! Foundry MCP gateway kernel: per-tenant MCP discovery and authorization.
//!
//! This crate is pure domain logic for ADR-0021's MCP-compatible gateway. It
//! deliberately does not open sockets, parse bearer tokens, or serialize JSON;
//! API/runtime crates can project these typed records onto HTTP + JSON-RPC.

use std::collections::BTreeMap;

use data_boundary_kernel::{
    Classified, DataClass, PrivacyDataClass, data_classes_from_privacy_data_classes,
};
use intelligence_capability_domain::{AutonomyTier, Capability};

pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
pub const DISCOVER_SCOPE: &str = "foundry.capability.discover";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum McpGatewayError {
    InvalidTenantId,
    EmptySubjectId,
    EmptyRegion,
    EmptyTld,
    InvalidHostSegment,
    EmptyAuthorizationServer,
    InvalidAuthorizationServer,
    TenantMismatch,
    MissingScope,
    AutonomyCeilingExceeded,
    TokenAudienceMismatch,
    TokenIssuerMismatch,
    TokenExpired,
    InvalidRateLimitPolicy,
    RateLimitExceeded,
    EmptyToolName,
    InvalidToolName,
    ToolNameTooLong,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpTenantEndpoint {
    pub tenant_id: Classified<String>,
    pub region: Classified<String>,
    pub tld: Classified<String>,
    pub authorization_server: Classified<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpPrincipal {
    pub tenant_id: Classified<String>,
    pub subject_id: Classified<String>,
    pub autonomy_ceiling: AutonomyTier,
    pub scopes: Classified<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpAccessTokenClaims {
    pub tenant_id: String,
    pub subject_id: String,
    pub issuer: String,
    pub audience: String,
    pub expires_at_epoch_seconds: u64,
    pub scopes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpServerCapabilities {
    pub tools_list_changed: bool,
    pub prompts_list_changed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpGatewayDescriptor {
    pub protocol_version: Classified<String>,
    pub endpoint: McpTenantEndpoint,
    pub capabilities: McpServerCapabilities,
    pub tools: Vec<McpTool>,
    pub prompts: Vec<McpPrompt>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpTool {
    pub name: Classified<String>,
    pub title: Classified<String>,
    pub description: Classified<String>,
    pub input_schema: Classified<String>,
    pub output_schema: Option<Classified<String>>,
    pub required_scope: Classified<String>,
    pub required_tier: AutonomyTier,
    pub privacy_data_classes: Vec<PrivacyDataClass>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpPrompt {
    pub name: Classified<String>,
    pub title: Classified<String>,
    pub description: Classified<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpAuthorizationChallenge {
    pub status_code: u16,
    pub resource_metadata_uri: Classified<String>,
    pub required_scopes: Classified<Vec<String>>,
    pub error: Option<Classified<String>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct McpRateLimitPolicy {
    pub max_calls: u32,
    pub window_seconds: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpRateLimitWindow {
    pub window_started_at_epoch_seconds: u64,
    pub calls: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpRateLimiter {
    policy: McpRateLimitPolicy,
    windows: BTreeMap<(String, String), McpRateLimitWindow>,
}

impl McpTenantEndpoint {
    pub fn new(
        tenant_id: String,
        region: String,
        tld: String,
        authorization_server: String,
    ) -> Result<Self, McpGatewayError> {
        validate_tenant_id(&tenant_id)?;
        validate_host_segment(&region, McpGatewayError::EmptyRegion)?;
        validate_host_segment(&tld, McpGatewayError::EmptyTld)?;
        validate_authorization_server(&authorization_server)?;
        Ok(Self {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            region: Classified::new(region, DataClass::InternalOnly),
            tld: Classified::new(tld, DataClass::InternalOnly),
            authorization_server: Classified::new(authorization_server, DataClass::InternalOnly),
        })
    }

    pub fn url(&self) -> String {
        format!(
            "https://mcp.foundry.{}.oyatie.{}/tenants/{}",
            self.region.value, self.tld.value, self.tenant_id.value
        )
    }

    pub fn protected_resource_metadata_uri(&self) -> String {
        format!(
            "https://mcp.foundry.{}.oyatie.{}/.well-known/oauth-protected-resource/tenants/{}",
            self.region.value, self.tld.value, self.tenant_id.value
        )
    }
}

impl McpPrincipal {
    pub fn new(
        tenant_id: String,
        subject_id: String,
        autonomy_ceiling: AutonomyTier,
        scopes: Vec<String>,
    ) -> Result<Self, McpGatewayError> {
        validate_tenant_id(&tenant_id)?;
        if subject_id.trim().is_empty() {
            return Err(McpGatewayError::EmptySubjectId);
        }
        Ok(Self {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            subject_id: Classified::new(subject_id, DataClass::InternalOnly),
            autonomy_ceiling,
            scopes: Classified::new(scopes, DataClass::InternalOnly),
        })
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.value.iter().any(|candidate| candidate == scope)
    }
}

impl McpGatewayDescriptor {
    pub fn new(
        endpoint: McpTenantEndpoint,
        principal: &McpPrincipal,
        capabilities: &[Capability],
    ) -> Result<Self, McpGatewayError> {
        assert_same_tenant(&endpoint, principal)?;
        require_scope(principal, DISCOVER_SCOPE)?;
        let mut tools = capabilities
            .iter()
            .map(project_capability_tool)
            .collect::<Result<Vec<_>, _>>()?;
        tools.sort_by(|left, right| left.name.value.cmp(&right.name.value));
        Ok(Self {
            protocol_version: Classified::new(
                MCP_PROTOCOL_VERSION.to_string(),
                DataClass::InternalOnly,
            ),
            endpoint,
            capabilities: McpServerCapabilities {
                tools_list_changed: true,
                prompts_list_changed: true,
            },
            tools,
            prompts: default_gateway_prompts(),
        })
    }
}

impl McpTool {
    pub fn privacy_data_classes(&self) -> &[PrivacyDataClass] {
        &self.privacy_data_classes
    }

    /// Legacy MCP descriptor projection for clients that still expect raw
    /// `DataClass` labels. Tool publication stores typed privacy classes, so
    /// this projection is derived from validated state and cannot expose
    /// operational or subject markers as privacy classes.
    pub fn legacy_data_classes(&self) -> Vec<DataClass> {
        data_classes_from_privacy_data_classes(&self.privacy_data_classes)
    }

    #[deprecated(
        note = "use privacy_data_classes for canonical typed access or legacy_data_classes for the compatibility projection"
    )]
    pub fn data_classes(&self) -> Vec<DataClass> {
        self.legacy_data_classes()
    }
}

impl McpAuthorizationChallenge {
    pub fn missing_token(endpoint: &McpTenantEndpoint, required_scopes: Vec<String>) -> Self {
        Self {
            status_code: 401,
            resource_metadata_uri: Classified::new(
                endpoint.protected_resource_metadata_uri(),
                DataClass::InternalOnly,
            ),
            required_scopes: Classified::new(required_scopes, DataClass::InternalOnly),
            error: None,
        }
    }

    pub fn insufficient_scope(endpoint: &McpTenantEndpoint, required_scopes: Vec<String>) -> Self {
        Self {
            status_code: 403,
            resource_metadata_uri: Classified::new(
                endpoint.protected_resource_metadata_uri(),
                DataClass::InternalOnly,
            ),
            required_scopes: Classified::new(required_scopes, DataClass::InternalOnly),
            error: Some(Classified::new(
                "insufficient_scope".to_string(),
                DataClass::InternalOnly,
            )),
        }
    }

    pub fn www_authenticate_header(&self) -> String {
        let mut parts = vec![format!(
            "resource_metadata=\"{}\"",
            self.resource_metadata_uri.value
        )];
        if !self.required_scopes.value.is_empty() {
            parts.push(format!(
                "scope=\"{}\"",
                self.required_scopes.value.join(" ")
            ));
        }
        if let Some(error) = &self.error {
            parts.push(format!("error=\"{}\"", error.value));
        }
        format!("Bearer {}", parts.join(", "))
    }
}

impl McpRateLimitPolicy {
    pub fn new(max_calls: u32, window_seconds: u64) -> Result<Self, McpGatewayError> {
        if max_calls == 0 || window_seconds == 0 {
            return Err(McpGatewayError::InvalidRateLimitPolicy);
        }
        Ok(Self {
            max_calls,
            window_seconds,
        })
    }
}

impl Default for McpRateLimitPolicy {
    fn default() -> Self {
        Self {
            max_calls: 60,
            window_seconds: 60,
        }
    }
}

impl McpRateLimiter {
    pub fn new(policy: McpRateLimitPolicy) -> Self {
        Self {
            policy,
            windows: BTreeMap::new(),
        }
    }

    pub fn policy(&self) -> McpRateLimitPolicy {
        self.policy
    }

    pub fn set_policy(&mut self, policy: McpRateLimitPolicy) {
        self.policy = policy;
        self.windows.clear();
    }

    pub fn check_and_record(
        &mut self,
        tenant_id: &str,
        tool_name: &str,
        now_epoch_seconds: u64,
    ) -> Result<(), McpGatewayError> {
        validate_tenant_id(tenant_id)?;
        validate_tool_name(tool_name)?;
        let key = (tenant_id.to_string(), tool_name.to_string());
        let window = self.windows.entry(key).or_insert(McpRateLimitWindow {
            window_started_at_epoch_seconds: now_epoch_seconds,
            calls: 0,
        });
        if now_epoch_seconds.saturating_sub(window.window_started_at_epoch_seconds)
            >= self.policy.window_seconds
        {
            window.window_started_at_epoch_seconds = now_epoch_seconds;
            window.calls = 0;
        }
        if window.calls >= self.policy.max_calls {
            return Err(McpGatewayError::RateLimitExceeded);
        }
        window.calls += 1;
        Ok(())
    }
}

impl Default for McpRateLimiter {
    fn default() -> Self {
        Self::new(McpRateLimitPolicy::default())
    }
}

pub fn project_capability_tool(capability: &Capability) -> Result<McpTool, McpGatewayError> {
    validate_tool_name(&capability.id)?;
    let mcp_contract = capability.mcp_contract();
    Ok(McpTool {
        name: Classified::new(capability.id.clone(), DataClass::InternalOnly),
        title: Classified::new(capability_title(&capability.id), DataClass::InternalOnly),
        description: mcp_contract.agent_readable_description.clone(),
        input_schema: mcp_contract.input_schema.clone(),
        output_schema: Some(mcp_contract.output_schema.clone()),
        required_scope: Classified::new(
            scope_for_tool_name(&capability.id),
            DataClass::InternalOnly,
        ),
        required_tier: capability.required_tier,
        privacy_data_classes: capability.touched_privacy_data_classes().to_vec(),
    })
}

pub fn authorize_tool_call(
    endpoint: &McpTenantEndpoint,
    principal: &McpPrincipal,
    tool: &McpTool,
) -> Result<(), McpGatewayError> {
    assert_same_tenant(endpoint, principal)?;
    require_scope(principal, &tool.required_scope.value)?;
    if principal.autonomy_ceiling < tool.required_tier {
        return Err(McpGatewayError::AutonomyCeilingExceeded);
    }
    Ok(())
}

pub fn validate_access_token(
    endpoint: &McpTenantEndpoint,
    claims: McpAccessTokenClaims,
    now_epoch_seconds: u64,
    autonomy_ceiling: AutonomyTier,
) -> Result<McpPrincipal, McpGatewayError> {
    if claims.expires_at_epoch_seconds <= now_epoch_seconds {
        return Err(McpGatewayError::TokenExpired);
    }
    if claims.issuer != endpoint.authorization_server.value {
        return Err(McpGatewayError::TokenIssuerMismatch);
    }
    if claims.audience != endpoint.url() {
        return Err(McpGatewayError::TokenAudienceMismatch);
    }
    if claims.tenant_id != endpoint.tenant_id.value {
        return Err(McpGatewayError::TenantMismatch);
    }
    McpPrincipal::new(
        claims.tenant_id,
        claims.subject_id,
        autonomy_ceiling,
        claims.scopes,
    )
}

pub fn scope_for_tool_name(tool_name: &str) -> String {
    format!("foundry.capability.invoke:{tool_name}")
}

pub fn default_gateway_prompts() -> Vec<McpPrompt> {
    // "foundation bypass" is the documented name for a bounded,
    // ledger-backed governance exception review; this prompt does not bypass
    // runtime authorization or validation.
    [
        (
            "workflow.preview-vertical",
            "Preview a vertical workflow",
            "Guide an operator through previewing a tenant-scoped vertical workflow before publish.",
        ),
        (
            "regional-pack-authoring",
            "Author a regional pack",
            "Draft a regional compliance pack with data residency and regulator evidence prompts.",
        ),
        (
            "adr-promotion",
            "Promote an ADR",
            "Collect implementation evidence before moving an ADR from proposed to accepted.",
        ),
        (
            "foundation-bypass-renewal",
            "Renew a foundation bypass",
            "Evaluate whether a bounded foundation bypass can be renewed or must be closed.",
        ),
        (
            "capability-publish",
            "Publish a capability",
            "Check eval coverage, autonomy tier, privacy data classes, cost budget, and evidence topics before publish.",
        ),
    ]
    .into_iter()
    .map(|(name, title, description)| McpPrompt {
        name: Classified::new(name.to_string(), DataClass::InternalOnly),
        title: Classified::new(title.to_string(), DataClass::InternalOnly),
        description: Classified::new(description.to_string(), DataClass::InternalOnly),
    })
    .collect()
}

fn assert_same_tenant(
    endpoint: &McpTenantEndpoint,
    principal: &McpPrincipal,
) -> Result<(), McpGatewayError> {
    if endpoint.tenant_id.value != principal.tenant_id.value {
        return Err(McpGatewayError::TenantMismatch);
    }
    Ok(())
}

fn require_scope(principal: &McpPrincipal, scope: &str) -> Result<(), McpGatewayError> {
    if !principal.has_scope(scope) {
        return Err(McpGatewayError::MissingScope);
    }
    Ok(())
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), McpGatewayError> {
    if !tenant_id.starts_with("ten_") {
        return Err(McpGatewayError::InvalidTenantId);
    }
    Ok(())
}

fn validate_host_segment(segment: &str, empty: McpGatewayError) -> Result<(), McpGatewayError> {
    if segment.trim().is_empty() {
        return Err(empty);
    }
    if segment
        .chars()
        .any(|ch| !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'))
    {
        return Err(McpGatewayError::InvalidHostSegment);
    }
    Ok(())
}

fn validate_authorization_server(authorization_server: &str) -> Result<(), McpGatewayError> {
    if authorization_server.trim().is_empty() {
        return Err(McpGatewayError::EmptyAuthorizationServer);
    }
    if !authorization_server.starts_with("https://") || authorization_server.contains('#') {
        return Err(McpGatewayError::InvalidAuthorizationServer);
    }
    Ok(())
}

fn validate_tool_name(tool_name: &str) -> Result<(), McpGatewayError> {
    if tool_name.is_empty() {
        return Err(McpGatewayError::EmptyToolName);
    }
    if tool_name.len() > 128 {
        return Err(McpGatewayError::ToolNameTooLong);
    }
    if tool_name
        .chars()
        .any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.'))
    {
        return Err(McpGatewayError::InvalidToolName);
    }
    Ok(())
}

fn capability_title(capability_id: &str) -> String {
    capability_id
        .trim_start_matches("cap.")
        .split(['.', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
