//! Backup verification: a file copy of a WAL database without its `-wal`
//! passes an id-set check while missing every commit since the last
//! checkpoint, so per-account heads are compared too.
use crate::{schema, storage};
use mail_kernel::Error;
use rusqlite::{Connection, OpenFlags};
use std::collections::BTreeMap;
use std::path::Path;

/// Why a backup was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackupCheck {
    SamePath,
    Unreadable,
    /// Not a legacy database, or a different recorded schema than the source.
    Shape,
    AccountSet,
    /// `(account, source head, backup head)` of the first mismatch.
    HistoryHead(String, Option<u64>, Option<u64>),
    SubmissionHead(String, Option<u64>, Option<u64>),
}
impl std::fmt::Display for BackupCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SamePath => write!(f, "backup path resolves to the database itself"),
            Self::Unreadable => write!(f, "backup is not a readable SQLite database"),
            Self::Shape => write!(
                f,
                "backup schema does not match the source (expected a legacy database)"
            ),
            Self::AccountSet => write!(f, "backup account id set differs from the source"),
            Self::HistoryHead(a, s, b) => write!(
                f,
                "backup history head for account {a} is {b:?}, source has {s:?} (copy taken without the -wal file?)"
            ),
            Self::SubmissionHead(a, s, b) => write!(
                f,
                "backup submission head for account {a} is {b:?}, source has {s:?}"
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Heads {
    pub(super) accounts: Vec<String>,
    pub(super) history: BTreeMap<String, Option<u64>>,
    pub(super) submission: BTreeMap<String, Option<u64>>,
}

pub(super) fn heads(db: &Connection) -> Result<Heads, Error> {
    let table = |name: &str| -> Result<bool, Error> {
        db.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
            [name],
            |r| r.get(0),
        )
        .map_err(storage)
    };
    let source = if table("legacy_accounts")? {
        "legacy_accounts"
    } else {
        "accounts"
    };
    let mut query = db
        .prepare(&format!("SELECT id FROM {source} ORDER BY id"))
        .map_err(storage)?;
    let accounts: Vec<String> = query
        .query_map([], |r| r.get(0))
        .map_err(storage)?
        .collect::<Result<_, _>>()
        .map_err(storage)?;
    let mut history = BTreeMap::new();
    let mut submission = BTreeMap::new();
    for account in &accounts {
        let head: Option<u64> = if table("history_commits")? {
            db.query_row(
                "SELECT max(revision) FROM history_commits WHERE account=?1",
                [account],
                |r| r.get(0),
            )
            .map_err(storage)?
        } else {
            None
        };
        history.insert(account.clone(), head);
        let head: Option<u64> = if table("submission_heads")? {
            db.query_row(
                "SELECT max(revision) FROM submission_heads WHERE account=?1",
                [account],
                |r| r.get(0),
            )
            .map_err(storage)?
        } else {
            None
        };
        submission.insert(account.clone(), head);
    }
    Ok(Heads {
        accounts,
        history,
        submission,
    })
}

/// Verify `backup` against the source connection (already holding the lock).
pub(super) fn backup(
    source: &Connection,
    database: &Path,
    backup: &Path,
) -> Result<Result<(), BackupCheck>, Error> {
    let same = match (
        std::fs::canonicalize(database),
        std::fs::canonicalize(backup),
    ) {
        (Ok(a), Ok(b)) => a == b,
        (_, Err(_)) => return Ok(Err(BackupCheck::Unreadable)),
        _ => false,
    };
    if same {
        return Ok(Err(BackupCheck::SamePath));
    }
    let Ok(copy) = Connection::open_with_flags(backup, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
        return Ok(Err(BackupCheck::Unreadable));
    };
    if schema::inspect(&copy).is_err() {
        return Ok(Err(BackupCheck::Unreadable));
    }
    let expected = schema::inspect(source)?;
    let actual = schema::inspect(&copy)?;
    let legacy_pair = matches!(
        (&expected, &actual),
        (schema::SchemaState::Legacy, schema::SchemaState::Legacy)
            | (
                schema::SchemaState::Converting { .. },
                schema::SchemaState::Legacy
            )
    );
    if !legacy_pair && expected != actual {
        return Ok(Err(BackupCheck::Shape));
    }
    if matches!(expected, schema::SchemaState::Converting { .. }) {
        // Resuming: heads were verified before the marker was written and the
        // source has already moved; the recorded path must match instead.
        return Ok(Ok(()));
    }
    let Ok(theirs) = heads(&copy) else {
        return Ok(Err(BackupCheck::Unreadable));
    };
    let ours = heads(source)?;
    if ours.accounts != theirs.accounts {
        return Ok(Err(BackupCheck::AccountSet));
    }
    for account in &ours.accounts {
        if ours.history[account] != theirs.history[account] {
            return Ok(Err(BackupCheck::HistoryHead(
                account.clone(),
                ours.history[account],
                theirs.history[account],
            )));
        }
        if ours.submission[account] != theirs.submission[account] {
            return Ok(Err(BackupCheck::SubmissionHead(
                account.clone(),
                ours.submission[account],
                theirs.submission[account],
            )));
        }
    }
    Ok(Ok(()))
}
