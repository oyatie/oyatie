//! ADR-0109 lifecycle-status gate kernel — I/O-free.
//!
//! ADR-0109 §1 declares one generic lifecycle kernel with each lifecycle expressed as DATA (a JSON
//! config under `specs/lifecycle-configs/`). §2 then shipped a thin per-lifecycle dev-CLI crate
//! under `tools/oya-governance-<name>-lifecycle-app/`. Nine of those crates existed, built, and
//! were never referenced by `.github/workflows/**` — they enforced nothing. This crate is the
//! §1-faithful replacement: ONE required lane parameterized over every config on disk.
//!
//! Two dimensions, both pure here and fed by the live-corpus test:
//!
//! 1. **Completeness.** Every config discovered on disk must be evaluated, and every lane named in
//!    the frozen baseline must still have a config. A config nobody evaluates is exactly the
//!    dark-gate failure this crate exists to retire, so it fails closed rather than being skipped.
//! 2. **Shrink-only violation ratchet (regression half).** Per `(lane, violation_kind)` the observed
//!    count may not exceed the frozen baseline, and a NEW `(lane, kind)` pair is born-blocking.
//!    Counts that fall below baseline are NOT merge-blocking (PROCESS_TAX DELETE of BaselineStale
//!    hand re-freeze); slack may accumulate until a reviewed shrink.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// Frozen per-`(lane, violation_kind)` violation counts.
pub type ViolationCounts = BTreeMap<String, BTreeMap<String, usize>>;

/// The gate's policy-as-data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    /// Repo-relative directory holding one `<lane>.json` config per lifecycle.
    pub configs_dir: String,
    /// Frozen shrink-only violation counts, keyed lane → violation_kind → count.
    pub baseline: ViolationCounts,
    /// Lanes KNOWN not to observe a live corpus today, each with its defect recorded. A listed lane
    /// may fail discovery or observe zero artifacts without reddening the required context; an
    /// UNLISTED lane doing either is born-blocking. Shrink-only: a listed lane that starts working
    /// makes its entry stale and must be removed in the same PR, so a repaired lane cannot keep a
    /// standing excuse. Growing this set requires a review-visible edit, never a silent append.
    pub known_broken_lanes: BTreeMap<String, String>,
}

/// A gate finding. Every variant is blocking; there is deliberately no advisory tier, because an
/// advisory lifecycle lane is indistinguishable from the dark lanes this gate replaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Finding {
    /// A config exists on disk but no lane evaluated it.
    ConfigNotEvaluated { lane: String },
    /// A config's declared source root could not be walked. This is the reorg-move blindness class:
    /// a lane whose corpus moved out from under it reports zero violations forever, which reads
    /// exactly like compliance. It is blocking, never skipped.
    LaneDiscoveryFailed { lane: String, error: String },
    /// A lane walked its roots successfully and matched ZERO artifacts. The kernel's glob only
    /// errors on a missing DIRECTORY, so a live directory whose pattern matches nothing returns
    /// `Ok(vec![])` and evaluates clean. Zero observations is not evidence of zero violations.
    LaneDiscoveredNothing { lane: String },
    /// A lane recorded as known-broken now observes a live corpus; its ledger entry must go.
    KnownBrokenLaneNowLive { lane: String, artifacts: usize },
    /// The known-broken ledger names a lane with no config.
    KnownBrokenLaneWithoutConfig { lane: String },
    /// The baseline names a lane whose config is gone; the baseline row must be removed.
    BaselineLaneWithoutConfig { lane: String },
    /// A `(lane, kind)` pair with no baseline row at all.
    UnbaselinedViolation {
        lane: String,
        kind: String,
        observed: usize,
    },
    /// Observed count exceeds the frozen baseline.
    BaselineRegression {
        lane: String,
        kind: String,
        observed: usize,
        baseline: usize,
    },
    /// Observed count fell below the frozen baseline; shrink it in the same PR.
    BaselineStale {
        lane: String,
        kind: String,
        observed: usize,
        baseline: usize,
    },
}

