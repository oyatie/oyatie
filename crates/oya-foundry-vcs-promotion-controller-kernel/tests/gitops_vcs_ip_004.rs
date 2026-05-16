// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use oya_foundry_vcs_changebundle_kernel::{
    BundleAttestation, ChangeBundle, ChangeBundleDraft, Digest, EvidenceKind,
    EvidenceRecord as BundleEvidenceRecord, EvidenceResult as BundleEvidenceResult, KgEdgeRecord,
    Provenance, SemanticDiffSummary,
};
use oya_foundry_vcs_kernel::{
    ArtifactPointer, ChangeSet, ChangeSetDraft, ChangeSetLineage, Claim, SymbolId, SymbolLanguage,
};
use oya_foundry_vcs_promotion_controller_kernel::{
    ArgoGitOpsContractFixture, CiContractFixture, Environment, EnvironmentHealth,
    EnvironmentStatus, FreshnessEnvelope, GitHubActionsContractFixture, IdempotencyKey,
    PromotionController, PromotionControllerState, PromotionError, PromotionPolicy,
    PromotionRequest, ProviderAvailability, ProviderContractEvidence, ProviderDecision,
    ProviderEvidenceSet, ProviderKind, TrivyContractFixture,
};
use oya_foundry_vcs_test_standard_gate_kernel::{AdmissionDecision, FixupTask};

const NOW: u64 = 1_800_000_000;
const BASE_SHA: &str = "0123456789012345678901234567890123456789";
const SHA256_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn digest() -> Digest {
    Digest::new("sha256", SHA256_A).unwrap()
}

fn symbol() -> SymbolId {
    SymbolId::new(
        SymbolLanguage::Rust,
        ArtifactPointer::file("crates/oya-foundry-vcs-promotion-controller/src/lib.rs").unwrap(),
        "PromotionController",
    )
    .unwrap()
}

fn claim(sym: SymbolId) -> Claim {
    Claim::new(
        "claim_ip004",
        "agent-ip004",
        "M-CC-P00-IP-004 promotion controller",
        vec![sym],
        vec![],
        900,
    )
    .unwrap()
    .grant()
    .start_work()
    .unwrap()
}

fn changeset(sym: SymbolId) -> ChangeSet {
    ChangeSet::new(ChangeSetDraft {
        id: "cs_ip004".into(),
        agent_id: "agent-ip004".into(),
        target_branch: "main".into(),
        base_sha: BASE_SHA.into(),
        branch_or_workspace_ref: "workspace/ip004".into(),
        patch_id: "patch_ip004".into(),
        write_symbols: vec![sym],
        read_symbols: vec![],
        touched_files: vec![
            ArtifactPointer::file("crates/oya-foundry-vcs-promotion-controller/src/lib.rs")
                .unwrap(),
        ],
        dependencies: vec![],
        lineage: ChangeSetLineage::new("wi_m_cc_p00", "ip_004", vec![]).unwrap(),
        evidence_refs: vec!["evidence/gitops-vcs/ip-004-controller.json".into()],
    })
    .unwrap()
}

fn bundle() -> ChangeBundle {
    let sym = symbol();
    let changeset = changeset(sym.clone());
    let manifest_digest = digest();
    let evidence = vec![
        BundleEvidenceRecord::new(
            "ev_unit_ip004",
            EvidenceKind::UnitTest,
            "rustc --test promotion controller",
            BundleEvidenceResult::Passed,
            manifest_digest.clone(),
            NOW - 30,
            NOW + 86_000,
            "unit",
        )
        .unwrap(),
        BundleEvidenceRecord::new(
            "ev_build_ip004",
            EvidenceKind::Build,
            "rustc lib promotion controller",
            BundleEvidenceResult::Passed,
            manifest_digest.clone(),
            NOW - 30,
            NOW + 86_000,
            "build",
        )
        .unwrap(),
    ];
    ChangeBundle::new(ChangeBundleDraft {
        id: "cb_ip004".into(),
        changeset,
        claims: vec![claim(sym.clone())],
        manifest_digest: manifest_digest.clone(),
        attestation: BundleAttestation::new(
            "ed25519",
            "key-ip004",
            "signature-ip004",
            manifest_digest.clone(),
        )
        .unwrap(),
        provenance: Provenance::new(
            "agent-ip004",
            vec!["claim_ip004".into()],
            manifest_digest.clone(),
            "EVT-IP004",
            NOW - 60,
            "main",
            "workspace/ip004",
            vec!["build".into()],
            vec!["deploy".into()],
        )
        .unwrap(),
        semantic_diff: SemanticDiffSummary::new(
            vec![sym],
            vec![
                ArtifactPointer::file("crates/oya-foundry-vcs-promotion-controller/src/lib.rs")
                    .unwrap(),
            ],
            vec!["unit".into(), "integration".into(), "e2e".into()],
            manifest_digest,
            "promotion controller state machine",
        )
        .unwrap(),
        evidence,
        kg_edges: vec![KgEdgeRecord::new("bundle", "promotion", "validated_by").unwrap()],
    })
    .unwrap()
}

