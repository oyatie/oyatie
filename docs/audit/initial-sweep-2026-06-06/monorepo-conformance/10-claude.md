---
title: Monorepo Conformance Audit — /Users/jasonlee/Developer/claude (claude-agent-sdk)
status: Audit-finding
date: 2026-06-06
auditor: workflow-subagent (read-only)
checklist: ./00-policy-checklist.md
repo: /Users/jasonlee/Developer/claude
target_home: cloud/cloud-intelligence/<service>/crates/<crate>  (Claude SDK = anthropic-claude-adapter)
note: READ-ONLY. No file in the target repo was modified. Verdicts cite evidence (paths, Cargo.toml fields, grep results).
---

# 10 — claude (claude-agent-sdk) conformance audit

## Repo snapshot (evidence)

- Single crate, flat layout. Exactly ONE `Cargo.toml` at repo root (`find … -name Cargo.toml -not -path '*/target/*'` → only `/Users/jasonlee/Developer/claude/Cargo.toml`). NO nested `[workspace]`; NO `[workspace]` at all (it is a bare `[package]`).
- `[package].name = "claude-agent-sdk"`, `version = "0.1.2"`, `edition = "2024"`, `rust-version = "1.85"`, `license = "MIT"`, `repository = "https://github.com/anthropics/claude-agent-sdk-rust"`.
- Source: 17 flat modules under `src/` (`assistant, bridge, callbacks, client, direct_connect, error, messages, options, query, runtime, session_store, sessions, settings, status, tools, transport`, + `lib.rs`). No `src/crates/`.
- `std` + async (Tokio). `grep no_std src/` → empty (no `#![no_std]`). Uses `tokio::process`, optional `reqwest`/`tokio-tungstenite` behind a `network` feature.
- Dependencies: futures, serde, serde_json, thiserror, tokio, uuid; optional http/reqwest/tokio-tungstenite. dev-dep: tempfile.
- Tests: 13 integration tests under `tests/` (fake-CLI driven). Examples: `examples/quick_start.rs`. Docs: `docs/PLAN.md`, `docs/SPEC.md` (SPEC names upstream parity sources + commit snapshots).
- NO `BUCK` / `.buckconfig` / `.buckroot` (`find` → none).
- NO vendored dirs (`vendor/`, `third-party/` absent).
- Brand residue scan `grep -niE 'oyatie|oyaoffice|oyago|oyapy|kuberos|talos|foundry|\boya-|\boya\b'` over `src/ docs/ README.md Cargo.toml` → ZERO hits. (Repo uses upstream "claude/anthropic" naming, not OMC/oyatie codenames.)
- `data_class` / `DataClass` → ZERO hits in `src/`.
- `.gitignore` ignores `.omx/`, `.omc/`, `target`, `.env*`. A `.omx/` dir exists (OMC working state, gitignored — not product surface).

## Per-checklist verdict

### A. Canonical homes & topology

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 1 | Code only at `{oya,cloud}/<service>/crates/<crate>/` | **VIOLATES** | Code lives at repo root `src/*.rs` as a standalone crate. As a sibling repo it has no `oya/` or `cloud/` topology. Target home per task: `cloud/cloud-intelligence/anthropic-claude-adapter/crates/<crate>/`. Reshape required. |
| 2 | Shared cross-cutting only at `libs/<lib>/` | **N/A → NEEDS-RESHAPE** | No `libs/`; this is a leaf SDK, not shared governance. On migration, the protocol/types kernel may need to land under the service's `crates/`, not `libs/`. |
| 3 | `microservices/` forbidden | **CONFORMS** | No `microservices/` dir. |
| 4 | Per-service colocation (PRD/README/PHASE/IP/decisions/contracts/specs/catalog/runbooks/threat-model/slos/iac/src/crates/tests/evidence) | **VIOLATES** | Has `README.md`, `docs/PLAN.md`, `docs/SPEC.md`, `tests/`, `examples/`. MISSING: `PRD.md`, `PHASE-NN-*.md`, `IP-NNN-*.md`, `decisions/`, `contracts/{openapi,asyncapi,proto}/`, `specs/`, `catalog/<crate>.yaml`, `runbooks/`, `threat-model.md`, `slos/*.openslo.yaml`, `iac/`, `evidence/multispectrum/`, and `src/crates/` (code is flat `src/`, not `src/crates/`). Heavy authoring gap. |
| 5 | Cross-cutting artifacts central only | **N/A** | No cross-cutting artifacts authored here. |
| 6 | Aggregation indices generated | **N/A** | None present. |
| 7 | Packs only under `packs/` | **CONFORMS (vacuous)** | No pack artifacts. |

