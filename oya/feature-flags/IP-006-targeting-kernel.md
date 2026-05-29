# IP-006 — Targeting Kernel Crate

**microservice**: feature-flags
**bc**: targeting
**layer**: kernel
**crate**: oya-feature-flags-targeting-kernel
**status**: design-ready
**acceptance_status**: design-ready
**adrs**: ADR-0105, ADR-0131, ADR-0159, ADR-0243, ADR-0244, ADR-0248, ADR-0292, ADR-0294
**companion_ips**: IP-002, IP-004, IP-007

## Scope

Targeting rule evaluation engine: audience_type routing, cohort membership, pack-mandated overrides, Cedar predicate evaluation (Wasm-isolated per ADR-0254). Soak window enforcement: Cedar fragment activation ≥60s after upload per ADR-0294.

## Deliverables

| # | Artifact | Acceptance Criterion |
|---|----------|---------------------|
| 1 | `AudienceTypeRouter` | Routes `EMERGENCY_SERVICES` to bypass path; `MINOR_TARGETED` enforces COPPA/KOSA checks per ADR-0292 |
| 2 | `CohortMatcher` | Evaluates cohort membership rules (attribute-based); tenant-scoped; O(log N) lookup via BTreeMap |
| 3 | `CedarPredicateEvaluator` | Wasm-isolated Cedar evaluation; soak window: rejects fragments uploaded <60s ago |
| 4 | `PackOverrideResolver` | Resolves pack-mandated targeting overrides; FORBID pack overrides disabling EMERGENCY_SERVICES flags |
| 5 | `TargetingRuleCache` | DashMap<(tenant_id, flag_key), Vec<TargetingRule>>; TTL 30s; invalidated by Kafka `flag-state-changed` |
| 6 | MINOR_TARGETED guard | If `audience_type == MINOR_TARGETED` and pack `oya-pack-eu-child-safety` not active → `FORBID` |
| 7 | Tests | EMERGENCY_SERVICES bypass unit test; soak window rejection (57s → fail, 61s → pass); cohort membership property tests |

## Cedar Soak Window Logic

```rust
fn check_soak_window(uploaded_at: Instant) -> Result<(), SoakWindowError> {
    let age = uploaded_at.elapsed();
    if age < Duration::from_secs(60) {
        return Err(SoakWindowError::TooFresh { age_s: age.as_secs() });
    }
    Ok(())
}
```

## Definition of Done

- `cargo test -p oya-feature-flags-targeting-kernel` green
- EMERGENCY_SERVICES never blocked by targeting rules
- Soak window: fragments <60s old rejected with `TooFresh` error
- Wasm isolation: Cedar predicates run in separate Wasm module instance
