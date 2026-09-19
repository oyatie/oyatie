use crate::storage;
use mail_kernel::Error;
use rusqlite::{Connection, OptionalExtension};

/// Schema version this binary reads and writes. `serve` refuses a database
/// below it; a `converting` marker refuses every binary until `convert`
/// finishes.
pub const SCHEMA_VERSION: u64 = 3;

/// Relational metadata: account, mailbox, message and link records, history
/// rows keyed by revision, plus the unchanged content, queue and submission
/// tables. No triggers and no JSON columns beyond the two settings blobs.
pub(crate) const DDL: &str = "
    CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY,
        state TEXT NOT NULL CHECK(state IN ('converting','complete')),
        backup_path TEXT, backup_sha256 TEXT, converted_at_utc TEXT, operator TEXT);
    CREATE TABLE IF NOT EXISTS accounts (
        id TEXT PRIMARY KEY, address TEXT NOT NULL UNIQUE, token BLOB UNIQUE,
        tenant TEXT NOT NULL, owner TEXT NOT NULL,
        revision INTEGER NOT NULL, mail_modseq INTEGER NOT NULL DEFAULT 0,
        history_floor INTEGER NOT NULL DEFAULT 0,
        identity TEXT NOT NULL, identity_revision INTEGER NOT NULL DEFAULT 0,
        vacation TEXT NOT NULL, vacation_revision INTEGER NOT NULL DEFAULT 0,
        quota_bytes INTEGER NOT NULL CHECK(quota_bytes>=0),
        used_bytes INTEGER NOT NULL DEFAULT 0 CHECK(used_bytes>=0));
    CREATE INDEX IF NOT EXISTS account_credentials ON accounts(token,id);
    CREATE TABLE IF NOT EXISTS mailboxes (
        account TEXT NOT NULL, id TEXT NOT NULL, name TEXT NOT NULL, role TEXT, parent_id TEXT,
        sort_order INTEGER NOT NULL DEFAULT 0, is_subscribed INTEGER NOT NULL DEFAULT 1,
        uid_next INTEGER NOT NULL, uid_validity INTEGER NOT NULL,
        created_revision INTEGER NOT NULL DEFAULT 0, highest_modseq INTEGER NOT NULL DEFAULT 0,
        total_emails INTEGER NOT NULL DEFAULT 0, unread_emails INTEGER NOT NULL DEFAULT 0,
        size_bytes INTEGER NOT NULL DEFAULT 0, PRIMARY KEY(account,id));
    CREATE TABLE IF NOT EXISTS messages (
        account TEXT NOT NULL, id TEXT NOT NULL, modseq INTEGER NOT NULL,
        created_revision INTEGER NOT NULL, thread TEXT NOT NULL,
        email_identity TEXT, thread_identity TEXT, size INTEGER NOT NULL,
        received_at INTEGER NOT NULL, keywords TEXT NOT NULL DEFAULT '',
        PRIMARY KEY(account,id));
    CREATE INDEX IF NOT EXISTS message_threads ON messages(account,thread);
    CREATE TABLE IF NOT EXISTS message_mailboxes (
        account TEXT NOT NULL, mailbox TEXT NOT NULL, uid INTEGER NOT NULL, message TEXT NOT NULL,
        PRIMARY KEY(account,mailbox,uid));
    CREATE INDEX IF NOT EXISTS message_links ON message_mailboxes(account,message,mailbox);
    CREATE TABLE IF NOT EXISTS history_commits (
        account TEXT NOT NULL, revision INTEGER NOT NULL, committed_at INTEGER NOT NULL,
        PRIMARY KEY(account,revision));
    CREATE TABLE IF NOT EXISTS history (
        account TEXT NOT NULL, revision INTEGER NOT NULL, seq INTEGER NOT NULL,
        kind TEXT NOT NULL CHECK(kind IN ('added','removed','flags','thread','mailbox')),
        id TEXT NOT NULL, mailbox TEXT, uid INTEGER, thread TEXT,
        PRIMARY KEY(account,revision,seq));
    CREATE TABLE IF NOT EXISTS events (
        sequence INTEGER PRIMARY KEY AUTOINCREMENT,
        tenant TEXT NOT NULL, account TEXT NOT NULL, revision INTEGER NOT NULL,
        delivered INTEGER NOT NULL DEFAULT 0, observed_at_ms INTEGER NOT NULL DEFAULT 0,
        UNIQUE(account, revision));
    CREATE TABLE IF NOT EXISTS event_cursors(consumer TEXT PRIMARY KEY,sequence INTEGER NOT NULL CHECK(sequence>=0));
    CREATE TABLE IF NOT EXISTS queued_messages (
        id TEXT PRIMARY KEY,sender TEXT NOT NULL,content BLOB NOT NULL,size INTEGER NOT NULL,received_at INTEGER NOT NULL);
    CREATE TABLE IF NOT EXISTS delivery_jobs (
        message TEXT NOT NULL,account TEXT NOT NULL,address TEXT NOT NULL,
        next_attempt INTEGER NOT NULL,lease_until INTEGER NOT NULL DEFAULT 0,
        token TEXT,attempt INTEGER NOT NULL DEFAULT 0,last_error TEXT,PRIMARY KEY(message,account));
    CREATE INDEX IF NOT EXISTS delivery_due ON delivery_jobs(next_attempt,lease_until);
    CREATE INDEX IF NOT EXISTS delivery_account ON delivery_jobs(account,message);
    CREATE TABLE IF NOT EXISTS delivery_receipts (
        account TEXT NOT NULL,id TEXT NOT NULL,digest BLOB NOT NULL,PRIMARY KEY(account,id));
    CREATE TABLE IF NOT EXISTS thread_members(account TEXT NOT NULL,message TEXT NOT NULL,subject BLOB NOT NULL,thread TEXT NOT NULL,PRIMARY KEY(account,message));
    CREATE INDEX IF NOT EXISTS thread_groups ON thread_members(account,thread);
    CREATE TABLE IF NOT EXISTS thread_references(account TEXT NOT NULL,message TEXT NOT NULL,reference BLOB NOT NULL,PRIMARY KEY(account,message,reference));
    CREATE INDEX IF NOT EXISTS thread_lookup ON thread_references(account,reference,message);
    CREATE TABLE IF NOT EXISTS failed_delivery_messages (
        id TEXT PRIMARY KEY,sender TEXT NOT NULL,content BLOB NOT NULL,size INTEGER NOT NULL,received_at INTEGER NOT NULL);
    CREATE TABLE IF NOT EXISTS failed_delivery_jobs (
        message TEXT NOT NULL,account TEXT NOT NULL,address TEXT NOT NULL,failed_at INTEGER NOT NULL,reason TEXT NOT NULL,PRIMARY KEY(message,account));
    CREATE INDEX IF NOT EXISTS failed_deliveries_account ON failed_delivery_jobs(account,failed_at,message);
    CREATE TABLE IF NOT EXISTS submitted_messages (
        id TEXT PRIMARY KEY,account TEXT NOT NULL,sender TEXT NOT NULL,content BLOB NOT NULL,size INTEGER NOT NULL,received_at INTEGER NOT NULL);
    CREATE TABLE IF NOT EXISTS outbound_jobs (
        message TEXT NOT NULL,account TEXT NOT NULL,recipient TEXT NOT NULL,
        next_attempt INTEGER NOT NULL,lease_until INTEGER NOT NULL DEFAULT 0,
        token TEXT,attempt INTEGER NOT NULL DEFAULT 0,last_code INTEGER,PRIMARY KEY(message,recipient));
    CREATE INDEX IF NOT EXISTS outbound_due ON outbound_jobs(next_attempt,lease_until);
    CREATE INDEX IF NOT EXISTS outbound_account ON outbound_jobs(account,message);
    CREATE TABLE IF NOT EXISTS vacation_sent(account TEXT NOT NULL, sender TEXT NOT NULL, PRIMARY KEY(account,sender));";