### B. Crate naming — BNF v4.1 + 13-layer enum

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 8 | `oya-` prefix mandatory | **NEEDS-RENAME** | `claude-agent-sdk` has no `oya-` prefix. Must become e.g. `oya-anthropic-claude-sdk` / `oya-anthropic-claude-adapter-*`. |
| 9 | BNF v4.1 `oya-<microservice>(-<bc>)?-<layer>` | **NEEDS-RENAME** | `claude-agent-sdk` — `sdk` IS a valid layer token, but the microservice slot (`claude-agent`) is unregistered and lacks `oya-`. Reshape into hexagonal crates with proper layer suffixes (see below). |
| 10 | `[package].name == dir basename` | **NEEDS-RESHAPE** | Currently package `claude-agent-sdk` lives at repo root (basename `claude`), so name ≠ basename. After migration into `crates/<crate>/`, basename must equal the renamed `oya-…` package. |
| 11 | `[lib].name == snake_case(package.name)` | **NEEDS-RESHAPE** | No explicit `[lib]` block; defaults to `claude_agent_sdk`. After rename must be explicit `oya_…`. |
| 12 | Layer suffix ∈ 13-value enum | **NEEDS-RESHAPE** | One monolithic crate spanning kernel-like types (`messages`, `options`, `error`), adapter/infra (`transport`, `direct_connect`, `bridge`, `runtime`), and SDK surface (`client`, `query`, `sessions`). Must be split: pure protocol/message types → `*-kernel`; subprocess/network transport → `*-adapter-subprocess` / `*-adapter-cli`; public client surface → `*-sdk`. |
| 13 | check-family / `*-adapter-<backend>` impls ≥1 kernel port | **NEEDS-RESHAPE** | `transport.rs` already has port-shaped traits (`ClaudeProcessSpawner`, `SpawnedClaudeProcess`) — promising seam. Formalize: ports in `*-kernel`, the subprocess/network impls as `*-adapter-<backend>` impling those ports. No check crates needed (leaf SDK). |
| 14 | `tools/` crates explicit suffix | **N/A** | No `tools/` crates. (`src/tools.rs` is SDK in-process tool support, unrelated.) |
| 15 | Microservice slot registered in root Cargo.toml | **VIOLATES** | No root workspace, so no `[workspace.metadata.oya.microservices.<name>]`. On migration, register `anthropic-claude` (or `cloud-intelligence` service) with owner + rationale + adr_cite. |

### C. Brand residue

| # | Item | Verdict | Evidence |
|---|------|---------|----------|
| 16 | `oyatie-*` Cargo prefix forbidden | **CONFORMS** | No `oyatie-*` prefix (no `oya`/`oyatie` token at all). The required rename ADDS `oya-`; it does not remove residue. |
| 17 | Codename residue forbidden (oyaoffice/oyago/oyapy/kuberos/foundry/oyatie-*/talos-*) | **CONFORMS** | grep over `src/ docs/ README.md Cargo.toml` → ZERO hits. "claude"/"anthropic"/"Claude Code" are legitimate upstream product names, not OMC codenames. |
| 18 | No tautological rebrand residue (arrows / "retired terms" / "after rename") | **CONFORMS** | None present (repo predates any rebrand framing). |

