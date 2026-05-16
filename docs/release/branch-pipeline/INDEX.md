---
doc_class: ComposerIndex
shape: anchor
length_cap: 80
authority_tier: 1
status: Accepted
date: 2026-05-12
purpose: |
  File catalogue + lift targets + cross-references for the four-layer branch-pipeline
  composer. Sibling to docs/release/progressive-delivery/.
related_adrs: [ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Branch Pipeline Composer — INDEX

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12. **Sibling:** [`../progressive-delivery/`](../progressive-delivery/). **ADR:** [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md). **Sanctioned primitives:** [ADR-0053](../../decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md).

## 1. Files in this composer

| # | File | Role |
|---|---|---|
| 1 | [`branch-pipeline-architecture.md`](branch-pipeline-architecture.md) | Four-layer architecture + Mermaid promotion graph + 3-gate + 5-gate definitions |
| 2 | [`agent-roles-spec.md`](agent-roles-spec.md) | `dev-promoter`, `staging-promoter`, `staging-fixer`, `prod-promoter` + reviewer-agent dispatch table |
| 3 | [`velocity-without-stability-loss.md`](velocity-without-stability-loss.md) | The trade-off doc + 5-model comparison + metrics |
| 4 | [`ci-policy-per-branch.md`](ci-policy-per-branch.md) | CI gate matrix per layer; provider-agnostic adapter |
| 5 | [`foundry-pipeline-mirror.md`](foundry-pipeline-mirror.md) | Capability stages in lockstep + schema extension + diagram |
| 6 | [`branch-protection-rules.md`](branch-protection-rules.md) | Mutator allowlist + per-branch protection YAML + drift detection |
| 7 | [`rollback-mechanics-per-stage.md`](rollback-mechanics-per-stage.md) | Per-layer rollback procedures + hot-fix path |
| 8 | [`fitness-lanes-for-branch-pipeline.md`](fitness-lanes-for-branch-pipeline.md) | 6 new lanes + 3 mutator-allowlist lanes + 4 cadence/quality lanes |
| 9 | [`playbooks-by-axis-stage.md`](playbooks-by-axis-stage.md) | Per-axis cadence + reviewer re-affirm requirement |
| 10 | `INDEX.md` (this file) | Catalogue |

## 2. ADR

This composer is governed by [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) (Accepted; supersedes ADR-0041).

## 3. Cross-references

- **Supersedes:** [ADR-0041](../../decisions/ADR-0041-gitops-trunk-based-and-release-branch-cut-at-tag.md) (trunk-based posture) via ADR-0055.
- **Inherits from:** [ADR-0040](../../decisions/ADR-0040-progressive-delivery-canary-blue-green-metric-gated-rollback.md) (canary + SLO mechanics applied at staging + prod), [ADR-0039](../../decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md) (Cosign + SLSA), [ADR-0050](../../decisions/ADR-0050-automation-first-pipeline.md) (automation-first stance).
- **Sister composer:** [`../progressive-delivery/`](../progressive-delivery/) — feature-flag-architecture, canary-rail-spec, blue-green-spec, progressive-delivery-strategy.
- **Source directives:** [MASTERPLAN.md](../../../docs/MASTERPLAN.md) Directives 1-12; principal Directives applied: 3 (final shape), 4 (provider-agnostic adapters), 5 (distroless), 6 (hyperscaler bar with documented deviation), 12 (pragmatic git/gh with logged rationale).

## 4. The three new agent roles

`dev-promoter`, `staging-promoter`, `prod-promoter` — plus the rescoped `staging-fixer`. All distroless; all Cosign-keyless-signed; all icm-topic-emitting per role spec §2-5. Sanctioned primitives per [ADR-0053](../../decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md): `grit` + `icm` + `oya-tooling-agent-read`.

## 5. The six core new fitness lanes

(1) promotion-gate-local-dev-to-origin-dev, (2) promotion-gate-staging-to-prod, (3) pr-review-verdict-present, (4) pr-comment-resolution, (5) canary-regression-sla, (6) capability-stage-binding. Plus 3 mutator-allowlist lanes + 4 cadence/quality lanes. Detail in fitness-lanes-for-branch-pipeline.md.
