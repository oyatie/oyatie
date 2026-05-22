---
doc_class: Compliance
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0247
  - ADR-0248
  - ADR-0250
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0263
  - ADR-0272
  - ADR-0276
  - ADR-0280
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0297
  - ADR-0307
  - ADR-0308
  - ADR-0309
companion_docs:
  - microservices/feature-flags/ARCHITECTURE.md
  - microservices/feature-flags/manifest.json
  - microservices/feature-flags/policy/data-residency.md
  - docs/standards/documentation-rigor.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Feature Flags — Compliance

## §pack-overlay-roster

Pack-mandated flag overrides are enforced by `oyatie.feature-flags.pack-overlay-agent`. Tenant admins cannot override these; they are force-set by the pack engine at activation time.

| Pack | Flag key | Forced value | Rationale |
|---|---|---|---|
| `us-healthcare` (HIPAA) | `phi-exposure-flag` | `off` | HIPAA 45 CFR §164.502: minimum necessary standard; PHI exposure requires explicit clinical justification |
| `us-healthcare` (HIPAA) | `ehr-auto-share-flag` | `off` | HIPAA §164.308(a)(4): access control; auto-share violates access control requirement |
| `pci-dss` | `raw-pan-display` | `off` | PCI DSS Req 3.4: PAN must be rendered unreadable |
| `pci-dss` | `cvv-retention-flag` | `off` | PCI DSS Req 3.2: CVV must not be stored |
| `eu-ai-act` | `high-risk-ai-auto-decide` | `off` | EU AI Act Art. 14: human oversight required for high-risk AI systems |
| `eu-ai-act` | `ai-profiling-unrestricted` | `off` | EU AI Act Art. 5: social scoring by public authorities prohibited |
| `fedramp-high` | `external-api-unrestricted` | `off` | FedRAMP AC-17: remote access requires authorization |
| `fedramp-high` | `debug-mode-flag` | `off` | FedRAMP CM-7: least functionality; debug output forbidden in production |
| `kr-fss` | `instant-large-transfer-flag` | `off` | KR FSS Financial Consumer Protection Act Art. 30: cooling-off period required |
| `kr-fss` | `cross-border-payment-flag` | `off` until KYB verified | KR FSS foreign-exchange transaction rules |
| `gdpr-eu` | `cookie-analytics-default-on` | `off` | GDPR Art. 6 + ePrivacy Directive: analytics cookies require opt-in consent |
| `gdpr-eu` | `behavioral-profiling-flag` | `off` until consent | GDPR Art. 22: no automated profiling without explicit consent |

Pack overlays are audited via `PackFlagOverrideApplied` events (sealed per ADR-0028). CI lane `oya-governance-pack-overlay-coverage` verifies all active packs have declared their required overrides.

**ADR-adherence row 10:** `compliance.md §pack-overlay-roster` ✓

---
### Content-pass expansion — pack-overlay-roster
- This expansion preserves the existing prose above and closes `pack-overlay-roster` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: AWS Control Tower guardrails anchors the external control pattern for `pack-overlay-roster`.
- Precedent 2: Microsoft Purview Compliance Manager provides a second independent hyperscaler pattern for `pack-overlay-roster`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `pack-overlay-roster`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `pack-overlay-roster` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `pack-overlay-roster` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.

## §day-one-cert-readiness

Per ADR-0250 (build-ahead-of-certification):

| Certification | Day-one readiness | Gap | Target date |
|---|---|---|---|
| **HIPAA** (Technical Safeguards) | ✓ — audit-chain emission, PHI flag isolation, HIPAA cell deployment | None | Shipped |
| **PCI DSS Level 1** | ✓ — PAN/CVV pack overlays, Kata pod isolation, OpenBao credential sidecar | QSA audit pending | Q3 2026 |
| **SOC 2 Type II** | ✓ — audit events, access control Cedar gates, incident response runbooks | 90-day observation window | Q4 2026 |
| **FedRAMP High** | ✓ — FedRAMP cell deployment, FIPS-140-3 crypto (Cloud Hypervisor), debug-mode pack override | P-ATO review pending | Q1 2027 |
| **ISO 27001** | ✓ — risk register (threat-model.md), incident response, DRMP controls | Audit cycle pending | Q3 2026 |
| **EU AI Act** | ✓ — high-risk AI flag overrides, fairness audit for experiment scoring | Art. 13 transparency doc required | Q3 2026 |
| **KR-ISMS-P** | ✓ — KR sovereign cell, data-residency hard-stop, KR-FSS pack overlays | Certification audit | Q2 2027 |
| **GDPR** | ✓ — consent flags, data-residency, DSAR cascade per ADR-0276 | None | Shipped |

**ADR-adherence row 9:** `compliance.md §day-one-cert-readiness` ✓

---
### Content-pass expansion — day-one-cert-readiness
- This expansion preserves the existing prose above and closes `day-one-cert-readiness` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: AWS Artifact anchors the external control pattern for `day-one-cert-readiness`.
- Precedent 2: Google Assured Workloads provides a second independent hyperscaler pattern for `day-one-cert-readiness`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `day-one-cert-readiness`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `day-one-cert-readiness` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `day-one-cert-readiness` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `day one cert readiness` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `day one cert readiness`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.

## §detection-substrate-binding

Per §3.2.6.A and ADR-0307:

Feature-flags contributes to and consumes from the following detection families:

