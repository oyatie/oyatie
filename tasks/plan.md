# Implementation Plan: Hyperscaler Gap-Closure (M01-observability)

## Overview

Close 30 hyperscaler-grade gaps surfaced in the M01-observability + repo-restructure changeset before Turn 2 (15 observability IPs) begins. Authored in four vertical slices at audit-grade depth, satisfying SOC 2 Type 2 / ISO 27001:2022 / GDPR globally and per-pack frameworks (KR-ISMS-P, KR PIPA, HIPAA, APPI, PDPA, DPDPA, LGPD, etc.) via the regional-packs overlay system.

## Architecture Decisions

- **Audit-grade depth, not skeletons** (user 2026-05-17 directive: "deeper"). Real STRIDE analysis, concrete legal citations, complete per-pack overlays.
- **Vertical slicing** (A → B → C → D) so each slice leaves the µservice in a complete-posture state. Reviewable per slice.
- **Industry-standard artifact paths** (`threat-model.md`, `dpia.md`, `compliance.md`, etc.) at `microservices/<ms>/` so external auditors find what they expect.
- **Mimir multi-tenancy**: hashed-customer-id `X-Scope-OrgID` + reserved `oya-ci` tenant for CI lane reads (gap #2).
- **Self-observability bootstrap**: fail-closed (≥3 evaluator cycles of clean data before verdicts emit) (gap #15 + PRD OQ#4).
- **Layer-A cluster**: dedicated observability cluster (not co-located) (PRD OQ#2).

## Task List

### Phase 1: Slice A — Security + Privacy posture

- [ ] Task A1: `threat-model.md` — STRIDE analysis (12+ threats per asset; concrete mitigations; owner per mitigation; residual risk)
- [ ] Task A2: `dpia.md` — Purpose / Lawful basis / Data classes / Risks / Mitigations / Council sign-offs per ICO + CNIL templates
- [ ] Task A3: `policy/tenant-isolation.md` — Mimir multi-tenancy spec; failure modes; audit-trail
- [ ] Task A4: `policy/data-residency.md` — per-pack jurisdiction map; Mimir tenant tagging
- [ ] Task A5: `policy/*.cedar` — Cedar policy fragments (tenant scope, CI scope, public read scope)
- [ ] Task A6: Update `/specs/agentic-slo-gated-promotion.json` with Mimir multi-tenancy spec

### Checkpoint: Slice A complete
- [ ] All 6 Slice A files exist at canonical paths
- [ ] Cedar fragments validate against Cedar schema
- [ ] Threat-model lists ≥12 distinct threats with owners
- [ ] DPIA carries concrete legal citations (GDPR Arts. 5/6/7/9/13/14/17/22/25/32/35; KR PIPA Arts. 3/15/17/18/22-2/23/24/25/28/29; ISO 27001 Annex A.5/A.6/A.7/A.8 control families; SOC 2 CC1-CC9)
- [ ] User reviews Slice A before Slice B begins

### Phase 2: Slice B — Operational posture

- [ ] Task B1: `cost-budget.md` — line-item Mimir/Loki/Tempo/Pyroscope/Grafana/Alertmanager/OnCall costs + scale formulas
- [ ] Task B2: `failure-modes.md` — 8-12 failure scenarios with impact + detection + mitigation + recovery
- [ ] Task B3: `capacity-model.md` — sizing formulas (N tenants → M replicas, X TB storage; per Grafana reference architectures)
- [ ] Task B4: `compliance.md` — control-to-framework mapping (SOC 2 Type 2 / ISO 27001:2022 / GDPR globally; per-pack overlays)
- [ ] Task B5: `multi-region.md` — federated Mimir + replicated Loki/Tempo per region; ADR-0117 residency
- [ ] Task B6: `incident-response.md` — escalation policy + comms templates + on-call rotation
- [ ] Task B7: `runbooks/{rollback,held-promotion-recovery,canary-graduation,mimir-outage,evaluator-down,oncall-rotation}.md`

### Checkpoint: Slice B complete
- [ ] All Slice B files exist
- [ ] Failure-modes ≥10 scenarios
- [ ] Compliance mapping covers all enforced framework controls
- [ ] Multi-region story complete (not deferred)
- [ ] User reviews Slice B before Slice C begins

### Phase 3: Slice C — Interface + Tenant posture

- [ ] Task C1: `contracts/openapi/slo-engine.yaml` — REST API for SLO query + manifest CRUD
- [ ] Task C2: `contracts/asyncapi/eligibility-events.yaml` — EligibilityChanged + PromotionExecuted + RollbackExecuted
- [ ] Task C3: `contracts/proto/slo-engine.proto` — gRPC peer
- [ ] Task C4: `capabilities/{slo-evaluate,eligibility-query,openslo-validate}.yaml` — Foundry capability records with eval-sets
- [ ] Task C5: `dashboards/{tenant-slo-overview,operator-burn-rate,gate-eligibility}.json` — Grafana JSON dashboards
- [ ] Task C6: `backfill-replay.md` — historical signal computation contract
- [ ] Task C7: `sdk-plan.md` — Rust SDK + bindings strategy (TS/Python/JVM)

### Checkpoint: Slice C complete
- [ ] All Slice C files exist
- [ ] OpenAPI + AsyncAPI + Proto contracts validate against their schemas
- [ ] Capability records carry golden_inputs + expected_outputs + eval_metric
- [ ] User reviews Slice C before Slice D begins

### Phase 4: Slice D — Convention + Tooling addenda

- [ ] Task D1: `docs/standards/observability-slo.md` — cross-cutting OpenSLO authoring + SLI catalog + burn-rate thresholds + version-pinning
- [ ] Task D2: `/specs/microservice-migration-tooling.json` — `oya dev migrate-microservice` command spec
- [ ] Task D3: `microservices/observability/competitor-parity-matrix.md` — quantitative parity vs Grafana SLO / Datadog SLO / Nobl9 / Sloth / GCP SLO
- [ ] Task D4: Edit ADR-0130 — Bominal-inheritance verification result; competitor parity citation
- [ ] Task D5: Edit ADR-0131 — migration cost quantification; src/ path corrections in sub-table
- [ ] Task D6: Edit ADR-0132 — migration cost quantification
- [ ] Task D7: Edit PHASE-01 — ChangeSet contract per IP; per-IP test coverage threshold; branch-protection.yaml diff preview
- [ ] Task D8: Edit `/specs/agentic-slo-gated-promotion.json` — backfill-replay + canary cohort finalization

### Checkpoint: Slice D complete + Turn 2 unblocked
- [ ] All Slice D files exist
- [ ] All edits applied to upstream ADRs and specs
- [ ] `oya gate validate per-microservice-layout --microservice observability` exit 0
- [ ] `oya gate validate authority-cohesion` exit 0
- [ ] User explicit go-ahead to begin Turn 2

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Audit-grade depth produces 40+ KB per artifact; context budget pressure across turns | Medium | One slice per turn; artifacts batch-parallel within a turn |
| Cedar fragment syntax errors not caught until CI runs | Medium | Each .cedar file authored against Cedar v3.4 schema reference; validator pass before slice-A checkpoint |
| Compliance citations drift if framework versions update | Low | Slice D pins versions: SOC 2 Type 2 (2017 TSC + 2022 points of focus), ISO 27001:2022, GDPR (latest EDPB guidelines), KR PIPA (post-2020 amendments) |
| Multi-region capacity model assumes Grafana published numbers; oyatie scale may differ | Medium | Capacity-model.md cites Grafana benchmarks + adds 30% buffer for oyatie-specific load patterns |
| User changes scope mid-slice | Low | Slice checkpoints surface drift; user reviews before next slice begins |

## Open Questions

- Pack-overlay schema for compliance frameworks — does `regional-packs/<pack>/compliance.md` use the same shape as the µservice's `compliance.md`? Slice D resolves.
- ADR-NNNN follow-up numbers (multi-region, federated-Mimir, retired suite ADR consolidation). Slice D allocates.

## References

- `docs/ideas/hyperscaler-gap-closure-plan.md` — idea-refine one-pager
- `tasks/todo.md` — task list (this plan's tracking surface)
- ADR-0130, ADR-0131, ADR-0132 — shipped
- `microservices/observability/PRD.md`, `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md` — shipped
- `/specs/agentic-slo-gated-promotion.json`, `/specs/per-microservice-flat-layout.json` — shipped
