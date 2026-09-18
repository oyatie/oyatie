use mail_api::{
    DeliveryOutcome, DeliveryQueue, SubmissionAcceptance, SubmissionFailure, SubmissionQueue,
    SubmissionStore,
};
use mail_kernel::{
    Account, EnvelopeAddress, Error, SubmissionEnvelope, SubmissionFilter as Filter,
    SubmissionQuery, SubmissionRecord, UndoStatus,
};
use mail_sqlite_store::SqliteStore;
use std::collections::BTreeMap;

const RAW: &[u8] =
    b"From: alice@example.org\r\nTo: remote@example.net\r\nSubject: queue\r\n\r\nbody\r\n";
fn request(now: i64) -> SubmissionAcceptance {
    let address = |email: &str| EnvelopeAddress {
        email: email.into(),
        parameters: BTreeMap::new(),
    };
    SubmissionAcceptance {
        record: SubmissionRecord {
            id: String::new(),
            identity_id: "a".into(),
            email_id: "e1".into(),
            thread_id: String::new(),
            envelope: SubmissionEnvelope {
                mail_from: address("alice@example.org"),
                rcpt_to: vec![address("alice@example.org"), address("remote@example.net")],
            },
            send_at: now,
            undo_status: UndoStatus::Pending,
            delivery_status: BTreeMap::new(),
        },
        email_revision: 1,
        raw: RAW.into(),
        allow_remote: true,
    }
}
fn provision(db: &SqliteStore) {
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    db.deliver(&["alice@example.org".into()], RAW).unwrap();
}
fn query(filter: Filter) -> SubmissionQuery {
    SubmissionQuery {
        filter,
        sort: vec![],
        position: 0,
        anchor: None,
        anchor_offset: 0,
        limit: 256,
    }
}

