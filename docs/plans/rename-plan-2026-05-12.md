---
doc_class: HowTo
shape: ~
length_cap: 500
authority_tier: 3
status: Superseded
superseded_by: docs/plans/rename-plan-v2-2026-05-12.md
superseded_on: 2026-05-12
date: 2026-05-12
purpose: |
  Cost-justified rename plan for every RED entry in
  `docs/audits/convention-audit-2026-05-12.md`. Bounds the cutover scope,
  classifies each row by effort + risk, and prescribes a single execution
  order (leaves → roots). Audit-only; nothing is renamed by this document.
canonical_authority: docs/CONSTITUTION.md
companion_docs:
  - docs/standards/crate-naming-convention.md
  - docs/standards/clean-architecture.md
  - docs/audits/convention-audit-2026-05-12.md
  - .omc/governance-lanes/architecture-conventions.md
related_adrs:
  - ADR-0015
  - ADR-0017
doc_status: published
---

> **SUPERSEDED — 2026-05-12.** This v1 plan is **superseded by**
> [`rename-plan-v2-2026-05-12.md`](rename-plan-v2-2026-05-12.md).
>
> locked **Policy B** (collapse foundry-fitness under a `fitness` umbrella),
> elevating the rename cohort from 9 → **39** and adding an immediate
> **AMBER-metadata cutover** for all 140 workspace members. The Policy A
> sub-plan, the 9-row leaves→roots order, and the "AMBER-metadata deferred to
> Q3-2026" recommendation in this document **no longer apply**. Retained for
> do not execute.
>
> Authoritative execution plan: `docs/plans/rename-plan-v2-2026-05-12.md`.

# Rename Plan — 2026-05-12 *(SUPERSEDED — see banner above)*

## 0. Scope and constraint

The audit found **37 RED rows** out of 140 crates. This plan covers only
RED. AMBER rows ride on a follow-up "metadata block" PR; GREEN rows
require no action. The plan **does not rename anything**; it sequences the
work for a future execution PR.

Per the grammar standard §6, **the lowest-cost remediation** for 28 of the
37 RED rows is to extend the workspace compound-feature registry by a
single ADR rather than rename crates. Renaming any of the 28 NEW-COMPOUND
rows touches 50+ `path = "../..."` rewrites apiece. The plan therefore
splits RED into two sub-plans:

- **Sub-plan A — Registry-admit (28 crates, NEW-COMPOUND).** One ADR adds
  every NEW-COMPOUND feature name to `[workspace.metadata.oya]
  compound_features`. Zero crate renames; the lane re-validates and these
  rows turn AMBER (registered compound).
- **Sub-plan B — Rename (9 crates).** The remaining RED rows have an
  actual grammar violation (long feature, no role, too short, etc.) that
  registry-admission cannot fix.

If sub-plan B's 9 renames are accepted, the rename cohort is
**under 25** and per the grammar standard §11.1, a small
`cargo-rename-helper` script is recommended (not full consensus). If the
user instead chooses Policy B (collapse foundry-fitness under a `fitness`
umbrella per audit §6), the cohort grows past 30 and consensus
adjudication is REQUIRED before execution.

## 1. Sub-plan A — Registry-admit (preferred; 28 crates)

A single ADR proposal `ADR-FND-008 — Workspace compound-feature registry
extension` adds the following 28 features to
`[workspace.metadata.oya] compound_features`:

```
adr-citation, adr-index, authority-cohesion, brand-residue,
claim-ceiling, cloud-mutation, codeowners-mirror, cohesion-fitness,
constitution-cite, cost-budget, data-boundary, doc-catalog,
documentation-system, glossary-coverage, glossary-vocabulary,
license-policy, mcp-gateway, mobile-native, placeholder-debt,
pr-traceability, pre-push, quality-lane, runbook-freshness,
runbook-index, slo-coverage, supply-chain, typescript-workspace,
api-semver
```

Risk: low. No crate renames. The lane re-runs and re-buckets each row
from RED → AMBER. The ADR cite makes the registry decision durable.

Effort: **S** (single ADR + workspace `Cargo.toml` edit + lane re-roll).

## 2. Sub-plan B — Rename (9 crates)

Each row below is a true grammar violation. Renames are sequenced by
**dependency-graph depth**: leaves (no internal consumers) first; roots
(consumed by many) last. Sampled consumer counts come from a grep over
the workspace `Cargo.toml` files (`grep -r 'path = "../<crate>"'`).

