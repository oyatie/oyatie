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
| `hr-move-plan.json` | **LIVE executable** — HR absorb #2192 (`oya/hr` → `app/hr`) |
| `ci-keystone-rename-map.json` | **rename SSOT** (not a move-plan) — gate_registration + disposition |
| `kernel-move-plan.BLOCKED.json` | **BLOCKED** — mechanical blockers |
| `ci-graph-additions.json` | Companion graph for historical keystone lockfile edges |

> **Spent leaf (no live plan file):** R-CAS-3A NativeLink storage rehome (`infra/nativelink` → `storage/adapters/nativelink/`) is **executed and promoted** — #1563 merged 2026-08-05, promoted `010c132ec`; completion evidence `evidence/completion/oyatie-oso.5-packet.md` (criteria_met=false only because the promoted-SHA CI run was cancelled; re-verify is post-merge hygiene, not a move lane). Plan file deleted as spent in PR #1954.

> **Spent leaf (no live plan file):** R-DUAL-CI-TIDE-MOVE (`oya/ci-tide` → `ci/tide/`) is applied in-tree; evidence `evidence/reorg/rr-dual-ci-tide-move-20260806.json`. Plan not kept LIVE because #1581 already holds the singleton for intelligence-remainder.

> **Spent leaf (2026-08-14):** REORG-INTEL-REMAINDER (G024) is applied in-tree: 78 crates `oya/intelligence/crates` → `intelligence/{core,adapters,facade}` + 78 catalog co-moves; evidence `evidence/reorg/rr-intel-remainder-execute-20260814.json`. The plan file was kept committed through the move PR for ADR-0563 relabel authentication and REMOVED by this follow-up cleanup PR after #1956 merged. The RR-MOVEPLAN-SINGLETON slot is now occupied by `hr-move-plan.json` (HR absorb #2192).
>
> **Wave25/26 closeout (2026-08-10):** PR #1620 (`1d3105277`) landed 29 residual rehomes; closeout evidence `evidence/reorg/rr-wave25-26-residual-consolidation-closeout-20260810.json`. Drafts #1580–#1608 superseded except **#1587** and **#1607** held on `oyatie-0s8` founder rulings — both resolved 2026-08-14 (see `evidence/reorg/rr-held-draft-dispositions-20260814.json`): #1587 superseded by #1620, #1607 superseded by integ/cloud PR #1938.

## North-star / anti-debt

See delivery-fabric `NORTH-STAR-SHAPE.md` when kit is on trunk. Short form:

- Reorg targets (`cloud/`, `oya/`, `infra/`, …): **reduce only**
- Process automation: **`.grok/`** (not new scripts under `infra/`)
- No hand-edit of `*.generated.json`

## Human pointers

Process notes live here under `specs/reorg/` only — **do not** dual-home a `docs/reorg/` tree (creates unowned total-accounting debt; ADR-0555).
