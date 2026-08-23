//! The generic conformance checks every metering sink must pass — the
//! D-14 analogue of the G001 resource-provider harness. Each check is a
//! pure generic fn over a [`SinkFixture`]; it builds a FRESH sink,
//! drives it through the contract scenario, and returns the first
//! divergence as a typed [`ConformanceViolation`]. The durable G03-port
//! sink runs the SAME checks against a real database in its integration
//! rung (AMENDMENT 7 test ladder).

use std::fmt;

use crate::{
    BatchUsageRecord, CellId, ConsumedUnit, Dimension, IngestOutcome, MeteringPipelineError,
    MeteringSink, ResourceId, TenantId, UsageHour, UsageRecord, UsageRejection,
};

/// A single conformance divergence: which check failed and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceViolation {
    pub check: &'static str, // data_class: INTERNAL_ONLY
    pub detail: String,      // data_class: INTERNAL_ONLY
}

impl fmt::Display for ConformanceViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.check, self.detail)
    }
}

impl std::error::Error for ConformanceViolation {}

fn violation(check: &'static str, detail: impl Into<String>) -> ConformanceViolation {
    ConformanceViolation {
        check,
        detail: detail.into(),
    }
}

/// What an implementation supplies to run the harness: a fresh sink per
/// check.
pub trait SinkFixture {
    /// The sink under test.
    type Sink: MeteringSink;

    /// A FRESH, empty sink (checks never share state).
    fn fresh_sink(&self) -> Self::Sink;
}

fn record(
    check: &'static str,
    ordinal: u32,
    usage_hour: UsageHour,
    quantity_microunits: u64,
) -> Result<UsageRecord, ConformanceViolation> {
    let fail = |e: MeteringPipelineError| violation(check, format!("fixture: {e}"));
    Ok(UsageRecord {
        tenant: TenantId::parse(&format!("ten_{ordinal:04}")).map_err(fail)?,
        cell: CellId::parse("cell-kr-1").map_err(fail)?,
        resource: ResourceId::parse("meter").map_err(fail)?,
        dimension: Dimension::parse("requests").map_err(fail)?,
        usage_hour,
        consumed_quantity_microunits: quantity_microunits,
        consumed_unit: ConsumedUnit::parse("request").map_err(fail)?,
    })
}

/// Replay idempotency: the same record ingested twice records once and
/// reports the replay as a duplicate; the stored record is unchanged.
pub fn check_replay_is_duplicate<F: SinkFixture>(fixture: &F) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "replay_is_duplicate";
    let sink = fixture.fresh_sink();
    let hour = UsageHour::from_epoch_seconds(7200);
    let arrival = hour.start_epoch_seconds() + 60;
    let usage = record(CHECK, 1, hour, 5_000_000)?;
    let first = sink
        .ingest(usage.clone(), arrival)
        .map_err(|e| violation(CHECK, format!("first ingest failed: {e}")))?;
    if first != IngestOutcome::Recorded {
        return Err(violation(CHECK, "first ingest was not Recorded"));
    }
    let replay = sink
        .ingest(usage.clone(), arrival + 30)
        .map_err(|e| violation(CHECK, format!("replay errored: {e}")))?;
    if replay != IngestOutcome::Duplicate {
        return Err(violation(
            CHECK,
            "identical replay was not reported as Duplicate",
        ));
    }
    let stored = sink
        .lookup(&usage.dedup_key())
        .map_err(|e| violation(CHECK, format!("lookup failed: {e}")))?;
    if stored.as_ref() != Some(&usage) {
        return Err(violation(CHECK, "stored record diverged after replay"));
    }
    Ok(())
}

/// Conflict surfacing: a replay under an existing key with a DIFFERENT
/// quantity is an error and the stored record is untouched.
pub fn check_conflicting_replay_is_surfaced<F: SinkFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "conflicting_replay_is_surfaced";
    let sink = fixture.fresh_sink();
    let hour = UsageHour::from_epoch_seconds(7200);
    let arrival = hour.start_epoch_seconds() + 60;
    let usage = record(CHECK, 1, hour, 5_000_000)?;
    sink.ingest(usage.clone(), arrival)
        .map_err(|e| violation(CHECK, format!("first ingest failed: {e}")))?;
    let mut conflicting = usage.clone();
    conflicting.consumed_quantity_microunits = 9_000_000;
    match sink.ingest(conflicting, arrival + 30) {
        Err(MeteringPipelineError::QuantityConflict { .. }) => {}
        Err(other) => {
            return Err(violation(
                CHECK,
                format!("conflict raised the wrong error: {other}"),
            ));
        }
        Ok(outcome) => {
            return Err(violation(
                CHECK,
                format!("conflicting replay was accepted as {outcome:?}"),
            ));
        }
    }
    let stored = sink
        .lookup(&usage.dedup_key())
        .map_err(|e| violation(CHECK, format!("lookup failed: {e}")))?;
    if stored.as_ref() != Some(&usage) {
        return Err(violation(
            CHECK,
            "stored record was mutated by a conflicting replay",
        ));
    }
    Ok(())
}

