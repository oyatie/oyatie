//! Outbox-pattern kernel — per-µservice trait surface for ADR-0153.
//!
//! # ADR-0153 (Tier-A hyperscaler pattern)
//!
//! Every µservice with event-emission requirements appends an
//! outbox row IN THE SAME transaction as the aggregate mutation. A
//! publisher worker drains the outbox FIFO and stamps `published_at`.
//! Direct `event_bus.publish(...)` outside the outbox is FORBIDDEN.
//!
//! # Naming justification
//!
//! `oya-shared-outbox-pattern-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:outbox-pattern>-<layer:kernel>`.
//!
//! # References
//!
//! - docs/standards/outbox-pattern-canonical.md
//! - ADR-0153-outbox-pattern.md

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::fmt;

/// ULID-shaped outbox row identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct OutboxId(pub String);

/// ULID-shaped aggregate identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AggregateId(pub String);

/// Canonical headers attached to every outbox row (request-id, trace,
/// idempotency-key per ADR-0149).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OutboxHeaders {
    pub idempotency_key: Option<String>,
    pub traceparent: Option<String>,
    pub request_id: Option<String>,
}

/// One outbox row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboxRow {
    pub outbox_id: OutboxId,
    pub aggregate_id: AggregateId,
    pub aggregate_kind: String,
    pub event_kind: String,
    pub event_version: String, // ADR-0154
    pub payload: Vec<u8>,
    pub headers: OutboxHeaders,
    pub occurred_at_unix_ms: i64,
    pub published_at_unix_ms: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutboxError {
    EmptyField(&'static str),
    DuplicateOutboxId(OutboxId),
    SkeletonNotYetImplemented(&'static str),
}

impl fmt::Display for OutboxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutboxError::EmptyField(name) => write!(
                f,
                "oya-shared-outbox-pattern-kernel: required field {name:?} is empty"
            ),
            OutboxError::DuplicateOutboxId(id) => write!(
                f,
                "oya-shared-outbox-pattern-kernel: duplicate outbox_id {id:?}"
            ),
            OutboxError::SkeletonNotYetImplemented(method) => write!(
                f,
                "oya-shared-outbox-pattern-kernel: {method} is skeleton-only \
                 (tracked under registry/placeholder-debt/adr-follow-ups.yaml#adr-0153-outbox-impl)"
            ),
        }
    }
}

impl std::error::Error for OutboxError {}

/// The trait every µservice integrates for the outbox write/publish
/// path. `TxContext` is the per-µservice tx handle (e.g. sqlx pool
/// tx, redis pipeline) so the append happens IN THE SAME tx as the
/// aggregate mutation.
pub trait OutboxStore: Send + Sync {
    type TxContext;

    /// Append a row inside an open transaction.
    ///
    /// # Errors
    /// - `EmptyField` when any required field is empty.
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn append(&self, tx: &mut Self::TxContext, row: OutboxRow) -> Result<(), OutboxError>;

    /// Fetch the next unpublished rows in `occurred_at` order.
    ///
    /// # Errors
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn next_unpublished(&self, batch_size: usize) -> Result<Vec<OutboxRow>, OutboxError>;

    /// Mark rows published.
    ///
    /// # Errors
    /// - `SkeletonNotYetImplemented` for the skeleton impl.
    fn mark_published(&self, ids: &[OutboxId]) -> Result<(), OutboxError>;
}

/// In-memory reference implementation for per-µservice integration
/// tests. Production stores use a sqlx-backed adapter.
#[derive(Default)]
pub struct InMemoryOutboxStore {
    inner: std::sync::Mutex<Vec<OutboxRow>>,
}

/// Trivial transaction context for the in-memory store. Production
/// adapters use a sqlx::Transaction or similar.
#[derive(Default)]
pub struct InMemoryTx;

// Mutex lock panics on thread poisoning — equivalent to expect_used
// in test infrastructure. ADR-0083 §Tier-3 permits this pattern in
// reference implementations.
#[allow(clippy::expect_used)]
impl OutboxStore for InMemoryOutboxStore {
    type TxContext = InMemoryTx;

