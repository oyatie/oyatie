use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
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
    pub fn new(
        domain_ref: String,
        spf: bool,
        dkim: bool,
        policy: DmarcPolicy,
        evidence_ref: String,
    ) -> Result<Self, MailGovernanceError> {
        ne(&domain_ref)?;
        ne(&evidence_ref)?;
        let action = match (spf || dkim, policy) {
            (true, _) | (false, DmarcPolicy::None) => DmarcAction::Accept,
            (false, DmarcPolicy::Quarantine) => DmarcAction::Quarantine,
            (false, DmarcPolicy::Reject) => DmarcAction::Reject,
        };
        Ok(Self {
            domain_ref: int(domain_ref),
            action: int(action),
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
}
