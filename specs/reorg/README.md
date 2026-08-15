# `specs/reorg/` — move recipes (north-star reorg)

**Not merge authority.** Doctrine: live ADR-0614 + capability ADRs on `origin/dev`, with the
runtime-neutral delivery mirror at `templates/portable-swarm-doctrine.md`.

## What belongs here

| Artifact | Role |
|----------|------|
| `*-move-plan.json` | **Executable** move bijection for a **live** rehome lane |
| `*-move-plan.PARKED.json` | Authored design, **not** executable until unparked |
| `*-move-plan.BLOCKED.json` | Authored but fail-closed until blockers clear |
| `*-graph-additions.json` | Lock/graph companion for a plan (not a second move-plan) |
| `move-manifest.generated.json` | **Not tracked** (ADR-0614) — materialize on demand |

## Singleton rule (RR-MOVEPLAN-SINGLETON)

At most **one** committed file matching:

```text
specs/reorg/*-move-plan.json
```

(excluding names containing `.PARKED.` or `.BLOCKED.`) may exist as an **executable** live rehome plan.

- Multiple PARKED/BLOCKED plans are fine.
- Starting a new rehome: **park or finish** the current live plan first, then commit the next.
- Process/product debt must **not** grow under `cloud/`, `oya/`, `infra/`, `libs/`, `tools/` except net-negative rehomes.

Enforced by: `ci/facade/baseline-ratchet` test `reorg_at_most_one_executable_move_plan` (this PR).

## Inventory (maintain when renaming)

| File | Status |
|------|--------|
| `intelligence-remainder-move-plan.json` | **LIVE** — G024 remainder rehome (78 crates under `oya/intelligence`); sole executable move-plan |
| `ci-keystone-rename-map.json` | **executed rename record** (not a move-plan) — gate_registration + disposition consume rows for still-live gates; retired rows remain migration provenance |
| `kernel-move-plan.BLOCKED.json` | **BLOCKED** — mechanical blockers |
| `ci-graph-additions.json` | Companion graph for historical keystone lockfile edges |

> **Spent leaf (no live plan file):** R-CAS-3A NativeLink storage rehome (`infra/nativelink` → `storage/adapters/nativelink/`) is **executed and promoted** — #1563 merged 2026-08-05, promoted `010c132ec`; completion evidence `evidence/completion/oyatie-oso.5-packet.md` (criteria_met=false only because the promoted-SHA CI run was cancelled; re-verify is post-merge hygiene, not a move lane). Plan file deleted as spent in PR #1954.

> **Spent leaf (no live plan file):** R-DUAL-CI-TIDE-MOVE (`oya/ci-tide` → `ci/tide/`) is applied in-tree; evidence `evidence/reorg/rr-dual-ci-tide-move-20260806.json`. Plan not kept LIVE because #1581 already holds the singleton for intelligence-remainder.
>
> **Wave25/26 closeout (2026-08-10):** PR #1620 (`1d3105277`) landed 29 residual rehomes; closeout evidence `evidence/reorg/rr-wave25-26-residual-consolidation-closeout-20260810.json`. Drafts #1580–#1608 superseded except **#1587** and **#1607** held on `oyatie-0s8` founder rulings.

## North-star / anti-debt

See the owning ADRs and `specs/integ-branch-envelopes.json#reorg_debt_freeze`. Short form:

- Reorg targets (`cloud/`, `oya/`, `infra/`, …): **reduce only**
- Process automation: owned Rust under **`ci/process-kit/`** (not agent dot-directories or new
  scripts under `infra/`)
- No hand-edit of `*.generated.json`

## Human pointers

Process notes live here under `specs/reorg/` only — **do not** dual-home a `docs/reorg/` tree (creates unowned total-accounting debt; ADR-0555).