impl Finding {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ConfigNotEvaluated { .. } => "lifecycle_status_config_not_evaluated",
            Self::LaneDiscoveryFailed { .. } => "lifecycle_status_lane_discovery_failed",
            Self::LaneDiscoveredNothing { .. } => "lifecycle_status_lane_discovered_nothing",
            Self::KnownBrokenLaneNowLive { .. } => "lifecycle_status_known_broken_lane_now_live",
            Self::KnownBrokenLaneWithoutConfig { .. } => {
                "lifecycle_status_known_broken_lane_without_config"
            }
            Self::BaselineLaneWithoutConfig { .. } => "lifecycle_status_baseline_lane_without_config",
            Self::UnbaselinedViolation { .. } => "lifecycle_status_unbaselined_violation",
            Self::BaselineRegression { .. } => "lifecycle_status_baseline_regression",
            Self::BaselineStale { .. } => "lifecycle_status_baseline_stale",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::ConfigNotEvaluated { lane } => format!(
                "{}: lifecycle config `{lane}` was discovered but never evaluated — a config no \
                 lane evaluates enforces nothing",
                self.code()
            ),
            Self::LaneDiscoveryFailed { lane, error } => format!(
                "{}: lane `{lane}` could not walk its declared corpus ({error}) — a lane that \
                 cannot see its artifacts reports zero violations, which is indistinguishable from \
                 compliance",
                self.code()
            ),
            Self::LaneDiscoveredNothing { lane } => format!(
                "{}: lane `{lane}` matched ZERO artifacts — its glob is live but selects nothing, \
                 so it evaluates clean without observing anything",
                self.code()
            ),
            Self::KnownBrokenLaneNowLive { lane, artifacts } => format!(
                "{}: lane `{lane}` now observes {artifacts} artifact(s) — remove its \
                 known_broken_lanes entry and baseline its real violations in this PR",
                self.code()
            ),
            Self::KnownBrokenLaneWithoutConfig { lane } => format!(
                "{}: known_broken_lanes names `{lane}` but no config exists for it — drop the entry",
                self.code()
            ),
            Self::BaselineLaneWithoutConfig { lane } => format!(
                "{}: baseline names lane `{lane}` but no config exists for it — drop the baseline \
                 row in the same PR that dropped the config",
                self.code()
            ),
            Self::UnbaselinedViolation {
                lane,
                kind,
                observed,
            } => format!(
                "{}: {lane}/{kind} has {observed} violation(s) and no baseline row — fix the \
                 artifacts rather than adding a row",
                self.code()
            ),
            Self::BaselineRegression {
                lane,
                kind,
                observed,
                baseline,
            } => format!(
                "{}: {lane}/{kind} grew {baseline} -> {observed}",
                self.code()
            ),
            Self::BaselineStale {
                lane,
                kind,
                observed,
                baseline,
            } => format!(
                "{}: {lane}/{kind} shrank {baseline} -> {observed}; shrink the frozen baseline in \
                 this PR",
                self.code()
            ),
        }
    }
}

/// Parse the gate's policy JSON.
pub fn parse_policy(raw: &str) -> Result<Policy, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|error| format!("policy is not valid JSON: {error}"))?;

    let configs_dir = value
        .get("configs_dir")
        .and_then(serde_json::Value::as_str)
        .ok_or("policy is missing a string `configs_dir`")?
        .to_owned();

    let baseline_value = value
        .get("frozen_violation_baseline")
        .and_then(serde_json::Value::as_object)
        .ok_or("policy is missing an object `frozen_violation_baseline`")?;

    let mut baseline = ViolationCounts::new();
    for (lane, kinds) in baseline_value {
        // `_comment` / `_provenance` are review prose, not lane rows.
        if lane.starts_with('_') {
            continue;
        }
        let kinds = kinds
            .as_object()
            .ok_or_else(|| format!("baseline lane `{lane}` must map violation_kind -> count"))?;
        let mut row = BTreeMap::new();
        for (kind, count) in kinds {
            if kind.starts_with('_') {
                continue;
            }
            let count = count.as_u64().ok_or_else(|| {
                format!("baseline {lane}/{kind} must be a non-negative integer count")
            })?;
            // A zero row is not a baseline, it is noise that hides whether the pair was ever
            // measured. Require its removal so `absent` unambiguously means `must be zero`.
            if count == 0 {
                return Err(format!(
                    "baseline {lane}/{kind} is 0 — remove the row; an absent pair is already \
                     born-blocking"
                ));
            }
            row.insert(kind.clone(), count as usize);
        }
        baseline.insert(lane.clone(), row);
    }

    let broken_value = value
        .get("known_broken_lanes")
        .and_then(serde_json::Value::as_object)
        .ok_or("policy is missing an object `known_broken_lanes`")?;

    let mut known_broken_lanes = BTreeMap::new();
    for (lane, entry) in broken_value {
        if lane.starts_with('_') {
            continue;
        }
        let defect = entry
            .get("defect")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("known_broken_lanes.{lane} must carry a string `defect`"))?;
        // A recorded defect without a stated resolution is a permanent excuse, not debt.
        let resolution = entry
            .get("resolution")
            .and_then(serde_json::Value::as_str)
            .filter(|resolution| !resolution.trim().is_empty())
            .ok_or_else(|| {
                format!("known_broken_lanes.{lane} must carry a non-empty string `resolution`")
            })?;
        known_broken_lanes.insert(lane.clone(), format!("{defect} → {resolution}"));
    }

    Ok(Policy {
        configs_dir,
        baseline,
        known_broken_lanes,
    })
}

