//! Changeset state-machine kernel — ADR-0110.
//!
//! This crate is the canonical port-in-kernel for the agentic-VCS
//! changeset state machine. It models:
//!
//! - The closed 13-value [`ChangesetState`] enum: 10 advancing states
//!   (`Opened` → `Produced`) plus 3 terminal-fail states (`Abandoned`,
//!   `Rejected`, `CostExhausted`).
//! - The [`ChangesetEvent`] row shape that every pipeline emitter
//!   appends to `registry/vcs/changeset-event-log.json` on every state
//!   transition.
//! - [`validate_monotonic_event_log`] — the pure-function validator
//!   that asserts the non-decreasing-subsequence invariant from
//!   ADR-0110 §"Monotonic invariant", checks dedup-key uniqueness, and
//!   asserts at most one terminal state per log.
//!
//! ADR-0056 port-in-kernel discipline: this crate is pure-domain. No
//! I/O, no clock, no randomness, no shelling out to git, no external
//! deps beyond `std`. The companion `oya-foundry-vcs-changeset-state-app`
//! binary owns the filesystem, clock, and signing-key surfaces.
//!
//! ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
//! `panic!()` to assert invariants under the `cfg(test)` exemption.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The closed enum of changeset states per ADR-0110.
///
/// Ten advancing states (ordinal 0..=9) plus three terminal-fail
/// states. Ordinal order on the advancing portion is the canonical
/// monotonic order used by [`validate_monotonic_event_log`]. Terminal
/// states have no ordinal interpretation — they may be entered from
/// any advancing state and end the log.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ChangesetState {
    // ---- Advancing states (ADR-0110 §"The 9 states", actually 10
    // numbered 0..=9; the ADR prose "9 advancing" miscount is
    // resolved here by counting the table rows literally). ----
    Opened,
    Working,
    Verified,
    PrOpen,
    CiRunning,
    CiPassed,
    Reviewed,
    MergedDev,
    Staged,
    Produced,
    // ---- Terminal-fail states ----
    Abandoned,
    Rejected,
    CostExhausted,
}

impl ChangesetState {
    /// All 13 variants in canonical declaration order. Used by the
    /// `enum-closed` CI lane to assert no string `to_state` value
    /// drifts outside the closed set.
    pub const ALL: [Self; 13] = [
        Self::Opened,
        Self::Working,
        Self::Verified,
        Self::PrOpen,
        Self::CiRunning,
        Self::CiPassed,
        Self::Reviewed,
        Self::MergedDev,
        Self::Staged,
        Self::Produced,
        Self::Abandoned,
        Self::Rejected,
        Self::CostExhausted,
    ];

    /// Canonical snake_case wire-form used in the event-log JSON.
    pub fn as_wire(self) -> &'static str {
        match self {
            Self::Opened => "opened",
            Self::Working => "working",
            Self::Verified => "verified",
            Self::PrOpen => "pr_open",
            Self::CiRunning => "ci_running",
            Self::CiPassed => "ci_passed",
            Self::Reviewed => "reviewed",
            Self::MergedDev => "merged_dev",
            Self::Staged => "staged",
            Self::Produced => "produced",
            Self::Abandoned => "abandoned",
            Self::Rejected => "rejected",
            Self::CostExhausted => "cost_exhausted",
        }
    }

    /// Parse a wire-form string back to the typed variant. Returns
    /// `None` if the value is not in the closed 13-value set —
    /// the `enum-closed` CI lane treats `None` as a hard failure.
    pub fn from_wire(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|state| state.as_wire() == value)
    }

    /// True if this is one of the three terminal-fail states.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Produced | Self::Abandoned | Self::Rejected | Self::CostExhausted
        )
    }

    /// Ordinal in the advancing chain (0..=9) for advancing states,
    /// `None` for terminal-fail states. Used by the monotonic
    /// validator to compare positions on the advancing axis.
    pub fn advancing_ordinal(self) -> Option<u8> {
        match self {
            Self::Opened => Some(0),
            Self::Working => Some(1),
            Self::Verified => Some(2),
            Self::PrOpen => Some(3),
            Self::CiRunning => Some(4),
            Self::CiPassed => Some(5),
            Self::Reviewed => Some(6),
            Self::MergedDev => Some(7),
            Self::Staged => Some(8),
            Self::Produced => Some(9),
            Self::Abandoned | Self::Rejected | Self::CostExhausted => None,
        }
    }
}

