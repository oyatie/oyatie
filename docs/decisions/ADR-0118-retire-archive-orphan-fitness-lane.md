---
id: ADR-0118
status: Accepted
deciders: council-architecture, axis-foundry
owner: axis-foundry
date: 2026-05-16
supersedes: [ADR-0052]
superseded_by: []
related: [ADR-0116, ADR-0110, ADR-0111, ADR-0112, ADR-0113]
purpose: Retire the one-time archive-orphan fitness lane and pre-grit archive payload after ADR-0116 establishes the Foundry pipeline (M01-P18) as the canonical VCS substrate.
---

# ADR-0118: Retire archive-orphan fitness lane

## Status

Accepted — 2026-05-16.

## Context

M01-P08-IP-008 created a temporary `archive-orphan` lane to verify that pre-cutover Bominal ultragoal state moved under `bominal/agents/ultragoal/archive/pre-grit-cutover-2026-05-12/` and that active originals disappeared. That gate was useful for the grit-era cutover boundary, but it is now a stale one-time cleanup lane.

ADR-0116 retires grit, rtk, icm, and vox from the prescribed agent-coordination surface. The Foundry pipeline (M01-P18) is now the canonical VCS substrate: isolated worktree branch, PR against `dev`, webhook receiver, router, admission gate, projected merge state, conflict kernel, merge queue, reviewer approval, and CI green before merge.

Keeping a live `archive-orphan` crate after that transition creates two problems:

1. It preserves a deleted pre-grit archive payload as if it were still operationally useful.
2. It makes an old grit-era invariant look like an active M01-P18 admission primitive.

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
- M01-P18 remains the only forward VCS/concurrent-work substrate after ADR-0116.
- Historical evidence can still cite IP-008 without implying that archive-orphan is active CI.

## Rejected alternatives

- **Keep the lane active with an empty archive set.** Rejected because an always-empty runner would be false mechanical confidence.
- **Rename the lane into an M01-P18 gate.** Rejected because projected-merge-state and conflict-kernel already own the replacement invariant.
- **Delete every historical mention.** Rejected because prior evidence and ADR-0052 need traceability for why the cutover artifact existed.

## Verification plan

- Confirm the retired crates and archive payload are removed from git.
- Confirm Cargo metadata resolves with the direct Rust 1.95.0 cargo path.
- Validate this ADR shape with the repo validator.
- Run targeted checks for the adjacent authoritative-tracked lane that now remains responsible for canonical-tree enforcement.

## Sunset / Reversal

Terminal retirement; no future sunset clause applies.

**Reversal procedure (if M01-P18 admission gate proves insufficient as the replacement enforcer):**

1. `git revert <merge-sha-of-PR-13>` — pure-deletion revert is mechanically clean; restores the kernel + app + 3 catalog yamls + workspace.members + archive payload in one atomic commit. `data_loss_class: none` because there is no state to reconcile.
2. Re-add CI lane invocation if it had been removed from `.github/workflows/`.
3. Re-add forbidden-operations.json FO-01/FO-07 enforcer ref (this fix-PR's Fix #6 commit cleared it; reverting that commit + this commit together restores the prior state).

**ADR-0108 lifecycle-policy waiver:** PR #13 bypassed the canonical sunset → deprecation → removal window (ADR-0108 default 30d + 90d). The waiver was implicit ("one-time pre-grit cutover hygiene with payload deleted in same ChangeSet — sunset window cannot produce new violations when inputs are deleted simultaneously"). Codifying that exception as a `one_time_lane: true` carve-out in ADR-0108 is filed as `F-ADR0108-ONETIME-LANE-CARVEOUT` (see registries/cross-cutting/fixuptasks.jsonl).

**Related cross-checks:** ADR-0052 (now Superseded by this ADR; previously the cutover inventory), ADR-0056 (12-layer enum — kernel+app deletion is enum-neutral), ADR-0108 (sunset-lifecycle automation; bypassed here, waiver above), ADR-0116 (Foundry pipeline canonical substrate).
