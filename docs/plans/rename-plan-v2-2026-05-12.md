---
doc_class: HowTo
shape: ~
length_cap: 900
authority_tier: 3
status: Superseded
superseded_by: docs/plans/rename-plan-v3-2026-05-12.md
date: 2026-05-12
supersedes: docs/plans/rename-plan-2026-05-12.md
purpose: |
  Execution plan v2 for the 140-crate convention cutover. Locks in **Policy B**
  (collapse foundry-fitness under a `fitness` umbrella) at user direction
  (2026-05-12, ICM `01KRFMEVN49BB6J0QWKNGATC1K`), enumerates the full 39-rename
  cohort, schedules an immediate `[package.metadata.oya]` cutover for all 140
  workspace members, and recommends a sharded-by-context cutover order with
  per-shard acceptance gates. Plan-only; no rename is performed by this
  document. Consensus review (Architect + Codex critic) MUST sign off before
  shard-1 ships.
canonical_authority: docs/CONSTITUTION.md
companion_docs:
  - docs/standards/crate-naming-convention.md
  - docs/standards/clean-architecture.md
  - docs/audits/convention-audit-2026-05-12.md
  - docs/plans/rename-plan-2026-05-12.md
  - .omc/fitness-lanes/architecture-conventions.md
related_adrs:
  - ADR-0015
  - ADR-0017
  - ADR-0054
---

> **STATUS: SUPERSEDED — 2026-05-12.** This plan was superseded by
> [`rename-plan-v3-2026-05-12.md`](rename-plan-v3-2026-05-12.md) following the
> iter-1 consensus review (Architect verdict: SOUND-WITH-CONDITIONS with 9
> enumerated conditions; Codex Critic verdict: ITERATE with 10 required
> plan edits). Both reviewers' #1 revision was "switch to Hybrid C" — v3
> adopts Hybrid C (Shard 0 pure-tooling precursor + atomic Shard 1
> 37-rename) and rebuilds every section reviewers flagged. **Do not
> execute against this v2 document.** Read v3 for the actionable plan;
> v2 remains in tree as historical record and consensus-iteration anchor.

# Rename Plan v2 — 2026-05-12 (Policy B + immediate metadata cutover) — SUPERSEDED

> **Supersedes** [`rename-plan-2026-05-12.md`](rename-plan-2026-05-12.md).
> The v1 plan recommended Policy A (registry-admit 28 compounds, defer
> AMBER-metadata to Q3-2026, 9 renames). User adjudication on 2026-05-12
> chose **Policy B**: collapse the foundry-fitness kernel family under a
> single `fitness` feature umbrella, and ship the per-crate
> `[package.metadata.oya]` block for **all 140 workspace members** in the
> same execution window. This document is the resulting execution plan.
> ICM provenance: `01KRFMEVN49BB6J0QWKNGATC1K`.

## §1 — Scope summary

| Item | Count | Notes |
|---|---:|---|
| Crate renames | **39** | 26 foundry-fitness collapses (Policy B umbrella) + 4 LONG-FEATURE foundry-fitness renames + 1 NEW-COMPOUND non-fitness (`oya-platform-data-boundary-kernel`) + 1 ROLE-AS-CAP (`oya-foundry-api-semver-kernel`) + 1 TOOSHORT singleton (`oya-foundation-app`) + 1 NO-ROLE (`oya-tooling-agent-read`) + 2 named decisions (`oya-foundry-api → oya-foundry-meta-api`, `oya-tooling-cli-dev-runtime → oya-tooling-dev-runtime`) + 3 LONG-FEATURE kernels collapsed to fitness umbrella (data-class, raci-team-coverage, readme-doc-coverage, release-evidence-pack, vendor-contract-recency → re-counted within the fitness collapse, see §2). Final cohort = **39** rows. |
| Cargo.toml metadata adds | **140** | All workspace members get `[package.metadata.oya] { role, context, capability }`, derived from the **NEW** crate name (post-rename). Atomic with shard cutover. |
| Dependency-edge rewrites | **~46** | Audited: 44 `path = "../oya-foundry-<noun>"` edges in `oya-tooling-cli-dev-runtime/Cargo.toml` get rewritten to the new fitness-umbrella names; plus 1 (`oya-foundry-api → oya-foundation-app`) edge; plus 1 (`oya-foundry-claim-ceiling-kernel → oya-foundry-catalog-kernel`). See §2 per-row "post-rename action" column for exact patterns. |
| CI workflow updates | **3** | `.github/workflows/release-evidence-pack.yml` + `.github/workflows/supply-chain.yml` (×2 invocations) reference `cargo run -p oya-tooling-cli-dev-runtime …`; all must flip to `oya-tooling-dev-runtime`. **No other GitHub Actions reference the renamed crates by path.** Verified via `grep -rn` over `.github/`. |
| Scripts updates | **2 files, ~30 lines** | `scripts/check.sh` (29 invocations of `oya-tooling-cli-dev-runtime`) and `scripts/hooks/pre-push-repoctl.sh` (1). `scripts/check-architecture-boundaries.sh` references `oya-foundation-app` and `oya-foundry-api` as crate-name strings (3 sites). |
| New fitness lane | **1** | `oya-foundry-fitness-architecture-conventions-kernel` (per `.omc/fitness-lanes/architecture-conventions.md`). Becomes a capability crate under the fitness umbrella by virtue of Policy B; lane spec wording is unaffected. |
| New ADR | **1** | `ADR-0055 — Adopt Policy B fitness-umbrella crate taxonomy and immediate metadata cutover`. (Next free slot per `docs/ADR-INDEX.md` "Next ADR number: 0055".) |

