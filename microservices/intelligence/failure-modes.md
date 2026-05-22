---
doc_class: FailureModeCatalog
template_id: TPL-FAILURE-MODES
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence + ops-sre-reliability
related_adrs: [ADR-0255, ADR-0263, ADR-0296]
related_artifacts:
  - microservices/intelligence/threat-model.md
  - microservices/intelligence/runbooks/
  - microservices/intelligence/incident-response.md
doc_status: published
---

# Failure Modes — intelligence µservice

## Purpose

Enumerate failure modes, the substrate's designed response, the evidence emitted, the tenant
impact, the runbook reference, and the recovery path. This document is the canonical operational
artifact reviewed by ops-sre-reliability at each on-call rotation handover.

## Failure modes table

| FM | Class | Failure | Designed response | Evidence | Runbook | Tenant impact |
|---|---|---|---|---|---|---|
| FM-01 | Provider | Anthropic API outage (5xx surge) | Fail-fast circuit-breaker; route to fallback (OpenAI / Bedrock-Claude); audit-tap records `provider_outage` | `oya_intelligence_provider_5xx_total{provider="anthropic"}` burn-rate alert | `runbooks/provider-outage-anthropic.md` | degraded latency; no failed dispatches if fallback configured |
| FM-02 | Provider | OpenAI API outage | Fallback to Azure OpenAI / Anthropic | analogous metric | `runbooks/provider-outage-openai.md` | as above |
| FM-03 | Provider | Google AI Studio + Vertex outage | Fallback to Anthropic / OpenAI | analogous | `runbooks/provider-outage-google.md` | as above |
| FM-04 | Provider | Provider rate-limit saturation (429) | Backoff per provider; token-bucket throttle pre-call; per-tenant quota enforcement | `oya_intelligence_provider_429_total` | `runbooks/provider-rate-limit-saturation.md` | dispatches may queue or refuse with `RefusalDecision::ProviderSaturated` |
| FM-05 | Credential | Sidecar credential-handle expired | Re-resolve via credential-resolver; if persistent, refuse with `RefusalDecision::CredentialUnavailable` | `oya_intelligence_credential_handle_expired_total` | `runbooks/sidecar-credential-handle-expired.md` | brief latency spike; no dispatch loss if recovery succeeds |
| FM-06 | Credential | Provider credential rotation cascade across tenants | Coordinate rotation via OpenBao; pre-warm new credentials | `oya_intelligence_credential_rotation_in_progress_total` | `runbooks/byok-rotation-tenant-cascade.md` + `docs/runbooks/byok-rotation-provider-tenant-duress.md` | none if rotation succeeds; refusal otherwise |
| FM-07 | Guardrails | Prompt-injection success detected post-call | Refuse output; audit-tap with `PromptInjectionDetected`; emit incident | `oya_intelligence_prompt_injection_detected_total` | `runbooks/prompt-injection-detected.md` | one dispatch refused; tenant notified |
| FM-08 | Guardrails | Refusal false-positive cascade (rate spike) | Roll back recent Cedar fragment change; failover refusal classifier to prior model | `oya_intelligence_refusal_false_positive_rate` burn-rate | `runbooks/refusal-false-positive-cascade.md` | over-refusal for affected cohort |
| FM-09 | Guardrails | Refusal false-negative (Annex III leak) | **Sev-1**: immediate halt for affected category; engage council-privacy + ops-legal; audit-tap forensic export | `oya_intelligence_eu_ai_act_annex_iii_leak_total` | `runbooks/refusal-false-positive-cascade.md` (inverse arm) | regulator notification per EU AI Act Art. 73 |
| FM-10 | Audit | Audit-tap emission failure (audit-chain seal stream down) | Buffer in-memory + on-disk; dispatch refuses to return until audit-tap commits (atomic) | `oya_intelligence_audit_tap_emit_failed_total` | `runbooks/audit-row-forgery-detected.md` (sibling) | brief dispatch unavailability |
| FM-11 | Audit | Audit-row forgery attempt detected | **Sev-1**: refuse all dispatch until investigation; engage ops-security | `oya_intelligence_audit_tap_signature_mismatch_total > 0` | `runbooks/audit-row-forgery-detected.md` | full dispatch halt for affected pod |
| FM-12 | Routing | Provider catalog drift (provider deprecates a model) | Detect via catalog versioning; route to successor model; audit-tap with `ModelDeprecated`; tenant notification | `oya_intelligence_provider_model_deprecated_total` | `docs/runbooks/byok-rotation-provider-tenant-duress.md` | tenant SDK call may see `ModelDeprecationWarning` |
| FM-13 | Routing | Per-tenant cost cap exceeded | Refuse with `RefusalDecision::CostCapExceeded`; emit alert to ops-finops + tenant | `oya_intelligence_tenant_cost_cap_exceeded_total` | `runbooks/provider-rate-limit-saturation.md` §"Cost cap" | dispatches refused until cap raised or cycle reset |
| FM-14 | Eval | Eval canonicalen-set score regression | Block promotion via SLO gate; eval-worker pages axis-intelligence | `oya_intelligence_eval_score_drop_total` | `runbooks/refusal-false-positive-cascade.md` (eval arm) | promotion blocked; dispatch continues |
| FM-15 | Routing | Cross-pack misroute attempt (EU→US without SCC) | Refuse with `RefusalDecision::DataResidencyViolation` | `oya_intelligence_cross_pack_route_attempt_total` | `runbooks/provider-rate-limit-saturation.md` §"Residency" | one dispatch refused |
| FM-16 | Modality | Multi-modal payload exceeds modality budget | Refuse with `RefusalDecision::ModalityBudgetExceeded` | analogous metric | `runbooks/provider-rate-limit-saturation.md` §"Modality" | refusal with retry-guidance |
| FM-17 | Brand UX | brand-ux-surface SDK fails to render SparkleIcon (EU AI Act Art. 13 violation risk) | Fallback to text-only AI disclosure; audit-tap with `BrandUXDegraded` | `oya_intelligence_brand_ux_fallback_total` | `runbooks/refusal-false-positive-cascade.md` §"Brand UX" | reduced visual disclosure; copy-only fallback remains compliant |
| FM-18 | App | Composition-root pod crash | k8s restart; HPA scales; readiness gate ensures no traffic until audit-chain reachable | `kube_pod_container_status_restarts_total` | `runbooks/provider-outage-anthropic.md` §"Pod restart" pattern | brief unavailability ≤ 30 s |
| FM-19 | Network | HTTP/3 + QUIC fallback to TCP/HTTP/2 | Transparent; no impact on availability | `oya_intelligence_http3_fallback_total` | n/a | latency uptick possible |
| FM-20 | Cell | Cell-level Tier-0 outage | Shuffle-shard route around; cell quarantine per ADR-0248 | `oya_intelligence_cell_quarantined_total` | cross-cell runbook (cell µservice) | per-cell tenant cohort impacted |

