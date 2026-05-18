---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-eval-harness-substrate
impl_plan_id: IP-014-parity-analyzer-bootstrap
status: pending
execution_unit: ChangeSet
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

# IP-014: parity-analyzer bootstrap (kernel + domain + adapter-clickhouse + usecase)

## Intent

Bootstrap the parity-analyzer BC: kernel (entities + ports) + domain (delta + DP-noise math) + adapter-clickhouse (parity_analytics INSERT + cohort rollup SELECT) + usecase (orchestrator). Per ADR-0024 §"A/B testing" + ADR-0026 §"In-house cutover".

## ChangeSet boundary

`microservices/foundry/src/crates/oya-foundry-eval-parity-analyzer-{kernel,domain,adapter-clickhouse,usecase}/`.

## Concrete File Targets

For each crate: `Cargo.toml + src/lib.rs + src/<modules>.rs + catalog/<crate>.yaml`.

### parity-analyzer-kernel

- `entities.rs`: `ParityReport`, `CohortDelta`, `InHouseCutoverVerdict`, `DpEpsilon`.
- `ports.rs`: `ParityAnalyzer`, `ParityAnalyticsStore`, `CutoverEligibilityEmitter`.
- `errors.rs`.

### parity-analyzer-domain

- `delta.rs`: per-cohort delta arithmetic; winning-margin.
- `dp_noise.rs`: ε-bounded DP aggregation per `policy/dp-analysis.md`.
- `cutover_eligibility.rs`: per-cohort dominance criteria.

### parity-analyzer-adapter-clickhouse

- `analytics_store.rs`: ClickHouse INSERT + rollup SELECT.
- `migrations.rs`: parity_analytics MergeTree schema; week-partition.
- HMAC computation per row.

### parity-analyzer-usecase

- `orchestrator.rs`: read two runs, compute delta, emit verdict.
- `cutover_decider.rs`: per-cohort dominance check.

## Test Plan

90% line/branch (domain); 90% line (usecase); 85% line (adapter); kernel 90/80.
