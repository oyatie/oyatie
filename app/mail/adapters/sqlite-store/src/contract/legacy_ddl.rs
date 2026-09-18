//! The `41ce37e61` schema, statement for statement as that binary's
//! `initialize` issued it: one `BEGIN IMMEDIATE` in the first batch, every
//! later batch inside that transaction, `COMMIT` at the end of the last.

/// First batch: pragmas, the transaction start, the JSON `state` account
/// table, events, content and the legacy journal tables.
pub const LEGACY_DDL: &str = "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY, address TEXT NOT NULL UNIQUE, token BLOB UNIQUE,
                state TEXT NOT NULL CHECK(json_valid(state)));
            CREATE TABLE IF NOT EXISTS events (
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                tenant TEXT NOT NULL, account TEXT NOT NULL, revision INTEGER NOT NULL,
                delivered INTEGER NOT NULL DEFAULT 0, observed_at_ms INTEGER NOT NULL DEFAULT 0,
                UNIQUE(account, revision));
            CREATE INDEX IF NOT EXISTS account_credentials ON accounts(token,id);
            CREATE TABLE IF NOT EXISTS blobs (
                account TEXT NOT NULL, id TEXT NOT NULL, content BLOB NOT NULL,
                expires_at INTEGER NOT NULL, PRIMARY KEY(account,id));
            CREATE TABLE IF NOT EXISTS message_bodies (
                account TEXT NOT NULL, id TEXT NOT NULL, content BLOB NOT NULL,
                PRIMARY KEY(account,id));
            CREATE INDEX IF NOT EXISTS blobs_expiry ON blobs(expires_at);
            CREATE TABLE IF NOT EXISTS history_commits (
                account TEXT NOT NULL, previous INTEGER NOT NULL, revision INTEGER NOT NULL,
                PRIMARY KEY(account,revision));
            CREATE TABLE IF NOT EXISTS message_changes (
                account TEXT NOT NULL, revision INTEGER NOT NULL, id TEXT NOT NULL,
                before_state TEXT, after_state TEXT, PRIMARY KEY(account,revision,id));
            CREATE TABLE IF NOT EXISTS mailbox_commits (
                account TEXT NOT NULL, previous INTEGER NOT NULL, revision INTEGER NOT NULL,
                PRIMARY KEY(account,revision));
            CREATE TABLE IF NOT EXISTS mailbox_changes (
                account TEXT NOT NULL, revision INTEGER NOT NULL, id TEXT NOT NULL,
                before_state TEXT, after_state TEXT, PRIMARY KEY(account,revision,id));";

/// `access()`: the denormalized authorization table, its three triggers on
/// `accounts`, and the backfill `INSERT ... SELECT json_extract(state, ...)`.
pub const LEGACY_ACCESS: &str = "CREATE TABLE IF NOT EXISTS account_access (
        account TEXT PRIMARY KEY, tenant TEXT NOT NULL, owner TEXT NOT NULL,
        address TEXT NOT NULL, quota_bytes INTEGER NOT NULL CHECK(quota_bytes>=0));
        CREATE TRIGGER IF NOT EXISTS account_access_insert AFTER INSERT ON accounts BEGIN
            INSERT INTO account_access(account,tenant,owner,address,quota_bytes)
            VALUES(NEW.id,json_extract(NEW.state,'$.tenant'),json_extract(NEW.state,'$.owner'),
                json_extract(NEW.state,'$.address'),json_extract(NEW.state,'$.quota_bytes'));
        END;
        CREATE TRIGGER IF NOT EXISTS account_access_update AFTER UPDATE OF state ON accounts BEGIN
            UPDATE account_access SET tenant=json_extract(NEW.state,'$.tenant'),
                owner=json_extract(NEW.state,'$.owner'),address=json_extract(NEW.state,'$.address'),
                quota_bytes=json_extract(NEW.state,'$.quota_bytes') WHERE account=NEW.id;
        END;
        CREATE TRIGGER IF NOT EXISTS account_access_delete AFTER DELETE ON accounts BEGIN
            DELETE FROM account_access WHERE account=OLD.id;
        END;
        INSERT INTO account_access(account,tenant,owner,address,quota_bytes)
            SELECT id,json_extract(state,'$.tenant'),json_extract(state,'$.owner'),
                json_extract(state,'$.address'),json_extract(state,'$.quota_bytes') FROM accounts a
            WHERE NOT EXISTS(SELECT 1 FROM account_access d WHERE d.account=a.id);";

/// `submission_schema::initialize`, unchanged between the two versions; the
/// `floor` column check that followed it is replayed by `legacy_initialize`.
pub const LEGACY_SUBMISSION: &str = "CREATE TABLE IF NOT EXISTS submission_heads(account TEXT PRIMARY KEY,revision INTEGER NOT NULL,floor INTEGER NOT NULL DEFAULT 0);
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
        CREATE TABLE IF NOT EXISTS submission_schedule(message TEXT PRIMARY KEY,account TEXT NOT NULL,send_at INTEGER NOT NULL,claimed INTEGER NOT NULL DEFAULT 0);";

/// Issued only when `PRAGMA table_info(events)` lacked the column.
pub const LEGACY_EVENTS_COLUMN: &str =
    "ALTER TABLE events ADD COLUMN observed_at_ms INTEGER NOT NULL DEFAULT 0";

