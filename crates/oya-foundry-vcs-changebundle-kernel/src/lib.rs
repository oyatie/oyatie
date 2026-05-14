//! ChangeBundle attestation and provenance kernel.
//!
//! This crate is deliberately pure: it models the signed bundle shape and
//! admission invariants used by Oya VCS controllers, but it never mutates a
//! protected ref, shells out to Git/GitHub, or verifies a real cryptographic
//! signature. Adapter/controller layers perform I/O; this kernel validates the
//! deterministic attestation fields that those layers supply.

use std::collections::BTreeSet;
use std::fmt;

use oya_foundry_vcs_kernel::{
    ArtifactPointer, ChangeSet, Claim, ClaimMode, ClaimState, SymbolId, SymbolLock, VcsKernelError,
    required_claim_coverage,
};

pub const CHANGE_BUNDLE_SCHEMA_VERSION: u32 = 1;
pub const MAX_EVIDENCE_AGE_SECONDS: u64 = 86_400;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Digest {
    pub algorithm: String, // data_class: INTERNAL_ONLY
    pub value: String,     // data_class: INTERNAL_ONLY
}

impl Digest {
    pub fn new(
        algorithm: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, BundleError> {
        let algorithm = normalize_non_empty(algorithm.into(), BundleError::UnsupportedDigest)?;
        let value = normalize_non_empty(value.into(), BundleError::InvalidDigest)?;
        if !matches!(algorithm.as_str(), "sha256" | "sha512") {
            return Err(BundleError::UnsupportedDigest);
        }
        if !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return Err(BundleError::InvalidDigest);
        }
        let valid_len = (algorithm == "sha256" && value.len() == 64)
            || (algorithm == "sha512" && value.len() == 128);
        if !valid_len {
            return Err(BundleError::InvalidDigest);
        }
        Ok(Self { algorithm, value })
    }

