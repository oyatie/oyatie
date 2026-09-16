use super::*;

#[test]
fn retention_prunes_sealed_history_without_exhausting_lifetime_acceptance() {
    let path = std::env::temp_dir().join(format!(
        "mail-submission-retention-{}.sqlite",
        std::process::id()
    ));
    let db = SqliteStore::open(&path).unwrap();
    provision(&db);
    let sql = rusqlite::Connection::open(&path).unwrap();
    let accepted = db.accept_submission("a", 0, request(2000000000)).unwrap();
    sql.execute_batch("BEGIN IMMEDIATE; UPDATE submission_versions SET until_revision=2 WHERE revision=1;
        INSERT INTO submission_versions(account,id,revision,until_revision,state,identity_id,email_id,thread_id,send_at,undo)
        WITH RECURSIVE seq(n) AS (VALUES(2) UNION ALL SELECT n+1 FROM seq WHERE n<20001)
        SELECT v.account,v.id,n,CASE WHEN n=20001 THEN NULL ELSE n+1 END,v.state,v.identity_id,v.email_id,v.thread_id,v.send_at,v.undo FROM seq,submission_versions v WHERE v.revision=1;
        UPDATE submission_heads SET revision=20001; COMMIT;").unwrap();
    db.accept_submission("a", 20001, request(2000000100))
        .unwrap();
    let floor: u64 = sql
        .query_row(
            "SELECT floor FROM submission_heads WHERE account='a'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(floor, 10002);
    assert!(matches!(
        db.submission_changes("a", 1, 1),
        Err(Error::Conflict)
    ));
    assert!(matches!(
        db.query_submissions("a", Some(1), &query(Filter::All)),
        Err(SubmissionFailure::Storage(Error::Conflict))
    ));
    assert_eq!(
        db.query_submissions("a", Some(floor), &query(Filter::All))
            .unwrap()
            .ids,
        vec![accepted.records[0].id.clone()]
    );
    assert_eq!(db.submissions("a", None).unwrap().records.len(), 2);
    assert_eq!(
        db.submission_changes("a", floor, 10000)
            .unwrap()
            .changes
            .len(),
        10000
    );
    assert!(
        sql.query_row("SELECT count(*) FROM submission_versions", [], |r| r
            .get::<_, usize>(0))
            .unwrap()
            <= 10002
    );
    let mut q = query(Filter::All);
    q.position = 99;
    let page = db.query_submissions("a", None, &q).unwrap();
    assert!(page.ids.is_empty());
    assert_eq!(page.position, 0);
    q.position = 2;
    assert_eq!(db.query_submissions("a", None, &q).unwrap().position, 2);
    q.position = 0;
    q.anchor = Some(accepted.records[0].id.clone());
    q.anchor_offset = 99;
    let page = db.query_submissions("a", None, &q).unwrap();
    assert!(page.ids.is_empty());
    assert_eq!(page.position, 99);
    drop((db, sql));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn floor_snapshot_survives_pruned_destroy_sentinel_and_preserves_crossing_version() {
    let path = std::env::temp_dir().join(format!(
        "mail-submission-floor-{}.sqlite",
        std::process::id()
    ));
    let db = SqliteStore::open(&path).unwrap();
    provision(&db);
    let sql = rusqlite::Connection::open(&path).unwrap();
    let first = db.accept_submission("a", 0, request(2000000000)).unwrap();
    let crossing = db.accept_submission("a", 1, request(2000000100)).unwrap();
    db.destroy_submission("a", 2, &first.records[0].id).unwrap();
    sql.execute_batch("BEGIN IMMEDIATE; UPDATE submission_heads SET floor=3 WHERE account='a'; DELETE FROM submission_versions WHERE account='a' AND (until_revision<=3 OR (state IS NULL AND revision<=3)); COMMIT;").unwrap();
    assert_eq!(
        sql.query_row(
            "SELECT count(*) FROM submission_versions WHERE revision=3",
            [],
            |r| r.get::<_, usize>(0)
        )
        .unwrap(),
        0
    );
    assert_eq!(
        db.query_submissions("a", Some(3), &query(Filter::All))
            .unwrap()
            .ids,
        vec![crossing.records[0].id.clone()]
    );
    assert!(matches!(
        db.query_submissions("a", Some(2), &query(Filter::All)),
        Err(SubmissionFailure::Storage(Error::Conflict))
    ));
    assert!(db.submission_changes("a", 3, 1).unwrap().changes.is_empty());
    assert_eq!(
        db.accept_submission("a", 3, request(2000000200))
            .unwrap()
            .revision,
        4
    );
    drop((db, sql));
    std::fs::remove_file(path).unwrap();
}