#[test]
fn future_release_does_not_spend_retry_budget_in_either_queue() {
    let path = std::env::temp_dir().join(format!(
        "mail-submission-retry-{}.sqlite",
        std::process::id()
    ));
    let db = SqliteStore::open(&path).unwrap();
    provision(&db);
    let sql = rusqlite::Connection::open(&path).unwrap();
    let now: i64 = sql
        .query_row("SELECT unixepoch()", [], |r| r.get(0))
        .unwrap();
    db.accept_submission("a", 0, request(now + 864000)).unwrap();
    sql.execute_batch("UPDATE queued_messages SET received_at=unixepoch()-864000; UPDATE submitted_messages SET received_at=unixepoch()-864000; UPDATE submission_schedule SET send_at=unixepoch(); UPDATE delivery_jobs SET next_attempt=0; UPDATE outbound_jobs SET next_attempt=0;").unwrap();
    let local = db.claim(1).unwrap().pop().unwrap();
    let remote = db.claim_outbound(1).unwrap().pop().unwrap();
    assert!(
        db.queued_message(&local).unwrap().retry_at
            > db.queued_message(&local).unwrap().received_at
    );
    assert!(
        db.outbound_message(&remote).unwrap().retry_at
            > db.outbound_message(&remote).unwrap().received_at
    );
    db.finish(&local, Err(Error::OverQuota)).unwrap();
    db.finish_outbound(&remote, DeliveryOutcome::Temporary(451))
        .unwrap();
    assert_eq!(
        sql.query_row(
            "SELECT (SELECT count(*) FROM delivery_jobs)+(SELECT count(*) FROM outbound_jobs)",
            [],
            |r| r.get::<_, usize>(0)
        )
        .unwrap(),
        2
    );
    assert_eq!(
        sql.query_row("SELECT count(*) FROM failed_delivery_jobs", [], |r| r
            .get::<_, usize>(0))
            .unwrap(),
        0
    );
    drop((db, sql));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn filtering_is_historical_and_not_means_none_of_the_conditions() {
    let db = SqliteStore::open(":memory:").unwrap();
    provision(&db);
    let first = db.accept_submission("a", 0, request(2000000000)).unwrap();
    let first_id = first.records[0].id.clone();
    let second = db.accept_submission("a", 1, request(2000000100)).unwrap();
    let second_id = second.records[0].id.clone();
    let q = query(Filter::Not(vec![
        Filter::Before(2000000050),
        Filter::EmailIds(vec!["missing".into()]),
    ]));
    assert_eq!(
        db.query_submissions("a", None, &q).unwrap().ids,
        vec![second_id.clone()]
    );
    let q = query(Filter::And(vec![
        Filter::After(2000000000),
        Filter::Before(2000000101),
    ]));
    assert_eq!(
        db.query_submissions("a", None, &q).unwrap().ids,
        vec![second_id.clone()]
    );
    db.cancel_submission("a", 2, &first_id).unwrap();
    let q = query(Filter::UndoStatus(UndoStatus::Pending));
    assert_eq!(
        db.query_submissions("a", None, &q).unwrap().ids,
        vec![second_id]
    );
    assert_eq!(
        db.query_submissions("a", Some(1), &q).unwrap().ids,
        vec![first_id.clone()]
    );
    let mut q = query(Filter::All);
    q.anchor = Some(first_id);
    q.anchor_offset = 1;
    assert_eq!(db.query_submissions("a", None, &q).unwrap().position, 1);
    q.anchor = Some("foreign".into());
    assert!(matches!(
        db.query_submissions("a", None, &q),
        Err(SubmissionFailure::AnchorNotFound)
    ));
    assert!(matches!(
        db.query_submissions("a", Some(4), &q),
        Err(SubmissionFailure::Storage(Error::Conflict))
    ));
    assert!(db.submissions("a", Some(&[])).unwrap().records.is_empty());
    assert!(matches!(
        db.submissions("missing", Some(&[])),
        Err(Error::NotFound)
    ));
}

#[test]
fn submission_lifecycle_rolls_back_queue_and_history_together() {
    let path = std::env::temp_dir().join(format!(
        "mail-submission-rollback-{}.sqlite",
        std::process::id()
    ));
    let db = SqliteStore::open(&path).unwrap();
    provision(&db);
    let sql = rusqlite::Connection::open(&path).unwrap();
    let accepted = db.accept_submission("a", 0, request(2000000000)).unwrap();
    let id = &accepted.records[0].id;
    sql.execute_batch("CREATE TRIGGER reject_history BEFORE INSERT ON submission_versions BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(db.cancel_submission("a", 1, id).is_err());
    assert_eq!(
        sql.query_row(
            "SELECT (SELECT count(*) FROM delivery_jobs)+(SELECT count(*) FROM outbound_jobs)",
            [],
            |r| r.get::<_, usize>(0)
        )
        .unwrap(),
        2
    );
    assert_eq!(
        db.submissions("a", None).unwrap().records[0].undo_status,
        UndoStatus::Pending
    );
    sql.execute_batch("DROP TRIGGER reject_history; UPDATE delivery_jobs SET next_attempt=0; UPDATE outbound_jobs SET next_attempt=0;").unwrap();
    let local = db.claim(1).unwrap().pop().unwrap();
    let remote = db.claim_outbound(1).unwrap().pop().unwrap();
    sql.execute_batch("CREATE TRIGGER reject_history BEFORE INSERT ON submission_versions BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(db.finish(&local, Ok(())).is_err());
    assert_eq!(db.queued_message(&local).unwrap().raw, RAW);
    assert!(
        db.finish_outbound(&remote, DeliveryOutcome::Delivered)
            .is_err()
    );
    assert_eq!(db.outbound_message(&remote).unwrap().raw, RAW);
    assert_eq!(db.submissions("a", Some(&[])).unwrap().revision, 1);
    sql.execute_batch("DROP TRIGGER reject_history;").unwrap();
    db.finish(&local, Ok(())).unwrap();
    db.finish_outbound(&remote, DeliveryOutcome::Delivered)
        .unwrap();
    assert_eq!(
        db.submissions("a", None).unwrap().records[0].undo_status,
        UndoStatus::Final
    );
    drop((db, sql));
    std::fs::remove_file(path).unwrap();
}

#[path = "submission_store/retention.rs"]
mod retention;
