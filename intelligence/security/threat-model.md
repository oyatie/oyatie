---
doc_class: ThreatModel
microservice: intelligence
version: 1.0.0
status: Proposed
date: 2026-05-20
owner: axis-intelligence + council-security
related_oyatie_adrs:
  - ADR-0003
  - ADR-0009
  - ADR-0145
  - ADR-0243
  - ADR-0244
  - ADR-0263
  - ADR-0297
  - ADR-0313
  - ADR-0319
---

# Intelligence Security Threat Model

This document covers the intelligence substrate security posture for dispatch,
model routing, provider adapters, credential handles, guardrails, refusal
decisions, RAG retrieval, evals, attribution, audit taps, and output filtering.
The service is the controlled boundary between tenants and model providers; it
must assume every prompt, retrieval document, provider response, tool call, and
model output can be hostile until filtered, attributed, scoped, and sealed.

## Asset Inventory

### Named Data Classes

| Asset ID | Named data class | Description | Primary store | Security objective |
|---|---|---|---|---|
| INT-A01 | DispatchPromptContent | User/system/developer prompt text, multimodal input, tool context. | Transient plus audit tap | Prevent prompt injection and sensitive data leakage. |
| INT-A02 | DispatchOutputContent | Model output, tool plan, generated text, structured response. | Transient plus audit tap | Prevent unsafe output and data exfiltration. |
| INT-A03 | CredentialHandle | Short-lived provider credential handle bound to tenant, audience, provider, and TTL. | OpenBao sidecar memory | Prevent provider credential exfiltration. |
| INT-A04 | ProviderAdapterSecretReference | BYOK or platform-default secret reference for OpenAI/Anthropic/Vertex/Bedrock. | OpenBao | Prevent provider account takeover. |
| INT-A05 | ModelRoutingDecision | Provider/model selection, fallback, cost, region, data-residency result. | Routing store and audit-chain | Prevent tenant model isolation breach. |
| INT-A06 | GuardrailPolicyDecision | Refusal, allow, redact, transform, or human-review decision. | Guardrail log and audit-chain | Prevent output filter bypass. |
| INT-A07 | RagRetrievalContext | Retrieved document ids, chunks, embeddings references, citation graph, rank scores. | RAG index/caller-owned store plus audit tap | Prevent retrieval poisoning and cross-tenant leakage. |
| INT-A08 | TrainingEvalCorpus | retired-advanceden sets, eval prompts, synthetic cases, tuning or evaluation data. | Git/S3 eval store | Prevent training data poisoning. |
| INT-A09 | AttributionRecord | Source citations, model/provider metadata, tool trace, confidence. | audit-chain | Preserve explainability and non-repudiation. |
| INT-A10 | TenantModelIsolationState | Tenant model routing, BYOK binding, provider account boundary, region policy. | Tenant config and routing store | Prevent per-tenant model isolation breach. |
| INT-A11 | ToolInvocationPlan | Tool calls, function args, connector target, side-effect classification. | Transient plus audit tap | Prevent prompt-driven tool abuse. |
| INT-A12 | AbuseSignalProfile | Injection score, jailbreak score, rate limit, bot score, prompt fingerprint. | Observability and audit-chain | Detect injection and abuse. |
| INT-A13 | AuditEmissionEnvelope | ADR-0263 envelope with tenant_id, trace_id, span_id, audit_id, schema_version, source_microservice. | audit-chain | Preserve detection and non-repudiation. |

### Named External Interfaces

