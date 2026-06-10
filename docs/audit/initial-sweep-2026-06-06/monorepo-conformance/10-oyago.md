---
title: Monorepo Conformance Audit — oyago (Go→Rust transpiler)
status: Audit-complete
date: 2026-06-06
auditor: workflow-subagent (read-only)
repo: /Users/jasonlee/Developer/oyago
target_home: oya/transpiler-go-to-rust/   (codename `oyago-*` FORBIDDEN per ADR-0017)
checklist: ./00-policy-checklist.md
verdict_legend: CONFORMS | NEEDS-RESHAPE | NEEDS-RENAME | VIOLATES | N/A
---

# Audit: oyago vs Monorepo Conformance Checklist

## Repo snapshot (evidence)

- Root: `Cargo.toml` (`resolver = "3"`, `license = "Apache-2.0 OR MIT"`, `rust-version = "1.96"`, edition 2024), `AGENTS.md`, `README.md` (~51 KB), `go.mod` (`module github.com/jasonlee/oyago`, go 1.26), `go.sum`, `Cargo.lock`, `.gitignore`, `.omc/`, `.omx/`, `docs/`, `fixtures/`, `go/`, `crates/`, `target/`.
- **Two stray executables checked into the worktree root:** `oyago-i64lit` (462 KB), `test_join_temp` (463 KB). Both are compiled binaries, NOT gitignored (`git check-ignore` → not ignored), would be committed.
- Workspace members (root `Cargo.toml`): `crates/oyago-cli`, `crates/oyago-core`, `crates/oyago-runtime`.
- Crate `[package].name`: `oyago-cli`, `oyago-core`, `oyago-runtime`. CLI bin name = `oyago`.
- All crates are `std` (no `#![no_std]` anywhere). No `[lib]` sections, no `doctest = false`, no `publish = false`, no `license = "..."` per member, no `[lints] workspace = true`.
- `crates/oyago-core/src/`: flat module set — `analyzer_runner, capability, codegen, error, fixture, ir, mapping, schema, stop_packet, target_corpus, trace, verifier` + `lib.rs`. Public type prefix `Oyago*` (e.g. `OyagoError`, generated runtime type `OyagoGoSlice`).
- Go analyzer: `go/cmd/oyago-analyzer/main.go`, `go/internal/analyzer/{analyze,schema}.go`.
- Docs: `docs/adr/0001-compiler-pipeline.md` (one ADR), plus design SSOT, capability-matrix, mapping-catalog, stop-packet-format, support-matrix, etc.
- **Git state: zero commits** (`git log` → "does not have any commits yet"; `git ls-files` → 0 tracked). Everything is untracked working tree on branch `main`.
- ABSENT: `deny.toml`, any `BUCK`/`.buckconfig`/`.buckroot`, `registry/`, `specs/`, `catalog/`, `libs/`, `oya/`, `cloud/`, CI/Jenkins config, `data_class`/`DataClass`.
- Codename residue (files, excl. target/.git): `oyago` = 177 files. `oyaoffice/oyapy/kuberos/foundry/oyatie` = 0. `talos` = 38 files but **all benign**: string-literal test data inside Go fixtures (`map[string]int{"kubernetes":1,"talos":2,"containerd":3}`) + README target-corpus prose ("Kubernetes, Talos, and containerd"); this is the OS project name as transpiler input data, NOT the forbidden `talos-*` codename surface. No brand violation from `talos`.

---

## A. Canonical homes & topology

