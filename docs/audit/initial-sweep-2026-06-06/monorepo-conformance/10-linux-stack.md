---
title: Monorepo Conformance Audit — linux/stack (pilot)
status: Audit-findings
date: 2026-06-06
auditor: workflow-subagent (read-only)
checklist: ./00-policy-checklist.md
repo: /Users/jasonlee/Developer/linux/stack
verdict_legend: CONFORMS | NEEDS-RESHAPE | NEEDS-RENAME | VIOLATES | N/A
note: READ-ONLY audit. Every verdict is backed by a cited path. No files in /stack were modified.
---

# Monorepo Conformance Audit — `linux/stack`

## 0. What `/stack` actually is (evidence)

`ls /Users/jasonlee/Developer/linux/stack` →

| Sub-tree | Nature | Workspace? | Crates | Brand |
|---|---|---|---|---|
| `kernel/` | no_std framekernel (→ target `cloud/cloud-kernel`) | own `[workspace]` resolver=2, edition 2021, nightly-2026-02-28 | 7 members (`kernel,hal,frame,ksync,user_layout,arch-x86_64,arch-aarch64`) + ~10 excluded standalone user binaries (each its OWN `[workspace]`) | none (generic `kernel`/`hal`) |
| `kernel-usermode-tests/` | 6 standalone EL0 ring-3 test programs (`hello,init,exec,spawn,signal,clock`) | each a separate crate; NOT in any workspace | `user-hello`, `user-init`, … | generic |
| `operating-system/` | talos-* STD node OS (→ target `cloud/cloud-node-os`) | own `[workspace]` resolver=2, edition 2024, 1.96.0 | 45 members, all `talos-*` (+ bare `init`,`svc`,`talosctl`,`talos_init`) | **`talos-*` codename + `Kuberos` residue** |
| `kubernetes/` | 139-crate K8s+containerd port (44 `ctrd_` + 95 k8s) | own `[workspace]` resolver=**3**, edition 2024, 1.96.0 | 139 flat `crates/<crate>` | snake_case generic (`ctrd_seccomp`, `meta_v1`) |
| `talos-reference/` | **upstream Sidero Talos Go source** (`module github.com/siderolabs/talos`) | Go (`go.mod`/`go.work`), not Rust | n/a | vendored reference |

This is **four separate Cargo workspaces + one Go reference tree** sitting under one directory — NOT a single oyatie-policy monorepo. That fact drives most verdicts below.

---

## A. Canonical homes & topology

| # | Item | Verdict | Evidence / fit-work |
|---|---|---|---|
| 1 | Service code only at `{oya,cloud}/<service>/crates/<crate>/` | **VIOLATES** | No `oya/` or `cloud/` dirs anywhere. Code sits at `kernel/crates/*`, `operating-system/talos-*` (flat, no `crates/`), `kubernetes/crates/*`. Fit-work: relocate to `cloud/cloud-kernel/crates/*`, `cloud/cloud-node-os/crates/*`, `cloud/cloud-orchestrator/crates/*` (kernel→`cloud-kernel`, OS→`cloud-node-os`, k8s+ctrd→one or two cloud services). `git mv` per item 47. |
| 2 | Shared cross-cutting at `libs/<lib>/` | **VIOLATES** | No `libs/`. No `oya-check-*`, `oya-http-*`, `oya-data-boundary-kernel`. Fit-work: factor shared kernels (e.g. `user_layout`, k8s `apimachinery`-style crates `meta_v1`/`runtime_*`) into `libs/`. |
| 3 | `microservices/` forbidden | **CONFORMS** | No `microservices/` dir present. |
| 4 | Per-service colocation (PRD.md, README.md, decisions/, contracts/, specs/, catalog/, slos/, threat-model.md, src/crates/, evidence/) | **VIOLATES** | `find … -name PRD.md` = 0. No `decisions/`, `contracts/`, `slos/`, `catalog/`, `threat-model.md`, `evidence/multispectrum/` per service. Each tree has ad-hoc `*_REPORT.md`/`*_PLAN.md`/`ROADMAP.md` at root instead. Fit-work: author the per-service colocation set under each `cloud/<service>/`. |
| 5 | Cross-cutting artifacts central only | **N/A → NEEDS-RESHAPE** | No central `docs/decisions`, `docs/standards`, `specs/` inside `/stack` (those live in the parent `linux/docs`). Once relocated under oyatie root, the existing root-level `.md` docs must move to per-service or central per policy. |
| 6 | Aggregation indices generated | **VIOLATES** | `generate_buck_files.py` exists (k8s, OS) but no generated `registry/catalog/` or `docs/prds/INDEX.md`. Hand-authored `UPSTREAM_VERSIONS.md`, `ALLOWED_CRATES.md` stand in. |
| 7 | Packs only under `packs/` | **N/A** | No pack artifacts present. |

