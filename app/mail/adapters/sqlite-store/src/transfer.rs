use super::storage;
use mail_kernel::Error;
use rusqlite::{Connection, params};

/// Copy within the account transaction without loading message bodies into the
/// protocol worker. Source cleanup follows only after all copies succeed.
pub(super) fn copy(
    db: &Connection,
    account: &str,
    source: &str,
    target: &str,
) -> Result<(), Error> {
    let copied = db.execute(
        "INSERT INTO blob_links(account,owner,hash,version_id) SELECT account,'message:'||?3,hash,version_id FROM blob_links WHERE account=?1 AND owner='message:'||?2",
        params![account, source, target],
    ).map_err(storage)?;
    if copied != 1 {
        return Err(Error::Unavailable);
    }
    let members = db.execute(
        "INSERT INTO thread_members(account,message,subject,thread) SELECT account,?3,subject,thread FROM thread_members WHERE account=?1 AND message=?2",
        params![account,source,target],
    ).map_err(storage)?;
    if members != 1 {
        return Err(Error::Unavailable);
    }
    db.execute(
        "INSERT INTO thread_references(account,message,reference) SELECT account,?3,reference FROM thread_references WHERE account=?1 AND message=?2",
        params![account,source,target],
    ).map_err(storage)?;
    Ok(())
}