/// Key isolation: records differing in any key component coexist.
pub fn check_distinct_keys_are_isolated<F: SinkFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "distinct_keys_are_isolated";
    let sink = fixture.fresh_sink();
    let hour = UsageHour::from_epoch_seconds(7200);
    let arrival = hour.start_epoch_seconds() + 60;
    let base = record(CHECK, 1, hour, 1_000_000)?;
    let mut other_tenant = base.clone();
    other_tenant.tenant =
        TenantId::parse("ten_other").map_err(|e| violation(CHECK, format!("fixture: {e}")))?;
    let mut other_dimension = base.clone();
    other_dimension.dimension = Dimension::parse("storage-gb-seconds")
        .map_err(|e| violation(CHECK, format!("fixture: {e}")))?;
    let mut other_hour = base.clone();
    // The PREVIOUS hour: a distinct bucket that is still admissible at
    // `arrival` under the sink's lateness window (an hour bucket equal to
    // the base's would collide; a future one would be rejected).
    other_hour.usage_hour = UsageHour::from_epoch_seconds(
        hour.start_epoch_seconds()
            .saturating_sub(crate::SECONDS_PER_HOUR),
    );
    for usage in [&base, &other_tenant, &other_dimension, &other_hour] {
        let outcome = sink
            .ingest((*usage).clone(), arrival)
            .map_err(|e| violation(CHECK, format!("ingest failed: {e}")))?;
        if outcome != IngestOutcome::Recorded {
            return Err(violation(
                CHECK,
                "a distinct key was misreported as a duplicate",
            ));
        }
    }
    for usage in [&base, &other_tenant, &other_dimension, &other_hour] {
        let stored = sink
            .lookup(&usage.dedup_key())
            .map_err(|e| violation(CHECK, format!("lookup failed: {e}")))?;
        if stored.as_ref() != Some(usage) {
            return Err(violation(CHECK, "a stored record was lost or mutated"));
        }
    }
    Ok(())
}

/// Lateness enforcement: an event past the window is REJECTED with the
/// typed reason and leaves no record; a future-hour event likewise.
pub fn check_lateness_is_rejected_explicitly<F: SinkFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "lateness_is_rejected_explicitly";
    let sink = fixture.fresh_sink();
    let window = sink.lateness_policy().window_seconds;
    let hour = UsageHour::from_epoch_seconds(7200);
    let usage = record(CHECK, 1, hour, 1_000_000)?;
    let too_late = hour.end_epoch_seconds() + window;
    match sink.ingest(usage.clone(), too_late) {
        Err(MeteringPipelineError::Rejected(UsageRejection::LateArrival { .. })) => {}
        Err(other) => {
            return Err(violation(
                CHECK,
                format!("late event raised the wrong error: {other}"),
            ));
        }
        Ok(outcome) => {
            return Err(violation(
                CHECK,
                format!("late event was accepted as {outcome:?}"),
            ));
        }
    }
    match sink.ingest(usage.clone(), hour.start_epoch_seconds().saturating_sub(1)) {
        Err(MeteringPipelineError::Rejected(UsageRejection::FutureUsage { .. })) => {}
        Err(other) => {
            return Err(violation(
                CHECK,
                format!("future event raised the wrong error: {other}"),
            ));
        }
        Ok(outcome) => {
            return Err(violation(
                CHECK,
                format!("future event was accepted as {outcome:?}"),
            ));
        }
    }
    let stored = sink
        .lookup(&usage.dedup_key())
        .map_err(|e| violation(CHECK, format!("lookup failed: {e}")))?;
    if stored.is_some() {
        return Err(violation(CHECK, "a rejected event left a stored record"));
    }
    Ok(())
}

