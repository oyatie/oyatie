---
title: Monorepo Conformance Audit — codex (OpenAI Codex Rust SDK)
status: Read-only audit artifact
date: 2026-06-06
repo: /Users/jasonlee/Developer/codex
auditor: workflow-subagent
checklist: ./00-policy-checklist.md
target_home: cloud/cloud-intelligence/<...>/codex-adapter (per task brief)
package_under_audit: openai-codex-sdk
note: |
  READ-ONLY. No files in the audited repo were modified. Verdicts cite evidence
  (paths relative to repo root /Users/jasonlee/Developer/codex unless absolute).
---

# Monorepo Conformance Audit — `codex`

## 0. What this repo actually is (evidence)

A **single-crate Rust SDK**, nothing else. Full topology:

- Only manifest: `sdk/rust/Cargo.toml` (a second copy exists at
  `sdk/rust/target/package/openai-codex-sdk-0.1.0-beta.1/Cargo.toml` — that is a
  `cargo package` build artifact, not a real member).
- Crate: `[package].name = "openai-codex-sdk"`, `version = "0.1.0-beta.1"`,
  `edition = "2021"`, `rust-version = "1.74"`, `license = "Apache-2.0"`,
  `repository = "https://github.com/openai/codex"`.
- `[lib].name = "openai_codex_sdk"`, `path = "src/lib.rs"`.
- Source: `sdk/rust/src/{lib,app_server,async_app_server,codex,error,events,exec,input,items,options,protocol_schema,schema,thread}.rs`;
  `examples/{app_server,quickstart,streaming}.rs`; `tests/*.rs`;
  `protocol/codex_app_server_protocol.v2.schemas.json`; `scripts/check_app_server_methods.py`.
- Dependencies (`sdk/rust/Cargo.toml`): `serde`, `serde_json`, `tempfile`,
  `tokio` (optional, behind `async` feature). All Class-A community-standard.
- **It is a `std` crate** (no `#![no_std]` anywhere in `src/`). It spawns the
  Codex CLI as a child process and reads JSONL: `src/exec.rs:5` uses
  `std::process::{Child,Command,Stdio}` + `std::io::{BufRead,BufReader,Read,Write}`;
  `src/async_app_server.rs` wraps blocking calls in `tokio::task::spawn_blocking`.
- **No traits / ports** of its own (only one doc-comment reference to
  `std::io::trait.BufRead` in `src/exec.rs:209`). No `static mut` / `OnceLock` /
  `Lazy` / `lazy_static` in `src/`.

**BLOCKER-CLASS finding (provenance):** the repo has **no git history** —
`git log` → "your current branch 'main' does not have any commits yet";
`git status` shows the entire `sdk/` tree as **untracked**; `git remote -v` is
empty. There is therefore **no upstream remote and no commit lineage** to drive
the `git mv` (history-preserving) migration that checklist item 47 / AGENTS.md
mandates. The `sdk/rust/target/` build directory (incl. `target/package/…`) is
present and untracked, and the only `.gitignore` is `sdk/rust/.gitignore`
(repo-root has none).

---

## 1. Per-checklist-item verdict

Legend: CONFORMS / NEEDS-RESHAPE / NEEDS-RENAME / VIOLATES / N-A (not yet
applicable — single external SDK, no services).

### A. Canonical homes & topology

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 1 | Code only at `{oya,cloud}/<service>/crates/<crate>/` | **NEEDS-RESHAPE** | Code lives at `sdk/rust/`. Must relocate under `cloud/cloud-intelligence/<svc>/crates/codex-adapter` (per task brief). `cloud/` is correct band (platform/intelligence substrate, not product domain). |
| 2 | Shared code only at `libs/<lib>/` | N-A | No shared lib carved out; if the transport types are to be reused cross-service they belong in a `libs/` kernel. Currently monolithic. |
| 3 | `microservices/` forbidden | CONFORMS | Not used. |
| 4 | Per-service colocation (PRD, README, decisions/, contracts/, specs/, catalog/<crate>.yaml, runbooks/, threat-model, slos/, iac/, evidence/) | **VIOLATES** | Only `README.md` + ad-hoc docs (`SPEC.md`, `PLAN.md`, `CONTRACT_PARITY.md`, `PARITY_CLOSURE.md`, `FULL_SHEBANG.md`, `RELEASE.md`, `CHANGELOG.md`) at `sdk/rust/`. No PRD.md, no `decisions/`, no `contracts/{openapi,asyncapi,proto}/`, no `catalog/<crate>.yaml`, no `runbooks/`, no `threat-model.md`, no `slos/*.openslo.yaml`, no `iac/`, no `evidence/multispectrum/`. The `protocol/*.v2.schemas.json` is the natural seed for `contracts/`. |
| 5 | Cross-cutting artifacts central only | N-A | No cross-cutting artifacts authored here. |
| 6 | Aggregation indices generated | N-A | None present. |
| 7 | Packs only under `packs/` | CONFORMS | No pack artifacts. |

