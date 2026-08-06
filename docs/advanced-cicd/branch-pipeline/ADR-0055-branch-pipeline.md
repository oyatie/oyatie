---
doc_class: ADR-Draft
shape: anchor
length_cap: 250
authority_tier: 1
status: Accepted
id: ADR-0055
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  ADR draft. Supersedes ADR-0041 trunk-based posture. Decision: four-layer pipeline
  (worktree → local-dev clone → origin/dev → staging → prod) with the review-and-CI gate
  at local-dev → origin/dev, autonomous origin/dev → staging, 5-gate staging → prod.
  No human-button at any transition. Reviewer-agent verdict gates shared-world entry.
related_adrs: [ADR-0040, ADR-0041, ADR-0042, ADR-0050]
supersedes: [ADR-0041]
doc_status: published
---

# ADR-DRAFT: Four-Layer Branch Pipeline with Reviewer-Agent-Gated Auto-Promotion (Supersedes ADR-0041)

> **Status:** Draft (pending approval). **Owner:** `axis-foundry`. **Date:** 2026-05-12. **Supersedes:** [ADR-0041](../../../docs/decisions/ADR-0709-general-live-apex.md).

---

## Context

ADR-0041 pinned trunk-based development on `main` with short-lived feature branches, squash-merge, branch-protection-as-code, and merge-queue serialization for root-Cargo-touch PRs. The model is the hyperscaler default (Google, Microsoft per `.omc/scratch/hyperscaler-best-practices-2026-05-12.md`).

Three forces require revisiting:


2. **Per-change-class reviewer roster.** `docs/AGENTS.md` defines 12 reviewer agents (rust / typescript / python / database / security / privacy / tdd / silent-failure / doc / capability / perf / doc-style). Their verdicts have to bind to a specific transition; trunk-based has only one transition (merge to main) and forces all verdicts to converge there.

3. **Progressive-delivery binding.** [ADR-0040](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md) requires canary stages that take 24h+ to soak. Trunk-based pushes this work to feature flags; we have feature flags ([`feature-flag-architecture.md`](../progressive-delivery/feature-flag-architecture.md)) but use them for **cohort intersection**, not **release-stage surrogates**.

We need a model where the **reviewer-agent verdict gates entry to the shared world**, **canary observation gates entry to production**, and **everything in between is autonomous**.

---

## Decision

We adopt a **four-layer pipeline** with **asymmetric auto-promotion gates** and **no human button at any transition**:

- **Layer 2 (`origin/dev`).** Shared remote dev branch. Inbound gate: **3-gate verification** (PR shape + reviewer-agent `APPROVE` per change-class dispatch + CI green). Promoted by `dev-promoter` agent via PR auto-merge.
- **Layer 3 (`staging`).** Canary-deployment branch. Inbound gate: **none** (autonomous; CI was already cleared at dev entry). Promoted by `staging-promoter` agent via fast-forward; event-driven on every `origin/dev` commit OR ≤ 5 min batch.
- **Layer 4 (`prod`).** Verified production branch. Inbound gate: **5-gate verification** (comments-resolved + CI-green ≥ N consecutive runs + canary-100% ≥ M hours + zero open `slo-burn-rate-fast` alerts + optional reviewer re-affirm per change class). Promoted by `prod-promoter` agent via fast-forward.

**Mutator allowlist.** Each long-lived branch has exactly one mutator agent identity:

- `origin/dev` ← `dev-promoter` (Cosign identity `oya-intelligence-dev-promoter`).
- `staging` ← `staging-promoter` (Cosign identity `oya-intelligence-staging-promoter`).
- `prod` ← `prod-promoter` (Cosign identity `oya-intelligence-prod-promoter`).

Direct commits forbidden by branch-protection. Planned advisory lanes: `oya-governance-no-direct-origin-dev-commit`, `oya-governance-no-direct-staging-commit`, `oya-governance-no-direct-prod-commit`.


**Exception path (Directive 12 carve-out).** Compliance-pack updates and KMS root rotation classes flagged `requires_human_signoff: true` require a Cosign-signed approval commit from `@council-architecture` before `prod-promoter` fires. No other class requires a human button.

**Linear history.** Squash-merge into `origin/dev`; fast-forward into `staging` and `prod`. No merge commits. Bisect always works.

**Foundry capability mirror.** Capabilities flow through the same four-layer lifecycle in lockstep: `stage: dev-draft` → `stage: dev` → `stage: staging` → `stage: prod`. Schema extended with `stage:`, `promoted_from:`, `promoted_to:`, `stage_history[]` fields. New BLOCKER lane `oya-governance-capability-stage-binding` verifies stage matches source branch.

---

## Decision Drivers

1. **Velocity at the agent landing point.** Worktree → local-dev → autonomous; agents are not slowed by ceremony at private boundaries.
2. **Quality at the first shared-world boundary.** Reviewer agent verdict authored against the smallest, freshest change set — the cheapest place to catch defects.
3. **Autonomy preserved at the deployment boundary.** origin/dev → staging is mechanical; the work was already done.
4. **Runtime validation at the prod boundary.** Canary + SLO observation can only happen post-deploy; that's where the 5-gate naturally sits.
5. **Separation of concerns at the agent role.** Three small agents (dev-promoter, staging-promoter, prod-promoter) + one fixer beats one fat orchestrator.

---

## Alternatives Considered

### Alternative A — Stay on ADR-0041 trunk-based

