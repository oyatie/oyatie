use oya_foundry_vcs_kernel::{
    ArtifactPointer, ChangeSet, ChangeSetDraft, ChangeSetLineage, CiState, Claim,
    ClaimCompatibility, LeaseState, QueueAwareLease, ReviewState, SymbolId, SymbolLanguage,
    VcsKernelError, VirtualHead, VirtualHeadStatus, claim_compatibility, required_claim_coverage,
};

const BASE_SHA: &str = "0123456789012345678901234567890123456789";

fn symbol(path: &str, name: &str) -> SymbolId {
    SymbolId::new(
        SymbolLanguage::Rust,
        ArtifactPointer::file(path).expect("valid path"),
        name,
    )
    .expect("valid symbol")
}

fn working_claim(id: &str, writes: Vec<SymbolId>, reads: Vec<SymbolId>) -> Claim {
    Claim::new(id, "agent-alpha", "IP-001 fixture", writes, reads, 900)
        .expect("valid claim")
        .grant()
        .start_work()
        .expect("claim can enter work")
}

#[test]
fn same_file_different_symbol_claims_do_not_collide() {
    let left = working_claim(
        "claim_left",
        vec![symbol("crates/a/src/lib.rs", "module::alpha")],
        vec![],
    );
    let right = working_claim(
        "claim_right",
        vec![symbol("crates/a/src/lib.rs", "module::beta")],
        vec![],
    );

    assert_eq!(
        claim_compatibility(&left, &right),
        ClaimCompatibility::Compatible
    );
}

#[test]
fn conflicting_write_claim_is_rejected() {
    let shared = symbol("crates/a/src/lib.rs", "module::alpha");
    let left = working_claim("claim_left", vec![shared.clone()], vec![]);
    let right = working_claim("claim_right", vec![shared], vec![]);

    assert_eq!(
        claim_compatibility(&left, &right),
        ClaimCompatibility::Conflict
    );
}

#[test]
fn whole_file_and_pointer_symbol_ids_do_not_collide() {
    let whole_file = SymbolId::new(
        SymbolLanguage::OpenApi,
        ArtifactPointer::file("contracts/vcs.yaml#openapi-operation:POST /claim").unwrap(),
        "operation::claim",
    )
    .unwrap();
    let pointer = SymbolId::new(
        SymbolLanguage::OpenApi,
        ArtifactPointer::new(
            "contracts/vcs.yaml",
            oya_foundry_vcs_kernel::ArtifactSelectorKind::OpenApiOperation,
            Some("POST /claim".into()),
        )
        .unwrap(),
        "operation::claim",
    )
    .unwrap();

    assert_ne!(whole_file.value, pointer.value);
}

#[test]
fn queue_aware_lease_cannot_override_grit_lock_before_terminal_promotion() {
    let active_claim = working_claim(
        "claim_main",
        vec![symbol("crates/a/src/lib.rs", "module::alpha")],
        vec![],
    );
    let mut lease = QueueAwareLease::from_claim(&active_claim).expect("lease from working claim");

    assert_eq!(
        lease.release_after_terminal_promotion(),
        Err(VcsKernelError::LeaseCannotReleaseBeforeTerminalPromotion)
    );

    lease.submit().expect("submitted");
    lease
        .mark_queue_stable("virtual/main/001")
        .expect("queue stable");
    assert!(lease.allows_next_agent_with_virtual_predecessor("virtual/main/001"));
    lease.mark_virtual_merged().expect("virtual merge");
    lease.mark_merged_dev().expect("dev merge");
    lease.mark_promoted_staging().expect("staging");
    lease.mark_promoted_production().expect("production");
    lease
        .release_after_terminal_promotion()
        .expect("terminal release");

    assert_eq!(lease.state, LeaseState::Released);
}

