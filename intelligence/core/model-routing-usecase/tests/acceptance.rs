//! Acceptance tests for the model-routing usecase slice:
//! catalog-walk fallback, per-candidate denial trail, and idempotency replay
//! hardening (SUB-1, SUB-2, SUB-3).
//!
//! These tests are the authoritative red-first specification for the three
//! subtasks. Each test name states the expected behavior. No test verifies
//! more than one observable fact.

use std::collections::BTreeSet;

use intelligence_model_routing_usecase::{
    CredentialMode, IntelligenceDataClass, IntelligenceModelRoutingUsecase, ModelCapability,
    ModelProvider, ModelRouteRequest, ModelRoutingUsecaseDenialKind, ModelRoutingUsecaseInput,
    ModelRoutingUsecaseStatus, ProviderRouteProfile, RequestAudience, RouteDenialReason,
};

// ---------------------------------------------------------------------------
// Shared fixtures
// ---------------------------------------------------------------------------

fn base_request() -> ModelRouteRequest {
    ModelRouteRequest {
        tenant_id: "ten_acme".to_owned(),
        capability: ModelCapability::ChatCompletion,
        credential_mode: CredentialMode::TenantScoped,
        data_class: IntelligenceDataClass::InternalOnly,
        audience: RequestAudience::TenantOperator,
        request_evidence_ref: "req:acceptance:1".to_owned(),
    }
}

fn full_profile(provider: ModelProvider, model_id: &str, priority: u16) -> ProviderRouteProfile {
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
        evidence_refs: vec![format!("catalog:acceptance:{model_id}")],
    }
}

fn base_input(key: &str, catalog: Vec<ProviderRouteProfile>) -> ModelRoutingUsecaseInput {
    ModelRoutingUsecaseInput {
        idempotency_key: key.to_owned(),
        principal_id: "principal:acceptance-test".to_owned(),
        trace_context_ref: "trace:acceptance:1".to_owned(),
        policy_decision_ref: "policy:acceptance:1".to_owned(),
        route_registry_snapshot_ref: "route-registry:snapshot:acceptance:1".to_owned(),
        request: base_request(),
        catalog,
    }
}

// ---------------------------------------------------------------------------
// SUB-1(a): recoverable top-candidate denial falls through to next eligible
// ---------------------------------------------------------------------------

/// When the top candidate (priority 1) has a capability mismatch the usecase
/// falls through to the next eligible candidate and returns a Routed receipt
/// naming that candidate.
#[test]
fn sub1a_recoverable_capability_mismatch_falls_through_to_next_candidate() {
    let mut denied = full_profile(ModelProvider::OpenAi, "gpt-embed", 1);
    denied.capabilities = BTreeSet::from([ModelCapability::Embedding]);

    let allowed = full_profile(ModelProvider::Anthropic, "claude-chat", 2);

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let receipt = usecase.route(base_input(
        "idem:sub1a:cap-fallthrough",
        vec![denied, allowed],
    ));

    assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Routed);
    assert_eq!(
        receipt.route_selection.as_ref().unwrap().model_id,
        "claude-chat",
        "second candidate must be selected after first is denied recoverably"
    );
}

/// When three candidates exist and the first two are denied by different
/// recoverable reasons (capability, credential) the third eligible candidate
/// is selected and candidate_denials carries exactly two entries.
#[test]
fn sub1a_two_recoverable_denials_then_third_candidate_selected() {
    // priority 1: capability mismatch
    let mut p1 = full_profile(ModelProvider::OpenAi, "gpt-embed", 1);
    p1.capabilities = BTreeSet::from([ModelCapability::Embedding]);

    // priority 2: credential mode mismatch
    let mut p2 = full_profile(ModelProvider::AzureOpenAi, "azure-byok", 2);
    p2.credential_modes = BTreeSet::from([CredentialMode::BringYourOwnKey]);

    // priority 3: fully eligible
    let p3 = full_profile(ModelProvider::Anthropic, "claude-eligible", 3);

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let receipt = usecase.route(base_input("idem:sub1a:two-skip-then-hit", vec![p1, p2, p3]));

    assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Routed);
    assert_eq!(
        receipt.route_selection.as_ref().unwrap().model_id,
        "claude-eligible"
    );
    assert_eq!(
        receipt.candidate_denials.len(),
        2,
        "exactly two candidates must appear in the denial trail before the selected one"
    );
    assert_eq!(receipt.candidate_denials[0].model_id, "gpt-embed");
    assert_eq!(receipt.candidate_denials[1].model_id, "azure-byok");
}

