# IP-010 — Kill-Switch Kernel Crate

**microservice**: feature-flags
**bc**: killswitch
**layer**: kernel
**crate**: oya-feature-flags-killswitch-kernel
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0244, ADR-0248, ADR-0252, ADR-0263, ADR-0295, ADR-0296, ADR-0298
**companion_ips**: IP-004, IP-011

## Scope

Life-safety kill-switch: engage/disengage, Kafka broadcast ≤1s to all cells, TrueTime-stamped events, Cedar step-up C enforcement, SPIFFE workload identity verification, hardcoded FORBID list for EMERGENCY_SERVICES flags.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `KillSwitchState` | Enum: `Engaged { timestamp_tt: TrueTimeInterval, engaged_by: Principal, scope: KillSwitchScope }` / `Disengaged` |
| 2 | `KillSwitchService` | `engage(scope, principal)` → Cedar step-up C check → broadcast to Kafka → TrueTime stamp; `disengage` requires step-up B |
| 3 | `KafkaBroadcastProducer` | Publishes to `oya.feature-flags.killswitch-engaged`; 50 partitions; `X-priority: HIGH` header; ≤1s fan-out to all cells SLO |
| 4 | Life-safety FORBID list | Cedar FORBID cannot disengage `NENA_I3_ROUTING`, `NCMEC_REPORTING`, `FEMA_BROADCAST` — these are permanently locked unless platform-safety-officer + warrant |
| 5 | SPIFFE verification | `engage()` verifies caller SPIFFE identity: `spiffe://oyatie.io/ns/feature-flags/*` |
| 6 | TrueTime stamping | All kill-switch events use TrueTime (`tt_earliest`, `tt_latest`) per ADR-0252 |
| 7 | Audit events | `KillSwitchEngaged` (SEV-1 tagged), `KillSwitchDisengaged`; sealed in audit chain |
| 8 | Tests | Fan-out latency simulation: 50 partitions × mock consumer ≤1s; SPIFFE mismatch → `Unauthorized`; life-safety FORBID test |

## Kill-Switch Scope

```rust
pub enum KillSwitchScope {
    Global,                     // All flags for all tenants
    Tenant(TenantId),           // All flags for one tenant
    Flag(TenantId, FlagKey),    // Single flag
    Pack(PackId),               // All flags under a compliance pack
}
```

## Definition of Done

- `cargo test -p oya-feature-flags-killswitch-kernel` green
- Kafka broadcast: 50-partition mock consumers all receive event ≤1s
- Life-safety FORBID: `NENA_I3_ROUTING` disengage without warrant → `Cedar::Deny`
- TrueTime: every event has `tt_earliest` ≤ `tt_latest`; interval ≤10ms
