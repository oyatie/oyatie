---
title: Monorepo Conformance Audit — oyapy
status: Audit-finding
date: 2026-06-06
auditor: workflow-subagent (read-only)
repo: /Users/jasonlee/Developer/oyapy
target_home: oya/transpiler-python-to-rust/ (codename oyapy-* FORBIDDEN)
checklist: /Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/monorepo-conformance/00-policy-checklist.md
verdict_summary: VIOLATES on naming/brand/layout/seam/build; partial CONFORMS only on single-workspace + inward-leaning structure intent.
note: READ-ONLY. Cites evidence (paths, Cargo.toml contents, crate names). No edits made.
---

# Monorepo Conformance Audit — `oyapy`

## 0. Repo snapshot (evidence)

- Root: `/Users/jasonlee/Developer/oyapy`
- Root `Cargo.toml` (full): `[workspace]` with `members = ["crates/oyapy-cli","crates/oyapy-core","crates/oyapy-runtime"]`, `resolver = "3"`, `[workspace.package] edition = "2024", rust-version = "1.96"`, `[workspace.lints.rust] unsafe_code = "deny"`, `[workspace.lints.clippy] all = "deny"`. NO `[workspace.metadata.oya]`, NO `version`, NO `license`, NO `[workspace.dependencies]`.
- Crates (3, all under `crates/`): `oyapy-cli` (bin `src/main.rs`), `oyapy-core` (lib; modules analyzer/artifact/capability/fixture/json/mapping/schema/stop_packet/validate), `oyapy-runtime` (lib).
- Crate package names: `oyapy-cli`, `oyapy-core`, `oyapy-runtime` (all `oyapy-` prefix).
- Other top-level: `AGENTS.md`, `Cargo.lock`, `docs/`, `fixtures/`, `python/oyapy_analyzer.py` (105 KB), `target/`, `.omx/`, `.gitignore` (`/target/`, `**/*.rs.bk`).
- `docs/adr/` has exactly one file: `0001-compiler-pipeline.md` (Accepted; codename `oyapy` in body).
- ABSENT: `oya/`, `cloud/`, `libs/`, `registry/`, `specs/`, `deny.toml`, `.buckconfig`, `.buckroot`, any `BUCK` file, `third-party/`, `vendor/`, `rust-toolchain*`.
- No nested `[workspace]` (only root). No vendored dirs. No `data_class`/`DataClass` anywhere in `crates/`.
- Brand residue `oyapy` appears in 181 `.rs` lines + nearly every docs/fixtures/manifests file + AGENTS.md + all 3 crate Cargo.tomls (codename is the project's own identity).

## A. Canonical homes & topology

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 1 | Service code only at `{oya,cloud}/<service>/crates/<crate>/` | **VIOLATES** | Code lives at repo-root `crates/oyapy-*`. No `oya/`/`cloud/` dir. As a product/domain transpiler this is an `oya/` service. Reshape: move to `oya/transpiler-python-to-rust/crates/`. |
| 2 | Shared cross-cutting code only at `libs/<lib>/` | **N/A → CONFORMS-by-absence** | No shared/governance crates; nothing misplaced into a (nonexistent) `libs/`. When merged, central `libs/oya-check-*` come from source, not this repo. |
| 3 | `microservices/` FORBIDDEN | **CONFORMS** | No `microservices/` dir. |
| 4 | Per-service colocation (PRD.md, decisions/, contracts/, specs/, catalog/<crate>.yaml, runbooks/, threat-model.md, slos/, iac/, evidence/multispectrum/) | **NEEDS-RESHAPE** | Has `docs/` + `docs/adr/0001` + `fixtures/`, but NOT the per-service skeleton. No PRD.md, no README.md at service root, no contracts/, no catalog/<crate>.yaml, no slos/, no threat-model.md, no iac/, no evidence/multispectrum/. Must build the full `oya/transpiler-python-to-rust/` colocation tree. |
| 5 | Cross-cutting artifacts central only | **NEEDS-RESHAPE** | Service-scoped ADR `docs/adr/0001-compiler-pipeline.md` should become `oya/transpiler-python-to-rust/decisions/` (service-scoped), and naming `ADR-####-*.md`. Numbering `0001` collides with central scheme; renumber/relocate as service-scoped. |
| 6 | Aggregation indices GENERATED | **NEEDS-RESHAPE** | `docs/refactor-rewrite-registry.{json,md}`, `docs/capability-matrix.md`, `docs/support-matrix.md`, `docs/mapping-catalog.md` are hand-authored aggregation-style docs; on merge must be sourced/generated or recast as per-service source-of-record (not central INDEX duplicates). |
| 7 | Packs only under `packs/`/`regional-packs/` | **CONFORMS** | No pack artifacts. |

## B. Crate naming — BNF v4.1 + 13-layer enum

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 8 | `oya-` prefix mandatory | **VIOLATES / NEEDS-RENAME** | All three crates are `oyapy-*` (Cargo.toml `name = "oyapy-cli"` / `"oyapy-core"` / `"oyapy-runtime"`). `oyapy-` is a FORBIDDEN codename prefix (item 17) AND lacks `oya-`. |
| 9 | BNF `oya-<microservice>(-<bc>)?-<layer>` | **VIOLATES / NEEDS-RENAME** | None match. Microservice slot must be a registry-validated 1..3-token name (e.g. `transpiler-python-to-rust`), last token a canonical layer. Current names have no layer token and an illegal `oyapy` prefix. |
| 10 | `[package].name` == directory basename | **CONFORMS** | `oyapy-cli`/`oyapy-core`/`oyapy-runtime` each match their dir basename. (Mechanically conformant, but the names themselves are forbidden — fixing #8/#9 must keep this invariant.) |
| 11 | `[lib].name` == snake_case(package.name) | **NOT-EVALUABLE → likely CONFORMS-by-default** | No explicit `[lib]` section in any Cargo.toml; Cargo defaults `oyapy-core` → `oyapy_core`, which satisfies the rule by default. After rename, default still satisfies. |
| 12 | Layer suffix ∈ 13-value enum | **VIOLATES / NEEDS-RENAME** | `-cli` IS a valid layer; `-core` and `-runtime` are NOT in the enum {kernel,domain,usecase,app,adapter,infrastructure,cli,rest,grpc,graphql,worker,sdk,api}. Reshape: `oyapy-core` → split into `oya-transpiler-python-to-rust-kernel` (pure types/ports) + `*-domain`/`*-usecase` + `*-app`; `oyapy-runtime` → `*-infrastructure` or a runtime `*-adapter-<backend>`; `oyapy-cli` → `oya-transpiler-python-to-rust-cli`. |
| 13 | Adopted patterns (`oya-check-*`, `*-adapter-<backend>`) | **N/A** | No check crates; no adapter crates. If a runtime backend seam is introduced it must follow `*-adapter-<backend>` + impl a `*-kernel` port. |
| 14 | `tools/` crates take explicit `-app` suffix | **N/A** | No `tools/` crates. `python/oyapy_analyzer.py` is a non-Rust tool sitting at repo root (not a Cargo crate); see blockers. |
| 15 | Microservice slot2 registered in `[workspace.metadata.oya.microservices.<name>]` | **VIOLATES** | Root Cargo.toml has NO `[workspace.metadata.oya]` block at all. On merge, central root must register `transpiler-python-to-rust` (owner + rationale + adr_cite). |

## C. Brand residue — FORBIDDEN list

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 16 | Cargo prefix `oyatie-*` FORBIDDEN | **CONFORMS** | No `oyatie-*` crate prefix present. |
| 17 | Codename residue FORBIDDEN (`oyaoffice`,`oyago`,**`oyapy`**,`kuberos`,`foundry-*`,`oyatie-*`,`talos-*`) | **VIOLATES (severe, pervasive)** | `oyapy` is the project's own codename — banned in product surface/source/docs. Present in: all 3 crate names + Cargo.tomls; `oyapy-core/src/lib.rs:1` doc comment "machinery for `oyapy`"; 181 `.rs` lines; AGENTS.md ("Build `oyapy` as..."); ADR-0001 ("Build `oyapy` as a staged..."); nearly every `docs/*.md`, `docs/refactor-rewrite-registry.json`, all `fixtures/manifests/*.json`; `python/oyapy_analyzer.py` filename + content. ALSO references sibling codename `oyago` (AGENTS.md L9, design SSOT L5) — also forbidden. Rename target: `oya-transpiler-python-to-rust-*` crates; prose brand "Oyatie"; the product concept = "Python→Rust transpiler". |
| 18 | No tautological rebrand residue (arrows, "retired terms", "after rename") | **CONFORMS (with caveat)** | The `→` arrows found in `docs/architecture.md` and the design SSOT are pipeline-stage arrows ("Python source → parser → IR"), NOT `oldbrand→newbrand` rebrand arrows; no "retired terms" table or "after rename" phrasing detected. Caveat: once the `oyapy→oya-...` rename happens, do NOT leave migration-arrow residue behind. |

## D. One-workspace invariants

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 19 | Exactly ONE `[workspace]` (repo root); no nested | **CONFORMS (standalone)** | `grep '^[workspace]'` returns only root Cargo.toml; member crates have none. BUT on merge this repo's root `[workspace]` itself dissolves into the source monorepo root — its members re-home under the single source workspace. So "conforms" only as a standalone repo. |
| 20 | `resolver = "2"` | **VIOLATES** | Root sets `resolver = "3"` (Cargo.toml L7). Source policy = `"2"`. NEEDS-RESHAPE to `2` (or drops out entirely on merge into source root). |
| 21 | Members inherit `version.workspace`/`edition.workspace`/`rust-version.workspace` (edition 2024, version 0.1.0, rust 1.95.0) | **NEEDS-RESHAPE** | edition: `2024` matches. version: members hardcode `version = "0.1.0"` instead of `version.workspace = true` (workspace.package has NO `version` key) — VIOLATES one-version inheritance. rust-version: workspace sets `1.96`, source policy = `1.95.0` — MISMATCH. Fix: add `version` to `[workspace.package]`, switch members to `version.workspace = true`, align rust-version to source. |
| 22 | Each member carries `publish = false` + `license = "Apache-2.0"` + `[lints] workspace = true` | **VIOLATES (2 of 3 missing)** | All 3 members have `[lints] workspace = true` (good). NONE declare `publish = false`. NONE declare `license` (root has no `license` either). Add `publish = false` + `license = "Apache-2.0"` to every member (and `license` to workspace.package). |
| 23 | `[lib]` sets `doctest = false` | **VIOLATES** | `oyapy-core` and `oyapy-runtime` have NO `[lib]` section at all → `doctest` not set to false. Add `[lib] doctest = false`. |
| 24 | Single `[workspace.dependencies]` seam + `registry/dependency-rationales.json` rows | **VIOLATES** | Root Cargo.toml has NO `[workspace.dependencies]` section. The one inter-crate dep (`oyapy-cli` → `oyapy-core = { path = "../oyapy-core" }`) is a path dep declared inline, not via the workspace seam. No `registry/dependency-rationales.json`. (Currently std-only externally, so the seam is empty, but the section + path-dep-via-seam discipline is still required; ADR-0092.) |

## E. Hexagonal clean architecture

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 25 | Inward-only dependency flow per 13-layer matrix | **NEEDS-RESHAPE** | Only edge is `cli → core` (presentation → "core"). Direction is plausibly inward, but layers aren't typed (no kernel/domain/app split), so the matrix can't be enforced. Must re-layer first (see #12). |
| 26 | Port traits in `kernel` (not domain); impls in `adapter` | **NEEDS-RESHAPE** | No kernel/adapter separation exists. `oyapy-core` mixes schema + validators + json + mapping in one lib; `oyapy-runtime` is undifferentiated. Extract ports into a `*-kernel`, impls into `*-adapter-*`/`*-infrastructure`. |
| 27 | `kernel` = pure types + ports, no_std-capable, zero I/O/async | **NEEDS-RESHAPE** | `oyapy-core/src/lib.rs:1-6` notes std-only-by-choice (good signal), but it contains validators/json/fixture logic — not a pure ports kernel. No `#![no_std]`. Carve a no_std-capable kernel of pure types + traits; push fixture/json/validate I/O into infrastructure/adapter. |
| 28 | `api`/`sdk` depend on `kernel` only | **N/A** | No `api`/`sdk` crate. If a transpiler API/SDK surface is added it must depend on kernel only. |
| 29 | Only `app` (composition root) has unrestricted inward deps; `app→app` FORBIDDEN | **NEEDS-RESHAPE** | No `app` crate; `oyapy-cli` is acting as a de-facto composition root. Introduce explicit `*-app` composition root and reduce `cli` to thin presentation. |
| 30 | No direct cross-microservice imports (LEAN-A2) | **CONFORMS (standalone)** | Single service; no cross-service imports. References to sibling `oyago` (AGENTS.md L9) are doc-only "reference effort," not code imports. |

## F. Data governance & quality gates

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 31 | `data_class` on kernel data fields w/ tenant/regulated data | **N/A (likely CONFORMS-by-absence)** | `grep data_class/DataClass` in `crates/` = 0 hits. A transpiler core processes source code, not tenant/PHI/PII runtime data, so DataClass annotations may be genuinely N/A — but this must be asserted in the service threat-model, not assumed. |
| 32 | Statelessness (no module-level mutable state in presentation/app/worker) | **NOT-VERIFIED** | Not scanned in depth; `oya-check-statelessness` not present. Re-run under source gates after re-layering. |
| 33 | Shardability (`tenant_id` + RLS) | **N/A** | No DB designs in a transpiler core. |
| 34 | Buildability bar (PRD ≥5 stories; IP ≥150 lines; ADR Context+Decision+≥3 alternatives+≥3 consequences+sources+roadmap; scorecard GREEN cites evidence) | **NEEDS-RESHAPE** | No PRD.md, no IP-NNN docs. ADR-0001 has Context + Decision + 3 Alternatives ("Rejected because") + 4 Consequences — close, but MISSING named industry sources and In-house roadmap section, and consequences aren't labeled Positive/Negative/Operational. Docs are substantial (design SSOT 18 KB, support-matrix 23 KB) but not in the mandated PRD/IP/ADR buildability shapes. |

## G. Supply chain & license policy

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 35 | Root `deny.toml` license allowlist | **VIOLATES (missing)** | NO `deny.toml` in repo. Must inherit/import source root `deny.toml` on merge. |
| 36 | `[bans]` deny openssl/openssl-sys/old-time; multi-version=warn; wildcards=warn | **VIOLATES (missing)** | No `deny.toml`. (Currently std-only, so no banned crates present in `Cargo.lock` — `Cargo.lock` is 292 B, near-empty — but the policy file is absent.) |
| 37 | `[sources]` crates.io only; unknown-registry/git=deny | **VIOLATES (missing)** | No `deny.toml`. |
| 38 | `[advisories]` yanked=deny, empty ignore | **VIOLATES (missing)** | No `deny.toml`. |
| 39 | Vendor classification A/B/C (ADR-0211); registry/vendor-lockin-phaseout/index.json | **NEEDS-RESHAPE** | No external deps yet (std-only by AGENTS.md policy + lib.rs note), so no A/B/C entries exist; no `registry/vendor-lockin-phaseout/index.json`. AGENTS.md dependency-review discipline is a good signal but is not the source's A/B/C registry mechanism. Any future parser/serde/codegen dep MUST be classified + registered. |

## H. Buck2 build graph

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 40 | Per-crate `BUCK` file (`rust_library`/`rust_binary`, crate_root, visibility, deps) | **VIOLATES (missing)** | NO `BUCK` files anywhere (`find -name BUCK` = none). `docs/build-system-plan.md` describes intent only. Each crate needs a `BUCK`. (Status of ADR-0392/0408 = Proposed/two-way, so this is target-state, not hard-fail.) |
| 41 | Third-party Reindeer → `third-party/BUCK` + fixups; Cargo SSOT | **VIOLATES (missing)** | No `third-party/`. (Empty external dep set today.) |
| 42 | `.buckconfig` + `.buckroot` present | **VIOLATES (missing)** | Neither present. |
| 43 | NativeLink RBE / Jenkins buck2 CI | **N/A (0% adopted)** | No CI config in repo at all. Doctrine/target only. |

## I. Agent / contribution protocol

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 44 | Sanctioned primitives only (`git`, `oya-gate`, `oya-verify`) | **NEEDS-RESHAPE** | `AGENTS.md` is project-specific (transpiler mission/gates) and does NOT reference `oya-gate`/`oya-verify` or the sanctioned-primitive doctrine. Its quality gates are raw `cargo fmt/check/clippy/test` — good but not the governance-gate engine. On merge, defer to source AGENTS.md. |
| 45 | Required sequence (worktree branch → PR vs `dev` → Jenkins + `oya gate run-all` → reviewer APPROVE) | **NEEDS-RESHAPE** | oyapy AGENTS.md prescribes "small, verified, reversible diffs" but not the worktree-lane / `dev`-PR / `oya gate run-all` sequence. Adopt source protocol. |
| 46 | Trust boundary (tool/file/web/MCP output = DATA) | **NEEDS-RESHAPE** | Not stated in oyapy AGENTS.md. Inherit from source AGENTS.md on merge. |
| 47 | `git mv` (never rm+add) for moves | **PROCESS-GATE** | The entire re-home (`crates/oyapy-* → oya/transpiler-python-to-rust/crates/oya-...`) MUST use `git mv` so history is preserved during reshape. |

## J. Migration completion gate

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 48 | Service "done" gate (PRD+README at new path; zombies removed; members → new paths; build/nextest = 0; gate packets green) | **VIOLATES (not started)** | None of: `oya/transpiler-python-to-rust/PRD.md`, `README.md` (no README.md anywhere in repo), new-path members, cross-ref/per-service-layout/aggregation gate packets. Migration is at 0%. |

## Top conformance gaps (priority order)

1. **Brand violation is total and load-bearing (item 17).** `oyapy` is the project's own codename and saturates crate names, source (181 `.rs` lines incl. `oyapy-core/src/lib.rs:1`), AGENTS.md, ADR-0001, all docs, all `fixtures/manifests/*.json`, and `python/oyapy_analyzer.py`. Sibling codename `oyago` also referenced. This is not cosmetic — it gates items 8, 9, 12, 16-17.
2. **Wrong home + wrong layout (items 1, 4, 48).** Code sits at root `crates/`; must move to `oya/transpiler-python-to-rust/crates/` with the full per-service colocation skeleton (PRD/README/decisions/contracts/specs/catalog/threat-model/slos/runbooks/iac/evidence). No README.md exists at all.
3. **No layer taxonomy (items 9, 12, 25-29).** `-core`/`-runtime` are non-enum suffixes; no kernel/domain/app/adapter split; no ports-in-kernel. Needs hexagonal re-layering.
4. **One-version + member-manifest hygiene (items 21, 22, 23, 24).** Missing `version.workspace` inheritance (members hardcode `0.1.0`; workspace.package lacks `version`), missing `publish=false`, missing `license="Apache-2.0"`, missing `[lib] doctest=false`, missing `[workspace.dependencies]` seam, missing `[workspace.metadata.oya]` microservice registration. rust-version `1.96` ≠ source `1.95.0`; resolver `3` ≠ source `2`.
5. **Governance/build files entirely absent (items 35-38, 40-42).** No `deny.toml`, no `BUCK`/`.buckconfig`/`.buckroot`, no `registry/`. (Buck2 is Proposed/two-way → target, not hard-fail; deny.toml is hard policy.)

## Rename / reshape work needed (concrete)

- **Crate renames (use `git mv`):**
  - `crates/oyapy-cli` → `oya/transpiler-python-to-rust/crates/oya-transpiler-python-to-rust-cli` (layer `cli` ✓).
  - `crates/oyapy-core` → decompose into `oya-transpiler-python-to-rust-kernel` (pure types + ports, no_std-capable) + `oya-transpiler-python-to-rust-domain`/`-usecase` (validation/mapping logic) + `oya-transpiler-python-to-rust-app` (composition root).
  - `crates/oyapy-runtime` → `oya-transpiler-python-to-rust-infrastructure` (or a runtime-shim `*-adapter-<backend>` implementing a kernel port).
- **Microservice slot:** register `transpiler-python-to-rust` in source root `[workspace.metadata.oya.microservices]` (owner + rationale + adr_cite). Confirm slot2 token (1..3 kebab tokens) with registry; `transpiler-python-to-rust` is 4 tokens — likely needs a registry-approved short name (e.g. `pyrust-transpiler` or BC-token split). **DECISION NEEDED.**
- **Manifest hygiene:** add `version` to `[workspace.package]`; members → `version.workspace = true`; add `publish = false` + `license = "Apache-2.0"` per member; add `[lib] doctest = false`; create `[workspace.dependencies]` seam; set `resolver = "2"`; align `rust-version = "1.95.0"`. (On full merge, root `[workspace]` dissolves into source root.)
- **Brand purge:** rewrite every `oyapy`/`oyago` occurrence in source, docs, fixtures, manifests, lib doc-comments, and rename `python/oyapy_analyzer.py`. Product prose → "Oyatie" / "Python→Rust transpiler"; no migration-arrow residue left behind (guard against item 18 regression).
- **Per-service skeleton:** create `oya/transpiler-python-to-rust/{PRD.md (≥5 stories), README.md, decisions/ (rehome+renumber ADR-0001 as service-scoped), contracts/, specs/, catalog/<crate>.yaml, runbooks/, threat-model.md (assert DataClass N/A here), slos/, iac/, evidence/multispectrum/}`. Recast capability/support/mapping matrices as generated views.
- **Governance/build:** inherit source `deny.toml`; add per-crate `BUCK` + `.buckconfig`/`.buckroot` + `third-party/` (Reindeer) per ADR-0392/0408 (target-state). Adopt source AGENTS.md trust-boundary + worktree→`dev`-PR→`oya gate run-all` protocol.
- **Buildability bar:** upgrade ADR-0001 to add named industry sources, In-house roadmap, and Positive/Negative/Operational consequence labels; author PRD + IP-NNN docs ≥150 substantive lines.

## Blockers / open questions

- **No source-side service home exists yet.** `oya/transpiler-python-to-rust/` is a target, not present in either repo's tree — reshape is greenfield migration, not a move-in-place.
- **Microservice name length:** `transpiler-python-to-rust` exceeds the BNF 1..3-token microservice budget (item 9). A registry-approved short slot name is required before crate renames can finalize. **Founder/registry decision needed.**
- **Non-Rust analyzer:** `python/oyapy_analyzer.py` (105 KB) is the Python-side analyzer — not a Cargo crate; needs a sanctioned home + rename, and a policy decision on whether a Python tool lives inside an `oya/` Rust service tree or as a service-scoped tool with explicit classification.
- **`.omx/` state dir** at repo root contains stray brand-residue state files (`notify-fallback-authority-*.json`) — out of scope for crate policy but should not migrate.
- No CI, no `deny.toml`, no Buck files means items 34/43/48 gate packets cannot be evidenced today; verdicts there are documentary-only.
