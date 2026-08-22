---
purpose: "M02b/P22 exit-gate — enumeration of the 14 quality/scalability validator lanes and their dev-cli bindings"
doc_status: published
change_id: claude-m02b-p22-doc-coverage-mv-1779009579
meta_policy: ADR-0133 (chained-enforcement planning contract, pending)
---

# M02b/P22 Exit-Gate Validators

> **Status (2026-05-17):** nine lanes wired + tested per PRs #112, #114, #117, #118,
> #119, #120, #121, #122, #123. Remaining lanes (6, 10, 11, 12) are wired in the
> CLI but lack integration tests; each follow-on PR adds one test per lane until
> all 14 are covered, at which point the BLOCKER workflow YAML flip lands.

## What "wired" means

A lane is **wired** when:
1. `oya gate validate <lane>` dispatches to a real Rust kernel (not a stub).
2. At least one integration test in `crates/dev-cli/tests/gate_cli.rs` proves the validator catches a known-bad input.

A lane is **not wired** (implementation exists but test absent) when (1) is true but (2) is false.

## The 14 lanes

The "14 quality/scalability lanes" tracked for M02b/P22 exit gate are the lanes in `ci-lanes.md §1.2` that:
- have a real Rust kernel under `crates/check-*`, AND
- are callable via `oya gate validate <lane>`.

| # | Lane slug | `oya gate validate` subcommand | Kernel crate | Status |
|---|---|---|---|---|
| 1 | `quality-statelessness` | `statelessness` | `check-statelessness` | **wired + tested** (PR #112) |
| 2 | `quality-shardability` | `shardability` | `check-shardability` | **wired + tested** (PR #114) |
| 3 | `quality-perf-budget` | `perf-budget` | `check-perf-budget` | **wired + tested** (PR #117) |
| 4 | `quality-benchmark` | `benchmark` | `check-benchmark` | **wired + tested** (PR #118) |
| 5 | `lean-a-active-artifact-contract` | `active-artifact-contract` | `check-active-artifact-contract` | **wired + tested** (PR #119) |
| 6 | `lean-a-cedar-fragment-coverage` | `cedar-fragment-coverage` | `check-cedar-fragment-coverage` | wired; test pending |
| 7 | `lean-a-openapi-rest-route-parity` | `openapi-rest-route-parity` | `check-openapi-rest-route-parity` | **wired + tested** (PR #120) |
| 8 | `foundation-bypass` | `foundation-bypass` | `check-foundation-bypass` | wired; test exists (gate_cli.rs) |
| 9 | `audit-chain-replay` | `audit-chain-replay` | `check-audit-chain-replay` | **wired + tested** (PR #121) |
| 10 | `foundry-capability-schema` | `foundry-capability-schema` | `check-foundry-capability-schema` | wired; test pending |
| 11 | `foundry-eval` | `foundry-eval` | `check-foundry-eval` | wired; test pending |
| 12 | `cross-tenant-access-fuzz` | `cross-tenant-access-fuzz` | `check-cross-tenant-access-fuzz` | wired; test pending |
| 13 | `lean-a4-semver` | `api-semver` | `check-api-semver` | **wired + tested** (PR #122) |
| 14 | `lean-a5-documentation` | `documentation-system` | `check-documentation-system` | **wired + tested** (PR #123) |

## Lanes that do NOT map to `oya gate validate`

The following lanes in ci-lanes.md run via other mechanisms (cargo toolchain, external scripts, or Foundry fitness crates) and are **out of scope** for this document:

- `cargo-fmt`, `cargo-check`, `cargo-clippy`, `cargo-nextest`, `cargo-deny`, `cargo-machete`
- All `governance-*` lanes (dispatched by their own fitness crate binary, not via `gate validate`)
- `pnpm-typecheck`, `pnpm-test`

## BLOCKER workflow flip policy

Per OP-11, the BLOCKER workflow YAML (`.github/workflows/`) is **not** wired until all 14 rows in the table above read "wired + tested". Each follow-on PR adds exactly one test and flips one row. The final PR flips the YAML.

## Sources

- `docs/standards/ci-lanes.md` — authoritative lane catalog
- `crates/dev-cli/src/scalability_gates.rs` — statelessness/shardability/perf-budget/benchmark runners
- `crates/check-statelessness/src/lib.rs` — statelessness kernel (ADR-0062)
- `crates/dev-cli/tests/gate_cli.rs` — integration tests
