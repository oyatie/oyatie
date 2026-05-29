#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use oya_mail_domain::{
    DmarcAction, DmarcPolicy, DmarcVerdict, MailAiAssistRequest, MailContextKind, MailEnvelope,
    MailGovernanceError, MailMessageGovernance, MailMessageGovernanceCreate, MailWorkflowHandoff,
    OwnershipPillar as MailOwnershipPillar,
};
use oya_mail_mailbox_store_api::{
    AiAssistRequest, AuthorizedMailContext, DmarcApiAction, DmarcApiPolicy, DmarcCheckRequest,
    HandoffReceipt, MailApiContext, MailApiEnvelope, MailApiError, SubmissionReceipt,
    SubmitMessageRequest, WorkflowHandoffRequest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MailUsecaseError {
    Api(MailApiError),
    Domain(MailGovernanceError),
    PersonalHandoffForbidden,
    PrincipalMismatch,
    DmarcRejected,
}

pub fn submit_message(
    ctx: &AuthorizedMailContext,
    req: SubmitMessageRequest,
) -> Result<SubmissionReceipt, MailUsecaseError> {
    ctx.validate().map_err(MailUsecaseError::Api)?;
    if req.subject_ref != ctx.principal_ref {
        return Err(MailUsecaseError::PrincipalMismatch);
    }
    let governance = MailMessageGovernance::new(MailMessageGovernanceCreate {
        message_id: req.message_id,
        mailbox_id: req.mailbox_id,
        scope_ref: ctx.scope_ref.clone(),
        context: map_context(ctx.context),
        pillar: map_pillar(ctx.context),
        subject_ref: req.subject_ref,
        envelope: map_envelope(req.envelope),
        retention_policy_id: req.retention_policy_id,
        legal_hold_ids: vec![],
        confidential_expires_at: None,
    })
    .map_err(MailUsecaseError::Domain)?;

    let dmarc_action = match req.dmarc_check {
        Some(check) => decide_dmarc(check)?,
        None => DmarcAction::Accept,
    };
    if dmarc_action == DmarcAction::Reject {
        return Err(MailUsecaseError::DmarcRejected);
    }

    Ok(SubmissionReceipt {
        message_id: governance.message_id.value,
        event_type: "mail.message.submitted",
        audit_correlation_id: ctx.audit_correlation_id.clone(),
        idempotency_key: ctx.idempotency_key.clone(),
        policy_decision_ref: ctx.policy_decision_ref.clone(),
        dmarc_action: map_dmarc_action(dmarc_action),
    })
}

pub fn create_workflow_handoff(
    ctx: &AuthorizedMailContext,
    req: WorkflowHandoffRequest,
) -> Result<(MailWorkflowHandoff, HandoffReceipt), MailUsecaseError> {
    ctx.validate().map_err(MailUsecaseError::Api)?;
    if ctx.context == MailApiContext::Personal {
        return Err(MailUsecaseError::PersonalHandoffForbidden);
    }
    if req.subject_ref != ctx.principal_ref {
        return Err(MailUsecaseError::PrincipalMismatch);
    }
    let governance = mail_governance_from_handoff(ctx, &req)?;
    let handoff =
        MailWorkflowHandoff::new(&governance, req.lawful_basis_ref, req.policy_snapshot_ref)
            .map_err(MailUsecaseError::Domain)?;
    let receipt = HandoffReceipt {
        message_id: handoff.message_id.value.clone(),
        event_type: "mail.workflow_handoff.created",
        audit_correlation_id: ctx.audit_correlation_id.clone(),
        idempotency_key: ctx.idempotency_key.clone(),
        policy_decision_ref: ctx.policy_decision_ref.clone(),
    };
    Ok((handoff, receipt))
}

pub fn prepare_ai_assist(
    ctx: &AuthorizedMailContext,
    req: AiAssistRequest,
) -> Result<MailAiAssistRequest, MailUsecaseError> {
    ctx.validate().map_err(MailUsecaseError::Api)?;
    if req.subject_ref != ctx.principal_ref {
        return Err(MailUsecaseError::PrincipalMismatch);
    }
    let governance = MailMessageGovernance::new(MailMessageGovernanceCreate {
        message_id: req.message_id,
        mailbox_id: req.mailbox_id,
        scope_ref: ctx.scope_ref.clone(),
        context: map_context(ctx.context),
        pillar: map_pillar(ctx.context),
        subject_ref: req.subject_ref,
        envelope: map_envelope(req.envelope),
        retention_policy_id: req.retention_policy_id,
        legal_hold_ids: vec![],
        confidential_expires_at: None,
    })
    .map_err(MailUsecaseError::Domain)?;
    MailAiAssistRequest::new(
        &governance,
        req.encrypted_content_ref,
        req.plaintext_in_prompt,
        req.training_enabled,
    )
    .map_err(MailUsecaseError::Domain)
}

pub fn decide_dmarc(req: DmarcCheckRequest) -> Result<DmarcAction, MailUsecaseError> {
    let verdict = DmarcVerdict::new(
        req.domain_ref,
        req.spf_aligned,
        req.dkim_aligned,
        map_dmarc_policy(req.policy),
        req.evidence_ref,
    )
    .map_err(MailUsecaseError::Domain)?;
    Ok(verdict.action.value)
}

fn mail_governance_from_handoff(
    ctx: &AuthorizedMailContext,
    req: &WorkflowHandoffRequest,
) -> Result<MailMessageGovernance, MailUsecaseError> {
    MailMessageGovernance::new(MailMessageGovernanceCreate {
        message_id: req.message_id.clone(),
        mailbox_id: req.mailbox_id.clone(),
        scope_ref: ctx.scope_ref.clone(),
        context: map_context(ctx.context),
        pillar: map_pillar(ctx.context),
        subject_ref: req.subject_ref.clone(),
        envelope: map_envelope(req.envelope.clone()),
        retention_policy_id: req.retention_policy_id.clone(),
        legal_hold_ids: req.legal_hold_ids.clone(),
        confidential_expires_at: None,
    })
    .map_err(MailUsecaseError::Domain)
}

fn map_context(context: MailApiContext) -> MailContextKind {
    match context {
        MailApiContext::Personal => MailContextKind::Personal,
        MailApiContext::Work => MailContextKind::Work,
    }
}

fn map_pillar(context: MailApiContext) -> MailOwnershipPillar {
    match context {
        MailApiContext::Personal => MailOwnershipPillar::Personal,
        MailApiContext::Work => MailOwnershipPillar::Work,
    }
}

fn map_envelope(envelope: MailApiEnvelope) -> MailEnvelope {
    match envelope {
        MailApiEnvelope::PersonalClientOnly { envelope_ref } => {
            MailEnvelope::PersonalClientOnly { envelope_ref }
        }
        MailApiEnvelope::TenantDek { dek_ref } => MailEnvelope::TenantDek { dek_ref },
        MailApiEnvelope::Imported {
            source_hash,
            evidence_ref,
        } => MailEnvelope::Imported {
            source_hash,
            evidence_ref,
        },
    }
}

fn map_dmarc_policy(policy: DmarcApiPolicy) -> DmarcPolicy {
    match policy {
        DmarcApiPolicy::None => DmarcPolicy::None,
        DmarcApiPolicy::Quarantine => DmarcPolicy::Quarantine,
        DmarcApiPolicy::Reject => DmarcPolicy::Reject,
    }
}

fn map_dmarc_action(action: DmarcAction) -> DmarcApiAction {
    match action {
        DmarcAction::Accept => DmarcApiAction::Accept,
        DmarcAction::Quarantine => DmarcApiAction::Quarantine,
        DmarcAction::Reject => DmarcApiAction::Reject,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work_ctx() -> AuthorizedMailContext {
        AuthorizedMailContext {
            context: MailApiContext::Work,
            scope_ref: "tenant:t".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow:mail-submit".into(),
            audit_correlation_id: "audit".into(),
        }
    }

    fn handoff_req() -> WorkflowHandoffRequest {
        WorkflowHandoffRequest {
            message_id: "m".into(),
            mailbox_id: "mb".into(),
            subject_ref: "user:u".into(),
            envelope: MailApiEnvelope::TenantDek {
                dek_ref: "dek".into(),
            },
            retention_policy_id: "retain".into(),
            legal_hold_ids: vec![],
            lawful_basis_ref: "basis".into(),
            policy_snapshot_ref: "policy".into(),
        }
    }

    #[test]
    fn submit_message_rejects_hard_dmarc_failure() {
        let req = SubmitMessageRequest {
            message_id: "m".into(),
            mailbox_id: "mb".into(),
            subject_ref: "user:u".into(),
            envelope: MailApiEnvelope::TenantDek {
                dek_ref: "dek".into(),
            },
            retention_policy_id: "retain".into(),
            dmarc_check: Some(DmarcCheckRequest {
                domain_ref: "example.com".into(),
                spf_aligned: false,
                dkim_aligned: false,
                policy: DmarcApiPolicy::Reject,
                evidence_ref: "authres".into(),
            }),
        };
        assert_eq!(
            submit_message(&work_ctx(), req),
            Err(MailUsecaseError::DmarcRejected)
        );
    }

    #[test]
    fn submit_message_accepts_quarantine_with_receipt() {
        let req = SubmitMessageRequest {
            message_id: "m".into(),
            mailbox_id: "mb".into(),
            subject_ref: "user:u".into(),
            envelope: MailApiEnvelope::TenantDek {
                dek_ref: "dek".into(),
            },
            retention_policy_id: "retain".into(),
            dmarc_check: Some(DmarcCheckRequest {
                domain_ref: "example.com".into(),
                spf_aligned: false,
                dkim_aligned: false,
                policy: DmarcApiPolicy::Quarantine,
                evidence_ref: "authres".into(),
            }),
        };
        let receipt = submit_message(&work_ctx(), req).unwrap();
        assert_eq!(receipt.dmarc_action, DmarcApiAction::Quarantine);
        assert_eq!(receipt.event_type, "mail.message.submitted");
    }

    #[test]
    fn personal_mail_handoff_is_refused_at_usecase_boundary() {
        let ctx = AuthorizedMailContext {
            context: MailApiContext::Personal,
            scope_ref: "person:u".into(),
            principal_ref: "user:u".into(),
            idempotency_key: "idem".into(),
            policy_decision_ref: "cedar:allow:mail-submit".into(),
            audit_correlation_id: "audit".into(),
        };
        let mut req = handoff_req();
        req.envelope = MailApiEnvelope::PersonalClientOnly {
            envelope_ref: "sealed".into(),
        };
        assert_eq!(
            create_workflow_handoff(&ctx, req),
            Err(MailUsecaseError::PersonalHandoffForbidden)
        );
    }

    #[test]
    fn handoff_requires_lawful_basis() {
        let mut req = handoff_req();
        req.lawful_basis_ref.clear();
        assert_eq!(
            create_workflow_handoff(&work_ctx(), req),
            Err(MailUsecaseError::Domain(
                MailGovernanceError::WorkflowPolicyRequired
            ))
        );
    }

    #[test]
    fn ai_assist_rejects_plaintext_prompt() {
        let req = AiAssistRequest {
            message_id: "m".into(),
            mailbox_id: "mb".into(),
            subject_ref: "user:u".into(),
            envelope: MailApiEnvelope::TenantDek {
                dek_ref: "dek".into(),
            },
            retention_policy_id: "retain".into(),
            encrypted_content_ref: "cipher".into(),
            plaintext_in_prompt: true,
            training_enabled: false,
        };
        assert_eq!(
            prepare_ai_assist(&work_ctx(), req),
            Err(MailUsecaseError::Domain(
                MailGovernanceError::AiPlaintextForbidden
            ))
        );
    }

    #[test]
    fn dmarc_reject_maps_to_reject_action() {
        assert_eq!(
            decide_dmarc(DmarcCheckRequest {
                domain_ref: "example.com".into(),
                spf_aligned: false,
                dkim_aligned: false,
                policy: DmarcApiPolicy::Reject,
                evidence_ref: "authres".into(),
            })
            .unwrap(),
            DmarcAction::Reject
        );
    }
}