fn accepted_admission() -> AdmissionDecision {
    AdmissionDecision {
        schema_version: 1,
        accepted: true,
        required_tiers: Default::default(),
        required_suites: Vec::new(),
        advisory_evidence: Vec::new(),
        fixup_tasks: Vec::new(),
    }
}

fn provider(
    kind: ProviderKind,
    availability: ProviderAvailability,
    decision: ProviderDecision,
) -> ProviderContractEvidence {
    ProviderContractEvidence::new(
        kind,
        availability,
        decision,
        format!("fixture-{}", kind.as_str()),
        7,
        NOW,
        "fixture",
    )
    .unwrap()
}

fn providers() -> ProviderEvidenceSet {
    ProviderEvidenceSet {
        ci: CiContractFixture {
            provider_evidence: provider(
                ProviderKind::Ci,
                ProviderAvailability::Available,
                ProviderDecision::Passed,
            ),
            build_id: "build-ip004".into(),
        },
        github: GitHubActionsContractFixture {
            provider_evidence: provider(
                ProviderKind::GitHubActions,
                ProviderAvailability::Available,
                ProviderDecision::Passed,
            ),
            workflow_run_id: "gha-ip004".into(),
        },
        trivy: TrivyContractFixture {
            provider_evidence: provider(
                ProviderKind::Trivy,
                ProviderAvailability::Available,
                ProviderDecision::Passed,
            ),
            critical_findings: 0,
            high_findings: 0,
        },
        argo: ArgoGitOpsContractFixture {
            provider_evidence: provider(
                ProviderKind::ArgoGitOps,
                ProviderAvailability::Available,
                ProviderDecision::Passed,
            ),
            application: "oyatie-dev".into(),
            target_revision: "bundle/cb_ip004".into(),
        },
    }
}

fn health() -> Vec<EnvironmentStatus> {
    vec![
        EnvironmentStatus::healthy(Environment::Dev, NOW),
        EnvironmentStatus::healthy(Environment::Staging, NOW),
        EnvironmentStatus::healthy(Environment::Production, NOW),
    ]
}

fn request(key: &str) -> PromotionRequest {
    PromotionRequest {
        request_id: "req-ip004".into(),
        idempotency_key: IdempotencyKey::new(key).unwrap(),
        bundle: bundle(),
        admission: accepted_admission(),
        freshness: FreshnessEnvelope::fresh("index-ip004", 7, NOW - 30, NOW).unwrap(),
        providers: providers(),
        environment_health: health(),
        policy: PromotionPolicy::strict(),
    }
}

#[test]
fn transition_reducer_promotes_bundle_to_reconciled_release_train() {
    let mut controller = PromotionController::new();
    let outcome = controller.promote(request("ip004:happy:path"));

    assert_eq!(outcome.final_state, PromotionControllerState::Reconciled);
    assert_eq!(outcome.release_train.len(), 3);
    assert_eq!(outcome.release_train[0].environment, Environment::Dev);
    assert_eq!(
        outcome.release_train[2].environment,
        Environment::Production
    );
    assert_eq!(outcome.bundle.promotion_evidence.len(), 3);
    assert!(outcome.rejected_reasons.is_empty());
}

#[test]
fn duplicate_promotion_requests_collapse_by_idempotency_key() {
    let mut controller = PromotionController::new();
    let first = controller.promote(request("ip004:duplicate"));
    let second = controller.promote(request("ip004:duplicate"));

    assert!(!first.duplicate_collapsed);
    assert!(second.duplicate_collapsed);
    assert_eq!(first.release_train, second.release_train);
    assert_eq!(second.final_state, PromotionControllerState::Reconciled);
}

