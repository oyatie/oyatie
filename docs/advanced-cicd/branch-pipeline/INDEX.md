---
doc_class: ComposerIndex
shape: anchor
length_cap: 80
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  File catalogue + lift targets + cross-references for the four-layer branch-pipeline
  composer. Sibling to .omc/advanced-cicd/progressive-delivery/.
doc_status: published
---

# Branch Pipeline Composer — INDEX

> **Status:** pending approval. **Owner:** `axis-foundry`. **Date:** 2026-05-12. **Sibling:** [`../progressive-delivery/`](../progressive-delivery/).

## 1. Files in this composer

| # | File | Role |
|---|---|---|
| 1 | [`branch-pipeline-architecture.md`](branch-pipeline-architecture.md) | Four-layer architecture + Mermaid promotion graph + 3-gate + 5-gate definitions |
| 2 | [`agent-roles-spec.md`](agent-roles-spec.md) | `dev-promoter`, `staging-promoter`, `staging-fixer`, `prod-promoter` + reviewer-agent dispatch table |
| 3 | [`velocity-without-stability-loss.md`](velocity-without-stability-loss.md) | The trade-off doc + 5-model comparison + metrics |
| 4 | [`ci-policy-per-branch.md`](ci-policy-per-branch.md) | CI gate matrix per layer; provider-agnostic adapter |
| 5 | [`governance-pipeline-mirror.md`](governance-pipeline-mirror.md) | Capability stages in lockstep + schema extension + diagram |
| 6 | [`branch-protection-rules.md`](branch-protection-rules.md) | Mutator allowlist + per-branch protection YAML + drift detection |
| 7 | [`rollback-mechanics-per-stage.md`](rollback-mechanics-per-stage.md) | Per-layer rollback procedures + hot-fix path |
| 8 | [`governance-lanes-for-branch-pipeline.md`](governance-lanes-for-branch-pipeline.md) | 6 new lanes + 3 mutator-allowlist lanes + 4 cadence/quality lanes |
| 9 | [`ADR-0055-branch-pipeline.md`](ADR-0055-branch-pipeline.md) | ADR draft (supersedes ADR-0041) |
| 10 | [`playbooks-by-axis-stage.md`](playbooks-by-axis-stage.md) | Per-axis cadence + reviewer re-affirm requirement |
| 11 | `INDEX.md` (this file) | Catalogue |

## 2. Lift target

All files lift to `oyatie/docs/release/branch-pipeline/` on approval. ADR draft lifts to `oyatie/docs/decisions/registry/placeholder-debt/adr-follow-ups.yaml#four-layer-branch-pipeline (drafting)` and supersedes `oyatie/docs/decisions/ADR-0709-general-live-apex.md

## 3. Cross-references

- **Supersedes:** [ADR-0041](../../../docs/decisions/ADR-0709-general-live-apex.md) (trunk-based posture).
- **Inherits from:** [ADR-0040](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md) (canary + SLO mechanics applied at staging + prod), [ADR-0039](../../../docs/decisions/ADR-0709-general-live-apex.md) (Cosign + SLSA), [ADR-0050](../../../docs/decisions/ADR-0709-general-live-apex.md) (automation-first stance).
- **Sister composer:** [`../progressive-delivery/`](../progressive-delivery/) — feature-flag-architecture, canary-rail-spec, blue-green-spec, progressive-delivery-strategy.
- **Source directives:** [MASTERPLAN.md](../../plans/MASTERPLAN.md) Directives 1-12; principal Directives applied: 3 (final shape), 4 (provider-agnostic adapters), 5 (distroless), 6 (hyperscaler bar with documented deviation), 12 (pragmatic git/gh with logged rationale).

## 4. The three new agent roles


## 5. The six core new fitness lanes

(1) promotion-gate-local-dev-to-origin-dev, (2) promotion-gate-staging-to-prod, (3) pr-review-verdict-present, (4) pr-comment-resolution, (5) canary-regression-sla, (6) capability-stage-binding. Plus 3 mutator-allowlist lanes + 4 cadence/quality lanes. Detail in governance-lanes-for-branch-pipeline.md.