#[test]
fn changeset_requires_claim_coverage_and_fresh_evidence_ref() {
    let claimed = symbol("crates/oya-foundry-vcs-kernel/src/lib.rs", "ChangeSet");
    let claim = working_claim("claim_main", vec![claimed.clone()], vec![]);
    let mut changeset = ChangeSet::new(ChangeSetDraft {
        id: "cs_ip001".into(),
        agent_id: "agent-alpha".into(),
        target_branch: "main".into(),
        base_sha: BASE_SHA.into(),
        branch_or_workspace_ref: "workspace/agent-alpha".into(),
        patch_id: "patch_ip001".into(),
        write_symbols: vec![claimed],
        read_symbols: vec![],
        touched_files: vec![
            ArtifactPointer::file("crates/oya-foundry-vcs-kernel/src/lib.rs").unwrap(),
        ],
        dependencies: vec![],
        lineage: ChangeSetLineage::new("wi_p00", "ip_001", vec![]).unwrap(),
        evidence_refs: vec![".omc/evidence/gitops-vcs/ip-001-claim-kernel.json".into()],
    })
    .expect("valid changeset");

    required_claim_coverage(&changeset, &claim).expect("claim covers changeset writes");
    changeset.attach_review(ReviewState::Approved);
    changeset.attach_ci(CiState::Passed);
    changeset.mark_ready_for_queue().expect("admissible");
}

#[test]
fn pointer_claim_does_not_cover_whole_file_touched_artifact() {
    let pointer = ArtifactPointer::new(
        "contracts/vcs.yaml",
        oya_foundry_vcs_kernel::ArtifactSelectorKind::OpenApiOperation,
        Some("POST /claim".into()),
    )
    .unwrap();
    let claimed = SymbolId::new(SymbolLanguage::OpenApi, pointer, "operation::claim").unwrap();
    let claim = working_claim("claim_main", vec![claimed.clone()], vec![]);
    let changeset = ChangeSet::new(ChangeSetDraft {
        id: "cs_ip001".into(),
        agent_id: "agent-alpha".into(),
        target_branch: "main".into(),
        base_sha: BASE_SHA.into(),
        branch_or_workspace_ref: "workspace/agent-alpha".into(),
        patch_id: "patch_ip001".into(),
        write_symbols: vec![claimed],
        read_symbols: vec![],
        touched_files: vec![ArtifactPointer::file("contracts/vcs.yaml").unwrap()],
        dependencies: vec![],
        lineage: ChangeSetLineage::new("wi_p00", "ip_001", vec![]).unwrap(),
        evidence_refs: vec![".omc/evidence/gitops-vcs/ip-001-claim-kernel.json".into()],
    })
    .expect("valid shape but pointer claim cannot cover whole file");

    assert_eq!(
        required_claim_coverage(&changeset, &claim),
        Err(VcsKernelError::UnclaimedTouchedArtifact)
    );
}

#[test]
fn unclaimed_touched_file_is_rejected() {
    let claimed = symbol("crates/oya-foundry-vcs-kernel/src/lib.rs", "ChangeSet");
    let claim = working_claim("claim_main", vec![claimed.clone()], vec![]);
    let changeset = ChangeSet::new(ChangeSetDraft {
        id: "cs_ip001".into(),
        agent_id: "agent-alpha".into(),
        target_branch: "main".into(),
        base_sha: BASE_SHA.into(),
        branch_or_workspace_ref: "workspace/agent-alpha".into(),
        patch_id: "patch_ip001".into(),
        write_symbols: vec![claimed],
        read_symbols: vec![],
        touched_files: vec![ArtifactPointer::file("crates/other/src/lib.rs").unwrap()],
        dependencies: vec![],
        lineage: ChangeSetLineage::new("wi_p00", "ip_001", vec![]).unwrap(),
        evidence_refs: vec![".omc/evidence/gitops-vcs/ip-001-claim-kernel.json".into()],
    })
    .expect("valid shape but uncovered touched file");

    assert_eq!(
        required_claim_coverage(&changeset, &claim),
        Err(VcsKernelError::UnclaimedTouchedArtifact)
    );
}

#[test]
fn virtual_head_is_review_build_projection_only() {
    let mut virtual_head = VirtualHead::new(
        "main",
        vec!["cs_ip001".into()],
        "main@base",
        "virtual/main/ip001",
        vec!["cargo-test-ip001".into()],
    )
    .expect("valid virtual head");

    assert!(virtual_head.is_projection_only());
    assert_eq!(virtual_head.status, VirtualHeadStatus::Pending);
    virtual_head
        .invalidate_for("base advanced")
        .expect("can invalidate projection");
    assert_eq!(virtual_head.status, VirtualHeadStatus::Invalidated);
    assert_eq!(virtual_head.invalidated_by, vec!["base advanced"]);
}
