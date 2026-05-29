//! M02-P01-IP-004 — Usage-window enforcement kernel.
//! Re-exports UsageWindow + UsageWindowKind from oya-intelligence-account-domain
//! and adds the enforcement port that route-policy consumes.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub use oya_intelligence_account_domain::{UsageWindow, UsageWindowError, UsageWindowKind};

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

pub struct UsageEnforcement;

impl UsageEnforcement {
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
