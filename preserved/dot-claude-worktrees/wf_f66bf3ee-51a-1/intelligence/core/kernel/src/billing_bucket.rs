//! G0 billing-bucket drift-canary — the program's hard gate (GO-FORWARD-PLAN §3).
//!
//! A subscription-OAuth-pooling gateway is "done" only when a real pooled call
//! bills **on-plan** (subscription), not metered. Token counting alone is blind
//! to that (premortem P4): the one number that matters is *which billing bucket
//! each response landed in*. This module is the pure brain that reads a
//! provider response and answers that question, fail-closed.
//!
//! ## Two functions, one gate
//! 1. [`classify_billing`] — read the per-provider billing signals off a
//!    response (headers + body) and map to a single [`BillingBucket`].
//! 2. [`assert_on_plan`] — turn a bucket into a [`PlanAssertion`]:
//!    - `Subscription | SubscriptionFallback` ⇒ **Pass** (on-plan).
//!    - `ExtraUsage | Api` ⇒ **Fail** — this is the **P9 detector**: the token
//!      authenticated but the call billed metered (device-identity missing, or
//!      Anthropic-subscription silently downgraded). A green HTTP 200 with a
//!      `Fail` bucket is exactly the silent surprise-invoice failure we gate on.
//!    - `Unknown` ⇒ **Inconclusive** — never a false Pass (fail-closed).
//!
//! ## Purity (non-negotiable)
//! Zero I/O, no clock, no network. The adapters hand in the already-collected
//! response `headers`/`body`; emitting the resulting bucket as telemetry is the
//! adapter's job ([`BillingBucketObservation`] is the EventSink-shaped payload).
//! This is the per-request analog of "a pool can be 100% authenticated and 0%
//! on-plan" — and the permanent drift gate (P8) once wired to the live canary.
//!
//! ## Relationship to [`crate::overage_guard`]
//! The overage-guard (SC8, a *separate* lane) reads the SAME Anthropic
//! `anthropic-ratelimit-unified-representative-claim` header, but as a
//! circuit-breaker (`Continue`/`Warn`/`Halt`) collapsing everything non-overage
//! to one "allowed" class. This module is the finer drift taxonomy the gate
//! asserts on (it must distinguish `subscription` from `*_fallback` from
//! `extra_usage` from `api`). They share the header vocabulary by design; when
//! both land on `dev`, [`classify_representative_claim`] here is the canonical
//! 5-bucket parse and the guard's 3-way classifier should delegate to it.
//!
//! ## LIVE-RECONFIRM
//! Every reverse-engineered header name / body field below is marked
//! `LIVE-RECONFIRM`: it came from a dissection of third-party proxies
//! (`reference-dissection-findings.md` §3–4, `subscription-proxy-dissection-findings.md`
//! §4/§9), NOT from live provider traffic this session. The gated live canary
//! (the K8s Job) is what turns these from "reverse-engineered" into "confirmed".

use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Header / body signal names (LIVE-RECONFIRM — see module docs)
// ---------------------------------------------------------------------------

/// Anthropic response header naming the binding rate-limit window for a request.
/// Subscription buckets are `five_hour | seven_day | five_hour_fallback |
/// seven_day_fallback`; `overage` / `api` mean metered.
/// LIVE-RECONFIRM (reference-dissection §4; dario `src/overage-guard.ts`).
pub const ANTHROPIC_REPRESENTATIVE_CLAIM_HEADER: &str =
    "anthropic-ratelimit-unified-representative-claim";

/// Speculative Codex/OpenAI equivalent of the representative-claim header. The
/// OpenAI `chatgpt.com/backend-api/codex` responses were NOT observed to carry
/// a representative-claim header in any dissected proxy — Codex billing is
/// inferred from the device-identity + account headers on the *request* and the
/// absence of a `usage_limit_reached` error, not a response bucket header.
/// We still scan for a header here so that if OpenAI ships one we classify it
/// rather than silently fall through to body/Unknown. LIVE-RECONFIRM: confirm
/// the real name (if any) against live Codex traffic before depending on it.
pub const CODEX_REPRESENTATIVE_CLAIM_HEADER: &str = "x-openai-ratelimit-representative-claim";

