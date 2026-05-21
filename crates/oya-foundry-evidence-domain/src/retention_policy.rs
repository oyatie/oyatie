//! Retention policy domain types for M04-P04 evidence-retention-audit.
//!
//! Encodes how long evidence records must be retained under a given
//! regulatory schedule.  Pure value types — no I/O, no serde, std-only.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// The minimum number of calendar days evidence must be retained.
///
/// A value of zero is invalid; callers must supply a positive duration.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RetentionDays(pub u32);

impl RetentionDays {
    /// Returns the inner day count.
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

/// Regulatory schedule that governs how evidence is retained.
///
/// Each named variant maps to a product-level retention duty rather than a
/// jurisdiction identifier. The `Custom` variant is provided for extension
/// without breaking the enum; callers should prefer a named variant wherever
/// one exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RegulatorySchedule {
    /// Standard evidence record — 3-year minimum (1 095 days).
    EvidenceRecordThreeYear,
    /// Processing activity record — 3-year minimum (1 095 days).
    ProcessingActivityRecordThreeYear,
    /// Custom schedule; duration is caller-supplied.
    Custom,
}

impl RegulatorySchedule {
    /// Default minimum retention period prescribed by the schedule.
    ///
    /// Returns `None` for `Custom` because the duration is caller-defined.
    pub fn default_retention_days(self) -> Option<RetentionDays> {
        match self {
            RegulatorySchedule::EvidenceRecordThreeYear => Some(RetentionDays(1_095)),
            RegulatorySchedule::ProcessingActivityRecordThreeYear => Some(RetentionDays(1_095)),
            RegulatorySchedule::Custom => None,
        }
    }
}

/// Errors produced when constructing a [`RetentionPolicy`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetentionPolicyError {
    /// The supplied retention duration is zero, which is never valid.
    ZeroRetentionDays,
    /// `Custom` schedule requires an explicit `RetentionDays` value.
    MissingCustomDuration,
    /// The supplied duration is shorter than the schedule's mandatory minimum.
    BelowScheduleMinimum {
        supplied: RetentionDays,
        minimum: RetentionDays,
    },
}

/// A validated retention policy binding a [`RegulatorySchedule`] to a
/// concrete [`RetentionDays`] duration.
///
/// # Invariants
/// - `retention_days` > 0 always.
/// - `retention_days` ≥ `schedule.default_retention_days()` when the
///   schedule prescribes a minimum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    schedule: RegulatorySchedule,
    retention_days: RetentionDays,
}

impl RetentionPolicy {
    /// Returns the regulatory schedule bound to this policy.
    pub fn schedule(&self) -> RegulatorySchedule {
        self.schedule
    }

    /// Returns the validated retention duration.
    pub fn retention_days(&self) -> RetentionDays {
        self.retention_days
    }

    /// Construct a [`RetentionPolicy`] from a schedule and an explicit
    /// duration, enforcing all invariants. This is the ONLY way to obtain a
    /// `RetentionPolicy` (struct fields are private), so the documented
    /// invariants always hold for any value of this type.
    pub fn new(
        schedule: RegulatorySchedule,
        retention_days: RetentionDays,
    ) -> Result<Self, RetentionPolicyError> {
        if retention_days.0 == 0 {
            return Err(RetentionPolicyError::ZeroRetentionDays);
        }
        if let Some(minimum) = schedule.default_retention_days()
            && retention_days < minimum
        {
            return Err(RetentionPolicyError::BelowScheduleMinimum {
                supplied: retention_days,
                minimum,
            });
        }
        Ok(Self {
            schedule,
            retention_days,
        })
    }

    /// Construct a [`RetentionPolicy`] using the schedule's prescribed
    /// default minimum.  Fails for `Custom` (no default exists).
    pub fn from_schedule_default(
        schedule: RegulatorySchedule,
    ) -> Result<Self, RetentionPolicyError> {
        let retention_days = schedule
            .default_retention_days()
            .ok_or(RetentionPolicyError::MissingCustomDuration)?;
        Self::new(schedule, retention_days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_record_default_is_1095_days() {
        let policy =
            RetentionPolicy::from_schedule_default(RegulatorySchedule::EvidenceRecordThreeYear)
                .unwrap();
        assert_eq!(policy.retention_days().as_u32(), 1_095);
        assert_eq!(
            policy.schedule(),
            RegulatorySchedule::EvidenceRecordThreeYear
        );
    }

    #[test]
    fn processing_activity_record_default_is_1095_days() {
        let policy = RetentionPolicy::from_schedule_default(
            RegulatorySchedule::ProcessingActivityRecordThreeYear,
        )
        .unwrap();
        assert_eq!(policy.retention_days().as_u32(), 1_095);
    }

    #[test]
    fn custom_schedule_requires_explicit_duration() {
        let err = RetentionPolicy::from_schedule_default(RegulatorySchedule::Custom).unwrap_err();
        assert_eq!(err, RetentionPolicyError::MissingCustomDuration);
    }

    #[test]
    fn custom_schedule_accepts_explicit_duration() {
        let policy = RetentionPolicy::new(RegulatorySchedule::Custom, RetentionDays(365)).unwrap();
        assert_eq!(policy.retention_days().as_u32(), 365);
    }

    #[test]
    fn zero_retention_days_is_rejected() {
        let err = RetentionPolicy::new(
            RegulatorySchedule::EvidenceRecordThreeYear,
            RetentionDays(0),
        )
        .unwrap_err();
        assert_eq!(err, RetentionPolicyError::ZeroRetentionDays);
    }

    #[test]
    fn below_minimum_is_rejected() {
        let err = RetentionPolicy::new(
            RegulatorySchedule::EvidenceRecordThreeYear,
            RetentionDays(364),
        )
        .unwrap_err();
        assert_eq!(
            err,
            RetentionPolicyError::BelowScheduleMinimum {
                supplied: RetentionDays(364),
                minimum: RetentionDays(1_095),
            }
        );
    }

    #[test]
    fn above_minimum_is_accepted() {
        let policy = RetentionPolicy::new(
            RegulatorySchedule::EvidenceRecordThreeYear,
            RetentionDays(2_000),
        )
        .unwrap();
        assert_eq!(policy.retention_days().as_u32(), 2_000);
    }

    #[test]
    fn retention_days_ordering() {
        assert!(RetentionDays(100) < RetentionDays(200));
        assert!(RetentionDays(200) > RetentionDays(100));
        assert_eq!(RetentionDays(100), RetentionDays(100));
    }
}
