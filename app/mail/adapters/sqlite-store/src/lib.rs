#![forbid(unsafe_code)]
mod access;
mod blob;
mod content;
#[cfg(feature = "contract")]
pub mod contract;
pub mod convert;
mod delivery;
mod events;
mod failures;
mod history;
mod mutation;
mod outbound;
mod queue;
mod records;
mod schema;
mod submission;
mod submission_history;
mod submission_query;
mod submission_schema;
mod submission_store;
mod threads;
mod transfer;
mod vacation;

use mail_api::{
    AccountInfo, Execution, HistoryPage, MailboxSelection, MetadataStore, Precondition,
};
use mail_kernel::{Account, Command, Error};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
pub use schema::{Refusal, SCHEMA_VERSION, SchemaState, inspect};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Mutex, time::Duration};

pub struct SqliteStore {
    connection: Mutex<Connection>,
}

/// Why `open` did not return a store.
#[derive(Debug)]
pub enum OpenError {
    Refused(Refusal),
    Storage(Error),
}
impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(refusal) => refusal.fmt(f),
            Self::Storage(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for OpenError {}
impl From<Error> for OpenError {
    fn from(error: Error) -> Self {
        Self::Storage(error)
    }
}

impl SqliteStore {
    /// Opens a database at exactly this binary's schema version; a fresh file
    /// is initialized, anything else is refused with the operator's next step.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, OpenError> {
        let db = Connection::open(path).map_err(storage)?;
        db.busy_timeout(Duration::from_secs(5)).map_err(storage)?;
        schema::initialize(&db)?.map_err(OpenError::Refused)?;
        submission_schema::initialize(&db)?;
        Ok(Self {
            connection: Mutex::new(db),
        })
    }

    /// Local administration API; listeners do not expose provisioning or raw tokens.
    pub fn provision(&self, account: Account, token: &str) -> Result<(), Error> {
        if !account.messages.is_empty() {
            return Err(Error::Invalid);
        }
        if token.len() < 32 || token.len() > 4096 || token.chars().any(char::is_control) {
            return Err(Error::Invalid);
        }
        Account::new(
            &account.id,
            &account.tenant,
            &account.owner,
            &account.address,
        )?;
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM accounts WHERE id=?1 OR address=?2)",
                params![account.id, account.address.to_ascii_lowercase()],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if exists {
            return Err(Error::Conflict);
        }
        records::upsert_header(&tx, &account)?;
        tx.execute(
            "UPDATE accounts SET token=?2 WHERE id=?1",
            params![account.id, Sha256::digest(token.as_bytes()).as_slice()],
        )
        .map_err(storage)?;
        for mailbox in &account.mailboxes {
            records::upsert_mailbox(&tx, &account.id, mailbox)?;
        }
        tx.commit().map_err(storage)
    }

    pub fn revoke(&self, id: &str) -> Result<(), Error> {
        self.connection
            .lock()
            .map_err(|_| Error::Unavailable)?
            .execute("UPDATE accounts SET token=NULL WHERE id=?1", [id])
            .map_err(storage)?;
        Ok(())
    }
}

fn storage(error: rusqlite::Error) -> Error {
    match error {
        rusqlite::Error::SqliteFailure(e, _)
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            Error::Conflict
        }
        rusqlite::Error::SqliteFailure(e, _)
            if matches!(
                e.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            ) =>
        {
            Error::Busy
        }
        _ => Error::Unavailable,
    }
}

impl MetadataStore for SqliteStore {
    fn messages(&self, account: &str, ids: &[String]) -> Result<mail_api::MessageSelection, Error> {
        if ids.len() > 256 {
            return Err(Error::OverQuota);
        }
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db.transaction().map_err(storage)?;
        let revision: u64 = tx
            .query_row(
                "SELECT revision FROM accounts WHERE id=?1",
                [account],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?
            .ok_or(Error::NotFound)?;
        let ids = ids.iter().map(String::as_str).collect();
        let messages = records::by_ids(&tx, account, &ids)?;
        Ok(mail_api::MessageSelection { revision, messages })
    }
    fn mailbox_uids(&self, account: &str, mailbox: &str) -> Result<MailboxSelection, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db.transaction().map_err(storage)?;
        let (revision, uid_validity, uid_next, highest_modseq): (u64, u32, u32, u64) = tx
            .query_row(
                "SELECT a.revision,m.uid_validity,m.uid_next,m.highest_modseq FROM accounts a JOIN mailboxes m ON m.account=a.id WHERE a.id=?1 AND m.id=?2",
                params![account, mailbox],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()
            .map_err(storage)?
            .ok_or(Error::NotFound)?;
        let mut query = tx
            .prepare("SELECT uid,message FROM message_mailboxes WHERE account=?1 AND mailbox=?2 ORDER BY uid")
            .map_err(storage)?;
        let uids = query
            .query_map(params![account, mailbox], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(storage)?
            .collect::<Result<_, _>>()
            .map_err(storage)?;
        Ok(MailboxSelection {
            revision,
            uid_validity,
            uid_next,
            highest_modseq,
            uids,
        })
    }
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        access::load(&db, id)
    }
    fn history(&self, account: &str, since: u64, limit: usize) -> Result<HistoryPage, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db.transaction().map_err(storage)?;
        history::page(&tx, account, since, limit)
    }
    fn compact_history(
        &self,
        account: &str,
        now: i64,
        policy: mail_kernel::RetentionPolicy,
        cursors: &[(mail_api::Consumer, u64)],
    ) -> Result<mail_kernel::Retention, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let retention = history::compact(&tx, account, now, policy, cursors)?;
        tx.commit().map_err(storage)?;
        Ok(retention)
    }
    fn put_blob(&self, account: &str, raw: &[u8]) -> Result<String, Error> {
        blob::put(self, account, raw)
    }
    fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error> {
        blob::get(self, account, id)
    }
    fn account(&self, id: &str) -> Result<Account, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db.transaction().map_err(storage)?;
        records::projection(&tx, id)
    }
    fn resolve(&self, address: &str) -> Result<String, Error> {
        self.connection
            .lock()
            .map_err(|_| Error::Unavailable)?
            .query_row(
                "SELECT id FROM accounts WHERE address=?1",
                [address.to_ascii_lowercase()],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage)?
            .ok_or(Error::NotFound)
    }
    fn execute(
        &self,
        id: &str,
        precondition: Precondition,
        commands: Vec<Command>,
    ) -> Result<Execution, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let (execution, _) = mutation::run(&tx, id, precondition, commands)?;
        tx.commit().map_err(storage)?;
        Ok(execution)
    }
    fn deliver_once(
        &self,
        account: &str,
        key: &str,
        raw: &[u8],
        received_at: i64,
    ) -> Result<(), Error> {
        delivery::once(self, account, key, raw, received_at)
    }
}
