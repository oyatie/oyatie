use std::fs;
use std::path::{Path, PathBuf};

use oya_governance_byok_disambiguation::{
    ByokClassification, ByokViolationKind, EnforcementStatus, RULE_ID, enforce_byok_disambiguation,
};

#[test]
fn accepts_provider_byok_reference() {
    let root = fixture_root("byok-pass-provider");
    write(
        &root,
        "docs/provider.md",
        "Provider-BYOK (ADR-0255 §D-4) keeps external provider credentials in the sidecar.",
    );

    let outcome = enforce_byok_disambiguation(&root).expect("check should run");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(
        outcome.references[0].classification,
        ByokClassification::ProviderByok
    );
}

#[test]
fn accepts_encryption_byok_reference() {
    let root = fixture_root("byok-pass-encryption");
    write(
        &root,
        "microservices/cloud-kms/README.md",
        "Encryption-BYOK (ADR-0251 §D-10) imports tenant KEKs into cloud-kms.",
    );

    let outcome = enforce_byok_disambiguation(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(
        outcome.references[0].classification,
        ByokClassification::EncryptionByok
    );
}

#[test]
fn rejects_bare_byok_reference() {
    let root = fixture_root("byok-fail-bare");
    write(
        &root,
        "docs/security.md",
        "Enterprise tenants can enable BYOK during onboarding.",
    );

    let outcome = enforce_byok_disambiguation(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(outcome.violations.len(), 1);
    assert_eq!(outcome.violations[0].kind, ByokViolationKind::AmbiguousByok);
    assert_eq!(outcome.violations[0].line, 1);
}

#[test]
fn rejects_collapsed_provider_and_encryption_context() {
    let root = fixture_root("byok-fail-collapsed");
    write(
        &root,
        "docs/security.md",
        "BYOK uses ADR-0255 §D-4 provider credentials and tenant KEK import into KMS.",
    );

    let outcome = enforce_byok_disambiguation(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(
        outcome.violations[0].kind,
        ByokViolationKind::CollapsedProviderAndEncryptionByok
    );
}

#[test]
fn accepts_explicit_contrast_that_names_both_terms() {
    let root = fixture_root("byok-pass-both");
    write(
        &root,
        "microservices/mail/IP-001.md",
        "Provider-BYOK handles external API credentials; encryption-BYOK handles tenant KMS keys.",
    );

    let outcome = enforce_byok_disambiguation(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(
        outcome.references[0].classification,
        ByokClassification::ExplicitProviderAndEncryptionContrast
    );
}

fn fixture_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-governance-byok-{}-{}",
        name,
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture root");
    }
    fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture file has parent"))
        .expect("create fixture parent");
    fs::write(path, content).expect("write fixture file");
}
