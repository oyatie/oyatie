---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-011-worker-binaries
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, cloud-iac-drift-detection-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: cloud-iac-iac-*-worker crates (all 5 BCs)

## Intent

Implement `-worker` crates for all 5 BCs: long-lived loops that drive the pipeline. renderer-worker (consumes RenderRequested), validator-worker (continuous drift detection ≤1h cycle), applier-worker (consumes EligibilityChanged + ApplyRequested), rollback-worker (consumes RollbackExecuted from observability), registry-worker (catalog maintenance + provenance verification).

## ChangeSet boundary

Five new crates per ADR-0105: one `-worker` per BC. Catalog rows.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-iac/src/crates/cloud-iac-iac-renderer-worker/{Cargo.toml,src/lib.rs,src/main_loop.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-validator-worker/{Cargo.toml,src/lib.rs,src/drift_loop.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-applier-worker/{Cargo.toml,src/lib.rs,src/apply_loop.rs,src/eligibility_consumer.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-rollback-worker/{Cargo.toml,src/lib.rs,src/rollback_consumer.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-registry-worker/{Cargo.toml,src/lib.rs,src/registry_loop.rs}` | create |
| `microservices/cloud-iac/catalog/cloud-iac-iac-*-worker.yaml` | create (5 rows) |

## Code Shape

```rust
// validator-worker/src/drift_loop.rs
pub async fn run(deps: DriftWatcherDeps) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));  // ≤1h cycle per PRD
    loop {
        interval.tick().await;
        // Acquire lease (HA leader election)
        if !deps.lease.acquire().await? { continue; }

        for ms in deps.microservices.list_active().await? {
            for pack in deps.active_packs() {
                for env in [Environment::Staging, Environment::Production] {
                    match deps.validator.detect_drift(&ms, &pack, env).await {
                        Ok(report) if !report.drift_items.is_empty() => {
                            deps.event_emitter.emit_drift_detected(&report).await?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
```

```rust
// applier-worker/src/eligibility_consumer.rs
// Subscribes to observability's EligibilityChanged events; when verdict=eligible,
// applies the µservice's IaC at the eligible SHA.
pub async fn consume(deps: ConsumerDeps) -> anyhow::Result<()> {
    let mut stream = deps.event_bus.subscribe("workflow-events/eligibility.changed").await?;
    while let Some(event) = stream.next().await {
        if let Verdict::Eligible = event.verdict {
            let job = ApplyJob::from_eligibility(&event);
            deps.applier.apply(&job).await?;
        }
    }
    Ok(())
}
```

## Acceptance Gates

```bash
cargo check --workspace -p cloud-iac-iac-*-worker --all-features
cargo nextest run --workspace -p cloud-iac-iac-*-worker --all-features
cloud-ci/ci governance gate `drift-detection-coverage` for --microservice cloud-iac is green in the branch-protected `presubmit` context
```

## Test Plan

Per PHASE-01 worker class: 1 test per orchestration arm + ≥ 1 long-lived loop integration + 1 e2e. Coverage 85% line / 75% branch.

| Test | Verifies |
|---|---|
| `test_renderer_worker_consumes_render_requested` | event → render orchestrator invoked |
| `test_validator_drift_loop_cycle` | 1h cycle completes; reports emitted |
| `test_applier_consumes_eligibility_changed_eligible` | verdict eligible → apply orchestrator invoked |
| `test_rollback_consumer_rollback_executed` | observability rollback → iac-rollback |
| `e2e_60s_drift_cycle` | inject drift → DriftDetected event emitted within 60s |

## Halt Conditions

- Drift cycle exceeds ≤1h target — fix.
- Worker not HA-elected — fix.

## Next IP

[`IP-012-app-composition-roots.md`](IP-012-app-composition-roots.md)

## References

- ADR-0105.
- PRD §"Performance Targets" (drift cycle ≤1h).
- ADR-0139 (eligibility-changed event source).
