# RR-MOVEPLAN residual — `ci-move-plan` load-bearing retirement PLAN

**Bead:** `oyatie-oso.27`  
**Date:** 2026-08-05  
**Branch:** `agent/rr-moveplan-residual-20260805`  
**origin/dev:** `a1bd1f14af0f4f5fdf766928e25d6503a0f3ac02`  
**Class:** refactor · **Subclass:** PLAN-only inventory  
**Machine evidence:** [`ci-move-plan-load-bearing-residual-20260805.json`](./ci-move-plan-load-bearing-residual-20260805.json)

## Why this note exists

PR **#1567** (bead `oyatie-oso.19`) retires eight spent move-plans and **keeps** `specs/reorg/ci-move-plan.json` because it is still a **live SSOT reader** for CI gate registration — not because the keystone move is unfinished.

Hygiene inventory R2/R4 already said: keep load-bearing; optional later migrate then delete. This PR is that **PLAN** only.

## Authority re-query

| Source | Ruling |
|--------|--------|
| **REORG-DOCTRINE** | delete class OK only with consumer-absence proof |
| **ADR-0614** | move-plan is a move recipe; do not leave runtime/tests coupled forever |
| **ADR-0562/0563** | ci keystone rename bijection + `ci-graph-additions` lockfile companion |
| **cfcbcf967** | restored plan after unsafe delete `c538536bd` — Chesterton fence |

## Current residual surface (`specs/reorg/`)

After #1567 lands, residual should be:

| Path | Role |
|------|------|
| `ci-move-plan.json` | **Load-bearing** rename SSOT (this note) |
| `ci-graph-additions.json` | Companion lockfile graph contract (not a move-plan) |
| `intelligence-remainder-move-plan.PARKED.json` | PARKED residual |
| `kernel-move-plan.BLOCKED.json` | BLOCKED design |

`ci-move-plan` is **landed** (46 moves + 1 artifact; all old paths GONE / new PRESENT) and **inert for codemod active selection** via `plan_is_landed`, but still **hard-consumed**.

## Hard consumers (must clear before delete)

1. **`ci/facade/baseline-ratchet/tests/gate_registration.rs` → `ci_move_new_dir`**  
   - Reads the plan JSON.  
   - Maps `old_cargo_name` → `new_path` directory tail under `ci/facade/`.  
   - Required because de-brand renames are **semantic** (~32 non-prefix maps), e.g.  
     `oya-cloud-ci-accounting-registry-app` → `ci-artifact-inventory-registry`.  
   - **Deleting the plan without migration fails cloud-ci-firewall / gate-registration meta-tests.**

2. **`libs/oya-ci-config/src/bundled/gate-disposition.json`**  
   - `_stub_gates.topology-manifest-contract.reference` → `specs/reorg/ci-move-plan.json`.

3. **`specs/reorg/ci-graph-additions.json`** (companion, not a hard runtime reader of the plan file path in code, but operational peer contract).

## Retirement PLAN (not executed here)

| Stage | Class | Work |
|-------|-------|------|
| **S0** | inventory | **This PR** — consumers + plan recorded |
| **S1** | refactor | Extract bijection to an owned artifact **not** named `*-move-plan.json` under `specs/reorg/`; repoint `ci_move_new_dir`; green gate-registration |
| **S2** | delete | Delete `ci-move-plan.json` after consumer-absence proof; optional forensics archive under `evidence/reorg/` |
| **S3** | delete_or_park | Decide `ci-graph-additions.json` retention (historical ADR-0563 vs evidence archive) |

### S1 artifact options (pick one in execute PR)

| Opt | Path | Notes |
|-----|------|-------|
| **A** | `ci/facade/baseline-ratchet/testdata/ci-gate-rename-map.json` | Colocated with sole code reader |
| **B** | `libs/oya-ci-config/src/bundled/ci-gate-rename-map.json` | Next to gate-disposition |
| **C** | Inline const table in `gate_registration.rs` | No extra file; noisy review |

**Do not** re-home the map as another `specs/reorg/*-move-plan.json` (re-enters active glob / singleton surface).

## Explicit non-goals

- No delete of `ci-move-plan.json` or `ci-graph-additions.json` in this PR  
- No codemod apply; no new move-plan  
- No `gate_registration.rs` code change  
- No bulk multi-cap moves  
- No `RR-LIBS-DISPOSITION` inventory (separate card)

## Verify

- [ ] Diff is evidence-only under `evidence/reorg/`  
- [ ] Hard-consumer paths exist on `origin/dev`  
- [ ] Dual-critic APPROVE  
- [ ] `oya-ci-required` green  

## Follow-on bead suggestion

Execute S1+S2 as a **single-concern refactor+delete** PR after #1567 is on trunk (or stacked only if hygiene already merged). Keep blast radius to ci facade tests + optional oya-ci-config reference field.