// ---------------------------------------------------------------------------
// SUB-1(b): terminal denial reason fails fast
// ---------------------------------------------------------------------------

/// A terminal denial (ExternalEndUser + PiiIdentifying) stops the walk
/// immediately; candidate_denials is empty because no recoverable per-profile
/// denial was accumulated before the terminal condition fired.
#[test]
fn sub1b_terminal_denial_yields_empty_candidate_denials() {
    let mut inp = base_input(
        "idem:sub1b:terminal-empty-trail",
        vec![
            full_profile(ModelProvider::OpenAi, "gpt-1", 1),
            full_profile(ModelProvider::Anthropic, "claude-1", 2),
        ],
    );
    inp.request.audience = RequestAudience::ExternalEndUser;
    inp.request.data_class = IntelligenceDataClass::PiiIdentifying;

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let receipt = usecase.route(inp);

    assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Denied);
    assert_eq!(
        receipt.denial_kind,
        Some(ModelRoutingUsecaseDenialKind::RouteDenied)
    );
    assert!(
        receipt.candidate_denials.is_empty(),
        "terminal denial must not accumulate any candidate denial entries"
    );
}

/// On a terminal denial the route_denial field carries the terminal reason
/// and the route_selection field is None.
#[test]
fn sub1b_terminal_denial_sets_route_denial_and_clears_route_selection() {
    let mut inp = base_input(
        "idem:sub1b:terminal-fields",
        vec![full_profile(ModelProvider::OpenAi, "gpt-1", 1)],
    );
    inp.request.audience = RequestAudience::ExternalEndUser;
    inp.request.data_class = IntelligenceDataClass::PiiIdentifying;

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let receipt = usecase.route(inp);

    assert!(
        receipt.route_selection.is_none(),
        "Routed selection must be None on a terminal denial"
    );
    let denial = receipt
        .route_denial
        .as_ref()
        .expect("route_denial must be Some on terminal path");
    assert!(
        denial
            .reasons
            .contains(&RouteDenialReason::AudienceNotAllowed)
            || denial
                .reasons
                .contains(&RouteDenialReason::DataClassNotAllowed),
        "terminal route_denial must carry audience or data-class denial reason"
    );
}

// ---------------------------------------------------------------------------
// SUB-1(c): ordering is deterministic with shuffled-but-equivalent catalogs
// ---------------------------------------------------------------------------

/// Two identical catalogs submitted in reversed input order produce receipts
/// with the same chosen candidate model_id.
#[test]
fn sub1c_reversed_catalog_order_yields_same_selected_candidate() {
    let p_low = full_profile(ModelProvider::Anthropic, "claude-low", 1);
    let p_high = {
        let mut p = full_profile(ModelProvider::OpenAi, "gpt-high", 10);
        p.capabilities = BTreeSet::from([ModelCapability::Embedding]); // skipped
        p
    };

    let mut usecase = IntelligenceModelRoutingUsecase::default();

    let r1 = usecase.route(base_input(
        "idem:sub1c:order-fwd",
        vec![p_high.clone(), p_low.clone()],
    ));
    let r2 = usecase.route(base_input(
        "idem:sub1c:order-rev",
        vec![p_low.clone(), p_high.clone()],
    ));

    assert_eq!(r1.status, ModelRoutingUsecaseStatus::Routed);
    assert_eq!(r2.status, ModelRoutingUsecaseStatus::Routed);
    assert_eq!(
        r1.route_selection.as_ref().unwrap().model_id,
        r2.route_selection.as_ref().unwrap().model_id,
        "selected candidate must be identical regardless of input catalog order"
    );
}

/// With equal-priority candidates, the stable provider ordering determines the
/// winner. Anthropic (ord 0) beats OpenAi (ord 4) at equal priority.
#[test]
fn sub1c_equal_priority_tie_broken_by_provider_ord_anthropic_before_openai() {
    // Both priority 5, both eligible
    let p_anthropic = full_profile(ModelProvider::Anthropic, "claude-tie", 5);
    let p_openai = full_profile(ModelProvider::OpenAi, "gpt-tie", 5);

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    // Submit OpenAi first to verify input order doesn't override tie-break
    let receipt = usecase.route(base_input(
        "idem:sub1c:tie-break",
        vec![p_openai, p_anthropic],
    ));

    assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Routed);
    assert_eq!(
        receipt.route_selection.as_ref().unwrap().model_id,
        "claude-tie",
        "Anthropic must win the tie-break over OpenAi (lower Ord variant)"
    );
}

