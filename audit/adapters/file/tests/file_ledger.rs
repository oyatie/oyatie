// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use audit_chain_domain::{
    AuditAppendInput, AuditChain, Ed25519SigningKey, Ed25519VerificationKeySet, Plane,
};
use audit_file_adapter::{FileAuditLedger, FileAuditLedgerError};
use data_boundary_kernel::{DataClass, Purpose};

#[test]
fn file_audit_ledger_replays_events_and_appends_only_new_suffix() {
    let path = temp_ledger_path("append");
    let ledger = FileAuditLedger::new(path.clone());
    let mut chain = AuditChain::default();
    chain
        .append_classifications(
            "ten_alpha",
            "tenant.create",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )
        .expect("test fixture: tenant.create append must succeed");
    chain
        .append_classifications(
            "ten_alpha",
            "identity.user.upsert",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::PiiIdentifying],
            "ALLOW",
        )
        .expect("test fixture: identity.user.upsert append must succeed");

    assert_eq!(ledger.append_chain(&chain).expect("initial append"), 2);
    assert_eq!(ledger.append_chain(&chain).expect("idempotent replay"), 0);

    chain
        .append_classifications(
            "ten_alpha",
            "foundry.capability.invoke",
            Plane::Data,
            Purpose::CapabilityInvocation,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )
        .expect("test fixture: foundry.capability.invoke append must succeed");
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
    original
        .append_classifications(
            "ten_alpha",
            "tenant.create",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )
        .expect("test fixture: original tenant.create append must succeed");
    ledger.append_chain(&original).expect("initial append");

    let mut divergent = AuditChain::default();
    divergent
        .append_classifications(
            "ten_alpha",
            "tenant.delete",
            Plane::Control,
            Purpose::CoreService,
            vec![DataClass::InternalOnly],
            "ALLOW",
        )
        .expect("test fixture: divergent tenant.delete append must succeed");
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

#[test]
fn file_audit_ledger_roundtrips_signed_merkle_records() {
    let path = temp_ledger_path("signed");
    let ledger = FileAuditLedger::new(path.clone());
    let signer = Ed25519SigningKey::from_seed_bytes("audit-file-ledger-key", [11_u8; 32])
        .expect("test signing key");
    let trusted_keys =
        Ed25519VerificationKeySet::single(signer.verification_key()).expect("trusted key set");
    let mut chain = AuditChain::default();
    chain
        .append_signed(
            AuditAppendInput {
                tenant_id: "ten_alpha".to_string(),
                surface: "foundry.capability.invoke".to_string(),
                plane: Plane::Audit,
                purpose: Purpose::CapabilityInvocation,
                data_classes: vec![DataClass::InternalOnly, DataClass::Audit],
                decision: "ALLOW".to_string(),
            },
            &signer,
        )
        .expect("signed append");

    assert_eq!(
        ledger.append_chain(&chain).expect("persist signed event"),
        1
    );
    let restored = ledger
        .load_with_trusted_keys(&trusted_keys)
        .expect("signed ledger can be replayed");

    assert_eq!(restored.events(), chain.events());
    assert!(restored.events()[0].ed25519_signature.is_some());
    assert_eq!(
        restored.verify_signed_with_keys(&trusted_keys),
        chain.verify_signed_with_keys(&trusted_keys)
    );

    fs::remove_file(path).ok();
}

#[test]
fn file_audit_ledger_rejects_length_prefix_inside_utf8_boundary() {
    let path = temp_ledger_path("utf8-boundary");
    let ledger = FileAuditLedger::new(path.clone());
    fs::write(&path, "v2|0|1:é|").expect("write malformed utf8-boundary record");

    assert_eq!(ledger.load(), Err(FileAuditLedgerError::MalformedRecord));

    fs::remove_file(path).ok();
}

fn temp_ledger_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "audit-ledger-{label}-{}-{nanos}.log",
        std::process::id()
    ))
}
