//! A store whose file went through `convert` from the `41ce37e61` DDL, so a
//! harness exercises the converted schema instead of a freshly initialized one.
use super::legacy::{LegacyAccountSpec, LegacyFixture};
use crate::SqliteStore;
use crate::convert::Converter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Scratch directory holding the converted file; removed when the store drops.
pub struct Scratch(PathBuf);

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

impl Scratch {
    fn new() -> Self {
        // Process id plus an in-process sequence: parallel tests in one binary
        // start within the same clock tick.
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("mail-converted-{}-{sequence}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn path(&self, file: &str) -> PathBuf {
        self.0.join(file)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Build a legacy database holding `accounts`, back it up under the lock,
/// convert it, and open the converted file as this binary's store.
pub fn converted_store(accounts: &[LegacyAccountSpec]) -> SqliteStore {
    let scratch = Scratch::new();
    let database = scratch.path("mail.sqlite");
    LegacyFixture::create(&database, accounts).unwrap();
    let converter = Converter::open(&database).unwrap();
    converter
        .backup_into(&scratch.path("backup.sqlite"))
        .unwrap();
    converter
        .convert(
            Path::new(&scratch.path("backup.sqlite")),
            "oracle@localhost",
        )
        .unwrap();
    let mut store = SqliteStore::open(&database).unwrap();
    store.scratch = Some(scratch);
    store
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SCHEMA_VERSION, SchemaState, inspect};

    #[test]
    fn a_converted_store_reports_the_binary_schema_and_cleans_up_on_drop() {
        let store = converted_store(&[]);
        let dir = store.scratch.as_ref().unwrap().0.clone();
        let db = store.connection.lock().unwrap();
        assert!(matches!(
            inspect(&db).unwrap(),
            SchemaState::Complete { version } if version == SCHEMA_VERSION
        ));
        drop(db);
        drop(store);
        assert!(!dir.exists());
    }
}
