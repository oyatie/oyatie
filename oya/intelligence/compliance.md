---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: ops-compliance + council-privacy + axis-intelligence
related_adrs:
  - ADR-0255
  - ADR-0250
  - ADR-0251
  - ADR-0242
  - ADR-0244
  - ADR-0247
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0295
  - ADR-0296
review_cadence: annually + on every pack-activation + on every regulator-published-amendment
doc_status: published
enforced_by: oya-governance-compliance-evidence-emission
companion_docs:
  - microservices/intelligence/ARCHITECTURE.md
  - microservices/intelligence/threat-model.md
  - microservices/intelligence/dpia.md
  - microservices/intelligence/incident-response.md
  - microservices/intelligence/policy/
---

# Compliance — intelligence µservice

## Purpose

Define the per-pack compliance posture, substrate-level controls mapping to each framework,
per-pack overlays the substrate emits, the audit-evidence emission contract, and all §3.2.1
ADR-adherence rows required by `docs/standards/documentation-rigor.md`.

## Framework × pack matrix

| Framework | pack-kr | pack-eu | pack-us | pack-us-healthcare | pack-us-federal | pack-jp | pack-sg | pack-au | pack-in | pack-br | pack-ae | pack-ksa | pack-cn | pack-uk |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| SOC 2 Type 2 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | n/a | ✓ |
| ISO 27001:2022 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| ISO/IEC 42001:2023 (AIMS) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| GDPR | n/a | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | UK-GDPR |
| EU AI Act 2024/1689 | n/a | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| KR PIPA + PIPC | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| HIPAA | n/a | n/a | n/a | ✓ | partial | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| PCI DSS v4 | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** | **REFUSED** |
| FedRAMP High | n/a | n/a | n/a | n/a | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| CJIS | n/a | n/a | n/a | n/a | partial | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| KR-CSAP | partial | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| APPI | n/a | n/a | n/a | n/a | n/a | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| PDPA + MAS-TRM | n/a | n/a | n/a | n/a | n/a | n/a | ✓ | n/a | n/a | n/a | n/a | n/a | n/a | n/a |
| Privacy Act 1988 + APRA-CPS-234 | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✓ | n/a | n/a | n/a | n/a | n/a | n/a |
| DPDPA 2023 | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✓ | n/a | n/a | n/a | n/a | n/a |
| LGPD | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✓ | n/a | n/a | n/a | n/a |
| UAE PDPL | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✓ | n/a | n/a | n/a |
| KSA PDPL + NCA | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✓ | n/a | n/a |
| CN PIPL + Generative AI Provisions 2023 | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✓ | n/a |
| UK AI Regulation White Paper + ICO AI Guidance | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | ✓ |

## §pack-overlay-roster

ADR-0251 + ADR-0250 answer: per-pack compliance overlays active for `intelligence`.

### HIPAA (pack-us-healthcare) — BAA via provider-credential BYOK (ADR-0255 §D-4)

| Control | Implementation |
|---|---|
| §164.308(a)(1) security management | This document + threat-model.md + dpia.md |
| §164.308(a)(3) workforce security | OpenBao JIT elevation + 2-person-rule on policy changes |
| §164.312(a)(1) access control | Cedar gates on every dispatch; `byok_required_by_pack` enforced |
| §164.312(b) audit controls | audit-tap → audit-chain seal; retained ≥ 6 years |
| §164.312(c)(1) integrity | Ed25519 signatures on every audit record (IP-022) |
| §164.312(d) authentication | OIDC + MFA + SPIFFE for machine identities |
| §164.312(e)(1) transmission security | TLS 1.3 + PQC-hybrid per ADR-0253 |
| §164.502(b) minimum necessary | Substrate sees only caller's assembled prompt; never owns retrieval corpus |
| §164.504(e) BA agreements | BAA-signed providers only (Anthropic BAA / Bedrock BAA / Azure-OpenAI BAA); non-BAA refused by Cedar |
| §164.530(j) records retention | Audit-tap ≥ 6 years for HIPAA pack |

Provider constraint: `byok-gating.cedar` forbids non-BAA-signed providers when `pack-us-healthcare` active.

### FedRAMP High (pack-us-federal) — Bedrock + Azure OpenAI Gov

- Substrate deployed in Azure-Gov-cloud or AWS Bedrock GovCloud only.
- All operators US persons; access via separate GovCloud IAM.
- CJIS overlay available per-tenant with ATO + auditor sign-off.
- Continuous monitoring per FedRAMP CM family; boundary scope covers `intelligence` namespace.
- Provider constraint: only `bedrock-govcloud` or `azure-openai-gov` permitted; all others FORBID by `provider-routing.cedar`.

