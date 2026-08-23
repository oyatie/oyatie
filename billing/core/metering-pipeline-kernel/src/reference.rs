//! The in-memory reference sink — the executable specification of the
//! D-14 idempotent metering sink, proven by [`crate::conformance`] in
//! this crate's tests. NOT a production store: no durability across
//! process restart — the durable sink arrives via the G03 `data`
//! port behind the same trait.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::{
    DedupKey, IngestOutcome, LatenessPolicy, MeteringPipelineError, MeteringSink, UsageRecord,
};

/// In-memory reference implementation of [`MeteringSink`].
#[derive(Debug)]
pub struct InMemorySink {
    policy: LatenessPolicy,
    records: Mutex<BTreeMap<DedupKey, UsageRecord>>,
}

impl InMemorySink {
    /// An empty sink under the given lateness policy.
    #[must_use]
    pub fn new(policy: LatenessPolicy) -> Self {
        Self {
            policy,
            records: Mutex::new(BTreeMap::new()),
        }
    }

    /// Number of stored records (operator/test visibility).
    ///
    /// # Errors
    /// Returns [`MeteringPipelineError::SinkUnavailable`] when the store
    /// lock is poisoned.
    pub fn len(&self) -> Result<usize, MeteringPipelineError> {
        Ok(self.locked()?.len())
    }

    /// Whether the sink holds no records.
    ///
    /// # Errors
    /// Returns [`MeteringPipelineError::SinkUnavailable`] when the store
    /// lock is poisoned.
    pub fn is_empty(&self) -> Result<bool, MeteringPipelineError> {
        Ok(self.locked()?.is_empty())
    }

    fn locked(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, BTreeMap<DedupKey, UsageRecord>>, MeteringPipelineError>
    {
        self.records
            .lock()
            .map_err(|_| MeteringPipelineError::SinkUnavailable {
                detail: "reference sink lock poisoned".to_owned(),
            })
    }
}

impl Default for InMemorySink {
    fn default() -> Self {
        Self::new(LatenessPolicy::default())
    }
}

impl MeteringSink for InMemorySink {
    fn lateness_policy(&self) -> LatenessPolicy {
        self.policy
    }

    fn ingest(
        &self,
        record: UsageRecord,
        arrived_at_epoch_seconds: u64,
    ) -> Result<IngestOutcome, MeteringPipelineError> {
        self.policy
            .admit(record.usage_hour, arrived_at_epoch_seconds)
            .map_err(MeteringPipelineError::Rejected)?;
        let key = record.dedup_key();
        let mut records = self.locked()?;
        match records.get(&key) {
            None => {
                records.insert(key, record);
                Ok(IngestOutcome::Recorded)
            }
            Some(existing)
                if existing.consumed_quantity_microunits == record.consumed_quantity_microunits =>
            {
                Ok(IngestOutcome::Duplicate)
            }
            Some(existing) => Err(MeteringPipelineError::QuantityConflict {
                key,
                recorded_microunits: existing.consumed_quantity_microunits,
                replayed_microunits: record.consumed_quantity_microunits,
            }),
        }
    }

    fn lookup(&self, key: &DedupKey) -> Result<Option<UsageRecord>, MeteringPipelineError> {
        Ok(self.locked()?.get(key).cloned())
    }
}
