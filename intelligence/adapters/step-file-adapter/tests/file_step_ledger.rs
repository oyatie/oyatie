// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use oya_data_boundary_kernel::{DataClass, PrivacyDataClass, privacy_data_classes_from};
use intelligence_step_domain::{StepDisposition, StepKind, StepLedger, StepStart};
use intelligence_step_file_adapter::{FileStepLedgerStore, FileStepLedgerStoreError};

fn privacy_data_classes(data_classes: Vec<DataClass>) -> Vec<PrivacyDataClass> {
    privacy_data_classes_from(&data_classes).expect("test fixture uses privacy data classes")
}

#[test]
fn file_step_ledger_store_replays_records_and_persists_state_transitions() {
    let path = temp_store_path("append");
    let store = FileStepLedgerStore::new(path.clone());
    let mut ledger = StepLedger::default();
    let step = ledger
        .start(valid_start("provider.demo", StepKind::ProviderCall, 1_000))
        .expect("step starts");

    assert_eq!(store.save_ledger(&ledger).expect("initial save"), 1);
    assert_eq!(store.save_ledger(&ledger).expect("idempotent save"), 0);

    ledger
        .complete(&step.step_id.value, StepDisposition::Succeeded, 42, 1_001)
        .expect("step completes");
    assert_eq!(store.save_ledger(&ledger).expect("transition save"), 1);

    let restored = store.load().expect("ledger can be replayed");
    assert_eq!(restored.steps(), ledger.steps());

    fs::remove_file(path).ok();
}

#[test]
fn file_step_ledger_store_rejects_divergent_or_malformed_history() {
    let path = temp_store_path("diverge");
    let store = FileStepLedgerStore::new(path.clone());
    let mut original = StepLedger::default();
    original
        .start(valid_start("provider.demo", StepKind::ProviderCall, 1_000))
        .expect("original starts");
    store.save_ledger(&original).expect("initial save");
    let valid_wire_record = fs::read_to_string(&path).expect("step ledger wire record readable");

    let mut divergent = StepLedger::default();
    divergent
        .start(valid_start("provider.other", StepKind::ProviderCall, 1_000))
        .expect("divergent starts");
    assert_eq!(
        store.save_ledger(&divergent),
        Err(FileStepLedgerStoreError::LedgerDiverged)
    );

    fs::write(&path, valid_wire_record.replace("InternalOnly", "Audit"))
        .expect("non-privacy marker write");
    assert_eq!(store.load(), Err(FileStepLedgerStoreError::MalformedRecord));

    fs::write(&path, "not-a-step-record\n").expect("malform write");
    assert_eq!(store.load(), Err(FileStepLedgerStoreError::MalformedRecord));

    fs::remove_file(path).ok();
}

fn valid_start(provider_kind: &str, kind: StepKind, started_at: u64) -> StepStart {
    StepStart::new(
        "run_000000000001".into(),
        kind,
        provider_kind.into(),
        Some("model.demo".into()),
        Some(12),
        Some(34),
        privacy_data_classes(vec![DataClass::InternalOnly]),
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
        "oya-step-ledger-{label}-{}-{nanos}.log",
        std::process::id()
    ))
}
