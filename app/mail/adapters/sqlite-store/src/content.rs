use super::storage;
use mail_kernel::Error;
use rusqlite::{Connection, OptionalExtension, params};

pub(super) fn get(db: &Connection, account: &str, id: &str) -> Result<Vec<u8>, Error> {
    db.query_row(
        "SELECT content FROM message_bodies WHERE account=?1 AND id=?2",
        params![account, id],
        |r| r.get(0),
    )
    .optional()
    .map_err(storage)?
    .ok_or(Error::NotFound)
}