    pub fn stable_ref(&self) -> String {
        format!("{}:{}", self.algorithm, self.value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleAttestation {
    pub algorithm: String,     // data_class: INTERNAL_ONLY
    pub key_id: String,        // data_class: INTERNAL_ONLY
    pub signature: String,     // data_class: INTERNAL_ONLY
    pub signed_digest: Digest, // data_class: INTERNAL_ONLY
}

impl BundleAttestation {
    pub fn new(
        algorithm: impl Into<String>,
        key_id: impl Into<String>,
        signature: impl Into<String>,
        signed_digest: Digest,
    ) -> Result<Self, BundleError> {
        let algorithm = normalize_non_empty(algorithm.into(), BundleError::UnsupportedSignature)?;
        if !matches!(
            algorithm.as_str(),
            "ed25519" | "ecdsa-p256-sha256" | "cosign-keyless-v1"
        ) {
            return Err(BundleError::UnsupportedSignature);
        }
        Ok(Self {
            algorithm,
            key_id: normalize_non_empty(key_id.into(), BundleError::MissingSignature)?,
            signature: normalize_non_empty(signature.into(), BundleError::MissingSignature)?,
            signed_digest,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EvidenceKind {
    UnitTest,
    IntegrationTest,
    Build,
    Deploy,
    Review,
    Policy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceResult {
    Passed,
    Failed,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRecord {
    pub id: String,              // data_class: INTERNAL_ONLY
    pub kind: EvidenceKind,      // data_class: INTERNAL_ONLY
    pub command: String,         // data_class: INTERNAL_ONLY
    pub result: EvidenceResult,  // data_class: INTERNAL_ONLY
    pub artifact_digest: Digest, // data_class: INTERNAL_ONLY
    pub observed_at_epoch: u64,  // data_class: INTERNAL_ONLY
    pub expires_at_epoch: u64,   // data_class: INTERNAL_ONLY
    pub lineage_step: String,    // data_class: INTERNAL_ONLY
}

impl EvidenceRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        kind: EvidenceKind,
        command: impl Into<String>,
        result: EvidenceResult,
        artifact_digest: Digest,
        observed_at_epoch: u64,
        expires_at_epoch: u64,
        lineage_step: impl Into<String>,
    ) -> Result<Self, BundleError> {
        if observed_at_epoch == 0 || expires_at_epoch <= observed_at_epoch {
            return Err(BundleError::StaleEvidence);
        }
        Ok(Self {
            id: validate_prefixed(id.into(), "ev_", BundleError::InvalidEvidence)?,
            kind,
            command: normalize_non_empty(command.into(), BundleError::InvalidEvidence)?,
            result,
            artifact_digest,
            observed_at_epoch,
            expires_at_epoch,
            lineage_step: normalize_non_empty(lineage_step.into(), BundleError::InvalidEvidence)?,
        })
    }

    pub fn is_fresh_at(&self, now_epoch: u64) -> bool {
        self.result == EvidenceResult::Passed
            && self.observed_at_epoch <= now_epoch
            && now_epoch <= self.expires_at_epoch
            && now_epoch.saturating_sub(self.observed_at_epoch) <= MAX_EVIDENCE_AGE_SECONDS
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    pub agent_id: String,            // data_class: INTERNAL_ONLY
    pub claim_ids: Vec<String>,      // data_class: INTERNAL_ONLY
    pub source_digest: Digest,       // data_class: INTERNAL_ONLY
    pub audit_event_id: String,      // data_class: INTERNAL_ONLY
    pub created_at_epoch: u64,       // data_class: INTERNAL_ONLY
    pub base_ref: String,            // data_class: INTERNAL_ONLY
    pub workspace_ref: String,       // data_class: INTERNAL_ONLY
    pub build_lineage: Vec<String>,  // data_class: INTERNAL_ONLY
    pub deploy_lineage: Vec<String>, // data_class: INTERNAL_ONLY
}

impl Provenance {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        agent_id: impl Into<String>,
        claim_ids: Vec<String>,
        source_digest: Digest,
        audit_event_id: impl Into<String>,
        created_at_epoch: u64,
        base_ref: impl Into<String>,
        workspace_ref: impl Into<String>,
        build_lineage: Vec<String>,
        deploy_lineage: Vec<String>,
    ) -> Result<Self, BundleError> {
        if claim_ids.is_empty() || created_at_epoch == 0 || build_lineage.is_empty() {
            return Err(BundleError::InvalidProvenance);
        }
        for claim_id in &claim_ids {
            validate_prefixed(claim_id.clone(), "claim_", BundleError::InvalidProvenance)?;
        }
        Ok(Self {
            agent_id: normalize_non_empty(agent_id.into(), BundleError::InvalidProvenance)?,
            claim_ids,
            source_digest,
            audit_event_id: validate_prefixed(
                audit_event_id.into(),
                "EVT-",
                BundleError::InvalidProvenance,
            )?,
            created_at_epoch,
            base_ref: normalize_non_empty(base_ref.into(), BundleError::InvalidProvenance)?,
            workspace_ref: normalize_non_empty(
                workspace_ref.into(),
                BundleError::InvalidProvenance,
            )?,
            build_lineage,
            deploy_lineage,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticDiffSummary {
    pub touched_symbols: Vec<SymbolId>, // data_class: INTERNAL_ONLY
    pub touched_files: Vec<ArtifactPointer>, // data_class: INTERNAL_ONLY
    pub affected_test_tiers: Vec<String>, // data_class: INTERNAL_ONLY
    pub dependency_fingerprint: Digest, // data_class: INTERNAL_ONLY
    pub human_summary: String,          // data_class: INTERNAL_ONLY
}

impl SemanticDiffSummary {
    pub fn new(
        touched_symbols: Vec<SymbolId>,
        touched_files: Vec<ArtifactPointer>,
        affected_test_tiers: Vec<String>,
        dependency_fingerprint: Digest,
        human_summary: impl Into<String>,
    ) -> Result<Self, BundleError> {
        if touched_symbols.is_empty() || touched_files.is_empty() || affected_test_tiers.is_empty()
        {
            return Err(BundleError::InvalidSemanticDiff);
        }
        Ok(Self {
            touched_symbols,
            touched_files,
            affected_test_tiers,
            dependency_fingerprint,
            human_summary: normalize_non_empty(
                human_summary.into(),
                BundleError::InvalidSemanticDiff,
            )?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KgEdgeRecord {
    pub from: String,      // data_class: INTERNAL_ONLY
    pub to: String,        // data_class: INTERNAL_ONLY
    pub edge_type: String, // data_class: INTERNAL_ONLY
}

impl KgEdgeRecord {
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        edge_type: impl Into<String>,
    ) -> Result<Self, BundleError> {
        Ok(Self {
            from: normalize_non_empty(from.into(), BundleError::InvalidKgEdge)?,
            to: normalize_non_empty(to.into(), BundleError::InvalidKgEdge)?,
            edge_type: normalize_non_empty(edge_type.into(), BundleError::InvalidKgEdge)?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromotionStatus {
    Requested,
    Published,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromotionEvidence {
    pub request_id: String,          // data_class: INTERNAL_ONLY
    pub environment: String,         // data_class: INTERNAL_ONLY
    pub status: PromotionStatus,     // data_class: INTERNAL_ONLY
    pub evidence_ref: String,        // data_class: INTERNAL_ONLY
    pub protected_ref_mutated: bool, // data_class: INTERNAL_ONLY
}

impl PromotionEvidence {
    pub fn new(
        request_id: impl Into<String>,
        environment: impl Into<String>,
        status: PromotionStatus,
        evidence_ref: impl Into<String>,
        protected_ref_mutated: bool,
    ) -> Result<Self, BundleError> {
        Ok(Self {
            request_id: validate_prefixed(
                request_id.into(),
                "promo_",
                BundleError::InvalidPromotion,
            )?,
            environment: normalize_non_empty(environment.into(), BundleError::InvalidPromotion)?,
            status,
            evidence_ref: normalize_non_empty(evidence_ref.into(), BundleError::InvalidPromotion)?,
            protected_ref_mutated,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BundleStatus {
    Draft,
    Accepted,
    Published,
    Quarantined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationDecision {
    Accepted,
    Quarantined,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    pub decision: ValidationDecision, // data_class: INTERNAL_ONLY
    pub reasons: Vec<BundleError>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeBundle {
    pub id: String,                                 // data_class: INTERNAL_ONLY
    pub changeset: ChangeSet,                       // data_class: INTERNAL_ONLY
    pub claims: Vec<Claim>,                         // data_class: INTERNAL_ONLY
    pub symbol_locks: Vec<SymbolLock>,              // data_class: INTERNAL_ONLY
    pub manifest_digest: Digest,                    // data_class: INTERNAL_ONLY
    pub attestation: BundleAttestation,             // data_class: INTERNAL_ONLY
    pub provenance: Provenance,                     // data_class: INTERNAL_ONLY
    pub semantic_diff: SemanticDiffSummary,         // data_class: INTERNAL_ONLY
    pub evidence: Vec<EvidenceRecord>,              // data_class: INTERNAL_ONLY
    pub kg_edges: Vec<KgEdgeRecord>,                // data_class: INTERNAL_ONLY
    pub promotion_evidence: Vec<PromotionEvidence>, // data_class: INTERNAL_ONLY
    pub status: BundleStatus,                       // data_class: INTERNAL_ONLY
    pub schema_version: u32,                        // data_class: INTERNAL_ONLY
}

impl ChangeBundle {
    pub fn new(draft: ChangeBundleDraft) -> Result<Self, BundleError> {
        let id = validate_prefixed(draft.id, "cb_", BundleError::InvalidBundleId)?;
        if draft.claims.is_empty() || draft.evidence.is_empty() || draft.kg_edges.is_empty() {
            return Err(BundleError::MissingBundleEvidence);
        }
        if draft.attestation.signed_digest != draft.manifest_digest {
            return Err(BundleError::AttestationDigestMismatch);
        }
        Ok(Self {
            id,
            changeset: draft.changeset,
            symbol_locks: collect_symbol_locks(&draft.claims),
            claims: draft.claims,
            manifest_digest: draft.manifest_digest,
            attestation: draft.attestation,
            provenance: draft.provenance,
            semantic_diff: draft.semantic_diff,
            evidence: draft.evidence,
            kg_edges: draft.kg_edges,
            promotion_evidence: Vec::new(),
            status: BundleStatus::Draft,
            schema_version: CHANGE_BUNDLE_SCHEMA_VERSION,
        })
    }

    pub fn validate_at(&mut self, now_epoch: u64) -> ValidationReport {
        let mut reasons = Vec::new();
        if self.attestation.signed_digest != self.manifest_digest {
            reasons.push(BundleError::AttestationDigestMismatch);
        }
        if self.provenance.source_digest != self.manifest_digest {
            reasons.push(BundleError::ProvenanceDigestMismatch);
        }
        if !self.provenance.claim_ids.iter().all(|claim_id| {
            self.claims
                .iter()
                .any(|claim| claim.id.as_str() == claim_id.as_str())
        }) {
            reasons.push(BundleError::InvalidProvenance);
        }
        if !self
            .semantic_diff
            .touched_symbols
            .iter()
            .all(|symbol| self.changeset.write_symbols.contains(symbol))
        {
            reasons.push(BundleError::UnclaimedDiff);
        }
        if self
            .claims
            .iter()
            .any(|claim| claim.state != ClaimState::Working)
        {
            reasons.push(BundleError::ClaimNotWorking);
        }
        if !claims_cover_changeset(&self.changeset, &self.claims) {
            reasons.push(BundleError::ClaimCoverage(
                VcsKernelError::UnclaimedTouchedArtifact,
            ));
        }
        if !self
            .evidence
            .iter()
            .all(|record| record.is_fresh_at(now_epoch))
        {
            reasons.push(BundleError::StaleEvidence);
        }
        if self
            .evidence
            .iter()
            .any(|record| record.artifact_digest != self.manifest_digest)
        {
            reasons.push(BundleError::ArtifactDigestMismatch);
        }
        if !has_required_lineage_evidence(&self.evidence) {
            reasons.push(BundleError::MissingBundleEvidence);
        }
        if self
            .promotion_evidence
            .iter()
            .any(|evidence| evidence.protected_ref_mutated)
        {
            reasons.push(BundleError::ProtectedRefMutation);
        }

        if reasons.iter().any(BundleError::quarantines_bundle) {
            self.status = BundleStatus::Quarantined;
            ValidationReport {
                decision: ValidationDecision::Quarantined,
                reasons,
            }
        } else {
            self.status = BundleStatus::Accepted;
            ValidationReport {
                decision: ValidationDecision::Accepted,
                reasons,
            }
        }
    }

    pub fn emit_done_bundle(mut self, now_epoch: u64) -> Result<DoneEmission, BundleError> {
        let report = self.validate_at(now_epoch);
        if report.decision != ValidationDecision::Accepted || !report.reasons.is_empty() {
            return Err(report
                .reasons
                .first()
                .cloned()
                .unwrap_or(BundleError::BundleQuarantined));
        }
        Ok(DoneEmission {
            bundle: self,
            protected_ref_mutated: false,
        })
    }

    pub fn publish_promotion_evidence(
        &mut self,
        promotion: PromotionEvidence,
        now_epoch: u64,
    ) -> Result<(), BundleError> {
        if promotion.protected_ref_mutated {
            self.status = BundleStatus::Quarantined;
            return Err(BundleError::ProtectedRefMutation);
        }
        if promotion.status != PromotionStatus::Published {
            self.promotion_evidence.push(promotion);
            return Err(BundleError::NonPublishedPromotion);
        }
        let report = self.validate_at(now_epoch);
        if report.decision != ValidationDecision::Accepted || !report.reasons.is_empty() {
            return Err(report
                .reasons
                .first()
                .cloned()
                .unwrap_or(BundleError::BundleQuarantined));
        }
        self.promotion_evidence.push(promotion);
        self.status = BundleStatus::Published;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeBundleDraft {
    pub id: String,                         // data_class: INTERNAL_ONLY
    pub changeset: ChangeSet,               // data_class: INTERNAL_ONLY
    pub claims: Vec<Claim>,                 // data_class: INTERNAL_ONLY
    pub manifest_digest: Digest,            // data_class: INTERNAL_ONLY
    pub attestation: BundleAttestation,     // data_class: INTERNAL_ONLY
    pub provenance: Provenance,             // data_class: INTERNAL_ONLY
    pub semantic_diff: SemanticDiffSummary, // data_class: INTERNAL_ONLY
    pub evidence: Vec<EvidenceRecord>,      // data_class: INTERNAL_ONLY
    pub kg_edges: Vec<KgEdgeRecord>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoneEmission {
    pub bundle: ChangeBundle,        // data_class: INTERNAL_ONLY
    pub protected_ref_mutated: bool, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BundleError {
    InvalidBundleId,
    UnsupportedDigest,
    InvalidDigest,
    UnsupportedSignature,
    MissingSignature,
    AttestationDigestMismatch,
    ProvenanceDigestMismatch,
    InvalidProvenance,
    InvalidSemanticDiff,
    InvalidEvidence,
    StaleEvidence,
    ArtifactDigestMismatch,
    MissingBundleEvidence,
    InvalidKgEdge,
    InvalidPromotion,
    NonPublishedPromotion,
    ProtectedRefMutation,
    ClaimNotWorking,
    UnclaimedDiff,
    ClaimCoverage(VcsKernelError),
    BundleQuarantined,
}

impl BundleError {
    fn quarantines_bundle(&self) -> bool {
        matches!(
            self,
            Self::ArtifactDigestMismatch
                | Self::AttestationDigestMismatch
                | Self::ProvenanceDigestMismatch
                | Self::ProtectedRefMutation
                | Self::UnclaimedDiff
                | Self::ClaimCoverage(_)
                | Self::ClaimNotWorking
                | Self::StaleEvidence
                | Self::MissingBundleEvidence
        )
    }
}

impl fmt::Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for BundleError {}

fn collect_symbol_locks(claims: &[Claim]) -> Vec<SymbolLock> {
    let mut locks = Vec::new();
    for claim in claims {
        locks.extend(claim.all_locks());
    }
    locks
}

fn claims_cover_changeset(changeset: &ChangeSet, claims: &[Claim]) -> bool {
    if claims
        .iter()
        .any(|claim| required_claim_coverage(changeset, claim).is_ok())
    {
        return true;
    }

    let write_values: BTreeSet<&str> = claims
        .iter()
        .flat_map(|claim| {
            claim
                .write_symbols
                .iter()
                .map(|symbol| symbol.value.as_str())
        })
        .collect();
    let changeset_values: BTreeSet<&str> = changeset
        .write_symbols
        .iter()
        .map(|symbol| symbol.value.as_str())
        .collect();
    changeset_values.is_subset(&write_values)
        && changeset.touched_files.iter().all(|touched| {
            claims.iter().any(|claim| {
                claim.write_symbols.iter().any(|symbol| {
                    SymbolLock::write(symbol.clone()).mode == ClaimMode::Write
                        && symbol.artifact.covers(touched)
                })
            })
        })
}

fn has_required_lineage_evidence(evidence: &[EvidenceRecord]) -> bool {
    let kinds: BTreeSet<EvidenceKind> = evidence.iter().map(|record| record.kind).collect();
    kinds.contains(&EvidenceKind::UnitTest) && kinds.contains(&EvidenceKind::Build)
}

fn normalize_non_empty(value: String, error: BundleError) -> Result<String, BundleError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}

fn validate_prefixed(
    value: String,
    prefix: &str,
    error: BundleError,
) -> Result<String, BundleError> {
    let value = normalize_non_empty(value, error.clone())?;
    if value.starts_with(prefix) && value.len() > prefix.len() {
        Ok(value)
    } else {
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_foundry_vcs_kernel::{
        ArtifactPointer, ChangeSetDraft, ChangeSetLineage, SymbolId, SymbolLanguage,
    };

    const BASE_SHA: &str = "0123456789012345678901234567890123456789";
    const NOW: u64 = 1_800_000_000;
    const SHA256_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SHA256_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn digest_a() -> Digest {
        Digest::new("sha256", SHA256_A).unwrap()
    }

    fn digest_b() -> Digest {
        Digest::new("sha256", SHA256_B).unwrap()
    }

    fn symbol(path: &str, name: &str) -> SymbolId {
        SymbolId::new(
            SymbolLanguage::Rust,
            ArtifactPointer::file(path).expect("valid path"),
            name,
        )
        .expect("valid symbol")
    }

    fn working_claim(write_symbols: Vec<SymbolId>) -> Claim {
        Claim::new(
            "claim_ip003",
            "agent-ip003",
            "IP-003 ChangeBundle fixture",
            write_symbols,
            vec![],
            900,
        )
        .unwrap()
        .grant()
        .start_work()
        .unwrap()
    }

    fn changeset(write_symbol: SymbolId, touched_file: ArtifactPointer) -> ChangeSet {
        ChangeSet::new(ChangeSetDraft {
            id: "cs_ip003".into(),
            agent_id: "agent-ip003".into(),
            target_branch: "main".into(),
            base_sha: BASE_SHA.into(),
            branch_or_workspace_ref: "workspace/agent-ip003".into(),
            patch_id: "patch_ip003".into(),
            write_symbols: vec![write_symbol],
            read_symbols: vec![],
            touched_files: vec![touched_file],
            dependencies: vec![],
            lineage: ChangeSetLineage::new("wi_p00", "ip_003", vec![]).unwrap(),
            evidence_refs: vec![".omc/evidence/gitops-vcs/ip-003-changebundle.json".into()],
        })
        .unwrap()
    }

    fn evidence(kind: EvidenceKind, digest: Digest) -> EvidenceRecord {
        EvidenceRecord::new(
            match kind {
                EvidenceKind::UnitTest => "ev_unit",
                EvidenceKind::IntegrationTest => "ev_integration",
                EvidenceKind::Build => "ev_build",
                EvidenceKind::Deploy => "ev_deploy",
                EvidenceKind::Review => "ev_review",
                EvidenceKind::Policy => "ev_policy",
            },
            kind,
            "rustc targeted fixture",
            EvidenceResult::Passed,
            digest,
            NOW - 60,
            NOW + 60,
            match kind {
                EvidenceKind::Build => "build-lineage",
                EvidenceKind::Deploy => "deploy-lineage",
                _ => "test-lineage",
            },
        )
        .unwrap()
    }

    fn bundle() -> ChangeBundle {
        let touched_file =
            ArtifactPointer::file("crates/oya-foundry-vcs-changebundle-kernel/src/lib.rs").unwrap();
        let touched_symbol = symbol(
            "crates/oya-foundry-vcs-changebundle-kernel/src/lib.rs",
            "ChangeBundle",
        );
        let claim = working_claim(vec![touched_symbol.clone()]);
        let changeset = changeset(touched_symbol.clone(), touched_file.clone());
        let digest = digest_a();
        ChangeBundle::new(ChangeBundleDraft {
            id: "cb_ip003".into(),
            changeset,
            claims: vec![claim],
            manifest_digest: digest.clone(),
            attestation: BundleAttestation::new(
                "ed25519",
                "key-ip003",
                "signature-ip003",
                digest.clone(),
            )
            .unwrap(),
            provenance: Provenance::new(
                "agent-ip003",
                vec!["claim_ip003".into()],
                digest.clone(),
                "EVT-IP003",
                NOW - 120,
                "main@base",
                "workspace/agent-ip003",
                vec!["rustc-lib".into(), "rustc-test".into()],
                vec![],
            )
            .unwrap(),
            semantic_diff: SemanticDiffSummary::new(
                vec![touched_symbol],
                vec![touched_file],
                vec!["unit".into(), "build".into()],
                digest.clone(),
                "ChangeBundle attestation kernel fixture",
            )
            .unwrap(),
            evidence: vec![
                evidence(EvidenceKind::UnitTest, digest.clone()),
                evidence(EvidenceKind::Build, digest),
            ],
            kg_edges: vec![KgEdgeRecord::new("cs_ip003", "cb_ip003", "PACKAGED_AS").unwrap()],
        })
        .unwrap()
    }

    #[test]
    fn schema_provenance_and_coverage_validation_accepts_signed_fixture() {
        let mut bundle = bundle();

        let report = bundle.validate_at(NOW);

        assert_eq!(bundle.schema_version, CHANGE_BUNDLE_SCHEMA_VERSION);
        assert_eq!(report.decision, ValidationDecision::Accepted);
        assert!(report.reasons.is_empty());
        assert_eq!(bundle.status, BundleStatus::Accepted);
        assert_eq!(bundle.symbol_locks.len(), 1);
    }

    #[test]
    fn digest_and_evidence_freshness_are_enforced() {
        let mut stale = bundle();
        stale.evidence[0].observed_at_epoch = NOW - MAX_EVIDENCE_AGE_SECONDS - 1;
        stale.evidence[0].expires_at_epoch = NOW + 60;

        let report = stale.validate_at(NOW);

        assert_eq!(report.decision, ValidationDecision::Quarantined);
        assert!(report.reasons.contains(&BundleError::StaleEvidence));
        assert_eq!(stale.status, BundleStatus::Quarantined);
    }

    #[test]
    fn unsigned_bundle_is_rejected() {
        assert_eq!(
            BundleAttestation::new("ed25519", "key-ip003", "", digest_a()),
            Err(BundleError::MissingSignature)
        );
        assert_eq!(
            BundleAttestation::new("rsa-pkcs1", "key-ip003", "sig", digest_a()),
            Err(BundleError::UnsupportedSignature)
        );
    }

    #[test]
    fn unclaimed_diff_is_rejected() {
        let mut bundle = bundle();
        bundle.semantic_diff.touched_symbols = vec![symbol(
            "crates/oya-foundry-vcs-changebundle-kernel/src/lib.rs",
            "UnclaimedSymbol",
        )];

        let report = bundle.validate_at(NOW);

        assert_eq!(report.decision, ValidationDecision::Quarantined);
        assert!(report.reasons.contains(&BundleError::UnclaimedDiff));
    }

    #[test]
    fn unclaimed_touched_artifact_is_rejected() {
        let claimed = symbol("crates/owned/src/lib.rs", "Owned");
        let claim = working_claim(vec![claimed.clone()]);
        let mut bundle = bundle();
        bundle.claims = vec![claim];
        bundle.changeset = changeset(
            claimed,
            ArtifactPointer::file("crates/unowned/src/lib.rs").unwrap(),
        );

        let report = bundle.validate_at(NOW);

        assert_eq!(report.decision, ValidationDecision::Quarantined);
        assert!(
            report
                .reasons
                .iter()
                .any(|reason| matches!(reason, BundleError::ClaimCoverage(_)))
        );
    }

    #[test]
    fn artifact_mismatch_quarantines_bundle() {
        let mut bundle = bundle();
        bundle.evidence[0].artifact_digest = digest_b();

        let report = bundle.validate_at(NOW);

        assert_eq!(report.decision, ValidationDecision::Quarantined);
        assert!(
            report
                .reasons
                .contains(&BundleError::ArtifactDigestMismatch)
        );
        assert_eq!(bundle.status, BundleStatus::Quarantined);
    }

    #[test]
    fn done_emits_bundle_without_protected_ref_mutation() {
        let bundle = bundle();

        let emission = bundle.emit_done_bundle(NOW).unwrap();

        assert!(!emission.protected_ref_mutated);
        assert!(
            emission
                .bundle
                .promotion_evidence
                .iter()
                .all(|evidence| !evidence.protected_ref_mutated)
        );
        assert_eq!(emission.bundle.status, BundleStatus::Accepted);
    }

    #[test]
    fn bundle_publishes_promotion_evidence() {
        let mut bundle = bundle();
        let promotion = PromotionEvidence::new(
            "promo_ip003_dev",
            "dev",
            PromotionStatus::Published,
            ".omc/evidence/gitops-vcs/ip-003-changebundle.json#promotion",
            false,
        )
        .unwrap();

        bundle.publish_promotion_evidence(promotion, NOW).unwrap();

        assert_eq!(bundle.status, BundleStatus::Published);
        assert_eq!(bundle.promotion_evidence.len(), 1);
        assert_eq!(
            bundle.promotion_evidence[0].status,
            PromotionStatus::Published
        );
    }

    #[test]
    fn requested_or_rejected_promotion_does_not_publish_bundle() {
        for status in [PromotionStatus::Requested, PromotionStatus::Rejected] {
            let mut bundle = bundle();
            let promotion = PromotionEvidence::new(
                format!("promo_ip003_{status:?}"),
                "staging",
                status,
                ".omc/evidence/gitops-vcs/ip-003-changebundle.json#promotion-pending",
                false,
            )
            .unwrap();

            assert_eq!(
                bundle.publish_promotion_evidence(promotion, NOW),
                Err(BundleError::NonPublishedPromotion)
            );
            assert_eq!(bundle.status, BundleStatus::Draft);
            assert_eq!(bundle.promotion_evidence.len(), 1);
            assert_eq!(bundle.promotion_evidence[0].status, status);
        }
    }

    #[test]
    fn protected_ref_mutation_quarantines_promotion() {
        let mut bundle = bundle();
        let promotion = PromotionEvidence::new(
            "promo_ip003_bad",
            "production",
            PromotionStatus::Published,
            "mutated protected ref",
            true,
        )
        .unwrap();

        assert_eq!(
            bundle.publish_promotion_evidence(promotion, NOW),
            Err(BundleError::ProtectedRefMutation)
        );
        assert_eq!(bundle.status, BundleStatus::Quarantined);
    }
}
