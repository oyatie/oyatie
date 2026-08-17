// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use intelligence_capability_domain::AutonomyTier;
use intelligence_run_domain::{RunDisposition, RunLedger, RunStart};
use oya_data_boundary_kernel::{DataClass, PrivacyDataClass, privacy_data_classes_from};
use oya_intelligence_run_file_adapter::{FileRunLedgerStore, FileRunLedgerStoreError};

fn privacy_data_classes(data_classes: Vec<DataClass>) -> Vec<PrivacyDataClass> {
    privacy_data_classes_from(&data_classes).expect("test fixture uses privacy data classes")
}

#[test]
fn file_run_ledger_store_replays_records_and_persists_state_transitions() {
    let path = temp_store_path("append");
    let store = FileRunLedgerStore::new(path.clone());
    let mut ledger = RunLedger::default();
    let run = ledger
        .start(valid_start("cap.demo.invoke", "idem-1", 1_000))
        .expect("run starts");

    assert_eq!(store.save_ledger(&ledger).expect("initial save"), 1);
    assert_eq!(store.save_ledger(&ledger).expect("idempotent save"), 0);

    ledger
        .complete(&run.run_id.value, RunDisposition::Success, 1_001)
        .expect("run completes");
    assert_eq!(store.save_ledger(&ledger).expect("transition save"), 1);

    let restored = store.load().expect("ledger can be replayed");
    assert_eq!(restored.runs(), ledger.runs());

    fs::remove_file(path).ok();
}

#[test]
fn file_run_ledger_store_rejects_divergent_or_malformed_history() {
    let path = temp_store_path("diverge");
    let store = FileRunLedgerStore::new(path.clone());
    let mut original = RunLedger::default();
    original
        .start(valid_start("cap.demo.invoke", "idem-1", 1_000))
        .expect("original starts");
    store.save_ledger(&original).expect("initial save");
    let valid_wire_record = fs::read_to_string(&path).expect("run ledger wire record readable");

    let mut divergent = RunLedger::default();
    divergent
        .start(valid_start("cap.demo.delete", "idem-1", 1_000))
        .expect("divergent starts");
    assert_eq!(
        store.save_ledger(&divergent),
        Err(FileRunLedgerStoreError::LedgerDiverged)
    );

    fs::write(&path, valid_wire_record.replace("InternalOnly", "Audit"))
        .expect("non-privacy marker write");
    assert_eq!(store.load(), Err(FileRunLedgerStoreError::MalformedRecord));

    fs::write(&path, "not-a-run-record\n").expect("malform write");
    assert_eq!(store.load(), Err(FileRunLedgerStoreError::MalformedRecord));

    fs::remove_file(path).ok();
}

fn valid_start(capability_id: &str, idempotency_key: &str, started_at: u64) -> RunStart {
    RunStart::new(
        "ten_alpha".into(),
        capability_id.into(),
        "usr_admin".into(),
        AutonomyTier::T2Advisory,
        privacy_data_classes(vec![DataClass::InternalOnly]),
        "region-home".into(),
        idempotency_key.into(),
        started_at,
    )
    .unwrap()
}

fn temp_store_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "oya-run-ledger-{label}-{}-{nanos}.log",
        std::process::id()
    ))
}
