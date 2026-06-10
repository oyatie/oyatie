---
title: Monorepo Conformance Register (synthesis of 6 repo audits vs source policy)
status: Synthesis-complete
date: 2026-06-06
auditor: workflow-subagent (read-only synthesizer)
inputs:
  - ./00-policy-checklist.md            # 48-item authoritative checklist
  - ./10-linux-stack.md                 # linux/stack (4 workspaces + Go ref)
  - ./10-oyago.md                       # Go→Rust transpiler
  - ./10-oyapy.md                       # Python→Rust transpiler
  - ./10-office.md                      # OyaOffice scaffold
  - ./10-claude.md                      # claude-agent-sdk
  - ./10-codex.md                       # openai-codex-sdk
wip_plan:
  - /Users/jasonlee/Developer/source/.omc/plans/monorepo-consolidation-migration.md  # HOW (gates)
  - ../UNIFIED-EXECUTION-PLAN.md                                                      # WHAT+HOW merge
note: READ-ONLY. Every cell traces to a cited audit verdict. No source repos modified.
---

# Monorepo Conformance Register

Synthesizes the 6 per-repo audits against the 48-item source policy, then tests whether the
WIP migration plan's conformance gates (`monorepo-consolidation-migration.md` §9 + STEP 1–14)
COVER every gap, and rules whether consolidation is achievable as-planned or needs ralplan revision.

Verdict legend: **C** CONFORMS · **RS** NEEDS-RESHAPE · **RN** NEEDS-RENAME · **V** VIOLATES · **B** BLOCKER · **N** N/A · **P** PARTIAL.
Where an audit gave a compound verdict (e.g. "VIOLATES/NEEDS-RENAME") the dominant/most-severe is shown.

---

## 1. MATRIX — repo × policy-item

Columns: linux/stack (LX), oyago (GO), oyapy (PY), office (OF), claude (CL), codex (CX).
Rows grouped by checklist section A–J.

### A. Canonical homes & topology
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 1 | code only at `{oya,cloud}/<svc>/crates/` | V | V | V | RS | V | RS |
| 2 | shared only at `libs/` | V | N | N | RS | N | N |
| 3 | `microservices/` forbidden | C | C | C | C | C | C |
| 4 | per-service colocation set | V | V | RS | V | V | V |
| 5 | cross-cutting central only | RS | RS | RS | RS | N | N |
| 6 | aggregation indices generated | V | N | RS | RS | N | N |
| 7 | packs only under `packs/` | N | C | C | C | C | C |

### B. Crate naming — BNF v4.1 + 13-layer enum
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 8 | `oya-` prefix mandatory | V | RN | RN | RN | RN | RN |
| 9 | BNF `oya-<ms>(-bc)?-<layer>` | V | V | V | RN | RN | RN |
| 10 | `package.name`==dir basename | RN | C | C | C | RS | RS |
| 11 | `lib.name`==snake(name) | RS | RS | C | C | RS | RN |
| 12 | layer ∈ 13-enum | V | V | V | V | RS | RN |
| 13 | check-family / `*-adapter-<backend>` | V | N | N | RS | RS | V |
| 14 | `tools/` crates explicit `-app` | RN | N | N | C | N | N |
| 15 | microservice slot2 registered | V | V | V | RS | V | V |

### C. Brand residue
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 16 | `oyatie-*` prefix forbidden | C | C | C | C | C | C |
| 17 | codename residue forbidden | V(severe) | V(severe) | V(severe) | V | C | C |
| 18 | no tautological rebrand residue | RS | C | C | C | C | C |

### D. One-workspace invariants
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 19 | exactly ONE `[workspace]` (no nested) | V(hard) | C* | C* | C | RS | RS |
| 20 | `resolver="2"` | RS(k8s=3) | V(3) | V(3) | V(3) | RS | N(inherit) |
| 21 | one-version inheritance | RS | RS | RS | RS | V | V |
| 22 | publish=false + Apache-2.0 + lints.workspace | V | V | V | V | V | RS |
| 23 | `[lib] doctest=false` | V | V | V | V | V | V |
| 24 | `[workspace.dependencies]` seam + rationale rows | RS | RS | V | RS | V | RS |

