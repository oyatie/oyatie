//! The owner epoch: a claim increments it, every read and settlement names
//! it, so a worker whose lease expired and was re-claimed can neither read
//! nor settle; the epoch survives failure retention; retries follow one
//! jittered, capped schedule.
use mail_api::{
    DeliveryOutcome, DeliveryQueue, DeliveryTarget, MetadataStore, SubmissionQueue, retry,
};
use mail_kernel::{Account, Error};
use mail_sqlite_store::SqliteStore;

const RAW: &[u8] = b"From: alice@example.org\r\nSubject: fenced\r\n\r\nbody\r\n";

fn store() -> (SqliteStore, std::path::PathBuf) {
    let path = std::env::temp_dir().join(format!(
        "mail-fence-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let db = SqliteStore::open(&path).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    (db, path)
}

fn expire(path: &std::path::Path, table: &str) {
    rusqlite::Connection::open(path)
        .unwrap()
        .execute(
            &format!("UPDATE {table} SET lease_until=0,next_attempt=0"),
            [],
        )
        .unwrap();
}

fn cleanup(path: &std::path::Path) {
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(path.with_extension(format!("sqlite{suffix}")));
    }
}

#[test]
fn a_superseded_delivery_epoch_can_neither_read_nor_settle_and_the_epoch_survives_retention() {
    let (db, path) = store();
    let targets = [DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    }];
    db.enqueue("sender@example.net", &targets, RAW).unwrap();
    let first = db.claim(1).unwrap().pop().unwrap();
    assert_eq!(first.epoch, 1);
    // A retryable failure re-schedules on the jittered schedule, fenced.
    let before = mail_api::Clock.now_secs();
    db.finish(&first, Err(Error::Unavailable)).unwrap();
    let next: i64 = rusqlite::Connection::open(&path)
        .unwrap()
        .query_row("SELECT next_attempt FROM delivery_jobs", [], |r| r.get(0))
        .unwrap();
    assert!((60..76).contains(&(next - before)), "{}", next - before);
    assert_eq!(
        db.finish(&first, Ok(None)),
        Err(Error::Conflict),
        "settled once"
    );
    expire(&path, "delivery_jobs");
    let second = db.claim(1).unwrap().pop().unwrap();
    assert_eq!(second.epoch, 2);
    // The expired owner comes back: nothing it does touches the job.
    assert!(matches!(db.queued_message(&first), Err(Error::Conflict)));
    assert_eq!(db.finish(&first, Ok(None)), Err(Error::Conflict));
    assert_eq!(
        db.finish(&first, Err(Error::Unavailable)),
        Err(Error::Conflict)
    );
    assert!(db.queued_message(&second).is_ok());
    // A terminal failure retains the epoch; the retry continues it.
    db.finish(&second, Err(Error::Forbidden)).unwrap();
    db.retry_failed_delivery("a", &second.message).unwrap();
    let third = db.claim(1).unwrap().pop().unwrap();
    assert_eq!(third.epoch, 3);
    assert_eq!(db.finish(&second, Ok(None)), Err(Error::Conflict));
    let message = db.queued_message(&third).unwrap();
    db.deliver_once("a", &third.delivery_id(), &message.raw, message.received_at)
        .unwrap();
    db.finish(&third, Ok(None)).unwrap();
    assert_eq!(db.account("a").unwrap().messages.len(), 1);
    drop(db);
    cleanup(&path);
}

#[test]
fn a_superseded_outbound_epoch_cannot_renew_or_settle_and_a_bounded_attempt_count_ends_a_job() {
    let (db, path) = store();
    db.enqueue_submission("a", "alice@example.org", &["one@remote.org".into()], RAW)
        .unwrap();
    let first = db.claim_outbound(1).unwrap().pop().unwrap();
    assert_eq!(first.epoch, 1);
    db.renew_outbound(&first).unwrap();
    expire(&path, "outbound_jobs");
    let second = db.claim_outbound(1).unwrap().pop().unwrap();
    assert_eq!(second.epoch, 2);
    assert!(matches!(db.outbound_message(&first), Err(Error::Conflict)));
    assert_eq!(db.renew_outbound(&first), Err(Error::Conflict));
    assert_eq!(
        db.finish_outbound(&first, DeliveryOutcome::Delivered),
        Err(Error::Conflict)
    );
    assert_eq!(
        db.finish_outbound(&first, DeliveryOutcome::Temporary(451)),
        Err(Error::Conflict)
    );
    db.finish_outbound(&second, DeliveryOutcome::Temporary(451))
        .unwrap();
    // Outbound: the attempt bound is terminal — a notice, no job left.
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute(
            "UPDATE outbound_jobs SET attempt=?1,next_attempt=0",
            [retry::MAX_ATTEMPTS - 1],
        )
        .unwrap();
    let last = db.claim_outbound(1).unwrap().pop().unwrap();
    assert_eq!(last.attempt, retry::MAX_ATTEMPTS);
    db.finish_outbound(&last, DeliveryOutcome::Temporary(451))
        .unwrap();
    assert!(db.claim_outbound(10).unwrap().is_empty());
    let notice = db.claim(1).unwrap().pop().expect("the queued notice");
    assert_eq!(notice.account, "a");
    db.finish(&notice, Ok(None)).unwrap();
    // Delivery: the attempt bound is terminal even before the queue lifetime.
    let targets = [DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    }];
    db.enqueue("sender@example.net", &targets, RAW).unwrap();
    rusqlite::Connection::open(&path)
        .unwrap()
        .execute(
            "UPDATE delivery_jobs SET attempt=?1",
            [retry::MAX_ATTEMPTS - 1],
        )
        .unwrap();
    let lease = db.claim(1).unwrap().pop().unwrap();
    assert_eq!(lease.attempt, retry::MAX_ATTEMPTS);
    db.finish(&lease, Err(Error::Unavailable)).unwrap();
    assert_eq!(db.failed_deliveries("a", 10).unwrap().len(), 1);
    assert!(db.claim(10).unwrap().is_empty());
    drop(db);
    cleanup(&path);
}
