---
id: ADR-0040
status: Superseded
superseded_by: [ADR-700]
doc_status: published
---

# ADR-0040: Progressive delivery — Argo Rollouts canary, blue-green for stateful surfaces, metric-gated rollback at SLO burn-rate ≥ 14.4×

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `foundry`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0003, ADR-0028, ADR-0037, ADR-0038, ADR-0041, ADR-0042, ADR-0050

---

## Context

A release that ships to 100% of tenants on cut is the highest-blast-radius operation in the system. The pack-of-19 foundation ADRs decided that progressive delivery is mandatory but did not pin the mechanics: which controller, what canary percentages, what gating signals, what rollback math, what posture for stateful surfaces (databases, message queues, mail spools) where blue-green is the only credible path. Without pinning, every axis evolves its own rollout pattern; every per-microservice pattern fails differently; the cohesion thesis collapses at the deployment plane.

The release-management dimension binds the SLO catalog (per ADR-0042) to the deployment pipeline: a rollout is just a controlled experiment in degrading reliability, and the experimenter must abort when the data says abort. This ADR pins the controller, the canary stages, the metric-gated rollback math (Google SRE Workbook burn-rate alerts), the per-region phased pattern, and the per-cell rollback unit.

---

## Decision

We adopt **Argo Rollouts** as the canonical progressive-delivery controller; **canary 5% → 25% → 50% → 100%** as the default stage progression; **metric-gated rollback** at SLO 1h burn-rate ≥ 14.4× (Sev-1-class trigger); **blue-green** for stateful surfaces; **per-region phased rollout** as the geographic progression; **per-cell rollback** as the unit of revert.

### Argo Rollouts as canonical controller

```yaml
# infra/argo-rollouts/templates/canary-default.yaml
apiVersion: argoproj.io/v1alpha1
kind: Rollout
metadata:
  name: <axis>-<service>
spec:
  strategy:
    canary:
      steps:
      - setWeight: 5
      - pause: { duration: 10m }
      - analysis:
          templates: [{ templateName: slo-burn-rate-1h }]
      - setWeight: 25
      - pause: { duration: 30m }
      - analysis:
          templates: [{ templateName: slo-burn-rate-1h }, { templateName: slo-burn-rate-5m }]
      - setWeight: 50
      - pause: { duration: 1h }
      - analysis:
          templates: [{ templateName: slo-burn-rate-1h }, { templateName: slo-burn-rate-5m }]
      - setWeight: 100
```

Argo Rollouts (Apache-2; CNCF Graduated) ships with: canary / blue-green / experiment / analysis-template primitives; per-step pause + analysis; auto-rollback on failed analysis; per-step abort hooks.

### Canary stage progression

| Stage | Weight | Min hold | Required signals |
|---|---|---|---|
| 1 | 5% | 10 min | error rate stable; latency P95 within 10% of prev |
| 2 | 25% | 30 min | SLO 1h burn-rate < 14.4×; SLO 5m burn-rate < 144× |
| 3 | 50% | 1 h | SLO 1h burn-rate < 14.4×; SLO 5m burn-rate < 144×; per-tenant error rate stable |
| 4 | 100% | 24 h soak | SLO 1h burn-rate < 14.4× sustained |

Per-axis defaults can extend stages but cannot shorten them without ADR amendment.

### Metric-gated rollback math (Google SRE Workbook burn-rate alerts)

For an SLO with target T (e.g. 99.9%), monthly error budget = 1 − T = 0.1%. A 1-hour window that consumes 2% of monthly budget = 14.4× the steady-state burn rate. This is the Sev-1-class trigger:

```
burn_rate(window) = (errors_observed_in_window / requests_in_window) / (1 - SLO_target)

If burn_rate(1h) >= 14.4  → automatic rollback (Sev-1)
If burn_rate(5m) >= 144   → automatic rollback (Sev-1, faster path)
If burn_rate(6h) >= 6     → page on-call (Sev-2)
If burn_rate(3d) >= 1     → ticket (Sev-3)
```