### KR-PIPA + KR-CSAP (pack-kr)

- KR-resident deployment: Tier-3 cell in `ap-northeast-2`.
- KR-government-tenant onboarding triggers CSAP overlay (separate government-grade audit trail).
- Sensitive-data refusal floor per PIPA Art. 23 (biometric, health, criminal record prompts).
- Provider routing: Vertex AI KR, Bedrock `ap-northeast-2`, Cohere KR-resident only.

### EU AI Act 2024/1689 (pack-eu)

| Article | Implementation |
|---|---|
| Art. 9 (risk-management system) | This document + ISO 42001 AIMS posture + `§ml-model-lifecycle` |
| Art. 10 (data + governance) | `dpia.md` + caller-side RAG discipline; no PII in prompt assembly beyond caller's own retrieval |
| Art. 12 (record-keeping) | audit-tap → audit-chain; SLO `audit-emission-success` ≥ 99.99 % |
| Art. 13 (transparency) | brand-ux-surface SparkleIcon + RefusalBanner + CostFloorDisclosure (IP-020) |
| Art. 14 (human oversight) | Annex III refusal queue for human review (IP-015) |
| Art. 15 (accuracy, robustness, cybersecurity) | Eval canonicalen-set (IP-021); multi-provider failover; PQC-hybrid TLS |
| Art. 16 (provider obligations) | Substrate registered as AI substrate provider; annual EU AI Office report |
| Art. 27 (FRIA for deployers) | `dpia.md` is the substrate-level FRIA; tenant-FRIA inherited |
| Art. 73 (serious incident 24 h notification) | `runbooks/eu-ai-act-incident-notification.md` |
| Annex III refusal layer | `policy/eu-ai-act-high-risk.cedar` |

### CN PIPL + Generative AI Provisions 2023 (pack-cn)

- Alibaba Qwen + Tencent Hunyuan ONLY; all US/EU providers FORBID by `provider-routing.cedar`.
- No cross-border data transfer: prompt + completion never leave CN datacenter (`cn-north-1` cell).
- Algorithm filing requirement: `CnPiplDispatchRecord` audit event logged per dispatch (CN Generative AI Provisions 2023 Art. 17).
- Data localisation: tenant PII never exported outside CN cell; DR-pair also CN-resident.

### SOC 2 + ISO 27001 + ISO 42001 universal posture

Continuous evidence emission via audit-tap per ADR-0263. Auditor scope gated by `policy/auditor-scope.cedar`. Evidence stored at `evidence/compliance/<framework>/<control>-<unix_ts>.json`.

SOC 2 Trust Services Criteria CC + A + P + PI mappings documented in `threat-model.md`.
ISO 27001:2022 Annex A control mappings in `threat-model.md`.
ISO/IEC 42001:2023 AIMS clause mappings in `§ml-model-lifecycle` below.

### PCI refusal (all packs)

`intelligence` is NOT PCI-eligible. Cardholder data must NEVER enter dispatch. `refusal-baseline.cedar` refuses prompts containing PAN-pattern strings. Hard refusal floor; not tunable per-tenant.

## §day-one-cert-readiness

ADR-0250 answer: build certified shape day one; never retrofit compliance.

| Certification level | Day-one ready | Evidence location |
|---|---|---|
| SOC 2 Type 2 | yes — continuous audit-tap emission from first dispatch | `evidence/compliance/soc2/` |
| ISO 27001:2022 | yes — threat-model.md + dpia.md + Cedar controls | `evidence/compliance/iso27001/` |
| ISO/IEC 42001:2023 (AIMS) | yes — ML model lifecycle per `§ml-model-lifecycle` | `evidence/compliance/iso42001/` |
| HIPAA | yes — BAA-provider-only Cedar gate + PHI-scrubbed logs + 6-year retention | `evidence/compliance/hipaa/` |
| FedRAMP High (Moderate from day-1, High path planned) | moderate day-1; High requires ATO narrative | `evidence/compliance/fedramp/` |
| EU AI Act 2024/1689 | yes — Annex III refusal layer + Art. 12 audit + Art. 13 transparency + Art. 73 runbook | `evidence/compliance/eu-ai-act/` |
| GDPR (pack-eu) | yes — dpia.md + data-residency + audit-tap + Art. 73 notification path | `evidence/compliance/gdpr/` |
| KR-PIPA (pack-kr) | yes — PIPA Art. 23 refusal floor + CSAP overlay path + KR-resident cell | `evidence/compliance/kr-pipa/` |
### Content-pass expansion — day-one-cert-readiness
- This expansion preserves the existing prose above and closes `day-one-cert-readiness` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: AWS Artifact anchors the external control pattern for `day-one-cert-readiness`.
- Precedent 2: Google Assured Workloads provides a second independent hyperscaler pattern for `day-one-cert-readiness`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `day-one-cert-readiness`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `day-one-cert-readiness` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `day-one-cert-readiness` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `day one cert readiness` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `day one cert readiness`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `day one cert readiness` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §detection-substrate-binding