/// Batch ingest: one batch can contain first-write rows, idempotent
/// duplicates, conflicting duplicates, and late rows. Every row must be
/// reported with the same typed semantics as single-row ingest; valid
/// independent rows still store exactly once.
pub fn check_batch_ingest_reports_per_row_outcomes<F: SinkFixture>(
    fixture: &F,
) -> Result<(), ConformanceViolation> {
    const CHECK: &str = "batch_ingest_reports_per_row_outcomes";
    let sink = fixture.fresh_sink();
    let hour = UsageHour::from_epoch_seconds(7200);
    let arrival = hour.start_epoch_seconds() + 60;
    let primary = record(CHECK, 1, hour, 5_000_000)?;
    let mut conflicting = primary.clone();
    conflicting.consumed_quantity_microunits = 9_000_000;
    let mut distinct_dimension = primary.clone();
    distinct_dimension.dimension = Dimension::parse("storage-gb-seconds")
        .map_err(|e| violation(CHECK, format!("fixture: {e}")))?;
    let late_hour = UsageHour::from_epoch_seconds(0);
    let late = record(CHECK, 2, late_hour, 1_000_000)?;
    let late_arrival = late_hour.end_epoch_seconds() + sink.lateness_policy().window_seconds;

    let results = sink.ingest_batch(&[
        BatchUsageRecord {
            record: primary.clone(),
            arrived_at_epoch_seconds: arrival,
        },
        BatchUsageRecord {
            record: primary.clone(),
            arrived_at_epoch_seconds: arrival + 30,
        },
        BatchUsageRecord {
            record: conflicting,
            arrived_at_epoch_seconds: arrival + 60,
        },
        BatchUsageRecord {
            record: distinct_dimension.clone(),
            arrived_at_epoch_seconds: arrival + 90,
        },
        BatchUsageRecord {
            record: late.clone(),
            arrived_at_epoch_seconds: late_arrival,
        },
    ]);

    if results.len() != 5 {
        return Err(violation(
            CHECK,
            format!("batch returned {} results for 5 inputs", results.len()),
        ));
    }
    if results[0].outcome != Ok(IngestOutcome::Recorded) {
        return Err(violation(CHECK, "first row was not Recorded"));
    }
    if results[1].outcome != Ok(IngestOutcome::Duplicate) {
        return Err(violation(CHECK, "idempotent duplicate was not Duplicate"));
    }
    if !matches!(
        &results[2].outcome,
        Err(MeteringPipelineError::QuantityConflict { key, .. }) if key == &primary.dedup_key()
    ) {
        return Err(violation(
            CHECK,
            "conflicting duplicate did not surface QuantityConflict",
        ));
    }
    if results[3].outcome != Ok(IngestOutcome::Recorded) {
        return Err(violation(
            CHECK,
            "distinct post-conflict row was not Recorded",
        ));
    }
    if !matches!(
        results[4].outcome,
        Err(MeteringPipelineError::Rejected(
            UsageRejection::LateArrival { .. }
        ))
    ) {
        return Err(violation(CHECK, "late row did not surface LateArrival"));
    }
    if sink
        .lookup(&primary.dedup_key())
        .map_err(|e| violation(CHECK, format!("lookup failed: {e}")))?
        .as_ref()
        != Some(&primary)
    {
        return Err(violation(CHECK, "primary row was not stored exactly once"));
    }
    if sink
        .lookup(&distinct_dimension.dedup_key())
        .map_err(|e| violation(CHECK, format!("lookup failed: {e}")))?
        .as_ref()
        != Some(&distinct_dimension)
    {
        return Err(violation(CHECK, "distinct row was not stored"));
    }
    if sink
        .lookup(&late.dedup_key())
        .map_err(|e| violation(CHECK, format!("lookup failed: {e}")))?
        .is_some()
    {
        return Err(violation(CHECK, "late row was stored"));
    }
    Ok(())
}

/// One conformance check as run by [`run_all`].
pub type Check<F> = fn(&F) -> Result<(), ConformanceViolation>;

/// Runs every check, collecting all violations.
pub fn run_all<F: SinkFixture>(fixture: &F) -> Vec<ConformanceViolation> {
    let checks: [Check<F>; 5] = [
        check_replay_is_duplicate,
        check_conflicting_replay_is_surfaced,
        check_distinct_keys_are_isolated,
        check_lateness_is_rejected_explicitly,
        check_batch_ingest_reports_per_row_outcomes,
    ];
    checks
        .iter()
        .filter_map(|check| check(fixture).err())
        .collect()
}