#[test]
fn stale_index_cache_admission_evidence_rejects_promotion() {
    let mut stale = request("ip004:stale-index");
    stale.freshness = FreshnessEnvelope::new("index-ip004", 6, 7, NOW - 30, NOW, 86_400).unwrap();

    let outcome = PromotionController::new().promote(stale);

    assert_eq!(outcome.final_state, PromotionControllerState::Rejected);
    assert!(
        outcome
            .rejected_reasons
            .contains(&PromotionError::StaleIndexEvidence)
    );
}

#[test]
fn provider_outage_falls_back_only_when_policy_allows() {
    let mut denied = request("ip004:outage-denied");
    denied.providers.argo.provider_evidence = provider(
        ProviderKind::ArgoGitOps,
        ProviderAvailability::Outage,
        ProviderDecision::Pending,
    );
    let denied_outcome = PromotionController::new().promote(denied);
    assert_eq!(
        denied_outcome.final_state,
        PromotionControllerState::Rejected
    );
    assert!(
        denied_outcome
            .rejected_reasons
            .contains(&PromotionError::ProviderOutageNoFallback(
                ProviderKind::ArgoGitOps
            ))
    );

    let mut allowed = request("ip004:outage-allowed");
    allowed.policy = PromotionPolicy::allow_degraded_native_manual();
    allowed.providers.argo.provider_evidence = provider(
        ProviderKind::ArgoGitOps,
        ProviderAvailability::Outage,
        ProviderDecision::Pending,
    );
    let allowed_outcome = PromotionController::new().promote(allowed);
    assert_eq!(
        allowed_outcome.final_state,
        PromotionControllerState::Reconciled
    );
    assert!(!allowed_outcome.degraded_path.is_empty());
    assert!(allowed_outcome.release_train.iter().all(|hop| hop.degraded));
}

#[test]
fn provider_slot_kind_mismatch_rejects_promotion() {
    let mut swapped = request("ip004:swapped-provider-slot");
    swapped.providers.github.provider_evidence = provider(
        ProviderKind::Ci,
        ProviderAvailability::Available,
        ProviderDecision::Passed,
    );

    let outcome = PromotionController::new().promote(swapped);

    assert_eq!(outcome.final_state, PromotionControllerState::Rejected);
    assert!(
        outcome
            .rejected_reasons
            .contains(&PromotionError::ProviderSlotMismatch {
                expected: ProviderKind::GitHubActions,
                actual: ProviderKind::Ci,
            })
    );
}

#[test]
fn rollback_and_readiness_states_are_explicit() {
    let key = IdempotencyKey::new("ip004:rollback").unwrap();
    let mut controller = PromotionController::new();
    let promoted = controller.promote(request(key.as_str()));
    assert_eq!(promoted.final_state, PromotionControllerState::Reconciled);

    let rolled_back = controller
        .rollback(key, "production health regression")
        .unwrap();
    assert_eq!(
        rolled_back.final_state,
        PromotionControllerState::RolledBack
    );
    assert!(
        rolled_back
            .transitions
            .iter()
            .any(|transition| transition.to == PromotionControllerState::RollbackRequested)
    );
}

#[test]
fn unhealthy_environment_and_admission_fixup_reject() {
    let mut blocked = request("ip004:blocked");
    blocked.environment_health[2] = EnvironmentStatus::new(
        Environment::Production,
        EnvironmentHealth::Unhealthy,
        NOW,
        "production smoke failing",
    )
    .unwrap();
    blocked.admission = AdmissionDecision {
        accepted: false,
        fixup_tasks: vec![FixupTask {
            task_id: "fixup_1".into(),
            reason: oya_foundry_vcs_test_standard_gate_kernel::FixupReason::MissingRequiredEvidence,
            tier: None,
            suite_id: None,
            affected_refs: vec!["workflow-e2e".into()],
            blocking: true,
        }],
        ..accepted_admission()
    };

    let outcome = PromotionController::new().promote(blocked);

    assert_eq!(outcome.final_state, PromotionControllerState::Rejected);
    assert!(
        outcome
            .rejected_reasons
            .contains(&PromotionError::EnvironmentNotReady(
                Environment::Production
            ))
    );
}
