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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: Automated rollback primitive

## Intent

When a production-tier fast-burn alert fires within 1h post-promotion, `slo-engine-worker` opens a rollback pull request against `dev` that pins the affected service back to the prior signed image digest/release pointer. Jenkins promotion checks and ArgoCD rollout health gate the rollback; the action is signed, audit-chain-sealed, and emits `RollbackExecuted`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/observability/src/crates/oya-observability-slo-engine-worker/src/rollback_pr.rs` | create — rollback PR builder using plain `git` + Jenkins promotion checks |
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
git switch -c rollback/observability-<incident-id> dev
buck2 build //:repo-hygiene-automation-check --ci-required
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

- ADR-0139 §"Layer-B item 16 — Automated rollback"
- `runbooks/rollback.md`
- `/specs/agentic-slo-gated-promotion.json` §"rollback_primitive"
