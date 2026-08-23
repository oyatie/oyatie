---
doc_class: Enforcement
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
purpose: |
  Named fitness lanes that enforce the progressive-delivery strategy + specs + playbooks.
planned_enforcement_ref: self
related_adrs: [ADR-0040, ADR-0042, ADR-0037, ADR-0038, ADR-0039, ADR-0050, ADR-0053, ADR-0052, ADR-0054]
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
doc_status: published
---

# Enforcement Lanes — Progressive Delivery

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Lane catalogue

| Lane ID | Status | Severity | Scope | What it verifies |
|---|---|---|---|---|
| `governance-feature-flag-debt` | **NEW** | HIGH at 90 d / BLOCKER at 180 d | flag registry + repo grep | Release-class flags have a declared retire date; stable flags age out per [`feature-flag-architecture.md`](feature-flag-architecture.md) §5 |
| `governance-canary-required` | **NEW** | BLOCKER | PR | Kernel/domain/app/api/adapter changes carry a Flagger Canary or Argo Rollout manifest with the canonical 1→5→25→50→100 progression |
| `pipeline-slo-coverage` | **EXTENDED** | HIGH | service catalog + Prometheus | Every GA+ service has burn-rate alerts wired (fast 5min×1h@14.4×, slow 30min×6h@6.0×); per [`slo-burn-rate-rollback-spec.md`](slo-burn-rate-rollback-spec.md) |
| `governance-rollback-evidence` | **NEW** | BLOCKER | release artefact | Signed D14 rollback artefact present, covering up / down / dry-run / per-tenant / per-cell paths; Cosign signature valid |
| `governance-cohort-honor` | **NEW** | HIGH | mesh manifests + cohort kernel | Stable-regulated and connect-no-ads cohorts are intersected at flag-evaluation, canary traffic split, and blue/green cutover |
| `governance-shadow-diff` | **NEW** | HIGH for high-risk surfaces | shadow-diff kernel + manifests | High-risk surfaces (per [`dark-launch-spec.md`](dark-launch-spec.md) §2) carry a shadow-diff manifest with sample rate + threshold + side-effect stub list |

## 2. Lane wiring

Lanes run in the `pre-merge` and `pre-release` phases of `governance-quality-lane-kernel` (existing). Per [ADR-0050](../../../docs/decisions/ADR-0709-general-live-apex.md), lane runs are evidence-emitting and signed.

```
pre-merge:
  - feature-flag-debt (HIGH gate)
  - canary-required  (BLOCKER)
  - cohort-honor     (HIGH gate)
  - shadow-diff      (HIGH gate for high-risk surface PRs)

pre-release:
  - slo-coverage         (HIGH gate)
  - rollback-evidence    (BLOCKER)
  - (re-run) cohort-honor (BLOCKER if regression detected)
```

## 3. `governance-feature-flag-debt` (NEW)

Inputs: flag registry (`crates/platform-feature-flag-api/`), repo grep for flag references, per-flag metadata.

Checks:
1. Every flag has `type:` (release / experiment / kill-switch / permission / operational).
2. Release-class flags older than 90 d post-stable emit HIGH finding + auto-PR.
3. Release-class flags older than 180 d post-stable emit BLOCKER + auto-PR ready to merge.
4. Per-axis active-release-flag count ≤ 25; older-than-90d count ≤ 5.

Outputs: PR comment + auto-PR + D14 evidence row.

## 4. `governance-canary-required` (NEW)

Inputs: PR diff, change-class detection (kernel / domain / app / api / adapter / runtime / migration / capability).

Checks:
1. Change class identified via path + crate frontmatter.
2. For kernel/domain/app/api/adapter: Flagger Canary or Argo Rollout manifest present, with stage weights matching [`canary-rail-spec.md`](canary-rail-spec.md) §2.
3. For runtime/migration: blue/green manifest present per [`blue-green-spec.md`](blue-green-spec.md).
4. Rolling-update manifest present in disallowed paths → BLOCKER.

Outputs: pass / fail / advisory.

## 5. `pipeline-slo-coverage` (EXTENDED)

Existing scope: every GA+ service declares an SLO with target + window. Extension:
1. Every SLO declaration has a paired burn-rate alert wired in Prometheus 3.11+ (per [`slo-burn-rate-rollback-spec.md`](slo-burn-rate-rollback-spec.md)).
2. Alert thresholds match canonical values (14.4× fast, 6.0× urgent, 3.0× slow, 1.0× info).
3. Multi-window AND-gate present in alert expression.

Failure surfaces in `oyatie/docs/SLO-CATALOG.md` freshness report.

## 6. `governance-rollback-evidence` (NEW)

Inputs: release artefact bundle.

Checks:
1. D14 artefact present + signed (Cosign keyless OIDC).
2. Five-mode coverage: up / down / dry-run / per-tenant / per-cell.
3. Per-mode rehearsal evidence (last 30 d) — at least one rehearsal recorded.
4. Audit-chain link to previous release's rollback artefact (continuity).

BLOCKER if any check fails.

## 7. `governance-cohort-honor` (NEW)

Inputs: mesh manifests (Flagger Canary / Argo Rollout / Istio VirtualService), cohort kernel state, flag-evaluation traces.

Checks:
1. Webhook to `platform-tenant-cohort-kernel` present in every canary manifest.
2. Stable-regulated cohort excluded from canary stages 1-3 (1%, 5%, 25%).
3. Connect-no-ads cohort excluded from Ads-axis canary at all stages.
4. Cohort intersection traces (last 24 h) show ≥ 99.9% honour rate.

HIGH severity; escalates to BLOCKER on regression.

## 8. `governance-shadow-diff` (NEW)

Inputs: shadow-diff manifests, surface high-risk tag, side-effect stub list.

Checks for high-risk surfaces (per [`dark-launch-spec.md`](dark-launch-spec.md) §2):
1. Shadow-diff manifest present with sample rate ≥ surface-required minimum.
2. Side-effect stub list covers all external calls (emails, payment, webhooks).
3. `x-shadow` header propagation verified in receiving services.
4. Diff threshold ≤ 0.01% configured.

HIGH severity for high-risk surfaces; advisory elsewhere.

## 9. Lane evolution

New lanes are pre-registered in `governance-quality-lane-kernel` and turned on in two phases:
1. **Phase A (audit-only).** Lane runs and emits findings, but does not block PRs/releases. 30-d soak.
2. **Phase B (enforcement).** Lane blocks per its declared severity.

Phase A → B promotion gated by per-lane PR with 30-d evidence summary.

## 10. Lift target

`oyatie/docs/standards/enforcement-lanes-progressive-delivery.md` on approval.
