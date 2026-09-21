use super::{SqliteStore, mutation, storage, threads};
use mail_api::Precondition;
use mail_kernel::{Command, Error, MAX_MESSAGE_BYTES};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Append an SMTP-sourced message to INBOX unless an exact Message-ID /
/// References-set match is already linked to INBOX or Junk.
fn ingest(
    db: &Connection,
    enabled: &[mail_api::Consumer],
    id: &str,
    raw: &[u8],
    received_at: i64,
) -> Result<Option<String>, Error> {
    let (mut batch, _) = mutation::Batch::open(db, id, Precondition::Observed(0), &[], enabled)?;
    let refs = threads::references(raw)?;
    // A duplicate writes nothing: the body is persisted only once it is known
    // to be linked by this same transaction, so a ttl of 0 is enough.
    if !threads::duplicate(db, &batch.account, "inbox", &refs)? {
        let blob = super::blob::persist_tx(
            db,
            id,
            &format!("deliver:{:x}", Sha256::digest(raw)),
            raw,
            0,
        )?;
        batch.apply(
            db,
            Command::Append {
                mailboxes: vec!["inbox".into()],
                blob,
                keywords: vec![],
                received_at,
            },
        )?;
    }
    let (execution, account) = batch.commit(db)?;
    super::vacation::maybe_reply(db, &account, received_at, raw)?;
    Ok(execution.ids.into_iter().next())
}

pub(super) fn once(
    store: &SqliteStore,
    id: &str,
    key: &str,
    raw: &[u8],
    received_at: i64,
) -> Result<Option<String>, Error> {
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
            Ok(None)
        } else {
            Err(Error::Conflict)
        };
    }
    let minted = ingest(&tx, &store.enabled, id, raw, received_at)?;
    tx.execute(
        "INSERT INTO delivery_receipts(account,id,digest) VALUES(?1,?2,?3)",
        params![id, key, digest.as_slice()],
    )
    .map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(minted)
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
        let received_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::Unavailable)?
            .as_secs()
            .try_into()
            .map_err(|_| Error::Unavailable)?;
        for id in ids {
            ingest(&transaction, &self.enabled, &id, raw, received_at)?;
        }
        transaction.commit().map_err(storage)
    }
}
