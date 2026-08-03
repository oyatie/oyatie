//! Platform Tenant API boundary.
//!
//! This crate owns authenticated REST-boundary normalization, path/body tenant
//! binding, request fingerprint idempotency, and global tenant-id uniqueness for
//! `tenant.create` before handing typed construction to the platform tenant
//! kernel.

use std::collections::BTreeMap;
use std::fmt;

use oya_residency_domain::parse_residency_class_label;
use oya_shared_pdp_kernel::{EntityRecord, EntitySlice, PolicyDecisionPoint};
use oya_shared_platform_contracts_kernel::pdp::{AuthorizationRequest, Decision, EntityRef};
use oya_tenancy_domain::{Tenant, TenantError};

pub const TENANT_CREATE_SURFACE: &str = "tenant.create";
pub const TENANT_CREATE_OPENAPI_CONTRACT: &str =
    "contracts/openapi/platform/platform-tenant-v1.yaml";
pub const TENANT_ENVIRONMENT_TIERS_OPENAPI_CONTRACT: &str =
    "cloud/tenancy/contracts/openapi/tenancy.yaml";
pub const TENANT_API_KEY_ISSUE_ENDPOINT: &str = "/v1/tenancy/api-keys";
pub const TENANT_ENVIRONMENTS_ENDPOINT_TEMPLATE: &str =
    "/v1/tenancy/tenants/{tenant_id}/environments";
pub const TENANT_OUTBOUND_CONFIG_ENDPOINT_TEMPLATE: &str =
    "/v1/tenancy/tenants/{tenant_id}/environments/{tier}/outbound-config";
pub const TENANT_API_KEY_ISSUE_SURFACE: &str = "tenancy.api_key.issue";
pub const TENANT_ENVIRONMENTS_READ_SURFACE: &str = "tenancy.environments.read";
pub const TENANT_OUTBOUND_CONFIG_UPDATE_SURFACE: &str =
    "tenancy.environment.outbound_config.update";
pub const PROD_DESTRUCTIVE_ACK_HEADER: &str = "x-oya-prod-destructive-ack";

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TenantEnvironmentTier {
    Test,
    Staging,
    Prod,
}

impl TenantEnvironmentTier {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Staging => "staging",
            Self::Prod => "prod",
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "test" => Some(Self::Test),
            "staging" => Some(Self::Staging),
            "prod" => Some(Self::Prod),
            _ => None,
        }
    }

    pub const fn server_key_prefix(self) -> &'static str {
        match self {
            Self::Test => "sk_test_",
            Self::Staging => "sk_stage_",
            Self::Prod => "sk_live_",
        }
    }

    pub const fn public_key_prefix(self) -> &'static str {
        match self {
            Self::Test => "pk_test_",
            Self::Staging => "pk_stage_",
            Self::Prod => "pk_live_",
        }
    }

    pub const fn outbound_mode(self) -> TenantOutboundMode {
        match self {
            Self::Test => TenantOutboundMode::Intercept,
            Self::Staging => TenantOutboundMode::TestRecipients,
            Self::Prod => TenantOutboundMode::Live,
        }
    }

    pub const fn audit_chain_tag(self) -> &'static str {
        match self {
            Self::Test => "env_tier=test",
            Self::Staging => "env_tier=staging",
            Self::Prod => "env_tier=prod",
        }
    }

    pub const fn destructive_operation_acknowledgment_required(self) -> bool {
        matches!(self, Self::Prod)
    }

    pub const fn outbound_config_patch_allowed(self) -> bool {
        !matches!(self, Self::Prod)
    }

    pub const fn minimum_api_key_issuer_role(self) -> TenantApiKeyIssuerRole {
        match self {
            Self::Test => TenantApiKeyIssuerRole::Developer,
            Self::Staging => TenantApiKeyIssuerRole::Maintainer,
            Self::Prod => TenantApiKeyIssuerRole::Admin,
        }
    }
}

impl fmt::Display for TenantEnvironmentTier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

impl std::str::FromStr for TenantEnvironmentTier {
    type Err = TenantEnvironmentTierParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_label(value).ok_or(TenantEnvironmentTierParseError(value.to_string()))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantEnvironmentTierParseError(pub String); // data_class: INTERNAL_ONLY

impl fmt::Display for TenantEnvironmentTierParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unknown tenant environment tier label: {:?}",
            self.0
        )
    }
}

