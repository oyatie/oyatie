use mail_api::{DeliveryQueue, DeliveryTarget, MetadataStore, Precondition};
use mail_kernel::{Account, Error};
use mail_sqlite_store::SqliteStore;

#[test]
fn queue_refuses_unindexable_mail_before_acceptance() {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let refs = (0..1001)
        .map(|n| format!("<{n}@example.org>\r\n "))
        .collect::<String>();
    let raw = format!("Subject: bounded index\r\nReferences: {refs}\r\n\r\nbody");
    let recipients = [DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    }];
    assert_eq!(
        db.enqueue("", &recipients, raw.as_bytes()),
        Err(Error::Invalid)
    );
    assert!(db.claim(1).unwrap().is_empty());
}

#[test]
fn recipient_binding_is_validated_before_deduplication() {
    let db = SqliteStore::open(":memory:").unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let valid = DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    };
    let forged = DeliveryTarget {
        account: "a".into(),
        address: "bob@example.org".into(),
    };
    for targets in [
        vec![valid.clone(), forged.clone()],
        vec![forged, valid.clone()],
    ] {
        assert_eq!(db.enqueue("", &targets, b"body"), Err(Error::Forbidden));
        assert!(db.claim(10).unwrap().is_empty());
    }
    db.enqueue("", &[valid.clone(), valid], b"body").unwrap();
    assert_eq!(db.claim(10).unwrap().len(), 1);
}

#[test]
fn queue_capacity_bounds_bytes_and_empty_jobs_and_recovers_after_completion() {
    let db = SqliteStore::open(":memory:").unwrap();
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account.quota_bytes = 4;
    db.provision(account, &"a".repeat(32)).unwrap();
    let targets = [DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    }];
    db.enqueue("", &targets, b"body").unwrap();
    assert_eq!(db.enqueue("", &targets, b"x"), Err(Error::OverQuota));
    let lease = db.claim(1).unwrap().pop().unwrap();
    db.finish(&lease, Ok(())).unwrap();
    for _ in 0..1000 {
        db.enqueue("", &targets, b"").unwrap();
    }
    assert_eq!(db.enqueue("", &targets, b""), Err(Error::OverQuota));
    let lease = db.claim(1).unwrap().pop().unwrap();
    db.finish(&lease, Ok(())).unwrap();
    db.enqueue("", &targets, b"body").unwrap();
    assert_eq!(db.enqueue("", &targets, b""), Err(Error::OverQuota));
}

