---
doc_class: Spec
shape: anchor
length_cap: 250
authority_tier: 1
status: Accepted
date: 2026-05-12
purpose: |
  via standard local-dev → origin/dev path. staging: revert via the standard path
  (cannot push directly to staging). prod: SLO-burn-rate-fast auto-rollback per
  ADR-0040 + hot-fix path with reduced gate set + Directive 12 human-orchestrator signature.
planned_enforcement_ref:
  - governance-rollback-evidence
related_adrs: [ADR-0040, ADR-0041, ADR-0043, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Rollback Mechanics Per Stage


## 1. Scope

Rollback procedures per layer of the four-layer pipeline. Every rollback emits D14 audit-chain evidence per [ADR-0003](../../decisions/ADR-0003-audit-chain-and-evidence-emission.md) and signed-rollback artefact per [ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md) §rollback-evidence.

## 2. Layer 0 — worktree rollback



**Authority.** The originating agent only.

## 3. Layer 1 — agent local-dev clone rollback


**Evidence.** `EVT-LOCAL-DEV-RESET` emitted. No D14 (still private).

**Authority.** The originating agent only.

## 4. Layer 2 — `origin/dev` rollback

**Mechanism.** A revert is a new PR through the standard local-dev → origin/dev path. The agent (or `staging-fixer` Mode-B) authors a revert commit in their local-dev clone, opens a PR, the 3-gate fires (PR shape + reviewer-agent verdict on the revert + CI green), and on merge the revert lands on `origin/dev`. **Cannot bypass the PR flow** — direct push to `origin/dev` is forbidden by `governance-no-direct-origin-dev-commit` (BLOCKER).

**Evidence.** D14 artefact: `revert_record { reverted_sha, revert_sha, reviewer_verdict_id, reason, reverted_at }`. Signed per [ADR-0039](../../decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md).

**Authority.** Any sanctioned agent; reviewer-agent verdict still required.

## 5. Layer 3 — `staging` rollback

**Mechanism.** A staging rollback is a revert authored at `origin/dev` (Layer 2 procedure) which auto-promotes to `staging` on the next `staging-promoter` cycle. **Direct revert on `staging` is forbidden** — `governance-no-direct-staging-commit` (BLOCKER). The reason: preserve linear-fast-forward history from `origin/dev` to `staging`; a direct staging revert creates a divergence that the next staging-promoter cycle cannot reconcile cleanly.

**Evidence.** D14 artefact (same shape as Layer 2 revert) plus `EVT-STAGING-PROMOTED` with the revert sha. Per-cell rollback if the regression affected a deployed cell (cell scope follows progressive-delivery rails).

**Authority.** `staging-fixer` (typical), any agent (general); reviewer-agent verdict required on the revert PR.

**Latency budget.** From regression detection (canary metric red OR `slo-burn-rate-fast` alert) to staging-stable: ≤ 4 hours (`governance-canary-regression-sla`, HIGH).

## 6. Layer 4 — `prod` rollback

### 6.1 SLO-burn-rate-fast auto-rollback (the runtime safety net)

Per [ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md) and [`../progressive-delivery/canary-rail-spec.md`](../progressive-delivery/canary-rail-spec.md):

- If `slo-burn-rate-fast` ≥ 14.4× (1h window) during a prod rollout → Argo Rollouts / Flagger auto-aborts and routes traffic to the previous revision.
- Per-cell auto-rollback. A single bad cell reverts without disturbing healthy cells.
- D14 evidence emitted automatically (signed rollback artefact + per-cell traffic-shift log + per-tenant incident record).
- Per-axis on-call paged within 60 sec (Sev-1) or 5 min (Sev-2).

**No prod-promoter or staging-fixer intervention needed for the rollback itself** — the progressive-delivery controller handles it. The fixer picks up afterward to author the forward-fix.

### 6.2 Hot-fix path (the controlled forward-fix)

When a defect requires an out-of-band fix faster than the standard 4-layer cadence allows:

1. Fix authored on local-dev (Layer 0 → Layer 1) by the on-call agent.
2. PR opened against `origin/dev` (Layer 1 → Layer 2). The 3-gate fires — **no shortcut**, but reviewer-agent verdict is invoked with `priority: emergency`, which routes to the per-axis on-call reviewer with a 30-min SLA (vs the standard 15-min P95 reviewer-verdict latency).
3. On merge to `origin/dev`, `staging-promoter` fires the **emergency batch** (≤ 60 sec coalescing window instead of ≤ 5 min).
4. On staging, the canary holds are **shortened** for the emergency change class: gate 1 at 1% for ≤ 2 min, gate 2 at 5% for ≤ 5 min, gate 3 at 25% for ≤ 10 min (vs the standard 5/10/30 floors). Burn-rate sample sufficiency still gates progress.
5. Staging → prod gate set is **reduced** for emergency: gates 1 + 2 + 4 (comments-resolved, CI-green ≥ N=1 run on staging, zero open SLO-fast). Gate 3 (canary 100% ≥ M hours) is reduced to canary 100% ≥ 30 min. Gate 5 (optional reviewer re-affirm) is skipped except for `security-reviewer` / `database-reviewer` / `privacy-reviewer` classes.
6. **Directive 12 human-orchestrator signature required.** `prod-promoter` refuses to fire the emergency-class promotion without a Cosign-signed approval commit from a `@council-architecture` member or the per-axis on-call lead (per `docs/RACI-OWNERSHIP.md`). This is the only place in the standard flow where a human button exists — and it exists only for emergency-class promotions.

**Evidence.** D14 artefact: `hotfix_record { incident_id, fixed_sha, emergency_class, reviewer_verdict_id, human_signoff_identity, deployed_at }`. Per-tenant trust portal updated within 5 min (per [ADR-0038](../../decisions/ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md)).

### 6.3 Per-cell prod rollback unit

Rollback unit is per-cell (per ADR-0040). A bad release reverts in one cell without disturbing others. Per-cell rollback emits an audit event and a per-cell incident record. The cell-level controller (Argo Rollouts BlueGreen or Flagger Canary) handles the traffic-shift.

### 6.4 Per-tenant prod rollback

For regulated tenants (per [ADR-0034](../../decisions/ADR-0034-per-vertical-data-class-overrides.md)), per-tenant rollback is supported via the stable-cohort traffic-shift mechanism (per [`../progressive-delivery/blue-green-spec.md`](../progressive-delivery/blue-green-spec.md) §per-tenant blue/green). One tenant reverts to blue while others stay on green.

## 7. Rollback evidence catalogue

Every rollback emits a signed D14 artefact via `intelligence-evidence-kernel`:

| Rollback class | Artefact shape | Signed by | Stored in |
|---|---|---|---|
| per-cell rollback | `per_cell_rollback` | rollout-controller | per-cell audit chain |
| per-tenant rollback | `per_tenant_rollback` | rollout-controller + cohort kernel | per-tenant + audit chain |

Every artefact verified by `governance-rollback-evidence` (BLOCKER per [`governance-lanes-for-branch-pipeline.md`](governance-lanes-for-branch-pipeline.md)).

## 8. KMS root rotation (special case)

KMS root rotation ([ADR-0043](../../decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md)) is **never auto-rolled-back**. The blue/green stateful-surface protocol applies (per [`../progressive-delivery/blue-green-spec.md`](../progressive-delivery/blue-green-spec.md)). Rollback is "re-shift traffic to blue (old root)"; the green root remains warm during the 7-day soak. Requires human-orchestrator signature per Directive 12.

## 9. Anti-scope

This file does not own:

- Progressive-delivery rollback math — owned by [ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md).
- Per-cell architecture — owned by [ADR-0009](../../decisions/ADR-0009-cell-architecture-per-tenant-per-region.md).
- KMS rotation — owned by [ADR-0043](../../decisions/ADR-0043-secrets-management-openbao-and-hsm-per-cell.md).

## 10. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — rollback mechanics respect the four-layer mutator allowlists; no direct push to `origin/dev`, `staging`, or `prod` even for rollbacks.
