use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailGovernanceError {
    Invalid,
    ContextPillarMismatch,
    PersonalRequiresClientOnly,
    WorkRequiresTenantDek,
    WorkflowPolicyRequired,
    AiPlaintextForbidden,
    AiTrainingForbidden,
    TrackerNeedsSanitizedBody,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailContextKind {
    Personal,
    Work,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnershipPillar {
    Personal,
    Work,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailEnvelope {
    PersonalClientOnly {
        envelope_ref: String,
    },
    TenantDek {
        dek_ref: String,
    },
    Imported {
        source_hash: String,
        evidence_ref: String,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageGovernanceCreate {
    pub message_id: String,
    pub mailbox_id: String,
    pub scope_ref: String,
    pub context: MailContextKind,
    pub pillar: OwnershipPillar,
    pub subject_ref: String,
    pub envelope: MailEnvelope,
    pub retention_policy_id: String,
    pub legal_hold_ids: Vec<String>,
    pub confidential_expires_at: Option<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailMessageGovernance {
    pub message_id: Classified<String>,
    pub mailbox_id: Classified<String>,
    pub scope_ref: Classified<String>,
    pub context: Classified<MailContextKind>,
    pub pillar: Classified<OwnershipPillar>,
    pub subject_ref: Classified<String>,
    pub envelope: Classified<MailEnvelope>,
    pub retention_policy_id: Classified<String>,
    pub legal_hold_ids: Classified<Vec<String>>,
    pub confidential_expires_at: Classified<Option<u64>>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeleteOutcome {
    Allowed,
    BlockedByLegalHold,
    ExpiredButHeld,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmarcPolicy {
    None,
    Quarantine,
    Reject,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmarcAction {
    Accept,
    Quarantine,
    Reject,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DmarcVerdict {
    pub domain_ref: Classified<String>,
    pub action: Classified<DmarcAction>,
    pub report_only: Classified<bool>,
    pub evidence_ref: Classified<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailWorkflowHandoff {
    pub message_id: Classified<String>,
    pub lawful_basis_ref: Classified<String>,
    pub policy_snapshot_ref: Classified<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MailAiAssistRequest {
    pub message_id: Classified<String>,
    pub encrypted_content_ref: Classified<String>,
    pub plaintext_in_prompt: Classified<bool>,
    pub training_enabled: Classified<bool>,
}
impl MailMessageGovernance {
    pub fn new(i: MailMessageGovernanceCreate) -> Result<Self, MailGovernanceError> {
        for s in [
            &i.message_id,
            &i.mailbox_id,
            &i.scope_ref,
            &i.subject_ref,
            &i.retention_policy_id,
        ] {
            ne(s)?
        }
        match (i.context, i.pillar) {
            (MailContextKind::Personal, OwnershipPillar::Personal)
            | (MailContextKind::Work, OwnershipPillar::Work) => (),
            _ => return Err(MailGovernanceError::ContextPillarMismatch),
        };
        match (&i.context, &i.envelope) {
            (MailContextKind::Personal, MailEnvelope::PersonalClientOnly { envelope_ref }) => {
                ne(envelope_ref)?
            }
            (MailContextKind::Personal, _) => {
                return Err(MailGovernanceError::PersonalRequiresClientOnly);
            }
            (MailContextKind::Work, MailEnvelope::TenantDek { dek_ref }) => ne(dek_ref)?,
            (
                MailContextKind::Work,
                MailEnvelope::Imported {
                    source_hash,
                    evidence_ref,
                },
            ) => {
                ne(source_hash)?;
                ne(evidence_ref)?
            }
            (MailContextKind::Work, MailEnvelope::PersonalClientOnly { .. }) => {
                return Err(MailGovernanceError::WorkRequiresTenantDek);
            }
        };
        Ok(Self {
            message_id: int(i.message_id),
            mailbox_id: int(i.mailbox_id),
            scope_ref: int(i.scope_ref),
            context: int(i.context),
            pillar: int(i.pillar),
            subject_ref: Classified::new(i.subject_ref, PrivacyDataClass::pii_identifying()),
            envelope: int(i.envelope),
            retention_policy_id: int(i.retention_policy_id),
            legal_hold_ids: int(i.legal_hold_ids),
            confidential_expires_at: int(i.confidential_expires_at),
        })
    }
    pub fn org_admin_export_allowed(&self) -> bool {
        self.context.value == MailContextKind::Work
            && matches!(self.envelope.value, MailEnvelope::TenantDek { .. })
    }
    pub fn deletion_outcome(&self, now: u64) -> DeleteOutcome {
        if self.legal_hold_ids.value.is_empty() {
            DeleteOutcome::Allowed
        } else if self.confidential_expires_at.value.is_some_and(|t| t <= now) {
            DeleteOutcome::ExpiredButHeld
        } else {
            DeleteOutcome::BlockedByLegalHold
        }
    }
}
impl DmarcVerdict {
    /// RFC 7489 §3.1-compliant: derives action from identifier-aligned pass results.
    /// `spf_aligned` — SPF passed AND the authenticated domain is aligned with From:.
    /// `dkim_aligned` — DKIM passed AND the d= tag is aligned with From:.
    pub fn new_aligned(
        domain_ref: String,
        spf_aligned: bool,
        dkim_aligned: bool,
        policy: DmarcPolicy,
        evidence_ref: String,
    ) -> Result<Self, MailGovernanceError> {
        ne(&domain_ref)?;
        ne(&evidence_ref)?;
        let aligned = spf_aligned || dkim_aligned;
        let action = match (aligned, policy) {
            (true, _) => DmarcAction::Accept,
            (false, DmarcPolicy::None) => DmarcAction::Accept,
            (false, DmarcPolicy::Quarantine) => DmarcAction::Quarantine,
            (false, DmarcPolicy::Reject) => DmarcAction::Reject,
        };
        // p=none non-aligned: Accept but flag for aggregate reporting (RFC 7489 §6.3).
        let report_only = policy == DmarcPolicy::None && !aligned;
        Ok(Self {
            domain_ref: int(domain_ref),
            action: int(action),
            report_only: int(report_only),
            evidence_ref: Classified::new(evidence_ref, DataClass::Audit),
        })
    }

    /// Back-compat wrapper: treats raw SPF/DKIM pass as identifier-aligned.
    /// Pre-RFC-7489 callers that supply raw pass booleans observe no behavioral change
    /// for messages where raw pass == aligned pass.
    pub fn new(
        domain_ref: String,
        spf: bool,
        dkim: bool,
        policy: DmarcPolicy,
        evidence_ref: String,
    ) -> Result<Self, MailGovernanceError> {
        Self::new_aligned(domain_ref, spf, dkim, policy, evidence_ref)
    }
}
/// RFC 7489 §3.1 alignment mode for SPF and DKIM identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlignmentMode {
    /// Organizational Domain match (subdomain of the same registered domain is
    /// sufficient).
    Relaxed,
    /// Exact domain match required.
    Strict,
}

/// Published DMARC record fields relevant to verdict derivation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DmarcRecord {
    /// p= tag: policy for the Organizational Domain.
    pub policy: DmarcPolicy,
    /// sp= tag: optional override policy for subdomains.
    pub subdomain_policy: Option<DmarcPolicy>,
    /// aspf= tag: SPF alignment mode (default Relaxed per RFC 7489).
    pub spf_alignment: AlignmentMode,
    /// adkim= tag: DKIM alignment mode (default Relaxed per RFC 7489).
    pub dkim_alignment: AlignmentMode,
}

/// SPF alignment inputs: the domain authenticated by SPF and the RFC5322.From
/// domain for the message being evaluated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpfAlignmentInput {
    /// Domain authenticated by the SPF mechanism (MAIL FROM / HELO domain).
    pub authenticated_domain: String,
    /// RFC5322.From domain extracted from the message header.
    pub from_domain: String,
}

/// DKIM alignment inputs: the d= tag from a passing DKIM signature and the
/// RFC5322.From domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DkimAlignmentInput {
    /// d= tag from the DKIM-Signature header.
    pub d_tag: String,
    /// RFC5322.From domain extracted from the message header.
    pub from_domain: String,
}

/// Extract the Organizational Domain (registrable domain) from a fully
/// qualified domain name.  This is a simplified implementation that strips
/// all labels except the last two (SLD + TLD), matching RFC 7489 relaxed
/// alignment semantics for single-segment TLDs.
///
/// For example:
///   `mail.example.com` → `example.com`
///   `example.com`      → `example.com`
///   `a.b.example.com`  → `example.com`
pub fn organizational_domain(domain: &str) -> &str {
    let dot_count = domain.bytes().filter(|&b| b == b'.').count();
    if dot_count <= 1 {
        // Already at most SLD.TLD
        domain
    } else {
        // Strip all labels except the last two: find the dot that is
        // (dot_count - 1) positions from the start (0-indexed), i.e. the
        // (dot_count - 1)th dot counting from 1 = the one right before the
        // second-to-last label.
        //
        // For "a.b.example.com" dot_count=3, we want the dot at index 1
        // (0-indexed among dots): positions are [1, 3, 11]; index 1 → pos 3,
        // giving "example.com". Correct.
        let dot_pos = domain
            .char_indices()
            .filter(|&(_, c)| c == '.')
            .nth(dot_count - 2) // 0-indexed: skip the first (dot_count-2) dots
            .map(|(i, _)| i)
            .unwrap();
        &domain[dot_pos + 1..]
    }
}

/// Returns `true` when `candidate` is aligned with `from` under `mode`.
fn domains_aligned(candidate: &str, from: &str, mode: AlignmentMode) -> bool {
    match mode {
        AlignmentMode::Strict => candidate.eq_ignore_ascii_case(from),
        AlignmentMode::Relaxed => {
            organizational_domain(candidate).eq_ignore_ascii_case(organizational_domain(from))
        }
    }
}

impl DmarcVerdict {
    /// Full RFC 7489 §3.1 verdict: computes alignment for SPF and DKIM using
    /// the published `DmarcRecord` alignment modes, applies subdomain policy
    /// (sp=) when the From domain is a proper subdomain of the policy domain,
    /// and sets `report_only` for p=none / sp=none non-aligned messages.
    pub fn new_from_record(
        domain_ref: String,
        spf: SpfAlignmentInput,
        dkim: Option<DkimAlignmentInput>,
        record: &DmarcRecord,
        evidence_ref: String,
    ) -> Result<Self, MailGovernanceError> {
        ne(&domain_ref)?;
        ne(&evidence_ref)?;

        let spf_aligned = domains_aligned(
            &spf.authenticated_domain,
            &spf.from_domain,
            record.spf_alignment,
        );
        let dkim_aligned = dkim
            .as_ref()
            .map(|d| domains_aligned(&d.d_tag, &d.from_domain, record.dkim_alignment))
            .unwrap_or(false);
        let aligned = spf_aligned || dkim_aligned;

        // Determine which policy applies per RFC 7489 §6.3.
        // The DMARC record is published at the Organizational Domain level.
        // sp= applies when the RFC5322.From domain is NOT the Organizational
        // Domain itself — i.e. it is a proper subdomain of the org domain.
        // We compute the org domain of domain_ref as the anchor.
        let policy_org = organizational_domain(&domain_ref);
        let from_is_subdomain = {
            let from = spf.from_domain.to_ascii_lowercase();
            let org = policy_org.to_ascii_lowercase();
            // from must be a proper subdomain: ends with ".<org>" and != org
            from != org && from.ends_with(&format!(".{}", org))
        };
        let effective_policy = if from_is_subdomain {
            record.subdomain_policy.unwrap_or(record.policy)
        } else {
            record.policy
        };

        let action = match (aligned, effective_policy) {
            (true, _) => DmarcAction::Accept,
            (false, DmarcPolicy::None) => DmarcAction::Accept,
            (false, DmarcPolicy::Quarantine) => DmarcAction::Quarantine,
            (false, DmarcPolicy::Reject) => DmarcAction::Reject,
        };
        let report_only = effective_policy == DmarcPolicy::None && !aligned;

        Ok(Self {
            domain_ref: int(domain_ref),
            action: int(action),
            report_only: int(report_only),
            evidence_ref: Classified::new(evidence_ref, DataClass::Audit),
        })
    }
}

impl MailWorkflowHandoff {
    pub fn new(
        m: &MailMessageGovernance,
        lawful_basis_ref: String,
        policy_snapshot_ref: String,
    ) -> Result<Self, MailGovernanceError> {
        if lawful_basis_ref.trim().is_empty() || policy_snapshot_ref.trim().is_empty() {
            return Err(MailGovernanceError::WorkflowPolicyRequired);
        };
        Ok(Self {
            message_id: int(m.message_id.value.clone()),
            lawful_basis_ref: int(lawful_basis_ref),
            policy_snapshot_ref: int(policy_snapshot_ref),
        })
    }
}
impl MailAiAssistRequest {
    pub fn new(
        m: &MailMessageGovernance,
        encrypted_content_ref: String,
        plaintext: bool,
        training: bool,
    ) -> Result<Self, MailGovernanceError> {
        ne(&encrypted_content_ref)?;
        if plaintext {
            return Err(MailGovernanceError::AiPlaintextForbidden);
        };
        if training {
            return Err(MailGovernanceError::AiTrainingForbidden);
        };
        Ok(Self {
            message_id: int(m.message_id.value.clone()),
            encrypted_content_ref: int(encrypted_content_ref),
            plaintext_in_prompt: int(plaintext),
            training_enabled: int(training),
        })
    }
}
pub fn tracker_pixel_block(
    message_id: String,
    blocked: u32,
    sanitized_body_ref: String,
) -> Result<Classified<String>, MailGovernanceError> {
    ne(&message_id)?;
    if blocked > 0 && sanitized_body_ref.trim().is_empty() {
        return Err(MailGovernanceError::TrackerNeedsSanitizedBody);
    };
    Ok(int(sanitized_body_ref))
}
fn ne(s: &str) -> Result<(), MailGovernanceError> {
    if s.trim().is_empty() {
        Err(MailGovernanceError::Invalid)
    } else {
        Ok(())
    }
}
fn int<T>(v: T) -> Classified<T> {
    Classified::new(v, DataClass::InternalOnly)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn work() -> MailMessageGovernance {
        MailMessageGovernance::new(MailMessageGovernanceCreate {
            message_id: "mw".into(),
            mailbox_id: "mb".into(),
            scope_ref: "tenant:t".into(),
            context: MailContextKind::Work,
            pillar: OwnershipPillar::Work,
            subject_ref: "user:w".into(),
            envelope: MailEnvelope::TenantDek {
                dek_ref: "dek".into(),
            },
            retention_policy_id: "retain".into(),
            legal_hold_ids: vec!["hold".into()],
            confidential_expires_at: Some(10),
        })
        .unwrap()
    }
    fn personal() -> MailMessageGovernance {
        MailMessageGovernance::new(MailMessageGovernanceCreate {
            message_id: "mp".into(),
            mailbox_id: "mbp".into(),
            scope_ref: "person:p".into(),
            context: MailContextKind::Personal,
            pillar: OwnershipPillar::Personal,
            subject_ref: "user:p".into(),
            envelope: MailEnvelope::PersonalClientOnly {
                envelope_ref: "e2e".into(),
            },
            retention_policy_id: "r".into(),
            legal_hold_ids: vec![],
            confidential_expires_at: None,
        })
        .unwrap()
    }
    #[test]
    fn work_mail_tenant_dek_and_retention() {
        assert!(work().org_admin_export_allowed())
    }
    #[test]
    fn org_admin_cannot_export_personal_mail() {
        assert!(!personal().org_admin_export_allowed())
    }
    #[test]
    fn hold_blocks_mail_deletion() {
        assert_eq!(
            work().deletion_outcome(5),
            DeleteOutcome::BlockedByLegalHold
        );
        assert_eq!(work().deletion_outcome(11), DeleteOutcome::ExpiredButHeld)
    }
    #[test]
    fn dmarc_fail_quarantine_and_logged() {
        assert_eq!(
            DmarcVerdict::new(
                "example.com".into(),
                false,
                false,
                DmarcPolicy::Quarantine,
                "audit".into()
            )
            .unwrap()
            .action
            .value,
            DmarcAction::Quarantine
        )
    }
    #[test]
    fn mail_to_workflow_requires_policy_basis() {
        assert_eq!(
            MailWorkflowHandoff::new(&work(), "".into(), "p".into()),
            Err(MailGovernanceError::WorkflowPolicyRequired)
        )
    }
    #[test]
    fn ai_compose_no_plaintext_exfil() {
        assert_eq!(
            MailAiAssistRequest::new(&work(), "enc".into(), true, false),
            Err(MailGovernanceError::AiPlaintextForbidden)
        )
    }
    #[test]
    fn tracker_pixel_injection_blocked() {
        assert!(tracker_pixel_block("m".into(), 1, "body:safe".into()).is_ok())
    }

    // ST1: alignment-aware verdict path

    #[test]
    fn dmarc_aligned_false_reject_policy_rejects() {
        // spf=true, dkim=true but neither is aligned — RFC 7489 requires Reject.
        let v = DmarcVerdict::new_aligned(
            "example.com".into(),
            false,
            false,
            DmarcPolicy::Reject,
            "audit".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Reject);
    }

    #[test]
    fn dmarc_aligned_false_quarantine_policy_quarantines() {
        let v = DmarcVerdict::new_aligned(
            "example.com".into(),
            false,
            false,
            DmarcPolicy::Quarantine,
            "audit".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Quarantine);
    }

    #[test]
    fn dmarc_aligned_spf_only_accepts() {
        let v = DmarcVerdict::new_aligned(
            "example.com".into(),
            true,
            false,
            DmarcPolicy::Reject,
            "audit".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Accept);
    }

    // ST2: report_only accounting for p=none

    #[test]
    fn dmarc_none_policy_non_aligned_is_report_only() {
        let v = DmarcVerdict::new_aligned(
            "example.com".into(),
            false,
            false,
            DmarcPolicy::None,
            "audit".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Accept);
        assert!(v.report_only.value);
    }

    #[test]
    fn dmarc_aligned_pass_not_report_only() {
        let v = DmarcVerdict::new_aligned(
            "example.com".into(),
            true,
            false,
            DmarcPolicy::None,
            "audit".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Accept);
        assert!(!v.report_only.value);
    }

    #[test]
    fn dmarc_reject_not_report_only() {
        let v = DmarcVerdict::new_aligned(
            "example.com".into(),
            false,
            false,
            DmarcPolicy::Reject,
            "audit".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Reject);
        assert!(!v.report_only.value);
    }

    #[test]
    fn dmarc_evidence_ref_is_audit_class() {
        use data_boundary_kernel::DataClassification;
        let v = DmarcVerdict::new_aligned(
            "example.com".into(),
            false,
            false,
            DmarcPolicy::None,
            "audit-ref-001".into(),
        )
        .unwrap();
        assert_eq!(
            v.evidence_ref.data_class,
            DataClassification::Operational(data_boundary_kernel::OperationalDataClass::Audit)
        );
    }

    // -----------------------------------------------------------------------
    // RFC 7489 §3.1 alignment-mode tests (RED — AlignmentMode + DmarcRecord
    // + organizational_domain() not yet implemented).
    // -----------------------------------------------------------------------

    /// RFC 7489 §3.1 relaxed SPF alignment: the Organizational Domain of the
    /// SPF-authenticated domain (mail.example.com) matches the Organizational
    /// Domain of the RFC5322.From domain (example.com) — should be ALIGNED.
    #[test]
    fn relaxed_spf_alignment_accepts_subdomain_of_same_org_domain() {
        let record = DmarcRecord {
            policy: DmarcPolicy::Reject,
            subdomain_policy: None,
            spf_alignment: AlignmentMode::Relaxed,
            dkim_alignment: AlignmentMode::Relaxed,
        };
        let v = DmarcVerdict::new_from_record(
            "example.com".into(),
            // SPF authenticated domain is a subdomain; relaxed → org-domain match
            SpfAlignmentInput {
                authenticated_domain: "mail.example.com".into(),
                from_domain: "example.com".into(),
            },
            // DKIM not present
            None,
            &record,
            "audit-relax-spf".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Accept);
        assert!(!v.report_only.value);
    }

    /// RFC 7489 §3.1 strict SPF alignment: the SPF-authenticated domain must
    /// exactly equal the RFC5322.From domain.  mail.example.com ≠ example.com
    /// under strict mode → no aligned pass → Reject.
    #[test]
    fn strict_spf_alignment_rejects_subdomain_mismatch() {
        let record = DmarcRecord {
            policy: DmarcPolicy::Reject,
            subdomain_policy: None,
            spf_alignment: AlignmentMode::Strict,
            dkim_alignment: AlignmentMode::Strict,
        };
        let v = DmarcVerdict::new_from_record(
            "example.com".into(),
            SpfAlignmentInput {
                authenticated_domain: "mail.example.com".into(),
                from_domain: "example.com".into(),
            },
            None,
            &record,
            "audit-strict-spf".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Reject);
    }

    /// RFC 7489 §3.1 relaxed DKIM alignment: d= tag is a subdomain sharing the
    /// same Organizational Domain as From — should be ALIGNED (Accept).
    #[test]
    fn relaxed_dkim_alignment_accepts_subdomain_d_tag() {
        let record = DmarcRecord {
            policy: DmarcPolicy::Reject,
            subdomain_policy: None,
            spf_alignment: AlignmentMode::Relaxed,
            dkim_alignment: AlignmentMode::Relaxed,
        };
        let v = DmarcVerdict::new_from_record(
            "example.com".into(),
            // SPF fails alignment
            SpfAlignmentInput {
                authenticated_domain: "unrelated.net".into(),
                from_domain: "example.com".into(),
            },
            // DKIM d= is subdomain of same org domain
            Some(DkimAlignmentInput {
                d_tag: "smtp.example.com".into(),
                from_domain: "example.com".into(),
            }),
            &record,
            "audit-relax-dkim".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Accept);
    }

    /// RFC 7489 §3.1 strict DKIM alignment: d= must exactly equal From domain.
    /// smtp.example.com ≠ example.com under strict → no aligned pass.
    #[test]
    fn strict_dkim_alignment_rejects_subdomain_d_tag() {
        let record = DmarcRecord {
            policy: DmarcPolicy::Reject,
            subdomain_policy: None,
            spf_alignment: AlignmentMode::Strict,
            dkim_alignment: AlignmentMode::Strict,
        };
        let v = DmarcVerdict::new_from_record(
            "example.com".into(),
            SpfAlignmentInput {
                authenticated_domain: "unrelated.net".into(),
                from_domain: "example.com".into(),
            },
            Some(DkimAlignmentInput {
                d_tag: "smtp.example.com".into(),
                from_domain: "example.com".into(),
            }),
            &record,
            "audit-strict-dkim".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Reject);
    }

    /// RFC 7489 §6.3 subdomain policy (sp=): when the RFC5322.From is a
    /// subdomain of the policy domain and an explicit sp= is present, sp=
    /// overrides p= for that message.  p=none + sp=reject → Reject for subdomain.
    #[test]
    fn subdomain_policy_overrides_parent_policy_for_subdomain_from() {
        let record = DmarcRecord {
            policy: DmarcPolicy::None,
            subdomain_policy: Some(DmarcPolicy::Reject),
            spf_alignment: AlignmentMode::Relaxed,
            dkim_alignment: AlignmentMode::Relaxed,
        };
        // From domain is a subdomain of the policy domain; no aligned pass.
        let v = DmarcVerdict::new_from_record(
            "sub.example.com".into(),
            SpfAlignmentInput {
                authenticated_domain: "unrelated.net".into(),
                from_domain: "sub.example.com".into(),
            },
            None,
            &record,
            "audit-sp".into(),
        )
        .unwrap();
        // sp=reject must apply, not p=none
        assert_eq!(v.action.value, DmarcAction::Reject);
        // report_only must be false because this is an enforcing sp=reject
        assert!(!v.report_only.value);
    }

    /// RFC 7489 §6.3: when sp= is absent and the message is from a subdomain,
    /// the parent p= applies.  p=none + no sp= → Accept with report_only for
    /// a non-aligned subdomain message.
    #[test]
    fn absent_subdomain_policy_falls_back_to_parent_policy() {
        let record = DmarcRecord {
            policy: DmarcPolicy::None,
            subdomain_policy: None,
            spf_alignment: AlignmentMode::Relaxed,
            dkim_alignment: AlignmentMode::Relaxed,
        };
        let v = DmarcVerdict::new_from_record(
            "sub.example.com".into(),
            SpfAlignmentInput {
                authenticated_domain: "unrelated.net".into(),
                from_domain: "sub.example.com".into(),
            },
            None,
            &record,
            "audit-sp-absent".into(),
        )
        .unwrap();
        assert_eq!(v.action.value, DmarcAction::Accept);
        assert!(v.report_only.value);
    }

    /// organizational_domain() must extract the registrable domain component,
    /// stripping sub-labels per the Public Suffix List semantics used in
    /// RFC 7489 relaxed alignment.
    #[test]
    fn organizational_domain_strips_subdomains() {
        assert_eq!(organizational_domain("mail.example.com"), "example.com");
        assert_eq!(organizational_domain("example.com"), "example.com");
        assert_eq!(organizational_domain("a.b.example.com"), "example.com");
    }
}