/// What `schema_version` says about a database file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchemaState {
    /// No `schema_version` table: a `41ce37e61` database (version 1).
    Legacy,
    Converting {
        version: u64,
    },
    Complete {
        version: u64,
    },
}

pub fn inspect(db: &Connection) -> Result<SchemaState, Error> {
    let present: bool = db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_version')",
            [],
            |r| r.get(0),
        )
        .map_err(storage)?;
    if !present {
        return Ok(SchemaState::Legacy);
    }
    let row: Option<(u64, String)> = db
        .query_row(
            "SELECT version,state FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(storage)?;
    Ok(match row {
        None => SchemaState::Legacy,
        Some((version, state)) if state == "converting" => SchemaState::Converting { version },
        Some((version, _)) => SchemaState::Complete { version },
    })
}

/// Reason a database cannot be served by this binary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Run `mail-app convert`; `accounts` and `messages` size the estimate.
    BelowVersion {
        found: u64,
        accounts: u64,
        messages: u64,
    },
    Converting {
        version: u64,
    },
    AboveVersion {
        found: u64,
    },
}
impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BelowVersion {
                found,
                accounts,
                messages,
            } => write!(
                f,
                "database schema version {found} is below this binary's {SCHEMA_VERSION}; run `mail-app convert DATABASE --backup-verified PATH` first (estimated {} for {accounts} accounts, {messages} messages)",
                crate::convert::estimate(*accounts, *messages)
            ),
            Self::Converting { version } => write!(
                f,
                "database conversion to schema version {version} is incomplete (`converting` marker present); finish or restore from the verified backup"
            ),
            Self::AboveVersion { found } => write!(
                f,
                "database schema version {found} is newer than this binary's {SCHEMA_VERSION}"
            ),
        }
    }
}
impl std::error::Error for Refusal {}

