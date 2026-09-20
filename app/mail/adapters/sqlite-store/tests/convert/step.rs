//! A released schema steps forward: a version-3 file (S3's, on `dev`) is
//! refused by `serve` with the operator step, and `convert` advances it in
//! one audited step that adds the owner epoch and releases live leases.
use super::{OPERATOR, Temp, sha256_of};
use mail_api::{DeliveryQueue, DeliveryTarget};
use mail_kernel::Account;
use mail_sqlite_store::convert::{ConvertError, Converter};
use mail_sqlite_store::{OpenError, Refusal, SqliteStore};
use std::path::Path;

/// A version-3 file as an S3 binary left it: no `epoch` columns, one live
/// delivery lease, and the `schema_version` row at 3.
fn version_three(database: &Path) {
    let db = SqliteStore::open(database).unwrap();
    db.provision(
        Account::new("a", "t", "alice", "alice@example.org").unwrap(),
        &"a".repeat(32),
    )
    .unwrap();
    let targets = [DeliveryTarget {
        account: "a".into(),
        address: "alice@example.org".into(),
    }];
    db.enqueue("s@example.net", &targets, b"Subject: v3\r\n\r\nx")
        .unwrap();
    assert_eq!(db.claim(1).unwrap().len(), 1);
    drop(db);
    rusqlite::Connection::open(database)
        .unwrap()
        .execute_batch(
            "ALTER TABLE delivery_jobs DROP COLUMN epoch; ALTER TABLE delivery_jobs ADD COLUMN token TEXT;
             ALTER TABLE outbound_jobs DROP COLUMN epoch; ALTER TABLE outbound_jobs ADD COLUMN token TEXT;
             ALTER TABLE failed_delivery_jobs DROP COLUMN epoch;
             UPDATE delivery_jobs SET token='0123456789abcdef0123456789abcdef';
             UPDATE schema_version SET version=3;",
        )
        .unwrap();
}

#[test]
fn a_version_three_file_is_refused_until_the_audited_step_adds_the_epoch_and_releases_leases() {
    let temp = Temp::new("step");
    let database = temp.path("mail.sqlite");
    version_three(&database);
    let refusal = match SqliteStore::open(&database) {
        Err(OpenError::Refused(refusal)) => refusal,
        other => panic!("expected refusal, got {:?}", other.map(|_| ())),
    };
    assert!(matches!(refusal, Refusal::BelowVersion { found: 3, .. }));
    let text = refusal.to_string();
    assert!(
        text.contains("--backup-verified") && text.contains("versioned step"),
        "{text}"
    );
    let backup = temp.path("backup.sqlite");
    let converter = Converter::open(&database).unwrap();
    converter.backup_into(&backup).unwrap();
    let conversion = converter.convert(&backup, OPERATOR).unwrap();
    assert_eq!(conversion.accounts, 0);
    assert_eq!(conversion.backup_sha256, sha256_of(&backup));
    let sql = rusqlite::Connection::open(&database).unwrap();
    let rows: Vec<(u64, String, String)> = sql
        .prepare("SELECT version,state,coalesce(operator,'') FROM schema_version ORDER BY version")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        rows,
        [
            (3, "complete".to_owned(), String::new()),
            (4, "complete".to_owned(), OPERATOR.to_owned())
        ]
    );
    // The live lease from the S3 binary is released; its token is left as
    // it was and never consulted again.
    let (lease_until, epoch, token): (i64, u64, Option<String>) = sql
        .query_row(
            "SELECT lease_until,epoch,token FROM delivery_jobs",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!((lease_until, epoch), (0, 0));
    assert_eq!(token.as_deref(), Some("0123456789abcdef0123456789abcdef"));
    drop(sql);
    let db = SqliteStore::open(&database).unwrap();
    let lease = db.claim(1).unwrap().pop().unwrap();
    assert_eq!(lease.epoch, 1);
    assert!(db.queued_message(&lease).is_ok());
    // A second run has nothing to do and says so.
    drop(db);
    assert!(matches!(
        Converter::open(&database)
            .unwrap()
            .convert(&backup, OPERATOR),
        Err(ConvertError::NotLegacy(_))
    ));
}

#[test]
fn a_converting_marker_from_another_binary_is_refused_not_resumed() {
    // An S3 binary crashed after `begin` (marker `converting@3`); this binary
    // must not finish that conversion with its own table shapes.
    let temp = Temp::new("foreign");
    let database = temp.path("mail.sqlite");
    version_three(&database);
    let backup = temp.path("backup.sqlite");
    std::fs::copy(&database, &backup).unwrap();
    rusqlite::Connection::open(&database)
        .unwrap()
        .execute(
            "UPDATE schema_version SET state='converting',backup_path=?1,backup_sha256=?2",
            rusqlite::params![backup.to_string_lossy(), sha256_of(&backup)],
        )
        .unwrap();
    let error = Converter::open(&database)
        .unwrap()
        .convert(&backup, OPERATOR)
        .unwrap_err();
    assert!(matches!(error, ConvertError::ForeignMarker { version: 3 }));
    let text = error.to_string();
    assert!(
        text.contains("started by another binary") && text.contains("restore the verified backup"),
        "{text}"
    );
    assert!(matches!(
        SqliteStore::open(&database),
        Err(OpenError::Refused(Refusal::Converting { version: 3 }))
    ));
}
