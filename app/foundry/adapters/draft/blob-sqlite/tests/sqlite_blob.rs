//! The SQLite blob store, held to the port's conformance suite — including
//! the durability clause the in-memory reference could only decline.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use foundry_blob_draft::conformance::{
    BlobFixture, check_content_address_integrity, check_durability_across_reopen,
    check_empty_blob_is_a_valid_blob, check_missing_blob_reads_none,
    check_put_is_idempotent_by_content, check_round_trip_preserves_bytes,
    check_tenants_never_share_blobs,
};
use foundry_blob_draft::{BlobRef, BlobStore};
use foundry_blob_sqlite_draft::SqliteBlobStore;

struct SqliteFixture {
    path: PathBuf,
    store: SqliteBlobStore,
}

impl SqliteFixture {
    fn new(case: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "foundry-blob-{case}-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&path);
        let store = SqliteBlobStore::open(&path).expect("open a fresh database");
        Self { path, store }
    }
}

impl Drop for SqliteFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl BlobFixture for SqliteFixture {
    type Store = SqliteBlobStore;

    fn store(&mut self) -> &mut Self::Store {
        &mut self.store
    }

    fn reopen(&mut self) -> bool {
        self.store = SqliteBlobStore::open(&self.path).expect("reopen the database");
        true
    }
}

type Check = fn(&mut SqliteFixture) -> Result<(), String>;

#[test]
fn sqlite_store_satisfies_every_conformance_check() {
    let checks: [(&str, Check); 7] = [
        ("round trip", check_round_trip_preserves_bytes),
        ("content address", check_content_address_integrity),
        ("idempotent put", check_put_is_idempotent_by_content),
        ("tenant isolation", check_tenants_never_share_blobs),
        ("missing is none", check_missing_blob_reads_none),
        ("empty blob", check_empty_blob_is_a_valid_blob),
        ("durability", check_durability_across_reopen),
    ];
    for (name, check) in checks {
        let mut fixture = SqliteFixture::new(name.replace(' ', "-").as_str());
        check(&mut fixture).unwrap_or_else(|violation| panic!("{name}: {violation}"));
    }
}

#[test]
fn a_corrupted_row_reads_as_a_loud_error_not_wrong_bytes() {
    let mut fixture = SqliteFixture::new("corruption");
    let reference = fixture.store.put("ten_a", b"pristine bytes").unwrap();
    fixture
        .store
        .corrupt_for_test(&reference)
        .expect("test hook flips a stored byte");
    let read = fixture.store.get("ten_a", &reference);
    assert!(
        read.is_err(),
        "corrupted content must surface as an error, got {read:?}"
    );
    assert!(
        !fixture.reopen()
            || fixture
                .store
                .get("ten_a", &BlobRef::for_bytes(b"pristine bytes"))
                .is_err()
    );
}