Argo Rollouts analysis template queries the metric store (per ADR-0042) on each pause; threshold breach aborts the rollout and triggers automatic rollback to the previous revision.

### Blue-green for stateful surfaces

Surfaces with persistent state in the data plane (database primaries per ADR-0045, message queues, mail spools, search index masters) cannot canary safely — a rolled-back tenant whose write hit the new schema cannot replay against the old. For these, we use **blue-green**:

- Stand up the new (green) deployment alongside the old (blue).
- Replicate writes; read-traffic split test.
- Cutover traffic atomically; both stacks run for a soak period (24h to 7d depending on surface).
- Rollback = redirect traffic to blue.
- Tear down blue after soak + downstream confirmation.

### Per-region phased rollout

Geographic progression (after canary stages succeed in primary region):

1. **Primary cell in primary region** (KR-Seoul1).
2. **Secondary cells in primary region** (KR-Seoul1 AZ-2, AZ-3).
3. **Secondary region cells** (KR-Chuncheon).
4. **Other regions** (Phase 2: per-region progression, including non-KR regions when onboarded).

Per-region progression runs the same canary stages on each region; per-region analysis is independent so a regional regression does not stall the global rollout indefinitely.

### Per-cell rollback unit

Rollback is per-cell. A single bad cell can be rolled back without affecting others. Per-cell rollback emits an audit event (per ADR-0003) and a per-cell incident record.

### Sev-1-class auto-rollback semantics

If 1h burn-rate ≥ 14.4× during a rollout:

1. Argo Rollouts aborts the canary and routes 100% of traffic to the previous revision.
2. Per-cell incident opened (per ADR-0042 SLO catalog).
3. Per-axis on-call paged.
4. Per-tenant trust portal (per ADR-0038) updated within 5 minutes with the active incident.
5. Postmortem-doctrine-replacement: per the prevention doctrine in `docs/standards/prevention-doctrine.md`, the lane runs a fix-the-system pass — what gate / hook / validator / test would have caught this pre-rollout? That gate is added.

### Pre-release verification gate

A release-candidate cut requires (per `/oya-release-verify`):

- `oya-governance-cohesion` PASS (per ADR-0001).
- `oya-governance-supply-chain` PASS (per ADR-0039).
- `oya-governance-api-semver` PASS (per ADR-0037).
- Per-axis fitness lanes PASS.
- SLO catalog freshness within 1h.
- Per-deprecation telemetry within tolerance (per ADR-0037).
- Trust-portal sync confirmed (per ADR-0038).

### Per-axis rollout cadence

| Axis | Canary cadence | Blue-green cadence |
|---|---|---|
| SaaS platform | weekly | per-database-migration |
| Workspace | weekly | per-mail-spool / per-Drive-replica change |
| Vertical | bi-weekly per pack | per-canonical-entity-model change |
| Foundry | weekly | per-agent-runtime breaking change |
| Cloud | bi-weekly per cell | per-control-plane upgrade |
| Search | weekly | per-index-shard rebuild |
| Ads/Analytics | weekly | per-attribution-model change |

### Anti-scope

This ADR does not own the SLO catalog (per ADR-0042). Does not own the gitops branch model (per ADR-0041). Does not own the supply-chain signing chain (per ADR-0039). Does not define the prevention doctrine (per repo-root prevention-doctrine.md, applied here).

---

## Consequences

### Positive

- Mechanical canary stages + metric-gated rollback take the human judgment out of the abort decision; the data decides.
- Blue-green for stateful surfaces is the only credible posture; pinning it prevents axis teams from improvising under deadline pressure.
- Per-cell rollback unit means a bad release can be reverted in one cell without disturbing healthy cells — minimizes blast radius.
- Per-region phased rollout gives KR-launch the time-and-distance buffer to detect KR-specific regressions before they spread.
- Burn-rate math comes from Google SRE Workbook; this is a battle-tested formulation that doesn't need re-derivation.

