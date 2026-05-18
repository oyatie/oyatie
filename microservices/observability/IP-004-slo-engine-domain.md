---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-004-slo-engine-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-observability-slo-engine-domain

## Intent

Pure domain math: multi-window multi-burn-rate computation per Google SRE Workbook ch. 5. Pure functions over `kernel` entities; no I/O; verified against reference values.

## ChangeSet boundary

One new Rust crate; consumes `kernel`. Property-tested + reference-table-tested.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/observability/src/crates/oya-observability-slo-engine-domain/Cargo.toml` | create | depends on kernel |
| `.../src/lib.rs` | create | module surface |
| `.../src/burn_rate.rs` | create | multi-window burn-rate computation |
| `.../src/window.rs` | create | rolling-window arithmetic |
| `.../src/budget.rs` | create | error-budget computation |
| `Cargo.toml` (workspace) | update | add member path |
| `microservices/observability/catalog/oya-observability-slo-engine-domain.yaml` | create | catalog row |

## Crate Naming

```
NAME: oya-observability-slo-engine-domain
JUSTIFICATION:
- microservice = observability; bc-tokens = slo-engine; layer = domain
- pure logic over kernel types; no I/O
- exemptions claimed: none
```

## Code Shape

```rust
// src/burn_rate.rs
use oya_observability_slo_engine_kernel::{SloTarget, BurnRateSnapshot, BurnRateWindow};

/// Compute the burn rate over a window: (errors / window_seconds) / (error_budget / slo_window_seconds)
pub fn burn_rate(errors_in_window: f64, slo_target: &SloTarget, window: BurnRateWindow) -> f64 {
    // pure
    let allowed_error_rate = (1.0 - slo_target.target) / slo_target.window.duration_secs() as f64;
    let observed_error_rate = errors_in_window / window.duration_secs() as f64;
    observed_error_rate / allowed_error_rate
}

/// Snapshot the four canonical burn-rate windows (Google SRE Workbook ch. 5).
pub fn snapshot(target: &SloTarget, fast_1h: f64, slow_6h: f64, ticket_3d: f64, total_30d: f64) -> BurnRateSnapshot {
    BurnRateSnapshot {
        fast_burn_1h: burn_rate(fast_1h, target, BurnRateWindow::Fast1h),
        slow_burn_6h: burn_rate(slow_6h, target, BurnRateWindow::Slow6h),
        ticket_burn_3d: burn_rate(ticket_3d, target, BurnRateWindow::Ticket3d),
        budget_remaining_pct: (1.0 - total_30d / target.error_budget) * 100.0,
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-observability-slo-engine-domain --all-features
cargo nextest run -p oya-observability-slo-engine-domain --all-features
cargo clippy -p oya-observability-slo-engine-domain -- -D warnings
```

## Test Plan

Per PHASE-01 domain class: 1 test per public function + property tests for math. Coverage 95% line / 90% branch.

| Test | Verifies |
|---|---|
| `test_burn_rate_pure` | f(input)→output deterministic |
| `test_burn_rate_google_sre_reference` | matches SRE Workbook ch. 5 worked examples (2%/1h ⇒ 14.4×; 5%/6h ⇒ 6×; 10%/3d ⇒ 3×) |
| `prop_burn_rate_monotonic` | proptest: more errors ⇒ higher burn rate |
| `prop_budget_remaining_bounded` | proptest: budget remaining ∈ [0, 100] |
| `test_window_arithmetic` | window-overlap + duration math |

## Halt Conditions

- Reference-value mismatch with SRE Workbook — fix math, do not adjust tests
- Any I/O reachable — refactor to usecase

## Next IP

[`IP-005-slo-engine-usecase.md`](IP-005-slo-engine-usecase.md)

## References

- Google SRE Workbook ch. 5 §"Multiwindow, Multi-Burn-Rate Alerts"
- `/specs/agentic-slo-gated-promotion.json` §"openslo_manifest_profile.alert_burn_rates"
