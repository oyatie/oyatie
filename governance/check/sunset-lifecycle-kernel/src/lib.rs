//! Foundry sunset-lifecycle fitness kernel.
//!
//! # Naming justification
//!
//! - Crate `oya-governance-sunset-lifecycle-kernel` —
//!   v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:sunset-lifecycle>-<layer:kernel>`;
//!   13-layer-enum suffix `kernel` (innermost ring: I/O-free pure check
//!   functions per ADR-0056 §"port-in-kernel" and ADR-0105 §"Amendment 1").
//! - Dev-CLI `oya-governance-sunset-lifecycle-app` —
//!   v4 BNF `oya-<product:foundry>-<facet:fitness>-<topic:sunset-lifecycle>-<layer:app>`;
//!   13-layer-enum suffix `app` (composition-root binary tool surface per
//!   ADR-0107 §"Amendment 2026-05-15 — no-exception canonical naming").
//!
//! # Intent
//!
//! Operationalize the user directive (2026-05-15) — sunset clauses are
//! canonical *because of* their sunset date, not despite it (per
//! `feedback_no_exceptions_canonical.md`). To honor that framing, every
//! sunset clause MUST be enforceable by a fitness lane that:
//!
//! 1. detects when the sunset date is reached and the surface is still live
//!    without a deprecation marker,
//! 2. detects when the removal date is reached and the surface still
//!    exists, and
//! 3. detects when sunset prose lacks the machine-readable schema needed
//!    to participate in (1) and (2).
//!
//! Schema is anchored in ADR-0108. Default deprecation lag is 30 days
//! after sunset; default removal lag is 90 days after deprecation. These
//! defaults are **canonical sub-rules**, not exceptions; per the
//! no-exceptions doctrine they are extensions of the canonical schema.
//!
//! # Algorithm (kernel — I/O-free)
//!
//! Runners discover sunset clauses (in ADR frontmatter, spec JSON
//! `_sunset` objects, `[package.metadata.oya.sunset]` Cargo manifest
//! sections) and pass them as [`SunsetClause`] records into [`evaluate`].
//! The kernel:
//!
//! 1. Resolves the effective `deprecation_at` and `removal_at` for each
//!    clause (applying the 30 / 90 day canonical defaults when absent).
//! 2. Classifies the clause into a [`LifecycleState`] using `now` and
//!    `reached_milestones` (date OR milestone equivalence, never both
//!    silently — milestone takes precedence when both are present
//!    AND the milestone is reached).
//! 3. Emits a [`Violation`] for the three failure states:
//!    [`LifecycleState::SunsetReached`], [`LifecycleState::RemovalReached`],
//!    [`LifecycleState::MissingFields`]. The two healthy states
//!    ([`LifecycleState::PreSunset`], [`LifecycleState::Deprecated`])
//!    are silent.
//!
//! Filesystem walking, frontmatter parsing, and exit-code mapping live in
//! the dev-CLI runner.

#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Proleptic-Gregorian calendar date (year / month / day, 1-based month
/// and day). Kernel-local std-only date type — keeps the kernel
/// dependency-free per ADR-0083 Tier 1 (no chrono in library crates
/// without an upstream-supplied workspace dep).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Date {
    // data_class: INTERNAL_ONLY
    pub year: i32, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub month: u8, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub day: u8, // data_class: INTERNAL_ONLY
}

impl Date {
    /// Construct 1970-01-01 (Unix epoch) as an infallible constant.
    ///
    /// ADR-0083 Tier 1: callers that previously wrote
    /// `Date::new(1970, 1, 1).expect("epoch is valid")` should call this
    /// instead — the validity check is encoded statically (the year/month/day
    /// triplet is a verified privacy-program-free constant of the date
    /// vocabulary). No `Result`, no `.expect()`, no panic path.
    pub const fn epoch() -> Self {
        Self {
            year: 1970,
            month: 1,
            day: 1,
        }
    }