/// What one lane actually observed when it walked the live corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LaneObservation {
    /// The lane could not walk a declared source root.
    DiscoveryFailed(String),
    /// The lane walked its roots and evaluated `artifacts` of them.
    Observed {
        artifacts: usize,
        violations: BTreeMap<String, usize>,
    },
}

/// Compare the live corpus against the frozen baseline and the known-broken ledger.
///
/// `discovered_lanes` is every config found on disk; `observations` is what each lane actually saw.
/// Findings come back grouped by dimension; the caller feeds ordered `BTreeMap`s and a sorted
/// `discovered_lanes`, so the output is deterministic.
pub fn compare(
    discovered_lanes: &[String],
    observations: &BTreeMap<String, LaneObservation>,
    policy: &Policy,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    for lane in discovered_lanes {
        if !observations.contains_key(lane) {
            findings.push(Finding::ConfigNotEvaluated { lane: lane.clone() });
        }
    }

    for lane in policy.baseline.keys() {
        if !discovered_lanes.iter().any(|found| found == lane) {
            findings.push(Finding::BaselineLaneWithoutConfig { lane: lane.clone() });
        }
    }
    for lane in policy.known_broken_lanes.keys() {
        if !discovered_lanes.iter().any(|found| found == lane) {
            findings.push(Finding::KnownBrokenLaneWithoutConfig { lane: lane.clone() });
        }
    }

    for (lane, observation) in observations {
        let known_broken = policy.known_broken_lanes.contains_key(lane);
        match observation {
            LaneObservation::DiscoveryFailed(error) if !known_broken => {
                findings.push(Finding::LaneDiscoveryFailed {
                    lane: lane.clone(),
                    error: error.clone(),
                });
            }
            LaneObservation::Observed { artifacts: 0, .. } if !known_broken => {
                findings.push(Finding::LaneDiscoveredNothing { lane: lane.clone() });
            }
            LaneObservation::Observed { artifacts, .. } if known_broken && *artifacts > 0 => {
                findings.push(Finding::KnownBrokenLaneNowLive {
                    lane: lane.clone(),
                    artifacts: *artifacts,
                });
            }
            _ => {}
        }
    }

    let empty = BTreeMap::new();
    let no_violations = BTreeMap::new();
    for (lane, observation) in observations {
        let kinds = match observation {
            LaneObservation::Observed { violations, .. } => violations,
            LaneObservation::DiscoveryFailed(_) => &no_violations,
        };
        let baselined = policy.baseline.get(lane).unwrap_or(&empty);
        for (kind, observed) in kinds {
            match baselined.get(kind) {
                None => findings.push(Finding::UnbaselinedViolation {
                    lane: lane.clone(),
                    kind: kind.clone(),
                    observed: *observed,
                }),
                Some(baseline) if observed > baseline => {
                    findings.push(Finding::BaselineRegression {
                        lane: lane.clone(),
                        kind: kind.clone(),
                        observed: *observed,
                        baseline: *baseline,
                    })
                }
                Some(_) => {}
            }
        }
        // PROCESS_TAX DELETE: BaselineStale (hand re-freeze when counts shrink) is not emitted as a
        // merge blocker. Regression above the frozen baseline remains born-blocking. Slack below
        // may accumulate until a reviewed shrink; docs deletes must not force clerkwork re-freeze.
    }

    findings
}

#[cfg(test)]
mod tests;