| # | Current name | Proposed name | Class | Direct consumers | Effort | Risk | Cutover order |
|---:|---|---|---|---:|:---:|---|---:|
| 1 | `foundation-app` | `foundation-composition-app` | TOOSHORT | 1 (`tooling-cli-dev-runtime`) | M | low-MED — sole foundation singleton; dep is the dev-runtime CLI; doc cross-refs need update | 8 |
| 2 | `intelligence-api` | `intelligence-policy-binding-api` *(see §2.1)* | TOOSHORT | 0–1 | S | low — newly-stood-up crate, minimal consumers | 1 |
| 3 | `governance-data-class-fitness-kernel` | `governance-data-class-kernel` *(drop `fitness`; the foundry context already implies fitness)* | LONG-FEATURE | 1 (`tooling-cli-dev-runtime`) | S | low — fitness is the foundry context's purpose | 2 |
| 4 | `governance-raci-team-coverage-kernel` | `governance-raci-coverage-kernel` *(drop `team`; RACI implies team)* | LONG-FEATURE | 1 (`tooling-cli-dev-runtime`) | S | low | 3 |
| 5 | `governance-readme-doc-coverage-kernel` | `governance-readme-coverage-kernel` *(README implies doc)* | LONG-FEATURE | 1 (`tooling-cli-dev-runtime`) | S | low | 4 |
| 6 | `intelligence-release-evidence-pack-kernel` | `intelligence-release-pack-kernel` *(evidence is implied by foundry context)* OR `intelligence-evidence-pack-kernel` *(drop `release`; the artifact is the pack)* | LONG-FEATURE | 1 (`tooling-cli-dev-runtime`) | S | MED — pick one; doc cross-refs in `release-management.md` need update | 5 |
| 7 | `governance-vendor-contract-recency-kernel` | `governance-vendor-recency-kernel` *(contract is implied)* | LONG-FEATURE | 1 (`tooling-cli-dev-runtime`) | S | low | 6 |
| 8 | `tooling-agent-read` | `tooling-agent-cli-read` *(insert role `cli` before capability `read`)* OR `tooling-agent-read-cli` *(role last)* | NO-ROLE | 0 | S | low — newly-added bin-only crate; not yet imported | 7 |
| 9 | `tooling-cli-dev-runtime` | `tooling-dev-runtime` *(`cli` and `runtime` are redundant; pick role `runtime`)* OR keep as-is by admitting `cli-dev` as compound | LONG-CAPTAIL | many (workspace dev-binary host) | L | **HIGH** — this is the workspace's primary dev binary (`oya`, `repoctl`); CI scripts, AGENTS.md, docs reference it by name | 9 |

### 2.1 Note on `intelligence-api`

`intelligence-api` parses as `context=foundry` + (no feature) + role `api`.
Possible re-interpretations:

1. Insert a feature segment naming the bound surface. The crate's
   `[lib]` exposes the foundry policy binding for external consumers
   (foundry policy ↔ cloud / platform contracts). Proposed:
   `intelligence-policy-binding-api` — but this clashes semantically with
   the existing `intelligence-policy-api`. The right answer is probably to
   **merge** the two into `intelligence-policy-api` and retire
   `intelligence-api`. This adds a **MERGE** row to the plan; flagged for
   architect review.
2. If the crate truly has a distinct surface, choose a feature segment
   that does not clash (e.g. `intelligence-meta-api`).

Carrying forward as a rename candidate; the merge option is the user's call.

### 2.2 Note on `tooling-cli-dev-runtime`

The token sequence parses three ways:

| Parse | feature | role | capability | Verdict |
|---|---|---|---|---|
| A | `cli` | `dev` (invalid role) | — | rejected; `dev` is not a role |
| B | `cli-dev` (compound) | `runtime` | — | acceptable if `cli-dev` registered as compound; semantically thin |
| C | `cli` | `runtime` | `dev` | semantically wrong; `dev` describes the *audience* not a capability |

Recommend **rename to `tooling-dev-runtime`** (feature `dev`, role
`runtime`, no capability) since `cli` is redundant with `runtime` for a
bin-only crate. This is the canonical interpretation per
`crate-naming-convention.md` §6.1. Effort is L because the crate ships
two bin targets (`oya`, `repoctl`) and is the universal CI entry point;
every CI workflow file, every doc reference, and the executor's run
scripts will need a coordinated update.

## 3. Cutover order — leaves → roots

