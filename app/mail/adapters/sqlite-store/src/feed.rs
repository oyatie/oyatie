//! `ChangeFeed`: a cursor and a dirty key per `(consumer, tenant, account)`.
//! The dirty key is written by the change transaction for every enabled
//! consumer; it is cleared only by an acknowledgement that reads the tail in
//! the same transaction. Poison is kept, never skipped or falsely acked, and
//! every operator action leaves an audited row.
use super::{SqliteStore, history, storage};
use mail_api::{AuditRow, ChangeFeed, Consumer, Cursor, Dirty, FeedRead, Resume};
use mail_kernel::Error;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

pub(super) const DDL: &str = "
    CREATE TABLE IF NOT EXISTS feed_cursors(consumer TEXT NOT NULL,account TEXT NOT NULL,revision INTEGER NOT NULL,PRIMARY KEY(consumer,account));
    CREATE TABLE IF NOT EXISTS feed_dirty(consumer TEXT NOT NULL,tenant TEXT NOT NULL,account TEXT NOT NULL,not_before INTEGER NOT NULL DEFAULT 0,poison TEXT,PRIMARY KEY(consumer,account));
    CREATE INDEX IF NOT EXISTS feed_dirty_tenant ON feed_dirty(consumer,tenant,account);
    CREATE TABLE IF NOT EXISTS audit(id INTEGER PRIMARY KEY AUTOINCREMENT,kind TEXT NOT NULL,reason TEXT NOT NULL,at_utc TEXT NOT NULL,operator TEXT NOT NULL,detail TEXT NOT NULL);";

/// Written inside the change transaction: one dirty key per enabled consumer.
pub(super) fn mark(
    db: &Connection,
    enabled: &[Consumer],
    tenant: &str,
    account: &str,
) -> Result<(), Error> {
    for consumer in enabled {
        db.execute(
            "INSERT INTO feed_dirty(consumer,tenant,account) VALUES(?1,?2,?3) ON CONFLICT(consumer,account) DO NOTHING",
            params![consumer.name(), tenant, account],
        )
        .map_err(storage)?;
    }
    Ok(())
}