    fn append(&self, _tx: &mut Self::TxContext, row: OutboxRow) -> Result<(), OutboxError> {
        if row.outbox_id.0.is_empty() {
            return Err(OutboxError::EmptyField("outbox_id"));
        }
        if row.aggregate_id.0.is_empty() {
            return Err(OutboxError::EmptyField("aggregate_id"));
        }
        if row.event_kind.is_empty() {
            return Err(OutboxError::EmptyField("event_kind"));
        }
        let mut inner = self.inner.lock().expect("mutex poisoned");
        if inner.iter().any(|r| r.outbox_id == row.outbox_id) {
            return Err(OutboxError::DuplicateOutboxId(row.outbox_id));
        }
        inner.push(row);
        Ok(())
    }

    fn next_unpublished(&self, batch_size: usize) -> Result<Vec<OutboxRow>, OutboxError> {
        let inner = self.inner.lock().expect("mutex poisoned");
        let mut out: Vec<OutboxRow> = inner
            .iter()
            .filter(|r| r.published_at_unix_ms.is_none())
            .take(batch_size)
            .cloned()
            .collect();
        out.sort_by_key(|r| r.occurred_at_unix_ms);
        Ok(out)
    }

    fn mark_published(&self, ids: &[OutboxId]) -> Result<(), OutboxError> {
        let mut inner = self.inner.lock().expect("mutex poisoned");
        for row in inner.iter_mut() {
            if ids.contains(&row.outbox_id) {
                row.published_at_unix_ms = Some(1);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str, agg: &str) -> OutboxRow {
        OutboxRow {
            outbox_id: OutboxId(id.into()),
            aggregate_id: AggregateId(agg.into()),
            aggregate_kind: "Channel".into(),
            event_kind: "oya.messenger.channel.created".into(),
            event_version: "1.0.0".into(),
            payload: br#"{"ok":true}"#.to_vec(),
            headers: OutboxHeaders {
                idempotency_key: Some("idem-1".into()),
                traceparent: Some("00-...-...-01".into()),
                request_id: Some("req-1".into()),
            },
            occurred_at_unix_ms: 1_700_000_000_000,
            published_at_unix_ms: None,
        }
    }

    fn make_store_and_tx() -> (InMemoryOutboxStore, InMemoryTx) {
        (InMemoryOutboxStore::default(), InMemoryTx)
    }

    #[test]
    fn append_validates_required_fields() {
        let (store, mut tx) = make_store_and_tx();
        let mut empty = row("", "agg-1");
        empty.outbox_id = OutboxId(String::new());
        assert_eq!(
            store.append(&mut tx, empty),
            Err(OutboxError::EmptyField("outbox_id"))
        );
    }

    #[test]
    fn append_rejects_duplicate_outbox_id() {
        let (store, mut tx) = make_store_and_tx();
        store
            .append(&mut tx, row("01HMZ1", "agg-1"))
            .expect("first");
        let err = store
            .append(&mut tx, row("01HMZ1", "agg-1"))
            .expect_err("dup");
        assert!(matches!(err, OutboxError::DuplicateOutboxId(_)));
    }

    #[test]
    fn next_unpublished_filters_published() {
        let (store, mut tx) = make_store_and_tx();
        store.append(&mut tx, row("01HMZ1", "agg-1")).expect("ok");
        store.append(&mut tx, row("01HMZ2", "agg-2")).expect("ok");
        store
            .mark_published(&[OutboxId("01HMZ1".into())])
            .expect("publish");
        let pending = store.next_unpublished(10).expect("pending");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].outbox_id, OutboxId("01HMZ2".into()));
    }

    #[test]
    fn next_unpublished_orders_by_occurred_at() {
        let (store, mut tx) = make_store_and_tx();
        let mut a = row("01HMZ1", "agg-1");
        a.occurred_at_unix_ms = 200;
        let mut b = row("01HMZ2", "agg-2");
        b.occurred_at_unix_ms = 100;
        store.append(&mut tx, a).expect("ok");
        store.append(&mut tx, b).expect("ok");
        let pending = store.next_unpublished(10).expect("pending");
        assert_eq!(pending[0].outbox_id, OutboxId("01HMZ2".into()));
        assert_eq!(pending[1].outbox_id, OutboxId("01HMZ1".into()));
    }

    #[test]
    fn error_display_carries_follow_up_pointer() {
        let err = OutboxError::SkeletonNotYetImplemented("append");
        let msg = format!("{err}");
        assert!(msg.contains("adr-0153-outbox-impl"));
    }
}