    /// Construct a new date. Returns `None` for out-of-range month/day or
    /// day-greater-than-month-length.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }
        let max_day = days_in_month(year, month);
        if day < 1 || day > max_day {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// Parse an `YYYY-MM-DD` date string. Returns `None` on any format or
    /// range error. Strictly 10 characters; no leniency.
    pub fn parse_iso(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year: i32 = s.get(0..4)?.parse().ok()?;
        let month: u8 = s.get(5..7)?.parse().ok()?;
        let day: u8 = s.get(8..10)?.parse().ok()?;
        Self::new(year, month, day)
    }

    /// Returns the date `days` days after this one. Negative `days` moves
    /// backward. Uses proleptic-Gregorian arithmetic via day-number.
    pub fn add_days(self, days: i64) -> Self {
        let n = self.to_day_number() + days;
        Self::from_day_number(n)
    }

    /// Signed day-difference `self - other`. Positive when `self` is after
    /// `other`.
    pub fn days_since(self, other: Date) -> i64 {
        self.to_day_number() - other.to_day_number()
    }

    /// Convert to a serial day number (Rata Die-ish — proleptic Gregorian,
    /// monotonic). The origin is arbitrary; only differences matter.
    fn to_day_number(self) -> i64 {
        let (y, m) = if (self.month as i32) <= 2 {
            (self.year as i64 - 1, self.month as i64 + 12)
        } else {
            (self.year as i64, self.month as i64)
        };
        let d = self.day as i64;
        // Howard Hinnant's days_from_civil algorithm
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * (m - 3) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    fn from_day_number(z: i64) -> Self {
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = if m <= 2 { y + 1 } else { y } as i32;
        Date {
            year,
            month: m as u8,
            day: d as u8,
        }
    }
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// A sunset clause discovered in the repo (ADR frontmatter, spec JSON
/// `_sunset` object, or `[package.metadata.oya.sunset]` Cargo section).
///
/// Per ADR-0108: a clause MUST carry either `sunset_at` (RFC3339 date) OR
/// `sunset_milestone` (canonical milestone identifier). A clause that
/// carries neither is classified [`LifecycleState::MissingFields`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SunsetClause {
    /// Repo-relative location for diagnostics (e.g.
    /// `docs/adr-archive/ADR-0107-tools-implicit-app-convention.md#sunset` or
    /// `crates/foo/Cargo.toml#package.metadata.oya.sunset`).
    /// data_class: INTERNAL_ONLY
    pub location: String, // data_class: INTERNAL_ONLY
    /// Optional explicit sunset date (`YYYY-MM-DD`).
    // data_class: INTERNAL_ONLY
    pub sunset_at: Option<Date>, // data_class: INTERNAL_ONLY
    /// Optional milestone-anchored sunset (e.g. `M01-P08-merge`).
    /// data_class: INTERNAL_ONLY
    pub sunset_milestone: Option<String>, // data_class: INTERNAL_ONLY
    /// Optional explicit deprecation date. When `None` and `sunset_at` is
    /// present, defaults to `sunset_at + 30 days` per ADR-0108
    /// §"Canonical sub-rule (defaulting)".
    // data_class: INTERNAL_ONLY
    pub deprecation_at: Option<Date>, // data_class: INTERNAL_ONLY
    /// Optional explicit removal date. When `None`, defaults to
    /// `deprecation_at + 90 days` (effective deprecation_at, including
    /// the 30-day default) per ADR-0108.
    // data_class: INTERNAL_ONLY
    pub removal_at: Option<Date>, // data_class: INTERNAL_ONLY
    /// Short slug used for cross-referencing the same sunset in multiple
    /// surfaces. data_class: INTERNAL_ONLY
    pub sunset_topic: String, // data_class: INTERNAL_ONLY
    /// True when the surface carries a deprecation marker
    /// (`#[deprecated]`, `status: Deprecated`, `Deprecated:` doc-comment).
    /// Discovery layer is responsible for populating this honestly.
    // data_class: INTERNAL_ONLY
    pub has_deprecation_marker: bool, // data_class: INTERNAL_ONLY
}

/// Lifecycle states per ADR-0108 §"State machine".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    /// Sunset has NOT yet been reached. Clause is healthy.
    PreSunset,
    /// Sunset has been reached (date or milestone) but no deprecation
    /// marker is present. **Finding: should-be-deprecated.**
    SunsetReached,
    /// Deprecation marker is present and the removal date has not yet
    /// been reached. Clause is healthy (informational).
    Deprecated,
    /// Removal date has been reached but the clause / code is still
    /// present in the repo. **Finding: must-be-removed.**
    RemovalReached,
    /// Clause has sunset prose but lacks the machine-readable schema
    /// (neither `sunset_at` nor `sunset_milestone`). **Finding:
    /// needs-schema-upgrade.**
    MissingFields,
}

/// Single violation for the lane to emit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    /// data_class: INTERNAL_ONLY
    pub clause_location: String, // data_class: INTERNAL_ONLY
    // data_class: INTERNAL_ONLY
    pub state: LifecycleState, // data_class: INTERNAL_ONLY
    /// data_class: INTERNAL_ONLY
    pub expected_action: String, // data_class: INTERNAL_ONLY
    /// Days overdue. `None` for [`LifecycleState::MissingFields`] (no
    /// reference date to measure overdue against).
    // data_class: INTERNAL_ONLY
    pub days_overdue: Option<i64>, // data_class: INTERNAL_ONLY
}

