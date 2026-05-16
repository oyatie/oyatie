---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02b-substrate
phase: P01-foundry-engine-consolidation
status: Proposed
acceptance_lanes: []
entry_gate: 'M01-P05 complete; oya-tooling-agent-read scaffold merged; grit v0.3.0

  installed; icm CLI installed; cargo check --workspace exits 0 on main.

  '
exit_gate: 'All 4 LEAN check binaries compile and pass --report-only on workspace;

  7 sub-commands each (28 total) implemented and exit 0; grit done called

  on all phase symbols; ICM phase-handoff row emitted under context-oyatie.

  '
depends_on:
- milestone: M01
  phase: P05-scaffold-locks
  reason: grit + icm CLI primitives must exist before Foundry consolidation
owner_team: council-foundry
purpose: "This phase consolidates the grit/icm/oya-tooling-agent-read scaffolds into a coherent Foundry engine composed of 4 LEAN check binaries (`oya-shared-architecture-check-cli`, `oya-shared-bounded-contexts-check-cli`."
---
# P01-foundry-engine-consolidation: Consolidate Foundry engine into 4 LEAN check binaries with 7 sub-commands each

## Purpose

This phase consolidates the grit/icm/oya-tooling-agent-read scaffolds into a coherent Foundry engine composed of 4 LEAN check binaries (`oya-shared-architecture-check-cli`, `oya-shared-bounded-contexts-check-cli`, `oya-shared-supply-chain-check-cli`, `oya-shared-semver-check-cli`) plus 3 quality-gate binaries (`oya-check-statelessness-cli`, `oya-check-shardability-cli`, `oya-check-perf-budget-cli`). Together these 7 binaries implement the 14 CI enforcement lanes mandated by ADR-0056 v4.1. The phase advances Master Plan principle: "CI-enforced architecture" — every future phase ships knowing the fitness lanes exist and run. Without these binaries the cross-product-refusal, port-location, and shardability gates are advisory only; this phase makes them executable.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `foundry` | `grit`, `icm`, `architecture-check`, `bounded-contexts-check`, `supply-chain-check`, `semver-check` | `crates/oya-foundry-grit-cli/`, `crates/oya-foundry-icm-cli/`, `crates/oya-foundry-agent-read-cli/`, `crates/oya-shared-architecture-check-cli/`, `crates/oya-shared-bounded-contexts-check-cli/`, `crates/oya-shared-supply-chain-check-cli/`, `crates/oya-shared-semver-check-cli/`, `crates/oya-check-statelessness-cli/`, `crates/oya-check-shardability-cli/`, `crates/oya-check-perf-budget-cli/` | `oya-foundry-grit-cli`, `oya-foundry-icm-cli`, `oya-foundry-agent-read-cli`, `oya-shared-architecture-check-cli`, `oya-shared-bounded-contexts-check-cli`, `oya-shared-supply-chain-check-cli`, `oya-shared-semver-check-cli`, `oya-check-statelessness-cli`, `oya-check-shardability-cli`, `oya-check-perf-budget-cli` |

Naming justification for new crates:

```
NAME: oya-shared-architecture-check-cli
JUSTIFICATION:
- microservice = shared-architecture-check: cross-cutting fitness lane for
  architecture rule enforcement; ADR-0056 v4.1 flat BNF; check-namespace exemption
  applies (oya-check-*); registered under foundry owner council-foundry
- bc-tokens = (none): single-concept CLI binary at this layer
- layer = cli: CLI binary with subcommands; ADR-0056 §"Layer semantics"
- exemptions claimed: none

NAME: oya-check-statelessness-cli
JUSTIFICATION:
- microservice = check-statelessness: check-namespace flat exemption per ADR-0056;
  verifies no module-level mutable state in presentation/application/worker layers
- bc-tokens = (none): single-concept
- layer = cli: CLI binary
- exemptions claimed: oya-check-* namespace is BNF-exempt per ADR-0056
```

### Out-of-scope

- `oya-foundry-fitness-plan-hierarchy` CI gate — owned by M02-P02+ phases; not touched here.
- Any product-µservice crates — this phase only touches foundry and shared check crates.
- grit server-side protocol changes — grit v0.3.0 consumed as installed binary.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Scaffold all 10 foundry/check crates; implement 7 sub-commands each on 4 LEAN check binaries; wire CI matrix | pending | `council-foundry` |

---

