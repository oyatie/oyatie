---
title: Monorepo Conformance Audit — office (OyaOffice → oya/office)
status: Read-only audit
date: 2026-06-06
repo: /Users/jasonlee/Developer/office
checklist: ./00-policy-checklist.md
verdict_summary: STRUCTURALLY-PROMISING, GLOBALLY-NONCONFORMING — clean hexagonal scaffold + Buck2 wiring present, but BLANKET RENAME (oyaoffice-* → oya-office-*) + topology reshape ({oya,cloud}/<service>/crates) + per-service colocation are all unmet. No source code lost; everything is a scaffold (constants + value objects), so reshape is mechanical, not destructive.
---

# Monorepo Conformance Audit — `office`

Sibling repo: `/Users/jasonlee/Developer/office`. Product = OyaOffice (cloud-native office suite: Drive/Docs/Sheets/Slides + collab + format). Target identity per task: **OyaOffice → oya/office**, crates **`oyaoffice-*` FORBIDDEN → `oya-office-*`**.

Repo is an early **scaffold**: 13 lib crates + 6 app crates, each `src/lib.rs` (some + `src/main.rs`) holding constants + value objects + validation, no third-party runtime deps (`Cargo.lock` lists only first-party packages). Buck2 wiring (`.buckconfig`, `.buckroot`, per-crate `BUCK`, `third-party/BUCK`, `toolchains/`) is in place. No `git mv` history concerns — reshape is mechanical.

## Per-checklist-item verdict

### A. Canonical homes & topology

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 1 | Code only at `{oya,cloud}/<service>/crates/<crate>/` | **NEEDS-RESHAPE** | No `oya/` or `cloud/` top-level dirs (`ls` → "No such file or directory"). Code sits at flat `crates/` + `apps/`. Reshape: route domain/product services (docs/sheets/slides/drive/format/collab) under `oya/<service>/crates/`; platform services (tenancy/authz) under `cloud/<service>/crates/` (or `oya/` per product framing). |
| 2 | Shared cross-cutting at `libs/<lib>/` | **NEEDS-RESHAPE** | No `libs/` dir. `oyaoffice-kernel`, `-storage-port`, `-search-port`, `-api-contracts` are cross-cutting but live in flat `crates/`. Move shared kernel/ports/contracts → `libs/`. |
| 3 | `microservices/` FORBIDDEN | **CONFORMS** | No `microservices/` dir present. |
| 4 | Per-service colocation (PRD/README/decisions/contracts/specs/catalog/runbooks/threat-model/slos/iac/src/tests/evidence) | **VIOLATES** | None of this exists per-service. Docs/specs/decisions are CENTRAL (`docs/decisions/ADR-001..006`, `specs/*.md`). Only PRD is `.omx/plans/prd-...md` (scratch, not `oya/<service>/PRD.md`). Must create per-service folders with colocated PRD.md/README.md/decisions/contracts/specs/slos/etc. |
| 5 | Cross-cutting artifacts central ONLY; per-service in central = violation | **NEEDS-RESHAPE** | Inverse problem: everything is central, nothing per-service. Service-scoped ADRs (e.g. ADR-003 CRDT, ADR-004 sheets) belong under `oya/<service>/decisions/`; only truly cross-cutting ADRs stay at `docs/decisions/`. |
| 6 | Aggregation indices GENERATED | **NEEDS-RESHAPE** | No `registry/catalog/` or generated `docs/prds/INDEX.md`. `workspace_metadata.bzl` + `[workspace.metadata.oyaoffice.first_party_layout]` are HAND-AUTHORED layout lists — must become generated views from per-service `catalog/<crate>.yaml`. |
| 7 | Packs only under `packs/`/`regional-packs/` | **CONFORMS (N/A)** | No pack artifacts present; nothing misplaced. |

