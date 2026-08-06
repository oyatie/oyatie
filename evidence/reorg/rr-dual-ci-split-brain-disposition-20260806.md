# REORG-DUAL-CI-DISPOSITION — ci split_brain inventory

**Date:** 2026-08-06  
**Branch:** `agent/reorg-dual-ci-disposition-20260806`  
**origin/dev:** `1ff54ddc5`  
**Class:** mixed inventory only · **Net path debt this PR:** 0  
**Machine:** [`rr-dual-ci-split-brain-disposition-20260806.json`](./rr-dual-ci-split-brain-disposition-20260806.json)

## Ruling

**ADR-0562:** `ci/` absorbs `cloud-ci` + `ci-controller` + `ci-tide` + `ci-webhook-gateway` as sub-modules.  
**ADR-0515:** singular admission / delivery fabric; gates already keystoned under `ci/facade/`.  
**NORTH-STAR durable home:** `ci/` (gates at `ci/facade/`; product services as `ci/<service>/` subdirs — not folded into facade).

**Move-plan singleton:** #1581 holds sole live `intelligence-remainder-move-plan.json` — **no second live move-plan** in this PR.

## Inventory (origin/dev)

| Path | Crates | Files | Class | Suggested dest |
|------|-------:|------:|-------|----------------|
| `ci/` | **58** | **390** | durable_home | *(already home: facade/ports/adapters)* |
| `oya/ci-controller/` | **4** | **24** | **move** (staged) | `ci/controller/` |
| `oya/ci-tide/` | **3** | **10** | **move** (staged) | `ci/tide/` |
| `oya/ci-webhook-gateway/` | **5** | **57** | **move** (staged) | `ci/webhook-gateway/` |

**Totals:** oya leaves **12 crates / 91 files** · **zero** zero-crate residual leaves under absorb paths.

**Already burned (non-absorb note):** `cloud/cloud-ci` = **0** tracked files (keystone → `ci/facade/*` via rename map #1578 history).

## Live consumers (why not delete/rehome now)

| Leaf | Consumer surface |
|------|------------------|
| all three | `specs/capability-registry.json` absorbs |
| all three | `ci/facade/module-membership` `legacy_root_freeze` census (12 crate dirs) |
| controller | `affected-target-set` BUCK targets; design-doc fixture |
| tide | freeze census only (lightest) |
| webhook-gateway | tier classification · product-protocol · authz-coverage · catalog YAML×5 · affected-target-set · Cargo.toml comment path |

## Disposition

| Option | Decision |
|--------|----------|
| Mega-move all `oya/ci-*` → `ci/*` this PR | **Rejected** (task + singleton + blast radius) |
| Delete without rehome | **Rejected** (crate-bearing + consumers) |
| Second live `*-move-plan.json` | **Rejected** (#1581 owns slot) |
| Fold services into `ci/facade/` | **Rejected** (facade = gates; services = apps) |
| Zero-crate residual rehome | **N/A** (no zero-crate absorb leaf) |
| **Park + inventory + board cards** | **This PR** |

## Board cards (follow-on execute — after singleton free)

1. **`R-DUAL-CI-TIDE-MOVE`** — smallest (3c/10f) → `ci/tide/`  
2. **`R-DUAL-CI-CONTROLLER-MOVE`** — 4c/24f + iac → `ci/controller/`  
3. **`R-DUAL-CI-WEBHOOK-GATEWAY-MOVE`** — heaviest consumer surface → `ci/webhook-gateway/`  

Each leaf: move-plan (when singleton free) or single-concern git-mv + consumer retarget; **net_path_debt under source path ≤ 0**; shrink freeze census; drop absorb entry. Final absorb collapse → `["ci"]` only.

## Explicit non-actions (this PR)

- No path moves / deletes  
- No second `*-move-plan.json`  
- No hand-edit `*.generated.json`  
- No crate renames / de-brand  
- No product tree edits outside `evidence/reorg/`

## Acceptance

- [x] Per-path file/crate inventory on origin/dev  
- [x] Consumer list with retarget notes  
- [x] All oya leaves crate-bearing → disposition-only OK  
- [x] Board cards + recommended execute order  
- [x] net_path_debt this PR = 0  
- [x] No conflict with #1581 move-plan singleton  
