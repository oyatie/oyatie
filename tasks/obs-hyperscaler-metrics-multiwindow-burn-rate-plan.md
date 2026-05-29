# Plan: obs-hyperscaler-metrics-multiwindow-burn-rate

## Goal

Extend `oya-shared-hyperscaler-metrics-kernel` with a deterministic multi-window-multi-burn-rate
(MWMB) evaluator matching the canonical Google SRE alert pattern.

## Background

`assess_slo_burn` / `SloBurnAssessment` compute a single-window burn-rate. `SloBurnProfile` carries
`fast_burn_threshold_basis_points` (14x/1h) and `slow_burn_threshold_basis_points` (6x/6h) but
nothing combines a short and long window the way the canonical Google MWMB alert pattern requires.

The MWMB pattern requires that BOTH the short AND long window exceed the threshold before an alert
fires — this prevents noisy transient spikes from producing false positives.

## Tasks

1. Add `WindowEvents` struct (`total_events: u64, success_events: u64`)
2. Add `MultiWindowBurnAssessment` struct with both window assessments + alert flags
3. Add `assess_multiwindow_burn(short, long, profile)` function with profile validation
4. Add hermetic unit tests covering all acceptance criteria:
   - (a) both-windows-hot => fast_burn_alert true
   - (b) only-short-hot => fast_burn_alert false (alert suppressed)
   - (c) only-long-hot => false
   - (d) zero-traffic windows => no alert, no panic
   - (e) invalid profile => MetricsError::InvalidSloProfile

## Constraints

- Pure, deterministic, no new deps, no I/O
- Saturating integer math (no panics on zero-traffic)
- Reuse `assess_slo_burn` for per-window computation
- Validate profile via `SloBurnProfile::validate`
- No changes outside `oya-shared-hyperscaler-metrics-kernel`
