---
doc_class: ADR
adr_id: ADR-0103
title: Grit cutover inventory of legacy primitives
status: Superseded
doc_status: published
owner: council-architecture
deciders: council-architecture + axis-foundry
date: 2026-05-14
supersedes: []
superseded_by: [ADR-0116, ADR-0363]
supersession_note: "grit/icm sanctioned-VCS + ban-git context retired; superseded by ADR-0116 (retire external agent-coordination tooling) and ADR-0363 (agentic-VCS retired). D-DISPOSITIONS-RATIFIED: SUPERSEDE-9-clean, C-6."
relates_to:
  - ADR-0052-inventory-grit-cutover.md
  - ADR-0054-grit-scaffold-claim-pattern.md
  - /specs/master-plan-sequencing.json
renumbered_from: ADR-0052
renumbered_on: 2026-05-15
renumber_reason: "ID-collision with ADR-0052-inventory-grit-cutover.md (Canonical Inventory Ledger for the grit/icm Cutover, dated 2026-05-12). Both ADRs cover the grit cutover but at different abstraction levels — this one is the high-level legacy-primitive replacement matrix; the older ADR-0052 is the operational ledger. Renumbered to ADR-0103 (next free slot) so both surface in the index."
---

# ADR-0103: Grit cutover inventory of legacy primitives

## Context

Prior to the agentic-pipeline cutover (M01-P08), agents reached into the
repository via direct `git`/`gh` commands, hand-rolled bash scripts, and
ad-hoc lock files. The cutover replaces those primitives with the
sanctioned trio `grit + icm + oya-tooling-agent-read`. Closing out the
cutover requires a single auditable record of what was inventoried, what
was retired, and what compatibility shims remain.

## Decision

Adopt this ADR as the canonical inventory of legacy primitives addressed
by the grit cutover, with sanctioned replacements and (where applicable)
retirement timing.

| Legacy primitive | Replacement | Status |
|---|---|---|
| Direct `git` from agents | `grit claim`/`grit done` + `oya-tooling-agent-read log/diff/pr-view` | Banned (banned-primitives lane enforces) |
| Direct `gh` from agents | `oya-tooling-agent-read pr-view`/`pr-comments` | Banned |
| `git rebase` / `git merge` by agents | Controller-owned merge queue (M01-P07 IP-007) | Banned |
| Local bash lock files | grit claim → work → done state machine | Retired |
| `cargo run -p oya-dev-cli -- check ...` aggregate commands | Per-lane `oya-dev-cli -- gate validate <lane>` | Retired (scripts/check.sh:79) |
| Pre-cutover monolithic checklists | ChangeSet-sized IP files under `.omc/plans/milestones/*/phases/*/IP-*.md` | Retired |
| Markdown-only plans | Machine-readable JSON via PHASE-5 migration | In-progress (markdown-retirement-policy) |

## Compatibility window

`grit` remains the compatibility shim for repo-state transitions until
M01-P07 promotion-controller acceptance lifts the waiver
(`gitops-vcs-replacement.json` §moved_earlier_in_masterplan).

## Consequences

- **Banned-primitives lane** (`oya-governance-banned-primitives-kernel`) checks for direct `git`/`gh` invocations in fenced agent-instruction blocks.
- **Scaffold-claim fallback** (ADR-0054) is the documented ICM path when grit FK errors block a claim.
- Any reintroduction of a banned primitive requires a new ADR superseding the relevant row above.

## Linus good-taste row

Special cases eliminated by this ADR:
- One inventory row per legacy primitive — no scattered "are we still allowed to call git here?" arguments.
- Replacement column is mandatory — banning a primitive without naming its replacement is rejected at PR review.
- Compatibility window is explicit and ADR-anchored — agents cannot indefinitely fall back to legacy paths "just this once."