\*GO/PY conform only as standalone roots; both dissolve into the source root on merge.

### E. Hexagonal clean architecture
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 25 | inward-only dep flow | RS | V | RS | RS | RS | V |
| 26 | ports in kernel, impls in adapter | V | V | RS | V | P | V |
| 27 | kernel pure / no_std-capable | P | V | RS | RS | RS | V |
| 28 | api/sdk depend on kernel only | V | N | N | RS | RS | RS |
| 29 | only `app` unrestricted; no app→app | V | RS | RS | RS | RS | N |
| 30 | no cross-microservice imports | RS | N | C | RS | C | N |

### F. Data governance & quality gates
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 31 | `data_class` on regulated kernel fields | V | N? | N? | C(partial) | V | RS |
| 32 | statelessness | RS | RS | ? | C | ? | C |
| 33 | shardability (tenant_id+RLS) | N | N | N | RS | N | N |
| 34 | buildability bar (PRD/IP/ADR) | V | V | RS | RS | V | V |

### G. Supply chain & license
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 35 | root `deny.toml` allowlist | V | V | V | RS(misplaced) | V | N(inherit) |
| 36 | `[bans]` openssl/old-time | V | V | V | RS | C(de-facto) | C(incidental) |
| 37 | `[sources]` crates.io only | V+risk | V | V | C | ?verify | ?verify |
| 38 | `[advisories]` yanked=deny | V | V | V | C | N | N |
| 39 | vendor A/B/C + registry | V | V | RS | RS | V | RS |

### H. Buck2 build graph (Proposed, door:two-way)
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 40 | per-crate BUCK | P(867 files, wrong cells) | V | V | RS | V | V |
| 41 | third-party via Reindeer | P | V | V | RS | V | N(inherit) |
| 42 | `.buckconfig`+`.buckroot` | P(2 roots) | V | V | C | V | V |
| 43 | NativeLink RBE / Jenkins CI | N(0%) | N | N | RS | N | N |

### I. Agent / contribution protocol
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 44 | sanctioned primitives only | N | RS | RS | RS | N | N |
| 45 | worktree→PR-vs-dev→gate | N | V | RS | RS | N | **B** |
| 46 | trust boundary | N | RS | RS | RS | N | N |
| 47 | `git mv` (preserve history) | adv | N(0 commits) | proc | RS | migrate-risk | **B** |

### J. Migration completion gate
| # | item | LX | GO | PY | OF | CL | CX |
|---|------|----|----|----|----|----|----|
| 48 | service "done" gate | V(0/N) | V | V | V | not-met | not-met |

---

## 2. Per-repo conformance debt (what each migration lane MUST clear)

Lane numbering follows the WIP plan §6 (L1 office … L11 framekernel).

### linux/stack — splits into L6 k8s(MERGE) · L7 containerd(CREATE) · L9 node-os(CREATE) · L11 framekernel(no_std, LAST); docs→L10
The single heaviest surface (191 Rust crates across 4 workspaces + 1 vendored Go tree).
- **Topology:** no `oya/`/`cloud/`/`libs/`; relocate kernel→`cloud/cloud-kernel`, OS→`cloud/cloud-node-os`, k8s(95)→`cloud/managed-k8s-*`(+`cloud/cloud-k8s`), ctrd(44)→`cloud/cloud-container-runtime`. Author full per-service colocation.
- **One-workspace (HARD):** collapse 16 `[workspace]` decls → 1 root; the 11 nested freestanding kernel user-binary workspaces + `kubernetes/third-party/rust` resist merge (custom link addrs / build-std) → need sanctioned exclude or out-of-tree home.
- **Naming:** all 191 crates `oya-<ms>(-bc)?-<layer>`; de-snake `ctrd_*`/`meta_v1`/`util_json`; add layer suffixes.
- **Brand (severe):** purge `talos-*` (45 crates + ~388 .rs) AND `Kuberos` prior-codename leaked into PRODUCT source (`talos-secrets` `KUBEROS_*` env vars, `difftest`, 51× in `MIGRATION_REPORT.md`).
- **Hexagonal:** no kernel/adapter/app split; disambiguate the framekernel's literal `kernel` crate from the policy "ports-kernel" layer.
- **Governance:** no `deny.toml`, no `data_class`, no rationales/vendor registries; resolver k8s 3→2; license MIT-OR-Apache→Apache-2.0; edition kernel 2021→2024.
- **Buck2:** best-aligned (867 BUCK) but two `.buckroot`s + custom `generate_buck_files.py`; rewire to `//{cloud,libs}` + `third-party//`, reconcile vendored `third-party/` with crates.io-only `[sources]`.
- **Exclude:** `talos-reference/` (upstream Go) must be excluded from the Rust workspace + marked vendored so brand scan does not flag it.

