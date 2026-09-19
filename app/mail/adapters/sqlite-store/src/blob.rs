//! Bodies: content-addressed `blob_content` versions, per-account links that
//! authorize reads, command-scoped reservations that keep an unlinked body
//! alive, and tombstones so a swept `(hash, version_id)` never resolves again.
use super::{SqliteStore, access, storage};
use mail_api::{BlobStore, UPLOAD_RESERVATION_SECS};
use mail_kernel::{BlobRef, Error, MAX_MESSAGE_BYTES, valid_identifier};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};

pub(super) const DDL: &str = "
    CREATE TABLE IF NOT EXISTS blob_content(hash TEXT NOT NULL,version_id TEXT NOT NULL,size INTEGER NOT NULL,content BLOB NOT NULL,PRIMARY KEY(hash,version_id));
    CREATE TABLE IF NOT EXISTS blob_tombstones(hash TEXT NOT NULL,version_id TEXT NOT NULL,PRIMARY KEY(hash,version_id));
    CREATE TABLE IF NOT EXISTS blob_links(account TEXT NOT NULL,owner TEXT NOT NULL,hash TEXT NOT NULL,version_id TEXT NOT NULL,PRIMARY KEY(account,owner));
    CREATE INDEX IF NOT EXISTS blob_links_content ON blob_links(hash,version_id);
    CREATE INDEX IF NOT EXISTS blob_links_hash ON blob_links(account,hash);
    CREATE TABLE IF NOT EXISTS blob_reservations(account TEXT NOT NULL,scope TEXT NOT NULL,expires_at INTEGER NOT NULL,PRIMARY KEY(account,scope));
    CREATE TABLE IF NOT EXISTS blob_reserved(account TEXT NOT NULL,scope TEXT NOT NULL,hash TEXT NOT NULL,version_id TEXT NOT NULL,PRIMARY KEY(account,scope,hash,version_id));
    CREATE INDEX IF NOT EXISTS blob_reserved_content ON blob_reserved(hash,version_id);";

fn now(db: &Connection) -> Result<i64, Error> {
    db.query_row("SELECT unixepoch()", [], |r| r.get(0))
        .map_err(storage)
}