## B. Crate naming — BNF v4.1 + 13-layer enum

| # | Item | Verdict | Evidence / fit-work |
|---|---|---|---|
| 8 | `oya-` prefix mandatory | **VIOLATES (all 191 crates)** | `grep '^name=' … | grep oya` = 0 hits. Names are `kernel`, `hal`, `frame`, `talos-*`, `ctrd_*`, `meta_v1`, `cri_api_v1`, … Fit-work: rename every crate `oya-<microservice>(-bc)?-<layer>`. |
| 9 | BNF v4.1 grammar `oya-<ms>(-bc)?-<layer>` | **VIOLATES** | No crate matches grammar; many use snake_case (`ctrd_seccomp`, `meta_v1`, `util_json`) which is doubly illegal (snake + no layer). |
| 10 | `[package].name == dir basename (kebab)` | **NEEDS-RENAME** | Basenames match names today (`crates/version`→`version`), but snake_case dirs (`ctrd_seccomp`, `core_v1_proto`) violate "snake-free kebab". OS bare crates `init`/`svc`/`talosctl` live in `talos-init`/`talos-talosctl` dirs → name≠basename. |
| 11 | `[lib].name == snake_case(package.name)` | **NEEDS-RESHAPE** | Members don't set `[lib].name`; default applies. Re-derive after rename. |
| 12 | Layer suffix ∈ 13-value enum | **VIOLATES** | `grep` for `-(kernel|domain|adapter|app|usecase|rest|grpc|sdk|api|cli|worker|infrastructure)` suffix = 0. No crate carries a canonical layer suffix. |
| 13 | `oya-check-*` family + `*-adapter-<backend>` | **VIOLATES / absent** | No check family, no adapter-backend crates, no port/adapter split. |
| 14 | `tools/` crates explicit `-app` suffix | **NEEDS-RENAME** | `operating-system/tools/`, `kubernetes/scripts/`, `kernel/scripts/` exist; none `-app`-suffixed. |
| 15 | Microservice slot2 registered in `[workspace.metadata.oya.microservices]` | **VIOLATES** | No `[workspace.metadata.oya]` block in any of the 4 workspace roots. |

## C. Brand residue — FORBIDDEN

