//! Acceptance tests for the attribution composition root, grounded in the
//! vertical slice's actual behaviors:
//! - `oya-intelligence-attribution-kernel::plan_attribution`
//! - `oya-intelligence-attribution-domain::plan_domain_attribution`
//! - `oya-intelligence-attribution-usecase::IntelligenceAttributionUsecase::plan`
//! - `oya-intelligence-attribution-adapter::IntelligenceAttributionAdapter::dispatch`
//!
//! These drive the FULL flow through the REAL kernel/domain/usecase pipeline
//! plus the REAL renderer adapter. There are NO stubs for those layers. The
//! renderer transport is the in-memory scripted adapter so the dispatch loop
//! is deterministic (acceptance tests must not require network egress).
//!
//! Mapped behaviors (vertical-slice test contract):
//! - happy-path citation planning + envelope dispatch    (kernel `plans_metadata_only_citations_for_public_external_answer` + adapter `builds_metadata_only_citation_renderer_envelope`)
//! - cross-tenant isolation by (TenantId, IdempotencyKey)
//! - idempotency cache short-circuits re-execution        (usecase `idempotent_replay_and_conflict_are_deterministic`)
//! - kernel/domain denial surfaces verbatim               (domain `audience_source_kind_and_data_class_policy_are_enforced`)
//! - missing policy default-denies before usecase work
//! - renderer transport retryable failure exhausts retries
//! - renderer transport non-retryable short-circuits
//! - HyperCitationRendererTransport honest boundary       (Unimplemented::OpenBaoCredentialResolution)

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::{Arc, Mutex};

use intelligence_attribution_app::{
    AttributionAudience, AttributionAuditEventKind, AttributionAuditSink, AttributionClaim,
    AttributionDataClass, AttributionDispatchError, AttributionDispatchRequest,
    AttributionPolicyDecision, AttributionPolicyStore, AttributionRendererAdapterConfig,
    AttributionRendererDispatchReceipt, AttributionRendererDispatchStatus, AttributionRepository,
    AttributionRequest, AttributionSource, AttributionSourceKind, AttributionUsecaseStatus,
    DomainAttributionRequest, HyperCitationRendererTransport, IdempotencyKey,
    InMemoryAttributionAuditSink, InMemoryAttributionPolicyStore, InMemoryAttributionRepository,
    InMemoryCitationRendererTransport, IntelligenceAttributionAdapter,
    IntelligenceAttributionUsecase, PrincipalId, RendererTransportError, RendererTransportScript,
    TenantId, Unimplemented, dispatch_attribution,
};

fn ten(s: &str) -> TenantId {
    TenantId(s.to_owned())
}

fn pid(s: &str) -> PrincipalId {
    PrincipalId(s.to_owned())
}

fn idk(s: &str) -> IdempotencyKey {
    IdempotencyKey(s.to_owned())
}

fn sample_policy(
    tenant: &str,
    principal: &str,
    evidence_suffix: &str,
) -> AttributionPolicyDecision {
    AttributionPolicyDecision {
        decision_id: format!("attribution-policy-decision:{evidence_suffix}"),
        tenant_id: tenant.to_owned(),
        principal_id: principal.to_owned(),
        allowed_surfaces: vec!["surface:dispatch-response".to_owned()],
        allowed_audiences: vec![AttributionAudience::External, AttributionAudience::Internal],
        allowed_source_kinds: vec![
            AttributionSourceKind::KnowledgeGraph,
            AttributionSourceKind::PolicyDocument,
            AttributionSourceKind::RetrievalDocument,
        ],
        allowed_data_classes: vec![AttributionDataClass::Public, AttributionDataClass::Internal],
        max_citations: 8,
        min_confidence_bps: 7_000,
        evidence_ref: format!("policy:evidence:attribution-app:{evidence_suffix}"),
        attribution_registry_snapshot_ref: format!(
            "attribution-registry:snapshot:app:{evidence_suffix}"
        ),
    }
}

