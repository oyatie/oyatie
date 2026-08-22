//! Transactional outbox + CDC change-stream shapes on the owned `data`
//! SQL port.
//!
//! Story G003 sub-slice 3 (ADR-0536 D-10 persistence, D-13 messaging):
//! the outbox row commits in the SAME port [`WriteBatch`] transaction as the
//! business rows it announces, so an at-least-once relay downstream yields
//! effectively-once delivery (transactional-outbox pattern; precedent:
//! the D-13 messaging doctrine, Debezium outbox practice, and the existing
//! messenger-local `shared-transactional-outbox-kernel` seam this
//! generalizes). The [`ChangeStreamSource`] port models the W5 engine's
//! native changefeeds with HLC checkpoints (CockroachDB changefeed /
//! Spanner change-stream shape); the transitional implementation is outbox
//! polling behind the same trait, so consumers never see the engine swap.
//!
//! Pure kernel: NO broker IO, NO polling loops, NO payload serialization —
//! adapters own those.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use data_sql_kernel::{DataSqlError, SqlValue, Statement, WriteBatch};

/// Default outbox insert, parameterized only — generic across services
/// (the messenger-local SQL stays in its transitional crate). `ON CONFLICT
/// DO NOTHING` on the idempotency key makes producer retries safe.
pub const INSERT_OUTBOX_EVENT_SQL: &str = "INSERT INTO data_outbox.outbox_events (tenant_id, event_id, event_kind, aggregate_id, schema_version, idempotency_key, payload) VALUES ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT (tenant_id, idempotency_key) DO NOTHING";

/// A generic outbox event: the unit the relay publishes after commit.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutboxEvent {
    pub tenant_id: String,      // data_class: TENANT_SCOPED
    pub event_id: String,       // data_class: INTERNAL_ONLY
    pub event_kind: String,     // data_class: INTERNAL_ONLY
    pub aggregate_id: String,   // data_class: TENANT_SCOPED
    pub schema_version: String, // data_class: INTERNAL_ONLY
    /// Producer-supplied idempotency key (client-UUID discipline, AIP-155):
    /// the conflict target that makes write retries effectively-once.
    pub idempotency_key: String, // data_class: INTERNAL_ONLY
    /// Opaque serialized payload; the kernel never interprets it.
    pub payload: Vec<u8>, // data_class: TENANT_SCOPED
}

impl OutboxEvent {
    pub fn validate(&self) -> Result<(), DataSqlError> {
        for (field, value) in [
            ("outbox.tenant_id", &self.tenant_id),
            ("outbox.event_id", &self.event_id),
            ("outbox.event_kind", &self.event_kind),
            ("outbox.aggregate_id", &self.aggregate_id),
            ("outbox.schema_version", &self.schema_version),
            ("outbox.idempotency_key", &self.idempotency_key),
        ] {
            if value.trim().is_empty() {
                return Err(DataSqlError::MissingField { field });
            }
        }
        Ok(())
    }

    /// The parameterized insert statement for this event.
    pub fn insert_statement(&self) -> Result<Statement, DataSqlError> {
        self.validate()?;
        Statement::new(
            "insert_outbox_event",
            INSERT_OUTBOX_EVENT_SQL,
            vec![
                SqlValue::Text(self.tenant_id.clone()),
                SqlValue::Text(self.event_id.clone()),
                SqlValue::Text(self.event_kind.clone()),
                SqlValue::Text(self.aggregate_id.clone()),
                SqlValue::Text(self.schema_version.clone()),
                SqlValue::Text(self.idempotency_key.clone()),
                SqlValue::Bytes(self.payload.clone()),
            ],
        )
    }
}

/// Append the outbox insert to an existing business-row batch so both commit
/// in one transaction — the load-bearing atomicity of the pattern. Returns a
/// NEW batch (the port keeps batches immutable once validated).
pub fn with_outbox_event(
    batch: &WriteBatch,
    event: &OutboxEvent,
) -> Result<WriteBatch, DataSqlError> {
    let mut statements = batch.statements.clone();
    statements.push(event.insert_statement()?);
    WriteBatch::new(statements)
}

/// An opaque, strictly-monotone CDC stream position — a resumable changefeed
/// offset, NOT a clock. The transitional Postgres adapter sources it from the
/// table's global `commit_logical` IDENTITY sequence (a strict total order
/// over committed rows); the W5 engine-native changefeed sources it from its
/// own monotone resolved-timestamp/offset cursor. It is `u64` so the bigint
/// sequence is carried WITHOUT narrowing (a CDC offset must never wrap or
/// saturate the order key), and it is deliberately distinct from
/// `HlcTimestamp`: a global sequence is not a per-wall HLC tie-counter, so
/// using an HLC as the checkpoint key would misrepresent the offset semantics
/// and force a lossy narrowing.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct StreamPosition(pub u64);

impl StreamPosition {
    /// The position before every real change (resume-from-the-beginning).
    #[must_use]
    pub fn zero() -> Self {
        Self(0)
    }
}

