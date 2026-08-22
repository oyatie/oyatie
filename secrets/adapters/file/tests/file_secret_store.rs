// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use secrets_domain::{SecretMaterial, SecretRef, SecretVault};
use secrets_file::{FileSecretMetadata, FileSecretStore, FileSecretStoreError};

#[test]
fn file_secret_store_persists_metadata_without_reversible_secret_material() {
    let path = temp_secret_path("append");
    let store = FileSecretStore::new(path.clone());
    let secret_ref = SecretRef::new(
        "ten_alpha".into(),
        "cap.provider.openai".into(),
        "api-key".into(),
    )
    .expect("secret ref is valid");
    let mut vault = SecretVault::default();
    vault
        .put(
            secret_ref.clone(),
            SecretMaterial::from_bytes(b"sk-live-secret".to_vec())
                .expect("secret material is non-empty"),
            None,
        )
        .expect("initial secret is valid");
    vault
        .rotate(
            &secret_ref,
            SecretMaterial::from_bytes(b"sk-rotated-secret".to_vec())
                .expect("secret material is non-empty"),
            Some(2_000),
        )
        .expect("rotation is valid");

    assert_eq!(store.append_vault(&vault).expect("initial append"), 2);
    assert_eq!(store.append_vault(&vault).expect("idempotent append"), 0);
    let persisted = fs::read_to_string(&path).expect("store is readable");
    assert!(persisted.contains("v2|"));
    assert!(persisted.contains("cap.provider.openai"));
    assert!(persisted.contains(vault.records()[0].material.fingerprint()));
    assert!(persisted.contains(vault.records()[1].material.fingerprint()));
    assert!(!persisted.contains("sk-live-secret"));
    assert!(!persisted.contains("sk-rotated-secret"));
    assert!(!persisted.contains("736b2d6c6976652d736563726574"));
    assert!(!persisted.contains("736b2d726f74617465642d736563726574"));

    let expected_metadata = vault
        .records()
        .iter()
        .map(FileSecretMetadata::from_secret_version)
        .collect::<Vec<_>>();
    assert_eq!(
        store.load_metadata().expect("metadata loads"),
        expected_metadata
    );
    assert_eq!(
        store.load(),
        Err(FileSecretStoreError::SecretMaterialUnavailable)
    );
    assert!(
        store
            .matches_vault_metadata(&vault)
            .expect("metadata comparison works")
    );

    fs::remove_file(path).ok();
}

#[test]
fn file_secret_store_rejects_divergent_or_malformed_metadata_history() {
    let path = temp_secret_path("diverge");
    let store = FileSecretStore::new(path.clone());
    let secret_ref = SecretRef::new(
        "ten_alpha".into(),
        "cap.provider.openai".into(),
        "api-key".into(),
    )
    .expect("secret ref is valid");
    let mut original = SecretVault::default();
    original
        .put(
            secret_ref.clone(),
            SecretMaterial::from_bytes(b"sk-original".to_vec())
                .expect("secret material is non-empty"),
            None,
        )
        .expect("initial secret is valid");
    store.append_vault(&original).expect("initial append");

    let mut divergent = SecretVault::default();
    divergent
        .put(
            secret_ref,
            SecretMaterial::from_bytes(b"sk-divergent".to_vec())
                .expect("secret material is non-empty"),
            None,
        )
        .expect("initial secret is valid");
    assert_eq!(
        store.append_vault(&divergent),
        Err(FileSecretStoreError::SecretHistoryDiverged)
    );

    fs::write(&path, "not-a-secret-record\n").expect("malform write");
    assert_eq!(
        store.load_metadata(),
        Err(FileSecretStoreError::MalformedRecord)
    );
    assert_eq!(store.load(), Err(FileSecretStoreError::MalformedRecord));

    fs::remove_file(path).ok();
}

#[test]
fn file_secret_store_refuses_legacy_records_that_contain_secret_material() {
    let path = temp_secret_path("legacy");
    let store = FileSecretStore::new(path.clone());
    fs::write(
        &path,
        "v1|9:ten_alpha|19:cap.provider.openai|7:api-key|1|none|none|active|736b2d6c6976652d736563726574|22:fnv1a64:0000000000000000\n",
    )
    .expect("legacy material record written");

    assert_eq!(
        store.load_metadata(),
        Err(FileSecretStoreError::SecretMaterialUnavailable)
    );
    assert_eq!(
        store.load(),
        Err(FileSecretStoreError::SecretMaterialUnavailable)
    );

    fs::remove_file(path).ok();
}

fn temp_secret_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "secret-store-{label}-{}-{nanos}.log",
        std::process::id()
    ))
}
