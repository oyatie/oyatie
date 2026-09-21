use mail_api::{DeliveryOutcome, DeliveryQueue, SubmissionQueue};
use mail_kernel::{Account, Error};
use mail_sqlite_store::SqliteStore;

const RAW: &[u8] = b"From: alice@example.org\r\nSubject: outbound\r\n\r\nbody\r\n";

fn provision(db: &SqliteStore) {
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
}

#[test]
fn submission_accepts_local_and_remote_recipients_atomically_and_isolates_jobs() {
    let db = SqliteStore::open(":memory:").unwrap();
    provision(&db);
    let recipients = [
        "alice@example.org".into(),
        "one@remote.org".into(),
        "two@remote.org".into(),
        "one@remote.org".into(),
        "One@REMOTE.ORG".into(),
    ];
    assert_eq!(
        db.enqueue_submission("a", "forged@example.org", &recipients, RAW),
        Err(Error::Forbidden)
    );
    db.enqueue_submission("a", "alice@example.org", &recipients, RAW)
        .unwrap();
    let local = db.claim(10).unwrap();
    assert_eq!(local.len(), 1);
    db.finish(&local[0], Ok(())).unwrap();
    let leases = db.claim_outbound(10).unwrap();
    assert_eq!(leases.len(), 3);
    assert!(
        leases
            .iter()
            .any(|lease| lease.recipient == "One@remote.org")
    );
    assert_ne!(leases[0].recipient, leases[1].recipient);
    for lease in &leases {
        assert_eq!(db.outbound_message(lease).unwrap().raw, RAW);
    }
    // The fence is ownership in time: a lease from before this claim (epoch
    // 0, never claimed) can neither read nor settle the job.
    let mut forged = leases[0].clone();
    forged.epoch = 0;
    assert!(matches!(db.outbound_message(&forged), Err(Error::Conflict)));
    assert_eq!(
        db.finish_outbound(&forged, DeliveryOutcome::Delivered),
        Err(Error::Conflict)
    );
    db.finish_outbound(&leases[0], DeliveryOutcome::Delivered)
        .unwrap();
    assert_eq!(db.outbound_message(&leases[1]).unwrap().raw, RAW);
    db.finish_outbound(&leases[1], DeliveryOutcome::Delivered)
        .unwrap();
    db.finish_outbound(&leases[2], DeliveryOutcome::Delivered)
        .unwrap();
    assert!(db.claim_outbound(10).unwrap().is_empty());
}