| Interface ID | Interface | Entry point | Principal | Notes |
|---|---|---|---|---|
| INT-I01 | Dispatch API | `../contracts/openapi/intelligence.yaml` | User, service, or SDK | Main prompt and response path. |
| INT-I02 | Streaming SSE API | `../IP-016-streaming-sse-transport.md` | Browser or SDK | Streams tokens and refusal transitions. |
| INT-I03 | Streaming WebSocket API | `../IP-017-streaming-websocket-transport.md` | Browser or SDK | Bi-directional streaming. |
| INT-I04 | Provider Adapter API | `../contracts/provider-adapter-trait.md` | Intelligence service | Calls OpenAI, Anthropic, Vertex, Bedrock. |
| INT-I05 | Credential Resolver | `../IP-023-byok-credential-rotation.md` | Router/adapter | Resolves provider credential handle. |
| INT-I06 | Guardrail Stack | `../IP-008-kernel-guardrail-stack.md` | Dispatch flow | Applies policy, refusal, redaction, filter. |
| INT-I07 | RAG Retrieval | `../capabilities/context-aware-retrieval.yaml` | Dispatch flow | Retrieves tenant-bound context. |
| INT-I08 | Eval Pipeline | `../IP-021-eval-canonicalen-set.md` | Eval worker | Measures false positives/negatives and regressions. |
| INT-I09 | Audit Tap | `../IP-022-audit-tap-merkle-seal.md` | Dispatch flow and worker | Seals request, routing, refusal, output metadata. |
| INT-I10 | Brand UX Surface | `../IP-020-brand-ux-surface-components.md` | Consumer UI | Renders refusal and attribution states. |

### Named Dependencies

| Dependency ID | Dependency | Use | Failure impact | Guardrail |
|---|---|---|---|---|
| INT-D01 | Model providers | Inference | Provider outage, data leakage, model behavior drift | `../runbooks/provider-outage-openai.md` and peers. |
| INT-D02 | OpenBao | Credential handles and provider secrets | Credential exfiltration | `../runbooks/byok-rotation-tenant-cascade.md`. |
| INT-D03 | Cedar policy-engine | Dispatch, BYOK, provider route, high-risk policy | Authorization bypass | `../policy/dispatch-authorization.cedar`. |
| INT-D04 | RAG storage/index | Retrieval context | Retrieval poisoning or cross-tenant leakage | `../runbooks/rag-retrieval-quality-regression.md`. |
| INT-D05 | Guardrail policy stack | Injection and output safety | Output filter bypass or false refusal | `../policy/refusal-baseline.cedar`. |
| INT-D06 | Eval corpus store | retired-advanceden set and regression evaluation | Training/eval poisoning | `../IP-021-eval-canonicalen-set.md`. |
| INT-D07 | audit-chain | Sealed AI evidence | Repudiation and regulatory gap | ADR-0003 and ADR-0263. |
| INT-D08 | observability | Injection, refusal, latency, provider SLOs | Missed abuse or outage | `../dashboards/prompt-injection-detection.md`. |
| INT-D09 | identity | Tenant/user auth, context, step-up | Cross-tenant access | Identity and tenant scope policies. |
| INT-D10 | finops/cost | Per-call metering | Cost abuse | `../dashboards/finops-cost-attribution.md`. |

## Trust Boundaries

| Boundary ID | Named boundary | Crosses from | Crosses to | Primary concern |
|---|---|---|---|---|
| INT-B01 | Public dispatch boundary | Browser, SDK, tenant backend | Intelligence ingress | Prompt injection, abuse, data exfiltration. |
| INT-B02 | Tenant boundary | Tenant A prompt/model/RAG | Tenant B prompt/model/RAG | Per-tenant model isolation breach. |
| INT-B03 | System prompt boundary | Tenant/user prompt | System/developer policy prompt | Prompt injection and instruction hierarchy abuse. |
| INT-B04 | RAG retrieval boundary | User request | Retrieval index/corpus | RAG retrieval poisoning and cross-tenant data. |
| INT-B05 | Provider adapter boundary | Intelligence adapter | External model provider | Provider credential and prompt/output leakage. |
| INT-B06 | Credential boundary | Adapter/router | OpenBao credential handle | Provider secret exfiltration. |
| INT-B07 | Guardrail boundary | Model output | Filter/refusal/redaction engine | Output filter bypass. |
| INT-B08 | Tool boundary | Model/tool plan | Tool executor or connector | Prompt-driven side effects. |
| INT-B09 | Eval/training boundary | retired-advanceden set or tuning data | Eval pipeline/model improvement process | Training data poisoning. |
| INT-B10 | Streaming boundary | Provider token stream | Browser/SSE/WebSocket stream | Partial unsafe output before filter. |
| INT-B11 | Audit boundary | Dispatch/routing/refusal/output event | audit-chain tap | Missing audit_id or sensitive telemetry. |
| INT-B12 | Cost boundary | Model call | FinOps and quota | Cost explosion or quota bypass. |
| INT-B13 | Information-barrier boundary | Office-scope prompt | Retrieval/output policy | Restricted deal or taint leakage. |