fn sample_kernel_request(tenant: &str, evidence_suffix: &str) -> AttributionRequest {
    AttributionRequest {
        tenant_id: tenant.to_owned(),
        output_ref: format!("answer://responses/resp-app-{evidence_suffix}"),
        audience: AttributionAudience::External,
        policy_evidence_ref: format!("policy:evidence:attribution-app:{evidence_suffix}"),
        trace_context_ref: format!("trace:attribution-app:{evidence_suffix}"),
        max_citations: 8,
        max_citations_per_claim: 8,
        sources: vec![
            AttributionSource {
                source_id: "src-kg-policy".to_owned(),
                resource_ref: "kg://entity/accounting-policy".to_owned(),
                title_ref: "title://knowledge/accounting-policy".to_owned(),
                source_kind: AttributionSourceKind::KnowledgeGraph,
                data_class: AttributionDataClass::Public,
                evidence_ref: "evidence:kg:accounting-policy".to_owned(),
                freshness_epoch_seconds: 1_779_523_200,
            },
            AttributionSource {
                source_id: "src-doc-refund".to_owned(),
                resource_ref: "doc://help-center/refund-policy".to_owned(),
                title_ref: "title://help/refund-policy".to_owned(),
                source_kind: AttributionSourceKind::RetrievalDocument,
                data_class: AttributionDataClass::Public,
                evidence_ref: "evidence:doc:refund-policy".to_owned(),
                freshness_epoch_seconds: 1_779_523_201,
            },
        ],
        claims: vec![
            AttributionClaim {
                claim_id: "claim-2".to_owned(),
                answer_segment_ref: format!("answer-segment://resp-app-{evidence_suffix}/2"),
                source_ids: vec!["src-doc-refund".to_owned()],
                confidence_bps: 9_000,
            },
            AttributionClaim {
                claim_id: "claim-1".to_owned(),
                answer_segment_ref: format!("answer-segment://resp-app-{evidence_suffix}/1"),
                source_ids: vec!["src-kg-policy".to_owned()],
                confidence_bps: 9_200,
            },
        ],
    }
}

fn sample_domain_request(
    tenant: &str,
    principal: &str,
    evidence_suffix: &str,
) -> DomainAttributionRequest {
    DomainAttributionRequest {
        tenant_id: tenant.to_owned(),
        principal_id: principal.to_owned(),
        attribution_surface: "surface:dispatch-response".to_owned(),
        request_evidence_ref: format!("request:evidence:attribution-app:{evidence_suffix}"),
        trace_context_ref: format!("trace:attribution-app:{evidence_suffix}"),
        policy_decision_ref: format!("policy:evidence:attribution-app:{evidence_suffix}"),
        policy_decision: sample_policy(tenant, principal, evidence_suffix),
        request: sample_kernel_request(tenant, evidence_suffix),
    }
}

fn accepted_adapter(evidence_suffix: &str) -> IntelligenceAttributionAdapter {
    IntelligenceAttributionAdapter::try_new(
        AttributionRendererAdapterConfig::new(
            "https://citation-renderer.oyatie.internal/",
            "secretref://ten_a/citation-renderer/byok",
            "audit://tap/intelligence/attribution",
            "audience://intelligence/citation-renderer",
        ),
        intelligence_attribution_app::AttributionRendererStatus::Accepted {
            renderer_request_ref: format!("citation-renderer://requests/req-{evidence_suffix}"),
            render_ref: format!("citation-renderer://renders/render-{evidence_suffix}"),
            evidence_ref: format!("citation-renderer:evidence:accepted:{evidence_suffix}"),
        },
    )
    .expect("valid adapter config")
}

fn ok_renderer_script() -> RendererTransportScript {
    Arc::new(|envelope| {
        Ok(AttributionRendererDispatchReceipt {
            status: AttributionRendererDispatchStatus::Accepted,
            renderer_request_ref: Some(format!(
                "citation-renderer://requests/transport-ok-{}",
                envelope.tenant_id
            )),
            render_ref: Some(format!(
                "citation-renderer://renders/transport-ok-{}",
                envelope.tenant_id
            )),
            queue_ref: None,
            citation_bundle_ref: None,
            evidence_ref: format!(
                "citation-renderer:evidence:transport-ok:{}",
                envelope.tenant_id
            ),
        })
    })
}