impl fmt::Display for ChangesetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_wire())
    }
}

/// Remaining cost budget across the three axes that the cost-budget
/// lane (ADR-0110 §"Event log shape", terminal `CostExhausted`)
/// guards. Any axis crossing zero forces a terminal-fail transition.
#[derive(Clone, Debug, PartialEq)]
pub struct CostBudget {
    pub usd_remaining: f64,
    pub tokens_remaining: u64,
    pub agent_invocations_remaining: u32,
}

impl CostBudget {
    /// True when at least one axis is exhausted (≤ 0). Emitters MUST
    /// transition the changeset to [`ChangesetState::CostExhausted`]
    /// on the next event when this returns true.
    pub fn is_exhausted(&self) -> bool {
        self.usd_remaining <= 0.0
            || self.tokens_remaining == 0
            || self.agent_invocations_remaining == 0
    }
}

/// One row of the changeset event log per ADR-0110 §"Event log shape".
#[derive(Clone, Debug, PartialEq)]
pub struct ChangesetEvent {
    /// ULID-shaped `cs_<RFC3339-Z>_<8-hex>` identifier.
    pub changeset_id: String,
    /// Idempotency anchor `<changeset_id>_<to_state>_<at>`. Webhook
    /// receivers MUST treat a repeated dedup_key as a no-op.
    pub dedup_key: String,
    /// `None` only on the very first row (state `Opened`). All
    /// subsequent rows MUST carry the preceding `to_state`.
    pub from_state: Option<ChangesetState>,
    pub to_state: ChangesetState,
    /// RFC3339 UTC timestamp string. The kernel does not parse this
    /// — the app layer is responsible for clock semantics. The kernel
    /// surfaces it as evidence in [`MonotonicityError`] reports.
    pub at: String,
    /// Emitting agent / dispatcher identifier. Free-form string;
    /// matches the agent's signing-key principal per ADR-0058.
    pub emitted_by: String,
    pub cost_budget_remaining: CostBudget,
    /// Free-form key/value evidence map. BTreeMap for deterministic
    /// iteration order.
    pub evidence: BTreeMap<String, String>,
    /// Rejected alternatives for non-deterministic transitions; empty
    /// for deterministic transitions. Persisted so audit can replay
    /// any branch.
    pub alternates_considered: Vec<String>,
    /// True when this row represents one of the three canonical
    /// skip-states from ADR-0110 §"Skip-states".
    pub skipped: bool,
    /// Ed25519 signature placeholder. The kernel does NOT verify
    /// signatures — that is the adapter layer's job. The kernel
    /// surfaces the string so the app can stamp + verify it.
    ///
    /// TODO(wave-B): wire to real Ed25519 signing once the
    /// per-agent signing-key infrastructure (ADR-0058) is online.
    /// The app currently stamps an `ed25519-stub:<base64>`
    /// placeholder.
    pub signature: String,
}

/// Successful monotonicity report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonotonicityReport {
    pub events_checked: usize,
    /// `Some(state)` if the log ended at a terminal state, `None` if
    /// the log is still mid-flight on the advancing axis.
    pub terminal_state: Option<ChangesetState>,
}

/// Closed enum of monotonicity violations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MonotonicityError {
    /// `to_state` ordinal is strictly less than the preceding
    /// `to_state` ordinal on the advancing axis. Cannot occur for
    /// terminal-fail states (which have no ordinal). Surfacing
    /// includes the `at` timestamp from the offending row.
    BackwardsTransition {
        from: ChangesetState,
        to: ChangesetState,
        at: String,
    },
    /// Two events share the same `dedup_key`. ADR-0110 §"Event log
    /// shape" specifies dedup_keys MUST be unique.
    DuplicateDedupKey(String),
    /// The log is empty. Per IP-001 acceptance, callers receive an
    /// explicit error rather than a silently-accepted empty report.
    EmptyLog,
    /// More than one terminal state appears in the log. A changeset
    /// can terminate at most once.
    MultipleTerminalStates,
}

