---
doc_class: COMPLIANCE
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: council-privacy + axis-foundry + ops-compliance
related_adrs: [ADR-0117, ADR-0136, ADR-0137]
---

# Compliance Mapping — foundry (consolidated)

## Regulatory frameworks in scope

| Framework | Jurisdiction | Scope of applicability |
|---|---|---|
| GDPR | EU (pack-eu) | Tenant data processing, DSR, Art.30 records |
| PIPA | KR (pack-kr; M01 launch) | Art.23 sensitive data; Art.39 cross-border |
| HIPAA | US-HC (pack-us-healthcare) | §164.312 technical safeguards; §164.316 retention |
| CCPA / CPRA | US (pack-us) | Consumer rights + opt-out |
| EU AI Act | EU (pack-eu) | High-risk AI categorisation; conformity assessment |
| KISA / K-ISMS | KR (pack-kr) | Korean cybersecurity certification |
| SOC 2 Type II | Global | Security + Availability + Confidentiality TSCs |
| ISO 27001 | Global | ISMS controls |
| FedRAMP | US gov (subsequent-to-M01-completion) | Moderate baseline |

## Control mapping (cross-BC)

| Control | GDPR | PIPA | HIPAA | EU AI Act | SOC 2 | Resident BC |
|---|---|---|---|---|---|---|
| Audit logging of all access | Art.30 | Art.29 | §164.312(b) | Art.12 | CC4.1 | evidence (canonical) |
| Encryption at rest | Art.32 | Art.29 | §164.312(a)(2)(iv) | — | CC6.7 | all BCs (per-BC adapters) |
| Encryption in transit | Art.32 | Art.29 | §164.312(e) | — | CC6.7 | all BCs (mTLS) |
| Access controls (RBAC + Cedar) | Art.32 | Art.29 | §164.312(a)(1) | Art.10 | CC6.1 | all BCs |
| DSR endpoints (access/rectification/erasure) | Art.15–17 | Art.35–37 | — | — | — | runtime+evidence (canonical) |
| Retention enforcement | Art.5(1)(e) | Art.21 | §164.316(b)(2) | — | CC4.2 | evidence (canonical) |
| High-risk AI conformity record | — | — | — | Art.12 + Annex IV | — | evidence + supervisor |
| Kill-switch / human-oversight | — | — | — | Art.14 | — | supervisor (canonical) |
| Provider risk management | Art.28 | Art.26 | §164.314(a) | Art.16 | CC9.2 | providers (canonical) |
| Eval+monitoring of model performance | — | — | — | Art.15 | — | eval (canonical) |
| Guardrail evidence of safety | — | — | — | Art.9 | — | guardrails+evidence |

## Per-BC compliance mappings

| BC | Primary compliance scope | Archive |
|---|---|---|
| runtime | Data subject rights cascade (session-state); per-tenant retention | `bc-sources/runtime/compliance.md` |
| supervisor | Human oversight (EU AI Act Art.14); supervision audit | `bc-sources/supervisor/compliance.md` |
| eval | Model-quality monitoring (EU AI Act Art.15); synthetic-PHI rule | `bc-sources/eval/compliance.md` |
| evidence | Retention; regulator-export; audit-chain | `bc-sources/evidence/compliance.md` |
| guardrails | Safety filter evidence (EU AI Act Art.9); content-safety rules | `bc-sources/guardrails/compliance.md` |
| providers | Provider risk management; credential isolation; cross-border data flow per Art.28+Art.16 | `bc-sources/providers/compliance.md` |

## Per-pack overlays

| Pack | Jurisdiction overlay | Notes |
|---|---|---|
| pack-kr | PIPA + KISA | M01 launch; OCI ap-seoul-1; cross-pack flow forbidden |
| pack-eu | GDPR + EU AI Act | Post-M01; per-DPO sign-off prior to promotion |
| pack-us | CCPA + CPRA | Post-M01 |
| pack-us-healthcare | HIPAA + state laws | 6y retention; BAA required with tenant |
| pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa | per-jurisdiction overlays | Post-M01 expansion |

## Audit cadence

