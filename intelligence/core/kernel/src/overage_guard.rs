//! SC8 overage-guard — pure circuit-breaker brains for the OAuth subscription
//! pool (parity row XPROXY-OBS-003).
//!
//! Two independent trip signals feed one [`GuardDecision`]:
//!
//! 1. **Anthropic representative-claim overage.** The
//!    `anthropic-ratelimit-unified-representative-claim` response header names
//!    the rate-limit window that is the binding constraint for the request.
//!    The four subscription buckets
//!    (`five_hour`, `seven_day`, `five_hour_fallback`, `seven_day_fallback`)
//!    mean *normal* usage — keep serving. Absent / empty / `unknown` is a
//!    transient, unobservable signal — also keep serving (never halt on a
//!    signal we cannot trust). Anything else (`overage`, `api`, or a
//!    future bucket the platform has not yet allow-listed) means the seat is
//!    serving outside the paid subscription envelope and the guard trips.
//!
//! 2. **Per-credential quota exhaustion.** Codex returns
//!    `error.type == "usage_limit_reached"` with a `resets_at` /
//!    `resets_in_seconds` hint when a credential's quota window is spent.
//!    That is a hard, non-discretionary halt with a provider-supplied resume
//!    horizon. The *transient* `rate_limit_error` is explicitly excluded — it
//!    flows through the existing 429 cooldown path, not the overage guard.
//!
//! Warn-vs-enforce is [policy-as-data](OverageGuardPolicy): in `Warn` mode an
//! Anthropic overage emits an observability event and keeps forwarding; in
//! `Enforce` mode it halts the seat. Quota exhaustion always halts (the
//! credential genuinely cannot serve — warn mode does not apply).
//!
//! This module is pure kernel code: no I/O, no clock, no wall-clock parsing.
//! The header string and the already-relative `resets_in_seconds` are supplied
//! by the REST/Codex adapters; mapping an absolute `resets_at` wall-clock to a
//! relative horizon is an adapter concern (the kernel only sees monotonic
//! [`Instant`]s). Event emission is likewise the adapter's job — the kernel
//! only decides *whether* to warn/halt and hands back the reason.

use std::time::{Duration, Instant};

/// Default auto-resume cooldown for a halted seat when the provider does not
/// supply an explicit reset horizon. Operators may resume earlier via
/// admin-resume; this is the upper bound before the seat self-heals
/// (cooldown-resume).
pub const DEFAULT_RESUME_COOLDOWN: Duration = Duration::from_secs(30 * 60);

/// The four representative-claim buckets that denote normal subscription usage.
/// A response carrying one of these keeps the seat serving.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RepresentativeClaim {
    FiveHour,
    SevenDay,
    FiveHourFallback,
    SevenDayFallback,
}

impl RepresentativeClaim {
    /// The canonical lowercase header token for this bucket.
    pub const fn as_str(self) -> &'static str {
        match self {
            RepresentativeClaim::FiveHour => "five_hour",
            RepresentativeClaim::SevenDay => "seven_day",
            RepresentativeClaim::FiveHourFallback => "five_hour_fallback",
            RepresentativeClaim::SevenDayFallback => "seven_day_fallback",
        }
    }
}

/// Classification of the `anthropic-ratelimit-unified-representative-claim`
/// response header value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimClassification {
    /// Header absent, empty, or `unknown` — transient / unobservable. Never halt.
    Transient,
    /// A recognised normal-usage bucket. Never halt.
    Allowed(RepresentativeClaim),
    /// Any other bucket (`overage`, `api`, or a value not yet allow-listed) —
    /// the seat is serving outside its representative claim.
    Overage { bucket: String },
}

