# Hyperscaler Gap-Closure — Task List

Status legend: ⬜ pending · 🟦 in-progress · ✅ done

## Slice A — Security + Privacy posture

- 🟦 A1 — threat-model.md (STRIDE per asset; 12+ threats; mitigations; owners; residual risk)
- ⬜ A2 — dpia.md (ICO + CNIL template; GDPR / KR PIPA / HIPAA citations)
- ⬜ A3 — policy/tenant-isolation.md (Mimir X-Scope-OrgID + oya-ci reserved)
- ⬜ A4 — policy/data-residency.md (per-pack jurisdiction map)
- ⬜ A5 — policy/*.cedar (tenant + ci + public read; Cedar v3.4 schema)
- ⬜ A6 — /specs/agentic-slo-gated-promotion.json multi-tenancy update

## Slice B — Operational posture (depends on Slice A)

- ⬜ B1 — cost-budget.md (line-item Grafana stack costs + scale formulas)
- ⬜ B2 — failure-modes.md (10+ scenarios; impact / detection / mitigation / recovery)
- ⬜ B3 — capacity-model.md (sizing formulas)
- ⬜ B4 — compliance.md (control-to-framework mapping)
- ⬜ B5 — multi-region.md (federated Mimir + replication)
- ⬜ B6 — incident-response.md (escalation + comms templates)
- ⬜ B7 — runbooks/ (6 runbooks)

## Slice C — Interface + Tenant posture (depends on Slice A)

- ⬜ C1 — contracts/openapi/slo-engine.yaml
- ⬜ C2 — contracts/asyncapi/eligibility-events.yaml
- ⬜ C3 — contracts/proto/slo-engine.proto
- ⬜ C4 — capabilities/*.yaml (3 capability records with eval-sets)
- ⬜ C5 — dashboards/*.json (3 Grafana dashboards)
- ⬜ C6 — backfill-replay.md
- ⬜ C7 — sdk-plan.md

## Slice D — Convention + Tooling addenda (depends on A + B + C)

- ⬜ D1 — docs/standards/observability-slo.md
- ⬜ D2 — /specs/microservice-migration-tooling.json
- ⬜ D3 — microservices/observability/competitor-parity-matrix.md
- ⬜ D4 — ADR-0130 edits (Bominal verify + competitor parity)
- ⬜ D5 — ADR-0131 edits (migration cost + src/ path fix)
- ⬜ D6 — ADR-0132 edits (migration cost)
- ⬜ D7 — PHASE-01 edits (ChangeSet per IP + test coverage + branch-protection diff)
- ⬜ D8 — /specs/agentic-slo-gated-promotion.json (backfill + canary finalization)

## Exit criteria

- ✅ All four slices closed
- ✅ `oya gate validate per-microservice-layout --microservice observability` exit 0
- ✅ `oya gate validate authority-cohesion` exit 0
- ✅ Turn 2 (15 observability IPs) unblocked

---

After all 28 tasks complete: begin Turn 2 (Tasks #7, #8 in TaskCreate index).