- Internal: every 90 days, SOC 2 + ISO controls walked by ops-compliance.
- External: SOC 2 Type II annual audit by Big-4 firm; HIPAA assessment when
  pack-us-healthcare opens; ISO 27001 annual surveillance.
- Regulator-driven: ad-hoc per regulator-export API; EU AI Act conformity
  assessment when a capability crosses the high-risk threshold (declared
  via `evidence/regulator-export` profile).

## References

- ADR-0117: Data-residency + jurisdiction codes.
- ADR-0136 / ADR-0137: foundry topology authority.
- ADR-0028: Audit-chain Ed25519+Merkle.
- Bominal ADR-NNN: DSR cascade.
- `bc-sources/<bc>/compliance.md` — per-BC full mapping.

---



## §day-one-cert-readiness
This anchor is closed for `foundry` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `foundry` covers packs `ae`, `au`, `br`, `eu`, `in`, `jp`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +30 more.
- Example: `eval-run` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §pack-overlay-roster
This anchor is closed for `foundry` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `ae`, `au`, `br`, `eu`, `in`, `jp`; +5 more.
- Pack overlays modify Cedar fragments `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `eval-run` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §self-modification-attestation
This anchor is closed for `foundry` against ADR-0247 §D-3: Foundry-touching self-modification attestation path.

### Service-specific answer
- Foundry-touch status is determined by whether `foundry` publishes policy, contract, IaC, catalog, or generated code artifacts.
- Mutable artifacts in scope: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +38 more.
- Attestation chain: proposed artifact hash, Foundry agent SVID, two-human approval where required, 60s soak for Cedar, audit-chain seal, and rollback pointer.
- Self-modification cannot alter certification claims, ADR status, or production IaC without human approval and a promotion gate.
- Example: a generated `foundry` Cedar update for `eval-run` includes fragment hash, signer SVID, tenant/pack scope, and activation/sunset timestamps.
- Non-Foundry paths are still documented: if this service does not self-modify today, the attestation path is an explicit deny-by-default control.
- Evidence lives in audit-chain plus `AUDIT-FINDINGS-<date>.json`; failures block promotion rather than leaving a pending marker.
- The attestation follows SLSA provenance: source, build, signer, and deploy target are independently verifiable.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance levels is the reference pattern for the control shape described here.
- Precedent 2: Google Binary Authorization attestations is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §meta-trust-attestation
This anchor is closed for `foundry` against ADR-0293 §D-1: meta-trust-root chain and transparency evidence.

### Service-specific answer
- `foundry` trusts meta-trust-root only for signing policy/contract/IaC promotion attestations, not for normal tenant data access.
- Root keys use 5-of-9 Shamir across at least three jurisdictions for platform roots; tenant operational keys can use narrower tenant-controlled ceremonies.
- Attested artifacts: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +38 more.
- Every attestation records root key id, signer SVID, artifact hash, policy decision hash, and transparency-log pointer.
- Example: `eval-run` policy publish is invalid unless the meta-trust attestation references the exact fragment hash loaded by policy evaluation.
- Revocation is targeted: revoke the compromised signer/root version, quarantine affected artifact versions, replay policy evaluation with previous accepted version.
- This section prevents circular trust where the workflow being modified signs its own authorization to modify itself.
- Audit-chain and observability own verification evidence; Foundry only proposes changes.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: Sigstore Rekor transparency log is the reference pattern for the control shape described here.
- Precedent 2: The Update Framework root metadata is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §bootstrap-trust-chain
This anchor is closed for `foundry` against ADR-0295 §D-2: Tier-1 bootstrap SPIFFE attestation and kill switch.

### Service-specific answer
- Bootstrap trust applies to `foundry` control-plane deployment, CI principals, and first-run OpenBao/SPIFFE bindings.
- Stage-1 trust root is offline-rooted and time-boxed; the kill switch disables bootstrap trust after the declared window even if later stages fail.
- Workload SVIDs protect API/worker surfaces for `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- CI principals can run synthetic tests and publish evidence, but cannot read production tenant data or mint tenant-scoped credentials.
- Example: `eval-run` app pod starts only after SPIFFE identity, OpenBao policy, and Cedar CI-scope permits are all present.
- Bootstrap failures default to halt: no unauthenticated fallback and no long-lived bootstrap token.
- Evidence: sigstore/cosign attestation, audit-chain bootstrap event, branch-protection gate, and SLO smoke report.
- Tier-1 bootstrap status is listed here even for non-bootstrap services so auditors know whether the service inherits or owns the ceremony.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: SPIFFE/SPIRE workload identity is the reference pattern for the control shape described here.
- Precedent 2: AWS Nitro Enclaves attestation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §platform-owner-indirection
This anchor is closed for `foundry` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `foundry` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`; +34 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `eval-run` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.foundry.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §detection-substrate-binding
This anchor is closed for `foundry` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `foundry` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `eval-run` touches those data classes.
- Signal sources: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +34 more.
- Example event class: `oya.foundry.eval.run.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §ml-model-lifecycle
This anchor is closed for `foundry` against documentation-rigor.md §3.2.6.E: model inventory, retrain cadence and promotion gates.

