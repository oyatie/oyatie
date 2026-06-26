//! Request-level resilience for the cloud-intelligence subscription-OAuth pool.
//!
//! This module is **pure kernel code**: it expresses the complete request-level
//! retry/fallback *decision logic* as deterministic predicates over an
//! [`ErrorClass`] plus the request's [`AttemptState`]. It performs **zero I/O** —
//! no clock, no sleep, no network, no rng. The caller (a REST/proxy adapter)
//! samples the wall clock, samples the jitter source, performs the actual sleep,
//! and drives the bounded ladder; the kernel only *decides* what should happen
//! next so the policy is unit/property testable in isolation.
//!
//! Two cooperating capabilities:
//!
//! * [`RetryPolicy`] — the bounded resilience **ladder**:
//!   `in-seat retry -> seat rotation -> provider fallback -> graceful 503`.
//!   Honors an upstream `Retry-After` header (delta-seconds *and* HTTP-date
//!   forms per RFC 7231 §7.1.3), with jittered exponential backoff (AWS
//!   full/equal jitter) when the server gives no hint. Retry-After takes
//!   precedence over the computed backoff and is clamped to a policy ceiling so
//!   a hostile/buggy upstream cannot pin a worker indefinitely.
//!
//! * [`FallbackChain`] — per-model failover chain with **context-window
//!   fallback**: when a request exceeds the current model's context window the
//!   ladder escalates to the next model in the chain whose window fits.
//!
//! [`ErrorClass`] bridges back to the pool state machine via
//! [`ErrorClass::seat_outcome`], keeping the kernel the single source of truth
//! for "what does this upstream failure mean" rather than duplicating that
//! mapping in every adapter.

use std::time::Duration;

use crate::{Provider, SeatOutcome};

// ---------------------------------------------------------------------------
// ErrorClass — the pure input to every resilience decision
// ---------------------------------------------------------------------------

/// Platform-invariant classification of a single upstream attempt failure.
///
/// Adapters translate provider-specific HTTP statuses / SDK errors into one of
/// these before consulting the [`RetryPolicy`]. The retry ladder, the seat
/// state machine, and telemetry all branch on this single taxonomy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorClass {
    /// HTTP 429. The seat's quota / rate budget is exhausted; the right move is
    /// to rotate to another seat rather than hammer the same credential.
    /// `retry_after` carries any parsed `Retry-After` hint for telemetry and
    /// for the eventual graceful-degradation response.
    RateLimited { retry_after: Option<Duration> },
    /// HTTP 5xx (>= 500). A transient upstream fault; the *same* seat is likely
    /// healthy and may recover, so an in-seat retry is attempted first.
    ServerError {
        status: u16,
        retry_after: Option<Duration>,
    },
    /// Network timeout / connection reset / read timeout. Transient and
    /// seat-local; retry the same seat with backoff.
    Timeout,
    /// OAuth token-refresh transient failure (e.g. secret-provider blip). The
    /// seat cannot mint a token *right now*; rotate to a seat with a live token.
    RefreshTransient,
    /// OAuth refresh token permanently revoked. The seat is dead for the
    /// foreseeable future; rotate away (the pool will blacklist it).
    RefreshRevoked,
    /// The request's prompt exceeds the selected model's context window.
    /// Not a seat fault — escalate to a larger-context model via the
    /// [`FallbackChain`].
    ContextWindowExceeded,
    /// Authorization forbade the request (D7 forbid-wins). Terminal → 403.
    Forbidden,
    /// A non-retryable client error (4xx other than 429/403). Terminal; the
    /// upstream status is propagated to the caller unchanged.
    ClientError { status: u16 },
}

/// Internal ladder disposition for an [`ErrorClass`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Disposition {
    /// Stop immediately and return this HTTP status to the client.
    Terminal(u16),
    /// Escalate to a larger-context model (no seat penalty).
    ContextWindow,
    /// Transient and seat-local: try the same seat again before rotating.
    SeatLocal,
    /// The seat itself cannot serve this request: rotate without in-seat retry.
    SeatExhausted,
}

