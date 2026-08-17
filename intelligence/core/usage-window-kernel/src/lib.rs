//! M02-P01-IP-004 — Usage-window enforcement kernel.
//! Re-exports UsageWindow + UsageWindowKind from oya-intelligence-account-domain
//! and adds the enforcement port that route-policy consumes.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use intelligence_account_domain::{UsageWindow, UsageWindowError, UsageWindowKind};

/// Verdict returned by UsageEnforcement::check_limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnforcementVerdict {
    /// Window has headroom; usage_pct < usage_limit_pct AND reserve held.
    WithinLimit { usage_pct: u8, headroom_pct: u8 },
    /// Window over its configured usage_limit_pct ceiling.
    OverUsageLimit { usage_pct: u8, limit_pct: u8 },
    /// Reserve floor breached (reserve_remaining_pct can't be honoured).
    ReserveBreached { remaining_pct: u8, reserve_pct: u8 },
    /// Wall-clock past ends_at; window is closed.
    WindowExpired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnforcementError {
    InvalidWindow(String),
    ClockBeforeWindowStart,
    /// Supplied window slice was empty.
    NoWindows,
    /// A single window's check_limit call failed; aggregation aborted.
    WindowFailed {
        window_kind: UsageWindowKind,
        window_index: usize,
        source: Box<EnforcementError>,
    },
}

/// Identifies which window produced the most-restrictive verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VerdictProvenance {
    pub window_kind: UsageWindowKind,
    pub window_index: usize,
}

/// Result of a multi-window aggregation: the most-restrictive verdict and its provenance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnforcedDecision {
    pub verdict: EnforcementVerdict,
    pub provenance: VerdictProvenance,
}

/// Projected exhaustion from a linear burn-rate extrapolation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExhaustionForecast {
    /// Seconds until `usage_limit_pct` is breached (capped at window end).
    UsageLimitEtaSecs(u64),
    /// Seconds until reserve floor is breached (capped at window end).
    /// Returned when it is smaller than the usage-limit ETA.
    ReserveBreachEtaSecs(u64),
    /// Burn rate is zero; no projected breach.
    NoBurn,
    /// A threshold is already breached as of `now`.
    AlreadyBreached,
    /// `now >= ends_at`; window is closed.
    WindowExpired,
}

/// Full forecast result from a linear burn-rate projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UsageForecast {
    pub exhaustion: ExhaustionForecast,
    /// Tokens consumed per second (integer floor); 0 when elapsed == 0.
    pub burn_rate_tokens_per_sec: u64,
    /// Elapsed seconds since window start.
    pub elapsed_secs: u64,
}

pub struct UsageEnforcement;

