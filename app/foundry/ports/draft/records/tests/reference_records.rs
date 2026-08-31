//! Reference in-memory records log, driven through the port's conformance
//! suite. The reference is volatile on purpose: it proves the contract, and
//! honestly reports that it cannot prove durability.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use foundry_records_draft::conformance::{
    RecordsFixture, check_append_assigns_dense_per_tenant_ordinals,
    check_conflicting_idempotency_key_reuse_is_refused, check_durability_across_reopen,
    check_head_tracks_the_last_ordinal, check_idempotent_replay_returns_the_original_receipt,
    check_object_sequences_are_dense_per_object, check_replay_is_tenant_isolated,
    check_replay_returns_envelopes_in_order,
};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};

#[derive(Default)]
struct InMemoryLog {
    per_tenant: BTreeMap<String, Vec<SealedEnvelope>>,
    receipts_by_key: BTreeMap<(String, String), (Receipt, ActionEnvelope)>,
}

impl RecordsLog for InMemoryLog {
    fn append(&mut self, envelope: ActionEnvelope) -> Result<Receipt, RecordsLogError> {
        let key = (envelope.tenant_id.clone(), envelope.idempotency_key.clone());
        if let Some((receipt, original)) = self.receipts_by_key.get(&key) {
            if *original != envelope {
                return Err(RecordsLogError::IdempotencyConflict {
                    tenant_id: envelope.tenant_id,
                    idempotency_key: envelope.idempotency_key,
                });
            }
            return Ok(Receipt {
                deduplicated: true,
                ..receipt.clone()
            });
        }
        let stream = self
            .per_tenant
            .entry(envelope.tenant_id.clone())
            .or_default();
        let ordinal = stream.len() as u64 + 1;
        let object_sequence = stream
            .iter()
            .filter(|sealed| sealed.envelope.object_ref == envelope.object_ref)
            .count() as u64
            + 1;
        let receipt = Receipt {
            ordinal,
            object_sequence,
            deduplicated: false,
        };
        stream.push(SealedEnvelope {
            envelope: envelope.clone(),
            receipt: receipt.clone(),
        });
        self.receipts_by_key
            .insert(key, (receipt.clone(), envelope));
        Ok(receipt)
    }

    fn replay(
        &self,
        tenant_id: &str,
        from_ordinal: u64,
    ) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        Ok(self
            .per_tenant
            .get(tenant_id)
            .map(|stream| {
                stream
                    .iter()
                    .filter(|sealed| sealed.receipt.ordinal >= from_ordinal)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    fn head(&self, tenant_id: &str) -> Result<u64, RecordsLogError> {
        Ok(self
            .per_tenant
            .get(tenant_id)
            .map(|stream| stream.len() as u64)
            .unwrap_or(0))
    }
}

#[derive(Default)]
struct InMemoryFixture {
    log: InMemoryLog,
}

impl RecordsFixture for InMemoryFixture {
    type Log = InMemoryLog;

    fn log(&mut self) -> &mut Self::Log {
        &mut self.log
    }

    fn reopen(&mut self) -> bool {
        // Volatile by design: reopening would lose the log, and pretending
        // otherwise is exactly the lie the durability check exists to catch.
        false
    }
}

type Check = fn(&mut InMemoryFixture) -> Result<(), String>;

#[test]
fn reference_log_satisfies_every_conformance_check() {
    let checks: [(&str, Check); 8] = [
        (
            "dense ordinals",
            check_append_assigns_dense_per_tenant_ordinals,
        ),
        (
            "object sequences",
            check_object_sequences_are_dense_per_object,
        ),
        (
            "idempotent replay",
            check_idempotent_replay_returns_the_original_receipt,
        ),
        (
            "conflict refusal",
            check_conflicting_idempotency_key_reuse_is_refused,
        ),
        ("ordered replay", check_replay_returns_envelopes_in_order),
        ("tenant isolation", check_replay_is_tenant_isolated),
        ("head tracking", check_head_tracks_the_last_ordinal),
        ("durability", check_durability_across_reopen),
    ];
    for (name, check) in checks {
        let mut fixture = InMemoryFixture::default();
        check(&mut fixture).unwrap_or_else(|violation| panic!("{name}: {violation}"));
    }
}

#[test]
fn envelope_construction_is_fail_closed() {
    assert!(ActionEnvelope::new("", "finance.invoice:01", "create", "k1", 1, vec![], 1).is_err());
    assert!(ActionEnvelope::new("ten_a", " padded", "create", "k1", 1, vec![], 1).is_err());
    assert!(ActionEnvelope::new("ten_a", "finance.invoice:01", "", "k1", 1, vec![], 1).is_err());
    assert!(
        ActionEnvelope::new("ten_a", "finance.invoice:01", "create", "", 1, vec![], 1).is_err()
    );
    assert!(
        ActionEnvelope::new("ten_a", "finance.invoice:01", "create", "k1", 0, vec![], 1).is_err()
    );
    let envelope = ActionEnvelope::new(
        "ten_a",
        "finance.invoice:01",
        "create",
        "k1",
        1,
        b"{}".to_vec(),
        7,
    )
    .unwrap();
    assert_eq!(envelope.schema_revision, 1);
    assert_eq!(envelope.observed_at_epoch_ms, 7);
}
