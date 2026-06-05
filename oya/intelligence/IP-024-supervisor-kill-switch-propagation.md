---
doc_class: ImplementationPlan
milestone: M01-foundation
phase: P01-control-plane-landing
impl_plan_id: IP-009-kill-switch-propagation
status: pending
execution_unit: ChangeSet
owner: axis-foundry-control-plane
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, oya-check-kill-switch-latency-p99]
depends_on: [IP-008]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: Sub-second kill-switch propagation (p99 ≤ 1 s)

## Intent

Implement sub-second propagation to in-flight foundry-runtime workers via dual channels:
1. **Primary**: Kubernetes CRD watch (`KillSwitch` CR) — foundry-runtime workers subscribe to the watch.
2. **Fallback**: Valkey pub-sub on `kill-switch-events:<tenant>` channel — covers CRD-watch delays.

Worker + adapter-k8s-operator crates for kill-switch-circuit-breaker BC.

## Concrete File Targets

- `…-worker/Cargo.toml` + `src/lib.rs` + `src/propagation.rs`
- `…-adapter-k8s-operator/Cargo.toml` + `src/lib.rs` + `src/crd_watch_fan_out.rs`

## Key code

```rust
// worker/src/propagation.rs
pub async fn propagate_engage_event(
    event: &KillSwitchEngagedEvent,
    crd_propagator: &dyn KillSwitchCrdPropagator,
    redis_pubsub: &dyn KillSwitchRedisPubSub,
) -> Result<PropagationLatency, KernelError> {
    let start = Instant::now();

    // Dual-channel: fire both in parallel
    let (crd_res, redis_res) = tokio::join!(
        crd_propagator.update_crd(&event.scope, &event.target, true),
        redis_pubsub.publish(&event.scope, &event.target, true),
    );

    crd_res?; redis_res?;
    Ok(start.elapsed())
}
```

## Acceptance Gates

```bash
# Drill: 100k workers; engage; verify p99 ≤ 1 s
cargo nextest run -p oya-foundry-supervisor-kill-switch-circuit-breaker-worker --test kill_switch_latency_at_scale
buck2 build //:quality-lane-registry-authority-check # lane=kill-switch-latency-p99 --microservice foundry-supervisor
```

## Test Plan

| Test | Verifies |
|---|---|
| `kill_switch_latency_at_scale` | end-to-end p99 ≤ 1 s for 100k workers (AC-02) |
| `crd_watch_propagation` | CRD update reaches workers within p99 ≤ 500 ms |
| `redis_pubsub_fallback` | Valkey pub-sub reaches workers within p99 ≤ 200 ms |
| `dual_channel_redundancy` | One channel down still keeps p99 ≤ 1 s |
| `fail_closed_on_both_unreachable` | If both channels unreachable, workers assume engaged |

## Halt Conditions

- p99 > 1 s in test.
- Either channel breaks fail-closed invariant.

## Next IP

[`IP-010-fleet-state-postgres-adapter.md`](IP-010-fleet-state-postgres-adapter.md)

## References

- ADR-0133 §"Hyperscaler safety-claim parity (AWS Bedrock Guardrails ~2 s p99)".
- PRD §"Performance" kill-switch row.
- `runbooks/kill-switch-engage.md`.

## Wave 15 counterpart anchor

- Counterparts: Palantir AIP Operator, Azure AI Foundry deployments, and GitHub merge-queue controls.
- Gap closure: this IP closes fleet control, kill-switch propagation, and deployability evidence with tenant-scoped policy enforcement.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
