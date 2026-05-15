use oya_foundry_vcs_kernel::{ArtifactPointer, SymbolLanguage};
use oya_foundry_vcs_test_standard_gate_kernel::{
    AccountingKind, AccountingRecord, AdmissionInput, DeployEdge, EvidenceDisposition,
    EvidenceRecord, EvidenceResult, FixupReason, FreshnessPolicy, SemanticChange, SurfaceKind,
    TestSuiteRegistry, TestTier, evaluate_admission, resolve_required_tiers,
};

fn artifact(path: &str) -> ArtifactPointer {
    ArtifactPointer::file(path).expect("valid artifact")
}

fn change(surface: SurfaceKind, path: &str, symbol: &str) -> SemanticChange {
    SemanticChange::new(
        artifact(path),
        SymbolLanguage::Rust,
        surface,
        "ip008-fixture",
        symbol,
    )
    .expect("valid change")
}

fn evidence(suite: &str, tier: TestTier) -> EvidenceRecord {
    EvidenceRecord::required_pass(suite, tier, 3, 10_000, format!("{suite} {}", tier.as_str()))
        .expect("valid evidence")
}

fn admission(changes: Vec<SemanticChange>, evidence: Vec<EvidenceRecord>) -> AdmissionInput {
    AdmissionInput {
        changes,
        registry: TestSuiteRegistry::oyatie_default(),
        evidence,
        accounting: Vec::new(),
        freshness_policy: FreshnessPolicy::new(3, 10_100),
    }
}

#[test]
fn deploy_edge_and_semantic_diff_require_e2e_contract_and_property_tiers() {
    let required = resolve_required_tiers(&[
        change(
            SurfaceKind::ContractSchema,
            "contracts/openapi/demo.yaml",
            "POST /demo",
        )
        .contract("demo-api-v1")
        .parser_or_serializer(),
        change(
            SurfaceKind::Workflow,
            "crates/demo/src/lib.rs",
            "demo::checkout",
        )
        .with_deploy_edge(DeployEdge::GitOpsPromotion),
    ]);

    assert!(required.contains(&TestTier::Integration));
    assert!(required.contains(&TestTier::Contract));
    assert!(required.contains(&TestTier::Property));
    assert!(required.contains(&TestTier::E2e));
}

#[test]
fn controller_admission_blocks_missing_required_e2e_evidence_with_fixup() {
    let decision = evaluate_admission(admission(
        vec![
            change(
                SurfaceKind::Workflow,
                "crates/demo/src/lib.rs",
                "demo::checkout",
            )
            .tenant_visible(),
        ],
        vec![evidence("rust-nextest-workspace", TestTier::Unit)],
    ));

    assert!(!decision.accepted);
    assert!(decision.fixup_tasks.iter().any(|task| {
        task.reason == FixupReason::MissingRequiredEvidence
            && task.tier == Some(TestTier::E2e)
            && task.suite_id.as_deref() == Some("workflow-e2e")
            && task.blocking
    }));
}

#[test]
fn stale_after_rebase_evidence_blocks_even_when_result_passed() {
    let stale = EvidenceRecord::required_pass(
        "rust-nextest-workspace",
        TestTier::Unit,
        2,
        10_000,
        "cargo nextest run --workspace --all-features --no-fail-fast",
    )
    .unwrap();

    let decision = evaluate_admission(admission(
        vec![change(
            SurfaceKind::Kernel,
            "crates/demo/src/lib.rs",
            "demo::pure",
        )],
        vec![stale],
    ));

    assert!(!decision.accepted);
    assert!(
        decision
            .fixup_tasks
            .iter()
            .any(|task| task.reason == FixupReason::StaleRequiredEvidence)
    );
}

#[test]
fn advisory_required_tier_is_blocking_not_a_pass() {
    let advisory = EvidenceRecord::new(
        "rust-nextest-workspace",
        TestTier::Unit,
        EvidenceDisposition::Advisory,
        EvidenceResult::Pass,
        3,
        10_000,
        "cargo test -p ip008-fixture",
    )
    .unwrap();

    let decision = evaluate_admission(admission(
        vec![change(
            SurfaceKind::Kernel,
            "crates/demo/src/lib.rs",
            "demo::pure",
        )],
        vec![advisory],
    ));

    assert!(!decision.accepted);
    assert!(
        decision
            .fixup_tasks
            .iter()
            .any(|task| { task.reason == FixupReason::AdvisoryEvidenceCannotSatisfyRequiredTier })
    );
}

#[test]
fn generated_client_accounting_is_required_for_admission() {
    let generated = change(
        SurfaceKind::GeneratedClient,
        "crates/demo-client/src/generated.rs",
        "demo_client::generated",
    )
    .generated_client_for("demo-api-v1");
    let mut admitted = admission(
        vec![generated],
        vec![
            evidence("rust-nextest-workspace", TestTier::Integration),
            evidence("rust-nextest-workspace", TestTier::Contract),
            evidence("generated-client-contract-parity", TestTier::Integration),
            evidence("generated-client-contract-parity", TestTier::Contract),
        ],
    );
    let rejected = evaluate_admission(admitted.clone());
    assert!(!rejected.accepted);
    assert!(
        rejected
            .fixup_tasks
            .iter()
            .any(|task| task.reason == FixupReason::UnaccountedGeneratedClient)
    );

    admitted.accounting.push(AccountingRecord {
        id: "demo-api-v1".into(),
        kind: AccountingKind::GeneratedClient,
        source_artifact: artifact("contracts/openapi/demo.yaml"),
        target_artifact: artifact("crates/demo-client/src/generated.rs"),
    });
    let accepted = evaluate_admission(admitted);
    assert!(accepted.accepted, "fixups: {:?}", accepted.fixup_tasks);
}

#[test]
fn generated_client_accounting_must_reference_changed_artifact() {
    let generated = change(
        SurfaceKind::GeneratedClient,
        "crates/demo-client/src/generated.rs",
        "demo_client::generated",
    )
    .generated_client_for("demo-api-v1");
    let mut admission_input = admission(
        vec![generated],
        vec![
            evidence("rust-nextest-workspace", TestTier::Integration),
            evidence("rust-nextest-workspace", TestTier::Contract),
            evidence("generated-client-contract-parity", TestTier::Integration),
            evidence("generated-client-contract-parity", TestTier::Contract),
        ],
    );
    admission_input.accounting.push(AccountingRecord {
        id: "demo-api-v1".into(),
        kind: AccountingKind::GeneratedClient,
        source_artifact: artifact("contracts/openapi/demo.yaml"),
        target_artifact: artifact("crates/other-client/src/generated.rs"),
    });

    let rejected = evaluate_admission(admission_input);

    assert!(!rejected.accepted);
    assert!(
        rejected
            .fixup_tasks
            .iter()
            .any(|task| task.reason == FixupReason::UnaccountedGeneratedClient)
    );
}