### B. Crate naming — BNF v4.1 + 13-layer enum

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 8 | `oya-` prefix mandatory | **VIOLATES / NEEDS-RENAME** | `name = "openai-codex-sdk"` — third-party vendor prefix. Must become `oya-codex-adapter-…` family (or `oya-codex-sdk-sdk`? no — see #12). |
| 9 | BNF v4.1 `oya-<microservice>(-<bc>)?-<layer>` | **NEEDS-RENAME** | `openai-codex-sdk` parses as `<vendor>-codex-sdk`; not registry-validated; microservice token not `oya-…`. Proposed: microservice `codex`, layer `adapter` → at minimum `oya-codex-adapter` (the brief's `codex-adapter` directory basename). If a thin client SDK surface is also published, that is a separate `…-sdk` crate. |
| 10 | `[package].name` == directory basename | **NEEDS-RESHAPE** | Today basename is `rust` (dir `sdk/rust/`), name is `openai-codex-sdk` → mismatch. After move to `cloud/.../crates/oya-codex-adapter/`, name must equal `oya-codex-adapter`. |
| 11 | `[lib].name` == snake_case(package.name) | **NEEDS-RENAME** | Currently `openai_codex_sdk` (matches the *current* package name, so internally consistent) but must follow the renamed package → `oya_codex_adapter`. |
| 12 | Layer suffix ∈ closed 13-enum | **NEEDS-RENAME** | `sdk` IS a legal layer in the enum. But this crate is not a pure protocol-typed `sdk` (which may depend on `kernel` only — see #28); it shells out to a CLI via `std::process` (an outbound-I/O driver). That makes it an **`adapter`** (or split: `kernel` for protocol types + `adapter` for the CLI driver). Layer must be reclassified to `adapter`, not `sdk`. |
| 13 | `*-adapter-<backend>` impls ≥1 port trait from matching `*-kernel`; `oya-check-*` family rules | **VIOLATES (structural)** | Crate defines NO port traits and has NO matching `*-kernel`. A CLI-spawning adapter must implement a port trait declared in an `oya-codex-kernel`. As-is it is a flat std monolith with no hexagonal seam. |
| 14 | `tools/` crates explicit suffix | N-A | No `tools/` crate. |
| 15 | Microservice slot2 registered in `[workspace.metadata.oya.microservices.<name>]` | **VIOLATES** | Not registered (no workspace metadata at all here; registration happens in the destination root `Cargo.toml`). `codex` microservice slot must be added with owner + rationale + adr_cite. |

### C. Brand residue

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 16 | `oyatie-*` cargo prefix forbidden | CONFORMS | grep for `oyatie` across `*.rs/*.toml/*.md` → no matches. |
| 17 | Codename residue forbidden (`oyaoffice/oyago/oyapy/kuberos/talos/foundry/…`) | CONFORMS | grep for all → no matches. (The reverse risk applies: this is an *OpenAI* vendor SDK; `openai`/`codex` vendor identifiers are expected and are not OYA-brand violations, but the crate prefix still must be re-homed to `oya-` per #8.) |
| 18 | No tautological rebrand residue (arrows / "retired terms" / "after rename") | CONFORMS | No rebrand-arrow / retired-terms-table / rename-phrase residue (this repo predates any OYA rebrand; nothing to clean). |

### D. One-workspace invariants

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 19 | Exactly ONE `[workspace]` (repo root); no nested | **NEEDS-RESHAPE** | `sdk/rust/Cargo.toml` has NO `[workspace]` block (grep `^\[workspace\]` → none). Good: no nested-workspace conflict. But it is currently a standalone package crate; on merge it must become a plain member of the destination root workspace (delete `Cargo.lock` here — one lockfile lives at the monorepo root). |
| 20 | `resolver = "2"` | N-A (inherited) | No `[workspace]` here; resolver is set at destination root. |
| 21 | `version/edition/rust-version` `.workspace = true` | **VIOLATES** | Hard-pins `version = "0.1.0-beta.1"`, `edition = "2021"`, `rust-version = "1.74"`. Must switch to `version.workspace = true`, `edition.workspace = true` (root is **2024**), `rust-version.workspace = true` (root **1.95.0**). Edition 2021→2024 + MSRV 1.74→1.95 is a real migration delta. |
| 22 | Each member: `publish=false` + `license="Apache-2.0"` + `[lints] workspace=true` | **NEEDS-RESHAPE** | `license = "Apache-2.0"` ✓. But `publish` is unset (currently publishable — has `keywords`/`categories`/`repository` for crates.io) → must add `publish = false`. No `[lints] workspace = true` → must add. |
| 23 | `[lib] doctest = false` | **VIOLATES** | `[lib]` block present (`src/lib.rs`) but no `doctest = false`. Must add. |
| 24 | Workspace dep seam + `registry/dependency-rationales.json` rows | **NEEDS-RESHAPE** | All deps declared inline with literal versions (`serde "1.0"`, `serde_json "1.0"`, `tempfile "3"`, `tokio "1"`). Must convert to `.workspace = true` and add rationale rows at destination. `tokio` stays feature-gated (`async`). |

### E. Hexagonal clean architecture

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 25 | Inward-only dep flow per 13-layer matrix | **VIOLATES (structural)** | Single flat crate; no layer separation to enforce flow against. |
| 26 | Port traits in `kernel`, impls in `adapter` | **VIOLATES** | No traits at all (only a doc-comment ref in `src/exec.rs:209`). Needs an `oya-codex-kernel` defining the transport port + `oya-codex-adapter` implementing it over `std::process`. |
| 27 | `kernel` pure types + ports: zero biz-logic / I/O / async; no_std-capable | **VIOLATES** | Today everything (protocol types in `schema.rs`/`items.rs`/`events.rs`/`protocol_schema.rs` AND the process-spawning transport in `exec.rs`/`app_server.rs` AND async wrappers) is fused in one `std` crate. The pure-type/protocol-schema modules are kernel-shaped and could be lifted to a no_std-friendly `oya-codex-kernel`; the I/O lives in the adapter. |
| 28 | `api`/`sdk` depend on `kernel` only | **NEEDS-RESHAPE** | If a published `sdk` surface is kept, it must depend on the new kernel only and contain no `std::process` driver (that moves to the adapter). |
| 29 | Only `app` has unrestricted inward deps; `app→app` forbidden | N-A | No `app` crate. |
| 30 | No direct cross-microservice imports (LEAN-A2); `public_layers` allowlist | N-A | No cross-service imports today. On merge, expose only via `public_layers = ["sdk"]` (or adapter port) if other services consume Codex. |

### F. Data governance & quality gates

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 31 | `data_class` on kernel data fields w/ tenant/regulated data | **NEEDS-RESHAPE** | Prompt/response payloads flowing to/from the OpenAI Codex CLI may carry PII / source code (regulated). The protocol structs (`items.rs`, `events.rs`, `input.rs`) carry no `DataClass` annotation. After kernel split, fields routing user content must declare `data_class`. |
| 32 | Statelessness (no module-level mutable state) | CONFORMS | No `static mut`/`OnceLock`/`Lazy`/`lazy_static` in `src/`. |
| 33 | Shardability (`tenant_id` + RLS) | N-A | No DB layer. |
| 34 | Buildability bar (PRD ≥5 stories, IP ≥150 lines, ADR Context+Decision+≥3 alts+≥3 consequences, scorecard evidence) | **VIOLATES** | No PRD, no IP, no ADR, no scorecard. Existing docs (`SPEC.md`, `PLAN.md`, `CONTRACT_PARITY.md`, `PARITY_CLOSURE.md`, `FULL_SHEBANG.md`) are vendor-parity notes, not buildability-bar artifacts. Must author PRD.md (≥5 stories) + an integration plan + a service-scoped ADR justifying the Codex adapter. |

### G. Supply chain & license policy

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 35 | Root `deny.toml` license allowlist | N-A (no local deny.toml) | No `deny.toml` in repo. Direct deps (serde/serde_json/tempfile/tokio) are MIT/Apache — allowlist-compatible. Inherits destination root `deny.toml`. |
| 36 | `[bans]` deny openssl/openssl-sys/old-time | CONFORMS (incidental) | No `openssl`/`openssl-sys`/`old-time` in `Cargo.lock`. No TLS dep at all (it shells out to the CLI). |
| 37 | `[sources]` crates.io only | **NEEDS-VERIFY at merge** | `Cargo.lock` deps look like crates.io packages; no git/path source entries observed. Re-resolve against root deny.toml `[sources]` on merge. |
| 38 | `[advisories]` yanked=deny, empty ignore | N-A (inherited) | Enforced at destination root. |
| 39 | Vendor classification (A/B/C; registry rows for B) | **NEEDS-RESHAPE** | The *crate itself* is a vendor bridge to the OpenAI Codex CLI → classic **Class B vendor-lockin** (replaceable behind a port). Must register in `registry/vendor-lockin-phaseout/index.json` with seam trait (the transport port) + impl (`oya-codex-adapter`) + replacement_path + value-anchored trigger. This is the strongest doctrinal reason to insert a `kernel` port (#26). |

### H. Buck2 build graph

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 40 | Per-crate `BUCK` (rust_library/binary, crate_root, visibility, deps) | **VIOLATES** | No `BUCK` files anywhere (find → none). Must add `BUCK` for each resulting crate (`oya-codex-kernel`, `oya-codex-adapter`) pointing at `third-party//:serde` etc. |
| 41 | Third-party buckified by Reindeer | N-A here | Generated at destination root `third-party/`. |
| 42 | `.buckconfig` + `.buckroot` present | **VIOLATES** | Neither present (find → none). Inherited from destination root on merge; not authored per-crate. |
| 43 | NativeLink RBE only; Jenkins buck2 CI | N-A | Doctrine/target; nothing CI-related in this repo. |

### I. Agent / contribution protocol

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 44 | Sanctioned primitives `git`/`oya-gate`/`oya-verify` | N-A | No CI/agent config in repo. |
| 45 | Worktree→commit→PR→`dev`→CI+`oya gate`→reviewer merge | **BLOCKED** | Cannot follow: repo has **no commits and no remote** (see §0). Merge must be done as a fresh `git mv` import into the oyatie monorepo, not a cross-repo PR. |
| 46 | Trust boundary (tool/web output = data) | N-A | No agent instructions embedded. |
| 47 | `git mv` (never rm+add) for moves | **BLOCKED / VIOLATES** | **No git history exists to preserve** — `sdk/` is entirely untracked, no prior commits. The "preserve history via `git mv`" guarantee is unsatisfiable from this source as-is. Relocation will be an add-with-provenance-note, OR commit-then-`git mv` inside the destination repo. This is the headline blocker. |

### J. Migration completion gate

| # | Item | Verdict | Evidence / note |
|---|------|---------|-----------------|
| 48 | Done = PRD+README exist, zombies removed, members updated, `cargo build/nextest`=0, gate packets=0 | **NOT MET** | No PRD; not yet a workspace member; build/test parity vs destination unverified; gate packets not run. |

---

## 2. Conformance scorecard (summary)

- CONFORMS: 16, 18, 32, 36, 3, 7 (6 items, mostly "absent so not violated").
- NEEDS-RENAME: 8, 9, 11, 12 (crate/lib naming + layer reclassification).
- NEEDS-RESHAPE: 1, 10, 19, 22, 24, 28, 31, 39 (re-home, manifest inheritance, dep seam, kernel/adapter split, data_class, vendor registry).
- VIOLATES (structural / artifact-gap): 4, 13, 15, 21, 23, 25, 26, 27, 34, 40, 42, 48.
- BLOCKED: 45, 47 (no git history / no remote).
- N-A (no service surface yet): 2, 5, 6, 14, 29, 30, 33, 35, 38, 41, 43, 44, 46.

---

## 3. Fit-work needed (reshape / rename plan to land in `cloud/cloud-intelligence/.../codex-adapter`)

1. **Re-home** the crate from `sdk/rust/` to `cloud/cloud-intelligence/<service>/crates/oya-codex-adapter/` (band `cloud/` = correct: platform intelligence substrate, not product `oya/`). Register the `codex` microservice slot2 in the destination root `[workspace.metadata.oya.microservices.codex]` (owner + rationale + adr_cite). [items 1, 10, 15]
2. **Hexagonal split (the big one).** Carve the flat `openai-codex-sdk` into:
   - `oya-codex-kernel` — pure protocol/event/item types lifted from `schema.rs`, `items.rs`, `events.rs`, `input.rs`, `options.rs`, `protocol_schema.rs`; define the transport **port trait**; make no_std-capable where feasible; zero I/O / async. Add `data_class` to fields carrying user prompts / source-code payloads (PII / regulated). [items 13, 26, 27, 31]
   - `oya-codex-adapter` — the `std::process::Command` CLI driver (`exec.rs`, `app_server.rs`) + `async_app_server.rs` (tokio behind `async` feature); implements the kernel port. Mark as `*-adapter-<backend>` (backend = the OpenAI Codex CLI). [items 12, 13, 26, 28]
   - Optional `oya-codex-sdk` only if a thin public client surface is published; depends on `kernel` only. [item 28]
3. **Rename** `openai-codex-sdk` → `oya-codex-adapter` (+ kernel/sdk siblings); set `[lib].name` to `oya_codex_adapter` (snake) etc. Vendor identifiers `openai`/`codex` stay as the *microservice/backend* tokens — only the prefix and layer change. [items 8, 9, 11, 12]
4. **Manifest one-version conversion:** drop hard-pins; adopt `version.workspace`, `edition.workspace` (2021→**2024**), `rust-version.workspace` (1.74→**1.95.0**); add `publish = false`, `[lints] workspace = true`, `[lib] doctest = false`; remove crates.io `keywords`/`categories`/`repository` publish metadata; delete the local `Cargo.lock` (single root lockfile). [items 21, 22, 23]
5. **Dependency seam:** move serde/serde_json/tempfile/tokio to `.workspace = true`; add `registry/dependency-rationales.json` rows; keep `tokio` feature-gated. [item 24]
6. **Vendor-lockin registry:** add a Class-B entry in `registry/vendor-lockin-phaseout/index.json` (seam = transport port, impl = `oya-codex-adapter`, replacement_path, value-anchored trigger — no date anchors). [item 39]
7. **Per-service colocation artifacts:** author `PRD.md` (≥5 user stories, measurable), an integration/implementation plan (≥150 substantive lines), a service-scoped ADR (Context+Decision+≥3 alternatives+≥3 consequences+sources+in-house roadmap), `README.md`, `decisions/`, `contracts/` seeded from `protocol/codex_app_server_protocol.v2.schemas.json`, `catalog/oya-codex-adapter.yaml`, `runbooks/`, `threat-model.md`, `slos/*.openslo.yaml`, `iac/`, `evidence/multispectrum/`. The existing `SPEC.md`/`CONTRACT_PARITY.md`/`PARITY_CLOSURE.md`/`PLAN.md`/`FULL_SHEBANG.md` are useful raw material but are not bar-compliant artifacts. [items 4, 34]
8. **Buck2:** add per-crate `BUCK` files (rust_library, crate_root, visibility, deps → `//cloud/...:oya-codex-kernel` + `third-party//:serde` etc.). `.buckconfig`/`.buckroot` inherited from destination root. [items 40, 42]
9. **Drop build cruft:** do not import `sdk/rust/target/` (incl. `target/package/openai-codex-sdk-0.1.0-beta.1/`); it is a `cargo package` artifact, not source.

---

## 4. Blockers

- **B1 (history / provenance) — HARD.** The codex repo has **no git commits and no remote** (`git log` → no commits; `git status` → `sdk/` untracked; `git remote -v` empty). Checklist items 45 & 47 (PR-against-`dev` flow; history-preserving `git mv`) are **unsatisfiable as-is** — there is no lineage to preserve and no upstream to PR from. The migration must be a fresh, attributed import into `jason931225/oyatie` (commit the source first, then `git mv` within the destination), with a provenance note recording the OpenAI upstream (`repository = "https://github.com/openai/codex"`). Founder sign-off recommended before importing un-versioned third-party SDK source.
- **B2 (architecture debt).** The crate has no port traits / no kernel-adapter seam; conformance to E (25–28) and item 13/39 requires a real hexagonal split, not a rename — this is the largest unit of work.
- **No missing-source blocker:** the SDK source itself is present and complete (13 src modules + protocol schema + tests + examples).
