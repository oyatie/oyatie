# IP-026 — Kill-Switch Broadcast Worker

**microservice**: feature-flags
**bc**: killswitch
**layer**: worker
**crate**: oya-feature-flags-killswitch-worker
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0248, ADR-0252, ADR-0263, ADR-0295, ADR-0298
**companion_ips**: IP-010, IP-019

## Scope

Dedicated Kafka consumer worker for kill-switch broadcast: consumes `oya.feature-flags.killswitch-engaged`, applies state to all cells within ≤1s SLO, writes to local flag-state cache, emits OTEL trace spans per cell, triggers SEV-1 alert on SLO breach.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `KillSwitchBroadcastWorker` | Kafka consumer group `feature-flags-killswitch-workers`; processes all 50 partitions across cell instances |
| 2 | Cell-local cache write | Writes `KillSwitchState::Engaged` to `FlagCache` DashMap within 50ms of Kafka message receipt |
| 3 | Cross-cell acknowledgement | Publishes `KillSwitchCellAcknowledged` event per cell to `oya.feature-flags.killswitch-cell-ack` topic |
| 4 | SLO enforcement | If any cell ACK not received within 800ms: emit `KillSwitchSLOBreach` metric; fire PagerDuty SEV-1 |
| 5 | EMERGENCY_SERVICES override | Even when worker is degraded, local cache `engaged=true` is the safe default (fail-closed) |
| 6 | TrueTime verification | Rejects kill-switch messages where `tt_latest < now - 5s` (stale message protection) |
| 7 | Tests | 50-partition consumer test; SLO breach alert fires at 801ms; fail-closed test: worker down → cache stays engaged |

## Fail-Closed Invariant

```rust
// On worker startup or reconnect: default state is ENGAGED for any flag
// that had a kill-switch event in the last 5 minutes.
// This ensures brief worker outages do not silently re-enable flags.
fn safe_default_on_startup(flag_key: &str) -> KillSwitchState {
    match self.last_known_state(flag_key) {
        Some(state) => state,
        None => KillSwitchState::Disengaged, // only if no prior event
    }
}
```

## Definition of Done

- `cargo test -p oya-feature-flags-killswitch-worker` green
- 50-partition simulation: all partitions consumed within 200ms
- SLO breach alert: mock 801ms ACK delay → `KillSwitchSLOBreach` metric emitted
- Fail-closed: worker restart with prior `Engaged` event → cache restores `Engaged` state
