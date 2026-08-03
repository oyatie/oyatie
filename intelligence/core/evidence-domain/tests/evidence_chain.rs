// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use oya_data_boundary_kernel::{DataClass, PrivacyDataClass, privacy_data_classes_from};
use intelligence_evidence_domain::{EvidenceChain, EvidenceError, EvidenceKind};

fn privacy_data_classes(data_classes: Vec<DataClass>) -> Vec<PrivacyDataClass> {
    privacy_data_classes_from(&data_classes).expect("test fixture uses privacy data classes")
}

#[test]
fn evidence_chain_appends_run_level_records_and_verifies_links() {
    let mut chain = EvidenceChain::default();
    let evidence = chain
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
        .expect("evidence record is valid");

    assert_eq!(evidence.evidence_id.value, "ev_000000000001");
    assert_eq!(evidence.prev_hash.value, "GENESIS");
    assert_eq!(evidence.data_class.value, DataClass::InternalOnly);
    assert_eq!(
        evidence.touched_privacy_data_classes(),
        privacy_data_classes(vec![DataClass::InternalOnly]).as_slice()
    );
    assert_eq!(
        evidence.legacy_touched_data_classes(),
        vec![DataClass::InternalOnly]
    );
    #[allow(deprecated)]
    {
        assert_eq!(
            evidence.touched_data_classes(),
            evidence.legacy_touched_data_classes()
        );
    }
    assert!(chain.verify());
    assert_eq!(chain.root_hash(), Some(evidence.hash.value.as_str()));
}

#[test]
fn evidence_chain_classifies_record_as_most_restrictive_touched_data_class() {
    let mut chain = EvidenceChain::default();
    let evidence = chain
        .append(
            "ten_alpha".into(),
            "run_000000000001".into(),
            Some("step_000000000001".into()),
            "cap.demo.invoke".into(),
            EvidenceKind::DataFlow,
            fields(("audit_event_hash", "fnv1a64:def")),
            privacy_data_classes(vec![
                DataClass::Public,
                DataClass::PiiIdentifying,
                DataClass::SensitivePipaArticle23,
            ]),
            1_001,
        )
        .expect("evidence record is valid");

    assert_eq!(evidence.data_class.value, DataClass::SensitivePipaArticle23);
    assert!(chain.verify());
}

#[test]
fn evidence_chain_rejects_operational_or_subject_markers_as_touched_privacy_classes() {
    for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
        let mut chain = EvidenceChain::default();
        assert_eq!(
            chain.try_append_legacy_data_classes_touched(
                "ten_alpha".into(),
                "run_000000000001".into(),
                None,
                "cap.demo.invoke".into(),
                EvidenceKind::DataFlow,
                fields(("audit_event_hash", "fnv1a64:invalid")),
                vec![data_class],
                1_001,
            ),
            Err(EvidenceError::InvalidDataClass),
            "{data_class:?} must not be accepted as a privacy touched class"
        );
    }
}

#[test]
fn evidence_chain_preserves_non_public_data_class_regressions() {
    let cases = [
        (vec![DataClass::PiiIdentifying], DataClass::PiiIdentifying),
        (
            vec![DataClass::FinancialRegulatedCredit, DataClass::InternalOnly],
            DataClass::FinancialRegulatedCredit,
        ),
        (vec![DataClass::Public, DataClass::Phi], DataClass::Phi),
    ];

    for (touched, expected) in cases {
        let mut chain = EvidenceChain::default();
        let evidence = chain
            .append(
                "ten_alpha".into(),
                "run_000000000001".into(),
                None,
                "cap.demo.invoke".into(),
                EvidenceKind::DataFlow,
                fields(("audit_event_hash", "fnv1a64:nonpublic")),
                privacy_data_classes(touched),
                1_002,
            )
            .expect("evidence record is valid");

        assert_eq!(evidence.data_class.value, expected);
        assert!(chain.verify());
    }
}

#[test]
fn evidence_chain_detects_divergent_history_and_validates_shape() {
    assert_eq!(
        EvidenceChain::default().append(
            "tenant-alpha".into(),
            "run_000000000001".into(),
            None,
            "cap.demo.invoke".into(),
            EvidenceKind::CapabilityInvocation,
            fields(("audit_event_hash", "fnv1a64:abc")),
            privacy_data_classes(vec![DataClass::InternalOnly]),
            1_000,
        ),
        Err(EvidenceError::InvalidTenantId)
    );

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
    let mut tampered = chain.records().to_vec();
    tampered[0]
        .fields
        .value
        .insert("extra".into(), "tamper".into());
    let replayed = EvidenceChain::from_records(tampered).expect("shape remains parseable");
    assert!(!replayed.verify());
}

#[test]
fn evidence_chain_rejects_replayed_records_with_incorrect_derived_data_class() {
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

    let mut records = chain.records().to_vec();
    records[0].data_class.value = DataClass::Public;

    assert_eq!(
        EvidenceChain::from_records(records),
        Err(EvidenceError::InvalidDataClass)
    );
}

#[test]
fn evidence_chain_rejects_malformed_evidence_ids_on_replay() {
    let malformed_ids = [
        "ev_1",
        "ev_000000000000",
        "ev_00000000001a",
        "EV_000000000001",
        "000000000001",
        "ev_0000000000010",
    ];

    for malformed_id in malformed_ids {
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
        let mut records = chain.records().to_vec();
        records[0].evidence_id.value = malformed_id.into();

        assert_eq!(
            EvidenceChain::from_records(records),
            Err(EvidenceError::InvalidEvidenceId),
            "{malformed_id} must be rejected"
        );
    }
}

fn fields(field: (&str, &str)) -> BTreeMap<String, String> {
    BTreeMap::from([(field.0.to_string(), field.1.to_string())])
}
