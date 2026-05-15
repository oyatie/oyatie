---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P05-post-cutover-hardening
status: Complete
acceptance_lanes: []
entry_gate: "P02-shard-1-atomic-rename complete (all 6 acceptance gates exit 0);\n\
  P03-shard-1-5-protocol-unknown-deferred complete (zero *-api crates remain);\nP04-iter-4-src-inspection\
  \ complete (zero STUB-pending cells in \xA73 audit body).\nAll three prior phases\
  \ merged to main.\n"
exit_gate: '4 LEAN check crates flipped from `--report-only` to BLOCKER in CI config;

  4 new quality lanes scaffolded and registered: oya-check-statelessness,

  oya-check-shardability, oya-check-perf-budget, oya-check-benchmark;

  registry/quality/lanes.yaml updated with all 8 active lanes (4 flipped +

  4 new); `cargo check --workspace exits 0`; ICM context-oyatie row emitted;

  M01 exit gate declared.

  '
depends_on:
- milestone: M01
  phase: P02-shard-1-atomic-rename
  reason: LEAN check crates must be at v4.1 names before flip to BLOCKER
- milestone: M01
  phase: P03-shard-1-5-protocol-unknown-deferred
  reason: All protocol layers must be final before architecture lane goes BLOCKER
- milestone: M01
  phase: P04-iter-4-src-inspection
  reason: layer-correctness subcommand requires evidence-confirmed layer assignments
owner_team: council-architecture
purpose: Completes M01 by hardening the quality infrastructure created in Shard 0 and validated in P02–P04.
---
# P05-post-cutover-hardening: Flip 4 LEAN checks to BLOCKER + scaffold 4 quality lanes

## Purpose

Completes M01 by hardening the quality infrastructure created in Shard 0 and
validated in P02–P04. The 4 LEAN check crates that ran `--report-only` during
the cutover window are flipped to BLOCKER in `registry/quality/lanes.yaml` and
CI config. Four new quality fitness lanes (statelessness, shardability,
perf-budget, benchmark) are scaffolded as `oya-check-*` crates per ADR-0062,
establishing the M02 enforcement surface.

Advances Master Plan principles: quality self-enforces via CI (no human review
gate for architecture violations); hyperscaler-grade observable quality
(statelessness + shardability are preconditions for 100M-user scale).

---

## Scope

### In-scope

| Item | Action | Files affected |
|---|---|---|
| `oya-shared-architecture-check-cli` (LEAN-A1) | Flip to BLOCKER; populate 7 subcommands | `crates/oya-shared-architecture-check-cli/`, `registry/quality/lanes.yaml` |
| `oya-shared-bounded-contexts-check-cli` (LEAN-A2) | Flip to BLOCKER; implement microservice-isolation rule (v4.1 override: no cross-µservice deps except via workflow/ontology) | same |
| `oya-shared-supply-chain-check-cli` (LEAN-A3) | Flip to BLOCKER (already day-1 BLOCKER per ADR-0056; confirm) | same |
| `oya-shared-semver-check-cli` (LEAN-A4) | Flip to BLOCKER (already day-1 BLOCKER per ADR-0056; confirm) | same |
| `oya-check-statelessness` | Scaffold new crate; `--report-only` initially | `crates/oya-check-statelessness/` |
| `oya-check-shardability` | Scaffold new crate; `--report-only` initially | `crates/oya-check-shardability/` |
| `oya-check-perf-budget` | Scaffold new crate; `--report-only` initially | `crates/oya-check-perf-budget/` |
| `oya-check-benchmark` | Scaffold new crate; `--report-only` initially | `crates/oya-check-benchmark/` |
| `registry/quality/lanes.yaml` | Update 4 existing lanes to `severity: BLOCKER`; add 4 new lanes | `registry/quality/lanes.yaml` |
| `Cargo.toml` | Add 4 new check crates to `[workspace.members]` | `Cargo.toml` |

Naming justifications for new `oya-check-*` crates:

```
NAME: oya-check-statelessness
JUSTIFICATION:
- microservice = check: BNF second production "oya-check-<rule-name>"; BNF-exempt per ADR-0056 line 79-80
- bc-tokens = statelessness: rule-name (1 token); enforces no module-level mutable state in presentation/application/worker layers
- layer = check-namespace-exempt: flat check namespace
- exemptions claimed: ADR-0056 BNF second production

NAME: oya-check-shardability
JUSTIFICATION:
- microservice = check: BNF-exempt check namespace
- bc-tokens = shardability: rule-name; enforces DB designs declare tenant_id partition key + RLS
- layer = check-namespace-exempt
- exemptions claimed: ADR-0056 BNF second production

NAME: oya-check-perf-budget
JUSTIFICATION:
- microservice = check: BNF-exempt check namespace
- bc-tokens = perf-budget: rule-name; verifies impl plans include load-test results meeting declared perf targets
- layer = check-namespace-exempt
- exemptions claimed: ADR-0056 BNF second production

NAME: oya-check-benchmark
JUSTIFICATION:
- microservice = check: BNF-exempt check namespace
- bc-tokens = benchmark: rule-name; verifies PRDs include competitive-benchmark section before L4→L5
- layer = check-namespace-exempt
- exemptions claimed: ADR-0056 BNF second production
```

### Out-of-scope

- Full implementation of `oya-check-statelessness/shardability/perf-budget/benchmark` — scaffold only; implementation in M02 per `feedback_quality_performance_scalability_bar.md`.
- LEAN-A1 through LEAN-A4 full implementations (those ship complete in M02 substrate phase).

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Flip 4 LEAN lanes to BLOCKER; scaffold 4 new check crates; update registry | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features                        # exit 0; includes 4 new check crate scaffolds
cargo build --workspace --all-features                        # exit 0
cargo clippy --workspace --all-targets -- -D warnings         # exit 0
cargo deny check                                              # exit 0
```

### Lane configuration gate

```bash
# 4 LEAN lanes must show severity: BLOCKER
grep -c "severity: BLOCKER" registry/quality/lanes.yaml   # must be >= 4

# 4 new quality check crates present
ls crates/oya-check-statelessness/ crates/oya-check-shardability/ \
   crates/oya-check-perf-budget/ crates/oya-check-benchmark/   # all dirs present
```

---

## Clean Architecture Compliance

### Layer assignments for new check crates

| Crate (BNF v4.1) | Layer | Justification |
|---|---|---|
| `oya-check-statelessness` | check-namespace-exempt | `oya-check-<rule-name>` BNF second production |
| `oya-check-shardability` | check-namespace-exempt | same |
| `oya-check-perf-budget` | check-namespace-exempt | same |
| `oya-check-benchmark` | check-namespace-exempt | same |

### New BCs registered

None — check-namespace crates are BNF-exempt; no BC registration required.

---

## Grit Claim Symbols

```
registry/quality/lanes.yaml::lanes
crates/oya-check-statelessness/src/lib.rs::CheckStatelessness
crates/oya-check-shardability/src/lib.rs::CheckShardability
crates/oya-check-perf-budget/src/lib.rs::CheckPerfBudget
crates/oya-check-benchmark/src/lib.rs::CheckBenchmark
Cargo.toml::workspace.members
```

TTL: 3600s.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "P05-post-cutover-hardening COMPLETE. 4 LEAN check lanes flipped to BLOCKER (LEAN-A1..A4). 4 new quality check crates scaffolded: oya-check-statelessness, oya-check-shardability, oya-check-perf-budget, oya-check-benchmark (--report-only; implementation in M02). registry/quality/lanes.yaml updated. M01-foundation EXIT GATE DECLARED." \
  -i high \
  -k "M01,P05,LEAN,BLOCKER,quality-lanes,M01-complete"
```

---

## References

- ADR-0056 §"CI enforcement matrix": `docs/decisions/ADR-0056-rust-clean-architecture-bnf.md`
- ADR-0062: Quality/Performance/Scalability bar
- `registry/quality/lanes.yaml`
- Memory: `feedback_quality_performance_scalability_bar.md`, `feedback_clean_architecture_requirements.md`, `feedback_autonomous_decision_principles.md`
