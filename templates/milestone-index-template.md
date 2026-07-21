---
doc_class: Template
template_id: TPL-MILE
status: pending approval
purpose: |
  Canonical Milestone INDEX shape (≤100 lines). One file per milestone under `.omc/plans/milestones/M0N/INDEX.md`. Anchors phase list, names dependencies, acceptance gate, hyperscaler practices inherited, agent-navigability pointer.
lift_target: oyatie/docs/templates/milestone-index-template.md
enforcing_fitness_lane: oya-governance-plan-hierarchy
owner_team: council-architecture
related:
  - .omc/plans/MASTERPLAN.md
  - /templates/phase-index-template.md
length_cap: 100
---

```yaml
# Required frontmatter on every milestone INDEX.md
---
doc_class: MilestoneIndex
template_id: TPL-MILE
milestone_id: M0N
parent: ../../MASTERPLAN.md
masterplan_work_item_id: MPV2-<nnnn> # /specs/masterplan.json#masterplan_v2.work_items; derived wave is in .sequencing
status: pending approval | gated | open | in-progress | merged | blocked
purpose: |
  One paragraph: what this milestone delivers; which axes it touches; which wave it aligns to.
owner_axis: <axis-id>
co_owners: [<team-id>]
gates_on: [M0N-1, ...]          # predecessor milestones
hyperscaler_practices_inherited: [working-backwards | design-doc | postmortem-blameless | 1ES-templated-pipelines | engineering-excellence | trunk-based-development | slsa-l2 | feature-flags-canary | cargo-vet | distroless]
length_cap: 100
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---
```

# M0N: <one-line milestone title>

## Purpose

One paragraph. What this milestone delivers, which axes it touches, and its `masterplan_work_item_id` from `/specs/masterplan.json#masterplan_v2.work_items`.

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
| M-CC-P0M | open | per `milestones/M01-foundation/phases/P0M/INDEX.md` |

## Acceptance gate

Numbered list of measurable criteria the milestone must meet to merge. Each row names a fitness lane OR command OR `(advisory)` marker. **BLOCKED:** masterplan v2 has no field-level successor for legacy wave-gate acceptance criteria, so do not claim this gate traces to one.

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
→ then `grit claim <first symbol from that IP §Symbols to grit-claim>`
```

## Inherited hyperscaler practices

- Working-Backwards / PRFAQ (per `.omc/scratch/hyperscaler-best-practices-2026-05-12.md §Domain 1 AWS`) — applied to: `<phase list>`
- Google Design Doc — applied to: `<phase list>`
- Blameless Postmortem — every Sev-1/Sev-2 in milestone scope.
- Trunk-Based Development + Feature Flags + Canary — release strategy per `docs/RELEASE-MANAGEMENT.md`.
- SLSA L2 + Cosign keyless OIDC + Syft SBOM — supply-chain (M01-P15).
- `cargo-vet` + `cargo-deny` + `cargo-audit` triad — Rust supply chain.
- Distroless / Chainguard images — runtime image policy.

## Risk register (milestone-scoped slice)

| ID | Risk | Owner | Status |
|---|---|---|---|
| RM-<N> | <one line> | <team> | open |

Full register: `docs/RISK-REGISTER.md`.

## Sources

- `.omc/plans/MASTERPLAN.md`
- `/specs/masterplan.json#masterplan_v2.work_items`
- `/specs/masterplan.json#masterplan_v2.sequencing`
- `docs/RACI-OWNERSHIP.md`
- `.omc/scratch/hyperscaler-best-practices-2026-05-12.md`