1. **Service code only at `{oya,cloud}/<service>/crates/<crate>/`** — **VIOLATES/NEEDS-RESHAPE.** Code lives at top-level `crates/oyago-*`. No `oya/` or `cloud/` dir exists. Target home is `oya/transpiler-go-to-rust/crates/...`.
2. **Shared code only at `libs/<lib>/`** — **N/A (today) / NEEDS-RESHAPE.** No `libs/`. `oyago-runtime` (the emitted-Rust support runtime) and any cross-cutting pieces would need a home decision (likely service-local `crates/`, not `libs/`, since it is product-specific).
3. **`microservices/` FORBIDDEN** — **CONFORMS** (no `microservices/` dir).
4. **Per-service colocation** (PRD.md, README.md, PHASE-NN, IP-NNN, decisions/, contracts/, specs/, catalog/<crate>.yaml, runbooks/, threat-model.md, slos/, iac/, src/crates/, tests/, evidence/multispectrum/) — **VIOLATES.** None present. Has a README + scattered `docs/` but no per-service PRD.md, no `decisions/`, no `contracts/`, no `catalog/`, no `runbooks/`, no `threat-model.md`, no `slos/`, no `iac/`, no `evidence/multispectrum/`. Service-scoped ADR is mislocated at repo-central `docs/adr/0001-*` rather than `oya/transpiler-go-to-rust/decisions/`.
5. **Cross-cutting artifacts central only** — **NEEDS-RESHAPE.** `docs/adr/0001-compiler-pipeline.md` is a service-scoped ADR sitting in a central-looking `docs/adr/`; under policy, cross-cutting ADRs go to `docs/decisions/ADR-####-*.md` and service ADRs go to the service `decisions/`. Naming `0001-` also does not match `ADR-####-` convention.
6. **Aggregation indices GENERATED** — **N/A** (no indices exist yet). `docs/refactor-rewrite-registry.{json,md}` is hand-authored content, not a generated aggregation index of the policy kind.
7. **Packs only under `packs/`** — **CONFORMS** (no pack artifacts; no misplaced packs).

## B. Crate naming — BNF v4.1 + 13-layer enum

8. **`oya-` prefix mandatory** — **VIOLATES / NEEDS-RENAME.** Every crate is `oyago-*` (a FORBIDDEN codename prefix), not `oya-*`.
9. **BNF `oya-<microservice>(-<bc>)?-<layer>`** — **VIOLATES.** `oyago-core`, `oyago-cli`, `oyago-runtime` carry no microservice token and no canonical layer suffix (`core`/`runtime` are not in the enum; `cli` is, but the prefix is wrong).
10. **`[package].name == dir basename`** — **CONFORMS mechanically** (names match `crates/oyago-*` basenames) but the basenames themselves are non-conformant (item 8/9). After rename, both sides must move together.
11. **`[lib].name == snake_case(package.name)`** — **N/A / NEEDS-RESHAPE.** No `[lib]` sections; default lib name = `oyago_core` etc. Acceptable as default, but post-rename the kebab→snake mapping must hold (e.g. `oya-go-to-rust-codegen` → `oya_go_to_rust_codegen`).
12. **Layer suffix ∈ 13-value enum** (`kernel|domain|usecase|app|adapter|infrastructure|cli|rest|grpc|graphql|worker|sdk|api`) — **VIOLATES.** `-core` and `-runtime` are not enum layers. Needs decomposition into real layers (kernel/domain/app/cli/...).
13. **Adopted patterns** (`oya-check-*`, `*-adapter-<backend>` impl ≥1 kernel port) — **N/A.** No check crates, no adapter crates, no kernel ports. Would only apply after a hexagonal reshape.
14. **`tools/` crates explicit `-app` suffix** — **N/A** (no `tools/` dir). Note: the CLI bin is named `oyago` — must become `oya-go-...-cli` family with bin renamed off the codename.
15. **Microservice slot2 registered in `[workspace.metadata.oya.microservices.<name>]`** — **VIOLATES** (no `[workspace.metadata.oya]` block at all; `transpiler-go-to-rust` not registered with owner/rationale/adr_cite).

## C. Brand residue — FORBIDDEN list