### D. One-workspace invariants

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 19 | Exactly ONE `[workspace]` (repo root) | **NEEDS-RESHAPE** | As a sibling repo it has NO `[workspace]` (bare `[package]`). It does NOT carry an illegal nested workspace — GOOD. On migration it must drop into the source root workspace as a member (no own `[workspace]`). |
| 20 | `resolver = "2"` | **NEEDS-RESHAPE** | Not declared (edition 2024 implies resolver 3 behavior; no explicit resolver). Inherited from root workspace post-migration. |
| 21 | Members inherit `version/edition/rust-version .workspace = true` | **VIOLATES** | All pinned locally (`version = "0.1.2"`, `edition = "2024"`, `rust-version = "1.85"`). Note `rust-version 1.85 ≠ workspace 1.95.0`, and `version 0.1.2 ≠ 0.1.0`. Must switch to `.workspace = true`. |
| 22 | `publish = false` + `license = "Apache-2.0"` + `[lints] workspace = true` | **VIOLATES** | No `publish` field (defaults publishable — it even sets `documentation`/`keywords`/`categories` for crates.io). `license = "MIT"` (must become `Apache-2.0`). No `[lints]` block. |
| 23 | `[lib] doctest = false` | **VIOLATES** | No `[lib]` block at all; doctest not disabled. |
| 24 | Single `[workspace.dependencies]` seam + rationale rows | **VIOLATES** | Deps declared inline with raw version strings (`futures = "0.3"`, `tokio = {…}`, etc.), not `.workspace = true`. On migration each must move to root `[workspace.dependencies]` + a `registry/dependency-rationales.json` row. |