impl ErrorClass {
    /// Classify an upstream HTTP status into an [`ErrorClass`].
    ///
    /// Returns `None` for success / redirect statuses (`< 400`), mirroring the
    /// adapter's `Ok` bucket. Context-window exhaustion cannot be derived from
    /// the status line alone (it is body-shaped), so adapters construct
    /// [`ErrorClass::ContextWindowExceeded`] directly when they detect it.
    pub fn from_http_status(status: u16) -> Option<Self> {
        Self::from_http_response(status, None)
    }

    /// Classify an upstream HTTP status together with an already-parsed
    /// `Retry-After` duration (see [`parse_retry_after`]).
    pub fn from_http_response(status: u16, retry_after: Option<Duration>) -> Option<Self> {
        match status {
            0..=399 => None,
            403 => Some(ErrorClass::Forbidden),
            408 => Some(ErrorClass::Timeout),
            429 => Some(ErrorClass::RateLimited { retry_after }),
            400..=499 => Some(ErrorClass::ClientError { status }),
            _ => Some(ErrorClass::ServerError {
                status,
                retry_after,
            }),
        }
    }

    fn disposition(&self) -> Disposition {
        match self {
            ErrorClass::RateLimited { .. }
            | ErrorClass::RefreshTransient
            | ErrorClass::RefreshRevoked => Disposition::SeatExhausted,
            ErrorClass::ServerError { .. } | ErrorClass::Timeout => Disposition::SeatLocal,
            ErrorClass::ContextWindowExceeded => Disposition::ContextWindow,
            ErrorClass::Forbidden => Disposition::Terminal(403),
            ErrorClass::ClientError { status } => Disposition::Terminal(*status),
        }
    }

    /// The upstream `Retry-After` hint carried by this error, if any.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            ErrorClass::RateLimited { retry_after }
            | ErrorClass::ServerError { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// True if any rung of the ladder may still recover from this error.
    pub fn is_retryable(&self) -> bool {
        !matches!(self.disposition(), Disposition::Terminal(_))
    }

    /// Bridge to the pool state machine. Keeps the kernel the single source of
    /// truth for "what does this upstream failure do to a seat":
    ///
    /// * rate-limit / 5xx / timeout → cooldown-inducing seat penalties,
    /// * refresh transient/revoked → the matching refresh outcomes,
    /// * forbidden / client-error / context-window → [`SeatOutcome::Released`]
    ///   (the seat is healthy and must not be penalized for a request-shaped
    ///   problem the seat did not cause).
    pub fn seat_outcome(&self) -> SeatOutcome {
        match self {
            ErrorClass::RateLimited { .. } => SeatOutcome::RateLimited429,
            ErrorClass::ServerError { .. } | ErrorClass::Timeout => SeatOutcome::ServerError5xx,
            ErrorClass::RefreshTransient => SeatOutcome::RefreshFailed,
            ErrorClass::RefreshRevoked => SeatOutcome::RefreshTokenRevoked,
            ErrorClass::ContextWindowExceeded
            | ErrorClass::Forbidden
            | ErrorClass::ClientError { .. } => SeatOutcome::Released,
        }
    }
}

// ---------------------------------------------------------------------------
// AttemptState — the bounded ladder's position
// ---------------------------------------------------------------------------

/// Mutable position within the bounded resilience ladder for a single inbound
/// request. The kernel reads it to decide the next rung; the caller mutates it
/// as it climbs (the `record_*` helpers keep the counters consistent).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AttemptState {
    /// Retries already performed against the *current* seat.
    pub in_seat_attempts: u32,
    /// Seats already abandoned (rotated away from) for the *current* provider.
    pub seats_tried: u32,
    /// Providers already abandoned (fallen back from) for this request.
    pub providers_tried: u32,
}

impl AttemptState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an in-seat retry: the same seat is tried again.
    pub fn record_same_seat_retry(&mut self) {
        self.in_seat_attempts = self.in_seat_attempts.saturating_add(1);
    }

    /// Record a seat rotation: abandon the current seat, reset its retry budget.
    pub fn record_seat_rotation(&mut self) {
        self.seats_tried = self.seats_tried.saturating_add(1);
        self.in_seat_attempts = 0;
    }

    /// Record a provider fallback: abandon the current provider, reset the
    /// per-provider seat and in-seat budgets.
    pub fn record_provider_fallback(&mut self) {
        self.providers_tried = self.providers_tried.saturating_add(1);
        self.seats_tried = 0;
        self.in_seat_attempts = 0;
    }
}

