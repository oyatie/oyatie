# REORG-DUAL-GOVERNANCE-DEFERRED — disposition inventory

**Date:** 2026-08-06  
**Branch:** `agent/reorg-dual-governance-20260806`  
**origin/dev:** `1ff54ddc5`  
**Class:** mixed inventory only · **Net path debt this PR:** 0  
**Machine:** [`rr-dual-governance-residual-20260806.json`](./rr-dual-governance-residual-20260806.json)

## Ruling

**ADR-0615 §2 Q13:** Do **not** fold `oya/governance` into `compliance`. Zero-crate residue **decomposes**:

| Residue | Destination |
|---------|-------------|
| authority / policy-as-data / conformance | `governance/` meta-dir |
| gate / enforcement SLOs | `ci/observability/slos/` |
| `envoy-wasm-filter-latency-p99` | `gateway/observability/slos/` |
| `autosharding-events` | `cell/observability/slos/` |
| IaC | `iac/` (or operated capability) |
| runbooks | operated capability homes |

PR **#1589** burned `oya/compliance` and correctly **deferred** this leaf as multi-cap residue.

## Inventory (origin/dev)

| Field | Value |
|-------|--------|
| Path | `oya/governance` |
| Crates / Cargo.toml | **0 / 0** |
| Files | **152** (yaml 117 · md 18 · json 8 · cedar 6 · other) |
| Live dual | `governance/` already **62 crates / 214 files** (check + corpus) |

Largest residual slices: `iac/` 65 · `catalog/` 41 · `runbooks/` 14 · `slos/` 8.

## Live gate consumers (why not delete/rehome now)

1. `specs/capability-registry.json` — `compliance.absorbs_current_dirs` + `pending_relocations`
2. `specs/microservice-tier-classification.json` — `oya/governance/manifest.json` row
3. `ci/facade/policy-deploy-parity/cedar-deploy-parity-policy.json` — cedar helm template path

## Disposition

| Option | Decision |
|--------|----------|
| Wholesale → `compliance/` or `compliance/governance/` | **Rejected** (Q13) |
| Wholesale → `governance/` only | **Rejected** (multi-destination; would re-hide SLO/IaC debt) |
| Delete now | **Rejected** (gate consumers) |
| Mega multi-cap move this PR | **Rejected** (doctrine + task) |
| **Park + inventory + board card** | **This PR** |

## Board card (follow-on execute)

**`R-DUAL-GOVERNANCE-DECOMPOSE`** (W2+, mixed, multi-cap)

- Atomic Batch-5 split per decomposition map  
- Retarget three live consumers  
- Delete empty `oya/governance`  
- **net_path_debt under `oya/governance` ≤ 0** after execute  

## Explicit non-actions (this PR)

- No path moves / deletes  
- No second `*-move-plan.json`  
- No hand-edit `*.generated.json`  
- No product tree edits outside `evidence/reorg/`

## Acceptance

- [x] ADR-0615 Q13 re-query + absorb/pending_relocation proof  
- [x] Consumer list with retarget notes  
- [x] Decomposition map for residual subtrees  
- [x] Rejected dispositions with authority  
- [x] Board card `R-DUAL-GOVERNANCE-DECOMPOSE`  
- [x] net_path_debt this PR = 0  
