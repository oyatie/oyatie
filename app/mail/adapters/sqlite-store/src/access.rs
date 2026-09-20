use super::{SqliteStore, storage};
use mail_api::{AccountInfo, Identity, Principal};
use mail_kernel::Error;
use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};

pub(super) fn load(db: &Connection, id: &str) -> Result<AccountInfo, Error> {
    db.query_row(
        "SELECT id,tenant,owner,address,quota_bytes FROM accounts WHERE id=?1",
        [id],
        |row| {
            Ok(AccountInfo {
                id: row.get(0)?,
                tenant: row.get(1)?,
                owner: row.get(2)?,
                address: row.get(3)?,
                quota_bytes: row.get(4)?,
            })
        },
    )
    .optional()
    .map_err(storage)?
    .ok_or(Error::NotFound)
}

impl Identity for SqliteStore {
    fn authenticate(&self, token: &str) -> Result<Principal, Error> {
        if token.len() < 32 || token.len() > 4096 {
            return Err(Error::Forbidden);
        }
        let db = self.connection.lock().map_err(|_| Error::Unavailable)?;
        db.query_row(
            "SELECT tenant,owner,id FROM accounts WHERE token=?1",
            [Sha256::digest(token.as_bytes()).as_slice()],
            |r| {
                Ok(Principal {
                    tenant: r.get(0)?,
                    subject: r.get(1)?,
                    account: r.get(2)?,
                })
            },
        )
        .optional()
        .map_err(storage)?
        .ok_or(Error::Forbidden)
    }
}