/// AC: happy-path — a known policy + a fresh idempotency key runs the full
/// kernel/domain/usecase pipeline, persists the receipt, and dispatches the
/// renderer envelope.
#[tokio::test]
async fn happy_path_dispatches_envelope_after_kernel_plan() {
    let tenant = ten("tenant:alpha");
    let principal = pid("principal:attribution-owner");
    let mut repo = InMemoryAttributionRepository::new();
    let policy_store = InMemoryAttributionPolicyStore::new().with_decision(
        tenant.clone(),
        principal.clone(),
        sample_policy("tenant:alpha", "principal:attribution-owner", "happy"),
    );
    let mut sink = InMemoryAttributionAuditSink::new();
    let mut adapter = accepted_adapter("happy");
    let transport = InMemoryCitationRendererTransport::new(ok_renderer_script());
    let mut usecase = IntelligenceAttributionUsecase::default();

    let outcome = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase,
        AttributionDispatchRequest {
            tenant_id: tenant.clone(),
            principal_id: principal.clone(),
            idempotency_key: idk("idem:app:happy-1"),
            domain_request: sample_domain_request(
                "tenant:alpha",
                "principal:attribution-owner",
                "happy",
            ),
            max_renderer_retries: 1,
        },
    )
    .await
    .expect("happy-path dispatch must succeed");

    assert_eq!(
        outcome.usecase_receipt.status,
        AttributionUsecaseStatus::Rendered
    );
    assert_eq!(outcome.usecase_receipt.citation_count, 2);
    assert!(!outcome.served_from_cache);
    assert_eq!(outcome.attempts, 1);
    assert_eq!(repo.len(), 1);
    // AttributionRequested + AttributionRendered events were fanned to the sink.
    assert_eq!(sink.len(), 2);
    assert_eq!(
        sink.events()[0].kind,
        AttributionAuditEventKind::AttributionRequested
    );
    assert_eq!(
        sink.events()[1].kind,
        AttributionAuditEventKind::AttributionRendered
    );
    // Adapter recorded the envelope; transport saw exactly one envelope.
    let envelope = adapter
        .last_envelope()
        .expect("adapter must retain envelope");
    assert_eq!(envelope.tenant_id, "tenant:alpha");
    assert_eq!(envelope.citation_count, 2);
    assert_eq!(transport.call_log().len(), 1);
}

/// AC: idempotency — replaying the same `(tenant, idempotency_key)` skips the
/// kernel/domain/usecase work and serves the receipt from the cache; the
/// renderer is still dispatched against (idempotent request to the renderer
/// is expected because the envelope carries the idempotency key).
#[tokio::test]
async fn idempotent_replay_serves_from_cache_without_kernel_reentry() {
    let tenant = ten("tenant:alpha");
    let principal = pid("principal:attribution-owner");
    let mut repo = InMemoryAttributionRepository::new();
    let policy_store = InMemoryAttributionPolicyStore::new().with_decision(
        tenant.clone(),
        principal.clone(),
        sample_policy("tenant:alpha", "principal:attribution-owner", "idem"),
    );
    let mut sink = InMemoryAttributionAuditSink::new();
    let mut adapter = accepted_adapter("idem");
    let transport = InMemoryCitationRendererTransport::new(ok_renderer_script());
    let mut usecase = IntelligenceAttributionUsecase::default();

    let request = AttributionDispatchRequest {
        tenant_id: tenant.clone(),
        principal_id: principal.clone(),
        idempotency_key: idk("idem:app:replay-1"),
        domain_request: sample_domain_request(
            "tenant:alpha",
            "principal:attribution-owner",
            "idem",
        ),
        max_renderer_retries: 1,
    };

    let first = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase,
        request.clone(),
    )
    .await
    .expect("first dispatch must succeed");
    assert!(!first.served_from_cache);
    let usecase_events_after_first = usecase.audit_events().len();

    let replay = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase,
        request,
    )
    .await
    .expect("replay must succeed from cache");
    assert!(replay.served_from_cache);
    // The kernel/usecase layer was NOT re-entered: usecase audit-event
    // count is unchanged.
    assert_eq!(usecase.audit_events().len(), usecase_events_after_first);
    // Receipt is identical to the first dispatch.
    assert_eq!(first.usecase_receipt, replay.usecase_receipt);
    // Renderer transport was invoked twice (idempotent at the renderer).
    assert_eq!(transport.call_log().len(), 2);
}

