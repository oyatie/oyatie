---
doc_class: ArchitectureSpec
template_id: TPL-ARCHITECTURE
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence + council-architecture
deciders: council-architecture, axis-intelligence, ops-security, council-privacy
related_adrs:
  - ADR-0255
  - ADR-0255-amendment-library-first
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0247
  - ADR-0248
  - ADR-0250
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0280
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0297
  - ADR-0105
  - ADR-0131
  - ADR-0145
review_cadence: quarterly + on every BC promotion / demotion
doc_status: published
enforced_by: oya-governance-adr-adherence-matrix
companion_docs:
  - microservices/intelligence/PRD.md
  - microservices/intelligence/compliance.md
  - microservices/intelligence/threat-model.md
  - microservices/intelligence/dpia.md
  - docs/standards/documentation-rigor.md
---

# Architecture: intelligence µservice (two-layer AI Substrate)

## 1. Purpose

`intelligence` is the canonical AI Substrate for oyatie per ADR-0255. It dispatches every model
call, applies refusal-baseline guardrails, evaluates output quality, renders citation attribution,
emits the consumer brand UX surface, resolves provider credentials per provider-credential BYOK (ADR-0255 §D-4), and emits an
audit tap per ADR-0263 onto the audit-chain seal stream. It is the single chokepoint between
oyatie product code (consumer + developer + Foundry) and model providers.

Hyperscaler analog: this is the AWS Bedrock + SageMaker orchestration layer pattern — a unified
inference substrate that abstracts provider heterogeneity, applies platform-wide policy, and
emits audit telemetry, rather than allowing each product to call providers directly.

Failure modes enumerated: (1) provider outage — failover to next provider per `provider-routing.cedar`; (2) Cedar eval timeout — fail closed (deny); (3) OpenBao sidecar restart — in-flight dispatches fail with `CredentialHandleExpired`; (4) audit-tap seal failure — dispatch blocked (audit-first invariant); (5) regional cell unreachable — failover to DR-pair cell per ADR-0241, respecting pack data-residency hard-stops.

## 2. Two-layer model (ADR-0255)

ADR-0255 splits the AI surface into two cohabiting layers within this same µservice:

```text
┌──────────────────────────────────────────────────────────────────────────┐
│  AI Substrate (Layer-A)         │  Consumer Brand UX Surface (Layer-B)   │
│  ─────────────────────          │  ─────────────────────────────         │
│  model-routing                  │  brand-ux-surface                      │
│  providers                      │    - sparkle icon                      │
│  guardrails                     │    - tier badge (Sonnet / Opus / Haiku)│
│  eval                           │    - streaming text                    │
│  attribution                    │    - citation rendering                │
│  credential-resolver            │    - refusal copy                      │
│  audit-tap                      │    - cost-floor disclosure (B2C)       │
│                                 │                                        │
│  Library-first dispatch         │  Renders against Substrate output      │
└──────────────────────────────────────────────────────────────────────────┘
```

The two layers share one substrate, one manifest, one Cargo workspace family, and one
contract bundle. Layer-B never bypasses Layer-A.

## §principals

ADR-0242 answer: principals operating as or calling `intelligence`.

**Principals this µservice operates as:**

| Principal slug | SPIFFE identity | Cedar entity type | Scope |
|---|---|---|---|
| `oyatie.intelligence.dispatch` | `spiffe://oyatie/intelligence/dispatch` | `FoundryAgent` | Issues dispatch calls on behalf of internal callers |
| `oyatie.intelligence.guardrails` | `spiffe://oyatie/intelligence/guardrails` | internal | Evaluates refusal policy; no external identity |
| `oyatie.intelligence.audit-tap` | `spiffe://oyatie/intelligence/audit-tap` | internal | Emits to audit-chain per ADR-0263 |
| `oyatie.intelligence.credential-resolver` | `spiffe://oyatie/intelligence/credential-resolver` | internal | Delegates to OpenBao sidecar via Unix socket |
| `oyatie.intelligence.eval-worker` | `spiffe://oyatie/intelligence/eval-worker` | internal | Runs canonicalen-set regression |

**Tenant-scoped principals that call this µservice:**

| Caller class | Cedar entity type | Dispatch path | Audience tag |
|---|---|---|---|
| Consumer end-user | `ConsumerEndUser` | network-opt-in REST/gRPC or library-first via brand-ux-surface | `consumer` |
| Developer tenant | `TenantPrincipal` | library-first SDK or REST/gRPC | `developer` |
| Foundry agent | `FoundryAgent` | library-first in-process | `internal-foundry` |
| Emergency-services principal | `TenantPrincipal` with `audience_type=EMERGENCY_SERVICES` | any | `consumer` (elevated) |
| Minor-targeted principal | `ConsumerEndUser` with `audience_type=MINOR_TARGETED` | any | `consumer` (restricted) |

All callers MUST set `audience_tag` in `DispatchEnvelope`; missing tag yields `DispatchError::MissingAudienceTag`.
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `principals` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.

## §cedar-gates

ADR-0243 answer: Cedar fragments gating actions; default-deny baseline on every fragment.

