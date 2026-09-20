//! What `serve` and `convert` refuse: a below-version file, a `converting`
//! marker, a backup that does not match the source, and a database another
//! connection has touched.
use super::{OPERATOR, Temp, specs};
use mail_sqlite_store::contract::legacy::{LegacyAccountSpec, LegacyFixture, Shape};
use mail_sqlite_store::convert::{BackupCheck, ConvertError, Converter};
use mail_sqlite_store::{OpenError, Refusal, SqliteStore};

#[test]
fn serve_refuses_a_legacy_file_with_the_operator_step_and_an_estimate() {
    let temp = Temp::new("below");
    let database = temp.path("mail.sqlite");
    LegacyFixture::create(&database, &specs()).unwrap();
    let error = match SqliteStore::open(&database) {
        Err(OpenError::Refused(refusal)) => refusal,
        other => panic!("expected refusal, got {:?}", other.map(|_| ())),
    };
    assert_eq!(
        error,
        Refusal::BelowVersion {
            found: 1,
            accounts: 4,
            // Only the newer shape keeps bodies in `message_bodies`.
            messages: 11,
        }
    );
    let text = error.to_string();
    assert!(text.contains("mail-app convert"), "{text}");
    assert!(
        text.contains("estimated 1 s for 4 accounts, 11 messages"),
        "{text}"
    );
    // The refusal is read-only: the file is still legacy and converts.
    let converter = Converter::open(&database).unwrap();
    converter.backup_into(&temp.path("b.sqlite")).unwrap();
    converter.convert(&temp.path("b.sqlite"), OPERATOR).unwrap();
    assert!(SqliteStore::open(&database).is_ok());
}

#[test]
fn every_binary_refuses_a_file_carrying_the_converting_marker() {
    let temp = Temp::new("converting");
    let database = temp.path("mail.sqlite");
    LegacyFixture::create(&database, &specs()).unwrap();
    let db = rusqlite::Connection::open(&database).unwrap();
    db.execute_batch(
        "CREATE TABLE schema_version (version INTEGER PRIMARY KEY, state TEXT NOT NULL,
            backup_path TEXT, backup_sha256 TEXT, converted_at_utc TEXT, operator TEXT);
         INSERT INTO schema_version(version,state,backup_path) VALUES(3,'converting','/nowhere/backup.sqlite');",
    )
    .unwrap();
    drop(db);
    let error = match SqliteStore::open(&database) {
        Err(OpenError::Refused(refusal)) => refusal,
        other => panic!("expected refusal, got {:?}", other.map(|_| ())),
    };
    assert_eq!(error, Refusal::Converting { version: 3 });
    assert!(error.to_string().contains("`converting` marker present"));
    // A resumed conversion must name the recorded backup.
    assert!(matches!(
        Converter::open(&database).unwrap().convert(&temp.path("other.sqlite"), OPERATOR),
        Err(ConvertError::BackupMismatch { recorded }) if recorded == "/nowhere/backup.sqlite"
    ));
}

#[test]
fn convert_refuses_a_backup_that_is_not_a_verified_copy_of_the_source() {
    let temp = Temp::new("backup");
    let database = temp.path("mail.sqlite");
    LegacyFixture::create(&database, &specs()).unwrap();
    // Same file as the database.
    assert!(matches!(
        Converter::open(&database)
            .unwrap()
            .convert(&database, OPERATOR),
        Err(ConvertError::Backup(BackupCheck::SamePath))
    ));
    // A path that is not a database.
    assert!(matches!(
        Converter::open(&database)
            .unwrap()
            .convert(&temp.path("missing.sqlite"), OPERATOR),
        Err(ConvertError::Backup(BackupCheck::Unreadable))
    ));
    // A legacy database with a different account set.
    let other = temp.path("other.sqlite");
    LegacyFixture::create(&other, &[LegacyAccountSpec::new("zeta", Shape::Newer)]).unwrap();
    assert!(matches!(
        Converter::open(&database)
            .unwrap()
            .convert(&other, OPERATOR),
        Err(ConvertError::Backup(BackupCheck::AccountSet))
    ));
    // A copy taken before the last commit: the id set matches, the head does not.
    let stale = temp.path("stale.sqlite");
    let db = rusqlite::Connection::open(&database).unwrap();
    db.execute("VACUUM INTO ?1", [stale.to_string_lossy().as_ref()])
        .unwrap();
    db.execute(
        "INSERT INTO history_commits(account,previous,revision) VALUES('alpha',8,9)",
        [],
    )
    .unwrap();
    drop(db);
    assert_eq!(
        match Converter::open(&database)
            .unwrap()
            .convert(&stale, OPERATOR)
        {
            Err(ConvertError::Backup(check)) => check,
            other => panic!("expected a backup refusal, got {:?}", other.map(|_| ())),
        },
        BackupCheck::HistoryHead("alpha".into(), Some(9), Some(8))
    );
    // Nothing above touched the source: it is still legacy and unconverted.
    assert!(matches!(
        SqliteStore::open(&database),
        Err(OpenError::Refused(Refusal::BelowVersion { .. }))
    ));
    // `backup_into` never overwrites an existing file.
    assert!(matches!(
        Converter::open(&database).unwrap().backup_into(&stale),
        Err(ConvertError::Backup(_))
    ));
}

#[test]
fn convert_is_busy_while_another_connection_has_touched_the_database() {
    let temp = Temp::new("busy");
    let database = temp.path("mail.sqlite");
    LegacyFixture::create(&database, &specs()).unwrap();
    let reader = rusqlite::Connection::open(&database).unwrap();
    let accounts: u64 = reader
        .query_row("SELECT count(*) FROM accounts", [], |r| r.get(0))
        .unwrap();
    assert_eq!(accounts, 4);
    let started = std::time::Instant::now();
    assert!(
        matches!(Converter::open(&database), Err(ConvertError::Busy)),
        "an open WAL reader holds the shared lock the exclusive converter needs"
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(5));
    drop(reader);
    let converter = Converter::open(&database).unwrap();
    let backup = temp.path("backup.sqlite");
    converter.backup_into(&backup).unwrap();
    let conversion = converter.convert(&backup, OPERATOR).unwrap();
    assert_eq!(conversion.accounts, 4);
    assert!(SqliteStore::open(&database).is_ok());
}

#[test]
fn a_versioned_file_below_this_binary_is_told_to_recreate_not_convert() {
    // Only legacy (version 1) files have a conversion; an intermediate version
    // never reached a release, so the refusal must not point at `convert`.
    let temp = Temp::new("intermediate");
    let database = temp.path("mail.sqlite");
    drop(SqliteStore::open(&database).unwrap());
    rusqlite::Connection::open(&database)
        .unwrap()
        .execute_batch("UPDATE schema_version SET version=2")
        .unwrap();
    let error = match SqliteStore::open(&database) {
        Err(OpenError::Refused(refusal)) => refusal,
        other => panic!("expected refusal, got {:?}", other.map(|_| ())),
    };
    let text = error.to_string();
    assert!(
        text.contains("no conversion path from schema version 2"),
        "{text}"
    );
    assert!(!text.contains("--backup-verified"), "{text}");
    assert!(matches!(
        Converter::open(&database)
            .unwrap()
            .convert(&temp.path("b.sqlite"), OPERATOR),
        Err(ConvertError::NotLegacy(_))
    ));
}
