---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P20-ci-lanes-operational
status: Proposed
entry_gate: |
  Wave-A phases P01–P11 all complete; at least Wave-B phases P12–P14 complete (enough
  substrate exists to run meaningful statelessness + shardability checks); oya-check-*
  namespace reserved in [workspace.metadata.oya.microservices]; cargo check clean;
  ICM phase-handoff rows emitted for all prerequisite phases.
exit_gate: |
  All 4 new CI fitness lane binaries operational: oya-check-statelessness,
  oya-check-shardability, oya-check-perf-budget, oya-check-benchmark; all 7
  architecture-check sub-commands ship in oya-check-architecture (renamed from
  oya-shared-architecture-check-cli per BNF v4.1); all 14 CI lanes run on every PR
  in --report-only mode (flip to BLOCKER at P22 exit gate); all crates pass cargo
  check/build/clippy/nextest/deny; grit done on all P20 symbols; ICM phase-complete
  row emitted.
depends_on:
  - milestone: M02
    phase: P01-foundry-engine-consolidation
    reason: "CI lane binaries are Foundry internal-engine tooling; P01 Foundry engine consolidation must complete first to establish the oya-foundry-* crate namespace and xtask-metadata-augment binary that the fitness lanes call."
owner_team: council-foundry
---

# P20-ci-lanes-operational: New CI Fitness Lanes — Statelessness + Shardability + Perf-Budget + Benchmark + 7 Architecture-Check Sub-Commands

## Purpose

Delivers the four new CI fitness lane binaries required by
[[feedback-quality-performance-scalability-bar]] and the full 7-sub-command
`oya-check-architecture` binary required by [[feedback-clean-architecture-requirements]] §13.
These lanes enforce the hyperscaler-grade quality bar at compile time and CI time so that
every subsequent PR (M02 Wave-B through M03) runs against automated checks.

The four new lanes:
- `oya-check-statelessness` — verifies no module-level mutable state in presentation /
  application / worker layer crates. Horizontal scalability prerequisite.
- `oya-check-shardability` — verifies every DB table with tenant-bound data declares
  `tenant_id` as distribution column (comment `distribution_column:tenant_id`) and has
  an RLS policy. Citus-readiness enforcement.
- `oya-check-perf-budget` — verifies every impl plan includes a `## Load test` section
  with declared p99 targets; CI blocks merge if missing.
- `oya-check-benchmark` — verifies every PRD includes a `## Competitive Benchmark` section
  before the µservice graduates L4 → L5 on the Proof Ladder.

The 7 `oya-check-architecture` sub-commands (renamed from `oya-shared-architecture-check-cli`
per BNF v4.1 flat namespace):
`dependency-direction`, `layer-correctness`, `lib-name-parity`, `port-location`,
`cross-product-refusal`, `composition-root-only`, `sdk-kernel-only`.

Wave-D classification: runs in parallel with Waves A–C since it produces tooling not
product substrate. Can begin once enough substrate crates exist to validate against.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `check` (exempt namespace) | — | `crates/oya-check-statelessness/` | `oya-check-statelessness` |
| `check` (exempt namespace) | — | `crates/oya-check-shardability/` | `oya-check-shardability` |
| `check` (exempt namespace) | — | `crates/oya-check-perf-budget/` | `oya-check-perf-budget` |
| `check` (exempt namespace) | — | `crates/oya-check-benchmark/` | `oya-check-benchmark` |
| `check` (exempt namespace) | — | `crates/oya-check-architecture/` | `oya-check-architecture` |
| `foundry` | `ci` | `.github/workflows/ci-fitness-lanes.yml` | — |

Naming justification:

```
NAME: oya-check-statelessness
JUSTIFICATION:
- "oya-check-*" is the BNF-exempt namespace per ADR-0056 v4.1 §"Check-namespace
  exemption": "oya-check-<rule-name>" is a flat namespace for cross-cutting check
  rules; not bound to any microservice slot2
- rule-name = statelessness: 1 token; checks for module-level mutable state in
  presentation/application/worker crates; ADR-0056 v4.1 rule-name format
- exemptions claimed: oya-check-* namespace exemption (explicitly stated in ADR-0056)
```

### Out-of-scope

- Flipping lanes from `--report-only` to BLOCKER — deferred to P22-m02-exit-gate
- LEAN-A2 (bounded-contexts-check-cli), LEAN-A3 (supply-chain-check-cli), LEAN-A4
  (semver-check-cli) — these existed pre-M02; only the 4 new lanes ship here
