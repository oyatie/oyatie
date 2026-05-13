---
doc_class: HowTo
shape: ~
length_cap: 1200
authority_tier: 3
status: Superseded
superseded_by: docs/plans/rename-plan-v4-clean-arch-2026-05-13.md
pending: superseded
iteration: 3
architect_iter_1: SOUND-WITH-CONDITIONS
critic_iter_1: ITERATE
architect_iter_2: SOUND-WITH-CONDITIONS-3
critic_iter_2: ITERATE-7
architect_iter_3: SOUND-WITH-CONDITIONS (3 residuals, pre-folded → CLOSED)
critic_iter_3: APPROVE-WITH-CONDITIONS (3 conditions, folded)
date: 2026-05-12
last_modified: 2026-05-13
post_approval_correction_1: "lockfile-rename Python→Rust (consolidate to xtask-metadata-augment subcommand; rationale: workspace consistency, parser reuse, eliminate cross-language toolchain pin)"
supersedes: docs/plans/rename-plan-v2-2026-05-12.md
purpose: |
  Execution plan v3 for the 140-crate convention cutover. Revises v2 in
  response to iter-1 consensus signals (Architect SOUND-WITH-CONDITIONS,
  Critic ITERATE). Adopts **Hybrid C** (Shard 0 pure-tooling precursor +
  Shard 1 atomic 37-rename) per both reviewers' dominant convergence.
  Re-baselines row 35 consumer surface to **95 manifest consumers**.
  Aligns metadata-block schema to crate-naming-convention.md §7 (adds
  `feature`, `layer`, `audit_chain`). Promotes [lib]-name-drift control
  to permanent-controls ledger (5 layers). Pre-authorises emergency
  revert lane. Replaces all human judgement gates with deterministic
  allowlisted commands. Plan-only; no rename is performed by this
  document. Consensus iter-2 (Architect + Codex critic) MUST sign off
  before Shard 1 ships.
canonical_authority: docs/CONSTITUTION.md
companion_docs:
  - docs/standards/crate-naming-convention.md
  - docs/standards/clean-architecture.md
  - docs/standards/git-workflow.md
  - docs/standards/testing.md
  - docs/audits/convention-audit-2026-05-12.md
  - docs/plans/rename-plan-v2-2026-05-12.md
  - .omc/fitness-lanes/architecture-conventions.md
related_adrs:
  - ADR-0015
  - ADR-0017
  - ADR-0054
  - ADR-0055
---

> # SUPERSEDED — 2026-05-13
>
> **This plan is superseded by**
> [`docs/plans/rename-plan-v4-clean-arch-2026-05-13.md`](rename-plan-v4-clean-arch-2026-05-13.md).
>
> v3 reached consensus-approval (Architect SOUND-WITH-CONDITIONS-CLOSED +
> Critic APPROVE-WITH-CONDITIONS-CLOSED) but a user pressure-test exposed
> three over-engineered layers that v3 carried forward:
>
> 1. Verbose `oya-<context>-<feature>-<capability>-<role>` BNF (4–5
>    segments; produced names like
>    `oya-foundry-fitness-architecture-conventions-kernel`).
> 2. `oya-foundry-fitness-freeze-window-kernel` lane primitive — duplicates
>    grit's existing claim/symbol-lock system.
> 3. "Fitness" terminology imported wholesale from *Building Evolutionary
>    Architectures* jargon — replaced by plain "check" in v4.
>
> v4 adopts a canonical Rust Clean Architecture grammar
> (`oya-<bounded-context>-<layer>` with a 9-value closed layer enum
> covering domain/application/infrastructure/cli/rest/grpc/graphql/worker/sdk)
> plus a flat `oya-check-<rule-name>` namespace. Hybrid C topology,
> xtask-metadata-augment Rust crate, lockfile-rename subcommand, 4-layer
> branch pipeline, 48 h coordinated freeze, and deterministic acceptance
> gates all port forward from v3 unchanged.
>
> **v3 is retained in tree as historical record.** Do NOT execute v3.
> Defer to v4 for all rename-cutover work.
>
> ---

# Rename Plan v3 — 2026-05-12 (Hybrid C: Shard 0 precursor + atomic Shard 1) — SUPERSEDED by v4 (2026-05-13)

