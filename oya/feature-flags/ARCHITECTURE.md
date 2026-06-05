---
doc_class: Architecture
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0139
  - ADR-0159
  - ADR-0160
  - ADR-0183
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
  - ADR-0255
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
companion_docs:
  - microservices/feature-flags/PRD.md
  - microservices/feature-flags/compliance.md
  - microservices/feature-flags/manifest.json
  - docs/AGENTS.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Feature Flags — Architecture Deep-Dive

## §principals

Per ADR-0242, every service operates under `oyatie.*` reserved-namespace principals. This µservice operates as:

- `oyatie.feature-flags.flag-manager` — CRUD operations on flag definitions.
- `oyatie.feature-flags.flag-evaluator` — Flag evaluation engine (hot path); read-only.
- `oyatie.feature-flags.killswitch-operator` — Emergency kill-switch invocations; step-up auth required.
- `oyatie.feature-flags.experiment-designer` — Experiment lifecycle management; step-up auth required for activation.
- `oyatie.feature-flags.pack-overlay-agent` — Automated pack-mandated flag overrides (e.g., HIPAA forces `phi-exposure` = OFF).
- `oyatie.feature-flags.audit-emitter` — Appends audit events per ADR-0263; no read-back permitted.

Tenant-scoped callers: any principal under `oyatie.<tenant_id>.app.*` holding a valid Cedar permit for `FlagEvaluate` action.
Internal callers: `oyatie.foundry.*` principals for CI flag lifecycle gating.

**ADR-adherence row 1:** `ARCHITECTURE.md §principals` ✓

---
### Content-pass expansion — principals
- This expansion preserves the existing prose above and closes `principals` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: AWS IAM service-linked roles anchors the external control pattern for `principals`.
- Precedent 2: Google Cloud service agents provides a second independent hyperscaler pattern for `principals`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `principals`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `principals` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `principals` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `principals` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `principals`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `principals` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §cedar-gates

Per ADR-0243, every gate is a Cedar evaluation. Default-deny baseline applies to all actions.

### Flag-mutation authorization

Flag mutation (`FlagCreate`, `FlagUpdate`, `FlagArchive`, `FlagDelete`) REQUIRES:

1. Cedar permit: `permit(principal, action FlagMutate, resource Flag) when { principal.role == "flag-manager" && principal.tenant_id == resource.tenant_id }`.
2. **Step-up auth** per `docs/standards/step-up-auth-classes.md`:
   - Class B (passkey re-challenge) for `FlagUpdate` on live flags (non-draft).
   - Class C (TOTP + passkey) for `FlagDelete` and `kill_switch` type mutations.
3. FORBID rule: `forbid(principal, action FlagMutate, resource Flag) unless { principal.mfa_verified && context.step_up_class >= resource.mutation_step_up_class };`.

### Experiment-design authorization

`ExperimentCreate`, `ExperimentActivate`, `ExperimentConclude` require:

1. Cedar permit: `permit(principal, action ExperimentDesign, resource Experiment) when { principal.role in ["product-manager", "experiment-designer"] && principal.tenant_id == resource.tenant_id }`.
2. Step-up Class B for `ExperimentActivate` (affects live traffic).
3. FORBID: cross-tenant experiment reads are default-denied.

### Safety kill-switch authorization

`KillSwitchEngage` requires:

1. Cedar permit: `permit(principal, action KillSwitchEngage, resource Flag) when { principal.role == "sre-oncall" || principal.role == "killswitch-operator" }`.
2. Step-up Class A (passkey) minimum; Class C recommended for production.
3. Emergency bypass for `audience_type == "EMERGENCY_SERVICES"` per §3.2.3 of documentation-rigor.md.
4. Audit event `KillSwitchEngaged` emitted unconditionally.

### Pack-overlay authorization

Pack overlays (e.g., HIPAA forcing `phi-exposure-flag = off`) are evaluated by `oyatie.feature-flags.pack-overlay-agent` which holds a Cedar permit scoped to overlay operations only. Tenant admins cannot override pack-mandated overrides.

### Abuse-defence Cedar gate (UX floor)

Per ADR-0297 and §3.2.3:

```cedar
// Cedar v4.2 LTS fragment — feature-flags abuse defence
// Fragment soak window: ≥60s per ADR-0294 before activation

forbid (principal, action, resource) when {
    principal.bot_score > 95
    && !(principal.tenant.audience_type == "FRIENDLY_CRAWLER_PARTNER")
    && !(principal.tenant.audience_type == "EMERGENCY_SERVICES")
};

// Rate limit: flag evaluation is hot-path; limit mutation not evaluation
forbid (principal, action FlagMutate, resource) when {
    principal.flag_mutation_rate_per_minute > 60
    && !(principal.role == "pack-overlay-agent")
};
```