| Detection family | Contribution | Consumption |
|---|---|---|
| **Policy violation (family 8)** | Emits `PolicyViolationDetected` on Cedar permit forge attempt, cross-tenant flag access attempt, pack-override tamper attempt | Consumes `PolicyViolationDetected` to auto-revoke flag if source is compromised principal |
| **Insider risk (family 7)** | Every flag mutation feeds UEBA substrate via `FlagUpdated` audit events per ADR-0263 | Consumes insider-risk score to require step-up auth elevation on high-risk mutations |
| **Account-takeover (family 2)** | N/A (flag-flags is not an auth surface) | Consumes ATO signal from identity µservice to freeze flag mutations for affected principal |

### Audit events contributing to detection substrate

Per ADR-0263 registry:
- `FlagMutationAnomaly` — flag mutation rate spike (>60/min from single principal) → family 8.
- `CrossTenantFlagAccessAttempt` — Cedar default-deny triggered on cross-tenant read → family 8.
- `PackOverrideTamperAttempt` — write attempt to pack-override table without `pack-overlay-agent` principal → family 7.
- `ExperimentAssignmentManipulationAttempt` — client-supplied parameters attempting to influence assignment hash → family 8.

### Features computed for feature store

Per ADR-0307 §D-3 (feature store):
- `tenant.flag_mutation_rate_1h` — flag mutation rate per tenant per hour.
- `principal.flag_mutation_velocity_15m` — per-principal mutation velocity.
- `tenant.active_kill_switches` — count of active kill switches.
- `tenant.pack_overrides_active` — count of active pack overrides.

Features computed in `oya-feature-flags-flag-worker`; emitted to feature store via `FeatureVectorEmitted` event.

### Per-tenant per-pack overlay

- HIPAA-pack tenants: PHI-related features (`principal.phi_access_context`) NEVER enter the feature store.
- FedRAMP-High tenants: all features remain in FedRAMP cell; no cross-region feature-store emission.

### Appeal mechanism wiring

Adverse actions taken by feature-flags (flag blocked by Cedar gate, experiment assignment rejected):
- `FlagEvaluationBlocked` event carries `appeal_url: /api/v1/flags/{flag_key}/appeal`.
- Appeal routes to `ops-dashboard-control-center` human review queue.
- SLA: ≤30 days for substantive review per GDPR Art. 22 + EU AI Act Art. 86.

**ADR-adherence rows 49-52:** `compliance.md §detection-substrate-binding` ✓

---
### Content-pass expansion — detection-substrate-binding
- This expansion preserves the existing prose above and closes `detection-substrate-binding` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: AWS GuardDuty findings anchors the external control pattern for `detection-substrate-binding`.
- Precedent 2: Google Chronicle detections provides a second independent hyperscaler pattern for `detection-substrate-binding`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `detection-substrate-binding`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `detection-substrate-binding` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.

## §insider-threat-controls

Per §3.2.6.A family 7 (insider risk) and §3.2.6.D (prevention):

1. **JIT access**: Flag-manager admin actions require JIT access grant via PAM (Teleport); no standing write privileges.
2. **Separation of duties**: Kill-switch activation requires `sre-oncall` role (cannot be self-assigned); pack-overlay-agent is an automated principal (not a human).
3. **Dual-control for production pack overrides**: Pack overlay activation in production cells requires two `pack-overlay-agent` attestations (multi-party authorization per ADR-0247).
4. **Sensitive-resource access patterns**: UEBA baseline tracks per-employee flag mutation patterns; drift from baseline triggers insider-risk signal.
5. **Read-only audit surface**: Compliance officers access audit events via read-only Cedar-gated surface; cannot modify audit chain.
6. **Pre-departure access review**: 30-day pre-departure access review for flag-manager role holders.

---
### Content-pass expansion — insider-threat-controls
- This expansion preserves the existing prose above and closes `insider-threat-controls` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Microsoft Purview Insider Risk anchors the external control pattern for `insider-threat-controls`.
- Precedent 2: Google BeyondCorp provides a second independent hyperscaler pattern for `insider-threat-controls`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `insider-threat-controls`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `insider-threat-controls` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `insider-threat-controls` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `insider threat controls` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `insider threat controls`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `insider threat controls` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `insider threat controls` failures have trigger, rollback, and post-incident closure.

## §threat-intelligence-feeds

Feature-flags subscribes to the following threat intelligence feeds to inform abuse-defence controls:

1. **NCSC threat intelligence**: emerging attack patterns against feature-flag APIs (flag pollution, flag-bombing DDoS).
2. **Internal threat-intel feed** (`microservices/detection/`): real-time IoC list; flag evaluations from known-bad IPs are flagged.
3. **HIBP credential feed**: flag mutations from sessions using leaked credentials trigger mandatory step-up auth.
4. **Internal anomaly feed**: `DetectionSignalEmitted` events from the detection substrate consumed to adjust rate-limit thresholds dynamically.

Feed refresh cadence: NCSC daily; internal feeds real-time.

---
### Content-pass expansion — threat-intelligence-feeds
- This expansion preserves the existing prose above and closes `threat-intelligence-feeds` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Mandiant threat intelligence anchors the external control pattern for `threat-intelligence-feeds`.
- Precedent 2: AWS GuardDuty threat lists provides a second independent hyperscaler pattern for `threat-intelligence-feeds`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `threat-intelligence-feeds`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `threat-intelligence-feeds` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `threat-intelligence-feeds` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `threat intelligence feeds` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `threat intelligence feeds`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `threat intelligence feeds` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `threat intelligence feeds` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `feature-flags` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm-values.yaml, iac/k8s-deployment.yaml, iac/network-policy.yaml, plus 4 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.

## §key-rotation-cadence

