# R-DUAL-0615 — dual-home `oya/*` disposition inventory

**Bead:** `oyatie-oso.20`  
**Date:** 2026-08-05  
**Branch:** `agent/rr-dual-home-0615-20260805`  
**Head / origin/dev:** `a1bd1f14af0f4f5fdf766928e25d6503a0f3ac02`  
**Class:** mixed inventory (not bulk move) · **Capability span:** multi  
**Machine evidence:** [`r-dual-0615-oya-disposition-inventory-20260805.json`](./r-dual-0615-oya-disposition-inventory-20260805.json)

## ADR re-query (required)

Re-read on this `origin/dev` head:

| Authority | Status | Disposition relevant to this card |
|-----------|--------|-----------------------------------|
| **ADR-0615** | Accepted 2026-07-10; amends 0562 | Q2 emergency → `app/healthcare`; Q3 drive/recordings **confirm** `storage/facade/*`; Q4 imaging → `app/healthcare`; Q8 diagnostics → `app/health-diagnostics` |
| **ADR-0562** | Accepted (as amended) | Substrate/product split §3#5 / §6; membership absorb rules |
| **ADR-0614** | Accepted | Move-plan only for move-class bijections — **none opened** |
| **REORG-DOCTRINE** | process law 2026-08-05 | multi-cap OK as epic; one concern per PR; classes move\|refactor\|rewrite\|delete\|rebrand\|mixed; task uses **park** for deferred execute |

Registry `pending_relocations` already encodes emergency / imaging / diagnostics Batch-5 targets. Drive/recordings have **no** pending row because domain crates are confirmed at `storage/facade`.

## Inventory summary

| Path | Crates | Files | OWNERS file | Registry absorb (today) | Live registered home | Disposition |
|------|-------:|------:|-------------|-------------------------|----------------------|-------------|
| `oya/drive` | **0** | 83 | none (`axis-drive` in manifest) | `storage` | `storage/facade/drive` (`storage-drive-domain`, 518 LOC) | **park** |
| `oya/recordings` | **0** | 74 | none (`axis-recordings`) | `storage` | `storage/facade/recordings` (`storage-recordings-domain`, ~740 LOC) | **park** |
| `oya/emergency` | **0** | 47 | none | `comms` (**wrong**) | — | **park** → future **move** `app/healthcare` |
| `oya/imaging` | **0** | 35 | none | `storage` (**wrong**) | — | **park** → future **move** `app/healthcare` |
| `oya/diagnostics` | **0** | 31 | none | `observability` (**name collision**) | — | **park** → future **move** `app/health-diagnostics` |

All five trees are **non-empty scaffolds** (manifest, contracts, cedar, IaC, runbooks; emergency also has 13 OpenSLO files). None is a proven-empty dead path.

## Dual-home shapes

1. **Scaffold vs live domain crate (drive, recordings)**  
   - Live: `storage/facade/{drive,recordings}` — single-capability storage facades, **confirmed** ADR-0615 Q3.  
   - Scaffold: `oya/{drive,recordings}` — product-shaped design residue still listed under `storage.absorbs_current_dirs`.  
   - Future consumer products compose 2+ capabilities → `app/` when built; **do not** relocate domain crates into `app/`.

2. **Wrong absorb vs ruled app destination (emergency, imaging, diagnostics)**  
   - Zero crates at either home.  
   - Absorb retained only to avoid membership-lint orphaning until Batch-5 atomic relocate (`pending_relocations`).  
   - Diagnostics is a **separate** product (`app/health-diagnostics`), not an `app/healthcare` context.

## Disposition (this PR)

| Path | class now | class when executed | blast radius (execute) | prerequisite |
|------|-----------|---------------------|------------------------|--------------|
| `oya/drive` | **park** | mixed (keep facade; later app + absorb cleanup) | high if naive move of domain crate | storage Batch-5 / product lane + gate co-edit |
| `oya/recordings` | **park** | mixed | high if naive app-home of facade | same |
| `oya/emergency` | **park** | **move** | medium (gates, SLOs, multi-cloud IaC) | `specs/reorg/comms-move-plan.json` Batch-5 + `app/healthcare` layout |
| `oya/imaging` | **park** | **move** | medium | `specs/reorg/storage-move-plan.json` Batch-5 |
| `oya/diagnostics` | **park** | **move** | medium; mis-route risk into healthcare | `specs/reorg/observability-move-plan.json` Batch-5 |

### Explicit non-moves / non-deletes this PR

- **No path moves.**  
- **No deletes** — cedar-deploy-parity, product-protocol-policy, tier-classification, and capability-registry still reference these trees.  
- **No new move-plan** (ADR-0614 one-active rule; inventory-only concern).

## Live gate consumers (shared)

- `specs/capability-registry.json`  
- `specs/microservice-tier-classification.json`  
- `ci/facade/policy-deploy-parity/cedar-deploy-parity-policy.json`  
- `ci/facade/product-protocol-policy/product-protocol-policy.json` (emergency, imaging, diagnostics)  
- `registry/fixuptasks.jsonl` (emergency, imaging)

## Follow-on cards (not this PR)

1. `R-DUAL-0615-EMERGENCY-MOVE` — move-class, comms Batch-5  
2. `R-DUAL-0615-IMAGING-MOVE` — move-class, storage Batch-5  
3. `R-DUAL-0615-DIAGNOSTICS-MOVE` — move-class, observability Batch-5  
4. `R-DUAL-0615-DRIVE-RECORDINGS-ABSORB-CLEANUP` — mixed; keep facades; decide scaffold fate  

## Acceptance checklist

- [x] Inventory each tree (crate count, OWNERS, registry mapping, dual-home vs registered home)  
- [x] Disposition per tree with blast radius + prerequisite  
- [x] ADR-0615 re-query recorded  
- [x] evidence JSON + short markdown under `evidence/reorg/`  
- [x] Zero bulk moves / no unsafe deletes  
- [x] Draft PR + dual-critic packet under `.grok/programs/`  
