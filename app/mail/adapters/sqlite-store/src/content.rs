use mail_kernel::Error;
use rusqlite::Connection;

/// A message body through its `message:<id>` link.
pub(super) fn get(db: &Connection, account: &str, id: &str) -> Result<Vec<u8>, Error> {
    if id.starts_with('b') {
        return Err(Error::NotFound);
    }
    super::blob::get(db, account, id)
}