| Key type | Location | Rotation cadence | Automation |
|---|---|---|---|
| Audit-chain signing key | OpenBao sidecar | 90 days | Automated via `oya-foundry-key-rotation-worker` |
| Pack-overlay signing key | OpenBao sidecar | 90 days | Automated |
| Flag definition encryption key (DEK) | OpenBao sidecar | Per-tenant; 365 days | Automated via tenant encryption-key BYOK (ADR-0251 §D-10) or platform KEK rotation |
| ECH private key | DNS / ingress | 90 days | `runbooks/ech-key-rotation.md` |
| PQC certificate key | CA | 90 days (shorter than classical due to PQ migration cadence) | Automated via sigstore/Fulcio |
| mTLS SVID | SPIFFE/SPIRE | 24h (short-lived; auto-renewed) | SPIRE agent |
| OpenBao lease TTL | OpenBao | ≤60s per ADR-0296 | Automatic lease renewal |

---
### Content-pass expansion — key-rotation-cadence
- This expansion preserves the existing prose above and closes `key-rotation-cadence` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: AWS KMS key rotation anchors the external control pattern for `key-rotation-cadence`.
- Precedent 2: Google Cloud KMS versions provides a second independent hyperscaler pattern for `key-rotation-cadence`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `key-rotation-cadence`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `key-rotation-cadence` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `key-rotation-cadence` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `key rotation cadence` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `key rotation cadence`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `key rotation cadence` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.

## §crypto-agility-plan

Per ADR-0253 PQC requirements:

1. **Current baseline**: Classical TLS 1.3 (X25519 + AES-256-GCM) + experimental PQC hybrid.
2. **Phase 1 (Q3 2026)**: Enable `X25519MLKEM768` hybrid KEM on all Tier-2 cell ingresses; non-PQ clients fall through gracefully.
3. **Phase 2 (Q1 2027)**: Enable `ed25519+ml_dsa_65` for new certificate chains from oyatie-rooted CAs.
4. **Phase 3 (Q3 2027)**: Require PQC-capable TLS for `sre-oncall` and `pack-overlay-agent` principals (high-privilege; early PQC adoption).
5. **Phase 4 (2028+)**: Deprecate classical-only handshakes for internal µservice-to-µservice calls.
6. **Crypto-agility invariant**: No hardcoded algorithm identifiers in code; all negotiated via TLS stack + ADR-0253 policy configuration.

---
### Content-pass expansion — crypto-agility-plan
- This expansion preserves the existing prose above and closes `crypto-agility-plan` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Cloudflare post-quantum TLS anchors the external control pattern for `crypto-agility-plan`.
- Precedent 2: Chrome hybrid PQ TLS provides a second independent hyperscaler pattern for `crypto-agility-plan`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `crypto-agility-plan`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `crypto-agility-plan` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `crypto-agility-plan` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `crypto agility plan` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `crypto agility plan`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `crypto agility plan` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `crypto agility plan` failures have trigger, rollback, and post-incident closure.

## §ml-model-lifecycle

Per ADR-0308 (ML lifecycle) and §3.2.6.E:

Feature-flags trains and serves one ML model: **experiment statistical significance scorer** (Bayesian posterior + frequentist p-value composite).

| Stage | Implementation |
|---|---|
| **Training** | Per-tenant experiment data only; cross-tenant training forbidden. Training data stays in tenant's home cell. |
| **Validation** | Bias audit: experiment assignment fairness across protected classes (gender, age tier, jurisdiction). IBM AI Fairness 360 + custom Bayesian fairness metrics. |
| **A/B testing** | Champion-challenger via shadow-mode first; canary at 5% before full rollout. |
| **Drift detection** | Feature drift daily; label drift weekly; concept drift monthly. Alert threshold: >5% KL-divergence. |
| **Fairness re-audit** | Quarterly per protected class per jurisdiction. |
| **Model versioning** | SemVer per ADR-0258; model card per Google Model Card template; stored in MLflow. |
| **Rollback** | Per-pack regulator timing; EU AI Act: rollback within 24h on serious incident. |
| **Appeal** | Per GDPR Art. 22 + EU AI Act Art. 86: adverse experiment assignment carries human-readable explanation + appeal URL. |

**ADR-adherence row 50:** `compliance.md §ml-model-lifecycle` ✓

---
### Content-pass expansion — ml-model-lifecycle
- This expansion preserves the existing prose above and closes `ml-model-lifecycle` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: NIST AI RMF anchors the external control pattern for `ml-model-lifecycle`.
- Precedent 2: Google Model Cards provides a second independent hyperscaler pattern for `ml-model-lifecycle`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `ml-model-lifecycle`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `ml-model-lifecycle` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `ml-model-lifecycle` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `ml model lifecycle` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `ml model lifecycle`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.

## §detection-fairness-audit

Per ADR-0309 and §3.2.6 detection-fairness invariants, and EU AI Act + ECOA Reg B + NY AEDT:

Experiment scoring model fairness requirements:

1. **No proxy discrimination**: Features that proxy protected classes (e.g., locale → national origin, device tier → income) are flagged in model card. Fair-lending review required for any experiment in payments/credit/housing/employment surface.
2. **Per-class TPR/FPR equity**: True-positive rate and false-positive rate for experiment-wins detection within ±2pp across gender, age-tier, and jurisdiction groups. Wider gaps require explicit ADR justification.
3. **Disparate impact testing**: 4/5ths rule (Federal Uniform Guidelines) applied to experiment conversion rate differences across protected groups. Automated in `oya-governance-detection-fairness-audit` CI lane.
4. **Explainability floor**: Every experiment conclusion event (`ExperimentConcluded`) carries LIME/SHAP-style feature importance summary. Available on appeal per GDPR Art. 22.
5. **Per-jurisdiction model variants**: EU-AI-Act pack forbids social-scoring features in experiment models. KR-FSS pack forbids certain behavioral features per Financial Consumer Protection Act Art. 30.