impl UsageEnforcement {
    /// Linear burn-rate forecast: projects seconds until `usage_limit_pct` or
    /// reserve floor is breached, capped at the remaining wall-clock to `ends_at`.
    ///
    /// Reuses all existing window-validity guards (ends_at>started_at, clock>=start,
    /// expiry) and error variants. Zero-burn and zero-elapsed handled explicitly.
    /// All arithmetic is saturating; no panics on `u64::MAX` inputs.
    pub fn forecast(
        window: &UsageWindow,
        now_epoch_secs: u64,
        budget_tokens: u64,
    ) -> Result<UsageForecast, EnforcementError> {
        // Guard 1: invalid window geometry
        if window.ends_at_epoch_secs <= window.started_at_epoch_secs {
            return Err(EnforcementError::InvalidWindow(
                "ends_at <= started_at".to_owned(),
            ));
        }
        // Guard 2: zero budget
        if budget_tokens == 0 {
            return Err(EnforcementError::InvalidWindow(
                "budget_tokens must be > 0".to_owned(),
            ));
        }
        // Guard 3: clock before window
        if now_epoch_secs < window.started_at_epoch_secs {
            return Err(EnforcementError::ClockBeforeWindowStart);
        }
        // Guard 4: window already expired
        if now_epoch_secs >= window.ends_at_epoch_secs {
            return Ok(UsageForecast {
                exhaustion: ExhaustionForecast::WindowExpired,
                burn_rate_tokens_per_sec: 0,
                elapsed_secs: now_epoch_secs.saturating_sub(window.started_at_epoch_secs),
            });
        }

        let elapsed_secs = now_epoch_secs.saturating_sub(window.started_at_epoch_secs);
        let used = window.tokens_in.saturating_add(window.tokens_out);

        // Burn rate: integer tokens/sec; 0 when elapsed == 0 (division guard).
        let burn_rate = if elapsed_secs == 0 {
            0u64
        } else {
            used / elapsed_secs
        };

        // Zero-burn branch
        if burn_rate == 0 {
            return Ok(UsageForecast {
                exhaustion: ExhaustionForecast::NoBurn,
                burn_rate_tokens_per_sec: 0,
                elapsed_secs,
            });
        }

        // Compute thresholds using u128 to avoid intermediate overflow.
        let budget_u128 = budget_tokens as u128;
        let limit_tokens =
            ((budget_u128 * window.usage_limit_pct as u128) / 100).min(budget_u128) as u64;
        let reserve_floor_tokens =
            ((budget_u128 * (100u128 - window.reserve_remaining_pct.min(100) as u128)) / 100)
                .min(budget_u128) as u64;

        // Already-breached check (either threshold).
        if used >= limit_tokens || used >= reserve_floor_tokens {
            return Ok(UsageForecast {
                exhaustion: ExhaustionForecast::AlreadyBreached,
                burn_rate_tokens_per_sec: burn_rate,
                elapsed_secs,
            });
        }

        // Remaining wall-clock seconds to window end.
        let remaining_wall = window.ends_at_epoch_secs.saturating_sub(now_epoch_secs);

        // ETA to usage-limit breach (capped at wall clock).
        let limit_headroom = limit_tokens.saturating_sub(used);
        let limit_eta = (limit_headroom / burn_rate).min(remaining_wall);

        // ETA to reserve-floor breach (capped at wall clock).
        let reserve_headroom = reserve_floor_tokens.saturating_sub(used);
        let reserve_eta = (reserve_headroom / burn_rate).min(remaining_wall);

        // Return the earliest (smallest) ETA.
        let exhaustion = if reserve_eta < limit_eta {
            ExhaustionForecast::ReserveBreachEtaSecs(reserve_eta)
        } else {
            ExhaustionForecast::UsageLimitEtaSecs(limit_eta)
        };

        Ok(UsageForecast {
            exhaustion,
            burn_rate_tokens_per_sec: burn_rate,
            elapsed_secs,
        })
    }

    /// Rank a verdict for most-restrictive comparison (higher = more restrictive).
    fn verdict_rank(v: &EnforcementVerdict) -> u8 {
        match v {
            EnforcementVerdict::WithinLimit { .. } => 0,
            EnforcementVerdict::OverUsageLimit { .. } => 1,
            EnforcementVerdict::ReserveBreached { .. } => 2,
            EnforcementVerdict::WindowExpired => 3,
        }
    }

    /// Aggregate over an ordered set of `(UsageWindow, budget_tokens)` pairs,
    /// returning the most-restrictive `EnforcedDecision` + its provenance.
    /// Short-circuits with `WindowFailed` on the first `check_limit` error.
    /// Returns `NoWindows` if the slice is empty.
    pub fn check_limits(
        windows: &[(UsageWindow, u64)],
        now_epoch_secs: u64,
    ) -> Result<EnforcedDecision, EnforcementError> {
        if windows.is_empty() {
            return Err(EnforcementError::NoWindows);
        }
        let mut best: Option<EnforcedDecision> = None;
        for (index, (window, budget)) in windows.iter().enumerate() {
            let verdict = Self::check_limit(window, now_epoch_secs, *budget).map_err(|e| {
                EnforcementError::WindowFailed {
                    window_kind: window.kind,
                    window_index: index,
                    source: Box::new(e),
                }
            })?;
            let candidate = EnforcedDecision {
                verdict,
                provenance: VerdictProvenance {
                    window_kind: window.kind,
                    window_index: index,
                },
            };
            best = Some(match best {
                None => candidate,
                Some(current) => {
                    if Self::verdict_rank(&candidate.verdict) > Self::verdict_rank(&current.verdict)
                    {
                        candidate
                    } else {
                        current
                    }
                }
            });
        }
        Ok(best.expect("non-empty slice guarantees Some"))
    }