pub const LEGACY_EVENT_CURSORS: &str = "CREATE TABLE IF NOT EXISTS event_cursors(consumer TEXT PRIMARY KEY,sequence INTEGER NOT NULL CHECK(sequence>=0))";

pub const LEGACY_QUEUE: &str = "CREATE TABLE IF NOT EXISTS queued_messages (
        id TEXT PRIMARY KEY,sender TEXT NOT NULL,content BLOB NOT NULL,size INTEGER NOT NULL,received_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS delivery_jobs (
            message TEXT NOT NULL,account TEXT NOT NULL,address TEXT NOT NULL,
            next_attempt INTEGER NOT NULL,lease_until INTEGER NOT NULL DEFAULT 0,
            token TEXT,attempt INTEGER NOT NULL DEFAULT 0,last_error TEXT,PRIMARY KEY(message,account));
        CREATE INDEX IF NOT EXISTS delivery_due ON delivery_jobs(next_attempt,lease_until);
        CREATE INDEX IF NOT EXISTS delivery_account ON delivery_jobs(account,message);
        CREATE TABLE IF NOT EXISTS delivery_receipts (
            account TEXT NOT NULL,id TEXT NOT NULL,digest BLOB NOT NULL,PRIMARY KEY(account,id));";

/// Last batch: thread index, its content trigger, the lazy message index
/// with its invalidation triggers on `accounts.state`, the failure and
/// outbound tables, and the `COMMIT` that closes the first batch's
/// transaction.
pub const LEGACY_THREADS: &str = "CREATE TABLE IF NOT EXISTS thread_indexed(account TEXT PRIMARY KEY);
        CREATE TABLE IF NOT EXISTS thread_members(account TEXT NOT NULL,message TEXT NOT NULL,subject BLOB NOT NULL,thread TEXT NOT NULL,PRIMARY KEY(account,message));
        CREATE INDEX IF NOT EXISTS thread_groups ON thread_members(account,thread);
        CREATE TABLE IF NOT EXISTS thread_references(account TEXT NOT NULL,message TEXT NOT NULL,reference BLOB NOT NULL,PRIMARY KEY(account,message,reference));
        CREATE INDEX IF NOT EXISTS thread_lookup ON thread_references(account,reference,message);
        CREATE TRIGGER IF NOT EXISTS thread_content_delete AFTER DELETE ON message_bodies BEGIN
            DELETE FROM thread_members WHERE account=OLD.account AND message=OLD.id;
            DELETE FROM thread_references WHERE account=OLD.account AND message=OLD.id;
        END;
        CREATE TABLE IF NOT EXISTS message_index_state(account TEXT PRIMARY KEY,revision INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS message_metadata(account TEXT NOT NULL,id TEXT NOT NULL,state TEXT NOT NULL CHECK(json_valid(state)),PRIMARY KEY(account,id));
        CREATE TRIGGER IF NOT EXISTS message_index_invalidated AFTER UPDATE OF state ON accounts BEGIN
            DELETE FROM message_index_state WHERE account=NEW.id;
        END;
        CREATE TRIGGER IF NOT EXISTS thread_index_invalidated AFTER UPDATE OF state ON accounts BEGIN
            DELETE FROM thread_indexed WHERE account=NEW.id;
        END;
        CREATE TRIGGER IF NOT EXISTS message_index_deleted AFTER DELETE ON accounts BEGIN
            DELETE FROM message_index_state WHERE account=OLD.id;
            DELETE FROM message_metadata WHERE account=OLD.id;
        END;
        CREATE TABLE IF NOT EXISTS failed_delivery_messages (
            id TEXT PRIMARY KEY,sender TEXT NOT NULL,content BLOB NOT NULL,size INTEGER NOT NULL,received_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS failed_delivery_jobs (
            message TEXT NOT NULL,account TEXT NOT NULL,address TEXT NOT NULL,failed_at INTEGER NOT NULL,reason TEXT NOT NULL,PRIMARY KEY(message,account));
        CREATE INDEX IF NOT EXISTS failed_deliveries_account ON failed_delivery_jobs(account,failed_at,message);
        CREATE TABLE IF NOT EXISTS submitted_messages (
            id TEXT PRIMARY KEY,account TEXT NOT NULL,sender TEXT NOT NULL,content BLOB NOT NULL,size INTEGER NOT NULL,received_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS outbound_jobs (
            message TEXT NOT NULL,account TEXT NOT NULL,recipient TEXT NOT NULL,
            next_attempt INTEGER NOT NULL,lease_until INTEGER NOT NULL DEFAULT 0,
            token TEXT,attempt INTEGER NOT NULL DEFAULT 0,last_code INTEGER,PRIMARY KEY(message,recipient));
        CREATE INDEX IF NOT EXISTS outbound_due ON outbound_jobs(next_attempt,lease_until);
        CREATE INDEX IF NOT EXISTS outbound_account ON outbound_jobs(account,message);
        CREATE TABLE IF NOT EXISTS vacation_sent(account TEXT NOT NULL, sender TEXT NOT NULL, PRIMARY KEY(account,sender));
        COMMIT";
