---
doc_status: published
---

# Fitness Lane: cohesion

- status: Accepted
- date: 2026-05-12
- purpose: Verify every cross-axis contract has owner_axis, consumer_axes, and matching catalog crate ids.
- enforces: STANDARD/cross-axis-cohesion; AGENTS.md fitness-lane `governance-cohesion` (already implemented as the reference kernel).
- kernel_crate: `governance-cohesion-fitness-kernel` (EXISTING) — `CrossAxisContract`, verdict `CohesionFitnessReport`.
- runner_path: `tools/governance-cohesion`
- inputs: `docs/contracts/cross-axis-contracts.md`, workspace `Cargo.toml` crate list, catalog `docs/catalog.md`.
- failure_modes:
  - duplicate contract id
  - implemented source crate missing catalog record
  - empty consumer_axes
- ci_invocation: `cargo run -p governance-cohesion`
- runtime_budget: 300 ms
- severity: BLOCKER
- kernel_sketch: SEE existing `crates/governance-cohesion-fitness-kernel/src/lib.rs` (treated as the canonical reference; copy `CrossAxisContract` + `validate_cohesion_fitness` shape verbatim for new lanes).
- notes: this lane is the canonical template all other lanes derive from. New lanes match its pure-value-object pattern: `Vec<Input>` -> `Result<Report, Error>` with `BTreeSet` for uniqueness, no I/O, no async.