### Negative

- 24h soak at 100% slows the release cadence; a "ship in 4 hours" expectation is gone.
- Argo Rollouts adds a controller; per-cluster overhead.
- Blue-green for databases is expensive at scale (2× capacity during cutover).
- Burn-rate math assumes well-defined SLOs; new services in preview tier may not have SLOs yet — those skip metric-gated rollback (and accept higher operational risk).

### Operational

- Per-rollout dashboard with stage-progression + analysis result; per-microservice SLO panel.
- Per-cell rollback runbook.
- Argo Rollouts + Prometheus / VictoriaMetrics adapter (per ADR-0042) maintained per-cell.
- Per-quarter rollback drill: deliberately deploy a faulty version to a preview cell and confirm auto-rollback fires.
- Per-month review of any rollout that did not follow the canonical stages. Stage adherence is mandatory; a non-conforming rollout triggers an ADR-amendment proposal whose acceptance is the canonical extension path. No grandfathered deviations.

---

## Alternatives considered

### Alternative A — Flagger instead of Argo Rollouts

- **Pros:** also CNCF; lighter weight; service-mesh native.
- **Cons:** less rich analysis primitives; less mature blue-green support; smaller community in 2026.
- **Rejected because:** Argo Rollouts has the richer feature set we need for cross-microservice use.

### Alternative B — Big-bang releases with feature flags

- **Pros:** simpler infrastructure.
- **Cons:** every flag-flip becomes a quasi-release with no metric gate; flag-flipping discipline in practice degrades to "we toggled it and watched dashboards".
- **Rejected because:** the metric gate is the moat. Feature flags complement progressive delivery; they don't replace it.

### Alternative C — Per-axis controller choice

- **Pros:** microservice-team flexibility.
- **Cons:** N controllers; per-controller drift; the cohesion thesis applied to ops collapses.
- **Rejected because:** the cohesion thesis applies to deployment.

### Alternative D — Single-stage canary (5% → 100%)

- **Pros:** faster.
- **Cons:** insufficient time for tail-latency regressions to manifest; per-tenant regressions hidden.
- **Rejected because:** the staged hold-times are exactly the times needed to detect the regression class we're guarding against.

---

## Open questions

1. **Q1.** Per-axis SLO target — 99.9% (3 nines) or 99.95% (3.5 nines) for GA? Default: 99.95% per ADR-0037 GA tier; some critical paths (audit chain, identity) at 99.99%. → ADR-0042.
2. **Q2.** Stateful blue-green soak — 24h or 7d default? Default: 24h for non-regulated; 7d for regulated (healthcare / fintech). → ADR-0034.
3. **Q3.** Per-cell rollback authority — automatic only on Sev-1, or per-cell on-call discretion? Default: automatic on Sev-1; on-call discretion for Sev-2. → owner: `foundry`.
4. **Q4.** Cross-region rollout halt — does a Sev-2 in region 1 halt rollout to region 2? Default: yes by default; explicit override allowed. → owner: `foundry`.
5. **Q5.** Per-axis cadence enforcement — does shipping early require ADR? Default: yes for stable+GA tiers; preview tier ships at axis discretion. → ADR-0037.

---

## References

- `docs/PRD.md` §10 (release management), §11 (per-tenant SLA)
- `docs/DESIGN.md` §11 (release pipeline), §10 (cross-microservice contracts)
- Google SRE Workbook §5 (Alerting on SLOs); CNCF Argo Rollouts spec
- `docs/standards/prevention-doctrine.md` (post-incident fix-the-system discipline)
- ADR-0001 (cohesion), ADR-0003 (audit), ADR-0028 (cloud cells), ADR-0037 (API stability), ADR-0038 (trust portal), ADR-0039 (supply chain), ADR-0041 (gitops), ADR-0042 (observability), ADR-0050 (automation pipeline)