/// Body rollup field (`dario`'s `/analytics`): the percentage of a request that
/// billed against the subscription allocation. 100 ⇒ fully on-plan; 0 ⇒ fully
/// metered. Used as a corroborating / fallback signal when no claim header is
/// present. LIVE-RECONFIRM (subscription-proxy-dissection §4/§9).
pub const SUBSCRIPTION_PERCENT_FIELD: &str = "subscriptionPercent";

// ---------------------------------------------------------------------------
// BillingBucket
// ---------------------------------------------------------------------------

/// Which billing bucket a single provider response landed in.
///
/// Ordering of the enum is by "on-plan-ness" so a future need to compare
/// buckets (e.g. "at least as good as fallback") has a natural total order, but
/// no policy depends on the numeric discriminant — [`assert_on_plan`] is the
/// only sanctioned interpreter.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BillingBucket {
    /// Billed fully against the paid subscription plan allocation (`five_hour`,
    /// `seven_day`, or `subscriptionPercent == 100`). On-plan.
    Subscription,
    /// Billed against a subscription *fallback* window (`*_fallback`). Still
    /// on-plan — the provider drew from a secondary subscription allocation, not
    /// metered usage.
    SubscriptionFallback,
    /// Billed as metered "extra usage" / overage beyond the plan allocation
    /// (`overage`, or `subscriptionPercent == 0` with a present signal). This is
    /// the P9 silent-metered-billing outcome: authenticated, but off-plan.
    ExtraUsage,
    /// Billed as direct metered API usage (`api`) — a console-API-key-class
    /// bill, not subscription at all.
    Api,
    /// No trustworthy billing signal present (header absent/empty/`unknown`, no
    /// rollup). Fail-closed: never assert on-plan from this.
    Unknown,
}

impl BillingBucket {
    /// True for the two on-plan buckets. Single source of truth shared by
    /// [`assert_on_plan`] and any caller that wants the boolean directly.
    pub const fn is_on_plan(self) -> bool {
        matches!(self, BillingBucket::Subscription | BillingBucket::SubscriptionFallback)
    }

    /// Stable lowercase token for telemetry / logs.
    pub const fn as_str(self) -> &'static str {
        match self {
            BillingBucket::Subscription => "subscription",
            BillingBucket::SubscriptionFallback => "subscription_fallback",
            BillingBucket::ExtraUsage => "extra_usage",
            BillingBucket::Api => "api",
            BillingBucket::Unknown => "unknown",
        }
    }
}

// ---------------------------------------------------------------------------
// Representative-claim parsing (shared Anthropic/Codex header vocabulary)
// ---------------------------------------------------------------------------

/// Classify a representative-claim header *value* into a [`BillingBucket`].
///
/// Whitespace-trimmed, ASCII-case-insensitive. `None` / empty / `unknown` map to
/// [`BillingBucket::Unknown`] (fail-closed — a signal we cannot trust must never
/// produce a Pass). Recognised subscription windows map to `Subscription` /
/// `SubscriptionFallback`; `overage` → `ExtraUsage`; `api` → `Api`. Any other
/// (future, not-yet-allow-listed) token is treated as [`BillingBucket::Unknown`]
/// so an unseen bucket can never masquerade as on-plan.
pub fn classify_representative_claim(value: Option<&str>) -> BillingBucket {
    let Some(raw) = value else {
        return BillingBucket::Unknown;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return BillingBucket::Unknown;
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "five_hour" | "seven_day" => BillingBucket::Subscription,
        "five_hour_fallback" | "seven_day_fallback" => BillingBucket::SubscriptionFallback,
        "overage" => BillingBucket::ExtraUsage,
        "api" => BillingBucket::Api,
        // `unknown` and any future/unseen bucket: fail-closed.
        _ => BillingBucket::Unknown,
    }
}

/// Case-insensitive header lookup over a `BTreeMap` keyed by the provider's
/// original casing (response headers are not normalized upstream).
fn header_ci<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

/// Extract a `subscriptionPercent`-style rollup from a JSON response body, if
/// present and numeric. Returns the clamped \[0,100] integer percent.
///
/// Pure string/JSON scan — no allocation of the whole body beyond serde's parse.
/// A malformed or non-JSON body yields `None` (then handled fail-closed by the
/// caller). The field is matched case-sensitively on the exact dissected name
/// (`subscriptionPercent`); providers that nest it elsewhere are a LIVE-RECONFIRM
/// follow-up, not a guess to bake in now.
fn subscription_percent_from_body(body: &[u8]) -> Option<u8> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let pct = find_subscription_percent(&value)?;
    Some(pct.clamp(0.0, 100.0).round() as u8)
}

