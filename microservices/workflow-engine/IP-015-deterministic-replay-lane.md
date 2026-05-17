---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-015-deterministic-replay-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, oya-governance-per-microservice-layout]
---

# IP-015: oya-governance-deterministic-replay BLOCKER lane

## Intent

Implement the BLOCKER CI lane `oya-governance-deterministic-replay` that validates: for every workflow spec submitted in a PR (or already published), running the spec against its golden-input set produces an identical step sequence on every replay. Catches non-determinism regressions at PR-time before they reach production.

## ChangeSet boundary

One new lane validator crate + GitHub Actions workflow + golden-input set bootstrap.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-check-deterministic-replay/{Cargo.toml,src/lib.rs,src/main.rs}` | create | Lane validator binary; reads PR-affected specs, runs each through workflow-engine in a hermetic test container, verifies replay invariant |
| `.github/workflows/governance-deterministic-replay.yml` | create | CI workflow that runs the validator on every PR touching `microservices/workflow-engine/**` or any `**/*.workflow.yaml` |
| `microservices/workflow-engine/capabilities/eval/{workflow-execute,workflow-pause,workflow-replay}-golden.jsonl` | create | Golden input sets for the 3 capabilities (per `capabilities/*.yaml` `eval_set` references) |
| `Cargo.toml` (workspace) | update | register `oya-check-deterministic-replay` |

## Code Shape

```rust
// crates/oya-check-deterministic-replay/src/lib.rs
pub fn validate_deterministic_replay(spec: &WorkflowSpec, golden_inputs: &[GoldenInput])
    -> Result<ValidationResult, ValidationError> {
    for input in golden_inputs {
        // 1. Run spec against input
        let original_run = engine.execute(spec, input).await?;
        // 2. Replay the run from event log
        let replayed_steps = replay_engine.replay(&original_run.event_log)?;
        // 3. Verify identical
        if !replay_engine.verify_identical(&original_run.steps, &replayed_steps) {
            return Err(ValidationError::ReplayNonDeterministic);
        }
    }
    Ok(ValidationResult::Pass)
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-check-deterministic-replay --all-features
cargo run -p oya-check-deterministic-replay -- validate --spec <path> --golden <path>
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_deterministic_spec_passes` | well-formed spec passes |
| `test_non_deterministic_spec_fails` | spec using system-time / non-deterministic RNG fails |
| `test_lane_blocks_pr_on_failure` | CI workflow exits non-zero |

## Halt Conditions

- Any in-tree workflow spec fails the lane — block; fix spec or escalate to remove from registry.

## Next IP (terminal)

(none — phase exit gate)

## References

- PRD AC-02, AC-11, AC-15
- `policy/spec-integrity.md` §"Forbidden Spec Constructs"
- `backfill-replay.md`
- ADR-0130 §"Promotion gate"