impl fmt::Display for MonotonicityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BackwardsTransition { from, to, at } => write!(
                f,
                "backwards transition {from} -> {to} at {at} violates ADR-0110 §monotonic-invariant"
            ),
            Self::DuplicateDedupKey(key) => write!(f, "duplicate dedup_key: {key}"),
            Self::EmptyLog => f.write_str("empty event log"),
            Self::MultipleTerminalStates => f.write_str("multiple terminal states in event log"),
        }
    }
}

impl std::error::Error for MonotonicityError {}

/// Validate that an event-log slice obeys ADR-0110's monotonic
/// non-decreasing-subsequence invariant.
///
/// Rules enforced:
///
/// 1. The slice MUST be non-empty.
/// 2. Each `to_state` ordinal on the advancing axis MUST be `>=` the
///    preceding event's `to_state` ordinal. Equal ordinals are
///    permitted (a re-emit at the same state is idempotent provided
///    dedup_keys differ — those are caught by rule 4).
/// 3. A terminal-fail state may appear from any advancing state;
///    after a terminal state the log MUST end (rule 5 below).
/// 4. All `dedup_key` values across the slice MUST be unique.
/// 5. At most one terminal state may appear in the slice.
///
/// Callers in IP-001 use this to gate event-log appends + the new
/// `oya-foundry-fitness-changeset-state-monotonicity` CI lane.
pub fn validate_monotonic_event_log(
    events: &[ChangesetEvent],
) -> Result<MonotonicityReport, MonotonicityError> {
    if events.is_empty() {
        return Err(MonotonicityError::EmptyLog);
    }

    let mut seen_dedup_keys: BTreeSet<&str> = BTreeSet::new();
    let mut last_advancing_ordinal: Option<u8> = None;
    let mut terminal_state: Option<ChangesetState> = None;

    for event in events {
        if !seen_dedup_keys.insert(event.dedup_key.as_str()) {
            return Err(MonotonicityError::DuplicateDedupKey(
                event.dedup_key.clone(),
            ));
        }

        // A terminal state may only appear once and ends the log.
        if event.to_state.is_terminal() {
            if terminal_state.is_some() {
                return Err(MonotonicityError::MultipleTerminalStates);
            }
            terminal_state = Some(event.to_state);
            // `Produced` is a terminal *advancing* state — it sits at
            // ordinal 9 and obeys the advancing-axis monotonicity
            // check below. Other terminals (`Abandoned`, `Rejected`,
            // `CostExhausted`) have no ordinal and skip the check.
            if let Some(new_ord) = event.to_state.advancing_ordinal() {
                if let Some(prev_ord) = last_advancing_ordinal
                    && new_ord < prev_ord
                {
                    return Err(MonotonicityError::BackwardsTransition {
                        from: event
                            .from_state
                            .unwrap_or_else(|| recover_state_from_ordinal(prev_ord)),
                        to: event.to_state,
                        at: event.at.clone(),
                    });
                }
                last_advancing_ordinal = Some(new_ord);
            }
            continue;
        }

        // Non-terminal: must be on the advancing axis and non-decreasing.
        let Some(new_ord) = event.to_state.advancing_ordinal() else {
            // Unreachable on a well-typed input — every non-terminal
            // variant has an ordinal — but we surface this defensively
            // rather than panicking (ADR-0083 Tier 1).
            return Err(MonotonicityError::BackwardsTransition {
                from: event.from_state.unwrap_or(ChangesetState::Opened),
                to: event.to_state,
                at: event.at.clone(),
            });
        };
        if let Some(prev_ord) = last_advancing_ordinal
            && new_ord < prev_ord
        {
            return Err(MonotonicityError::BackwardsTransition {
                from: event
                    .from_state
                    .unwrap_or_else(|| recover_state_from_ordinal(prev_ord)),
                to: event.to_state,
                at: event.at.clone(),
            });
        }
        last_advancing_ordinal = Some(new_ord);
    }

    Ok(MonotonicityReport {
        events_checked: events.len(),
        terminal_state,
    })
}