// ---------------------------------------------------------------------------
// RetryAction — the kernel's verdict for one rung
// ---------------------------------------------------------------------------

/// The next action the caller should take. Exactly one rung of the ladder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RetryAction {
    /// Wait `delay`, then retry the *same* seat. `delay` already honors any
    /// `Retry-After` hint (clamped to the policy ceiling) or jittered
    /// exponential backoff.
    RetrySameSeat { delay: Duration },
    /// Abandon the current seat; select another eligible seat in the same
    /// provider pool.
    RotateSeat,
    /// The provider pool is exhausted; advance to the next provider in the
    /// [`FallbackChain`].
    FallbackProvider,
    /// The request exceeds the current model's context window; escalate to a
    /// larger-context model via the [`FallbackChain`].
    EscalateContextWindow,
    /// Terminal. Respond to the client with `http_status` (graceful
    /// degradation — `503` once the ladder is exhausted, or the propagated
    /// upstream status for a non-retryable error).
    Fail { http_status: u16 },
}

// ---------------------------------------------------------------------------
// JitterKind — backoff jitter strategy
// ---------------------------------------------------------------------------

/// Jitter applied to the exponential backoff to avoid thundering-herd
/// synchronization across a fleet of workers. See the AWS Architecture Blog,
/// "Exponential Backoff And Jitter".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JitterKind {
    /// No jitter: `delay = capped`. Deterministic; mostly for tests.
    None,
    /// Full jitter: `delay = random_in(0, capped)`.
    Full,
    /// Equal jitter: `delay = capped/2 + random_in(0, capped/2)`.
    Equal,
}

// ---------------------------------------------------------------------------
// RetryPolicy — the bounded ladder
// ---------------------------------------------------------------------------

/// Policy-as-data configuration for the resilience ladder. Every bound is
/// explicit so it can be sourced from per-tenant policy without code changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    /// Max retries against a single seat before rotating.
    pub max_in_seat_retries: u32,
    /// Max seats to rotate through within one provider before falling back.
    pub max_seat_rotations: u32,
    /// Max providers to fall back through before giving up with 503.
    pub max_provider_fallbacks: u32,
    /// Base backoff for the first in-seat retry.
    pub base_backoff: Duration,
    /// Upper bound on a single computed backoff.
    pub max_backoff: Duration,
    /// Exponential base (typically 2). `delay = base * multiplier^attempt`.
    pub backoff_multiplier: u32,
    /// Hard ceiling on a server-supplied `Retry-After` wait — a buggy or
    /// hostile upstream must not be able to pin a worker indefinitely.
    pub retry_after_ceiling: Duration,
    /// Jitter strategy for computed (non-`Retry-After`) backoff.
    pub jitter: JitterKind,
}

impl Default for RetryPolicy {
    /// Enterprise defaults: 2 in-seat retries, 3 seat rotations, 2 provider
    /// fallbacks, 200ms..20s full-jittered exponential backoff, 60s
    /// `Retry-After` ceiling.
    fn default() -> Self {
        Self {
            max_in_seat_retries: 2,
            max_seat_rotations: 3,
            max_provider_fallbacks: 2,
            base_backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(20),
            backoff_multiplier: 2,
            retry_after_ceiling: Duration::from_secs(60),
            jitter: JitterKind::Full,
        }
    }
}

impl RetryPolicy {
    /// Decide the next rung of the ladder for `error` at ladder position
    /// `state`. `jitter_unit` is a caller-sampled uniform value in `[0, 1]`
    /// (clamped); it is only consulted for computed backoff, and ignored when
    /// the server supplied a `Retry-After`. Pure and deterministic in its
    /// inputs.
    pub fn decide(
        &self,
        error: &ErrorClass,
        state: &AttemptState,
        jitter_unit: f64,
    ) -> RetryAction {
        match error.disposition() {
            Disposition::Terminal(status) => RetryAction::Fail {
                http_status: status,
            },
            Disposition::ContextWindow => RetryAction::EscalateContextWindow,
            Disposition::SeatLocal => {
                if state.in_seat_attempts < self.max_in_seat_retries {
                    RetryAction::RetrySameSeat {
                        delay: self.backoff(state.in_seat_attempts, error.retry_after(), jitter_unit),
                    }
                } else {
                    self.escalate_from_seat(state)
                }
            }
            Disposition::SeatExhausted => self.escalate_from_seat(state),
        }
    }