#[test]
fn outbound_leases_retry_survive_restart_and_create_one_durable_failure_notice() {
    let path = std::env::temp_dir().join(format!("mail-outbound-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    provision(&db);
    db.enqueue_submission("a", "alice@example.org", &["one@remote.org".into()], RAW)
        .unwrap();
    let first = db.claim_outbound(1).unwrap().pop().unwrap();
    let sql = rusqlite::Connection::open(&path).unwrap();
    sql.execute("UPDATE outbound_jobs SET lease_until=0,next_attempt=0", [])
        .unwrap();
    assert_eq!(db.renew_outbound(&first), Err(Error::Conflict));
    drop(db);
    let db = SqliteStore::open(&path).unwrap();
    let second = db.claim_outbound(1).unwrap().pop().unwrap();
    assert_eq!((first.epoch, second.epoch), (1, 2));
    assert_eq!(second.attempt, 2);
    assert_eq!(db.renew_outbound(&first), Err(Error::Conflict));
    let shortened: i64 = sql
        .query_row(
            "UPDATE outbound_jobs SET lease_until=unixepoch()+60 RETURNING lease_until",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let started: i64 = sql
        .query_row("SELECT unixepoch()", [], |r| r.get(0))
        .unwrap();
    db.renew_outbound(&second).unwrap();
    let (deadline, next_attempt, finished): (i64, i64, i64) = sql
        .query_row(
            "SELECT lease_until,next_attempt,unixepoch() FROM outbound_jobs",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert!(deadline > shortened);
    assert_eq!(deadline, next_attempt);
    assert!((started + 120..=finished + 120).contains(&deadline));
    assert_eq!(
        db.finish_outbound(&first, DeliveryOutcome::Delivered),
        Err(Error::Conflict)
    );
    db.finish_outbound(&second, DeliveryOutcome::Temporary(451))
        .unwrap();
    assert!(db.claim_outbound(1).unwrap().is_empty());
    assert_eq!(
        sql.query_row("SELECT count(*) FROM submission_notices", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    sql.execute("UPDATE outbound_jobs SET next_attempt=0", [])
        .unwrap();
    let third = db.claim_outbound(1).unwrap().pop().unwrap();
    assert_eq!(
        db.finish_outbound(&third, DeliveryOutcome::Permanent(451)),
        Err(Error::Invalid)
    );
    sql.execute_batch("CREATE TRIGGER reject_notice BEFORE INSERT ON delivery_jobs BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert_eq!(
        db.finish_outbound(&third, DeliveryOutcome::Permanent(550)),
        Err(Error::Conflict)
    );
    assert_eq!(db.outbound_message(&third).unwrap().raw, RAW);
    sql.execute_batch("DROP TRIGGER reject_notice").unwrap();
    db.finish_outbound(&third, DeliveryOutcome::Permanent(550))
        .unwrap();
    assert_eq!(
        db.finish_outbound(&third, DeliveryOutcome::Permanent(550)),
        Err(Error::Conflict)
    );
    let notice = db.claim(10).unwrap();
    assert_eq!(notice.len(), 1);
    let linked: Vec<(String, String)> = sql
        .prepare("SELECT notice,submission FROM submission_notices")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        linked,
        vec![(notice[0].message.clone(), third.message.clone())]
    );
    let notice = db.queued_message(&notice[0]).unwrap();
    assert!(notice.sender.is_empty());
    let notice = String::from_utf8(notice.raw).unwrap();
    assert!(notice.contains("multipart/report; report-type=delivery-status"));
    assert!(notice.contains("Final-Recipient: rfc822; one@remote.org\r\n"));
    assert!(notice.contains("Diagnostic-Code: smtp; 550\r\n"));
    assert!(!notice.contains("Subject: outbound"));
    assert_eq!(
        sql.query_row("SELECT count(*) FROM submitted_messages", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    drop(db);
    drop(sql);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn expired_remote_mail_becomes_a_notice_and_capacity_recovers() {
    let path = std::env::temp_dir().join(format!("mail-expiry-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    provision(&db);
    let sql = rusqlite::Connection::open(&path).unwrap();
    sql.execute("UPDATE accounts SET quota_bytes=?1", [RAW.len()])
        .unwrap();
    let recipients = ["one@remote.org".into()];
    db.enqueue_submission("a", "alice@example.org", &recipients, RAW)
        .unwrap();
    assert_eq!(
        db.enqueue_submission("a", "alice@example.org", &recipients, RAW),
        Err(Error::OverQuota)
    );
    let lease = db.claim_outbound(1).unwrap().pop().unwrap();
    sql.execute(
        "UPDATE submitted_messages SET received_at=unixepoch()-432001",
        [],
    )
    .unwrap();
    assert!(db.outbound_message(&lease).unwrap().received_at > 0);
    db.finish_outbound(&lease, DeliveryOutcome::Temporary(451))
        .unwrap();
    assert!(db.claim_outbound(1).unwrap().is_empty());
    let notice = db.claim(1).unwrap().pop().unwrap();
    assert!(
        String::from_utf8(db.queued_message(&notice).unwrap().raw)
            .unwrap()
            .contains("Status: 5.4.7")
    );
    assert_eq!(
        db.enqueue_submission("a", "alice@example.org", &recipients, RAW),
        Err(Error::OverQuota)
    );
    db.finish(&notice, Ok(())).unwrap();
    db.enqueue_submission("a", "alice@example.org", &recipients, RAW)
        .unwrap();
    drop(db);
    drop(sql);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn mixed_submission_rolls_back_local_work_if_remote_insert_fails() {
    let path = std::env::temp_dir().join(format!("mail-mixed-{}.sqlite", std::process::id()));
    let db = SqliteStore::open(&path).unwrap();
    provision(&db);
    let sql = rusqlite::Connection::open(&path).unwrap();
    sql.execute_batch("CREATE TRIGGER reject_remote BEFORE INSERT ON outbound_jobs BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    let recipients = ["alice@example.org".into(), "one@remote.org".into()];
    assert!(
        db.enqueue_submission("a", "alice@example.org", &recipients, RAW)
            .is_err()
    );
    assert!(db.claim(1).unwrap().is_empty());
    assert!(db.claim_outbound(1).unwrap().is_empty());
    assert_eq!(sql.query_row("SELECT (SELECT count(*) FROM queued_messages)+(SELECT count(*) FROM submitted_messages)", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    sql.execute_batch("DROP TRIGGER reject_remote").unwrap();
    sql.execute("UPDATE accounts SET quota_bytes=?1", [RAW.len()])
        .unwrap();
    // A local copy to the sender and a remote recipient both consume its queue budget.
    assert_eq!(
        db.enqueue_submission("a", "alice@example.org", &recipients, RAW),
        Err(Error::OverQuota)
    );
    assert!(db.claim(1).unwrap().is_empty());
    assert!(db.claim_outbound(1).unwrap().is_empty());
    drop(db);
    drop(sql);
    std::fs::remove_file(path).unwrap();
}