/// AC: cross-tenant isolation — separate `(TenantId)`s do NOT share cache or
/// policy decisions even with the same idempotency key. Mirrors a sharded
/// production deployment where each tenant context owns its own usecase
/// aggregate behind the composition root.
#[tokio::test]
async fn cross_tenant_isolation_does_not_share_cache_or_policy() {
    let tenant_a = ten("tenant:alpha");
    let tenant_b = ten("tenant:beta");
    let principal = pid("principal:attribution-owner");
    let mut repo = InMemoryAttributionRepository::new();
    let policy_store = InMemoryAttributionPolicyStore::new()
        .with_decision(
            tenant_a.clone(),
            principal.clone(),
            sample_policy("tenant:alpha", "principal:attribution-owner", "iso-a"),
        )
        .with_decision(
            tenant_b.clone(),
            principal.clone(),
            sample_policy("tenant:beta", "principal:attribution-owner", "iso-b"),
        );
    let mut sink = InMemoryAttributionAuditSink::new();
    let mut adapter = accepted_adapter("iso");
    let transport = InMemoryCitationRendererTransport::new(ok_renderer_script());
    // Per-tenant usecase aggregates: the composition root keys the
    // idempotency cache by tenant via the AttributionRepository, but the
    // inner usecase layer itself is one aggregate per tenant context in
    // production (sharded by tenant). The test mirrors that posture.
    let mut usecase_a = IntelligenceAttributionUsecase::default();
    let mut usecase_b = IntelligenceAttributionUsecase::default();

    let outcome_a = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase_a,
        AttributionDispatchRequest {
            tenant_id: tenant_a.clone(),
            principal_id: principal.clone(),
            idempotency_key: idk("idem:shared"),
            domain_request: sample_domain_request(
                "tenant:alpha",
                "principal:attribution-owner",
                "iso-a",
            ),
            max_renderer_retries: 1,
        },
    )
    .await
    .expect("tenant-a dispatch must succeed");
    assert!(!outcome_a.served_from_cache);

    let outcome_b = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase_b,
        AttributionDispatchRequest {
            tenant_id: tenant_b.clone(),
            principal_id: principal.clone(),
            idempotency_key: idk("idem:shared"),
            domain_request: sample_domain_request(
                "tenant:beta",
                "principal:attribution-owner",
                "iso-b",
            ),
            max_renderer_retries: 1,
        },
    )
    .await
    .expect("tenant-b dispatch must succeed");
    // Tenant-b sees its own fresh kernel/domain run; not a cache hit from
    // tenant-a's identical idempotency key.
    assert!(!outcome_b.served_from_cache);
    assert_eq!(repo.len(), 2);
    let cached_a = repo
        .load(&tenant_a, &idk("idem:shared"))
        .unwrap()
        .expect("tenant-a receipt persisted");
    let cached_b = repo
        .load(&tenant_b, &idk("idem:shared"))
        .unwrap()
        .expect("tenant-b receipt persisted");
    assert_eq!(cached_a.tenant_id, "tenant:alpha");
    assert_eq!(cached_b.tenant_id, "tenant:beta");
}

