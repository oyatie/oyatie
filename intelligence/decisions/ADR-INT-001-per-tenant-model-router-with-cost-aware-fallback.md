---
id: ADR-INT-001
title: Per-Tenant Model Router with Cost-Aware Fallback
status: Proposed
date: 2026-05-20
microservice: intelligence
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0708-platform-foundations-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
decision_owner: axis-intelligence
---

# ADR-INT-001: Per-Tenant Model Router with Cost-Aware Fallback

## Context

- Intelligence is the canonical AI substrate for model routing, provider adapters, guardrails, evaluation, attribution, credential resolution, audit tap, and brand UX rendering.
- ADR-0255 splits the AI surface into substrate Layer-A and consumer brand UX Layer-B; Layer-B never bypasses Layer-A.
- The local architecture names `model-routing`, `providers`, `guardrails`, `eval`, `attribution`, `credential-resolver`, and `audit-tap` bounded contexts.
- The local manifest declares audience types `CONSUMER`, `DEVELOPER`, `INTERNAL_FOUNDRY`, `EMERGENCY_SERVICES`, `MINOR_TARGETED`, and `HIGH_RISK_USER`.
- The local manifest declares provider credential mode overrides for healthcare, federal, and China packs where provider-credential BYOK is required (ADR-0255 §D-4).
- The local PRD says output is advisory draft or citation-bearing retrieval, not direct mutation of tenant configuration.
- ADR-0243 requires Cedar for dispatch, provider selection, refusal baseline, credential resolution, auditor reads, CI reads, and cross-tenant isolation.
- ADR-0244 requires every dispatch to carry tenant, principal, audience, home cell, jurisdiction, and data class.
- ADR-0308 requires AI model lifecycle compliance evidence for regulated model use.
- EU AI Act Article 43 conformity assessment duties affect high-risk AI systems before placing them on the market or putting them into service.
- User-facing AI requests can vary from low-risk drafting to high-risk workflows involving employment, education, finance, healthcare, safety, or legal access.
- A cheaper fallback model can reduce spend, but unsafe fallback can violate residency, risk, quality, provider-contract, or tenant provider-credential BYOK constraints (ADR-0255 §D-4).
- A higher-quality model can improve completion quality, but not every tenant budget or pack allows every provider.
- Provider outages must degrade predictably without hiding the fact that a fallback occurred.
- Tenant policy may require a fixed provider, tenant-owned provider keys, no training retention, regional endpoint, or model family allowlist.
- Emergency-services traffic may need latency-prioritized routing but still cannot skip audit or safety policy.
- Minor-targeted traffic requires stricter safety and refusal policy than default consumer traffic.
- Internal Foundry traffic can have different autonomy ceilings from consumer assistant traffic.
- The router must treat cost as a policy dimension, not a hidden implementation optimization.
- The router must track token cost, cache hit cost, tool cost, retrieval cost, and fallback cost.
- The router must preserve citation attribution and source ids across fallbacks.
- The router must not retry prompts across providers if the first provider received regulated personal data and the next provider lacks residency or provider-credential BYOK approval (ADR-0255 §D-4).
- The router must never downgrade from a high-risk reviewed model to an unreviewed model for an EU high-risk pathway.
- The router must emit audit evidence for route decision, refusal, fallback, provider credential resolution, and budget exhaustion.
- The router must provide tenants with explainable controls: preferred model, max cost per request, fallback allowed, quality floor, and residency constraints.
- The router must preserve evaluation fixtures so future model changes are regression-tested.
- The router must integrate with finops-portal so model spend is visible per tenant, capability, and product surface.
- The router must integrate with observability so first-token latency, completion latency, refusal correctness, and audit emission success are measurable.
- The router must support local provider adapters and remote provider adapters without exposing provider SDK details to product services.
- The router must define exact numeric fallback thresholds and stop conditions.

## Decision