/// Recursively search a JSON value for the first `subscriptionPercent` number.
/// dario emits it at the analytics top level, but adapters may wrap the upstream
/// body, so we tolerate one level of nesting rather than pin an exact path.
fn find_subscription_percent(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(v) = map.get(SUBSCRIPTION_PERCENT_FIELD)
                && let Some(n) = v.as_f64()
            {
                return Some(n);
            }
            map.values().find_map(find_subscription_percent)
        }
        serde_json::Value::Array(items) => items.iter().find_map(find_subscription_percent),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// classify_billing — the provider-agnostic entry point
// ---------------------------------------------------------------------------

/// Classify the billing bucket of one provider response from its headers + body.
///
/// Signal precedence (most-authoritative first):
/// 1. **Anthropic representative-claim header** — the load-bearing on/off-plan
///    signal for Anthropic-subscription.
/// 2. **Codex/OpenAI representative-claim header** (if the provider ships one —
///    LIVE-RECONFIRM; none observed yet).
/// 3. **`subscriptionPercent` body rollup** — corroborating / cross-provider
///    fallback: `100` ⇒ `Subscription`, `0` ⇒ `ExtraUsage`, anything strictly
///    between ⇒ `SubscriptionFallback` (partial on-plan draw).
/// 4. Nothing trustworthy ⇒ [`BillingBucket::Unknown`] (fail-closed).
///
/// A present-but-`unknown` claim header does NOT stop the scan — we fall through
/// to the body rollup, because `unknown` is explicitly the "untrusted/transient"
/// value (reference-dissection §4), not a verdict.
pub fn classify_billing(headers: &BTreeMap<String, String>, body: &[u8]) -> BillingBucket {
    // 1 + 2: representative-claim headers (Anthropic, then Codex/OpenAI).
    for header_name in [
        ANTHROPIC_REPRESENTATIVE_CLAIM_HEADER,
        CODEX_REPRESENTATIVE_CLAIM_HEADER,
    ] {
        match classify_representative_claim(header_ci(headers, header_name)) {
            BillingBucket::Unknown => {} // untrusted/absent — keep scanning.
            decided => return decided,
        }
    }

    // 3: subscriptionPercent rollup (cross-provider, e.g. Codex/Gemini that
    // carry no claim header).
    if let Some(pct) = subscription_percent_from_body(body) {
        return match pct {
            100 => BillingBucket::Subscription,
            0 => BillingBucket::ExtraUsage,
            _ => BillingBucket::SubscriptionFallback,
        };
    }

    // 4: fail-closed.
    BillingBucket::Unknown
}

// ---------------------------------------------------------------------------
// assert_on_plan — the gate verdict
// ---------------------------------------------------------------------------

/// Verdict of the drift gate for one response. Deliberately three-state: a
/// binary pass/fail cannot express "no trustworthy signal" without risking a
/// false Pass, which would defeat the entire fail-closed premise.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanAssertion {
    /// On-plan: `Subscription` or `SubscriptionFallback`. The gate is green.
    Pass,
    /// Off-plan metered billing (`ExtraUsage` or `Api`). The P9 detector tripped:
    /// authenticated but billed metered. The gate is RED.
    Fail { bucket: BillingBucket },
    /// No trustworthy billing signal (`Unknown`). Never a Pass — surfaced as a
    /// distinct state so the canary can demand a real bucket, not silently pass.
    Inconclusive,
}

impl PlanAssertion {
    pub const fn is_pass(self) -> bool {
        matches!(self, PlanAssertion::Pass)
    }
}

/// Assert that a classified [`BillingBucket`] represents on-plan subscription
/// billing.
///
/// - `Subscription | SubscriptionFallback` ⇒ [`PlanAssertion::Pass`].
/// - `ExtraUsage | Api` ⇒ [`PlanAssertion::Fail`] (the P9 metered-billing detector).
/// - `Unknown` ⇒ [`PlanAssertion::Inconclusive`] (fail-closed; never Pass).
pub const fn assert_on_plan(bucket: BillingBucket) -> PlanAssertion {
    match bucket {
        BillingBucket::Subscription | BillingBucket::SubscriptionFallback => PlanAssertion::Pass,
        BillingBucket::ExtraUsage | BillingBucket::Api => PlanAssertion::Fail { bucket },
        BillingBucket::Unknown => PlanAssertion::Inconclusive,
    }
}