/// AC: missing policy — default-deny without running the kernel/domain/
/// usecase pipeline; renderer transport must NOT be called.
#[tokio::test]
async fn missing_policy_default_denies_before_kernel_or_renderer() {
    let tenant = ten("tenant:alpha");
    let principal = pid("principal:attribution-owner");
    let mut repo = InMemoryAttributionRepository::new();
    let policy_store = InMemoryAttributionPolicyStore::new();
    let mut sink = InMemoryAttributionAuditSink::new();
    let mut adapter = accepted_adapter("missing-policy");
    let script: RendererTransportScript =
        Arc::new(|_| panic!("renderer transport must not be called when policy is missing"));
    let transport = InMemoryCitationRendererTransport::new(script);
    let mut usecase = IntelligenceAttributionUsecase::default();

    let err = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase,
        AttributionDispatchRequest {
            tenant_id: tenant.clone(),
            principal_id: principal.clone(),
            idempotency_key: idk("idem:app:missing-policy"),
            domain_request: sample_domain_request(
                "tenant:alpha",
                "principal:attribution-owner",
                "missing-policy",
            ),
            max_renderer_retries: 1,
        },
    )
    .await
    .expect_err("missing policy must default-deny");
    match err {
        AttributionDispatchError::PolicyNotFound {
            tenant_id,
            principal_id,
        } => {
            assert_eq!(tenant_id, "tenant:alpha");
            assert_eq!(principal_id, "principal:attribution-owner");
        }
        other => panic!("expected PolicyNotFound, got {other:?}"),
    }
    assert!(repo.is_empty());
    assert!(sink.is_empty());
    assert!(transport.call_log().is_empty());
    assert!(adapter.last_envelope().is_none());
}

/// AC: kernel denial (missing source) surfaces verbatim through the
/// composition root; the renderer transport must NOT be called.
#[tokio::test]
async fn kernel_missing_source_denial_short_circuits_before_renderer() {
    let tenant = ten("tenant:alpha");
    let principal = pid("principal:attribution-owner");
    let mut repo = InMemoryAttributionRepository::new();
    let policy_store = InMemoryAttributionPolicyStore::new().with_decision(
        tenant.clone(),
        principal.clone(),
        sample_policy("tenant:alpha", "principal:attribution-owner", "kernel-deny"),
    );
    let mut sink = InMemoryAttributionAuditSink::new();
    let mut adapter = accepted_adapter("kernel-deny");
    let script: RendererTransportScript =
        Arc::new(|_| panic!("renderer transport must not be called on kernel/domain denial"));
    let transport = InMemoryCitationRendererTransport::new(script);
    let mut usecase = IntelligenceAttributionUsecase::default();

    let mut domain =
        sample_domain_request("tenant:alpha", "principal:attribution-owner", "kernel-deny");
    // Inject a claim referencing a source id that's not in the sources list.
    domain.request.claims[0]
        .source_ids
        .push("src-missing".to_owned());

    let err = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase,
        AttributionDispatchRequest {
            tenant_id: tenant.clone(),
            principal_id: principal.clone(),
            idempotency_key: idk("idem:app:kernel-deny"),
            domain_request: domain,
            max_renderer_retries: 1,
        },
    )
    .await
    .expect_err("kernel denial must surface");

    match err {
        AttributionDispatchError::UsecaseDenied { receipt } => {
            assert_eq!(receipt.status, AttributionUsecaseStatus::Denied);
            assert!(receipt.denial_kind.is_some());
        }
        other => panic!("expected UsecaseDenied, got {other:?}"),
    }
    // Renderer was not called; envelope was not built.
    assert!(transport.call_log().is_empty());
    assert!(adapter.last_envelope().is_none());
    // The receipt was NOT persisted (denial path).
    assert!(repo.is_empty());
    // Audit events still recorded (request + denied).
    assert_eq!(sink.len(), 2);
    assert_eq!(
        sink.events()[1].kind,
        AttributionAuditEventKind::AttributionDenied
    );
}