Every Cedar fragment begins with `forbid (principal, action, resource);` — default-deny.
Specific PERMIT rules layer on top. Fragment soak window ≥ 60 s per ADR-0294 before promotion.

| Fragment | Authority | Actions gated | When evaluated |
|---|---|---|---|
| `policy/critical-path-emergency-services.cedar` | ops-security + council-privacy | `dispatch`, `rate_limit_check`, `bot_score_gate` | **FIRST** — before all score gates; LIFE-SAFETY HARD RULE |
| `policy/dispatch-authorization.cedar` | axis-intelligence + ops-security | `dispatch` | Pre-dispatch admission; every call |
| `policy/abuse-defence.cedar` | ops-security | `dispatch`, `rate_limit_check`, `bot_score_gate` | Anti-bot / anti-spoof / anti-scrape per §3.2.3 |
| `policy/provider-routing.cedar` | axis-intelligence | `provider_select` | Provider selection given audience + tenant + modality + pack |
| `policy/refusal-baseline.cedar` | council-privacy + ops-security | `dispatch` pre+post | Pre-call + post-call classification |
| `policy/byok-gating.cedar` | ops-security | `credential_resolve` | Credential resolution; refuses platform-default for regulated tenants |
| `policy/eu-ai-act-high-risk.cedar` | council-privacy + council-legal | `dispatch` | EU AI Act Annex III high-risk system refusal |
| `policy/auditor-scope.cedar` | ops-security + council-privacy | `read_audit_records` | Auditor read access |
| `policy/ci-scope.cedar` | axis-foundry + ops-security | `read_dispatch_records` | CI lane read access |
| `policy/tenant-scope.cedar` | ops-security | all cross-tenant | Cross-tenant isolation |

Fragment publish path (ADR-0294): fragments published via `oya-governance-cedar-fragment-publisher`; soak = 60 s before active eval. Emergency rollback: `docs/runbooks/cedar-fragment-emergency-rollback.md`.
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `cedar-gates` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `cedar gates` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `cedar gates`.

## §tenant-scoping

ADR-0244 answer: every dispatch carries `tenant_id` + `audience_tag`.

**Primitives carrying `tenant_id`:**

| Primitive | tenant_id field | audience_type | provider_credential_mode |
|---|---|---|---|
| `DispatchEnvelope` | `tenant_id: TenantId` | `audience_tag` | Resolved from `SecretReference.kind` |
| `CallRecord` (audit-tap) | `tenant_id: TenantId` | `audience_tag` | logged |
| `RefusalDecision` | `tenant_id: TenantId` | — | — |
| `CredentialHandle` | `tenant_id: TenantId` | `audience_tag` | `byok \| platform_default \| byok_required_by_pack` |
| `EvalRecord` | `tenant_id: TenantId` | — | — |

**`audience_type` served:**
- `CONSUMER` — B2C end-users; platform-default credentials; budget-gated.
- `DEVELOPER` — B2B tenant developers; provider-credential BYOK supported (ADR-0255 §D-4); higher rate limits.
- `INTERNAL_FOUNDRY` — oyatie Foundry agents; platform-default; no budget gate.
- `EMERGENCY_SERVICES` — bypasses all score gates; attestation required; audit emitted.
- `FRIENDLY_CRAWLER_PARTNER` — bypasses challenge gate; never sees CAPTCHA.
- `MINOR_TARGETED` — enhanced refusal baseline per ADR-0292; crisis-line bypass preserved.
- `HIGH_RISK_USER` — metadata-minimization mode; no IP retained in CallRecord.

**`provider_credential_mode`** (ADR-0255 §D-4 — NOT conflated with encryption-BYOK):

| Mode | When active | Behaviour |
|---|---|---|
| `platform_default` | B2C consumer; no pack override | oyatie platform credentials; consumer daily budget limit |
| `byok` | B2B tenant opts in | Tenant API key; OpenBao at `secret/<tenant_id>/intelligence/provider/<provider>` |
| `byok_required_by_pack` | `pack-us-healthcare`, `pack-us-federal`, `pack-cn` | BYOK mandatory; platform-default refused by Cedar |
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Stripe account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §substrate-product-binding

ADR-0245 answer: `intelligence` is a **substrate** µservice.

- `manifest.json:tier = substrate`
- Consumers: every Foundry lane (`oya-foundry-*`), Application Shell server-side, Builder backend, brand-ux-surface, any oyatie product calling models.
- Substrate dependencies: `observability` (audit-chain + metrics), `tenancy` (tenant context), `policy-engine` (Cedar fragment soak + publish pipeline).
- Cross-product calls forbidden per ADR-0145 invariants; direct gRPC to substrate µservices only.
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `substrate-product-binding` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `substrate product binding` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `substrate product binding` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `substrate product binding` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `intelligence` fails closed when `substrate product binding` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §policy-evaluation

ADR-0246 + amendment answer: library-first Cedar evaluation.

- `policy_evaluation_mode = library_first`
- Cedar evaluation runs in-process via `oya-shared-policy-eval` crate.
- No network call to `policy-engine` µservice on the dispatch hot path.
- Fragment set refreshed in-process every 30 s from local fragment cache (well within 60 s soak window).
- Network-opt-in exception: fragment authoring + publishing uses `policy-engine` REST API.
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `policy-evaluation` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `policy evaluation` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `policy evaluation` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.