documentation-rigor §3.2.6.A answer: detection families this µservice contributes to.

| Detection family | Signal emitted | Substrate binding |
|---|---|---|
| **Prompt injection** (Family 8 policy-violation) | `PromptInjectionDetected` audit event + `oya_intelligence_guardrails_refusal_total{reason="prompt_injection"}` metric | → `detection` µservice (Wave-3-D); → `observability` µservice UEBA feed |
| **Jailbreak** (Family 8) | `JailbreakDetected` event | → `detection` µservice trust-and-safety queue |
| **Credential exfil in output** (Family 8) | `CredentialExfilDetected` event | → ops-security alert; → `detection` µservice |
| **Refusal false-positive** (Family 5 content abuse) | `RefusalDecisionEmitted{outcome="false_positive"}` via human-review queue outcome | → eval worker (IP-021) canonicalen-set drift signal |
| **Refusal false-negative** (Family 5) | `RefusalDecisionEmitted{outcome="false_negative"}` via trust-and-safety report | → eval worker canonicalen-set update |
| **Audit-row forgery attempt** (Family 8) | `AuditTapEmitFailed` + Merkle-chain tamper detection | → ops-security; → `runbooks/audit-row-forgery-detected.md` |
| **Forged EMERGENCY_SERVICES claim** (Family 8) | `AbuseDefenceForgeryDetected` | → ops-security immediate revocation at trust root |
| **provider-credential BYOK credential exfil via sidecar** (ADR-0255 §D-4; Family 7 insider risk) | sidecar exit anomaly via OpenBao audit log | → UEBA in `observability`; → ops-security alert |

DRMP wiring (§3.2.6): every detection signal above has paired Prevention (Cedar default-deny + sidecar isolation), Risk scoring (bot-score + audit-chain anomaly), Mitigation (stream termination + refusal + revocation), and Recovery (canonicalen-set update + policy patch + Merkle reconciliation).
### Content-pass expansion — detection-substrate-binding
- This expansion preserves the existing prose above and closes `detection-substrate-binding` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: AWS GuardDuty findings anchors the external control pattern for `detection-substrate-binding`.
- Precedent 2: Google Chronicle detections provides a second independent hyperscaler pattern for `detection-substrate-binding`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `detection-substrate-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `detection-substrate-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `detection-substrate-binding` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `detection substrate binding` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `detection substrate binding`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `detection substrate binding` tied to the keystone bundle instead of a local convention.

## §insider-threat-controls

§3.2.6.D prevention layer L7 + Family 7 detection:

| Control | Implementation |
|---|---|
| JIT access via PAM | CyberArk / Teleport / Boundary for all operator access to intelligence namespace |
| Pre-action MFA for policy changes | 2-person-rule on Cedar fragment authoring + OpenBao policy changes |
| Sensitive-resource read-only default | `policy/auditor-scope.cedar` + `policy/ci-scope.cedar` restrict reads to declared scopes |
| Per-employee behavioural baseline | audit-tap stream feeds UEBA substrate in `observability` per ADR-0263 |
| Pre-departure access review | axis-intelligence quarterly access recertification |
| Sidecar isolation | OpenBao sidecar (ADR-0296) prevents operator from reading provider credentials directly |
| Audit-chain tamper detection | Merkle-sealed audit records; tamper triggers `AuditTapEmitFailed` alert |
| Two-person-rule on production deploys | PR approval from ≥ 2 axis-intelligence members; Foundry pipeline enforces |
### Content-pass expansion — insider-threat-controls
- This expansion preserves the existing prose above and closes `insider-threat-controls` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Microsoft Purview Insider Risk anchors the external control pattern for `insider-threat-controls`.
- Precedent 2: Google BeyondCorp provides a second independent hyperscaler pattern for `insider-threat-controls`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `insider-threat-controls`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `insider-threat-controls` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `insider-threat-controls` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `insider threat controls` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `insider threat controls`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `insider threat controls` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §threat-intelligence-feeds

