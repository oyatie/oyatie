---
id: ADR-0118
status: Accepted
deciders: council-architecture, axis-foundry
owner: axis-foundry
date: 2026-05-16
supersedes: [ADR-0052]
superseded_by: []
related: [ADR-0116, ADR-0110, ADR-0111, ADR-0112, ADR-0113]
purpose: Retire the one-time archive-orphan fitness lane and pre-grit archive payload after ADR-0116 establishes the Foundry pipeline (M-CC-P11) as the canonical VCS substrate.
---

# ADR-0118: Retire archive-orphan fitness lane

## Status

Accepted — 2026-05-16.

## Context

M-CC-P01-IP-008 created a temporary `archive-orphan` lane to verify that pre-cutover Bominal ultragoal state moved under `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/` and that active originals disappeared. That gate was useful for the grit-era cutover boundary, but it is now a stale one-time cleanup lane.

ADR-0116 retires grit, rtk, icm, and vox from the prescribed agent-coordination surface. The Foundry pipeline (M-CC-P11) is now the canonical VCS substrate: isolated worktree branch, PR against `dev`, webhook receiver, router, admission gate, projected merge state, conflict kernel, merge queue, reviewer approval, and CI green before merge.

Keeping a live `archive-orphan` crate after that transition creates two problems:

1. It preserves a deleted pre-grit archive payload as if it were still operationally useful.
2. It makes an old grit-era invariant look like an active M-CC-P11 admission primitive.

## Decision

Retire `archive-orphan` as an executable fitness lane.

The retirement removes:

- `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/`
- `crates/oya-foundry-fitness-archive-orphan-kernel`
- `tools/oya-foundry-fitness-archive-orphan-app`
- workspace members for both retired crates
- catalog entries for the retired kernel/app capability

The retirement keeps a small historical lane record at `docs/fitness-lanes/archive-orphan.md` and `.omc/fitness-lanes/archive-orphan.md` so IP-008 evidence remains explainable without keeping executable code.

## Naming justification

Filename `ADR-0118-retire-archive-orphan-fitness-lane.md` uses the next free local ADR number after ADR-0117; `retire` is the lifecycle verb matching the action, and `archive-orphan-fitness-lane` is the exact historical lane id plus artifact class being removed.

## Consequences

- The repo no longer carries a one-time pre-grit archive payload.
- Cargo workspace discovery no longer includes the archive-orphan kernel/app packages.
- M-CC-P11 remains the only forward VCS/concurrent-work substrate after ADR-0116.
- Historical evidence can still cite IP-008 without implying that archive-orphan is active CI.

## Rejected alternatives

- **Keep the lane active with an empty archive set.** Rejected because an always-empty runner would be false mechanical confidence.
- **Rename the lane into an M-CC-P11 gate.** Rejected because projected-merge-state and conflict-kernel already own the replacement invariant.
- **Delete every historical mention.** Rejected because prior evidence and ADR-0052 need traceability for why the cutover artifact existed.

## Verification plan

- Confirm the retired crates and archive payload are removed from git.
- Confirm Cargo metadata resolves with the direct Rust 1.95.0 cargo path.
- Validate this ADR shape with the repo validator.
- Run targeted checks for the adjacent authoritative-tracked lane that now remains responsible for canonical-tree enforcement.