/// Persist inside an open transaction: reuse the live version of `hash` or
/// create the next one; reserve it under `scope` for `account`.
pub(super) fn persist_tx(
    tx: &Connection,
    account: &str,
    scope: &str,
    raw: &[u8],
    ttl_secs: i64,
) -> Result<BlobRef, Error> {
    if raw.len() > MAX_MESSAGE_BYTES || scope.is_empty() || scope.len() > 256 {
        return Err(Error::OverQuota);
    }
    let owner = access::load(tx, account)?;
    let hash = format!("{:x}", Sha256::digest(raw));
    let now = now(tx)?;
    let (count, bytes): (u64, u64) = tx.query_row(
        "SELECT count(*),coalesce(sum(c.size),0) FROM (SELECT DISTINCT r.hash,r.version_id FROM blob_reserved r JOIN blob_reservations s ON s.account=r.account AND s.scope=r.scope WHERE r.account=?1 AND s.expires_at>?2 AND r.hash!=?3 AND NOT EXISTS(SELECT 1 FROM blob_links l WHERE l.account=r.account AND l.hash=r.hash AND l.version_id=r.version_id)) x JOIN blob_content c ON c.hash=x.hash AND c.version_id=x.version_id",
        params![account, now, hash], |r| Ok((r.get(0)?, r.get(1)?))
    ).map_err(storage)?;
    if count >= 1000
        || bytes
            .checked_add(raw.len() as u64)
            .is_none_or(|n| n > owner.quota_bytes as u64)
    {
        return Err(Error::OverQuota);
    }
    let live: Option<String> = tx
        .query_row(
            "SELECT version_id FROM blob_content WHERE hash=?1 ORDER BY CAST(version_id AS INTEGER) DESC LIMIT 1",
            [&hash],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?;
    let version_id = match live {
        Some(version) => version,
        None => {
            let next: i64 = tx
                .query_row(
                    "SELECT coalesce(max(CAST(version_id AS INTEGER)),0)+1 FROM blob_tombstones WHERE hash=?1",
                    [&hash],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            tx.execute(
                "INSERT INTO blob_content(hash,version_id,size,content) VALUES(?1,?2,?3,?4)",
                params![hash, next.to_string(), raw.len() as i64, raw],
            )
            .map_err(storage)?;
            next.to_string()
        }
    };
    tx.execute(
        "INSERT INTO blob_reservations(account,scope,expires_at) VALUES(?1,?2,?3) ON CONFLICT(account,scope) DO UPDATE SET expires_at=max(expires_at,excluded.expires_at)",
        params![account, scope, now + ttl_secs],
    )
    .map_err(storage)?;
    tx.execute(
        "INSERT OR IGNORE INTO blob_reserved(account,scope,hash,version_id) VALUES(?1,?2,?3,?4)",
        params![account, scope, hash, version_id],
    )
    .map_err(storage)?;
    Ok(BlobRef {
        hash,
        version_id,
        size: raw.len(),
    })
}

/// The commit rule: a reference is accepted iff no tombstone exists for it
/// and the account holds a reservation row or a live link; the link `owner`
/// is written and the bytes returned for header parsing. Time is never read.
pub(super) fn reference(
    tx: &Connection,
    account: &str,
    blob: &BlobRef,
    owner: &str,
) -> Result<Vec<u8>, Error> {
    let admitted: bool = tx.query_row(
        "SELECT NOT EXISTS(SELECT 1 FROM blob_tombstones WHERE hash=?2 AND version_id=?3) AND (EXISTS(SELECT 1 FROM blob_reserved WHERE account=?1 AND hash=?2 AND version_id=?3) OR EXISTS(SELECT 1 FROM blob_links WHERE account=?1 AND hash=?2 AND version_id=?3))",
        params![account, blob.hash, blob.version_id], |r| r.get(0)
    ).map_err(storage)?;
    if !admitted {
        return Err(Error::NotFound);
    }
    let (size, content): (i64, Vec<u8>) = tx
        .query_row(
            "SELECT size,content FROM blob_content WHERE hash=?1 AND version_id=?2",
            params![blob.hash, blob.version_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(storage)?
        .ok_or(Error::NotFound)?;
    if size != blob.size as i64 {
        return Err(Error::Invalid);
    }
    tx.execute(
        "INSERT INTO blob_links(account,owner,hash,version_id) VALUES(?1,?2,?3,?4)",
        params![account, owner, blob.hash, blob.version_id],
    )
    .map_err(storage)?;
    Ok(content)
}

/// Bytes of the version `account` links or holds live; `NotFound` otherwise.
fn read_tx(db: &Connection, account: &str, blob: &BlobRef) -> Result<Vec<u8>, Error> {
    let owned: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM blob_links WHERE account=?1 AND hash=?2 AND version_id=?3) OR EXISTS(SELECT 1 FROM blob_reserved r JOIN blob_reservations s ON s.account=r.account AND s.scope=r.scope WHERE r.account=?1 AND r.hash=?2 AND r.version_id=?3 AND s.expires_at>?4)",
        params![account, blob.hash, blob.version_id, now(db)?], |r| r.get(0)
    ).map_err(storage)?;
    if !owned {
        return Err(Error::NotFound);
    }
    db.query_row(
        "SELECT content FROM blob_content WHERE hash=?1 AND version_id=?2",
        params![blob.hash, blob.version_id],
        |r| r.get(0),
    )
    .optional()
    .map_err(storage)?
    .ok_or(Error::NotFound)
}

/// A client-visible `b<hash>` id resolves to the newest version the account
/// links or holds live; a message id resolves through its link.
pub(super) fn get(db: &Connection, account: &str, id: &str) -> Result<Vec<u8>, Error> {
    if !valid_identifier(id) {
        return Err(Error::NotFound);
    }
    let blob = if let Some(hash) = id.strip_prefix('b') {
        db.query_row(
            "SELECT hash,version_id FROM (SELECT hash,version_id FROM blob_links WHERE account=?1 AND hash=?2 UNION SELECT r.hash,r.version_id FROM blob_reserved r JOIN blob_reservations s ON s.account=r.account AND s.scope=r.scope WHERE r.account=?1 AND r.hash=?2 AND s.expires_at>?3) ORDER BY CAST(version_id AS INTEGER) DESC LIMIT 1",
            params![account, hash, now(db)?],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
    } else {
        db.query_row(
            "SELECT hash,version_id FROM blob_links WHERE account=?1 AND owner=?2",
            params![account, format!("message:{id}")],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
    }
    .optional()
    .map_err(storage)?
    .ok_or(Error::NotFound)?;
    read_tx(
        db,
        account,
        &BlobRef {
            hash: blob.0,
            version_id: blob.1,
            size: 0,
        },
    )
}

impl BlobStore for SqliteStore {
    fn persist(
        &self,
        account: &str,
        scope: &str,
        raw: &[u8],
        ttl_secs: i64,
    ) -> Result<BlobRef, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let blob = persist_tx(&tx, account, scope, raw, ttl_secs)?;
        tx.commit().map_err(storage)?;
        Ok(blob)
    }

    fn renew(&self, account: &str, scope: &str, ttl_secs: i64) -> Result<(), Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let renewed = db
            .execute(
                "UPDATE blob_reservations SET expires_at=max(expires_at,?3) WHERE account=?1 AND scope=?2",
                params![account, scope, now(&db)? + ttl_secs],
            )
            .map_err(storage)?;
        (renewed == 1).then_some(()).ok_or(Error::NotFound)
    }

    fn read(&self, account: &str, blob: &BlobRef) -> Result<Vec<u8>, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        read_tx(&db, account, blob)
    }

    fn orphan_sweep(&self, now: i64, limit: usize) -> Result<usize, Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut orphans = tx.prepare(
            "SELECT c.hash,c.version_id FROM blob_content c WHERE NOT EXISTS(SELECT 1 FROM blob_links l WHERE l.hash=c.hash AND l.version_id=c.version_id) AND NOT EXISTS(SELECT 1 FROM blob_reserved r JOIN blob_reservations s ON s.account=r.account AND s.scope=r.scope WHERE r.hash=c.hash AND r.version_id=c.version_id AND s.expires_at>?1) ORDER BY c.hash,c.version_id LIMIT ?2",
        ).map_err(storage)?;
        let swept: Vec<(String, String)> = orphans
            .query_map(params![now, limit as i64], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(storage)?
            .collect::<Result<_, _>>()
            .map_err(storage)?;
        drop(orphans);
        for (hash, version_id) in &swept {
            tx.execute(
                "DELETE FROM blob_content WHERE hash=?1 AND version_id=?2",
                params![hash, version_id],
            )
            .map_err(storage)?;
            tx.execute(
                "INSERT OR IGNORE INTO blob_tombstones(hash,version_id) VALUES(?1,?2)",
                params![hash, version_id],
            )
            .map_err(storage)?;
            tx.execute(
                "DELETE FROM blob_reserved WHERE hash=?1 AND version_id=?2",
                params![hash, version_id],
            )
            .map_err(storage)?;
        }
        tx.execute(
            "DELETE FROM blob_reserved WHERE (account,scope) IN (SELECT account,scope FROM blob_reservations WHERE expires_at<=?1)",
            [now],
        )
        .map_err(storage)?;
        tx.execute("DELETE FROM blob_reservations WHERE expires_at<=?1", [now])
            .map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(swept.len())
    }
}

/// JMAP upload: a 24 h reservation under the body's own scope.
pub(super) fn put(store: &SqliteStore, account: &str, raw: &[u8]) -> Result<String, Error> {
    let hash = format!("{:x}", Sha256::digest(raw));
    store.persist(
        account,
        &format!("upload:{hash}"),
        raw,
        UPLOAD_RESERVATION_SECS,
    )?;
    Ok(format!("b{hash}"))
}