## Severity assignment

| Severity | Definition | Failure modes |
|---|---|---|
| Sev-1 | Production user-visible regression OR security breach OR regulator-reportable | FM-09, FM-11 |
| Sev-2 | Operational degradation; tenant-visible but not breach | FM-01..FM-04, FM-05, FM-08, FM-10, FM-15, FM-20 |
| Sev-3 | Internal-only; recoverable; observable | FM-06, FM-07, FM-12, FM-13, FM-14, FM-16, FM-17 |
| Sev-4 | Self-healing or transient | FM-18, FM-19 |

## Recovery time objectives (RTO)

| RTO tier | Target | Failure modes |
|---|---|---|
| Hot | ≤ 5 min | FM-01..FM-05, FM-10, FM-18, FM-19, FM-20 |
| Warm | ≤ 30 min | FM-06, FM-08, FM-12, FM-13, FM-14, FM-15, FM-16, FM-17 |
| Cold (forensic + comms) | ≤ 4 h | FM-07, FM-09, FM-11 |

## References

- ADR-0255, ADR-0263, ADR-0296.
- `microservices/intelligence/threat-model.md`.
- `microservices/intelligence/incident-response.md`.
- `microservices/intelligence/runbooks/`.
- Industry references: AWS Well-Architected Reliability Pillar; Google SRE Workbook ch. 1-5.