/// AC: domain audience denial — composition layer surfaces the typed
/// `UsecaseDenied` without rendering an envelope.
#[tokio::test]
async fn domain_audience_denial_short_circuits_before_renderer() {
    let tenant = ten("tenant:alpha");
    let principal = pid("principal:attribution-owner");
    let mut repo = InMemoryAttributionRepository::new();
    let mut policy = sample_policy(
        "tenant:alpha",
        "principal:attribution-owner",
        "audience-deny",
    );
    // Disallow External audience while the request still asks for External.
    policy.allowed_audiences = vec![AttributionAudience::Internal];
    let policy_store = InMemoryAttributionPolicyStore::new().with_decision(
        tenant.clone(),
        principal.clone(),
        policy.clone(),
    );
    let mut sink = InMemoryAttributionAuditSink::new();
    let mut adapter = accepted_adapter("audience-deny");
    let script: RendererTransportScript =
        Arc::new(|_| panic!("renderer transport must not be called on audience denial"));
    let transport = InMemoryCitationRendererTransport::new(script);
    let mut usecase = IntelligenceAttributionUsecase::default();

    let mut domain = sample_domain_request(
        "tenant:alpha",
        "principal:attribution-owner",
        "audience-deny",
    );
    domain.policy_decision = policy;

    let err = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase,
        AttributionDispatchRequest {
            tenant_id: tenant.clone(),
            principal_id: principal.clone(),
            idempotency_key: idk("idem:app:audience-deny"),
            domain_request: domain,
            max_renderer_retries: 1,
        },
    )
    .await
    .expect_err("audience denial must surface");
    match err {
        AttributionDispatchError::UsecaseDenied { receipt } => {
            assert_eq!(receipt.status, AttributionUsecaseStatus::Denied);
        }
        other => panic!("expected UsecaseDenied, got {other:?}"),
    }
    assert!(adapter.last_envelope().is_none());
    assert!(transport.call_log().is_empty());
}

/// AC: renderer transport non-retryable failure short-circuits without
/// consuming the retry budget.
#[tokio::test]
async fn renderer_non_retryable_short_circuits_without_retries() {
    let tenant = ten("tenant:alpha");
    let principal = pid("principal:attribution-owner");
    let mut repo = InMemoryAttributionRepository::new();
    let policy_store = InMemoryAttributionPolicyStore::new().with_decision(
        tenant.clone(),
        principal.clone(),
        sample_policy("tenant:alpha", "principal:attribution-owner", "non-retry"),
    );
    let mut sink = InMemoryAttributionAuditSink::new();
    let mut adapter = accepted_adapter("non-retry");
    let calls: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let counter = Arc::clone(&calls);
    let script: RendererTransportScript = Arc::new(move |_envelope| {
        if let Ok(mut guard) = counter.lock() {
            *guard += 1;
        }
        Err(RendererTransportError::NonRetryable {
            detail: "renderer wire-format rejected envelope".to_owned(),
        })
    });
    let transport = InMemoryCitationRendererTransport::new(script);
    let mut usecase = IntelligenceAttributionUsecase::default();

    let err = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase,
        AttributionDispatchRequest {
            tenant_id: tenant.clone(),
            principal_id: principal.clone(),
            idempotency_key: idk("idem:app:non-retry"),
            domain_request: sample_domain_request(
                "tenant:alpha",
                "principal:attribution-owner",
                "non-retry",
            ),
            max_renderer_retries: 4,
        },
    )
    .await
    .expect_err("non-retryable must short-circuit");
    match err {
        AttributionDispatchError::NonRetryableTransport(RendererTransportError::NonRetryable {
            detail,
        }) => {
            assert!(detail.contains("renderer wire-format rejected"));
        }
        other => panic!("expected NonRetryableTransport, got {other:?}"),
    }
    assert_eq!(*calls.lock().unwrap(), 1);
}

