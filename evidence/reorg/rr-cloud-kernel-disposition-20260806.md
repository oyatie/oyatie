# RR-CLOUD-KERNEL-DISPOSITION — multi-home inventory (plan-first)

**Date:** 2026-08-06  
**Branch:** `agent/reorg-cloud-kernel-disposition-20260806`  
**origin/dev:** `1ff54ddc5`  
**Class:** mixed inventory only · **Net path debt:** 0 (evidence under `evidence/reorg/`)  
**Machine:** [`rr-cloud-kernel-disposition-20260806.json`](./rr-cloud-kernel-disposition-20260806.json)

## Why this packet

`cloud/cloud-kernel` is a **21-`Cargo.toml` nested bare-metal workspace** (20 packages + workspace root) dual-homed with `kernel/` (ADR-0611 asterinas). Prior execute attempt is parked as [`specs/reorg/kernel-move-plan.BLOCKED.json`](../../specs/reorg/kernel-move-plan.BLOCKED.json) (B1–B3). This W2 card inventories every package, assigns a durable home (`kernel/` vs rejected `os/` / `compute/`), and recommends **staged move leaves** — **no crate moves in this PR**.

## Scope counts (origin/dev tip)

| Metric | Value |
|--------|------:|
| `Cargo.toml` under `cloud/cloud-kernel` | **21** |
| Package crates | **20** |
| Workspace members | 7 |
| Workspace `exclude` listed | 8 |
| Nested freestanding (not in exclude list) | 5 |
| Tree files | ~170 |

### Workspace members (explicit future moves)

| Package | Face | Suggested path | Suggested cargo name |
|---------|------|----------------|----------------------|
| `oya-cloud-kernel-hal-kernel` | **ports** | `kernel/ports/hal` | `kernel-hal-kernel` |
| `oya-cloud-kernel-frame-kernel` | **core** | `kernel/core/frame` | `kernel-frame-kernel` |
| `oya-cloud-kernel-ksync-kernel` | **core** | `kernel/core/ksync` | `kernel-ksync-kernel` |
| `oya-cloud-kernel-user-layout-kernel` | **core** | `kernel/core/user-layout` | `kernel-user-layout-kernel` |
| `oya-cloud-kernel-app` | **core** | `kernel/core/app` | `kernel-app` |
| `oya-cloud-kernel-arch-aarch64-adapter` | **adapters** | `kernel/adapters/arch-aarch64` | `kernel-arch-aarch64-adapter` |
| `oya-cloud-kernel-arch-x86-64-adapter` | **adapters** | `kernel/adapters/arch-x86-64` | `kernel-arch-x86-64-adapter` |

### Nested ride-along (13 packages — not independent leaves)

Host harness + freestanding EL0/ring-3 user programs under the two arch adapters (`tests-host`, `user-*`, `fsbase-worker-*`). They **ride with the parent adapter directory**; promoting them to independent move rows breaks `include!` / `include_bytes!` relative couplings.

## Durable home ruling

| Candidate | Verdict | Why |
|-----------|---------|-----|
| **`kernel/`** | **Accepted durable home** | Capability registry `meta_dir: kernel/`; `current_dirs: [cloud/cloud-kernel, kernel]`; BLOCKED plan face map; rung-0 framekernel |
| **`os/`** | **Rejected** | Talos-class node OS (rung 1): apid/block/archiver… — already absorbed `cloud-os` |
| **`compute/`** | **Rejected** | Multi-tenant compute faces (vm/k8s/functions) + provider adapters |

**Park now.** Safe delete now: **false** (not zero-crate residual).

Backlog card `RR-CLOUD-KERNEL-DEL` as *live-crate deletion* is **rejected** by this inventory. Deletion is only **S5 zero-crate residual** after successful rehome + consumer updates.

## Blockers still open (from BLOCKED plan)

1. **B1** — `kernel/` already hosts incompatible asterinas nested workspace (edition / `.cargo` bare-metal target / rust-toolchain nightly vs stable).  
2. **B2** — Codemod does not rewrite `[workspace.dependencies]` path deps (silent green risk).  
3. **B3** — `include!` / `include_bytes!` path-ups + tests-host literals naming `oya-cloud-kernel-user-layout-kernel/`.

## Staged move leaves (execute later)

| Stage | Class | Title | Independent leaf now? |
|-------|-------|-------|------------------------|
| **S0** | inventory | This PR | n/a (docs only) |
| **S1** | refactor (tooling) | Fix codemod `[workspace.dependencies]` (B2) | tooling PR |
| **S2** | refactor | include! path rewrite or tests-host decoupling (B3) | tooling/refactor PR |
| **S3** | mixed | Resolve B1 coexistence — **prefer S3-A nested subworkspace under `kernel/`** (e.g. `kernel/kuberos/`) over forcing single-valued toolchain merge | decision + optional small move |
| **S4** | move | **Atomic** kuberos rehome (one move-class PR). Sub-leaves L-ports / L-core-pure / L-core-frame-app / L-adapters+nested are **ordering guidance inside one plan**, not sequential half-moved dual-homes | no partial member leaves until workspace root moves |
| **S5** | delete | **Zero-crate residual only** — delete empty `cloud/cloud-kernel` + drop registry absorb | only when package count == 0 |

### Explicit non-leaves

- No single nested `user-*` package move  
- No move to `os/` or `compute/`  
- No execute of `kernel-move-plan.BLOCKED` until B1–B3 clear  
- No new active `*-move-plan.json` in this PR  

## Explicit non-actions (this PR)

- No path moves / bulk rehomes  
- No deletes  
- No new move-plan  
- No CAS / RE / runners / k8s port coupling  

## Acceptance

- [x] 21 `Cargo.toml` inventoried with suggested durable home  
- [x] `os/` and `compute/` explicitly rejected with rationale  
- [x] Staged move leaves recommended; zero crate moves  
- [x] Blockers B1–B3 re-stated with live evidence  
- [x] Evidence under `evidence/reorg/` only  