### E. Hexagonal clean architecture

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 25 | Inward-only dependency flow | **NEEDS-RESHAPE** | Single crate → no enforced layer matrix. Module shape is loosely layered but not crate-separated. Split required (see #12). |
| 26 | Port traits in `kernel`, impls in `adapter` | **PARTIAL → NEEDS-RESHAPE** | Good news: trait/impl seam already exists (`ClaudeProcessSpawner` port + `SpawnedClaudeProcess`, `SessionStore` trait). But ports and impls co-reside in one crate. Move ports to `*-kernel`, impls to `*-adapter-*`. |
| 27 | `kernel` = pure types, no I/O/async, no_std-capable | **NEEDS-RESHAPE** | `messages.rs`/`options.rs`/`error.rs` are largely pure serde types and are good kernel candidates, BUT the crate is uniformly `std` + async with no `no_std` boundary. Extract a std-free/no_std-capable types kernel; keep async/I/O in adapter/infrastructure. |
| 28 | `api`/`sdk` depend on `kernel` only | **NEEDS-RESHAPE** | The public surface (`client`, `query`) currently depends on transport/runtime directly. A clean `*-sdk` crate should depend on `*-kernel` (+ adapter via composition), not reach into infra. |
| 29 | Only `app` has unrestricted inward deps | **N/A → NEEDS-RESHAPE** | No composition-root `app` crate; the monolith IS the de-facto root. Introduce an `*-app`/composition crate if a runnable service is required. |
| 30 | No direct cross-microservice imports | **CONFORMS (vacuous)** | Leaf SDK; no cross-service imports. As a `cloud` adapter it must expose only via `public_layers = ["sdk"]` once placed. |

### F. Data governance & quality gates

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 31 | `data_class` on tenant/regulated kernel fields | **VIOLATES** | ZERO `data_class`/`DataClass` annotations. SDK message/options types carry prompt + session content (arguably internal/PII-adjacent); kernel fields must declare `DataClass` post-split. |
| 32 | Statelessness (no module-level mutable state) | **NEEDS-VERIFY** | Not audited line-by-line; presentation/worker layers don't exist as such. Re-check after split. |
| 33 | Shardability (`tenant_id` + RLS) | **N/A** | No DB/persistence-with-tenancy; `SessionStore` is an abstract trait. |
| 34 | Buildability bar (PRD≥5 stories / IP≥150 lines / ADR 3+ alts & consequences / scorecard) | **VIOLATES** | No PRD.md, no IP-NNN, no ADRs. `docs/PLAN.md` + `docs/SPEC.md` exist (SPEC cites upstream parity sources + pinned commits — decent raw material) but do not meet the ADR-0212 buildability artifact bar. |

### G. Supply chain & license policy

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 35 | Root `deny.toml` license allowlist | **VIOLATES** | No `deny.toml` in this repo. Crate `license = "MIT"` (MIT IS in the allowlist, but crate license must become `Apache-2.0` per #22). Covered by root deny.toml post-migration. |
| 36 | `[bans]` deny openssl (rustls-only) + old-time | **CONFORMS (de-facto)** | Strong positive: reqwest uses `default-features = false, features=["rustls-tls"]`; tungstenite uses `rustls-tls-webpki-roots`. No openssl pulled in. No own `deny.toml` to formalize the ban yet. |
| 37 | `[sources]` crates.io only | **NEEDS-VERIFY** | `Cargo.lock` present; no git/unknown sources observed in deps. Confirm under root deny.toml. |
| 38 | `[advisories]` yanked=deny | **N/A (no deny.toml)** | Inherited post-migration. |
| 39 | Vendor classification A/B/C (ADR-0211) | **VIOLATES** | No `registry/vendor-lockin-phaseout/index.json` participation. Deps need A/B/C classification: tokio/serde/futures → Class A; reqwest/tokio-tungstenite → likely Class B (network adapter seam already exists, good for replaceability). |

### H. Buck2 build graph

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 40 | Per-crate `BUCK` file | **VIOLATES** | No BUCK files. Author per-crate `BUCK` (`rust_library`/`rust_binary`) after the crate split. |
| 41 | Third-party buckified by Reindeer | **VIOLATES** | No `third-party/{BUCK,fixups/}`. Covered by root once migrated. |
| 42 | `.buckconfig` + `.buckroot` present | **VIOLATES** | Neither present (this repo). Inherited from source root post-migration. |
| 43 | NativeLink RBE / Jenkins CI | **N/A** | Doctrine/target; 0% adopted globally. No claim. |

### I. Agent / contribution protocol

| # | Item | Verdict | Evidence / fit-work |
|---|------|---------|---------------------|
| 44 | Sanctioned primitives (git / oya-gate / oya-verify) | **N/A** | No AGENTS.md / gate tooling in this repo. Governed by source AGENTS.md post-migration. |
| 45 | Worktree→PR→`dev`→CI+gate→approve | **N/A** | Repo has its own linear history (`git log` shows feature commits on its own line); migration must land via the source PR/gate flow. |
| 46 | Trust boundary | **N/A** | No agent protocol declared here. |
| 47 | `git mv` for moves (preserve history) | **APPLIES-AT-MIGRATION** | The reshape (root `src/*.rs` → `cloud/cloud-intelligence/anthropic-claude-adapter/crates/<crate>/src/`) MUST use `git mv`. Cross-repo move into the monorepo will likely break literal history continuity — flag as a known reshape risk. |

### J. Migration completion gate

| # | Item | Verdict |
|---|------|---------|
| 48 | Service "done" gate (PRD+README, zombies removed, members updated, build/test 0, gate packets 0) | **NOT MET** | No PRD, not yet a workspace member, no gate packets. This is a NET-NEW migration, not a finished one. |

## Conformance scorecard (rollup)

- CONFORMS: #3, #7(vacuous), #16, #17, #18, #30(vacuous), #36(de-facto) — brand + supply-chain hygiene are the bright spots.
- NEEDS-RENAME: #8, #9 (the `oya-` prefix + BNF naming).
- NEEDS-RESHAPE: #2, #10, #11, #12, #13, #19, #20, #25, #26, #27, #28, #29 (topology + hexagonal split).
- VIOLATES: #1, #4, #15, #21, #22, #23, #24, #31, #34, #35, #39, #40, #41, #42, #48 (homes, workspace inheritance, governance/Buck artifacts).
- N/A / NEEDS-VERIFY: #5, #6, #14, #32, #33, #37, #38, #43, #44, #45, #46.

## Fit-work to make `claude` conform (ordered)

1. **Relocate** into `cloud/cloud-intelligence/anthropic-claude-adapter/crates/` (cloud = platform/tenant-substrate; this is an LLM-provider adapter). Use `git mv` for the in-tree shuffle.
2. **Split the monolith into hexagonal crates** (drop the implicit `claude-agent-sdk`):
   - `oya-anthropic-claude-kernel` — pure protocol/message/option/error types (from `messages.rs`, `options.rs`, `error.rs`, parts of `status.rs`); declare `data_class` on session/prompt-bearing fields; make `no_std`-capable where feasible; host the port traits (`ClaudeProcessSpawner`, `SpawnedClaudeProcess`, `SessionStore`).
   - `oya-anthropic-claude-adapter-subprocess` (a.k.a. `-cli`) — `transport.rs`, `bridge.rs`, `runtime.rs`, `direct_connect.rs`; impls the kernel ports; carries `std` + tokio + network feature.
   - `oya-anthropic-claude-sdk` — public client surface (`client.rs`, `query.rs`, `sessions.rs`, `session_store.rs`, `callbacks.rs`, `tools.rs`, `assistant.rs`, `settings.rs`); depends on kernel (+ adapter via composition). This is the `public_layers = ["sdk"]` surface for cross-service callers.
3. **Workspace inheritance:** drop local `version/edition/rust-version`; set `version.workspace`, `edition.workspace`, `rust-version.workspace` (aligns to edition 2024 / 0.1.0 / 1.95.0). Add `publish = false`, `license = "Apache-2.0"` (relicense from MIT — founder decision/ADR needed), `[lints] workspace = true`, and `[lib] doctest = false` per crate.
4. **Dependency seam:** move every inline dep to root `[workspace.dependencies]` + `.workspace = true`; add `registry/dependency-rationales.json` rows; classify A/B/C in `registry/vendor-lockin-phaseout/index.json` (reqwest/tungstenite = Class B behind the existing network seam).
5. **Register microservice** in root `[workspace.metadata.oya.microservices.<name>]` (owner + rationale + adr_cite).
6. **Author service artifacts** (ADR-0131 colocation + ADR-0212 buildability): `PRD.md` (≥5 stories), `IP-NNN` (≥150 substantive lines), service `decisions/` ADRs (incl. the relicense + the slot2 registration + the network-adapter Class-B seam), `contracts/`, `catalog/<crate>.yaml`, `threat-model.md`, `runbooks/`, `slos/`, `evidence/multispectrum/`. Reuse `docs/SPEC.md`/`docs/PLAN.md` as raw input.
7. **Buck2:** author per-crate `BUCK` after the split; third-party handled by the root Reindeer-generated `third-party/`.

## Digest

**Top conformance gaps:** (1) topology — flat root-level single crate, not `cloud/<service>/crates/<crate>/`, and missing the entire ADR-0131 service-colocation set (PRD/IP/decisions/contracts/catalog/threat-model/slos/evidence); (2) workspace hygiene — local `version 0.1.2` / `edition 2024` / `rust-version 1.85` / `license = MIT` instead of workspace inheritance + Apache-2.0, no `publish=false`, no `[lints]`, no `[lib] doctest=false`, deps inline instead of `.workspace=true` (no rationale rows, no A/B/C vendor classification); (3) zero `data_class` annotations; (4) no Buck2 (`BUCK`/`.buckconfig`/`.buckroot`/`third-party/`); (5) buildability artifacts absent.

**Rename/reshape needed:** RENAME `claude-agent-sdk` → `oya-`-prefixed crates and SPLIT the monolith hexagonally into `oya-anthropic-claude-kernel` (pure types + ports), `oya-anthropic-claude-adapter-subprocess`/`-cli` (transport/runtime impls), `oya-anthropic-claude-sdk` (public client surface, `public_layers=["sdk"]`); RELOCATE under `cloud/cloud-intelligence/anthropic-claude-adapter/crates/`; register the microservice slot; relicense MIT→Apache-2.0 (needs an ADR). The existing `ClaudeProcessSpawner`/`SpawnedClaudeProcess`/`SessionStore` port-and-impl seam and the rustls-only network feature gate make the hexagonal split and the Class-B vendor seam straightforward.

**Bright spots (already conform):** ZERO brand residue (no oyatie/oyaoffice/kuberos/talos/foundry codenames — uses legitimate upstream "claude"/"anthropic" naming); no illegal nested `[workspace]`; rustls-only (no openssl), satisfying the spirit of `[bans]`; clean trait/impl seams ready for the kernel/adapter split.

**Blockers / risks:** No source workspace context inside this repo (no `deny.toml`/root `Cargo.toml`/`registry/` here — those live in `/Users/jasonlee/Developer/source`); cross-repo move into the monorepo will not preserve literal git history (ADR-0131 item 47 `git mv` only applies to in-tree moves — flag history-continuity risk). MIT→Apache-2.0 relicense requires explicit founder/ADR sign-off. No source-of-truth confirmation that `cloud/cloud-intelligence/anthropic-claude-adapter` is the agreed destination slug (task asserts it; needs registry registration).