## STRIDE Analysis

### Spoofing

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| INT-S01 | DispatchPromptContent | INT-B01 | Caller spoofs tenant, principal, or audience tag. | Cross-tenant dispatch or policy bypass. |
| INT-S02 | CredentialHandle | INT-B06 | Adapter spoofs tenant/provider to obtain credential. | Provider account compromise. |
| INT-S03 | ModelRoutingDecision | INT-B05 | Response is spoofed as coming from approved provider/model. | Trust and audit failure. |
| INT-S04 | RagRetrievalContext | INT-B04 | Poisoned source impersonates trusted document. | Bad answer with trusted citation. |
| INT-S05 | ToolInvocationPlan | INT-B08 | Prompt injects fake tool authorization. | Unauthorized side effect. |
| INT-S06 | AuditEmissionEnvelope | INT-B11 | Event emitted with wrong tenant or source_microservice. | Forensic gap. |

### Tampering

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| INT-T01 | DispatchPromptContent | INT-B03 | Prompt injection overrides system/developer instructions. | Policy bypass and data leakage. |
| INT-T02 | RagRetrievalContext | INT-B04 | RAG retrieval poisoning changes chunks, ranks, or citations. | False or malicious answer. |
| INT-T03 | TrainingEvalCorpus | INT-B09 | Training/eval data poisoning alters future model or evaluation behavior. | Safety regression. |
| INT-T04 | GuardrailPolicyDecision | INT-B07 | Refusal or redaction decision is modified after generation. | Output filter bypass. |
| INT-T05 | ModelRoutingDecision | INT-B05 | Router changes provider/model to non-approved region or tenant. | Residency and isolation breach. |
| INT-T06 | AttributionRecord | INT-B11 | Citation or provider metadata is changed before audit tap. | Non-repudiation and explainability failure. |
| INT-T07 | CredentialHandle | INT-B06 | Credential TTL, provider, or audience binding is extended. | Credential misuse. |
| INT-T08 | DispatchOutputContent | INT-B10 | Streaming output bypasses final filter before completion. | Unsafe content leak. |

### Repudiation

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| INT-R01 | DispatchPromptContent | INT-B11 | Caller denies submitting prompt or tool context. | Abuse investigation gap. |
| INT-R02 | GuardrailPolicyDecision | INT-B07 | Service cannot prove why output was refused or allowed. | Regulatory and support gap. |
| INT-R03 | ModelRoutingDecision | INT-B05 | Provider route cannot be proven after incident. | DPA/residency dispute. |
| INT-R04 | RagRetrievalContext | INT-B04 | Retriever cannot prove which chunks influenced output. | Citation dispute. |
| INT-R05 | TrainingEvalCorpus | INT-B09 | Eval corpus change lacks reviewer evidence. | Poisoning investigation gap. |
| INT-R06 | ToolInvocationPlan | INT-B08 | Tool side effect lacks sealed model/caller approval chain. | Unauthorized action dispute. |

### Information Disclosure

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| INT-I01 | DispatchPromptContent | INT-B05 | Prompt sends secrets, PHI, or tenant data to wrong provider. | Data exfiltration. |
| INT-I02 | DispatchOutputContent | INT-B07 | Output leaks hidden system prompt, retrieved private data, or another tenant's data. | Model exfiltration or tenant breach. |
| INT-I03 | TenantModelIsolationState | INT-B02 | Tenant A model route, cache, or BYOK context reused for tenant B. | Per-tenant model isolation breach. |
| INT-I04 | CredentialHandle | INT-B06 | Provider key handle or BYOK secret leaks in logs. | Provider account compromise. |
| INT-I05 | RagRetrievalContext | INT-B04 | Retrieval returns chunks from unauthorized tenant or office scope. | RAG data leak. |
| INT-I06 | AuditEmissionEnvelope | INT-B11 | ADR-0263 telemetry stores raw prompt/output PII. | Observability privacy breach. |