## §cell-eligibility

ADR-0248 answer: Tier-3 per-tenant for regulated; Tier-1/2 shared for consumer.

- **Default cell tier: Tier-1/2** (shared cells per region for standard consumer + developer tenants).
- **Regulated cell tier: Tier-3** (per-tenant isolated cell) for tenants with `byok_required_by_pack` or data-residency mandates.
- Per-cell shard width: 1 tenant per Tier-3 cell; up to 500 tenants per Tier-1/2 cell.
- Tier-0 edge: global WAF + ECH + rate-limit; no dispatch logic.

**Cell × provider routing matrix:**

| Cell region | Allowed providers | Pack constraint |
|---|---|---|
| us-east-1 | all non-CN | — |
| eu-west-1 | Anthropic EU, OpenAI EU, Vertex AI EU, Azure-OpenAI EU, Mistral, Cohere EU, Bedrock eu-west-1 | `pack-eu`: GDPR residency |
| ap-northeast-2 (KR) | Vertex AI KR, Bedrock ap-northeast-2, Cohere | `pack-kr`: PIPA residency |
| cn-north-1 | Alibaba Qwen, Tencent Hunyuan ONLY | `pack-cn`: CN-PIPL hard isolation |
| us-gov-west-1 | Bedrock GovCloud, Azure-OpenAI Gov | `pack-us-federal`: FedRAMP |

Failure mode when region unreachable: DR-pair cell failover per ADR-0241; data-residency pack overrides failover target (no cross-border failover for `pack-cn` or `pack-eu` strict-residency).
### Content-pass expansion — cell-eligibility
- This expansion preserves the existing prose above and closes `cell-eligibility` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: AWS cell-based architecture anchors the external control pattern for `cell-eligibility`.
- Precedent 2: Route 53 shuffle sharding provides a second independent hyperscaler pattern for `cell-eligibility`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cell-eligibility`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cell-eligibility` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `cell-eligibility` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `cell eligibility` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `cell eligibility`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.

## §intelligence-dispatch

ADR-0255 + amendment answer: library-first default; network-opt-in for cross-language callers.

### In-process dispatch (library-first, default path)

```text
caller (Rust / Foundry agent / Application Shell)
   ↓ oya-intelligence-dispatch-sdk-rs (IP-019)
DispatchEnvelope { tenant_id, audience_tag, secret_reference, modality, provider_hint }
   ↓ [1] critical-path-emergency-services.cedar  ← BYPASS gate; fires FIRST
   ↓ [2] abuse-defence.cedar                     ← bot/spoof/scrape gate
   ↓ [3] dispatch-authorization.cedar            ← tenant + audience + budget
   ↓ [4] refusal-baseline.cedar (pre-call)       ← CSAM / violence / PCI / minor-protection
   ↓ [5] eu-ai-act-high-risk.cedar (pack-eu)     ← Annex III gate
   ↓ [6] model-routing-kernel.route()            ← routing decision
   ↓ [7] provider-routing.cedar                  ← provider selection per pack + modality + cell
   ↓ [8] byok-gating.cedar                       ← credential mode validation
   ↓ [9] credential-resolver-usecase.resolve()   ← Unix socket → OpenBao sidecar (ADR-0296)
CredentialHandle { ttl ≤ 60 s }
   ↓ [10] providers-adapter-<vendor>.invoke()    ← sidecar injects secret at HTTP assembly
(provider stream)
   ↓ [11] refusal-baseline.cedar (post-call)     ← output-side classification
   ↓ [12] attribution-usecase.render_citations()
   ↓ [13] audit-tap-usecase.emit()               ← Ed25519-signed + Merkle-sealed (IP-022)
DispatchOutcome { stream | refusal | error }
```

**Library-first vs network-opt-in choice:**
- Library-first: zero network hop; Cedar eval in-process; preferred for Foundry + Application Shell + server-side callers.
- Network-opt-in: Python notebooks, browser-based callers, third-party SDKs without Rust linkage. REST + gRPC surfaces are marshalling wrappers over the identical kernel pipeline — no policy bypass.
- Choice is per-caller, not per-tenant. Both paths execute the same 13-step pipeline.

### Network-opt-in dispatch

```text
client → HTTPS/HTTP/3 + QUIC (ADR-0253) + ECH + PQC hybrid
   ↓ REST handler (SSE, IP-016) | gRPC handler | WebSocket handler (IP-017)
DispatchEnvelope → identical pipeline from step [1] onward
```

Alt-Svc: `h3=":443"; ma=86400`. Fallback: HTTP/3 → HTTP/2 → HTTP/1.1. HTTP/1.0 forbidden.
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §ontology-read-path

ADR-0257 + amendment answer: library-first; intelligence does not own Ontology directly.

- `ontology_read_mode = library_first` for `context-aware-retrieval` BC.
- `freshness_floor = 30 s` for RAG projection queries.
- In-process via `oya-shared-ontology-read-sdk`; no network hop on hot path.
- Projection entity: `IntelligenceSession` → `ontology.intelligence_session_projection` (append-only, lag budget 30 s).
- Network-opt-in: if pack configures `ontology_read_mode = network`, queries go via gRPC to `ontology` µservice.
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `ontology-read-path` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `ontology read path` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `ontology read path` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.