/// Classify a representative-claim header value.
///
/// The comparison is whitespace-trimmed and ASCII-case-insensitive. `None`,
/// empty/whitespace-only, and `unknown` all map to [`ClaimClassification::Transient`]
/// so the guard never halts on a signal it cannot trust.
pub fn classify_representative_claim(header: Option<&str>) -> ClaimClassification {
    let Some(raw) = header else {
        return ClaimClassification::Transient;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return ClaimClassification::Transient;
    }
    let lowered = trimmed.to_ascii_lowercase();
    match lowered.as_str() {
        "unknown" => ClaimClassification::Transient,
        "five_hour" => ClaimClassification::Allowed(RepresentativeClaim::FiveHour),
        "seven_day" => ClaimClassification::Allowed(RepresentativeClaim::SevenDay),
        "five_hour_fallback" => ClaimClassification::Allowed(RepresentativeClaim::FiveHourFallback),
        "seven_day_fallback" => ClaimClassification::Allowed(RepresentativeClaim::SevenDayFallback),
        _ => ClaimClassification::Overage { bucket: lowered },
    }
}

/// Guard enforcement mode (policy-as-data).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardMode {
    /// Observability-only: an Anthropic overage emits an event but the seat
    /// keeps forwarding.
    Warn,
    /// An Anthropic overage halts the seat.
    Enforce,
}

/// Per (tenant, provider) overage-guard policy. Carried as data so the control
/// plane can flip warn/enforce and tune the resume cooldown without code
/// changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OverageGuardPolicy {
    pub mode: GuardMode,
    /// Upper-bound auto-resume horizon for a halted seat when the provider does
    /// not supply its own reset hint.
    pub resume_cooldown: Duration,
}

impl Default for OverageGuardPolicy {
    /// Fail-closed: enforce by default with the 30-minute resume cooldown.
    fn default() -> Self {
        Self {
            mode: GuardMode::Enforce,
            resume_cooldown: DEFAULT_RESUME_COOLDOWN,
        }
    }
}

impl OverageGuardPolicy {
    pub fn enforce() -> Self {
        Self::default()
    }

    pub fn warn() -> Self {
        Self {
            mode: GuardMode::Warn,
            resume_cooldown: DEFAULT_RESUME_COOLDOWN,
        }
    }

    pub fn with_resume_cooldown(mut self, resume_cooldown: Duration) -> Self {
        self.resume_cooldown = resume_cooldown;
        self
    }
}

/// Why a seat was warned or halted by the overage guard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HaltReason {
    /// Anthropic representative-claim moved into an overage / non-subscription
    /// bucket. Carries the offending bucket token for observability.
    RepresentativeClaimOverage { bucket: String },
    /// Provider reported the per-credential quota window is exhausted
    /// (e.g. Codex `usage_limit_reached`).
    QuotaExhausted,
}

/// The guard's verdict for one response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuardDecision {
    /// Normal / transient — keep forwarding, no state change, no event.
    Continue,
    /// Overage detected but policy is warn-only — keep forwarding and emit one
    /// observability event carrying the reason.
    Warn { reason: HaltReason },
    /// Halt the seat. `resume_at == None` means admin-resume only; `Some(t)`
    /// means auto-resume (cooldown-resume) once `now >= t`, and an operator may
    /// still resume earlier.
    Halt {
        reason: HaltReason,
        resume_at: Option<Instant>,
    },
}

impl GuardDecision {
    /// True when this decision changes seat state (i.e. is a halt).
    pub fn is_halt(&self) -> bool {
        matches!(self, GuardDecision::Halt { .. })
    }
}

