#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnonymousError {
    Invalid,
    IdentityFieldForbidden,
    VerificationTokenConsumed,
    VerificationTokenExpired,
    CohortTooSmall,
    IndividualAttributionForbidden,
    LegalHoldRequired,
    TwoDistinctOfficersRequired,
    CourtOrderRequired,
    VaultAccessEvidenceRequired,
    PersonalAnonymousBoardForbidden,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    EmployerHrAdmin,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextKind {
    Work,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousPostPayload {
    pub post_id: String,
    pub channel_id: String,
    pub anonymous_author_token: String,
    pub content_encrypted: String,
    pub topic_tags: Vec<String>,
    pub user_id: Option<String>,
    pub employee_id: Option<String>,
    pub identity_fk: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousPost {
    post_id: String,
    channel_id: String,
    anonymous_author_token: String,
    content_encrypted: String,
    topic_tags: Vec<String>,
}

impl AnonymousPost {
    pub fn new(
        post_id: &str,
        channel_id: &str,
        anonymous_author_token: &str,
        content_encrypted: &str,
        topic_tags: &[&str],
    ) -> Result<Self, AnonymousError> {
        Self::from_payload(AnonymousPostPayload {
            post_id: post_id.into(),
            channel_id: channel_id.into(),
            anonymous_author_token: anonymous_author_token.into(),
            content_encrypted: content_encrypted.into(),
            topic_tags: topic_tags.iter().map(|tag| (*tag).into()).collect(),
            user_id: None,
            employee_id: None,
            identity_fk: None,
        })
    }

    pub fn from_payload(payload: AnonymousPostPayload) -> Result<Self, AnonymousError> {
        reject_identity_fields(&[&payload.user_id, &payload.employee_id, &payload.identity_fk])?;
        for value in [
            &payload.post_id,
            &payload.channel_id,
            &payload.anonymous_author_token,
            &payload.content_encrypted,
        ] {
            non_empty(value)?;
        }
        Ok(Self {
            post_id: payload.post_id,
            channel_id: payload.channel_id,
            anonymous_author_token: payload.anonymous_author_token,
            content_encrypted: payload.content_encrypted,
            topic_tags: payload.topic_tags,
        })
    }

    pub fn anonymous_author_token(&self) -> &str {
        &self.anonymous_author_token
    }

    pub fn identity_fields(&self) -> Vec<&str> {
        Vec::new()
    }

    pub fn search_document(&self) -> String {
        format!(
            "post_id={} channel_id={} anonymous_author_token={} topics={}",
            self.post_id,
            self.channel_id,
            self.anonymous_author_token,
            self.topic_tags.join(",")
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousSalaryPayload {
    pub entry_id: String,
    pub anonymous_author_token: String,
    pub role_level: String,
    pub total_comp_usd: u64,
    pub company_tier: String,
    pub yoe_bucket: String,
    pub region_bucket: String,
    pub user_id: Option<String>,
    pub identity_fk: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnonymousSalaryEntry {
    entry_id: String,
    anonymous_author_token: String,
    role_level: String,
    total_comp_usd: u64,
    company_tier: String,
    yoe_bucket: String,
    region_bucket: String,
}

impl AnonymousSalaryEntry {
    pub fn new(
        entry_id: &str,
        anonymous_author_token: &str,
        role_level: &str,
        total_comp_usd: u64,
        company_tier: &str,
        yoe_bucket: &str,
        region_bucket: &str,
    ) -> Result<Self, AnonymousError> {
        Self::from_payload(AnonymousSalaryPayload {
            entry_id: entry_id.into(),
            anonymous_author_token: anonymous_author_token.into(),
            role_level: role_level.into(),
            total_comp_usd,
            company_tier: company_tier.into(),
            yoe_bucket: yoe_bucket.into(),
            region_bucket: region_bucket.into(),
            user_id: None,
            identity_fk: None,
        })
    }

    pub fn from_payload(payload: AnonymousSalaryPayload) -> Result<Self, AnonymousError> {
        reject_identity_fields(&[&payload.user_id, &payload.identity_fk])?;
        for value in [
            &payload.entry_id,
            &payload.anonymous_author_token,
            &payload.role_level,
            &payload.company_tier,
            &payload.yoe_bucket,
            &payload.region_bucket,
        ] {
            non_empty(value)?;
        }
        if payload.total_comp_usd == 0 {
            return Err(AnonymousError::Invalid);
        }
        Ok(Self {
            entry_id: payload.entry_id,
            anonymous_author_token: payload.anonymous_author_token,
            role_level: payload.role_level,
            total_comp_usd: payload.total_comp_usd,
            company_tier: payload.company_tier,
            yoe_bucket: payload.yoe_bucket,
            region_bucket: payload.region_bucket,
        })
    }

    pub fn anonymous_author_token(&self) -> &str {
        &self.anonymous_author_token
    }

    pub fn identity_fields(&self) -> Vec<&str> {
        Vec::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationToken {
    token_id: String,
    user_id: String,
    tenant_id: String,
    issued_at_hour: u64,
    expires_after_hours: u64,
    consumed_at_hour: Option<u64>,
}

impl VerificationToken {
    pub fn new(
        token_id: &str,
        user_id: &str,
        tenant_id: &str,
        issued_at_hour: u64,
        expires_after_hours: u64,
    ) -> Result<Self, AnonymousError> {
        for value in [token_id, user_id, tenant_id] {
            non_empty(value)?;
        }
        if expires_after_hours == 0 || expires_after_hours > 24 {
            return Err(AnonymousError::Invalid);
        }
        Ok(Self {
            token_id: token_id.into(),
            user_id: user_id.into(),
            tenant_id: tenant_id.into(),
            issued_at_hour,
            expires_after_hours,
            consumed_at_hour: None,
        })
    }

    pub fn consume(&mut self, now_hour: u64) -> Result<(), AnonymousError> {
        if self.consumed_at_hour.is_some() {
            return Err(AnonymousError::VerificationTokenConsumed);
        }
        if now_hour > self.issued_at_hour + self.expires_after_hours {
            return Err(AnonymousError::VerificationTokenExpired);
        }
        self.consumed_at_hour = Some(now_hour);
        Ok(())
    }

    pub fn has_post_foreign_key(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SalaryAggregate {
    pub cohort_size: usize,
    pub min_total_comp_usd: u64,
    pub max_total_comp_usd: u64,
}

impl SalaryAggregate {
    pub fn identity_fields(&self) -> Vec<&str> {
        Vec::new()
    }
}

pub fn salary_benchmark_for_hr_admin(
    entries: &[AnonymousSalaryEntry],
) -> Result<SalaryAggregate, AnonymousError> {
    if entries.len() < 10 {
        return Err(AnonymousError::CohortTooSmall);
    }
    let min_total_comp_usd = entries
        .iter()
        .map(|entry| entry.total_comp_usd)
        .min()
        .unwrap_or(0);
    let max_total_comp_usd = entries
        .iter()
        .map(|entry| entry.total_comp_usd)
        .max()
        .unwrap_or(0);
    Ok(SalaryAggregate {
        cohort_size: entries.len(),
        min_total_comp_usd,
        max_total_comp_usd,
    })
}

pub fn individual_attribution_query(_role: Role) -> Result<(), AnonymousError> {
    Err(AnonymousError::IndividualAttributionForbidden)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEvent {
    pub event_type: String,
    pub anonymous_author_token: String,
    pub reason_code: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModeratorQueueItem {
    pub anonymous_author_token: String,
    pub audit_event: AuditEvent,
}

impl ModeratorQueueItem {
    pub fn identity_fields(&self) -> Vec<&str> {
        Vec::new()
    }
}

pub fn moderator_queue_item(post: &AnonymousPost) -> ModeratorQueueItem {
    ModeratorQueueItem {
        anonymous_author_token: post.anonymous_author_token.clone(),
        audit_event: AuditEvent {
            event_type: "audit.community.anonymous.policy.v1".into(),
            anonymous_author_token: post.anonymous_author_token.clone(),
            reason_code: "moderation_token_only".into(),
        },
    }
}

pub fn moderator_identity_reveal() -> Result<(), AnonymousError> {
    Err(AnonymousError::LegalHoldRequired)
}

pub fn policy_denial_event(resource: &str, error: AnonymousError) -> AuditEvent {
    AuditEvent {
        event_type: "audit.community.anonymous.policy.v1".into(),
        anonymous_author_token: resource.into(),
        reason_code: match error {
            AnonymousError::IdentityFieldForbidden => "identity_field_forbidden",
            AnonymousError::IndividualAttributionForbidden => "individual_attribution_forbidden",
            AnonymousError::CohortTooSmall => "cohort_too_small",
            _ => "anonymous_policy_denied",
        }
        .into(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegalHoldIdentityRevealRequest {
    request_id: String,
    court_order_ref: String,
    requesting_officer_id: String,
    approving_officer_id: String,
    anonymous_author_token: String,
}

impl LegalHoldIdentityRevealRequest {
    pub fn new(
        request_id: &str,
        court_order_ref: &str,
        requesting_officer_id: &str,
        approving_officer_id: &str,
        anonymous_author_token: &str,
    ) -> Result<Self, AnonymousError> {
        for value in [
            request_id,
            court_order_ref,
            requesting_officer_id,
            approving_officer_id,
            anonymous_author_token,
        ] {
            non_empty(value)?;
        }
        if requesting_officer_id == approving_officer_id {
            return Err(AnonymousError::TwoDistinctOfficersRequired);
        }
        Ok(Self {
            request_id: request_id.into(),
            court_order_ref: court_order_ref.into(),
            requesting_officer_id: requesting_officer_id.into(),
            approving_officer_id: approving_officer_id.into(),
            anonymous_author_token: anonymous_author_token.into(),
        })
    }

    pub fn reveal_with_vault(
        self,
        evidence: VaultAccessEvidence,
    ) -> Result<SealedAuditPackage, AnonymousError> {
        if !evidence.court_order_attached {
            return Err(AnonymousError::CourtOrderRequired);
        }
        if !evidence.hsm_access_logged || evidence.immutable_audit_event_ref.trim().is_empty() {
            return Err(AnonymousError::VaultAccessEvidenceRequired);
        }
        Ok(SealedAuditPackage {
            anonymous_author_token: self.anonymous_author_token,
            sealed_audit_ref: format!(
                "sealed:{}:{}",
                self.request_id, evidence.immutable_audit_event_ref
            ),
            normal_path_identity_exposed: false,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultAccessEvidence {
    pub court_order_attached: bool,
    pub hsm_access_logged: bool,
    pub immutable_audit_event_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealedAuditPackage {
    pub anonymous_author_token: String,
    pub sealed_audit_ref: String,
    pub normal_path_identity_exposed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextBoundary {
    pub context_model: Vec<ContextKind>,
    pub no_cross_feed: bool,
    pub no_cross_search: bool,
    pub no_cross_suggest: bool,
    pub mediators: Vec<&'static str>,
}

pub fn context_boundary() -> ContextBoundary {
    ContextBoundary {
        context_model: vec![ContextKind::Work],
        no_cross_feed: true,
        no_cross_search: true,
        no_cross_suggest: true,
        mediators: vec!["Workflow", "Ontology"],
    }
}

pub fn personal_anonymous_board() -> Result<(), AnonymousError> {
    Err(AnonymousError::PersonalAnonymousBoardForbidden)
}

fn reject_identity_fields(fields: &[&Option<String>]) -> Result<(), AnonymousError> {
    if fields.iter().any(|field| {
        field
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
    }) {
        Err(AnonymousError::IdentityFieldForbidden)
    } else {
        Ok(())
    }
}

fn non_empty(value: &str) -> Result<(), AnonymousError> {
    if value.trim().is_empty() {
        Err(AnonymousError::Invalid)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anonymous_post_and_salary_store_token_only_without_identity_fields() {
        let post = AnonymousPost::new(
            "post-1",
            "channel-acme",
            "anon-token-1",
            "ciphertext",
            &["pay"],
        )
        .expect("valid anonymous post");
        assert_eq!(post.anonymous_author_token(), "anon-token-1");
        assert!(post.identity_fields().is_empty());
        assert!(!post.search_document().contains("user_id"));
        assert!(!post.search_document().contains("employee_id"));
        assert!(!post.search_document().contains("identity_fk"));

        let salary = AnonymousSalaryEntry::new(
            "salary-1",
            "anon-token-1",
            "SWE-L5",
            185_000,
            "tier-1",
            "5-8",
            "us-ca",
        )
        .expect("valid anonymous salary entry");
        assert_eq!(salary.anonymous_author_token(), "anon-token-1");
        assert!(salary.identity_fields().is_empty());
    }

    #[test]
    fn identity_carrying_payloads_are_rejected_and_audited() {
        let post_err = AnonymousPost::from_payload(AnonymousPostPayload {
            post_id: "post-1".into(),
            channel_id: "channel-acme".into(),
            anonymous_author_token: "anon-token-1".into(),
            content_encrypted: "ciphertext".into(),
            topic_tags: vec!["pay".into()],
            user_id: Some("user-123".into()),
            employee_id: None,
            identity_fk: None,
        })
        .unwrap_err();
        assert_eq!(post_err, AnonymousError::IdentityFieldForbidden);
        assert_eq!(
            policy_denial_event("post", post_err).reason_code,
            "identity_field_forbidden"
        );

        let salary_err = AnonymousSalaryEntry::from_payload(AnonymousSalaryPayload {
            entry_id: "salary-1".into(),
            anonymous_author_token: "anon-token-1".into(),
            role_level: "SWE-L5".into(),
            total_comp_usd: 185_000,
            company_tier: "tier-1".into(),
            yoe_bucket: "5-8".into(),
            region_bucket: "us-ca".into(),
            user_id: None,
            identity_fk: Some("identity-row".into()),
        })
        .unwrap_err();
        assert_eq!(salary_err, AnonymousError::IdentityFieldForbidden);
    }

    #[test]
    fn verification_vault_token_is_one_time_expires_and_has_no_post_fk() {
        let mut token = VerificationToken::new("token-1", "user-123", "tenant-acme", 100, 24)
            .expect("valid verification token");
        assert!(!token.has_post_foreign_key());
        assert!(token.consume(110).is_ok());
        assert_eq!(
            token.consume(111),
            Err(AnonymousError::VerificationTokenConsumed)
        );

        let mut expired = VerificationToken::new("token-2", "user-123", "tenant-acme", 100, 24)
            .expect("valid verification token");
        assert_eq!(
            expired.consume(125),
            Err(AnonymousError::VerificationTokenExpired)
        );
    }

    #[test]
    fn hr_admin_gets_aggregates_only_and_small_cohorts_are_suppressed() {
        let entries: Vec<_> = (0..10)
            .map(|idx| {
                AnonymousSalaryEntry::new(
                    &format!("salary-{idx}"),
                    &format!("anon-token-{idx}"),
                    "SWE-L5",
                    150_000 + idx,
                    "tier-1",
                    "5-8",
                    "us-ca",
                )
                .unwrap()
            })
            .collect();
        let aggregate = salary_benchmark_for_hr_admin(&entries).expect("k=10 cohort is eligible");
        assert_eq!(aggregate.cohort_size, 10);
        assert_eq!(aggregate.min_total_comp_usd, 150_000);
        assert_eq!(aggregate.max_total_comp_usd, 150_009);
        assert!(aggregate.identity_fields().is_empty());
        assert_eq!(
            individual_attribution_query(Role::EmployerHrAdmin),
            Err(AnonymousError::IndividualAttributionForbidden)
        );

        assert_eq!(
            salary_benchmark_for_hr_admin(&entries[..9]),
            Err(AnonymousError::CohortTooSmall)
        );
    }

    #[test]
    fn moderator_sees_token_only_and_never_real_identity() {
        let post = AnonymousPost::new(
            "post-1",
            "channel-acme",
            "anon-token-1",
            "ciphertext",
            &["policy"],
        )
        .unwrap();
        let view = moderator_queue_item(&post);
        assert_eq!(view.anonymous_author_token, "anon-token-1");
        assert!(view.identity_fields().is_empty());
        assert_eq!(
            view.audit_event.event_type,
            "audit.community.anonymous.policy.v1"
        );
        assert_eq!(view.audit_event.anonymous_author_token, "anon-token-1");
        assert_eq!(
            moderator_identity_reveal(),
            Err(AnonymousError::LegalHoldRequired)
        );
    }

    #[test]
    fn legal_hold_identity_reveal_requires_four_eyes_court_order_vault_and_audit() {
        let denied = LegalHoldIdentityRevealRequest::new(
            "req-1",
            "order-1",
            "officer-a",
            "officer-a",
            "anon-token-1",
        )
        .unwrap_err();
        assert_eq!(denied, AnonymousError::TwoDistinctOfficersRequired);

        let request = LegalHoldIdentityRevealRequest::new(
            "req-1",
            "order-1",
            "officer-a",
            "officer-b",
            "anon-token-1",
        )
        .expect("valid four-eyes request");
        assert_eq!(
            request.clone().reveal_with_vault(VaultAccessEvidence {
                court_order_attached: false,
                hsm_access_logged: true,
                immutable_audit_event_ref: "audit-chain:evt-1".into(),
            }),
            Err(AnonymousError::CourtOrderRequired)
        );
        assert_eq!(
            request.clone().reveal_with_vault(VaultAccessEvidence {
                court_order_attached: true,
                hsm_access_logged: false,
                immutable_audit_event_ref: "".into(),
            }),
            Err(AnonymousError::VaultAccessEvidenceRequired)
        );
        let sealed = request
            .reveal_with_vault(VaultAccessEvidence {
                court_order_attached: true,
                hsm_access_logged: true,
                immutable_audit_event_ref: "audit-chain:evt-1".into(),
            })
            .expect("legal reveal should be sealed");
        assert_eq!(sealed.anonymous_author_token, "anon-token-1");
        assert_eq!(sealed.sealed_audit_ref, "sealed:req-1:audit-chain:evt-1");
        assert!(!sealed.normal_path_identity_exposed);
    }

    #[test]
    fn work_context_routes_through_workflow_and_ontology_without_personal_leakage() {
        let boundary = context_boundary();
        assert_eq!(boundary.context_model, vec![ContextKind::Work]);
        assert!(boundary.no_cross_feed);
        assert!(boundary.no_cross_search);
        assert!(boundary.no_cross_suggest);
        assert_eq!(boundary.mediators, vec!["Workflow", "Ontology"]);
        assert_eq!(
            personal_anonymous_board(),
            Err(AnonymousError::PersonalAnonymousBoardForbidden)
        );
    }
}
