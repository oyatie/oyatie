use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};
use mail_api::{Event, Events};
use mail_kernel::{Account, Error};
use mail_sqlite::SqliteStore;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
struct Log {
    entries: Vec<SealedEnvelope>,
    reject: bool,
}
impl RecordsLog for Log {
    fn append(&mut self, envelope: ActionEnvelope) -> Result<Receipt, RecordsLogError> {
        if self.reject {
            return Err(RecordsLogError::Storage {
                detail: "injected".into(),
            });
        }
        if let Some(old) = self.entries.iter().find(|v| {
            v.envelope.tenant_id == envelope.tenant_id
                && v.envelope.idempotency_key == envelope.idempotency_key
        }) {
            if old.envelope != envelope {
                return Err(RecordsLogError::IdempotencyConflict {
                    tenant_id: envelope.tenant_id,
                    idempotency_key: envelope.idempotency_key,
                });
            }
            return Ok(Receipt {
                deduplicated: true,
                ..old.receipt.clone()
            });
        }
        let tenant = self
            .entries
            .iter()
            .filter(|v| v.envelope.tenant_id == envelope.tenant_id)
            .collect::<Vec<_>>();
        let receipt = Receipt {
            ordinal: tenant.len() as u64 + 1,
            object_sequence: tenant
                .iter()
                .filter(|v| v.envelope.object_ref == envelope.object_ref)
                .count() as u64
                + 1,
            deduplicated: false,
        };
        self.entries.push(SealedEnvelope {
            envelope,
            receipt: receipt.clone(),
        });
        Ok(receipt)
    }
    fn replay(&self, tenant: &str, from: u64) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        Ok(self
            .entries
            .iter()
            .filter(|v| v.envelope.tenant_id == tenant && v.receipt.ordinal >= from)
            .cloned()
            .collect())
    }
    fn head(&self, tenant: &str) -> Result<u64, RecordsLogError> {
        Ok(self
            .entries
            .iter()
            .filter(|v| v.envelope.tenant_id == tenant)
            .count() as u64)
    }
}

struct Outbox {
    store: SqliteStore,
    reject_ack: AtomicBool,
}
impl Events for Outbox {
    fn pending(&self, consumer: &str, limit: usize) -> Result<Vec<Event>, Error> {
        self.store.pending(consumer, limit)
    }
    fn acknowledge(&self, consumer: &str, sequence: u64) -> Result<(), Error> {
        if self.reject_ack.load(Ordering::SeqCst) {
            return Err(Error::Unavailable);
        }
        self.store.acknowledge(consumer, sequence)
    }
}

#[test]
fn source_namespaces_and_subscription_positions_are_independent() {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "tenant", "owner", "a@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    db.deliver(&["a@example.org".into()], b"Subject: work\r\n\r\nbody")
        .unwrap();
    let mut log = Log::default();
    assert_eq!(
        mail_foundry::publish(&db, &mut log, "mail-main", "foundry", 10).unwrap(),
        1
    );
    assert_eq!(
        mail_foundry::publish(&db, &mut log, "mail-main", "console", 10).unwrap(),
        1
    );
    assert_eq!(
        log.entries.len(),
        1,
        "destination subscriptions share source idempotency"
    );
    assert_eq!(
        mail_foundry::publish(&db, &mut log, "mail-secondary", "second-foundry", 10).unwrap(),
        1
    );
    assert_eq!(
        log.entries.len(),
        2,
        "independent mail sources cannot collide"
    );
    assert_ne!(
        log.entries[0].envelope.object_ref,
        log.entries[1].envelope.object_ref
    );
    assert_eq!(
        mail_foundry::publish(&db, &mut log, "bad/source", "new", 10),
        Err(Error::Invalid)
    );
    assert_eq!(db.pending("new", 10).unwrap().len(), 1);
}

struct Untimed;
impl Events for Untimed {
    fn pending(&self, _: &str, _: usize) -> Result<Vec<Event>, Error> {
        Ok(vec![Event {
            sequence: 1,
            tenant: "t".into(),
            account: "a".into(),
            revision: 1,
            observed_at_ms: 0,
        }])
    }
    fn acknowledge(&self, _: &str, _: u64) -> Result<(), Error> {
        panic!("unverifiable event must never be acknowledged")
    }
}

#[test]
fn legacy_events_do_not_invent_observation_timestamps() {
    let mut log = Log::default();
    assert_eq!(
        mail_foundry::publish(&Untimed, &mut log, "mail", "foundry", 1),
        Err(Error::Unavailable)
    );
    assert!(log.entries.is_empty());
}

#[test]
fn foundry_exports_preserve_tenant_scope_and_replay_identically_after_lost_ack() {
    let outbox = Outbox {
        store: SqliteStore::open(":memory:").unwrap(),
        reject_ack: AtomicBool::new(true),
    };
    for (id, tenant) in [("a", "tenant-a"), ("b", "tenant-b")] {
        outbox
            .store
            .provision(
                Account::new(id, tenant, id, &format!("{id}@example.org")).unwrap(),
                &id.repeat(32),
            )
            .unwrap();
        outbox
            .store
            .deliver(
                &[format!("{id}@example.org")],
                b"Subject: confidential\r\n\r\nsecret body",
            )
            .unwrap();
    }
    let mut log = Log::default();
    assert_eq!(
        mail_foundry::publish(&outbox, &mut log, "mail-main", "foundry", 10),
        Err(Error::Unavailable)
    );
    assert_eq!(log.entries.len(), 1);
    let first = log.entries[0].clone();
    outbox.reject_ack.store(false, Ordering::SeqCst);
    assert_eq!(
        mail_foundry::publish(&outbox, &mut log, "mail-main", "foundry", 10).unwrap(),
        2
    );
    assert_eq!(log.entries.len(), 2);
    assert_eq!(log.entries[0], first);
    assert!(outbox.pending("foundry", 10).unwrap().is_empty());
    let mut accounts = BTreeMap::new();
    for entry in &log.entries {
        assert_eq!(entry.envelope.action_type, "mail.account.changed");
        assert!(entry.envelope.observed_at_epoch_ms > 0);
        let payload: serde_json::Value = serde_json::from_slice(&entry.envelope.payload).unwrap();
        accounts.insert(
            entry.envelope.tenant_id.clone(),
            payload["accountId"].as_str().unwrap().to_owned(),
        );
        assert!(!String::from_utf8_lossy(&entry.envelope.payload).contains("secret"));
    }
    assert_eq!(accounts["tenant-a"], "a");
    assert_eq!(accounts["tenant-b"], "b");
    outbox
        .store
        .deliver(&["a@example.org".into()], b"new message")
        .unwrap();
    log.reject = true;
    assert_eq!(
        mail_foundry::publish(&outbox, &mut log, "mail-main", "foundry", 10),
        Err(Error::Unavailable)
    );
    assert_eq!(outbox.pending("foundry", 10).unwrap().len(), 1);
}