/// Days added between sunset and deprecation when `deprecation_at` is
/// absent. Per ADR-0108 §"Canonical sub-rule (defaulting)" — NOT an
/// exception, an extension of the canonical schema.
pub const DEFAULT_DEPRECATION_LAG_DAYS: i64 = 30;
/// Days added between deprecation and removal when `removal_at` is
/// absent. Per ADR-0108 §"Canonical sub-rule (defaulting)".
pub const DEFAULT_REMOVAL_LAG_DAYS: i64 = 90;

/// Canonical sentinel milestone identifier (ADR-0108 §"Amendment 2026-05-15
/// — doctrine-not-time-bounded canonical sentinel"). When
/// `sunset_milestone == DOCTRINE_NOT_TIME_BOUNDED_SENTINEL`, the clause
/// references the artifact's role as a doctrine schema rather than a
/// time-bounded transition of the artifact itself. Such clauses are
/// **canonically exempt** from SunsetReached / RemovalReached findings:
/// they describe a permanent doctrinal reference, not a sunset commitment.
///
/// Per `feedback_no_exceptions_canonical.md` vocabulary registry, this
/// IS the canonical pattern for non-time-bounded doctrine references —
/// not an exception to canonical, but a canonical sub-rule of the
/// sunset-clause schema (parallel to the 30/90-day defaulting sub-rule).
pub const DOCTRINE_NOT_TIME_BOUNDED_SENTINEL: &str = "doctrine-not-time-bounded";

/// Enumerated canonical sentinel milestone identifiers. Authors MUST
/// use one of these when the sunset clause references a non-time-bounded
/// concept; the kernel recognizes them and exempts matching clauses from
/// time-based lifecycle findings.
///
/// Per ADR-0108 §"Amendment 2026-05-15 — doctrine-not-time-bounded
/// canonical sentinel", new sentinels MUST be added to this list and
/// declared in the ADR amendment to remain machine-readable.
pub const CANONICAL_SENTINEL_MILESTONES: &[&str] = &[DOCTRINE_NOT_TIME_BOUNDED_SENTINEL];

/// Returns `true` when the supplied milestone identifier is a canonical
/// sentinel per [`CANONICAL_SENTINEL_MILESTONES`]. Sentinel-anchored
/// clauses are exempt from SunsetReached / RemovalReached findings and
/// classify as [`LifecycleState::PreSunset`] (silent/healthy).
pub fn is_sentinel_milestone(milestone: &str) -> bool {
    CANONICAL_SENTINEL_MILESTONES.contains(&milestone)
}

