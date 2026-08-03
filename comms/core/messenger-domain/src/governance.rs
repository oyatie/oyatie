use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessengerGovernanceError {
    Invalid,
    ContextPillarMismatch,
    PersonalRequiresE2e,
    WorkRequiresTenantDek,
    FourEyesRequired,
    PersonalNotDiscoverable,
    CrossPillarPresenceDenied,
    CrossOrgIsolation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessengerContextKind {
    Personal,
    Work,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnershipPillar {
    Personal,
    Work,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageEnvelope {
    PersonalE2e {
        envelope_ref: String,
    },
    TenantDek {
        dek_ref: String,
        four_eyes: bool,
    },
    CrossOrg {
        local_dek_ref: String,
        partner_scope_ref: String,
        partner_dek_ref: String,
        partner_ediscovery_allowed: bool,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageGovernanceCreate {
    pub message_id: String,
    pub scope_ref: String,
    pub context: MessengerContextKind,
    pub pillar: OwnershipPillar,
    pub author_ref: String,
    pub envelope: MessageEnvelope,
    pub retention_policy_id: String,
    pub legal_hold_ids: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageGovernance {
    pub message_id: Classified<String>,
    pub scope_ref: Classified<String>,
    pub context: Classified<MessengerContextKind>,
    pub pillar: Classified<OwnershipPillar>,
    pub author_ref: Classified<String>,
    pub envelope: Classified<MessageEnvelope>,
    pub retention_policy_id: Classified<String>,
    pub legal_hold_ids: Classified<Vec<String>>,
    pub schema_version: Classified<u32>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageAuditAction {
    DisclosureDecrypt,
    Edit,
    Delete,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageAuditRecord {
    pub message_id: Classified<String>,
    pub action: Classified<MessageAuditAction>,
    pub requested_by: Classified<String>,
    pub approved_by: Classified<String>,
    pub reason_ref: Classified<String>,
    pub audit_chain_ref: Classified<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InheritedMessageChild {
    pub child_id: Classified<String>,
    pub parent_message_id: Classified<String>,
    pub retention_policy_id: Classified<String>,
    pub legal_hold_ids: Classified<Vec<String>>,
    pub envelope: Classified<MessageEnvelope>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresenceState {
    pub identity_ref: Classified<String>,
    pub context: Classified<MessengerContextKind>,
    pub visible_to_context: Classified<MessengerContextKind>,
    pub explicit_grant_ref: Classified<Option<String>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousAuthorEnvelope {
    pub routine_display_ref: Classified<String>,
    pub audit_author_ref: Classified<String>,
    pub disclosure_policy_ref: Classified<String>,
}
impl MessageGovernance {
    pub fn new(i: MessageGovernanceCreate) -> Result<Self, MessengerGovernanceError> {
        ne(&i.message_id)?;
        ne(&i.scope_ref)?;
        ne(&i.author_ref)?;
        ne(&i.retention_policy_id)?;
        match (i.context, i.pillar) {
            (MessengerContextKind::Personal, OwnershipPillar::Personal)
            | (MessengerContextKind::Work, OwnershipPillar::Work) => (),
            _ => return Err(MessengerGovernanceError::ContextPillarMismatch),
        };
        match (&i.context, &i.envelope) {
            (MessengerContextKind::Personal, MessageEnvelope::PersonalE2e { envelope_ref }) => {
                ne(envelope_ref)?
            }
            (MessengerContextKind::Personal, _) => {
                return Err(MessengerGovernanceError::PersonalRequiresE2e);
            }
            (MessengerContextKind::Work, MessageEnvelope::TenantDek { dek_ref, four_eyes }) => {
                ne(dek_ref)?;
                if !four_eyes {
                    return Err(MessengerGovernanceError::FourEyesRequired);
                }
            }
            (
                MessengerContextKind::Work,
                MessageEnvelope::CrossOrg {
                    local_dek_ref,
                    partner_scope_ref,
                    partner_dek_ref,
                    partner_ediscovery_allowed,
                },
            ) => {
                ne(local_dek_ref)?;
                ne(partner_scope_ref)?;
                ne(partner_dek_ref)?;
                if partner_scope_ref == &i.scope_ref
                    || local_dek_ref == partner_dek_ref
                    || *partner_ediscovery_allowed
                {
                    return Err(MessengerGovernanceError::CrossOrgIsolation);
                }
            }
            (MessengerContextKind::Work, MessageEnvelope::PersonalE2e { .. }) => {
                return Err(MessengerGovernanceError::WorkRequiresTenantDek);
            }
        };
        Ok(Self {
            message_id: int(i.message_id),
            scope_ref: int(i.scope_ref),
            context: int(i.context),
            pillar: int(i.pillar),
            author_ref: Classified::new(i.author_ref, PrivacyDataClass::pii_identifying()),
            envelope: int(i.envelope),
            retention_policy_id: int(i.retention_policy_id),
            legal_hold_ids: int(i.legal_hold_ids),
            schema_version: int(1),
        })
    }
    pub fn org_admin_can_decrypt(&self) -> bool {
        self.context.value == MessengerContextKind::Work
    }
    pub fn personal_undecryptable_by_org(&self) -> bool {
        self.context.value == MessengerContextKind::Personal
            && matches!(self.envelope.value, MessageEnvelope::PersonalE2e { .. })
            && !self.org_admin_can_decrypt()
    }
}
impl MessageAuditRecord {
    pub fn new(
        message: &MessageGovernance,
        action: MessageAuditAction,
        requested_by: String,
        approved_by: String,
        reason_ref: String,
        audit_chain_ref: String,
    ) -> Result<Self, MessengerGovernanceError> {
        if message.context.value != MessengerContextKind::Work {
            return Err(MessengerGovernanceError::PersonalNotDiscoverable);
        };
        ne(&requested_by)?;
        ne(&approved_by)?;
        ne(&reason_ref)?;
        ne(&audit_chain_ref)?;
        if requested_by == approved_by {
            return Err(MessengerGovernanceError::FourEyesRequired);
        };
        Ok(Self {
            message_id: int(message.message_id.value.clone()),
            action: int(action),
            requested_by: Classified::new(requested_by, PrivacyDataClass::pii_identifying()),
            approved_by: Classified::new(approved_by, PrivacyDataClass::pii_identifying()),
            reason_ref: int(reason_ref),
            audit_chain_ref: Classified::new(audit_chain_ref, DataClass::Audit),
        })
    }
}
impl InheritedMessageChild {
    pub fn from_parent(
        child_id: String,
        parent: &MessageGovernance,
    ) -> Result<Self, MessengerGovernanceError> {
        ne(&child_id)?;
        Ok(Self {
            child_id: int(child_id),
            parent_message_id: int(parent.message_id.value.clone()),
            retention_policy_id: int(parent.retention_policy_id.value.clone()),
            legal_hold_ids: int(parent.legal_hold_ids.value.clone()),
            envelope: int(parent.envelope.value.clone()),
        })
    }
}
impl PresenceState {
    pub fn new(
        identity_ref: String,
        context: MessengerContextKind,
        visible_to_context: MessengerContextKind,
        explicit_grant_ref: Option<String>,
    ) -> Result<Self, MessengerGovernanceError> {
        ne(&identity_ref)?;
        if context != visible_to_context
            && explicit_grant_ref
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            return Err(MessengerGovernanceError::CrossPillarPresenceDenied);
        };
        Ok(Self {
            identity_ref: Classified::new(identity_ref, PrivacyDataClass::pii_identifying()),
            context: int(context),
            visible_to_context: int(visible_to_context),
            explicit_grant_ref: int(explicit_grant_ref),
        })
    }
}
impl AnonymousAuthorEnvelope {
    pub fn new(
        routine: String,
        audit: String,
        policy: String,
    ) -> Result<Self, MessengerGovernanceError> {
        ne(&routine)?;
        ne(&audit)?;
        ne(&policy)?;
        if routine == audit {
            return Err(MessengerGovernanceError::Invalid);
        };
        Ok(Self {
            routine_display_ref: Classified::new(routine, PrivacyDataClass::pii_quasi_identifier()),
            audit_author_ref: Classified::new(audit, PrivacyDataClass::pii_identifying()),
            disclosure_policy_ref: int(policy),
        })
    }
}
fn ne(s: &str) -> Result<(), MessengerGovernanceError> {
    if s.trim().is_empty() {
        Err(MessengerGovernanceError::Invalid)
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
    fn personal() -> MessageGovernance {
        MessageGovernance::new(MessageGovernanceCreate {
            message_id: "m1".into(),
            scope_ref: "person:u1".into(),
            context: MessengerContextKind::Personal,
            pillar: OwnershipPillar::Personal,
            author_ref: "user:p".into(),
            envelope: MessageEnvelope::PersonalE2e {
                envelope_ref: "mls:1".into(),
            },
            retention_policy_id: "personal".into(),
            legal_hold_ids: vec![],
        })
        .unwrap()
    }
    fn work() -> MessageGovernance {
        MessageGovernance::new(MessageGovernanceCreate {
            message_id: "m2".into(),
            scope_ref: "tenant:t1".into(),
            context: MessengerContextKind::Work,
            pillar: OwnershipPillar::Work,
            author_ref: "user:w".into(),
            envelope: MessageEnvelope::TenantDek {
                dek_ref: "dek:t1".into(),
                four_eyes: true,
            },
            retention_policy_id: "retain".into(),
            legal_hold_ids: vec!["hold".into()],
        })
        .unwrap()
    }
    #[test]
    fn personal_dm_never_org_discoverable() {
        assert!(personal().personal_undecryptable_by_org())
    }
    #[test]
    fn work_message_four_eyes_disclosure() {
        assert!(
            MessageAuditRecord::new(
                &work(),
                MessageAuditAction::DisclosureDecrypt,
                "a".into(),
                "b".into(),
                "case".into(),
                "audit".into()
            )
            .is_ok()
        )
    }
    #[test]
    fn verified_anonymity_routine_vs_audit() {
        let a = AnonymousAuthorEnvelope::new("anon".into(), "user:real".into(), "policy".into())
            .unwrap();
        assert_ne!(a.routine_display_ref.value, a.audit_author_ref.value)
    }
    #[test]
    fn attachment_thread_reaction_inherit_parent() {
        let p = work();
        let c = InheritedMessageChild::from_parent("child".into(), &p).unwrap();
        assert_eq!(c.retention_policy_id.value, p.retention_policy_id.value);
        assert_eq!(c.envelope.value, p.envelope.value)
    }
    #[test]
    fn presence_pillar_isolation() {
        assert_eq!(
            PresenceState::new(
                "u".into(),
                MessengerContextKind::Personal,
                MessengerContextKind::Work,
                None
            ),
            Err(MessengerGovernanceError::CrossPillarPresenceDenied)
        )
    }
    #[test]
    fn cross_org_channel_dek_isolation() {
        assert!(
            MessageGovernance::new(MessageGovernanceCreate {
                message_id: "m3".into(),
                scope_ref: "tenant:a".into(),
                context: MessengerContextKind::Work,
                pillar: OwnershipPillar::Work,
                author_ref: "u".into(),
                envelope: MessageEnvelope::CrossOrg {
                    local_dek_ref: "dek:a".into(),
                    partner_scope_ref: "tenant:b".into(),
                    partner_dek_ref: "dek:b".into(),
                    partner_ediscovery_allowed: false
                },
                retention_policy_id: "retain".into(),
                legal_hold_ids: vec![]
            })
            .is_ok()
        )
    }
}
