---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-010-replay-debugger-backend-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: oya-workflow-engine-replay-debugger-backend-{kernel,domain}

## Intent

Scaffold replay-debugger-backend kernel + domain. Kernel: port traits (`EventLogReader`, `ReplayEngine`, `RunAnalyticsRepository`) + entities (`ReplaySession`, `StepSnapshot`, `RunAnalytics`). Domain: pure replay logic over event log (deterministic).

## ChangeSet boundary

2 new crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-replay-debugger-backend-kernel/{...}` | create | port traits + entities |
| `src/crates/oya-workflow-engine-replay-debugger-backend-domain/{...}` | create | pure replay engine |
| `microservices/workflow-engine/catalog/oya-workflow-engine-replay-debugger-backend-{kernel,domain}.yaml` | create | 2 catalog rows |

## Code Shape

```rust
pub struct ReplayEngine;

impl ReplayEngine {
    pub fn replay(&self, event_log: &[WorkflowEvent], from_step: u32, to_step: u32)
        -> Result<Vec<StepSnapshot>, ReplayError> {
        // Pure logic: reconstruct state at each event boundary
        // Returns: vec of (state_before, state_after, step_index, ...)
        // Deterministic; no I/O
    }

    pub fn verify_identical(&self, original: &[StepSnapshot], replayed: &[StepSnapshot]) -> bool {
        // Verify step sequence matches; field-by-field comparison
    }
}
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_replay_deterministic` (property) | same event log → same step sequence |
| `test_replay_identical_to_original` | replay matches original run snapshot |
| `test_replay_partial_range` | from_step + to_step bounds respected |
| `test_replay_throughput_baseline` | ≥ 1000 steps/s/CPU baseline (informational) |

## Next IP

[`IP-011-replay-debugger-backend-usecase-adapter.md`](IP-011-replay-debugger-backend-usecase-adapter.md)

## References

- PRD AC-02, AC-11
- `backfill-replay.md`