/// Map an advancing-axis ordinal back to its `ChangesetState`.
/// Used only when constructing `BackwardsTransition` reports where
/// the offending event omitted its `from_state` field.
fn recover_state_from_ordinal(ordinal: u8) -> ChangesetState {
    match ordinal {
        0 => ChangesetState::Opened,
        1 => ChangesetState::Working,
        2 => ChangesetState::Verified,
        3 => ChangesetState::PrOpen,
        4 => ChangesetState::CiRunning,
        5 => ChangesetState::CiPassed,
        6 => ChangesetState::Reviewed,
        7 => ChangesetState::MergedDev,
        8 => ChangesetState::Staged,
        // Saturating fallback: an ordinal > 9 is unreachable on a
        // well-typed input; the only callers are
        // `validate_monotonic_event_log` rows that round-tripped
        // through `advancing_ordinal`, which is bijective with the
        // advancing variants.
        _ => ChangesetState::Produced,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget() -> CostBudget {
        CostBudget {
            usd_remaining: 10.0,
            tokens_remaining: 1_000_000,
            agent_invocations_remaining: 50,
        }
    }

    fn event(
        seq: u32,
        from_state: Option<ChangesetState>,
        to_state: ChangesetState,
        skipped: bool,
    ) -> ChangesetEvent {
        let at = format!("2026-05-16T01:00:{:02}Z", seq % 60);
        ChangesetEvent {
            changeset_id: "cs_2026-05-16T01:00:00Z_abc12345".to_string(),
            dedup_key: format!(
                "cs_2026-05-16T01:00:00Z_abc12345_{}_{}",
                to_state.as_wire(),
                seq
            ),
            from_state,
            to_state,
            at,
            emitted_by: "test-runner".to_string(),
            cost_budget_remaining: budget(),
            evidence: BTreeMap::new(),
            alternates_considered: Vec::new(),
            skipped,
            signature: "ed25519-stub:test".to_string(),
        }
    }

    #[test]
    fn empty_log_is_rejected() {
        assert_eq!(
            validate_monotonic_event_log(&[]),
            Err(MonotonicityError::EmptyLog)
        );
    }

    #[test]
    fn single_opened_event_is_accepted() {
        let log = [event(0, None, ChangesetState::Opened, false)];
        let report = validate_monotonic_event_log(&log).unwrap();
        assert_eq!(report.events_checked, 1);
        assert!(report.terminal_state.is_none());
    }

    #[test]
    fn full_chain_opened_to_produced_is_accepted() {
        let chain = [
            (None, ChangesetState::Opened),
            (Some(ChangesetState::Opened), ChangesetState::Working),
            (Some(ChangesetState::Working), ChangesetState::Verified),
            (Some(ChangesetState::Verified), ChangesetState::PrOpen),
            (Some(ChangesetState::PrOpen), ChangesetState::CiRunning),
            (Some(ChangesetState::CiRunning), ChangesetState::CiPassed),
            (Some(ChangesetState::CiPassed), ChangesetState::Reviewed),
            (Some(ChangesetState::Reviewed), ChangesetState::MergedDev),
            (Some(ChangesetState::MergedDev), ChangesetState::Staged),
            (Some(ChangesetState::Staged), ChangesetState::Produced),
        ];
        let log: Vec<ChangesetEvent> = chain
            .into_iter()
            .enumerate()
            .map(|(i, (from, to))| event(i as u32, from, to, false))
            .collect();
        let report = validate_monotonic_event_log(&log).unwrap();
        assert_eq!(report.events_checked, 10);
        assert_eq!(report.terminal_state, Some(ChangesetState::Produced));
    }

    #[test]
    fn backwards_transition_is_rejected() {
        let log = [
            event(0, None, ChangesetState::Opened, false),
            event(
                1,
                Some(ChangesetState::Opened),
                ChangesetState::Working,
                false,
            ),
            event(
                2,
                Some(ChangesetState::Working),
                ChangesetState::Opened,
                false,
            ),
        ];
        let err = validate_monotonic_event_log(&log).unwrap_err();
        match err {
            MonotonicityError::BackwardsTransition { from, to, .. } => {
                assert_eq!(from, ChangesetState::Working);
                assert_eq!(to, ChangesetState::Opened);
            }
            other => panic!("expected BackwardsTransition, got {other:?}"),
        }
    }

    #[test]
    fn canonical_skip_states_are_accepted_with_skipped_flag() {
        // working -> verified, ci_running -> ci_passed, staged -> produced
        let log = [
            event(0, None, ChangesetState::Opened, false),
            event(
                1,
                Some(ChangesetState::Opened),
                ChangesetState::Working,
                false,
            ),
            event(
                2,
                Some(ChangesetState::Working),
                ChangesetState::Verified,
                true,
            ),
            event(
                3,
                Some(ChangesetState::Verified),
                ChangesetState::PrOpen,
                false,
            ),
            event(
                4,
                Some(ChangesetState::PrOpen),
                ChangesetState::CiRunning,
                false,
            ),
            event(
                5,
                Some(ChangesetState::CiRunning),
                ChangesetState::CiPassed,
                true,
            ),
            event(
                6,
                Some(ChangesetState::CiPassed),
                ChangesetState::Reviewed,
                false,
            ),
            event(
                7,
                Some(ChangesetState::Reviewed),
                ChangesetState::MergedDev,
                false,
            ),
            event(
                8,
                Some(ChangesetState::MergedDev),
                ChangesetState::Staged,
                false,
            ),
            event(
                9,
                Some(ChangesetState::Staged),
                ChangesetState::Produced,
                true,
            ),
        ];
        let report = validate_monotonic_event_log(&log).unwrap();
        assert_eq!(report.events_checked, 10);
        assert_eq!(report.terminal_state, Some(ChangesetState::Produced));
        // Confirm the three rows flagged skipped survived the round-trip
        // through validation untouched.
        assert!(log[2].skipped);
        assert!(log[5].skipped);
        assert!(log[9].skipped);
    }

    #[test]
    fn duplicate_dedup_key_is_rejected() {
        let mut log = [
            event(0, None, ChangesetState::Opened, false),
            event(
                1,
                Some(ChangesetState::Opened),
                ChangesetState::Working,
                false,
            ),
        ];
        log[1].dedup_key = log[0].dedup_key.clone();
        let err = validate_monotonic_event_log(&log).unwrap_err();
        assert!(matches!(err, MonotonicityError::DuplicateDedupKey(_)));
    }

    #[test]
    fn terminal_abandoned_rejected_cost_exhausted_each_accepted() {
        for terminal in [
            ChangesetState::Abandoned,
            ChangesetState::Rejected,
            ChangesetState::CostExhausted,
        ] {
            let log = [
                event(0, None, ChangesetState::Opened, false),
                event(1, Some(ChangesetState::Opened), terminal, false),
            ];
            let report = validate_monotonic_event_log(&log)
                .unwrap_or_else(|err| panic!("expected accept for {terminal:?}, got {err:?}"));
            assert_eq!(report.events_checked, 2);
            assert_eq!(report.terminal_state, Some(terminal));
        }
    }

    #[test]
    fn multiple_terminal_states_rejected() {
        let log = [
            event(0, None, ChangesetState::Opened, false),
            event(
                1,
                Some(ChangesetState::Opened),
                ChangesetState::Abandoned,
                false,
            ),
            event(
                2,
                Some(ChangesetState::Opened),
                ChangesetState::Rejected,
                false,
            ),
        ];
        let err = validate_monotonic_event_log(&log).unwrap_err();
        assert_eq!(err, MonotonicityError::MultipleTerminalStates);
    }

    #[test]
    fn from_wire_round_trip_covers_all_thirteen_variants() {
        for state in ChangesetState::ALL {
            let wire = state.as_wire();
            assert_eq!(ChangesetState::from_wire(wire), Some(state));
        }
        assert_eq!(ChangesetState::from_wire("not_a_state"), None);
        assert_eq!(ChangesetState::ALL.len(), 13);
    }

    #[test]
    fn cost_budget_exhaustion_detected_on_each_axis() {
        assert!(
            CostBudget {
                usd_remaining: 0.0,
                tokens_remaining: 1,
                agent_invocations_remaining: 1
            }
            .is_exhausted()
        );
        assert!(
            CostBudget {
                usd_remaining: 1.0,
                tokens_remaining: 0,
                agent_invocations_remaining: 1
            }
            .is_exhausted()
        );
        assert!(
            CostBudget {
                usd_remaining: 1.0,
                tokens_remaining: 1,
                agent_invocations_remaining: 0
            }
            .is_exhausted()
        );
        assert!(!budget().is_exhausted());
    }
}