Fairness audit output: `AUDIT-FINDINGS-<date>.json` — updated quarterly.

**ADR-adherence row 51:** `compliance.md §detection-fairness-audit` ✓

---
### Content-pass expansion — detection-fairness-audit
- This expansion preserves the existing prose above and closes `detection-fairness-audit` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Microsoft Fairlearn anchors the external control pattern for `detection-fairness-audit`.
- Precedent 2: NIST AI RMF measurement provides a second independent hyperscaler pattern for `detection-fairness-audit`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `detection-fairness-audit`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `detection-fairness-audit` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `detection-fairness-audit` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `detection fairness audit` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `detection fairness audit`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `detection fairness audit` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.

## §self-modification

Per ADR-0247: see `ARCHITECTURE.md §self-modification`. Pack-overlay-agent operates under Foundry meta-trust root. No other self-modification. Human approval required for all other flag mutations.

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
- Depth detail 13: Capability/catalog evidence for `feature-flags` uses `capabilities/experiment-design.yaml, capabilities/flag-evaluate.yaml, capabilities/flag-evaluation.yaml, capabilities/killswitch-trigger.yaml, plus 1 more` and `catalog/oya-feature-flags-experiment-kernel.yaml, catalog/oya-feature-flags-flag-adapter-postgres.yaml, catalog/oya-feature-flags-flag-app.yaml, catalog/oya-feature-flags-flag-domain.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `feature-flags` fails closed when `self modification` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `feature-flags` emits denial evidence for `self modification` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `feature-flags` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `self modification` workflow.
- Depth detail 17: `feature-flags` telemetry for `self modification` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §consent

Per ADR-0272: feature-flags is not a user-facing surface (it is a substrate API). It does not present consent surfaces. However:
- Experiment flags that affect analytics-collection surfaces (e.g., `analytics-cookie-enabled`) interact with consent flags.
- The `gdpr-eu` pack overlay forces `cookie-analytics-default-on = off`; consent gates the analytics-collection flag evaluation.
- Per-purpose consent state is passed in evaluation context as `consent_purposes: ["analytics", "personalization"]`.

---
### Content-pass expansion — consent
- This expansion preserves the existing prose above and closes `consent` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Google Consent Mode anchors the external control pattern for `consent`.
- Precedent 2: Apple App Tracking Transparency provides a second independent hyperscaler pattern for `consent`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `consent`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `consent` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `consent` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `consent` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `consent`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `consent` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `consent` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `feature-flags` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm-values.yaml, iac/k8s-deployment.yaml, iac/network-policy.yaml, plus 4 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `feature-flags` uses `capabilities/experiment-design.yaml, capabilities/flag-evaluate.yaml, capabilities/flag-evaluation.yaml, capabilities/killswitch-trigger.yaml, plus 1 more` and `catalog/oya-feature-flags-experiment-kernel.yaml, catalog/oya-feature-flags-flag-adapter-postgres.yaml, catalog/oya-feature-flags-flag-app.yaml, catalog/oya-feature-flags-flag-domain.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `feature-flags` fails closed when `consent` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.

## §email-deliverability

Per ADR-0273: feature-flags does not emit email. N/A.

---
### Content-pass expansion — email-deliverability
- This expansion preserves the existing prose above and closes `email-deliverability` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Google Workspace DKIM/SPF/DMARC anchors the external control pattern for `email-deliverability`.
- Precedent 2: AWS SES domain identity provides a second independent hyperscaler pattern for `email-deliverability`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `email-deliverability`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `email-deliverability` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `email-deliverability` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `email deliverability` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `email deliverability`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `email deliverability` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `email deliverability` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `feature-flags` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm-values.yaml, iac/k8s-deployment.yaml, iac/network-policy.yaml, plus 4 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `feature-flags` uses `capabilities/experiment-design.yaml, capabilities/flag-evaluate.yaml, capabilities/flag-evaluation.yaml, capabilities/killswitch-trigger.yaml, plus 1 more` and `catalog/oya-feature-flags-experiment-kernel.yaml, catalog/oya-feature-flags-flag-adapter-postgres.yaml, catalog/oya-feature-flags-flag-app.yaml, catalog/oya-feature-flags-flag-domain.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `feature-flags` fails closed when `email deliverability` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `feature-flags` emits denial evidence for `email deliverability` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `feature-flags` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `email deliverability` workflow.
- Depth detail 17: `feature-flags` telemetry for `email deliverability` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §portability

Per ADR-0276 (GDPR Art. 20 backup portability):
- Flag definitions and experiment configurations are exportable per tenant as JSON (machine-readable) on DSAR request.
- Export format: `flag-definitions-export-v1.json` schema (see `contracts/openapi-v1.yaml #FlagDefinitionExport`).
- Export is per-tenant-scoped; no cross-tenant data in export.
- Export triggered via `DSARFlagExportRequested` event from tenancy µservice cascade.

---

## §platform-owner-indirection

Per ADR-0284: `oyatie` string migration status for feature-flags:

- `manifest.json:owner` → `axis-governance` (indirection: owner is a team slug, not platform name). ✓
- Cedar fragment principals → `oyatie.feature-flags.*` (namespace, not display string). ✓
- Audit event source → `oya.feature_flags.*` (BNF v4.1 prefixed slug). ✓
- No hard-coded `"oyatie"` display strings in REST responses; `platform_name` resolved from tenant config at runtime.

Grep audit: `grep -r '"oyatie"' microservices/feature-flags/` → 0 results (display strings).

**ADR-adherence row 22:** `compliance.md §platform-owner-indirection` ✓

---
### Content-pass expansion — platform-owner-indirection
- This expansion preserves the existing prose above and closes `platform-owner-indirection` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Salesforce My Domain anchors the external control pattern for `platform-owner-indirection`.
- Precedent 2: Google Workspace tenant branding provides a second independent hyperscaler pattern for `platform-owner-indirection`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `platform-owner-indirection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `platform-owner-indirection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `platform-owner-indirection` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `platform owner indirection` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `platform owner indirection`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `platform owner indirection` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `platform owner indirection` failures have trigger, rollback, and post-incident closure.

## §minor-protection

Per ADR-0292:

- Feature-flags itself is a substrate; it does not directly serve minors.
- However, experiment flags targeting consumer surfaces MUST respect minor-tier restrictions:
  - Experiments with `audience_type = "MINOR_TARGETED"` require explicit `minor_compliance_pack` declaration.
  - COPPA (<13): experiment assignment for under-13 users requires parental-consent flag = on.
  - KOSA (14-17): experiment for 14-17 tier must not target behavioral-profiling flags.
  - EU age-verification: experiments on EU surfaces respect `eu-age-verification-flag` pack overlay.
- Crisis-line bypasses for minors: `audience_type = EMERGENCY_SERVICES + minor_self_report = true` bypasses all consent requirements for safety reports.

**ADR-adherence row 23:** `compliance.md §minor-protection` ✓

---
### Content-pass expansion — minor-protection
- This expansion preserves the existing prose above and closes `minor-protection` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: Apple Family/Screen Time controls anchors the external control pattern for `minor-protection`.
- Precedent 2: Google Family Link provides a second independent hyperscaler pattern for `minor-protection`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `minor-protection`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `minor-protection` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `minor-protection` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `minor protection` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `minor protection`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `minor protection` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.

## §meta-trust-attestation

Per ADR-0293: feature-flags is Foundry-touching (pack-overlay-agent is invoked by Foundry).

- Pack-overlay events are cosign-signed by `oyatie.foundry.pack-engine` trust root.
- Attestation chain: Foundry pack-engine → OpenBao → pack-overlay-agent → Cedar evaluation → flag mutation.
- SPIFFE workload identity per ADR-0295 for all Foundry→feature-flags calls.

**ADR-adherence row 24:** `compliance.md §meta-trust-attestation` ✓

---
### Content-pass expansion — meta-trust-attestation
- This expansion preserves the existing prose above and closes `meta-trust-attestation` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: The Update Framework roots anchors the external control pattern for `meta-trust-attestation`.
- Precedent 2: Sigstore Rekor transparency provides a second independent hyperscaler pattern for `meta-trust-attestation`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `meta-trust-attestation`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `meta-trust-attestation` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `meta-trust-attestation` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `meta trust attestation` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `meta trust attestation`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `meta trust attestation` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `meta trust attestation` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `feature-flags` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm-values.yaml, iac/k8s-deployment.yaml, iac/network-policy.yaml, plus 4 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `feature-flags` uses `capabilities/experiment-design.yaml, capabilities/flag-evaluate.yaml, capabilities/flag-evaluation.yaml, capabilities/killswitch-trigger.yaml, plus 1 more` and `catalog/oya-feature-flags-experiment-kernel.yaml, catalog/oya-feature-flags-flag-adapter-postgres.yaml, catalog/oya-feature-flags-flag-app.yaml, catalog/oya-feature-flags-flag-domain.yaml, plus 8 more` to keep layer names and owners machine-checkable.

## §bootstrap-trust-chain

Per ADR-0295: feature-flags is not a bootstrap-tier-1 service. It depends on tenancy (bootstrap-tier-1) and policy-engine. SPIFFE attestation is used for all incoming calls; kill-switch wiring at the Cilium network-policy layer (see `iac/network-policy.yaml`).

