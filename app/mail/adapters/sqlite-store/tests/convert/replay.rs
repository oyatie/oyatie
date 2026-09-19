//! What happens when the retired `41ce37e61` binary's `initialize` runs
//! against a converted database, recorded step by step. The observation is
//! part of the conversion's operator record.
use super::{Temp, convert_all, specs};
use mail_api::MetadataStore;
use mail_kernel::{Account, Error};
use mail_sqlite_store::SqliteStore;
use mail_sqlite_store::contract::legacy::legacy_initialize;
use mail_sqlite_store::contract::{LEGACY_ACCESS, LEGACY_DDL};

fn tables(db: &rusqlite::Connection, kind: &str) -> Vec<String> {
    let mut query = db
        .prepare("SELECT name FROM sqlite_master WHERE type=?1 ORDER BY name")
        .unwrap();
    query
        .query_map([kind], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn legacy_initialize_fails_on_the_access_backfill_and_rolls_back_leaving_the_file_serviceable() {
    let temp = Temp::new("replay");
    let specs = specs();
    let (_, _, _) = convert_all(&temp, &specs);
    let database = temp.path("mail.sqlite");
    let before = SqliteStore::open(&database)
        .unwrap()
        .account("alpha")
        .unwrap();

    let db = rusqlite::Connection::open(&database).unwrap();
    let error = legacy_initialize(&db).unwrap_err();
    // Step 1 (`LEGACY_DDL`): every `CREATE TABLE IF NOT EXISTS` is a no-op
    // for a table that exists under the relational shape (`accounts`,
    // `history_commits`, ...) and recreates the dropped journal tables.
    // Step 2 (`access()`): the table and its three triggers are created —
    // SQLite resolves `NEW.state` only when a trigger fires — and then the
    // backfill `INSERT ... SELECT json_extract(state, ...) FROM accounts`
    // fails to prepare: `accounts` has no `state` column.
    let text = error.to_string();
    assert!(
        text.starts_with("no such column: state in ")
            && text
                .contains("INSERT INTO account_access(account,tenant,owner,address,quota_bytes)")
            && text.contains("json_extract(state,'$.tenant')"),
        "legacy access backfill against the relational accounts table: {text}"
    );
    // The first batch opened `BEGIN IMMEDIATE` and never reached `COMMIT`:
    // the connection is still inside that transaction.
    assert!(!db.is_autocommit());
    assert!(tables(&db, "table").contains(&"account_access".to_owned()));
    assert!(tables(&db, "table").contains(&"message_changes".to_owned()));
    assert_eq!(
        tables(&db, "trigger"),
        [
            "account_access_delete",
            "account_access_insert",
            "account_access_update"
        ]
    );
    // Dropping the connection rolls the open transaction back.
    drop(db);
    let db = rusqlite::Connection::open(&database).unwrap();
    assert!(db.is_autocommit());
    assert!(tables(&db, "trigger").is_empty());
    for gone in [
        "account_access",
        "message_changes",
        "mailbox_commits",
        "mailbox_changes",
    ] {
        assert!(!tables(&db, "table").contains(&gone.to_owned()), "{gone}");
    }
    drop(db);
    let store = SqliteStore::open(&database).unwrap();
    assert_eq!(store.account("alpha").unwrap(), before);
}

#[test]
fn a_committed_legacy_trigger_breaks_provisioning_until_dropped() {
    let temp = Temp::new("replay-trigger");
    let (_, _, _) = convert_all(&temp, &specs());
    let database = temp.path("mail.sqlite");
    let db = rusqlite::Connection::open(&database).unwrap();
    // Run only the DDL of the two legacy batches, without the failing
    // backfill, and commit — the state an operator would reach by replaying
    // the legacy schema by hand.
    db.execute_batch(LEGACY_DDL).unwrap();
    // The last `INSERT INTO account_access` is the backfill; the first is
    // inside the insert trigger's body.
    let (access_without_backfill, _) = LEGACY_ACCESS
        .rsplit_once("INSERT INTO account_access")
        .unwrap();
    db.execute_batch(access_without_backfill).unwrap();
    db.execute_batch("COMMIT").unwrap();
    assert_eq!(tables(&db, "trigger").len(), 3);
    drop(db);
    // The store opens: `schema_version` is complete and the DDL is idempotent.
    let store = SqliteStore::open(&database).unwrap();
    assert_eq!(store.account("alpha").unwrap().messages.len(), 7);
    // The insert trigger fires on `provision` and fails: `NEW.state` does
    // not exist on the relational table; the insert is rolled back.
    let outcome = store.provision(
        Account::new("epsilon", "t", "eve", "epsilon@example.org").unwrap(),
        &"e".repeat(32),
    );
    assert_eq!(outcome, Err(Error::Unavailable));
    assert_eq!(store.account_info("epsilon"), Err(Error::NotFound));
    drop(store);
    let db = rusqlite::Connection::open(&database).unwrap();
    db.execute_batch(
        "DROP TRIGGER account_access_insert; DROP TRIGGER account_access_update;
         DROP TRIGGER account_access_delete; DROP TABLE account_access;",
    )
    .unwrap();
    drop(db);
    let store = SqliteStore::open(&database).unwrap();
    store
        .provision(
            Account::new("epsilon", "t", "eve", "epsilon@example.org").unwrap(),
            &"e".repeat(32),
        )
        .unwrap();
    assert_eq!(store.account_info("epsilon").unwrap().owner, "eve");
}