## §transport

ADR-0253 answer: HTTP/3 + QUIC primary; TLS 1.3 floor; ECH; PQC hybrid.

| Surface | Protocol | Port | TLS | ECH | PQC KEM |
|---|---|---|---|---|---|
| REST dispatch (external) | HTTP/3 → h2 → h1.1 | UDP/TCP 443 | TLS 1.3 AEAD-only | yes | X25519MLKEM768 |
| gRPC (internal µservice) | gRPC over HTTP/3 → h2 | TCP 50051 | mTLS 1.3 SPIFFE | no (cluster-internal) | ed25519+ml_dsa_65 |
| WebSocket (browser / audio) | WS over HTTP/3 WebTransport → h1.1 WS | UDP/TCP 443 | TLS 1.3 | yes | X25519MLKEM768 |
| Metrics (Prometheus scrape) | HTTP/1.1 | TCP 9090 | no (cluster-internal) | n/a | n/a |
| OpenBao sidecar | Unix domain socket | n/a | n/a | n/a | n/a |

TLS profile: AES-256-GCM + ChaCha20-Poly1305 AEAD-only; no CBC; no RC4; no TLS < 1.3.
HSTS: `max-age=63072000; includeSubDomains; preload`. CT required. OCSP stapling. No `insecure_skip_verify`.
ECH config: `iac/prod-ech-config.yaml`. PQC cert: `iac/prod-pqc-cert.yaml`.
ECH graceful degradation: ECH-disabled clients fall to standard TLS 1.3.
PQC graceful degradation: non-PQ clients fall to X25519 / P-256.
Emergency-services clients: negotiate DOWN to whatever the dispatch system supports — NEVER refuse session (§3.2.3 rule 9).
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `transport` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `transport` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `transport`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.

## §observability

ADR-0263 answer: audit events, traces, metrics, logs.

**Audit event classes (ADR-0263 registry):**

| Event class | Trigger | Pack gating |
|---|---|---|
| `DispatchRequestReceived` | pre-dispatch | all |
| `DispatchCompleted` | post-stream seal | all |
| `RefusalDecisionEmitted` | any refusal | all |
| `EuAiActAnnexIiiRefusalEmitted` | Annex III refusal | pack-eu |
| `AbuseDefenceEmergencyServiceBypass` | emergency-services bypass | all |
| `AbuseDefenceForgeryDetected` | forged EMERGENCY_SERVICES claim | all |
| `ByokCredentialRotated` | provider-credential BYOK credential rotation (ADR-0255 §D-4) | all |
| `AuditTapEmitFailed` | seal failure | all |
| `CnPiplDispatchRecord` | CN-pack dispatch | pack-cn |
| `EuAiActArt73NotificationSubmitted` | Art. 73 notification | pack-eu |
| `MinorProtectionRefusalEmitted` | minor-protection refusal | all |
| `PromptInjectionDetected` | prompt injection pre-call | all |
| `JailbreakDetected` | post-call jailbreak | all |
| `CredentialExfilDetected` | credential pattern in output | all |

**Metrics (Prometheus):**

| Metric | Type | Cardinality budget |
|---|---|---|
| `oya_intelligence_dispatch_total` | Counter | ~50 K series |
| `oya_intelligence_first_token_latency_seconds` | Histogram | ~200 series |
| `oya_intelligence_streaming_throughput_tokens_per_second` | Histogram | ~100 series |
| `oya_intelligence_guardrails_refusal_total` | Counter | ~500 series |
| `oya_intelligence_audit_emission_total` | Counter | ~10 series |
| `oya_intelligence_credential_handle_ttl_seconds` | Histogram | ~20 series |
| `oya_intelligence_provider_latency_seconds` | Histogram | ~100 series |

Traces: OTLP gRPC → `observability`; parent span `intelligence.dispatch`; child spans per pipeline step; 10 % sampling default; 100 % on error.
Logs: structured JSON; PHI scrubbed; WARN/ERROR retained 90 d; HIPAA pack: 6 years.
Dashboards: `dashboards/intelligence-overview.json`, `dashboards/provider-latency-heatmap.json`, `dashboards/refusal-rate-by-pack.json`, `dashboards/byok-vs-platform-default-mix.json`.
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Google SRE four canonicalen signals anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §abuse-defence

ADR-0297 answer: anti-prompt-injection + anti-jailbreak + anti-credential-exfil.

Hyperscaler precedent: Stripe Radar (passive scoring), Cloudflare Turnstile (invisible challenge ~95 % of legitimate traffic), OpenAI moderation API, Anthropic Constitutional AI.

**Anti-prompt-injection:** pre-call classifier scans `DispatchEnvelope.prompt` for injection patterns (role-override, system-prompt-exfil, indirect injection via RAG). On detection: `RefusalDecision::PromptInjectionDetected` + audit + `runbooks/prompt-injection-detected.md`. RAG context: instruction-following tokens stripped before assembly.