16. **`oyatie-*` cargo prefix FORBIDDEN** — **CONFORMS** (0 `oyatie` occurrences).
17. **Codename residue FORBIDDEN** (`oyaoffice|oyago|oyapy|kuberos|foundry-*|oyatie-*|talos-*`) — **VIOLATES (severe).** `oyago` appears in **177 files**: crate names, CLI bin name, `Oyago*` public types/runtime types, Go module path (`github.com/jasonlee/oyago`), Go binary dir `go/cmd/oyago-analyzer/`, README title `# oyago`, AGENTS.md ("Project: oyago"), docs throughout, root binary filename `oyago-i64lit`. Product surface must be the **Oyatie** brand; the transpiler is a feature of the `transpiler-go-to-rust` service, not a standalone "oyago" product. (`talos` hits are benign input-data, see snapshot — not a violation.)
18. **No tautological rebrand residue** (`old→new` arrows, "retired terms" tables, "after rename") — **CONFORMS** (none observed; repo predates any rebrand so carries no rebrand-arrow scar tissue — it just needs first-time renaming).

## D. One-workspace invariants

19. **Exactly ONE `[workspace]` at root** — **CONFORMS** (no nested `[workspace]` in member Cargo.tomls). NOTE: this is a *standalone* root workspace; on migration into the source monorepo it must DISSOLVE into the single source root workspace (its 3 crates become `[workspace.members]` entries of the source root), so "conforms" only in isolation.
20. **`resolver = "2"`** — **VIOLATES.** Root sets `resolver = "3"`. Policy/source pins `"2"`. (On absorption, the source root resolver governs anyway.)
21. **One-version inheritance (`version.workspace`, `edition.workspace`, `rust-version.workspace`)** — **NEEDS-RESHAPE.** Members inherit `edition/license/rust-version/repository.workspace = true` (good pattern) BUT: (a) members set `version = "0.0.0"` literally instead of `version.workspace = true`; (b) `[workspace.package]` lacks `version`; (c) values differ from source (`edition 2024` ok, but source is `version 0.1.0` / `rust 1.95.0` vs here `rust 1.96`, no version).
22. **Member carries `publish = false` + `license = "Apache-2.0"` + `[lints] workspace = true`** — **VIOLATES.** None of the three present in any member. License is workspace-inherited as dual `Apache-2.0 OR MIT` (see item 35).
23. **`[lib] doctest = false`** — **VIOLATES** (no `[lib]` section / no `doctest = false`).
24. **Single `[workspace.dependencies]` seam + every entry has `registry/dependency-rationales.json` row** — **NEEDS-RESHAPE.** A `[workspace.dependencies]` seam exists (`serde`, `serde_json`, plus path entries `oyago-core`, `oyago-runtime`). BUT members reference `serde_json.workspace = true` etc. correctly; the path-dep entries must convert to source-monorepo workspace members, and NO `registry/dependency-rationales.json` exists, so all external deps are orphan-unjustified.

## E. Hexagonal clean architecture

25. **Inward-only dependency flow per 13-layer matrix** — **VIOLATES / NEEDS-RESHAPE.** No layer structure; `oyago-core` is a 12-module monolith mixing pure IR/mapping with `codegen`, `verifier`, `analyzer_runner` (process/IO), `fixture`, `target_corpus` (filesystem/git IO). No enforced import matrix.
26. **Port traits in `kernel`, impls in `adapter`** — **VIOLATES** (no kernel, no adapters, no port traits).
27. **`kernel` = pure, no I/O/async, no_std-capable** — **VIOLATES.** Pure types (`ir`, `schema`, `stop_packet`, `mapping` types) are entangled with I/O modules in one std crate; nothing is no_std-capable. A `kernel` must be carved out (IR + schema + stop-packet + capability-matrix types + port traits) free of `std::fs`/`std::process`.
28. **`api`/`sdk` depend on `kernel` only** — **N/A** (no api/sdk crates). The analyzer JSON schema (`ANALYZER_SCHEMA_ID`, versioned contract) is a natural `api`-layer surface to extract.
29. **Only `app` (composition root) has unrestricted inward deps; `app→app` FORBIDDEN** — **NEEDS-RESHAPE.** `oyago-cli` is the de-facto composition root but is not modeled as an `app` layer; today it depends straight on the `core` monolith.
30. **No direct cross-microservice imports (LEAN-A2)** — **N/A** (single service, no cross-service edges). Holds vacuously now; must be honored once inside the monorepo (transpiler must not reach into other services except via sanctioned seams).