    /// The seat cannot serve (or has exhausted its in-seat retries): rotate,
    /// fall back, or give up gracefully with 503.
    fn escalate_from_seat(&self, state: &AttemptState) -> RetryAction {
        if state.seats_tried < self.max_seat_rotations {
            RetryAction::RotateSeat
        } else if state.providers_tried < self.max_provider_fallbacks {
            RetryAction::FallbackProvider
        } else {
            RetryAction::Fail { http_status: 503 }
        }
    }

    /// Compute the wait before an in-seat retry.
    ///
    /// Precedence: a server-supplied `Retry-After` always wins (clamped to
    /// [`Self::retry_after_ceiling`]); otherwise jittered exponential backoff
    /// `min(base * multiplier^attempt, max_backoff)`.
    pub fn backoff(
        &self,
        attempt: u32,
        retry_after: Option<Duration>,
        jitter_unit: f64,
    ) -> Duration {
        if let Some(server_hint) = retry_after {
            return server_hint.min(self.retry_after_ceiling);
        }

        let factor = self
            .backoff_multiplier
            .checked_pow(attempt)
            .unwrap_or(u32::MAX);
        let capped = self.base_backoff.saturating_mul(factor).min(self.max_backoff);

        let unit = jitter_unit.clamp(0.0, 1.0);
        match self.jitter {
            JitterKind::None => capped,
            JitterKind::Full => capped.mul_f64(unit),
            JitterKind::Equal => {
                let half = capped / 2;
                half + half.mul_f64(unit)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// FallbackChain — per-model failover with context-window awareness
// ---------------------------------------------------------------------------

/// One model target in a [`FallbackChain`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelTarget {
    pub provider: Provider,
    /// Upstream model id (post-routing), e.g. `claude-sonnet-4-5`.
    pub model: String,
    /// The model's usable context window in tokens.
    pub context_window_tokens: u64,
}

impl ModelTarget {
    pub fn new(
        provider: Provider,
        model: impl Into<String>,
        context_window_tokens: u64,
    ) -> Self {
        Self {
            provider,
            model: model.into(),
            context_window_tokens,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResilienceError {
    EmptyFallbackChain,
}

/// An ordered per-request failover chain: the primary model first, then the
/// fallbacks to try as the ladder escalates across providers. Pure data + pure
/// selection predicates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FallbackChain {
    targets: Vec<ModelTarget>,
}

impl FallbackChain {
    /// Build a chain from an ordered list of targets (primary first). Rejects
    /// an empty chain — there is always at least the primary model.
    pub fn new(targets: Vec<ModelTarget>) -> Result<Self, ResilienceError> {
        if targets.is_empty() {
            return Err(ResilienceError::EmptyFallbackChain);
        }
        Ok(Self { targets })
    }

    pub fn primary(&self) -> &ModelTarget {
        // Invariant: non-empty by construction.
        &self.targets[0]
    }

    pub fn targets(&self) -> &[ModelTarget] {
        &self.targets
    }

    pub fn len(&self) -> usize {
        self.targets.len()
    }

    pub fn is_empty(&self) -> bool {
        // Always false by construction; provided for API completeness/clippy.
        self.targets.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&ModelTarget> {
        self.targets.get(index)
    }

    /// Per-model failover: the next target strictly after `current_index`.
    /// `None` once the chain is exhausted (caller should give up with 503).
    pub fn failover_after(&self, current_index: usize) -> Option<(usize, &ModelTarget)> {
        let next = current_index.checked_add(1)?;
        self.targets.get(next).map(|t| (next, t))
    }

    /// Context-window fallback: scanning from `from_index` onward (inclusive),
    /// the first target whose context window can hold `required_tokens`. Used
    /// to escalate a too-large request to a larger-context model.
    pub fn fit_context_from(
        &self,
        from_index: usize,
        required_tokens: u64,
    ) -> Option<(usize, &ModelTarget)> {
        self.targets
            .iter()
            .enumerate()
            .skip(from_index)
            .find(|(_, t)| t.context_window_tokens >= required_tokens)
    }

    /// Cost-optimal initial pick: the smallest-context target in the whole
    /// chain that still fits `required_tokens`. Lets the gateway start on the
    /// cheapest sufficient model and only escalate on demand.
    pub fn smallest_fitting(&self, required_tokens: u64) -> Option<(usize, &ModelTarget)> {
        self.targets
            .iter()
            .enumerate()
            .filter(|(_, t)| t.context_window_tokens >= required_tokens)
            .min_by_key(|(_, t)| t.context_window_tokens)
    }
}

// ---------------------------------------------------------------------------
// Retry-After parsing (RFC 7231 §7.1.3)
// ---------------------------------------------------------------------------

/// Parse a `Retry-After` header value into a delay, honoring both forms:
///
/// * **delta-seconds**: a non-negative integer (`Retry-After: 120`).
/// * **HTTP-date**: an absolute time (`Retry-After: Wed, 21 Oct 2015 07:28:00
///   GMT`). Resolved against `now_unix_secs` (caller-supplied so the kernel
///   stays clock-free); a date at or before `now` yields `Duration::ZERO`.
///
/// All three HTTP-date formats RFC 7231 requires recipients to accept are
/// supported: IMF-fixdate, obsolete RFC 850, and asctime. Returns `None` for
/// malformed input.
pub fn parse_retry_after(value: &str, now_unix_secs: u64) -> Option<Duration> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(secs) = trimmed.parse::<u64>() {
        return Some(Duration::from_secs(secs));
    }
    let target = parse_http_date(trimmed)?;
    let now = i64::try_from(now_unix_secs).unwrap_or(i64::MAX);
    let delta = target.saturating_sub(now).max(0);
    Some(Duration::from_secs(delta as u64))
}

/// Parse an HTTP-date (RFC 7231 §7.1.1.1) to unix seconds. Accepts IMF-fixdate,
/// RFC 850, and asctime forms. `None` on any malformed input.
fn parse_http_date(value: &str) -> Option<i64> {
    parse_imf_fixdate(value)
        .or_else(|| parse_rfc850(value))
        .or_else(|| parse_asctime(value))
}

/// `Sun, 06 Nov 1994 08:49:37 GMT`
fn parse_imf_fixdate(value: &str) -> Option<i64> {
    let body = value.strip_suffix(" GMT")?;
    let (_weekday, rest) = body.split_once(", ")?;
    let mut it = rest.split(' ');
    let day = it.next()?.parse::<u32>().ok()?;
    let month = month_num(it.next()?)?;
    let year = it.next()?.parse::<i64>().ok()?;
    let (h, m, s) = parse_hms(it.next()?)?;
    if it.next().is_some() {
        return None;
    }
    unix_from(year, month, day, h, m, s)
}

/// `Sunday, 06-Nov-94 08:49:37 GMT`
fn parse_rfc850(value: &str) -> Option<i64> {
    let body = value.strip_suffix(" GMT")?;
    let (_weekday, rest) = body.split_once(", ")?;
    let (date, time) = rest.split_once(' ')?;
    let mut d = date.split('-');
    let day = d.next()?.parse::<u32>().ok()?;
    let month = month_num(d.next()?)?;
    let yy = d.next()?.parse::<i64>().ok()?;
    if d.next().is_some() {
        return None;
    }
    // RFC 7231: a 2-digit year is windowed; >= 70 => 19xx, else 20xx.
    let year = if yy < 70 { 2000 + yy } else { 1900 + yy };
    let (h, m, s) = parse_hms(time)?;
    unix_from(year, month, day, h, m, s)
}

/// `Sun Nov  6 08:49:37 1994`
fn parse_asctime(value: &str) -> Option<i64> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if parts.len() != 5 {
        return None;
    }
    let month = month_num(parts[1])?;
    let day = parts[2].parse::<u32>().ok()?;
    let (h, m, s) = parse_hms(parts[3])?;
    let year = parts[4].parse::<i64>().ok()?;
    unix_from(year, month, day, h, m, s)
}

fn parse_hms(value: &str) -> Option<(u32, u32, u32)> {
    let mut it = value.split(':');
    let h = it.next()?.parse::<u32>().ok()?;
    let m = it.next()?.parse::<u32>().ok()?;
    let s = it.next()?.parse::<u32>().ok()?;
    if it.next().is_some() {
        return None;
    }
    Some((h, m, s))
}

fn month_num(month: &str) -> Option<u32> {
    Some(match month {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    })
}

/// Civil date + time -> unix seconds. Validates field ranges. Uses Howard
/// Hinnant's `days_from_civil` algorithm (proleptic Gregorian, no leap-second
/// modeling — HTTP-dates are UTC wall-clock).
fn unix_from(year: i64, month: u32, day: u32, h: u32, m: u32, s: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || h >= 24 || m >= 60 || s > 60 {
        return None;
    }
    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + i64::from(h) * 3_600 + i64::from(m) * 60 + i64::from(s))
}

/// Days since 1970-01-01 for a proleptic-Gregorian civil date.
/// Reference: Howard Hinnant, "chrono-Compatible Low-Level Date Algorithms".
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> RetryPolicy {
        RetryPolicy {
            jitter: JitterKind::None,
            ..RetryPolicy::default()
        }
    }

    // --- ErrorClass classification ------------------------------------------

    #[test]
    fn http_status_classification() {
        assert_eq!(ErrorClass::from_http_status(200), None);
        assert_eq!(ErrorClass::from_http_status(302), None);
        assert_eq!(
            ErrorClass::from_http_status(429),
            Some(ErrorClass::RateLimited { retry_after: None })
        );
        assert_eq!(ErrorClass::from_http_status(403), Some(ErrorClass::Forbidden));
        assert_eq!(ErrorClass::from_http_status(408), Some(ErrorClass::Timeout));
        assert_eq!(
            ErrorClass::from_http_status(400),
            Some(ErrorClass::ClientError { status: 400 })
        );
        assert_eq!(
            ErrorClass::from_http_status(503),
            Some(ErrorClass::ServerError {
                status: 503,
                retry_after: None
            })
        );
    }

    #[test]
    fn seat_outcome_bridge() {
        assert_eq!(
            ErrorClass::RateLimited { retry_after: None }.seat_outcome(),
            SeatOutcome::RateLimited429
        );
        assert_eq!(ErrorClass::Timeout.seat_outcome(), SeatOutcome::ServerError5xx);
        assert_eq!(
            ErrorClass::RefreshRevoked.seat_outcome(),
            SeatOutcome::RefreshTokenRevoked
        );
        // Seat-blameless errors must not penalize the seat.
        assert_eq!(
            ErrorClass::ContextWindowExceeded.seat_outcome(),
            SeatOutcome::Released
        );
        assert_eq!(ErrorClass::Forbidden.seat_outcome(), SeatOutcome::Released);
    }

    // --- The bounded ladder -------------------------------------------------

    #[test]
    fn server_error_retries_same_seat_then_rotates() {
        let p = policy();
        let mut st = AttemptState::new();
        let err = ErrorClass::ServerError {
            status: 500,
            retry_after: None,
        };
        // First two attempts retry the same seat.
        for _ in 0..p.max_in_seat_retries {
            match p.decide(&err, &st, 0.0) {
                RetryAction::RetrySameSeat { .. } => st.record_same_seat_retry(),
                other => panic!("expected RetrySameSeat, got {other:?}"),
            }
        }
        // In-seat budget exhausted -> rotate.
        assert_eq!(p.decide(&err, &st, 0.0), RetryAction::RotateSeat);
    }

    #[test]
    fn rate_limit_rotates_immediately_no_in_seat_retry() {
        let p = policy();
        let st = AttemptState::new();
        let err = ErrorClass::RateLimited { retry_after: None };
        assert_eq!(p.decide(&err, &st, 0.0), RetryAction::RotateSeat);
    }

    #[test]
    fn ladder_exhaustion_yields_503() {
        let p = policy();
        let err = ErrorClass::RateLimited { retry_after: None };
        // Exhaust seat rotations.
        let mut st = AttemptState::new();
        st.seats_tried = p.max_seat_rotations;
        assert_eq!(p.decide(&err, &st, 0.0), RetryAction::FallbackProvider);
        // Exhaust provider fallbacks too -> graceful 503.
        st.providers_tried = p.max_provider_fallbacks;
        assert_eq!(
            p.decide(&err, &st, 0.0),
            RetryAction::Fail { http_status: 503 }
        );
    }

    #[test]
    fn full_ladder_walk_terminates_in_503() {
        let p = policy();
        let err = ErrorClass::Timeout;
        let mut st = AttemptState::new();
        let mut steps = 0;
        loop {
            steps += 1;
            assert!(steps < 1000, "ladder must terminate");
            match p.decide(&err, &st, 0.0) {
                RetryAction::RetrySameSeat { .. } => st.record_same_seat_retry(),
                RetryAction::RotateSeat => st.record_seat_rotation(),
                RetryAction::FallbackProvider => st.record_provider_fallback(),
                RetryAction::EscalateContextWindow => panic!("timeout is not context window"),
                RetryAction::Fail { http_status } => {
                    assert_eq!(http_status, 503);
                    break;
                }
            }
        }
    }

    #[test]
    fn forbidden_and_client_errors_are_terminal() {
        let p = policy();
        let st = AttemptState::new();
        assert_eq!(
            p.decide(&ErrorClass::Forbidden, &st, 0.0),
            RetryAction::Fail { http_status: 403 }
        );
        assert_eq!(
            p.decide(&ErrorClass::ClientError { status: 422 }, &st, 0.0),
            RetryAction::Fail { http_status: 422 }
        );
    }

    #[test]
    fn context_window_escalates() {
        let p = policy();
        let st = AttemptState::new();
        assert_eq!(
            p.decide(&ErrorClass::ContextWindowExceeded, &st, 0.0),
            RetryAction::EscalateContextWindow
        );
    }

    // --- Retry-After precedence + backoff -----------------------------------

    #[test]
    fn retry_after_takes_precedence_over_backoff() {
        let p = policy();
        let st = AttemptState::new();
        let err = ErrorClass::ServerError {
            status: 503,
            retry_after: Some(Duration::from_secs(7)),
        };
        // Server hint wins over the computed exponential backoff.
        assert_eq!(
            p.decide(&err, &st, 0.0),
            RetryAction::RetrySameSeat {
                delay: Duration::from_secs(7)
            }
        );
    }

    #[test]
    fn retry_after_is_clamped_to_ceiling() {
        let p = policy(); // ceiling = 60s
        assert_eq!(
            p.backoff(0, Some(Duration::from_secs(9999)), 0.0),
            Duration::from_secs(60)
        );
    }

    #[test]
    fn exponential_backoff_grows_and_caps() {
        let p = policy(); // base 200ms, mult 2, max 20s, no jitter
        assert_eq!(p.backoff(0, None, 0.0), Duration::from_millis(200));
        assert_eq!(p.backoff(1, None, 0.0), Duration::from_millis(400));
        assert_eq!(p.backoff(2, None, 0.0), Duration::from_millis(800));
        // Huge attempt saturates to max_backoff, never overflows/panics.
        assert_eq!(p.backoff(1_000, None, 0.0), Duration::from_secs(20));
    }

    #[test]
    fn full_jitter_bounds() {
        let p = RetryPolicy {
            jitter: JitterKind::Full,
            ..RetryPolicy::default()
        };
        let lo = p.backoff(1, None, 0.0);
        let hi = p.backoff(1, None, 1.0);
        assert_eq!(lo, Duration::ZERO);
        assert_eq!(hi, Duration::from_millis(400));
    }

    // --- FallbackChain ------------------------------------------------------

    fn chain() -> FallbackChain {
        FallbackChain::new(vec![
            ModelTarget::new(Provider::Anthropic, "claude-sonnet-4-5", 200_000),
            ModelTarget::new(Provider::Codex, "gpt-5", 400_000),
            ModelTarget::new(Provider::Anthropic, "claude-opus-4-5", 1_000_000),
        ])
        .unwrap()
    }

    #[test]
    fn empty_chain_rejected() {
        assert_eq!(
            FallbackChain::new(vec![]),
            Err(ResilienceError::EmptyFallbackChain)
        );
    }

    #[test]
    fn failover_walks_then_exhausts() {
        let c = chain();
        assert_eq!(c.primary().model, "claude-sonnet-4-5");
        assert_eq!(c.failover_after(0).unwrap().1.model, "gpt-5");
        assert_eq!(c.failover_after(1).unwrap().1.model, "claude-opus-4-5");
        assert_eq!(c.failover_after(2), None);
    }

    #[test]
    fn context_window_fallback_picks_larger_model() {
        let c = chain();
        // 300k tokens won't fit sonnet (200k); escalate to gpt-5 (400k).
        let (idx, t) = c.fit_context_from(0, 300_000).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(t.model, "gpt-5");
        // 500k only fits opus (1M).
        assert_eq!(c.fit_context_from(0, 500_000).unwrap().1.model, "claude-opus-4-5");
        // Nothing fits beyond the largest window.
        assert_eq!(c.fit_context_from(0, 2_000_000), None);
    }

    #[test]
    fn smallest_fitting_is_cost_optimal() {
        let c = chain();
        assert_eq!(c.smallest_fitting(100_000).unwrap().1.model, "claude-sonnet-4-5");
        assert_eq!(c.smallest_fitting(250_000).unwrap().1.model, "gpt-5");
        assert_eq!(c.smallest_fitting(450_000).unwrap().1.model, "claude-opus-4-5");
    }

    // --- Retry-After parsing ------------------------------------------------

    #[test]
    fn parse_retry_after_delta_seconds() {
        assert_eq!(parse_retry_after("120", 0), Some(Duration::from_secs(120)));
        assert_eq!(parse_retry_after("  0 ", 0), Some(Duration::ZERO));
        assert_eq!(parse_retry_after("", 0), None);
        assert_eq!(parse_retry_after("-5", 0), None);
        assert_eq!(parse_retry_after("abc", 0), None);
    }

    #[test]
    fn parse_retry_after_imf_fixdate() {
        // 2015-10-21 07:28:00 UTC = 1445412480.
        let target = 1_445_412_480;
        assert_eq!(
            parse_retry_after("Wed, 21 Oct 2015 07:28:00 GMT", target as u64 - 100),
            Some(Duration::from_secs(100))
        );
        // Date in the past -> zero, never negative.
        assert_eq!(
            parse_retry_after("Wed, 21 Oct 2015 07:28:00 GMT", target as u64 + 50),
            Some(Duration::ZERO)
        );
    }

    #[test]
    fn parse_retry_after_all_three_date_forms_agree() {
        let imf = parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT");
        let rfc850 = parse_http_date("Sunday, 06-Nov-94 08:49:37 GMT");
        let asctime = parse_http_date("Sun Nov  6 08:49:37 1994");
        assert_eq!(imf, Some(784_111_777));
        assert_eq!(imf, rfc850);
        assert_eq!(imf, asctime);
    }

    #[test]
    fn parse_http_date_epoch_is_zero() {
        assert_eq!(parse_http_date("Thu, 01 Jan 1970 00:00:00 GMT"), Some(0));
    }

    #[test]
    fn parse_http_date_rejects_garbage() {
        // The redundant weekday name is intentionally not validated against the
        // date (lenient, RFC-recipient-friendly); the date FIELDS must be valid.
        assert_eq!(parse_http_date("not a date"), None);
        assert_eq!(parse_http_date("Sun, 32 Nov 1994 08:49:37 GMT"), None);
        assert_eq!(parse_http_date("Sun, 06 Foo 1994 08:49:37 GMT"), None);
        assert_eq!(parse_http_date("Sun, 06 Nov 1994 25:49:37 GMT"), None);
        assert_eq!(parse_http_date("Sun, 06 Nov 1994 08:60:37 GMT"), None);
        assert_eq!(parse_http_date("Sun, 06 Nov 1994 08:49:37"), None); // no GMT
    }
}
