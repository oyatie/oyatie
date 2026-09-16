use super::{SqliteStore, storage};
use mail_api::{Event, Events};
use mail_kernel::{Error, valid_identifier};
use rusqlite::{OptionalExtension, TransactionBehavior, params};

impl Events for SqliteStore {
    fn pending(&self, consumer: &str, limit: usize) -> Result<Vec<Event>, Error> {
        if !valid_identifier(consumer) {
            return Err(Error::Invalid);
        }
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let mut query = db.prepare("SELECT sequence,tenant,account,revision,observed_at_ms FROM events WHERE sequence>coalesce((SELECT sequence FROM event_cursors WHERE consumer=?1),0) ORDER BY sequence LIMIT ?2").map_err(storage)?;
        query
            .query_map(params![consumer, limit.min(1000)], |r| {
                Ok(Event {
                    sequence: r.get(0)?,
                    tenant: r.get(1)?,
                    account: r.get(2)?,
                    revision: r.get(3)?,
                    observed_at_ms: r.get(4)?,
                })
            })
            .map_err(storage)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage)
    }
    fn acknowledge(&self, consumer: &str, sequence: u64) -> Result<(), Error> {
        if !valid_identifier(consumer) {
            return Err(Error::Invalid);
        }
        if sequence > i64::MAX as u64 {
            return Err(Error::Conflict);
        }
        let mut db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let current: u64 = tx
            .query_row(
                "SELECT sequence FROM event_cursors WHERE consumer=?1",
                [consumer],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?
            .unwrap_or(0);
        if sequence <= current {
            return Ok(());
        }
        let next: Option<u64> = tx
            .query_row(
                "SELECT min(sequence) FROM events WHERE sequence>?1",
                [current],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if next != Some(sequence) {
            return Err(Error::Conflict);
        }
        tx.execute("INSERT INTO event_cursors(consumer,sequence) VALUES(?1,?2) ON CONFLICT(consumer) DO UPDATE SET sequence=excluded.sequence",params![consumer,sequence]).map_err(storage)?;
        tx.commit().map_err(storage)
    }
}