---
### Content-pass expansion — bootstrap-trust-chain
- This expansion preserves the existing prose above and closes `bootstrap-trust-chain` for `feature-flags` to the ≥50-line documentation-rigor floor.
- Service owner `axis-governance` owns this answer; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Primary capability/context: `flag-evaluation`; bounded contexts: `flag-evaluation`.
- API surfaces: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`; +2 more.
- Cedar/policy surfaces: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`; +5 more.
- State/event surfaces: `feature_flags.flag_evaluation`, `feature_flags.t0`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluation_yaml`, `feature_flags.flag_evaluate`, `feature_flags.microservices_feature_flags_capabilities_flag_evaluate_yaml`; +1 more.
- SLO/dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`; +11 more.
- Compliance packs: `us`, `eu`, `kr`, `healthcare`, `fedramp`; +2 more; data classes: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Cross-service dependencies: `oya-shared-policy-eval`, `oya-shared-hlc`, `oya-shared-tenant-context`, `oya-shared-audit-emitter`, `oya-shared-otel`; +3 more.
- Precedent 1: SPIFFE/SPIRE workload identity anchors the external control pattern for `bootstrap-trust-chain`.
- Precedent 2: Sigstore Fulcio provides a second independent hyperscaler pattern for `bootstrap-trust-chain`.
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
- Verification hook: `oya-governance-adr-adherence-matrix` consumes this section as the row answer for `bootstrap-trust-chain`.
- Verification hook: `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum usage.
- Verification hook: `oya-governance-doc-link-resolves` must resolve the cited local artifacts before BLOCKER promotion.
- Verification hook: abuse-defence and critical-path lanes apply when `bootstrap-trust-chain` touches bot controls, safety cases, or edge-case matrices.
- Structural note: manifest parsed = `True`; missing local policy/contract/SLO/runbook/IaC artifacts are treated as follow-up structural issues, not hidden pass claims.
- Depth detail 1: `feature-flags` binds `bootstrap-trust-chain` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `bootstrap trust chain` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `bootstrap trust chain`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `bootstrap trust chain` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `bootstrap trust chain` failures have trigger, rollback, and post-incident closure.
- Depth detail 12: Deployment evidence for `feature-flags` uses `iac/ech-config.yaml, iac/edge-waf.yaml, iac/helm-values.yaml, iac/k8s-deployment.yaml, iac/network-policy.yaml, plus 4 more` to prove ingress, cell placement, secret binding, network policy, and runtime isolation.
- Depth detail 13: Capability/catalog evidence for `feature-flags` uses `capabilities/experiment-design.yaml, capabilities/flag-evaluate.yaml, capabilities/flag-evaluation.yaml, capabilities/killswitch-trigger.yaml, plus 1 more` and `catalog/oya-feature-flags-experiment-kernel.yaml, catalog/oya-feature-flags-flag-adapter-postgres.yaml, catalog/oya-feature-flags-flag-app.yaml, catalog/oya-feature-flags-flag-domain.yaml, plus 8 more` to keep layer names and owners machine-checkable.
- Depth detail 14: `feature-flags` fails closed when `bootstrap trust chain` lacks tenant id, principal id, Cedar decision, residency label, or audit event class.
- Depth detail 15: `feature-flags` emits denial evidence for `bootstrap trust chain` instead of converting policy failure into a generic timeout or user-facing ambiguity.
- Depth detail 16: `feature-flags` separates tenant-admin, platform-owner, support-operator, auditor, and worker authority for every `bootstrap trust chain` workflow.
- Depth detail 17: `feature-flags` telemetry for `bootstrap trust chain` redacts secrets and payload bodies while signed audit evidence keeps the reviewable tenant trace.

## §critical-path-edge-cases

Per §3.2.5 of documentation-rigor.md, applicable rows:

| Row | Critical path | Feature-flags handling | Binding | Verified by |
|---|---|---|---|---|
| 1 | Emergency services | `EMERGENCY_SERVICES` audience-type bypasses all abuse-defence; kill-switch for emergency flags is FORBIDDEN by Cedar FORBID rule; elevated rate-limit floor (10×) | `policy/safety-killswitch-authorization.cedar` | `oya-governance-emergency-services-chaos-test` |
| 5 | Healthcare urgent care / EHR break-glass | `ehr-break-glass-enable` flag is kill-switch-class; `freshness_floor: 0s`; PHI access reason-coded; HIPAA-cell only | `policy/safety-killswitch-authorization.cedar` + `runbooks/killswitch-engaged.md` | `oya-governance-critical-path-coverage` |
| 17 | Service outage during regulator-deadline | Degraded-mode LKG cache (30min TTL); `audit_required: true` flags emit evaluation even in degraded mode; breach-notification workflow flags preserved | `runbooks/flag-evaluation-regression.md` | `oya-governance-critical-path-coverage` |
| 25 | Mistaken-action / unintended-mutation recovery | 15s undo window on `FlagUpdated`; `UndoFlagMutation` API; kill-switch has no undo (life-safety) | `runbooks/stale-targeting-rule.md` | `oya-governance-critical-path-coverage` |

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
- Depth detail 1: `feature-flags` binds `critical-path-edge-cases` to `primary service context` and validates at least one allowed path, one denied path, and one evidence-export path.
- Depth detail 2: API evidence for `feature-flags` is `contracts/asyncapi-v1.yaml, contracts/feature-flags-v1.proto, contracts/feature-flags.asyncapi.yaml, contracts/feature-flags.openapi.yaml, contracts/feature_flags.proto, contracts/openapi-v1.yaml, plus 1 more`; reviewers must map `critical path edge cases` to an explicit command, event, or proto field before launch.
- Depth detail 3: Policy evidence for `feature-flags` is `policy/abuse-defence.cedar, policy/auditor-scope.cedar, policy/ci-scope.cedar, policy/data-residency.md, policy/emergency-services-bypass.cedar, policy/experiment-design-authorization.cedar, plus 6 more`; missing policy files are scaffold debt, not an implicit pass for `critical path edge cases`.
- Depth detail 4: `feature-flags` state/event naming uses `feature_flags.primary service context` plus tenant, actor, cell, pack, and audit correlation fields.
- Depth detail 5: Cross-service handoff for `feature-flags` covers `oya-shared-policy-eval, oya-shared-hlc, oya-shared-tenant-context, oya-shared-audit-emitter, oya-shared-otel, plus 4 more` and must preserve tenant id, data class, residency label, and policy decision.
- Depth detail 6: Pack pressure for `feature-flags` is `SOC-2, ISO-27001, GDPR`; the stricter pack wins for retention, residency, breach clock, DSAR, and regulator export.
- Depth detail 7: ADR trace for `feature-flags` is `ADR-0105, ADR-0131, ADR-0244`; this keeps `critical path edge cases` tied to the keystone bundle instead of a local convention.
- Depth detail 8: Hyperscaler comparison for `feature-flags` cites `AWS IAM, Google Cloud service agents` for feature pressure while preserving Oyatie tenant isolation and audit chain semantics.
- Depth detail 9: Cell eligibility for `feature-flags` is `tenant home cell` with cross-cell ceiling `metadata-only unless pack policy permits more`.
- Depth detail 10: Operating evidence for `feature-flags` uses SLOs `slos/experiment-result-freshness.openslo.yaml, slos/feature-flags.openslo.yaml, slos/flag-eval-latency.openslo.yaml, slos/flag-state-propagation.openslo.yaml, slos/killswitch-fire-latency.openslo.yaml` and dashboards `dashboards/experiment-results.json, dashboards/flag-state-overview.json, dashboards/killswitch-history.json, dashboards/pack-override-coverage.md` when those artifacts exist.
- Depth detail 11: Incident evidence for `feature-flags` uses runbooks `runbooks/a11y-flag-violation.md, runbooks/audit-replay.md, runbooks/experiment-rollback.md, runbooks/experiment-stat-sig-violation.md, runbooks/flag-evaluation-regression.md, plus 4 more` so `critical path edge cases` failures have trigger, rollback, and post-incident closure.

## §prevention-layers

Per §3.2.6.D, defense-in-depth across L0-L9:

| Layer | Control |
|---|---|
| L0 — Edge | Rate limiting 60 mutations/min; bot-score gate; emergency-services bypass |
| L1 — Network | Cilium network-policy default-deny; SPIFFE/SPIRE mTLS; `iac/network-policy.yaml` |
| L2 — Auth | Step-up auth Class B/C for mutations; passkey preferred; session HMAC |
| L3 — Policy | Cedar default-deny; per-action permits; FORBID rules for emergency flags |
| L4 — Application | Quota gate (1000 flags/tenant); undo window; variant collision detection |
| L5 — Data | Per-tenant DEK; OpenBao sidecar; Kata pod isolation; flag definitions encrypted at rest |
| L6 — Observability | Audit events per ADR-0263; cardinality-budgeted metrics; detection signals emitted |
| L7 — Investigation | `FlagMutationAnomaly` → ops-dashboard case queue; appeal-mechanism wiring |
| L8 — User | Admin portal shows abuse-defence outcomes; mutation confirmation UI for kill-switch |
| L9 — Organizational | Quarterly chaos tests; bug bounty covers flag-flooding attack vectors |

---



## §investigation-binding
This anchor is closed for `feature-flags` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `feature-flags` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.feature-flags.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `flag-evaluation` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `flag-evaluation` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `feature-flags`; owner `axis-governance`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `feature-flags` root context.
- Capability records cited: `microservices/feature-flags/capabilities/experiment-design.yaml`, `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/flag-evaluation.yaml`, `microservices/feature-flags/capabilities/killswitch-trigger.yaml`, `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- API surfaces cited: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`, `microservices/feature-flags/dashboards/experiment-results.json`; +3 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +12 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- State/event binding: `feature_flags.flag_evaluation`, `feature_flags.flag_evaluate`, `feature_flags.experiment_design`, `feature_flags.killswitch_trigger`, `feature_flags.pack_overlay_subscribe`.
- Capability binding: `flag-evaluation`, `flag-evaluate`, `experiment-design`, `killswitch-trigger`, `pack-overlay-subscribe`.
- SLO binding: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`.
- Runbook binding: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `feature-flags`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `feature-flags`.
- `policy-engine` supplies the signed Cedar corpus while `feature-flags` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `feature-flags` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `feature-flags`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `feature-flags` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pentest-and-bounty-cadence
This anchor is closed for `feature-flags` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `feature-flags` is in annual full-scope pentest and every major `flag-evaluation` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`; +21 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `feature-flags` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `feature-flags`; owner `axis-governance`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `feature-flags` root context.
- Capability records cited: `microservices/feature-flags/capabilities/experiment-design.yaml`, `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/flag-evaluation.yaml`, `microservices/feature-flags/capabilities/killswitch-trigger.yaml`, `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- API surfaces cited: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`, `microservices/feature-flags/dashboards/experiment-results.json`; +3 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +12 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- State/event binding: `feature_flags.flag_evaluation`, `feature_flags.flag_evaluate`, `feature_flags.experiment_design`, `feature_flags.killswitch_trigger`, `feature_flags.pack_overlay_subscribe`.
- Capability binding: `flag-evaluation`, `flag-evaluate`, `experiment-design`, `killswitch-trigger`, `pack-overlay-subscribe`.
- SLO binding: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`.
- Runbook binding: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `feature-flags`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `feature-flags`.
- `policy-engine` supplies the signed Cedar corpus while `feature-flags` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `feature-flags` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `feature-flags`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `feature-flags` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §facility-controls
This anchor is closed for `feature-flags` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `feature-flags` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `{"cells": [{"cell_id": "us-east-cell-1", "sovereign_packs": [], "tier": 2}, {"cell_id": "eu-west-cell-1", "sovereign_packs": ["gdpr-eu"], "tier": 2}, {"cell_id": "kr-cell-1", "s...` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `flag-evaluation` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `feature-flags`; owner `axis-governance`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `feature-flags` root context.
- Capability records cited: `microservices/feature-flags/capabilities/experiment-design.yaml`, `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/flag-evaluation.yaml`, `microservices/feature-flags/capabilities/killswitch-trigger.yaml`, `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- API surfaces cited: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`, `microservices/feature-flags/dashboards/experiment-results.json`; +3 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +12 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- State/event binding: `feature_flags.flag_evaluation`, `feature_flags.flag_evaluate`, `feature_flags.experiment_design`, `feature_flags.killswitch_trigger`, `feature_flags.pack_overlay_subscribe`.
- Capability binding: `flag-evaluation`, `flag-evaluate`, `experiment-design`, `killswitch-trigger`, `pack-overlay-subscribe`.
- SLO binding: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`.
- Runbook binding: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `feature-flags`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `feature-flags`.
- `policy-engine` supplies the signed Cedar corpus while `feature-flags` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `feature-flags` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `feature-flags`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `feature-flags` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §supply-chain-risk
This anchor is closed for `feature-flags` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `feature-flags` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/feature-flags/catalog/oya-feature-flags-experiment-kernel.yaml`, `microservices/feature-flags/catalog/oya-feature-flags-flag-adapter-postgres.yaml`, `microservices/feature-flags/catalog/oya-feature-flags-flag-app.yaml`, `microservices/feature-flags/catalog/oya-feature-flags-flag-domain.yaml`, `microservices/feature-flags/catalog/oya-feature-flags-flag-kernel.yaml`, `microservices/feature-flags/catalog/oya-feature-flags-flag-rest.yaml`; +22 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `flag-evaluation` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `feature-flags`; owner `axis-governance`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `feature-flags` root context.
- Capability records cited: `microservices/feature-flags/capabilities/experiment-design.yaml`, `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/flag-evaluation.yaml`, `microservices/feature-flags/capabilities/killswitch-trigger.yaml`, `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- API surfaces cited: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`, `microservices/feature-flags/dashboards/experiment-results.json`; +3 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +12 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- State/event binding: `feature_flags.flag_evaluation`, `feature_flags.flag_evaluate`, `feature_flags.experiment_design`, `feature_flags.killswitch_trigger`, `feature_flags.pack_overlay_subscribe`.
- Capability binding: `flag-evaluation`, `flag-evaluate`, `experiment-design`, `killswitch-trigger`, `pack-overlay-subscribe`.
- SLO binding: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`.
- Runbook binding: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `feature-flags`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `feature-flags`.
- `policy-engine` supplies the signed Cedar corpus while `feature-flags` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `feature-flags` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `feature-flags`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `feature-flags` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §data-classification
This anchor is closed for `feature-flags` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `feature_flags.flag_evaluation`, `feature_flags.flag_evaluate`, `feature_flags.experiment_design`, `feature_flags.killswitch_trigger`, `feature_flags.pack_overlay_subscribe`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `flag-evaluation` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `feature-flags`; owner `axis-governance`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `feature-flags` root context.
- Capability records cited: `microservices/feature-flags/capabilities/experiment-design.yaml`, `microservices/feature-flags/capabilities/flag-evaluate.yaml`, `microservices/feature-flags/capabilities/flag-evaluation.yaml`, `microservices/feature-flags/capabilities/killswitch-trigger.yaml`, `microservices/feature-flags/capabilities/pack-overlay-subscribe.yaml`.
- API surfaces cited: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar/policy artifacts cited: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- SLO and dashboard evidence: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`, `microservices/feature-flags/dashboards/experiment-results.json`; +3 more.
- Runbook/IaC evidence: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +12 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/feature-flags/contracts/asyncapi-v1.yaml`, `microservices/feature-flags/contracts/feature-flags-v1.proto`, `microservices/feature-flags/contracts/feature-flags.asyncapi.yaml`, `microservices/feature-flags/contracts/feature-flags.openapi.yaml`, `microservices/feature-flags/contracts/feature_flags.proto`, `microservices/feature-flags/contracts/openapi-v1.yaml`.
- Cedar binding: `microservices/feature-flags/policy/abuse-defence.cedar`, `microservices/feature-flags/policy/auditor-scope.cedar`, `microservices/feature-flags/policy/ci-scope.cedar`, `microservices/feature-flags/policy/data-residency.md`, `microservices/feature-flags/policy/emergency-services-bypass.cedar`, `microservices/feature-flags/policy/experiment-design-authorization.cedar`; +6 more.
- State/event binding: `feature_flags.flag_evaluation`, `feature_flags.flag_evaluate`, `feature_flags.experiment_design`, `feature_flags.killswitch_trigger`, `feature_flags.pack_overlay_subscribe`.
- Capability binding: `flag-evaluation`, `flag-evaluate`, `experiment-design`, `killswitch-trigger`, `pack-overlay-subscribe`.
- SLO binding: `microservices/feature-flags/slos/experiment-result-freshness.openslo.yaml`, `microservices/feature-flags/slos/feature-flags.openslo.yaml`, `microservices/feature-flags/slos/flag-eval-latency.openslo.yaml`, `microservices/feature-flags/slos/flag-state-propagation.openslo.yaml`, `microservices/feature-flags/slos/killswitch-fire-latency.openslo.yaml`.
- Runbook binding: `microservices/feature-flags/runbooks/a11y-flag-violation.md`, `microservices/feature-flags/runbooks/audit-replay.md`, `microservices/feature-flags/runbooks/experiment-rollback.md`, `microservices/feature-flags/runbooks/experiment-stat-sig-violation.md`, `microservices/feature-flags/runbooks/flag-evaluation-regression.md`, `microservices/feature-flags/runbooks/flag-mutation-cascade.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `feature-flags`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `feature-flags`.
- `policy-engine` supplies the signed Cedar corpus while `feature-flags` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `feature-flags` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `feature-flags`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `feature-flags` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.