pub(super) fn tail(db: &Connection, account: &str) -> Result<(String, u64, u64), Error> {
    db.query_row(
        "SELECT tenant,revision,history_floor FROM accounts WHERE id=?1",
        [account],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .optional()
    .map_err(storage)?
    .ok_or(Error::NotFound)
}

fn cursor(db: &Connection, consumer: Consumer, account: &str) -> Result<Option<u64>, Error> {
    db.query_row(
        "SELECT revision FROM feed_cursors WHERE consumer=?1 AND account=?2",
        params![consumer.name(), account],
        |r| r.get(0),
    )
    .optional()
    .map_err(storage)
}

/// Every operator action that changes feed state is a durable row.
pub(super) fn audited(
    db: &Connection,
    kind: &str,
    reason: &str,
    operator: &str,
    detail: &str,
) -> Result<(), Error> {
    if reason.trim().is_empty() || operator.trim().is_empty() {
        return Err(Error::Invalid);
    }
    db.execute(
        "INSERT INTO audit(kind,reason,at_utc,operator,detail) VALUES(?1,?2,strftime('%Y-%m-%dT%H:%M:%SZ','now'),?3,?4)",
        params![kind, reason, operator, detail],
    )
    .map_err(storage)?;
    Ok(())
}

impl ChangeFeed for SqliteStore {
    fn dirty(
        &self,
        consumer: Consumer,
        resume: &mut Resume,
        now: i64,
        per_tenant: usize,
        limit: usize,
    ) -> Result<Vec<Dirty>, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        // Tenants in round-robin order from the resume position, wrapping.
        let mut tenants = db
            .prepare("SELECT DISTINCT tenant FROM feed_dirty WHERE consumer=?1 AND poison IS NULL AND not_before<=?2 ORDER BY tenant")
            .map_err(storage)?;
        let all: Vec<String> = tenants
            .query_map(params![consumer.name(), now], |r| r.get(0))
            .map_err(storage)?
            .collect::<Result<_, _>>()
            .map_err(storage)?;
        let start = resume
            .after_tenant
            .as_ref()
            .map_or(0, |t| all.iter().position(|x| x > t).unwrap_or(0));
        let mut out = Vec::new();
        let mut accounts = db
            .prepare("SELECT account,not_before FROM feed_dirty WHERE consumer=?1 AND tenant=?2 AND poison IS NULL AND not_before<=?3 ORDER BY account LIMIT ?4")
            .map_err(storage)?;
        for i in 0..all.len() {
            if out.len() >= limit {
                break;
            }
            let tenant = &all[(start + i) % all.len()];
            let rows = accounts
                .query_map(
                    params![
                        consumer.name(),
                        tenant,
                        now,
                        per_tenant.min(limit - out.len()) as i64
                    ],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
                )
                .map_err(storage)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(storage)?;
            for (account, not_before) in rows {
                out.push(Dirty {
                    tenant: tenant.clone(),
                    account,
                    not_before,
                });
            }
            resume.after_tenant = Some(tenant.clone());
        }
        Ok(out)
    }

    fn changes(&self, consumer: Consumer, account: &str, limit: usize) -> Result<FeedRead, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let (_, revision, floor) = tail(&db, account)?;
        // Absent cursor ⇒ start at the tail.
        let since = cursor(&db, consumer, account)?.unwrap_or(revision);
        if since < floor {
            return Ok(FeedRead::BelowFloor {
                cursor: Cursor(since),
                floor,
            });
        }
        let page = history::page(&db, account, since, limit)?;
        Ok(FeedRead::Changes {
            cursor: Cursor(since),
            page,
        })
    }

    fn acknowledge(
        &self,
        consumer: Consumer,
        account: &str,
        revision: u64,
        now: i64,
        delay_secs: i64,
    ) -> Result<(), Error> {
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let (_, tail, floor) = tail(&tx, account)?;
        if revision > tail || revision < floor {
            return Err(Error::Conflict);
        }
        let poisoned: Option<String> = tx
            .query_row(
                "SELECT poison FROM feed_dirty WHERE consumer=?1 AND account=?2",
                params![consumer.name(), account],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?
            .flatten();
        if poisoned.is_some() {
            return Err(Error::Forbidden);
        }
        tx.execute(
            "INSERT INTO feed_cursors(consumer,account,revision) VALUES(?1,?2,?3) ON CONFLICT(consumer,account) DO UPDATE SET revision=max(revision,excluded.revision)",
            params![consumer.name(), account, revision],
        )
        .map_err(storage)?;
        if revision == tail {
            tx.execute(
                "DELETE FROM feed_dirty WHERE consumer=?1 AND account=?2",
                params![consumer.name(), account],
            )
            .map_err(storage)?;
        } else {
            tx.execute(
                "UPDATE feed_dirty SET not_before=?3 WHERE consumer=?1 AND account=?2",
                params![consumer.name(), account, now + delay_secs],
            )
            .map_err(storage)?;
        }
        tx.commit().map_err(storage)
    }

    fn poison(&self, consumer: Consumer, account: &str, reason: &str) -> Result<(), Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let (tenant, _, _) = tail(&db, account)?;
        db.execute(
            "INSERT INTO feed_dirty(consumer,tenant,account,poison) VALUES(?1,?2,?3,?4) ON CONFLICT(consumer,account) DO UPDATE SET poison=excluded.poison",
            params![consumer.name(), tenant, account, reason],
        )
        .map_err(storage)?;
        Ok(())
    }

    fn poisoned(&self, consumer: Consumer) -> Result<Vec<(Dirty, String)>, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let mut query = db
            .prepare("SELECT tenant,account,not_before,poison FROM feed_dirty WHERE consumer=?1 AND poison IS NOT NULL ORDER BY tenant,account")
            .map_err(storage)?;
        query
            .query_map([consumer.name()], |r| {
                Ok((
                    Dirty {
                        tenant: r.get(0)?,
                        account: r.get(1)?,
                        not_before: r.get(2)?,
                    },
                    r.get(3)?,
                ))
            })
            .map_err(storage)?
            .collect::<Result<_, _>>()
            .map_err(storage)
    }

    fn cursors(&self, account: &str, enabled: &[Consumer]) -> Result<Vec<(Consumer, u64)>, Error> {
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let mut out = Vec::new();
        for consumer in enabled {
            if let Some(revision) = cursor(&db, *consumer, account)? {
                out.push((*consumer, revision));
            }
        }
        Ok(out)
    }

    fn reconcile(&self, consumer: Consumer, operator: &str, reason: &str) -> Result<u64, Error> {
        super::feed_ops::reconcile(self, consumer, operator, reason)
    }
    fn dead_letter(
        &self,
        consumer: Consumer,
        account: &str,
        revision: u64,
        operator: &str,
        reason: &str,
    ) -> Result<(), Error> {
        super::feed_ops::dead_letter(self, consumer, account, revision, operator, reason)
    }
    fn retire_cursor(
        &self,
        consumer: Consumer,
        tenant: Option<&str>,
        operator: &str,
        reason: &str,
    ) -> Result<u64, Error> {
        super::feed_ops::retire_cursor(self, consumer, tenant, operator, reason)
    }
    fn audit(&self, limit: usize) -> Result<Vec<AuditRow>, Error> {
        super::feed_ops::audit(self, limit)
    }
}