### B. Crate naming — BNF v4.1 + 13-layer enum

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 8 | `oya-` prefix mandatory | **NEEDS-RENAME** | ALL 19 crates use `oyaoffice-*` (e.g. `oyaoffice-kernel`, `oyaoffice-drive-api`). This is the forbidden codename-prefix form (checklist #16/#17). Rename to `oya-office-*` across `[workspace].members`, `[package].name`, `[lib].name` (snake), `[[bin]].name`, all `path` deps, `Cargo.lock`, all `BUCK` `crate=`/`name=`, `workspace_metadata.bzl`. |
| 9 | BNF v4.1 `oya-<microservice>(-<bc>)?-<layer>` | **NEEDS-RENAME** | After rename, grammar mostly holds: `oya-office-drive-api`, `oya-office-collab-domain`. But several LAST tokens are NOT canonical layers (see #12). `office` would be the microservice token (1 of 1..3). |
| 10 | `[package].name` == dir basename | **CONFORMS** | Verified all 19: package name == directory basename (e.g. dir `oyaoffice-kernel` ↔ name `oyaoffice-kernel`). Rename must keep this invariant. |
| 11 | `[lib].name` == snake_case(package.name) | **CONFORMS** | Verified all: `oyaoffice-kernel`→`oyaoffice_kernel`, `oyaoffice-drive-api-contracts`→`oyaoffice_drive_api_contracts`, etc. |
| 12 | Layer suffix ∈ closed 13-enum (`kernel\|domain\|usecase\|app\|adapter\|infrastructure\|cli\|rest\|grpc\|graphql\|worker\|sdk\|api`) | **VIOLATES** | Suffix tally: `domain`×8 OK, `kernel`×1 OK, `api`×2 OK, `worker`×2 OK; but **`port`×2** (`storage-port`,`search-port`), **`api-contracts`×2**, **`gateway`×1** (`collab-gateway`), **`web`×1** (`oyaoffice-web`) are NOT in the enum. Reshape: `*-port`→`*-kernel` (ports belong in kernel per item 26) or fold into a kernel; `*-api-contracts`→`*-api`; `*-gateway`→`*-rest`/`-grpc`/`-worker`; `oyaoffice-web`→`oya-office-web-app` (or `-rest` for SSR server). |
| 13 | `oya-check-*` family; `*-adapter-<backend>` impls ≥1 kernel port | **NEEDS-RESHAPE** | No `oya-check-*` crates and NO adapter crates exist at all. Ports (`storage-port`,`search-port`) declare intents but have ZERO backend adapters (`fake`/`inmemory`/`aws`/...). Add `*-adapter-<backend>` crates implementing the port traits once ports move to kernel. |
| 14 | `tools/` crates explicit `-app` suffix | **CONFORMS (N/A)** | No `tools/` crates present. |
| 15 | Microservice slot2 registered in `[workspace.metadata.oya.microservices.<name>]` | **NEEDS-RESHAPE** | Registry EXISTS but under wrong key `[workspace.metadata.oyaoffice.microservices]` (drive/docs/sheets/slides/format/collab/tenancy/authz with owner+rationale). Missing `adr_cite` field. Re-key to `[workspace.metadata.oya...]` and add adr_cite per microservice. |

### C. Brand residue — FORBIDDEN

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 16 | `oyatie-*` cargo prefix forbidden | **CONFORMS** | No `oyatie-*` crate prefix. (Filesystem path is `office`, not an `oyatie` slug.) |
| 17 | Codename residue forbidden in product surface/source/docs (`oyaoffice`, `oyago`, `oyapy`, `kuberos`, ...); product surface = "Oyatie" | **VIOLATES** | `oyaoffice` is pervasive: 19 crate names, all `[lib].name`s, `product = "OyaOffice"`, metric strings (`oyaoffice_drive_storage_upload_seconds` in storage-port lib.rs), `repository = "https://example.invalid/oyaoffice"`, `workspace_metadata.bzl`, README/specs (65 hits in specs+README). This is the single largest violation. Purge `oyaoffice`→`oya-office` in code/identifiers and product prose to the canonical Oyatie brand. |
| 18 | No tautological rebrand residue (arrows/"retired terms"/"after rename") | **CONFORMS (caveat)** | No old→new arrow tables or "retired terms" sections found in live docs at this scaffold stage. Avoid introducing them during the rename. |

### D. One-workspace invariants

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 19 | Exactly ONE `[workspace]` (root) | **CONFORMS** | Only `/office/Cargo.toml` carries `[workspace]`; no nested workspaces in members. |
| 20 | `resolver = "2"` | **VIOLATES** | Root `Cargo.toml` line 23 = `resolver = "3"`. Checklist mandates `"2"`. Change to `resolver = "2"` (or escalate a 1-ADR amendment if 2024-edition resolver=3 is desired — but current policy says 2). |
| 21 | Members inherit `version/edition/rust-version .workspace = true` | **NEEDS-RESHAPE** | edition+rust-version inherit OK, BUT every member hardcodes `version = "0.1.0"` (kernel L3, api L3) instead of `version.workspace = true`. Also `[workspace.package]` lacks a `version` field. Add `version = "0.1.0"` to `[workspace.package]` and switch members to `version.workspace = true`. Note: checklist baselines edition 2024 (match), rust 1.95.0 — repo pins **1.96.0** (mismatch). |
| 22 | Each member: `publish = false` + `license = "Apache-2.0"` + `[lints] workspace = true` | **VIOLATES** | `publish = false` ✓ and `[lints] workspace = true` ✓ on members. BUT license is inherited `license.workspace = true` → resolves to `[workspace.package] license = "Proprietary"` (line 28), NOT `Apache-2.0`. Change workspace license to `Apache-2.0`. |
| 23 | `[lib]` sets `doctest = false` | **VIOLATES** | No `doctest = false` in any `[lib]` block (checked kernel, storage-port, drive-api). Add `doctest = false` to every `[lib]`. |
| 24 | Single `[workspace.dependencies]` seam; every entry has rationale row | **NEEDS-RESHAPE** | No `[workspace.dependencies]` table exists (zero third-party deps yet) and no `registry/dependency-rationales.json`. Vacuously clean now, but the seam + rationales registry must be established before any third-party dep is added. |

### E. Hexagonal clean architecture

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 25 | Inward-only dep flow per layer matrix | **NEEDS-RESHAPE** | Deps look inward at scaffold (api→domain→kernel; storage-port→kernel). But `port` and `api-contracts` are non-canonical layers (#12), so the matrix can't be validated until layers are normalized. No automated import-matrix check present. |
| 26 | Port traits in `kernel`, impls in `adapter` | **VIOLATES** | Ports live in standalone `*-port` crates, NOT in kernel. Storage/search ports model intents (`UploadIntent`/`DownloadIntent`) but expose no port TRAITS and have no `adapter` impls. Reshape: fold port traits into `oya-office-<svc>-kernel`; add `oya-office-<svc>-adapter-<backend>`. |
| 27 | `kernel` pure: no business logic/I/O/async; no_std-capable | **NEEDS-RESHAPE** | Kernel is framework-free, no async, no I/O (good), BUT uses `std::error::Error` (kernel lib.rs line 62; storage-port line 44) and `String` without `#![no_std]`+`alloc`. Not no_std-capable as written. Make kernels `#![no_std]` + `extern crate alloc` where feasible; drop `std::error::Error` impls or gate behind a std feature. |
| 28 | `api` deps kernel only; `sdk` deps kernel only | **NEEDS-RESHAPE** | App-layer `oyaoffice-api`/`oyaoffice-drive-api` are composition binaries, not the protocol-neutral `api` contract crate. The contract crates are mis-suffixed `*-api-contracts` and `oyaoffice-api-contracts` depends on BOTH `oyaoffice-kernel` AND `oyaoffice-sheet-domain` (Cargo.lock) — VIOLATES "api depends on kernel only". No `sdk` crate exists. Rename contracts→`-api` and strip the domain dep. |
| 29 | Only `app` has unrestricted inward deps; `app→app` forbidden | **NEEDS-RESHAPE** | No `-app` composition-root crates exist; app binaries are named `*-api`/`*-worker`/`*-gateway`/`*-web`. Introduce explicit `*-app` composition roots. |
| 30 | No direct cross-microservice imports (LEAN-A2); via Workflow/Ontology or `public_layers` allowlist | **NEEDS-RESHAPE** | `oyaoffice-api-contracts` depends on `oyaoffice-sheet-domain` (cross-concept domain import) — a cross-microservice coupling with no declared `public_layers` allowlist. Remove or route through a sanctioned seam. |

### F. Data governance & quality gates

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 31 | `data_class` on kernel data fields w/ tenant/regulated data | **CONFORMS (partial)** | `DataClass` enum defined in `oyaoffice-kernel` and applied on `UploadIntent`/`DownloadIntent` fields in storage-port (lib.rs lines 123/175/187/235). Good pattern; extend to all tenant-data-bearing kernel fields across domains as they grow. |
| 32 | Statelessness (no module-level mutable state in presentation/app/worker) | **CONFORMS (N/A)** | grep found no `static mut`/`lazy_static`/`once_cell`/module-level `Mutex` in any crate src. Currently vacuously stateless. No `oya-check-statelessness` gate present to enforce. |
| 33 | Shardability (tenant_id partition key + RLS) | **NEEDS-RESHAPE** | `TenantId` value object exists and intents are tenant-scoped, but no DB designs / partition-key / RLS declarations yet (scaffold has no persistence). Must declare when storage adapters land. No `oya-check-shardability` gate. |
| 34 | Buildability bar (PRD ≥5 stories, IP ≥150 lines, ADR Context+Decision+≥3 alts+≥3 consequences+sources+roadmap) | **NEEDS-RESHAPE** | ADRs exist (`docs/decisions/ADR-001..006`) and specs are substantial, but not audited line-by-line against the bar here; PRD lives in `.omx/plans/` not as a governed per-service `PRD.md`. Promote PRD to per-service, verify each ADR meets the alternatives/consequences/sources/roadmap structure. |

### G. Supply chain & license policy

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 35 | Root `deny.toml` license allowlist exact set; per-crate exceptions only | **NEEDS-RESHAPE** | `supply-chain/deny.toml` allowlist matches the mandated set EXACTLY {0BSD, Apache-2.0, BSD-2/3, ISC, MIT, MPL-2.0, Unicode-3.0}, `exceptions=[]`. BUT it lives at `supply-chain/deny.toml`, not ROOT `deny.toml` (checklist cites root). Move/symlink to repo-root `deny.toml` (or confirm tooling reads supply-chain path). |
| 36 | `[bans]`: deny openssl/openssl-sys + old-time; multiple-versions=warn; wildcards=warn | **NEEDS-RESHAPE** | `multiple-versions = "warn"` ✓ BUT `wildcards = "deny"` (checklist wants `warn` with path-deps OK) and `deny = []` does NOT list `openssl`/`openssl-sys`/`old-time`. Add the three banned crates; set wildcards=warn. |
| 37 | `[sources]`: only crates.io; unknown-registry/git=deny; empty allow-git | **CONFORMS** | `unknown-registry=deny`, `unknown-git=deny`, `allow-git=[]`, `allow-registry=[crates.io-index]`. Matches. |
| 38 | `[advisories]`: yanked=deny, empty ignore | **CONFORMS** | `yanked="deny"`, `ignore=[]`. Matches (also `unmaintained="warn"`). |
| 39 | Vendor classification A/B/C; B registered in `registry/vendor-lockin-phaseout/index.json` | **NEEDS-RESHAPE** | No `registry/vendor-lockin-phaseout/index.json` (find → none). `specs/dependency-policy.md` exists as prose. Zero third-party deps now, so nothing to classify yet, but the registry + A/B/C seam must exist before deps land. |

### H. Buck2 build graph

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 40 | Per-crate `BUCK` (rust_library/rust_binary, crate_root, visibility, deps) | **NEEDS-RESHAPE** | Every crate HAS a `BUCK` (`rust_library`+`rust_test`, `crate=`, `visibility=["PUBLIC"]`). BUT they lack `crate_root` and in-repo `deps` pointing to `//crates/...:lib` / `third-party//:<dep>`; binaries should use `rust_binary` not just `rust_library`. Targets reference `oyaoffice_*` crate names → must rename to `oya_office_*`. Add deps + crate_root + use `//{oya,cloud,libs}/...` paths after topology reshape. |
| 41 | Third-party buckified by Reindeer (`third-party/BUCK` + fixups/); Cargo SSOT | **NEEDS-RESHAPE** | `third-party/BUCK` exists (62 bytes, stub) but no `third-party/fixups/` and no Reindeer-generated content (zero deps). Establish Reindeer flow before adding deps. Note: `third-party/.buckroot` present — verify it's not creating a spurious second buck root. |
| 42 | `.buckconfig` + `.buckroot`; cells root/prelude/toolchains/third-party; prelude bundled | **CONFORMS** | `.buckconfig` defines cells root/prelude/toolchains/none/third-party with `[external_cells] prelude = bundled`; `.buckroot` present; `toolchains/BUCK` present. Matches well. |
| 43 | NativeLink RBE only; Jenkins buck2 CI (doctrine, 0% adopted) | **NEEDS-RESHAPE** | No CI config observed for NativeLink/Jenkins buck2. Doctrine target only; no false build/CI claims to retract. Wire when adopted. |

### I. Agent / contribution protocol

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 44 | Sanctioned primitives only (`git`, `oya-gate`, `oya-verify`) | **NEEDS-RESHAPE** | `supply-chain/gates.toml` exists (gate config) but no `oya-gate`/`oya-verify` wiring evident. Repo uses `.omx/` scratch tooling. Adopt the sanctioned gate engine. |
| 45 | Worktree branch/lane → commit/push → PR vs `dev` → Jenkins+`oya gate run-all`+reviewer | **NEEDS-RESHAPE** | No `dev`-branch PR/CI flow evidenced. Establish the lane→PR→gate→reviewer sequence. |
| 46 | Trust boundary (tool/file/web output = data) | **NEEDS-RESHAPE** | No AGENTS.md in repo (find → none); trust-boundary doctrine not declared. Add an AGENTS.md aligned to source. |
| 47 | `git mv` (never rm+add) for moves | **NEEDS-RESHAPE (forward-looking)** | The upcoming rename/topology reshape MUST use `git mv` for every crate-dir move to preserve history. No prior violation observed (scaffold age). |

### J. Migration completion gate

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 48 | "Done" gate: per-service PRD/README, zombies removed, members→new paths, cargo build/nextest=0, gate packets=0 | **VIOLATES (not started)** | None of the gate conditions met: no `{oya,cloud}/<service>/` paths, no per-service PRD/README, members still point at flat `crates/`/`apps/` with `oyaoffice-*`. This is the terminal gate — blocked until A/B/C/D/E reshape lands. |

## Top conformance gaps (ranked)

1. **Blanket crate rename `oyaoffice-* → oya-office-*` (items 8, 9, 17).** 19 crates + all `[lib]`/`[[bin]]`/path-dep/`Cargo.lock`/`BUCK`/`workspace_metadata.bzl` references + metric strings + `product`/`repository` fields. Codename `oyaoffice` is the single largest violation surface (65+ doc hits, pervasive in source identifiers).
2. **Topology reshape to `{oya,cloud}/<service>/crates/` + `libs/` (items 1, 2, 4, 5).** Flat `crates/`+`apps/` must become per-service trees; shared kernel/ports/contracts → `libs/`; per-service colocation (PRD/README/decisions/contracts/specs/slos/...) created from currently-central docs.
3. **Layer-suffix normalization to the 13-enum (items 12, 26, 28).** `*-port`→fold into `*-kernel` (+ add `*-adapter-<backend>`); `*-api-contracts`→`*-api` (and drop its domain dep); `*-gateway`→`*-rest`/`-worker`; `oyaoffice-web`→`*-web-app`/`-rest`; add `*-app` composition roots.
4. **One-workspace hygiene (items 20, 21, 22, 23).** `resolver "3"→"2"`; add `[workspace.package] version` + members use `version.workspace=true`; workspace `license "Proprietary"→"Apache-2.0"`; add `doctest=false` to every `[lib]`. (rust pinned 1.96.0 vs checklist 1.95.0 — reconcile.)
5. **Hexagonal/kernel purity (items 26, 27).** Port traits into kernel; make kernels `#![no_std]`+`alloc` (drop bare `std::error::Error`); add adapter crates implementing ≥1 port trait.
6. **Supply-chain deltas (items 35, 36, 39).** Move `deny.toml` to root; `[bans]` add openssl/openssl-sys/old-time + wildcards=warn; create `registry/vendor-lockin-phaseout/index.json` + dependency-rationales seam before any dep lands.
7. **Governance scaffolding (items 6, 15, 24, 44–46).** Generated aggregation indices (not hand-authored `workspace_metadata.bzl`); re-key microservices registry to `[workspace.metadata.oya...]` + add `adr_cite`; add AGENTS.md + `oya gate`/`oya verify` + dev-branch PR flow.

## Reshape / rename work (mechanical, non-destructive)

- `git mv crates/oyaoffice-<x> oya/<svc>/crates/oya-office-<x>` (domain/product) and `cloud/<svc>/crates/...` (tenancy/authz); shared → `libs/oya-office-<x>` — all via `git mv` (item 47).
- Global identifier rewrite `oyaoffice` → `oya_office`/`oya-office` across Cargo.toml, lib/bin/path deps, Cargo.lock, every BUCK (`crate=`/`name=`/add `crate_root`+`deps`), `workspace_metadata.bzl`, metric strings, `product`/`repository`.
- Suffix remap per the 13-enum; introduce `*-app` roots and `*-adapter-<backend>` crates.
- Root-Cargo fixes: resolver=2, workspace `version` + `version.workspace=true` members, license=Apache-2.0, `[lib] doctest=false`, establish `[workspace.dependencies]` seam.
- Kernel: `#![no_std]`+`extern crate alloc`; relocate port traits into kernel; add adapters.
- Supply-chain: root `deny.toml`; `[bans]` openssl/openssl-sys/old-time + wildcards=warn; create vendor-lockin + dependency-rationales registries.
- Create per-service colocation (PRD/README/decisions/contracts/specs/slos/runbooks/threat-model/catalog/iac/tests/evidence); move service-scoped ADRs out of central `docs/decisions/`; make aggregation indices generated.
- Add AGENTS.md + sanctioned gate wiring (`oya gate`/`oya verify`) + dev-branch PR/CI flow.

## Blockers / notes

- **No source loss, no missing repo.** `office` exists and is a coherent scaffold (constants + value objects, zero third-party deps), so the entire reshape/rename is mechanical, not a salvage. Cargo.lock confirms only first-party packages.
- **`resolver = "3"` vs policy `"2"`** and **rust 1.96.0 vs checklist 1.95.0** are direct conflicts with the cited baseline — either conform down or raise a 1-ADR amendment; do not silently diverge.
- **`.omx/` (not `.omc/`)** scratch dir present — out of audit scope but worth flagging as non-standard tooling state.
- **`third-party/.buckroot`** alongside root `.buckroot` — verify it does not establish a spurious second Buck root.
