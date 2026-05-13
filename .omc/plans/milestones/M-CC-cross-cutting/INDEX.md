---
doc_class: MilestoneIndex
parent: ../../MASTERPLAN.md
id: M-CC
title: Cross-cutting workstreams
wave: n/a (threads across all milestones)
status: open
owner: per-phase owner (see §Phases); council-architecture coordinates
purpose: Thread the eight cross-cutting principles (agentic-pipeline, doc-automation, purpose-discipline, agentic-navigability, provider-agnosticism, distroless+LTS, hyperscaler practices, supply-chain) through every milestone via CI lanes and recurring discipline.
acceptance_authority: ../../MASTERPLAN.md §8
---

# M-CC — Cross-cutting workstreams

## Purpose
The ten compound principles from MASTERPLAN §2 are enforced by lanes and discipline that thread through every milestone. M-CC is the home for those threads; its phases are concurrent with whichever main-spine milestone is in flight.

## Status
**open.** M-CC-P01 (agentic-pipeline cutover) is in-flight iter-2 already; M-CC-P02..P08 begin in parallel with M01.

## Scope
Eight phases, each owning one cross-cutting concern and one or more CI fitness lanes. Lanes ratchet WARN → BLOCK per wave (per [`../../../docs/PRD.md`](../../../docs/PRD.md) §4.1 last row).

## Dependencies
M-CC-P01 (agentic-pipeline cutover) is foundational — every other M-CC phase and every main-spine milestone phase depends on it at ≥ P5 merged. M-CC-P02..P08 then run in parallel with M01 onward.

## Acceptance gate
- All eight phases' CI lanes green on `main`.
- No main-spine milestone can pass its acceptance gate without inheriting the relevant M-CC lanes (e.g., M01 acceptance requires M-CC-P03 orphan-detection green; M02 acceptance requires M-CC-P05 provider-agnosticism green).

## Phases
| ID | Title | Status | Index |
|---|---|---|---|
| P01 | Agentic-Pipeline Cutover (grit/icm SoT) — lifts [`../../ralplan-oyatie-sst-consolidation.md`](../../ralplan-oyatie-sst-consolidation.md) | in-flight iter-2 | [`phases/P01-agentic-pipeline-cutover/INDEX.md`](phases/P01-agentic-pipeline-cutover/INDEX.md) |
| P02 | Doc Auto-Generation + Freshness | stub | [`phases/P02-doc-automation-freshness/INDEX.md`](phases/P02-doc-automation-freshness/INDEX.md) |
| P03 | Purpose-Discipline + Orphan-Detection | stub | [`phases/P03-purpose-orphan-detection/INDEX.md`](phases/P03-purpose-orphan-detection/INDEX.md) |
| P04 | Agentic-Dev Optimization (Navigability Lanes) | stub | [`phases/P04-agentic-navigability/INDEX.md`](phases/P04-agentic-navigability/INDEX.md) |
| P05 | Provider-Agnosticism + Adapter Discipline | stub | [`phases/P05-provider-agnosticism/INDEX.md`](phases/P05-provider-agnosticism/INDEX.md) |
| P06 | Distroless + Image-Discipline + LTS-Dependency | stub | [`phases/P06-distroless-lts-image/INDEX.md`](phases/P06-distroless-lts-image/INDEX.md) |
| P07 | Hyperscaler-Practice Adoption (Working Backwards / Design Doc / Postmortem / 1ES / Eng-Excellence) | stub | [`phases/P07-hyperscaler-practices/INDEX.md`](phases/P07-hyperscaler-practices/INDEX.md) |
| P08 | Supply-Chain Security (Cosign / Rekor / SLSA / SBOM) | stub | [`phases/P08-supply-chain-security/INDEX.md`](phases/P08-supply-chain-security/INDEX.md) |
| P09 | Visualization-as-Code (Foundry-owned architecture / product / service / tech-stack maps) | stub | [`phases/P09-visualization-as-code/INDEX.md`](phases/P09-visualization-as-code/INDEX.md) |

## Parallelism strategy
After P01 ≥ P5 merged, P02..P08 all run in parallel (each writes a distinct fitness-lane crate suffix + a distinct discipline-doc set). Each P0N has 2-3 IPs running concurrently. Target: 8 agents in parallel across M-CC at peak.

## Hyperscaler practices adopted
This milestone IS the hyperscaler-practice rollout for the whole project. P07 lifts named practices (AWS Working Backwards / PRFAQ, Google Design Doc / Postmortem, Microsoft 1ES, Oracle Engineering Excellence Council) into the workflow.

## Agent-navigability-pointer
First-claim seed for M-CC overall: continue [`../../ralplan-oyatie-sst-consolidation.md`](../../ralplan-oyatie-sst-consolidation.md) iter-2 phases — the cutover IS M-CC-P01. After P5 merge, the M-CC-P02..P08 fan-out begins. Each phase has its own first-claim seed in its INDEX.