/// A fresh database gets the current schema; an existing one is only served
/// when its recorded version is exactly this binary's and complete.
pub(super) fn initialize(db: &Connection) -> Result<Result<(), Refusal>, Error> {
    db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;")
        .map_err(storage)?;
    match inspect(db)? {
        SchemaState::Legacy => {
            let fresh: bool = db
                .query_row(
                    "SELECT NOT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='accounts')",
                    [],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            if !fresh {
                return Ok(Err(Refusal::BelowVersion {
                    found: 1,
                    accounts: count(db, "SELECT count(*) FROM accounts")?,
                    messages: count(db, "SELECT count(*) FROM message_bodies")?,
                }));
            }
            db.execute_batch(&format!(
                "BEGIN IMMEDIATE; {DDL} {}
                 INSERT INTO schema_version(version,state,converted_at_utc) VALUES({SCHEMA_VERSION},'complete',strftime('%Y-%m-%dT%H:%M:%SZ','now'));
                 COMMIT",
                super::blob::DDL
            ))
            .map_err(storage)?;
            Ok(Ok(()))
        }
        SchemaState::Converting { version } => Ok(Err(Refusal::Converting { version })),
        SchemaState::Complete { version } if version < SCHEMA_VERSION => {
            Ok(Err(Refusal::BelowVersion {
                found: version,
                accounts: count(db, "SELECT count(*) FROM accounts")?,
                messages: count(db, "SELECT count(*) FROM messages")?,
            }))
        }
        SchemaState::Complete { version } if version > SCHEMA_VERSION => {
            Ok(Err(Refusal::AboveVersion { found: version }))
        }
        SchemaState::Complete { .. } => {
            db.execute_batch(&format!("{DDL}{}", super::blob::DDL))
                .map_err(storage)?;
            Ok(Ok(()))
        }
    }
}

fn count(db: &Connection, sql: &str) -> Result<u64, Error> {
    db.query_row(sql, [], |r| r.get(0))
        .optional()
        .map_err(storage)
        .map(|n| n.unwrap_or(0))
}