#[test]
fn leases_recover_after_crashes_and_mailbox_delivery_is_idempotent() {
    let path = std::env::temp_dir().join(format!("mail-queue-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    for (id, owner) in [("a", "alice"), ("b", "bob")] {
        db.provision(
            Account::new(id, "t", owner, &format!("{owner}@example.org")).unwrap(),
            &id.repeat(32),
        )
        .unwrap();
    }
    let targets = vec![
        DeliveryTarget {
            account: "a".into(),
            address: "alice@example.org".into(),
        },
        DeliveryTarget {
            account: "b".into(),
            address: "bob@example.org".into(),
        },
    ];
    let raw = b"Subject: durable queue\r\n\r\n\0\xff";
    let id = db.enqueue("sender@example.net", &targets, raw).unwrap();
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    let other = SqliteStore::open(&path).unwrap();
    let first = db.claim(1).unwrap().pop().unwrap();
    let second = other.claim(1).unwrap().pop().unwrap();
    assert_ne!(first.account, second.account);
    assert!(db.claim(10).unwrap().is_empty());
    let message = db.queued_message(&first).unwrap();
    assert_eq!(message.sender, "sender@example.net");
    assert_eq!(message.raw, raw);
    assert_eq!(first.message, id);
    db.deliver_once(
        &first.account,
        &first.delivery_id(),
        &message.raw,
        message.received_at,
    )
    .unwrap();
    // Crash after mailbox commit, before queue acknowledgement.
    let sql = rusqlite::Connection::open(&path).unwrap();
    sql.execute(
        "UPDATE delivery_jobs SET lease_until=0,next_attempt=0 WHERE account=?1",
        [&first.account],
    )
    .unwrap();
    let reclaimed = other.claim(1).unwrap().pop().unwrap();
    assert_ne!(reclaimed.token, first.token);
    assert_eq!(reclaimed.delivery_id(), first.delivery_id());
    assert_eq!(db.finish(&first, Ok(())), Err(Error::Conflict));
    other
        .deliver_once(
            &reclaimed.account,
            &reclaimed.delivery_id(),
            &message.raw,
            message.received_at,
        )
        .unwrap();
    assert_eq!(db.account(&first.account).unwrap().messages.len(), 1);
    assert_eq!(
        db.deliver_once(
            &first.account,
            &first.delivery_id(),
            b"different bytes",
            message.received_at
        ),
        Err(Error::Conflict)
    );
    other.finish(&reclaimed, Ok(())).unwrap();
    db.finish(&second, Err(Error::OverQuota)).unwrap();
    assert!(
        db.claim(10).unwrap().is_empty(),
        "retries must wait until due"
    );
    sql.execute("UPDATE delivery_jobs SET next_attempt=0", [])
        .unwrap();
    let retry = db.claim(1).unwrap().pop().unwrap();
    assert_eq!(retry.attempt, 2);
    let message = db.queued_message(&retry).unwrap();
    assert_eq!(message.raw, raw);
    db.deliver_once(
        &retry.account,
        &retry.delivery_id(),
        &message.raw,
        message.received_at,
    )
    .unwrap();
    db.finish(&retry, Ok(())).unwrap();
    assert!(db.claim(10).unwrap().is_empty());
    assert_eq!(
        sql.query_row("SELECT count(*) FROM queued_messages", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    drop((sql, db, other));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn queue_and_delivery_receipts_rollback_with_their_transactions() {
    let path = std::env::temp_dir().join(format!("mail-queue-fault-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let target = DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    };
    let sql = rusqlite::Connection::open(&path).unwrap();
    sql.execute_batch("CREATE TRIGGER reject_job BEFORE INSERT ON delivery_jobs BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(
        db.enqueue("", std::slice::from_ref(&target), b"body")
            .is_err()
    );
    assert_eq!(
        sql.query_row("SELECT count(*) FROM queued_messages", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    sql.execute_batch("DROP TRIGGER reject_job").unwrap();
    db.enqueue("", &[target], b"body").unwrap();
    let job = db.claim(1).unwrap().pop().unwrap();
    let message = db.queued_message(&job).unwrap();
    sql.execute_batch("CREATE TRIGGER reject_receipt BEFORE INSERT ON delivery_receipts BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(
        db.deliver_once(
            &job.account,
            &job.delivery_id(),
            &message.raw,
            message.received_at
        )
        .is_err()
    );
    assert!(db.account("a").unwrap().messages.is_empty());
    sql.execute_batch("DROP TRIGGER reject_receipt").unwrap();
    db.deliver_once(
        &job.account,
        &job.delivery_id(),
        &message.raw,
        message.received_at,
    )
    .unwrap();
    assert_eq!(db.account("a").unwrap().messages.len(), 1);
    drop((sql, db));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn local_queue_refuses_full_mailboxes_before_accepting() {
    let db = SqliteStore::open(":memory:").unwrap();
    let mut account = Account::new("a", "t", "alice", "alice@example.org").unwrap();
    account.quota_bytes = 4;
    db.provision(account, &"a".repeat(32)).unwrap();
    db.deliver(&["alice@example.org".into()], b"body").unwrap();
    let target = DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    };
    assert_eq!(
        db.enqueue("sender@example.net", &[target], b"new"),
        Err(Error::OverQuota)
    );
    assert!(db.claim(1).unwrap().is_empty());
}

#[path = "queue/failures.rs"]
mod failures;