### oyago — L2 (`oya/transpiler-go-to-rust`, CREATE)
- **De-codename (dominant, 177 files):** `oyago-*` crates, `oyago` CLI bin, `Oyago*`/`OyagoGoSlice` types, Go module `github.com/jasonlee/oyago`, `go/cmd/oyago-analyzer/`, README/AGENTS/docs, root artifact `oyago-i64lit`.
- **Re-home + register** microservice slot2; **hexagonal split** of the `oyago-core` 12-module monolith (mixes pure IR/schema with codegen/analyzer_runner/target_corpus I/O) → `-kernel`/`-api`/`-domain`/`-codegen-adapter`/`-analyzer-adapter-go`/`-cli`.
- **Manifest:** resolver 3→2; `version="0.0.0"`→`version.workspace`; add `version` to `[workspace.package]`; rust 1.96→1.95.0; drop dual `Apache-2.0 OR MIT`→Apache-2.0; add publish=false/lints/doctest=false; fix `repository="https://example.invalid/oyago"`.
- **Governance:** add `deny.toml` + rationales (serde* = A; `golang.org/x/tools` analyzer = classify w/ seam) + per-service colocation + buildability ADR upgrade.
- **Housekeeping:** remove/gitignore stray root binaries `oyago-i64lit`+`test_join_temp` (NOT ignored, would be committed).

### oyapy — L3 (`oya/transpiler-python-to-rust`, CREATE)
- Mirror of oyago: de-codename `oyapy` (181 .rs lines + all docs/fixtures + `python/oyapy_analyzer.py`) + sibling `oyago` references; re-home; hexagonal split of `oyapy-core` (`-kernel`/`-domain`/`-app`) + `oyapy-runtime`→`-infrastructure`.
- Manifest: resolver 3→2; members hardcode `version="0.1.0"`→`version.workspace` (workspace has no version); rust 1.96→1.95.0; add publish=false/license/doctest=false/`[workspace.dependencies]` seam; no README anywhere.
- **OPEN:** `transpiler-python-to-rust` = 4 microservice tokens > BNF 1..3 budget → registry-approved short name needed; `python/oyapy_analyzer.py` (105 KB non-Rust) needs sanctioned home + classification.

### office — L1 (`oya/office` + `cloud/<svc>`, CREATE)
Most policy-ready non-stack repo: clean hexagonal scaffold + Buck2 wiring; reshape is mechanical, not salvage.
- **Blanket rename** `oyaoffice-*`→`oya-office-*` across 19 crates + lib/bin/path-deps/Cargo.lock/all BUCK/`workspace_metadata.bzl`/metric strings/`product`/`repository` (largest single violation surface, 65+ doc hits).
- **Topology:** flat `crates/`+`apps/`→`oya/<svc>/crates/` + platform→`cloud/` + shared→`libs/`; per-service colocation from currently-central docs.
- **Layer suffixes:** `*-port`×2→fold into kernel; `*-api-contracts`×2→`*-api` (drop illegal `oyaoffice-sheet-domain` dep — api→kernel only); `*-gateway`→`*-rest/-worker`; `oyaoffice-web`→`*-web-app`; add `*-app` roots + `*-adapter-<backend>`.
- **Manifest:** resolver 3→2; member `version="0.1.0"`→`version.workspace`; license `Proprietary`→Apache-2.0; add `doctest=false`; rust 1.96 vs 1.95.0.
- **Kernel purity:** make kernels `#![no_std]`+alloc (drop bare `std::error::Error`); add adapter impls.
- **Supply chain:** move `supply-chain/deny.toml`→root; `[bans]` add openssl/openssl-sys/old-time + wildcards deny→warn; add vendor/rationales registries.
- **Flags:** no AGENTS.md; second `third-party/.buckroot` may create spurious buck root — verify.

