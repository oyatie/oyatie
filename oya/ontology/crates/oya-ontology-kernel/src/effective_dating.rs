//! Bitemporal effective-dating primitives for ontology-kernel state.
//!
//! The kernel models valid time and transaction time as half-open intervals:
//! `[start, end_exclusive)`. Half-open ranges make adjacent versions safe at
//! exact boundaries and keep overlap detection deterministic under clock skew.

/// A logical timestamp used by valid-time and transaction-time ranges.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EffectiveInstant(i64); // data_class: INTERNAL_ONLY

impl EffectiveInstant {
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

impl From<i64> for EffectiveInstant {
    fn from(value: i64) -> Self {
        Self::new(value)
    }
}

/// Half-open time range: `[start, end_exclusive)` or `[start, +∞)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectiveTimeRange {
    pub start: EffectiveInstant,                 // data_class: INTERNAL_ONLY
    pub end_exclusive: Option<EffectiveInstant>, // data_class: INTERNAL_ONLY
}

impl EffectiveTimeRange {
    pub fn new(
        start: impl Into<EffectiveInstant>,
        end_exclusive: Option<impl Into<EffectiveInstant>>,
    ) -> Result<Self, EffectiveDatingError> {
        let start = start.into();
        let end_exclusive = end_exclusive.map(Into::into);
        if end_exclusive.is_some_and(|end| end <= start) {
            return Err(EffectiveDatingError::InvalidRange);
        }
        Ok(Self {
            start,
            end_exclusive,
        })
    }

    #[must_use]
    pub fn open_ended(start: impl Into<EffectiveInstant>) -> Self {
        Self {
            start: start.into(),
            end_exclusive: None,
        }
    }

    #[must_use]
    pub fn contains(&self, instant: impl Into<EffectiveInstant>) -> bool {
        let instant = instant.into();
        self.start <= instant && self.end_exclusive.is_none_or(|end| instant < end)
    }

    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.starts_before_other_end(other) && other.starts_before_other_end(self)
    }

    fn starts_before_other_end(&self, other: &Self) -> bool {
        other.end_exclusive.is_none_or(|end| self.start < end)
    }
}

/// Bitemporal envelope for one effective-dated ontology version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitemporalRange {
    pub valid_time: EffectiveTimeRange, // data_class: INTERNAL_ONLY
    pub transaction_time: EffectiveTimeRange, // data_class: INTERNAL_ONLY
}

impl BitemporalRange {
    #[must_use]
    pub const fn new(valid_time: EffectiveTimeRange, transaction_time: EffectiveTimeRange) -> Self {
        Self {
            valid_time,
            transaction_time,
        }
    }

    #[must_use]
    pub fn contains(
        &self,
        valid_at: impl Into<EffectiveInstant>,
        transaction_at: impl Into<EffectiveInstant>,
    ) -> bool {
        self.valid_time.contains(valid_at) && self.transaction_time.contains(transaction_at)
    }

    #[must_use]
    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.valid_time.overlaps(&other.valid_time)
            && self.transaction_time.overlaps(&other.transaction_time)
    }
}

/// One value plus its bitemporal envelope and deterministic revision marker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveDatedVersion<T> {
    pub value: T,               // data_class: EFFECTIVE_DATED_VALUE_INHERITS_CALLER_CLASS
    pub range: BitemporalRange, // data_class: INTERNAL_ONLY
    pub revision: u64,          // data_class: INTERNAL_ONLY
}

impl<T> EffectiveDatedVersion<T> {
    pub fn new(
        value: T,
        range: BitemporalRange,
        revision: u64,
    ) -> Result<Self, EffectiveDatingError> {
        if revision == 0 {
            return Err(EffectiveDatingError::InvalidRevision);
        }
        Ok(Self {
            value,
            range,
            revision,
        })
    }
}

/// In-memory bitemporal history for one ontology object or property stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveDatedHistory<T> {
    versions: Vec<EffectiveDatedVersion<T>>, // data_class: EFFECTIVE_DATED_VALUE_INHERITS_CALLER_CLASS
    next_revision: u64,                      // data_class: INTERNAL_ONLY
}

