//! Proves the sink conformance harness with the in-memory reference
//! sink (GREEN) and with a deliberately broken sink (RED) — the
//! masterplan no-false-green rule.

use oya_metering_pipeline_kernel::conformance::{self, SinkFixture};
use oya_metering_pipeline_kernel::reference::InMemorySink;
use oya_metering_pipeline_kernel::{
    DedupKey, IngestOutcome, LatenessPolicy, MeteringPipelineError, MeteringSink, UsageRecord,
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