## Acceptance Gates

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P01-foundry-engine-consolidation   # LEAN-A1: layer ordering
oya gate validate lean-a2 --phase P01-foundry-engine-consolidation   # LEAN-A2: cross-vertical refusal
oya gate validate lean-a3 --phase P01-foundry-engine-consolidation   # LEAN-A3: BC boundary
oya gate validate lean-a4 --phase P01-foundry-engine-consolidation   # LEAN-A4: naming conformance
```

### Sub-command smoke tests

```bash
# LEAN-A1: dependency-direction
oya-shared-architecture-check-cli dependency-direction --workspace . --report-only  # exit 0
# LEAN-A2: cross-product-refusal
oya-shared-architecture-check-cli cross-product-refusal --workspace . --report-only  # exit 0
# LEAN-A3: port-location
oya-shared-architecture-check-cli port-location --workspace . --report-only  # exit 0
# LEAN-A4: layer-correctness
oya-shared-architecture-check-cli layer-correctness --workspace . --report-only  # exit 0
# lib-name-parity
oya-shared-architecture-check-cli lib-name-parity --workspace . --report-only  # exit 0
# composition-root-only
oya-shared-architecture-check-cli composition-root-only --workspace . --report-only  # exit 0
# sdk-kernel-only
oya-shared-architecture-check-cli sdk-kernel-only --workspace . --report-only  # exit 0
# BC boundary
oya-shared-bounded-contexts-check-cli bc-boundary --workspace . --report-only  # exit 0
# Supply chain
oya-shared-supply-chain-check-cli deny-check --workspace .  # exit 0
# Semver
oya-shared-semver-check-cli api-stability --workspace . --report-only  # exit 0
# Quality gates
oya-check-statelessness-cli --workspace . --report-only  # exit 0
oya-check-shardability-cli --workspace . --report-only   # exit 0
oya-check-perf-budget-cli --workspace . --report-only    # exit 0
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-foundry-grit-cli` | `cli` | N/A | N/A | Yes — CLI entry point |
| `oya-foundry-icm-cli` | `cli` | N/A | N/A | Yes — CLI entry point |
| `oya-foundry-agent-read-cli` | `cli` | N/A | N/A | Yes — CLI entry point |
| `oya-shared-architecture-check-cli` | `cli` | N/A | N/A | Yes — CLI entry point |
| `oya-shared-bounded-contexts-check-cli` | `cli` | N/A | N/A | Yes — CLI entry point |
| `oya-shared-supply-chain-check-cli` | `cli` | N/A | N/A | Yes — CLI entry point |
| `oya-shared-semver-check-cli` | `cli` | N/A | N/A | Yes — CLI entry point |
| `oya-check-statelessness-cli` | `cli` | N/A | N/A | Yes — CLI entry point |
| `oya-check-shardability-cli` | `cli` | N/A | N/A | Yes — CLI entry point |
| `oya-check-perf-budget-cli` | `cli` | N/A | N/A | Yes — CLI entry point |

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P01-foundry-engine-consolidation` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P01-foundry-engine-consolidation` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P01-foundry-engine-consolidation` | exit 0 |
| `layer-correctness` | `oya gate validate layer-correctness --phase P01-foundry-engine-consolidation` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P01-foundry-engine-consolidation` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P01-foundry-engine-consolidation` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `grit` | `foundry` | pending |
| `icm` | `foundry` | pending |
| `agent-read` | `foundry` | pending |

---

## Grit Claim Symbols

```
crates/oya-shared-architecture-check-cli/src/main.rs::main
crates/oya-shared-architecture-check-cli/src/lib.rs::ArchitectureCheckCli
crates/oya-shared-bounded-contexts-check-cli/src/main.rs::main
crates/oya-shared-supply-chain-check-cli/src/main.rs::main
crates/oya-shared-semver-check-cli/src/main.rs::main
crates/oya-check-statelessness-cli/src/main.rs::main
crates/oya-check-shardability-cli/src/main.rs::main
crates/oya-check-perf-budget-cli/src/main.rs::main
crates/oya-foundry-grit-cli/src/main.rs::main
crates/oya-foundry-icm-cli/src/main.rs::main
```

TTL recommendation: `--ttl 7200` (2 h); re-claim if exceeding.

Fallback: ICM topic `scaffold-locks-oyatie` per ADR-0054.

---

## ICM Rationale Fields

```bash
# At phase start
icm store \
  -t context-oyatie \
  -c "Phase P01-foundry-engine-consolidation started; milestone M02b-substrate; scope: foundry 10 check crates + 4 LEAN binaries; entry gate met: M01-P05 complete" \
  -i high \
  -k "M02,P01,phase-start,foundry,lean-checks"

# At phase complete
icm store \
  -t context-oyatie \
  -c "Phase P01-foundry-engine-consolidation complete; IPs merged: impl-plan; grit symbols released: oya-shared-architecture-check-cli + 9 others; lanes green: all 14 CI lanes --report-only; next phase: P02-ontology" \
  -i high \
  -k "M02,P01,phase-complete,foundry,lean-checks"
```

---

## References

- Milestone README: `../../README.md`
- Bominal ADRs inherited: ADR-0100 (hexagonal reference), ADR-0101 (hexagonal standard)
- oyatie ADRs cited: ADR-0056 (BNF v4.1), ADR-0057 (LEAN checks)
- Memory files: `feedback_clean_architecture_requirements.md §13`, `feedback_grit_claim_work_done.md`
- unblocks: P02-ontology, P03-identity, P04-audit-chain, P05-eventing, P06-secrets, P07-observability, P08-kms, P09-search, P10-vector, P11-finance-library (all Wave-A phases consume these CI lanes)
