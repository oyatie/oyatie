#![forbid(unsafe_code)]
//! `mail-app convert` and the schema refusal in `serve`, driven through the
//! built binary against a `41ce37e61`-shaped database file.
#[path = "convert/legacy.rs"]
mod legacy;
use legacy::{mail_app, text};
use mail_api::MetadataStore;
use mail_sqlite_store::{SchemaState, SqliteStore};
use std::{collections::BTreeMap, ffi::OsStr, path::Path};

fn sidecar(path: &Path, suffix: &str) -> std::path::PathBuf {
    let mut name = path.file_name().unwrap().to_owned();
    name.push(suffix);
    path.with_file_name(name)
}

fn inspect(path: &Path) -> SchemaState {
    let db =
        rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    mail_sqlite_store::inspect(&db).unwrap()
}

#[test]
fn serve_refuses_a_legacy_database_before_listening() {
    let root = tempfile::tempdir().unwrap();
    let database = root.path().join("mail.sqlite");
    legacy::populated(&database);
    let (cert, key) = legacy::tls(root.path());
    let output = mail_app([
        OsStr::new("serve"),
        database.as_os_str(),
        cert.as_os_str(),
        key.as_os_str(),
    ]);
    let stderr = text(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    for expected in ["below this binary's", "mail-app convert", "estimated"] {
        assert!(stderr.contains(expected), "expected {expected}: {stderr}");
    }
    assert!(
        !stderr.contains("mail-app: SMTP"),
        "listener announced: {stderr}"
    );
    assert_eq!(
        inspect(&database),
        SchemaState::Legacy,
        "refusal must not write"
    );
    // Administration commands share the refusal.
    let output = mail_app([OsStr::new("failed"), database.as_os_str(), OsStr::new("a")]);
    assert!(!output.status.success());
    assert!(text(&output.stderr).contains("below this binary's"));
}

#[test]
fn convert_with_backup_into_holds_one_lock_and_yields_a_servable_store() {
    let root = tempfile::tempdir().unwrap();
    let database = root.path().join("mail.sqlite");
    let backup = root
        .path()
        .join("backups")
        .join("mail-before-convert.sqlite");
    std::fs::create_dir_all(backup.parent().unwrap()).unwrap();
    legacy::populated(&database);
    let output = mail_app([
        OsStr::new("convert"),
        database.as_os_str(),
        OsStr::new("--backup-into"),
        backup.as_os_str(),
    ]);
    let (stdout, stderr) = (text(&output.stdout), text(&output.stderr));
    assert!(output.status.success(), "{stderr}");
    let line = stdout
        .lines()
        .find(|l| l.contains("converted"))
        .unwrap_or_default();
    assert!(line.contains("converted 1 accounts in"), "{stdout}");
    assert!(
        line.contains(&format!("backup {} sha256 ", backup.display())),
        "{stdout}"
    );
    let digest = line
        .split("sha256 ")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    assert_eq!(digest.len(), 64, "{line}");
    assert!(
        digest
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    );
    assert!(line.contains("; operator "), "{line}");
    assert!(
        !sidecar(&database, "-wal").exists(),
        "-wal left beside the database"
    );
    assert_eq!(
        inspect(&backup),
        SchemaState::Legacy,
        "backup must be the legacy bytes"
    );
    // The audit row inside the database matches the audit line.
    let db = rusqlite::Connection::open(&database).unwrap();
    let (state, recorded, operator): (String, String, String) = db
        .query_row(
            "SELECT state,backup_sha256,operator FROM schema_version WHERE version=?1",
            [mail_sqlite_store::SCHEMA_VERSION],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    drop(db);
    assert_eq!((state.as_str(), recorded.as_str()), ("complete", digest));
    assert!(operator.contains('@'), "{operator}");
    let store = SqliteStore::open(&database).unwrap();
    let account = store.account("a").unwrap();
    assert_eq!(account.revision, 2);
    assert_eq!(account.history_floor, 1);
    assert_eq!(account.messages.len(), 1);
    assert_eq!(account.messages[0].id, "e1");
    assert_eq!(account.messages[0].size, legacy::BODY.len());
    assert_eq!(
        account.messages[0].mailboxes,
        BTreeMap::from([("inbox".to_owned(), 1)])
    );
}

#[test]
fn convert_refuses_a_backup_holding_a_different_account_set() {
    let root = tempfile::tempdir().unwrap();
    let database = root.path().join("mail.sqlite");
    let backup = root.path().join("empty-copy.sqlite");
    legacy::populated(&database);
    legacy::empty(&backup);
    let output = mail_app([
        OsStr::new("convert"),
        database.as_os_str(),
        OsStr::new("--backup-verified"),
        backup.as_os_str(),
    ]);
    let stderr = text(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("backup refused"), "{stderr}");
    assert_eq!(
        inspect(&database),
        SchemaState::Legacy,
        "refused conversion must not write"
    );
    // The same path as source and backup is refused too.
    let output = mail_app([
        OsStr::new("convert"),
        database.as_os_str(),
        OsStr::new("--backup-verified"),
        database.as_os_str(),
    ]);
    assert!(!output.status.success());
    assert!(text(&output.stderr).contains("backup refused"));
}

#[test]
fn convert_refuses_while_another_connection_uses_the_database() {
    let root = tempfile::tempdir().unwrap();
    let database = root.path().join("mail.sqlite");
    let backup = root.path().join("mail-backup.sqlite");
    legacy::populated(&database);
    let reader = rusqlite::Connection::open(&database).unwrap();
    let accounts: u64 = reader
        .query_row("SELECT count(*) FROM accounts", [], |r| r.get(0))
        .unwrap();
    assert_eq!(accounts, 1);
    let output = mail_app([
        OsStr::new("convert"),
        database.as_os_str(),
        OsStr::new("--backup-into"),
        backup.as_os_str(),
    ]);
    let stderr = text(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("stop `serve` first"), "{stderr}");
    assert!(
        !backup.exists(),
        "no backup may be taken while the database is in use"
    );
    assert_eq!(inspect(&database), SchemaState::Legacy);
    drop(reader);
    let output = mail_app([
        OsStr::new("convert"),
        database.as_os_str(),
        OsStr::new("--backup-into"),
        backup.as_os_str(),
    ]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert!(
        matches!(inspect(&database), SchemaState::Complete { version } if version == mail_sqlite_store::SCHEMA_VERSION)
    );
}

#[test]
fn serve_refuses_a_database_left_converting() {
    let root = tempfile::tempdir().unwrap();
    let database = root.path().join("mail.sqlite");
    let recorded = root.path().join("recorded-backup.sqlite");
    legacy::populated(&database);
    legacy::mark_converting(&database, &recorded.to_string_lossy());
    let (cert, key) = legacy::tls(root.path());
    let output = mail_app([
        OsStr::new("serve"),
        database.as_os_str(),
        cert.as_os_str(),
        key.as_os_str(),
    ]);
    let stderr = text(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("converting"), "{stderr}");
    assert!(
        !stderr.contains("mail-app: SMTP"),
        "listener announced: {stderr}"
    );
    // Resuming with a different backup than the marker names is refused.
    let other = root.path().join("other-backup.sqlite");
    legacy::empty(&other);
    let output = mail_app([
        OsStr::new("convert"),
        database.as_os_str(),
        OsStr::new("--backup-verified"),
        other.as_os_str(),
    ]);
    let stderr = text(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("started with backup"), "{stderr}");
    assert_eq!(
        inspect(&database),
        SchemaState::Converting {
            version: mail_sqlite_store::SCHEMA_VERSION
        }
    );
}
