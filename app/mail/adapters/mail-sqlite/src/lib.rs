#![forbid(unsafe_code)]
mod access;
mod blob;
mod compatibility;
mod content;
mod delivery;
mod events;
mod failures;
mod journal;
mod metadata;
mod outbound;
mod queue;
mod schema;
mod submission;
mod submission_history;
mod submission_query;
mod submission_schema;
mod submission_store;
mod threads;
mod transfer;
mod vacation;

use mail_api::{AccountInfo, Store};
use mail_kernel::{Account, Command, Error};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Mutex, time::Duration};

pub struct SqliteStore {
    connection: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let db = Connection::open(path).map_err(storage)?;
        db.busy_timeout(Duration::from_secs(5)).map_err(storage)?;
        schema::initialize(&db)?;
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
        let state = serde_json::to_string(&account).map_err(|_| Error::Invalid)?;
        self.connection
            .lock()
            .map_err(|_| Error::Unavailable)?
            .execute(
                "INSERT INTO accounts(id,address,token,state) VALUES(?1,?2,?3,?4)",
                params![
                    account.id,
                    account.address.to_ascii_lowercase(),
                    Sha256::digest(token.as_bytes()).as_slice(),
                    state
                ],
            )
            .map_err(storage)?;
        Ok(())
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
        _ => Error::Unavailable,
    }
}

fn load(db: &Connection, id: &str) -> Result<Account, Error> {
    compatibility::decode(&content::load(db, id)?)
}

fn save(db: &Connection, account: &mut Account) -> Result<(), Error> {
    let before = content::prepare_save(db, account)?;
    let indexed = metadata::revision(db, &account.id)?.is_some();
    journal::record(db, &before, account)?;
    let state = serde_json::to_string(account).map_err(|_| Error::Unavailable)?;
    db.execute(
        "UPDATE accounts SET state=?2 WHERE id=?1",
        params![account.id, state],
    )
    .map_err(storage)?;
    metadata::record(db, &before, account, indexed)?;
    threads::seal(db, &account.id)?;
    db.execute(
        "INSERT INTO events(tenant,account,revision,observed_at_ms) VALUES(?1,?2,?3,unixepoch()*1000)",
        params![account.tenant, account.id, account.revision],
    )
    .map_err(storage)?;
    Ok(())
}

impl Store for SqliteStore {
    fn messages(&self, account: &str, ids: &[String]) -> Result<mail_api::MessageSelection, Error> {
        metadata::selected(self, account, ids)
    }
    fn account_info(&self, id: &str) -> Result<AccountInfo, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        access::load(&db, id)
    }
    fn mailbox_changes(
        &self,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<mail_api::MailboxChange>, Error> {
        journal::read(self, account, since, until, journal::Kind::Mailbox)
    }
    fn message_changes(
        &self,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<mail_api::MessageChange>, Error> {
        journal::read(self, account, since, until, journal::Kind::Message)
    }
    fn message_changes_after(
        &self,
        account: &str,
        since: u64,
        until: u64,
    ) -> Result<Vec<mail_api::MessageChange>, Error> {
        journal::read(self, account, since, until, journal::Kind::MessageAfter)
    }
    fn put_blob(&self, account: &str, raw: &[u8]) -> Result<String, Error> {
        blob::put(self, account, raw)
    }

    fn blob(&self, account: &str, id: &str) -> Result<Vec<u8>, Error> {
        blob::get(self, account, id)
    }

    fn account(&self, id: &str) -> Result<Account, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        load(&db, id)
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

    fn execute(&self, id: &str, revision: u64, commands: Vec<Command>) -> Result<Account, Error> {
        // ponytail: metadata snapshots still cost O(message count); indexed metadata
        // queries are required before large mailbox qualification. Bodies are separate.
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let transaction = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut account = load(&transaction, id)?;
        if account.revision != revision {
            return Err(Error::Conflict);
        }
        for command in commands {
            content::apply(&transaction, &mut account, command)?;
        }
        if account.revision != revision {
            save(&transaction, &mut account)?;
        }
        transaction.commit().map_err(storage)?;
        Ok(account)
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