/// One observed change, ordered by its monotone CDC [`StreamPosition`]. Shaped
/// for the W5 engine's native changefeed records; the outbox-polling
/// transitional adapter synthesizes the same shape. `commit_wall_nanos` is the
/// informational physical commit instant only — it is NEVER the order or
/// checkpoint key (a non-monotone wall clock would silently skip later rows);
/// `position` is the sole order/checkpoint key.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeRecord {
    pub tenant_id: String,        // data_class: TENANT_SCOPED
    pub event_id: String,         // data_class: INTERNAL_ONLY
    pub event_kind: String,       // data_class: INTERNAL_ONLY
    pub aggregate_id: String,     // data_class: TENANT_SCOPED
    pub position: StreamPosition, // data_class: INTERNAL_ONLY
    /// Informational physical commit instant; NOT the ordering/checkpoint key.
    pub commit_wall_nanos: u64, // data_class: INTERNAL_ONLY
    pub payload: Vec<u8>,         // data_class: TENANT_SCOPED
}

/// A resumable page of changes. `resume_from` is the checkpoint to pass to
/// the next poll — at-least-once semantics: re-polling from the same
/// checkpoint may re-deliver, and per-aggregate ordering is the only
/// ordering guarantee (D-13: per-key ordering only).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeBatch {
    pub records: Vec<ChangeRecord>,  // data_class: TENANT_SCOPED
    pub resume_from: StreamPosition, // data_class: INTERNAL_ONLY
}

impl ChangeBatch {
    /// Surface-all invariants: records strictly increasing by stream position
    /// (a CDC offset is unique + monotone, so equal positions are also a
    /// violation) and the resume checkpoint not behind the last record.
    pub fn validate(&self) -> Result<(), DataSqlError> {
        for window in self.records.windows(2) {
            if window[0].position >= window[1].position {
                return Err(DataSqlError::Adapter(
                    "change records must be strictly increasing by stream position".to_owned(),
                ));
            }
        }
        if let Some(last) = self.records.last()
            && self.resume_from < last.position
        {
            return Err(DataSqlError::Adapter(
                "resume checkpoint must not be behind the last delivered record".to_owned(),
            ));
        }
        Ok(())
    }
}

/// The CDC port (ADR-0536 D-10 change streams). W5: engine-native
/// changefeed. Transitional: outbox polling. Consumers checkpoint with an
/// opaque monotone [`StreamPosition`] and never observe which implementation
/// serves them.
pub trait ChangeStreamSource {
    fn poll_changes(
        &mut self,
        tenant_id: &str,
        checkpoint: StreamPosition,
        limit: usize,
    ) -> Result<ChangeBatch, DataSqlError>;
}

/// In-crate reference implementation for contract tests: serves a fixed,
/// ordered record set with correct checkpoint semantics.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordingChangeStream {
    records: Vec<ChangeRecord>,
}

impl RecordingChangeStream {
    pub fn new(mut records: Vec<ChangeRecord>) -> Self {
        records.sort_by_key(|record| record.position);
        Self { records }
    }
}