/// Effective deprecation date for a clause: explicit if present, else
/// `sunset_at + 30 days`. `None` when sunset has no date anchor.
pub fn effective_deprecation_at(clause: &SunsetClause) -> Option<Date> {
    clause.deprecation_at.or_else(|| {
        clause
            .sunset_at
            .map(|s| s.add_days(DEFAULT_DEPRECATION_LAG_DAYS))
    })
}

/// Effective removal date for a clause: explicit if present, else
/// `effective_deprecation_at + 90 days`. `None` when neither anchor is
/// resolvable.
pub fn effective_removal_at(clause: &SunsetClause) -> Option<Date> {
    if let Some(removal) = clause.removal_at {
        return Some(removal);
    }
    effective_deprecation_at(clause).map(|d| d.add_days(DEFAULT_REMOVAL_LAG_DAYS))
}

fn milestone_reached(clause: &SunsetClause, reached: &[String]) -> bool {
    match &clause.sunset_milestone {
        Some(ms) => reached.iter().any(|m| m == ms),
        None => false,
    }
}

fn classify(
    clause: &SunsetClause,
    now: Date,
    reached_milestones: &[String],
) -> (LifecycleState, Option<i64>) {
    // MissingFields: no date AND no milestone — cannot evaluate.
    if clause.sunset_at.is_none() && clause.sunset_milestone.is_none() {
        return (LifecycleState::MissingFields, None);
    }

    // Canonical sentinel milestone: clause references the artifact's role
    // as a doctrine schema, not a time-bounded transition. Exempt from
    // SunsetReached / RemovalReached findings per ADR-0108 §"Amendment
    // 2026-05-15 — doctrine-not-time-bounded canonical sentinel". The
    // sentinel applies only when `sunset_at` is absent (a clause that
    // pairs a calendar date with a sentinel milestone is a schema error
    // — the date wins and the sentinel is ignored, no silent contradiction).
    if clause.sunset_at.is_none()
        && let Some(ms) = clause.sunset_milestone.as_deref()
        && is_sentinel_milestone(ms)
    {
        return (LifecycleState::PreSunset, None);
    }

    let removal = effective_removal_at(clause);
    let deprecation = effective_deprecation_at(clause);

    // Determine whether sunset is reached.
    let sunset_by_date = clause.sunset_at.map(|s| now >= s).unwrap_or(false);
    let sunset_by_milestone = milestone_reached(clause, reached_milestones);
    let sunset_reached = sunset_by_date || sunset_by_milestone;

    // REMOVAL_REACHED takes precedence over SUNSET_REACHED: if removal
    // date passed, the clause is well past sunset and the surface is
    // overdue for hard removal regardless of deprecation marker.
    if let Some(r) = removal
        && now >= r
    {
        let days = now.days_since(r);
        return (LifecycleState::RemovalReached, Some(days));
    }

    if sunset_reached {
        if clause.has_deprecation_marker {
            return (LifecycleState::Deprecated, None);
        }
        // Days overdue measured against the effective deprecation_at
        // (when available) so the lane reports "should-have-been-deprecated-by".
        let days = deprecation.map(|d| now.days_since(d).max(0));
        return (LifecycleState::SunsetReached, days);
    }

    LifecycleState::PreSunset.into_violation_pair()
}

impl LifecycleState {
    fn into_violation_pair(self) -> (LifecycleState, Option<i64>) {
        (self, None)
    }
}