## F. Data governance & quality gates

31. **`data_class` on kernel data fields with tenant/regulated data** — **N/A (likely).** No `DataClass` anywhere. A Go→Rust transpiler is a build-time dev tool processing source code, not tenant/PHI/PII runtime data, so `data_class` may legitimately not apply — but this needs an explicit "no regulated data" assertion in the service threat-model rather than silent absence. Verdict: NEEDS-RESHAPE (must document the determination).
32. **Statelessness (no module-level mutable state in presentation/application/worker)** — **NEEDS-RESHAPE / unverified.** No `oya-check-statelessness` run; not mechanically asserted. (No obvious `static mut` seen, but not gate-verified.)
33. **Shardability (`tenant_id` + RLS)** — **N/A** (no DB designs in a transpiler).
34. **Buildability bar (PRD ≥5 stories, IP ≥150 lines, ADR ≥3 alternatives/≥3 consequences + sources + roadmap, scorecard evidence)** — **VIOLATES.** No PRD.md, no IP-NNN files. Only `docs/adr/0001-compiler-pipeline.md` (single ADR) — not verified to meet the ≥3-alternatives/≥3-consequences/named-sources/roadmap structure; no scorecard. README is exhaustive prose but is not a PRD with measurable user stories.

## G. Supply chain & license policy

35. **Root `deny.toml` license allowlist** — **VIOLATES.** No `deny.toml` exists. Also the declared crate license `Apache-2.0 OR MIT` conflicts with the per-member requirement `license = "Apache-2.0"` (item 22) — must be reset to `Apache-2.0`.
36. **`[bans]` (deny openssl/openssl-sys, old-time; multiple-versions=warn; wildcards=warn)** — **VIOLATES** (no `deny.toml`). Dependency surface is currently tiny (`serde`, `serde_json`; Go side `golang.org/x/tools`,`x/mod`,`x/sync`) so risk is low, but the gate file is absent.
37. **`[sources]` crates.io only; unknown-registry/git=deny; empty allow-git** — **VIOLATES** (no `deny.toml`). Note `repository = "https://example.invalid/oyago"` is a placeholder that must be corrected to the source repo URL.
38. **`[advisories]` yanked=deny, empty ignore** — **VIOLATES** (no `deny.toml`).
39. **Vendor classification A/B/C (ADR-0211) + `registry/vendor-lockin-phaseout/index.json`** — **VIOLATES.** No classification, no `registry/`. `serde`/`serde_json` → Class A; `golang.org/x/tools` (the Go AST analyzer dependency) needs explicit classification since the analyzer is a load-bearing in-house component crossing the Go boundary.

## H. Buck2 build graph (Proposed, door:two-way)

