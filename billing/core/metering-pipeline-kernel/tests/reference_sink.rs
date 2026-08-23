//! Proves the sink conformance harness with the in-memory reference
//! sink (GREEN) and with a deliberately broken sink (RED) — the
//! masterplan no-false-green rule.

use billing_metering_pipeline_kernel::conformance::{self, SinkFixture};
use billing_metering_pipeline_kernel::reference::InMemorySink;
use billing_metering_pipeline_kernel::{
    BatchUsageRecord, CellId, ConsumedUnit, DedupKey, Dimension, IngestOutcome, LatenessPolicy,
    MeteringPipelineError, MeteringSink, ResourceId, TenantId, UsageHour, UsageRecord,
    UsageRejection,
};

struct ReferenceFixture;

impl SinkFixture for ReferenceFixture {
    type Sink = InMemorySink;

    fn fresh_sink(&self) -> Self::Sink {
        InMemorySink::new(LatenessPolicy::default())
    }
}

#[test]
fn reference_sink_is_fully_conformant() {
    let violations = conformance::run_all(&ReferenceFixture);
    assert!(
        violations.is_empty(),
        "reference sink diverged: {violations:?}"
    );
}

fn usage_record(
    tenant: &str,
    resource: &str,
    dimension: &str,
    hour: UsageHour,
    quantity_microunits: u64,
) -> UsageRecord {
    UsageRecord {
        tenant: TenantId::parse(tenant).expect("fixture tenant is valid"),
        cell: CellId::parse("cell-kr-1").expect("fixture cell is valid"),
        resource: ResourceId::parse(resource).expect("fixture resource is valid"),
        dimension: Dimension::parse(dimension).expect("fixture dimension is valid"),
        usage_hour: hour,
        consumed_quantity_microunits: quantity_microunits,
        consumed_unit: ConsumedUnit::parse("request").expect("fixture unit is valid"),
    }
}

#[test]
fn batch_ingest_fixture_records_duplicates_and_rejects_late_rows() {
    let sink = InMemorySink::new(LatenessPolicy::default());
    let hour = UsageHour::from_epoch_seconds(7200);
    let arrived = hour.start_epoch_seconds() + 60;
    let primary = usage_record("ten_batch", "meter", "requests", hour, 5_000_000);
    let mut conflicting_replay = primary.clone();
    conflicting_replay.consumed_quantity_microunits = 7_000_000;
    let distinct_dimension =
        usage_record("ten_batch", "meter", "storage-gb-seconds", hour, 2_500_000);
    let late_hour = UsageHour::from_epoch_seconds(0);
    let late = usage_record("ten_batch", "meter", "late-requests", late_hour, 1_000_000);
    let late_arrival = late_hour.end_epoch_seconds() + sink.lateness_policy().window_seconds;

    let results = sink.ingest_batch(&[
        BatchUsageRecord {
            record: primary.clone(),
            arrived_at_epoch_seconds: arrived,
        },
        BatchUsageRecord {
            record: primary.clone(),
            arrived_at_epoch_seconds: arrived + 30,
        },
        BatchUsageRecord {
            record: conflicting_replay,
            arrived_at_epoch_seconds: arrived + 60,
        },
        BatchUsageRecord {
            record: distinct_dimension.clone(),
            arrived_at_epoch_seconds: arrived + 90,
        },
        BatchUsageRecord {
            record: late.clone(),
            arrived_at_epoch_seconds: late_arrival,
        },
    ]);

    assert_eq!(results.len(), 5);
    assert_eq!(results[0].key, primary.dedup_key());
    assert_eq!(results[0].outcome, Ok(IngestOutcome::Recorded));
    assert_eq!(results[1].key, primary.dedup_key());
    assert_eq!(results[1].outcome, Ok(IngestOutcome::Duplicate));
    assert!(matches!(
        &results[2].outcome,
        Err(MeteringPipelineError::QuantityConflict { key, .. }) if key == &primary.dedup_key()
    ));
    assert_eq!(results[3].key, distinct_dimension.dedup_key());
    assert_eq!(results[3].outcome, Ok(IngestOutcome::Recorded));
    assert!(matches!(
        results[4].outcome,
        Err(MeteringPipelineError::Rejected(
            UsageRejection::LateArrival { .. }
        ))
    ));

    assert_eq!(sink.lookup(&primary.dedup_key()).unwrap(), Some(primary));
    assert_eq!(
        sink.lookup(&distinct_dimension.dedup_key()).unwrap(),
        Some(distinct_dimension)
    );
    assert_eq!(sink.lookup(&late.dedup_key()).unwrap(), None);
    assert_eq!(sink.len().unwrap(), 2);
}

/// A sink that last-write-wins on conflicting replays and silently drops
/// late events as fake duplicates — the harness must catch both.
struct LastWriteWinsSink {
    inner: InMemorySink,
}

impl MeteringSink for LastWriteWinsSink {
    fn lateness_policy(&self) -> LatenessPolicy {
        self.inner.lateness_policy()
    }

    fn ingest(
        &self,
        record: UsageRecord,
        arrived_at_epoch_seconds: u64,
    ) -> Result<IngestOutcome, MeteringPipelineError> {
        match self.inner.ingest(record, arrived_at_epoch_seconds) {
            // Broken: conflicting and late ingests are swallowed as
            // "duplicates" instead of being surfaced.
            Err(MeteringPipelineError::QuantityConflict { .. })
            | Err(MeteringPipelineError::Rejected(_)) => Ok(IngestOutcome::Duplicate),
            other => other,
        }
    }

    fn lookup(&self, key: &DedupKey) -> Result<Option<UsageRecord>, MeteringPipelineError> {
        self.inner.lookup(key)
    }
}

struct LastWriteWinsFixture;

impl SinkFixture for LastWriteWinsFixture {
    type Sink = LastWriteWinsSink;

    fn fresh_sink(&self) -> Self::Sink {
        LastWriteWinsSink {
            inner: InMemorySink::new(LatenessPolicy::default()),
        }
    }
}

#[test]
fn harness_catches_conflict_swallowing_and_silent_lateness() {
    let violations = conformance::run_all(&LastWriteWinsFixture);
    let failed: Vec<&str> = violations.iter().map(|v| v.check).collect();
    assert!(
        failed.contains(&"conflicting_replay_is_surfaced"),
        "harness missed the swallowed-conflict violation: {failed:?}"
    );
    assert!(
        failed.contains(&"lateness_is_rejected_explicitly"),
        "harness missed the silent-lateness violation: {failed:?}"
    );
}
