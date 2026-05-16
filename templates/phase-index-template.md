---
doc_class: Template
template_id: TPL-PHASE
status: pending approval
purpose: |
  Canonical Phase INDEX shape (≤50 lines). One file per phase under `.omc/plans/milestones/M*/phases/P*/INDEX.md`. Anchors the IP list, names symbols touched, names parallelism, names agent-handoff icm event.
lift_target: oyatie/docs/templates/phase-index-template.md
enforcing_fitness_lane: oya-foundry-fitness-plan-hierarchy
owner_team: council-architecture
related:
  - .omc/plans/MASTERPLAN.md
  - /templates/milestone-index-template.md
  - /templates/implementation-plan-template.md
length_cap: 50
---

```yaml
# Required frontmatter on every phase INDEX.md
---
doc_class: PhaseIndex
template_id: TPL-PHASE
phase_id: P0N-<slug>
parent: ../INDEX.md            # milestone INDEX
milestone: M0N
status: pending approval | open | in-progress | merged | blocked
purpose: |
  One sentence: what this phase delivers and which Master Plan principles it inherits.
owner_team: <team-id>
co_owners: [<team-id>]
hyperscaler_practices_inherited: [working-backwards | design-doc | postmortem-blameless | 1ES-templated-pipelines | engineering-excellence | trunk-based-development | slsa-l2 | feature-flags-canary]
length_cap: 50
---
```

# P0N-<slug>: <one-line phase title>

## Purpose

One sentence stating what this phase delivers in present tense.

## Acceptance

- Numbered list of measurable acceptance criteria (≤5 items). Each row names a fitness lane OR a command OR an explicit `(advisory)` marker.

## Implementation Plans

- [`IP-001-<slug>.md`](IP-001-<slug>.md) — one-line summary — `<status>`
- [`IP-002-<slug>.md`](IP-002-<slug>.md) — one-line summary — `<status>`

## Estimated parallelism

`<N>` agents in parallel; serialization bottleneck = `<root Cargo.toml | shared kernel crate | none>`.

## Symbols touched (high level)

- `crates/oya-<context>-<role>/` family
- `contracts/<surface>.<format>`
- `docs/<canonical-doc>.md` (per `docs/DOC-CATALOG.md` trigger)

## Agent-handoff (icm event at phase complete)

```
icm store -t phase-handoff -c "P0N-<slug> complete at <git-sha>; IPs merged: <list>; next phase: P0N+1-<slug>; gate: <fitness lane>" -i high -k "M0N,P0N,handoff"
```
