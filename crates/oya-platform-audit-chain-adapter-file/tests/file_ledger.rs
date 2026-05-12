use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use oya_platform_audit_chain_adapter_file::{FileAuditLedger, FileAuditLedgerError};
use oya_platform_audit_chain_kernel::{AuditChain, Plane};
use oya_platform_data_boundary_kernel::{DataClass, Purpose};

#[test]
fn file_audit_ledger_replays_events_and_appends_only_new_suffix() {
    let path = temp_ledger_path("append");
    let ledger = FileAuditLedger::new(path.clone());
    let mut chain = AuditChain::default();
    chain.append_classifications(
        "ten_alpha",
        "tenant.create",
        Plane::Control,
        Purpose::CoreService,
        vec![DataClass::InternalOnly],
        "ALLOW",
    );
    chain.append_classifications(
        "ten_alpha",
        "identity.user.upsert",
        Plane::Control,
        Purpose::CoreService,
        vec![DataClass::PiiIdentifying],
        "ALLOW",
    );

    assert_eq!(ledger.append_chain(&chain).expect("initial append"), 2);
    assert_eq!(ledger.append_chain(&chain).expect("idempotent replay"), 0);

    chain.append_classifications(
        "ten_alpha",
        "foundry.capability.invoke",
        Plane::Data,
        Purpose::CapabilityInvocation,
        vec![DataClass::InternalOnly],
        "ALLOW",
    );
    assert_eq!(ledger.append_chain(&chain).expect("suffix append"), 1);

    let restored = ledger.load().expect("ledger can be replayed");
    assert!(restored.verify());
    assert_eq!(restored.events(), chain.events());

    fs::remove_file(path).ok();
}

#[test]
fn file_audit_ledger_rejects_divergent_history_and_tampered_records() {
    let path = temp_ledger_path("tamper");
    let ledger = FileAuditLedger::new(path.clone());
    let mut original = AuditChain::default();
    original.append_classifications(
        "ten_alpha",
        "tenant.create",
        Plane::Control,
        Purpose::CoreService,
        vec![DataClass::InternalOnly],
        "ALLOW",
    );
    ledger.append_chain(&original).expect("initial append");

    let mut divergent = AuditChain::default();
    divergent.append_classifications(
        "ten_alpha",
        "tenant.delete",
        Plane::Control,
        Purpose::CoreService,
        vec![DataClass::InternalOnly],
        "ALLOW",
    );
    assert_eq!(
        ledger.append_chain(&divergent),
        Err(FileAuditLedgerError::ChainDiverged)
    );

    let tampered = fs::read_to_string(&path)
        .expect("ledger readable")
        .replace("tenant.create", "tenant.delete");
    fs::write(&path, tampered).expect("tamper write");
    assert_eq!(ledger.load(), Err(FileAuditLedgerError::InvalidChain));

    fs::remove_file(path).ok();
}

fn temp_ledger_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "oya-audit-ledger-{label}-{}-{nanos}.log",
        std::process::id()
    ))
}