### Denial of Service

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| INT-DOS01 | DispatchPromptContent | INT-B01 | Prompt flood or long-context flood exhausts model quota. | Tenant or platform outage. |
| INT-DOS02 | ProviderAdapterSecretReference | INT-B05 | Provider outage, rate limit, or credential rejection. | Inference outage. |
| INT-DOS03 | RagRetrievalContext | INT-B04 | Retrieval query explosion overloads vector/search store. | Slow or failed RAG. |
| INT-DOS04 | GuardrailPolicyDecision | INT-B07 | Guardrail loop or false-positive cascade blocks legitimate output. | Functional outage. |
| INT-DOS05 | Streaming boundary | INT-B10 | Token stream held open by client or provider. | Connection exhaustion. |
| INT-DOS06 | CredentialHandle | INT-B06 | OpenBao sidecar failure prevents provider calls. | Inference outage. |

### Elevation of Privilege

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| INT-E01 | ToolInvocationPlan | INT-B08 | Prompt injection causes tool call requiring higher privilege. | Unauthorized side effect. |
| INT-E02 | GuardrailPolicyDecision | INT-B07 | User bypasses output filter or refusal baseline. | Unsafe output or prohibited advice. |
| INT-E03 | ModelRoutingDecision | INT-B05 | Tenant forces route to premium/provider-restricted model. | Cost and policy bypass. |
| INT-E04 | CredentialHandle | INT-B06 | BYOK handle used by another tenant or provider. | Credential and isolation breach. |
| INT-E05 | TrainingEvalCorpus | INT-B09 | Contributor gains ability to alter safety evals without review. | Safety regression. |
| INT-E06 | TenantModelIsolationState | INT-B13 | Restricted-deal taint is removed before retrieval or output. | Information-barrier bypass. |

## DREAD Scoring

| Rank | Threat ID | Threat | Damage | Reproducibility | Exploitability | Affected users | Discoverability | Total |
|---|---|---|---:|---:|---:|---:|---:|---:|
| 1 | INT-I03 | Per-tenant model isolation breach. | 10 | 8 | 7 | 10 | 7 | 42 |
| 2 | INT-T01 | Prompt injection overrides system/developer policy. | 9 | 10 | 8 | 8 | 7 | 42 |
| 3 | INT-I02 | Model output exfiltrates hidden prompt or tenant data. | 10 | 8 | 7 | 9 | 7 | 41 |
| 4 | INT-I04 | Provider credential or BYOK handle leaks. | 10 | 7 | 7 | 9 | 7 | 40 |
| 5 | INT-T02 | RAG retrieval poisoning. | 9 | 8 | 8 | 8 | 7 | 40 |
| 6 | INT-E02 | Output filter bypass. | 9 | 9 | 8 | 7 | 7 | 40 |
| 7 | INT-T03 | Training/eval data poisoning. | 9 | 7 | 7 | 8 | 7 | 38 |
| 8 | INT-E01 | Prompt-driven privileged tool call. | 9 | 8 | 7 | 7 | 7 | 38 |
| 9 | INT-DOS01 | Long-context or prompt flood exhausts quota. | 8 | 9 | 8 | 8 | 4 | 37 |
| 10 | INT-T08 | Streaming output bypasses final filter. | 8 | 8 | 7 | 7 | 6 | 36 |
| 11 | INT-R03 | Provider route cannot be proven. | 8 | 7 | 6 | 8 | 6 | 35 |
| 12 | INT-DOS02 | Provider outage or rate limit. | 8 | 8 | 5 | 9 | 5 | 35 |
| 13 | INT-I05 | RAG returns unauthorized chunk. | 9 | 6 | 6 | 8 | 6 | 35 |
| 14 | INT-T05 | Router selects non-approved region/provider. | 8 | 6 | 6 | 8 | 6 | 34 |
| 15 | INT-I06 | Raw prompt/output PII in telemetry. | 8 | 6 | 5 | 8 | 5 | 32 |

## Attack Trees

### Opportunistic Adversary: Prompt Injection

- Goal: bypass policy and extract sensitive data.
  - Path O1: submit prompt that claims to override system/developer instructions.
  - Path O2: include tool instructions disguised as user content.
  - Path O3: ask model to reveal hidden prompt, retrieval context, or credential metadata.
  - Path O4: exploit streaming to leak partial content before final filter.
  - Path O5: retry variants until guardrail false negative occurs.