/// Evaluate every clause and emit findings for the three failure states.
/// Healthy states ([`LifecycleState::PreSunset`], [`LifecycleState::Deprecated`])
/// are silent.
///
/// `now` is passed in (kernel is deterministic — no system clock read);
/// `reached_milestones` lists canonical milestone identifiers whose merge
/// gates have been crossed.
pub fn evaluate(
    clauses: &[SunsetClause],
    now: Date,
    reached_milestones: &[String],
) -> Vec<Violation> {
    let mut out = Vec::new();
    for clause in clauses {
        let (state, days_overdue) = classify(clause, now, reached_milestones);
        match state {
            LifecycleState::PreSunset | LifecycleState::Deprecated => continue,
            LifecycleState::SunsetReached => {
                out.push(Violation {
                    clause_location: clause.location.clone(),
                    state,
                    expected_action: format!(
                        "add deprecation marker to sunset topic `{}` (sunset reached; canonical sub-rule: deprecation_at = sunset_at + {} days)",
                        clause.sunset_topic, DEFAULT_DEPRECATION_LAG_DAYS,
                    ),
                    days_overdue,
                });
            }
            LifecycleState::RemovalReached => {
                out.push(Violation {
                    clause_location: clause.location.clone(),
                    state,
                    expected_action: format!(
                        "remove sunset topic `{}` from repo (removal_at reached; canonical sub-rule: removal_at = deprecation_at + {} days)",
                        clause.sunset_topic, DEFAULT_REMOVAL_LAG_DAYS,
                    ),
                    days_overdue,
                });
            }
            LifecycleState::MissingFields => {
                out.push(Violation {
                    clause_location: clause.location.clone(),
                    state,
                    expected_action: format!(
                        "add machine-readable schema to sunset topic `{}` (sunset_at: YYYY-MM-DD OR sunset_milestone: <canonical-id>) per ADR-0108",
                        clause.sunset_topic,
                    ),
                    days_overdue,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, day: u8) -> Date {
        Date::new(y, m, day).expect("valid test date")
    }

    fn clause_with_sunset(topic: &str, sunset: Date, has_marker: bool) -> SunsetClause {
        SunsetClause {
            location: format!("test://{}", topic),
            sunset_at: Some(sunset),
            sunset_milestone: None,
            deprecation_at: None,
            removal_at: None,
            sunset_topic: topic.to_string(),
            has_deprecation_marker: has_marker,
        }
    }

    #[test]
    fn date_parse_iso_roundtrips() {
        let parsed = Date::parse_iso("2026-05-15").expect("parse");
        assert_eq!(parsed, d(2026, 5, 15));
        assert!(Date::parse_iso("2026/05/15").is_none());
        assert!(Date::parse_iso("2026-13-01").is_none());
        assert!(Date::parse_iso("2026-02-30").is_none());
    }

    #[test]
    fn date_arithmetic_handles_leap_and_month_boundaries() {
        assert_eq!(d(2024, 2, 28).add_days(1), d(2024, 2, 29)); // 2024 leap
        assert_eq!(d(2024, 2, 29).add_days(1), d(2024, 3, 1));
        assert_eq!(d(2025, 2, 28).add_days(1), d(2025, 3, 1)); // 2025 not leap
        assert_eq!(d(2025, 12, 31).add_days(1), d(2026, 1, 1));
        assert_eq!(d(2026, 1, 1).add_days(-1), d(2025, 12, 31));
        assert_eq!(d(2026, 5, 15).add_days(30), d(2026, 6, 14));
        assert_eq!(d(2026, 5, 15).days_since(d(2026, 4, 15)), 30);
    }

    #[test]
    fn pre_sunset_clause_is_silent() {
        let clauses = [clause_with_sunset("future-thing", d(2030, 1, 1), false)];
        let violations = evaluate(&clauses, d(2026, 5, 15), &[]);
        assert!(
            violations.is_empty(),
            "pre-sunset should be silent: {violations:?}"
        );
    }

    #[test]
    fn sunset_reached_without_marker_emits_finding() {
        let clauses = [clause_with_sunset("past-thing", d(2026, 4, 15), false)];
        let violations = evaluate(&clauses, d(2026, 5, 15), &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].state, LifecycleState::SunsetReached);
        // sunset 2026-04-15, default deprecation_at = 2026-05-15, now =
        // 2026-05-15 -> days_overdue = 0 (just reached deprecation due-date).
        assert_eq!(violations[0].days_overdue, Some(0));
        assert!(violations[0].expected_action.contains("deprecation marker"));
    }

    #[test]
    fn sunset_reached_with_marker_is_deprecated_and_silent() {
        let clauses = [clause_with_sunset("past-thing", d(2026, 4, 15), true)];
        let violations = evaluate(&clauses, d(2026, 5, 15), &[]);
        assert!(
            violations.is_empty(),
            "deprecated clause should be silent: {violations:?}"
        );
    }

    #[test]
    fn removal_reached_emits_finding_even_with_marker() {
        let mut clause = clause_with_sunset("ancient-thing", d(2025, 1, 1), true);
        // Default removal_at = 2025-01-01 + 30 + 90 = 2025-05-01.
        clause.removal_at = None;
        let violations = evaluate(&[clause], d(2026, 5, 15), &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].state, LifecycleState::RemovalReached);
        // 2025-05-01 -> 2026-05-15 = 379 days.
        assert_eq!(violations[0].days_overdue, Some(379));
        assert!(violations[0].expected_action.contains("remove"));
    }

    #[test]
    fn explicit_deprecation_and_removal_dates_override_defaults() {
        let clause = SunsetClause {
            location: "test://explicit".into(),
            sunset_at: Some(d(2026, 1, 1)),
            sunset_milestone: None,
            deprecation_at: Some(d(2026, 6, 1)), // overrides +30 = 2026-01-31
            removal_at: Some(d(2027, 1, 1)),     // overrides +90 = 2026-08-30
            sunset_topic: "explicit".into(),
            has_deprecation_marker: false,
        };
        // now = 2026-05-15. Sunset reached (Jan 1). Default deprecation
        // would be Jan 31 (overdue) but explicit deprecation Jun 1 has
        // NOT yet arrived. Lane still emits SunsetReached because no
        // marker present; days_overdue reflects explicit deprecation:
        // 2026-05-15 - 2026-06-01 = -17 days, clamped to 0.
        let violations = evaluate(&[clause], d(2026, 5, 15), &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].state, LifecycleState::SunsetReached);
        assert_eq!(violations[0].days_overdue, Some(0));
    }

    #[test]
    fn milestone_takes_precedence_when_reached() {
        let clause = SunsetClause {
            location: "test://ms".into(),
            sunset_at: None,
            sunset_milestone: Some("M01-P08-merge".into()),
            deprecation_at: None,
            removal_at: None,
            sunset_topic: "milestone-thing".into(),
            has_deprecation_marker: false,
        };
        let now = d(2026, 5, 15);
        // Milestone NOT reached -> healthy.
        let v1 = evaluate(std::slice::from_ref(&clause), now, &[]);
        assert!(v1.is_empty(), "milestone-not-reached: {v1:?}");
        // Milestone reached -> SunsetReached. days_overdue is None
        // because no date anchor exists to measure against (milestone-
        // only clauses have no deprecation due-date).
        let v2 = evaluate(&[clause], now, &["M01-P08-merge".to_string()]);
        assert_eq!(v2.len(), 1);
        assert_eq!(v2[0].state, LifecycleState::SunsetReached);
        assert_eq!(v2[0].days_overdue, None);
    }

    #[test]
    fn missing_fields_emits_schema_upgrade_finding() {
        let clause = SunsetClause {
            location: "test://prose-only".into(),
            sunset_at: None,
            sunset_milestone: None,
            deprecation_at: None,
            removal_at: None,
            sunset_topic: "prose-only-thing".into(),
            has_deprecation_marker: false,
        };
        let violations = evaluate(&[clause], d(2026, 5, 15), &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].state, LifecycleState::MissingFields);
        assert_eq!(violations[0].days_overdue, None);
        assert!(violations[0].expected_action.contains("ADR-0108"));
    }

    #[test]
    fn effective_dates_apply_canonical_30_90_defaults() {
        let clause = clause_with_sunset("default-rule", d(2026, 1, 1), false);
        assert_eq!(effective_deprecation_at(&clause), Some(d(2026, 1, 31)));
        assert_eq!(effective_removal_at(&clause), Some(d(2026, 5, 1)));
    }

    #[test]
    fn empty_input_returns_empty_violations() {
        let violations = evaluate(&[], d(2026, 5, 15), &[]);
        assert!(violations.is_empty());
    }

    #[test]
    fn doctrine_sentinel_is_recognized() {
        assert!(is_sentinel_milestone(DOCTRINE_NOT_TIME_BOUNDED_SENTINEL));
        assert!(is_sentinel_milestone("doctrine-not-time-bounded"));
        assert!(!is_sentinel_milestone("M01-P08-merge"));
        assert!(!is_sentinel_milestone("doctrine-not-time-bounded-typo"));
    }

    #[test]
    fn doctrine_sentinel_clause_is_silent_indefinitely() {
        // Sentinel-anchored clauses describe a doctrinal schema reference,
        // not a time-bounded transition. They must classify as PreSunset
        // (silent/healthy) regardless of `now` — there is no calendar
        // anchor to overshoot. Per ADR-0108 §"Amendment 2026-05-15 —
        // doctrine-not-time-bounded canonical sentinel".
        let clause = SunsetClause {
            location: "test://doctrine".into(),
            sunset_at: None,
            sunset_milestone: Some(DOCTRINE_NOT_TIME_BOUNDED_SENTINEL.to_string()),
            deprecation_at: None,
            removal_at: None,
            sunset_topic: "doctrine-reference".into(),
            has_deprecation_marker: false,
        };

        // Today.
        let v_today = evaluate(std::slice::from_ref(&clause), d(2026, 5, 15), &[]);
        assert!(v_today.is_empty(), "sentinel-today: {v_today:?}");

        // Far future — still silent (no time-bounded transition exists).
        let v_future = evaluate(std::slice::from_ref(&clause), d(2099, 12, 31), &[]);
        assert!(v_future.is_empty(), "sentinel-future: {v_future:?}");

        // Even if the literal sentinel string appears in reached_milestones
        // it has no effect: there is no date anchor and no genuine merge
        // gate the sentinel corresponds to.
        let v_w_milestone = evaluate(
            std::slice::from_ref(&clause),
            d(2099, 12, 31),
            &[DOCTRINE_NOT_TIME_BOUNDED_SENTINEL.to_string()],
        );
        assert!(
            v_w_milestone.is_empty(),
            "sentinel-with-reached-list: {v_w_milestone:?}"
        );
    }

    #[test]
    fn doctrine_sentinel_with_calendar_date_does_not_silently_exempt() {
        // A clause that pairs `sunset_at` (calendar anchor) with the
        // sentinel milestone is a schema error — the kernel honors the
        // calendar date and ignores the sentinel rather than silently
        // exempting. This avoids a backdoor where authors set
        // `sunset_milestone: doctrine-not-time-bounded` to mute a real
        // overdue sunset_at.
        let clause = SunsetClause {
            location: "test://contradiction".into(),
            // Past sunset but pre-removal window: 14 days past sunset_at;
            // effective_removal_at defaults to sunset_at + 30 + 90 = 120 days
            // out, which is still future relative to `now`.
            sunset_at: Some(d(2026, 5, 1)),
            sunset_milestone: Some(DOCTRINE_NOT_TIME_BOUNDED_SENTINEL.to_string()),
            deprecation_at: None,
            removal_at: None,
            sunset_topic: "calendar-anchored".into(),
            has_deprecation_marker: false,
        };
        let v = evaluate(std::slice::from_ref(&clause), d(2026, 5, 15), &[]);
        // sunset_at present and reached -> SunsetReached, NOT silently exempt.
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].state, LifecycleState::SunsetReached);
    }
}
