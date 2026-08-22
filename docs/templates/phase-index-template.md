---
doc_class: Template
template_id: TPL-PHASE
status: Accepted
date: 2026-05-12
purpose: |
enforcing_fitness_lane: governance-plan-hierarchy
owner_team: council-architecture
related:
  - .omc/plans/MASTERPLAN.md
  - docs/templates/milestone-index-template.md
  - docs/templates/implementation-plan-template.md
adrs_cited:
  - ADR-0054  # scaffold-claim pattern (symbols touched)
length_cap: 50
doc_status: published
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


```
```
