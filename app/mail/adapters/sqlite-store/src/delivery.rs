use super::{SqliteStore, content, load, save, storage};
use mail_kernel::{Command, Error, MAX_MESSAGE_BYTES};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub(super) fn once(
    store: &SqliteStore,
    id: &str,
    key: &str,
    raw: &[u8],
    received_at: i64,
) -> Result<(), Error> {
    if key.is_empty() || key.len() > 512 || key.chars().any(char::is_control) {
        return Err(Error::Invalid);
    }
    if raw.len() > MAX_MESSAGE_BYTES {
        return Err(Error::OverQuota);
    }
    let mut hash = Sha256::new();
    hash.update(received_at.to_be_bytes());
    hash.update(raw);
    let digest = hash.finalize();
    let mut db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let previous: Option<Vec<u8>> = tx
        .query_row(
            "SELECT digest FROM delivery_receipts WHERE account=?1 AND id=?2",
            params![id, key],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?;
    if let Some(previous) = previous {
        return if previous == digest.as_slice() {
            Ok(())
        } else {
            Err(Error::Conflict)
        };
    }
    let mut account = load(&tx, id)?;
    content::apply(
        &tx,
        &mut account,
        Command::Append {
            mailboxes: vec!["inbox".into()],
            raw: raw.to_vec(),
            keywords: vec![],
            received_at,
        },
    )?;
    save(&tx, &mut account)?;
    super::vacation::maybe_reply(&tx, &account, received_at, raw)?;
    tx.execute(
        "INSERT INTO delivery_receipts(account,id,digest) VALUES(?1,?2,?3)",
        params![id, key, digest.as_slice()],
    )
    .map_err(storage)?;
    tx.commit().map_err(storage)
}

impl SqliteStore {
    pub fn deliver(&self, recipients: &[String], raw: &[u8]) -> Result<(), Error> {
        if recipients.is_empty() || recipients.len() > 100 {
            return Err(Error::Invalid);
        }
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let transaction = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut ids = BTreeSet::new();
        for address in recipients {
            let id: String = transaction
                .query_row(
                    "SELECT id FROM accounts WHERE address=?1",
                    [address.to_ascii_lowercase()],
                    |row| row.get(0),
                )
                .optional()
                .map_err(storage)?
                .ok_or(Error::NotFound)?;
            ids.insert(id);
        }
        for id in ids {
            let mut account = load(&transaction, &id)?;
            let received_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| Error::Unavailable)?
                .as_secs()
                .try_into()
                .map_err(|_| Error::Unavailable)?;
            content::apply(
                &transaction,
                &mut account,
                Command::Append {
                    mailboxes: vec!["inbox".into()],
                    received_at,
                    raw: raw.to_vec(),
                    keywords: vec![],
                },
            )?;
            save(&transaction, &mut account)?;
            super::vacation::maybe_reply(&transaction, &account, received_at, raw)?;
        }
        transaction.commit().map_err(storage)
    }
}
