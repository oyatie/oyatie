---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P05-post-cutover-hardening
impl_plan_id: IP-001-lean-flip-quality-scaffold
status: pending
owner: council-architecture
blocked_by:
- impl_plan: P02/IP-001-shard-1-atomic-rename
  reason: LEAN check crates must be at v4.1 names before flip to BLOCKER
- impl_plan: P03/IP-001-shard-1-5-protocol-rename
  reason: All protocol layers must be final before LEAN-A1 layer-correctness goes
    BLOCKER
- impl_plan: P04/IP-001-iter-4-src-inspection
  reason: layer-correctness evidence required before LEAN-A1 BLOCKER flip
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-deny
purpose: Auto-backfilled purpose for impl-plan.md
---
# IP-001-lean-flip-quality-scaffold: Flip 4 LEAN lanes to BLOCKER + scaffold 4 quality check crates

## Intent

Flips the 4 LEAN architecture check lanes from `--report-only` to `BLOCKER`
in `registry/quality/lanes.yaml` and CI configuration. Scaffolds 4 new quality
check crates (`oya-check-statelessness`, `oya-check-shardability`,
`oya-check-perf-budget`, `oya-check-benchmark`) as empty `--report-only`
crates. Declares M01-foundation exit gate.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `registry/quality/lanes.yaml` | update | Set `severity: BLOCKER` for LEAN-A1..A4; add 4 new quality lanes |
| `crates/oya-check-statelessness/Cargo.toml` | create | Scaffold empty check crate |
| `crates/oya-check-statelessness/src/lib.rs` | create | `// scaffold: implementation in M02` |
| `crates/oya-check-shardability/Cargo.toml` | create | Scaffold empty check crate |
| `crates/oya-check-shardability/src/lib.rs` | create | `// scaffold: implementation in M02` |
| `crates/oya-check-perf-budget/Cargo.toml` | create | Scaffold empty check crate |
| `crates/oya-check-perf-budget/src/lib.rs` | create | `// scaffold: implementation in M02` |
| `crates/oya-check-benchmark/Cargo.toml` | create | Scaffold empty check crate |
| `crates/oya-check-benchmark/src/lib.rs` | create | `// scaffold: implementation in M02` |
| `Cargo.toml` | update | Add 4 new check crates to `[workspace.members]` |
| `.github/workflows/ci.yml` (or equivalent) | update | Flip LEAN-A1..A4 from `--report-only` to blocking |

---

## Crate Naming

```
NAME: oya-check-statelessness
JUSTIFICATION:
- microservice = check: BNF second production; BNF-exempt per ADR-0056
- bc-tokens = statelessness: rule-name (1 token); enforces no module-level mutable state in application/worker/presentation layers per feedback_quality_performance_scalability_bar.md
- layer = check-namespace-exempt
- exemptions claimed: ADR-0056 BNF second production

NAME: oya-check-shardability
JUSTIFICATION:
- microservice = check: BNF-exempt check namespace
- bc-tokens = shardability: rule-name; verifies tenant_id partition key + RLS on all DB designs
- layer = check-namespace-exempt
- exemptions claimed: ADR-0056 BNF second production

NAME: oya-check-perf-budget
JUSTIFICATION:
- microservice = check: BNF-exempt check namespace
- bc-tokens = perf-budget: rule-name; verifies impl plans include load-test results meeting declared targets per ADR-0062
- layer = check-namespace-exempt
- exemptions claimed: ADR-0056 BNF second production

NAME: oya-check-benchmark
JUSTIFICATION:
- microservice = check: BNF-exempt check namespace
- bc-tokens = benchmark: rule-name; verifies PRDs include competitive-benchmark section before L4→L5 per ADR-0062
- layer = check-namespace-exempt
- exemptions claimed: ADR-0056 BNF second production
```

---

## Code Shape

### `crates/oya-check-statelessness/Cargo.toml`

```toml
[package]
name = "oya-check-statelessness"
version.workspace = true
edition.workspace = true
publish = false
license = "Apache-2.0"

[lib]
name = "oya_check_statelessness"
path = "src/lib.rs"
doctest = false
```

### `crates/oya-check-statelessness/src/lib.rs`