```
1. intelligence-api                         (zero/one consumer; cheapest first)
2. governance-data-class-fitness-kernel   (1 consumer)
3. governance-raci-team-coverage-kernel   (1 consumer)
4. governance-readme-doc-coverage-kernel  (1 consumer)
5. intelligence-release-evidence-pack-kernel (1 consumer)
6. governance-vendor-contract-recency-kernel (1 consumer)
7. tooling-agent-read                  (0 consumers)
8. foundation-app                      (1 consumer; coordinate with sub-plan B step 9)
9. tooling-cli-dev-runtime             (top of dep graph; coordinated final cutover)
```

Each step's PR MUST: (a) bump the workspace `Cargo.toml` member list,
(b) rewrite every `path = "../<old-name>"` reference, (c) update doc
cross-references found via `rg -l "<old-name>"`, (d) re-run the lane.

## 4. Tooling recommendation

**Cohort size = 9** ⇒ per crate-naming-convention.md §11.1, the
recommended path is **a small `cargo-rename-helper` script** (not pure
manual; not consensus-gated). The script:

1. Reads `--from <old-name> --to <new-name>`.
2. Renames the directory under `crates/`.
3. Rewrites `Cargo.toml` `name`, `[lib] name` (underscored), and the
   `[package.metadata.oya]` block.
4. Walks every workspace `Cargo.toml` and rewrites `path = "..."` and
   `name = "..."` references.
5. Runs `rg -l "<old-name>" docs/ .omc/ AGENTS.md CLAUDE.md` and prints
   doc-update candidates (does NOT auto-rewrite docs — they need review).
6. Runs `cargo check --workspace` to verify the rename compiles.

This script SHOULD live in `crates/tooling-dev-runtime/src/bin/`
(after step 9) or temporarily in `tools/cargo-rename-helper.sh`.

## 5. Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Step 9 (`tooling-cli-dev-runtime` rename) breaks every CI workflow | M | H | Land in one atomic PR with all `.github/workflows/*.yml` updates; freeze main during the merge |
| `foundation-app` rename collides with the public crate `foundation-app` (unlikely) on crates.io | L | L | All crates `publish = false`; no namespace collision |
| Doc cross-refs forgotten in `MASTERPLAN.md` / `RUNBOOKS-INDEX.md` / `AGENTS.md` | M | M | Lane includes a `rg`-based "no orphaned references" sub-check; sub-plan A registers the names before any rename |
| Consensus required for step 9 (high-risk leaf) | M | M | Run `/plan --consensus` on step 9 alone before scheduling |
| Sub-plan A is "approve all 28 compounds without per-feature review" | M | L | The single ADR enumerates each compound + a one-sentence rationale, reviewable by architect |

## 6. Recommended sequencing with rest of MASTERPLAN

1. **First**: land `docs/standards/crate-naming-convention.md` and
   `docs/standards/clean-architecture.md` (this PR's deliverables).
2. **Second**: land Sub-plan A as ADR-FND-008 + workspace registry edit.
   This converts 28 RED rows to AMBER without code churn. Turn on the
   lane in **advisory** mode.
3. **Third**: land Sub-plan B steps 1–8 in any order (single PR per
   step or one rolled-up PR). The cargo-rename-helper script lands first
   if multiple authors will run renames.
4. **Fourth**: open the consensus loop on Sub-plan B step 9 (the
   `tooling-cli-dev-runtime` rename). Coordinate with CI freeze.
5. **Fifth**: turn the lane from advisory to **BLOCKER** once RED = 0
   and AMBER-metadata has a per-crate `[package.metadata.oya]` block
   landed across the workspace.

## 7. Effort totals

| Sub-plan | Crates | ADRs | PRs | Effort sum |
|---|---:|---:|---:|---|
| A — Registry-admit | 28 | 1 | 1 | **S** |
| B — Rename steps 1–8 | 8 | 0 | 1 (rolled) or 8 (per-step) | **S+M** |
| B — Rename step 9 | 1 | 0 | 1 (atomic; consensus-gated) | **L** |

## 8. Open question

The single decision deferred to user adjudication is whether to choose
Policy A (registry-admit, this plan as written) or Policy B (collapse
foundry-fitness under a `fitness` feature umbrella, escalating cohort
size past 30 and requiring consensus). Recorded in
[`/Users/jasonlee/oyatie/.omc/plans/open-questions.md`](../../.omc/plans/open-questions.md).