- Adopt a per-tenant model router as the only dispatch path from product code to model providers.
- Route every request through `model-routing` before provider adapter invocation.
- Evaluate Cedar policy before provider selection and again before credential resolution.
- Represent tenant routing config as signed policy-backed data, not application constants.
- Use a `DispatchEnvelope` that includes tenant, principal, audience, data class, risk tier, purpose, modality, budget, quality floor, and residency.
- Use risk tiers `minimal`, `limited`, `high-impact`, and `eu-high-risk-review-required`.
- Treat EU AI Act Article 43 scoped requests as `eu-high-risk-review-required` unless legal/compliance classification says otherwise.
- Require a model release record with evaluation evidence before any model can serve `eu-high-risk-review-required` traffic.
- Require conformity-assessment evidence pointer before routing high-risk EU traffic to a tenant-visible feature.
- Use tenant BYOK provider credentials where pack or tenant config requires it; never fallback to platform-default credentials for BYOK-required dispatch.
- Use cost-aware fallback only after policy, residency, risk, quality, and credential constraints have produced an allowed candidate set.
- Use three route classes: `primary`, `same_quality_fallback`, and `budget_fallback`.
- Use primary route when tenant budget headroom is at least 15 percent of the monthly AI budget and provider health is green.
- Use same-quality fallback when primary provider is unhealthy, rate-limited, or above p95 latency budget.
- Use budget fallback when tenant budget headroom is below 15 percent and an approved lower-cost model meets the task quality floor.
- Stop fallback after two provider attempts or one provider attempt for high-risk traffic unless explicit human-approved retry exists.
- Do not send the same prompt to a second provider after a completed refusal, safety block, or policy denial.
- Do not send protected health, financial, child, employment, education, or legal data to a fallback provider unless the provider is approved for that data class and region.
- Use tenant-specific quality floors: default `eval_score >= 0.92` for high-impact tasks, `>= 0.85` for limited tasks, and `>= 0.75` for minimal drafting.
- Use first-token latency target p95 below 1.5 seconds for interactive assistants and completion latency p95 below 8 seconds for standard draft tasks.
- Attach `route_decision_id` to every provider request and returned assistant response.
- Emit `EVT-INT-ROUTE-SELECTED`, `EVT-INT-FALLBACK-USED`, `EVT-INT-BUDGET-FALLBACK-SUPPRESSED`, and `EVT-INT-HIGH-RISK-ROUTE-DENIED`.
- Keep provider adapters behind `ProviderAdapterPort` and normalize request, streaming, tool-call, citation, and usage outputs.
- Store normalized token and cost usage in finops-compatible events.
- Store model eval evidence separately from runtime dispatch records and link by `model_release_id`.
- Keep prompt and completion retention off by default unless tenant policy explicitly enables retention for eval or debugging.
- Require refusal-baseline post-check before output reaches Layer-B brand UX.
- Require attribution context for retrieval-augmented requests; absence of citations triggers refusal or degraded answer depending on task type.
- Use OpenBao sidecar for credential resolution with <=60 second provider credential lease.
- Keep route policies additively versioned and soaked for 60 seconds before activation.
- Publish model route decisions as tenant-visible explanations where the route does not expose sensitive security signals.

## Alternatives Considered

### Product-specific provider calls

- Pros: product teams can optimize quickly for their own UX.
- Pros: fewer substrate dependencies for prototypes.
- Pros: direct provider SDK examples are easy to follow.
- Cons: policy, audit, cost, residency, and safety behavior diverge across products.
- Cons: tenants cannot set one provider or budget policy for all AI surfaces.
- Cons: EU AI Act and model lifecycle evidence become impossible to enforce consistently.
- Rejected because ADR-0255 makes intelligence the single AI substrate chokepoint.

### One global default model

- Pros: simple routing and uniform evaluation.
- Pros: easier caching and provider contract management.
- Pros: predictable UX behavior across tenants.
- Cons: ignores tenant provider-credential BYOK, residency, cost, modality, and risk differences (ADR-0255 §D-4).
- Cons: creates single-provider outage and account-risk concentration.
- Cons: overpays for low-risk tasks and under-controls high-risk tasks.
- Rejected because Oyatie needs tenant policy, pack policy, and cost-aware model selection.

### Lowest-cost routing first

- Pros: simple FinOps posture.
- Pros: keeps tenant AI spend controlled.
- Pros: encourages cheap default experiences.
- Cons: can silently degrade quality, safety, citations, or regulatory posture.
- Cons: high-impact workflows need evidence-backed quality floors.
- Cons: tenant trust suffers if cost optimization is hidden.
- Rejected because cost is a constraint after policy and quality, not the primary objective.

### External model gateway SaaS

- Pros: provider abstraction, dashboards, and retries are available quickly.
- Pros: can reduce adapter implementation work.
- Pros: may provide provider health and cost reporting out of the box.
- Cons: creates another processor of sensitive prompts and tenant data.
- Cons: can obscure Cedar, audit-chain, and EU AI Act evidence.
- Cons: conflicts with in-house substrate control for a load-bearing surface.
- Rejected for the canonical router; third-party gateways may be benchmarked but not the policy authority.

