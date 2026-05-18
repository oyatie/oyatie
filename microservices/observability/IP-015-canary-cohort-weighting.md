---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-015-canary-cohort-weighting
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: Canary cohort weighting

## Intent

Service-mesh traffic-split per microservice via Istio VirtualService; progressive ramp 1 % → 10 % → 50 % → 100 % per `/specs/agentic-slo-gated-promotion.json` §"canary_cohort_weighting_finalized". Abort + drain on any burn-rate alert. Closes the FUTURE-stub decommission.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/observability/iac/kustomize/base/istio/virtualservice-template.yaml` | create — per-µservice VirtualService template with canary cohort |
| `microservices/observability/src/crates/oya-observability-slo-engine-worker/src/canary_controller.rs` | create — adjusts VirtualService weights based on burn-rate state |
| `microservices/observability/runbooks/canary-graduation.md` | already authored Slice B |
| `microservices/observability/tests/e2e/canary_ramp.rs` | create |

## Code Shape

```rust
// worker/src/canary_controller.rs
pub async fn step(deps: ControllerDeps, ms: &str) -> anyhow::Result<()> {
    let current_weight = deps.mesh_client.get_canary_weight(ms).await?;
    let burn_clean = deps.burn_rate_clean(ms).await?;
    if !burn_clean {
        deps.mesh_client.set_canary_weight(ms, 0).await?;  // abort + drain
        deps.event_dispatcher.dispatch_canary_aborted(ms, current_weight).await?;
        return Ok(());
    }
    let next_weight = match current_weight {
        0..=0 => 1,
        1..=9 => 10,
        10..=49 => 50,
        50..=99 => 100,
        _ => 100,
    };
    if next_weight != current_weight {
        deps.mesh_client.set_canary_weight(ms, next_weight).await?;
    }
    Ok(())
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-observability-slo-engine-worker --test canary_controller
helm lint microservices/observability/iac/kustomize/base/istio
kubectl --dry-run=client apply -f microservices/observability/iac/kustomize/base/istio/
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_canary_step_1_to_10` | clean signal → weight advances |
| `test_canary_abort_on_burn` | burn-rate alert → weight drains to 0 |
| `test_canary_min_duration_honored` | each step honors its `min_duration_seconds` |
| `e2e_canary_ramp` | scripted ramp 1→10→50→100 with clean signal completes |

## Halt Conditions

- Canary weight changes faster than `min_duration_seconds` — bug; fix
- Istio VirtualService apply fails — engage cloud-k8s µservice

## Next IP

(end of P01)

## References

- ADR-0130 §"Layer-B item 17 — Canary cohort weighting"
- `runbooks/canary-graduation.md`
- `/specs/agentic-slo-gated-promotion.json` §"canary_cohort_weighting_finalized"