### claude — L4 (`cloud/cloud-intelligence/.../oya-cloud-intelligence-anthropic-claude-adapter`, CREATE)
- **Bright spots:** ZERO brand residue; no illegal nested `[workspace]`; rustls-only (no openssl); `ClaudeProcessSpawner`/`SpawnedClaudeProcess`/`SessionStore` trait seams ready for kernel/adapter split.
- **Rename+split:** `claude-agent-sdk`→`oya-anthropic-claude-kernel` (pure types+ports, no_std-capable, data_class on prompt/session fields) / `-adapter-subprocess`(-cli) (transport/bridge/runtime) / `-sdk` (`public_layers=["sdk"]`).
- **Manifest:** local `version 0.1.2`/`edition 2024`/`rust 1.85`/`license MIT`→workspace inheritance + Apache-2.0; add publish=false/lints/doctest=false; deps inline→`.workspace`+rationales; classify reqwest/tungstenite = Class B behind existing network seam.
- **Relicense MIT→Apache-2.0 needs founder/ADR sign-off.**

### codex — L5 (MERGE into existing `cloud/cloud-intelligence/crates/oya-cloud-intelligence-codex-adapter`)
- **Reclassify layer `sdk`→`adapter`** (shells out to Codex CLI via `std::process` in `exec.rs` — outbound-I/O driver, textbook Class-B vendor-lockin).
- **Rename+split:** `openai-codex-sdk`→`oya-codex-kernel` (pure protocol/event/item types from schema/items/events/input/options/protocol_schema + transport port + data_class) / `oya-codex-adapter` (std::process+tokio CLI driver impls port).
- **Manifest:** edition 2021→2024, rust 1.74→1.95.0 (real migration delta); add publish=false/lints/doctest=false; drop crates.io publish metadata + local Cargo.lock; deps→`.workspace`+rationales; Class-B vendor registry entry.
- **MERGE-surface diff** vs the existing on-dev adapter crate before landing.

---

## 3. Does the WIP plan's conformance gating COVER the gaps?

The WIP plan (`monorepo-consolidation-migration.md`) carries conformance enforcement in two places:
**STEP 1–14** (per-lane loop, §7/§8) and the **§9 Verification & Test Plan**. Mapping its gates to the 48 items:

| Plan gate (verbatim) | Checklist items COVERED |
|---|---|
| STEP 3 `package.name==basename` + `oya-*` prefix | 8, 10 |
| STEP 3 brand-residue scan FORBID `foundry-*/oyatie-*/oyago/oyapy/oyaoffice/kuberos` | 16, 17 |
| STEP 3 codename→`oya-*` rename pass | 8, 17 |
| STEP 2 + §6 allowlist-copy first-party + per-tree deny-globs (strip vendored/`talos-reference`/`_upstream*`) | 1(partial), 41(exclusion) |
| §6 canonical homes `{oya,cloud}/<svc>/crates`+`libs/` with tools/ standing exception | 1, 2, 14(exception) |
| STEP 5 add to 723-member root `Cargo.toml`, one-version, **no nested `[workspace]`** | 19, 20, 21 |
| STEP 6 `reindeer buckify` + per-crate BUCK → Cargo+Buck2 dual build | 40, 41, 42 |
| STEP 7 cargo deny + clippy + nextest + **`data_class` on every new kernel-struct field** | 23(via clippy/build), 31, 35, 36, 38 |
| STEP 8 whole-graph buck2 `//:...-check` matrix + affected-gate | 40, 48(build=0) |
| STEP 9 multispectrum evidence + 5-H2 PR body + DOC-CATALOG/CHANGELOG | 6(partial), 34(partial) |
| Pre-lane 0.5 ratify codename canonical names + k8s/ctrd split + db-engine confirm | 9(names), 15(implied), L8 gate |
| Pre-lane 0.6 no_std excluded-state inertness (incl. exclude-key edit) | LX framekernel workspace-exclusion |
| Pre-lane 0.7 governance-file bootstrap | 34(scaffold), 48(packets) |
| §9 one-root-workspace/no-nested-`[workspace]` + signed commits + linear history | 19, 45, 47 |