### No fallback for any request

- Pros: deterministic provider behavior and easy audit story.
- Pros: avoids cross-provider data exposure.
- Pros: simpler provider and cost accounting.
- Cons: poor availability under provider incidents.
- Cons: tenants with budget pressure lose useful lower-risk assistance instead of controlled degradation.
- Cons: product UX degrades too sharply for low-risk drafting.
- Rejected because bounded, policy-aware fallback improves resilience without weakening high-risk controls.

## Consequences

- Positive: every model call has tenant, policy, cost, risk, and audit context.
- Positive: tenants can control provider credentials, preferred models, and fallback permissions centrally.
- Positive: high-risk EU workflows can be blocked until conformity evidence exists.
- Positive: FinOps can attribute model spend by route decision and fallback reason.
- Positive: outages can degrade to approved alternatives without product-code changes.
- Positive: model lifecycle evidence becomes enforceable at dispatch time.
- Positive: refusal and citation checks are centralized.
- Negative: router complexity becomes high and must be tested with adversarial fixtures.
- Negative: provider adapter surface must keep up with streaming, tool-call, and usage differences.
- Negative: route decision explanations may expose sensitive provider health or policy details if not redacted.
- Negative: strict high-risk controls may reject requests that product teams expected to work.
- Negative: budget fallback can create visible quality changes and needs tenant-facing explanation.
- Neutral: model providers remain replaceable because product code depends on the router, not SDKs.
- Neutral: no single quality score can prove broad safety; eval sets remain task-specific.
- Neutral: Article 43 gating is a compliance control, not a claim that every feature is high-risk.
- Neutral: some tenants may choose fixed-provider no-fallback mode.
- Follow-up: add a route-policy schema and canonicalen route fixtures.
- Follow-up: add conformity evidence registry entries for approved high-risk routes.
- Follow-up: add finops-portal model-spend dashboard dimensions.
- Follow-up: add incident runbook for provider outage and fallback suppression.
- Follow-up: add tenant-facing explanation copy for budget fallback and policy denial.

## Implementation Notes

- Data shape `DispatchEnvelope`: `{tenant_id, principal_id, audience_type, purpose, data_class, risk_tier, modality, prompt_ref, retrieval_refs, budget_ref}`.
- Data shape `RoutePolicy`: `{tenant_id, allowed_providers, denied_providers, preferred_models, fallback_allowed, max_cost_minor, residency_set, quality_floor}`.
- Data shape `ModelCandidate`: `{provider_id, model_id, model_release_id, modality, region, cost_per_1k_input, cost_per_1k_output, eval_score, approved_data_classes}`.
- Data shape `RouteDecision`: `{route_decision_id, dispatch_id, selected_candidate, fallback_class, reason_codes, policy_version, estimated_cost, budget_headroom}`.
- Data shape `ProviderAttempt`: `{attempt_id, route_decision_id, provider_id, model_id, credential_mode, state, latency_ms, usage, normalized_error}`.
- Data shape `ModelReleaseEvidence`: `{model_release_id, eval_set_id, eval_score, risk_approval, conformity_ref, activated_at, deprecated_at}`.
- Data shape `UsageCostEvent`: `{tenant_id, dispatch_id, route_decision_id, model_id, input_tokens, output_tokens, cache_tokens, effective_cost}`.
- REST endpoint `POST /v1/intelligence/dispatch` accepts dispatch envelopes and returns streaming or non-streaming responses.
- REST endpoint `POST /v1/intelligence/route-simulations` returns selected candidate and denial reasons without provider calls.
- REST endpoint `GET /v1/intelligence/model-releases/{id}/evidence` returns approved evidence metadata.
- REST endpoint `POST /v1/intelligence/tenant-route-policies` creates or updates signed tenant route policy.
- REST endpoint `GET /v1/intelligence/dispatch/{id}/route-decision` returns tenant-visible explanation.
- Async event `intelligence.route.selected.v1` carries route decision metadata.
- Async event `intelligence.fallback.used.v1` carries fallback class and suppression-safe reason code.
- Async event `intelligence.high_risk_route.denied.v1` carries missing evidence or policy reason.
- Async event `intelligence.usage.cost_recorded.v1` feeds finops-portal.
- Cedar permit `intelligence::dispatch::execute` requires tenant scope, audience tag, purpose, and data class eligibility.
- Cedar permit `intelligence::provider::select` requires provider allowlist, residency, model approval, and risk compatibility.
- Cedar forbid `intelligence::provider::fallback` blocks fallback across provider-credential BYOK, residency, or high-risk boundaries (ADR-0255 §D-4).
- Cedar permit `intelligence::credential::resolve` requires provider credential mode and OpenBao lease eligibility.
- Cedar forbid `intelligence::output::release` blocks responses failing refusal-baseline or citation requirements.
- SLO target `dispatch_api_availability`: 99.9 percent monthly for low and limited risk requests.
- SLO target `first_token_latency`: p95 below 1.5 seconds for interactive dispatches.
- SLO target `assist_draft_latency`: p95 below 8 seconds for standard draft tasks.
- SLO target `policy_refusal_correctness`: 100 percent pass on high-severity refusal fixtures.
- SLO target `audit_emission_success`: 100 percent for state-changing or high-risk route events.
- Metric `intelligence_route_decision_total` dimensions include route class, provider, risk tier, and reason code.
- Metric `intelligence_fallback_suppressed_total` dimensions include suppression reason.
- Metric `intelligence_budget_headroom_ratio` feeds FinOps budget fallback decisions.
- Metric `intelligence_high_risk_denial_total` feeds compliance review.
- OpenBao path `secret/<tenant_id>/intelligence/provider/<provider_id>/<credential_epoch>` stores credential handles.
- Provider adapter output normalizes text, tool calls, citations, refusal class, token usage, and provider error.
- Budget fallback estimates use current provider pricing cache and tenant monthly budget ledger.
- Prompt and completion retention defaults to zero days for tenant content unless policy enables retention.
- High-risk dispatch logs store prompt hash, model release id, evidence refs, and decision ids, not raw prompt text.
- Route policy rollback reverts signed policy pointer and emits an audit event.