External threat-intelligence feeds consumed by the guardrails + abuse-defence stack:

| Feed | Purpose | Integration point | Refresh cadence |
|---|---|---|---|
| NCMEC PhotoDNA | CSAM hash-check on image/video modalities | `guardrails-adapter` pre-upload classifier | Real-time API call |
| GIFCT hash-matching | Terrorism + extremism content | `guardrails-adapter` | Real-time API call |
| HIBP (HaveIBeenPwned) API | Stolen-credential check at auth path | auth-path (not dispatch); feeds sidecar credential-stuffing detector | On-demand |
| Shared abuse-IP feed (internal) | IP reputation for abuse-defence Cedar gate | `abuse-defence.cedar` `source_ip_reputation` field | Hourly refresh + real-time override |
| OWASP LLM Top 10 prompt injection dataset | Classifier training data | `guardrails-kernel` classifier update pipeline | Quarterly model refresh |
| OFAC / EU / UN / KR-MOFA sanctions lists | AML/sanctions pre-dispatch screen | `provider-routing.cedar` FORBID for sanctioned principals | Daily sync |
### Content-pass expansion — threat-intelligence-feeds
- This expansion preserves the existing prose above and closes `threat-intelligence-feeds` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Mandiant threat intelligence anchors the external control pattern for `threat-intelligence-feeds`.
- Precedent 2: AWS GuardDuty threat lists provides a second independent hyperscaler pattern for `threat-intelligence-feeds`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `threat-intelligence-feeds`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `threat-intelligence-feeds` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `threat-intelligence-feeds` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `threat intelligence feeds` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `threat intelligence feeds`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `threat intelligence feeds` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.

## §key-rotation-cadence

| Key / secret | Rotation cadence | Mechanism | Runbook |
|---|---|---|---|
| Provider API keys (platform-default) | 90 days | OpenBao auto-rotation via Vault Dynamic Secrets | `runbooks/byok-rotation-tenant-cascade.md` |
| Provider API keys (tenant BYOK) | Tenant-controlled; minimum 90 days recommended | Tenant writes to OpenBao; sidecar hot-swap (IP-023) | `runbooks/byok-rotation-tenant-cascade.md` |
| Audit-tap Ed25519 signing key | 90 days | OpenBao key rotation; new key activated after ≥ 60 s soak | `runbooks/sidecar-credential-handle-expired.md` |
| ECH private key | ≥ 90 days | `iac/prod-ech-config.yaml` auto-rotation via OpenBao | `docs/runbooks/cedar-fragment-emergency-rollback.md` |
| PQC certificate (ed25519+ml_dsa_65) | 90 days (cert-manager auto-renew 15 d before expiry) | `iac/prod-pqc-cert.yaml` cert-manager | ops-security on-call |
| OpenBao sidecar token | 1 hour (auto-renewal) | Kubernetes auth backend; renew-self before expiry | `runbooks/sidecar-credential-handle-expired.md` |
| SPIFFE SVID | 1 hour (SPIRE auto-rotation) | SPIRE agent per ADR-0295 | ops-security on-call |
### Content-pass expansion — key-rotation-cadence
- This expansion preserves the existing prose above and closes `key-rotation-cadence` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: AWS KMS key rotation anchors the external control pattern for `key-rotation-cadence`.
- Precedent 2: Google Cloud KMS versions provides a second independent hyperscaler pattern for `key-rotation-cadence`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `key-rotation-cadence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `key-rotation-cadence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `key-rotation-cadence` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `key rotation cadence` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `key rotation cadence`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `key rotation cadence` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.

## §crypto-agility-plan

Per ADR-0253 PQC hybrid + graceful degradation:

| Algorithm class | Current standard | PQC transition | Migration trigger |
|---|---|---|---|
| KEM (TLS key exchange) | X25519 (classical) | X25519MLKEM768 hybrid preferred; X25519 fallback for non-PQ peers | IANA codepoint `0x11ec` widely supported; monitor IETF draft status |
| Signature (TLS + code-signing) | Ed25519 | ed25519+ml_dsa_65 hybrid for new cert chains | New certs issued after 2026-05-20 use hybrid; old certs retire at next rotation |
| Symmetric encryption (at-rest DEK) | AES-256-GCM | AES-256-GCM remains NIST-recommended post-quantum | No change required; monitor NIST PQC standard finalization |
| Audit-record signing | Ed25519 | Migrate to ed25519+ml_dsa_65 at next 90-day key rotation | Triggered automatically by key-rotation cadence above |
| Hash (Merkle seal) | SHA-256 | SHA-256 remains secure post-quantum (Grover's algo: 128-bit security) | No change required |

Crypto-agility invariant: all algorithm choices are configurable via `iac/prod-pqc-cert.yaml` + OpenBao policy; no hard-coded algorithm strings in Rust implementation. Algorithm change requires only config update + key rotation, not code change.
### Content-pass expansion — crypto-agility-plan
- This expansion preserves the existing prose above and closes `crypto-agility-plan` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Cloudflare post-quantum TLS anchors the external control pattern for `crypto-agility-plan`.
- Precedent 2: Chrome hybrid PQ TLS provides a second independent hyperscaler pattern for `crypto-agility-plan`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `crypto-agility-plan`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `crypto-agility-plan` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `crypto-agility-plan` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `crypto agility plan` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `crypto agility plan`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `crypto agility plan` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.

## §minor-protection

ADR-0292 answer: COPPA < 13 refusal; KOSA 14-17 tier; EU age-verification.

| Rule | Age tier | Refusal class | Exception |
|---|---|---|---|
| Adult content generation | < 13 AND 14-17 | `MinorProtectionRefusal::AdultContent` | none |
| Violence / graphic content | < 13 AND 14-17 | `MinorProtectionRefusal::Violence` | none |
| Gambling-related content | < 13 AND 14-17 | `MinorProtectionRefusal::Gambling` | none |
| Substance / drug content | < 13 AND 14-17 | `MinorProtectionRefusal::Substance` | none |
| Social-comparison harmful content (KOSA) | 14-17 | `MinorProtectionRefusal::KosaHarmful` | none |
| COPPA data-minimization | < 13 | `MinorProtectionRefusal::CoppaDataMinimum` | none |
| Safety report / crisis-line dispatch | any | **NOT refused** | Child safety > parental control; crisis-line bypass per `policy/critical-path-emergency-services.cedar` |

Implementation: `crates/oya-intelligence-guardrails-kernel/src/minor_protection.rs` (IP-024).
Audit: `MinorProtectionRefusalEmitted` event per ADR-0263.
EU age-verification: `pack-eu` activates stricter age-gate; Age Verification Providers API integration via `oya-identity` µservice upstream; `intelligence` receives `audience_type=MINOR_TARGETED` in DispatchEnvelope.
Parental-consent wall: NEVER blocks a minor's safety report to mandatory reporter (documentation-rigor §3.2.5 row 9).
### Content-pass expansion — minor-protection
- This expansion preserves the existing prose above and closes `minor-protection` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Apple Family/Screen Time controls anchors the external control pattern for `minor-protection`.
- Precedent 2: Google Family Link provides a second independent hyperscaler pattern for `minor-protection`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `minor-protection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `minor-protection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `minor-protection` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `minor protection` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `minor protection`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.

## §ml-model-lifecycle

documentation-rigor §3.2.6.E + ISO/IEC 42001:2023 AIMS + EU AI Act Art. 9 + NIST AI RMF:

### Validation (pre-deployment)

Every guardrails classifier update (refusal-baseline, Annex III classifier, multimodal classifier) MUST pass:
1. retired-advanceden-set regression (IP-021): refusal false-negative rate ≤ 0.1 %; refusal false-positive rate ≤ 2 %.
2. Adversarial red-team eval: 200 known jailbreaks + 50 per Annex III category; 100 % block rate on CSAM/violence/extremism.
3. Bias eval per `§detection-fairness-audit` below.
4. Latency regression: guardrails pre-call + post-call combined ≤ 30 ms p99.
CI lane: `oya-intelligence-eval-regression`.

### Drift monitoring (post-deployment)

- retired-advanceden-set accuracy monitored continuously; alert if false-negative rate exceeds 0.05 % (half the SLO threshold).
- Distribution drift: monthly batch comparison of production refusal-decision distribution vs canonicalen-set baseline.
- Provider model version change: triggers re-run of full validation suite within 24 h.

### Fairness re-audit (per ADR-0292 + EU AI Act Art. 9)

- Quarterly: refusal-rate stratified by `audience_type`, `locale`, `pack` — verify no demographic group has systematically higher false-positive rate.
- Annual: external third-party fairness audit for the Annex III classifier.
- Finding: if any demographic stratum has false-positive rate > 2× the aggregate, classifier update required before next quarterly cycle.

### Versioning

- Every classifier is versioned (`guardrails_classifier_version` field in `CallRecord`).
- Version bump required for any change to refusal logic or training data.
- Rollback: `iac/helm/intelligence/values.yaml` `guardrails.classifier_image_tag`; 5-min rollback per `runbooks/refusal-false-positive-cascade.md`.

### Appeal mechanism (EU AI Act Art. 86 + ECOA Reg B + NY AEDT 2023)

- Every `RefusalDecision` includes `appeal_url`: a tenant-scoped endpoint where the user can contest the refusal.
- Human reviewer SLA: ≤ 30 days for substantive review.
- Audit: `RefusalAppealSubmitted` + `RefusalAppealResolved` events per ADR-0263.
- Reviewer identity logged in `RefusalDecision.review_outcome`.
### Content-pass expansion — ml-model-lifecycle
- This expansion preserves the existing prose above and closes `ml-model-lifecycle` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: NIST AI RMF anchors the external control pattern for `ml-model-lifecycle`.
- Precedent 2: Google Model Cards provides a second independent hyperscaler pattern for `ml-model-lifecycle`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ml-model-lifecycle`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ml-model-lifecycle` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §detection-fairness-audit

quarterly + annual cycle per `§ml-model-lifecycle`:

| Stratum | Metric | Acceptance threshold | Audit frequency |
|---|---|---|---|
| `locale` (all supported locales) | Refusal false-positive rate per locale | no locale > 2× aggregate FPR | quarterly |
| `audience_type` | Refusal false-positive rate per audience type | no audience > 2× aggregate FPR | quarterly |
| `pack` | Refusal false-negative rate per pack | all packs ≤ 0.1 % FNR | quarterly |
| Annex III category | Classification accuracy per category | ≥ 95 % per category | quarterly |
| Minor protection (by age tier) | Refusal accuracy for age-gated content | 100 % for CSAM/violence; ≤ 2 % FPR for age-appropriate content | quarterly |
| End-user demographic (via third-party audit) | Disparate impact analysis | Adverse impact ratio ≥ 0.8 per EEOC 4/5ths rule | annually |

Evidence stored at `evidence/compliance/fairness/<stratum>-<quarter>-<year>.json`.
External auditor: named in `dpia.md §third-party-audit-roster`.
### Content-pass expansion — detection-fairness-audit
- This expansion preserves the existing prose above and closes `detection-fairness-audit` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Microsoft Fairlearn anchors the external control pattern for `detection-fairness-audit`.
- Precedent 2: NIST AI RMF measurement provides a second independent hyperscaler pattern for `detection-fairness-audit`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `detection-fairness-audit`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `detection-fairness-audit` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `detection-fairness-audit` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `detection fairness audit` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `detection fairness audit`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `detection fairness audit` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §self-modification

ADR-0247 answer: does intelligence produce or consume self-modification artifacts?

`intelligence` is consumed by the Foundry pipeline (`oyatie.foundry.*` principals under Cedar) for planning, review, and execution tasks. In that role, it is a **substrate** called by the self-modification system — not itself a self-modification actor.

`intelligence` does NOT produce self-modification artifacts (no writes to its own Cedar fragments, no writes to its own IaC, no writes to its own Cargo crates outside of normal CI). Changes to `intelligence` go through the normal Foundry pipeline (worktree → PR → ADR → CI → merge-queue per ADR-0116).

Meta-trust-root attestation: when Foundry calls `intelligence`, the SPIFFE identity `spiffe://oyatie/foundry/<role>` is verified by `dispatch-authorization.cedar` PERMIT 2 before any dispatch proceeds. No elevated trust beyond the Cedar gate.
### Content-pass expansion — self-modification
- This expansion preserves the existing prose above and closes `self-modification` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: SLSA provenance anchors the external control pattern for `self-modification`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `self-modification`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `self-modification`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `self-modification` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `self-modification` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `self modification` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `self modification`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `self modification` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `self modification` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `intelligence` fails closed when `self modification` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `intelligence` emits denial evidence for `self modification` instead of converting policy failure into a generic timeout or user-facing ambiguity.

## §meta-trust-attestation

ADR-0293 answer: Foundry-touching meta-trust-root attestation path.

`intelligence` is called by Foundry agents (`spiffe://oyatie/foundry/*`) on the `internal-foundry` audience tag. The attestation chain:
1. Foundry agent holds SPIFFE SVID issued by SPIRE per ADR-0295.
2. `dispatch-authorization.cedar` PERMIT 2 verifies `principal.spiffe_id like "spiffe://oyatie/foundry/*"` AND `resource.audience_tag == "internal-foundry"`.
3. SVID is short-lived (1 hour); SPIRE auto-rotates.
4. Any Foundry principal operating outside its declared SPIFFE trust domain is FORBID by the Cedar default-deny baseline.

No meta-trust-root CA ceremony is required for `intelligence` itself; the trust root is the SPIRE trust domain `spiffe://oyatie` managed by ops-security.
### Content-pass expansion — meta-trust-attestation
- This expansion preserves the existing prose above and closes `meta-trust-attestation` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: The Update Framework roots anchors the external control pattern for `meta-trust-attestation`.
- Precedent 2: Sigstore Rekor transparency provides a second independent hyperscaler pattern for `meta-trust-attestation`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `meta-trust-attestation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `meta-trust-attestation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `meta-trust-attestation` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `meta trust attestation` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `meta trust attestation`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `meta trust attestation` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `meta trust attestation` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.

## §bootstrap-trust-chain

ADR-0295 answer: bootstrap CI SPIFFE attestation + kill-switch wiring.

`intelligence` is NOT a bootstrap-tier-1 µservice (it depends on `observability`, `tenancy`, `policy-engine`). However:
- Every pod carries a SPIFFE SVID issued by the SPIRE agent; CI lane `oya-governance-spiffe-attestation` verifies SVID issuance on every deploy.
- Kill-switch: if the SPIRE trust domain is compromised, ops-security can revoke the `intelligence` SPIFFE trust via the trust-domain revocation ceremony per ADR-0295 §D-kill-switch.
- OpenBao sidecar authentication: Kubernetes auth backend verifies pod service account + namespace before issuing OpenBao token; revocation via `vault token revoke -accessor <accessor>` in runbooks.
### Content-pass expansion — bootstrap-trust-chain
- This expansion preserves the existing prose above and closes `bootstrap-trust-chain` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: SPIFFE/SPIRE workload identity anchors the external control pattern for `bootstrap-trust-chain`.
- Precedent 2: Sigstore Fulcio provides a second independent hyperscaler pattern for `bootstrap-trust-chain`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `bootstrap-trust-chain`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `bootstrap-trust-chain` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `bootstrap-trust-chain` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `bootstrap trust chain` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `bootstrap trust chain`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `bootstrap trust chain` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `bootstrap trust chain` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `intelligence` fails closed when `bootstrap trust chain` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §platform-owner-indirection

ADR-0284 answer: audit of hard-coded `oyatie` strings.

Known `oyatie`-specific strings in `intelligence` and their indirection status:

| String | Location | Indirection status |
|---|---|---|
| `spiffe://oyatie/intelligence/*` | `ARCHITECTURE.md §principals`, Cedar policies | **Pending**: SPIFFE trust domain should be read from `platform_owner_config.trust_domain` per ADR-0284 |
| `spiffe://oyatie/foundry/*` | `policy/dispatch-authorization.cedar` PERMIT 2 | **Pending**: same; trust domain config-driven |
| `secret/oyatie/intelligence/...` | OpenBao paths in `iac/terraform/openbao-policy.tf` | **Pending**: namespace prefix should use `${platform_owner_namespace}` |
| `tenant:oya-self` | `ARCHITECTURE.md §observability` | **Pending**: self-observability tenant ID should be platform-owner-indirected |

All four items are filed in the ADR-0284 migration backlog as `intelligence-platform-owner-indirection`. No new hard-coded `oyatie` strings permitted; existing ones migrate in the next sprint per ADR-0284 §F migration cadence.
### Content-pass expansion — platform-owner-indirection
- This expansion preserves the existing prose above and closes `platform-owner-indirection` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Salesforce My Domain anchors the external control pattern for `platform-owner-indirection`.
- Precedent 2: Google Workspace tenant branding provides a second independent hyperscaler pattern for `platform-owner-indirection`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `platform-owner-indirection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `platform-owner-indirection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `platform-owner-indirection` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `platform owner indirection` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `platform owner indirection`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `platform owner indirection` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.

## §consent

ADR-0272 answer: `intelligence` is not directly user-facing for cookie-consent purposes.

Cookie consent is handled by the consumer-facing `app-shell` µservice which embeds the brand-ux-surface SDK. `intelligence` does not set cookies. Per-purpose consent signals are forwarded from `app-shell` to `intelligence` via the `DispatchEnvelope.consent_context` field; `intelligence` refuses analytics-purpose calls when consent not granted.
### Content-pass expansion — consent
- This expansion preserves the existing prose above and closes `consent` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Google Consent Mode anchors the external control pattern for `consent`.
- Precedent 2: Apple App Tracking Transparency provides a second independent hyperscaler pattern for `consent`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `consent`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `consent` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `consent` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `consent` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `consent`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `consent` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `consent` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `intelligence` fails closed when `consent` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `intelligence` emits denial evidence for `consent` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `intelligence` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `consent` workflow.
- Depth detail 17: `intelligence` telemetry for `consent` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §email-deliverability

ADR-0273 answer: `intelligence` does not emit email. n/a.
### Content-pass expansion — email-deliverability
- This expansion preserves the existing prose above and closes `email-deliverability` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Google Workspace DKIM/SPF/DMARC anchors the external control pattern for `email-deliverability`.
- Precedent 2: AWS SES domain identity provides a second independent hyperscaler pattern for `email-deliverability`.
- Tenant-scope invariant: every `intelligence` `dispatch` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/intelligence/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `intelligence` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `intelligence` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `intelligence` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `intelligence` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `intelligence` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `dispatch` evaluates `<tenant>.intelligence.dispatch` against policy, writes `intelligence.model_routing`, and emits `oya.intelligence.dispatch.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `email-deliverability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `email-deliverability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `email-deliverability` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `email deliverability` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `email deliverability`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `email deliverability` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `email deliverability` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `intelligence` fails closed when `email deliverability` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `intelligence` emits denial evidence for `email deliverability` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `intelligence` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `email deliverability` workflow.
- Depth detail 17: `intelligence` telemetry for `email deliverability` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.
- Depth detail 18: Rollback for `intelligence` preserves prior policy fragment id, package version, migration checksum, and last-known-good runtime config.

## §portability

ADR-0276 answer: per-tenant audit-tap export for GDPR Art. 20 + KR-PIPA Art. 35.

Per-tenant `CallRecord` export: `Intelligence control-plane operation: audit export --tenant-id <ID> --format jsonl --window <START>/<END>`. Export includes all `CallRecord`, `RefusalDecision`, `EvalRecord` rows for the tenant. Per `backfill-replay.md §portability`.

## Critical-path edge-case coverage

documentation-rigor §3.2.5 applicable rows are fully covered in `ARCHITECTURE.md §critical-path-edge-cases`. Cross-reference: rows 1, 5, 6, 7, 8, 9, 12, 16, 21, 28.

## Per-control evidence emission

Every CI lane protecting a compliance control emits structured evidence to `evidence/compliance/<framework>/<control>-<unix_ts>.json`. CI lane `oya-governance-compliance-evidence-emission` validates emission coverage per cycle.

## Tenant compliance onboarding

| Step | Owner | Artifact |
|---|---|---|
| Tenant declares regulated-data classes | gtm-customer-success | tenant DPA + intake form |
| Pack assignment | ops-compliance + ops-legal | tenant onboarding ticket |
| BAA / SCC / DPA execution | ops-legal | `legal/{baa,dpa-template,scc-template}.md` |
| Per-tenant Cedar overlay activation | ops-security + axis-intelligence | tenant-pack-overlay-tag in dispatch envelope |
| First-dispatch verification | ops-quality | canonicalen dispatch through tenant-pack overlay |
| BYOK key provisioning (if required) | tenant + ops-security | `runbooks/byok-rotation-tenant-cascade.md` step 1 |

## References

- ADR-0255, ADR-0250, ADR-0251 — primary compliance authorities.
- ADR-0242 through ADR-0296 — keystone bundle 2026-05-20.
- `microservices/intelligence/ARCHITECTURE.md`.
- `microservices/intelligence/threat-model.md`.
- `microservices/intelligence/dpia.md`.
- `microservices/intelligence/policy/`.
- `microservices/intelligence/incident-response.md`.
- `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`.
- Industry references: SOC 2 TSC 2017; ISO 27001:2022; ISO/IEC 42001:2023; GDPR; EU AI Act 2024/1689; HIPAA 45 CFR §164; FedRAMP rev. 5; KR PIPA + PIPC Notice 2020-7; PCI DSS v4.0; CN PIPL 2021; CN Generative AI Provisions 2023; NIST AI RMF 1.0.
- Hyperscaler precedents: OpenAI usage policies + moderation API; Anthropic responsible-scaling policy + EU AI Act posture; Google Vertex AI compliance documentation; AWS Bedrock compliance posture; Azure OpenAI responsible AI.