/// Evaluate the Anthropic representative-claim header against policy.
///
/// Transient and allowed buckets [`Continue`](GuardDecision::Continue). An
/// overage bucket [`Warn`](GuardDecision::Warn)s under [`GuardMode::Warn`] and
/// [`Halt`](GuardDecision::Halt)s under [`GuardMode::Enforce`] with a
/// `now + resume_cooldown` horizon.
pub fn evaluate_representative_claim(
    policy: OverageGuardPolicy,
    header: Option<&str>,
    now: Instant,
) -> GuardDecision {
    match classify_representative_claim(header) {
        ClaimClassification::Transient | ClaimClassification::Allowed(_) => GuardDecision::Continue,
        ClaimClassification::Overage { bucket } => {
            let reason = HaltReason::RepresentativeClaimOverage { bucket };
            match policy.mode {
                GuardMode::Warn => GuardDecision::Warn { reason },
                GuardMode::Enforce => GuardDecision::Halt {
                    reason,
                    resume_at: now.checked_add(policy.resume_cooldown),
                },
            }
        }
    }
}

/// Per-credential quota signal parsed from a Codex error envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodexQuotaSignal {
    /// Not a quota-exhaustion signal. Includes the transient `rate_limit_error`,
    /// which is handled by the 429 cooldown path, not the overage guard.
    None,
    /// `usage_limit_reached` — the credential's quota window is exhausted, with
    /// an optional provider-supplied relative reset horizon.
    Exhausted { resets_in_seconds: Option<u64> },
}

/// Classify a Codex `error.type`. Only `usage_limit_reached` is a quota
/// exhaustion; `rate_limit_error` and everything else are
/// [`CodexQuotaSignal::None`].
///
/// `resets_in_seconds` is the already-relative horizon (the adapter converts an
/// absolute `resets_at` wall-clock into seconds-from-now before calling in).
pub fn classify_codex_error(
    error_type: Option<&str>,
    resets_in_seconds: Option<u64>,
) -> CodexQuotaSignal {
    match error_type.map(str::trim) {
        Some("usage_limit_reached") => CodexQuotaSignal::Exhausted { resets_in_seconds },
        _ => CodexQuotaSignal::None,
    }
}

