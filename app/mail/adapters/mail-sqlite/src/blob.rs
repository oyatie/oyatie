use super::{SqliteStore, access, storage};
use mail_kernel::{Error, MAX_MESSAGE_BYTES, valid_identifier};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};

pub(super) fn put(store: &SqliteStore, account: &str, raw: &[u8]) -> Result<String, Error> {
    if raw.len() > MAX_MESSAGE_BYTES {
        return Err(Error::OverQuota);
    }
    let id = format!("b{:x}", Sha256::digest(raw));
    let mut db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    let tx = db
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(storage)?;
    let owner = access::load(&tx, account)?;
    tx.execute(
        "DELETE FROM blobs WHERE account=?1 AND expires_at <= unixepoch()",
        [account],
    )
    .map_err(storage)?;
    let (count, bytes): (u64, u64) = tx.query_row(
        "SELECT count(*), coalesce(sum(length(content)),0) FROM blobs WHERE account=?1 AND id!=?2",
        params![account, id], |r| Ok((r.get(0)?, r.get(1)?))
    ).map_err(storage)?;
    if count >= 1000
        || bytes
            .checked_add(raw.len() as u64)
            .is_none_or(|n| n > owner.quota_bytes as u64)
    {
        return Err(Error::OverQuota);
    }
    tx.execute(
        "INSERT INTO blobs(account,id,content,expires_at) VALUES(?1,?2,?3,unixepoch()+86400)
        ON CONFLICT(account,id) DO UPDATE SET expires_at=max(expires_at,excluded.expires_at)",
        params![account, id, raw],
    )
    .map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(id)
}

pub(super) fn get(store: &SqliteStore, account: &str, id: &str) -> Result<Vec<u8>, Error> {
    if !valid_identifier(id) {
        return Err(Error::NotFound);
    }
    let db = store.connection.lock().map_err(|_| Error::Unavailable)?;
    if id.starts_with('b') {
        db.query_row(
            "SELECT content FROM blobs WHERE account=?1 AND id=?2 AND expires_at>unixepoch()",
            params![account, id],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?
        .ok_or(Error::NotFound)
    } else {
        super::content::get(&db, account, id)
    }
}
