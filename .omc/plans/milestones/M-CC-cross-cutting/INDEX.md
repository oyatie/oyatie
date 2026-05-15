---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: MilestoneIndex
parent: ../../MASTERPLAN.md
id: M-CC
title: Cross-cutting workstreams
wave: n/a (threads across all milestones)
status: open
owner: per-phase owner (see §Phases); council-architecture coordinates
purpose: Thread cross-cutting principles through every milestone via CI lanes and recurring discipline; P00 Oya VCS is the first prerequisite for broad agent fan-out.
acceptance_authority: ../../MASTERPLAN.md §8
---

# M-CC — Cross-cutting workstreams

## Purpose
The ten compound principles from MASTERPLAN §2 are enforced by lanes and discipline that thread through every milestone. M-CC is the home for those threads; P00 is the first prerequisite for broad agent fan-out, and later phases are concurrent with whichever main-spine milestone is in flight.

## Status
**open.** M-CC-P00 (Oya VCS / GitOps-capable grit successor) is first-prerequisite planned from approved ralplan v5. M-CC-P01 remains the compatibility cutover lane already in flight; M-CC-P02..P09 fan out after the relevant P00/P01 gates are green.

## Scope
Ten phases, each owning one cross-cutting concern and one or more CI fitness lanes. Lanes ratchet WARN → BLOCK per wave (per [`../../../docs/PRD.md`](../../../docs/PRD.md) §4.1 last row).

## Dependencies
M-CC-P00 (Oya VCS / GitOps-capable grit successor) is the first prerequisite before broad multi-agent fan-out. M-CC-P01 (agentic-pipeline cutover) remains the compatibility foundation while P00 is built; every other M-CC phase and every main-spine milestone phase depends on P00 accepted or an explicit P00 compatibility waiver plus P01 ≥ P5 merged. M-CC-P02..P09 then run in parallel with M01 onward.

## Acceptance gate
- All ten phases' CI lanes green on `main`.
- M-CC-P00 acceptance requires Oya VCS ChangeSet scheduling, grit-authority projection, GitOps promotion, review/fix/rebase/merge-queue loop, issue-tracker integration, affected-build closure, and ops.oyatie.com evidence views green or explicitly compatibility-waived.
- M-CC-P06 acceptance also requires the dependency-seam discipline lane, tech-debt ledger, trigger DSL, replacement parity lane, and ADR-0091..ADR-0094 plan hooks from `ralplan-dep-seam-phaseout-round-5.md`.
- No main-spine milestone can pass its acceptance gate without inheriting the relevant M-CC lanes (e.g., M01 acceptance requires M-CC-P03 orphan-detection green; M02 acceptance requires M-CC-P05 provider-agnosticism green).

## Phases
| ID | Title | Status | Index |
|---|---|---|---|
| P00 | Polyglot GitOps-capable VCS Replacement (Oya VCS) — folds approved ralplan v5 | complete | [`phases/P00-gitops-vcs-replacement/INDEX.md`](phases/P00-gitops-vcs-replacement/INDEX.md) |
| P01 | Agentic-Pipeline Cutover (grit/icm SoT) — lifts [`../../ralplan-oyatie-sst-consolidation.md`](../../ralplan-oyatie-sst-consolidation.md) | complete | [`phases/P01-agentic-pipeline-cutover/INDEX.md`](phases/P01-agentic-pipeline-cutover/INDEX.md) |
| P02 | Doc Auto-Generation + Freshness | complete | [`phases/P02-doc-automation-freshness/INDEX.md`](phases/P02-doc-automation-freshness/INDEX.md) |
| P03 | Purpose-Discipline + Orphan-Detection | in-progress (IP-001 split-required) | [`phases/P03-purpose-orphan-detection/INDEX.md`](phases/P03-purpose-orphan-detection/INDEX.md) |
| P04 | Agentic-Dev Optimization (Navigability Lanes) | complete | [`phases/P04-agentic-navigability/INDEX.md`](phases/P04-agentic-navigability/INDEX.md) |
| P05 | Provider-Agnosticism + Adapter Discipline | in-progress (IP-002 split-required) | [`phases/P05-provider-agnosticism/INDEX.md`](phases/P05-provider-agnosticism/INDEX.md) |
| P06 | Distroless + Image-Discipline + Dependency-Seam/LTS Phaseout | complete | [`phases/P06-distroless-lts-image/INDEX.md`](phases/P06-distroless-lts-image/INDEX.md) |
| P07 | Hyperscaler-Practice Adoption (Working Backwards / Design Doc / Postmortem / 1ES / Eng-Excellence) | complete | [`phases/P07-hyperscaler-practices/INDEX.md`](phases/P07-hyperscaler-practices/INDEX.md) |
| P08 | Supply-Chain Security (Cosign / Rekor / SLSA / SBOM) | complete | [`phases/P08-supply-chain-security/INDEX.md`](phases/P08-supply-chain-security/INDEX.md) |
| P09 | Visualization-as-Code (Foundry-owned architecture / product / service / tech-stack maps) | complete | [`phases/P09-visualization-as-code/INDEX.md`](phases/P09-visualization-as-code/INDEX.md) |

## Parallelism strategy
After P00 reaches accepted/promote-capable state and P01 ≥ P5 merged, P02..P09 all run in parallel (each writes a distinct fitness-lane crate suffix + a distinct discipline-doc set). Each P0N has 2-3 IPs running concurrently. Target: 8 agents in parallel across M-CC at peak.

## Hyperscaler practices adopted
This milestone IS the hyperscaler-practice rollout for the whole project. P07 lifts named practices (AWS Working Backwards / PRFAQ, Google Design Doc / Postmortem, Microsoft 1ES, Oracle Engineering Excellence Council) into the workflow.

## Agent-navigability-pointer
First-claim seed for M-CC overall: start [`phases/P00-gitops-vcs-replacement/INDEX.md`](phases/P00-gitops-vcs-replacement/INDEX.md) IP-001 + IP-009, while existing [`../../ralplan-oyatie-sst-consolidation.md`](../../ralplan-oyatie-sst-consolidation.md) compatibility cutover remains M-CC-P01. After P00 accepted and P01 P5 merged, the M-CC-P02..P09 fan-out begins. Each phase has its own first-claim seed in its INDEX.
