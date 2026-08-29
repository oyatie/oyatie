//! Foundation application slice composing the W-Foundation kernels.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod capabilities;
mod classification;
mod deny;
mod error;
mod error_map;
mod foundation;
mod identity;
mod invoke;
mod mcp;
mod objects;
mod observability;
mod policy;
mod settle;
mod support;
mod tenants;
mod types;

pub use error::FoundationError;
pub use foundation::Foundation;
pub use observability::{FoundationObservability, InvocationSettlementStatus};
pub use types::*;

pub(crate) use classification::*;
pub(crate) use error::*;
pub(crate) use error_map::*;
pub(crate) use observability::*;
pub(crate) use support::*;

pub mod product_catalog;
pub use product_catalog::{ProductCatalogError, ProductEntry, ProductId, ProductMetadata};

pub(crate) use std::{collections::BTreeMap, fmt, sync::Arc};

pub use audit_chain_domain::{AuditChain, AuditEvent, Plane};
pub(crate) use cell_regional_pack::{RegionalPack, RegionalPackError};
pub(crate) use cell_routing::{CellBinding, CellBindingCreate, CellError, CellRouter, CellTier};
pub(crate) use check_cost_budget::{
    BudgetCeiling, BudgetError, BudgetLedger, BudgetScope, BudgetSnapshot, BudgetWarning,
};
pub use data_boundary_kernel::{
    AgeBand, ConsentScope, DataClass, PrivacyDataClass, Purpose, SubjectClass,
    privacy_data_classes_from,
};
pub(crate) use data_boundary_kernel::{
    Classified, DataClassification, DataUseAttributes, DataUseDenialReason, OperationalDataClass,
    evaluate_data_use,
};
pub use data_ontology_domain::PropertyTier;
pub(crate) use data_ontology_domain::{ObjectEntity, ObjectGraphError, ObjectProperty};
pub(crate) use governance_eval_domain::EvalError;
pub use governance_eval_domain::{
    AdversarialKind, EvalCaseInput, EvalGate, EvalMetric, EvalRunInput, EvalSetInput,
    REQUIRED_LINGUISTIC_COHORT_LOCALES,
};
pub(crate) use iam_identity_domain::{IdentityError, IdpBinding, Token, User, issue_token};
pub use iam_policy_cedar_domain::{
    AuthorizationDecision, PolicyEffect, PolicyRuleInput, PolicyScope, PolicyVersion,
};
pub(crate) use iam_policy_cedar_domain::{
    AuthorizationQuery, AuthorizationSubject, PolicyError, PolicySet,
};
pub(crate) use intelligence_adapter_kernel::{
    AdapterError, CostCeiling, InvocationPolicy, ProviderAuth, ProviderCallReceipt, ProviderId,
    ProviderMode, ProviderProfile, ProviderRoute, ProviderRoutePreference, ProviderRouteRequest,
    SubscriptionBindingRegistry, resolve_route,
};
pub use intelligence_bypass_domain::{AutonomyBreakGlass, AutonomyBreakGlassInput};
pub(crate) use intelligence_bypass_domain::{BypassError, BypassLedger, BypassLedgerRecord};
pub(crate) use intelligence_capability_domain::CapabilityError;
pub use intelligence_capability_domain::{
    AutonomyTier, Capability, CapabilityAction, CapabilityCostProfile, CapabilityMcpContract,
    CapabilityRegistry,
};
pub(crate) use intelligence_evidence_domain::EvidenceError;
pub use intelligence_evidence_domain::{EvidenceChain, EvidenceKind, EvidenceRecord};
pub use intelligence_mcp_gateway_domain::{
    DISCOVER_SCOPE, McpAccessTokenClaims, McpGatewayDescriptor, McpPrompt, McpRateLimitPolicy,
    McpTool, scope_for_tool_name,
};
pub(crate) use intelligence_mcp_gateway_domain::{
    McpGatewayError, McpPrincipal, McpRateLimiter, McpTenantEndpoint, authorize_tool_call,
    project_capability_tool, validate_access_token,
};
pub(crate) use intelligence_policy_domain::{
    AutonomyCapReason, AutonomyCapSource, AutonomyDecision, AutonomyVerdict, TenantPolicy,
};
pub use intelligence_run_domain::{Run, RunDisposition, RunState};
pub(crate) use intelligence_run_domain::{RunError, RunLedger, RunStart};
pub use intelligence_step_domain::{Step, StepDisposition, StepKind, StepState};
pub(crate) use intelligence_step_domain::{StepError, StepLedger, StepStart};
pub(crate) use messaging_domain::{EventingError, Outbox, OutboxRecord};
pub use network_residency::ResidencyClass;
pub(crate) use network_residency::{
    RegionRef, RegionRefCreate, infer_region_jurisdiction_label, parse_residency_class_label,
};
pub(crate) use observability_domain::{
    CAPABILITY_INVOCATION_OPERATION_NAME, CapabilityInvocationTraceContext,
    CapabilityInvocationTraceObserver, CapabilityInvocationTraceSpan, FOUNDRY_PROVIDER_NAME,
    InvocationTraceResult, NoopCapabilityInvocationTraceObserver,
    telemetry_data_classifications_label,
};
pub(crate) use secrets_domain::SecretRef;
pub use tenancy_domain::Tenant;
pub(crate) use tenancy_domain::TenantError;

pub(crate) const FOUNDATION_LOCAL_PROVIDER_ID: &str = "foundation-local";
pub(crate) const FOUNDATION_LOCAL_MODEL_REF: &str = "foundation-app";
pub(crate) const FOUNDATION_LOCAL_SECRET_REF_NAME: &str = "foundation-local-provider";
pub(crate) const FOUNDATION_LOCAL_PROVIDER_P95_LATENCY_MS: u32 = 1;
pub(crate) const FOUNDATION_LOCAL_PROVIDER_ATTEMPT: u32 = 1;

fn audit_classifications() -> [DataClassification; 1] {
    [DataClassification::from(OperationalDataClass::Audit)]
}
