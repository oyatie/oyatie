# Spec: usage-window-burn-rate-forecast

## Objective

Extend `intelligence-usage-window-kernel` with a pure deterministic
burn-rate forecasting API. No I/O. No new crates. No new workspace members.

## Crate boundary

**Only crate modified**: `microservices/intelligence/crates/intelligence-usage-window-kernel`

The crate already re-exports `UsageWindow` / `UsageWindowKind` / `UsageWindowError`
from `intelligence-account-domain` and owns `UsageEnforcement`. The new
API is an additive `impl UsageEnforcement` method.

## Mod layout (flat-clean-arch, ADR-0509)

All code lives in `src/lib.rs`. No sub-modules needed for this slice — the crate
is a single-concern kernel with < 300 lines.

## New public surface

```rust
/// Projected exhaustion from a linear burn-rate extrapolation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExhaustionForecast {
    /// Seconds until `usage_limit_pct` is breached (capped at window end).
    UsageLimitEtaSecs(u64),
    /// Seconds until reserve floor is breached (capped at window end).
    /// Returned in preference to UsageLimitEtaSecs when it is smaller.
    ReserveBreachEtaSecs(u64),
    /// Burn rate is zero; no projected breach.
    NoBurn,
    /// A threshold is already breached as of `now`.
    AlreadyBreached,
    /// `now >= ends_at`; window is closed.
    WindowExpired,
}

/// Full forecast result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UsageForecast {
    pub exhaustion: ExhaustionForecast,
    /// Tokens consumed per second (integer, floor); 0 when elapsed == 0.
    pub burn_rate_tokens_per_sec: u64,
    /// Elapsed seconds since window start.
    pub elapsed_secs: u64,
}

impl UsageEnforcement {
    pub fn forecast(
        window: &UsageWindow,
        now_epoch_secs: u64,
        budget_tokens: u64,
    ) -> Result<UsageForecast, EnforcementError>;
}
```

## Algorithm

```
1. Guard: ends_at <= started_at         → Err(InvalidWindow)
2. Guard: budget_tokens == 0            → Err(InvalidWindow)
3. Guard: now < started_at              → Err(ClockBeforeWindowStart)
4. Guard: now >= ends_at                → Ok(WindowExpired)
5. elapsed      = now - started_at      (u64, always >= 0 after guard 3)
6. used         = tokens_in + tokens_out (saturating_add)
7. burn_rate    = if elapsed == 0 { 0 } else { used / elapsed }
8. if used == 0 || burn_rate == 0       → NoBurn
   (zero-burn branch: no projected breach)
9. Already-breached check:
   limit_tokens    = (budget * usage_limit_pct as u128 / 100) as u64
   reserve_tokens  = (budget * (100 - reserve_pct) as u128 / 100) as u64
   if used >= limit_tokens || used >= reserve_tokens → AlreadyBreached
10. remaining_wall  = ends_at - now
    limit_eta    = (limit_tokens - used) / burn_rate  (integer, capped at remaining_wall)
    reserve_eta  = (reserve_tokens - used) / burn_rate (integer, capped at remaining_wall)
    return min(limit_eta, reserve_eta) wrapped in the appropriate variant:
      - if min is reserve_eta  → ReserveBreachEtaSecs(reserve_eta)
      - else                   → UsageLimitEtaSecs(limit_eta)
```

## Contracts

- No OpenAPI / proto surface change (kernel is internal; consumed by route-policy).
- No SLO change (kernel has no SLO file; this slice adds no observability).

## Testing strategy

Table-driven unit tests in `src/lib.rs` under `#[cfg(test)]`:

| Test | What it verifies |
|------|-----------------|
| `forecast_constant_burn_exact_eta` | Exact ETA for steady-state burn |
| `forecast_eta_capped_at_wall_clock` | ETA never exceeds remaining wall seconds |
| `forecast_zero_usage_returns_no_burn` | Zero tokens → NoBurn |
| `forecast_already_breached_usage_limit` | Current breach → AlreadyBreached |
| `forecast_already_breached_reserve` | Reserve breach → AlreadyBreached |
| `forecast_window_expired` | now >= ends_at → WindowExpired |
| `forecast_clock_before_start` | now < start → ClockBeforeWindowStart |
| `forecast_zero_budget_invalid` | budget==0 → InvalidWindow |
| `forecast_ends_lte_start_invalid` | ends<=start → InvalidWindow |
| `forecast_reserve_breach_eta_before_limit_eta` | Reserve ETA < limit ETA when reserve floor higher |
| `forecast_saturating_u64_max_no_panic` | u64::MAX inputs don't panic |
| `forecast_zero_elapsed_returns_no_burn` | elapsed==0 → NoBurn regardless of used |
| `forecast_burn_rate_zero_after_elapsed` | used==0, elapsed>0 → NoBurn |

## Observability / SLO

No new SLO file required: this slice is a pure in-process kernel function
with no network surface. OTel instrumentation is the caller's concern
(route-policy-kernel layer).

## Crate boundary enforcement

No new `[dependencies]` added. `intelligence-account-domain` already
provides `UsageWindow`, `UsageWindowKind`, `UsageWindowError`. All new types
(`ExhaustionForecast`, `UsageForecast`) live in the kernel crate.
