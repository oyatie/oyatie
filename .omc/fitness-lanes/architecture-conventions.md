# Fitness Lane — `oya-foundry-fitness-architecture-conventions`

**Severity:** BLOCKER (Top-5)
**Status:** Proposed
**Date:** 2026-05-12
**Enforces:**
- [`docs/standards/crate-naming-convention.md`](../../docs/standards/crate-naming-convention.md)
- [`docs/standards/clean-architecture.md`](../../docs/standards/clean-architecture.md)

This lane is the **combined** enforcement surface for the two
architecture standards: crate-naming grammar **and** layered-architecture
dependency direction. It supersedes the proposed-but-never-shipped lane
`oya-foundry-fitness-naming-convention` (renamed per scope expansion of
2026-05-12 directive).

## 0. Why combined

Naming and layering are **two views of one invariant**: a crate's role
token (naming) is the load-bearing declaration that the layer-direction
check (architecture) keys off. Separating them creates a window where a
crate is renamed but the layer guard hasn't refreshed, or vice versa.
Combining them into a single lane makes the two checks one transaction:
parse every package once, validate name + layer in one pass, fail the
build with a single composite report.

## 1. Implementation language — Rust

The lane is implemented in Rust as a small workspace binary,
**`oya-foundry-fitness-architecture-conventions-kernel`** (proposed
crate), packaged with a thin `[[bin]]` runner under
`oya-tooling-dev-runtime` (per
[rename plan step 9](../../docs/plans/rename-plan-2026-05-12.md#2-sub-plan-b-rename-9-crates)).

### 1.1 Rust vs shell — motivation

| Criterion | Shell + regex | Rust |
|---|---|---|
| Time-to-ship | hours | days |
| Ergonomics on `Cargo.toml` parsing | brittle (TOML edge cases) | first-class (`toml` crate) |
| Cross-platform CI parity | bash + `grep -P` differs Linux/macOS | identical |
| Integration with existing foundry lanes | bolt-on | first-class (`oya-foundry-fitness-*` corpus) |
| Reuses existing workspace primitives | none | reuses `oya-foundry-cargo-prefix-kernel`, `oya-foundry-catalog-kernel` |

The kernel is **<400 LOC** (parser + classifier + edge-walker) and lands
faster than a `bash` reimplementation once cross-platform CI is in
scope. It also reuses the existing foundry fitness-lane corpus, the
TOML parsing wheels that `oya-foundry-cargo-prefix-kernel` already
spins up, and the workspace evidence emission. The shell alternative
would re-implement TOML parsing as regex against `name = "..."` which is
exactly the brittle layer the standard's authority needs to retire.

## 2. Inputs

The lane reads:

1. The workspace `Cargo.toml` at repo root — `[workspace] members`,
   `[workspace.package]`, `[workspace.metadata.oya]`.
2. Every workspace member's `Cargo.toml` (full read; classified by role).
3. The compound-feature registry under `[workspace.metadata.oya]
   compound_features`.
4. The role enum + context enum from the same registry.

Optional inputs:

- `--allow-unregistered-compound` flag (CI-only, never default) for
  bootstrap windows during rename-plan execution.
- `--report-only` flag that runs the full validator but exits 0 (used
  during sub-plan A "advisory mode" rollout per the rename plan).

## 3. Checks

### 3.1 Naming-grammar checks (per crate)

The lane parses every package name and validates against
[`crate-naming-convention.md`](../../docs/standards/crate-naming-convention.md)
§2 BNF. Failure classes (mirror the audit doc):

| Class | Trigger | Severity |
|---|---|---|
| `NOPREFIX` | name does not start with `oya-` | RED |
| `TOOSHORT` | `<4` segments | RED |
| `TOOLONG` | `>6` segments | RED |
| `BAD-CONTEXT` | context token outside enum | RED |
| `NO-ROLE` | no role token found (rightmost-role-token rule) | RED |
| `BAD-ROLE` | role token outside enum | RED |
| `KERNEL-WITH-CAP` | role=`kernel` and capability tail present | RED |
| `ADAPTER-NO-CAP` | role=`adapter` and no capability tail | RED |
| `LONG-FEATURE` | feature has `>2` tokens | RED |
| `LONG-CAPTAIL` | capability tail has `>2` tokens | RED |
| `NEW-COMPOUND` | feature has 2 tokens AND not in compound registry | RED |
| `REGISTERED-COMPOUND` | feature has 2 tokens AND in compound registry | AMBER |
| `6-SEGMENT` | 6 segments total with registered compound | AMBER |
| `NAME-MISMATCH` | `Cargo.toml [package] name` ≠ directory name | RED |
| `LIB-NAME-MISMATCH` | `[lib] name` ≠ underscored package name | RED |

### 3.2 Metadata-block checks (per crate)

| Class | Trigger | Severity (Q3-2026) | Severity (today) |
|---|---|---|---|
| `META-MISSING` | no `[package.metadata.oya]` block | RED | AMBER |
| `META-MISMATCH` | block's `context` / `role` / `feature` / `capability` ≠ name decomposition | RED | RED |
| `META-NO-LAYER` | block missing `layer` key | RED | AMBER |
| `META-NO-AUDIT-CHAIN` | block missing `audit_chain` boolean | AMBER | AMBER |

Two severity columns reflect the cutover: today the missing-metadata case
is AMBER (advisory PR comment); after sub-plan A ships and the 28-row
registry-admit lands, missing metadata becomes a BLOCKER.

### 3.3 Layer-direction checks (per crate, depends on §3.1 + §3.2)

The lane runs `cargo metadata --no-deps --format-version 1`, builds the
dep graph over **workspace members only**, classifies each node by its
role token, and walks every edge. Forbidden-edge classes (mirror
[`clean-architecture.md`](../../docs/standards/clean-architecture.md)
§3 table):

| Class | Trigger | Severity |
|---|---|---|
| `KERNEL-DEP-UP` | kernel crate depends on a non-kernel workspace crate (allowed only: `data-boundary` kernel) | RED |
| `DOMAIN-DEP-UP` | domain crate depends on app / api / worker / adapter / runtime | RED |
| `APP-DEP-UP` | app crate depends on api / worker / adapter / runtime | RED |
| `ADAPTER-DEP-ADAPTER` | adapter crate depends on another adapter | RED |
| `ADAPTER-DEP-APP` | adapter crate depends on an app crate | RED |
| `ADAPTER-DEP-INBOUND` | adapter crate depends on api / worker / runtime | RED |
| `API-WORKER-PEER` | api↔worker peer dep | RED |
| `ROLE-LAYER-MISMATCH` | role=`kernel` but `[dependencies]` includes `tokio` / `reqwest` / `sqlx` / known adapter crates | RED |
| `BIN-WITH-KERNEL-ROLE` | crate ships `[[bin]]` but role=`kernel` | RED |
| `NO-LIB-WITH-LIBRARY-ROLE` | role ∈ {kernel, domain, app, adapter, sdk} but no `[lib]` target | AMBER |

### 3.4 Testing-posture checks (advisory, per crate)

| Class | Trigger | Severity |
|---|---|---|
| `KERNEL-ASYNC-TEST` | role=`kernel` and dev-dependencies include `tokio` | AMBER |
| `RUNTIME-NO-SMOKE` | role=`runtime` and crate has no `tests/` with a binary-smoke test | AMBER |
| `ADAPTER-NO-INTEGRATION` | role=`adapter` and no `--features integration` test entry | AMBER |

Testing-posture violations never block the build; they surface as PR
comments to give authors a soft pull toward the layer-test charter in
clean-architecture.md §5.

## 4. Output

The lane emits two artifacts:

1. **Human-readable report** — Markdown table of every RED + AMBER row,
   linked to the originating standard section. Posted as a PR comment.
2. **Machine-readable JSON** — `evidence/architecture-conventions.json`,
   schema:
   ```json
   {
     "lane": "oya-foundry-fitness-architecture-conventions",
     "ran_at": "2026-05-12T11:00:00Z",
     "totals": {"green": 81, "amber": 22, "red": 37},
     "rows": [
       {"crate": "...", "severity": "RED", "classes": ["LONG-FEATURE"],
        "remediation_doc": "docs/plans/rename-plan-2026-05-12.md#step-3"}
     ]
   }
   ```
   The JSON feeds the foundry evidence pipeline per
   `oya-foundry-evidence-kernel`.

## 5. CI wiring

```yaml
# .github/workflows/architecture-conventions.yml (sketch)
name: oya-foundry-fitness-architecture-conventions
on: [pull_request, push]
jobs:
  conventions:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { toolchain: "1.95.0" }
      - run: cargo run --bin oya-foundry-fitness-architecture-conventions \
             -- --workspace-root . --emit-json evidence/architecture-conventions.json
      - if: github.event_name == 'pull_request'
        uses: actions/github-script@v7  # post PR comment from report.md
        with:
          script: |
            const fs = require('fs');
            const body = fs.readFileSync('evidence/architecture-conventions.md', 'utf8');
            await github.rest.issues.createComment({
              issue_number: context.payload.pull_request.number,
              owner: context.repo.owner, repo: context.repo.repo, body,
            });
```

Pin Rust to the workspace-pinned stable per
[`code-style-rust.md`](../../docs/standards/code-style-rust.md) §1 — the
verified pin is **1.95.0** ([LTS verified
2026-05-12](../../docs/research/lts-versions-verified-2026-05-12.md#languages)).

## 6. Rollout

Mirrors the rename plan rollout (`docs/plans/rename-plan-2026-05-12.md`
§6):

1. Land standards + lane in **`--report-only`** mode. Lane runs on every
   PR but exits 0. Authors see the report; nothing blocks.
2. Land Sub-plan A (registry-admit ADR + workspace registry edit). Lane
   re-runs; RED count drops from 37 to 9.
3. Turn lane to **BLOCKER** for the naming-grammar checks (§3.1) only.
   Authors must fix new naming violations to merge.
4. Land Sub-plan B rename steps (1..8). Lane confirms each step's diff.
5. Land Sub-plan B step 9 (`oya-tooling-cli-dev-runtime` rename). Lane
   confirms.
6. Turn lane to **BLOCKER** for layer-direction checks (§3.3). At this
   point every layer violation refuses merge.
7. Turn lane to **BLOCKER** for metadata checks (§3.2). At this point
   every crate has a `[package.metadata.oya]` block.

## 7. Test plan (for the lane itself)

The lane's kernel ships with a test suite that asserts every failure
class against a fixture corpus:

- `tests/fixtures/<class>/Cargo.toml` — a minimal manifest exhibiting
  the violation.
- `tests/fixtures/green/Cargo.toml` — a clean manifest that MUST parse
  green.
- `tests/integration.rs` — runs the lane against the live workspace and
  asserts the audit numbers (81 GREEN / 22 AMBER / 37 RED on 2026-05-12)
  to detect snapshot drift.

The lane's own crate `oya-foundry-fitness-architecture-conventions-kernel`
MUST itself be GREEN under its own rules (bootstrap consistency).

## 8. Failure classes — cross-reference

Every failure class above appears in either
[`docs/audits/convention-audit-2026-05-12.md`](../../docs/audits/convention-audit-2026-05-12.md)
§4.1 (audit class summary) or
[`docs/plans/rename-plan-2026-05-12.md`](../../docs/plans/rename-plan-2026-05-12.md)
§2 (rename rows). Authors hitting a class in CI follow the rename plan's
linked step.

## 9. Sources scanned

- [`docs/standards/crate-naming-convention.md`](../../docs/standards/crate-naming-convention.md)
- [`docs/standards/clean-architecture.md`](../../docs/standards/clean-architecture.md)
- [`docs/audits/convention-audit-2026-05-12.md`](../../docs/audits/convention-audit-2026-05-12.md)
- [`docs/plans/rename-plan-2026-05-12.md`](../../docs/plans/rename-plan-2026-05-12.md)
- [`docs/research/hyperscaler-best-practices-2026-05-12.md`](../../docs/research/hyperscaler-best-practices-2026-05-12.md)
- [`docs/research/lts-versions-verified-2026-05-12.md`](../../docs/research/lts-versions-verified-2026-05-12.md)
- [Cargo Book — Metadata](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table)
- [cargo-deny — Bans](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html)
- [cargo-metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