impl<T> Default for EffectiveDatedHistory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EffectiveDatedHistory<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            versions: Vec::new(),
            next_revision: 1,
        }
    }

    pub fn insert(
        &mut self,
        value: T,
        range: BitemporalRange,
    ) -> Result<u64, EffectiveDatingError> {
        let revision = self.next_revision;
        let version = EffectiveDatedVersion::new(value, range, revision)?;
        self.insert_version(version)?;
        Ok(revision)
    }

    pub fn insert_version(
        &mut self,
        version: EffectiveDatedVersion<T>,
    ) -> Result<(), EffectiveDatingError> {
        if self
            .versions
            .iter()
            .any(|existing| existing.range.conflicts_with(&version.range))
        {
            return Err(EffectiveDatingError::OverlappingBitemporalRange);
        }
        self.next_revision = self.next_revision.max(version.revision.saturating_add(1));
        self.versions.push(version);
        self.sort_versions();
        Ok(())
    }

    pub fn version_as_of(
        &self,
        valid_at: impl Into<EffectiveInstant>,
        transaction_at: impl Into<EffectiveInstant>,
    ) -> Result<&EffectiveDatedVersion<T>, EffectiveDatingError> {
        let valid_at = valid_at.into();
        let transaction_at = transaction_at.into();
        self.versions
            .iter()
            .rev()
            .find(|version| version.range.contains(valid_at, transaction_at))
            .ok_or(EffectiveDatingError::NoVersionAtAsOf)
    }

    pub fn as_of(
        &self,
        valid_at: impl Into<EffectiveInstant>,
        transaction_at: impl Into<EffectiveInstant>,
    ) -> Result<&T, EffectiveDatingError> {
        self.version_as_of(valid_at, transaction_at)
            .map(|version| &version.value)
    }

    #[must_use]
    pub fn versions(&self) -> &[EffectiveDatedVersion<T>] {
        &self.versions
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.versions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.versions.is_empty()
    }

    fn sort_versions(&mut self) {
        self.versions.sort_by_key(|version| {
            (
                version.range.transaction_time.start,
                version.range.valid_time.start,
                version.revision,
            )
        });
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveDatingError {
    InvalidRange,
    InvalidRevision,
    OverlappingBitemporalRange,
    NoVersionAtAsOf,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: i64, end: i64) -> EffectiveTimeRange {
        EffectiveTimeRange::new(start, Some(end)).expect("valid half-open range")
    }

    fn open(start: i64) -> EffectiveTimeRange {
        EffectiveTimeRange::open_ended(start)
    }

    fn bitemporal(
        valid_time: EffectiveTimeRange,
        transaction_time: EffectiveTimeRange,
    ) -> BitemporalRange {
        BitemporalRange::new(valid_time, transaction_time)
    }

    #[test]
    fn half_open_ranges_include_start_and_exclude_end() {
        let measured = range(10, 20);

        assert!(measured.contains(10));
        assert!(measured.contains(19));
        assert!(!measured.contains(20));
        assert_eq!(
            EffectiveTimeRange::new(20, Some(20)),
            Err(EffectiveDatingError::InvalidRange)
        );
    }

    #[test]
    fn as_of_returns_correct_version_across_valid_and_transaction_time() {
        let mut history = EffectiveDatedHistory::new();
        history
            .insert("v1", bitemporal(range(0, 10), open(0)))
            .expect("first non-overlapping version inserts");
        history
            .insert("v2", bitemporal(range(10, 20), open(0)))
            .expect("adjacent valid-time version inserts");

        assert_eq!(history.as_of(5, 1), Ok(&"v1"));
        assert_eq!(history.as_of(10, 1), Ok(&"v2"));
        assert_eq!(
            history.as_of(25, 1),
            Err(EffectiveDatingError::NoVersionAtAsOf)
        );
    }

    #[test]
    fn overlapping_valid_ranges_rejected_when_transaction_time_overlaps() {
        let mut history = EffectiveDatedHistory::new();
        history
            .insert("original", bitemporal(range(0, 10), open(0)))
            .expect("seed inserts");

        assert_eq!(
            history.insert("overlap", bitemporal(range(5, 15), open(1))),
            Err(EffectiveDatingError::OverlappingBitemporalRange)
        );
        assert_eq!(history.len(), 1, "rejected overlap must not mutate history");
    }

    #[test]
    fn open_ended_ranges_support_far_future_as_of_queries() {
        let mut history = EffectiveDatedHistory::new();
        history
            .insert("current", bitemporal(open(100), open(200)))
            .expect("open-ended version inserts");

        assert_eq!(history.as_of(100, 200), Ok(&"current"));
        assert_eq!(history.as_of(1_000_000, 1_000_000), Ok(&"current"));
    }

    #[test]
    fn out_of_order_transaction_time_inserts_are_sorted_and_queryable() {
        let mut history = EffectiveDatedHistory::new();
        history
            .insert("late", bitemporal(range(0, 10), range(100, 200)))
            .expect("later transaction-time version inserts first");
        history
            .insert("early", bitemporal(range(0, 10), range(10, 20)))
            .expect("earlier non-overlapping transaction-time version inserts second");

        let values = history
            .versions()
            .iter()
            .map(|version| version.value)
            .collect::<Vec<_>>();
        assert_eq!(values, vec!["early", "late"]);
        assert_eq!(history.as_of(5, 15), Ok(&"early"));
        assert_eq!(history.as_of(5, 150), Ok(&"late"));
    }

    #[test]
    fn property_grid_exercises_validity_range_overlap_boundaries() {
        for start in 0..8 {
            for width in 1..5 {
                let end = start + width;
                let first = range(start, end);
                let touching = range(end, end + width);
                let overlapping = range(end - 1, end + width);

                assert!(first.contains(start));
                assert!(!first.contains(end));
                assert!(!first.overlaps(&touching));
                assert!(first.overlaps(&overlapping));
            }
        }
    }
}
