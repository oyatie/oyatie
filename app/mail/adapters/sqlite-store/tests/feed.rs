//! `ChangeFeed`: a dirty key per enabled consumer written by the change
//! transaction; an acknowledgement clears it only at the account's tail;
//! poison is never skipped or falsely acked; a cursor below the floor is
//! refused until retired; fair round-robin over tenants; audited operator rows.
use mail_api::{BlobStore, ChangeFeed, Consumer, FeedRead, MetadataStore, Precondition, Resume};
use mail_kernel::{Account, Command, Error, RetentionPolicy};
use mail_sqlite_store::SqliteStore;

const FOUNDRY: Consumer = Consumer::FoundryRecords;
const OP: &str = "ops@example.org";

fn store(enabled: &[Consumer]) -> SqliteStore {
    let db = SqliteStore::open(":memory:")
        .unwrap()
        .with_consumers(enabled);
    for (id, tenant) in [("a1", "ta"), ("a2", "ta"), ("b1", "tb"), ("b2", "tb")] {
        db.provision(
            Account::new(id, tenant, id, &format!("{id}@example.org")).unwrap(),
            &id.repeat(16),
        )
        .unwrap();
    }
    db
}

fn touch(db: &SqliteStore, account: &str) -> u64 {
    let revision = db.account(account).unwrap().revision;
    let append = db
        .append(
            account,
            vec!["inbox".into()],
            b"Subject: x\r\n\r\nx",
            vec![],
            1,
        )
        .unwrap();
    db.execute(account, Precondition::Observed(revision), vec![append])
        .unwrap()
        .revision
}

fn dirty(db: &SqliteStore, resume: &mut Resume) -> Vec<String> {
    db.dirty(FOUNDRY, resume, 0, 100, 100)
        .unwrap()
        .into_iter()
        .map(|d| d.account)
        .collect()
}

#[test]
fn a_commit_marks_every_enabled_consumer_and_an_ack_clears_only_at_the_tail() {
    let silent = store(&[]);
    touch(&silent, "a1");
    assert!(
        dirty(&silent, &mut Resume::default()).is_empty(),
        "no consumer enabled"
    );

    let db = store(&[FOUNDRY]);
    // SMTP ingest marks too, in its own transaction.
    db.deliver(&["b1@example.org".into()], b"Subject: in\r\n\r\nx")
        .unwrap();
    assert_eq!(dirty(&db, &mut Resume::default()), ["b1"]);
    db.retire_cursor(FOUNDRY, Some("tb"), OP, "reset").unwrap();
    let r1 = touch(&db, "a1");
    let r2 = touch(&db, "a1");
    assert_eq!(dirty(&db, &mut Resume::default()), ["a1"]);
    // Absent cursor ⇒ the feed starts at the tail: nothing to replay.
    let FeedRead::Changes { cursor, page } = db.changes(FOUNDRY, "a1", 100).unwrap() else {
        panic!()
    };
    assert_eq!((cursor.0, page.rows.len()), (r2, 0));
    // An ack short of the tail keeps the key with a delay; at the tail it clears.
    db.acknowledge(FOUNDRY, "a1", r1, 100, 30).unwrap();
    assert!(
        dirty(&db, &mut Resume::default()).is_empty(),
        "not_before defers it"
    );
    assert_eq!(
        db.dirty(FOUNDRY, &mut Resume::default(), 130, 100, 100)
            .unwrap()[0]
            .account,
        "a1"
    );
    let FeedRead::Changes { cursor, page } = db.changes(FOUNDRY, "a1", 100).unwrap() else {
        panic!()
    };
    assert_eq!((cursor.0, page.revision), (r1, r2));
    db.acknowledge(FOUNDRY, "a1", r2, 130, 30).unwrap();
    assert!(
        db.dirty(FOUNDRY, &mut Resume::default(), 1000, 100, 100)
            .unwrap()
            .is_empty()
    );
    // Cursors never move backwards and never past the tail.
    db.acknowledge(FOUNDRY, "a1", r1, 0, 0).unwrap();
    assert_eq!(db.cursors("a1", &[FOUNDRY]).unwrap(), [(FOUNDRY, r2)]);
    assert_eq!(
        db.acknowledge(FOUNDRY, "a1", r2 + 1, 0, 0),
        Err(Error::Conflict)
    );
}

#[test]
fn poison_is_kept_and_never_acked_until_dead_lettered_or_retired() {
    let db = store(&[FOUNDRY]);
    let r = touch(&db, "a1");
    db.poison(FOUNDRY, "a1", "consumer refused").unwrap();
    assert!(
        dirty(&db, &mut Resume::default()).is_empty(),
        "poison is not scheduled"
    );
    assert_eq!(
        db.acknowledge(FOUNDRY, "a1", r, 0, 0),
        Err(Error::Forbidden)
    );
    let poisoned = db.poisoned(FOUNDRY).unwrap();
    assert_eq!(
        (poisoned[0].0.account.as_str(), poisoned[0].1.as_str()),
        ("a1", "consumer refused")
    );
    assert_eq!(
        db.dead_letter(FOUNDRY, "a1", r, OP, ""),
        Err(Error::Invalid),
        "a reason is required"
    );
    db.dead_letter(FOUNDRY, "a1", r, OP, "operator skipped the change")
        .unwrap();
    assert!(db.poisoned(FOUNDRY).unwrap().is_empty());
    assert_eq!(db.cursors("a1", &[FOUNDRY]).unwrap(), [(FOUNDRY, r)]);
    // Released: the account is schedulable again and a tail ack clears it.
    let r2 = touch(&db, "a1");
    assert_eq!(dirty(&db, &mut Resume::default()), ["a1"]);
    db.acknowledge(FOUNDRY, "a1", r2, 0, 0).unwrap();
    assert!(dirty(&db, &mut Resume::default()).is_empty());
    let audit = db.audit(10).unwrap();
    assert_eq!(audit[0].kind, "dead-letter");
    assert!(
        audit[0].detail.contains("account=a1") && audit[0].operator == OP,
        "{audit:?}"
    );
}

