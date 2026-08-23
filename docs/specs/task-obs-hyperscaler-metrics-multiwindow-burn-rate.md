# Spec: obs-hyperscaler-metrics-multiwindow-burn-rate

## Summary

Add a pure deterministic multi-window-multi-burn-rate (MWMB) evaluator to the
`shared-hyperscaler-metrics-kernel` crate, implementing the canonical Google SRE alert pattern
referenced in the PrometheusRule doc-comments.

## Motivation

The canonical PrometheusRule fires `OyaErrorBudgetFastBurn1h14x` and `OyaErrorBudgetSlowBurn6h6x`
alerts. Today `SloBurnAssessment` / `assess_slo_burn` compute per-window burn-rates, and
`SloBurnProfile` carries fast/slow thresholds, but nothing in the kernel combines two windows into
the required AND-gate that is the key safety property of MWMB alerting: an alert fires ONLY when
BOTH the short and long windows simultaneously exceed the threshold. This prevents a short transient
spike from producing a false positive alert.

## API Surface

### `WindowEvents`

```rust
pub struct WindowEvents {
    pub total_events: u64,    // data_class: INTERNAL_ONLY
    pub success_events: u64,  // data_class: INTERNAL_ONLY
}
```

Carries the raw event counts for one observation window (e.g. 1h or 6h). Used as inputs to
`assess_multiwindow_burn`.

### `MultiWindowBurnAssessment`

```rust
pub struct MultiWindowBurnAssessment {
    pub short_window: SloBurnAssessment,  // data_class: INTERNAL_ONLY
    pub long_window: SloBurnAssessment,   // data_class: INTERNAL_ONLY
    pub fast_burn_alert: bool,            // data_class: INTERNAL_ONLY
    pub slow_burn_alert: bool,            // data_class: INTERNAL_ONLY
}
```

`fast_burn_alert` is true IFF both `short_window.burn_rate_basis_points` AND
`long_window.burn_rate_basis_points` are >= `profile.fast_burn_threshold_basis_points`.

`slow_burn_alert` is true IFF both exceed `profile.slow_burn_threshold_basis_points`.

### `assess_multiwindow_burn`

```rust
pub fn assess_multiwindow_burn(
    short: WindowEvents,
    long: WindowEvents,
    profile: SloBurnProfile,
) -> Result<MultiWindowBurnAssessment, MetricsError>
```

1. Calls `profile.validate()` — returns `MetricsError::InvalidSloProfile` on invalid profile.
2. Calls `assess_slo_burn(short.total_events, short.success_events, profile)` for the short window.
3. Calls `assess_slo_burn(long.total_events, long.success_events, profile)` for the long window.
4. Computes `fast_burn_alert` as the AND of both windows exceeding `fast_burn_threshold_basis_points`.
5. Computes `slow_burn_alert` as the AND of both windows exceeding `slow_burn_threshold_basis_points`.
6. Returns `Ok(MultiWindowBurnAssessment { short_window, long_window, fast_burn_alert, slow_burn_alert })`.

Integer math is saturating throughout (delegated to `assess_slo_burn`). Zero-traffic windows return
`burn_rate_basis_points = 0` (no alert) — no division by zero, no panic.

## Acceptance Criteria

| Case | short window | long window | expected fast_burn_alert | expected slow_burn_alert |
|------|-------------|-------------|--------------------------|--------------------------|
| (a) both hot | >14x | >14x | true | true |
| (b) only short hot | >14x | normal | false | false |
| (c) only long hot | normal | >14x | false | false |
| (d) zero traffic | 0/0 | 0/0 | false | false |
| (e) invalid profile | — | — | Err(InvalidSloProfile) | — |

## Layer / Dependency Constraints

- Layer: `domain` (kernel) — pure value types, no I/O, no external crate deps.
- Reuses `assess_slo_burn` internally. No new dependencies in `Cargo.toml`.
- Saturating integer arithmetic only — no `unwrap()` on division, no panics in production paths.

## References

- `microservices/observability/iac/helm/observability/templates/hyperscaler-invariants-canonical-prometheusrule.yaml`
- Google SRE Workbook, Chapter 5: Alerting on SLOs — Multi-window, multi-burn-rate alerts
- ADR-0056, ADR-0128, ADR-0130
