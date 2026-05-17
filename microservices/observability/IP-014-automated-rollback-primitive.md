---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-014-automated-rollback-primitive
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-nextest]
---

# IP-014: Automated rollback primitive

## Intent

When a production-tier fast-burn alert fires within 1h post-promotion, `slo-engine-worker` invokes `vcs rollback` against the affected `release/<ms>/production` ref, reverting to the prior pointer. Signed; audit-chain-sealed; emits `RollbackExecuted` event.

## Concrete File Targets

| Path | Action |
|---|---|
| `crates/oya-dev-cli/src/commands/vcs/rollback.rs` | create — `oya vcs rollback` subcommand |
| `microservices/observability/src/crates/oya-observability-slo-engine-worker/src/rollback_watcher.rs` | create — 1h post-promotion watcher loop |
| `microservices/observability/runbooks/rollback.md` | already authored Slice B |
| `microservices/observability/tests/e2e/rollback_drill.rs` | create — induce burn-rate → assert ref reverted |

## Code Shape

```rust
// worker/src/rollback_watcher.rs
pub async fn watch(deps: WatcherDeps) -> anyhow::Result<()> {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        for promotion in deps.recent_production_promotions(Duration::from_hours(1)).await? {
            if deps.production_fast_burn_breached(&promotion.microservice).await? {
                deps.cli.invoke_rollback(&promotion).await?;
            }
        }
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-observability-slo-engine-worker --test rollback_watcher
cargo run -p oya-dev-cli -- vcs rollback --microservice observability --env production --to-sha <prior-sha> --reason "fast-burn breach drill"
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_rollback_watcher_detects_breach` | post-promotion fast-burn → rollback invoked |
| `test_rollback_signs_payload` | Ed25519 signature attached |
| `e2e_rollback_drill` | inject burn-rate breach → production ref reverts within ≤60s; audit-chain seal recorded |

## Halt Conditions

- Rollback applied without audit-chain seal — fail
- Rollback chain depth > 1 (rollback-of-rollback) — escalate to ExecSponsor

## Next IP

[`IP-015-canary-cohort-weighting.md`](IP-015-canary-cohort-weighting.md)

## References

- ADR-0130 §"Layer-B item 16 — Automated rollback"
- `runbooks/rollback.md`
- `/specs/agentic-slo-gated-promotion.json` §"rollback_primitive"
