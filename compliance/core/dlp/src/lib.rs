//! Workspace DLP kernel.
//!
//! Typed kernel records for the Workspace GA DLP surface named by
//! `docs/products/workspace/PRD.md` and ADR-0029. The kernel owns per-tenant
//! policy records, rule/finding validation, and admin-review hold decisions;
//! mail, drive, chat, forms, sites, scanners, and regional-pack adapters remain
//! outside this crate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use data_boundary_kernel::{Classified, DataClass, DataClassMatcher, PrivacyDataClass};

const DLP_POLICY_SCHEMA_VERSION: u32 = 1;
const DLP_SCAN_SCHEMA_VERSION: u32 = 1;
const DLP_VERDICT_SCHEMA_VERSION: u32 = 1;
const SHA256_PREFIX: &str = "sha256:";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DlpError {
    InvalidPolicyId,
    InvalidTenantId,
    InvalidRegion,
    InvalidQueueId,
    EmptyRuleSet,
    InvalidRuleId,
    InvalidRuleName,
    DuplicateRuleId,
    DuplicateRulePriority,
    InvalidDetectorRef,
    HighRiskRuleMustHold,
    InvalidScanId,
    InvalidActorRef,
    InvalidContentRef,
    InvalidFindingId,
    UnknownFindingRule,
    DuplicateFindingId,
    InvalidEvidenceHash,
    FindingActionMismatch,
    InvalidVerdictAction,
    MissingHoldQueue,
    UnexpectedHoldQueue,
    InvalidHoldUntil,
    InvalidTimeOrder,
    InvalidDataClass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DlpSurface {
    MailOutbound,
    MailInbound,
    DriveUpload,
    ChatMessage,
    FormSubmission,
    SitePublish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DlpAction {
    AllowWithAudit,
    Redact,
    Quarantine,
    AdminReviewHold,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DlpSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DlpPolicyCreate {
    pub policy_id: String,             // data_class: INTERNAL_ONLY
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub region: String,                // data_class: INTERNAL_ONLY
    pub admin_review_queue_id: String, // data_class: INTERNAL_ONLY
    pub rules: Vec<DlpRule>,           // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DlpPolicy {
    pub policy_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub region: Classified<String>,    // data_class: INTERNAL_ONLY
    pub admin_review_queue_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub rules: Classified<Vec<DlpRule>>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DlpRuleCreate {
    pub rule_id: String,                      // data_class: INTERNAL_ONLY
    pub name: String,                         // data_class: INTERNAL_ONLY
    pub surface: DlpSurface,                  // data_class: INTERNAL_ONLY
    pub detector_ref: String,                 // data_class: INTERNAL_ONLY
    pub matched_data_class: PrivacyDataClass, // data_class: INTERNAL_ONLY
    pub action: DlpAction,                    // data_class: INTERNAL_ONLY
    pub severity: DlpSeverity,                // data_class: INTERNAL_ONLY
    pub priority: u32,                        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DlpRule {
    pub rule_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub name: Classified<String>,         // data_class: INTERNAL_ONLY
    pub surface: Classified<DlpSurface>,  // data_class: INTERNAL_ONLY
    pub detector_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub matched_data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub action: Classified<DlpAction>,    // data_class: INTERNAL_ONLY
    pub severity: Classified<DlpSeverity>, // data_class: INTERNAL_ONLY
    pub priority: Classified<u32>,        // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DlpScanRequestCreate {
    pub scan_id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub region: String,                        // data_class: INTERNAL_ONLY
    pub surface: DlpSurface,                   // data_class: INTERNAL_ONLY
    pub actor_ref: String,                     // data_class: PII_IDENTIFYING
    pub content_ref: String,                   // data_class: INTERNAL_ONLY
    pub declared_data_class: PrivacyDataClass, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DlpScanRequest {
    pub scan_id: Classified<String>,     // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub region: Classified<String>,      // data_class: INTERNAL_ONLY
    pub surface: Classified<DlpSurface>, // data_class: INTERNAL_ONLY
    pub actor_ref: Classified<String>,   // data_class: PII_IDENTIFYING
    pub content_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub declared_data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub requested_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DlpFinding {
    pub finding_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub rule_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub detector_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub matched_data_class: Classified<PrivacyDataClass>, // data_class: INTERNAL_ONLY
    pub evidence_hash: Classified<String>, // data_class: INTERNAL_ONLY
    pub severity: Classified<DlpSeverity>, // data_class: INTERNAL_ONLY
    pub action: Classified<DlpAction>,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DlpScanVerdictCreate {
    pub scan_id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub policy_id: String,                     // data_class: INTERNAL_ONLY
    pub findings: Vec<DlpFinding>,             // data_class: INTERNAL_ONLY
    pub final_action: DlpAction,               // data_class: INTERNAL_ONLY
    pub admin_review_queue_id: Option<String>, // data_class: INTERNAL_ONLY
    pub hold_until_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub decided_at_epoch_seconds: u64,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DlpScanVerdict {
    pub scan_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub policy_id: Classified<String>, // data_class: INTERNAL_ONLY
    pub findings: Classified<Vec<DlpFinding>>, // data_class: INTERNAL_ONLY
    pub final_action: Classified<DlpAction>, // data_class: INTERNAL_ONLY
    pub admin_review_queue_id: Classified<Option<String>>, // data_class: INTERNAL_ONLY
    pub hold_until_epoch_seconds: Classified<Option<u64>>, // data_class: INTERNAL_ONLY
    pub decided_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: INTERNAL_ONLY
}

pub trait DlpScanner {
    fn scan(&self, request: &DlpScanRequest) -> Result<Vec<DlpFinding>, DlpError>;
}

impl DlpPolicy {
    pub fn new(input: DlpPolicyCreate) -> Result<Self, DlpError> {
        validate_non_empty(&input.policy_id, DlpError::InvalidPolicyId)?;
        validate_non_empty(&input.tenant_id, DlpError::InvalidTenantId)?;
        validate_non_empty(&input.region, DlpError::InvalidRegion)?;
        validate_non_empty(&input.admin_review_queue_id, DlpError::InvalidQueueId)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        validate_rules(&input.rules)?;
        Ok(Self {
            policy_id: internal(input.policy_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            admin_review_queue_id: internal(input.admin_review_queue_id),
            rules: internal(input.rules),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: internal(DLP_POLICY_SCHEMA_VERSION),
        })
    }

    pub fn rule_ids(&self) -> BTreeSet<String> {
        self.rules
            .value
            .iter()
            .map(|rule| rule.rule_id.value.clone())
            .collect()
    }
}

impl DlpRule {
    pub fn new(input: DlpRuleCreate) -> Result<Self, DlpError> {
        validate_non_empty(&input.rule_id, DlpError::InvalidRuleId)?;
        validate_non_empty(&input.name, DlpError::InvalidRuleName)?;
        validate_non_empty(&input.detector_ref, DlpError::InvalidDetectorRef)?;
        validate_high_risk_action(input.matched_data_class, input.action)?;
        Ok(Self {
            rule_id: internal(input.rule_id),
            name: internal(input.name),
            surface: internal(input.surface),
            detector_ref: internal(input.detector_ref),
            matched_data_class: internal(input.matched_data_class),
            action: internal(input.action),
            severity: internal(input.severity),
            priority: internal(input.priority),
        })
    }
}

impl DlpScanRequest {
    pub fn new(input: DlpScanRequestCreate, policy: &DlpPolicy) -> Result<Self, DlpError> {
        validate_non_empty(&input.scan_id, DlpError::InvalidScanId)?;
        validate_non_empty(&input.tenant_id, DlpError::InvalidTenantId)?;
        validate_non_empty(&input.region, DlpError::InvalidRegion)?;
        validate_non_empty(&input.actor_ref, DlpError::InvalidActorRef)?;
        validate_non_empty(&input.content_ref, DlpError::InvalidContentRef)?;
        if input.tenant_id != policy.tenant_id.value {
            return Err(DlpError::InvalidTenantId);
        }
        if input.region != policy.region.value {
            return Err(DlpError::InvalidRegion);
        }
        Ok(Self {
            scan_id: internal(input.scan_id),
            tenant_id: internal(input.tenant_id),
            region: internal(input.region),
            surface: internal(input.surface),
            actor_ref: Classified::new(input.actor_ref, dlp_actor_data_class()),
            content_ref: internal(input.content_ref),
            declared_data_class: internal(input.declared_data_class),
            requested_at_epoch_seconds: internal(input.requested_at_epoch_seconds),
            schema_version: internal(DLP_SCAN_SCHEMA_VERSION),
        })
    }
}

impl DlpFinding {
    pub fn new(
        finding_id: String,
        rule: &DlpRule,
        evidence_hash: String,
    ) -> Result<Self, DlpError> {
        validate_non_empty(&finding_id, DlpError::InvalidFindingId)?;
        validate_checksum(&evidence_hash)?;
        Ok(Self {
            finding_id: internal(finding_id),
            rule_id: internal(rule.rule_id.value.clone()),
            detector_ref: internal(rule.detector_ref.value.clone()),
            matched_data_class: internal(rule.matched_data_class.value),
            evidence_hash: internal(evidence_hash),
            severity: internal(rule.severity.value),
            action: internal(rule.action.value),
        })
    }
}

impl DlpScanVerdict {
    pub fn new(
        input: DlpScanVerdictCreate,
        request: &DlpScanRequest,
        policy: &DlpPolicy,
    ) -> Result<Self, DlpError> {
        validate_non_empty(&input.scan_id, DlpError::InvalidScanId)?;
        validate_non_empty(&input.tenant_id, DlpError::InvalidTenantId)?;
        validate_non_empty(&input.policy_id, DlpError::InvalidPolicyId)?;
        if input.scan_id != request.scan_id.value {
            return Err(DlpError::InvalidScanId);
        }
        if input.tenant_id != request.tenant_id.value || input.tenant_id != policy.tenant_id.value {
            return Err(DlpError::InvalidTenantId);
        }
        if input.policy_id != policy.policy_id.value {
            return Err(DlpError::InvalidPolicyId);
        }
        validate_findings(&input.findings, policy)?;
        validate_final_action(
            &input.findings,
            input.final_action,
            input.admin_review_queue_id.as_deref(),
            input.hold_until_epoch_seconds,
            policy,
            input.decided_at_epoch_seconds,
        )?;
        Ok(Self {
            scan_id: internal(input.scan_id),
            tenant_id: internal(input.tenant_id),
            policy_id: internal(input.policy_id),
            findings: internal(input.findings),
            final_action: internal(input.final_action),
            admin_review_queue_id: internal(input.admin_review_queue_id),
            hold_until_epoch_seconds: internal(input.hold_until_epoch_seconds),
            decided_at_epoch_seconds: internal(input.decided_at_epoch_seconds),
            schema_version: internal(DLP_VERDICT_SCHEMA_VERSION),
        })
    }
}

pub fn default_workspace_dlp_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn dlp_actor_data_class() -> PrivacyDataClass {
    PrivacyDataClass::pii_identifying()
}

pub fn workspace_dlp_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, DlpError> {
    PrivacyDataClass::new(data_class).map_err(|_| DlpError::InvalidDataClass)
}

fn validate_rules(rules: &[DlpRule]) -> Result<(), DlpError> {
    if rules.is_empty() {
        return Err(DlpError::EmptyRuleSet);
    }
    let mut ids = BTreeSet::new();
    let mut priorities = BTreeSet::new();
    for rule in rules {
        validate_non_empty(&rule.rule_id.value, DlpError::InvalidRuleId)?;
        validate_non_empty(&rule.name.value, DlpError::InvalidRuleName)?;
        validate_non_empty(&rule.detector_ref.value, DlpError::InvalidDetectorRef)?;
        validate_high_risk_action(rule.matched_data_class.value, rule.action.value)?;
        if !ids.insert(rule.rule_id.value.clone()) {
            return Err(DlpError::DuplicateRuleId);
        }
        if !priorities.insert(rule.priority.value) {
            return Err(DlpError::DuplicateRulePriority);
        }
    }
    Ok(())
}

fn validate_high_risk_action(
    matched_data_class: PrivacyDataClass,
    action: DlpAction,
) -> Result<(), DlpError> {
    let data_class = matched_data_class.data_class();
    let high_risk = matches!(
        data_class,
        DataClass::Phi | DataClass::Pci | DataClass::SensitivePipaArticle23
    ) || DataClassMatcher::RegulatedFinancial.matches(data_class);
    if high_risk && action != DlpAction::AdminReviewHold {
        Err(DlpError::HighRiskRuleMustHold)
    } else {
        Ok(())
    }
}

fn validate_findings(findings: &[DlpFinding], policy: &DlpPolicy) -> Result<(), DlpError> {
    let known_rule_ids = policy.rule_ids();
    let mut ids = BTreeSet::new();
    for finding in findings {
        validate_non_empty(&finding.finding_id.value, DlpError::InvalidFindingId)?;
        if !known_rule_ids.contains(&finding.rule_id.value) {
            return Err(DlpError::UnknownFindingRule);
        }
        validate_checksum(&finding.evidence_hash.value)?;
        let Some(rule) = policy
            .rules
            .value
            .iter()
            .find(|rule| rule.rule_id.value == finding.rule_id.value)
        else {
            return Err(DlpError::UnknownFindingRule);
        };
        if finding.action.value != rule.action.value
            || finding.matched_data_class.value != rule.matched_data_class.value
            || finding.detector_ref.value != rule.detector_ref.value
        {
            return Err(DlpError::FindingActionMismatch);
        }
        if !ids.insert(finding.finding_id.value.clone()) {
            return Err(DlpError::DuplicateFindingId);
        }
    }
    Ok(())
}

fn validate_final_action(
    findings: &[DlpFinding],
    final_action: DlpAction,
    admin_review_queue_id: Option<&str>,
    hold_until_epoch_seconds: Option<u64>,
    policy: &DlpPolicy,
    decided_at_epoch_seconds: u64,
) -> Result<(), DlpError> {
    if findings.is_empty() && final_action != DlpAction::AllowWithAudit {
        return Err(DlpError::InvalidVerdictAction);
    }
    let needs_hold = findings
        .iter()
        .any(|finding| finding.action.value == DlpAction::AdminReviewHold)
        || final_action == DlpAction::AdminReviewHold;
    if needs_hold {
        let Some(queue_id) = admin_review_queue_id else {
            return Err(DlpError::MissingHoldQueue);
        };
        if queue_id != policy.admin_review_queue_id.value {
            return Err(DlpError::InvalidQueueId);
        }
        let Some(hold_until) = hold_until_epoch_seconds else {
            return Err(DlpError::InvalidHoldUntil);
        };
        if hold_until <= decided_at_epoch_seconds {
            return Err(DlpError::InvalidHoldUntil);
        }
    } else if admin_review_queue_id.is_some() || hold_until_epoch_seconds.is_some() {
        return Err(DlpError::UnexpectedHoldQueue);
    }
    if !findings.is_empty()
        && !findings
            .iter()
            .any(|finding| finding.action.value == final_action)
    {
        return Err(DlpError::InvalidVerdictAction);
    }
    Ok(())
}

fn validate_checksum(checksum: &str) -> Result<(), DlpError> {
    if checksum.trim() != checksum
        || !checksum.starts_with(SHA256_PREFIX)
        || checksum.len() == SHA256_PREFIX.len()
        || checksum.chars().any(char::is_control)
    {
        Err(DlpError::InvalidEvidenceHash)
    } else {
        Ok(())
    }
}

fn validate_time_order(created_at: u64, updated_at: u64) -> Result<(), DlpError> {
    if updated_at < created_at {
        Err(DlpError::InvalidTimeOrder)
    } else {
        Ok(())
    }
}

fn validate_non_empty(value: &str, error: DlpError) -> Result<(), DlpError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_boundary_kernel::{DataClassification, OperationalDataClass};

    fn privacy_class(data_class: DataClass) -> PrivacyDataClass {
        PrivacyDataClass::new(data_class).unwrap()
    }

    fn rule(rule_id: &str, data_class: DataClass, action: DlpAction, priority: u32) -> DlpRule {
        DlpRule::new(DlpRuleCreate {
            rule_id: rule_id.into(),
            name: format!("Rule {rule_id}"),
            surface: DlpSurface::MailOutbound,
            detector_ref: format!("regional-pack:{rule_id}"),
            matched_data_class: privacy_class(data_class),
            action,
            severity: DlpSeverity::High,
            priority,
        })
        .unwrap()
    }

    fn policy() -> DlpPolicy {
        DlpPolicy::new(DlpPolicyCreate {
            policy_id: "dlp-policy-1".into(),
            tenant_id: "tenant-1".into(),
            region: "region-alpha1".into(),
            admin_review_queue_id: "admin-review-queue".into(),
            rules: vec![
                rule("phi", DataClass::Phi, DlpAction::AdminReviewHold, 10),
                rule("pii", DataClass::PiiIdentifying, DlpAction::Redact, 20),
            ],
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_010,
        })
        .unwrap()
    }

    fn request() -> DlpScanRequest {
        DlpScanRequest::new(
            DlpScanRequestCreate {
                scan_id: "scan-1".into(),
                tenant_id: "tenant-1".into(),
                region: "region-alpha1".into(),
                surface: DlpSurface::MailOutbound,
                actor_ref: "user:sender@example.com".into(),
                content_ref: "mail:message-1".into(),
                declared_data_class: default_workspace_dlp_data_class(),
                requested_at_epoch_seconds: 1_700_000_020,
            },
            &policy(),
        )
        .unwrap()
    }

    #[test]
    fn high_risk_rules_require_admin_review_hold() {
        assert_eq!(
            DlpRule::new(DlpRuleCreate {
                rule_id: "pci".into(),
                name: "PCI".into(),
                surface: DlpSurface::DriveUpload,
                detector_ref: "regional-pack:pci".into(),
                matched_data_class: privacy_class(DataClass::Pci),
                action: DlpAction::Quarantine,
                severity: DlpSeverity::Critical,
                priority: 1,
            }),
            Err(DlpError::HighRiskRuleMustHold)
        );
    }

    #[test]
    fn regulated_financial_rules_require_admin_review_hold() {
        assert_eq!(
            DlpRule::new(DlpRuleCreate {
                rule_id: "financial".into(),
                name: "Financial".into(),
                surface: DlpSurface::DriveUpload,
                detector_ref: "regional-pack:financial".into(),
                matched_data_class: privacy_class(DataClass::Financial),
                action: DlpAction::Quarantine,
                severity: DlpSeverity::Critical,
                priority: 2,
            }),
            Err(DlpError::HighRiskRuleMustHold)
        );
    }

    #[test]
    fn policy_rejects_duplicate_rule_identity_and_priority() {
        assert_eq!(policy().schema_version.value, 1);

        assert_eq!(
            DlpPolicy::new(DlpPolicyCreate {
                policy_id: "dlp-policy-2".into(),
                tenant_id: "tenant-1".into(),
                region: "region-alpha1".into(),
                admin_review_queue_id: "admin-review-queue".into(),
                rules: vec![
                    rule("pii-a", DataClass::PiiIdentifying, DlpAction::Redact, 20),
                    rule(
                        "pii-b",
                        DataClass::PiiIdentifying,
                        DlpAction::Quarantine,
                        20
                    ),
                ],
                created_at_epoch_seconds: 1,
                updated_at_epoch_seconds: 2,
            }),
            Err(DlpError::DuplicateRulePriority)
        );
    }

    #[test]
    fn finding_and_verdict_must_match_policy_rule_and_hold_queue() {
        let policy = policy();
        let request = request();
        let finding = DlpFinding::new(
            "finding-1".into(),
            &policy.rules.value[0],
            "sha256:abc123".into(),
        )
        .unwrap();
        let verdict = DlpScanVerdict::new(
            DlpScanVerdictCreate {
                scan_id: "scan-1".into(),
                tenant_id: "tenant-1".into(),
                policy_id: "dlp-policy-1".into(),
                findings: vec![finding],
                final_action: DlpAction::AdminReviewHold,
                admin_review_queue_id: Some("admin-review-queue".into()),
                hold_until_epoch_seconds: Some(1_700_086_400),
                decided_at_epoch_seconds: 1_700_000_030,
            },
            &request,
            &policy,
        )
        .unwrap();
        assert_eq!(verdict.schema_version.value, 1);

        assert_eq!(
            DlpScanVerdict::new(
                DlpScanVerdictCreate {
                    scan_id: "scan-1".into(),
                    tenant_id: "tenant-1".into(),
                    policy_id: "dlp-policy-1".into(),
                    findings: verdict.findings.value.clone(),
                    final_action: DlpAction::AdminReviewHold,
                    admin_review_queue_id: None,
                    hold_until_epoch_seconds: Some(1_700_086_400),
                    decided_at_epoch_seconds: 1_700_000_030,
                },
                &request,
                &policy,
            ),
            Err(DlpError::MissingHoldQueue)
        );
    }

    #[test]
    fn scan_request_classifies_actor_and_requires_policy_tenant_region() {
        let request = request();
        assert_eq!(
            request.actor_ref.data_class,
            DataClassification::Privacy(dlp_actor_data_class())
        );

        assert_eq!(
            DlpScanRequest::new(
                DlpScanRequestCreate {
                    scan_id: "scan-2".into(),
                    tenant_id: "tenant-2".into(),
                    region: "region-alpha1".into(),
                    surface: DlpSurface::MailOutbound,
                    actor_ref: "user:sender@example.com".into(),
                    content_ref: "mail:message-1".into(),
                    declared_data_class: default_workspace_dlp_data_class(),
                    requested_at_epoch_seconds: 1,
                },
                &policy(),
            ),
            Err(DlpError::InvalidTenantId)
        );
    }

    #[test]
    fn legacy_data_class_conversion_rejects_operational_markers() {
        assert_eq!(
            workspace_dlp_data_class_from_legacy(DataClass::Audit),
            Err(DlpError::InvalidDataClass)
        );
        assert_eq!(
            DataClassification::from(OperationalDataClass::Audit).privacy_data_class(),
            None
        );
    }
}
