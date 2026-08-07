# `specs/reorg/` — move recipes (north-star reorg)

**Not merge authority.** Doctrine: `.grok/programs/REORG-DOCTRINE.md` (when kit lands) / live ADR-0614 + capability ADRs on `origin/dev`.

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
| `ci-keystone-rename-map.json` | **rename SSOT** (not a move-plan) — gate_registration + disposition |
| `nativelink-storage-move-plan.PARKED.json` | **PARKED** — CAS 3A; unpark after G039 + this live plan finished/parked |
| `kernel-move-plan.BLOCKED.json` | **BLOCKED** — mechanical blockers |
| `ci-graph-additions.json` | Companion graph for historical keystone lockfile edges |

> **Spent leaf (no live plan file):** R-DUAL-CI-TIDE-MOVE (`oya/ci-tide` → `ci/tide/`) is applied in-tree; evidence `evidence/reorg/rr-dual-ci-tide-move-20260806.json`. Plan not kept LIVE because #1581 already holds the singleton for intelligence-remainder.

## North-star / anti-debt

See delivery-fabric `NORTH-STAR-SHAPE.md` when kit is on trunk. Short form:

- Reorg targets (`cloud/`, `oya/`, `infra/`, …): **reduce only**
- Process automation: **`.grok/`** (not new scripts under `infra/`)
- No hand-edit of `*.generated.json`

## Human pointers

Process notes live here under `specs/reorg/` only — **do not** dual-home a `docs/reorg/` tree (creates unowned total-accounting debt; ADR-0555).
