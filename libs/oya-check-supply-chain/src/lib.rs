//! Foundry supply-chain fitness kernel.
//!
//! ADR-0039's final posture is Trivy + Cosign + dual-format SBOM + signed
//! provenance. Bootstrap is not allowed to claim that posture early. This pure
//! kernel enforces the source-only phase: Rust dependency scanning must be wired,
//! higher catalog attestations require corresponding evidence, and release
//! manifests cannot appear before release-artifact scanning/signing evidence or
//! an explicit pre-release empty-scope declaration.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainRecord {
    pub subject: String,     // data_class: INTERNAL_ONLY
    pub attestation: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupplyChainEvidence {
    pub deny_config_present: bool,             // data_class: INTERNAL_ONLY
    pub cargo_deny_check_wired: bool,          // data_class: INTERNAL_ONLY
    pub cargo_audit_check_wired: bool,         // data_class: INTERNAL_ONLY
    pub third_party_actions_pinned: bool,      // data_class: INTERNAL_ONLY
    pub require_adr0039_evidence: bool,        // data_class: INTERNAL_ONLY
    pub release_manifest_present: bool,        // data_class: INTERNAL_ONLY
    pub release_images_declared: bool,         // data_class: INTERNAL_ONLY
    pub release_empty_scope_declared: bool,    // data_class: INTERNAL_ONLY
    pub trivy_release_scan_wired: bool,        // data_class: INTERNAL_ONLY
    pub trivy_filesystem_scan_wired: bool,     // data_class: INTERNAL_ONLY
    pub trivy_container_scan_wired: bool,      // data_class: INTERNAL_ONLY
    pub trivy_iac_scan_wired: bool,            // data_class: INTERNAL_ONLY
    pub trivy_dependency_scan_wired: bool,     // data_class: INTERNAL_ONLY
    pub cosign_release_signing_wired: bool,    // data_class: INTERNAL_ONLY
    pub cosign_rekor_verification_wired: bool, // data_class: INTERNAL_ONLY
    pub sbom_dual_format_wired: bool,          // data_class: INTERNAL_ONLY
    pub sbom_spdx_wired: bool,                 // data_class: INTERNAL_ONLY
    pub sbom_cyclonedx_wired: bool,            // data_class: INTERNAL_ONLY
    pub provenance_attestation_wired: bool,    // data_class: INTERNAL_ONLY
    pub signed_commit_policy_wired: bool,      // data_class: INTERNAL_ONLY
    pub admission_policy_wired: bool,          // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupplyChainReport {
    pub records_checked: usize,     // data_class: INTERNAL_ONLY
    pub source_only_records: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseArtifact {
    pub artifact_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseSupplyChainEvidence {
    pub artifact_ref: String,               // data_class: INTERNAL_ONLY
    pub artifact_digest: String,            // data_class: INTERNAL_ONLY
    pub release_version: String,            // data_class: INTERNAL_ONLY
    pub source_revision: String,            // data_class: INTERNAL_ONLY
    pub sbom_spdx_ref: String,              // data_class: INTERNAL_ONLY
    pub sbom_cyclonedx_ref: String,         // data_class: INTERNAL_ONLY
    pub cosign_signature_ref: String,       // data_class: INTERNAL_ONLY
    pub cosign_certificate_ref: String,     // data_class: INTERNAL_ONLY
    pub rekor_log_index: u64,               // data_class: INTERNAL_ONLY
    pub trivy_filesystem_scan_ref: String,  // data_class: INTERNAL_ONLY
    pub trivy_container_scan_ref: String,   // data_class: INTERNAL_ONLY
    pub trivy_iac_scan_ref: String,         // data_class: INTERNAL_ONLY
    pub trivy_dependency_scan_ref: String,  // data_class: INTERNAL_ONLY
    pub provenance_attestation_ref: String, // data_class: INTERNAL_ONLY
    pub audit_event_type: String,           // data_class: INTERNAL_ONLY
    pub attestor: String,                   // data_class: INTERNAL_ONLY
    pub high_critical_findings_open: u64,   // data_class: INTERNAL_ONLY
    pub signed: bool,                       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseSupplyChainReport {
    pub artifacts_checked: usize,        // data_class: INTERNAL_ONLY
    pub evidence_records_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ImagePromotionTier {
    Dev,
    Staging,
    Prod,
}

impl ImagePromotionTier {
    pub fn name(self) -> &'static str {
        match self {
            Self::Dev => "dev",
            Self::Staging => "staging",
            Self::Prod => "prod",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImagePromotionVerifier {
    Kubewarden,
    Kyverno,
}

impl ImagePromotionVerifier {
    pub fn name(self) -> &'static str {
        match self {
            Self::Kubewarden => "kubewarden",
            Self::Kyverno => "kyverno",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImagePromotionRecord {
    pub artifact_ref: String,               // data_class: INTERNAL_ONLY
    pub artifact_digest: String,            // data_class: INTERNAL_ONLY
    pub tier: ImagePromotionTier,           // data_class: INTERNAL_ONLY
    pub cosign_identity: String,            // data_class: INTERNAL_ONLY
    pub verifier: ImagePromotionVerifier,   // data_class: INTERNAL_ONLY
    pub verifier_ref: String,               // data_class: INTERNAL_ONLY
    pub provenance_attestation_ref: String, // data_class: INTERNAL_ONLY
    pub runner_kill_switch_ref: String,     // data_class: INTERNAL_ONLY
    pub audit_event_type: String,           // data_class: INTERNAL_ONLY
    pub signed: bool,                       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImagePromotionReport {
    pub artifacts_checked: usize,           // data_class: INTERNAL_ONLY
    pub promotion_records_checked: usize,   // data_class: INTERNAL_ONLY
    pub kubewarden_verifier_records: usize, // data_class: INTERNAL_ONLY
    pub kyverno_verifier_records: usize,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VulnerabilityProductSurface {
    CloudNativeApi,
    ScannerCli,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VulnerabilityAdvisoryFeed {
    CveNvd,
    Osv,
    RustSec,
    GitHubAdvisories,
    VendorAdvisories,
}

impl VulnerabilityAdvisoryFeed {
    pub fn name(self) -> &'static str {
        match self {
            Self::CveNvd => "cve-nvd",
            Self::Osv => "osv",
            Self::RustSec => "rustsec",
            Self::GitHubAdvisories => "github-advisories",
            Self::VendorAdvisories => "vendor-advisories",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SbomFormat {
    Spdx,
    CycloneDx,
}

impl SbomFormat {
    pub fn name(self) -> &'static str {
        match self {
            Self::Spdx => "SPDX",
            Self::CycloneDx => "CycloneDX",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VexStatus {
    Affected,
    NotAffected,
    Fixed,
    UnderInvestigation,
}

impl VexStatus {
    pub fn name(self) -> &'static str {
        match self {
            Self::Affected => "affected",
            Self::NotAffected => "not_affected",
            Self::Fixed => "fixed",
            Self::UnderInvestigation => "under_investigation",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VulnerabilityPrioritySignal {
    CisaKev,
    Epss,
    Cvss,
    Ssvc,
}

impl VulnerabilityPrioritySignal {
    pub fn name(self) -> &'static str {
        match self {
            Self::CisaKev => "CISA_KEV",
            Self::Epss => "EPSS",
            Self::Cvss => "CVSS",
            Self::Ssvc => "SSVC",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RemediationSlaClass {
    KevOrActivelyExploited,
    Critical,
    High,
    Medium,
}

impl RemediationSlaClass {
    pub fn name(self) -> &'static str {
        match self {
            Self::KevOrActivelyExploited => "kev_or_actively_exploited",
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemediationSla {
    pub class: RemediationSlaClass, // data_class: INTERNAL_ONLY
    pub max_days: u32,              // data_class: INTERNAL_ONLY
    pub deployment_blocking: bool,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VulnerabilityExceptionPolicy {
    pub max_ttl_days: u32,                // data_class: INTERNAL_ONLY
    pub requires_owner: bool,             // data_class: INTERNAL_ONLY
    pub requires_expiry: bool,            // data_class: INTERNAL_ONLY
    pub requires_vex_justification: bool, // data_class: INTERNAL_ONLY
    pub requires_audit_event: bool,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VulnerabilityAuditEvidencePolicy {
    pub advisory_snapshot_signed: bool,  // data_class: INTERNAL_ONLY
    pub sbom_artifacts_signed: bool,     // data_class: INTERNAL_ONLY
    pub vex_artifacts_signed: bool,      // data_class: INTERNAL_ONLY
    pub priority_decision_signed: bool,  // data_class: INTERNAL_ONLY
    pub exception_decision_signed: bool, // data_class: INTERNAL_ONLY
    pub admission_verdict_signed: bool,  // data_class: INTERNAL_ONLY
    pub audit_event_type: String,        // data_class: INTERNAL_ONLY
    pub retention_days: u32,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VulnerabilityAdmissionPolicy {
    pub block_missing_or_unsigned_sbom: bool, // data_class: INTERNAL_ONLY
    pub block_missing_vex: bool,              // data_class: INTERNAL_ONLY
    pub block_expired_exception: bool,        // data_class: INTERNAL_ONLY
    pub block_kev_or_exploited: bool,         // data_class: INTERNAL_ONLY
    pub block_fix_available_past_sla: bool,   // data_class: INTERNAL_ONLY
    pub block_unknown_component_match: bool,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VulnerabilityIntelligenceContract {
    pub lane_id: String,                                // data_class: INTERNAL_ONLY
    pub canonical_surface: VulnerabilityProductSurface, // data_class: INTERNAL_ONLY
    pub advisory_feeds: Vec<VulnerabilityAdvisoryFeed>, // data_class: INTERNAL_ONLY
    pub sbom_formats: Vec<SbomFormat>,                  // data_class: INTERNAL_ONLY
    pub vex_statuses: Vec<VexStatus>,                   // data_class: INTERNAL_ONLY
    pub priority_signals: Vec<VulnerabilityPrioritySignal>, // data_class: INTERNAL_ONLY
    pub remediation_slas: Vec<RemediationSla>,          // data_class: INTERNAL_ONLY
    pub exception_policy: VulnerabilityExceptionPolicy, // data_class: INTERNAL_ONLY
    pub audit_evidence: VulnerabilityAuditEvidencePolicy, // data_class: INTERNAL_ONLY
    pub admission_policy: VulnerabilityAdmissionPolicy, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VulnerabilityIntelligenceReport {
    pub feeds_checked: usize,            // data_class: INTERNAL_ONLY
    pub sbom_formats_checked: usize,     // data_class: INTERNAL_ONLY
    pub vex_statuses_checked: usize,     // data_class: INTERNAL_ONLY
    pub priority_signals_checked: usize, // data_class: INTERNAL_ONLY
    pub remediation_slas_checked: usize, // data_class: INTERNAL_ONLY
    pub admission_blocks_checked: usize, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupplyChainError {
    NoCatalogRecords,
    MissingDenyConfig,
    MissingCargoDenyCheck,
    MissingCargoAuditCheck,
    UnpinnedThirdPartyAction,
    UnknownAttestation {
        subject: String,
        attestation: String,
    },
    UnsupportedAttestationClaim {
        subject: String,
        attestation: String,
        missing_evidence: &'static str,
    },
    MissingAdr0039Evidence {
        missing_evidence: &'static str,
    },
    ReleaseManifestWithoutAdr0039Evidence {
        missing_evidence: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReleaseSupplyChainError {
    NoReleaseArtifacts,
    NoEvidenceRecords,
    MissingPreReleaseEmptyScopeRationale,
    InvalidArtifactRef {
        artifact_ref: String,
    },
    DuplicateArtifact {
        artifact_ref: String,
    },
    DuplicateEvidence {
        artifact_ref: String,
    },
    MissingEvidenceForArtifact {
        artifact_ref: String,
    },
    EvidenceForUnknownArtifact {
        artifact_ref: String,
    },
    MissingField {
        artifact_ref: String,
        field: &'static str,
    },
    InvalidDigest {
        artifact_ref: String,
    },
    DigestNotPinnedInArtifactRef {
        artifact_ref: String,
        artifact_digest: String,
    },
    InvalidReleaseVersion {
        artifact_ref: String,
        release_version: String,
    },
    InvalidSourceRevision {
        artifact_ref: String,
        source_revision: String,
    },
    MissingRekorInclusion {
        artifact_ref: String,
    },
    OpenHighCriticalFindings {
        artifact_ref: String,
        count: u64,
    },
    UnsignedEvidence {
        artifact_ref: String,
    },
    InvalidAuditEventType {
        artifact_ref: String,
        audit_event_type: String,
    },
    InvalidSbomRef {
        artifact_ref: String,
        field: &'static str,
    },
    InvalidTrivyRef {
        artifact_ref: String,
        field: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImagePromotionError {
    NoPromotionRecords,
    MissingField {
        artifact_ref: String,
        field: &'static str,
    },
    InvalidArtifactRef {
        artifact_ref: String,
    },
    InvalidDigest {
        artifact_ref: String,
    },
    DigestNotPinnedInArtifactRef {
        artifact_ref: String,
        artifact_digest: String,
    },
    TierTagMismatch {
        artifact_ref: String,
        tier: ImagePromotionTier,
    },
    DuplicateTierPromotion {
        artifact_digest: String,
        tier: ImagePromotionTier,
    },
    MissingTierPromotion {
        artifact_digest: String,
        tier: ImagePromotionTier,
    },
    MissingDefaultVerifier {
        artifact_digest: String,
    },
    InvalidCosignIdentity {
        artifact_ref: String,
        tier: ImagePromotionTier,
        cosign_identity: String,
    },
    InvalidVerifierRef {
        artifact_ref: String,
        verifier: ImagePromotionVerifier,
        verifier_ref: String,
    },
    InvalidProvenanceRef {
        artifact_ref: String,
    },
    InvalidRunnerKillSwitchRef {
        artifact_ref: String,
    },
    UnsignedPromotion {
        artifact_ref: String,
    },
    InvalidAuditEventType {
        artifact_ref: String,
        audit_event_type: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VulnerabilityIntelligenceError {
    InvalidLaneId {
        lane_id: String,
    },
    ScannerCliDeclaredAsCanonicalSurface,
    MissingAdvisoryFeed {
        feed: &'static str,
    },
    MissingSbomFormat {
        format: &'static str,
    },
    MissingVexStatus {
        status: &'static str,
    },
    MissingPrioritySignal {
        signal: &'static str,
    },
    MissingRemediationSla {
        class: &'static str,
    },
    RemediationSlaTooLoose {
        class: &'static str,
        max_days: u32,
        allowed_days: u32,
    },
    SlaMustBlockDeployment {
        class: &'static str,
    },
    ExceptionPolicyMissingOwner,
    ExceptionPolicyMissingExpiry,
    ExceptionPolicyMissingVexJustification,
    ExceptionPolicyMissingAuditEvent,
    ExceptionTtlTooLong {
        max_ttl_days: u32,
        allowed_days: u32,
    },
    MissingSignedAuditEvidence {
        field: &'static str,
    },
    InvalidVulnerabilityAuditEvent {
        audit_event_type: String,
    },
    AuditRetentionTooShort {
        retention_days: u32,
        required_days: u32,
    },
    MissingAdmissionBlock {
        block: &'static str,
    },
}

pub const VULNERABILITY_INTELLIGENCE_LANE_ID: &str = "security-pipeline/vulnerability-intelligence";
pub const VULNERABILITY_DECISION_AUDIT_EVENT: &str =
    "oya.audit.vulnerability_intelligence_decision";
const VULNERABILITY_MIN_AUDIT_RETENTION_DAYS: u32 = 2_555;
const VULNERABILITY_EXCEPTION_MAX_TTL_DAYS: u32 = 30;

pub fn validate_vulnerability_intelligence_contract(
    contract: &VulnerabilityIntelligenceContract,
) -> Result<VulnerabilityIntelligenceReport, VulnerabilityIntelligenceError> {
    if contract.lane_id.trim() != VULNERABILITY_INTELLIGENCE_LANE_ID {
        return Err(VulnerabilityIntelligenceError::InvalidLaneId {
            lane_id: contract.lane_id.clone(),
        });
    }
    if contract.canonical_surface == VulnerabilityProductSurface::ScannerCli {
        return Err(VulnerabilityIntelligenceError::ScannerCliDeclaredAsCanonicalSurface);
    }

    for feed in [
        VulnerabilityAdvisoryFeed::CveNvd,
        VulnerabilityAdvisoryFeed::Osv,
        VulnerabilityAdvisoryFeed::RustSec,
        VulnerabilityAdvisoryFeed::GitHubAdvisories,
        VulnerabilityAdvisoryFeed::VendorAdvisories,
    ] {
        if !contract.advisory_feeds.contains(&feed) {
            return Err(VulnerabilityIntelligenceError::MissingAdvisoryFeed { feed: feed.name() });
        }
    }

    for format in [SbomFormat::Spdx, SbomFormat::CycloneDx] {
        if !contract.sbom_formats.contains(&format) {
            return Err(VulnerabilityIntelligenceError::MissingSbomFormat {
                format: format.name(),
            });
        }
    }

    for status in [
        VexStatus::Affected,
        VexStatus::NotAffected,
        VexStatus::Fixed,
        VexStatus::UnderInvestigation,
    ] {
        if !contract.vex_statuses.contains(&status) {
            return Err(VulnerabilityIntelligenceError::MissingVexStatus {
                status: status.name(),
            });
        }
    }

    for signal in [
        VulnerabilityPrioritySignal::CisaKev,
        VulnerabilityPrioritySignal::Epss,
        VulnerabilityPrioritySignal::Cvss,
        VulnerabilityPrioritySignal::Ssvc,
    ] {
        if !contract.priority_signals.contains(&signal) {
            return Err(VulnerabilityIntelligenceError::MissingPrioritySignal {
                signal: signal.name(),
            });
        }
    }

    for (class, allowed_days, must_block) in [
        (RemediationSlaClass::KevOrActivelyExploited, 7, true),
        (RemediationSlaClass::Critical, 7, true),
        (RemediationSlaClass::High, 14, true),
        (RemediationSlaClass::Medium, 30, false),
    ] {
        validate_remediation_sla(&contract.remediation_slas, class, allowed_days, must_block)?;
    }

    validate_vulnerability_exception_policy(&contract.exception_policy)?;
    validate_vulnerability_audit_evidence(&contract.audit_evidence)?;
    let admission_blocks_checked =
        validate_vulnerability_admission_policy(&contract.admission_policy)?;

    Ok(VulnerabilityIntelligenceReport {
        feeds_checked: contract.advisory_feeds.len(),
        sbom_formats_checked: contract.sbom_formats.len(),
        vex_statuses_checked: contract.vex_statuses.len(),
        priority_signals_checked: contract.priority_signals.len(),
        remediation_slas_checked: contract.remediation_slas.len(),
        admission_blocks_checked,
    })
}

fn validate_remediation_sla(
    slas: &[RemediationSla],
    class: RemediationSlaClass,
    allowed_days: u32,
    must_block: bool,
) -> Result<(), VulnerabilityIntelligenceError> {
    let Some(sla) = slas.iter().find(|sla| sla.class == class) else {
        return Err(VulnerabilityIntelligenceError::MissingRemediationSla {
            class: class.name(),
        });
    };
    if sla.max_days == 0 || sla.max_days > allowed_days {
        return Err(VulnerabilityIntelligenceError::RemediationSlaTooLoose {
            class: class.name(),
            max_days: sla.max_days,
            allowed_days,
        });
    }
    if must_block && !sla.deployment_blocking {
        return Err(VulnerabilityIntelligenceError::SlaMustBlockDeployment {
            class: class.name(),
        });
    }
    Ok(())
}

fn validate_vulnerability_exception_policy(
    policy: &VulnerabilityExceptionPolicy,
) -> Result<(), VulnerabilityIntelligenceError> {
    if !policy.requires_owner {
        return Err(VulnerabilityIntelligenceError::ExceptionPolicyMissingOwner);
    }
    if !policy.requires_expiry {
        return Err(VulnerabilityIntelligenceError::ExceptionPolicyMissingExpiry);
    }
    if !policy.requires_vex_justification {
        return Err(VulnerabilityIntelligenceError::ExceptionPolicyMissingVexJustification);
    }
    if !policy.requires_audit_event {
        return Err(VulnerabilityIntelligenceError::ExceptionPolicyMissingAuditEvent);
    }
    if policy.max_ttl_days == 0 || policy.max_ttl_days > VULNERABILITY_EXCEPTION_MAX_TTL_DAYS {
        return Err(VulnerabilityIntelligenceError::ExceptionTtlTooLong {
            max_ttl_days: policy.max_ttl_days,
            allowed_days: VULNERABILITY_EXCEPTION_MAX_TTL_DAYS,
        });
    }
    Ok(())
}

fn validate_vulnerability_audit_evidence(
    evidence: &VulnerabilityAuditEvidencePolicy,
) -> Result<(), VulnerabilityIntelligenceError> {
    for (present, field) in [
        (
            evidence.advisory_snapshot_signed,
            "advisory_snapshot_signed",
        ),
        (evidence.sbom_artifacts_signed, "sbom_artifacts_signed"),
        (evidence.vex_artifacts_signed, "vex_artifacts_signed"),
        (
            evidence.priority_decision_signed,
            "priority_decision_signed",
        ),
        (
            evidence.exception_decision_signed,
            "exception_decision_signed",
        ),
        (
            evidence.admission_verdict_signed,
            "admission_verdict_signed",
        ),
    ] {
        if !present {
            return Err(VulnerabilityIntelligenceError::MissingSignedAuditEvidence { field });
        }
    }
    if evidence.audit_event_type != VULNERABILITY_DECISION_AUDIT_EVENT {
        return Err(
            VulnerabilityIntelligenceError::InvalidVulnerabilityAuditEvent {
                audit_event_type: evidence.audit_event_type.clone(),
            },
        );
    }
    if evidence.retention_days < VULNERABILITY_MIN_AUDIT_RETENTION_DAYS {
        return Err(VulnerabilityIntelligenceError::AuditRetentionTooShort {
            retention_days: evidence.retention_days,
            required_days: VULNERABILITY_MIN_AUDIT_RETENTION_DAYS,
        });
    }
    Ok(())
}

fn validate_vulnerability_admission_policy(
    policy: &VulnerabilityAdmissionPolicy,
) -> Result<usize, VulnerabilityIntelligenceError> {
    let blocks = [
        (
            policy.block_missing_or_unsigned_sbom,
            "missing_or_unsigned_sbom",
        ),
        (policy.block_missing_vex, "missing_vex"),
        (policy.block_expired_exception, "expired_exception"),
        (policy.block_kev_or_exploited, "kev_or_exploited"),
        (
            policy.block_fix_available_past_sla,
            "fix_available_past_sla",
        ),
        (
            policy.block_unknown_component_match,
            "unknown_component_match",
        ),
    ];
    for (present, block) in blocks {
        if !present {
            return Err(VulnerabilityIntelligenceError::MissingAdmissionBlock { block });
        }
    }
    Ok(blocks.len())
}

pub fn validate_supply_chain<R>(
    records: R,
    evidence: SupplyChainEvidence,
) -> Result<SupplyChainReport, SupplyChainError>
where
    R: IntoIterator<Item = SupplyChainRecord>,
{
    if !evidence.deny_config_present {
        return Err(SupplyChainError::MissingDenyConfig);
    }
    if !evidence.cargo_deny_check_wired {
        return Err(SupplyChainError::MissingCargoDenyCheck);
    }
    if !evidence.cargo_audit_check_wired {
        return Err(SupplyChainError::MissingCargoAuditCheck);
    }
    if !evidence.third_party_actions_pinned {
        return Err(SupplyChainError::UnpinnedThirdPartyAction);
    }
    if evidence.require_adr0039_evidence {
        require_adr0039_evidence(evidence)?;
    }
    if evidence.release_manifest_present {
        if !evidence.trivy_release_scan_wired {
            return Err(SupplyChainError::ReleaseManifestWithoutAdr0039Evidence {
                missing_evidence: "trivy_release_scan_wired",
            });
        }
        if !evidence.cosign_release_signing_wired {
            return Err(SupplyChainError::ReleaseManifestWithoutAdr0039Evidence {
                missing_evidence: "cosign_release_signing_wired",
            });
        }
        if !evidence.sbom_dual_format_wired {
            return Err(SupplyChainError::ReleaseManifestWithoutAdr0039Evidence {
                missing_evidence: "sbom_dual_format_wired",
            });
        }
    }

    let mut records_checked = 0usize;
    let mut source_only_records = 0usize;
    for record in records {
        records_checked += 1;
        match record.attestation.as_str() {
            "source-only" => source_only_records += 1,
            "license-checked" => {
                if !evidence.cargo_deny_check_wired || !evidence.cargo_audit_check_wired {
                    return Err(SupplyChainError::UnsupportedAttestationClaim {
                        subject: record.subject,
                        attestation: record.attestation,
                        missing_evidence: "rust_dependency_scan_wiring",
                    });
                }
            }
            "sbom" => {
                if !evidence.sbom_dual_format_wired {
                    return Err(SupplyChainError::UnsupportedAttestationClaim {
                        subject: record.subject,
                        attestation: record.attestation,
                        missing_evidence: "sbom_dual_format_wired",
                    });
                }
            }
            "signed-provenance" => {
                if !evidence.sbom_dual_format_wired {
                    return Err(SupplyChainError::UnsupportedAttestationClaim {
                        subject: record.subject,
                        attestation: record.attestation,
                        missing_evidence: "sbom_dual_format_wired",
                    });
                }
                if !evidence.cosign_release_signing_wired {
                    return Err(SupplyChainError::UnsupportedAttestationClaim {
                        subject: record.subject,
                        attestation: record.attestation,
                        missing_evidence: "cosign_release_signing_wired",
                    });
                }
            }
            _ => {
                return Err(SupplyChainError::UnknownAttestation {
                    subject: record.subject,
                    attestation: record.attestation,
                });
            }
        }
    }

    if records_checked == 0 {
        Err(SupplyChainError::NoCatalogRecords)
    } else {
        Ok(SupplyChainReport {
            records_checked,
            source_only_records,
        })
    }
}

pub fn validate_release_supply_chain<A, E>(
    artifacts: A,
    evidence_records: E,
) -> Result<ReleaseSupplyChainReport, ReleaseSupplyChainError>
where
    A: IntoIterator<Item = ReleaseArtifact>,
    E: IntoIterator<Item = ReleaseSupplyChainEvidence>,
{
    let artifacts = release_artifact_map(artifacts)?;
    if artifacts.is_empty() {
        return Err(ReleaseSupplyChainError::NoReleaseArtifacts);
    }
    let evidence = release_evidence_map(evidence_records)?;
    if evidence.is_empty() {
        return Err(ReleaseSupplyChainError::NoEvidenceRecords);
    }

    validate_release_supply_chain_maps(artifacts, evidence)
}

fn validate_release_supply_chain_maps(
    artifacts: BTreeMap<String, ReleaseArtifact>,
    evidence: BTreeMap<String, ReleaseSupplyChainEvidence>,
) -> Result<ReleaseSupplyChainReport, ReleaseSupplyChainError> {
    for artifact_ref in artifacts.keys() {
        if !evidence.contains_key(artifact_ref) {
            return Err(ReleaseSupplyChainError::MissingEvidenceForArtifact {
                artifact_ref: artifact_ref.clone(),
            });
        }
    }
    for record in evidence.values() {
        if !artifacts.contains_key(&record.artifact_ref) {
            return Err(ReleaseSupplyChainError::EvidenceForUnknownArtifact {
                artifact_ref: record.artifact_ref.clone(),
            });
        }
        validate_release_evidence_record(record)?;
    }

    Ok(ReleaseSupplyChainReport {
        artifacts_checked: artifacts.len(),
        evidence_records_checked: evidence.len(),
    })
}

pub fn validate_pre_release_supply_chain<A, E>(
    artifacts: A,
    evidence_records: E,
    empty_scope_declared: bool,
) -> Result<ReleaseSupplyChainReport, ReleaseSupplyChainError>
where
    A: IntoIterator<Item = ReleaseArtifact>,
    E: IntoIterator<Item = ReleaseSupplyChainEvidence>,
{
    let artifacts = release_artifact_map(artifacts)?;
    let evidence = release_evidence_map(evidence_records)?;

    if artifacts.is_empty() {
        if evidence.is_empty() && empty_scope_declared {
            return Ok(ReleaseSupplyChainReport {
                artifacts_checked: 0,
                evidence_records_checked: 0,
            });
        }
        return Err(ReleaseSupplyChainError::MissingPreReleaseEmptyScopeRationale);
    }

    if evidence.is_empty() {
        return Ok(ReleaseSupplyChainReport {
            artifacts_checked: artifacts.len(),
            evidence_records_checked: 0,
        });
    }

    validate_release_supply_chain_maps(artifacts, evidence)
}

pub fn validate_image_promotion_pipeline<P>(
    promotion_records: P,
) -> Result<ImagePromotionReport, ImagePromotionError>
where
    P: IntoIterator<Item = ImagePromotionRecord>,
{
    let mut records_by_digest =
        BTreeMap::<String, BTreeMap<ImagePromotionTier, ImagePromotionRecord>>::new();
    let mut promotion_records_checked = 0usize;
    let mut kubewarden_verifier_records = 0usize;
    let mut kyverno_verifier_records = 0usize;

    for record in promotion_records {
        validate_image_promotion_record(&record)?;
        promotion_records_checked += 1;
        match record.verifier {
            ImagePromotionVerifier::Kubewarden => kubewarden_verifier_records += 1,
            ImagePromotionVerifier::Kyverno => kyverno_verifier_records += 1,
        }

        let tier_records = records_by_digest
            .entry(record.artifact_digest.clone())
            .or_default();
        if tier_records.contains_key(&record.tier) {
            return Err(ImagePromotionError::DuplicateTierPromotion {
                artifact_digest: record.artifact_digest,
                tier: record.tier,
            });
        }
        tier_records.insert(record.tier, record);
    }

    if promotion_records_checked == 0 {
        return Err(ImagePromotionError::NoPromotionRecords);
    }

    for (artifact_digest, tier_records) in &records_by_digest {
        for tier in [
            ImagePromotionTier::Dev,
            ImagePromotionTier::Staging,
            ImagePromotionTier::Prod,
        ] {
            if !tier_records.contains_key(&tier) {
                return Err(ImagePromotionError::MissingTierPromotion {
                    artifact_digest: artifact_digest.clone(),
                    tier,
                });
            }
        }
        if !tier_records
            .values()
            .any(|record| record.verifier == ImagePromotionVerifier::Kubewarden)
        {
            return Err(ImagePromotionError::MissingDefaultVerifier {
                artifact_digest: artifact_digest.clone(),
            });
        }
    }

    Ok(ImagePromotionReport {
        artifacts_checked: records_by_digest.len(),
        promotion_records_checked,
        kubewarden_verifier_records,
        kyverno_verifier_records,
    })
}

fn validate_image_promotion_record(
    record: &ImagePromotionRecord,
) -> Result<(), ImagePromotionError> {
    let artifact_ref =
        required_image_promotion_field(record, &record.artifact_ref, "artifact_ref")?;
    let artifact_digest =
        required_image_promotion_field(record, &record.artifact_digest, "artifact_digest")?;
    if !artifact_ref.contains('@') || !artifact_ref.contains("sha256:") {
        return Err(ImagePromotionError::InvalidArtifactRef {
            artifact_ref: record.artifact_ref.clone(),
        });
    }
    if !is_sha256_digest(artifact_digest) {
        return Err(ImagePromotionError::InvalidDigest {
            artifact_ref: record.artifact_ref.clone(),
        });
    }
    if !artifact_ref_pins_digest(artifact_ref, artifact_digest) {
        return Err(ImagePromotionError::DigestNotPinnedInArtifactRef {
            artifact_ref: record.artifact_ref.clone(),
            artifact_digest: record.artifact_digest.clone(),
        });
    }
    if !artifact_ref_matches_tier(artifact_ref, record.tier) {
        return Err(ImagePromotionError::TierTagMismatch {
            artifact_ref: record.artifact_ref.clone(),
            tier: record.tier,
        });
    }

    let cosign_identity =
        required_image_promotion_field(record, &record.cosign_identity, "cosign_identity")?;
    if !cosign_identity_matches_tier(cosign_identity, record.tier) {
        return Err(ImagePromotionError::InvalidCosignIdentity {
            artifact_ref: record.artifact_ref.clone(),
            tier: record.tier,
            cosign_identity: record.cosign_identity.clone(),
        });
    }

    let verifier_ref =
        required_image_promotion_field(record, &record.verifier_ref, "verifier_ref")?;
    if !verifier_ref_matches(verifier_ref, record.verifier) {
        return Err(ImagePromotionError::InvalidVerifierRef {
            artifact_ref: record.artifact_ref.clone(),
            verifier: record.verifier,
            verifier_ref: record.verifier_ref.clone(),
        });
    }

    let provenance_attestation_ref = required_image_promotion_field(
        record,
        &record.provenance_attestation_ref,
        "provenance_attestation_ref",
    )?;
    if !provenance_ref_valid(provenance_attestation_ref) {
        return Err(ImagePromotionError::InvalidProvenanceRef {
            artifact_ref: record.artifact_ref.clone(),
        });
    }

    let runner_kill_switch_ref = required_image_promotion_field(
        record,
        &record.runner_kill_switch_ref,
        "runner_kill_switch_ref",
    )?;
    if !runner_kill_switch_ref_valid(runner_kill_switch_ref) {
        return Err(ImagePromotionError::InvalidRunnerKillSwitchRef {
            artifact_ref: record.artifact_ref.clone(),
        });
    }

    if !record.signed {
        return Err(ImagePromotionError::UnsignedPromotion {
            artifact_ref: record.artifact_ref.clone(),
        });
    }
    if record.audit_event_type != "oya.audit.image_promotion" {
        return Err(ImagePromotionError::InvalidAuditEventType {
            artifact_ref: record.artifact_ref.clone(),
            audit_event_type: record.audit_event_type.clone(),
        });
    }
    Ok(())
}

fn required_image_promotion_field<'a>(
    record: &ImagePromotionRecord,
    value: &'a str,
    field: &'static str,
) -> Result<&'a str, ImagePromotionError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(ImagePromotionError::MissingField {
            artifact_ref: record.artifact_ref.clone(),
            field,
        })
    } else {
        Ok(trimmed)
    }
}

fn artifact_ref_matches_tier(artifact_ref: &str, tier: ImagePromotionTier) -> bool {
    artifact_ref.contains(&format!("-{}@", tier.name()))
}

fn artifact_ref_pins_digest(artifact_ref: &str, artifact_digest: &str) -> bool {
    match artifact_ref.rsplit_once('@') {
        Some((_, pinned_digest)) => pinned_digest == artifact_digest,
        None => false,
    }
}

fn cosign_identity_matches_tier(identity: &str, tier: ImagePromotionTier) -> bool {
    let lower = identity.to_ascii_lowercase();
    let tier_matches = lower.contains(tier.name())
        || matches!(tier, ImagePromotionTier::Prod) && lower.contains("production");
    tier_matches
        && [
            "oidc",
            "fulcio",
            "token.actions.githubusercontent.com",
            "spiffe://",
        ]
        .into_iter()
        .any(|marker| lower.contains(marker))
}

fn verifier_ref_matches(verifier_ref: &str, verifier: ImagePromotionVerifier) -> bool {
    let lower = verifier_ref.to_ascii_lowercase();
    lower.contains(verifier.name())
        && (lower.contains("signed-image")
            || lower.contains("signed-images")
            || lower.contains("verify-image"))
}

fn provenance_ref_valid(provenance_ref: &str) -> bool {
    let lower = provenance_ref.to_ascii_lowercase();
    lower.contains("provenance") && (lower.contains("intoto") || lower.contains("slsa"))
}

fn runner_kill_switch_ref_valid(kill_switch_ref: &str) -> bool {
    let lower = kill_switch_ref.to_ascii_lowercase();
    lower.contains("kill-switch")
        && lower.ends_with(".cedar")
        && (lower.contains("bootstrap-runner")
            || lower.contains("bootstrap-trust-roots")
            || lower.contains("stage-1-runner"))
}

fn release_artifact_map<A>(
    artifacts: A,
) -> Result<BTreeMap<String, ReleaseArtifact>, ReleaseSupplyChainError>
where
    A: IntoIterator<Item = ReleaseArtifact>,
{
    let mut map = BTreeMap::new();
    for artifact in artifacts {
        let artifact_ref = artifact.artifact_ref.trim();
        if artifact_ref.is_empty() {
            return Err(ReleaseSupplyChainError::InvalidArtifactRef {
                artifact_ref: artifact.artifact_ref,
            });
        }
        if !artifact_ref.contains('@') || !artifact_ref.contains("sha256:") {
            return Err(ReleaseSupplyChainError::InvalidArtifactRef {
                artifact_ref: artifact.artifact_ref,
            });
        }
        if map
            .insert(artifact_ref.to_string(), artifact.clone())
            .is_some()
        {
            return Err(ReleaseSupplyChainError::DuplicateArtifact {
                artifact_ref: artifact_ref.to_string(),
            });
        }
    }
    Ok(map)
}

fn release_evidence_map<E>(
    evidence_records: E,
) -> Result<BTreeMap<String, ReleaseSupplyChainEvidence>, ReleaseSupplyChainError>
where
    E: IntoIterator<Item = ReleaseSupplyChainEvidence>,
{
    let mut map = BTreeMap::new();
    for evidence in evidence_records {
        let artifact_ref = evidence.artifact_ref.trim();
        if artifact_ref.is_empty() {
            return Err(ReleaseSupplyChainError::InvalidArtifactRef {
                artifact_ref: evidence.artifact_ref,
            });
        }
        if map
            .insert(artifact_ref.to_string(), evidence.clone())
            .is_some()
        {
            return Err(ReleaseSupplyChainError::DuplicateEvidence {
                artifact_ref: artifact_ref.to_string(),
            });
        }
    }
    Ok(map)
}

fn validate_release_evidence_record(
    record: &ReleaseSupplyChainEvidence,
) -> Result<(), ReleaseSupplyChainError> {
    for (field, value) in [
        ("artifact_digest", &record.artifact_digest),
        ("release_version", &record.release_version),
        ("source_revision", &record.source_revision),
        ("sbom_spdx_ref", &record.sbom_spdx_ref),
        ("sbom_cyclonedx_ref", &record.sbom_cyclonedx_ref),
        ("cosign_signature_ref", &record.cosign_signature_ref),
        ("cosign_certificate_ref", &record.cosign_certificate_ref),
        (
            "trivy_filesystem_scan_ref",
            &record.trivy_filesystem_scan_ref,
        ),
        ("trivy_container_scan_ref", &record.trivy_container_scan_ref),
        ("trivy_iac_scan_ref", &record.trivy_iac_scan_ref),
        (
            "trivy_dependency_scan_ref",
            &record.trivy_dependency_scan_ref,
        ),
        (
            "provenance_attestation_ref",
            &record.provenance_attestation_ref,
        ),
        ("audit_event_type", &record.audit_event_type),
        ("attestor", &record.attestor),
    ] {
        if value.trim().is_empty() {
            return Err(ReleaseSupplyChainError::MissingField {
                artifact_ref: record.artifact_ref.clone(),
                field,
            });
        }
    }

    if !is_sha256_digest(&record.artifact_digest) {
        return Err(ReleaseSupplyChainError::InvalidDigest {
            artifact_ref: record.artifact_ref.clone(),
        });
    }
    if !artifact_ref_pins_digest(record.artifact_ref.trim(), &record.artifact_digest) {
        return Err(ReleaseSupplyChainError::DigestNotPinnedInArtifactRef {
            artifact_ref: record.artifact_ref.clone(),
            artifact_digest: record.artifact_digest.clone(),
        });
    }
    if !is_semver(&record.release_version) {
        return Err(ReleaseSupplyChainError::InvalidReleaseVersion {
            artifact_ref: record.artifact_ref.clone(),
            release_version: record.release_version.clone(),
        });
    }
    if !is_git_sha(&record.source_revision) {
        return Err(ReleaseSupplyChainError::InvalidSourceRevision {
            artifact_ref: record.artifact_ref.clone(),
            source_revision: record.source_revision.clone(),
        });
    }
    if record.rekor_log_index == 0 {
        return Err(ReleaseSupplyChainError::MissingRekorInclusion {
            artifact_ref: record.artifact_ref.clone(),
        });
    }
    if record.high_critical_findings_open != 0 {
        return Err(ReleaseSupplyChainError::OpenHighCriticalFindings {
            artifact_ref: record.artifact_ref.clone(),
            count: record.high_critical_findings_open,
        });
    }
    if !record.signed {
        return Err(ReleaseSupplyChainError::UnsignedEvidence {
            artifact_ref: record.artifact_ref.clone(),
        });
    }
    if record.audit_event_type != "oya.audit.builder_supply_attest" {
        return Err(ReleaseSupplyChainError::InvalidAuditEventType {
            artifact_ref: record.artifact_ref.clone(),
            audit_event_type: record.audit_event_type.clone(),
        });
    }
    for (field, value, suffix) in [
        ("sbom_spdx_ref", &record.sbom_spdx_ref, ".spdx.json"),
        (
            "sbom_cyclonedx_ref",
            &record.sbom_cyclonedx_ref,
            ".cyclonedx.json",
        ),
    ] {
        if !value.ends_with(suffix) {
            return Err(ReleaseSupplyChainError::InvalidSbomRef {
                artifact_ref: record.artifact_ref.clone(),
                field,
            });
        }
    }
    for (field, value) in [
        (
            "trivy_filesystem_scan_ref",
            &record.trivy_filesystem_scan_ref,
        ),
        ("trivy_container_scan_ref", &record.trivy_container_scan_ref),
        ("trivy_iac_scan_ref", &record.trivy_iac_scan_ref),
        (
            "trivy_dependency_scan_ref",
            &record.trivy_dependency_scan_ref,
        ),
    ] {
        if !value.to_ascii_lowercase().contains("trivy") {
            return Err(ReleaseSupplyChainError::InvalidTrivyRef {
                artifact_ref: record.artifact_ref.clone(),
                field,
            });
        }
    }
    Ok(())
}

fn is_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_git_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_semver(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .into_iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn require_adr0039_evidence(evidence: SupplyChainEvidence) -> Result<(), SupplyChainError> {
    let release_scope_declared =
        evidence.release_images_declared || evidence.release_empty_scope_declared;
    for (present, missing_evidence) in [
        (
            evidence.release_manifest_present,
            "release_manifest_present",
        ),
        (
            release_scope_declared,
            "release_images_declared_or_empty_scope_rationale",
        ),
        (
            evidence.trivy_filesystem_scan_wired,
            "trivy_filesystem_scan_wired",
        ),
        (
            evidence.trivy_container_scan_wired,
            "trivy_container_scan_wired",
        ),
        (evidence.trivy_iac_scan_wired, "trivy_iac_scan_wired"),
        (
            evidence.trivy_dependency_scan_wired,
            "trivy_dependency_scan_wired",
        ),
        (
            evidence.cosign_release_signing_wired,
            "cosign_release_signing_wired",
        ),
        (
            evidence.cosign_rekor_verification_wired,
            "cosign_rekor_verification_wired",
        ),
        (evidence.sbom_spdx_wired, "sbom_spdx_wired"),
        (evidence.sbom_cyclonedx_wired, "sbom_cyclonedx_wired"),
        (
            evidence.provenance_attestation_wired,
            "provenance_attestation_wired",
        ),
        (
            evidence.signed_commit_policy_wired,
            "signed_commit_policy_wired",
        ),
        (evidence.admission_policy_wired, "admission_policy_wired"),
    ] {
        if !present {
            return Err(SupplyChainError::MissingAdr0039Evidence { missing_evidence });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_source_only_bootstrap_with_dependency_scans_wired() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                evidence()
            ),
            Ok(SupplyChainReport {
                records_checked: 1,
                source_only_records: 1,
            })
        );
    }

    #[test]
    fn rejects_missing_dependency_scan_wiring() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    cargo_audit_check_wired: false,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::MissingCargoAuditCheck)
        );
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    cargo_deny_check_wired: false,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::MissingCargoDenyCheck)
        );
    }

    #[test]
    fn rejects_unpinned_third_party_actions() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    third_party_actions_pinned: false,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::UnpinnedThirdPartyAction)
        );
    }

    #[test]
    fn rejects_release_manifest_without_trivy_cosign_and_sbom_evidence() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    release_manifest_present: true,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::ReleaseManifestWithoutAdr0039Evidence {
                missing_evidence: "trivy_release_scan_wired",
            })
        );
    }

    #[test]
    fn rejects_full_adr0039_lane_without_required_evidence() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    require_adr0039_evidence: true,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::MissingAdr0039Evidence {
                missing_evidence: "release_manifest_present",
            })
        );
    }

    #[test]
    fn accepts_full_adr0039_lane_when_static_evidence_is_wired() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                full_adr0039_evidence()
            ),
            Ok(SupplyChainReport {
                records_checked: 1,
                source_only_records: 1,
            })
        );
    }

    #[test]
    fn accepts_full_adr0039_lane_with_explicit_pre_release_empty_scope() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    release_images_declared: false,
                    release_empty_scope_declared: true,
                    ..full_adr0039_evidence()
                }
            ),
            Ok(SupplyChainReport {
                records_checked: 1,
                source_only_records: 1,
            })
        );
    }

    #[test]
    fn rejects_full_adr0039_lane_without_release_artifacts_or_empty_scope_rationale() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "source-only")],
                SupplyChainEvidence {
                    release_images_declared: false,
                    release_empty_scope_declared: false,
                    ..full_adr0039_evidence()
                }
            ),
            Err(SupplyChainError::MissingAdr0039Evidence {
                missing_evidence: "release_images_declared_or_empty_scope_rationale",
            })
        );
    }

    #[test]
    fn rejects_sbom_or_signed_claim_without_evidence() {
        assert_eq!(
            validate_supply_chain(
                [record("oya-intelligence-capability-kernel", "sbom")],
                evidence()
            ),
            Err(SupplyChainError::UnsupportedAttestationClaim {
                subject: "oya-intelligence-capability-kernel".into(),
                attestation: "sbom".into(),
                missing_evidence: "sbom_dual_format_wired",
            })
        );
        assert_eq!(
            validate_supply_chain(
                [record(
                    "oya-intelligence-capability-kernel",
                    "signed-provenance"
                )],
                SupplyChainEvidence {
                    sbom_dual_format_wired: true,
                    ..evidence()
                }
            ),
            Err(SupplyChainError::UnsupportedAttestationClaim {
                subject: "oya-intelligence-capability-kernel".into(),
                attestation: "signed-provenance".into(),
                missing_evidence: "cosign_release_signing_wired",
            })
        );
    }

    #[test]
    fn rejects_empty_catalog_records() {
        assert_eq!(
            validate_supply_chain([], evidence()),
            Err(SupplyChainError::NoCatalogRecords)
        );
    }

    #[test]
    fn accepts_release_supply_chain_evidence_for_every_artifact() {
        assert_eq!(
            validate_release_supply_chain([release_artifact()], [release_evidence()]),
            Ok(ReleaseSupplyChainReport {
                artifacts_checked: 1,
                evidence_records_checked: 1,
            })
        );
    }

    #[test]
    fn accepts_signed_image_promotion_ladder() {
        assert_eq!(
            validate_image_promotion_pipeline([
                image_promotion_record(ImagePromotionTier::Dev, ImagePromotionVerifier::Kubewarden),
                image_promotion_record(
                    ImagePromotionTier::Staging,
                    ImagePromotionVerifier::Kubewarden,
                ),
                image_promotion_record(ImagePromotionTier::Prod, ImagePromotionVerifier::Kyverno),
            ]),
            Ok(ImagePromotionReport {
                artifacts_checked: 1,
                promotion_records_checked: 3,
                kubewarden_verifier_records: 2,
                kyverno_verifier_records: 1,
            })
        );
    }

    #[test]
    fn rejects_image_promotion_ladder_without_kubewarden_default_verifier() {
        assert_eq!(
            validate_image_promotion_pipeline([
                image_promotion_record(ImagePromotionTier::Dev, ImagePromotionVerifier::Kyverno),
                image_promotion_record(
                    ImagePromotionTier::Staging,
                    ImagePromotionVerifier::Kyverno,
                ),
                image_promotion_record(ImagePromotionTier::Prod, ImagePromotionVerifier::Kyverno),
            ]),
            Err(ImagePromotionError::MissingDefaultVerifier {
                artifact_digest:
                    "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".into(),
            })
        );
    }

    #[test]
    fn rejects_image_promotion_artifact_ref_with_mismatched_pinned_digest() {
        let mut record =
            image_promotion_record(ImagePromotionTier::Dev, ImagePromotionVerifier::Kubewarden);
        record.artifact_ref = format!(
            "ghcr.io/oyatie/tooling:{}-dev@sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
            record.artifact_digest
        );

        assert_eq!(
            validate_image_promotion_pipeline([record]),
            Err(ImagePromotionError::DigestNotPinnedInArtifactRef {
                artifact_ref: "ghcr.io/oyatie/tooling:sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef-dev@sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".into(),
                artifact_digest:
                    "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".into(),
            })
        );
    }

    #[test]
    fn accepts_pre_release_supply_chain_empty_scope_only_when_declared() {
        assert_eq!(
            validate_pre_release_supply_chain(
                Vec::<ReleaseArtifact>::new(),
                Vec::<ReleaseSupplyChainEvidence>::new(),
                true
            ),
            Ok(ReleaseSupplyChainReport {
                artifacts_checked: 0,
                evidence_records_checked: 0,
            })
        );
        assert_eq!(
            validate_pre_release_supply_chain(
                Vec::<ReleaseArtifact>::new(),
                Vec::<ReleaseSupplyChainEvidence>::new(),
                false
            ),
            Err(ReleaseSupplyChainError::MissingPreReleaseEmptyScopeRationale)
        );
    }

    #[test]
    fn accepts_pre_release_artifacts_before_attestation_records_exist() {
        assert_eq!(
            validate_pre_release_supply_chain(
                [release_artifact()],
                Vec::<ReleaseSupplyChainEvidence>::new(),
                false
            ),
            Ok(ReleaseSupplyChainReport {
                artifacts_checked: 1,
                evidence_records_checked: 0,
            })
        );
    }

    #[test]
    fn pre_release_evidence_still_must_cover_every_artifact() {
        assert_eq!(
            validate_pre_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    artifact_ref: release_artifact_ref("other"),
                    ..release_evidence()
                }],
                false
            ),
            Err(ReleaseSupplyChainError::MissingEvidenceForArtifact {
                artifact_ref: release_artifact().artifact_ref,
            })
        );
    }

    #[test]
    fn rejects_release_artifact_without_evidence() {
        assert_eq!(
            validate_release_supply_chain([release_artifact()], []),
            Err(ReleaseSupplyChainError::NoEvidenceRecords)
        );
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    artifact_ref: release_artifact_ref("other"),
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::MissingEvidenceForArtifact {
                artifact_ref: release_artifact().artifact_ref,
            })
        );
    }

    #[test]
    fn rejects_release_evidence_with_open_high_critical_findings() {
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    high_critical_findings_open: 1,
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::OpenHighCriticalFindings {
                artifact_ref: release_artifact().artifact_ref,
                count: 1,
            })
        );
    }

    #[test]
    fn rejects_release_evidence_without_rekor_or_signature() {
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    rekor_log_index: 0,
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::MissingRekorInclusion {
                artifact_ref: release_artifact().artifact_ref,
            })
        );
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    signed: false,
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::UnsignedEvidence {
                artifact_ref: release_artifact().artifact_ref,
            })
        );
    }

    #[test]
    fn rejects_release_evidence_not_pinned_to_artifact_digest() {
        assert_eq!(
            validate_release_supply_chain(
                [release_artifact()],
                [ReleaseSupplyChainEvidence {
                    artifact_digest:
                        "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
                            .into(),
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::DigestNotPinnedInArtifactRef {
                artifact_ref: release_artifact().artifact_ref,
                artifact_digest:
                    "sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".into(),
            })
        );
    }

    #[test]
    fn rejects_release_evidence_artifact_ref_with_mismatched_pinned_digest() {
        let artifact_digest =
            "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let artifact_ref = format!(
            "ghcr.io/oyatie/tooling:{artifact_digest}@sha256:fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"
        );

        assert_eq!(
            validate_release_supply_chain(
                [ReleaseArtifact {
                    artifact_ref: artifact_ref.clone(),
                }],
                [ReleaseSupplyChainEvidence {
                    artifact_ref: artifact_ref.clone(),
                    ..release_evidence()
                }]
            ),
            Err(ReleaseSupplyChainError::DigestNotPinnedInArtifactRef {
                artifact_ref,
                artifact_digest: artifact_digest.into(),
            })
        );
    }

    #[test]
    fn accepts_vulnerability_intelligence_closed_loop_contract() {
        assert_eq!(
            validate_vulnerability_intelligence_contract(&vulnerability_contract()),
            Ok(VulnerabilityIntelligenceReport {
                feeds_checked: 5,
                sbom_formats_checked: 2,
                vex_statuses_checked: 4,
                priority_signals_checked: 4,
                remediation_slas_checked: 4,
                admission_blocks_checked: 6,
            })
        );
    }

    #[test]
    fn rejects_vulnerability_intelligence_scanner_cli_as_canonical_surface() {
        let mut contract = vulnerability_contract();
        contract.canonical_surface = VulnerabilityProductSurface::ScannerCli;

        assert_eq!(
            validate_vulnerability_intelligence_contract(&contract),
            Err(VulnerabilityIntelligenceError::ScannerCliDeclaredAsCanonicalSurface)
        );
    }

    #[test]
    fn rejects_vulnerability_intelligence_missing_required_ingestion_feed() {
        let mut contract = vulnerability_contract();
        contract
            .advisory_feeds
            .retain(|feed| *feed != VulnerabilityAdvisoryFeed::RustSec);

        assert_eq!(
            validate_vulnerability_intelligence_contract(&contract),
            Err(VulnerabilityIntelligenceError::MissingAdvisoryFeed { feed: "rustsec" })
        );
    }

    #[test]
    fn rejects_vulnerability_intelligence_without_complete_vex_and_priority_coverage() {
        let mut contract = vulnerability_contract();
        contract
            .vex_statuses
            .retain(|status| *status != VexStatus::UnderInvestigation);
        assert_eq!(
            validate_vulnerability_intelligence_contract(&contract),
            Err(VulnerabilityIntelligenceError::MissingVexStatus {
                status: "under_investigation",
            })
        );

        let mut contract = vulnerability_contract();
        contract
            .priority_signals
            .retain(|signal| *signal != VulnerabilityPrioritySignal::Ssvc);
        assert_eq!(
            validate_vulnerability_intelligence_contract(&contract),
            Err(VulnerabilityIntelligenceError::MissingPrioritySignal { signal: "SSVC" })
        );
    }

    #[test]
    fn rejects_vulnerability_intelligence_missing_signed_decision_evidence() {
        let mut contract = vulnerability_contract();
        contract.audit_evidence.admission_verdict_signed = false;

        assert_eq!(
            validate_vulnerability_intelligence_contract(&contract),
            Err(VulnerabilityIntelligenceError::MissingSignedAuditEvidence {
                field: "admission_verdict_signed",
            })
        );
    }

    #[test]
    fn rejects_vulnerability_intelligence_loose_sla_and_open_ended_exceptions() {
        let mut contract = vulnerability_contract();
        contract.remediation_slas[0].max_days = 8;
        assert_eq!(
            validate_vulnerability_intelligence_contract(&contract),
            Err(VulnerabilityIntelligenceError::RemediationSlaTooLoose {
                class: "kev_or_actively_exploited",
                max_days: 8,
                allowed_days: 7,
            })
        );

        let mut contract = vulnerability_contract();
        contract.exception_policy.requires_expiry = false;
        assert_eq!(
            validate_vulnerability_intelligence_contract(&contract),
            Err(VulnerabilityIntelligenceError::ExceptionPolicyMissingExpiry)
        );
    }

    #[test]
    fn rejects_vulnerability_intelligence_missing_deployment_block() {
        let mut contract = vulnerability_contract();
        contract.admission_policy.block_fix_available_past_sla = false;

        assert_eq!(
            validate_vulnerability_intelligence_contract(&contract),
            Err(VulnerabilityIntelligenceError::MissingAdmissionBlock {
                block: "fix_available_past_sla",
            })
        );
    }

    fn vulnerability_contract() -> VulnerabilityIntelligenceContract {
        VulnerabilityIntelligenceContract {
            lane_id: VULNERABILITY_INTELLIGENCE_LANE_ID.into(),
            canonical_surface: VulnerabilityProductSurface::CloudNativeApi,
            advisory_feeds: vec![
                VulnerabilityAdvisoryFeed::CveNvd,
                VulnerabilityAdvisoryFeed::Osv,
                VulnerabilityAdvisoryFeed::RustSec,
                VulnerabilityAdvisoryFeed::GitHubAdvisories,
                VulnerabilityAdvisoryFeed::VendorAdvisories,
            ],
            sbom_formats: vec![SbomFormat::Spdx, SbomFormat::CycloneDx],
            vex_statuses: vec![
                VexStatus::Affected,
                VexStatus::NotAffected,
                VexStatus::Fixed,
                VexStatus::UnderInvestigation,
            ],
            priority_signals: vec![
                VulnerabilityPrioritySignal::CisaKev,
                VulnerabilityPrioritySignal::Epss,
                VulnerabilityPrioritySignal::Cvss,
                VulnerabilityPrioritySignal::Ssvc,
            ],
            remediation_slas: vec![
                RemediationSla {
                    class: RemediationSlaClass::KevOrActivelyExploited,
                    max_days: 7,
                    deployment_blocking: true,
                },
                RemediationSla {
                    class: RemediationSlaClass::Critical,
                    max_days: 7,
                    deployment_blocking: true,
                },
                RemediationSla {
                    class: RemediationSlaClass::High,
                    max_days: 14,
                    deployment_blocking: true,
                },
                RemediationSla {
                    class: RemediationSlaClass::Medium,
                    max_days: 30,
                    deployment_blocking: false,
                },
            ],
            exception_policy: VulnerabilityExceptionPolicy {
                max_ttl_days: 30,
                requires_owner: true,
                requires_expiry: true,
                requires_vex_justification: true,
                requires_audit_event: true,
            },
            audit_evidence: VulnerabilityAuditEvidencePolicy {
                advisory_snapshot_signed: true,
                sbom_artifacts_signed: true,
                vex_artifacts_signed: true,
                priority_decision_signed: true,
                exception_decision_signed: true,
                admission_verdict_signed: true,
                audit_event_type: VULNERABILITY_DECISION_AUDIT_EVENT.into(),
                retention_days: VULNERABILITY_MIN_AUDIT_RETENTION_DAYS,
            },
            admission_policy: VulnerabilityAdmissionPolicy {
                block_missing_or_unsigned_sbom: true,
                block_missing_vex: true,
                block_expired_exception: true,
                block_kev_or_exploited: true,
                block_fix_available_past_sla: true,
                block_unknown_component_match: true,
            },
        }
    }

    fn record(subject: &str, attestation: &str) -> SupplyChainRecord {
        SupplyChainRecord {
            subject: subject.into(),
            attestation: attestation.into(),
        }
    }

    fn release_artifact() -> ReleaseArtifact {
        ReleaseArtifact {
            artifact_ref: release_artifact_ref("tooling"),
        }
    }

    fn release_artifact_ref(name: &str) -> String {
        format!(
            "ghcr.io/oyatie/{name}@sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        )
    }

    fn release_evidence() -> ReleaseSupplyChainEvidence {
        ReleaseSupplyChainEvidence {
            artifact_ref: release_artifact().artifact_ref,
            artifact_digest:
                "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".into(),
            release_version: "0.1.0".into(),
            source_revision: "0123456789abcdef0123456789abcdef01234567".into(),
            sbom_spdx_ref: "artifact://release/0.1.0/tooling.spdx.json".into(),
            sbom_cyclonedx_ref: "artifact://release/0.1.0/tooling.cyclonedx.json".into(),
            cosign_signature_ref: "rekor://log/123/signature".into(),
            cosign_certificate_ref: "rekor://log/123/certificate".into(),
            rekor_log_index: 123,
            trivy_filesystem_scan_ref: "artifact://release/0.1.0/trivy-fs.sarif".into(),
            trivy_container_scan_ref: "artifact://release/0.1.0/trivy-image.sarif".into(),
            trivy_iac_scan_ref: "artifact://release/0.1.0/trivy-iac.sarif".into(),
            trivy_dependency_scan_ref: "artifact://release/0.1.0/trivy-dep.sarif".into(),
            provenance_attestation_ref: "artifact://release/0.1.0/provenance.intoto.jsonl".into(),
            audit_event_type: "oya.audit.builder_supply_attest".into(),
            attestor: "axis-foundry".into(),
            high_critical_findings_open: 0,
            signed: true,
        }
    }

    fn image_promotion_record(
        tier: ImagePromotionTier,
        verifier: ImagePromotionVerifier,
    ) -> ImagePromotionRecord {
        let digest = "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        ImagePromotionRecord {
            artifact_ref: format!(
                "ghcr.io/oyatie/tooling:0123456789abcdef0123456789abcdef01234567-{}@{digest}",
                tier.name()
            ),
            artifact_digest: digest.into(),
            tier,
            cosign_identity: format!(
                "https://token.actions.githubusercontent.com/oyatie/image-promotion-{}-oidc",
                tier.name()
            ),
            verifier,
            verifier_ref: format!(
                "infra/{}/policies/require-signed-images.yaml",
                verifier.name()
            ),
            provenance_attestation_ref: "artifact://release/0.1.0/tooling-provenance.intoto.jsonl"
                .into(),
            runner_kill_switch_ref: "artifact://fixtures/bootstrap-runner-kill-switch.cedar".into(),
            audit_event_type: "oya.audit.image_promotion".into(),
            signed: true,
        }
    }

    fn evidence() -> SupplyChainEvidence {
        SupplyChainEvidence {
            deny_config_present: true,
            cargo_deny_check_wired: true,
            cargo_audit_check_wired: true,
            third_party_actions_pinned: true,
            require_adr0039_evidence: false,
            release_manifest_present: false,
            release_images_declared: false,
            release_empty_scope_declared: false,
            trivy_release_scan_wired: false,
            trivy_filesystem_scan_wired: false,
            trivy_container_scan_wired: false,
            trivy_iac_scan_wired: false,
            trivy_dependency_scan_wired: false,
            cosign_release_signing_wired: false,
            cosign_rekor_verification_wired: false,
            sbom_dual_format_wired: false,
            sbom_spdx_wired: false,
            sbom_cyclonedx_wired: false,
            provenance_attestation_wired: false,
            signed_commit_policy_wired: false,
            admission_policy_wired: false,
        }
    }

    fn full_adr0039_evidence() -> SupplyChainEvidence {
        SupplyChainEvidence {
            require_adr0039_evidence: true,
            release_manifest_present: true,
            release_images_declared: true,
            release_empty_scope_declared: false,
            trivy_release_scan_wired: true,
            trivy_filesystem_scan_wired: true,
            trivy_container_scan_wired: true,
            trivy_iac_scan_wired: true,
            trivy_dependency_scan_wired: true,
            cosign_release_signing_wired: true,
            cosign_rekor_verification_wired: true,
            sbom_dual_format_wired: true,
            sbom_spdx_wired: true,
            sbom_cyclonedx_wired: true,
            provenance_attestation_wired: true,
            signed_commit_policy_wired: true,
            admission_policy_wired: true,
            ..evidence()
        }
    }
}
