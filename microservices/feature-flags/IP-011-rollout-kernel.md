# IP-011 — Rollout Kernel Crate

**microservice**: feature-flags
**bc**: rollout
**layer**: kernel
**crate**: oya-feature-flags-rollout-kernel
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0160, ADR-0243, ADR-0248, ADR-0263
**companion_ips**: IP-002, IP-006, IP-012

## Scope

Progressive delivery gating: SLO-gated stage advance (0/1/10/50/100%), automatic rollback on SLO breach, Flagger integration per ADR-0160, CI-controlled stage advance via `ci-scope.cedar`.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `RolloutStage` | Enum: `Dark(0) → Canary(1) → EarlyAdopter(10) → Majority(50) → Full(100)`; percentage stored in `FlagDefinition.rollout_stage` |
| 2 | `SloGate` | Reads SLO burn-rate state from observability substrate; blocks advance when burn rate > 5× (slow-burn threshold) |
| 3 | `RolloutAdvanceService` | Cedar `ci-scope` `RolloutAdvance` permit + `slo_gate_passed=true` context; advances to next stage |
| 4 | `RolloutRollbackService` | Cedar `ci-scope` `RolloutRollback` permit + `slo_breach_detected=true` context; reverts to previous stage |
| 5 | `FlaggerIntegration` | Emits `RolloutStageChanged` event consumed by Flagger for Kubernetes progressive delivery |
| 6 | Audit events | `RolloutAdvanced`, `RolloutRolledBack` — include `slo_gate_result` payload |
| 7 | Tests | SLO breach → auto-rollback within 30s; CI without `slo_gate_passed=true` → `Cedar::Deny` for advance |

## Rollout Flow

```
CI pipeline → evaluate SLO gates → Cedar RolloutAdvance → stage++ → FlaggerIntegration
                                         ↓
                                  slo_breach_detected → Cedar RolloutRollback → stage--
```

## Definition of Done

- `cargo test -p oya-feature-flags-rollout-kernel` green
- Advance without `slo_gate_passed=true` in Cedar context → `Deny`
- Rollback test: inject SLO breach signal → stage decrements within 30s
- `FlaggerIntegration` emits `RolloutStageChanged` event on every advance/rollback
