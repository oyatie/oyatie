//! Bodies outside the metadata transaction: reservations keep an unlinked
//! body alive, the sweep is the sole expiry decider and tombstones a swept
//! version, references bind `(hash, version_id)`, and reads authorize
//! through the account's own link — a hash hit is never a hit.
use mail_api::{BlobStore, MetadataStore, Precondition};
use mail_kernel::{Account, BlobRef, Command, Error};
use mail_sqlite_store::SqliteStore;

const BODY: &[u8] = b"Subject: shared bytes\r\n\r\nbody\r\n";

fn store() -> SqliteStore {
    let db = SqliteStore::open(":memory:").unwrap();
    for (id, tenant) in [("a", "t1"), ("b", "t2")] {
        db.provision(
            Account::new(id, tenant, id, &format!("{id}@example.org")).unwrap(),
            &id.repeat(32),
        )
        .unwrap();
    }
    db
}

fn far_future() -> i64 {
    i64::MAX / 4
}

#[test]
fn a_reservation_holds_an_unlinked_body_until_it_expires_then_the_sweep_tombstones_it() {
    let db = store();
    let blob = db.persist("a", "append:1", BODY, 60).unwrap();
    assert_eq!(db.persist("a", "append:2", BODY, 60).unwrap(), blob);
    assert_eq!(db.read("a", &blob).unwrap(), BODY);
    // Live reservation: nothing to sweep, even far in the past.
    assert_eq!(db.orphan_sweep(0, 10).unwrap(), 0);
    assert_eq!(db.orphan_sweep(far_future(), 10).unwrap(), 1);
    assert_eq!(db.read("a", &blob), Err(Error::NotFound));
    // The reference is dangling for good: a commit after the sweep fails closed.
    let append = Command::Append {
        mailboxes: vec!["inbox".into()],
        received_at: 1,
        blob: blob.clone(),
        keywords: vec![],
    };
    assert_eq!(
        db.execute("a", Precondition::Observed(0), vec![append]),
        Err(Error::NotFound)
    );
    // Re-persisting the same bytes is a new version.
    let again = db.persist("a", "append:3", BODY, 60).unwrap();
    assert_eq!(again.hash, blob.hash);
    assert_ne!(again.version_id, blob.version_id);
    assert_eq!(db.read("a", &again).unwrap(), BODY);
}

#[test]
fn a_linked_body_survives_the_sweep_and_the_last_unlink_makes_it_sweepable() {
    let db = store();
    let append = db
        .append("a", vec!["inbox".into()], BODY, vec![], 1)
        .unwrap();
    let Command::Append { blob, .. } = &append else {
        unreachable!()
    };
    let blob = blob.clone();
    db.execute("a", Precondition::Observed(0), vec![append])
        .unwrap();
    assert_eq!(db.orphan_sweep(far_future(), 10).unwrap(), 0);
    assert_eq!(db.blob("a", "e1").unwrap(), BODY);
    let revision = db.account("a").unwrap().revision;
    db.execute(
        "a",
        Precondition::Observed(revision),
        vec![Command::Destroy { id: "e1".into() }],
    )
    .unwrap();
    assert_eq!(db.orphan_sweep(far_future(), 10).unwrap(), 1);
    assert_eq!(db.read("a", &blob), Err(Error::NotFound));
}

#[test]
fn renewal_moves_the_scope_expiry_and_a_long_batch_commits_after_the_original_window() {
    let db = store();
    let first = db.persist("a", "append:batch", BODY, 10).unwrap();
    let second = db
        .persist("a", "append:batch", b"Subject: second\r\n\r\nbody\r\n", 10)
        .unwrap();
    // Each literal renews the one reservation key; both bodies ride on it.
    db.renew("a", "append:batch", 1000).unwrap();
    assert_eq!(db.orphan_sweep(500, 10).unwrap(), 0);
    let commands = [first, second]
        .into_iter()
        .map(|blob| Command::Append {
            mailboxes: vec!["inbox".into()],
            received_at: 1,
            blob,
            keywords: vec![],
        })
        .collect();
    let execution = db
        .execute("a", Precondition::Observed(0), commands)
        .unwrap();
    assert_eq!(execution.ids.len(), 2);
    assert_eq!(db.renew("a", "append:missing", 10), Err(Error::NotFound));
}

#[test]
fn the_same_bytes_in_two_accounts_share_one_version_but_never_each_other_s_link() {
    let db = store();
    db.provision(
        Account::new("c", "t3", "c", "c@example.org").unwrap(),
        &"c".repeat(32),
    )
    .unwrap();
    let a = db.put_blob("a", BODY).unwrap();
    let b = db.put_blob("b", BODY).unwrap();
    assert_eq!(a, b, "blob ids are content addressed");
    assert_eq!(db.blob("a", &a).unwrap(), BODY);
    assert_eq!(db.blob("b", &b).unwrap(), BODY);
    // Cross-tenant probe: knowing the hash and version grants nothing to an
    // account without a link or reservation — not a read, not a commit.
    let probe = BlobRef {
        hash: a[1..].into(),
        version_id: "1".into(),
        size: BODY.len(),
    };
    assert_eq!(db.read("b", &probe).unwrap(), BODY);
    assert_eq!(db.read("c", &probe), Err(Error::NotFound));
    assert_eq!(db.blob("c", &a), Err(Error::NotFound));
    let append = Command::Append {
        mailboxes: vec!["inbox".into()],
        received_at: 1,
        blob: probe,
        keywords: vec![],
    };
    assert_eq!(
        db.execute("c", Precondition::Observed(0), vec![append]),
        Err(Error::NotFound)
    );
}