impl ChangeStreamSource for RecordingChangeStream {
    fn poll_changes(
        &mut self,
        tenant_id: &str,
        checkpoint: StreamPosition,
        limit: usize,
    ) -> Result<ChangeBatch, DataSqlError> {
        if tenant_id.trim().is_empty() {
            return Err(DataSqlError::MissingField {
                field: "change_stream.tenant_id",
            });
        }
        let records: Vec<ChangeRecord> = self
            .records
            .iter()
            .filter(|record| record.tenant_id == tenant_id && record.position > checkpoint)
            .take(limit)
            .cloned()
            .collect();
        let resume_from = records.last().map_or(checkpoint, |record| record.position);
        let batch = ChangeBatch {
            records,
            resume_from,
        };
        batch.validate()?;
        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event() -> OutboxEvent {
        OutboxEvent {
            tenant_id: "acme".to_owned(),
            event_id: "evt-1".to_owned(),
            event_kind: "tenant.provisioned".to_owned(),
            aggregate_id: "tenants/acme".to_owned(),
            schema_version: "1".to_owned(),
            idempotency_key: "idem-1".to_owned(),
            payload: b"{}".to_vec(),
        }
    }

    fn record(tenant: &str, event_id: &str, position: u64) -> ChangeRecord {
        ChangeRecord {
            tenant_id: tenant.to_owned(),
            event_id: event_id.to_owned(),
            event_kind: "tenant.provisioned".to_owned(),
            aggregate_id: format!("tenants/{tenant}"),
            position: StreamPosition(position),
            // Informational only; deliberately NON-monotone relative to the
            // position to prove ordering never reads the wall clock.
            commit_wall_nanos: u64::MAX - position,
            payload: Vec::new(),
        }
    }

    #[test]
    fn outbox_event_requires_every_field() {
        for blank in [
            "tenant_id",
            "event_id",
            "event_kind",
            "aggregate_id",
            "schema_version",
            "idempotency_key",
        ] {
            let mut e = event();
            match blank {
                "tenant_id" => e.tenant_id = " ".to_owned(),
                "event_id" => e.event_id = String::new(),
                "event_kind" => e.event_kind = String::new(),
                "aggregate_id" => e.aggregate_id = String::new(),
                "schema_version" => e.schema_version = String::new(),
                _ => e.idempotency_key = String::new(),
            }
            assert!(e.validate().is_err(), "{blank} must be required");
        }
        event().validate().unwrap();
    }

    #[test]
    fn outbox_event_round_trips_closed() {
        let e = event();
        let json = serde_json::to_string(&e).unwrap();
        assert_eq!(serde_json::from_str::<OutboxEvent>(&json).unwrap(), e);
        let mut value = serde_json::to_value(&e).unwrap();
        value["surprise"] = serde_json::json!(1);
        assert!(serde_json::from_value::<OutboxEvent>(value).is_err());
    }

    #[test]
    fn insert_statement_is_fully_parameterized() {
        let statement = event().insert_statement().unwrap();
        assert_eq!(statement.params.len(), 7);
        assert!(!statement.sql.contains("acme"), "values never in SQL text");
        assert!(
            statement
                .sql
                .contains("ON CONFLICT (tenant_id, idempotency_key)")
        );
    }

    #[test]
    fn with_outbox_event_appends_atomically_to_the_business_batch() {
        let business = WriteBatch::new(vec![
            Statement::new(
                "insert_tenant",
                "INSERT INTO tenants VALUES ($1)",
                vec![SqlValue::Text("acme".to_owned())],
            )
            .unwrap(),
        ])
        .unwrap();
        let combined = with_outbox_event(&business, &event()).unwrap();
        assert_eq!(
            combined.statement_names(),
            vec!["insert_tenant", "insert_outbox_event"]
        );
        // Original batch is untouched (immutability of validated batches).
        assert_eq!(business.statements.len(), 1);
    }

    #[test]
    fn change_batch_rejects_disorder_and_stale_checkpoints() {
        let disordered = ChangeBatch {
            records: vec![record("acme", "e2", 20), record("acme", "e1", 10)],
            resume_from: StreamPosition(20),
        };
        assert!(disordered.validate().is_err());
        // Equal positions are also a violation: a CDC offset is unique.
        let duplicate_position = ChangeBatch {
            records: vec![record("acme", "e1", 10), record("acme", "e1b", 10)],
            resume_from: StreamPosition(10),
        };
        assert!(duplicate_position.validate().is_err());
        let stale_checkpoint = ChangeBatch {
            records: vec![record("acme", "e1", 10)],
            resume_from: StreamPosition(5),
        };
        assert!(stale_checkpoint.validate().is_err());
    }

    #[test]
    fn change_batch_position_above_u32_max_round_trips() {
        // A global bigint sequence past u32::MAX must carry through the kernel
        // shapes without narrowing or wrap (the MAJOR-1 boundary).
        let big = u64::from(u32::MAX) + 1;
        let records = vec![record("acme", "e-big", big)];
        let batch = ChangeBatch {
            records,
            resume_from: StreamPosition(big),
        };
        batch.validate().unwrap();
        assert_eq!(batch.records[0].position, StreamPosition(big));
        let json = serde_json::to_string(&batch).unwrap();
        assert_eq!(serde_json::from_str::<ChangeBatch>(&json).unwrap(), batch);
    }

    #[test]
    fn reference_stream_filters_by_tenant_and_checkpoint() {
        let mut stream = RecordingChangeStream::new(vec![
            record("acme", "e2", 20),
            record("acme", "e1", 10),
            record("globex", "g1", 15),
        ]);
        let batch = stream.poll_changes("acme", StreamPosition(10), 10).unwrap();
        // Strictly-after semantics: the checkpointed record is not redelivered.
        assert_eq!(batch.records.len(), 1);
        assert_eq!(batch.records[0].event_id, "e2");
        assert_eq!(batch.resume_from, StreamPosition(20));
        // Cross-tenant records are never visible.
        assert!(batch.records.iter().all(|r| r.tenant_id == "acme"));
    }

    #[test]
    fn reference_stream_resume_loop_drains_exactly_once() {
        let mut stream = RecordingChangeStream::new(vec![
            record("acme", "e1", 10),
            record("acme", "e2", 20),
            record("acme", "e3", 30),
        ]);
        let mut checkpoint = StreamPosition::zero();
        let mut seen = Vec::new();
        loop {
            let batch = stream.poll_changes("acme", checkpoint, 1).unwrap();
            if batch.records.is_empty() {
                break;
            }
            seen.extend(batch.records.iter().map(|r| r.event_id.clone()));
            checkpoint = batch.resume_from;
        }
        assert_eq!(seen, vec!["e1", "e2", "e3"]);
    }

    #[test]
    fn empty_poll_keeps_the_caller_checkpoint() {
        let mut stream = RecordingChangeStream::default();
        let checkpoint = StreamPosition(42);
        let batch = stream.poll_changes("acme", checkpoint, 10).unwrap();
        assert!(batch.records.is_empty());
        assert_eq!(batch.resume_from, checkpoint);
    }
}