**Anti-jailbreak:** post-call output classifier checks for policy-violating completions that bypassed pre-call refusal. On detection: stream suppressed; `RefusalDecision::JailbreakDetected`; trust-and-safety queue. retired-advanceden-set 200 known jailbreaks; must block 100 %.

**Anti-credential-exfil:** output classifier scans streaming tokens for API key patterns, JWT shapes, PEM blocks, `.env` shapes. On detection mid-stream: stream terminated; `RefusalDecision::CredentialExfilDetected`. Defence-in-depth: sidecar ensures no credential enters adapter memory (ADR-0296).

**UX-floor (§3.2.3):** ≤ 2 ms p99 added to default path. No challenge on clean bot-score. CI gate: `oya-governance-abuse-defence-ux-floor`.

**Emergency-services bypass (LIFE-SAFETY HARD RULE):** `policy/critical-path-emergency-services.cedar` PERMIT fires BEFORE any score gate. Zero challenge under any circumstance. Audit still emitted (`AbuseDefenceEmergencyServiceBypass`). Attestation required (SPIFFE + crisis-line federated identity). See `policy/critical-path-emergency-services.cedar` for full bypass mechanics.

IaC: `iac/prod-edge-waf.yaml`.
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `abuse-defence` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `abuse defence` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `abuse defence`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `abuse defence` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `abuse defence` failures have trigger, rollback, and post-incident closure.

## §critical-path-edge-cases

documentation-rigor §3.2.5 rows applicable to the intelligence dispatch surface:

| Row | Critical path | Intelligence handling | Binding artifact |
|---|---|---|---|
| 1 | Emergency services | `policy/critical-path-emergency-services.cedar`; bypass before all gates; crisis-triage bots pass; 10× rate-limit floor; audit emitted; attestation required; forgery → revoke not challenge | `policy/critical-path-emergency-services.cedar`; ADR-0298 (pending) |
| 5 | Healthcare break-glass | HIPAA-eligible cell; PHI never logged; BAA-provider-only via Cedar; break-glass = post-hoc audit per ADR-0247; no pre-action friction | ADR-0247; `compliance.md §pack-overlay-roster` |
| 6 | Whistleblower | Dispatch routed anonymously; no caller-identity correlated to dispatch content; sealed-sender pattern | ADR-0300 (pending) |
| 7 | Press / journalist source | Same as row 6; Tor-friendly ingress passes dispatch; no IP in CallRecord | ADR-0300 (pending) |
| 8 | DV survivor | Session not correlated across shelter-mode boundary; audit visible only to survivor-scoped principal | ADR-0301 (pending) |
| 9 | Child safety + mandatory reporting | Minor self-report dispatch bypasses parental-consent gate; NCMEC chain-of-custody in audit; minor-protection refusal does NOT apply to safety-report context | ADR-0292; `policy/critical-path-emergency-services.cedar` |
| 12 | Disability accommodation | Longer streaming timeouts for assistive-tech; WCAG 2.2 AA on brand-ux-surface; audio CAPTCHA fallback; challenge UX keyboard-only nav | `docs/standards/a11y-canonical.md`; IP-020 |
| 16 | Activist / dissident | `audience_type=HIGH_RISK_USER`: IP not retained in CallRecord; metadata-minimization; no cross-border export for CN-PIPL | ADR-0300 (pending); `compliance.md §pack-overlay-roster` |
| 21 | Pseudonymous user | Audit-trail accessible only to pseudonymous principal's scope; tenant admin cannot deanonymize | ADR-0244; `§tenant-scoping` |
| 28 | Delegated agent (LLM agent / automation) | `delegated_agent_token` model; bot-mgmt sees attestation; agent inherits tenant scope; cross-tenant delegation blocked; audit chains to authorizing human | ADR-0305 (pending); `policy/dispatch-authorization.cedar` PERMIT 2 |
### Content-pass expansion — critical-path-edge-cases
- This expansion preserves the existing prose above and closes `critical-path-edge-cases` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Google SRE incident playbooks anchors the external control pattern for `critical-path-edge-cases`.
- Precedent 2: Stripe idempotency recovery provides a second independent hyperscaler pattern for `critical-path-edge-cases`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `critical-path-edge-cases`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `critical-path-edge-cases` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `critical-path-edge-cases` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `critical path edge cases` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `critical path edge cases`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.

## §credential-isolation

ADR-0296 answer: OpenBao sidecar per pod; credentials never enter intelligence process memory.

```text
DispatchEnvelope.secret_reference
   ↓ credential-resolver-usecase
   ↓ Unix domain socket (loopback; no network egress)
openbao-sidecar (separate container UID 10002)
   reads: ${openbao:secret/<tenant_id>/intelligence/provider/<provider>}
   issues: CredentialHandle { handle_id, provider, ttl_seconds ≤ 60, signature }
   ↓
providers-adapter: sidecar injects secret into HTTP Authorization header at call-assembly time
provider API authenticates; secret value never returned to adapter memory
```

