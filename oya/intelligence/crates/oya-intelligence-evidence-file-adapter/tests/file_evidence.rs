// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use data_boundary_kernel::{DataClass, PrivacyDataClass, privacy_data_classes_from};
use oya_intelligence_evidence_domain::{EvidenceChain, EvidenceKind};
use oya_intelligence_evidence_file_adapter::{FileEvidenceChainStore, FileEvidenceStoreError};

fn privacy_data_classes(data_classes: Vec<DataClass>) -> Vec<PrivacyDataClass> {
    privacy_data_classes_from(&data_classes).expect("test fixture uses privacy data classes")
}

#[test]
fn file_evidence_store_replays_chain_and_appends_only_new_suffix() {
    let path = temp_store_path("append");
    let store = FileEvidenceChainStore::new(path.clone());
    let mut chain = EvidenceChain::default();
    chain
        .append(
            "ten_alpha".into(),
            "run_000000000001".into(),
            None,
            "cap.demo.invoke".into(),
            EvidenceKind::CapabilityInvocation,
            fields(("audit_event_hash", "fnv1a64:abc")),
            privacy_data_classes(vec![DataClass::InternalOnly]),
            1_000,
        )
        .unwrap();

    assert_eq!(store.append_chain(&chain).expect("initial append"), 1);
    assert_eq!(store.append_chain(&chain).expect("idempotent replay"), 0);

    chain
        .append(
            "ten_alpha".into(),
            "run_000000000001".into(),
            Some("step_000000000001_000001".into()),
            "cap.demo.invoke".into(),
            EvidenceKind::ToolCall,
            fields(("tool", "foundation.local|with separator")),
            privacy_data_classes(vec![
                DataClass::InternalOnly,
                DataClass::BehavioralTenantProduct,
            ]),
            1_001,
        )
        .unwrap();
    assert_eq!(store.append_chain(&chain).expect("suffix append"), 1);

    let restored = store.load().expect("store can be replayed");
    assert!(restored.verify());
    assert_eq!(restored.records(), chain.records());

    fs::remove_file(path).ok();
}

#[test]
fn file_evidence_store_rejects_divergent_history_and_tampered_records() {
    let path = temp_store_path("tamper");
    let store = FileEvidenceChainStore::new(path.clone());
    let mut original = EvidenceChain::default();
    original
        .append(
            "ten_alpha".into(),
            "run_000000000001".into(),
            None,
            "cap.demo.invoke".into(),
            EvidenceKind::CapabilityInvocation,
            fields(("audit_event_hash", "fnv1a64:abc")),
            privacy_data_classes(vec![DataClass::InternalOnly]),
            1_000,
        )
        .unwrap();
    store.append_chain(&original).expect("initial append");
    let valid_wire_record = fs::read_to_string(&path).expect("evidence wire record readable");

    let mut divergent = EvidenceChain::default();
    divergent
        .append(
            "ten_alpha".into(),
            "run_000000000001".into(),
            None,
            "cap.demo.other".into(),
            EvidenceKind::CapabilityInvocation,
            fields(("audit_event_hash", "fnv1a64:abc")),
            privacy_data_classes(vec![DataClass::InternalOnly]),
            1_000,
        )
        .unwrap();
    assert_eq!(
        store.append_chain(&divergent),
        Err(FileEvidenceStoreError::ChainDiverged)
    );

    fs::write(&path, valid_wire_record.replace("InternalOnly", "Audit"))
        .expect("non-privacy marker write");
    assert_eq!(store.load(), Err(FileEvidenceStoreError::MalformedRecord));

    let tampered = valid_wire_record.replace("cap.demo.invoke", "cap.demo.delete");
    fs::write(&path, tampered).expect("tamper write");
    assert_eq!(store.load(), Err(FileEvidenceStoreError::InvalidChain));

    fs::remove_file(path).ok();
}

fn fields(field: (&str, &str)) -> BTreeMap<String, String> {
    BTreeMap::from([(field.0.to_string(), field.1.to_string())])
}

fn temp_store_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "oya-evidence-store-{label}-{}-{nanos}.log",
        std::process::id()
    ))
}