### Service-specific answer
- Local ML posture: `False` for direct model use; inherited detection/intelligence models still require versioned consumption evidence.
- Model inventory key: `manifest.json:ml_models` or the Intelligence audience tag `foundry.eval-run` if models are substrate-hosted.
- Promotion gates: offline eval, bias/fairness report, drift threshold, SLO budget, rollback model id, and human approval for high-risk/adverse-action paths.
- Retraining cadence is model-specific; high-risk models require documented data cut, feature schema, holdout set, and pack-specific legal review.
- Example: `eval-run` model output is never the sole authority for a legal/financial/employment/minor-impacting decision; Cedar and human-review policies remain in control.
- Deprecated model versions sunset under ADR-0258 with traffic split, canary, rollback, and post-promotion audit.
- Model cards include intended use, non-use, data provenance, performance by segment, failure modes, and owner.
- Services without local models keep this as a negative declaration so future agents cannot silently add ML without the lifecycle gate.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: NIST AI RMF model-governance lifecycle is the reference pattern for the control shape described here.
- Precedent 2: Google Model Cards is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §detection-fairness-audit
This anchor is closed for `foundry` against documentation-rigor.md §3.2.6.E: fairness metrics, thresholds and disaggregated false-positive audit.

### Service-specific answer
- Fairness audit applies to `foundry` risk/detection decisions that affect access, ranking, safety, money, employment, health, or protected classes.
- Metrics: false-positive rate ratio, false-negative rate ratio, calibration by segment, equalized-odds gap, appeal overturn rate, and challenge-friction rate.
- Thresholds: no protected segment exceeds 1.25x baseline false-positive rate without documented mitigation and human review.
- Segments are derived from lawful, minimized attributes; `foundry` never stores protected attributes solely to make a product feature easier.
- Example: `eval-run` abuse/risk score challenge rate is compared across locale, accessibility profile, age tier, and jurisdiction pack.
- Audit cadence: every model/rule promotion, quarterly for active high-risk detectors, and after any SEV involving false positives.
- Fairness reports are retained in audit evidence; raw protected-attribute joins remain in restricted analytics cells.
- If the service has no ML, deterministic rules still get false-positive and appeal-rate monitoring.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: Microsoft Fairlearn audit pattern is the reference pattern for the control shape described here.
- Precedent 2: NIST AI RMF measurement function is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §investigation-binding
This anchor is closed for `foundry` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `foundry` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.foundry.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `eval-run` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `eval-run` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §insider-threat-controls
This anchor is closed for `foundry` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `foundry` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`; +6 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `foundry.foundry_eval` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §threat-intelligence-feeds
This anchor is closed for `foundry` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `foundry` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +22 more.
- Example: `eval-run` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §key-rotation-cadence
This anchor is closed for `foundry` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.foundry` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/foundry/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.
- Example: `eval-run` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §crypto-agility-plan
This anchor is closed for `foundry` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `foundry` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`; +22 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `eval-run` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `foundry` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `foundry` is in annual full-scope pentest and every major `eval-run` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`; +28 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `foundry` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `foundry` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `foundry` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `eval-run` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `foundry` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `foundry` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/foundry/catalog/oya-foundry-eval-eval-runner-adapter-gpu.yaml`, `microservices/foundry/catalog/oya-foundry-eval-eval-runner-adapter-s3.yaml`, `microservices/foundry/catalog/oya-foundry-eval-eval-runner-adapter.yaml`, `microservices/foundry/catalog/oya-foundry-eval-eval-runner-api.yaml`, `microservices/foundry/catalog/oya-foundry-eval-eval-runner-app.yaml`, `microservices/foundry/catalog/oya-foundry-eval-eval-runner-domain.yaml`; +34 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `eval-run` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §critical-path-edge-cases
This anchor is closed for `foundry` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `foundry` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `eval-run` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `eval-run` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `foundry` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `eval-run` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `foundry`; owner `axis-foundry`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `foundry-eval`, `foundry-evidence`, `foundry-guardrails`, `foundry-providers`, `foundry-runtime`, `foundry-supervisor`.
- Capability records cited: `microservices/foundry/capabilities/eval-eval-run.yaml`, `microservices/foundry/capabilities/eval-parity-compare.yaml`, `microservices/foundry/capabilities/eval-replay-execute.yaml`, `microservices/foundry/capabilities/evidence-evidence-pack-build.yaml`, `microservices/foundry/capabilities/evidence-evidence-query.yaml`, `microservices/foundry/capabilities/evidence-regulator-export.yaml`; +6 more.
- API surfaces cited: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar/policy artifacts cited: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- SLO and dashboard evidence: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +18 more.
- Runbook/IaC evidence: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +18 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/foundry/contracts/asyncapi/eval-eval-events.yaml`, `microservices/foundry/contracts/asyncapi/evidence-foundry-evidence-events.yaml`, `microservices/foundry/contracts/asyncapi/guardrails-decision-events.yaml`, `microservices/foundry/contracts/asyncapi/providers-provider-events.yaml`, `microservices/foundry/contracts/asyncapi/runtime-foundry-runtime-events.yaml`, `microservices/foundry/contracts/asyncapi/supervisor-foundry-supervisor-events.yaml`.
- Cedar binding: `microservices/foundry/policy/eval-auditor-scope.cedar`, `microservices/foundry/policy/eval-ci-scope.cedar`, `microservices/foundry/policy/eval-data-residency.md`, `microservices/foundry/policy/eval-dp-analysis.md`, `microservices/foundry/policy/eval-synthetic-phi-only.md`, `microservices/foundry/policy/eval-tenant-isolation.md`; +10 more.
- State/event binding: `foundry.foundry_eval`, `foundry.foundry_evidence`, `foundry.foundry_guardrails`, `foundry.foundry_providers`, `foundry.foundry_runtime`, `foundry.foundry_supervisor`.
- Capability binding: `eval-run`, `parity-compare`, `replay-execute`, `evidence-pack-build`, `evidence-query`, `regulator-export`; +6 more.
- SLO binding: `microservices/foundry/slos/eval-determinism-correctness.openslo.yaml`, `microservices/foundry/slos/eval-run-latency.openslo.yaml`, `microservices/foundry/slos/evidence-chain-integrity-correctness.openslo.yaml`, `microservices/foundry/slos/evidence-emit-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-policy-eval-latency.openslo.yaml`, `microservices/foundry/slos/guardrails-shadow-mode-false-positive-budget.openslo.yaml`; +6 more.
- Runbook binding: `microservices/foundry/runbooks/eval-clickhouse-rebalance.md`, `microservices/foundry/runbooks/eval-eval-set-rollback.md`, `microservices/foundry/runbooks/eval-baseline-output-restore.md`, `microservices/foundry/runbooks/eval-gpu-pool-rebalance.md`, `microservices/foundry/runbooks/eval-parity-regression-triage.md`, `microservices/foundry/runbooks/eval-replay-divergence-investigation.md`; +6 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `foundry`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `foundry`.
- `policy-engine` supplies the signed Cedar corpus while `foundry` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `foundry` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `foundry`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `foundry` applies the most restrictive policy and emits a degraded-mode audit event.
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
