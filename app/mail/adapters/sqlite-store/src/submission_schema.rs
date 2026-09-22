use crate::storage;
use mail_kernel::Error;
use rusqlite::Connection;

pub(super) fn initialize(db: &Connection) -> Result<(), Error> {
    db.execute_batch("CREATE TABLE IF NOT EXISTS submission_heads(account TEXT PRIMARY KEY,revision INTEGER NOT NULL,floor INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS submission_versions(
            account TEXT NOT NULL,id TEXT NOT NULL,revision INTEGER NOT NULL,until_revision INTEGER,
            state TEXT CHECK(state IS NULL OR json_valid(state)),
            identity_id TEXT NOT NULL,email_id TEXT NOT NULL,thread_id TEXT NOT NULL,
            send_at INTEGER NOT NULL,undo TEXT NOT NULL,PRIMARY KEY(account,revision));
        CREATE UNIQUE INDEX IF NOT EXISTS submission_current ON submission_versions(account,id) WHERE until_revision IS NULL;
        CREATE INDEX IF NOT EXISTS submission_history ON submission_versions(account,id,revision);
        CREATE INDEX IF NOT EXISTS submission_previous ON submission_versions(account,id,until_revision);
        CREATE INDEX IF NOT EXISTS submission_identity ON submission_versions(account,identity_id,send_at);
        CREATE INDEX IF NOT EXISTS submission_email ON submission_versions(account,email_id,send_at);
        CREATE INDEX IF NOT EXISTS submission_thread ON submission_versions(account,thread_id,send_at);
        CREATE INDEX IF NOT EXISTS submission_undo ON submission_versions(account,undo,send_at);
        CREATE INDEX IF NOT EXISTS submission_time ON submission_versions(account,send_at,id);
        CREATE TABLE IF NOT EXISTS submission_schedule(message TEXT PRIMARY KEY,account TEXT NOT NULL,send_at INTEGER NOT NULL,claimed INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS submission_notices(notice TEXT PRIMARY KEY,submission TEXT NOT NULL);")
        .map_err(storage)?;
    let mut statement = db
        .prepare("PRAGMA table_info(submission_heads)")
        .map_err(storage)?;
    let names = statement
        .query_map([], |r| r.get::<_, String>(1))
        .map_err(storage)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage)?;
    drop(statement);
    if !names.iter().any(|name| name == "floor") {
        db.execute_batch(
            "ALTER TABLE submission_heads ADD COLUMN floor INTEGER NOT NULL DEFAULT 0",
        )
        .map_err(storage)?;
    }
    Ok(())
}