- Per-crate Proof Ladder L4→L5 graduation — deferred to M03

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-ci-lanes-statelessness-shardability.md`](IP-001-ci-lanes-statelessness-shardability.md) | Implement oya-check-statelessness + oya-check-shardability binaries with Cargo metadata parsing | pending | `council-foundry` |
| [`IP-002-ci-lanes-perf-budget-benchmark.md`](IP-002-ci-lanes-perf-budget-benchmark.md) | Implement oya-check-perf-budget + oya-check-benchmark binaries with markdown parsing | pending | `council-foundry` |
| [`IP-003-ci-lanes-architecture-check.md`](IP-003-ci-lanes-architecture-check.md) | Implement oya-check-architecture with all 7 sub-commands; rename from oya-shared-architecture-check-cli | pending | `council-foundry` |
| [`IP-004-ci-lanes-github-workflow.md`](IP-004-ci-lanes-github-workflow.md) | Wire all 14 lanes into .github/workflows/ci-fitness-lanes.yml in --report-only mode | pending | `council-foundry` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
```

### Lane-specific gates

```bash
# Each check binary self-tests against known violations
cargo nextest run -p oya-check-statelessness --test self_test   # exit 0
cargo nextest run -p oya-check-shardability --test self_test    # exit 0
cargo nextest run -p oya-check-perf-budget --test self_test     # exit 0
cargo nextest run -p oya-check-benchmark --test self_test       # exit 0
cargo nextest run -p oya-check-architecture --test self_test    # exit 0; all 7 sub-cmds

# Run statelessness check against workspace (--report-only; no blocker yet)
cargo run -p oya-check-statelessness -- --workspace --report-only   # exit 0; report generated
# Run shardability check against workspace
cargo run -p oya-check-shardability -- --workspace --report-only    # exit 0; report generated
# Run architecture check all 7 sub-commands
cargo run -p oya-check-architecture -- dependency-direction --workspace --report-only
cargo run -p oya-check-architecture -- cross-product-refusal --workspace --report-only
cargo run -p oya-check-architecture -- port-location --workspace --report-only
```

---

## Clean Architecture Compliance

All `oya-check-*` crates are BNF-exempt (check-namespace). They are `cli` layer binaries
that depend only on `cargo_metadata`, `syn`, `pulldown-cmark`, and `clap` — no product
crate dependencies. They parse the workspace graph externally; they do not import any
`oya-*` product crates.

### CI lanes this phase itself must green

| Lane | Command | Expected |
|---|---|---|
| `cargo-check` | `cargo check --workspace` | exit 0 |
| `cargo-clippy` | `cargo clippy -- -D warnings` | exit 0 |
| `cargo-nextest` | `cargo nextest run --workspace` | exit 0 |

(Architecture LEAN lanes are `--report-only` until P22 flips them; this phase produces
the tools that will enforce those lanes.)

### New BCs registered in this phase

None — `oya-check-*` crates are BNF-exempt; no BC registration required.

---

## Grit Claim Symbols

```
crates/oya-check-statelessness/src/main.rs::main
crates/oya-check-shardability/src/main.rs::main
crates/oya-check-perf-budget/src/main.rs::main
crates/oya-check-benchmark/src/main.rs::main
crates/oya-check-architecture/src/main.rs::main
.github/workflows/ci-fitness-lanes.yml::fitness-lanes-job
```

TTL: `--ttl 3600`. Fallback: ICM `scaffold-locks-oyatie`.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P20-ci-lanes-operational started; 4 new fitness lane binaries; 7 architecture-check sub-commands; Wave-D parallel with A-C" \
  -i high \
  -k "M02,P20,phase-start,ci-lanes"

icm store \
  -t context-oyatie \
  -c "Phase P20-ci-lanes-operational complete; oya-check-statelessness/shardability/perf-budget/benchmark + oya-check-architecture all 7 sub-cmds operational in --report-only mode; flip to BLOCKER at P22; next: P21-architecture-planes-green" \
  -i high \
  -k "M02,P20,phase-complete,ci-lanes"
```

---

## References

- Memory: `feedback_quality_performance_scalability_bar.md`, `feedback_clean_architecture_requirements.md`
- oyatie ADRs cited: ADR-0056 v4.1 (check-namespace exemption), ADR-0062 (quality/perf/scale bar)
- Bominal ADRs inherited: ADR-0100/0101 (hexagonal; LEAN checks inherit)
