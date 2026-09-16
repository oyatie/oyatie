use mail_api::{
    DeliveryOutcome, DeliveryQueue, Store, SubmissionAcceptance, SubmissionFailure,
    SubmissionQueue, SubmissionStore,
};
use mail_kernel::{
    Account, EnvelopeAddress, Error, SubmissionEnvelope, SubmissionFilter, SubmissionQuery,
    SubmissionRecord, UndoStatus,
};
use mail_sqlite_store::SqliteStore;
use std::{collections::BTreeMap, path::PathBuf};

const RAW: &[u8] =
    b"From: alice@example.org\r\nTo: remote@example.net\r\nSubject: queued\r\n\r\nbody\r\n";
struct Fixture {
    db: SqliteStore,
    path: PathBuf,
}
impl Fixture {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "mail-submission-{name}-{}.sqlite",
            std::process::id()
        ));
        let db = SqliteStore::open(&path).unwrap();
        db.provision(
            Account::new("a", "t", "alice", "alice@example.org").unwrap(),
            &"a".repeat(32),
        )
        .unwrap();
        db.deliver(&["alice@example.org".into()], RAW).unwrap();
        Self { db, path }
    }
    fn sql(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(&self.path).unwrap()
    }
    fn request(&self, scheduled: bool) -> SubmissionAcceptance {
        let address = |email: &str| EnvelopeAddress {
            email: email.into(),
            parameters: BTreeMap::new(),
        };
        let now: i64 = self
            .sql()
            .query_row("SELECT unixepoch()", [], |r| r.get(0))
            .unwrap();
        SubmissionAcceptance {
            record: SubmissionRecord {
                id: String::new(),
                email_id: "e1".into(),
                identity_id: "a".into(),
                thread_id: String::new(),
                envelope: SubmissionEnvelope {
                    mail_from: address("alice@example.org"),
                    rcpt_to: vec![address("alice@example.org"), address("remote@example.net")],
                },
                send_at: now + if scheduled { 864000 } else { 0 },
                undo_status: UndoStatus::Pending,
                delivery_status: BTreeMap::new(),
            },
            email_revision: 1,
            raw: RAW.into(),
            allow_remote: true,
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[test]
fn acceptance_is_atomic_and_schedule_cancel_survives_reopen() {
    let f = Fixture::new("schedule");
    let mut bad = f.request(true);
    bad.email_revision = 0;
    assert!(matches!(
        f.db.accept_submission("a", 0, bad),
        Err(Error::Conflict)
    ));
    let mut bad = f.request(true);
    bad.allow_remote = false;
    assert!(matches!(
        f.db.accept_submission("a", 0, bad),
        Err(Error::Forbidden)
    ));
    f.sql().execute_batch("CREATE TRIGGER reject_submission BEFORE INSERT ON submission_versions BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert!(f.db.accept_submission("a", 0, f.request(true)).is_err());
    assert_eq!(
        f.sql()
            .query_row(
                "SELECT (SELECT count(*) FROM delivery_jobs)+(SELECT count(*) FROM outbound_jobs)",
                [],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
    f.sql()
        .execute_batch("DROP TRIGGER reject_submission")
        .unwrap();
    let accepted = f.db.accept_submission("a", 0, f.request(true)).unwrap();
    let record = &accepted.records[0];
    assert_eq!(accepted.revision, 1);
    assert!(!record.id.is_empty());
    assert!(!record.thread_id.is_empty());
    assert!(f.db.claim(10).unwrap().is_empty());
    assert!(f.db.claim_outbound(10).unwrap().is_empty());
    let reopened = SqliteStore::open(&f.path).unwrap();
    assert_eq!(
        reopened.submissions("a", None).unwrap().records,
        accepted.records
    );
    let canceled = reopened.cancel_submission("a", 1, &record.id).unwrap();
    assert_eq!(canceled.revision, 2);
    assert_eq!(canceled.records[0].undo_status, UndoStatus::Canceled);
    assert_eq!(
        f.sql()
            .query_row(
                "SELECT (SELECT count(*) FROM delivery_jobs)+(SELECT count(*) FROM outbound_jobs)",
                [],
                |r| r.get::<_, u64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn claimed_submission_cannot_cancel_even_after_lease_expiry_or_recipient_completion() {
    let f = Fixture::new("claimed");
    let accepted = f.db.accept_submission("a", 0, f.request(false)).unwrap();
    let id = &accepted.records[0].id;
    let local = f.db.claim(1).unwrap().pop().unwrap();
    f.db.finish(&local, Ok(())).unwrap();
    let remote = f.db.claim_outbound(1).unwrap().pop().unwrap();
    f.sql()
        .execute("UPDATE outbound_jobs SET lease_until=0,next_attempt=0", [])
        .unwrap();
    assert!(matches!(
        f.db.cancel_submission("a", f.db.submissions("a", Some(&[])).unwrap().revision, id),
        Err(SubmissionFailure::CannotUnsend)
    ));
    let next = f.db.claim_outbound(1).unwrap().pop().unwrap();
    assert_eq!(
        f.db.finish_outbound(&remote, DeliveryOutcome::Delivered),
        Err(Error::Conflict)
    );
    f.db.finish_outbound(&next, DeliveryOutcome::Delivered)
        .unwrap();
    let final_state = f.db.submissions("a", None).unwrap();
    assert_eq!(final_state.revision, 3);
    assert_eq!(final_state.records[0].undo_status, UndoStatus::Final);
    assert!(matches!(
        f.db.cancel_submission("a", 3, id),
        Err(SubmissionFailure::CannotUnsend)
    ));
}

#[test]
fn destroy_history_keeps_delivery_and_historical_query_and_changes() {
    let f = Fixture::new("history");
    let accepted = f.db.accept_submission("a", 0, f.request(false)).unwrap();
    let id = accepted.records[0].id.clone();
    assert_eq!(f.db.destroy_submission("a", 1, &id).unwrap(), 2);
    assert!(f.db.submissions("a", None).unwrap().records.is_empty());
    let query = SubmissionQuery {
        filter: SubmissionFilter::All,
        sort: vec![],
        position: 0,
        anchor: None,
        anchor_offset: 0,
        limit: 256,
    };
    assert_eq!(
        f.db.query_submissions("a", Some(1), &query).unwrap().ids,
        vec![id.clone()]
    );
    assert!(
        f.db.query_submissions("a", None, &query)
            .unwrap()
            .ids
            .is_empty()
    );
    let changes = f.db.submission_changes("a", 0, 1).unwrap();
    assert!(changes.has_more);
    assert_eq!(changes.revision, 1);
    assert!(changes.changes[0].after.is_some());
    let changes = f.db.submission_changes("a", 1, 10).unwrap();
    assert_eq!(changes.revision, 2);
    assert!(changes.changes[0].after.is_none());
    assert_eq!(f.db.claim(10).unwrap().len(), 1);
    let lease = f.db.claim_outbound(10).unwrap().pop().unwrap();
    assert_eq!(f.db.outbound_message(&lease).unwrap().raw, RAW);
    f.db.finish_outbound(&lease, DeliveryOutcome::Delivered)
        .unwrap();
    assert!(f.db.submissions("a", None).unwrap().records.is_empty());
}