// ---------------------------------------------------------------------------
// SUB-2: per-candidate denial trail — all-denied receipt
// ---------------------------------------------------------------------------

/// When all catalog candidates are denied the receipt status is Denied and
/// candidate_denials enumerates every candidate with its specific
/// RouteDenialReason in stable priority order.
#[test]
fn sub2_all_denied_receipt_enumerates_every_candidate_with_specific_reason() {
    // p1 priority 1: capability mismatch
    let mut p1 = full_profile(ModelProvider::OpenAi, "gpt-embed", 1);
    p1.capabilities = BTreeSet::from([ModelCapability::Embedding]);

    // p2 priority 2: credential mode mismatch
    let mut p2 = full_profile(ModelProvider::Anthropic, "claude-byok", 2);
    p2.credential_modes = BTreeSet::from([CredentialMode::BringYourOwnKey]);

    // p3 priority 3: tenant restriction
    let mut p3 = full_profile(ModelProvider::Gemini, "gemini-pro", 3);
    p3.allowed_tenants = BTreeSet::from(["ten_other".to_owned()]);

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let receipt = usecase.route(base_input("idem:sub2:all-denied", vec![p1, p2, p3]));

    assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Denied);
    assert_eq!(
        receipt.denial_kind,
        Some(ModelRoutingUsecaseDenialKind::RouteDenied)
    );

    // All three candidates appear in stable order
    assert_eq!(receipt.candidate_denials.len(), 3);
    assert_eq!(receipt.candidate_denials[0].priority, 1);
    assert_eq!(receipt.candidate_denials[1].priority, 2);
    assert_eq!(receipt.candidate_denials[2].priority, 3);

    assert!(
        receipt.candidate_denials[0]
            .reasons
            .contains(&RouteDenialReason::CapabilityUnavailable),
        "p1 must be denied for CapabilityUnavailable"
    );
    assert!(
        receipt.candidate_denials[1]
            .reasons
            .contains(&RouteDenialReason::CredentialModeUnavailable),
        "p2 must be denied for CredentialModeUnavailable"
    );
    assert!(
        receipt.candidate_denials[2]
            .reasons
            .contains(&RouteDenialReason::TenantNotAllowed),
        "p3 must be denied for TenantNotAllowed"
    );
}

/// On an all-denied receipt candidate_denials carries provider and model_id
/// fields but no raw credential data, network-resolved addresses, or secret
/// material — metadata-only invariant.
#[test]
fn sub2_candidate_denials_are_metadata_only_no_secrets_or_provider_payloads() {
    let mut p = full_profile(ModelProvider::OpenAi, "gpt-embed", 1);
    p.capabilities = BTreeSet::from([ModelCapability::Embedding]);

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let receipt = usecase.route(base_input("idem:sub2:metadata-only", vec![p]));

    let debug = format!("{receipt:?}");
    assert!(!debug.contains("sk-"), "no API key fragments");
    assert!(!debug.contains("bearer"), "no bearer tokens");
    assert!(!debug.contains("raw prompt"), "no raw prompt material");
    assert!(!debug.contains("raw output"), "no raw output material");
    assert!(
        !debug.contains("model answer"),
        "no raw model answer material"
    );

    // CandidateDenial must carry only refs, reasons, provider, model_id, priority
    let denial = &receipt.candidate_denials[0];
    assert_eq!(denial.provider, ModelProvider::OpenAi);
    assert!(!denial.model_id.is_empty());
    assert_eq!(denial.priority, 1);
}

/// An empty catalog (no profiles) produces Denied with empty candidate_denials
/// and NoEnabledProvider in route_denial.
#[test]
fn sub2_empty_catalog_produces_denied_with_empty_candidate_denials() {
    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let receipt = usecase.route(base_input("idem:sub2:empty-catalog", vec![]));

    assert_eq!(receipt.status, ModelRoutingUsecaseStatus::Denied);
    assert!(
        receipt.candidate_denials.is_empty(),
        "empty catalog walk must produce zero candidate denial entries"
    );
    let denial = receipt
        .route_denial
        .as_ref()
        .expect("route_denial must be present");
    assert!(
        denial
            .reasons
            .contains(&RouteDenialReason::NoEnabledProvider),
        "empty catalog must surface NoEnabledProvider"
    );
}

// ---------------------------------------------------------------------------
// SUB-3: idempotency replay under fallback
// ---------------------------------------------------------------------------

