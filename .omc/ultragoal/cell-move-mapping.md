# Cell capability — SOUND face-mapping (move-6, from workflow wgw8601or, critic verdict SOUND)

Dispatch move-6 executor once move-5 (storage) post-merge oya-ci-required is GREEN. Worktree /Users/jasonlee/oyatie-worktrees/p8-cell (RECREATE on the post-move-5 dev tip before executing — it's currently at 8a0c9dbde). cell = ADR-0280 leaf substrate / bootstrap floor; clean (NOT a violation source). 8 crates in 2 crate-bearing dirs (cloud/cloud-cell=6, cloud/cloud-capacity=2; cell-lifecycle + cell-rebalancer have 0 crates -> phase-2). All substrate -> core/ + ports/, NO facade (cell is internal, not sold).

## Final mapping (8 crates) — ONE REFINEMENT applied vs the workflow: cell/core/cell -> cell/core/routing (cargo cell-routing) to honor de-dup-path-doubling naming grammar + it IS the "Cell-routing kernel"
| old_crate | new_path | cargo | lib (snake) | face |
|---|---|---|---|---|
| oya-cell-domain | cell/core/routing | cell-routing | cell_routing | core |
| oya-cloud-region-domain | cell/core/region | cell-region | cell_region | core |
| oya-cloud-region-api | cell/ports/region | cell-region-api | cell_region_api | ports |
| oya-regional-pack-domain | cell/core/regional-pack | cell-regional-pack | cell_regional_pack | core |
| oya-regional-pack-api | cell/ports/regional-pack | cell-regional-pack-api | cell_regional_pack_api | ports |
| oya-cloud-cell-app | cell/ports/cell-bind | cell-bind-api | cell_bind_api | ports |
| oya-cloud-capacity-kernel | cell/core/capacity | cell-capacity | cell_capacity | core |
| oya-cloud-capacity-domain | cell/core/capacity-commercial | cell-capacity-commercial | cell_capacity_commercial | core |

## Face reasoning
All cell crates are the internal cellular-topology SUBSTRATE -> core/ (engines/kernels) + ports/ (inbound *-api boundaries). NO facade (cell is not a sold product; ADR-0280 leaf). cloud-cell-app is a no-dep LIB (not a composition-root bin) holding bind DTOs + a pure cell-lifecycle state machine proving the cloud-cell-bind-v1 OpenAPI contract -> ports/cell-bind (inbound API/binding boundary), NOT facade, NOT core/app. Intra edges (all legal): cell-region->cell-routing (core<-core), cell-region-api->cell-region (ports->core), cell-regional-pack-api->cell-regional-pack (ports->core), cell-capacity-commercial->cell-region (core<-core).

## EXTERNAL DEPENDENTS to rewrite (codemod) — 14, the biggest blast radius yet
oya-cloud-region-domain (->cell-region, lib cell_region) has 13 cross-capability consumers, ALL importing `oya_cloud_region_domain::` in src (lib-name cascade -> cell_region::):
  oya/application/crates/oya-application-app (also deps oya-regional-pack-* -> cell-regional-pack*), compute/core/domain, compute/core/resource, compute/core/dcops, observability/core/aggregate, cloud/cloud-billing/crates/oya-cloud-billing-domain, cloud/cloud-data/crates/oya-cloud-data-domain, cloud/cloud-finops/crates/oya-cloud-finops-domain, cloud/cloud-iam/crates/oya-cloud-iam-domain, cloud/cloud-kms/crates/oya-cloud-kms-api, cloud/cloud-kms/crates/oya-cloud-kms-domain, cloud/cloud-marketplace/crates/oya-cloud-marketplace-domain, cloud/cloud-network/crates/oya-cloud-network-domain, cloud/cloud-storage/crates/oya-cloud-storage-domain (NOTE: storage moved in move-5 -> after move-5 lands this is storage/core/domain; the codemod targets the live tree so it'll find it at its then-current path).
Codemod rewrites Cargo.toml path-deps + dep-keys + BUCK //labels + Rust use/extern (oya_cloud_region_domain::->cell_region:: etc.) for ALL. The review MUST verify all 14 (a half-rewritten dependent compiles-by-luck = the danger).

## Brand residue: scout reported none in the 8 cell crates (confirm). Move protocol: commit ONE specs/reorg/cell-move-plan.json, regenerate manifest, hard-gate, contract interactions (members cell/*/*, registry absorbs->cell, membership+acyclicity add cell), born-accounting (cell/OWNERS axis used by cloud-cell, ADR §10.x VERBATIM paths, reachability seeds; stage OWNERS before census), full gate suite GREEN, forbidden_* 0 regression, grep-clean.