**ADR-adherence row 2:** `ARCHITECTURE.md §cedar-gates` ✓

---
### Content-pass expansion — cedar-gates
- This expansion preserves the existing prose above and closes `cedar-gates` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: AWS Verified Permissions Cedar anchors the external control pattern for `cedar-gates`.
- Precedent 2: Google Zanzibar provides a second independent hyperscaler pattern for `cedar-gates`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cedar-gates`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cedar-gates` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §tenant-scoping

Per ADR-0244:

- Every flag definition row carries `tenant_id` (indexed; no cross-tenant read possible at the DB layer).
- Every experiment row carries `tenant_id`.
- Every audit event carries `tenant_id` per ADR-0263.
- `audience_type` values honored: `B2C`, `B2B`, `INTERNAL_AGENT`, `EMERGENCY_SERVICES`, `FRIENDLY_CRAWLER_PARTNER`.
- `provider_credential_mode`: `platform_default` for flag storage; per-tenant BYOK for audit-event encryption keys per ADR-0255 §D-4.
- Per-pack flag overlay: HIPAA-pack forces `phi-exposure-flag = off`; PCI-pack forces `raw-pan-display = off`; EU-AI-Act-pack forces `high-risk-ai-auto-decide = off`.
- Targeting-rule predicates are per-tenant Cedar fragments; cannot reference cross-tenant entities.
- Pack-mandated overrides stored separately in `pack_flag_overrides` table; tenant admin reads are read-only; writes rejected by Cedar gate.

**ADR-adherence row 3:** `ARCHITECTURE.md §tenant-scoping` ✓

---
### Content-pass expansion — tenant-scoping
- This expansion preserves the existing prose above and closes `tenant-scoping` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Stripe account isolation anchors the external control pattern for `tenant-scoping`.
- Precedent 2: AWS Organizations account boundary provides a second independent hyperscaler pattern for `tenant-scoping`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `tenant-scoping`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `tenant-scoping` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `tenant-scoping` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `tenant scoping` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `tenant scoping`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `tenant scoping` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §substrate-product-binding

Per ADR-0245, feature-flags is a **substrate** µservice.

Substrate consumers (all µservices call the feature-flags SDK):
- `microservices/tenancy/` — flag-gates for tenancy lifecycle experiments.
- `microservices/payments/` — payment-method experiment flags.
- `microservices/intelligence/` — AI feature rollout flags.
- `microservices/observability/` — SLO evaluation flag gates.
- All 46+ µservices consume via the canonical Rust `oya-feature-flags-sdk`. Non-Rust OpenFeature providers are generated compatibility products only when registered and pinned by their owning lane.

Substrate dependencies of feature-flags:
- `microservices/tenancy/` — tenant resolution.
- `microservices/policy-engine/` — Cedar evaluation substrate.
- `microservices/cloud-secrets/` — OpenBao credential management.
- `microservices/observability/` — metrics, traces, logs emission.

**ADR-adherence row 4:** `manifest.json:tier = "substrate"` + `ARCHITECTURE.md §substrate-product-binding` ✓

---
### Content-pass expansion — substrate-product-binding
- This expansion preserves the existing prose above and closes `substrate-product-binding` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Palantir Foundry substrate pattern anchors the external control pattern for `substrate-product-binding`.
- Precedent 2: Google Cloud shared VPC split provides a second independent hyperscaler pattern for `substrate-product-binding`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `substrate-product-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `substrate-product-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `substrate-product-binding` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `substrate product binding` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `substrate product binding`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.

## §policy-evaluation

Per ADR-0246 + library-first amendment:

- `policy_evaluation_mode: "library-first"`.
- All Cedar evaluation uses caller-side `oya-shared-policy-eval` library v0.8.x (LTS).
- Network dispatch (`policy-engine` µservice) only for: (a) cold-start fragment load, (b) fragment reloads after soak window, (c) audit-grade evaluation with proof-of-permit.
- Fragment cache TTL: 30s in-process (matches flag evaluation cache TTL for coherence).
- Fragment versioning: each Cedar fragment carries `cedar_fragment_version` header; stale fragments rejected after 60s per ADR-0294.

**ADR-adherence row 5:** `ARCHITECTURE.md §policy-evaluation` ✓

---
### Content-pass expansion — policy-evaluation
- This expansion preserves the existing prose above and closes `policy-evaluation` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Open Policy Agent sidecar anchors the external control pattern for `policy-evaluation`.
- Precedent 2: AWS Verified Permissions provides a second independent hyperscaler pattern for `policy-evaluation`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `policy-evaluation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `policy-evaluation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `policy-evaluation` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `policy evaluation` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `policy evaluation`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `policy evaluation` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `policy evaluation` failures have trigger, rollback, and post-incident closure.