```rust
//! `oya-check-statelessness` — M02 implementation pending.
//!
//! Verifies that `application`, `worker`, and presentation-layer crates
//! contain no module-level mutable state (`static mut`, `lazy_static!`,
//! `once_cell::sync::Lazy` with interior mutability).
//!
//! Running in `--report-only` mode until M02 substrate phase completes.
```

(Same pattern for shardability, perf-budget, benchmark — only doc comment changes.)

### `registry/quality/lanes.yaml` changes

```yaml
# LEAN-A1 flip
- id: lean-a1-architecture
  binary: oya-shared-architecture-check-cli
  severity: BLOCKER          # was: report-only
  mode: blocking

# LEAN-A2 flip
- id: lean-a2-bounded-contexts
  binary: oya-shared-bounded-contexts-check-cli
  severity: BLOCKER          # was: report-only
  mode: blocking

# LEAN-A3 (confirm BLOCKER day-1)
- id: lean-a3-supply-chain
  binary: oya-shared-supply-chain-check-cli
  severity: BLOCKER
  mode: blocking

# LEAN-A4 (confirm BLOCKER day-1)
- id: lean-a4-semver
  binary: oya-shared-semver-check-cli
  severity: BLOCKER
  mode: blocking

# New quality lanes (report-only; flip to BLOCKER in M02)
- id: quality-statelessness
  binary: oya-check-statelessness
  severity: report-only
  mode: advisory

- id: quality-shardability
  binary: oya-check-shardability
  severity: report-only
  mode: advisory

- id: quality-perf-budget
  binary: oya-check-perf-budget
  severity: report-only
  mode: advisory

- id: quality-benchmark
  binary: oya-check-benchmark
  severity: report-only
  mode: advisory
```

---

## Acceptance Gates

```bash
# 4 new check crates compile
rtk cargo check --workspace --all-features               # exit 0
rtk cargo build --workspace --all-features               # exit 0
rtk cargo clippy --workspace --all-targets -- -D warnings  # exit 0
rtk cargo deny check                                     # exit 0

# Lane config has BLOCKER severity for LEAN-A1..A4
grep -c "severity: BLOCKER" registry/quality/lanes.yaml  # >= 4

# 4 new check crate dirs present
ls crates/oya-check-statelessness/ crates/oya-check-shardability/ \
   crates/oya-check-perf-budget/ crates/oya-check-benchmark/   # all exist
```

---

## Test Plan

Scaffold-empty crates have zero tests. Acceptance criterion: compilation + lane
config verification above.

---

## Clean Architecture Compliance

All 4 new crates use `oya-check-*` BNF-exempt namespace. No dep edges to
product crates. Each `[lib] name` = snake_case(`[package] name`) (lib-name-parity).

---

## Load Test

Not applicable.

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent post-cutover-hardener \
  --intent "P05: flip 4 LEAN lanes BLOCKER; scaffold 4 quality check crates" \
  --ttl 3600 \
  registry/quality/lanes.yaml::lanes \
  Cargo.toml::workspace.members
```

---

## ICM Rows to Emit

```bash
# M01 completion row
icm store \
  -t context-oyatie \
  -c "M01-foundation EXIT GATE DECLARED. P05 complete: 4 LEAN lanes BLOCKER (LEAN-A1..A4); 4 quality lanes scaffolded (statelessness/shardability/perf-budget/benchmark, report-only). Total M01 output: 114 crates renamed, 26 deferred, 88 STUB cells resolved, 4 LEAN crates BLOCKER, 4 quality crates scaffolded. Next milestone: M02-substrate-schema-foundation." \
  -i high \
  -k "M01,P05,IP-001,LEAN-BLOCKER,quality-scaffold,M01-complete,M02-next"
```

---

## Halt Conditions

1. CI workflow file not found — check `.github/workflows/` for the quality-lanes runner; if not present, scaffold it.
2. `registry/quality/lanes.yaml` does not exist — create it with the schema above; it is the canonical quality-lane registry.
3. LEAN-A1/A2 report non-zero violations on current workspace — violations are advisory (report-only was active during rename); log in ICM, do not block M01 exit.

---

## Next IP Pointer

M02: `.omc/plans/milestones/M02-substrate-schema-foundation/` (separate milestone; not in M01 scope)

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0056 §"CI enforcement matrix"
- ADR-0062: quality/perf/scale bar
- `registry/quality/lanes.yaml`
- Memory: `feedback_quality_performance_scalability_bar.md`, `feedback_clean_architecture_requirements.md`
