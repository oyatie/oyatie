//! Intelligence model-routing usecase foundation.
//!
//! This crate provides a deterministic source-level usecase for provider route
//! selection. It validates request/catalog metadata, preserves in-memory
//! idempotency, delegates policy-safe route selection to the domain/kernel
//! layers, records metadata-only audit events, and performs no provider calls,
//! credential resolution, network I/O, filesystem access, durable idempotency,
//! durable audit-chain emission, or cloud runtime scheduling.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

pub use intelligence_model_routing_domain::{
    CredentialMode, DomainRouteDecision, IntelligenceDataClass, ModelCapability, ModelProvider,
    ModelRouteRequest, ProviderRouteProfile, RequestAudience, RouteDecision, RouteDenial,
    RouteDenialReason, RouteSelection, route_validated_request,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRoutingUsecaseInput {
    pub idempotency_key: String,             // data_class: INTERNAL_ONLY
    pub principal_id: String,                // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,           // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,         // data_class: INTERNAL_ONLY
    pub route_registry_snapshot_ref: String, // data_class: INTERNAL_ONLY
    pub request: ModelRouteRequest,          // data_class: INTERNAL_ONLY
    pub catalog: Vec<ProviderRouteProfile>,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRoutingUsecaseStatus {
    Denied,
    Routed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRoutingUsecaseDenialKind {
    IdempotencyConflict,
    InvalidInput,
    RouteDenied,
}

/// Metadata-only record of a single candidate that was evaluated and denied
/// during the catalog walk. Carries no credential fields or provider secrets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateDenial {
    pub provider: ModelProvider,              // data_class: PUBLIC
    pub model_id: String,                     // data_class: INTERNAL_ONLY
    pub priority: u16,                        // data_class: INTERNAL_ONLY
    pub reasons: BTreeSet<RouteDenialReason>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRoutingUsecaseReceipt {
    pub idempotency_key: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub principal_id: String,                // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,           // data_class: INTERNAL_ONLY
    pub policy_decision_ref: String,         // data_class: INTERNAL_ONLY
    pub route_registry_snapshot_ref: String, // data_class: INTERNAL_ONLY
    pub status: ModelRoutingUsecaseStatus,   // data_class: PUBLIC
    pub denial_kind: Option<ModelRoutingUsecaseDenialKind>, // data_class: INTERNAL_ONLY
    pub route_selection: Option<RouteSelection>, // data_class: INTERNAL_ONLY
    pub route_denial: Option<RouteDenial>,   // data_class: INTERNAL_ONLY
    /// Per-candidate denial trail produced by the catalog walk.
    /// On Routed: candidates tried and denied before the selected one (may be empty).
    /// On Denied: every candidate in stable priority order with its denial reasons.
    /// On InvalidInput / IdempotencyConflict: empty (walk was not reached).
    pub candidate_denials: Vec<CandidateDenial>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRoutingAuditEventKind {
    RouteDenied,
    RouteRequested,
    RouteSelected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRoutingAuditEvent {
    pub kind: ModelRoutingAuditEventKind, // data_class: INTERNAL_ONLY
    pub idempotency_key: String,          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                // data_class: INTERNAL_ONLY
    pub principal_id: String,             // data_class: INTERNAL_ONLY
    pub trace_context_ref: String,        // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,       // data_class: INTERNAL_ONLY
}

// ---------------------------------------------------------------------------
// Private state
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
struct ModelRouteIntent {
    fingerprint: String,
}

#[derive(Default, Debug)]
pub struct IntelligenceModelRoutingUsecase {
    receipts_by_idempotency_key: BTreeMap<String, (ModelRouteIntent, ModelRoutingUsecaseReceipt)>,
    events: Vec<ModelRoutingAuditEvent>,
}

// ---------------------------------------------------------------------------
// Usecase entry point
// ---------------------------------------------------------------------------

impl IntelligenceModelRoutingUsecase {
    pub fn route(&mut self, input: ModelRoutingUsecaseInput) -> ModelRoutingUsecaseReceipt {
        if let Err(evidence_ref) = validate_input(&input) {
            return receipt_from_input(
                &input,
                ModelRoutingUsecaseStatus::Denied,
                Some(ModelRoutingUsecaseDenialKind::InvalidInput),
                None,
                None,
                vec![],
                vec![evidence_ref],
            );
        }

        let intent = ModelRouteIntent {
            fingerprint: canonical_fingerprint(&input),
        };
        if let Some((existing_intent, existing_receipt)) =
            self.receipts_by_idempotency_key.get(&input.idempotency_key)
        {
            if existing_intent == &intent {
                return existing_receipt.clone();
            }
            return receipt_from_input(
                &input,
                ModelRoutingUsecaseStatus::Denied,
                Some(ModelRoutingUsecaseDenialKind::IdempotencyConflict),
                None,
                None,
                vec![],
                vec![
                    input.request.request_evidence_ref.clone(),
                    "model-routing:idempotency-conflict".to_owned(),
                ],
            );
        }

        self.record_event(
            ModelRoutingAuditEventKind::RouteRequested,
            &input,
            canonical_request_evidence_refs(&input),
        );

        let receipt = walk_catalog_for_route(&input);

        let event_kind = match receipt.status {
            ModelRoutingUsecaseStatus::Routed => ModelRoutingAuditEventKind::RouteSelected,
            ModelRoutingUsecaseStatus::Denied => ModelRoutingAuditEventKind::RouteDenied,
        };
        self.record_event(event_kind, &input, receipt.evidence_refs.clone());
        self.receipts_by_idempotency_key
            .insert(input.idempotency_key.clone(), (intent, receipt.clone()));
        receipt
    }

    pub fn events(&self) -> &[ModelRoutingAuditEvent] {
        &self.events
    }

    pub fn cached_receipt_count(&self) -> usize {
        self.receipts_by_idempotency_key.len()
    }

    fn record_event(
        &mut self,
        kind: ModelRoutingAuditEventKind,
        input: &ModelRoutingUsecaseInput,
        evidence_refs: Vec<String>,
    ) {
        self.events.push(ModelRoutingAuditEvent {
            kind,
            idempotency_key: safe_metadata(
                &input.idempotency_key,
                "redacted-invalid-idempotency-key",
            ),
            tenant_id: safe_tenant(&input.request.tenant_id),
            principal_id: safe_metadata(&input.principal_id, "redacted-invalid-principal-id"),
            trace_context_ref: safe_ref(
                &input.trace_context_ref,
                "model-routing:redacted-invalid-trace-context-ref",
            ),
            evidence_refs: sorted_unique(evidence_refs),
        });
    }
}

// ---------------------------------------------------------------------------
// Catalog-walk fallback (SUB-1 + SUB-2)
// ---------------------------------------------------------------------------

/// Walk the catalog in stable priority order, attempting each candidate
/// individually. Returns on the first allowed selection or on a terminal
/// (request-level) denial. Collects per-candidate denial metadata for the
/// receipt trail.
fn walk_catalog_for_route(input: &ModelRoutingUsecaseInput) -> ModelRoutingUsecaseReceipt {
    // Sort a local copy into the canonical priority order so the walk is
    // deterministic regardless of input ordering.
    let mut sorted_catalog = input.catalog.clone();
    sorted_catalog.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.provider.cmp(&b.provider))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });

    let mut candidate_denials: Vec<CandidateDenial> = Vec::new();

    for profile in &sorted_catalog {
        match route_validated_request(input.request.clone(), std::slice::from_ref(profile)) {
            DomainRouteDecision::Invalid(denial) => {
                // Terminal: the request itself is malformed; stop immediately.
                return receipt_from_input(
                    input,
                    ModelRoutingUsecaseStatus::Denied,
                    Some(ModelRoutingUsecaseDenialKind::RouteDenied),
                    None,
                    Some(denial.clone()),
                    candidate_denials,
                    sorted_unique(
                        [
                            canonical_request_evidence_refs(input),
                            denial.evidence_refs.clone(),
                            vec!["model-routing:route-denied".to_owned()],
                        ]
                        .concat(),
                    ),
                );
            }
            DomainRouteDecision::Routed(RouteDecision::Allow(selection)) => {
                return receipt_from_input(
                    input,
                    ModelRoutingUsecaseStatus::Routed,
                    None,
                    Some(selection.clone()),
                    None,
                    candidate_denials,
                    sorted_unique(
                        [
                            canonical_request_evidence_refs(input),
                            selection.evidence_refs.clone(),
                            vec!["model-routing:route-selected".to_owned()],
                        ]
                        .concat(),
                    ),
                );
            }
            DomainRouteDecision::Routed(RouteDecision::Deny(denial)) => {
                // Recoverable: record and continue to next candidate.
                candidate_denials.push(CandidateDenial {
                    provider: profile.provider,
                    model_id: profile.model_id.clone(),
                    priority: profile.priority,
                    reasons: denial.reasons.clone(),
                    evidence_refs: sorted_unique(denial.evidence_refs.clone()),
                });
            }
        }
    }

    // All candidates exhausted; build aggregate denial from the trail.
    let aggregate_denial = aggregate_denial_from_trail(&candidate_denials, input);
    let evidence_refs = sorted_unique(
        [
            canonical_request_evidence_refs(input),
            aggregate_denial.evidence_refs.clone(),
            vec!["model-routing:route-denied".to_owned()],
        ]
        .concat(),
    );
    receipt_from_input(
        input,
        ModelRoutingUsecaseStatus::Denied,
        Some(ModelRoutingUsecaseDenialKind::RouteDenied),
        None,
        Some(aggregate_denial),
        candidate_denials,
        evidence_refs,
    )
}

/// Build a `RouteDenial` that aggregates reasons and evidence from the full
/// denial trail. Used when the catalog is exhausted without a match.
fn aggregate_denial_from_trail(
    trail: &[CandidateDenial],
    input: &ModelRoutingUsecaseInput,
) -> RouteDenial {
    let mut reasons: BTreeSet<RouteDenialReason> = BTreeSet::new();
    let mut evidence_refs: Vec<String> = vec![input.request.request_evidence_ref.clone()];

    if trail.is_empty() || input.catalog.iter().all(|p| !p.enabled) {
        reasons.insert(RouteDenialReason::NoEnabledProvider);
    }

    for denial in trail {
        reasons.extend(denial.reasons.iter().copied());
        evidence_refs.extend(denial.evidence_refs.iter().cloned());
    }

    RouteDenial {
        reasons,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

// ---------------------------------------------------------------------------
// Receipt assembly
// ---------------------------------------------------------------------------

fn receipt_from_input(
    input: &ModelRoutingUsecaseInput,
    status: ModelRoutingUsecaseStatus,
    denial_kind: Option<ModelRoutingUsecaseDenialKind>,
    route_selection: Option<RouteSelection>,
    route_denial: Option<RouteDenial>,
    candidate_denials: Vec<CandidateDenial>,
    evidence_refs: Vec<String>,
) -> ModelRoutingUsecaseReceipt {
    ModelRoutingUsecaseReceipt {
        idempotency_key: safe_metadata(&input.idempotency_key, "redacted-invalid-idempotency-key"),
        tenant_id: safe_tenant(&input.request.tenant_id),
        principal_id: safe_metadata(&input.principal_id, "redacted-invalid-principal-id"),
        trace_context_ref: safe_ref(
            &input.trace_context_ref,
            "model-routing:redacted-invalid-trace-context-ref",
        ),
        policy_decision_ref: safe_ref(
            &input.policy_decision_ref,
            "model-routing:redacted-invalid-policy-decision-ref",
        ),
        route_registry_snapshot_ref: safe_ref(
            &input.route_registry_snapshot_ref,
            "model-routing:redacted-invalid-registry-snapshot-ref",
        ),
        status,
        denial_kind,
        route_selection,
        route_denial,
        candidate_denials,
        evidence_refs: sorted_unique(evidence_refs),
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

fn validate_input(input: &ModelRoutingUsecaseInput) -> Result<(), String> {
    require_metadata(
        &input.idempotency_key,
        "validation:model-routing-idempotency-key",
    )?;
    require_metadata(&input.principal_id, "validation:model-routing-principal")?;
    require_opaque(
        &input.trace_context_ref,
        "validation:model-routing-trace-context",
    )?;
    require_opaque(
        &input.policy_decision_ref,
        "validation:model-routing-policy-decision",
    )?;
    require_opaque(
        &input.route_registry_snapshot_ref,
        "validation:model-routing-registry-snapshot",
    )?;
    validate_request(&input.request)?;
    for profile in &input.catalog {
        validate_profile(profile)?;
    }
    Ok(())
}

fn validate_request(request: &ModelRouteRequest) -> Result<(), String> {
    require_tenant(&request.tenant_id, "validation:model-routing-tenant")?;
    require_opaque(
        &request.request_evidence_ref,
        "validation:model-routing-request-evidence",
    )?;
    Ok(())
}

fn validate_profile(profile: &ProviderRouteProfile) -> Result<(), String> {
    require_metadata(&profile.model_id, "validation:model-routing-model-id")?;
    for tenant in &profile.allowed_tenants {
        require_tenant(tenant, "validation:model-routing-allowed-tenant")?;
    }
    for evidence_ref in &profile.evidence_refs {
        require_opaque(evidence_ref, "validation:model-routing-catalog-evidence")?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Fingerprint
// ---------------------------------------------------------------------------

fn canonical_request_evidence_refs(input: &ModelRoutingUsecaseInput) -> Vec<String> {
    sorted_unique(vec![
        input.request.request_evidence_ref.clone(),
        input.trace_context_ref.clone(),
        input.policy_decision_ref.clone(),
        input.route_registry_snapshot_ref.clone(),
    ])
}

fn canonical_fingerprint(input: &ModelRoutingUsecaseInput) -> String {
    let mut parts = vec![
        canonical_entry("principal_id", &input.principal_id),
        canonical_entry("trace_context_ref", &input.trace_context_ref),
        canonical_entry("policy_decision_ref", &input.policy_decision_ref),
        canonical_entry(
            "route_registry_snapshot_ref",
            &input.route_registry_snapshot_ref,
        ),
        canonical_entry("tenant_id", &input.request.tenant_id),
        canonical_entry("capability", &format!("{:?}", input.request.capability)),
        canonical_entry(
            "credential_mode",
            &format!("{:?}", input.request.credential_mode),
        ),
        canonical_entry("data_class", &format!("{:?}", input.request.data_class)),
        canonical_entry("audience", &format!("{:?}", input.request.audience)),
        canonical_entry("request_evidence_ref", &input.request.request_evidence_ref),
        canonical_entry("catalog_len", &input.catalog.len().to_string()),
    ];
    let mut profiles: Vec<String> = input.catalog.iter().map(canonical_profile).collect();
    profiles.sort();
    parts.extend(profiles);
    parts.concat()
}

fn canonical_profile(profile: &ProviderRouteProfile) -> String {
    let entries = [
        canonical_entry("provider", &format!("{:?}", profile.provider)),
        canonical_entry("model_id", &profile.model_id),
        canonical_entry("enabled", &profile.enabled.to_string()),
        canonical_entry("priority", &profile.priority.to_string()),
        canonical_entry("capabilities", &format!("{:?}", profile.capabilities)),
        canonical_entry(
            "credential_modes",
            &format!("{:?}", profile.credential_modes),
        ),
        canonical_entry(
            "allowed_data_classes",
            &format!("{:?}", profile.allowed_data_classes),
        ),
        canonical_entry(
            "allowed_audiences",
            &format!("{:?}", profile.allowed_audiences),
        ),
        canonical_entry("allowed_tenants", &format!("{:?}", profile.allowed_tenants)),
        canonical_entry("evidence_refs", &format!("{:?}", profile.evidence_refs)),
    ];
    entries.concat()
}

fn canonical_entry(label: &str, value: &str) -> String {
    format!("{}:{}{}:{}", label.len(), label, value.len(), value)
}

// ---------------------------------------------------------------------------
// Metadata safety helpers
// ---------------------------------------------------------------------------

fn require_metadata(value: &str, evidence_ref: &str) -> Result<(), String> {
    if is_safe_metadata_ref(value) {
        Ok(())
    } else {
        Err(evidence_ref.to_owned())
    }
}

fn require_tenant(value: &str, evidence_ref: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.starts_with("ten_")
        && trimmed == value
        && !trimmed.contains('/')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
    {
        Ok(())
    } else {
        Err(evidence_ref.to_owned())
    }
}

fn require_opaque(value: &str, evidence_ref: &str) -> Result<(), String> {
    if is_safe_opaque_ref(value) {
        Ok(())
    } else {
        Err(evidence_ref.to_owned())
    }
}

fn safe_metadata(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if is_safe_metadata_ref(trimmed) && trimmed == value {
        trimmed.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn safe_tenant(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("ten_")
        && trimmed == value
        && !trimmed.contains('/')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
    {
        trimmed.to_owned()
    } else {
        "redacted-invalid-tenant-id".to_owned()
    }
}

fn safe_ref(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if is_safe_opaque_ref(trimmed) && trimmed == value {
        trimmed.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn is_safe_metadata_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn is_safe_opaque_ref(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.contains(':')
        && !contains_whitespace(trimmed)
        && !contains_raw_secret_material(trimmed)
        && !contains_raw_content_material(trimmed)
}

fn contains_whitespace(value: &str) -> bool {
    value.chars().any(char::is_whitespace)
}

fn contains_raw_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("sk-")
        || lower.contains("sk-")
        || lower.contains("bearer")
        || lower.contains("authorization:")
        || lower.contains("api_key=")
        || lower.contains("openai_api_key")
        || lower.contains("private key")
        || lower.contains("-----begin")
}

fn contains_raw_content_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("raw prompt")
        || lower.contains("raw model")
        || lower.contains("write an email")
        || lower.contains("customer message")
        || lower.contains("model answer")
        || lower.contains("raw output")
}

// ---------------------------------------------------------------------------
// Util
// ---------------------------------------------------------------------------

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn request() -> ModelRouteRequest {
        ModelRouteRequest {
            tenant_id: "ten_a".to_owned(),
            capability: ModelCapability::ChatCompletion,
            credential_mode: CredentialMode::TenantScoped,
            data_class: IntelligenceDataClass::InternalOnly,
            audience: RequestAudience::TenantOperator,
            request_evidence_ref: "req:model-routing:1".to_owned(),
        }
    }

    fn profile(provider: ModelProvider, model_id: &str, priority: u16) -> ProviderRouteProfile {
        ProviderRouteProfile {
            provider,
            model_id: model_id.to_owned(),
            enabled: true,
            priority,
            capabilities: BTreeSet::from([ModelCapability::ChatCompletion]),
            credential_modes: BTreeSet::from([CredentialMode::TenantScoped]),
            allowed_data_classes: BTreeSet::from([IntelligenceDataClass::InternalOnly]),
            allowed_audiences: BTreeSet::from([RequestAudience::TenantOperator]),
            allowed_tenants: BTreeSet::new(),
            evidence_refs: vec![format!("catalog:{model_id}")],
        }
    }

    fn input(idempotency_key: &str) -> ModelRoutingUsecaseInput {
        ModelRoutingUsecaseInput {
            idempotency_key: idempotency_key.to_owned(),
            principal_id: "principal:model-router".to_owned(),
            trace_context_ref: "trace:model-routing:1".to_owned(),
            policy_decision_ref: "policy:model-routing:1".to_owned(),
            route_registry_snapshot_ref: "route-registry:snapshot:1".to_owned(),
            request: request(),
            catalog: vec![
                profile(ModelProvider::OpenAi, "gpt-preview", 10),
                profile(ModelProvider::Anthropic, "claude-preview", 1),
            ],
        }
    }

    // -----------------------------------------------------------------------
    // Existing regression tests (unchanged behaviour)
    // -----------------------------------------------------------------------

    #[test]
    fn routes_authorized_request_with_metadata_audit_and_idempotency() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();

        let receipt = usecase.route(input("idem:model-routing:1"));
        let replay = usecase.route(input("idem:model-routing:1"));

        assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Routed);
        assert_eq!(
            receipt.route_selection.as_ref().unwrap().model_id,
            "claude-preview"
        );
        assert_eq!(receipt, replay);
        assert_eq!(usecase.cached_receipt_count(), 1);
        assert_eq!(
            usecase.events()[0].kind,
            ModelRoutingAuditEventKind::RouteRequested
        );
        assert_eq!(
            usecase.events()[1].kind,
            ModelRoutingAuditEventKind::RouteSelected
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"route-registry:snapshot:1".to_owned())
        );
    }

    #[test]
    fn route_denial_records_fail_closed_metadata_audit() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();
        let mut denied = input("idem:model-routing:denied");
        denied.catalog[0].enabled = false;
        denied.catalog[1].enabled = false;

        let receipt = usecase.route(denied);

        assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ModelRoutingUsecaseDenialKind::RouteDenied)
        );
        assert!(receipt.route_selection.is_none());
        assert!(
            receipt
                .route_denial
                .as_ref()
                .unwrap()
                .reasons
                .contains(&RouteDenialReason::NoEnabledProvider)
        );
        assert_eq!(
            usecase.events()[1].kind,
            ModelRoutingAuditEventKind::RouteDenied
        );
    }

    #[test]
    fn invalid_raw_metadata_denies_before_cache_or_audit_side_effects() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();
        let mut bad = input("sk-test-idem");
        bad.request.request_evidence_ref = "Authorization: Bearer sk-test".to_owned();
        bad.catalog[0].model_id = "write an email to a customer".to_owned();

        let receipt = usecase.route(bad);
        let debug = format!("{receipt:?}{:?}", usecase.events());

        assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ModelRoutingUsecaseDenialKind::InvalidInput)
        );
        assert_eq!(usecase.cached_receipt_count(), 0);
        assert!(usecase.events().is_empty());
        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email"));
    }

    #[test]
    fn idempotency_conflict_denies_without_replacing_original_receipt() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();
        let first = usecase.route(input("idem:model-routing:conflict"));
        let mut drifted = input("idem:model-routing:conflict");
        drifted.request.capability = ModelCapability::Embedding;

        let conflict = usecase.route(drifted);
        let replay = usecase.route(input("idem:model-routing:conflict"));

        assert_eq!(first.status, ModelRoutingUsecaseStatus::Routed);
        assert_eq!(conflict.status, ModelRoutingUsecaseStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(ModelRoutingUsecaseDenialKind::IdempotencyConflict)
        );
        assert_eq!(replay, first);
        assert_eq!(usecase.cached_receipt_count(), 1);
    }

    #[test]
    fn domain_sensitive_external_audience_denial_is_preserved() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();
        let mut external = input("idem:model-routing:external");
        external.request.audience = RequestAudience::ExternalEndUser;
        external.request.data_class = IntelligenceDataClass::PiiIdentifying;

        let receipt = usecase.route(external);

        assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ModelRoutingUsecaseDenialKind::RouteDenied)
        );
        assert!(
            receipt
                .evidence_refs
                .contains(&"validation:external-sensitive-data".to_owned())
        );
    }

    #[test]
    fn receipts_and_events_never_contain_raw_prompt_output_or_secret_bytes() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();

        let receipt = usecase.route(input("idem:model-routing:redaction"));
        let debug = format!("{receipt:?}{:?}", usecase.events());

        assert!(!debug.contains("sk-test"));
        assert!(!debug.contains("write an email"));
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("raw output"));
        assert!(!debug.contains("model answer"));
    }

    // -----------------------------------------------------------------------
    // SUB-1: catalog-walk fallback
    // -----------------------------------------------------------------------

    #[test]
    fn fallback_to_second_candidate_when_first_is_denied_by_capability_mismatch() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();
        // profile-A: priority 1, wrong capability -> recoverable denial
        let mut profile_a = profile(ModelProvider::OpenAi, "gpt-embedding", 1);
        profile_a.capabilities = BTreeSet::from([ModelCapability::Embedding]);
        // profile-B: priority 2, fully eligible
        let profile_b = profile(ModelProvider::Anthropic, "claude-chat", 2);

        let mut inp = input("idem:model-routing:fallback-cap");
        inp.catalog = vec![profile_a, profile_b];

        let receipt = usecase.route(inp);

        assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Routed);
        assert_eq!(
            receipt.route_selection.as_ref().unwrap().model_id,
            "claude-chat"
        );
        // exactly one candidate was tried and denied before the selected one
        assert_eq!(receipt.candidate_denials.len(), 1);
        assert_eq!(receipt.candidate_denials[0].model_id, "gpt-embedding");
        assert!(
            receipt.candidate_denials[0]
                .reasons
                .contains(&RouteDenialReason::CapabilityUnavailable)
        );
    }

    #[test]
    fn terminal_denial_fails_fast_without_walking_remaining_catalog() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();
        // Request that triggers DomainRouteDecision::Invalid (ExternalEndUser + PiiIdentifying)
        let mut inp = input("idem:model-routing:terminal");
        inp.request.audience = RequestAudience::ExternalEndUser;
        inp.request.data_class = IntelligenceDataClass::PiiIdentifying;
        // Two profiles; neither should be reached past the first terminal check
        inp.catalog = vec![
            profile(ModelProvider::OpenAi, "gpt-preview", 1),
            profile(ModelProvider::Anthropic, "claude-preview", 2),
        ];

        let receipt = usecase.route(inp);

        assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ModelRoutingUsecaseDenialKind::RouteDenied)
        );
        // Terminal failure: no candidates were accepted, so denial trail has at most 0
        // entries (the first domain call returned Invalid before any Allow/Deny walk)
        assert!(receipt.candidate_denials.is_empty());
        assert!(
            receipt
                .evidence_refs
                .contains(&"validation:external-sensitive-data".to_owned())
        );
    }

    #[test]
    fn fallback_ordering_is_deterministic_with_shuffled_catalog() {
        // Same two profiles submitted in two different orders must yield the
        // same Routed receipt (same chosen candidate, same candidate_denials).
        let profile_low = {
            // priority 1 — eligible
            profile(ModelProvider::Anthropic, "claude-preview", 1)
        };
        let profile_high = {
            // priority 10 — capability mismatch → recoverable denial
            let mut p = profile(ModelProvider::OpenAi, "gpt-embed", 10);
            p.capabilities = BTreeSet::from([ModelCapability::Embedding]);
            p
        };

        let mut inp_ab = input("idem:model-routing:order-ab");
        inp_ab.catalog = vec![profile_high.clone(), profile_low.clone()];

        let mut inp_ba = input("idem:model-routing:order-ba");
        inp_ba.catalog = vec![profile_low.clone(), profile_high.clone()];

        let mut usecase = IntelligenceModelRoutingUsecase::default();
        let receipt_ab = usecase.route(inp_ab);
        let receipt_ba = usecase.route(inp_ba);

        // Both should select the same eligible candidate
        assert_eq!(receipt_ab.status, ModelRoutingUsecaseStatus::Routed);
        assert_eq!(receipt_ba.status, ModelRoutingUsecaseStatus::Routed);
        assert_eq!(
            receipt_ab.route_selection.as_ref().unwrap().model_id,
            receipt_ba.route_selection.as_ref().unwrap().model_id,
        );
        assert_eq!(
            receipt_ab.candidate_denials.len(),
            receipt_ba.candidate_denials.len(),
        );
    }

    // -----------------------------------------------------------------------
    // SUB-2: per-candidate denial trail
    // -----------------------------------------------------------------------

    #[test]
    fn all_candidates_denied_receipt_enumerates_full_trail_in_stable_order() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();

        // Three profiles each denied for a distinct recoverable reason:
        // p1: wrong capability
        let mut p1 = profile(ModelProvider::OpenAi, "gpt-embed", 1);
        p1.capabilities = BTreeSet::from([ModelCapability::Embedding]);

        // p2: wrong credential mode
        let mut p2 = profile(ModelProvider::Anthropic, "claude-byok", 2);
        p2.credential_modes = BTreeSet::from([CredentialMode::BringYourOwnKey]);

        // p3: tenant-restricted (denied for TenantNotAllowed)
        let mut p3 = profile(ModelProvider::Gemini, "gemini-pro", 3);
        p3.allowed_tenants = BTreeSet::from(["ten_other".to_owned()]);

        let mut inp = input("idem:model-routing:full-trail");
        inp.catalog = vec![p1, p2, p3];

        let receipt = usecase.route(inp);

        assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Denied);
        assert_eq!(
            receipt.denial_kind,
            Some(ModelRoutingUsecaseDenialKind::RouteDenied)
        );
        assert_eq!(receipt.candidate_denials.len(), 3);

        // Stable order: priority 1, 2, 3
        assert_eq!(receipt.candidate_denials[0].priority, 1);
        assert_eq!(receipt.candidate_denials[1].priority, 2);
        assert_eq!(receipt.candidate_denials[2].priority, 3);

        assert!(
            receipt.candidate_denials[0]
                .reasons
                .contains(&RouteDenialReason::CapabilityUnavailable)
        );
        assert!(
            receipt.candidate_denials[1]
                .reasons
                .contains(&RouteDenialReason::CredentialModeUnavailable)
        );
        assert!(
            receipt.candidate_denials[2]
                .reasons
                .contains(&RouteDenialReason::TenantNotAllowed)
        );

        // Metadata-only invariant: no secrets or raw content in debug repr
        let debug = format!("{receipt:?}");
        assert!(!debug.contains("sk-"));
        assert!(!debug.contains("bearer"));
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("raw output"));
    }

    #[test]
    fn metadata_only_invariant_no_secrets_in_denial_trail() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();
        let mut p = profile(ModelProvider::OpenAi, "gpt-embed", 1);
        p.capabilities = BTreeSet::from([ModelCapability::Embedding]);
        let mut inp = input("idem:model-routing:secrets-check");
        inp.catalog = vec![p];

        let receipt = usecase.route(inp);
        let debug = format!("{receipt:?}");

        assert!(!debug.contains("sk-"));
        assert!(!debug.contains("bearer"));
        assert!(!debug.contains("raw prompt"));
        assert!(!debug.contains("raw output"));
        assert!(!debug.contains("model answer"));
    }

    // -----------------------------------------------------------------------
    // SUB-3: idempotency replay under fallback
    // -----------------------------------------------------------------------

    #[test]
    fn fallback_receipt_replays_identically_under_same_key() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();

        // First candidate denied (capability mismatch), second selected
        let mut p_bad = profile(ModelProvider::OpenAi, "gpt-embed", 1);
        p_bad.capabilities = BTreeSet::from([ModelCapability::Embedding]);
        let p_good = profile(ModelProvider::Anthropic, "claude-chat", 2);

        let mut inp = input("idem:model-routing:fallback-replay");
        inp.catalog = vec![p_bad, p_good];

        let first = usecase.route(inp.clone());
        let replay = usecase.route(inp);

        assert_eq!(first.status, ModelRoutingUsecaseStatus::Routed);
        assert_eq!(first, replay);
        assert_eq!(
            first.route_selection.as_ref().unwrap().model_id,
            "claude-chat"
        );
        assert_eq!(first.candidate_denials.len(), 1);
        assert_eq!(usecase.cached_receipt_count(), 1);
    }

    #[test]
    fn conflicting_payload_on_fallback_key_yields_idempotency_conflict() {
        let mut usecase = IntelligenceModelRoutingUsecase::default();

        let mut p_bad = profile(ModelProvider::OpenAi, "gpt-embed", 1);
        p_bad.capabilities = BTreeSet::from([ModelCapability::Embedding]);
        let p_good = profile(ModelProvider::Anthropic, "claude-chat", 2);

        let mut original = input("idem:model-routing:fallback-conflict");
        original.catalog = vec![p_bad, p_good];

        let first = usecase.route(original.clone());
        assert_eq!(first.status, ModelRoutingUsecaseStatus::Routed);

        // Same key, different capability → conflict
        let mut conflicting = original.clone();
        conflicting.request.capability = ModelCapability::Embedding;
        let conflict = usecase.route(conflicting);

        assert_eq!(conflict.status, ModelRoutingUsecaseStatus::Denied);
        assert_eq!(
            conflict.denial_kind,
            Some(ModelRoutingUsecaseDenialKind::IdempotencyConflict)
        );

        // Original receipt still retrievable via clean replay
        let replay = usecase.route(original);
        assert_eq!(replay, first);
        assert_eq!(usecase.cached_receipt_count(), 1);
    }
}