### COVERED well
Brand/codename (16–18), `oya-` prefix + basename (8, 10), canonical homes + tools/ exception (1, 2, 14), one-root-workspace collapse + no-nested (19), Buck2 dual-build + reindeer + per-crate BUCK (40–42), build/test green + whole-graph gate (48 build/test legs), `data_class` on new kernel fields (31), allowlist/deny-glob vendored stripping (1, 41), the no_std framekernel workspace-exclusion (the LX hard blocker, handled empirically in 0.6), db-engine absence (L8 conditional/droppable), authority-drift (G0).

### GAPS the WIP plan does NOT explicitly gate (ralplan MUST add)

1. **BNF layer-suffix enum (items 9, 12) is not a gate.** STEP 3 enforces `oya-*` prefix + basename but NOT "last token ∈ closed 13-value enum." The audits show every stack crate (`-core`, `-runtime`, `meta_v1`), office (`*-port`, `*-api-contracts`, `*-gateway`, `*-web`), oyago/oyapy (`-core`/`-runtime`) carry NON-enum suffixes. A name can pass the plan's scan and still violate the grammar. **Add an explicit layer-suffix-enum check.**

2. **Microservice slot2 registration (item 15) is implicit, not gated.** 0.5 "ratifies codename canonical names" but no STEP asserts `[workspace.metadata.oya.microservices.<name>]` exists with owner+rationale+adr_cite for each landed service. office even has it under the WRONG key (`[workspace.metadata.oyaoffice...]`) missing `adr_cite`. **Add a slot2-registration gate.**

3. **Hexagonal layering / inward-import-matrix (items 25–30) is not gated.** The plan renames and re-homes but has NO `oya-check`-style import-matrix enforcement, NO "ports-in-kernel/impls-in-adapter" check (26), NO "api/sdk→kernel-only" check (28 — office's `*-api-contracts` illegally depends on `*-sheet-domain`), NO "kernel pure / no_std-capable" check (27), NO `app→app` ban (29), NO LEAN-A2 cross-microservice-import check (30). The hexagonal SPLITS (oyago/oyapy/claude/codex monoliths; stack's missing kernel/adapter split) are the LARGEST units of work and are real engineering, but the plan treats them as in-lane work with no acceptance gate. **Add layer-matrix + kernel-purity + api-deps + app→app + LEAN-A2 gates.**

4. **`[lib] doctest=false` (item 23) not gated.** Universally VIOLATED across all 6 repos; clippy/nextest do not enforce it. **Add a manifest-lint gate.**

5. **`publish=false` + `license="Apache-2.0"` + `[lints] workspace=true` per member (item 22) not gated.** Universally violated/partial; the MIT→Apache-2.0 (claude) relicense needs an ADR. STEP 5 adds crates to root but does not assert these three member fields. **Add a member-manifest-hygiene gate** (and route the relicense decision).

6. **`[workspace.dependencies]` seam + `registry/dependency-rationales.json` no-orphans (item 24) not gated.** STEP 7 runs `cargo deny` but ADR-0092 "every workspace dep has a rationale row" is unchecked. **Add a dependency-rationale no-orphan gate.**

7. **Vendor A/B/C classification + `vendor-lockin-phaseout/index.json` (item 39) not gated.** codex (Class-B CLI bridge), claude (reqwest/tungstenite Class B), oyago (`golang.org/x/tools`) all need registered seams. The plan never asserts the registry row exists. **Add a vendor-classification gate** (esp. load-bearing for the two Class-B adapter lanes).