> **Supersedes** [`rename-plan-v2-2026-05-12.md`](rename-plan-v2-2026-05-12.md).
> v2 recommended Option B (6 sequential context-shards). Iter-1 consensus
> review surfaced a dominant convergence: both Architect (SOUND-WITH-CONDITIONS,
> condition 9) and Critic (ITERATE, edit #1) listed "switch to Hybrid C" as the
> top revision. v3 adopts Hybrid C and rebuilds every section that the
> reviewers flagged. ICM provenance: `01KRFMEVN49BB6J0QWKNGATC1K` (Policy B +
> immediate metadata cutover, locked 2026-05-12 ~23:00 ET).
>
> **What changed from v2 (summary):**
> 1. Recommendation flipped from Option B (6 shards) to **Hybrid C** (Shard 0 + atomic Shard 1).
> 2. Row 35 consumer surface re-baselined: `oya-platform-data-boundary-kernel` has **95 manifest consumers** (grep-verified), not "5-10" as v2 §:98 claimed.
> 3. Metadata schema aligned with `crate-naming-convention.md` lines 266-272: adds `feature`, `layer`, `audit_chain` keys + workspace metadata registry.
> 4. ADR-0054 amendment ships in the **same commit** as ADR-0055 (not deferred to Follow-up 1).
> 5. Context-vs-feature decision for `fitness` resolved BEFORE Shard 1 (Shard 0 deliverable).
> 6. Every compound capability (rows 16, 22, 23, others) audited with explicit token-count column + ADR-0055 citation when >1 token.
> 7. R7 ([lib] name drift) promoted to permanent-controls ledger with all 5 layers (preflight + ledger + lane + ICM + citation probe).
> 8. New R8 (rust-analyzer cache recovery), R9 (cargo-semver-checks baseline strategy), R10 (security-P0 expedite lane).
> 9. Every acceptance gate is now a runnable command with deterministic exit code (no human "reviewer signs off"/"doc-history hits").
> 10. Cargo.lock churn handled by single squash-merge + deterministic **scripted old→new workspace-name rewrite** of `Cargo.lock` (sed pass, no resolver), followed by `cargo check --workspace --locked --offline` which refuses any non-name change. (See §8.1 "Cargo.lock semver-section parity" gate and EDIT-1 rationale: `--offline` alone does NOT prevent the resolver from picking different patch versions already cached in the local registry; `--locked` is the gate that refuses any non-name delta.)

## §1 — Scope summary (re-baselined)

| Item | Count | Notes |
|---|---:|---|
| Crate renames | **37** | Same cohort as v2; row 4 (`oya-foundry-api → oya-foundry-meta-api`) and row 37 (`oya-tooling-cli-dev-runtime → oya-tooling-dev-runtime`) are the two named-decision rows the user counted separately to reach "39". v3 normalises to **37 unique renames**. ADR-0055 mirrors. |
| `[package.metadata.oya]` blocks added | **140** | All workspace members. Schema per §3 (now matches `crate-naming-convention.md:266-272`). Atomic with Shard 1 cutover. |
| Dependency-edge rewrites | **~44** | Audited in `oya-tooling-cli-dev-runtime/Cargo.toml` (becomes `oya-tooling-dev-runtime`); plus 1 `oya-foundry-api → oya-foundation-composition-app` path edge after row 1 rewrite; plus 1 `oya-foundry-claim-ceiling → oya-foundry-fitness-claim-ceiling` neighbour edge inside `oya-foundry-catalog-kernel` (rev-dep, unchanged target). |
| **Row 35 consumer surface (re-baselined)** | **95 manifests** | `oya-platform-data-boundary-kernel` is the **only kernel** permitted to receive cross-layer deps per `clean-architecture.md §3`. Evidence command (EDIT-6; unquoted, manifest-scoped, excludes root + the crate's own manifest): `rg -l oya-platform-data-boundary-kernel -g 'Cargo.toml' \| grep -v -E '^Cargo\.toml$\|crates/oya-platform-data-boundary-kernel/Cargo\.toml$'` returns **95 hits** (Codex iter-1 evidence). v2 §:98 claimed "likely 5-10 consumers"; that was wrong by an order of magnitude. Row 35's effort/risk numbers updated in §2 accordingly. The §8.1 cargo-metadata reverse-dep count gate enforces `== 95` post-rename. |
| Cargo.lock churn events | **1** | Hybrid C atomic Shard 1 ⇒ exactly **one** lockfile-regen commit (old names removed in one transaction). Option B's 6 shards would have produced **6 lockfile-churn events** with mid-state half-lockfiles consuming reviewer attention. This was a hidden Option B cost not priced in v2 §4.2. |
| CI workflow updates | **3 files** | `.github/workflows/release-evidence-pack.yml` (1 site) and `.github/workflows/supply-chain.yml` (2 sites: lines 24, 36) reference `cargo run -p oya-tooling-cli-dev-runtime …`. **No other GitHub Actions reference the renamed crates by path** (verified `rg -rn` over `.github/`). |
| Scripts updates | **2 files, ~30 lines** | `scripts/check.sh` (29 invocations of `oya-tooling-cli-dev-runtime`) and `scripts/hooks/pre-push-repoctl.sh` (1). `scripts/check-architecture-boundaries.sh` references `oya-foundation-app` and `oya-foundry-api` as crate-name strings (3 sites). |
| Registry references | **3 files** | `registry/quality/lanes.yaml`, `registry/docs/pipeline.tsv`, registry OpenAPI bindings — added per Critic edit #9; co-edit required in same commit. |
| Standards doc co-edits | **2 files** | `docs/standards/clean-architecture.md §3` (row 35: named-by-identity row updated to `oya-platform-fitness-data-boundary-kernel`); `docs/standards/crate-naming-convention.md §6` (compound features list refreshed). Both co-edit in the Shard 1 commit. |
| New fitness lane crate | **1** | `oya-foundry-fitness-architecture-conventions-kernel` per `.omc/fitness-lanes/architecture-conventions.md`. Scaffolded in **Shard 0**, populated in Shard 1. |
| New ADR | **1** | ADR-0055 + same-commit amendment to ADR-0054 (rename-scaffold-claim authority extension). |

## §2 — Full rename list (37 rows, alphabetical, with capability-token-count audit)

**Effort key**: S = 1 crate touch + ≤2 dep edges; M = 1–5 dep edges + ≥1 cross-doc cite; L = many dep edges + CI/script changes + cross-doc cites.

**Risk key (1–5)**: 1 = newly-stood-up, no consumers; 5 = workspace-wide CI entry point or public-API surface.

**Cap-tok column**: capability-token count (per `crate-naming-convention.md §2` BNF: capability is 1..2 tokens). `1` = simple capability; `2` = compound capability (requires ADR-0055 §"compound capability audit" cite + `[workspace.metadata.oya].compound_features` registry entry).

| # | Current → New | Effort | Risk | Cap-tok | Precondition | Post-rename action (grep + replace, workspace-wide) |
|---:|---|:---:|:---:|:---:|---|---|
| 1 | `oya-foundation-app` → `oya-foundation-composition-app` | M | 3 | n/a (role-only tail) | none | `\boya-foundation-app\b` → `oya-foundation-composition-app`; underscored form for `[lib]`; `path = "../oya-foundation-app"` (2 sites: `oya-foundry-api`, `oya-tooling-cli-dev-runtime`); cites in `docs/CONSTITUTION.md`, `scripts/check-architecture-boundaries.sh` lines 168/175/179 |
| 2 | `oya-foundry-adr-citation-kernel` → `oya-foundry-fitness-adr-citation-kernel` | S | 2 | **2** (`adr-citation`) | none — `compound_features` row added in Shard 0 | path edge in `oya-tooling-cli-dev-runtime`; `[lib]` name `oya_foundry_adr_citation_kernel` → `oya_foundry_fitness_adr_citation_kernel`; gate id `adr-citation` is the **gate name** (unaffected) |
| 3 | `oya-foundry-adr-index-kernel` → `oya-foundry-fitness-adr-index-kernel` | S | 2 | **2** (`adr-index`) | none — `compound_features` row | same pattern as row 2 |
| 4 | `oya-foundry-api` → `oya-foundry-meta-api` | S | 3 | 1 (`meta` is feature, no capability) | row 1 (this crate depends on renamed foundation) | `\boya-foundry-api\b` → `oya-foundry-meta-api`; MUST NOT collide with `oya-foundry-meta-api` sibling — token boundary regex `\bx\b`; `scripts/check-architecture-boundaries.sh` lines 199/201/207 |
| 5 | `oya-foundry-api-semver-kernel` → `oya-foundry-fitness-api-semver-kernel` | S | 2 | **2** (`api-semver`) | none | re-parse: feature=`fitness`, capability=`api-semver`, role=`kernel`; `api-semver` already in v2's compound registry — preserved |
| 6 | `oya-foundry-authority-cohesion-kernel` → `oya-foundry-fitness-authority-cohesion-kernel` | S | 2 | **2** (`authority-cohesion`) | none — new compound row | path edge + `[lib]` name |
| 7 | `oya-foundry-brand-residue-kernel` → `oya-foundry-fitness-brand-residue-kernel` | S | 2 | **2** (`brand-residue`) | none — new compound row | path edge + `[lib]` name |
| 8 | `oya-foundry-claim-ceiling-kernel` → `oya-foundry-fitness-claim-ceiling-kernel` | S | 2 | **2** (`claim-ceiling`) | none — new compound row | path edge + `[lib]` name; internal dep on `oya-foundry-catalog-kernel` (unaffected) |
| 9 | `oya-foundry-cloud-mutation-kernel` → `oya-foundry-fitness-cloud-mutation-kernel` | S | 2 | **2** (`cloud-mutation`) | none — new compound row | path edge + `[lib]` name |
| 10 | `oya-foundry-codeowners-mirror-kernel` → `oya-foundry-fitness-codeowners-mirror-kernel` | S | 2 | **2** (`codeowners-mirror`) | none — new compound row | path edge + `[lib]` name |
| 11 | `oya-foundry-cohesion-fitness-kernel` → `oya-foundry-fitness-cohesion-kernel` | S | 2 | 1 (`cohesion`) | none | drop redundant `fitness` token; capability = `cohesion` (1 token); path edge + `[lib]` name |
| 12 | `oya-foundry-constitution-cite-kernel` → `oya-foundry-fitness-constitution-cite-kernel` | S | 2 | **2** (`constitution-cite`) | none — new compound row | path edge + `[lib]` name |
| 13 | `oya-foundry-cost-budget-kernel` → `oya-foundry-fitness-cost-budget-kernel` | S | 2 | **2** (`cost-budget`) | none — new compound row | path edge + `[lib]` name; rev-dep edge in `oya-foundry-adapter-kernel` |
| 14 | `oya-foundry-data-class-fitness-kernel` → `oya-foundry-fitness-data-class-kernel` | S | 2 | **2** (`data-class`) | none | drop redundant `fitness`; `data-class` already in compound registry — preserved |
| 15 | `oya-foundry-doc-catalog-kernel` → `oya-foundry-fitness-doc-catalog-kernel` | S | 2 | **2** (`doc-catalog`) | none — new compound row | path edge + `[lib]` name |
| 16 | `oya-foundry-documentation-system-kernel` → `oya-foundry-fitness-documentation-system-kernel` | S | 3 | **2** (`documentation-system`) | none — new compound row; **AMBER 6-segment** | 6 segments total; per `crate-naming-convention.md §2` constraint 1, AMBER requires ADR-0055 cite. Token count is at the explicit 2-token capability cap. Path edge + `[lib]` name |
| 17 | `oya-foundry-glossary-coverage-kernel` → `oya-foundry-fitness-glossary-coverage-kernel` | S | 2 | **2** (`glossary-coverage`) | none — new compound row | path edge + `[lib]` name |
| 18 | `oya-foundry-glossary-vocabulary-kernel` → `oya-foundry-fitness-glossary-vocabulary-kernel` | S | 2 | **2** (`glossary-vocabulary`) | none — new compound row | path edge + `[lib]` name |
| 19 | `oya-foundry-license-policy-kernel` → `oya-foundry-fitness-license-policy-kernel` | S | 2 | **2** (`license-policy`) | none — new compound row | path edge + `[lib]` name |
| 20 | `oya-foundry-mcp-gateway-kernel` → `oya-foundry-fitness-mcp-gateway-kernel` | S | 2 | **2** (`mcp-gateway`) | none — new compound row | path edge + `[lib]` name. Internal dep on `oya-foundry-capability-kernel` unaffected |
| 21 | `oya-foundry-mobile-native-kernel` → `oya-foundry-fitness-mobile-native-kernel` | S | 2 | **2** (`mobile-native`) | none — new compound row | path edge + `[lib]` name |
| 22 | `oya-foundry-placeholder-debt-kernel` → `oya-foundry-fitness-placeholder-debt-kernel` | S | 2 | **2** (`placeholder-debt`) | none — new compound row | path edge + `[lib]` name |
| 23 | `oya-foundry-pr-traceability-kernel` → `oya-foundry-fitness-pr-traceability-kernel` | S | 2 | **2** (`pr-traceability`) | none — new compound row | path edge + `[lib]` name |
| 24 | `oya-foundry-pre-push-kernel` → `oya-foundry-fitness-pre-push-kernel` | S | 2 | **2** (`pre-push`) | none — new compound row | path edge + `[lib]` name |
| 25 | `oya-foundry-quality-lane-kernel` → `oya-foundry-fitness-quality-lane-kernel` | S | 2 | **2** (`quality-lane`) | none — new compound row | path edge + `[lib]` name |
| 26 | `oya-foundry-raci-team-coverage-kernel` → `oya-foundry-fitness-raci-coverage-kernel` | S | 2 | **2** (`raci-coverage`) | none — new compound row | drop redundant `team`; capability `raci-coverage` (2 tokens) |
| 27 | `oya-foundry-readme-doc-coverage-kernel` → `oya-foundry-fitness-readme-coverage-kernel` | S | 2 | **2** (`readme-coverage`) | none — new compound row | drop redundant `doc`; capability `readme-coverage` (2 tokens) |
| 28 | `oya-foundry-release-evidence-pack-kernel` → `oya-foundry-fitness-release-pack-kernel` | M | 3 | **2** (`release-pack`) | none — new compound row | drop redundant `evidence` (foundry implies evidence per ADR-0003); cross-doc cites in `docs/RELEASE-MANAGEMENT.md`. Workflow file name stays `release-evidence-pack.yml` (CI external observable) |
| 29 | `oya-foundry-runbook-freshness-kernel` → `oya-foundry-fitness-runbook-freshness-kernel` | S | 2 | **2** (`runbook-freshness`) | none — new compound row | path edge + `[lib]` name |
| 30 | `oya-foundry-runbook-index-kernel` → `oya-foundry-fitness-runbook-index-kernel` | S | 2 | **2** (`runbook-index`) | none — new compound row | path edge + `[lib]` name |
| 31 | `oya-foundry-slo-coverage-kernel` → `oya-foundry-fitness-slo-coverage-kernel` | S | 2 | **2** (`slo-coverage`) | none — new compound row | path edge + `[lib]` name |
| 32 | `oya-foundry-supply-chain-kernel` → `oya-foundry-fitness-supply-chain-kernel` | M | 3 | **2** (`supply-chain`) | none — new compound row | path edge + `[lib]` name; cites in `docs/RELEASE-MANAGEMENT.md`, `.github/workflows/supply-chain.yml` |
| 33 | `oya-foundry-typescript-workspace-kernel` → `oya-foundry-fitness-typescript-workspace-kernel` | S | 2 | **2** (`typescript-workspace`) | none — new compound row | path edge + `[lib]` name |
| 34 | `oya-foundry-vendor-contract-recency-kernel` → `oya-foundry-fitness-vendor-recency-kernel` | S | 2 | **2** (`vendor-recency`) | none — new compound row | drop redundant `contract`; capability `vendor-recency` (2 tokens) |
| 35 | `oya-platform-data-boundary-kernel` → `oya-platform-fitness-data-boundary-kernel` | **L** | **5** | **2** (`data-boundary`) | **none — but row 35 docs/code co-edit precondition**: `docs/standards/clean-architecture.md §3` row "Depends only on data-boundary kernel + kernel peers; library-only" MUST be updated to reference the new name in the **same commit**. Per Architect condition 8. | feature=`fitness`, capability=`data-boundary` (2 tokens; new compound registry row). **95 manifest consumers** (grep-verified Codex iter-1); workspace-wide path edge audit MANDATORY. Risk re-baselined from v2's "4" to **5** — this is the largest blast radius row in the cohort. |
| 36 | `oya-tooling-agent-read` → `oya-tooling-agent-cli-read` | S | 1 | 1 (`read`) | none | insert `cli` role before capability `read`; feature=`agent`, role=`cli`, capability=`read`. Zero internal consumers verified |
| 37 | `oya-tooling-cli-dev-runtime` → `oya-tooling-dev-runtime` | **L** | **5** | 1 (none; `dev` is feature, `runtime` is role) | rows 1, 4, all of rows 2–34 (this crate's `Cargo.toml` is the dep-edge hub) | feature=`dev`, role=`runtime`; collapse redundant `cli` + `runtime`. **Ships lib + 2 bins** (`oya`, `repoctl` — Codex iter-1 evidence). **Touches** (EDIT-7; expanded after iter-2 verification): 3 CI workflow files, `scripts/check.sh` (29 sites), `scripts/hooks/pre-push-repoctl.sh`, `AGENTS.md`, `docs/CONSTITUTION.md`, `docs/TOOLCHAIN.md`, `docs/research/hyperscaler-best-practices-2026-05-12.md`, `docs/RELEASE-MANAGEMENT.md`, **plus the following test/source fixtures with hardcoded old-name references (Codex iter-2 #7, verified file:line)**: `crates/oya-tooling-cli-dev-runtime/tests/gate_cli.rs` (lines 2830, 2868, 2879, 3456 `ghcr.io/oyatie/oya-tooling-cli-dev-runtime@...` digest reference, 3465 `registry/release/supply-chain/oya-tooling-cli-dev-runtime.yaml` filename, 3471-3472 `oya-tooling-cli-dev-runtime.spdx.json` + `.cyclonedx.json` SBOM filenames), `crates/oya-tooling-cli-dev-runtime/tests/repoctl_cli.rs` (lines 149, 159 — `cargo run -p oya-tooling-cli-dev-runtime --bin repoctl ...`), `crates/oya-tooling-cli-dev-runtime/src/commands/repoctl.rs:43` (hardcoded `crates/oya-tooling-cli-dev-runtime/Cargo.toml` path default for `--cli-manifest`). **`[lib]` name** `oya_tooling_cli_dev_runtime` → `oya_tooling_dev_runtime`; **bin name decision**: binaries `oya` and `repoctl` remain UNCHANGED (bin name is independent of crate name; verified in `src/main.rs` + `Cargo.toml` `[[bin]]` tables). **Release artifact names**: SBOM and supply-chain evidence files in `registry/release/supply-chain/oya-tooling-cli-dev-runtime.{yaml,spdx.json,cyclonedx.json}` ARE crate-name-scoped and MUST be renamed in Shard 1 — these are not in `crates/` so the xtask alone will not touch them; row 37 explicitly lists them as additional touched files. Container image ref `ghcr.io/oyatie/oya-tooling-cli-dev-runtime` is also crate-name-scoped — confirm with Release Engineering whether the GHCR image gets renamed (default: yes, atomically with Shard 1) or retains the legacy name via image alias. |

**Compound-capability summary (Architect condition 7, Critic edit #10 audit; EDIT 10-finalisation)**: Of the 37 rows, **31 carry a 2-token compound capability** that requires an ADR-0055 cite + `[workspace.metadata.oya].compound_features` row. Six rows (1, 4, 11, 36, 37) carry a 1-token or no-tail name. Row 16 (`documentation-system`) is the only AMBER (6-segment) row; its ADR-0055 cite is the architect-flagged AMBER carve-out. **Per-compound rationale policy (closes Codex edit #10 APPROVE-WITH-CONDITION)**: "The 31 new 2-token capabilities are admitted as one taxonomy family under the `fitness` umbrella per Policy B; individual per-row rationale is provided ONLY for AMBER exceptions (row 16 `documentation-system`)." Single batch ADR-0055 cite is sufficient for the remaining 30 GREEN compounds; the registry table itself acts as the manifest. ADR-0055 §"Compound capability audit" enumerates all 31 by name and writes the AMBER rationale for row 16 individually.

## §3 — `[package.metadata.oya]` schema (aligned with `crate-naming-convention.md:266-272`)

### 3.1 Per-crate block (BNF, full)

```bnf
metadata-oya       ::= "[package.metadata.oya]" NL
                       "name        = " name-str NL
                       "context     = " context-str NL
                       "role        = " role-str NL
                       "feature     = " feature-str NL
                       "capability  = " (capability-str | empty-str) NL
                       "layer       = " layer-str NL
                       "audit_chain = " bool-str NL
name-str           ::= "\"oya-" context "-" feature "-" role ( "-" capability )? "\""
context-str        ::= "\"cloud\"" | "\"foundation\"" | "\"foundry\""
                     | "\"platform\"" | "\"tooling\"" | "\"workspace\""
role-str           ::= "\"kernel\"" | "\"domain\"" | "\"app\"" | "\"api\""
                     | "\"worker\"" | "\"adapter\"" | "\"runtime\""
                     | "\"cli\"" | "\"sdk\""
feature-str        ::= "\"" kebab-feature "\""             ; 1..3 kebab-tokens
capability-str     ::= "\"" kebab-capability "\""          ; 1..2 kebab-tokens
empty-str          ::= "\"\""                              ; capability absent
layer-str          ::= "\"kernel\"" | "\"domain\"" | "\"app\""
                     | "\"inbound-adapter\"" | "\"outbound-adapter\""
                     | "\"runtime\""                       ; ADR-0015 layer enum
bool-str           ::= "true" | "false"
```

**Citation**: This block matches `docs/standards/crate-naming-convention.md` lines 266-272 verbatim. v2 §3.1 omitted `name`, `feature`, `layer`, `audit_chain` — that was Critic edit #2. v3 fixes the omission. The standard is authoritative; the plan no longer drifts from it.

**Context-vs-feature decision for `fitness`** (Architect condition 1; v2 §10 question 5 — **resolved in v3, not deferred**): `fitness` is a **feature**, NOT a context. The context enum stays at six values (`cloud | foundation | foundry | platform | tooling | workspace`). Rationale: (a) `clean-architecture.md §3` layer-direction rules are anchored to the six-context enum; promoting `fitness` to a context would require re-rolling the whole layer-direction table — out of scope for this rename. (b) Every "fitness" crate sits within `foundry` context already (foundry = "the engineering platform itself: fitness lanes, ..." per naming-convention §3); making `fitness` a context would invert that. (c) Both reviewers' iter-1 outputs treat `fitness` as a feature. ADR-0055 records this decision in §"Context-vs-feature decision" and is approved BEFORE Shard 1 opens.

### 3.2 Workspace-level registry (root `Cargo.toml`)

Per `crate-naming-convention.md §7.1` (lines 280-301), root `Cargo.toml` grows a `[workspace.metadata.oya]` block:

```toml
[workspace.metadata.oya]
contexts = ["cloud", "foundation", "foundry", "platform", "tooling", "workspace"]
roles    = ["kernel", "domain", "app", "api", "worker", "adapter",
            "runtime", "cli", "sdk"]
layers   = ["kernel", "domain", "app", "inbound-adapter", "outbound-adapter", "runtime"]
compound_features = [
  # Pre-existing (preserved from v2 standard):
  "audit-chain", "policy-cedar", "object-graph",
  "regional-pack", "regulatory-pack",
  "compute-vm", "compute-k8s", "compute-functions",
  "storage-object", "storage-block",
  "network-vpc", "network-dns", "network-lb",
  "billing-tax", "address-book", "document-format",
  "trust-portal", "collab-runtime", "agent-read",
  "api-semver", "cargo-prefix", "cli-dev",
  "data-class",
  # New compound capabilities admitted by ADR-0055 (rows 2-34 cap-tok=2):
  "adr-citation", "adr-index",
  "authority-cohesion", "brand-residue", "claim-ceiling",
  "cloud-mutation", "codeowners-mirror", "constitution-cite",
  "cost-budget", "data-boundary",
  "doc-catalog", "documentation-system",
  "glossary-coverage", "glossary-vocabulary",
  "license-policy", "mcp-gateway", "mobile-native",
  "placeholder-debt", "pr-traceability", "pre-push",
  "quality-lane", "raci-coverage", "readme-coverage",
  "release-pack", "runbook-freshness", "runbook-index",
  "slo-coverage", "supply-chain",
  "typescript-workspace", "vendor-recency",
  # 3-token feature compounds for fitness-lane infrastructure crates
  # (Architect iter-3 residual #1 BNF-compliance fold; lane crates parse as
  # feature=<fitness-*> + role=kernel; capability slot stays empty per kernel
  # constraint per crate-naming-convention.md §4):
  "fitness-architecture-conventions",
  "fitness-freeze-window",
  "fitness-baseline-reset",
]
```

Adding to any list REQUIRES an ADR cite — for Shard 1, ADR-0055 is the cite (added in same commit). The lane parses this registry; refusing-class is `COMPOUND-UNREGISTERED` if any crate's name decomposes to a capability not in `compound_features`.

### 3.3 Helper implementation — `cargo xtask metadata-augment` (Shard 0 deliverable)

The xtask ships in **Shard 0** (pure tooling, no renames), runs in `--check` mode through Shard 0 acceptance, and is invoked in `--apply` mode exactly once during Shard 1 to populate the 140 blocks. Lives at `tools/xtask-metadata-augment/` until the cutover; rehomes to `crates/oya-tooling-dev-runtime/src/bin/xtask-metadata-augment.rs` as part of the Shard 1 atomic commit.

Flags:
- `--check` — exit non-zero if any block is missing/wrong; deterministic; used by lane.
- `--apply` — rewrite manifests; idempotent via `toml_edit`.
- `--shard <name>` — limit to one shard scope (Shard 0 = tools/xtask-metadata-augment alone; Shard 1 = full 140).
- `--registry-check` — cross-check every `[package.metadata.oya].capability` against `[workspace.metadata.oya].compound_features` (new in v3 per Critic edit #2).

#### 3.3.1 Dep-edge form × table-type rewrite matrix (EDIT-2; Codex iter-2 #2 + Architect residual #3)

The xtask MUST cover every combination of dep-edge form × manifest table. Shard 0 acceptance is GATED on `cargo nextest run -p xtask-metadata-augment --test fixtures` exiting 0 with a fixture for each cell of the following matrix:

**Tables** (4):
1. `[dependencies]`
2. `[dev-dependencies]`
3. `[build-dependencies]`
4. `[target.'cfg(*)'.dependencies]` (target-conditional; including `target.'cfg(unix)'`, `target.'cfg(target_os = "linux")'`, etc.)

**Forms** (5 per table):
1. Bare string version: `oya-foundry-fitness-x = "0.1"`
2. Inline table with `path = "../oya-foundry-fitness-x"`
3. Inline table with `workspace = true`
4. Inline table with `package = "..."` (rename idiom: `xname = { package = "oya-foundry-fitness-x", version = "0.1" }`)
5. Inline table with `optional = true` (combined with any of the above)

**Specification table** (each row spells out the `toml_edit` traversal pattern):

| # | Table | Form | toml_edit pattern | Fixture file |
|--:|---|---|---|---|
| 1 | `[dependencies]` | bare string | `manifest["dependencies"][old_name]` re-keyed to `new_name`; value preserved | `tests/fixtures/01-deps-bare-string.toml` |
| 2 | `[dependencies]` | inline path | re-key + rewrite `["path"]` value if it contains `old_name` segment | `tests/fixtures/02-deps-inline-path.toml` |
| 3 | `[dependencies]` | inline workspace | re-key; preserve `workspace = true` | `tests/fixtures/03-deps-inline-workspace.toml` |
| 4 | `[dependencies]` | inline package= | key stays (alias); rewrite the `package` string value | `tests/fixtures/04-deps-inline-package.toml` |
| 5 | `[dependencies]` | inline optional | combine pattern from rows 1-4 with `optional = true` preserved | `tests/fixtures/05-deps-inline-optional.toml` |
| 6-10 | `[dev-dependencies]` | (5 forms) | same patterns as 1-5, scoped to dev table | `tests/fixtures/06..10-*.toml` |
| 11-15 | `[build-dependencies]` | (5 forms) | same patterns | `tests/fixtures/11..15-*.toml` |
| 16-20 | `[target.'cfg(*)'.dependencies]` | (5 forms) | walk all `target.*.dependencies` tables; same per-form rewriter | `tests/fixtures/16..20-*.toml` |

Total: **20 fixture files × golden-file assertion**. Shard 0 acceptance gate `cargo nextest run -p xtask-metadata-augment --test fixtures` is REQUIRED; cannot enter Shard 1 with fewer than 20 passing cells.

Edge cases included as **negative fixtures** (must NOT rewrite): `[dependencies.unrelated-crate]` (name not in rename map), bare references to old names inside comments, references inside `[features]` arrays (these are feature names, not crate names — must NOT be rewritten).

## §4 — Cutover order: ATOMIC vs SHARDED vs HYBRID C — fair pricing

### 4.1 Re-priced Option A (single atomic PR, no precursor)

| Aspect | Atomic-PR honest price |
|---|---|
| Half-state period | zero |
| Rollback | clean `git revert <sha>` |
| **Reviewer load** | 4 hotspot crates (rows 1, 4, 35, 37) × ~3 reviewers each + 95-manifest scan for row 35 + 30 tooling-runtime files (row 37) = realistic **6–8 h per primary reviewer, ~3 reviewers in parallel = 18–24 h calendar reviewer-hours** (SOFT-EDIT disambiguation), not v2's "1–2h". v2 understated review cost. |
| CI runtime | one `cargo check --workspace --all-features` + `cargo nextest run --workspace` cycle (~25–35 min on workspace CI today) |
| Bisectability | poor (190 file changes in one commit) |
| In-flight branch conflict | maximal — every open PR hits at least one renamed file. **Single rebase window** for in-flight work. |
| **Lockfile churn** | **1 event** (single squash-merge) |
| Helper-risk | xtask landed in same commit ⇒ debugging the helper means scrolling through 190 file changes |

**Verdict**: Atomic-without-precursor stresses reviewer load and bundles tooling risk with rename risk. Rejected.

### 4.2 Re-priced Option B (6 sequential context-shards) — hidden costs

| Aspect | Sharded honest price |
|---|---|
| Half-state period | 5 intermediate states |
| **Rebase windows for in-flight branches** | **5 sequential 48 h freezes** ⇒ ~10 calendar days of in-flight branch rebase burden, not the "absorbed by 48 h freeze" framing of v2. Every shard merge forces every open feature branch to rebase across the renamed file set **again**. |
| **Lockfile churn** | **6 events**. Each shard merges its own `Cargo.lock` regen. Reviewer must verify that the lockfile delta in shard N does not contain any name from shard N+1 (because shard N+1 hasn't merged yet — those crate names still exist in their old form). This is an N-way ordering check that scales with shard count. |
| **Row 37 ordering contradiction** | v2 §:209 placed `oya-tooling-cli-dev-runtime` rename in Shard 1 (smallest blast radius first) but v2 §:100 precondition column says row 37 depends on rows 1, 4, 2–34 (which land in Shards 5, 5, 6). Critic edit #1 flagged this contradiction; v2 had no clean repair short of switching to Hybrid C. |
| Bisectability | per-shard granularity — useful only if a downstream regression maps to a single shard. For renames, regressions usually involve a downstream consumer hitting a freshly-renamed crate's `[lib]` name; the bisect points at the consumer, not the rename. Diminishing returns. |
| **Lane bootstrap chicken-and-egg** | the fitness lane crate validates the renames but lands in Shard 6 — lane runs `--report-only` through Shards 1–5. v2 R6 acknowledges this. Hybrid C eliminates the chicken-and-egg entirely (lane lands in Shard 1 alongside its data). |
| Calendar cost | ~2 weeks shard 1 → shard 6 merge |
| Reviewer load | per-shard ~10–50 file changes; well within cap, BUT total review burden across 6 PRs ≈ 12–18 h (more than atomic when summed) |

**Verdict**: Sharded distributes reviewer load over calendar time but multiplies lockfile-churn and ordering-contradiction costs. Both Architect (condition 9) and Critic (edit #1) listed "switch to Hybrid C" as their #1 revision. **Rejected**.

### 4.3 Hybrid C — Shard 0 (pure tooling) + Shard 1 (atomic 37-rename)

| Aspect | Hybrid C price |
|---|---|
| **Shard 0**: pure-tooling precursor, no renames | Lands: `xtask-metadata-augment` (Rust); ADR-0055 draft + ADR-0054 amendment commit (same commit); fitness lane crate scaffold (`oya-foundry-fitness-architecture-conventions-kernel`, empty but registered); `[workspace.metadata.oya]` registry block (with `compound_features` already containing all 31 new compounds); context-vs-feature decision recorded; ICM rationale-row template + sanctioned-primitives audit completed |
| **Shard 0 effort** | 4–6 executor-hours; 1 reviewer-hour |
| **Shard 0 risk** | LOW — no renames, no rev-deps touched; xtask is workspace-internal; failing Shard 0 has zero blast radius |
| **Shard 1**: atomic 37-rename + 140 metadata + 44 dep-edges + CI cutover + lockfile-regen + clean-architecture.md §3 co-edit + registry refs co-edit | Single squash-merged PR; gated by Shard 0 sign-off and §8 deterministic acceptance gates |
| **Shard 1 effort** | 8–12 executor-hours (xtask --apply + lockfile + verification); **6–8 h per primary reviewer × ~3 reviewers parallel = 18–24 h calendar reviewer-hours** (honest atomic-review pricing); **mandatory squash-merge** |
| **Half-state period** | zero |
| **Rollback** | single `git revert <sha>` of Shard 1; Shard 0 stays merged (no harm — xtask + scaffolded-empty lane are harmless idle artifacts) |
| **In-flight branch conflict** | **single 48 h freeze window** (vs. 5 sequential windows for Option B); every open PR rebases once |
| **Lockfile churn** | **1 event** (Shard 1 squash) |
| **Reviewer cognitive load** | Shard 0 = 1 helper PR + 1 ADR PR (or combined); Shard 1 = 1 mechanical atomic PR where every change is xtask-derived — reviewers verify the xtask spec once + spot-check 4 hotspots (rows 1, 4, 35, 37). Honest budget: **6–8 h per primary reviewer × ~3 reviewers parallel = 18–24 h calendar reviewer-hours**. |
| **Lane bootstrap** | Shard 0 scaffolds the lane crate empty; Shard 1 populates it and flips to BLOCKER in the same commit. No chicken-and-egg. |
| **Calendar cost** | ~3–5 days (Shard 0 → Shard 1) |
| **Bisectability** | per-row bisectability NOT preserved (atomic), BUT mitigated by: (a) xtask is the rewriter — bugs in the rename pattern manifest as a class of failures, not a specific row; (b) `cargo metadata --no-deps` diff against pre-Shard-1 snapshot pinpoints the dep-edge regression; (c) Shard 0 separates tooling-bug surface from rename-bug surface. |

### 4.4 Decision: **Hybrid C**

Both reviewers' iter-1 #1 edit. The repricing math reverses v2's Option B preference: when Option A's review cost is priced honestly (6–8 h, not 1–2 h) and Option B's hidden lockfile/rebase costs are surfaced (6 lockfile events, 5 rebase windows), Hybrid C dominates on every aspect except per-row bisectability — which is mitigated by Shard 0's tooling/rename separation. v3 commits to Hybrid C.

## §5 — Per-shard checklist (Hybrid C)

Every checklist item is a **runnable command** with an expected exit code. The lane consumes the same commands. No item reads "reviewer signs off" or "doc-history hits" — those are now §8 deterministic allowlists.

### 5.0 Sanctioned-primitives ICM rationale (Architect condition 3 + Critic edit #3)

**Every non-sanctioned `git` / `gh` invocation in Shard 0 OR Shard 1 scope MUST have an ICM rationale row.** Per `docs/standards/git-workflow.md §2-3`, the sanctioned-primitive triad is grit / icm / `oya-tooling-agent-read`. Rename-cutover work touches `gh` for PR creation and `git` for branch ops outside the triad; v3 explicitly covers them under the cutover-bootstrap-window exception (`git-workflow.md §3`):

```sh
# Once per cutover session, log the bootstrap rationale (per git-workflow.md §3):
icm store \
  -t direct-tool-invocations \
  -c "rename-cutover-v3 bootstrap session (Shard 0 + Shard 1); covers gh pr create/merge, git branch ops; sanctioned-primitive gap rationale" \
  -i critical \
  -k "cutover,bootstrap,rename-v3"
```

The lane `oya-foundry-fitness-banned-primitives` PASSES when this row exists for the active session. Without it, any direct `git`/`gh` invocation FAILS the lane. **Item 0 of every shard checklist is this rationale store**.

**ADR exception for `gh pr merge --admin` (EDIT 3-finalisation; Codex iter-2 #3 sanctioned-primitives partial)**: The emergency-revert lane (§7.2) invokes `gh pr merge --admin`, normally forbidden per `git-workflow.md §10 Item 5`. This exception is admitted **only** when **ALL THREE** preconditions hold simultaneously: (1) `freeze_active == true` on the `oya-foundry-fitness-freeze-window-kernel` lane at invocation time; (2) the operator possesses an `expedite_override_token` minted by the **Security Council** (not council-architecture or axis-foundry — Security Council specifically, per R10 authority chain); (3) BEFORE the `gh pr merge --admin` invocation, the operator has logged: `icm store -t direct-tool-invocations -c "emergency-merge-shard1-revert: <reason>; freeze-active=true; security-council-token=<token-hash>" -i critical`. The ADR-0055 §"Rollback/expedite protocol" MUST cite this exception by name and enumerate the three preconditions verbatim; any `gh pr merge --admin` invocation without all three preconditions plus the ICM row remains a banned-primitives violation and FAILS the post-merge lane sweep.

### 5.1 Shard 0 checklist (pure tooling, no renames)

| # | Command | Expected exit | Verification |
|---:|---|:---:|---|
| 0 | `icm store -t direct-tool-invocations -c "rename-cutover-v3 bootstrap session" -i critical -k "cutover,bootstrap,rename-v3"` | 0 | Mandatory before any git/gh op |
| 1 | `cargo new --lib tools/xtask-metadata-augment` + author body | n/a | Helper authoring; `tools/xtask-metadata-augment/Cargo.toml`, `src/main.rs` exist |
| 1b | Extend `tools/xtask-metadata-augment` with the `lockfile-rename` subcommand + `lockfile_rename_fixtures.rs` integration tests per spec in §7.1.1 (Architect iter-3 residual #3; reuses the same crate scaffold-claim ICM row as step 1) | n/a | `tools/xtask-metadata-augment/src/` carries a `lockfile_rename` module wired into the binary's subcommand dispatch + `tools/xtask-metadata-augment/tests/lockfile_rename_fixtures.rs` (Rust integration tests, 8-row matrix per §7.1.1) exist on disk |
| 2 | `cargo build -p xtask-metadata-augment` | 0 | Helper compiles |
| 3 | `cargo nextest run -p xtask-metadata-augment` | 0 | Helper unit tests pass (parse, derive, idempotency) |
| 3a | `cargo nextest run -p xtask-metadata-augment --test fixtures` (EDIT-2; the 9-form × 4-table × 5-shape matrix per §3.3.1; 20 cells + negative fixtures) | 0 | REQUIRED Shard 0 acceptance gate; cannot enter Shard 1 with fewer than 20 passing fixture cells |
| 3b | `cargo nextest run -p xtask-metadata-augment --test lockfile_rename_fixtures` (Architect iter-3 residual #3 + Critic iter-3 condition 1) | 0 | REQUIRED Shard 0 acceptance gate; all **8 fixture rows** pass: (1) workspace-member rename, (2) workspace-member rename with dependents, (3) external package unchanged, (4) quoted name in deps array, (5) unquoted name in deps array, (6) missing-rename-map-entry no-op + warning, (7) dep entry with version disambiguator (`"old-name 0.1.0"` → `"new-name 0.1.0"`), (8) dep entry with version+source disambiguator (`"old-name 0.1.0 (registry+...)"` → `"new-name 0.1.0 (registry+...)"`) |
| 4 | `cargo run -p xtask-metadata-augment -- --check --shard tools-xtask-metadata-augment` | 0 | Helper self-check |
| 5 | Author ADR-0055 + ADR-0054 amendment in **same commit** | n/a | Single commit hash contains both `docs/decisions/ADR-0055-*.md` and `docs/decisions/ADR-0054-*.md` (amendment) |
| 6 | `git log -1 --name-only HEAD \| grep -E "ADR-005[45]"` | 0 (both files in diff) | Architect condition 2 + Critic edit #4 |
| 7 | Scaffold empty fitness lane crate `oya-foundry-fitness-architecture-conventions-kernel` | n/a | `crates/oya-foundry-fitness-architecture-conventions-kernel/` with empty lib + Cargo.toml |
| 7a | Scaffold empty fitness lane crate `oya-foundry-fitness-freeze-window-kernel` (EDIT-5; freeze-enforcement primitive per §6 R2) | n/a | `crates/oya-foundry-fitness-freeze-window-kernel/` with empty lib + Cargo.toml; lane config schema (freeze_active, freeze_end_ts, expedite_override_token) documented in crate README |
| 7b | Scaffold empty fitness lane crate `oya-foundry-fitness-baseline-reset-kernel` (EDIT-4; BASELINE-RESET classifier per §8.1 semver gate) | n/a | `crates/oya-foundry-fitness-baseline-reset-kernel/` with empty lib + Cargo.toml; JSON-post-processor stub documented in crate README |
| 7c | **ICM JSONL round-trip contract test (Critic iter-3 condition 3b)**. Verifies lane-runtime can deterministically parse PR-bound token payloads from `icm recall --format jsonl` output: <pre>TEST_TOKEN_UUID=$(uuidgen)<br/>TEST_PR=999999  # synthetic test PR number<br/>icm store -t lane-config-oyatie-test -c "freeze_window:expedite_token=${TEST_TOKEN_UUID};pr=${TEST_PR}" -i critical -k "lane=test,pr=${TEST_PR}"<br/>RECOVERED=$(icm recall -t lane-config-oyatie-test -k "lane=test,pr=${TEST_PR}" --format jsonl \| jq -r '.[] \| select(.content \| contains("freeze_window:expedite_token=")) \| .content' \| sed -E 's/.*expedite_token=([^;]+);pr=([0-9]+).*/\1,\2/')<br/>EXPECTED="${TEST_TOKEN_UUID},${TEST_PR}"<br/>test "${RECOVERED}" = "${EXPECTED}" \|\| { echo "ICM JSONL round-trip FAIL: expected=${EXPECTED} recovered=${RECOVERED}"; exit 1; }<br/># Cleanup<br/>icm store -t lane-config-oyatie-test -c "freeze_window:expedite_token=REVOKED;pr=${TEST_PR}" -i critical -k "lane=test,pr=${TEST_PR}"</pre> | 0 | REQUIRED Shard 0 acceptance gate; round-trip MUST exit 0 before Shard 1 opens. Anchors the lane-runtime's `icm recall \| jq` parse contract against the pinned `icm` version (Critic iter-3 condition 3a). |
| 8 | `cargo check --workspace --all-features` | 0 | Workspace still builds with empty new crate |
| 9 | Add `[workspace.metadata.oya]` block to root `Cargo.toml` (compound_features registry) | n/a | §3.2 block present, all 31 new compounds enumerated |
| 10 | `cargo run -p xtask-metadata-augment -- --registry-check` | 0 | Registry block parses and matches the 31 enumerated compounds |
| 11 | Record context-vs-feature decision in ADR-0055 §"Context-vs-feature decision": `fitness` is a feature, not a context | n/a | ADR-0055 contains the decision rationale |
| 12 | `rg "\\[workspace\\.metadata\\.oya\\]" Cargo.toml` | 0, 1 line | Single registry row present |
| 13 | `cargo deny check` | 0 | Existing licenses section still passes; no new bans introduced |
| 14 | `cargo doc --workspace --no-deps` | 0 | Doc build is green (Architect condition 4) |
| 15 | `cargo metadata --no-deps --format-version 1 \| jq -r '.workspace_members[]' \| sort > /tmp/shard0-metadata.txt` | 0 | Snapshot stored for Shard 1 path-edge diff |
| 15a | Generate `/tmp/old-crate-names.txt` (one old-name per line, from §2 rename inventory): `awk -F' → ' '/^\| *[0-9]+ \|/ {gsub(/^[ `]+\|[ `]+/,"",$1); gsub(/[ `]+$/,"",$1); print $1}' docs/plans/rename-plan-v3-2026-05-12.md \| sort -u > /tmp/old-crate-names.txt` and `/tmp/rename-map.tsv` (old<TAB>new from same inventory): `awk -F' → ' '/^\| *[0-9]+ \|/ {old=$1; new=$2; gsub(/.*`/,"",old); gsub(/`.*/,"",old); gsub(/.*`/,"",new); gsub(/`.*/,"",new); print old"\t"new}' docs/plans/rename-plan-v3-2026-05-12.md > /tmp/rename-map.tsv` (Architect iter-3 residual #3 generator de-forward-reference) | 0 | `wc -l /tmp/old-crate-names.txt` == 37; `wc -l /tmp/rename-map.tsv` == 37; consumed by Shard 1 step 12b + §8.1 zero-old-names gates |

### 5.2 Shard 1 checklist (atomic 37-rename + everything else)

Mandatory **squash-merge**. Single commit on `main`.

| # | Command | Expected exit | Verification |
|---:|---|:---:|---|
| 0 | `icm store -t direct-tool-invocations -c "rename-cutover-v3 Shard 1 session" -i critical -k "cutover,shard-1,rename-v3"` | 0 | Cutover-bootstrap rationale (renew per §5.0) |
| 1 | For each of 37 renames in §2, ICM scaffold-claim row per ADR-0054 (now amended): `icm store -t scaffold-locks-oyatie -c "agent=<id> path=crates/<new-name> window=open intent='rename per ADR-0055'" -i critical` | 0 (× 37) | ADR-0054 amendment authorises rename-events as scaffold-claim triggers |
| 2 | Update root `Cargo.toml` `[workspace] members = [...]` array: `crates/oya-<old>` → `crates/oya-<new>` × 37 | n/a | All 37 entries updated atomically |
| 3 | `git mv crates/oya-<old> crates/oya-<new>` × 37 | 0 each | Directory renames |
| 4 | `cargo run -p xtask-metadata-augment -- --apply` | 0 | All 140 manifests now carry `[package.metadata.oya]` per §3.1 |
| 5 | For each renamed crate: rewrite `[package] name` AND `[lib] name = "..."` (underscored form) | n/a | §6 R7 permanent control |
| 6 | Rewrite 44 dep-edge `path = "../oya-<old>"` entries in `oya-tooling-dev-runtime/Cargo.toml` (and 2 other consumer crates) | n/a | Workspace-wide grep verification in step 13 |
| 7 | Update 3 CI workflow files (`.github/workflows/release-evidence-pack.yml`, `.github/workflows/supply-chain.yml` × 2 sites) | n/a | `cargo run -p oya-tooling-cli-dev-runtime` → `cargo run -p oya-tooling-dev-runtime` |
| 8 | Update `scripts/check.sh` (29 sites), `scripts/hooks/pre-push-repoctl.sh` (1 site), `scripts/check-architecture-boundaries.sh` (3 sites) | n/a | Verified in step 13 grep |
| 9 | Update `docs/standards/clean-architecture.md §3` row 35 named-by-identity reference (Architect condition 8) | n/a | `oya-platform-data-boundary-kernel` → `oya-platform-fitness-data-boundary-kernel` in clean-architecture §3 row |
| 10 | Update `docs/standards/crate-naming-convention.md §6` compound features table | n/a | Reflects new 31 compounds |
| 11 | Update `registry/quality/lanes.yaml`, `registry/docs/pipeline.tsv`, registry OpenAPI bindings (Critic edit #9) | n/a | All rename references flipped |
| 12a | Snapshot pre-rename metadata: `cargo metadata --locked --format-version 1 > /tmp/cargo-metadata-pre-rename.json` | 0 | Captures the baseline that the §8.1 lockfile-diff gate compares against |
| 12b | Scripted old→new workspace-name rewrite of `Cargo.lock`: `cargo run --release -p xtask-metadata-augment -- lockfile-rename --rename-map /tmp/rename-map.tsv --lockfile Cargo.lock --inplace` (deterministic `toml_edit::DocumentMut`-based pass per §7.1.1 spec; rewrites only `[[package]] name` for workspace members + `dependencies` array entries; refuses any other edit). Map file from Shard 0 step 15a (37 rows). | 0 | Single deterministic rewrite, no resolver invocation; behaviour verified by Shard 0 step 3b `cargo nextest` gate |
| 12c | `cargo check --workspace --locked --offline` | 0 | `--locked` refuses any non-name-delta change to Cargo.lock; this is the load-bearing safety gate. If exit ≠ 0, the rewrite produced a non-name delta and Shard 1 aborts |
| 13 | All §8.1 deterministic acceptance gates (run as a script; full list below) | 0 (all) | This is the merge-gate |
| 14 | Close ICM scaffold-claim windows: `icm store -t scaffold-locks-oyatie -c "agent=<id> path=crates/<new-name> window=closed" -i high` × 37 | 0 each | ADR-0054 amendment compliance |
| 15 | Flip lane crate `oya-foundry-fitness-architecture-conventions-kernel` from `--report-only` to BLOCKER | n/a | Lane spec `severity` flips in same commit |
| 16 | Flip ADR-0055 status `Proposed → Accepted` in same commit | n/a | ADR header status field |

## §6 — Risk cone (R1–R10, expanded)

| Risk | Likelihood | Impact | Mitigation |
|---|:---:|:---:|---|
| **R1 — External repos break.** | L | M | Pre-flight `gh search code` across Oyatie org; all 140 crates `publish = false` (verified `rtk grep -c "publish = false"`); path imports from external repos use `[patch]` overrides listed in Shard 0 prep. |
| **R2 — In-flight feature branches conflict.** | M | M | Single 48 h freeze (vs. 5 for Option B); shard sequence is Shard 0 → Shard 1 atomic; `git rerere` enabled on merge queue. **Freeze-enforcement primitive (EDIT-5; Architect residual #1 + Codex iter-2 #5)**: new fitness lane `oya-foundry-fitness-freeze-window-kernel` (scaffolded in Shard 0, populated in Shard 1). Lane semantics: holds config `freeze_active: bool`, `freeze_end_ts: timestamp` (RFC3339), `expedite_override_token: string?` (nullable). Merge-queue invokes the lane on every dequeue; the lane FAILS (blocking merge to `main`) unless **`freeze_active == false` OR the PR carries a header `X-Rename-Expedite-Token: <token>` matching the lane's current `expedite_override_token`**. Security expedite override (R10) mints the token via `icm store` as the write primitive (chosen per Architect iter-3 residual #2 OPTION A: routes the write through the sanctioned `icm` primitive triad-member rather than overloading the READ-named `oya-tooling-agent-read` with write semantics, preserving the sanctioned-primitive triad in `docs/standards/git-workflow.md §1`). **PR-bound token payload (Critic iter-3 condition 2)**: tokens are bound to a specific PR number to eliminate the cross-PR rotation race surfaced in iter-3 soft-condition. Payload format is `expedite_token=<uuid>;pr=<n>` where `<n>` is the PR number being expedited. Mint command (Security Council): `icm store -t lane-config-oyatie -c "freeze_window:expedite_token=$(uuidgen);pr=${PR_NUM}" -i critical -k "lane=oya-foundry-fitness-freeze-window-kernel,pr=${PR_NUM}"`. The same operation auto-satisfies the Directive 12 rationale-row requirement because `icm store` IS the sanctioned primitive — no additional `direct-tool-invocations` row is required for the mint. **Lane runtime check (Critic iter-3 condition 2)**: on merge-queue dequeue the lane reads the token via `icm recall -t lane-config-oyatie -k "lane=oya-foundry-fitness-freeze-window-kernel,pr=${PR_NUM}" --format jsonl \| jq` and accepts merge ONLY when BOTH (a) the header token UUID matches the stored UUID AND (b) the requesting PR number `${PR_NUM}` matches the `pr=` field in the token payload. Mismatch on either condition FAILS merge. **Rotation policy**: token expires after **single use** — after the matching PR merges, the lane writes a tombstone row `icm store -t lane-config-oyatie -c "freeze_window:expedite_token=REVOKED;pr=${PR_NUM}" -i critical -k "lane=oya-foundry-fitness-freeze-window-kernel,pr=${PR_NUM}"` so subsequent `icm recall` queries return the REVOKED sentinel for that PR (latest-row-wins per-PR; cross-PR tokens are now independent because the keys carry distinct `pr=` values); a fresh mint produces a new uuidgen value and a new row bound to its own PR number. The freeze window itself is scheduled in Shard 1's PR description (`freeze_active=true` flipped 48 h pre-merge; `freeze_end_ts` set to Shard 1 merge timestamp + 4 h cooldown). |
| **R3 — `cargo-deny` rules referencing old crate names.** | L | L | `deny.toml` audited (205 bytes, `[licenses]` only — no `[bans]` rules referencing crate names); re-run audit in Shard 0 prep. **Codex iter-1 note**: clean-architecture.md says future bans generate from `[package.metadata.oya].role` — that generator is OUT OF SCOPE for v3 (deferred to a separate post-Shard-1 ADR); schema mismatch is now harmless because no generator runs against deny.toml in v3 scope. |
| **R4 — Hidden indirect deps via workspace edition inheritance.** | L | M | `cargo metadata --no-deps` diff in §8.1 gate; lockfile regen happens **once** in Shard 1 step 12. |
| **R5 — Crates.io publish collision.** | L | L | All 140 `publish = false` re-verified in Shard 0. |
| **R6 — Lane bootstrap chicken-and-egg.** | L (was M) | M | **Hybrid C eliminates this** — Shard 0 scaffolds the lane crate empty; Shard 1 populates and flips to BLOCKER in the same commit. |
| **R7 — `[lib]` name drift on rename.** Promoted to **permanent-controls ledger** (Architect condition 6 — all 5 layers explicit). | M | H | **(1) Preflight**: xtask emits a checklist of every `[lib] name = "..."` declaration in the cohort. **(2) Ledger**: MISTAKES-LEDGER row `MFL-LIBNAME` created in Shard 0 with class `mechanical`. **(3) Lane**: `oya-foundry-fitness-architecture-conventions-kernel` includes a `[lib]`-vs-`[package]`-name parity check; fails if any explicit `[lib] name` disagrees with `[package] name`'s underscored form. **(4) ICM**: rename-cutover ICM topic `decisions-oyatie-rename-v2` carries a `lib-name-drift-control` row referencing this risk and its 5-layer fix. **(5) Citation probe**: §8.1 gate `cargo doc --workspace --no-deps` exercises every `[lib]` name; mismatch surfaces as docgen failure with non-zero exit. |
| **R8 — rust-analyzer cache recovery (NEW).** Post-rename, rust-analyzer's project model goes stale; agents/editors see "unresolved import" until cache regen. | M | M | Shard 1 PR description includes runbook: `cargo clean -p <renamed-crates>` is NOT required (Cargo handles target reflow); `rust-analyzer: Restart server` from each editor's command palette is sufficient. CI runners are ephemeral — no cache to clear. ICM rationale row + runbook link recorded for first 14-day post-merge window. |
| **R9 — `cargo-semver-checks` baseline strategy (NEW).** Rename = breaking change at the package-name level; cargo-semver-checks compares against the previously-published crate by name. With rename, the previous baseline is at the old name; the new name has no baseline. | M | M | **Strategy**: rename PRs **reset the semver baseline** — Shard 1 commits `--baseline-rev <pre-shard-1-sha>` snapshots and re-publishes baselines under new names on first post-merge run. Documented in ADR-0055. Lane `oya-foundry-fitness-api-semver-kernel` (renamed from `oya-foundry-api-semver-kernel` per row 5) recognises the baseline-reset and emits an INFO row, not a FAIL. Operational: 14-day post-merge grace where any semver-checks failure on a renamed crate is auto-classified `BASELINE-RESET`. |
| **R10 — Security-P0 expedite lane / freeze-break protocol (NEW; Architect condition 5).** A security-critical hotfix lands during the 48 h freeze and needs main. | L | H | **Pre-authorised expedite lane**: any P0/P1 security ticket can break the freeze with a single approving security-council member's ICM stamp. The expedite commit ships against pre-Shard-1 main; the rename Shard 1 PR rebases through `git rerere` + the xtask-driven rewrite (xtask is idempotent — reapply against rebased base). ICM topic `security-expedite-rename-v3` carries the audit trail. |

## §7 — Rollback plan (with pre-authorised emergency revert lane)

### 7.1 Per-shard rollback (Hybrid C)

**Shard 0**: rarely rollback-worthy (no renames). If it must, `git revert <shard-0-sha>` removes xtask, ADR-0055, ADR-0054 amendment, lane scaffold, and registry block. Run `cargo check --workspace`; expected exit 0 (workspace was green before Shard 0).

**Shard 1**: `git revert <shard-1-sha>` restores all 37 directory names, member list, dep-edges, CI/scripts, doc references, and Cargo.lock (single commit ⇒ single revert). Then run §8.1 gates against pre-Shard-1 state (all should be 0). Lockfile regen if needed: apply the inverse rename map (`cargo run --release -p xtask-metadata-augment -- lockfile-rename --rename-map /tmp/rename-map.tsv --lockfile Cargo.lock --inplace --reverse`) then `cargo check --workspace --locked --offline`; refuses any non-name delta on the revert path too.

### 7.1.1 `tools/xtask-metadata-augment` `lockfile-rename` subcommand specification (Rust, in-workspace) (Architect iter-3 residual #3 — de-forward-reference)

The deterministic name-rewrite subcommand referenced by §5.2 step 12b and §7.1 above. Authored in Shard 0 step 1b (as a subcommand on the existing in-workspace `tools/xtask-metadata-augment` crate — Rust, no separate language toolchain); integration tests run in Shard 0 step 3b as a REQUIRED acceptance gate (`cargo nextest run -p xtask-metadata-augment --test lockfile_rename_fixtures` exit 0).

**Crate path**: `tools/xtask-metadata-augment` (Rust, in-workspace tooling crate; the `lockfile-rename` subcommand is added to the existing CLI dispatch). Parser is `toml_edit::DocumentMut`, already required by the metadata-augment xtask's other subcommands — consolidating to this crate eliminates a cross-language toolchain pin, a second non-Rust test runner, and a divergent TOML parser (`tomllib`/`tomli_w`) that would otherwise be introduced solely for this script.

**CLI**:
```
cargo run -p xtask-metadata-augment -- lockfile-rename \
  --rename-map /tmp/rename-map.tsv \
  --lockfile Cargo.lock \
  --inplace \
  [--reverse] \
  [--dry-run]
```

**Inputs**:
- `--rename-map`: TSV file, one `<old-name>\t<new-name>` row per workspace member rename (37 rows for v3 Shard 1; generated by Shard 0 step 15a).
- `--lockfile`: path to `Cargo.lock` (default: `./Cargo.lock`).
- `--inplace`: rewrite the lockfile in place; otherwise emit to stdout.
- `--reverse`: invert the map (new→old) for revert-path use per §7.1 Shard 1 rollback.
- `--dry-run`: print the diff (computed against the current lockfile bytes) and exit 0 without writing; mutually compatible with `--reverse` (reverse-direction dry-run for rollback rehearsals).

**Behaviour**:
1. Parse `Cargo.lock` with `toml_edit::DocumentMut` (already a dependency of the metadata-augment xtask — verified during Shard 0 step 1b authoring); for each `[[package]]` entry whose `name` field matches an old-name key in the rename map, replace `name` with the corresponding new-name. `toml_edit` preserves formatting and comments, satisfying the lossless-rewrite requirement.
2. For each `[[package]]` entry, walk its `dependencies` array; for any entry whose unquoted name token matches an old-name key, replace the name token with new-name. Preserve quoting style (quoted strings stay quoted; bare names stay bare). Preserve `version` and `source` suffixes within dep entries (e.g. `"foo 0.1.0 (registry+...)"` → `"foo-renamed 0.1.0 (registry+...)"`). **Disambiguator-preservation invariant (Critic iter-3 condition 1)**: The subcommand splits each dependency-array entry on the first whitespace; only the leading name token is rewritten; version and source-disambiguator suffixes are preserved character-for-character.
3. Do NOT touch: `version`, `source`, `checksum`, or any non-rename field on any package entry. Do NOT touch package entries whose `name` does not appear in the rename map (this is the "external package unchanged" invariant — external crates that happen to have the same name as an old workspace member but carry a `source` field are external and must not be renamed; the rule is: only rename packages whose `source` field is absent OR whose `name` is a workspace-member old name in the map).
4. Missing rename-map entry for a referenced name → no-op + emit warning to stderr; exit 0 (warning-only, not failure — caller verifies via downstream `cargo check --locked` gate).

**Integration-test matrix** (mandatory; run in Shard 0 acceptance gate via `cargo nextest run -p xtask-metadata-augment --test lockfile_rename_fixtures`; test file `tools/xtask-metadata-augment/tests/lockfile_rename_fixtures.rs`):

| # | Fixture | Asserts |
|--:|---|---|
| 1 | workspace-member package rename (no `source` field) | `name` rewritten; `version` preserved |
| 2 | workspace-member package rename with dependents that reference the renamed name | both the renamed `[[package]]` AND every dependent's `dependencies` array entry rewritten |
| 3 | external package unchanged (presence of `name` matching old-name in dep array but NOT a top-level package — skip) | external `[[package]]` with `source` field untouched; only workspace-member-old-name occurrences rewritten |
| 4 | quoted name in `dependencies` array | quote style preserved; rewrite applied |
| 5 | unquoted name in `dependencies` array | bare-name style preserved; rewrite applied |
| 6 | missing rename-map entry → no-op + warning to stderr | exit 0; stderr contains `WARN`; lockfile unchanged for unmatched names |
| 7 | dependency entry with version disambiguator (Critic iter-3 condition 1) | input form `"old-name 0.1.0"` → output `"new-name 0.1.0"` (only leading name token rewritten; version suffix `0.1.0` preserved verbatim) |
| 8 | dependency entry with version+source disambiguator (Critic iter-3 condition 1) | input form `"old-name 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)"` → output `"new-name 0.1.0 (registry+https://github.com/rust-lang/crates.io-index)"` (only leading name token rewritten; version AND source-disambiguator suffix preserved character-for-character) |

**Shard 0 acceptance gate** (added to §8.1; see new gate row "Lockfile-rename subcommand integration tests"): `cargo nextest run -p xtask-metadata-augment --test lockfile_rename_fixtures` exit 0 is REQUIRED before Shard 1 opens. Cannot enter Shard 1 with any of the **8 fixture rows** failing (6 original + 2 Critic-iter-3 disambiguator rows).

### 7.2 Pre-authorised emergency revert lane (NEW per Architect condition 5)

If Shard 1's main-branch state is broken AND a fix-forward would take >2 h, the **emergency revert lane** activates:

1. Any council-architecture or axis-foundry member can self-approve `git revert <shard-1-sha>` (no peer review required).
2. The revert PR uses `gh pr merge --admin` (normally forbidden per `git-workflow.md §10` Item 5) under the pre-authorised exception captured in ADR-0055 §"Rollback/expedite protocol".
3. CI bypass: the revert PR runs only `cargo check --workspace --all-features` (single command, ~3 min); full §8.1 gate suite is **bypassed** to restore trunk fast. Full suite re-runs post-merge as a non-blocking observability sweep.
4. ICM trail: `icm store -t direct-tool-invocations -c "EMERGENCY revert of Shard 1 via admin-merge, rationale: <reason>" -i critical` MANDATORY before the merge.

**Staging-promotion fallback (SOFT-EDIT; explicit since Shard 1 may already have begun promotion before revert is called)**: if Shard 1 was already **promoted to staging** before the emergency revert fires (i.e., the Shard 1 commit reached staging while main is being reverted), the revert PR ALSO blocks the next staging promotion cycle until the next stable Shard-1-equivalent commit is prepared. Operationally: (a) the revert PR title prefixes `REVERT-STAGING-BLOCK:` so release-engineering's staging-promotion gate recognises it; (b) the staging-promotion lane (`oya-foundry-fitness-staging-promotion`, future) reads this prefix and refuses promotion until a follow-up `STAGING-UNBLOCK:` commit on main; (c) post-revert observability sweep is BLOCKING in this case (vs. non-blocking for the normal revert path) because staging consumers must not pick up a renamed-then-reverted intermediate state.

### 7.3 Rollback time budget

| Path | Wall-clock |
|---|---|
| Shard 0 revert (rare) | < 15 min |
| Shard 1 revert (standard, full gate) | < 60 min |
| Shard 1 revert (**emergency lane**, CI bypass) | **< 15 min** |
| Post-emergency observability sweep | < 30 min (non-blocking) |

## §8 — Acceptance gate (per shard + global) — deterministic allowlist only

Every gate is a runnable command + expected exit code. No human judgement.

### 8.1 Shard-level gates

For each shard, ALL of the following commands MUST exit 0 on the shard's merge-candidate commit:

| Gate | Command | Expected exit |
|---|---|:---:|
| Workspace compiles | `cargo check --workspace --all-features` | 0 |
| Workspace builds | `cargo build --workspace --all-features` | 0 |
| Clippy clean | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| Cargo-deny clean | `cargo deny check` | 0 |
| Docs build | `cargo doc --workspace --no-deps` | 0 |
| Path-edge diff (Architect condition 4) | `cargo metadata --no-deps --format-version 1 \| jq -S '.packages[] \| {name, manifest_path}' > /tmp/post.txt && diff /tmp/shard0-metadata.txt /tmp/post.txt` | exit 1 with diff for Shard 1; exit 0 means nothing renamed (FAIL) |
| Tests pass | `cargo nextest run --workspace --all-features --no-fail-fast --message-format libtest-json > target/nextest/run.json && cargo nextest run --workspace --all-features --no-fail-fast --message-format junit > target/nextest/junit.xml` (per `testing.md:89`) | 0 |
| Semver-checks (R9 strategy; EDIT-4 pinned) | **Tool pin**: `cargo-semver-checks 0.46.0` (or current verified release; pinned in `tools/toolchain-versions.toml` and re-asserted by Shard 0 step 13 audit) **and `icm` version pinned via `tools/toolchain-versions.toml`** (Critic iter-3 condition 3a; pin row `icm = "<current verified release>"` added to the same toolchain file alongside the `cargo-semver-checks` pin during Shard 0 step 1b; ensures the lane-runtime JSONL parse contract is anchored to a known `icm recall --format jsonl` schema). **Invocation**: `cargo semver-checks --baseline-rev <pre-shard-1-sha> --workspace --format json > /tmp/semver-output.json 2>&1 ; SEMVER_EXIT=$?`. **Gate**: `test $SEMVER_EXIT -eq 0 \|\| test "$(jq '[.failures[] \| select(.class != "BASELINE-RESET")] \| length' < /tmp/semver-output.json)" -eq 0`. **BASELINE-RESET class**: NOT a cargo-semver-checks built-in. Defined as a custom rule produced by the new fitness lane `oya-foundry-fitness-baseline-reset-kernel` (scaffolded in Shard 0, populated in Shard 1) — the lane emits a JSON post-processor that re-classifies any name-change-only failure on a renamed crate as class `BASELINE-RESET` in `/tmp/semver-output.json`. The lane's classifier is deterministic: input is the rename map + raw cargo-semver-checks output; output is the augmented JSON. | 0 (only BASELINE-RESET failures allowed; `/tmp/semver-output.json` always produced deterministically) |
| Cargo.lock zero old-names | `rg -F -f /tmp/old-crate-names.txt Cargo.lock` returns no match | exit 1 (FAIL if any old name remains) |
| Cargo.lock semver-section parity (EDIT-1; refuses non-name deltas) | Snapshot pre-rename metadata via `cargo metadata --locked --format-version 1 > /tmp/cargo-metadata-pre-rename.json` BEFORE Shard 1 step 12; after step 12c, run: `diff <(jq -r '.packages[] \| select(.source != null) \| "\(.name) \(.version) \(.source) \(.checksum // "")"' <(cargo metadata --locked --format-version 1)) <(jq -r '.packages[] \| select(.source != null) \| "\(.name) \(.version) \(.source) \(.checksum // "")"' /tmp/cargo-metadata-pre-rename.json) && test $? -eq 0` — compares only external packages (those with `source != null`), excluding workspace members whose names changed. Expected: zero version/source/checksum delta. | 0 (any external version/source/checksum change ⇒ FAIL ⇒ Shard 1 aborts) |
| Registry refs zero old-names | `rg -F -f /tmp/old-crate-names.txt registry/ AGENTS.md docs/CONSTITUTION.md docs/TOOLCHAIN.md docs/RELEASE-MANAGEMENT.md scripts/ .github/workflows/ \| rg -v "docs/CHANGELOG.md\|docs/plans/rename-plan-\|docs/decisions/ADR-0055\|docs/decisions/ADR-0054"` returns no match | exit 1 (FAIL if any reference outside the doc-history allowlist) |
| Fitness lane | `cargo run -p oya-foundry-fitness-architecture-conventions-kernel -- --check` | 0 (Shard 0: `--report-only` mode; Shard 1: BLOCKER mode) |
| 30-day rolling lane-health (EDIT-3; deterministic shell predicate, no human judgement) | `test "$(oya-tooling-agent-read lane-health --window 30d --format jsonl \| jq -s 'map(select(.impossible_to_fail_count > 0)) \| length')" -eq 0` | 0 (threshold pinned at `impossible_to_fail_count == 0` for the 30-day window; ANY non-zero count across the window FAILS the gate) |
| Metadata block parity | `cargo run -p xtask-metadata-augment -- --check --registry-check` | 0 |
| `[lib]` name parity (R7 permanent control) | `cargo run -p xtask-metadata-augment -- --lib-name-check` | 0 |
| Lockfile-rename subcommand integration tests (Architect iter-3 residual #3 + Critic iter-3 condition 1; Shard 0 only) | `cargo nextest run -p xtask-metadata-augment --test lockfile_rename_fixtures` | 0 (all **8 fixture rows** pass: workspace-member rename, rename-with-dependents, external-unchanged, quoted-name, unquoted-name, missing-map-entry no-op, version-disambiguator preservation, version+source-disambiguator preservation) |
| ICM JSONL round-trip contract (Critic iter-3 condition 3b; Shard 0 only) | Shard 0 step 7c command sequence (mint synthetic PR-bound token via `icm store`, recall via `icm recall --format jsonl \| jq \| sed`, assert `RECOVERED == EXPECTED`, tombstone-cleanup) | 0 (deterministic round-trip of PR-bound token payload through pinned `icm` JSONL contract; FAIL ⇒ lane-runtime token parser cannot rely on JSONL stability and Shard 1 entry is blocked) |
| Row 35 reverse-dep count == 95 (EDIT-6; Codex iter-2 #6) | `test "$(cargo metadata --locked --format-version 1 \| jq -r '[.packages[] \| select(.dependencies[]?.name == "oya-platform-fitness-data-boundary-kernel") \| .name] \| unique \| length')" -eq 95` | 0 (exactly 95 unique consumers of the renamed crate post-Shard-1; any drift FAILS the gate and indicates an orphaned or duplicated dep-edge rewrite) |

### 8.2 Global gate (after Shard 1 merge)

All 8.1 commands MUST exit per the table above against `main` at the Shard 1 squash commit. Additionally:

| Gate | Command | Expected exit |
|---|---|:---:|
| Zero hits global sweep | `rg -F -f /tmp/old-crate-names.txt . -g '!docs/CHANGELOG.md' -g '!docs/plans/rename-plan-*.md' -g '!docs/decisions/ADR-0054*' -g '!docs/decisions/ADR-0055*'` | exit 1 (FAIL if any hit) |
| ADR-0055 status | `rg "^status: Accepted" docs/decisions/ADR-0055-*.md` | 0 |
| ADR-0054 amendment present | `rg "Amendment 2026-05-12: rename-event scaffold-claim authority" docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md` | 0 |
| Compound-features registry parity | `cargo run -p xtask-metadata-augment -- --registry-check --strict` | 0 |

## §9 — Estimated effort (Hybrid C honest pricing)

| Phase | Wall-clock (executor) | Reviewer time | Bottleneck |
|---|---:|---:|---|
| Shard 0 (xtask + ADR-0054 amendment + ADR-0055 draft + lane scaffold + registry block + context-vs-feature decision) | **4–6 h** | **1 h** | xtask authoring + ADR drafting |
| Shard 1 (atomic 37-rename + 140 metadata + 44 dep-edges + CI + scripts + docs + registry + lockfile) | **8–12 h** | **6–8 h per primary reviewer × ~3 reviewers parallel = 18–24 h calendar reviewer-hours** (SOFT-EDIT disambiguation; honest per Architect re-pricing) | Reviewer load on 4 hotspots (rows 1, 4, 35, 37) + 95-manifest scan for row 35 + xtask spec review |
| **Total** | **~12–18 h executor + 19–25 h calendar reviewer (6–8 h per primary × 3 reviewers parallel + 1 h Shard 0)** | — | — |
| Calendar (incl. 48 h freeze) | **3–5 days** | — | Shard 0 review (1 day) → 48 h freeze announcement → Shard 1 merge |
| Rollback (standard) | < 60 min | — | git revert + lockfile regen |
| Rollback (emergency lane) | **< 15 min** | — | admin-merge + 3-min CI |

Compare to v2's Option B forecast (22–30 h executor, 6–8 h reviewer, ~2 weeks calendar). Hybrid C is faster (~3–5 days vs ~2 weeks) and has honestly-priced reviewer load.

## §10 — Open questions for `/ralplan --critic` iter-2

Top 3 likely Codex pressure-test surfaces (ordered by suspected probe priority):

1. **xtask-metadata-augment specification completeness**. The xtask is the rewriter for 140 manifests; if its derivation rule for `feature`, `layer`, `audit_chain` has any edge case (especially around `audit_chain = true/false` determination — currently derived from "does the crate emit audit-chain events?" which is a behavioural property, not a name-derivable one), the atomic rewrite produces 140 wrong blocks at once. Codex iter-2 will likely demand: (a) the unit-test matrix for the xtask's derivation rule, (b) the `audit_chain` derivation source (cargo-metadata feature flags? hand-annotated allow-list? per-crate ADR cite?), (c) the dry-run diff sample for 5 representative crates with sign-off before Shard 1.
2. **48 h freeze enforcement mechanism**. v3 §6 R2 says "single 48 h freeze" but does not specify HOW the freeze is enforced. Options: (a) GitHub branch protection rule preventing merges to `main` for 48 h before Shard 1; (b) a fitness lane `oya-foundry-fitness-freeze-window-kernel` that fails any PR merged in the freeze window; (c) merge-queue label `freeze-rename-v3` that blocks dequeue. Critic iter-2 will likely pressure-test which mechanism is authoritative and how the security-expedite lane (R10) cleanly overrides it.
3. **Row 35 (`oya-platform-data-boundary-kernel`) 95-consumer dep-edge correctness verification**. The rename rewrites 95 path edges. v3 trusts the xtask to do it correctly. Codex iter-2 will likely demand: (a) is there a `cargo metadata` reverse-dep query that proves all 95 consumers are accounted for, (b) does the xtask diff against `cargo metadata` to confirm zero orphan edges remain, (c) what happens if one of the 95 has a build-script (`build.rs`) that constructs the dep-name string dynamically (none found per Codex iter-1, but pressure-testable).

## §11 — ADR-0055 outline (must include in same Shard 0 commit)

### ADR-0055 — Adopt Policy B fitness-umbrella crate taxonomy and atomic Shard 1 cutover (Hybrid C)

**Status**: Proposed in Shard 0 commit; flips to Accepted at end of Shard 1.

**Decision**: Collapse the foundry-fitness kernel family under a `fitness` feature umbrella within context `foundry`; ship `[package.metadata.oya]` blocks (per `crate-naming-convention.md:266-272` schema, including `feature`, `layer`, `audit_chain`) for all 140 workspace members in a single atomic Shard 1 PR, preceded by a pure-tooling Shard 0 precursor (xtask + ADR + lane scaffold + registry block).

**Decision drivers** (top 3):
1. Reviewer load is bounded by xtask spec correctness, not by file-change count — atomic review of 190 mechanical edits is cheaper than 6 staged reviews when the rewriter is verified once.
2. Lockfile churn collapses from 6 events (Option B) to 1 event (Hybrid C); semver-baseline reset happens exactly once.
3. Both Architect (condition 9) and Critic (edit #1) listed Hybrid C as their #1 revision — consensus convergence.

**Alternatives considered**:
- **Option A** (atomic, no precursor) — bundles tooling and rename risk in one commit. **Why rejected**: helper bugs and rename bugs are different failure classes; separating them via Shard 0 reduces investigation cost.
- **Option B** (6 sequential shards) — distributes review over calendar time. **Why rejected**: hidden costs (6 lockfile events, 5 rebase windows, row-37-ordering contradiction Critic edit #1, lane-bootstrap chicken-and-egg R6) exceed the bisectability gain.
- **Policy A** (registry-admit compounds, AMBER deferred) — leaves 140-crate AMBER-metadata obligation hanging 5 months. **Why rejected**: user-stated audit-chain rigor priority.

**Why chosen**: as above + iter-1 consensus.

**ADR-0054 amendment (same commit, per Architect condition 2 + Critic edit #4)**:

> **Amendment 2026-05-12: rename-event scaffold-claim authority.** ADR-0054 §Decision is extended to cover **crate-rename events** in addition to new-crate scaffolding. The same icm-coordination-lock fallback applies: open a `scaffold-locks-oyatie` window for each renamed crate's new directory path before `git mv`; close after `cargo check --workspace` is green. Rationale: from grit's perspective, the directory path change creates a "new" workspace member; the rename event is symmetric with new-crate scaffolding and the same race-condition class applies.

**Context-vs-feature decision (Architect condition 1)**: `fitness` is a **feature**, not a context. The context enum stays at six values. Rationale per v3 §3.1.

**Compound capability audit (Architect condition 7 + Critic edit #10; EDIT 10-finalisation)**: enumerates all 31 new 2-token capabilities admitted into `[workspace.metadata.oya].compound_features` (full list in v3 §3.2). **The 31 new 2-token capabilities are admitted as one taxonomy family under the fitness umbrella per Policy B; individual rationale only for AMBER exceptions (row 16 `documentation-system`).** Row 16 (`documentation-system`, AMBER 6-segment) cited individually with rationale; the remaining 30 GREEN compounds inherit the batch admission via the registry table. **Tooling rationale (post-approval correction 1, 2026-05-13)**: `lockfile-rename` ships as a subcommand of `tools/xtask-metadata-augment` (Rust, in-workspace) to avoid a Python toolchain pin, reuse the `toml_edit` parser already required by the metadata-augment xtask, and keep the test runner unified under `cargo nextest`.

**Rollback/expedite protocol (Architect condition 5 + v3 §7.2 + EDIT 3-finalisation + Critic iter-3 condition 2)**: cross-ref to v3 §7.2 and §5.0; pre-authorises `gh pr merge --admin` for emergency revert under the named exception. The exception requires **all three preconditions** at invocation time: (1) `freeze_active == true` on the `oya-foundry-fitness-freeze-window-kernel` lane; (2) operator possesses an `expedite_override_token` minted by **Security Council** (single-use, PR-bound, rotated on consumption per R10) **and the requesting PR number must match the token's `pr=` field** (Critic iter-3 condition 2); (3) `icm store -t direct-tool-invocations -c "emergency-merge-shard1-revert: <reason>; freeze-active=true; security-council-token=<token-hash>; pr=<n>" -i critical` logged BEFORE the admin-merge command runs. Any `gh pr merge --admin` invocation without all three preconditions remains a banned-primitives violation per `git-workflow.md §10 Item 5`.

**Consequences**:
- Positive: zero AMBER-metadata rows after Shard 1; one lockfile event; one rebase window; consistent taxonomy across foundry-fitness; fitness-lane crate parses unambiguously under Policy B; rename-event scaffold-claim authority formally extended.
- Negative: **6–8 h per primary reviewer × ~3 reviewers parallel = 18–24 h calendar reviewer-hours** on atomic Shard 1 (mitigated by Shard 0 separation); per-row bisectability lost (mitigated by xtask spec verification + cargo-metadata diff).

**Hybrid C-Lite escape hatch (SOFT-EDIT)**: If the 48 h freeze window cannot be scheduled within **2 weeks of Shard 0 merge** (e.g., recurring security expedites, ongoing incident response, calendar conflicts with release windows), Shard 1 is held in a long-lived feature branch (`feature/rename-shard-1`) with a daily `git rebase main` cadence. The xtask-driven rewrite is idempotent — re-applying against rebased base is safe. The feature branch carries the same §8.1 acceptance gates locally; only the final merge waits for the next viable 48 h window. This avoids the cliff-edge "freeze-or-abandon" failure mode.

**Follow-ups**:
1. Promote the deny.toml-from-metadata generator (clean-architecture.md note) as a separate post-Shard-1 ADR — out of scope for v3.
2. Document the rightmost-role-token parser rule in `crate-naming-convention.md §6.1` cleanup PR after Shard 1.
3. Consider context-enum extension (`fitness` as 7th context) as a future ADR if the foundry context grows unwieldy.
4. Update `docs/research/hyperscaler-best-practices-2026-05-12.md` Domain 3 cross-references to cite ADR-0055.

## §12 — Reference inventory (full, per Critic edit #9)

Files known to require co-edit in Shard 1 (verified via `rg`):

**Source**:
- 140 `crates/oya-*/Cargo.toml` (all get `[package.metadata.oya]`; 37 get rename + `[lib]` update)
- 1 root `Cargo.toml` (members list + workspace registry block)
- `Cargo.lock` (single regen event)

**CI / scripts**:
- `.github/workflows/release-evidence-pack.yml` (1 site)
- `.github/workflows/supply-chain.yml` (2 sites)
- `scripts/check.sh` (29 sites)
- `scripts/hooks/pre-push-repoctl.sh` (1 site)
- `scripts/check-architecture-boundaries.sh` (3 sites)

**Standards / decisions**:
- `docs/standards/clean-architecture.md` §3 (row 35 named-by-identity update)
- `docs/standards/crate-naming-convention.md` §6 (compound features table refresh)
- `docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md` (amendment block)
- `docs/decisions/ADR-0055-*.md` (new)
- `docs/ADR-INDEX.md` (new row for ADR-0055)
- `docs/CHANGELOG.md` (rename entry)

**Registry** (per Critic edit #9):
- `registry/quality/lanes.yaml`
- `registry/docs/pipeline.tsv`
- registry OpenAPI bindings (under `registry/openapi/`)
- release supply-chain refs (under `registry/release/`)

**Doc / team / product** (per Critic edit #9):
- `AGENTS.md`, `docs/CONSTITUTION.md`, `docs/TOOLCHAIN.md`, `docs/RELEASE-MANAGEMENT.md`
- `docs/research/hyperscaler-best-practices-2026-05-12.md`
- Product/team docs under `docs/teams/`, `docs/products/` — verified via global `rg` in §8.2 zero-hit sweep

**Crate tests** (per Critic edit #9 + Codex iter-2 #7):
- Every renamed crate's `tests/` directory: imports use `[lib] name` underscored form; xtask updates the `use oya_<old>::...` → `use oya_<new>::...` pattern across all `tests/*.rs`
- `crates/oya-tooling-cli-dev-runtime/tests/gate_cli.rs` — hardcoded `oya-tooling-cli-dev-runtime` references at **lines 2830, 2868, 2879** (`cargo run -p oya-tooling-cli-dev-runtime` in fixture shell scripts) and **lines 3456, 3465, 3471, 3472** (GHCR image digest ref + release supply-chain YAML/SBOM filenames). xtask must extend its rewrite pass to `tests/**/*.rs` string-literal scanning for the renamed crate's name, OR these fixtures must be updated by a parallel `sed -i` pass in the same Shard 1 commit.
- `crates/oya-tooling-cli-dev-runtime/tests/repoctl_cli.rs` — lines **149, 159** (`cargo run -p oya-tooling-cli-dev-runtime --bin repoctl`).
- `crates/oya-tooling-cli-dev-runtime/src/commands/repoctl.rs:43` — default value of `cli_manifest_path: PathBuf::from("crates/oya-tooling-cli-dev-runtime/Cargo.toml")`. This is a runtime default, not a test fixture; if not updated, every `repoctl pre-push --verify-contract` run after rename fails the contract check.

**Release artifacts** (per Codex iter-2 #7):
- `registry/release/supply-chain/oya-tooling-cli-dev-runtime.yaml` → `oya-tooling-dev-runtime.yaml`
- `registry/release/0.1.0/oya-tooling-cli-dev-runtime.spdx.json` → `oya-tooling-dev-runtime.spdx.json`
- `registry/release/0.1.0/oya-tooling-cli-dev-runtime.cyclonedx.json` → `oya-tooling-dev-runtime.cyclonedx.json`
- GHCR image ref `ghcr.io/oyatie/oya-tooling-cli-dev-runtime` — Release Engineering confirmation REQUIRED in Shard 0 to choose rename-vs-alias; default: rename atomically with Shard 1.

**Fitness lane**:
- `.omc/fitness-lanes/architecture-conventions.md` (BLOCKER mode flip)

## §13 — Cross-references

- **Superseded plan**: [`docs/plans/rename-plan-v2-2026-05-12.md`](rename-plan-v2-2026-05-12.md) (v2, Option B; status `Superseded` per v3-revision).
- **Audit inventory**: [`docs/audits/convention-audit-2026-05-12.md`](../audits/convention-audit-2026-05-12.md).
- **Grammar**: [`docs/standards/crate-naming-convention.md`](../standards/crate-naming-convention.md) (lines 122-124, 266-272 cited).
- **Layering**: [`docs/standards/clean-architecture.md`](../standards/clean-architecture.md) (§3 row 35 named-by-identity).
- **Git workflow**: [`docs/standards/git-workflow.md`](../standards/git-workflow.md) (lines 47-65 sanctioned-primitives + §3 cutover-bootstrap window).
- **Testing**: [`docs/standards/testing.md`](../standards/testing.md) (line 89 JUnit/JSON archival; §10 D9/D10/D11/D13).
- **Lane spec**: [`.omc/fitness-lanes/architecture-conventions.md`](../../.omc/fitness-lanes/architecture-conventions.md).
- **ADR-0054**: [`docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md`](../decisions/ADR-0054-grit-scaffold-claim-pattern.md) (lines 41-47 scaffold-locks-oyatie windows; amendment block lands in Shard 0).
- **ICM decision provenance**: `01KRFMEVN49BB6J0QWKNGATC1K` (Policy B + immediate metadata cutover, locked 2026-05-12 ~23:00 ET).
- **Open questions ledger**: [`/Users/jasonlee/oyatie/.omc/plans/open-questions.md`](../../.omc/plans/open-questions.md).