- Required break: instruction hierarchy and guardrail classifier fail.
- Required break: streaming path emits unfiltered unsafe content.
- Detection pivot: prompt injection dashboard, `PolicyRefused`, `AbuseDefenceChallengeIssued`.

### Targeted Adversary: RAG Retrieval Poisoning

- Goal: make model cite attacker-controlled content as trusted.
  - Path T1: inject poisoned document into tenant-accessible corpus.
  - Path T2: craft terms to rank above canonical source.
  - Path T3: include instructions in retrieved chunk.
  - Path T4: force answer with malicious citation.
  - Path T5: suppress attribution warning or source confidence.
- Required break: retrieval provenance and source allowlist fail.
- Required break: retrieved instructions are treated as policy, not data.
- Detection pivot: `RetrievalContextBound`, RAG quality regression alert, attribution mismatch.

### Insider Adversary: Eval/Training Poisoning

- Goal: weaken refusal and injection detection over time.
  - Path I1: gain access to canonicalen set or eval corpus.
  - Path I2: remove hard cases or add mislabeled safe examples.
  - Path I3: train/tune/evaluate against poisoned corpus.
  - Path I4: publish policy or threshold change.
  - Path I5: exploit new false-negative window.
- Required break: corpus change lacks review and sealed audit.
- Required break: eval regression thresholds fail open.
- Detection pivot: eval diff, `PolicyRefused` rate shift, `OfficeBoundaryClearanceRequested`.

### Nation-State Adversary: Tenant Model Isolation Breach

- Goal: obtain another tenant's data through routing, cache, provider account, or BYOK confusion.
  - Path N1: compromise tenant or provider routing preference.
  - Path N2: force route to shared provider account or wrong region.
  - Path N3: reuse credential handle across tenant/provider boundary.
  - Path N4: exploit cache, RAG context, or attribution record bleed.
  - Path N5: exfiltrate through generated output or tool side effect.
- Required break: tenant model isolation state is not bound to route and credential.
- Required break: ADR-0263 audit event lacks tenant_id or provider route metadata.
- Detection pivot: `AssistDraftCompleted`, credential handle anomaly, `ConglomerateInformationBarrierCrossingRefused`.

## Mitigations Currently In Place

| Threat ID | Named mitigation | ADR or policy | Named code path or doc |
|---|---|---|---|
| INT-T01 | Prompt injection detection, instruction hierarchy, and refusal baseline. | ADR-0243 | `../policy/refusal-baseline.cedar`; `../runbooks/prompt-injection-detected.md`. |
| INT-I02 | Output filter, redaction, and refusal decision audit. | ADR-0263 | `../IP-008-kernel-guardrail-stack.md`; `../runbooks/prompt-fence-bypass-attempt-response.md`. |
| INT-T02 | RAG source provenance, citation binding, and quality regression runbook. | ADR-0244 | `../capabilities/context-aware-retrieval.yaml`; `../runbooks/rag-retrieval-quality-regression.md`. |
| INT-T03 | retired-advanceden set review and eval record audit. | ADR-0003 | `../IP-021-eval-canonicalen-set.md`; `../IP-005-domain-layer-eval-record.md`. |
| INT-I03 | Tenant-bound routing and BYOK/provider isolation. | ADR-0244 | `../policy/provider-routing.cedar`; `../policy/byok-gating.cedar`. |
| INT-I04 | Short-lived credential handles from OpenBao sidecar. | ADR-0243 | `../IP-023-byok-credential-rotation.md`; `../runbooks/sidecar-credential-handle-expired.md`. |
| INT-E01 | Tool side effects require authorization outside model text. | ADR-0243 | `../policy/dispatch-authorization.cedar`. |
| INT-E02 | Refusal baseline Cedar policy and false-negative SLO. | ADR-0243 | `../slos/refusal-false-negative-rate.openslo.yaml`. |
| INT-T08 | Streaming transport gates partial output through policy state. | ADR-0263 | `../IP-016-streaming-sse-transport.md`; `../IP-017-streaming-websocket-transport.md`. |
| INT-DOS02 | Provider outage and rate-limit runbooks. | ADR-0145 | `../runbooks/provider-outage-openai.md`; `../runbooks/provider-outage-anthropic.md`; `../runbooks/provider-rate-limit-saturation.md`. |
| INT-I06 | PII scrubbing at emission boundary and audit tap minimization. | ADR-0263 | `../IP-009-kernel-audit-tap.md`; `../IP-022-audit-tap-merkle-seal.md`. |
| INT-E06 | Office/information-barrier taint respected by retrieval and output. | ADR-0319 | `../policy/eu-ai-act-high-risk.cedar`; `../policy/tenant-isolation.md`. |