40. **Per-crate `BUCK` (`rust_library`/`rust_binary`, crate_root, visibility, deps)** — **VIOLATES.** No `BUCK` files. (README *prose* claims emitted Rust gets a "Buck2 package graph" — that is generated OUTPUT for transpiled corpus, NOT this repo's own build files. The transpiler repo itself has zero Buck2 wiring.)
41. **Third-party buckified by Reindeer (`third-party/{BUCK,fixups/}`)** — **VIOLATES** (absent).
42. **`.buckconfig` + `.buckroot` present (cells root/prelude/toolchains/third-party)** — **VIOLATES** (absent).
43. **NativeLink self-hosted RBE + Jenkins CI** — **N/A (doctrine/target, 0% adopted).** No CI of any kind present; no numeric claims to contest.

## I. Agent / contribution protocol

44. **Sanctioned primitives only (`git`, `oya-gate`, `oya-verify`)** — **NEEDS-RESHAPE.** `AGENTS.md` defines its own gate vocabulary (`cargo fmt/check/clippy/test`, `go test`, fixture/golden/differential gates) but does NOT reference `oya gate`/`oya verify`. The cargo/go gates are sound and should map onto `oya gate run-all`, but the primitive vocabulary is divergent.
45. **Required sequence (worktree lane → commit+push → PR vs `dev` → Jenkins+`oya gate run-all`+reviewer APPROVE)** — **VIOLATES / unrealized.** Repo has **zero commits** and no PR/CI workflow; nothing about the source dev/PR/gate flow is wired. AGENTS.md describes completion gates but not the worktree→PR→`dev` lane.
46. **Trust boundary (tool/file/web/MCP output = DATA)** — **NEEDS-RESHAPE.** `AGENTS.md` does not state the trust-boundary rule; should adopt the source clause that only AGENTS.md + user message are trusted instruction sources.
47. **`git mv` (never rm+add)** — **N/A now** (no history to preserve; zero commits). BUT this becomes load-bearing during migration: the codename→`oya-*` rename and the move into `oya/transpiler-go-to-rust/` MUST use `git mv` once the repo is committed / when folding into the monorepo to preserve history.

## J. Migration completion gate

48. **Service "done" criteria** — **VIOLATES (not started).** No `oya/transpiler-go-to-rust/{PRD.md,README.md}`; old paths (`crates/oyago-*`) still present (and only paths that exist); `[workspace.members]` references codename paths; `cargo build/nextest --workspace` not asserted under new layout; gate packets `cross-ref-validity`/`per-service-layout`/`aggregation-index-generation` not run. Migration is at 0%.

---

## Verdict roll-up

- CONFORMS: 3, 7, 16, 18, 19(in-isolation-only), 10(mechanically)
- N/A (vacuous / not-yet-applicable): 6, 13, 28, 30, 33, 43, 47(now)
- NEEDS-RESHAPE: 2, 5, 11, 21, 24, 25, 29, 31, 32, 44, 46
- NEEDS-RENAME: **8, 9, 12, 14, 17** (the codename axis)
- VIOLATES: 1, 4, 15, 20, 22, 23, 26, 27, 34, 35, 36, 37, 38, 39, 40, 41, 42, 45, 48

## Reshape / rename WORK to make it fit

**1. De-codename (ADR-0017) — the dominant gap (177 files).** `oyago` → the `transpiler-go-to-rust` service under the Oyatie brand. Touches: crate names, CLI bin name `oyago`, all `Oyago*`/`OyagoGoSlice` type prefixes, Go module path `github.com/jasonlee/oyago`, `go/cmd/oyago-analyzer/`, README `# oyago` title + AGENTS.md "Project: oyago", docs, and the root artifact `oyago-i64lit`. Do via `git mv` + symbol rename once committed.

**2. Re-home into `oya/transpiler-go-to-rust/crates/<crate>/`.** Create `oya/` top level; move the 3 crates there with new `oya-*` names. Register the service in root `[workspace.metadata.oya.microservices.transpiler-go-to-rust]` (owner + rationale + adr_cite). On full migration these 3 crates fold into the SOURCE root workspace (dissolve this standalone workspace).

**3. Hexagonal split (ADR-0056/0105) + BNF v4.1 names.** Decompose the `oyago-core` 12-module monolith into layered crates with canonical suffixes, e.g.:
   - `oya-go-to-rust-kernel` — pure IR + analyzer schema types + stop-packet + capability-matrix types + port traits (no `std::fs`/`std::process`, no_std-capable target).
   - `oya-go-to-rust-api` — versioned analyzer JSON contract (`ANALYZER_SCHEMA_ID`), depends on kernel only.
   - `oya-go-to-rust-domain`/`-usecase` — deterministic mapping/lowering logic.
   - `oya-go-to-rust-codegen-adapter` (Rust emit) + `oya-go-to-rust-analyzer-adapter-go` (the `go/x/tools`-backed analyzer runner; backend = the Go process) impl kernel ports.
   - `oya-go-to-rust-cli` (was `oyago-cli`, bin renamed off codename) as the `app`/`cli` composition root.
   - `oya-go-to-rust-runtime` (emitted-Rust support) — decide layer (likely service-local infrastructure, not `libs/`).

**4. Per-service colocation scaffold (ADR-0131).** Add under `oya/transpiler-go-to-rust/`: `PRD.md` (≥5 measurable user stories), `README.md`, `IP-NNN-*.md` (≥150 lines), `decisions/` (MOVE `docs/adr/0001-compiler-pipeline.md` here, re-author to `ADR-####` with ≥3 alternatives + ≥3 consequences + named sources + roadmap), `contracts/` (the analyzer schema as a proto/openapi-style contract), `specs/`, `catalog/<crate>.yaml`, `runbooks/`, `threat-model.md` (explicitly assert NO regulated/tenant data → why `data_class` is N/A, item 31), `slos/`, `evidence/multispectrum/`.

**5. One-workspace + member hygiene (ADR-0092).** Set root `resolver = "2"`; add `version` to `[workspace.package]`; align `rust-version` to source (1.95.0) and `version` to 0.1.0; convert member `version = "0.0.0"` → `version.workspace = true`; add to EVERY member `publish = false`, `license = "Apache-2.0"` (drop dual `OR MIT`), `[lints] workspace = true`, and `[lib] doctest = false`. Fix `repository = "https://example.invalid/oyago"` placeholder.

**6. Supply chain (deny.toml + registry).** Add root `deny.toml` (license allowlist, openssl/old-time bans, crates.io-only sources, yanked=deny). Add `registry/dependency-rationales.json` rows for `serde`/`serde_json` and a `registry/vendor-lockin-phaseout/index.json` classification (serde* = Class A; `golang.org/x/tools` analyzer dependency = classify A/B/C with seam since it crosses the Go boundary).

**7. Buck2 wiring (ADR-0392/0408, Proposed/two-way — lower priority).** Add per-crate `BUCK`, `third-party/{BUCK,fixups/}` via Reindeer, `.buckconfig` + `.buckroot`. Keep Cargo as human SSOT. (Do not assert build/CI numbers until green.)

**8. Contribution protocol (AGENTS.md).** Adopt `oya gate`/`oya verify` primitives, the worktree-lane→PR-vs-`dev`→Jenkins+`oya gate run-all`+reviewer-APPROVE sequence, the DATA trust-boundary clause, and `git mv` for all moves. Make the first commit (repo currently has none).

**9. Housekeeping.** Remove/gitignore the two root build artifacts `oyago-i64lit` and `test_join_temp` (compiled binaries, ~462–463 KB each, currently NOT ignored and would be committed). Reconcile `.omx/` historical-plan dir vs `.omc/` (AGENTS.md already demotes `.omx/plans/` to background-only).

## Blockers / notes

- **No missing source** — the repo is fully present and readable; the audit is complete on evidence.
- **Biggest blocker to migration: the repo has ZERO git commits.** Item 47 (`git mv` history preservation) and item 45 (PR/lane flow) cannot be honored until an initial commit exists; the rename should ideally happen with history, so commit-then-`git mv` ordering matters.
- **Severity ranking of gaps:** (1) codename `oyago` saturation (177 files, items 8/9/12/14/17) — the rename touches everything else; (2) zero monorepo topology — no `oya/` home, no per-service colocation, no `[workspace.metadata.oya]` (items 1/4/15/48); (3) no hexagonal layering — `core` monolith mixes pure + I/O (items 25–27); (4) no governance gates — no `deny.toml`/`registry`/`BUCK`/CI (items 34–43).
- **False-positive cleared:** `talos` (38 files) is benign transpiler input data (OS name in fixture maps + README corpus prose), NOT the forbidden `talos-*` codename surface.
