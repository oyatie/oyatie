//! A `41ce37e61`-shaped database built directly with SQL, so this suite
//! exercises exactly the bytes an operator's disk holds, plus the process
//! helpers the conversion tests drive the binary with.
use rusqlite::{Connection, params};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

pub const BODY: &[u8] = b"Subject: x\r\n\r\nbody-bytes";

const TABLES: &str = "
    CREATE TABLE accounts(id TEXT PRIMARY KEY, address TEXT NOT NULL UNIQUE, token BLOB UNIQUE,
        state TEXT NOT NULL CHECK(json_valid(state)));
    CREATE TABLE history_commits(account TEXT NOT NULL, previous INTEGER NOT NULL,
        revision INTEGER NOT NULL, PRIMARY KEY(account,revision));
    CREATE TABLE submission_heads(account TEXT PRIMARY KEY, revision INTEGER NOT NULL,
        floor INTEGER NOT NULL DEFAULT 0);
    CREATE TABLE message_bodies(account TEXT NOT NULL, id TEXT NOT NULL, content BLOB NOT NULL,
        PRIMARY KEY(account,id));";

fn state() -> String {
    format!(
        r#"{{"id":"a","tenant":"t","owner":"alice","address":"alice@example.org","revision":1,"mail_modseq":2,"identity":{{"name":"","textSignature":"","htmlSignature":"","replyTo":null,"bcc":null}},"identity_revision":0,"vacation":{{"isEnabled":false,"fromDate":null,"toDate":null,"subject":null,"textBody":null,"htmlBody":null}},"vacation_revision":0,"quota_bytes":1073741824,"mailboxes":[{{"id":"inbox","name":"INBOX","role":"inbox","parent_id":null,"sort_order":0,"is_subscribed":true,"uid_next":2,"uid_validity":1}}],"messages":[{{"id":"e1","modseq":2,"mailboxes":{{"inbox":1}},"size":{},"keywords":[],"received_at":1}}]}}"#,
        BODY.len()
    )
}

fn create(path: &Path) -> Connection {
    let db = Connection::open(path).unwrap();
    db.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
    db.execute_batch(TABLES).unwrap();
    db
}

/// Legacy file with one account `a` holding message `e1` in INBOX at uid 1.
pub fn populated(path: &Path) {
    let db = create(path);
    db.execute(
        "INSERT INTO accounts(id,address,token,state) VALUES('a','alice@example.org',NULL,?1)",
        [state()],
    )
    .unwrap();
    db.execute(
        "INSERT INTO message_bodies(account,id,content) VALUES('a','e1',?1)",
        params![BODY],
    )
    .unwrap();
    db.execute(
        "INSERT INTO history_commits(account,previous,revision) VALUES('a',0,1)",
        [],
    )
    .unwrap();
}

/// Legacy file with the same tables and no accounts.
pub fn empty(path: &Path) {
    create(path);
}

/// Stamp a `converting` marker on a legacy file, as a conversion that stopped
/// after writing its DDL would have left behind.
pub fn mark_converting(path: &Path, backup: &str) {
    let db = Connection::open(path).unwrap();
    db.execute_batch(
        "CREATE TABLE schema_version(version INTEGER PRIMARY KEY, state TEXT NOT NULL,
            backup_path TEXT, backup_sha256 TEXT, converted_at_utc TEXT, operator TEXT);",
    )
    .unwrap();
    db.execute(
        "INSERT INTO schema_version(version,state,backup_path,backup_sha256) VALUES(2,'converting',?1,'00')",
        [backup],
    )
    .unwrap();
}

pub fn tls(dir: &Path) -> (PathBuf, PathBuf) {
    let cert = dir.join("cert.pem");
    let key = dir.join("key.pem");
    let rcgen::CertifiedKey {
        cert: issued,
        signing_key,
    } = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    std::fs::write(&cert, issued.pem()).unwrap();
    std::fs::write(&key, signing_key.serialize_pem()).unwrap();
    (cert, key)
}

/// Run the built binary to completion (bounded), with every listener on an
/// ephemeral port and no inherited outbound configuration.
pub fn mail_app<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary = option_env!("MAIL_APP_BINARY")
        .or(option_env!("CARGO_BIN_EXE_mail-app"))
        .expect("the build must provide the mail-app executable");
    let mut command = Command::new(binary);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for name in [
        "SMTP",
        "IMAP",
        "IMAP_STARTTLS",
        "SUBMISSION",
        "SUBMISSION_STARTTLS",
        "POP",
        "POP_STARTTLS",
        "HTTP",
    ] {
        command.env(format!("MAIL_{name}_LISTEN"), "127.0.0.1:0");
    }
    for (name, _) in std::env::vars_os() {
        let text = name.to_string_lossy();
        if text.starts_with("MAIL_RELAY_") || text.starts_with("MAIL_MX_") {
            command.env_remove(&name);
        }
    }
    let mut child = command.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("mail-app did not exit within 60 seconds");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

pub fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