    /// Pure check — no time travel; caller passes wall-clock.
    pub fn check_limit(
        window: &UsageWindow,
        now_epoch_secs: u64,
        budget_tokens: u64,
    ) -> Result<EnforcementVerdict, EnforcementError> {
        if window.ends_at_epoch_secs <= window.started_at_epoch_secs {
            return Err(EnforcementError::InvalidWindow(
                "ends_at <= started_at".to_owned(),
            ));
        }
        if now_epoch_secs < window.started_at_epoch_secs {
            return Err(EnforcementError::ClockBeforeWindowStart);
        }
        if now_epoch_secs >= window.ends_at_epoch_secs {
            return Ok(EnforcementVerdict::WindowExpired);
        }
        let used = window.tokens_in.saturating_add(window.tokens_out);
        if budget_tokens == 0 {
            return Err(EnforcementError::InvalidWindow(
                "budget_tokens must be > 0".to_owned(),
            ));
        }
        let usage_pct = ((used as u128 * 100) / budget_tokens as u128).min(100) as u8;
        let remaining_pct = 100u8.saturating_sub(usage_pct);

        if remaining_pct < window.reserve_remaining_pct {
            return Ok(EnforcementVerdict::ReserveBreached {
                remaining_pct,
                reserve_pct: window.reserve_remaining_pct,
            });
        }
        if usage_pct >= window.usage_limit_pct {
            return Ok(EnforcementVerdict::OverUsageLimit {
                usage_pct,
                limit_pct: window.usage_limit_pct,
            });
        }
        Ok(EnforcementVerdict::WithinLimit {
            usage_pct,
            headroom_pct: remaining_pct,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(
        kind: UsageWindowKind,
        used_in: u64,
        used_out: u64,
        limit: u8,
        reserve: u8,
    ) -> UsageWindow {
        let mut w = UsageWindow::new(kind, 0, 18_000, limit, reserve).unwrap();
        w.tokens_in = used_in;
        w.tokens_out = used_out;
        w
    }

    // -----------------------------------------------------------------------
    // Forecast tests (burn-rate-forecast slice)
    // -----------------------------------------------------------------------

    /// Helper: build a window with explicit start/end for forecast tests.
    fn forecast_window(
        started_at: u64,
        ends_at: u64,
        tokens_in: u64,
        tokens_out: u64,
        limit_pct: u8,
        reserve_pct: u8,
    ) -> UsageWindow {
        let mut w = UsageWindow::new(
            UsageWindowKind::FiveHour,
            started_at,
            ends_at,
            limit_pct,
            reserve_pct,
        )
        .unwrap();
        w.tokens_in = tokens_in;
        w.tokens_out = tokens_out;
        w
    }

    #[test]
    fn forecast_constant_burn_exact_eta() {
        // Window: start=0, end=10_000 ; budget=1_000 ; limit=80% → 800 tokens
        // At now=100: elapsed=100, used=200 → burn_rate=2 tok/s
        // Headroom to limit: 800-200 = 600 → ETA = 600/2 = 300s
        // Reserve=0% so no reserve breach concern
        let w = forecast_window(0, 10_000, 100, 100, 80, 0);
        let f = UsageEnforcement::forecast(&w, 100, 1_000).unwrap();
        assert_eq!(f.burn_rate_tokens_per_sec, 2);
        assert_eq!(f.elapsed_secs, 100);
        assert_eq!(f.exhaustion, ExhaustionForecast::UsageLimitEtaSecs(300));
    }

    #[test]
    fn forecast_eta_capped_at_wall_clock() {
        // Window: start=0, end=200 ; budget=1_000_000 ; limit=80%
        // now=100: elapsed=100, used=1 → burn_rate=0 → NoBurn (rate is 0 after floor)
        // Use higher burn: used=100 tok, budget=10_000, limit=80% (8000 tokens)
        // Headroom: 7900, burn=1/s → ETA=7900 but remaining_wall=200-100=100 → capped at 100
        let w = forecast_window(0, 200, 50, 50, 80, 0);
        let f = UsageEnforcement::forecast(&w, 100, 10_000).unwrap();
        // burn_rate = 100/100 = 1 tok/s
        // limit_tokens = 10_000 * 80 / 100 = 8_000
        // headroom = 8_000 - 100 = 7_900 → ETA = 7_900 / 1 = 7_900 → capped at 100
        assert_eq!(f.exhaustion, ExhaustionForecast::UsageLimitEtaSecs(100));
    }

    #[test]
    fn forecast_zero_usage_returns_no_burn() {
        let w = forecast_window(0, 18_000, 0, 0, 80, 5);
        let f = UsageEnforcement::forecast(&w, 100, 1_000).unwrap();
        assert_eq!(f.exhaustion, ExhaustionForecast::NoBurn);
        assert_eq!(f.burn_rate_tokens_per_sec, 0);
    }

    #[test]
    fn forecast_zero_elapsed_returns_no_burn() {
        // now == started_at; elapsed == 0 regardless of used
        let w = forecast_window(100, 18_000, 500, 500, 80, 5);
        let f = UsageEnforcement::forecast(&w, 100, 1_000).unwrap();
        assert_eq!(f.exhaustion, ExhaustionForecast::NoBurn);
        assert_eq!(f.elapsed_secs, 0);
        assert_eq!(f.burn_rate_tokens_per_sec, 0);
    }

    #[test]
    fn forecast_burn_rate_zero_after_elapsed() {
        // elapsed=100, used=0 → burn_rate=0 → NoBurn
        let w = forecast_window(0, 18_000, 0, 0, 80, 5);
        let f = UsageEnforcement::forecast(&w, 100, 1_000).unwrap();
        assert_eq!(f.exhaustion, ExhaustionForecast::NoBurn);
        assert_eq!(f.burn_rate_tokens_per_sec, 0);
    }

    #[test]
    fn forecast_already_breached_usage_limit() {
        // used=850, budget=1_000, limit=80% → 800 tokens; 850 >= 800 → already breached
        let w = forecast_window(0, 18_000, 850, 0, 80, 5);
        let f = UsageEnforcement::forecast(&w, 100, 1_000).unwrap();
        assert_eq!(f.exhaustion, ExhaustionForecast::AlreadyBreached);
    }

    #[test]
    fn forecast_already_breached_reserve() {
        // reserve_pct=10 → reserve floor = 10% of budget = 100 tokens reserved
        // used=950, budget=1_000 → remaining=50 < reserve=100 → already breached
        let w = forecast_window(0, 18_000, 950, 0, 100, 10);
        let f = UsageEnforcement::forecast(&w, 100, 1_000).unwrap();
        assert_eq!(f.exhaustion, ExhaustionForecast::AlreadyBreached);
    }

    #[test]
    fn forecast_window_expired() {
        let w = forecast_window(0, 18_000, 100, 100, 80, 5);
        let f = UsageEnforcement::forecast(&w, 20_000, 1_000).unwrap();
        assert_eq!(f.exhaustion, ExhaustionForecast::WindowExpired);
    }

    #[test]
    fn forecast_clock_before_start() {
        let w = forecast_window(100, 18_000, 10, 10, 80, 5);
        let err = UsageEnforcement::forecast(&w, 50, 1_000).unwrap_err();
        assert_eq!(err, EnforcementError::ClockBeforeWindowStart);
    }

    #[test]
    fn forecast_zero_budget_invalid() {
        let w = forecast_window(0, 18_000, 0, 0, 80, 5);
        let err = UsageEnforcement::forecast(&w, 100, 0).unwrap_err();
        assert!(matches!(err, EnforcementError::InvalidWindow(_)));
    }

    #[test]
    fn forecast_ends_lte_start_invalid() {
        // Construct manually with invalid window (ends_at == started_at)
        // Can't use UsageWindow::new (it validates), so use a valid window then mutate
        let mut w = forecast_window(0, 18_000, 0, 0, 80, 5);
        w.ends_at_epoch_secs = 0; // same as started_at → invalid
        let err = UsageEnforcement::forecast(&w, 100, 1_000).unwrap_err();
        assert!(matches!(err, EnforcementError::InvalidWindow(_)));
    }

    #[test]
    fn forecast_reserve_breach_eta_before_limit_eta() {
        // budget=10_000, limit=90% (9_000 tokens), reserve=20% (floor = 2_000 tokens reserved)
        // reserve_tokens = 10_000 * (100-20) / 100 = 8_000
        // used=100 in 100s → burn=1 tok/s
        // limit headroom: 9_000 - 100 = 8_900 → ETA_limit = 8_900  (capped at wall=9_900)
        // reserve headroom: 8_000 - 100 = 7_900 → ETA_reserve = 7_900 (capped at 9_900)
        // reserve_eta (7_900) < limit_eta (8_900) → must return ReserveBreachEtaSecs
        let w = forecast_window(0, 10_000, 50, 50, 90, 20);
        let f = UsageEnforcement::forecast(&w, 100, 10_000).unwrap();
        assert_eq!(
            f.exhaustion,
            ExhaustionForecast::ReserveBreachEtaSecs(7_900)
        );
    }

    #[test]
    fn forecast_saturating_u64_max_no_panic() {
        // Feeding u64::MAX tokens should not panic; saturating arithmetic required.
        let mut w = forecast_window(0, 18_000, 0, 0, 80, 5);
        w.tokens_in = u64::MAX;
        w.tokens_out = u64::MAX;
        // Just must not panic
        let _ = UsageEnforcement::forecast(&w, 100, u64::MAX);
    }

    #[test]
    fn within_limit_basic() {
        let w = window(UsageWindowKind::FiveHour, 100, 50, 80, 10);
        let v = UsageEnforcement::check_limit(&w, 100, 1_000).unwrap();
        assert!(matches!(
            v,
            EnforcementVerdict::WithinLimit {
                usage_pct: 15,
                headroom_pct: 85
            }
        ));
    }

    #[test]
    fn over_usage_limit() {
        let w = window(UsageWindowKind::OneWeek, 850, 0, 80, 5);
        let v = UsageEnforcement::check_limit(&w, 100, 1_000).unwrap();
        assert!(matches!(v, EnforcementVerdict::OverUsageLimit { .. }));
    }

    #[test]
    fn reserve_breached() {
        let w = window(UsageWindowKind::Project, 950, 0, 100, 10);
        let v = UsageEnforcement::check_limit(&w, 100, 1_000).unwrap();
        match v {
            EnforcementVerdict::ReserveBreached {
                remaining_pct,
                reserve_pct,
            } => {
                assert!(remaining_pct < reserve_pct);
            }
            other => panic!("expected ReserveBreached, got {other:?}"),
        }
    }

    #[test]
    fn window_expired_when_clock_past_end() {
        let w = window(UsageWindowKind::FiveHour, 100, 100, 80, 10);
        let v = UsageEnforcement::check_limit(&w, 20_000, 1_000).unwrap();
        assert_eq!(v, EnforcementVerdict::WindowExpired);
    }

    #[test]
    fn clock_before_window_rejected() {
        let mut w = UsageWindow::new(UsageWindowKind::FiveHour, 100, 200, 80, 10).unwrap();
        w.tokens_in = 10;
        assert_eq!(
            UsageEnforcement::check_limit(&w, 50, 1_000),
            Err(EnforcementError::ClockBeforeWindowStart),
        );
    }

    #[test]
    fn zero_budget_rejected() {
        let w = window(UsageWindowKind::FiveHour, 0, 0, 80, 10);
        assert!(matches!(
            UsageEnforcement::check_limit(&w, 100, 0),
            Err(EnforcementError::InvalidWindow(_)),
        ));
    }

    #[test]
    fn five_hour_one_week_project_all_supported() {
        for kind in [
            UsageWindowKind::FiveHour,
            UsageWindowKind::OneWeek,
            UsageWindowKind::Project,
        ] {
            let w = window(kind, 10, 10, 80, 5);
            let v = UsageEnforcement::check_limit(&w, 100, 1_000).unwrap();
            assert!(matches!(v, EnforcementVerdict::WithinLimit { .. }));
        }
    }
}
