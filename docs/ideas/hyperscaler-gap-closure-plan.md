# Hyperscaler Gap-Closure Plan (M01-observability + repo-restructure)

## Problem Statement

How might we close 30 hyperscaler-grade gaps in the M01 observability + repo-restructure changeset — without reducing scope, deferring, or compromising quality — by authoring the minimum number of audit-grade artifacts at canonical paths that satisfy every gap?

## Recommended Direction

Audit-grade artifacts at industry-standard paths (`microservices/<ms>/{threat-model,dpia,compliance,failure-modes,cost-budget,capacity-model}.md`, `policy/*.cedar`, `runbooks/*.md`, etc.) — matching the paths external auditors (SOC 2 Type 2 / ISO 27001 / GDPR DPIA / KR-ISMS-P / HIPAA / etc.) look for. Consolidate subpoint-gaps into existing artifacts; standalone files only where load-bearing.

Authored in four vertical slices:

1. **Slice A — Security + Privacy posture** (6 gaps): threat-model, DPIA, Cedar fragments, tenant-isolation spec, Mimir multi-tenancy spec, data-residency contract.
2. **Slice B — Operational posture** (7 gaps): cost budget, failure-mode catalog, capacity model, compliance posture, runbooks, multi-region, incident-response. Depends on Slice A.
3. **Slice C — Interface + Tenant posture** (5 gaps): OpenAPI/AsyncAPI/proto contracts, capability records, Grafana dashboards, backfill-replay spec, SDK plan. Depends on Slice A.
4. **Slice D — Convention + Tooling addenda** (12 gaps + finalization): observability-SLO standard, migration tooling, aggregation index generator, competitor parity matrix, Bominal-inheritance verification, version-pinning, ChangeSet per IP, test-coverage thresholds, branch-protection diff, migration cost quantification, PHASE-01 + ADR-0131/0126 addenda. Depends on Slices A + B + C.

After Slice D closes, Turn 2 (15 observability IPs + per-crate catalog records) begins.

## Key Assumptions to Validate

- [x] **Audit-grade depth required, not skeletons.** Confirmed 2026-05-17: "deeper" — full structural rigor + concrete per-jurisdiction citations.
- [x] **Globally-enforced frameworks: SOC 2 Type 2 + ISO 27001:2022 + GDPR.** Confirmed 2026-05-17.
- [x] **Suggested frameworks via regional-pack overlay: KR-ISMS-P, KR PIPA, KR 전자문서법, HIPAA, APPI, PDPA, DPDPA, LGPD, Privacy Act, PDPL, UAE-DPL, etc.** Confirmed 2026-05-17.
- [x] **Mimir tenant model: hashed-customer-id `X-Scope-OrgID` + reserved `ci` tenant.** Predicted, confirmed 2026-05-17.
- [x] **Self-observability bootstrap: fail-closed (≥3 evaluator cycles clean before verdicts emit).** Predicted, confirmed 2026-05-17.
- [x] **Layer-A cluster: dedicated observability cluster (not co-located).** Predicted, confirmed 2026-05-17.
- [ ] **Per-pack legal citations cover every market in scope.** Validated by: cross-reference `regional-packs/<pack>/PACK.md` for each pack against the artifacts' overlay sections.
- [ ] **OpenSLO v1.0 + OpenTelemetry semconv versions stable for the duration of M01.** Validated by: explicit version pinning in `docs/standards/observability-slo.md` (Slice D).

## Minimum-shippable scope

There is no smallest-actionable subset. The 30-gap closure ships as one phase across four slices; partial completion leaves audit gaps. Each slice's exit gate is "all artifacts authored at audit-grade depth + `presubmit` (retired CLI `gate validate per-microservice-layout --microservice observability`) exit 0".

## Not Doing (and Why)

- **Per-gap individual files** (30 separate artifacts) — over-decomposes subpoint-gaps; auditors expect consolidated artifacts at known paths.
- **Append-everything-to-PRD** — destroys structure; PRD becomes 5x longer; auditors can't find what they expect.
- **Per-jurisdiction placeholder sections** — explicitly ruled out 2026-05-17 ("deeper").
- **Auditor-firm engagement** — separate ops workstream.
- **Legal-citation auto-generation tooling** — manual citation in this pass; future automation possible but not in scope.
- **Slice parallelism (B + C running concurrently)** — keeps each posture coherent per slice; preserves reviewability; compressed-parallel saves ~1 turn at cost of cross-slice consistency drift.

## Open Questions (deferred to Slice D)

- ADR numbers for follow-up consolidation ADRs (multi-region, federated-Mimir, retired suite ADRs full citation list).
- Pack-overlay schema for compliance frameworks (the per-pack overlay file at `regional-packs/<pack>/compliance.md` or similar — pack-team owns the actual content; Slice D specifies the contract).

## Slice Dependency Order

```
Slice A (security + privacy)
    ↓
Slice B (operational)  ←  references Slice A's threat-model + tenant-isolation
    ↓
Slice C (interface + tenant)  ←  references Slice A's Cedar policy
    ↓
Slice D (convention + tooling addenda)  ←  consolidates A + B + C
    ↓
Turn 2 (15 observability IPs)
```

## Artifact Inventory by Slice

### Slice A artifacts (6)
- `microservices/observability/threat-model.md`
- `microservices/observability/dpia.md`
- `microservices/observability/policy/tenant-isolation.md`
- `microservices/observability/policy/data-residency.md`
- `microservices/observability/policy/*.cedar` (3-5 fragments)
- `/specs/agentic-slo-gated-promotion.json` update (Mimir multi-tenancy)

### Slice B artifacts (8)
- `microservices/observability/cost-budget.md`
- `microservices/observability/failure-modes.md`
- `microservices/observability/capacity-model.md`
- `microservices/observability/compliance.md`
- `microservices/observability/multi-region.md`
- `microservices/observability/incident-response.md`
- `microservices/observability/runbooks/{rollback, held-promotion-recovery, canary-graduation, mimir-outage, evaluator-down, oncall-rotation}.md`

### Slice C artifacts (7)
- `microservices/observability/contracts/openapi/slo-engine.yaml`
- `microservices/observability/contracts/asyncapi/eligibility-events.yaml`
- `microservices/observability/contracts/proto/slo-engine.proto`
- `microservices/observability/capabilities/{slo-evaluate, eligibility-query, openslo-validate}.yaml`
- `microservices/observability/dashboards/{tenant-slo-overview, operator-burn-rate, gate-eligibility}.json`
- `microservices/observability/backfill-replay.md`
- `microservices/observability/sdk-plan.md`

### Slice D artifacts (3 new + 5 edits)
- New: `docs/standards/observability-slo.md`, `/specs/microservice-migration-tooling.json`, `microservices/observability/competitor-parity-matrix.md`
- Edits: ADR-0139 (Bominal verify + competitor parity), ADR-0131 (migration cost + src/ path fix), ADR-0132 (migration cost), PHASE-01 (ChangeSet per IP + test coverage + branch-protection diff), `/specs/agentic-slo-gated-promotion.json` (backfill + canary finalization).

Total: 24 new files + 5 edits = 29 deliverables for 30 gaps (1 gap absorbed into PRD already shipped).