## §2 — Full rename list (39 rows, alphabetical by current name)

Effort key: **S** = 1 crate touch + ≤2 dep edges; **M** = 1–5 dep edges + ≥1 cross-doc cite; **L** = many dep edges + CI/script changes + cross-doc cites.

Risk key (1–5): 1 = newly-stood-up, no consumers; 5 = workspace-wide CI entry point or public-API surface.

The "Precondition" column references the row number(s) that MUST land first within the same shard (or in an earlier shard per §4). "Post-rename action" is the workspace-wide search-and-replace pattern.

| # | Current → New | Effort | Risk | Precondition | Post-rename action (grep + replace, workspace-wide) |
|---:|---|:---:|:---:|---|---|
| 1 | `oya-foundation-app` → `oya-foundation-composition-app` | M | 3 | none (foundation shard) | `\boya-foundation-app\b` → `oya-foundation-composition-app`; underscored form `oya_foundation_app` → `oya_foundation_composition_app`; `path = "../oya-foundation-app"` → `…composition-app` (2 sites: `oya-foundry-api`, `oya-tooling-cli-dev-runtime`); doc cites in `docs/CONSTITUTION.md`, `scripts/check-architecture-boundaries.sh` lines 168/175/179 |
| 2 | `oya-foundry-adr-citation-kernel` → `oya-foundry-fitness-adr-citation-kernel` | S | 2 | none | path edge in `oya-tooling-cli-dev-runtime`; `[lib]` name `oya_foundry_adr_citation_kernel` → `oya_foundry_fitness_adr_citation_kernel`; `scripts/check.sh` gate id `adr-citation` is the **gate name** (unaffected; gate identifiers are decoupled from crate names per `.omc/fitness-lanes/`) |
| 3 | `oya-foundry-adr-index-kernel` → `oya-foundry-fitness-adr-index-kernel` | S | 2 | none | same pattern as row 2 |
| 4 | `oya-foundry-api` → `oya-foundry-meta-api` | S | 2 | row 1 (this crate depends on the renamed `oya-foundation-app`) | `\boya-foundry-api\b` → `oya-foundry-meta-api`; **must NOT collide with** `oya-foundry-meta-api` already being a sibling of `oya-foundry-policy-api`; grep target is the bare `oya-foundry-api` token only (lookahead-bounded); also rewrite `scripts/check-architecture-boundaries.sh` lines 199/201/207 |
| 5 | `oya-foundry-api-semver-kernel` → `oya-foundry-fitness-api-semver-kernel` | S | 2 | none | re-parse: under the rightmost-role-token rule (per crate-naming-convention §6.1), name becomes feature=`fitness`, capability=`api-semver`, role=`kernel`. Path edge in `oya-tooling-cli-dev-runtime` |
| 6 | `oya-foundry-authority-cohesion-kernel` → `oya-foundry-fitness-authority-cohesion-kernel` | S | 2 | none | path edge + `[lib]` name |
| 7 | `oya-foundry-brand-residue-kernel` → `oya-foundry-fitness-brand-residue-kernel` | S | 2 | none | path edge + `[lib]` name |
| 8 | `oya-foundry-claim-ceiling-kernel` → `oya-foundry-fitness-claim-ceiling-kernel` | S | 2 | none | path edge + `[lib]` name; **internal dep edge**: this crate depends on `oya-foundry-catalog-kernel` (verified in §3 of inputs); the catalog crate is GREEN and unaffected |
| 9 | `oya-foundry-cloud-mutation-kernel` → `oya-foundry-fitness-cloud-mutation-kernel` | S | 2 | none | path edge + `[lib]` name |
| 10 | `oya-foundry-codeowners-mirror-kernel` → `oya-foundry-fitness-codeowners-mirror-kernel` | S | 2 | none | path edge + `[lib]` name |
| 11 | `oya-foundry-cohesion-fitness-kernel` → `oya-foundry-fitness-cohesion-kernel` | S | 2 | none | feature `cohesion-fitness` decomposes naturally: drop the redundant `fitness` token, capability = `cohesion`. Path edge + `[lib]` name |
| 12 | `oya-foundry-constitution-cite-kernel` → `oya-foundry-fitness-constitution-cite-kernel` | S | 2 | none | path edge + `[lib]` name |
| 13 | `oya-foundry-cost-budget-kernel` → `oya-foundry-fitness-cost-budget-kernel` | S | 2 | none | path edge + `[lib]` name; **internal dep edge**: `oya-foundry-adapter-kernel` consumes this crate; update that edge too |
| 14 | `oya-foundry-data-class-fitness-kernel` → `oya-foundry-fitness-data-class-kernel` | S | 2 | none | drop redundant `fitness` token; feature=`fitness`, capability=`data-class`; path edge + `[lib]` name |
| 15 | `oya-foundry-doc-catalog-kernel` → `oya-foundry-fitness-doc-catalog-kernel` | S | 2 | none | path edge + `[lib]` name |
| 16 | `oya-foundry-documentation-system-kernel` → `oya-foundry-fitness-documentation-system-kernel` | S | 2 | none | 6 segments total (foundry + fitness + documentation-system + kernel = `fitness` is the feature, `documentation-system` is the 2-token capability). Per crate-naming-convention §2, capability tail of 2 tokens is the explicit limit; AMBER (6-segment) but legal. Path edge + `[lib]` name |
| 17 | `oya-foundry-glossary-coverage-kernel` → `oya-foundry-fitness-glossary-coverage-kernel` | S | 2 | none | path edge + `[lib]` name |
| 18 | `oya-foundry-glossary-vocabulary-kernel` → `oya-foundry-fitness-glossary-vocabulary-kernel` | S | 2 | none | path edge + `[lib]` name |
| 19 | `oya-foundry-license-policy-kernel` → `oya-foundry-fitness-license-policy-kernel` | S | 2 | none | path edge + `[lib]` name |
| 20 | `oya-foundry-mcp-gateway-kernel` → `oya-foundry-fitness-mcp-gateway-kernel` | S | 2 | none | path edge + `[lib]` name. **Note**: this crate currently has an internal dep on `oya-foundry-capability-kernel`. That edge is unaffected (capability-kernel stays GREEN) |
| 21 | `oya-foundry-mobile-native-kernel` → `oya-foundry-fitness-mobile-native-kernel` | S | 2 | none | path edge + `[lib]` name |
| 22 | `oya-foundry-placeholder-debt-kernel` → `oya-foundry-fitness-placeholder-debt-kernel` | S | 2 | none | path edge + `[lib]` name |
| 23 | `oya-foundry-pr-traceability-kernel` → `oya-foundry-fitness-pr-traceability-kernel` | S | 2 | none | path edge + `[lib]` name |
| 24 | `oya-foundry-pre-push-kernel` → `oya-foundry-fitness-pre-push-kernel` | S | 2 | none | path edge + `[lib]` name |
| 25 | `oya-foundry-quality-lane-kernel` → `oya-foundry-fitness-quality-lane-kernel` | S | 2 | none | path edge + `[lib]` name |
| 26 | `oya-foundry-raci-team-coverage-kernel` → `oya-foundry-fitness-raci-coverage-kernel` | S | 2 | none | drop redundant `team` (RACI implies team); feature=`fitness`, capability=`raci-coverage`. Path edge + `[lib]` name |
| 27 | `oya-foundry-readme-doc-coverage-kernel` → `oya-foundry-fitness-readme-coverage-kernel` | S | 2 | none | drop redundant `doc` (README implies doc); feature=`fitness`, capability=`readme-coverage`. Path edge + `[lib]` name |
| 28 | `oya-foundry-release-evidence-pack-kernel` → `oya-foundry-fitness-release-pack-kernel` | M | 3 | none | drop redundant `evidence` (foundry context already implies evidence emission per ADR-0003); feature=`fitness`, capability=`release-pack`. Cross-doc cites in `docs/RELEASE-MANAGEMENT.md`, `.github/workflows/release-evidence-pack.yml` (the workflow file **name** stays per release-evidence-pack vernacular; only the `cargo run -p …` line changes — verify this with §10 critic question) |
| 29 | `oya-foundry-runbook-freshness-kernel` → `oya-foundry-fitness-runbook-freshness-kernel` | S | 2 | none | path edge + `[lib]` name |
| 30 | `oya-foundry-runbook-index-kernel` → `oya-foundry-fitness-runbook-index-kernel` | S | 2 | none | path edge + `[lib]` name |
| 31 | `oya-foundry-slo-coverage-kernel` → `oya-foundry-fitness-slo-coverage-kernel` | S | 2 | none | path edge + `[lib]` name |
| 32 | `oya-foundry-supply-chain-kernel` → `oya-foundry-fitness-supply-chain-kernel` | M | 3 | none | path edge + `[lib]` name. Cross-doc cites in `docs/RELEASE-MANAGEMENT.md`, `.github/workflows/supply-chain.yml` (workflow file name unchanged; `cargo run -p` lines flip per row 39) |
| 33 | `oya-foundry-typescript-workspace-kernel` → `oya-foundry-fitness-typescript-workspace-kernel` | S | 2 | none | path edge + `[lib]` name |
| 34 | `oya-foundry-vendor-contract-recency-kernel` → `oya-foundry-fitness-vendor-recency-kernel` | S | 2 | none | drop redundant `contract` (vendor implies contract in foundry context); feature=`fitness`, capability=`vendor-recency`. Path edge + `[lib]` name |
| 35 | `oya-platform-data-boundary-kernel` → `oya-platform-fitness-data-boundary-kernel` | M | 4 | **none — but coordinate with foundation shard**: this crate is the **only kernel allowed to receive cross-layer deps** per `clean-architecture.md` §3. Renaming it touches every consumer; audit needed in shard prep. | feature=`fitness`, capability=`data-boundary`. Workspace-wide path edge audit required (likely 5–10 consumers across `cloud`, `platform`, `workspace`, `foundry` shards). **High risk because of cross-shard consumers**. |
| 36 | `oya-tooling-agent-read` → `oya-tooling-agent-cli-read` | S | 1 | none | insert `cli` role before capability `read`; new parse: feature=`agent`, role=`cli`, capability=`read`. Zero internal consumers verified. Cargo.toml is short (2.1K) and well-formed |
| 37 | `oya-tooling-cli-dev-runtime` → `oya-tooling-dev-runtime` | L | **5** | rows 1, 4, all of rows 2–34 (this crate's `Cargo.toml` lists every fitness-umbrella kernel as a `path = "../…"` dep, so the rename must happen **after** the fitness collapses in the same shard) | feature=`dev`, role=`runtime`; collapse redundant `cli` + `runtime`. **Touches**: `.github/workflows/release-evidence-pack.yml` line 18, `.github/workflows/supply-chain.yml` lines 24+36, `scripts/check.sh` (29 sites), `scripts/hooks/pre-push-repoctl.sh` line 3, `AGENTS.md`, `docs/CONSTITUTION.md`, `docs/TOOLCHAIN.md`, `docs/research/hyperscaler-best-practices-2026-05-12.md`, `docs/RELEASE-MANAGEMENT.md`. **`[lib]` name** `oya_tooling_cli_dev_runtime` → `oya_tooling_dev_runtime` |
| 38 | *(reserved for `oya-foundry-fitness-architecture-conventions-kernel` — new lane crate; per the fitness-lane spec this is a NEW crate, not a rename; it appears here for shard-6 enumeration completeness)* | S | 1 | rows 2–34 (must land **after** fitness-umbrella collapse so the new crate name parses cleanly under Policy B) | NEW crate; ADR-0055 + lane spec already covers wording. Not counted in §1's "Crate renames: 39" total; tracked here for shard-6 visibility |
| 39 | *(reserved for AMBER 6-segment row `oya-platform-audit-chain-adapter-file` if architect elects the optional shorter-form rename to `oya-platform-audit-adapter-chain-file`)* | — | — | architect call | Plan §10 critic question: leave AMBER (no rename) vs rename for 5-segment hygiene. **Default: no rename, keep AMBER**, since the row passes grammar with `audit-chain` as a registered compound under ADR-GOV-002. Not counted in §1 total |

**Final cohort count = 37 actual renames (rows 1–37).** Rows 38 and 39 are reserved slots (one for the new lane crate per the lane spec, one for an architect-optional AMBER cleanup). The §1 summary says "39" to reflect the user-stated 39-row count (37 + 2 named decisions already absorbed at rows 4 and 37); the 2 named decisions are **rows 4 and 37 within the 37-row body**, not separate rows. **§1 reconciliation: actual unique renames = 37; the user-stated "39 total renames" double-counted the 2 named decisions as additions to a notional 37-row base. This plan adopts the unambiguous count: 37 unique crate renames, with rows 4 and 37 being the two specifically locked-in user decisions.** Architect/critic to confirm in §10.

> **Reconciliation flag for §10 critic**: User direction said "37 RED + 2 named = 39 renames." The audit's 37-RED list already includes `oya-foundry-api` (row #2 of the audit) and `oya-tooling-cli-dev-runtime` (row #37 of the audit). The two "named decisions" are **the renames specified for those two crates**, not additional rows. Hence the actual unique-rename count is **37**. The "39" in user direction reflects 37 RED rows + 2 distinct rename **decisions**, not 2 new rows. The plan goes with 37 to avoid double-counting; ADR-0055 to mirror this language.

## §3 — `[package.metadata.oya]` atomic addition

### 3.1 Block grammar (BNF)

```toml
[package.metadata.oya]
role       = "kernel" | "domain" | "app" | "api" | "worker" | "adapter" | "runtime" | "cli" | "sdk"
context    = "cloud" | "foundation" | "foundry" | "fitness" | "platform" | "tooling" | "workspace"
capability = "<kebab-case>"          # REQUIRED iff role = "adapter"; OPTIONAL otherwise; "" forbidden — omit key if absent
```

> **Context-enum note**: the canonical context enum in
> [`crate-naming-convention.md`](../standards/crate-naming-convention.md) §2 is
> the **six-value** set `cloud | foundation | foundry | platform | tooling | workspace`.
> The user-supplied BNF adds `fitness` as a seventh value. Under Policy B,
> `fitness` is a **feature** (always paired with `context = foundry`), not a
> context. **§10 critic question**: should the standard's context enum be
> extended to include `fitness` (making it a context), or should `fitness`
> remain a feature within the `foundry` context (current Policy B model)?
> The plan **assumes the latter** (no enum change); architect to confirm.
> Under that assumption the `context` enum stays at **six** values.

### 3.2 Per-crate derivation rule

For each crate's **NEW name** (post-rename for the 37 in §2, current name for the other 103):

1. Split kebab-name on `-`, drop the leading `oya` segment.
2. Match `context` against the enum (segment 1).
3. Identify `role` via the rightmost-role-token rule (per `crate-naming-convention.md` §6.1).
4. Slice the feature (segments between context and role) and the capability tail (segments after role, if any).
5. Emit:
   ```toml
   [package.metadata.oya]
   role       = "<role>"
   context    = "<context>"
   capability = "<capability-or-omit>"
   ```
6. **Special cases**:
   - Bin-only crates (e.g. `oya-tooling-agent-cli-read`, `oya-tooling-dev-runtime`): emit `role = "cli"` or `role = "runtime"` per name parse.
   - Adapter crates: capability is REQUIRED; the derivation MUST succeed or the metadata-augment helper MUST fail loudly.
   - The block MUST be added **immediately after the `[package]` block** in each `Cargo.toml`, before `[dependencies]`.

### 3.3 Helper implementation — Rust xtask

**Recommendation: ship as a Rust `xtask` binary `xtask-metadata-augment` in shard 1**, not a shell script. Rationale:

| Criterion | Bash + sed | Rust xtask |
|---|---|---|
| TOML correctness | brittle; sed against multi-line TOML loses idempotency on re-run | `toml_edit` crate preserves comments, key order, formatting |
| Idempotency | hard (sed -i + multi-line markers fight each other) | trivial (`toml_edit` upserts a block in-place) |
| Cross-platform CI | GNU sed vs BSD sed diverge on macOS runners | identical (rustc target-independent) |
| Re-use after cutover | one-shot; throwaway | reusable for every future workspace member (fitness-lane companion) |
| Time-to-ship | hours | ~half-day with `toml_edit` |
| Reviewability | regex-by-regex | structured AST |

The xtask lands as `crates/oya-tooling-dev-runtime/src/bin/xtask-metadata-augment.rs` (after row 37 rename) OR temporarily as `tools/xtask-metadata-augment/` in shard 1 if shard 1 lands before row 37 (which it does per §4). It accepts:

- `--check` — exit non-zero if any `[package.metadata.oya]` block is missing or wrong.
- `--apply` — rewrite the manifests to match the derivation rule.
- `--shard <context>` — limit to one shard's crates.

The fitness lane consumes `--check` as a sub-check (per `.omc/fitness-lanes/architecture-conventions.md` §3.2).

## §4 — Cutover order: ATOMIC PR vs SHARDED

### 4.1 Option A — single atomic PR

All 37 renames + 140 metadata blocks + ~46 dep-edge rewrites + 3 CI workflow updates + ~30 script-line updates in **one commit**.

| Aspect | Atomic-PR verdict |
|---|---|
| Half-state period | **zero** — workspace is either in v1 state or v2 state, never mid-cutover |
| Rollback | clean: `git revert <sha>` restores everything |
| Review surface | **enormous** — ~190 file changes; cognitive load exceeds the prevention-doctrine §3 reviewability cap |
| CI runtime | one very long `cargo check --workspace --all-features` + `cargo nextest run` cycle |
| Merge-queue contention | one PR holds the queue for the duration |
| Bisectability after merge | ❌ — all renames in one commit; can't bisect to find which row broke a downstream consumer |
| Concurrent feature branch conflict | maximal — every in-flight branch hits at least one renamed file |

### 4.2 Option B — sharded by context (6 shards)

6 PRs, each renames the crates within one context (cloud / foundation / foundry+fitness / platform / tooling / workspace) plus its metadata adds.

| Aspect | Sharded verdict |
|---|---|
| Half-state period | 5 intermediate states (shard 1 merged, shard 2..6 pending; etc.) |
| Rollback | per-shard revert is clean; cross-shard revert needs care |
| Review surface | per-shard ~10–50 file changes; **well within reviewability cap** |
| CI runtime | per-shard `cargo check --workspace` (still the whole workspace, but shorter to a stable green per shard) |
| Merge-queue contention | one shard at a time; 48 h freeze between merges |
| Bisectability | excellent — each shard is one commit; per-shard granularity |
| Concurrent feature branch conflict | per-shard scope; in-flight branches hit only the merging shard |

### 4.3 Recommendation: **Option B — sharded**

**Rationale**: review surface and bisectability dominate. The cohort touches 140 crates; one PR exceeds the reviewability cap and renders bisection useless. The 5 half-states are bounded by the per-shard `cargo check` + `cargo build` + fitness-lane gates (see §5), so no shard merges in a broken state. The 48 h freeze window between shards (§10 critic question) bounds branch-conflict cost.

### 4.4 Shard sequence (locked)

| Shard | Context(s) | Crates touched (rename + metadata) | Rationale for ordering |
|:---:|---|---:|---|
| **1** | `tooling` | 2 renames (rows 36, 37) + 4 metadata adds (`oya-tooling-cli-dev-runtime` becomes `oya-tooling-dev-runtime`; `oya-tooling-agent-read` becomes `oya-tooling-agent-cli-read`) + metadata for the 2 already-renamed crates + xtask-metadata-augment ships here | **Smallest blast radius for the 2 named renames; ships the metadata helper that powers every subsequent shard's metadata-add step.** Row 37 has the highest risk (CI, scripts) but the smallest cardinality. Get it out of the way first while no other shard depends on it. **CRITICAL**: shard 1 must land **before** the foundry+fitness shard, because the dev-runtime's `Cargo.toml` must already be in its post-rename state when its dep edges flip in shard 6. |
| **2** | `workspace` | 0 renames + 23 metadata adds (every workspace-* crate is GREEN today; only metadata is added) | Smallest non-tooling shard; pure metadata. Use to validate the xtask end-to-end before any rename-bearing shard. |
| **3** | `cloud` | 0 renames + 23 metadata adds | All cloud-* crates GREEN; pure metadata. |
| **4** | `platform` | 1 rename (row 35: `oya-platform-data-boundary-kernel → oya-platform-fitness-data-boundary-kernel`) + 18 metadata adds | Single rename; high-risk because data-boundary-kernel is the cross-layer-allowed kernel per clean-architecture §3. Land here before foundation+foundry shards so the platform's dep-direction is settled. |
| **5** | `foundation` | 1 rename (row 1: `oya-foundation-app → oya-foundation-composition-app`) + 1 metadata add | Singleton composition root; renaming here unblocks shard 6 row 4 (`oya-foundry-api → oya-foundry-meta-api` depends on the renamed foundation-app). |
| **6** | `foundry` + `fitness` collapse | 33 renames (rows 2–34, plus row 4 if not landed in shard 5 prep) + 71 metadata adds (17 GREEN foundry + 26 fitness-umbrella renamed + 3 LONG-FEATURE collapsed + 3 already-AMBER + new lane crate row 38) | **Largest shard; runs last because it depends on every prior shard's rename being settled.** The dev-runtime's `Cargo.toml` is rewritten in **shard 1**; shard 6 only updates dep names in the dev-runtime's path edges. Lane crate `oya-foundry-fitness-architecture-conventions-kernel` is created here. |

**Total: 6 shards. Total wall-clock: see §9.**

## §5 — Per-shard checklist (template; applied to each shard in §4.4)

Each shard PR MUST satisfy every item below **before** merge. Lane runs in `--report-only` on shard 1; flips to BLOCKER after shard 1 lands (post-shard-1 = the metadata helper exists and emits per-crate reports).

1. **Pre-flight: external-repo audit.** Run `gh search code 'oya-<old-name>' --owner <oyatie-org>` across every Oyatie repo and every Oyatie-internal consumer. List external coordination requirements; if any external repo imports the renamed crate **by path**, file a coordination issue **before** the shard's freeze window opens. Internal-only crates (all 140 today are `publish = false`; verified via `rtk grep -c "publish = false"`) **may proceed without external coordination**.
2. **ICM scaffold-claim lock per ADR-0054**. For each NEW crate name in the shard, write an ICM scaffold-claim row with topic `decisions-oyatie-rename-v2`, content="claim shard-N rename for `<new-name>`", importance=high, keywords=`rename,shard-N,policy-b`. This satisfies ADR-0054's "every new workspace member needs a scaffold-claim row" rule because the rename creates a "new" member from `grit`'s perspective (the directory path changes).
3. **Workspace `Cargo.toml` member list update.** For each rename, change `crates/oya-<old>` → `crates/oya-<new>` in the `[workspace] members = [...]` array.
4. **Per-crate `Cargo.toml` rename + metadata block.** For each crate in the shard (renamed or not), run the xtask-metadata-augment helper. Each crate's `Cargo.toml` ends with a correct `[package.metadata.oya]` block immediately after `[package]`.
5. **Per-crate `[lib]` name update.** For renamed crates, update the implicit underscored `[lib] name = "..."` if it is explicitly declared (sample audit shows most crates rely on the default; explicit declarations exist in adapter crates and must be updated).
6. **`cargo check --workspace --all-features` MUST pass after the shard's last commit.**
7. **`cargo build --workspace --release` MUST pass after the shard's last commit.**
8. **`cargo nextest run --workspace --all-features` MUST pass with test count ≥ pre-shard test count.**
9. **`cargo tree --workspace` diff audit** — capture before/after; verify no hidden indirect deps regressed (per §6 R4).
10. **New ADR row in `docs/ADR-INDEX.md`**. Shard 1 adds the `ADR-0055` row referencing the Policy B taxonomy decision (this plan). Each subsequent shard appends a one-line shard-completion entry to `docs/CHANGELOG.md`, citing ADR-0055.
11. **`docs/CHANGELOG.md` entry** per shard (one row per rename: `- Renamed `oya-<old>` → `oya-<new>` (shard-N, ADR-0055)`).
12. **Fitness lane `oya-foundry-fitness-architecture-conventions` MUST be GREEN** after the shard's last commit. Shard 1 runs in `--report-only` mode (lane infra not yet shipped); shard 6 must be GREEN with the lane in BLOCKER mode by end of shard.
13. **Doc-cross-reference sweep.** Run `rg -l "<old-name>"` over `docs/`, `.omc/`, `AGENTS.md`, `CLAUDE.md`, `scripts/`, `.github/`. List the hits in the PR description; the reviewer signs off on each as "renamed", "left as historical record", or "deferred to follow-up".
14. **48 h merge freeze on the next shard.** The next shard's PR may open during this window for review, but MUST NOT merge until the freeze elapses; gives in-flight feature branches a window to rebase.

## §6 — Risk-cone summary (R1–R5)

| Risk | Likelihood | Impact | Mitigation |
|---|:---:|:---:|---|
| **R1 — External repos break.** Any external repo importing a renamed crate by path or `crates.io` name breaks. | L | M | Pre-flight grep (per-shard checklist item 1); advance notice via Oyatie-internal channels; all 140 crates verified `publish = false` so crates.io collision is impossible; path-based imports from external repos go through `[patch]` overrides in the consumer's `Cargo.toml` and are listed in shard prep. |
| **R2 — In-flight feature branches conflict.** Open PRs against `main` collide with the renamed `Cargo.toml`s. | M | M | Announce a 48 h freeze window before shard 1 (per §5 item 14); shard sequence is leaves → roots so the largest shard (foundry+fitness, shard 6) hits the most in-flight work last; `git rerere` enabled on the merge queue to absorb repeat conflicts. |
| **R3 — `cargo-deny` rules referencing old crate names.** `deny.toml` could ban or `[bans.skip]` an old name. | L | L | Audited `deny.toml` (205 bytes, licenses section only — no `[bans]` rules); confirmed it references **no crate names**. Re-run the audit in shard 1 pre-flight to catch any drift. |
| **R4 — Hidden indirect deps via workspace edition inheritance.** Renamed crates inherit `edition.workspace = true` etc.; rename could expose a previously-hidden version skew. | L | M | `cargo tree --workspace` diff in checklist item 9; lock the lockfile in shard 1 (`cargo update --workspace` only on explicit lockfile-regen commits, never as a rename side-effect). |
| **R5 — Crates.io publish collision.** Renamed crate clashes with a public crate. | L | L | All 140 crates `publish = false` (re-verified); registry namespace not at risk. Documented for ADR-0055 to bind for future audits. |
| **R6 (new) — Lane bootstrap chicken-and-egg.** Shard 6 creates the fitness lane crate; the lane validates the renames; but the lane crate can only land **after** the renames it validates. | M | M | Lane runs in `--report-only` mode through shard 5; flips to BLOCKER as the **final commit** of shard 6 after `cargo check` is GREEN; the lane crate's tests cover the post-shard-6 state, not the in-flight shards. |
| **R7 (new) — `[lib]` name drift on rename.** Rust crates with explicit `[lib] name = "oya_foundry_..."` keep the old underscored form, breaking downstream `use oya_foundry_...::` imports. | M | H | Helper xtask MUST audit and rewrite `[lib] name` in lockstep with `[package] name`; per-shard checklist item 5 enforces it; reviewer sign-off required on any crate that ships an explicit `[lib]` declaration. |

## §7 — Rollback plan

### 7.1 Per-shard rollback

Each shard PR is independently revertable via `git revert <shard-merge-sha>`. The revert:

1. Restores the workspace `Cargo.toml` member list.
2. Restores each renamed crate's `Cargo.toml` to the pre-shard name.
3. Restores the dep-edge `path = "../..."` strings.
4. Restores the `[package.metadata.oya]` blocks to their pre-shard state (in shard 1's case, removes them entirely).

After the revert, run `cargo check --workspace`, `cargo build --workspace`, and `cargo nextest run --workspace --all-features`. Lockfile drift is the most common post-revert failure; if it occurs, regen via `cargo update --workspace -p <one-crate>` minimally and commit.

### 7.2 Cross-shard rollback

If a downstream shard fails review after merge and the rollback requires reverting an earlier shard, the order is **last-merged first**: revert shard 6, then 5, etc. Each revert is its own PR; the merge queue handles ordering. Never `git reset --hard` past a merged shard's tag.

### 7.3 ICM coordination locks

Each shard's scaffold-claim ICM rows (per §5 item 2) get a follow-up `icm store -t decisions-oyatie-rename-v2 -c "shard-N reverted on <date>"` row on rollback, with importance=critical to keep the audit chain intact.

### 7.4 Rollback time budget

< 30 min per shard if revert is clean. Lockfile drift adds ~10–20 min. CI re-runs add ~15–20 min. Total < 90 min worst case.

## §8 — Acceptance gate (per shard, then global)

### 8.1 Per-shard gate (every shard MUST satisfy before next shard opens)

1. All §5 per-shard checklist items GREEN.
2. `cargo check --workspace --all-features` GREEN on the shard's merge commit.
3. `cargo build --workspace --release` GREEN.
4. `cargo nextest run --workspace --all-features` GREEN with test count ≥ pre-shard baseline.
5. Fitness lane: shards 1–5 in `--report-only`, shard 6 in BLOCKER on merge commit.

### 8.2 Global gate (after shard 6 merge)

1. **Zero hits**: `rg "oya-foundation-app\b"` (post shard 5), `rg "oya-tooling-cli-dev-runtime\b"` (post shard 1), `rg "oya-foundry-fitness-" -g '!docs/CHANGELOG.md' -g '!docs/plans/rename-plan-2026-05-12.md'` MUST yield only doc-history hits (CHANGELOG, the superseded v1 plan, ADR-0055 ledger).
2. `cargo check --workspace --all-features` GREEN.
3. `cargo build --workspace --release` GREEN.
4. `cargo nextest run --workspace --all-features` GREEN.
5. Fitness lane `oya-foundry-fitness-architecture-conventions-kernel` GREEN in BLOCKER mode.
6. **Zero AMBER-metadata rows** (every workspace crate has a valid `[package.metadata.oya]` block per §3.1).
7. ADR-0055 status flipped from `Proposed` to `Accepted`.

## §9 — Estimated effort (sonnet executors)

| Shard | Wall-clock (executor) | Reviewer time | Bottleneck |
|:---:|---:|---:|---|
| 1 (tooling) | **4–6 h** | 1–2 h | xtask-metadata-augment authoring + row 37 CI cutover testing |
| 2 (workspace, metadata-only) | **2–3 h** | 30 min | xtask invocation + per-crate metadata verification |
| 3 (cloud, metadata-only) | **2–3 h** | 30 min | same as shard 2 |
| 4 (platform + 1 rename) | **3–4 h** | 1 h | cross-layer-allowed kernel dep-edge audit (row 35) |
| 5 (foundation + 1 rename) | **2 h** | 30 min | singleton; small |
| 6 (foundry + fitness, 33 renames) | **8–12 h** | 2–3 h | biggest dep-edge rewrite (~44 edges in dev-runtime alone) + new lane crate |
| **Total** | **~22–30 h executor + 6–8 h reviewer** | — | — |
| Per-shard rollback | < 30 min | — | git revert + lockfile regen if needed |

48 h freeze window between shards adds 48 h × 5 = ~10 days of wall-clock between shard merges. Total calendar time ≈ **2 weeks** from shard 1 open → shard 6 merged.

## §10 — Open questions for `/ralplan --critic` (Codex consensus pressure-test)

The plan defers the following decisions to consensus adjudication. Codex critic to pressure-test each:

1. **Sharded vs atomic (force the recommendation)**. Plan recommends Option B (6 shards). Critic to challenge: is the 2-week calendar cost worth the bisectability gain over the 1-day atomic-PR cost? Specifically, can the merge-queue throughput absorb the 6 sequential merges without starving other in-flight work?
2. **ADR slot reconciliation**. v1 plan proposed ADR-FND-008 for the registry-extension decision; that namespace never landed (ADR-INDEX shows numeric ADR-0001..0054, "Next ADR number: 0055"). This plan reserves **ADR-0055** for "Adopt Policy B fitness-umbrella crate taxonomy and immediate metadata cutover". Critic to confirm the numeric slot is correct and the title scope is right.
3. **xtask-metadata-augment in shard 1 (vs deferred)**. Plan recommends shipping the metadata helper as a Rust xtask in shard 1 so shards 2–6 can use `--apply` mechanically. Critic to challenge: does shard 1's already-high risk (row 37 CI cutover) absorb the additional risk of a new helper landing in the same shard, or should the helper be a shard 0 (pure tooling, no rename, no metadata) precursor?
4. **48 h freeze window length**. Plan suggests 48 h between shard merges. Critic to challenge: is 48 h enough for in-flight feature branches to rebase, or too long given the 6-shard sequence? Some teams prefer 24 h or 72 h; pick a defensible number.
5. **Context-enum extension (`fitness` as context vs feature)**. Plan §3.1 keeps `fitness` as a feature under context `foundry`. If critic prefers extending the context enum to 7 values (adding `fitness` as a context), every crate in shard 6 reparses (feature segment vanishes, capability segment grows); the change is mechanical but the implications for `clean-architecture.md` layer-direction rules are non-trivial. Decision needed **before** shard 6 opens.
6. **`oya-platform-data-boundary-kernel` rename scope (row 35)**. This is the **only kernel** that may receive cross-layer deps per `clean-architecture.md` §3. Renaming touches every consumer's import. Critic to challenge: is the rename worth the blast radius, or should the row be deferred (left AMBER with a registered `data-boundary` compound) until a separate dedicated PR?
7. **AMBER-row optional cleanup (row 39 reservation)**. Should `oya-platform-audit-chain-adapter-file` rename to a shorter 5-segment form? Default: no. Critic to confirm.
8. **Workflow file names**. The plan flips `cargo run -p oya-tooling-cli-dev-runtime` to `oya-tooling-dev-runtime` inside `.github/workflows/{release-evidence-pack,supply-chain}.yml`, but **keeps the workflow file names** themselves unchanged. Critic to confirm this is desirable (the file names are CI's external observable surface; flipping them rotates GitHub-side history).

## §11 — Final ADR (to be authored alongside shard 1)

### ADR-0055 — Adopt Policy B fitness-umbrella crate taxonomy and immediate metadata cutover

- **Status**: Proposed (flips to Accepted at end of shard 6).
- **Decision**: Collapse the foundry-fitness kernel family under a `fitness` feature umbrella within context `foundry`; ship `[package.metadata.oya]` blocks for all 140 workspace members in the same execution window; cut over in 6 sharded PRs.
- **Decision drivers** (top 3):
  1. Reviewability — atomic-PR cohort exceeds the prevention-doctrine reviewability cap; sharded PRs land within it.
  2. Audit-chain integrity — Policy A leaves AMBER-metadata deferred to Q3-2026, creating a 5-month window where 140 crates carry an AMBER row; Policy B closes that window in one cutover.
  3. Hyperscaler convergence — AWS / Azure / GCP all collapse "kind-of-crate" tail-noun families under a `context = <noun>` umbrella (per `hyperscaler-best-practices-2026-05-12.md` Domain 3); Policy B aligns with the convergent practice.
- **Alternatives considered**:
  - **Policy A (v1)** — registry-admit 28 compounds, 9 renames, AMBER-metadata deferred to Q3-2026. **Why rejected**: leaves 140-crate AMBER-metadata obligation hanging for 5 months; user-stated audit-chain rigor priority overrides the smaller-blast-radius argument.
  - **Atomic-PR cutover** — single commit, 190+ file changes. **Why rejected**: exceeds reviewability cap; destroys bisectability.
- **Why chosen**: as above.
- **Consequences**:
  - Positive: zero AMBER-metadata rows after shard 6; consistent taxonomy across foundry-fitness; fitness-lane crate parses unambiguously under Policy B.
  - Negative: 2-week calendar cost; 5 intermediate half-states (mitigated by per-shard cargo-check gates); larger PR review burden on shard 6.
- **Follow-ups**:
  1. Promote ADR-0054 scaffold-claim rule to also cover crate-rename events (currently scoped to new workspace members; renames need explicit coverage).
  2. Document the rightmost-role-token parser rule in `crate-naming-convention.md` §6.1 in a separate cleanup PR after shard 6.
  3. Consider extending the context enum to include `fitness` (per §10 question 5) as a follow-up ADR.
  4. Update `docs/research/hyperscaler-best-practices-2026-05-12.md` Domain 3 cross-references to cite ADR-0055.

## §12 — Cross-references

- **Superseded plan**: [`docs/plans/rename-plan-2026-05-12.md`](rename-plan-2026-05-12.md) (v1, Policy A; status `Superseded`).
- **Audit inventory**: [`docs/audits/convention-audit-2026-05-12.md`](../audits/convention-audit-2026-05-12.md).
- **Grammar**: [`docs/standards/crate-naming-convention.md`](../standards/crate-naming-convention.md).
- **Layering**: [`docs/standards/clean-architecture.md`](../standards/clean-architecture.md).
- **Lane spec**: [`.omc/fitness-lanes/architecture-conventions.md`](../../.omc/fitness-lanes/architecture-conventions.md).
- **ICM decision provenance**: `01KRFMEVN49BB6J0QWKNGATC1K` (Policy B + immediate metadata cutover, locked 2026-05-12 ~23:00 ET).
- **Open questions ledger**: [`/Users/jasonlee/oyatie/.omc/plans/open-questions.md`](../../.omc/plans/open-questions.md).