/// Map a [`CodexQuotaSignal`] to a [`GuardDecision`].
///
/// Quota exhaustion always halts — warn mode does not apply, because the
/// credential genuinely cannot serve. The resume horizon is the provider's
/// `resets_in_seconds` when present, else `policy.resume_cooldown`.
pub fn evaluate_codex_quota(
    policy: OverageGuardPolicy,
    signal: &CodexQuotaSignal,
    now: Instant,
) -> GuardDecision {
    match signal {
        CodexQuotaSignal::None => GuardDecision::Continue,
        CodexQuotaSignal::Exhausted { resets_in_seconds } => {
            let horizon = resets_in_seconds
                .map(Duration::from_secs)
                .unwrap_or(policy.resume_cooldown);
            GuardDecision::Halt {
                reason: HaltReason::QuotaExhausted,
                resume_at: now.checked_add(horizon),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_buckets_are_classified_as_allowed() {
        for (token, expected) in [
            ("five_hour", RepresentativeClaim::FiveHour),
            ("seven_day", RepresentativeClaim::SevenDay),
            ("five_hour_fallback", RepresentativeClaim::FiveHourFallback),
            ("seven_day_fallback", RepresentativeClaim::SevenDayFallback),
        ] {
            assert_eq!(
                classify_representative_claim(Some(token)),
                ClaimClassification::Allowed(expected)
            );
            // Case- and whitespace-insensitive.
            let noisy = format!("  {}  ", token.to_ascii_uppercase());
            assert_eq!(
                classify_representative_claim(Some(&noisy)),
                ClaimClassification::Allowed(expected)
            );
        }
    }

    #[test]
    fn absent_empty_and_unknown_are_transient() {
        assert_eq!(
            classify_representative_claim(None),
            ClaimClassification::Transient
        );
        assert_eq!(
            classify_representative_claim(Some("")),
            ClaimClassification::Transient
        );
        assert_eq!(
            classify_representative_claim(Some("   ")),
            ClaimClassification::Transient
        );
        assert_eq!(
            classify_representative_claim(Some("unknown")),
            ClaimClassification::Transient
        );
        assert_eq!(
            classify_representative_claim(Some("UNKNOWN")),
            ClaimClassification::Transient
        );
    }

    #[test]
    fn other_buckets_are_overage() {
        for token in ["overage", "api", "monthly", "some_future_bucket"] {
            assert_eq!(
                classify_representative_claim(Some(token)),
                ClaimClassification::Overage {
                    bucket: token.to_string()
                }
            );
        }
    }

    #[test]
    fn warn_mode_never_halts_on_overage() {
        let now = Instant::now();
        let decision =
            evaluate_representative_claim(OverageGuardPolicy::warn(), Some("overage"), now);
        assert_eq!(
            decision,
            GuardDecision::Warn {
                reason: HaltReason::RepresentativeClaimOverage {
                    bucket: "overage".to_string()
                }
            }
        );
        assert!(!decision.is_halt());
    }

    #[test]
    fn enforce_mode_halts_on_overage_with_default_cooldown() {
        let now = Instant::now();
        let decision =
            evaluate_representative_claim(OverageGuardPolicy::enforce(), Some("overage"), now);
        assert_eq!(
            decision,
            GuardDecision::Halt {
                reason: HaltReason::RepresentativeClaimOverage {
                    bucket: "overage".to_string()
                },
                resume_at: Some(now + DEFAULT_RESUME_COOLDOWN),
            }
        );
    }

    #[test]
    fn allowed_and_transient_never_halt_in_either_mode() {
        let now = Instant::now();
        for mode in [OverageGuardPolicy::warn(), OverageGuardPolicy::enforce()] {
            for header in [None, Some(""), Some("unknown"), Some("five_hour")] {
                assert_eq!(
                    evaluate_representative_claim(mode, header, now),
                    GuardDecision::Continue
                );
            }
        }
    }

    #[test]
    fn codex_usage_limit_reached_halts_with_provider_horizon() {
        let now = Instant::now();
        let signal = classify_codex_error(Some("usage_limit_reached"), Some(120));
        assert_eq!(
            signal,
            CodexQuotaSignal::Exhausted {
                resets_in_seconds: Some(120)
            }
        );
        assert_eq!(
            evaluate_codex_quota(OverageGuardPolicy::enforce(), &signal, now),
            GuardDecision::Halt {
                reason: HaltReason::QuotaExhausted,
                resume_at: Some(now + Duration::from_secs(120)),
            }
        );
    }

    #[test]
    fn codex_usage_limit_without_hint_falls_back_to_policy_cooldown() {
        let now = Instant::now();
        let signal = classify_codex_error(Some("usage_limit_reached"), None);
        assert_eq!(
            evaluate_codex_quota(OverageGuardPolicy::enforce(), &signal, now),
            GuardDecision::Halt {
                reason: HaltReason::QuotaExhausted,
                resume_at: Some(now + DEFAULT_RESUME_COOLDOWN),
            }
        );
    }

    #[test]
    fn codex_rate_limit_error_is_transient_not_a_halt() {
        // Transient rate limiting must NOT enter the overage guard — it rides
        // the existing 429 cooldown path.
        let signal = classify_codex_error(Some("rate_limit_error"), Some(5));
        assert_eq!(signal, CodexQuotaSignal::None);
        assert_eq!(
            evaluate_codex_quota(OverageGuardPolicy::enforce(), &signal, Instant::now()),
            GuardDecision::Continue
        );
    }

    #[test]
    fn codex_unknown_and_absent_error_types_are_not_a_halt() {
        for et in [None, Some("server_error"), Some("invalid_request_error")] {
            assert_eq!(classify_codex_error(et, None), CodexQuotaSignal::None);
        }
    }

    #[test]
    fn warn_policy_is_enforce_by_default_failclosed() {
        assert_eq!(OverageGuardPolicy::default().mode, GuardMode::Enforce);
        assert_eq!(
            OverageGuardPolicy::default().resume_cooldown,
            DEFAULT_RESUME_COOLDOWN
        );
    }
}