| # | Item | Verdict | Evidence / fit-work |
|---|---|---|---|
| 16 | `oyatie-*` cargo prefix forbidden | **CONFORMS** | No `oyatie-*` crate names (and no `oya-*` either — see #8). |
| 17 | Codename residue forbidden (`talos-*`, `kuberos`, `oyaoffice`, `oyago`, `oyapy`, `foundry-*`) | **VIOLATES (severe)** | (a) **`talos-*`** is the package prefix on all 45 OS crates (`talos-core`, `talos-apid`, … `operating-system/talos-*/Cargo.toml`) + ~388 `.rs` files reference `talos`. (b) **`Kuberos`** prior codename in PRODUCT SURFACE: `talos-secrets/src/bundle.rs` (5×), `talos-secrets/src/kubernetes_projection.rs` (6×) as `KUBEROS_*` env vars; `difftest/tests/differential.rs` (4×); `MIGRATION_REPORT.md` (51×); `prd-kuberos-rust-port.md` + many `.omx/context/kuberos-wave*.md` scratch files. Fit-work: rename `talos-*`→`oya-cloud-node-os-*`(layer); purge all `KUBEROS`/`Kuberos` tokens from source+docs. |
| 18 | No tautological rebrand residue (arrows / "retired terms" / "after rename") | **NEEDS-RESHAPE** | `MIGRATION_REPORT.md` + `PORT_REPORT.md` are migration-narrative docs likely carrying `X→Y` arrows; must be purged from live surface (would trip `oya-check-brand-residue`). |

## D. One-workspace invariants

| # | Item | Verdict | Evidence / fit-work |
|---|---|---|---|
| 19 | Exactly ONE `[workspace]` (repo root); no nested | **VIOLATES (hard)** | `grep '^[workspace]'` returns **16** Cargo.toml: 3 component roots + `kubernetes/third-party/rust` + **11 nested in `kernel/crates/arch-*/user-*-src`** (each excluded user binary is its own workspace). Fit-work: collapse into ONE root `[workspace]`; the standalone freestanding user binaries (different link addr / build-std) need either a sanctioned exclude mechanism or relocation outside the workspace tree. This is the single biggest structural blocker. |
| 20 | `resolver = "2"` | **NEEDS-RESHAPE** | kernel=`"2"` OK; OS=`"2"` OK; **kubernetes=`"3"`** (mismatch). After collapse to one root, pin `"2"` per policy. |
| 21 | Members inherit `version/edition/rust-version .workspace=true` (edition 2024, v0.1.0, rust 1.95.0) | **NEEDS-RESHAPE** | kernel uses `[workspace.package]` with **edition 2021, v0.0.1** (wrong edition+version). kubernetes crates set edition/version/rust-version **inline per crate** (e.g. `version`: `edition="2024"`, `rust-version="1.96"`, no `.workspace=true`). Policy wants edition 2024 / v0.1.0 / rust 1.95.0 inherited. |
| 22 | Member carries `publish=false` + `license="Apache-2.0"` + `[lints] workspace=true` | **VIOLATES** | k8s `crates/version/Cargo.toml`: NO `publish`, NO `license`, NO `[lints]`. kernel uses `license="MIT OR Apache-2.0"` (policy = Apache-2.0 only). No member sets `[lints] workspace=true`. |
| 23 | `[lib] doctest=false` | **VIOLATES** | No member sets `doctest=false` (grep clean). |
| 24 | Single `[workspace.dependencies]` seam + `registry/dependency-rationales.json` rows | **NEEDS-RESHAPE** | kernel has a `[workspace.dependencies]` seam (with prose justification in `ALLOWED_CRATES.md`); OS+k8s do not centralize. No `registry/dependency-rationales.json` anywhere → all deps would be orphans under ADR-0092. |

## E. Hexagonal clean architecture

| # | Item | Verdict | Evidence / fit-work |
|---|---|---|---|
| 25 | Inward-only dependency flow (13-layer matrix) | **NEEDS-RESHAPE** | No layer suffixes (#12) → matrix unenforceable. Dependency direction not modeled. |
| 26 | Port traits in `kernel`, impls in `adapter` | **VIOLATES** | No kernel/adapter split. `kernel/crates/kernel` is an OS microkernel (unrelated meaning); k8s/OS crates are flat ports of Go packages, not hexagonal. |
| 27 | `kernel` pure (no logic/IO/async), no_std where feasible | **PARTIAL / N/A** | The OS `kernel/crates/{hal,frame,arch-*}` ARE genuinely `#![no_std]` (good engineering) but this is a literal CPU kernel, not the policy's "ports kernel" layer. Semantic collision: policy `kernel` ≠ framekernel `kernel`. Reshape requires disambiguating naming. |
| 28 | `api`/`sdk` depend on `kernel` only | **VIOLATES/absent** | No `api`/`sdk` layer crates. |
| 29 | Only `app` (composition root) unrestricted; no `app→app` | **VIOLATES/absent** | No `app` layer. Composition is via bare bins (`talos-machined`, `talosctl`). |
| 30 | No cross-microservice imports (LEAN-A2); `public_layers` allowlist | **NEEDS-RESHAPE** | The 4 trees are independent workspaces today (no cross imports because not unified). Once merged under one workspace, cross-service edges (OS→k8s, OS→kernel ABI) must route through declared `public_layers`/adapter, not direct path deps. |

## F. Data governance & quality gates

| # | Item | Verdict | Evidence / fit-work |
|---|---|---|---|
| 31 | `data_class` on tenant/regulated kernel fields | **VIOLATES** | `grep data_class|DataClass` = 0 hits in `/stack`. Secrets/PKI data in `talos-secrets`, `talos-trustd` carry NO `DataClass`. Fit-work: annotate after `libs/oya-data-boundary-kernel` is introduced. |
| 32 | Statelessness (no module-level mutable state) | **UNKNOWN → NEEDS-RESHAPE** | Not checkable without `oya-check-statelessness`; OS uses `spin::Once`/lazy globals in kernel (acceptable in kernel layer, not presentation). |
| 33 | Shardability (`tenant_id` + RLS) | **N/A** | No multi-tenant DB schemas in scope yet. |
| 34 | Buildability bar (PRD≥5 stories, IP≥150 lines, ADR 3-alt/3-conseq, GREEN cites) | **VIOLATES** | No PRD/IP/ADR artifacts (#4). The `*_PLAN.md`/`*_SPEC.md` files are not the mandated buildability documents. |

## G. Supply chain & license policy

| # | Item | Verdict | Evidence / fit-work |
|---|---|---|---|
| 35 | Root `deny.toml` license allowlist | **VIOLATES** | `find -name deny.toml` = 0 in `/stack`. No cargo-deny policy. kernel license `MIT OR Apache-2.0` also off-allowlist intent (Apache-2.0). |
| 36 | `[bans]` openssl/old-time deny, rustls-only | **VIOLATES (no deny.toml)** | Cannot assert; must add. Need to scan `kubernetes`/`operating-system` deps for openssl. |
| 37 | `[sources]` crates.io-only, deny unknown | **VIOLATES (no deny.toml)** + **risk** | `kubernetes/third-party/` (121 MB) + `kubernetes/third-party/rust/Cargo.toml` is a **nested-workspace vendored** third-party tree (Reindeer-style) — needs reconciliation with `[sources]` crates.io-only doctrine. |
| 38 | `[advisories]` yanked=deny, empty ignore | **VIOLATES (no deny.toml)** | Add. |
| 39 | Vendor classification A/B/C + `registry/vendor-lockin-phaseout/index.json` | **VIOLATES** | No `vendor-lockin-phaseout` registry. `ALLOWED_CRATES.md` + `DEPENDENCY_VETTING.md` are prose stand-ins, not the A/B/C machine registry. |

## H. Buck2 build graph (Proposed, door:two-way)

| # | Item | Verdict | Evidence / fit-work |
|---|---|---|---|
| 40 | Per-crate `BUCK` (`rust_library`/`rust_binary`, crate_root, visibility, deps) | **PARTIAL / NEEDS-RESHAPE** | **kubernetes: 624 BUCK files; operating-system: 243 BUCK files; kernel: 0.** Best-aligned area in the whole repo, BUT BUCK targets point to local cells, not `//{oya,cloud,libs}/...:<crate>` + `third-party//:<dep>` path scheme. Generated via `generate_buck_files.py` (custom, not the sanctioned generator). |
| 41 | Third-party buckified by Reindeer; Cargo SSOT | **PARTIAL** | `kubernetes/third-party/` present with vendored rust tree; not confirmed Reindeer-shaped (`third-party/fixups/` not verified present). OS has none. |
| 42 | `.buckconfig` + `.buckroot`; cells root/prelude/toolchains/third-party | **PARTIAL** | kubernetes + operating-system BOTH have `.buckconfig`+`.buckroot`+`prelude`+`toolchains`. **kernel has none.** Two buckroots = two build graphs, not one. |
| 43 | NativeLink RBE only; Jenkins CI | **N/A (0% adopted)** | No CI config in scope; assert nothing. |

## I. Agent / contribution protocol

| # | Item | Verdict | Evidence / fit-work |
|---|---|---|---|
| 44 | Sanctioned primitives `git`/`oya-gate`/`oya-verify` | **N/A** | No `oya` gate engine wired into `/stack`. |
| 45 | Worktree-branch → PR vs `dev` → CI+gate+APPROVE | **N/A** | Out of repo scope. |
| 46 | Trust boundary (tool output = data) | **N/A** | Process item. |
| 47 | `git mv` for moves | **ADVISORY** | All relocation fit-work above MUST use `git mv` to preserve history. |

## J. Migration completion gate

| # | Item | Verdict | Evidence |
|---|---|---|---|
| 48 | Service "done" gate (paths exist, zombies removed, build=0, nextest=0, gate packets=0) | **VIOLATES (0/N services done)** | No service satisfies the gate; no `{oya,cloud}/<service>/{PRD.md,README.md}` exists. |

---

## Digest

**What it is:** `/stack` = 4 independent Cargo workspaces (`kernel` no_std nightly-2021ed, `kubernetes` 139 crates resolver=3 2024ed, `operating-system` 45 `talos-*` crates 2024ed, `kernel-usermode-tests`) + a vendored Go upstream `talos-reference`. It is NOT yet an oyatie-policy monorepo; it is pre-migration source material.

**Top conformance gaps (ranked):**
1. **Topology (A1–A2,J48):** no `cloud/`/`oya/`/`libs/` homes; no per-service PRD/README/contracts/decisions. 0 services pass the migration gate.
2. **One-workspace (D19):** 16 `[workspace]` decls (incl. 11 nested kernel user-bin workspaces + 1 third-party). Collapsing to a single root is the hardest structural blocker (freestanding user binaries with custom link addrs resist workspace merge).
3. **Naming (B8–B12):** ZERO `oya-` crates; ZERO layer suffixes; snake_case names (`ctrd_seccomp`, `meta_v1`) — all 191 crates need rename to `oya-<ms>(-bc)?-<layer>`.
4. **Brand residue (C17):** `talos-*` prefix on all 45 OS crates + 388 `.rs` files; **`Kuberos` prior-codename leaked into product source** (`talos-secrets` `KUBEROS_*` env vars, `difftest`) and 51× in `MIGRATION_REPORT.md`. Hard violation of `oya-check-brand-residue`.
5. **Hexagonal (E26–29):** no kernel/domain/adapter/app split; the framekernel's literal `kernel` crate semantically collides with the policy "ports-kernel" layer.
6. **Governance plumbing (D22–24,F31,G35–39):** no `[lints] workspace`, no `doctest=false`, no `publish=false`/Apache-only license, no `deny.toml`, no `data_class`, no dependency-rationales/vendor-lockin registries.

**Rename / reshape work:**
- `kernel/` → `cloud/cloud-kernel/crates/oya-cloud-kernel-{kernel→core,hal-adapter,frame-kernel,...}` (disambiguate "kernel"); fix edition→2024, version→0.1.0, license→Apache-2.0.
- `operating-system/talos-*` → `cloud/cloud-node-os/crates/oya-cloud-node-os-<bc>-<layer>`; purge every `talos`/`Kuberos`/`siderolabs` token from source+docs; bare `init`/`svc`/`talosctl` get `-app` suffixes.
- `kubernetes/` (44 `ctrd_` + 95 k8s) → `cloud/<orchestrator|runtime>/crates/oya-...-<layer>`, kebab + layer suffix, resolver 3→2, hoist inline edition/version into inherited `[workspace.package]`; reconcile `third-party/` vendoring with crates.io-only `[sources]` or Reindeer doctrine.
- Add: root single `[workspace]` + `[workspace.metadata.oya.microservices]`, `deny.toml`, `registry/dependency-rationales.json`, `registry/vendor-lockin-phaseout/index.json`, `libs/oya-data-boundary-kernel`, per-service PRD/README/contracts/decisions, BUCK target rewiring to `//{cloud,libs}/...` + `third-party//`.

**Blockers:**
- **No `oya-`/`cloud/` skeleton exists yet** — every relocation target is greenfield; this is a from-scratch migration, not a touch-up.
- **Nested freestanding kernel user-binary workspaces (D19)** cannot trivially join one root workspace (custom linker scripts / build-std / link addresses) — needs a sanctioned exclude or out-of-tree home decision before single-workspace collapse.
- **`talos-reference/` is upstream Go** (`github.com/siderolabs/talos`) — pure reference; must be excluded from the Rust workspace and ideally moved out of the product tree (or clearly marked vendored) so brand-residue checks don't flag it.
- Two separate `.buckroot`s (kubernetes + operating-system), kernel has none → three build graphs to unify into one Buck2 graph.