## Residual Risks Accepted

| Risk ID | Residual risk | Risk owner | Compensating control | Review trigger |
|---|---|---|---|---|
| INT-RR01 | Prompt injection is probabilistic and cannot be fully eliminated. | axis-intelligence | Defense in depth: policy, retrieval provenance, refusal, audit, human review. | Injection incident. |
| INT-RR02 | Provider models can change behavior without Oyatie code change. | axis-intelligence | Eval canonicalen set, provider version pin, regression dashboard. | Provider release. |
| INT-RR03 | RAG corpus can contain user-authored poisoned content. | tenant owner + axis-intelligence | Source ranking, provenance display, and quarantine. | RAG quality alert. |
| INT-RR04 | BYOK credentials depend on tenant key hygiene. | tenant security owner | Short-lived handles and rotation cascade. | BYOK rotation failure. |
| INT-RR05 | Output filters can false-positive legitimate regulated workflows. | council-product | Appeal, human review, and per-pack policy review. | Refusal false-positive spike. |
| INT-RR06 | Output filters can false-negative novel jailbreaks. | council-security | Red-team corpus and abuse telemetry. | Refusal false-negative. |
| INT-RR07 | Streaming UX can expose partial text before full context is known. | axis-intelligence | Token buffer policy and high-risk route full-buffer mode. | Streaming policy change. |
| INT-RR08 | Eval/training datasets can become stale against new attack patterns. | axis-intelligence | Quarterly refresh and incident-derived cases. | New attack class. |
| INT-RR09 | Cost controls can deny legitimate high-context work. | axis-finops | Tenant quota override with audit. | FinOps appeal. |
| INT-RR10 | External providers may log prompts under their own contractual controls. | council-privacy | DPA, region routing, and BYOK/provider policy. | Provider DPA change. |

## Specific Telemetry for Detection

ADR-0263 detection telemetry must include `tenant_id`, `sub_scope_path`,
`event_id`, `trace_id`, `span_id`, `audit_id`, `schema_version`,
`source_microservice`, `cell_id`, and `jurisdiction_code` for state-changing
intelligence events. Cedar denial events include policy id, principal, action,
resource, decision, and denied reason.

| Threat ID | Detection telemetry | ADR-0263 class or service event | Signal |
|---|---|---|---|
| INT-T01 | Injection score, jailbreak phrase, system-prompt extraction attempt. | `PolicyRefused`, `AbuseDefenceSpoofDetected` | Prompt injection. |
| INT-I02 | Output contains hidden prompt, retrieved private data, or prohibited class. | `PolicyRefused`, refusal false-negative SLO | Model exfiltration or filter bypass. |
| INT-T02 | Retrieved chunk source mismatch, rank anomaly, poisoned citation. | `RetrievalContextBound`, RAG regression alert | RAG retrieval poisoning. |
| INT-T03 | Eval corpus diff, label change, false-negative rate shift. | eval record event, `OfficeBoundaryClearanceRequested` | Training/eval poisoning. |
| INT-I03 | Route uses wrong tenant, provider account, BYOK handle, or region. | `AssistDraftCompleted`, credential anomaly | Tenant model isolation breach. |
| INT-I04 | Credential handle logged, reused, or used after TTL. | sidecar credential alert, `AbuseDefenceAttestationFailed` | Credential exfiltration. |
| INT-E01 | Tool call requested from untrusted prompt without authorization. | `OfficeBoundaryAttemptDenied`, dispatch deny | Tool privilege escalation. |
| INT-E02 | Refusal bypass attempt or sudden false-negative increase. | `PolicyRefused`, prompt injection dashboard | Output filter bypass. |
| INT-DOS01 | Prompt length, token volume, quota burn, rate-limit hit. | `AbuseDefenceQuotaExceeded`, provider rate alert | Long-context DoS. |
| INT-DOS02 | Provider timeout, 429, failover route. | `AbuseDefenceVendorOutage`, provider outage runbook | Provider outage. |
| INT-T08 | Unsafe partial token before final filter. | streaming guardrail alert | Streaming bypass. |
| INT-I06 | Raw prompt/output PII in logs. | PII scrubber failure, audit tap validator | Telemetry privacy breach. |