/// AC: renderer transport exhausts the configured retry budget on persistent
/// retryable failures.
#[tokio::test]
async fn renderer_retries_exhaust_after_configured_budget() {
    let tenant = ten("tenant:alpha");
    let principal = pid("principal:attribution-owner");
    let mut repo = InMemoryAttributionRepository::new();
    let policy_store = InMemoryAttributionPolicyStore::new().with_decision(
        tenant.clone(),
        principal.clone(),
        sample_policy(
            "tenant:alpha",
            "principal:attribution-owner",
            "retry-exhaust",
        ),
    );
    let mut sink = InMemoryAttributionAuditSink::new();
    let mut adapter = accepted_adapter("retry-exhaust");
    let script: RendererTransportScript = Arc::new(|_| {
        Err(RendererTransportError::Retryable {
            detail: "simulated 502".to_owned(),
        })
    });
    let transport = InMemoryCitationRendererTransport::new(script);
    let mut usecase = IntelligenceAttributionUsecase::default();

    let err = dispatch_attribution(
        &mut repo,
        &policy_store,
        &mut sink,
        &mut adapter,
        &transport,
        &mut usecase,
        AttributionDispatchRequest {
            tenant_id: tenant.clone(),
            principal_id: principal.clone(),
            idempotency_key: idk("idem:app:retry-exhaust"),
            domain_request: sample_domain_request(
                "tenant:alpha",
                "principal:attribution-owner",
                "retry-exhaust",
            ),
            max_renderer_retries: 3,
        },
    )
    .await
    .expect_err("retries must exhaust");
    match err {
        AttributionDispatchError::AllRetriesExhausted {
            attempts,
            last_error,
        } => {
            assert_eq!(attempts, 3);
            match last_error {
                RendererTransportError::Retryable { detail } => {
                    assert_eq!(detail, "simulated 502");
                }
                other => panic!("expected Retryable last_error, got {other:?}"),
            }
        }
        other => panic!("expected AllRetriesExhausted, got {other:?}"),
    }
    assert_eq!(transport.call_log().len(), 3);
}

/// AC: the production hyper transport scaffold surfaces the typed
/// `Unimplemented::OpenBaoCredentialResolution` honest boundary today.
#[tokio::test]
async fn hyper_transport_surfaces_honest_boundary_until_openbao_wires_in() {
    let transport =
        HyperCitationRendererTransport::new("https://citation-renderer.oyatie.internal");
    assert_eq!(
        transport.upstream_base_url(),
        "https://citation-renderer.oyatie.internal"
    );
    let envelope = intelligence_attribution_app::AttributionRendererRequestEnvelope {
        method: intelligence_attribution_app::AttributionRendererHttpMethod::Post,
        endpoint: "https://citation-renderer.oyatie.internal".to_owned(),
        path: "/v1/attribution/citation-renders".to_owned(),
        transport_mode:
            intelligence_attribution_app::AttributionRendererTransportMode::EnvelopeOnly,
        tenant_id: "tenant:alpha".to_owned(),
        principal_id: "principal:attribution-owner".to_owned(),
        attribution_surface: "surface:dispatch-response".to_owned(),
        idempotency_key: "idem:app:honest".to_owned(),
        output_ref: "answer://responses/resp-honest-1".to_owned(),
        audience: AttributionAudience::External,
        request_evidence_ref: "request:evidence:honest:1".to_owned(),
        trace_context_ref: "trace:honest:1".to_owned(),
        policy_decision_ref: "policy:evidence:honest:1".to_owned(),
        policy_evidence_ref: "policy:evidence:honest:1".to_owned(),
        attribution_registry_snapshot_ref: "attribution-registry:snapshot:honest:1".to_owned(),
        credential_handle_ref: "secretref://ten_a/citation-renderer/byok".to_owned(),
        audit_tap_ref: "audit://tap/intelligence/attribution".to_owned(),
        renderer_audience_ref: "audience://intelligence/citation-renderer".to_owned(),
        citation_count: 0,
        citation_resource_refs: Vec::new(),
        source_resource_refs: Vec::new(),
        source_title_refs: Vec::new(),
        source_evidence_refs: Vec::new(),
        claim_ids: Vec::new(),
        claim_answer_segment_refs: Vec::new(),
        claim_source_ids: Vec::new(),
        evidence_refs: Vec::new(),
        adapter_reference_refs: Vec::new(),
    };
    let result =
        intelligence_attribution_app::CitationRendererTransport::dispatch(&transport, envelope)
            .await;
    let err = result.expect_err("hyper transport is honest-claims today");
    match err {
        RendererTransportError::NonRetryable { detail } => {
            assert!(detail.contains(Unimplemented::OpenBaoCredentialResolution.as_str()));
            assert!(
                detail.contains(Unimplemented::OpenBaoCredentialResolution.placeholder_debt_id())
            );
        }
        other => panic!("expected NonRetryable, got {other:?}"),
    }
}