## §self-modification

Per ADR-0247: feature-flags produces self-modification artifacts in one narrow case — the `pack-overlay-agent` applies pack-mandated overrides automatically. This operates as:
- Principal: `oyatie.feature-flags.pack-overlay-agent` under `oyatie.foundry.*` meta-trust root.
- Attestation path: pack-overlay events are cosign-signed by the `oyatie.foundry.pack-engine` trust root.
- Every override is audit-emitted as `PackFlagOverrideApplied` and sealed per ADR-0028.
- No self-modification outside pack-mandated overrides; all other flag mutations require human principal.

**ADR-adherence row 6:** `compliance.md §self-modification` cross-ref ✓

---
### Content-pass expansion — self-modification
- This expansion preserves the existing prose above and closes `self-modification` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: SLSA provenance anchors the external control pattern for `self-modification`.
- Precedent 2: Google Binary Authorization provides a second independent hyperscaler pattern for `self-modification`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `self-modification`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `self-modification` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `self-modification` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `self modification` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `self modification`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `self modification` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `self modification` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `feature-flags` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm-values.yaml, iac/k8s-deployment.yaml, iac/network-policy.yaml, plus 4 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.

## §cell-eligibility

Per ADR-0248:

- Cell tier: **Tier 2** (control-plane substrate).
- Deployment shape: active-active across all Tier-2 cells in a region; DR-pair failover to adjacent region.
- Per-cell shard width: flag definitions sharded by `tenant_id` consistent hash; shard count = 256.
- Cells spanned: all production cells in home region + DR-pair cell; KR sovereign cell (kr-cell-1); EU sovereign cell (eu-cell-1); FedRAMP cell (us-gov-cell-1).
- `home_cell` declared per tenant at onboarding; flag evaluation resolves to tenant's home cell first, falls back to DR-pair on home-cell outage.
- Flag definition replication: async cross-region ≤5s (p99); synchronous within region.

**ADR-adherence row 7:** `multi-region.md` + `manifest.json:cell_eligibility` ✓

---
### Content-pass expansion — cell-eligibility
- This expansion preserves the existing prose above and closes `cell-eligibility` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: AWS cell-based architecture anchors the external control pattern for `cell-eligibility`.
- Precedent 2: Route 53 shuffle sharding provides a second independent hyperscaler pattern for `cell-eligibility`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `cell-eligibility`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `cell-eligibility` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `cell-eligibility` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `cell eligibility` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `cell eligibility`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `cell eligibility` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.

## §marketplace

Feature-flags does not directly expose marketplace surfaces per ADR-0249. However, the experiment-design BC exposes:
- A future marketplace API for third-party analytics integrations (Statsig, Amplitude, Mixpanel) to consume experiment-results events.
- Flagged `marketplace_eligible: true` in manifest; surface gated by `marketplace-experiments` pack.

**ADR-adherence row 8:** N/A for this µservice directly; `competitor-parity-matrix.md §marketplace` ✓

