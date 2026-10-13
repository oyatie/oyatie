use super::*;
use foundry_model_api::conformance::*;
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicU64, Ordering},
};

mod encoded_boundaries;
mod history_capacity;
mod refusal;
mod rollback;

static NEXT: AtomicU64 = AtomicU64::new(1);
std::thread_local! {
    static INITIALIZE_FAILURE: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}
pub(super) fn initialization_fault(step: &str) -> Result<(), ModelError> {
    INITIALIZE_FAILURE.with(|fault| {
        if fault.get() == Some(step) {
            fault.set(None);
            Err(ModelError::Storage {
                detail: "injected initialization boundary failure".into(),
            })
        } else {
            Ok(())
        }
    })
}
struct Database(std::path::PathBuf);
impl Database {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "oyatie-model-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        Self(directory)
    }
    fn path(&self) -> std::path::PathBuf {
        self.0.join("model.sqlite")
    }
    fn initialize(&self) -> SqliteModelStore {
        SqliteModelStore::initialize(&self.path(), binding(), baseline()).unwrap()
    }
    fn open(&self) -> SqliteModelStore {
        SqliteModelStore::open_existing(&self.path(), binding(), baseline()).unwrap()
    }
}
impl Drop for Database {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn durable_adapter_runs_owner_only_cursor_discovery_after_reopen() {
    let database = Database::new();
    let mut store = database.initialize();
    check_owned_draft_discovery(&mut store).unwrap();
    let actor = ModelCaller {
        owner: "usr_author\0one".into(),
        ..caller()
    };
    let request = DraftPageRequest {
        limit: 1,
        after: Some(2),
    };
    let expected = store.owned_drafts(&actor, &request).unwrap();
    drop(store);
    assert_eq!(
        database.open().owned_drafts(&actor, &request).unwrap(),
        expected
    );
}

#[test]
fn durable_adapter_runs_unchanged_owned_lifecycle_and_restart_history() {
    let database = Database::new();
    let mut store = database.initialize();
    check_owned_versioned_lifecycle(&mut store).unwrap();
    let history = store.history(&caller()).unwrap();
    let draft = store.open_draft(&caller(), 1).unwrap();
    drop(store);
    let opened = database.open();
    assert_eq!(opened.history(&caller()).unwrap(), history);
    assert_eq!(opened.open_draft(&caller(), 1).unwrap(), draft);
}

#[test]
fn durable_adapter_runs_unchanged_two_handle_conflicts_and_original_receipt() {
    let database = Database::new();
    let mut left = database.initialize();
    let mut right = database.open();
    check_two_handle_conflicts(&mut left, &mut right).unwrap();
}

#[test]
fn simultaneous_saves_and_publications_have_exactly_one_winner() {
    let database = Database::new();
    let mut store = database.initialize();
    let draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
    let first = database.open();
    let second = database.open();
    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = [first, second]
        .into_iter()
        .map(|mut handle| {
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                handle.save_draft(&caller(), draft.id, 1, 1, additive_model())
            })
        })
        .collect();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| **result == Err(ModelError::DraftConflict { current_version: 2 }))
            .count(),
        1
    );
    let second_draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
    let first = database.open();
    let second = database.open();
    let prep1 = first.prepare(&caller(), draft.id, 2, 1).unwrap();
    let prep2 = second.prepare(&caller(), second_draft.id, 1, 1).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = [(first, prep1), (second, prep2)]
        .into_iter()
        .map(|(mut handle, prepared)| {
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                handle.publish(&caller(), &prepared)
            })
        })
        .collect();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| **result
                == Err(ModelError::HeadConflict {
                    current_generation: 2
                }))
            .count(),
        1
    );
    assert_eq!(store.history(&caller()).unwrap().publications.len(), 2);
    assert_eq!(
        store
            .open_draft(&caller(), 1)
            .unwrap()
            .publication
            .is_some() as u8
            + store
                .open_draft(&caller(), 2)
                .unwrap()
                .publication
                .is_some() as u8,
        1
    );
}

#[test]
fn committed_publication_reopens_and_returns_original_receipt_after_lost_ack() {
    let database = Database::new();
    let mut store = database.initialize();
    let draft = store.create_draft(&caller(), 1, additive_model()).unwrap();
    let prepared = store
        .prepare(&caller(), draft.id, draft.version, draft.base_generation)
        .unwrap();
    let original = store.publish(&caller(), &prepared).unwrap();
    drop(store);
    drop(prepared);
    let reopened = database.open();
    assert_eq!(
        reopened
            .receipt(&caller(), draft.id, draft.version, draft.base_generation)
            .unwrap(),
        Some(original)
    );
    assert!(matches!(
        reopened.receipt(
            &caller(),
            draft.id,
            draft.version + 1,
            draft.base_generation
        ),
        Err(ModelError::DraftConflict { .. })
    ));
    assert_eq!(
        reopened
            .history(&caller())
            .unwrap()
            .head()
            .unwrap()
            .generation,
        2
    );
}

#[test]
fn model_transactions_preserve_existing_object_log_bytes() {
    let database = Database::new();
    let connection = Connection::open(database.path()).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE action_log (ordinal INTEGER PRIMARY KEY, payload BLOB);
        INSERT INTO action_log VALUES (1, X'00FF001020');",
        )
        .unwrap();
    drop(connection);
    let mut store = database.initialize();
    check_owned_versioned_lifecycle(&mut store).unwrap();
    let bytes: Vec<u8> = store
        .connection
        .query_row(
            "SELECT payload FROM action_log WHERE ordinal = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(bytes, [0, 255, 0, 16, 32]);
    assert_eq!(
        store
            .connection
            .query_row("SELECT COUNT(*) FROM action_log", [], |row| row
                .get::<_, u64>(0))
            .unwrap(),
        1
    );
}