/// Re-submitting the same idempotency_key for a fallback-resolved (Routed)
/// request returns the byte-identical receipt including the chosen candidate
/// and the per-candidate denial trail.
#[test]
fn sub3_fallback_routed_receipt_replays_byte_identical_including_denial_trail() {
    let mut p_denied = full_profile(ModelProvider::OpenAi, "gpt-embed", 1);
    p_denied.capabilities = BTreeSet::from([ModelCapability::Embedding]);
    let p_allowed = full_profile(ModelProvider::Anthropic, "claude-chat", 2);

    let inp = base_input("idem:sub3:fallback-replay-exact", vec![p_denied, p_allowed]);

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let first = usecase.route(inp.clone());
    let replay = usecase.route(inp);

    assert_eq!(first, replay, "replay must be byte-identical (PartialEq)");
    assert_eq!(
        first.route_selection.as_ref().unwrap().model_id,
        "claude-chat"
    );
    assert_eq!(
        first.candidate_denials.len(),
        1,
        "denial trail must survive replay intact"
    );
    assert_eq!(usecase.cached_receipt_count(), 1);
}

/// Re-submitting the same idempotency_key for an all-denied request returns
/// the byte-identical receipt including the full candidate_denials trail.
#[test]
fn sub3_all_denied_receipt_replays_byte_identical_including_full_denial_trail() {
    let mut p1 = full_profile(ModelProvider::OpenAi, "gpt-embed", 1);
    p1.capabilities = BTreeSet::from([ModelCapability::Embedding]);
    let mut p2 = full_profile(ModelProvider::Anthropic, "claude-byok", 2);
    p2.credential_modes = BTreeSet::from([CredentialMode::BringYourOwnKey]);

    let inp = base_input("idem:sub3:denied-replay-exact", vec![p1, p2]);

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let first = usecase.route(inp.clone());
    let replay = usecase.route(inp);

    assert_eq!(first.status, ModelRoutingUsecaseStatus::Denied);
    assert_eq!(
        first, replay,
        "replay of denied receipt must be byte-identical"
    );
    assert_eq!(
        first.candidate_denials.len(),
        replay.candidate_denials.len(),
        "candidate_denials must be identical on replay"
    );
    assert_eq!(usecase.cached_receipt_count(), 1);
}

/// A conflicting payload on a key that previously resolved via fallback yields
/// IdempotencyConflict; the original fallback receipt is still retrievable.
#[test]
fn sub3_conflicting_payload_on_fallback_key_yields_idempotency_conflict() {
    let mut p_denied = full_profile(ModelProvider::OpenAi, "gpt-embed", 1);
    p_denied.capabilities = BTreeSet::from([ModelCapability::Embedding]);
    let p_allowed = full_profile(ModelProvider::Anthropic, "claude-chat", 2);

    let original = base_input(
        "idem:sub3:conflict-key",
        vec![p_denied.clone(), p_allowed.clone()],
    );

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    let first = usecase.route(original.clone());
    assert_eq!(first.status, ModelRoutingUsecaseStatus::Routed);

    // Same key, different capability → conflict
    let mut conflicting = original.clone();
    conflicting.request.capability = ModelCapability::Embedding;
    let conflict_receipt = usecase.route(conflicting);

    assert_eq!(conflict_receipt.status, ModelRoutingUsecaseStatus::Denied);
    assert_eq!(
        conflict_receipt.denial_kind,
        Some(ModelRoutingUsecaseDenialKind::IdempotencyConflict)
    );

    // Original is still retrievable
    let replay = usecase.route(original);
    assert_eq!(
        replay, first,
        "original receipt must survive a conflicting-payload attempt"
    );
    assert_eq!(usecase.cached_receipt_count(), 1);
}

/// The idempotency store does not re-run the catalog walk on a replay — the
/// cached_receipt_count stays at 1 after multiple replays of the same key.
#[test]
fn sub3_multiple_replays_do_not_grow_the_cache() {
    let p = full_profile(ModelProvider::Anthropic, "claude-chat", 1);
    let inp = base_input("idem:sub3:no-cache-growth", vec![p]);

    let mut usecase = IntelligenceModelRoutingUsecase::default();
    usecase.route(inp.clone());
    usecase.route(inp.clone());
    usecase.route(inp.clone());

    assert_eq!(
        usecase.cached_receipt_count(),
        1,
        "multiple replays of the same key must not grow the receipt cache"
    );
}