## Threat Coverage Ledger

### INT-COV01: Prompt injection coverage

- Threats covered: INT-T01, INT-E01, INT-R01.
- Asset coverage: DispatchPromptContent, ToolInvocationPlan, AuditEmissionEnvelope.
- Boundary coverage: INT-B01, INT-B03, INT-B08, INT-B11.
- Required control evidence: instruction hierarchy, injection score, tool authorization, sealed prompt metadata.
- Detection evidence: `PolicyRefused`, prompt injection dashboard, and `AbuseDefenceSpoofDetected`.

### INT-COV02: RAG poisoning coverage

- Threats covered: INT-T02, INT-S04, INT-I05, INT-R04.
- Asset coverage: RagRetrievalContext and AttributionRecord.
- Boundary coverage: INT-B04 and INT-B13.
- Required control evidence: source provenance, chunk tenant binding, citation hash, rank anomaly detection.
- Detection evidence: `RetrievalContextBound`, attribution mismatch, and RAG regression runbook.

### INT-COV03: Training/eval poisoning coverage

- Threats covered: INT-T03, INT-E05, INT-R05.
- Asset coverage: TrainingEvalCorpus.
- Boundary coverage: INT-B09.
- Required control evidence: corpus review, diff approval, sealed eval record, regression threshold.
- Detection evidence: eval record, false-negative rate SLO, and office clearance event.

### INT-COV04: Model exfiltration coverage

- Threats covered: INT-I02, INT-E02, INT-T08.
- Asset coverage: DispatchOutputContent and GuardrailPolicyDecision.
- Boundary coverage: INT-B07 and INT-B10.
- Required control evidence: output filter, refusal baseline, high-risk full-buffer stream mode.
- Detection evidence: `PolicyRefused`, streaming guardrail alert, and refusal false-negative SLO.

### INT-COV05: Tenant model isolation coverage

- Threats covered: INT-I03, INT-T05, INT-E03, INT-E04.
- Asset coverage: TenantModelIsolationState, ModelRoutingDecision, CredentialHandle.
- Boundary coverage: INT-B02, INT-B05, INT-B06.
- Required control evidence: tenant-bound route, BYOK handle binding, region policy, provider account partition.
- Detection evidence: credential anomaly, route audit, and `ConglomerateInformationBarrierCrossingRefused`.

### INT-COV06: Provider credential coverage

- Threats covered: INT-S02, INT-I04, INT-T07, INT-DOS06.
- Asset coverage: CredentialHandle and ProviderAdapterSecretReference.
- Boundary coverage: INT-B06.
- Required control evidence: short-lived handle, provider/audience binding, no raw secret logging.
- Detection evidence: sidecar credential alert and OpenBao audit.

### INT-COV07: Provider boundary coverage

- Threats covered: INT-S03, INT-R03, INT-DOS02.
- Asset coverage: ModelRoutingDecision and ProviderAdapterSecretReference.
- Boundary coverage: INT-B05.
- Required control evidence: provider version metadata, route seal, failover policy, DPA/region guard.
- Detection evidence: provider outage runbook and `AssistDraftCompleted` route metadata.

### INT-COV08: Output filter coverage

- Threats covered: INT-T04, INT-E02, INT-R02.
- Asset coverage: GuardrailPolicyDecision and DispatchOutputContent.
- Boundary coverage: INT-B07.
- Required control evidence: refusal reason, policy id, redaction log, false-positive/negative SLO.
- Detection evidence: `PolicyRefused`, refusal-rate dashboard, and guardrail decision audit.

### INT-COV09: Cost and quota coverage

- Threats covered: INT-DOS01, INT-E03, INT-DOS05.
- Asset coverage: ModelRoutingDecision and AbuseSignalProfile.
- Boundary coverage: INT-B12 and INT-B10.
- Required control evidence: per-tenant quota, route cost cap, streaming connection limit.
- Detection evidence: `AbuseDefenceQuotaExceeded`, FinOps dashboard, and provider rate alert.

