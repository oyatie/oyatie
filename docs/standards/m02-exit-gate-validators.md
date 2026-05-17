---
purpose: "M02b/P22 exit-gate — enumeration of the 14 quality/scalability validator lanes and their oya-dev-cli bindings"
doc_status: published
change_id: claude-m02b-p22-doc-coverage-mv-1779009579
---

# M02b/P22 Exit-Gate Validators

> **Status:** three lanes wired + tested (statelessness, shardability, documentation-system). Remaining lanes are wired in the CLI but lack integration tests; each follow-on PR adds one test per lane until all 14 are covered, at which point the BLOCKER workflow YAML flip lands.

## What "wired" means

A lane is **wired** when:
1. `oya gate validate <lane>` dispatches to a real Rust kernel (not a stub).
2. At least one integration test in `crates/oya-dev-cli/tests/gate_cli.rs` proves the validator catches a known-bad input.

A lane is **not wired** (implementation exists but test absent) when (1) is true but (2) is false.

## The 14 lanes

The "14 quality/scalability lanes" tracked for M02b/P22 exit gate are the lanes in `ci-lanes.md §1.2` that:
- have a real Rust kernel under `crates/oya-check-*`, AND
- are callable via `oya gate validate <lane>`.

| # | Lane slug | `oya gate validate` subcommand | Kernel crate | Status |
|---|---|---|---|---|
| 1 | `quality-statelessness` | `statelessness` | `oya-check-statelessness` | **wired + tested** (this PR) |
| 2 | `quality-shardability` | `shardability` | `oya-check-shardability` | **wired + tested** (this PR) |
| 3 | `quality-perf-budget` | `perf-budget` | `oya-check-perf-budget` | wired; test pending |
| 4 | `quality-benchmark` | `benchmark` | `oya-check-benchmark` | wired; test pending |
| 5 | `lean-a-active-artifact-contract` | `active-artifact-contract` | `oya-check-active-artifact-contract` | wired; test pending |
| 6 | `lean-a-cedar-fragment-coverage` | `cedar-fragment-coverage` | `oya-check-cedar-fragment-coverage` | wired; test pending |
| 7 | `lean-a-openapi-rest-route-parity` | `openapi-rest-route-parity` | `oya-check-openapi-rest-route-parity` | wired; test pending |
| 8 | `foundation-bypass` | `foundation-bypass` | `oya-check-foundation-bypass` | wired; test exists (gate_cli.rs) |
| 9 | `audit-chain-replay` | `audit-chain-replay` | `oya-check-audit-chain-replay` | wired; test pending |
| 10 | `foundry-capability-schema` | `foundry-capability-schema` | `oya-check-foundry-capability-schema` | wired; test pending |
| 11 | `foundry-eval` | `foundry-eval` | `oya-check-foundry-eval` | wired; test pending |
| 12 | `cross-tenant-access-fuzz` | `cross-tenant-access-fuzz` | `oya-check-cross-tenant-access-fuzz` | wired; test pending |
| 13 | `lean-a4-semver` | `api-semver` | `oya-check-api-semver` | wired; test pending |
| 14 | `lean-a5-documentation` | `documentation-system` | `oya-check-documentation-system` | **wired + tested** (this PR) |

## Lanes that do NOT map to `oya gate validate`

The following lanes in ci-lanes.md run via other mechanisms (cargo toolchain, external scripts, or Foundry fitness crates) and are **out of scope** for this document:

- `cargo-fmt`, `cargo-check`, `cargo-clippy`, `cargo-nextest`, `cargo-deny`, `cargo-machete`
- All `oya-foundry-fitness-*` lanes (dispatched by their own fitness crate binary, not via `gate validate`)
- `pnpm-typecheck`, `pnpm-test`

## BLOCKER workflow flip policy

Per OP-11, the BLOCKER workflow YAML (`.github/workflows/`) is **not** wired until all 14 rows in the table above read "wired + tested". Each follow-on PR adds exactly one test and flips one row. The final PR flips the YAML.

## Sources

- `docs/standards/ci-lanes.md` — authoritative lane catalog
- `crates/oya-dev-cli/src/scalability_gates.rs` — statelessness/shardability/perf-budget/benchmark runners
- `crates/oya-check-statelessness/src/lib.rs` — statelessness kernel (ADR-0062)
- `crates/oya-dev-cli/tests/gate_cli.rs` — integration tests
