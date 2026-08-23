---
doc_class: Template
template_id: TPL-MILE
status: Accepted
date: 2026-05-12
purpose: |
  Canonical Milestone INDEX shape (≤100 lines). One file per milestone under `.omc/plans/milestones/M0N/INDEX.md`. Anchors phase list, names dependencies, acceptance gate, hyperscaler practices inherited, agent-navigability pointer.
enforcing_fitness_lane: governance-plan-hierarchy
owner_team: council-architecture
related:
  - .omc/plans/MASTERPLAN.md
  - docs/templates/phase-index-template.md
adrs_cited:
  - ADR-0052  # inventory ledger (migration milestones)
  - ADR-0053  # sanctioned primitives
  - ADR-0054  # scaffold-claim pattern
length_cap: 100
doc_status: published
---

```yaml
# Required frontmatter on every milestone INDEX.md
---
doc_class: MilestoneIndex
template_id: TPL-MILE
milestone_id: M0N
parent: ../../MASTERPLAN.md
wave: W-<wave-name>             # per docs/ROADMAP.md
status: pending approval | gated | open | in-progress | merged | blocked
purpose: |
  One paragraph: what this milestone delivers; which axes it touches; which wave it aligns to.
owner_axis: <axis-id>
co_owners: [<team-id>]
gates_on: [M0N-1, ...]          # predecessor milestones
hyperscaler_practices_inherited: [working-backwards | design-doc | postmortem-blameless | 1ES-templated-pipelines | engineering-excellence | trunk-based-development | slsa-l2 | feature-flags-canary | pipeline-supply-chain | distroless]
length_cap: 100
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---
```

# M0N: <one-line milestone title>

## Purpose

One paragraph. What this milestone delivers; which axes it touches; which wave from `docs/ROADMAP.md` it aligns to.

## Status

Current state + last update timestamp + blocker summary (if any).

## Scope

In-scope:
- Bulleted list of axis/surfaces this milestone affects.

Out-of-scope (referenced explicitly to prevent scope creep):
- Bulleted list with pointers to which future milestone owns each excluded item.

## Dependencies

| Predecessor | Status | Acceptance gate cite |
|---|---|---|
| M0N-1 | merged \| open | per phase INDEX `§Acceptance` |
| M01-P0N | open | per `milestones/M01-foundation/phases/P0N/INDEX.md` |

## Acceptance gate

Numbered list of measurable criteria the milestone must meet to merge. Each row names a fitness lane OR command OR `(advisory)` marker. The gate **MUST** trace to a wave-gate row in `docs/ROADMAP.md §2`.

## Phases

| ID | Title | Status | Index |
|---|---|---|---|
| P01 | <slug> | open | [`phases/P01-<slug>/INDEX.md`](phases/P01-<slug>/INDEX.md) |
| P02 | <slug> | gated | [`phases/P02-<slug>/INDEX.md`](phases/P02-<slug>/INDEX.md) |

## Parallelism strategy

State which phases run in parallel and which serialize. Name the serialization root (typically `Cargo.toml [workspace.members]` per `docs/DESIGN.md §3.0.5.2`). Target: ≥ 3-5 agents in parallel per active phase batch.

## Agent-navigability pointer

The first symbol/file a fresh agent **MUST** read after this INDEX to enter the milestone:

```
.omc/plans/milestones/M0N/phases/P01-<slug>/INDEX.md
→ then pick the first OPEN IP per its `§Implementation Plans` list
```

## Inherited hyperscaler practices

- Working-Backwards / PRFAQ (per `.omc/scratch/hyperscaler-best-practices-2026-05-12.md §Domain 1 AWS`) — applied to: `<phase list>`
- Google Design Doc — applied to: `<phase list>`
- Blameless Postmortem — every Sev-1/Sev-2 in milestone scope.
- Trunk-Based Development + Feature Flags + Canary — release strategy per `docs/RELEASE-MANAGEMENT.md`.
- SLSA L2 + Cosign keyless OIDC + Syft SBOM — supply-chain (M01-P15).
- Buck2/pipeline supply-chain gate packets — Rust dependency and provenance supply chain.
- Distroless / Chainguard images — runtime image policy.

## Risk register (milestone-scoped slice)

| ID | Risk | Owner | Status |
|---|---|---|---|
| RM-<N> | <one line> | <team> | open |

Full register: `docs/RISK-REGISTER.md`.

## Sources

- `.omc/plans/MASTERPLAN.md`
- `docs/ROADMAP.md §2`
- `docs/RACI-OWNERSHIP.md`
- `.omc/scratch/hyperscaler-best-practices-2026-05-12.md`
- ADR-0052 (inventory), ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim).