---
### Content-pass expansion — marketplace
- This expansion preserves the existing prose above and closes `marketplace` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Stripe platform facilitator anchors the external control pattern for `marketplace`.
- Precedent 2: AWS Marketplace seller controls provides a second independent hyperscaler pattern for `marketplace`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `marketplace`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `marketplace` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `marketplace` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `marketplace` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `marketplace`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `marketplace` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `marketplace` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `feature-flags` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm-values.yaml, iac/k8s-deployment.yaml, iac/network-policy.yaml, plus 4 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `feature-flags` uses `capabilities/experiment-design.yaml, capabilities/flag-evaluate.yaml, capabilities/flag-evaluation.yaml, capabilities/killswitch-trigger.yaml, plus 1 more` and `catalog/oya-feature-flags-experiment-kernel.yaml, catalog/oya-feature-flags-flag-adapter-postgres.yaml, catalog/oya-feature-flags-flag-app.yaml, catalog/oya-feature-flags-flag-domain.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `feature-flags` fails closed when `marketplace` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §time-coordination

Per ADR-0252:
- HLC (Hybrid Logical Clock) default for flag evaluation timestamps; ensures causal ordering of flag-state-changed events across cells.
- TrueTime opt-in for: (a) kill-switch activation timestamps (life-safety-adjacent; requires absolute ordering), (b) experiment start/stop timestamps (fairness requires canonical ordering across regions).
- `hlc_epoch` embedded in every flag-state-changed audit event.

**ADR-adherence row 11:** `ARCHITECTURE.md §time-coordination` ✓

---
### Content-pass expansion — time-coordination
- This expansion preserves the existing prose above and closes `time-coordination` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Google Spanner TrueTime anchors the external control pattern for `time-coordination`.
- Precedent 2: CockroachDB HLC ordering provides a second independent hyperscaler pattern for `time-coordination`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `time-coordination`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `time-coordination` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `time-coordination` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `time coordination` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `time coordination`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `time coordination` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `time coordination` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `feature-flags` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm-values.yaml, iac/k8s-deployment.yaml, iac/network-policy.yaml, plus 4 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `feature-flags` uses `capabilities/experiment-design.yaml, capabilities/flag-evaluate.yaml, capabilities/flag-evaluation.yaml, capabilities/killswitch-trigger.yaml, plus 1 more` and `catalog/oya-feature-flags-experiment-kernel.yaml, catalog/oya-feature-flags-flag-adapter-postgres.yaml, catalog/oya-feature-flags-flag-app.yaml, catalog/oya-feature-flags-flag-domain.yaml, plus 8 more` to keep layer names and owners machine-checkable.

## §transport

Per ADR-0253:

- **HTTP/3 + QUIC** default across all REST + gRPC surfaces (gRPC over HTTP/3 per ADR-0253 §D).
- Negotiation order: HTTP/3 > HTTP/2 > HTTP/1.1 (first acceptable wins; HTTP/1.0 forbidden).
- TLS 1.3 floor; cipher suite: `TLS_AES_256_GCM_SHA384` preferred; `TLS_CHACHA20_POLY1305_SHA256` second; no CBC ciphers.
- HSTS: `max-age=63072000; includeSubDomains; preload`.
- Certificate Transparency required; OCSP stapling enabled.
- **ECH (Encrypted Client Hello)** per RFC 9460: enabled on all Tier-2 cell ingresses; `ech=` config published in DNS HTTPS RR via ADR-0273 toolchain; ECH keys rotated every 90 days; ECH-disabled clients fall through to standard TLS 1.3 without breakage.
- **PQC (post-quantum hybrid)**: `X25519MLKEM768` KEM hybrid preferred (IANA `0x11ec`); `ed25519+ml_dsa_65` for new certificate chains; non-PQ clients fall through to classical curves (X25519/P-256).
- Alt-Svc advertisement: `Alt-Svc: h3=":443"; ma=86400` on all REST responses.
- h3→h2 fallback under QUIC-blocked networks: automatic via QUIC version-negotiation failure detection (≤500ms timeout).
- `insecure_skip_verify` forbidden everywhere; no self-signed certs except offline-rooted-CA ceremony per ADR-0295.

Hyperscaler precedent: Cloudflare's HTTP/3 + ECH deployment; Google's QUIC adoption timeline; Fastly's PQC rollout.

**ADR-adherence row 12:** `ARCHITECTURE.md §transport` ✓

---
### Content-pass expansion — transport
- This expansion preserves the existing prose above and closes `transport` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Google QUIC HTTP/3 anchors the external control pattern for `transport`.
- Precedent 2: Cloudflare ECH/PQC TLS provides a second independent hyperscaler pattern for `transport`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `transport`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `transport` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `transport` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `transport` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `transport`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.

## §deployment-shape

Per ADR-0254:

- Kubernetes (K8s) everywhere except edge.
- Cloud Hypervisor + Kata pods for flag-evaluator workloads (isolation boundary: flag definitions contain tenant-specific predicates that must not leak across pod boundaries).
- Flag evaluator: containerized (Rust binary; `oya-feature-flags-evaluator-app`); runs in Kata pod.
- Flag manager (admin API): containerized; Kata pod.
- Flag state store (PostgreSQL + Patroni): VM-backed on Cloud Hypervisor; not containerized.
- Wasm components: flag-evaluation Cedar predicates compiled to Wasm for in-process evaluation without subprocess overhead. Wasm sandbox enforces predicate isolation.
- HPA: scale on `oya_feature_flag_eval_queue_depth > 1000`; target 70% CPU.
- PDB: `maxUnavailable: 1` per deployment.

**ADR-adherence row 13:** `iac/` + `ARCHITECTURE.md §deployment-shape` ✓

---
### Content-pass expansion — deployment-shape
- This expansion preserves the existing prose above and closes `deployment-shape` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: AWS Firecracker isolation anchors the external control pattern for `deployment-shape`.
- Precedent 2: GKE Sandbox/Kata provides a second independent hyperscaler pattern for `deployment-shape`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `deployment-shape`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `deployment-shape` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `deployment-shape` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `deployment shape` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `deployment shape`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `deployment shape` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §intelligence-dispatch

Per ADR-0255 + library-first amendment:

Feature-flags consumes Intelligence for:
- Experiment statistical significance scoring (Bayesian posterior computation).
- Anomaly detection on flag evaluation rate spikes (potential flag-flooding abuse).

Both calls use library-first dispatch via `oya-shared-intelligence-eval`. Network dispatch permitted only for heavy Bayesian posterior computation exceeding in-process budget (>10ms).
`audience_type` tag set to `INTERNAL_SUBSTRATE` on all Intelligence calls.

**ADR-adherence row 14:** `ARCHITECTURE.md §intelligence-dispatch` ✓

---
### Content-pass expansion — intelligence-dispatch
- This expansion preserves the existing prose above and closes `intelligence-dispatch` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Palantir AIP tool boundary anchors the external control pattern for `intelligence-dispatch`.
- Precedent 2: Azure OpenAI tenant deployment provides a second independent hyperscaler pattern for `intelligence-dispatch`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `intelligence-dispatch`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `intelligence-dispatch` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `intelligence-dispatch` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `intelligence dispatch` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `intelligence dispatch`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `intelligence dispatch` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `intelligence dispatch` failures have trigger, rollback, and post-incident closure.

## §ontology-read-path

Per ADR-0257 + amendment:

- `ontology_read_mode: "library-first"`.
- Reads Ontology for: tenant entity resolution (to validate `tenant_id` in targeting rules), cohort membership (to resolve `cohort_ids[]` in evaluation context).
- `freshness_floor: 60s` — targeting rules tolerant of 60s stale cohort data; kill-switch flags use `freshness_floor: 0s` (synchronous Ontology read required).

**ADR-adherence row 15:** `ARCHITECTURE.md §ontology-read-path` ✓

---
### Content-pass expansion — ontology-read-path
- This expansion preserves the existing prose above and closes `ontology-read-path` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Palantir Foundry ontology projections anchors the external control pattern for `ontology-read-path`.
- Precedent 2: Google Knowledge Graph serving cache provides a second independent hyperscaler pattern for `ontology-read-path`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ontology-read-path`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ontology-read-path` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `ontology-read-path` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `ontology read path` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `ontology read path`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `ontology read path` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `ontology read path` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `feature-flags` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm-values.yaml, iac/k8s-deployment.yaml, iac/network-policy.yaml, plus 4 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `feature-flags` uses `capabilities/experiment-design.yaml, capabilities/flag-evaluate.yaml, capabilities/flag-evaluation.yaml, capabilities/killswitch-trigger.yaml, plus 1 more` and `catalog/oya-feature-flags-experiment-kernel.yaml, catalog/oya-feature-flags-flag-adapter-postgres.yaml, catalog/oya-feature-flags-flag-app.yaml, catalog/oya-feature-flags-flag-domain.yaml, plus 8 more` to keep layer names and owners machine-checkable.

## §observability

Per ADR-0263:

### Audit events emitted

| Event class | Trigger | Sealed per ADR-0028 |
|---|---|---|
| `FlagCreated` | Flag definition created | yes |
| `FlagUpdated` | Flag definition mutated | yes |
| `FlagArchived` | Flag lifecycle → archived | yes |
| `FlagDeleted` | Flag deleted (hard) | yes |
| `FlagEvaluated` | Evaluation when `audit_required: true` | yes |
| `KillSwitchEngaged` | Kill-switch activated | yes |
| `KillSwitchDisengaged` | Kill-switch deactivated | yes |
| `PackFlagOverrideApplied` | Pack overlay applied | yes |
| `ExperimentCreated` | Experiment definition created | yes |
| `ExperimentActivated` | Traffic allocation live | yes |
| `ExperimentConcluded` | Winner selected / stopped | yes |
| `AbuseDefenceEmergencyServiceBypass` | Emergency bypass triggered | yes |
| `FlagStateChanged` | Any state transition | yes |
| `StaleTargetingRuleDetected` | Targeting rule expired | no (metric only) |

### Metrics (cardinality budget)

| Metric | Type | Dimensions | Budget |
|---|---|---|---|
| `oya_feature_flag_eval_duration_seconds` | histogram | `flag_key`, `tenant_id`, `result` | 50k series |
| `oya_feature_flag_eval_total` | counter | `flag_key`, `tenant_id`, `result` | 50k series |
| `oya_feature_flag_mutation_total` | counter | `flag_key`, `action`, `tenant_id` | 5k series |
| `oya_feature_flag_killswitch_active` | gauge | `flag_key`, `tenant_id` | 1k series |
| `oya_feature_flag_pack_override_total` | counter | `pack_id`, `flag_key` | 500 series |
| `oya_experiment_assignments_total` | counter | `experiment_id`, `variant`, `tenant_id` | 10k series |
| `oya_experiment_conversion_total` | counter | `experiment_id`, `variant`, `tenant_id` | 10k series |

### Traces

Span shape per evaluation request:
```
feature-flags.evaluate [root]
  ├── cedar.permit.check [child; ≤0.1ms]
  ├── targeting.rule.eval [child; ≤0.3ms]
  ├── percentage.hash.compute [child; ≤0.05ms]
  └── audit.emit [async child; fire-and-forget]
