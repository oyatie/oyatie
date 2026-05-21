---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-feature-flags
microservice: feature-flags
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
related_adrs: [ADR-0001, ADR-0003, ADR-0007, ADR-0009, ADR-0049, ADR-0110, ADR-0114, ADR-0128, ADR-0131, ADR-0139, ADR-0145, ADR-0157, ADR-0158, ADR-0159, ADR-0163, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/feature-flag-substrate-canonical.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-18
owner_team: axis-governance
doc_status: published
---

# PRD-feature-flags: Runtime Feature-Flag Substrate (OpenFeature-compliant)

## Purpose

The `feature-flags` µservice is oyatie's canonical runtime feature-flag substrate. It is the third orthogonal gating tier alongside:

1. ChangeSet `acceptance_status` (code-deploy gate; ADR-0110).
2. Progressive delivery (traffic-shape gate; ADR-0160 / ADR-0114).
3. **Runtime feature flag (this µservice; ADR-0159).**

It implements the OpenFeature spec server-side with per-tenant + per-persona-tier + per-cohort targeting. Same Cedar evaluator that the governance µservice uses (per ADR-0007) for predicate evaluation. Audit-chain emission per ADR-0003 on flag definition changes; per-evaluation emission for flags tagged `audit_required: true`.

## Scope

In:
- Flag definition CRUD (create / edit / archive / delete).
- Flag evaluation: boolean / string / number / JSON object variants.
- Per-tenant + per-persona-tier + per-cohort targeting via Cedar fragment predicates.
- Percentage rollout via deterministic hash on `(tenant_id, flag_key)`.
- OpenFeature gRPC + REST surface.
- Per-µservice Rust / TypeScript / Python SDKs.
- Flag lifecycle gates (release_toggle / experiment / permission_toggle / kill_switch + sunset_at).
- Audit-chain emission on definition change + per-evaluation when `audit_required: true`.
- Per-cell deployment (`active_active` per ADR-0158).

Out:
- A/B experiment statistics + winner-selection (separate `experiments` µservice future scope).
- Tenant configuration management beyond flags (separate `tenant-config` future).
- Code-deploy gating (ADR-0110 ChangeSet).
- Traffic-shape gating (ADR-0160 Flagger).

## Personas

- **Engineer.** Defines flags in code; sunset_at declared.
- **Product manager.** Evaluates experiment flags; promotes winners.
- **SRE / on-call.** Operates kill-switches for emergency disable.
- **Compliance officer.** Audits `audit_required: true` flag history for sensitive-feature toggles.

## Functional requirements

1. **F-FF-01** — Define flag with key, default variant, targeting rules, intent, sunset_at.
2. **F-FF-02** — Evaluate flag against (tenant_id, persona_tier, pack_id, cohort_ids[], user_id-hash) context in < 1 ms p99.
3. **F-FF-03** — Cedar fragment predicates evaluated against evaluation context; same evaluator as governance µservice.
4. **F-FF-04** — Percentage rollout via stable deterministic hash on (tenant_id, flag_key) → 0-99 bucket.
5. **F-FF-05** — Emit audit-chain seal on flag definition change.
6. **F-FF-06** — Emit audit-chain seal per evaluation when flag tagged `audit_required: true`.
7. **F-FF-07** — Lifecycle enforcement: release_toggle / experiment > sunset_at past due → CI red.
8. **F-FF-08** — Per-cell deployment + global replication of definitions; eventually-consistent ≤ 5 s.
9. **F-FF-09** — Client SDK with local cache + 30 s TTL.
10. **F-FF-10** — OpenFeature provider conformance.

## Non-functional requirements

- **Eval latency p99** ≤ 1 ms (cell-local).
- **Definition replication** ≤ 5 s cross-region.
- **Throughput** ≥ 100k eval/s per replica.
- **Availability** ≥ 99.99%.
- **Cache hit rate** ≥ 99% at client SDK.

### DR posture (ADR-0343)

- Service target: flag evaluation remains cell-local with RTO p99 ≤ 60s for server restart and RPO p99 ≤ 5s for flag definition replication, matching the existing definition-replication SLO.
- Compliance floors considered: HIPAA-2024 RTO 3600s/RPO 300s/multi-region true, PCI-DSS-L1-v4 RTO 86400s/RPO 3600s, SOC2-T2 RTO 14400s/RPO 900s, and EU-AI-ACT-2024-HIGH-RISK RTO 1800s/RPO 300s/multi-region true for flagged automated-decision controls. The effective service target is 60s/5s; multi-region active-active is required by the existing cell model.
- Failover runbook reference: `runbooks/killswitch-engaged.md`, `runbooks/flag-evaluation-regression.md`, and `runbooks/audit-replay.md`.
- Multi-region posture: active-active per cell with local SDK cache fallback; definitions replicate asynchronously within the 5s p99 lag budget and each sovereign pack resolves inside its declared cell.
- Tenant-visible behavior: a tenant retains deterministic last-known/default flag behavior during cell failover, so an emergency kill-switch degrades toward safety instead of disappearing.

### Capacity model (ADR-0340)

- Per-tenant baseline: 0.1 vCPU/128MiB server capacity, 10MiB flag-definition storage, 100MiB SDK/evaluation cache, and one streaming SDK connection class per active application.
- Scaling dimension: `eval_per_second`, `flag_definition_count`, `targeting_rule_complexity`, `sdk_stream_connection`, and `audit_required_eval` size the evaluator and audit replay lanes.
- Cell placement class: Tier-2 minimum from `manifest.json` `cell_eligibility.tier_min`; ADR-0338 Tier-1/Kata applies to privileged kill-switch mutation workers while hot evaluation replicas stay in the low-latency runtime tier declared by the cell overlay.
- Autoscaling boundaries: minimum two evaluator replicas per cell, maximum 100 evaluator replicas per cell before tenant rate limits and SDK cache TTLs absorb additional load; audit-required tenants get a separate replay queue.
- Tenant load profile served: steady high-QPS SDK evaluation, brief rollout storms, and low-frequency compliance audits stay isolated from mutation and kill-switch control paths.

### Sustainability + cost attribution (ADR-0344)

- Every flag change, evaluation for `audit_required: true`, kill-switch invocation, experiment conclusion, pack override, rollout advance, and rollback row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours`.
- Carbon-aware provider routing: no for online flag evaluation or kill-switch paths because latency and safety dominate; yes for offline experiment aggregation, replay, and attribution jobs when ADR-0343 floors remain satisfied.
- Tenant cost transparency surface: the flag audit ledger, experiment attribution dashboard, and FinOps portal expose per-flag evaluation volume, audit replay cost, cell, and compliance_pack.
- Regulatory driver: CSRD, SB-253, and SEC climate-disclosure exports need auditable energy/cost evidence for automated decision flags; aggregate-only emissions would not prove which tenant or pack produced the workload.

### API versioning posture (ADR-0342)

- Public API version model: OpenFeature REST/gRPC, AsyncAPI, and proto contracts use the YYYY-MM-DD carrier triplet: `Oyatie-API-Version: <date>`, `/api/feature-flags/<date>/...`, and proto3 `api_version` fields.
- SDK semver model: Rust/TypeScript/Python/OpenFeature SDKs publish `major.minor.patch`; semver major aligns with breaking changes to a supported date-versioned contract.
- Support window: last N=3 public contract dates are supported for at least 180 days.
- Per-tenant pinning: yes for server-side providers and SDK clients, because rollouts can span application release windows.
- Internal-mesh exemption: yes; governance and progressive-delivery direct gRPC keep ADR-0145 behavior while the public OpenFeature surface remains date-versioned.

## Architecture

- Layers 1-7 per ADR-0105: kernel-domain-usecase-adapter trio + REST/gRPC surface + composition root.
- Storage: PostgreSQL backing for flag definitions; per-cell with cross-region async replication via Patroni.
- Cache: in-process at client SDK + Envoy cache at server side.
- Multi-region: `active_active` per cell (ADR-0158).
- Audit-chain: emission adapter calls audit-chain µservice.
- Cedar: predicate evaluator shared with governance µservice.

## Threat model summary (full at threat-model.md)

- Flag tampering threat (privileged-action gate via Cedar). Mitigation: Cedar-gated CRUD + audit-chain seal + flag-key reservation policy.
- Cohort PII leak through evaluation context. Mitigation: evaluation context fields whitelisted; user_id always hashed.
- Cross-tenant flag visibility. Mitigation: per-tenant Cedar scope on read API.

## Compliance

- SOC 2 CC4 (monitoring) + CC6.1 (logical access).
- ISO 27001 A.5.28 + A.8.15.
- GDPR Art 22 (automated decision-making) — flagged automated decisions are loggable.
- HIPAA §164.312(b) — audit-required flags emit per-evaluation seal.

## Failure modes (full at failure-modes.md)

- **Flag-server unavailable** → SDK returns last-known + default; alert.
- **Replication lag** → tenant sees flag change with ≤ 5 s delay; acceptable per SLO.
- **Audit-chain unavailable** → flag eval continues; emission retried; degraded-mode counter incremented.
- **Cedar predicate eval error** → falls back to default variant; predicate-error counter incremented; on-call paged on > 1% rate.

## Observability + SLOs

`slos/feature-flags.openslo.yaml`:

- `eval-success-rate` ≥ 99.99%.
- `eval-latency-p99` ≤ 1 ms.
- `definition-replication-lag-p99` ≤ 5 s.
- `flag-lifecycle-overdue-count` = 0 (CI-enforced).

## Status

Skeleton PRD shipped 2026-05-18 alongside ADR-0159. Full IP pack lands in stacked PR.