/// Fail-closed `Result` form for call sites that want `?`-propagation: only
/// [`PlanAssertion::Pass`] is `Ok`; both `Fail` and `Inconclusive` are `Err`,
/// because for a hard gate "we could not prove on-plan" must block exactly like
/// "we proved off-plan". The live canary uses this so an absent/unknown signal
/// can never be mistaken for success.
pub fn require_on_plan(bucket: BillingBucket) -> Result<(), PlanAssertion> {
    match assert_on_plan(bucket) {
        PlanAssertion::Pass => Ok(()),
        verdict => Err(verdict),
    }
}

// ---------------------------------------------------------------------------
// EventSink-shaped telemetry (P4 proof + permanent drift gate)
// ---------------------------------------------------------------------------

/// Structured, secret-free billing-bucket observation for one request — the P4
/// "billing-bucket is a per-request observable" payload. EventSink-shaped:
/// adapters attach request/seat identity and emit it on the same spine as
/// [`crate::LlmGatewayEvent`]. Pure data; the kernel only *produces* it.
///
/// Derives `Serialize` so the ClickHouse/Valkey sinks can persist it without a
/// second mapping layer. Carries no credential material.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct BillingBucketObservation {
    /// The classified bucket token (`as_str`).
    pub bucket: &'static str, // data_class: INTERNAL_ONLY
    /// The gate verdict: `pass` | `fail` | `inconclusive`.
    pub verdict: &'static str, // data_class: INTERNAL_ONLY
    /// True iff on-plan — denormalized for cheap dashboard filtering.
    pub on_plan: bool, // data_class: INTERNAL_ONLY
    /// `subscriptionPercent` rollup when the body carried one, else `None`.
    pub subscription_percent: Option<u8>, // data_class: INTERNAL_ONLY
}

impl BillingBucketObservation {
    /// Build an observation from a response's headers + body in one shot:
    /// classify, assert, and capture the rollup for telemetry.
    pub fn from_response(headers: &BTreeMap<String, String>, body: &[u8]) -> Self {
        let bucket = classify_billing(headers, body);
        let verdict = match assert_on_plan(bucket) {
            PlanAssertion::Pass => "pass",
            PlanAssertion::Fail { .. } => "fail",
            PlanAssertion::Inconclusive => "inconclusive",
        };
        Self {
            bucket: bucket.as_str(),
            verdict,
            on_plan: bucket.is_on_plan(),
            subscription_percent: subscription_percent_from_body(body),
        }
    }
}