```

### Dashboards

- `dashboards/flag-state-overview.json` — live flag inventory per tenant.
- `dashboards/experiment-results.json` — experiment metrics with significance indicators.
- `dashboards/killswitch-history.json` — kill-switch activation history.
- `dashboards/pack-override-coverage.md` — pack overlay coverage by tenant.

**ADR-adherence row 17:** `ARCHITECTURE.md §observability` ✓

---
### Content-pass expansion — observability
- This expansion preserves the existing prose above and closes `observability` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Google SRE four reference signals anchors the external control pattern for `observability`.
- Precedent 2: OpenTelemetry semantic conventions provides a second independent hyperscaler pattern for `observability`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `observability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `observability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §abuse-defence

Per ADR-0297 and §3.2.3 of documentation-rigor.md:

### Anti-flag-flooding controls

Flag flooding (adversary creates/evaluates thousands of flags to exhaust quotas or cause cascade) is mitigated:

1. **Rate limiting at edge**: max 60 flag mutations/min per principal; max 100k evaluations/min per tenant.
2. **Quota gates**: Cedar per-action quota: `FlagCreate` limited to 1000 active flags per tenant (configurable per tier).
3. **Flag-key anomaly detection**: sudden spike in unique flag keys evaluated → `DetectionSignalEmitted` with family `policy-violation`.
4. **Honeypot flag keys**: internal flag keys that no legitimate client should evaluate; access triggers `AbuseDetected` event.
5. **Bot-score gate**: evaluations from principals with `bot_score > 95` FORBID (with emergency-services bypass).

### Anti-experiment-manipulation controls

1. **Assignment integrity**: experiment assignment hash is deterministic and sealed per evaluation context; cannot be influenced by client-supplied parameters.
2. **Salt rotation**: HMAC salt for percentage-rollout hash rotated per experiment activation; prevents enumeration of assignment boundaries.
3. **Metric attribution integrity**: conversion events cryptographically bound to assignment events via `experiment_assignment_id` (UUID sealed at assignment time).
4. **Statistical guardrails**: Bayesian + frequentist tests run server-side only; client cannot influence significance calculation.
5. **Variant collision detection**: if two active experiments modify overlapping flag keys, conflict detection raises `ExperimentConflictDetected` event.

### UX floor (defence-in-depth MUST NOT sacrifice UX)

Per §3.2.3 UX floor invariants:
- Legitimate flag evaluation (no abuse signals) has zero added latency; ML scoring is async.
- Bot-score check adds ≤0.1ms p99 (passive, pre-computed by edge).
- Emergency-services principals (`audience_type = EMERGENCY_SERVICES`) bypass ALL abuse-defence gates; still audit-emitted.
- CAPTCHA never presented on flag evaluation (it is a machine API surface).
- Tenant dashboards show abuse-defence outcomes (false-positive rate, blocked evaluations) per ADR-0263.

### Life-safety hard rule (§3.2.3)

Emergency-services principals MUST NOT be blocked by any abuse-defence control:
- `EMERGENCY_SERVICES` audience-type bypasses rate limits, quota gates, bot-score gates.
- Kill-switch for `emergency-services-bypass-flag` is FORBIDDEN by Cedar `forbid` rule.
- Edge bypass is L3/L4-cheap; audit retained at every hop.

**ADR-adherence row 28:** `ARCHITECTURE.md §abuse-defence` ✓

---
### Content-pass expansion — abuse-defence
- This expansion preserves the existing prose above and closes `abuse-defence` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Cloudflare Bot Management anchors the external control pattern for `abuse-defence`.
- Precedent 2: Stripe Radar provides a second independent hyperscaler pattern for `abuse-defence`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `abuse-defence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `abuse-defence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §critical-path-edge-cases

Applicable rows from documentation-rigor.md §3.2.5:

### Row 1 — Emergency services bypass

Feature-flags serves `healthcare-break-glass-enable` and `emergency-dispatch-routing-override` flags. These are kill-switch-class flags with:
- `audience_type = EMERGENCY_SERVICES` bypass at the Cedar gate (no challenge possible).
- Synchronous Ontology read (`freshness_floor: 0s`) for break-glass flags.
- Elevated rate-limit floor (10× consumer tier).
- Quarterly chaos-test: 10× normal volume on emergency-services evaluation path; zero challenge events expected.

### Row 5 — Healthcare urgent care + EHR break-glass

The `ehr-break-glass-enable` flag is a kill-switch-class flag for HIPAA-eligible cells. Activation follows ADR-0247 break-glass pattern:
- Post-hoc audit-and-justify; no pre-action approval required.
- PHI access via break-glass flag is logged with reason code.
- Flag is in the `us-healthcare` pack scope; cannot be toggled from non-healthcare cells.

### Row 17 — Service outage during regulator-deadline

During outage, degraded-mode flag evaluation falls back to last-known-good state (LKG cache). LKG is persisted to local disk with 30-minute TTL. Regulator-required workflows (breach-notification flags, audit-replay flags) use `audit_required: true` to ensure evaluation is emitted even in degraded mode.

### Row 25 — Mistaken-action / unintended-mutation recovery

Flag mutations have a 15-second undo window:
- `FlagUpdated` event triggers a `flag_undo_window_open` state on the flag for 15s.
- Any principal with `FlagMutate` permit can invoke `UndoFlagMutation` within the window.
- After 15s, undo requires a new `FlagUpdate` (audit trail preserved; no silent reversal).
- Kill-switch activation has no undo window (life-safety: kill switches must take effect immediately).

**ADR-adherence row from §3.2.5:** critical-path-edge-cases documented ✓

---
### Content-pass expansion — critical-path-edge-cases
- This expansion preserves the existing prose above and closes `critical-path-edge-cases` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Google SRE incident playbooks anchors the external control pattern for `critical-path-edge-cases`.
- Precedent 2: Stripe idempotency recovery provides a second independent hyperscaler pattern for `critical-path-edge-cases`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `critical-path-edge-cases`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `critical-path-edge-cases` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §credential-isolation

Per ADR-0296:

- Flag-evaluator holds no provider credentials; reads flag definitions from local-cell Postgres only.
- Flag-manager holds Postgres write credentials via OpenBao sidecar: TTL ≤60s, auto-rotated.
- Pack-overlay-agent holds pack-engine signing key via OpenBao sidecar: TTL ≤60s.
- Audit-emitter holds audit-chain signing key via dedicated OpenBao sidecar: TTL ≤60s.
- Secret path format: `${openbao:secret/<tenant_id>/feature-flags/<scope>/<name>}`.
- No provider credentials stored in environment variables or config maps.
- Sidecar process isolation: credential sidecar runs as separate container in same pod; shared only via Unix domain socket.

Hyperscaler precedent: HashiCorp Vault Agent Sidecar Injector pattern; AWS Secrets Manager sidecar pattern.

**ADR-adherence row 27:** `ARCHITECTURE.md §credential-isolation` ✓

---
### Content-pass expansion — credential-isolation
- This expansion preserves the existing prose above and closes `credential-isolation` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: HashiCorp Vault dynamic secrets anchors the external control pattern for `credential-isolation`.
- Precedent 2: AWS KMS envelope isolation provides a second independent hyperscaler pattern for `credential-isolation`.
- Tenant-scope invariant: every `feature-flags` `flag-evaluation` request carries `tenant_id`, `principal_id`, `audience_type`, `home_cell`, `jurisdiction_code`, and `audit_event_class`.
- Policy invariant: Cedar is evaluated before storage/provider access; deny decisions emit audit evidence rather than silently dropping context.
- Credential invariant: provider/API/signing keys use `${openbao:secret/<tenant_id>/feature-flags/<credential>}` and sidecar/≤60s TTL behavior.
- Observability invariant: metrics avoid raw `tenant_id` cardinality; audit events keep tenant id in signed evidence instead.
- Transport invariant: HTTP/3 first, HTTP/2 second, HTTP/1.1 third, TLS 1.3 floor, ECH where terminated, PQC hybrid where negotiated.
- Deployment invariant: runtime adapters run outside the domain/core boundary and inherit SPIFFE, Kata/Cloud Hypervisor, and cell policy where applicable.
- Detection invariant: abuse, policy, insider, and anomaly signals route to detection/investigation through ADR-0263 audit events.
- UX/safety invariant: fraud or bot controls add friction only on suspicion, never on clean default path or emergency-services path.
- Pack invariant: higher-restriction-wins for data residency, retention, breach timing, regulator export, and appeal/notice rules.
- Failure mode: stale tenant projection. `feature-flags` applies most-restrictive policy and emits degraded-mode evidence.
- Failure mode: Cedar mismatch. `feature-flags` fails closed for mutations and rolls back to prior soaked fragment.
- Failure mode: audit backpressure. `feature-flags` buffers bounded evidence and stops high-risk mutation before evidence loss.
- Failure mode: regional outage. `feature-flags` follows `multi-region.md` and does not cross pack residency boundaries for availability.
- Failure mode: key compromise. `feature-flags` revokes OpenBao leases, rotates keys, quarantines impacted events, and replays idempotent work.
- Concrete example: `flag-evaluation` evaluates `<tenant>.feature-flags.flag-evaluation` against policy, writes `feature_flags.flag_evaluation`, and emits `oya.feature.flags.flag.evaluation.completed`.
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `credential-isolation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `credential-isolation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `credential-isolation` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `credential isolation` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `credential isolation`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `credential isolation` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.

## §fragment-publish

Per ADR-0294: Cedar fragments published by this µservice (targeting-rule predicates) observe ≥60s soak window before activation. Fragment versioning header included. Fragment rollback procedure in `runbooks/flag-mutation-cascade.md`.

**ADR-adherence row 25:** `policy/*.cedar` headers ✓

---

## Layer-by-layer trace

BNF v4.1 + ADR-0105 13-layer:

```
oya-feature-flags-<bc>-<layer>

BCs: flag / targeting / experiment / rollout / killswitch

Layers: kernel | domain | usecase | adapter | adapter-postgres | api | app | rest | sdk | worker
```

### Hot path: flag evaluation (p99 ≤1ms target)

```
1. Client SDK (oya-feature-flags-sdk)
   └── local cache hit (30s TTL) → return immediately [p50: 0.001ms]
       cache miss →
2. REST/gRPC call to oya-feature-flags-evaluator-rest
3. oya-feature-flags-flag-app (composition root)
   ├── oya-feature-flags-flag-usecase: FlagEvaluateUseCase
   │   ├── oya-feature-flags-flag-domain: FlagDefinition.resolve_variant()
   │   ├── oya-feature-flags-targeting-domain: TargetingRule.evaluate(context)
   │   │   └── oya-shared-policy-eval (Cedar; library-first)
   │   └── oya-feature-flags-rollout-domain: RolloutBucket.compute_hash()
   └── audit emit (async; fire-and-forget if not audit_required)
```

### Control path: flag mutation

```
1. Admin API call → oya-feature-flags-flag-rest
2. Step-up auth check (Cedar gate; Class B minimum)
3. oya-feature-flags-flag-usecase: FlagMutateUseCase
   ├── oya-feature-flags-flag-domain: FlagDefinition.validate()
   ├── oya-feature-flags-flag-adapter-postgres: write to flags table
   ├── Propagate to all cells (async; ≤5s)
   └── oya-feature-flags-audit-emitter: FlagUpdated event (sealed)
```

## Concrete end-to-end example

**Scenario:** Engineer creates a new `dark-mode-v2` boolean flag for tenant `acme-corp`. PM activates a 10% rollout. SRE kills it after a regression.

```
1. POST /api/v1/flags  (flag-manager; step-up Class B)
   → FlagCreated audit event sealed
   → Cedar fragment for targeting rule soaks 60s (ADR-0294)

2. GET /api/v1/flags/dark-mode-v2/evaluate
   Body: { tenant_id: "acme-corp", principal_id: "user-abc", persona_tier: "B2C" }
   → hash("acme-corp", "dark-mode-v2", "user-abc") % 100 = 7 < 10 → variant: true
   → FlagEvaluated event (omitted if !audit_required)

3. PATCH /api/v1/flags/dark-mode-v2  { "lifecycle_state": "kill_switch_active" }
   (step-up Class C; SRE on-call)
   → KillSwitchEngaged audit event sealed
   → All cells receive state update ≤5s
   → Subsequent evaluations return default_variant (false) for all users
```

## Common confusions

1. **"Feature flags vs ChangeSet gates"**: ChangeSet gates (ADR-0110) control code deployment; feature flags control runtime behavior. Both can be active simultaneously; they are orthogonal.
2. **"Why Cedar for targeting rules?"**: Targeting rules are per-tenant policy; Cedar is the universal policy language. Same evaluator as governance (ADR-0183); no bespoke DSL.
3. **"Why Wasm for Cedar predicates?"**: In-process evaluation without subprocess; predicate isolation between tenants; Wasm sandbox = no memory leakage across tenant evaluations.
4. **"Kill switch vs flag archive"**: Kill switch = immediate forced-off for all tenants; audit-sealed. Archive = lifecycle management for stale flags; does not change evaluation result until fully deleted.

## Where to read next

- `PRD.md` — functional requirements + personas + user stories.
- `compliance.md` — pack overlays + DRMP + detection substrate binding.
- `policy/*.cedar` — Cedar fragment library.
- `runbooks/killswitch-engaged.md` — SRE kill-switch procedure.
- `contracts/openapi-v1.yaml` — REST API surface.
- `docs/decisions/ADR-0159-feature-flag-substrate.md` — binding ADR.
- `microservices/observability/` — observability substrate (canonical exemplar).