### INT-COV10: Telemetry privacy coverage

- Threats covered: INT-I06, INT-S06.
- Asset coverage: AuditEmissionEnvelope.
- Boundary coverage: INT-B11.
- Required control evidence: PII scrubbing, prompt/output minimization, audit_id on state changes.
- Detection evidence: audit tap validator, PII scrubber failure, and audit-chain completeness SLO.

## Incident Response Playbook References

| Incident class | Runbook |
|---|---|
| Prompt injection detected | `../runbooks/prompt-injection-detected.md` |
| Prompt fence bypass attempt | `../runbooks/prompt-fence-bypass-attempt-response.md` |
| RAG retrieval quality regression | `../runbooks/rag-retrieval-quality-regression.md` |
| BYOK rotation cascade | `../runbooks/byok-rotation-tenant-cascade.md` |
| Credential handle expired | `../runbooks/sidecar-credential-handle-expired.md` |
| Provider outage: OpenAI | `../runbooks/provider-outage-openai.md` |
| Provider outage: Anthropic | `../runbooks/provider-outage-anthropic.md` |
| Provider outage: Google | `../runbooks/provider-outage-google.md` |
| Provider rate-limit saturation | `../runbooks/provider-rate-limit-saturation.md` |
| Model inference timeout | `../runbooks/model-inference-timeout-investigation.md` |
| Refusal false-positive cascade | `../runbooks/refusal-false-positive-cascade.md` |
| EU AI Act incident notification | `../runbooks/eu-ai-act-incident-notification.md` |
| Audit row forgery detected | `../runbooks/audit-row-forgery-detected.md` |

## Cross-References

- Root service architecture: `../ARCHITECTURE.md`.
- Product requirements: `../PRD.md`.
- Operational boundaries: `../operational-boundaries.md`.
- Intelligence OpenAPI contract: `../contracts/openapi/intelligence.yaml`.
- Intelligence AsyncAPI contract: `../contracts/asyncapi/intelligence-events.yaml`.
- Provider adapter trait: `../contracts/provider-adapter-trait.md`.
- Dispatch capability: `../capabilities/dispatch.yaml`.
- Guardrails capability: `../capabilities/guardrails.yaml`.
- Routing capability: `../capabilities/routing.yaml`.
- Context-aware retrieval capability: `../capabilities/context-aware-retrieval.yaml`.
- Audit tap capability: `../capabilities/audit-tap.yaml`.
- Model router implementation packet: `../IP-007-kernel-model-router.md`.
- Guardrail stack implementation packet: `../IP-008-kernel-guardrail-stack.md`.
- Audit tap implementation packet: `../IP-009-kernel-audit-tap.md`.
- Dispatch flow implementation packet: `../IP-010-usecase-dispatch-flow.md`.
- Eval canonicalen set packet: `../IP-021-eval-canonicalen-set.md`.
- BYOK rotation packet: `../IP-023-byok-credential-rotation.md`.
- Dispatch authorization policy: `../policy/dispatch-authorization.cedar`.
- Provider routing policy: `../policy/provider-routing.cedar`.
- BYOK gating policy: `../policy/byok-gating.cedar`.
- Refusal baseline policy: `../policy/refusal-baseline.cedar`.
- Tenant isolation policy: `../policy/tenant-isolation.md`.
- Prompt injection dashboard: `../dashboards/prompt-injection-detection.md`.
- Provider latency dashboard: `../dashboards/provider-latency-heatmap.json`.
- Refusal rate dashboard: `../dashboards/refusal-rate-by-pack.json`.
- ADR-0263 observability emission contract: `../../../docs/decisions/ADR-0706-observability-live-apex.md`.
- ADR-0243 Cedar as universal gate: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0244 tenant as universal scoping primitive: `../../../docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- ADR-0297 abuse defence baseline: `../../../docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- ADR-0319 information barrier: `../../../docs/decisions/ADR-0709-general-live-apex.md`.

## Checkpoint Notes

- This document does not modify intelligence decisions or runbooks.
- It treats RAG content, prompts, outputs, and eval data as untrusted by default.
- It assumes raw provider credentials never leave OpenBao and short-lived handles.
- It accepts that AI-safety controls are layered probabilistic controls with deterministic Cedar boundaries around authority and data access.