#[test]
fn a_cursor_below_the_floor_is_refused_until_retired_and_retirement_releases_the_hold() {
    let db = store(&[FOUNDRY]);
    let r1 = touch(&db, "a1");
    for _ in 0..3 {
        touch(&db, "a1");
    }
    db.acknowledge(FOUNDRY, "a1", r1, 0, 0).unwrap();
    // Compaction with the consumer enabled never passes its cursor.
    let policy = RetentionPolicy {
        max_age_secs: 0,
        max_rows: 1,
    };
    let cursors = db.cursors("a1", &[FOUNDRY]).unwrap();
    db.compact_history("a1", i64::MAX / 2, policy, &cursors)
        .unwrap();
    assert!(matches!(
        db.changes(FOUNDRY, "a1", 100).unwrap(),
        FeedRead::Changes { .. }
    ));
    // Disabled consumer: compaction passes the old cursor; re-enabled, the
    // feed reports BelowFloor and refuses to ack.
    db.compact_history("a1", i64::MAX / 2, policy, &[]).unwrap();
    let floor = db.account("a1").unwrap().history_floor;
    assert!(floor > r1);
    assert!(
        matches!(db.changes(FOUNDRY, "a1", 100).unwrap(), FeedRead::BelowFloor { cursor, floor: f } if cursor.0 == r1 && f == floor)
    );
    // The read poisoned the key: it no longer occupies a tenant slot.
    assert_eq!(db.poisoned(FOUNDRY).unwrap()[0].1, "cursor-below-floor");
    assert!(dirty(&db, &mut Resume::default()).is_empty());
    assert_eq!(
        db.acknowledge(FOUNDRY, "a1", r1, 0, 0),
        Err(Error::Conflict)
    );
    assert_eq!(
        db.retire_cursor(FOUNDRY, Some("ta"), OP, "consumer redeployed")
            .unwrap(),
        1
    );
    // Absent cursor ⇒ tail again; the audit row records it.
    assert!(
        matches!(db.changes(FOUNDRY, "a1", 100).unwrap(), FeedRead::Changes { cursor, .. } if cursor.0 == db.account("a1").unwrap().revision)
    );
    assert_eq!(db.audit(1).unwrap()[0].kind, "cursor-retired");
}

#[test]
fn scheduling_is_round_robin_over_tenants_with_a_per_tenant_cap_and_a_resume_position() {
    let db = store(&[FOUNDRY]);
    for account in ["a1", "a2", "b1", "b2"] {
        touch(&db, account);
    }
    let mut resume = Resume::default();
    let first = db.dirty(FOUNDRY, &mut resume, 0, 1, 2).unwrap();
    assert_eq!(
        first.iter().map(|d| d.tenant.as_str()).collect::<Vec<_>>(),
        ["ta", "tb"]
    );
    // A stalled tenant A (never acked) does not starve tenant B: the cap
    // bounds A's share and the resume position rotates the start.
    let mut seen_b = 0;
    for _ in 0..4 {
        for d in db.dirty(FOUNDRY, &mut resume, 0, 1, 1).unwrap() {
            if d.tenant == "tb" {
                seen_b += 1;
                db.acknowledge(
                    FOUNDRY,
                    &d.account,
                    db.account(&d.account).unwrap().revision,
                    0,
                    0,
                )
                .unwrap();
            }
        }
    }
    assert_eq!(seen_b, 2, "both B accounts were served within four rounds");
    assert!(
        db.dirty(FOUNDRY, &mut resume, 0, 10, 10)
            .unwrap()
            .iter()
            .all(|d| d.tenant == "ta")
    );
}

#[test]
fn reconcile_re_marks_accounts_whose_cursor_is_behind_their_tail() {
    // Two handles on one file: the enabled one acks at the tail; a commit
    // through the unconfigured one writes no dirty key — the lost-key state.
    let path = std::env::temp_dir().join(format!(
        "mail-feed-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let db = SqliteStore::open(&path).unwrap().with_consumers(&[FOUNDRY]);
    db.provision(
        Account::new("a1", "ta", "a1", "a1@example.org").unwrap(),
        &"a1".repeat(16),
    )
    .unwrap();
    let silent = SqliteStore::open(&path).unwrap();
    let r = touch(&db, "a1");
    db.acknowledge(FOUNDRY, "a1", r, 0, 0).unwrap();
    touch(&silent, "a1");
    assert!(dirty(&db, &mut Resume::default()).is_empty(), "key lost");
    assert_eq!(db.reconcile(FOUNDRY, OP, "after a crash").unwrap(), 1);
    assert_eq!(dirty(&db, &mut Resume::default()), ["a1"]);
    assert_eq!(db.reconcile(FOUNDRY, OP, "again").unwrap(), 0);
    assert_eq!(db.audit(1).unwrap()[0].kind, "reconcile");
    drop((db, silent));
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(path.with_extension(format!("sqlite{suffix}")));
    }
}