## Verification

- Unit test `byok_required_never_uses_platform_default` validates credential boundary.
- Unit test `high_risk_requires_model_release_evidence` blocks missing evidence.
- Unit test `article_43_route_requires_conformity_ref` validates EU high-risk gate.
- Unit test `budget_fallback_requires_quality_floor` prevents cheap unsafe downgrade.
- Unit test `fallback_stops_after_two_attempts` validates stop condition.
- Unit test `refusal_response_not_retried_to_second_provider` prevents policy bypass.
- Property test `route_policy_constraints_monotonic` generates stricter pack overlays.
- Property test `candidate_set_never_contains_disallowed_data_class` validates data-class gates.
- Fuzz test `provider_error_normalization_total` validates adapter error mapping.
- Integration test `provider_outage_same_quality_fallback` validates outage routing.
- Integration test `low_budget_budget_fallback_with_explanation` validates cost-aware routing.
- Integration test `tenant_fixed_provider_blocks_fallback` validates tenant policy.
- Integration test `minor_targeted_dispatch_uses_strict_refusal_baseline` validates audience policy.
- Integration test `retrieval_request_without_citation_denied` validates attribution gate.
- Integration test `usage_cost_event_reaches_finops` validates spend attribution.
- Load test `dispatch_first_token_p95_under_1_5s` validates interactive SLO.
- Load test `route_simulation_1000_qps_no_provider_calls` validates dry-run endpoint.
- Chaos test `cedar_timeout_fails_closed` validates policy dependency.
- Chaos test `audit_chain_backpressure_blocks_high_risk_dispatch` validates audit-first invariant.
- Dashboard check `model-router-health` shows provider health, fallback, and denial rates.
- Dashboard check `model-spend` shows cost by tenant, route class, and model release.
- Metric check `intelligence_fallback_suppressed_total` increments for provider-credential BYOK boundary tests (ADR-0255 §D-4).
- Static check product services do not import provider SDKs directly.
- Static check every provider adapter implements usage and citation normalization.
- Oya VCS evidence must include line count, root ADR cite count, and reference count for this ADR.

## References

- Regulation (EU) 2024/1689, AI Act, Article 43 Conformity Assessment.
- European Commission AI Act service desk, Article 43 explanatory page.
- NIST AI Risk Management Framework 1.0.
- ISO/IEC 42001:2023 AI management system standard.
- Cedar Policy Language authorization and schema documentation: https://docs.cedarpolicy.com/
- OpenTelemetry semantic conventions for AI and HTTP telemetry where applicable.
- AWS Bedrock and Azure AI Foundry model evaluation and guardrail documentation as benchmark references.
- ADR-0211, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0255, ADR-0263, and ADR-0308.
- Local intelligence PRD, architecture, manifest, provider adapter trait, SLOs, and refusal runbooks.
