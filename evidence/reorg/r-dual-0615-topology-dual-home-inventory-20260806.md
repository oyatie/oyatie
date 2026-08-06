# R-DUAL-0615-FOLLOW — topology dual-home inventory (plan-first)

**Date:** 2026-08-06  
**Branch:** `agent/reorg-dual-topo-inventory-20260806`  
**origin/dev:** `c7f60a9db`  
**Class:** mixed inventory only · **Net debt:** 0 (evidence only under `evidence/reorg/`)  
**Machine:** [`r-dual-0615-topology-dual-home-inventory-20260806.json`](./r-dual-0615-topology-dual-home-inventory-20260806.json)

## Why this packet

Complements the five-tree scaffold inventory (drive/recordings/emergency/imaging/diagnostics in `r-dual-0615-oya-disposition-inventory-20260805.*`, landing via ADR disposition train). This packet inventories **capability topology duals**: `oya/<cap>` coexisting with a registered top-level and/or `cloud/cloud-<cap>` home.

## FACE-DECOMMIT re-verify (0613–0616)

| Check | Result |
|-------|--------|
| Tracked `*.generated.json` on origin/dev | **0** |
| Prior residual PR | #1565 merged |
| Violations re-introduced | **none** |

RR-FACE-DECOMMIT mechanical debt remains **clean**; no fix PR required. Card may close after this evidence lands.

## Topology duals (park — no execute)

| Capability | Homes (crates / files) | Disposition |
|------------|------------------------|-------------|
| compliance | `oya/compliance` 0c/83f · `compliance/` 7c/37f | **park** — oya scaffold vs live `compliance/` |
| governance | `oya/governance` 0c/152f · `governance/` 62c/214f | **park** — large live home; oya scaffold |
| intelligence | `oya/intelligence` **78c**/742f · `intelligence/` 51c/277f · `cloud/cloud-intelligence` 0c/38f | **park** — multi-home; remainder plan after move-plan singleton (#1576) |
| marketplace | `oya/marketplace` 0c/101f · `marketplace/` 5c/200f | **park** |
| observability | `oya/observability` 0c/118f · `observability/` 5c/41f | **park** — clinical `oya/diagnostics` is separate (prior inventory) |
| tasks | `oya/tasks` 0c/74f · `tasks/` 0c/120f | **park** — both zero crates; gate consumers not yet cleared for delete |

## Explicit non-actions

- No path moves / bulk rehomes  
- No deletes (gate/registry consumers may still reference scaffolds)  
- No new `*-move-plan.json` (singleton rule; #1576 parks nativelink)  
- No new surfaces under `cloud/` `oya/` `infra/` `libs/` `tools/` `microservices/`

## Follow-on execute cards (after inventory merge + singleton)

1. `R-DUAL-TOPO-INTEL` — intelligence multi-home reduce (largest)  
2. `R-DUAL-TOPO-GOVERNANCE` / `MARKETPLACE` / `COMPLIANCE` / `OBSERVABILITY` / `TASKS` — one capability per PR  
3. Batch-5 scaffold moves from prior five-tree inventory (emergency/imaging/diagnostics)

## Acceptance

- [x] origin/dev re-query + file/crate counts  
- [x] FACE re-verify tracked generated == 0  
- [x] Complementary to 2026-08-05 five-tree inventory  
- [x] Zero execute; net path debt 0  