8. **Per-service colocation completeness (item 4) under-gated.** 0.7 bootstraps governance FILES the gate reads (evidence dir, PR template) but does NOT assert the full ADR-0131 set per service (PRD≥5 stories, IP≥150 lines, decisions/, contracts/, catalog/<crate>.yaml, runbooks/, threat-model.md, slos/, iac/, evidence/). Buildability bar (item 34) is partially carried by the 5-H2 PR body but not the ADR-shape (≥3 alternatives/≥3 consequences/named sources/roadmap). **Add a per-service-layout + buildability-bar gate** (matches the audits' `per-service-layout` / `aggregation-index-generation` packets named in item 48).

9. **`deny.toml` `[bans]`/`[sources]`/`[advisories]` content (items 36–38) only inherited, not per-lane verified.** STEP 7 runs `cargo deny check` (good, covers it transitively IF root deny.toml is correct) but office's deny.toml is MISPLACED (`supply-chain/`), has `wildcards=deny` (policy=warn), and is missing the openssl/old-time bans — none of which the plan's inheritance assumption catches. **Confirm root deny.toml content during 0.7; do not assume inheritance.**

10. **Statelessness (32) + shardability (33).** Named in policy (`oya-check-statelessness`/`-shardability`) but absent from the plan. Low urgency at scaffold stage but a future gate gap.

11. **Tautological rebrand-residue (item 18) — narrow but real.** The plan's brand scan forbids the codenames themselves but the audits flag that linux/stack `MIGRATION_REPORT.md`/`PORT_REPORT.md` carry `X→Y` migration arrows that would trip `oya-check-brand-residue`'s RebrandArrow/RetiredTermsTable patterns. The plan retires the pilot scaffold (L10) but does not explicitly scrub arrow/retired-terms residue in migrated docs. **Extend the brand scan to the RebrandArrow/RetiredTermsTable/RenamePhrase patterns.**

---

## 4. Hard BLOCKERS

| # | Blocker | Repos | Disposition in WIP plan |
|---|---------|-------|------------------------|
| B1 | **db-engine source NOT FOUND** | (cloud-data L8) | **COVERED** — 0.5 confirms; L8 CONDITIONAL/DROPPED if absent (G4). |
| B2 | **no_std framekernel cannot join the one root workspace** (custom link addrs / build-std / nightly-2026-02-28) | linux/stack `kernel` + 11 nested user-bin workspaces | **COVERED** — workspace-EXCLUDED, lands LAST (L11); 0.6 proves excluded-state-incl-exclude-edit inertness empirically. |
| B3 | **Nested-workspace conflict** — 16 `[workspace]` in /stack (11 nested kernel user-bins + `kubernetes/third-party/rust`) | linux/stack | **PARTIAL** — plan asserts no-nested-`[workspace]` at root, but the 11 freestanding kernel user-binary workspaces need an explicit disposition (exclude vs out-of-tree home). 0.6 covers the framekernel exclude; the user-bin sub-workspaces are NOT separately addressed. **ralplan must name their disposition.** |
| B4 | **Vendored-tree stripping** — `talos-reference/` (Go), `kubernetes/third-party/` (121 MB), `_upstream*`, reindeer caches | linux/stack | **COVERED** — per-tree deny-globs (§6) strip vendored/upstream/`talos-reference` at the boundary; only first-party moves. |
| B5 | **Zero git history** — no commits, untracked tree | oyago, codex | **PARTIAL** — plan mandates `git mv` + PR-vs-dev (items 45/47) but those are *unsatisfiable from sources with no lineage*. The codex audit flags this as a HARD blocker (B1 there). Migration must be commit-then-`git mv` into the destination (fresh attributed import), NOT a cross-repo history-preserving PR. **ralplan must state the no-history import procedure + founder sign-off for importing un-versioned third-party SDK source.** |
| B6 | **Microservice name > BNF token budget** — `transpiler-python-to-rust` = 4 tokens (limit 1..3) | oyapy (and likely oyago) | **NOT COVERED** — 0.5 "ratifies canonical names" but does not flag the token-budget overflow. **Founder/registry DECISION NEEDED** before crate renames finalize. |
| B7 | **MIT→Apache-2.0 relicense needs founder/ADR** | claude (also stack kernel MIT-OR-Apache) | **NOT COVERED** — no ADR/decision routed. **ralplan/decision-record must add a relicense ADR.** |
| B8 | **Non-Rust tools inside Rust service trees** — `python/oyapy_analyzer.py` (105 KB), `go/` analyzer (oyago) | oyapy, oyago | **NOT COVERED** — no home/classification policy for a Python/Go tool living inside an `oya/` Rust service. **DECISION NEEDED.** |
| B9 | **Authority-flip mid-campaign** (`github-lane-unlocker-required`→`oya-ci-required`+signing, ADR-0513) | source-wide | **COVERED** — 0.4 snapshot + loop STEP 0 re-diff + G0 HALT + signing pre-provisioned + 0.5 characterizes `oya-ci-required`. |
| B10 | **`cloud/cloud-k8s` sixth merge surface** + 139-crate k8s/containerd entanglement (95 k8s / 44 ctrd) | linux/stack | **COVERED** — 0.5 crate-level split manifest + merge-surface diffs (G4). |

---

## 5. VERDICT

**Consolidation is NOT yet achievable purely by the existing WIP plan as written — it needs a bounded ralplan revision before consolidation.** The plan is structurally sound and correctly owns the *hardest* mechanical/process risks (authority-drift B9, no_std exclusion B2, vendored stripping B4, db-engine B1, k8s entanglement B10, one-workspace collapse, brand/codename rename, Buck2 whole-graph gate). Those are the right top risks and they are well-mitigated.

But the plan's per-lane gates are **build-and-brand-centric**, not **architecture-and-governance-centric**. It would let a lane merge that:
- carries a non-enum layer suffix (9, 12),
- is an unregistered microservice (15),
- has no kernel/adapter hexagonal split or violates the import matrix (25–30) — the SINGLE LARGEST work item across oyago/oyapy/claude/codex and all of /stack,
- omits `doctest=false`/`publish=false`/Apache-2.0/lints (22, 23),
- has orphan workspace deps (24) and unclassified Class-B vendors (39),
- lacks the full ADR-0131 colocation + buildability artifacts (4, 34).

Plus four founder/registry DECISIONS are unrouted: **B5** (no-history import for oyago/codex), **B6** (microservice token-budget overflow), **B7** (MIT→Apache relicense), **B8** (non-Rust tool homing) — and one structural disposition is unnamed: **B3** (the 11 nested kernel user-bin workspaces).

### Required ralplan revision (bounded — add gates + route 5 decisions; do NOT re-architect)
1. **Add the missing conformance gates** to STEP-level acceptance (or as `oya-check-*` checks the Buck2 graph runs): layer-suffix-enum (9/12), microservice-slot2-registration (15), hexagonal layer-import-matrix + ports-in-kernel + api→kernel-only + app→app-ban + LEAN-A2 (25–30), member-manifest-hygiene incl. `doctest=false`/`publish=false`/Apache-2.0/lints (22/23), dependency-rationale no-orphan (24), vendor A/B/C registry (39), per-service-layout + buildability-bar (4/34), rebrand-arrow scan extension (18). These map exactly to the `cross-ref-validity`/`per-service-layout`/`aggregation-index-generation` gate packets the policy's item 48 already names — so the policy assumes them; the plan just hasn't wired them.
2. **Route the 5 unblocked decisions** through the decision-record: B5 no-history import procedure, B6 microservice short-name, B7 relicense ADR, B8 non-Rust-tool policy, B3 nested-user-bin-workspace disposition.
3. Keep everything else (the serial loop, pre-lanes 0.4–0.7, the lane sequence, MERGE-not-duplicate, the authority-drift machinery) unchanged.

**Net:** the assumed-merged monorepo's conformance is achievable, but only after the ralplan adds the architecture+governance gate layer and resolves the 5 open decisions. Without that, lanes would merge green-on-the-live-Buck2-gate while still VIOLATING ~12 policy items per repo — exactly the kind of silent conformance debt the 48-item checklist exists to prevent.

*End CONFORMANCE-REGISTER. READ-ONLY synthesis; all cells trace to 10-*.md audit verdicts + the cited WIP plan.*