impl std::error::Error for TenantEnvironmentTierParseError {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TenantOutboundMode {
    Intercept,
    TestRecipients,
    Live,
}

impl TenantOutboundMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Intercept => "intercept",
            Self::TestRecipients => "test_recipients",
            Self::Live => "live",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TenantApiKeyKind {
    Server,
    Public,
}

impl TenantApiKeyKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Server => "server",
            Self::Public => "public",
        }
    }

    pub const fn prefix_for_tier(self, environment_tier: TenantEnvironmentTier) -> &'static str {
        match self {
            Self::Server => environment_tier.server_key_prefix(),
            Self::Public => environment_tier.public_key_prefix(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TenantApiKeyIssuerRole {
    Developer,
    Maintainer,
    Admin,
    Owner,
}

impl TenantApiKeyIssuerRole {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Developer => "developer",
            Self::Maintainer => "maintainer",
            Self::Admin => "admin",
            Self::Owner => "owner",
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "developer" => Some(Self::Developer),
            "maintainer" => Some(Self::Maintainer),
            "admin" => Some(Self::Admin),
            "owner" => Some(Self::Owner),
            _ => None,
        }
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Developer => 1,
            Self::Maintainer => 2,
            Self::Admin => 3,
            Self::Owner => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TenantApiKeyPrefix {
    pub kind: TenantApiKeyKind,
    pub environment_tier: TenantEnvironmentTier,
    pub prefix: &'static str,
}

pub fn parse_tenant_api_key_prefix(value: &str) -> Option<TenantApiKeyPrefix> {
    [
        (
            TenantApiKeyKind::Server,
            TenantEnvironmentTier::Test,
            TenantEnvironmentTier::Test.server_key_prefix(),
        ),
        (
            TenantApiKeyKind::Public,
            TenantEnvironmentTier::Test,
            TenantEnvironmentTier::Test.public_key_prefix(),
        ),
        (
            TenantApiKeyKind::Server,
            TenantEnvironmentTier::Staging,
            TenantEnvironmentTier::Staging.server_key_prefix(),
        ),
        (
            TenantApiKeyKind::Public,
            TenantEnvironmentTier::Staging,
            TenantEnvironmentTier::Staging.public_key_prefix(),
        ),
        (
            TenantApiKeyKind::Server,
            TenantEnvironmentTier::Prod,
            TenantEnvironmentTier::Prod.server_key_prefix(),
        ),
        (
            TenantApiKeyKind::Public,
            TenantEnvironmentTier::Prod,
            TenantEnvironmentTier::Prod.public_key_prefix(),
        ),
    ]
    .into_iter()
    .find_map(|(kind, environment_tier, prefix)| {
        value.starts_with(prefix).then_some(TenantApiKeyPrefix {
            kind,
            environment_tier,
            prefix,
        })
    })
}

pub fn tenant_api_key_issuer_role_allowed(
    environment_tier: TenantEnvironmentTier,
    role: TenantApiKeyIssuerRole,
) -> bool {
    match environment_tier {
        TenantEnvironmentTier::Prod => role == TenantApiKeyIssuerRole::Admin,
        _ => role.rank() >= environment_tier.minimum_api_key_issuer_role().rank(),
    }
}

pub fn tenant_api_key_issuer_role_label_allowed(
    environment_tier: TenantEnvironmentTier,
    role_label: &str,
) -> bool {
    TenantApiKeyIssuerRole::parse_label(role_label)
        .is_some_and(|role| tenant_api_key_issuer_role_allowed(environment_tier, role))
}

pub fn destructive_operation_acknowledged(
    _environment_tier: TenantEnvironmentTier,
    header_value: Option<&str>,
) -> bool {
    // This boolean is projected into Cedar context as
    // `prod_destructive_acknowledged`; do not synthesize a local bypass just
    // because a non-prod tier does not require the ack. Only the explicit
    // boundary header is allowed to set the context bit.
    header_value.is_some_and(|value| value.trim().eq_ignore_ascii_case("true"))
}

pub const TENANT_ENVIRONMENT_READ_ACTION: &str = "read_tenant_environments";
pub const TENANT_API_KEY_ISSUE_ACTION: &str = "issue_api_key";
pub const TENANT_OUTBOUND_CONFIG_UPDATE_ACTION: &str = "update_outbound_config";
pub const TENANT_PROD_DESTRUCTIVE_OPERATION_ACTION: &str = "perform_prod_destructive_operation";

const TENANT_OPERATOR_ENTITY_TYPE: &str = "TenantOperator";
const TENANT_ENTITY_TYPE: &str = "Tenant";
const TENANT_ENVIRONMENT_ENTITY_TYPE: &str = "TenantEnvironment";
const TENANT_DESTRUCTIVE_OPERATION_ENTITY_TYPE: &str = "TenantDestructiveOperation";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantEnvironmentApiContext {
    pub request_id: String, // data_class: INTERNAL_ONLY
    pub principal: TenantApiPrincipal,
    pub plan_tier_role: TenantApiKeyIssuerRole,
    pub prod_destructive_ack_header: Option<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssueTenantApiKeyRequest {
    pub context: TenantEnvironmentApiContext,
    pub tenant_id: String, // data_class: TENANT_SCOPED
    pub environment_tier: TenantEnvironmentTier,
    pub key_kind: TenantApiKeyKind,
    pub label: Option<String>, // data_class: INTERNAL_ONLY
    pub created_at: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListTenantEnvironmentsRequest {
    pub context: TenantEnvironmentApiContext,
    pub path_tenant_id: String, // data_class: TENANT_SCOPED
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateTenantOutboundConfigRequest {
    pub context: TenantEnvironmentApiContext,
    pub path_tenant_id: String, // data_class: TENANT_SCOPED
    pub environment_tier: TenantEnvironmentTier,
    pub outbound_config: TenantOutboundConfig,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantOutboundConfig {
    pub mode: TenantOutboundMode,
    pub test_recipient_allowlist: Vec<String>, // data_class: INTERNAL_ONLY
    pub intercept_sink: Option<String>,        // data_class: INTERNAL_ONLY
}

impl TenantOutboundConfig {
    pub fn default_for_tier(environment_tier: TenantEnvironmentTier) -> Self {
        match environment_tier {
            TenantEnvironmentTier::Test => Self {
                mode: TenantOutboundMode::Intercept,
                test_recipient_allowlist: Vec::new(),
                intercept_sink: Some("audit-chain://outbound-intercepts".to_string()),
            },
            TenantEnvironmentTier::Staging => Self {
                mode: TenantOutboundMode::TestRecipients,
                test_recipient_allowlist: Vec::new(),
                intercept_sink: None,
            },
            TenantEnvironmentTier::Prod => Self {
                mode: TenantOutboundMode::Live,
                test_recipient_allowlist: Vec::new(),
                intercept_sink: None,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantEnvironmentRecord {
    pub tenant_id: String, // data_class: TENANT_SCOPED
    pub environment_tier: TenantEnvironmentTier,
    pub retention: String, // data_class: PUBLIC
    pub outbound_config: TenantOutboundConfig,
    pub server_key_prefix: &'static str,
    pub public_key_prefix: &'static str,
    pub audit_chain_tag: &'static str,
    pub destructive_op_acknowledgment_required: bool,
    pub authorization_decision_id: Option<String>, // data_class: INTERNAL_ONLY
}

impl TenantEnvironmentRecord {
    fn default_for_tenant(tenant_id: &str, environment_tier: TenantEnvironmentTier) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            environment_tier,
            retention: match environment_tier {
                TenantEnvironmentTier::Test => "90-day TTL default".to_string(),
                TenantEnvironmentTier::Staging => "durable".to_string(),
                TenantEnvironmentTier::Prod => "durable + residency-bound".to_string(),
            },
            outbound_config: TenantOutboundConfig::default_for_tier(environment_tier),
            server_key_prefix: environment_tier.server_key_prefix(),
            public_key_prefix: environment_tier.public_key_prefix(),
            audit_chain_tag: environment_tier.audit_chain_tag(),
            destructive_op_acknowledgment_required: environment_tier
                .destructive_operation_acknowledgment_required(),
            authorization_decision_id: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantEnvironmentListResponse {
    pub tenant_id: String, // data_class: TENANT_SCOPED
    pub environments: Vec<TenantEnvironmentRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantApiKeyMetadata {
    pub api_key_id: String, // data_class: INTERNAL_ONLY
    pub tenant_id: String,  // data_class: TENANT_SCOPED
    pub environment_tier: TenantEnvironmentTier,
    pub key_kind: TenantApiKeyKind,
    pub prefix: &'static str,
    pub label: Option<String>,             // data_class: INTERNAL_ONLY
    pub created_at: String,                // data_class: INTERNAL_ONLY
    pub authorization_decision_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantApiKeyIssueResponse {
    pub metadata: TenantApiKeyMetadata,
    pub secret_once: Option<String>, // data_class: SECRET
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantEnvironmentDirectory {
    environments: BTreeMap<(String, TenantEnvironmentTier), TenantEnvironmentRecord>,
    api_key_metadata: BTreeMap<String, TenantApiKeyMetadata>,
}

impl TenantEnvironmentDirectory {
    pub fn register_tenant(&mut self, tenant_id: &str) {
        for environment_tier in [
            TenantEnvironmentTier::Test,
            TenantEnvironmentTier::Staging,
            TenantEnvironmentTier::Prod,
        ] {
            self.environments
                .entry((tenant_id.to_string(), environment_tier))
                .or_insert_with(|| {
                    TenantEnvironmentRecord::default_for_tenant(tenant_id, environment_tier)
                });
        }
    }

    pub fn api_key_metadata_len(&self) -> usize {
        self.api_key_metadata.len()
    }

    pub fn api_key_metadata(&self, api_key_id: &str) -> Option<&TenantApiKeyMetadata> {
        self.api_key_metadata.get(api_key_id)
    }

    fn tenant_environments(&self, tenant_id: &str) -> Vec<TenantEnvironmentRecord> {
        [
            TenantEnvironmentTier::Test,
            TenantEnvironmentTier::Staging,
            TenantEnvironmentTier::Prod,
        ]
        .into_iter()
        .filter_map(|environment_tier| {
            self.environments
                .get(&(tenant_id.to_string(), environment_tier))
                .cloned()
        })
        .collect()
    }

    fn tenant_environment_mut(
        &mut self,
        tenant_id: &str,
        environment_tier: TenantEnvironmentTier,
    ) -> Option<&mut TenantEnvironmentRecord> {
        self.environments
            .get_mut(&(tenant_id.to_string(), environment_tier))
    }

    fn tenant_environment(
        &self,
        tenant_id: &str,
        environment_tier: TenantEnvironmentTier,
    ) -> Option<&TenantEnvironmentRecord> {
        self.environments
            .get(&(tenant_id.to_string(), environment_tier))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantEnvironmentApiError {
    EmptyRequestId,
    EmptyTenantId,
    EmptyPrincipalId,
    TenantMismatch,
    InvalidOutboundModeForTier,
    TenantEnvironmentNotFound,
    ProdOutboundConfigImmutable,
    InvalidAuthorizationProjection(String),
    InvalidEntityProjection(String),
    Pdp(String),
    InvalidPdpResponse(String),
    AuthorizationDenied,
}

pub fn issue_tenant_api_key_from_api(
    directory: &mut TenantEnvironmentDirectory,
    pdp: &dyn PolicyDecisionPoint,
    request: IssueTenantApiKeyRequest,
) -> Result<TenantApiKeyIssueResponse, TenantEnvironmentApiError> {
    validate_tenant_environment_context(&request.context, &request.tenant_id)?;
    if directory
        .tenant_environment(&request.tenant_id, request.environment_tier)
        .is_none()
    {
        return Err(TenantEnvironmentApiError::TenantEnvironmentNotFound);
    }
    let prefix = request.key_kind.prefix_for_tier(request.environment_tier);
    let decision_id = authorize_tenant_environment_action(
        pdp,
        &request.context,
        &request.tenant_id,
        TenantEnvironmentAuthorizationProjection {
            action: TENANT_API_KEY_ISSUE_ACTION,
            environment_tier: request.environment_tier,
            api_key_prefix: Some(prefix),
            outbound_mode: None,
            operation_id: None,
        },
    )?;
    let api_key_id = next_api_key_id(directory, &request);
    let metadata = TenantApiKeyMetadata {
        api_key_id: api_key_id.clone(),
        tenant_id: request.tenant_id,
        environment_tier: request.environment_tier,
        key_kind: request.key_kind,
        prefix,
        label: request.label,
        created_at: request.created_at,
        authorization_decision_id: decision_id,
    };
    let secret_once = if metadata.key_kind == TenantApiKeyKind::Server {
        Some(format!("{}{}", metadata.prefix, request.context.request_id))
    } else {
        None
    };
    directory
        .api_key_metadata
        .insert(api_key_id, metadata.clone());
    Ok(TenantApiKeyIssueResponse {
        metadata,
        secret_once,
    })
}

pub fn list_tenant_environments_from_api(
    directory: &TenantEnvironmentDirectory,
    pdp: &dyn PolicyDecisionPoint,
    request: ListTenantEnvironmentsRequest,
) -> Result<TenantEnvironmentListResponse, TenantEnvironmentApiError> {
    validate_tenant_environment_context(&request.context, &request.path_tenant_id)?;
    let environments = directory.tenant_environments(&request.path_tenant_id);
    if environments.len() != 3 {
        return Err(TenantEnvironmentApiError::TenantEnvironmentNotFound);
    }
    let mut authorized = Vec::with_capacity(environments.len());
    for environment in environments {
        let decision_id = authorize_tenant_environment_action(
            pdp,
            &request.context,
            &request.path_tenant_id,
            TenantEnvironmentAuthorizationProjection {
                action: TENANT_ENVIRONMENT_READ_ACTION,
                environment_tier: environment.environment_tier,
                api_key_prefix: None,
                outbound_mode: Some(environment.outbound_config.mode),
                operation_id: None,
            },
        )?;
        let mut environment = environment;
        environment.authorization_decision_id = Some(decision_id);
        authorized.push(environment);
    }
    Ok(TenantEnvironmentListResponse {
        tenant_id: request.path_tenant_id,
        environments: authorized,
    })
}

pub fn update_tenant_outbound_config_from_api(
    directory: &mut TenantEnvironmentDirectory,
    pdp: &dyn PolicyDecisionPoint,
    request: UpdateTenantOutboundConfigRequest,
) -> Result<TenantEnvironmentRecord, TenantEnvironmentApiError> {
    validate_tenant_environment_context(&request.context, &request.path_tenant_id)?;
    if !request.environment_tier.outbound_config_patch_allowed() {
        return Err(TenantEnvironmentApiError::ProdOutboundConfigImmutable);
    }
    if request.outbound_config.mode != request.environment_tier.outbound_mode() {
        return Err(TenantEnvironmentApiError::InvalidOutboundModeForTier);
    }
    let decision_id = authorize_tenant_environment_action(
        pdp,
        &request.context,
        &request.path_tenant_id,
        TenantEnvironmentAuthorizationProjection {
            action: TENANT_OUTBOUND_CONFIG_UPDATE_ACTION,
            environment_tier: request.environment_tier,
            api_key_prefix: None,
            outbound_mode: Some(request.outbound_config.mode),
            operation_id: None,
        },
    )?;
    let environment = directory
        .tenant_environment_mut(&request.path_tenant_id, request.environment_tier)
        .ok_or(TenantEnvironmentApiError::TenantEnvironmentNotFound)?;
    environment.outbound_config = request.outbound_config;
    environment.authorization_decision_id = Some(decision_id);
    Ok(environment.clone())
}

pub fn tenant_prod_destructive_operation_authorization_request(
    context: &TenantEnvironmentApiContext,
    tenant_id: &str,
    operation_id: &str,
    environment_tier: TenantEnvironmentTier,
) -> Result<AuthorizationRequest, TenantEnvironmentApiError> {
    validate_tenant_environment_context(context, tenant_id)?;
    tenant_environment_authorization_request(
        context,
        tenant_id,
        TenantEnvironmentAuthorizationProjection {
            action: TENANT_PROD_DESTRUCTIVE_OPERATION_ACTION,
            environment_tier,
            api_key_prefix: None,
            outbound_mode: None,
            operation_id: Some(operation_id),
        },
    )
}

fn validate_tenant_environment_context(
    context: &TenantEnvironmentApiContext,
    tenant_id: &str,
) -> Result<(), TenantEnvironmentApiError> {
    if context.request_id.trim().is_empty() {
        return Err(TenantEnvironmentApiError::EmptyRequestId);
    }
    if tenant_id.trim().is_empty() || context.principal.tenant_id.trim().is_empty() {
        return Err(TenantEnvironmentApiError::EmptyTenantId);
    }
    if context.principal.principal_id.trim().is_empty() {
        return Err(TenantEnvironmentApiError::EmptyPrincipalId);
    }
    if context.principal.tenant_id != tenant_id {
        return Err(TenantEnvironmentApiError::TenantMismatch);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct TenantEnvironmentAuthorizationProjection<'a> {
    action: &'a str,
    environment_tier: TenantEnvironmentTier,
    api_key_prefix: Option<&'a str>,
    outbound_mode: Option<TenantOutboundMode>,
    operation_id: Option<&'a str>,
}

fn authorize_tenant_environment_action(
    pdp: &dyn PolicyDecisionPoint,
    context: &TenantEnvironmentApiContext,
    tenant_id: &str,
    projection: TenantEnvironmentAuthorizationProjection<'_>,
) -> Result<String, TenantEnvironmentApiError> {
    let authorization_request =
        tenant_environment_authorization_request(context, tenant_id, projection)?;
    let entities = tenant_environment_entity_slice(context, tenant_id, projection)?;
    let outcome = pdp
        .authorize(&authorization_request, &entities)
        .map_err(|error| TenantEnvironmentApiError::Pdp(error.to_string()))?;
    outcome.response.validate().map_err(|violations| {
        TenantEnvironmentApiError::InvalidPdpResponse(format_violations(&violations))
    })?;
    if outcome.response.decision == Decision::Allow {
        Ok(outcome.response.decision_id)
    } else {
        Err(TenantEnvironmentApiError::AuthorizationDenied)
    }
}

fn tenant_environment_authorization_request(
    context: &TenantEnvironmentApiContext,
    tenant_id: &str,
    projection: TenantEnvironmentAuthorizationProjection<'_>,
) -> Result<AuthorizationRequest, TenantEnvironmentApiError> {
    let resource = tenant_environment_resource_ref(
        tenant_id,
        projection.environment_tier,
        projection.operation_id,
    );
    let mut abac_context = BTreeMap::from([
        (
            "tenant_id".to_string(),
            serde_json::Value::String(tenant_id.to_string()),
        ),
        (
            "env_tier".to_string(),
            serde_json::Value::String(projection.environment_tier.label().to_string()),
        ),
        (
            "plan_tier_role".to_string(),
            serde_json::Value::String(context.plan_tier_role.label().to_string()),
        ),
        (
            "prod_destructive_acknowledged".to_string(),
            serde_json::Value::Bool(destructive_operation_acknowledged(
                projection.environment_tier,
                context.prod_destructive_ack_header.as_deref(),
            )),
        ),
    ]);
    if let Some(prefix) = projection.api_key_prefix {
        abac_context.insert(
            "api_key_prefix".to_string(),
            serde_json::Value::String(prefix.to_string()),
        );
    }
    if let Some(mode) = projection.outbound_mode {
        abac_context.insert(
            "outbound_mode".to_string(),
            serde_json::Value::String(mode.label().to_string()),
        );
    }
    if let Some(operation_id) = projection.operation_id {
        abac_context.insert(
            "operation_id".to_string(),
            serde_json::Value::String(operation_id.to_string()),
        );
    }
    let request = AuthorizationRequest {
        request_id: context.request_id.clone(),
        tenant_id: tenant_id.to_string(),
        principal: tenant_operator_entity_ref(&context.principal.principal_id),
        action: projection.action.to_string(),
        resource,
        context: abac_context,
        min_policy_version: None,
    };
    request.validate().map_err(|violations| {
        TenantEnvironmentApiError::InvalidAuthorizationProjection(format_violations(&violations))
    })?;
    Ok(request)
}

fn tenant_environment_entity_slice(
    context: &TenantEnvironmentApiContext,
    tenant_id: &str,
    projection: TenantEnvironmentAuthorizationProjection<'_>,
) -> Result<EntitySlice, TenantEnvironmentApiError> {
    let principal = tenant_operator_entity_ref(&context.principal.principal_id);
    let resource = tenant_environment_resource_ref(
        tenant_id,
        projection.environment_tier,
        projection.operation_id,
    );
    let mut resource_attributes = BTreeMap::from([
        (
            "tenant_id".to_string(),
            serde_json::Value::String(tenant_id.to_string()),
        ),
        (
            "env_tier".to_string(),
            serde_json::Value::String(projection.environment_tier.label().to_string()),
        ),
    ]);
    if let Some(prefix) = projection.api_key_prefix {
        resource_attributes.insert(
            "api_key_prefix".to_string(),
            serde_json::Value::String(prefix.to_string()),
        );
    }
    if let Some(mode) = projection.outbound_mode {
        resource_attributes.insert(
            "outbound_mode".to_string(),
            serde_json::Value::String(mode.label().to_string()),
        );
    }
    let entities = EntitySlice {
        entities: vec![
            EntityRecord {
                uid: tenant_entity_ref(tenant_id),
                attributes: BTreeMap::from([(
                    "tenant_id".to_string(),
                    serde_json::Value::String(tenant_id.to_string()),
                )]),
                parents: Vec::new(),
            },
            EntityRecord {
                uid: principal,
                attributes: BTreeMap::from([
                    (
                        "tenant_id".to_string(),
                        serde_json::Value::String(tenant_id.to_string()),
                    ),
                    (
                        "plan_tier_role".to_string(),
                        serde_json::Value::String(context.plan_tier_role.label().to_string()),
                    ),
                ]),
                parents: vec![tenant_entity_ref(tenant_id)],
            },
            EntityRecord {
                uid: resource,
                attributes: resource_attributes,
                parents: vec![tenant_entity_ref(tenant_id)],
            },
        ],
    };
    entities.validate().map_err(|violations| {
        TenantEnvironmentApiError::InvalidEntityProjection(format_violations(&violations))
    })?;
    Ok(entities)
}

fn tenant_operator_entity_ref(principal_id: &str) -> EntityRef {
    EntityRef {
        entity_type: TENANT_OPERATOR_ENTITY_TYPE.to_string(),
        entity_id: principal_id.to_string(),
    }
}

fn tenant_entity_ref(tenant_id: &str) -> EntityRef {
    EntityRef {
        entity_type: TENANT_ENTITY_TYPE.to_string(),
        entity_id: tenant_id.to_string(),
    }
}

fn tenant_environment_resource_ref(
    tenant_id: &str,
    environment_tier: TenantEnvironmentTier,
    operation_id: Option<&str>,
) -> EntityRef {
    match operation_id {
        Some(operation_id) => EntityRef {
            entity_type: TENANT_DESTRUCTIVE_OPERATION_ENTITY_TYPE.to_string(),
            entity_id: format!("{tenant_id}:{}:{operation_id}", environment_tier.label()),
        },
        None => EntityRef {
            entity_type: TENANT_ENVIRONMENT_ENTITY_TYPE.to_string(),
            entity_id: format!("{tenant_id}:{}", environment_tier.label()),
        },
    }
}

fn next_api_key_id(
    directory: &TenantEnvironmentDirectory,
    request: &IssueTenantApiKeyRequest,
) -> String {
    format!(
        "ak_{}_{}_{}",
        safe_identifier_fragment(&request.context.request_id),
        request.environment_tier.label(),
        directory.api_key_metadata.len() + 1
    )
}

fn safe_identifier_fragment(value: &str) -> String {
    let fragment = value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || *character == '_' || *character == '-'
        })
        .collect::<String>();
    if fragment.is_empty() {
        "key".to_string()
    } else {
        fragment
    }
}

fn format_violations<T>(violations: &[T]) -> String
where
    T: fmt::Display,
{
    violations
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TenantCreateApiStatus {
    Created,
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

impl TenantCreateApiStatus {
    pub const fn code(self) -> u16 {
        match self {
            Self::Created => 201,
            Self::BadRequest => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::UnprocessableEntity => 422,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TenantCreateApiErrorCode {
    RequestIdEmpty,
    OperatorTenantHeaderEmpty,
    IdempotencyKeyEmpty,
    PrincipalIdEmpty,
    PathTenantIdEmpty,
    TenantPathBodyMismatch,
    AuthorizationDecisionIdEmpty,
    AuthorizationTenantMismatch,
    AuthorizationPrincipalMismatch,
    AuthorizationDenied,
    ResidencyClassInvalid,
    DuplicateTenant,
    IdempotencyKeyReused,
    TenantInvalidTenantId,
    TenantLegalNameEmpty,
    TenantHomeRegionEmpty,
    TenantHomeRegionDenied,
    TenantRegionalPackMissing,
}

impl TenantCreateApiErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestIdEmpty => "TENANT_CREATE_REQUEST_ID_EMPTY",
            Self::OperatorTenantHeaderEmpty => "TENANT_CREATE_OPERATOR_TENANT_HEADER_EMPTY",
            Self::IdempotencyKeyEmpty => "TENANT_CREATE_IDEMPOTENCY_KEY_EMPTY",
            Self::PrincipalIdEmpty => "TENANT_CREATE_PRINCIPAL_ID_EMPTY",
            Self::PathTenantIdEmpty => "TENANT_CREATE_PATH_TENANT_ID_EMPTY",
            Self::TenantPathBodyMismatch => "TENANT_CREATE_PATH_BODY_MISMATCH",
            Self::AuthorizationDecisionIdEmpty => "TENANT_CREATE_AUTHORIZATION_DECISION_ID_EMPTY",
            Self::AuthorizationTenantMismatch => "TENANT_CREATE_AUTHORIZATION_TENANT_MISMATCH",
            Self::AuthorizationPrincipalMismatch => {
                "TENANT_CREATE_AUTHORIZATION_PRINCIPAL_MISMATCH"
            }
            Self::AuthorizationDenied => "TENANT_CREATE_AUTHORIZATION_DENIED",
            Self::ResidencyClassInvalid => "TENANT_CREATE_RESIDENCY_CLASS_INVALID",
            Self::DuplicateTenant => "TENANT_CREATE_DUPLICATE_TENANT",
            Self::IdempotencyKeyReused => "TENANT_CREATE_IDEMPOTENCY_KEY_REUSED",
            Self::TenantInvalidTenantId => "TENANT_CREATE_TENANT_INVALID_ID",
            Self::TenantLegalNameEmpty => "TENANT_CREATE_LEGAL_NAME_EMPTY",
            Self::TenantHomeRegionEmpty => "TENANT_CREATE_HOME_REGION_EMPTY",
            Self::TenantHomeRegionDenied => "TENANT_CREATE_HOME_REGION_DENIED",
            Self::TenantRegionalPackMissing => "TENANT_CREATE_REGIONAL_PACK_MISSING",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantApiBoundaryContext {
    pub request_id: String,      // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantApiPrincipal {
    pub tenant_id: String,    // data_class: INTERNAL_ONLY
    pub principal_id: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantApiAuthorization {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub allowed_surfaces: Vec<String>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRegulatoryPackRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateRequest {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub legal_name: String,                             // data_class: INTERNAL_ONLY
    pub home_region: String,                            // data_class: INTERNAL_ONLY
    pub residency_class: String,                        // data_class: INTERNAL_ONLY
    pub regulatory_packs: Vec<TenantRegulatoryPackRef>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiRequest {
    pub path_tenant_id: String,                // data_class: INTERNAL_ONLY
    pub boundary: TenantApiBoundaryContext,    // data_class: INTERNAL_ONLY
    pub principal: TenantApiPrincipal,         // data_class: INTERNAL_ONLY
    pub authorization: TenantApiAuthorization, // data_class: INTERNAL_ONLY
    pub body: TenantCreateRequest,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantDirectory {
    tenants: BTreeMap<String, Tenant>, // data_class: INTERNAL_ONLY
}

impl TenantDirectory {
    pub fn len(&self) -> usize {
        self.tenants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tenants.is_empty()
    }

    pub fn get(&self, tenant_id: &str) -> Option<&Tenant> {
        self.tenants.get(tenant_id)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TenantCreateIdempotencyLedger {
    entries: BTreeMap<TenantCreateIdempotencyLedgerKey, TenantCreateIdempotencyLedgerEntry>, // data_class: INTERNAL_ONLY
}

impl TenantCreateIdempotencyLedger {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct TenantCreateIdempotencyLedgerKey {
    operator_tenant_id: String, // data_class: INTERNAL_ONLY
    principal_id: String,       // data_class: INTERNAL_ONLY
    surface: String,            // data_class: INTERNAL_ONLY
    idempotency_key: String,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TenantCreateIdempotencyLedgerEntry {
    fingerprint: TenantCreateRequestFingerprint, // data_class: INTERNAL_ONLY
    result: TenantCreateSuccessResponse,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TenantCreateRequestFingerprint {
    canonical: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateSuccessResponse {
    pub data: TenantRecord,             // data_class: INTERNAL_ONLY
    pub metadata: TenantCreateMetadata, // data_class: INTERNAL_ONLY
}

impl TenantCreateSuccessResponse {
    pub fn created(data: TenantRecord, request: &TenantCreateApiRequest) -> Self {
        Self {
            data,
            metadata: TenantCreateMetadata {
                request_id: request.boundary.request_id.clone(),
                operator_tenant_id: request.boundary.tenant_id.clone(),
                principal_id: request.principal.principal_id.clone(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateMetadata {
    pub request_id: String,         // data_class: INTERNAL_ONLY
    pub operator_tenant_id: String, // data_class: INTERNAL_ONLY
    pub principal_id: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantRecord {
    pub tenant_id: String,                              // data_class: INTERNAL_ONLY
    pub legal_name: String,                             // data_class: INTERNAL_ONLY
    pub home_region: String,                            // data_class: INTERNAL_ONLY
    pub residency_class: String,                        // data_class: INTERNAL_ONLY
    pub regulatory_packs: Vec<TenantRegulatoryPackRef>, // data_class: INTERNAL_ONLY
    pub schema_version: u32,                            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiErrorResponse {
    pub error: TenantCreateApiErrorBody, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiErrorBody {
    pub code: String,                             // data_class: INTERNAL_ONLY
    pub message: String,                          // data_class: INTERNAL_ONLY
    pub message_localized: Option<String>,        // data_class: INTERNAL_ONLY
    pub request_id: String,                       // data_class: INTERNAL_ONLY
    pub details: Vec<TenantCreateApiErrorDetail>, // data_class: INTERNAL_ONLY
    pub retry_after_seconds: Option<u64>,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCreateApiErrorDetail {
    pub field: String, // data_class: INTERNAL_ONLY
    pub issue: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantCreateApiError {
    EmptyRequestId,
    EmptyOperatorTenantHeader,
    EmptyIdempotencyKey,
    EmptyPrincipalId,
    EmptyPathTenantId,
    TenantPathBodyMismatch {
        path_tenant_id: String,
        body_tenant_id: String,
    },
    EmptyAuthorizationDecisionId,
    AuthorizationTenantMismatch {
        authorization_tenant_id: String,
        principal_tenant_id: String,
    },
    AuthorizationPrincipalMismatch {
        authorization_principal_id: String,
        principal_id: String,
    },
    AuthorizationDenied {
        surface: String,
    },
    InvalidResidencyClass {
        residency_class: String,
    },
    DuplicateTenant {
        tenant_id: String,
    },
    IdempotencyKeyReused {
        idempotency_key: String,
    },
    Tenant(TenantError),
}

impl TenantCreateApiError {
    pub fn tenant_create_status(&self) -> TenantCreateApiStatus {
        match self.status_kind() {
            TenantCreateApiStatusKind::BadRequest => TenantCreateApiStatus::BadRequest,
            TenantCreateApiStatusKind::Unauthorized => TenantCreateApiStatus::Unauthorized,
            TenantCreateApiStatusKind::Forbidden => TenantCreateApiStatus::Forbidden,
            TenantCreateApiStatusKind::Conflict => TenantCreateApiStatus::Conflict,
            TenantCreateApiStatusKind::UnprocessableEntity => {
                TenantCreateApiStatus::UnprocessableEntity
            }
        }
    }

    pub fn tenant_create_status_code(&self) -> u16 {
        self.tenant_create_status().code()
    }

    pub fn code(&self) -> TenantCreateApiErrorCode {
        match self {
            Self::EmptyRequestId => TenantCreateApiErrorCode::RequestIdEmpty,
            Self::EmptyOperatorTenantHeader => TenantCreateApiErrorCode::OperatorTenantHeaderEmpty,
            Self::EmptyIdempotencyKey => TenantCreateApiErrorCode::IdempotencyKeyEmpty,
            Self::EmptyPrincipalId => TenantCreateApiErrorCode::PrincipalIdEmpty,
            Self::EmptyPathTenantId => TenantCreateApiErrorCode::PathTenantIdEmpty,
            Self::TenantPathBodyMismatch { .. } => TenantCreateApiErrorCode::TenantPathBodyMismatch,
            Self::EmptyAuthorizationDecisionId => {
                TenantCreateApiErrorCode::AuthorizationDecisionIdEmpty
            }
            Self::AuthorizationTenantMismatch { .. } => {
                TenantCreateApiErrorCode::AuthorizationTenantMismatch
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                TenantCreateApiErrorCode::AuthorizationPrincipalMismatch
            }
            Self::AuthorizationDenied { .. } => TenantCreateApiErrorCode::AuthorizationDenied,
            Self::InvalidResidencyClass { .. } => TenantCreateApiErrorCode::ResidencyClassInvalid,
            Self::DuplicateTenant { .. } => TenantCreateApiErrorCode::DuplicateTenant,
            Self::IdempotencyKeyReused { .. } => TenantCreateApiErrorCode::IdempotencyKeyReused,
            Self::Tenant(error) => tenant_error_code(error),
        }
    }

    pub fn error_response(&self, request_id: impl Into<String>) -> TenantCreateApiErrorResponse {
        TenantCreateApiErrorResponse {
            error: TenantCreateApiErrorBody {
                code: self.code().as_str().to_string(),
                message: self.message().to_string(),
                message_localized: None,
                request_id: request_id.into(),
                details: self.details(),
                retry_after_seconds: None,
            },
        }
    }

    fn status_kind(&self) -> TenantCreateApiStatusKind {
        match self {
            Self::EmptyPrincipalId => TenantCreateApiStatusKind::Unauthorized,
            Self::EmptyAuthorizationDecisionId
            | Self::AuthorizationTenantMismatch { .. }
            | Self::AuthorizationPrincipalMismatch { .. }
            | Self::AuthorizationDenied { .. } => TenantCreateApiStatusKind::Forbidden,
            Self::DuplicateTenant { .. } => TenantCreateApiStatusKind::Conflict,
            Self::IdempotencyKeyReused { .. } => TenantCreateApiStatusKind::UnprocessableEntity,
            Self::EmptyRequestId
            | Self::EmptyOperatorTenantHeader
            | Self::EmptyIdempotencyKey
            | Self::EmptyPathTenantId
            | Self::TenantPathBodyMismatch { .. }
            | Self::InvalidResidencyClass { .. }
            | Self::Tenant(_) => TenantCreateApiStatusKind::BadRequest,
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::EmptyRequestId => "X-Request-Id header is required",
            Self::EmptyOperatorTenantHeader => "X-Tenant-Id operator header is required",
            Self::EmptyIdempotencyKey => "Idempotency-Key header is required",
            Self::EmptyPrincipalId => "Authenticated principal id is required",
            Self::EmptyPathTenantId => "Path tenant id is required",
            Self::TenantPathBodyMismatch { .. } => {
                "Path tenant id must match request body tenant_id"
            }
            Self::EmptyAuthorizationDecisionId => "Authorization decision id is required",
            Self::AuthorizationTenantMismatch { .. } => {
                "Authorization decision tenant must match the authenticated principal"
            }
            Self::AuthorizationPrincipalMismatch { .. } => {
                "Authorization decision principal must match the authenticated principal id"
            }
            Self::AuthorizationDenied { .. } => {
                "Authorization decision does not allow the requested tenant creation surface"
            }
            Self::InvalidResidencyClass { .. } => {
                "Request residency_class must be a supported residency class label"
            }
            Self::DuplicateTenant { .. } => "Tenant id already exists",
            Self::IdempotencyKeyReused { .. } => {
                "Idempotency key was already used with a different request"
            }
            Self::Tenant(error) => tenant_error_message(error),
        }
    }

    fn details(&self) -> Vec<TenantCreateApiErrorDetail> {
        match self {
            Self::EmptyRequestId => vec![detail("header.X-Request-Id", "must be non-empty")],
            Self::EmptyOperatorTenantHeader => {
                vec![detail("header.X-Tenant-Id", "must be non-empty")]
            }
            Self::EmptyIdempotencyKey => {
                vec![detail("header.Idempotency-Key", "must be non-empty")]
            }
            Self::EmptyPrincipalId => vec![detail("principal.principal_id", "must be non-empty")],
            Self::EmptyPathTenantId => vec![detail("path.tenant_id", "must be non-empty")],
            Self::TenantPathBodyMismatch { .. } => vec![detail(
                "body.tenant_id",
                "must match the tenant_id path parameter",
            )],
            Self::EmptyAuthorizationDecisionId => vec![detail(
                "authorization.decision_id",
                "must be non-empty authorization evidence",
            )],
            Self::AuthorizationTenantMismatch { .. } => vec![detail(
                "authorization.tenant_id",
                "must match the authenticated principal tenant",
            )],
            Self::AuthorizationPrincipalMismatch { .. } => vec![detail(
                "authorization.principal_id",
                "must match the authenticated principal id",
            )],
            Self::AuthorizationDenied { .. } => vec![detail(
                "authorization.allowed_surfaces",
                "must include the requested tenant.create surface",
            )],
            Self::InvalidResidencyClass { .. } => vec![detail(
                "body.residency_class",
                "must be one of strict_home_region, home_with_recovery_failover, or global",
            )],
            Self::DuplicateTenant { .. } => {
                vec![detail("body.tenant_id", "must be globally unique")]
            }
            Self::IdempotencyKeyReused { .. } => vec![detail(
                "header.Idempotency-Key",
                "same key cannot be reused with a different request fingerprint",
            )],
            Self::Tenant(error) => vec![detail("tenant_kernel", tenant_error_issue(error))],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TenantCreateApiStatusKind {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
    UnprocessableEntity,
}

pub fn validate_tenant_create_request(
    request: &TenantCreateApiRequest,
) -> Result<(), TenantCreateApiError> {
    validate_boundary(&request.boundary)?;
    validate_path_body_binding(&request.path_tenant_id, &request.body.tenant_id)?;
    validate_operator_binding(&request.boundary, &request.principal)?;
    validate_authorization(
        &request.principal,
        &request.authorization,
        TENANT_CREATE_SURFACE,
    )?;
    parse_api_residency_class(&request.body.residency_class)?;
    Ok(())
}

pub fn create_tenant_from_api(
    directory: &mut TenantDirectory,
    idempotency_ledger: &mut TenantCreateIdempotencyLedger,
    request: TenantCreateApiRequest,
) -> Result<TenantCreateSuccessResponse, TenantCreateApiError> {
    validate_tenant_create_request(&request)?;
    let key = idempotency_key_for(&request.boundary, &request.principal, TENANT_CREATE_SURFACE);
    let fingerprint = tenant_create_fingerprint_for(&request);
    if let Some(entry) = idempotency_ledger.entries.get(&key) {
        if entry.fingerprint == fingerprint {
            return Ok(entry.result.clone());
        }
        return Err(TenantCreateApiError::IdempotencyKeyReused {
            idempotency_key: request.boundary.idempotency_key,
        });
    }
    if directory.tenants.contains_key(&request.body.tenant_id) {
        return Err(TenantCreateApiError::DuplicateTenant {
            tenant_id: request.body.tenant_id,
        });
    }

    let tenant = tenant_from_request(&request.body)?;
    let response = TenantCreateSuccessResponse::created(tenant_record(&tenant), &request);
    directory.tenants.insert(tenant.id.clone(), tenant);
    idempotency_ledger.entries.insert(
        key,
        TenantCreateIdempotencyLedgerEntry {
            fingerprint,
            result: response.clone(),
        },
    );
    Ok(response)
}

fn validate_boundary(boundary: &TenantApiBoundaryContext) -> Result<(), TenantCreateApiError> {
    if boundary.request_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyRequestId);
    }
    if boundary.tenant_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyOperatorTenantHeader);
    }
    if boundary.idempotency_key.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyIdempotencyKey);
    }
    Ok(())
}

fn validate_path_body_binding(
    path_tenant_id: &str,
    body_tenant_id: &str,
) -> Result<(), TenantCreateApiError> {
    if path_tenant_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyPathTenantId);
    }
    if path_tenant_id != body_tenant_id {
        return Err(TenantCreateApiError::TenantPathBodyMismatch {
            path_tenant_id: path_tenant_id.to_string(),
            body_tenant_id: body_tenant_id.to_string(),
        });
    }
    Ok(())
}

fn validate_operator_binding(
    boundary: &TenantApiBoundaryContext,
    principal: &TenantApiPrincipal,
) -> Result<(), TenantCreateApiError> {
    if principal.principal_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyPrincipalId);
    }
    if boundary.tenant_id != principal.tenant_id {
        return Err(TenantCreateApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: boundary.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    Ok(())
}

fn validate_authorization(
    principal: &TenantApiPrincipal,
    authorization: &TenantApiAuthorization,
    surface: &str,
) -> Result<(), TenantCreateApiError> {
    if authorization.decision_id.trim().is_empty() {
        return Err(TenantCreateApiError::EmptyAuthorizationDecisionId);
    }
    if authorization.tenant_id != principal.tenant_id {
        return Err(TenantCreateApiError::AuthorizationTenantMismatch {
            authorization_tenant_id: authorization.tenant_id.clone(),
            principal_tenant_id: principal.tenant_id.clone(),
        });
    }
    if authorization.principal_id != principal.principal_id {
        return Err(TenantCreateApiError::AuthorizationPrincipalMismatch {
            authorization_principal_id: authorization.principal_id.clone(),
            principal_id: principal.principal_id.clone(),
        });
    }
    if !authorization
        .allowed_surfaces
        .iter()
        .any(|allowed_surface| allowed_surface == surface)
    {
        return Err(TenantCreateApiError::AuthorizationDenied {
            surface: surface.to_string(),
        });
    }
    Ok(())
}

fn tenant_from_request(body: &TenantCreateRequest) -> Result<Tenant, TenantCreateApiError> {
    Tenant::new(
        body.tenant_id.clone(),
        body.legal_name.clone(),
        body.home_region.clone(),
        parse_api_residency_class(&body.residency_class)?,
        body.regulatory_packs
            .iter()
            .map(|pack| pack.value.clone())
            .collect(),
    )
    .map_err(TenantCreateApiError::Tenant)
}

fn parse_api_residency_class(
    label: &str,
) -> Result<oya_residency_domain::ResidencyClass, TenantCreateApiError> {
    parse_residency_class_label(label).ok_or(TenantCreateApiError::InvalidResidencyClass {
        residency_class: label.to_string(),
    })
}

fn idempotency_key_for(
    boundary: &TenantApiBoundaryContext,
    principal: &TenantApiPrincipal,
    surface: &str,
) -> TenantCreateIdempotencyLedgerKey {
    TenantCreateIdempotencyLedgerKey {
        operator_tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn tenant_create_fingerprint_for(
    request: &TenantCreateApiRequest,
) -> TenantCreateRequestFingerprint {
    TenantCreateRequestFingerprint {
        canonical: [
            format!("path.tenant_id={}", request.path_tenant_id),
            format!("header.operator_tenant_id={}", request.boundary.tenant_id),
            format!("principal.tenant_id={}", request.principal.tenant_id),
            format!("principal.principal_id={}", request.principal.principal_id),
            format!(
                "authorization.tenant_id={}",
                request.authorization.tenant_id
            ),
            format!(
                "authorization.principal_id={}",
                request.authorization.principal_id
            ),
            format!(
                "authorization.decision_id={}",
                request.authorization.decision_id
            ),
            format!(
                "authorization.allowed_surfaces={}",
                request.authorization.allowed_surfaces.join(",")
            ),
            format!("body.tenant_id={}", request.body.tenant_id),
            format!("body.legal_name={}", request.body.legal_name),
            format!("body.home_region={}", request.body.home_region),
            format!("body.residency_class={}", request.body.residency_class),
            format!(
                "body.regulatory_packs={}",
                request
                    .body
                    .regulatory_packs
                    .iter()
                    .map(|pack| pack.value.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ]
        .join("|"),
    }
}

fn tenant_record(tenant: &Tenant) -> TenantRecord {
    TenantRecord {
        tenant_id: tenant.id.clone(),
        legal_name: tenant.legal_name.value.clone(),
        home_region: tenant.home_region.value.clone(),
        residency_class: tenant
            .residency_class
            .value
            .label()
            .unwrap_or("per_pack")
            .to_string(),
        regulatory_packs: tenant
            .regulatory_packs
            .value
            .iter()
            .cloned()
            .map(|value| TenantRegulatoryPackRef { value })
            .collect(),
        schema_version: 1,
    }
}

fn tenant_error_code(error: &TenantError) -> TenantCreateApiErrorCode {
    match error {
        TenantError::InvalidTenantId => TenantCreateApiErrorCode::TenantInvalidTenantId,
        TenantError::EmptyLegalName => TenantCreateApiErrorCode::TenantLegalNameEmpty,
        TenantError::EmptyHomeRegion => TenantCreateApiErrorCode::TenantHomeRegionEmpty,
        TenantError::HomeRegionNotAllowedForResidency => {
            TenantCreateApiErrorCode::TenantHomeRegionDenied
        }
        TenantError::MissingRegionalPack => TenantCreateApiErrorCode::TenantRegionalPackMissing,
    }
}

fn tenant_error_message(error: &TenantError) -> &'static str {
    match error {
        TenantError::InvalidTenantId => "Tenant id must use the ten_ prefix",
        TenantError::EmptyLegalName => "Tenant legal name is required",
        TenantError::EmptyHomeRegion => "Tenant home region is required",
        TenantError::HomeRegionNotAllowedForResidency => {
            "Tenant home region is not allowed for the requested residency class"
        }
        TenantError::MissingRegionalPack => "At least one regulatory pack is required",
    }
}

fn tenant_error_issue(error: &TenantError) -> &'static str {
    match error {
        TenantError::InvalidTenantId => "tenant id must be globally canonical and ten_-prefixed",
        TenantError::EmptyLegalName => "legal_name must be non-empty",
        TenantError::EmptyHomeRegion => "home_region must be non-empty",
        TenantError::HomeRegionNotAllowedForResidency => {
            "strict home-region residency classes require a kr-* home region"
        }
        TenantError::MissingRegionalPack => "regulatory_packs must contain at least one pack",
    }
}

fn detail(field: impl Into<String>, issue: impl Into<String>) -> TenantCreateApiErrorDetail {
    TenantCreateApiErrorDetail {
        field: field.into(),
        issue: issue.into(),
    }
}