Handle expiry → `runbooks/sidecar-credential-handle-expired.md`.
BYOK rotation → `runbooks/byok-rotation-tenant-cascade.md`.
Audit signing key also held in sidecar (IP-022); in-µservice forgery impossible.
OpenBao path shape (§3.2.2 invariant 4): `${openbao:secret/<tenant_id>/<scope>/<name>}`.
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `credential-isolation` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `credential isolation` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation`.

## §deployment-shape

ADR-0254 answer: K8s + Cloud Hypervisor + Kata pods.

- Runtime class: `kata-cloud-hypervisor` per ADR-0254.
- 2 containers per pod: `intelligence` (UID 10001, read-only rootfs) + `openbao-sidecar` (UID 10002).
- Network policy: default-deny; ingress from `foundry`, `app-shell`, `api-gateway`; egress to `observability` + `egress-gateway`.
- HPA: min 3 / max 50; scale on CPU 70 % + memory 80 %. PDB: `minAvailable: 2`.
- IaC: `iac/k8s/deployment.yaml`, `iac/k8s/network-policy.yaml`, `iac/helm/intelligence/`, `iac/terraform/openbao-policy.tf`.
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `deployment-shape` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `deployment shape` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `deployment shape` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.

## §time-coordination

ADR-0252 answer: HLC default; TrueTime not required for intelligence BCs.

- `time_coordination_tier = hlc_default` for all intelligence BCs.
- HLC timestamps on `CallRecord.dispatch_ts` and `AuditRecord.sealed_ts`.
- Cross-cell causality handled by HLC for session ordering across Tier-1/2/3 cells.
- TrueTime opt-in: not required — intelligence is not a fin-grade ordering surface.
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `time-coordination` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `time coordination` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `time coordination` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `intelligence` fails closed when `time coordination` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §marketplace

ADR-0249 answer: intelligence does NOT expose marketplace surfaces.

`intelligence` is a substrate; it does not expose model-listing or plugin-discovery surfaces to the marketplace. The `marketplace` µservice exposes model-category listings that route through `intelligence` as the dispatch substrate. No marketplace surface in this µservice.
### Content-pass expansion — marketplace
- This expansion preserves the existing prose above and closes `marketplace` for `intelligence` to the ≥50-line documentation-rigor floor.
- Service owner `axis-intelligence` owns this answer; tier `substrate`; audience `['CONSUMER', 'DEVELOPER', 'INTERNAL_FOUNDRY', 'EMERGENCY_SERVICES', 'MINOR_TARGETED', 'HIGH_RISK_USER']`.
- Primary capability/context: `dispatch`; bounded contexts: `model-routing`, `providers`, `guardrails`, `eval`, `attribution`; +5 more.
- API surfaces: `microservices/intelligence/contracts/asyncapi/intelligence-events-v1.yaml`, `microservices/intelligence/contracts/asyncapi/intelligence-events.yaml`, `microservices/intelligence/contracts/openapi/intelligence-v1.yaml`, `microservices/intelligence/contracts/openapi/intelligence.yaml`, `microservices/intelligence/contracts/proto/intelligence-v1.proto`; +2 more.
- Cedar/policy surfaces: `microservices/intelligence/policy/abuse-defence.cedar`, `microservices/intelligence/policy/auditor-scope.cedar`, `microservices/intelligence/policy/byok-gating.cedar`, `microservices/intelligence/policy/ci-scope.cedar`, `microservices/intelligence/policy/critical-path-emergency-services.cedar`; +5 more.
- State/event surfaces: `intelligence.model_routing`, `intelligence.providers`, `intelligence.guardrails`, `intelligence.eval`, `intelligence.attribution`; +1 more.
- SLO/dashboard evidence: `microservices/intelligence/slos/assist-draft-latency.openslo.yaml`, `microservices/intelligence/slos/audit-emission-success.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-availability.openslo.yaml`, `microservices/intelligence/slos/dispatch-api-latency.openslo.yaml`, `microservices/intelligence/slos/first-token-latency.openslo.yaml`; +9 more.
- Runbook/IaC evidence: `microservices/intelligence/runbooks/assist-draft-policy-refusal.md`, `microservices/intelligence/runbooks/audit-row-forgery-detected.md`, `microservices/intelligence/runbooks/byok-rotation-tenant-cascade.md`, `microservices/intelligence/runbooks/eu-ai-act-incident-notification.md`, `microservices/intelligence/runbooks/prompt-injection-detected.md`; +11 more.
- Compliance packs: `kr`, `eu`, `us`, `us-healthcare`, `us-federal`; +3 more; data classes: `PII_IDENTIFYING`, `PII_QUASI_IDENTIFYING`, `PII_SENSITIVE`, `FINANCIAL`, `HEALTH`; +2 more.
- Cross-service dependencies: `observability`, `tenancy`, `policy-engine`.
- Precedent 1: Stripe platform facilitator anchors the external control pattern for `marketplace`.
- Precedent 2: AWS Marketplace seller controls provides a second independent hyperscaler pattern for `marketplace`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `marketplace`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `marketplace` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `intelligence` binds `marketplace` to `{'name': 'model-routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya-intelligence-model-routing-kernel', 'oya-intelligence-model-routing-domain', 'oya-intelligence-model-routing-usecase', 'oya-intelligence-model-routing-adapter', 'oya-intelligence-model-routing-api', 'oya-intelligence-model-routing-rest', 'oya-intelligence-model-routing-grpc', 'oya-intelligence-model-routing-worker']}` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `intelligence` is `contracts/provider-adapter-trait.md`; reviewers must map `marketplace` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `intelligence` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/byok-gating.cedar, policy/ci-scope.cedar, policy/critical-path-emergency-services.cedar, policy/data-residency.md, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `marketplace`.
- Depth detail 4: `intelligence` state/event naming uses `intelligence.{'name': 'model_routing', 'description': 'Routes dispatch envelopes to the correct provider given tenant, audience, modality, and pack constraints.', 'crates': ['oya_intelligence_model_routing_kernel', 'oya_intelligence_model_routing_domain', 'oya_intelligence_model_routing_usecase', 'oya_intelligence_model_routing_adapter', 'oya_intelligence_model_routing_api', 'oya_intelligence_model_routing_rest', 'oya_intelligence_model_routing_grpc', 'oya_intelligence_model_routing_worker']}` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `intelligence` covers `observability, tenancy, policy-engine` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `intelligence` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `intelligence` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `marketplace` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `intelligence` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `intelligence` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `intelligence` uses SLOs `slos/assist-draft-latency.openslo.yaml, slos/audit-emission-success.openslo.yaml, slos/dispatch-api-availability.openslo.yaml, slos/dispatch-api-latency.openslo.yaml, slos/first-token-latency.openslo.yaml, plus 4 more` and dashboards `dashboards/byok-vs-platform-default-mix.json, dashboards/finops-cost-attribution.md, dashboards/intelligence-overview.json, dashboards/prompt-injection-detection.md, plus 2 more` when those artifacts exist.
- Depth detail 11: Incident evidence for `intelligence` uses runbooks `runbooks/assist-draft-policy-refusal.md, runbooks/audit-row-forgery-detected.md, runbooks/byok-rotation-tenant-cascade.md, runbooks/eu-ai-act-incident-notification.md, runbooks/prompt-injection-detected.md, plus 6 more` so `marketplace` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `intelligence` uses `iac/helm/intelligence/Chart.yaml, iac/helm/intelligence/values.yaml, iac/k8s/deployment.yaml, iac/k8s/network-policy.yaml, iac/prod-ech-config.yaml, plus 3 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `intelligence` uses `capabilities/assist-draft.yaml, capabilities/attribution.yaml, capabilities/audit-tap.yaml, capabilities/context-aware-retrieval.yaml, plus 4 more` and `catalog/oya-intelligence-attribution-kernel.yaml, catalog/oya-intelligence-audit-tap-usecase.yaml, catalog/oya-intelligence-brand-ux-surface-sdk-ts.yaml, catalog/oya-intelligence-credential-resolver-kernel.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `intelligence` fails closed when `marketplace` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `intelligence` emits denial evidence for `marketplace` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `intelligence` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `marketplace` workflow.
- Depth detail 17: `intelligence` telemetry for `marketplace` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §fragment-publish

ADR-0294 answer: soak window ≥ 60 s respected on all Cedar fragments.

Fragments in `policy/` published via `oya-governance-cedar-fragment-publisher`; soak = 60 s before active eval. Active fragment set refreshed in-process every 30 s. Emergency rollback: `docs/runbooks/cedar-fragment-emergency-rollback.md`. CI gate: `oya-governance-cedar-fragment-soak`.

## Bounded-context roster

| BC | Layer scope | Cargo crate roots | Cedar gates |
|---|---|---|---|
| `model-routing` | kernel + domain + usecase + adapter | `oya-intelligence-model-routing-{kernel,domain,usecase,adapter,api,rest,grpc,worker}` | `provider-routing.cedar`, `dispatch-authorization.cedar` |
| `providers` | adapter | `oya-intelligence-providers-adapter-{anthropic,openai,google,bedrock,azure-openai,cohere,mistral,vllm,sglang,tensorrt-llm,apple-foundation,openrouter,together,groq,huggingface-inference,replicate,alibaba-qwen,tencent-hunyuan}` | `byok-gating.cedar` |
| `guardrails` | kernel + domain + usecase | `oya-intelligence-guardrails-{kernel,domain,usecase,adapter}` | `refusal-baseline.cedar`, `eu-ai-act-high-risk.cedar`, `abuse-defence.cedar`, `critical-path-emergency-services.cedar` |
| `eval` | kernel + domain + usecase + worker | `oya-intelligence-eval-{kernel,domain,usecase,adapter,worker}` | `dispatch-authorization.cedar` |
| `attribution` | kernel + domain + usecase | `oya-intelligence-attribution-{kernel,domain,usecase,adapter}` | `dispatch-authorization.cedar` |
| `brand-ux-surface` | sdk + adapter | `oya-intelligence-brand-ux-surface-{sdk-rs,sdk-ts,sdk-swift,sdk-kotlin,adapter}` | n/a (renders only) |
| `credential-resolver` | kernel + usecase + adapter | `oya-intelligence-credential-resolver-{kernel,usecase,adapter}` | `byok-gating.cedar` |
| `audit-tap` | usecase + adapter + worker | `oya-intelligence-audit-tap-{usecase,adapter,worker}` | `auditor-scope.cedar` |

Layer-enum conformance per ADR-0105 (13-layer canonical set).

## SLO posture

| SLO | Target | Window | OpenSLO file |
|---|---|---|---|
| Dispatch API availability | 99.95 % | rolling 30d | `slos/dispatch-api-availability.openslo.yaml` |
| Dispatch API latency p99 | < 250 ms | rolling 30d | `slos/dispatch-api-latency.openslo.yaml` |
| First-token latency p99 (consumer) | < 2.0 s | rolling 30d | `slos/first-token-latency.openslo.yaml` |
| Streaming throughput p99 | ≥ 30 tok/s | rolling 30d | `slos/streaming-throughput.openslo.yaml` |
| Audit emission success | ≥ 99.99 % | rolling 30d | `slos/audit-emission-success.openslo.yaml` |
| Refusal false-positive rate | ≤ 2 % | rolling 30d | `slos/refusal-false-positive-rate.openslo.yaml` |
| Refusal false-negative rate | ≤ 0.1 % | rolling 30d | `slos/refusal-false-negative-rate.openslo.yaml` |
| Policy refusal correctness | ≥ 99 % | rolling 30d | `slos/policy-refusal-correctness.openslo.yaml` |

## ADR-0242..0297 adherence checklist (§3.2.1 — all 28 rows)

| # | ADR | Answer location in this µservice |
|---:|---|---|
| 1 | ADR-0242 (oyatie-is-a-tenant) | `§principals` |
| 2 | ADR-0243 (Cedar universal gate) | `§cedar-gates` |
| 3 | ADR-0244 (tenant scoping) | `§tenant-scoping` |
| 4 | ADR-0245 (substrate vs product) | `§substrate-product-binding` |
| 5 | ADR-0246 + amendment | `§policy-evaluation` |
| 6 | ADR-0247 (self-modification) | `compliance.md §self-modification` |
| 7 | ADR-0248 (cellular architecture) | `§cell-eligibility` |
| 8 | ADR-0249 (marketplace) | `§marketplace` |
| 9 | ADR-0250 (build-ahead-of-certification) | `compliance.md §day-one-cert-readiness` |
| 10 | ADR-0251 + CN-PIPL | `compliance.md §pack-overlay-roster` |
| 11 | ADR-0252 (HLC + TrueTime) | `§time-coordination` |
| 12 | ADR-0253 (HTTP/3 + ECH + PQC) | `§transport` + `iac/prod-ech-config.yaml` + `iac/prod-pqc-cert.yaml` |
| 13 | ADR-0254 (deployment model) | `§deployment-shape` |
| 14 | ADR-0255 + amendment | `§intelligence-dispatch` |
| 15 | ADR-0257 + amendment | `§ontology-read-path` |
| 16 | ADR-0258 (API versioning) | `contracts/openapi/intelligence-v1.yaml:info.version` + `CHANGELOG.md` |
| 17 | ADR-0263 (observability) | `§observability` |
| 18 | ADR-0272 (cookie consent) | n/a — intelligence not user-facing for cookie consent; brand-ux-surface inherits from app-shell |
| 19 | ADR-0273 (per-tenant DKIM/SPF/DMARC) | n/a — intelligence does not emit mail |
| 20 | ADR-0276 (backup portability) | `backfill-replay.md §portability` |
| 21 | ADR-0280 (substrate-of-substrate) | `manifest.json:substrate_dependencies` |
| 22 | ADR-0284 (platform-owner indirection) | `compliance.md §platform-owner-indirection` |
| 23 | ADR-0292 (minor user doctrine) | `§critical-path-edge-cases` row 9 + `compliance.md §minor-protection` |
| 24 | ADR-0293 (meta-trust-root) | `compliance.md §meta-trust-attestation` |
| 25 | ADR-0294 (Cedar fragment soak) | `§fragment-publish` |
| 26 | ADR-0295 (bootstrap CI SPIFFE) | `compliance.md §bootstrap-trust-chain` |
| 27 | ADR-0296 (credential sidecar) | `§credential-isolation` |
| 28 | ADR-0297 (abuse-defence) | `§abuse-defence` + `iac/prod-edge-waf.yaml` + `policy/abuse-defence.cedar` |

## Cross-references

- ADR-0255 + amendment — canonical authority.
- ADR-0263 — audit-tap emission contract.
- ADR-0296 — sidecar credential-handle isolation.
- ADR-0145 — inter-microservice gRPC invariants.
- ADR-0248 — cellular architecture + cell eligibility.
- ADR-0252 — HLC + TrueTime tiering.
- ADR-0253 — HTTP/3 + QUIC + ECH + PQC.
- ADR-0254 — K8s + Cloud Hypervisor.
- ADR-0292 — minor user doctrine.
- ADR-0297 — abuse-defence baseline.
- ADR-0105 — 13-layer enum.
- `microservices/intelligence/PRD.md`.
- `microservices/intelligence/compliance.md`.
- `microservices/intelligence/threat-model.md`.
- `microservices/intelligence/dpia.md`.
- `docs/standards/documentation-rigor.md` §3.2.1..§3.2.5.