- **Pros:** familiar; hyperscaler default; DORA-elite.
- **Cons:** reviewer-agent verdict has nowhere natural to bind; canary observation has to be encoded via feature flags or release-branch cut; agent landing flow forces every change through the same human-shaped review.
- **Rejected because:** the agent landing flow and the reviewer-agent roster are first-class facts of the codebase, not preferences. Trunk-based pretends they're not.

### Alternative B — Classic GitFlow (`master`, `develop`, `release-*`, `hotfix-*`)

- **Pros:** familiar; explicit stage separation.
- **Cons:** heavy ceremony; release-branch maintenance is its own discipline; agent-friendliness is poor.
- **Rejected because:** the per-branch ceremony cost dwarfs the gain over a tighter three-branch graph.

### Alternative C — GitHub Flow (single `main`, feature branches, PR-time review + CI)

- **Pros:** simpler than trunk-based with merge-queue.
- **Cons:** same fundamentals as trunk-based; same critique applies.
- **Rejected because:** identical to A in the failure modes that matter.

### Alternative D — Gerrit-style (CL voting, single `master`)

- **Pros:** reviewer voting is first-class (Code-Review +2, Verified +1).
- **Cons:** stage-poor; no graph between landed and deployed.
- **Rejected because:** verdict-rich is necessary but not sufficient; we also need stage separation for canary.

### Alternative E — Review-on-every-stage (5-gate at dev AND prod)

- **Pros:** maximum quality bias.
- **Cons:** review at dev → staging is redundant — the review was already done at local-dev → origin/dev; doubling it gains no signal at high agent-population cost.
- **Rejected because:** ceremony without a preserved invariant; Linus check fails.

---

## Why Chosen

The four-layer asymmetric model is the **simplest representation** that places each gate at the data-source-freshest boundary:

- Reviewer-agent verdict ⇔ smallest change set (local-dev PR).
- CI cleared ⇔ pre-shared-world entry (same boundary as above).
- Canary + SLO ⇔ post-deploy real traffic (staging → prod).
- Comment resolution ⇔ end-of-review-thread (staging → prod, when the thread has had time to be addressed).

Each gate sits where its input data is available; no gate is invoked before its data exists; no gate is duplicated where the data is unchanged.

---

## Consequences

### Positive

- **Agent-friendly velocity.** Worktree → local-dev is autonomous; origin/dev → staging is autonomous; agents spend ceremony only at the shared-world boundary.
- **Canary observation is first-class.** The 5-gate makes canary completion a load-bearing artefact, not an afterthought.
- **Three small promoter agents.** Each role is independently testable, restartable, observable. Failure of one doesn't cascade.
- **Linear history preserved.** Bisect works on every branch.
- **Lockstep with Foundry capabilities.** `capability.stage` mirrors the branch state; no separate capability-lifecycle concept needed.

### Negative

- **Three long-lived branches** to mirror, vs trunk-based's one. Storage cost is negligible at modern git scale; mental model cost is non-zero. Mitigated by clear naming (dev/staging/prod) + the trade-off doc.
- **Hot-fix path** still requires Directive 12 human-orchestrator signature (for emergency-class only). Documented in [`rollback-mechanics-per-stage.md`](rollback-mechanics-per-stage.md). The carve-out is bounded and audited.
- **Cross-axis lockstep** has to be re-verified at the staging → prod gate via canary-clean across all consumers. Adds latency budget M hours per affected axis.

### Neutral

- **ADR-0041 superseded.** The merge-queue + per-axis CODEOWNERS posture is **inherited** by this ADR (still applied at the local-dev → origin/dev gate). Branch-protection-as-code is **inherited** with extended schema (this ADR adds mutator allowlist + SLSA-attestation requirement on prod).
- **`docs/ROADMAP.md` wave-gate review** moves from "main + release-tag" to "prod-promoted + canary-complete." Wave-gate semantics unchanged.

---

## Follow-ups

1. Lift this draft to `oyatie/docs/decisions/registry/placeholder-debt/adr-follow-ups.yaml#four-layer-branch-pipeline (drafting)` (number assigned at lift time).
2. Update `oyatie/docs/decisions/ADR-0709-general-live-apex.md status to `Superseded-by: ADR-####`.
3. Implement the three promoter agents + the fixer per [`agent-roles-spec.md`](agent-roles-spec.md). Distroless images per [Directive 5](../../plans/MASTERPLAN.md).
4. Implement the six new fitness lanes per [`governance-lanes-for-branch-pipeline.md`](governance-lanes-for-branch-pipeline.md).
5. Apply branch-protection rules per [`branch-protection-rules.md`](branch-protection-rules.md) — nightly drift-check enforces.
6. Extend Foundry capability schema per [`governance-pipeline-mirror.md`](governance-pipeline-mirror.md) §3.
7. Wave-gate doc update: reflect prod-promotion semantics in `docs/ROADMAP.md`.
8. Per-axis playbook update per [`playbooks-by-axis-stage.md`](playbooks-by-axis-stage.md).

---

## References

- `.omc/scratch/hyperscaler-best-practices-2026-05-12.md` §branch-merge-strategy (the trunk-based default we deviate from)
- [ADR-0040](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md) (canary + SLO mechanics inherited)
- [ADR-0041](../../../docs/decisions/ADR-0709-general-live-apex.md) (superseded)
- [ADR-0039](../../../docs/decisions/ADR-0709-general-live-apex.md) (Cosign + SLSA inherited)
- `docs/AGENTS.md` (reviewer-agent roster)
- `.omc/advanced-cicd/progressive-delivery/` (sister composer; runtime mechanics)