// ---------------------------------------------------------------------------
// Runnable check — hermetic unit tests (always run, zero I/O).
// These ARE the smallest thing that fails if the money-path logic breaks.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn claim(value: &str) -> BTreeMap<String, String> {
        headers(&[(ANTHROPIC_REPRESENTATIVE_CLAIM_HEADER, value)])
    }

    #[test]
    fn anthropic_subscription_windows_are_on_plan() {
        for v in ["five_hour", "seven_day", "FIVE_HOUR", "  seven_day  "] {
            assert_eq!(classify_billing(&claim(v), b""), BillingBucket::Subscription, "{v}");
            assert!(assert_on_plan(classify_billing(&claim(v), b"")).is_pass());
        }
    }

    #[test]
    fn anthropic_fallback_windows_are_on_plan_fallback() {
        for v in ["five_hour_fallback", "seven_day_fallback"] {
            assert_eq!(
                classify_billing(&claim(v), b""),
                BillingBucket::SubscriptionFallback,
                "{v}"
            );
            // Fallback still PASSES the gate (on-plan).
            assert_eq!(
                assert_on_plan(classify_billing(&claim(v), b"")),
                PlanAssertion::Pass
            );
        }
    }

    #[test]
    fn overage_is_extra_usage_and_fails_gate() {
        let b = classify_billing(&claim("overage"), b"");
        assert_eq!(b, BillingBucket::ExtraUsage);
        assert_eq!(assert_on_plan(b), PlanAssertion::Fail { bucket: BillingBucket::ExtraUsage });
        // P9: a metered bill must be an Err in the fail-closed form.
        assert!(require_on_plan(b).is_err());
    }

    #[test]
    fn api_bucket_is_metered_and_fails_gate() {
        let b = classify_billing(&claim("api"), b"");
        assert_eq!(b, BillingBucket::Api);
        assert_eq!(assert_on_plan(b), PlanAssertion::Fail { bucket: BillingBucket::Api });
    }

    #[test]
    fn absent_empty_unknown_and_future_buckets_are_failclosed_unknown() {
        // No headers at all.
        assert_eq!(classify_billing(&headers(&[]), b""), BillingBucket::Unknown);
        // Present but untrusted/empty/unknown/unseen-future.
        for v in ["", "   ", "unknown", "UNKNOWN", "some_future_bucket_2027"] {
            assert_eq!(classify_billing(&claim(v), b""), BillingBucket::Unknown, "{v}");
            // Inconclusive — NOT a Pass, NOT a Fail.
            assert_eq!(
                assert_on_plan(classify_billing(&claim(v), b"")),
                PlanAssertion::Inconclusive
            );
            assert!(require_on_plan(classify_billing(&claim(v), b"")).is_err());
        }
    }

    #[test]
    fn subscription_percent_rollup_drives_bucket_when_no_claim_header() {
        // 100 => fully on-plan.
        let body100 = br#"{"subscriptionPercent": 100}"#;
        assert_eq!(classify_billing(&headers(&[]), body100), BillingBucket::Subscription);
        // 0 => metered.
        let body0 = br#"{"subscriptionPercent": 0}"#;
        assert_eq!(classify_billing(&headers(&[]), body0), BillingBucket::ExtraUsage);
        // Partial => fallback (partial subscription draw).
        let body50 = br#"{"subscriptionPercent": 50}"#;
        assert_eq!(classify_billing(&headers(&[]), body50), BillingBucket::SubscriptionFallback);
    }

    #[test]
    fn rollup_is_found_one_level_nested() {
        let body = br#"{"analytics": {"subscriptionPercent": 100, "other": 1}}"#;
        assert_eq!(classify_billing(&headers(&[]), body), BillingBucket::Subscription);
    }

    #[test]
    fn claim_header_outranks_body_rollup() {
        // Header says overage; body says 100%. The authoritative header wins.
        let body = br#"{"subscriptionPercent": 100}"#;
        assert_eq!(classify_billing(&claim("overage"), body), BillingBucket::ExtraUsage);
    }

    #[test]
    fn unknown_claim_falls_through_to_body_rollup() {
        // `unknown` is "untrusted", not a verdict — body rollup still consulted.
        let body = br#"{"subscriptionPercent": 100}"#;
        assert_eq!(classify_billing(&claim("unknown"), body), BillingBucket::Subscription);
    }

    #[test]
    fn codex_representative_claim_header_is_classified_if_present() {
        // Speculative Codex header (LIVE-RECONFIRM): if it ever ships, we read it.
        let h = headers(&[(CODEX_REPRESENTATIVE_CLAIM_HEADER, "overage")]);
        assert_eq!(classify_billing(&h, b""), BillingBucket::ExtraUsage);
    }

    #[test]
    fn malformed_body_is_failclosed_not_a_panic() {
        assert_eq!(classify_billing(&headers(&[]), b"not json at all"), BillingBucket::Unknown);
        assert_eq!(classify_billing(&headers(&[]), b""), BillingBucket::Unknown);
        assert_eq!(classify_billing(&headers(&[]), b"{"), BillingBucket::Unknown);
    }

    #[test]
    fn observation_is_secret_free_and_shaped_for_eventsink() {
        let obs = BillingBucketObservation::from_response(&claim("five_hour"), b"");
        assert_eq!(obs.bucket, "subscription");
        assert_eq!(obs.verdict, "pass");
        assert!(obs.on_plan);

        let obs_fail = BillingBucketObservation::from_response(&claim("overage"), b"");
        assert_eq!(obs_fail.verdict, "fail");
        assert!(!obs_fail.on_plan);

        let obs_inconclusive = BillingBucketObservation::from_response(&headers(&[]), b"");
        assert_eq!(obs_inconclusive.verdict, "inconclusive");
        assert!(!obs_inconclusive.on_plan);

        let obs_pct =
            BillingBucketObservation::from_response(&headers(&[]), br#"{"subscriptionPercent": 100}"#);
        assert_eq!(obs_pct.subscription_percent, Some(100));
        // Serializes without leaking anything (smoke).
        let json = serde_json::to_string(&obs_pct).unwrap();
        assert!(json.contains("\"on_plan\":true"));
    }
}
