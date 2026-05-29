//! Cedar-shaped authorization policy kernel.
//!
//! This is deliberately pure: it stores versioned policy records and evaluates
//! role + attribute predicates without network, storage, or runtime side effects.
//!
//! The `authz_engine` sub-module adds P14-policy `AuthzRequest` / `AuthzDecision`
//! / `EvalLogFilter` value types that encode the Cedar evaluation contract without
//! importing any framework crates beyond `serde`.  These types will migrate into a
//! dedicated `oya-policy-engine-kernel` crate when IP-001 scaffolds the full
//! policy-engine BC (P14 impl-plan, `execution_variant = merge-into-existing-crates`).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PolicyScope {
    Global,
    Tenant(String),
}

/// Effect of a policy rule; also re-used as the decision discriminant in `authz_engine`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyRuleInput {
    pub effect: PolicyEffect,
    pub principal_role: String,
    pub action: String,
    pub resource_prefix: String,
    pub required_attribute: Option<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyRule {
    pub effect: PolicyEffect,
    pub principal_role: String,
    pub action: String,
    pub resource_prefix: String,
    pub required_attribute: Option<(String, String)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyVersion {
    pub policy_id: String,
    pub version: String,
    pub scope: PolicyScope,
    pub supersedes: Option<String>,
    pub rules: Vec<PolicyRuleInput>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedPolicy {
    pub policy_id: String,
    pub version: String,
    pub scope: PolicyScope,
    pub supersedes: Option<String>,
    pub rules: Vec<PolicyRule>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationSubject {
    pub tenant_id: String,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationQuery {
    pub subject: AuthorizationSubject,
    pub action: String,
    pub resource: String,
    pub attributes: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationDecision {
    pub allowed: bool,
    pub reason: String,
    pub matched_policy: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyError {
    InvalidPolicyId,
    InvalidSemver,
    EmptyRules,
    EmptyRuleField,
    VersionAlreadyExists,
    SupersedesSelf,
    SupersedesMissing,
    SupersedesScopeMismatch,
    SupersedesNotOlder,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PolicySet {
    policies: BTreeMap<(String, String), PublishedPolicy>,
}

impl PolicySet {
    pub fn publish(&mut self, version: PolicyVersion) -> Result<PublishedPolicy, PolicyError> {
        validate_policy_id(&version.policy_id)?;
        let parsed_version = parse_semver(&version.version)?;
        let key = (version.policy_id.clone(), version.version.clone());
        if self.policies.contains_key(&key) {
            return Err(PolicyError::VersionAlreadyExists);
        }
        if version.rules.is_empty() {
            return Err(PolicyError::EmptyRules);
        }
        if let Some(superseded_version) = version.supersedes.as_ref() {
            let parsed_superseded_version = parse_semver(superseded_version)?;
            if superseded_version == &version.version {
                return Err(PolicyError::SupersedesSelf);
            }
            if parsed_superseded_version >= parsed_version {
                return Err(PolicyError::SupersedesNotOlder);
            }
            let superseded_policy = self
                .policies
                .get(&(version.policy_id.clone(), superseded_version.clone()))
                .ok_or(PolicyError::SupersedesMissing)?;
            if superseded_policy.scope != version.scope {
                return Err(PolicyError::SupersedesScopeMismatch);
            }
        }
        let rules = version
            .rules
            .into_iter()
            .map(PolicyRule::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let published = PublishedPolicy {
            policy_id: version.policy_id,
            version: version.version,
            scope: version.scope,
            supersedes: version.supersedes,
            rules,
        };
        self.policies.insert(key, published.clone());
        Ok(published)
    }

    pub fn get(&self, policy_id: &str, version: &str) -> Option<&PublishedPolicy> {
        self.policies
            .get(&(policy_id.to_string(), version.to_string()))
    }

    pub fn supersession_chain(
        &self,
        policy_id: &str,
        version: &str,
    ) -> Option<Vec<PublishedPolicy>> {
        let mut chain = Vec::new();
        let mut next_version = Some(version.to_string());
        while let Some(current_version) = next_version {
            let policy = self.get(policy_id, &current_version)?;
            next_version = policy.supersedes.clone();
            chain.push(policy.clone());
        }
        Some(chain)
    }

    pub fn authorize(&self, query: &AuthorizationQuery) -> AuthorizationDecision {
        let superseded_keys = self
            .policies
            .values()
            .filter_map(|policy| {
                policy
                    .supersedes
                    .as_ref()
                    .map(|version| (policy.policy_id.clone(), version.clone()))
            })
            .collect::<Vec<_>>();
        let mut scoped_policies = self
            .policies
            .values()
            .filter(|policy| {
                !superseded_keys
                    .iter()
                    .any(|key| key == &(policy.policy_id.clone(), policy.version.clone()))
            })
            .filter(|policy| match &policy.scope {
                PolicyScope::Global => true,
                PolicyScope::Tenant(tenant_id) => tenant_id == &query.subject.tenant_id,
            })
            .collect::<Vec<_>>();
        scoped_policies.sort_by(|left, right| {
            (&left.policy_id, &left.version).cmp(&(&right.policy_id, &right.version))
        });

        for policy in &scoped_policies {
            for rule in &policy.rules {
                if rule.matches(query) && rule.effect == PolicyEffect::Deny {
                    return AuthorizationDecision {
                        allowed: false,
                        reason: "explicit deny policy".to_string(),
                        matched_policy: Some(policy.policy_id.clone()),
                    };
                }
            }
        }
        for policy in scoped_policies {
            for rule in &policy.rules {
                if rule.matches(query) && rule.effect == PolicyEffect::Allow {
                    return AuthorizationDecision {
                        allowed: true,
                        reason: "matching allow policy".to_string(),
                        matched_policy: Some(policy.policy_id.clone()),
                    };
                }
            }
        }
        AuthorizationDecision {
            allowed: false,
            reason: "no matching allow policy".to_string(),
            matched_policy: None,
        }
    }
}

impl TryFrom<PolicyRuleInput> for PolicyRule {
    type Error = PolicyError;

    fn try_from(input: PolicyRuleInput) -> Result<Self, Self::Error> {
        if input.principal_role.trim().is_empty()
            || input.action.trim().is_empty()
            || input.resource_prefix.trim().is_empty()
        {
            return Err(PolicyError::EmptyRuleField);
        }
        Ok(Self {
            effect: input.effect,
            principal_role: input.principal_role,
            action: input.action,
            resource_prefix: input.resource_prefix,
            required_attribute: input.required_attribute,
        })
    }
}

impl PolicyRule {
    fn matches(&self, query: &AuthorizationQuery) -> bool {
        query
            .subject
            .roles
            .iter()
            .any(|role| role == &self.principal_role)
            && query.action == self.action
            && query.resource.starts_with(&self.resource_prefix)
            && match self.required_attribute.as_ref() {
                Some((key, expected)) => query
                    .attributes
                    .get(key)
                    .is_some_and(|actual| actual == expected),
                None => true,
            }
    }
}

fn validate_policy_id(policy_id: &str) -> Result<(), PolicyError> {
    if policy_id.starts_with("pol_") && policy_id.len() > 4 {
        Ok(())
    } else {
        Err(PolicyError::InvalidPolicyId)
    }
}

fn parse_semver(version: &str) -> Result<[u64; 3], PolicyError> {
    let parts = version.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(PolicyError::InvalidSemver);
    }
    let mut parsed = [0_u64; 3];
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty()
            || !part.bytes().all(|byte| byte.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0'))
        {
            return Err(PolicyError::InvalidSemver);
        }
        parsed[index] = part
            .parse::<u64>()
            .map_err(|_| PolicyError::InvalidSemver)?;
    }
    Ok(parsed)
}

/// P14-policy Cedar engine types — `AuthzRequest`, `AuthzDecision`, `EvalLogFilter`.
///
/// Pure value types for Cedar-based authorization evaluation. IDs use `String`
/// to match the existing codebase convention. Wire-marshaling crosses the gRPC/HTTP
/// boundary at the adapter layer — kernel keeps zero external deps beyond serde.
/// `PolicyEffect` (defined above) is re-used as the decision effect discriminant.
pub mod authz_engine {
    use serde::{Deserialize, Serialize};
    use serde_json::Value as JsonValue;

    use crate::PolicyEffect;

    /// The principal type that is making an authorization request.
    ///
    /// Maps 1:1 to Cedar entity types as defined in ADR-0007.
    /// Serialized with Cedar PascalCase names (`"User"`, `"Employee"`, …) so that
    /// the wire format matches `as_cedar_str()` and Cedar policy evaluation engines
    /// do not require remapping at every boundary.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub enum PrincipalType {
        User,
        Employee,
        System,
        Llm,
        Workflow,
    }

    impl PrincipalType {
        /// Returns the Cedar entity-type string for this principal.
        pub const fn as_cedar_str(&self) -> &'static str {
            match self {
                Self::User => "User",
                Self::Employee => "Employee",
                Self::System => "System",
                Self::Llm => "Llm",
                Self::Workflow => "Workflow",
            }
        }
    }

    /// An authorization request routed to the Cedar policy evaluator.
    ///
    /// `principal_id` is `None` for anonymous/system-level requests.
    /// `context` carries arbitrary key→value attributes used by Cedar condition clauses.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct AuthzRequest {
        /// Tenant the request is scoped to; matches `PolicyScope::Tenant`.
        pub tenant_id: String,
        /// Cedar principal entity type.
        pub principal_type: PrincipalType,
        /// Optional principal identifier (e.g. user-id, employee-id).
        pub principal_id: Option<String>,
        /// Cedar action string: `"Read"` | `"Write"` | `"Apply"` | …
        pub action: String,
        /// Cedar resource entity type: `"Object"` | `"WorkflowRun"` | …
        pub resource_type: String,
        /// Optional resource identifier.
        pub resource_id: Option<String>,
        /// Arbitrary Cedar context attributes (key → typed value).
        ///
        /// Values use `serde_json::Value` to preserve booleans, numbers, and
        /// nested objects that Cedar policy conditions commonly evaluate.
        /// Coercing to `String` would silently break numeric comparisons and
        /// boolean guards in Cedar policy rules.
        ///
        /// Defaults to an empty map when absent in JSON so minimal authz
        /// payloads that omit `context` deserialize successfully.
        #[serde(default)]
        pub context: std::collections::BTreeMap<String, JsonValue>,
    }

    /// The outcome of a Cedar policy evaluation.
    ///
    /// `determining_policies` lists the policy IDs that drove the decision;
    /// `errors` lists non-fatal evaluation errors (e.g. missing context keys).
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct AuthzDecision {
        /// The net effect after evaluating all applicable rule packs.
        pub effect: PolicyEffect,
        /// Policy IDs that contributed to this decision (may be empty for default-deny).
        pub determining_policies: Vec<String>,
        /// Non-fatal evaluation errors encountered during rule-pack evaluation.
        pub errors: Vec<String>,
    }

    impl AuthzDecision {
        /// Convenience constructor for an explicit allow decision.
        pub fn allow(determining_policies: Vec<String>) -> Self {
            Self {
                effect: PolicyEffect::Allow,
                determining_policies,
                errors: Vec::new(),
            }
        }

        /// Convenience constructor for a default-deny (no matching allow rule).
        pub fn default_deny() -> Self {
            Self {
                effect: PolicyEffect::Deny,
                determining_policies: Vec::new(),
                errors: Vec::new(),
            }
        }

        /// Convenience constructor for an explicit deny decision.
        pub fn explicit_deny(determining_policies: Vec<String>) -> Self {
            Self {
                effect: PolicyEffect::Deny,
                determining_policies,
                errors: Vec::new(),
            }
        }

        /// Returns `true` if the evaluation produced an `Allow` effect.
        pub fn is_allowed(&self) -> bool {
            self.effect == PolicyEffect::Allow
        }
    }

    /// Filter parameters for querying the evaluation log.
    ///
    /// All fields are optional; `limit` defaults to `100`.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct EvalLogFilter {
        /// Restrict to a specific principal identifier.
        pub principal_id: Option<String>,
        /// Restrict to a specific effect (`Allow` or `Deny`).
        pub effect: Option<PolicyEffect>,
        /// Restrict to a specific Cedar resource type.
        pub resource_type: Option<String>,
        /// Maximum number of log entries to return (default `100`).
        #[serde(default = "EvalLogFilter::default_limit")]
        pub limit: u32,
    }

    impl EvalLogFilter {
        fn default_limit() -> u32 {
            100
        }
    }

    impl Default for EvalLogFilter {
        fn default() -> Self {
            Self {
                principal_id: None,
                effect: None,
                resource_type: None,
                limit: 100,
            }
        }
    }
}

pub const BACKBONE_WRITE_POLICY_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackboneWriteOperation {
    MessengerPostMessage,
    MailSubmitMessage,
    SocialPublishPost,
    CommunityCreatePost,
    CommunityCastVote,
    CommunityApplyModerationAction,
}

impl BackboneWriteOperation {
    pub const fn all() -> [Self; 6] {
        [
            Self::MessengerPostMessage,
            Self::MailSubmitMessage,
            Self::SocialPublishPost,
            Self::CommunityCreatePost,
            Self::CommunityCastVote,
            Self::CommunityApplyModerationAction,
        ]
    }

    pub const fn policy_id(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "pol_backbone_messenger_message_post",
            Self::MailSubmitMessage => "pol_backbone_mail_message_submit",
            Self::SocialPublishPost => "pol_backbone_social_post_publish",
            Self::CommunityCreatePost => "pol_backbone_community_post_create",
            Self::CommunityCastVote => "pol_backbone_community_vote_cast",
            Self::CommunityApplyModerationAction => "pol_backbone_community_moderation_apply",
        }
    }

    pub const fn action(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "messenger.message.post",
            Self::MailSubmitMessage => "mail.message.submit",
            Self::SocialPublishPost => "social.post.publish",
            Self::CommunityCreatePost => "community.post.create",
            Self::CommunityCastVote => "community.vote.cast",
            Self::CommunityApplyModerationAction => "community.moderation.apply",
        }
    }

    pub const fn principal_role(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "messenger-writer",
            Self::MailSubmitMessage => "mail-sender",
            Self::SocialPublishPost => "social-publisher",
            Self::CommunityCreatePost => "community-author",
            Self::CommunityCastVote => "community-voter",
            Self::CommunityApplyModerationAction => "community-moderator",
        }
    }

    pub const fn resource_type(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "messenger:channel",
            Self::MailSubmitMessage => "mail:mailbox",
            Self::SocialPublishPost => "social:profile",
            Self::CommunityCreatePost => "community:space",
            Self::CommunityCastVote | Self::CommunityApplyModerationAction => "community:post",
        }
    }

    pub const fn resource_prefix(self) -> &'static str {
        match self {
            Self::MessengerPostMessage => "messenger:channel:",
            Self::MailSubmitMessage => "mail:mailbox:",
            Self::SocialPublishPost => "social:profile:",
            Self::CommunityCreatePost => "community:space:",
            Self::CommunityCastVote | Self::CommunityApplyModerationAction => "community:post:",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CedarRuntimeError {
    MissingTenantId,
    MissingAction,
    MissingResourceType,
    MissingAuditCorrelation,
    InvalidRoleContext,
    Policy(PolicyError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarEvaluationLogEntry {
    pub decision_ref: String,              // data_class: INTERNAL_ONLY
    pub tenant_id: String,                 // data_class: INTERNAL_ONLY
    pub principal_id: Option<String>,      // data_class: INTERNAL_ONLY
    pub action: String,                    // data_class: INTERNAL_ONLY
    pub resource_type: String,             // data_class: INTERNAL_ONLY
    pub resource_id: Option<String>,       // data_class: INTERNAL_ONLY
    pub effect: PolicyEffect,              // data_class: INTERNAL_ONLY
    pub determining_policies: Vec<String>, // data_class: INTERNAL_ONLY
    pub audit_correlation_id: String,      // data_class: INTERNAL_ONLY
    pub reason: String,                    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarRuntimeEvaluation {
    pub decision_ref: String,                  // data_class: INTERNAL_ONLY
    pub decision: authz_engine::AuthzDecision, // data_class: INTERNAL_ONLY
    pub log_entry: CedarEvaluationLogEntry,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CedarRuntimeEvaluator {
    policy_set: PolicySet,             // data_class: INTERNAL_ONLY
    log: Vec<CedarEvaluationLogEntry>, // data_class: INTERNAL_ONLY
    next_decision_sequence: u64,       // data_class: INTERNAL_ONLY
}

impl Default for CedarRuntimeEvaluator {
    fn default() -> Self {
        Self {
            policy_set: PolicySet::default(),
            log: Vec::new(),
            next_decision_sequence: 1,
        }
    }
}

impl CedarRuntimeEvaluator {
    pub fn from_policy_versions(
        versions: impl IntoIterator<Item = PolicyVersion>,
    ) -> Result<Self, CedarRuntimeError> {
        let mut policy_set = PolicySet::default();
        for version in versions {
            policy_set
                .publish(version)
                .map_err(CedarRuntimeError::Policy)?;
        }
        Ok(Self {
            policy_set,
            log: Vec::new(),
            next_decision_sequence: 1,
        })
    }

    pub fn with_backbone_write_policies(
        tenant_id: impl Into<String>,
    ) -> Result<Self, CedarRuntimeError> {
        Self::from_policy_versions(backbone_write_policy_versions(tenant_id))
    }

    pub fn evaluate(
        &mut self,
        request: authz_engine::AuthzRequest,
        audit_correlation_id: impl Into<String>,
    ) -> Result<CedarRuntimeEvaluation, CedarRuntimeError> {
        validate_authz_request(&request)?;
        let audit_correlation_id = audit_correlation_id.into();
        if audit_correlation_id.trim().is_empty() {
            return Err(CedarRuntimeError::MissingAuditCorrelation);
        }
        let query = authorization_query_from_authz_request(&request)?;
        let authorization = self.policy_set.authorize(&query);
        let decision = authz_decision_from_authorization(&authorization);
        let decision_ref = self.next_decision_ref(&decision);
        let log_entry = CedarEvaluationLogEntry {
            decision_ref: decision_ref.clone(),
            tenant_id: request.tenant_id,
            principal_id: request.principal_id,
            action: request.action,
            resource_type: request.resource_type,
            resource_id: request.resource_id,
            effect: decision.effect,
            determining_policies: decision.determining_policies.clone(),
            audit_correlation_id,
            reason: authorization.reason,
        };
        self.log.push(log_entry.clone());
        Ok(CedarRuntimeEvaluation {
            decision_ref,
            decision,
            log_entry,
        })
    }

    pub fn eval_log(&self, filter: &authz_engine::EvalLogFilter) -> Vec<CedarEvaluationLogEntry> {
        self.log
            .iter()
            .filter(|entry| {
                filter
                    .principal_id
                    .as_ref()
                    .is_none_or(|principal_id| entry.principal_id.as_ref() == Some(principal_id))
                    && filter.effect.is_none_or(|effect| entry.effect == effect)
                    && filter
                        .resource_type
                        .as_ref()
                        .is_none_or(|resource_type| entry.resource_type == *resource_type)
            })
            .take(filter.limit as usize)
            .cloned()
            .collect()
    }

    pub fn log_len(&self) -> usize {
        self.log.len()
    }

    fn next_decision_ref(&mut self, decision: &authz_engine::AuthzDecision) -> String {
        let effect = match decision.effect {
            PolicyEffect::Allow => "allow",
            PolicyEffect::Deny => "deny",
        };
        let policy_ref = decision
            .determining_policies
            .first()
            .map_or("default", String::as_str);
        let sequence = self.next_decision_sequence;
        self.next_decision_sequence += 1;
        format!("cedar:{effect}:{policy_ref}:{sequence}")
    }
}

pub fn backbone_write_policy_versions(tenant_id: impl Into<String>) -> Vec<PolicyVersion> {
    let tenant_id = tenant_id.into();
    BackboneWriteOperation::all()
        .into_iter()
        .map(|operation| PolicyVersion {
            policy_id: operation.policy_id().to_string(),
            version: BACKBONE_WRITE_POLICY_VERSION.to_string(),
            scope: PolicyScope::Tenant(tenant_id.clone()),
            supersedes: None,
            rules: vec![PolicyRuleInput {
                effect: PolicyEffect::Allow,
                principal_role: operation.principal_role().to_string(),
                action: operation.action().to_string(),
                resource_prefix: operation.resource_prefix().to_string(),
                required_attribute: Some(("data_plane".to_string(), "backbone".to_string())),
            }],
        })
        .collect()
}

fn validate_authz_request(request: &authz_engine::AuthzRequest) -> Result<(), CedarRuntimeError> {
    if request.tenant_id.trim().is_empty() {
        return Err(CedarRuntimeError::MissingTenantId);
    }
    if request.action.trim().is_empty() {
        return Err(CedarRuntimeError::MissingAction);
    }
    if request.resource_type.trim().is_empty() {
        return Err(CedarRuntimeError::MissingResourceType);
    }
    Ok(())
}

fn authorization_query_from_authz_request(
    request: &authz_engine::AuthzRequest,
) -> Result<AuthorizationQuery, CedarRuntimeError> {
    Ok(AuthorizationQuery {
        subject: AuthorizationSubject {
            tenant_id: request.tenant_id.clone(),
            roles: roles_from_context(request)?,
        },
        action: request.action.clone(),
        resource: resource_ref_from_request(request),
        attributes: string_attributes_from_context(request),
    })
}

fn roles_from_context(
    request: &authz_engine::AuthzRequest,
) -> Result<Vec<String>, CedarRuntimeError> {
    let Some(value) = request.context.get("roles") else {
        return Ok(vec![request.principal_type.as_cedar_str().to_string()]);
    };
    match value {
        serde_json::Value::String(role) if !role.trim().is_empty() => Ok(vec![role.clone()]),
        serde_json::Value::Array(values) => values
            .iter()
            .map(|value| match value {
                serde_json::Value::String(role) if !role.trim().is_empty() => Ok(role.clone()),
                _ => Err(CedarRuntimeError::InvalidRoleContext),
            })
            .collect(),
        _ => Err(CedarRuntimeError::InvalidRoleContext),
    }
}

fn resource_ref_from_request(request: &authz_engine::AuthzRequest) -> String {
    match request.resource_id.as_ref() {
        Some(resource_id) => format!("{}:{resource_id}", request.resource_type),
        None => request.resource_type.clone(),
    }
}

fn string_attributes_from_context(
    request: &authz_engine::AuthzRequest,
) -> BTreeMap<String, String> {
    request
        .context
        .iter()
        .filter_map(|(key, value)| match value {
            serde_json::Value::String(value) => Some((key.clone(), value.clone())),
            serde_json::Value::Bool(value) => Some((key.clone(), value.to_string())),
            serde_json::Value::Number(value) => Some((key.clone(), value.to_string())),
            _ => None,
        })
        .collect()
}

fn authz_decision_from_authorization(
    authorization: &AuthorizationDecision,
) -> authz_engine::AuthzDecision {
    match (authorization.allowed, authorization.matched_policy.clone()) {
        (true, Some(policy_id)) => authz_engine::AuthzDecision::allow(vec![policy_id]),
        (false, Some(policy_id)) => authz_engine::AuthzDecision::explicit_deny(vec![policy_id]),
        _ => authz_engine::AuthzDecision::default_deny(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authz_engine::{AuthzDecision, AuthzRequest, EvalLogFilter, PrincipalType};
    use serde_json::json;

    const POLICY_ID: &str = "pol_tenant_admin";
    const TEST_TENANT_ID: &str = "ten_alpha";
    const TEST_TENANT_RESOURCE: &str = "tenant:ten_alpha:settings";

    // ── existing tests ────────────────────────────────────────────────────────

    #[test]
    fn publish_accepts_global_and_tenant_scoped_semver_policy_versions() {
        let mut policies = PolicySet::default();

        let global = policies
            .publish(policy_version(
                "pol_global_reader",
                "1.0.0",
                PolicyScope::Global,
                None,
            ))
            .expect("global policy publishes");
        let tenant = policies
            .publish(policy_version(POLICY_ID, "1.0.0", tenant_scope(), None))
            .expect("tenant policy publishes");

        assert_eq!(global.scope, PolicyScope::Global);
        assert_eq!(
            tenant.scope,
            PolicyScope::Tenant(TEST_TENANT_ID.to_string())
        );
    }

    #[test]
    fn publish_rejects_non_semver_and_duplicate_policy_versions() {
        let mut policies = PolicySet::default();

        assert_eq!(
            policies.publish(policy_version(POLICY_ID, "01.0.0", tenant_scope(), None)),
            Err(PolicyError::InvalidSemver)
        );

        policies
            .publish(policy_version(POLICY_ID, "1.0.0", tenant_scope(), None))
            .expect("initial policy publishes");
        assert_eq!(
            policies.publish(policy_version(POLICY_ID, "1.0.0", tenant_scope(), None)),
            Err(PolicyError::VersionAlreadyExists)
        );
    }

    #[test]
    fn publish_enforces_supersession_chain_integrity() {
        let mut policies = PolicySet::default();
        policies
            .publish(policy_version(POLICY_ID, "1.0.0", tenant_scope(), None))
            .expect("initial policy publishes");
        policies
            .publish(policy_version(
                POLICY_ID,
                "1.1.0",
                tenant_scope(),
                Some("1.0.0"),
            ))
            .expect("newer policy can supersede older same-scope policy");

        let chain = policies
            .supersession_chain(POLICY_ID, "1.1.0")
            .expect("chain resolves");
        assert_eq!(
            chain
                .iter()
                .map(|policy| policy.version.as_str())
                .collect::<Vec<_>>(),
            vec!["1.1.0", "1.0.0"]
        );

        assert_eq!(
            policies.publish(policy_version(
                POLICY_ID,
                "1.2.0",
                PolicyScope::Global,
                Some("1.1.0")
            )),
            Err(PolicyError::SupersedesScopeMismatch)
        );
        assert_eq!(
            policies.publish(policy_version(
                POLICY_ID,
                "1.2.0",
                tenant_scope(),
                Some("2.0.0")
            )),
            Err(PolicyError::SupersedesNotOlder)
        );
        assert_eq!(
            policies.publish(policy_version(
                POLICY_ID,
                "1.2.0",
                tenant_scope(),
                Some("1.2.0")
            )),
            Err(PolicyError::SupersedesSelf)
        );
        assert_eq!(
            policies.publish(policy_version(
                POLICY_ID,
                "1.2.0",
                tenant_scope(),
                Some("1.0.1")
            )),
            Err(PolicyError::SupersedesMissing)
        );
    }

    #[test]
    fn authorization_uses_only_active_unsuperseded_policy_versions() {
        let mut policies = PolicySet::default();
        policies
            .publish(policy_version_with_effect(
                POLICY_ID,
                "1.0.0",
                tenant_scope(),
                None,
                PolicyEffect::Allow,
            ))
            .expect("initial allow policy publishes");
        policies
            .publish(policy_version_with_effect(
                POLICY_ID,
                "1.1.0",
                tenant_scope(),
                Some("1.0.0"),
                PolicyEffect::Deny,
            ))
            .expect("new deny policy supersedes old allow policy");

        let decision = policies.authorize(&AuthorizationQuery {
            subject: AuthorizationSubject {
                tenant_id: TEST_TENANT_ID.to_string(),
                roles: vec!["tenant-admin".to_string()],
            },
            action: "tenant.settings.update".to_string(),
            resource: TEST_TENANT_RESOURCE.to_string(),
            attributes: BTreeMap::new(),
        });

        assert!(!decision.allowed);
        assert_eq!(decision.matched_policy.as_deref(), Some(POLICY_ID));
    }

    // ── P1-fix synthetic violation tests ─────────────────────────────────────

    /// P1 PRRT_kwDOSbSl2s6CnhW0 / PRRT_kwDOSbSl2s6CnqTH:
    /// `PolicyEffect`, `AuthzDecision`, `AuthzRequest`, and `EvalLogFilter` must
    /// round-trip through JSON without compile or runtime errors.
    #[test]
    fn authz_engine_types_serialize_and_deserialize_via_serde_json() {
        let request = AuthzRequest {
            tenant_id: TEST_TENANT_ID.to_string(),
            principal_type: PrincipalType::User,
            principal_id: Some("usr_001".to_string()),
            action: "Read".to_string(),
            resource_type: "Object".to_string(),
            resource_id: Some("obj_abc".to_string()),
            context: BTreeMap::new(),
        };
        let json = serde_json::to_string(&request).expect("AuthzRequest serializes");
        let roundtrip: AuthzRequest =
            serde_json::from_str(&json).expect("AuthzRequest deserializes");
        assert_eq!(request, roundtrip);

        let decision = AuthzDecision::allow(vec!["pol_allow_read".to_string()]);
        let json = serde_json::to_string(&decision).expect("AuthzDecision serializes");
        let roundtrip: AuthzDecision =
            serde_json::from_str(&json).expect("AuthzDecision deserializes");
        assert_eq!(decision, roundtrip);
        assert!(roundtrip.is_allowed());

        let filter = EvalLogFilter::default();
        let json = serde_json::to_string(&filter).expect("EvalLogFilter serializes");
        let roundtrip: EvalLogFilter =
            serde_json::from_str(&json).expect("EvalLogFilter deserializes");
        assert_eq!(filter, roundtrip);
        assert_eq!(roundtrip.limit, 100);
    }

    /// P1 PRRT_kwDOSbSl2s6CnpDv:
    /// `PrincipalType` wire values must match Cedar PascalCase entity names, not
    /// snake_case. A payload carrying `"User"` must deserialize correctly, and
    /// the serialized form must equal `"User"` (not `"user"`).
    #[test]
    fn principal_type_serde_uses_cedar_pascalcase_wire_names() {
        // Serialize and check wire value is PascalCase.
        let serialized =
            serde_json::to_string(&PrincipalType::Employee).expect("PrincipalType serializes");
        assert_eq!(
            serialized, "\"Employee\"",
            "wire value must be Cedar PascalCase, got {serialized}"
        );

        // Deserialize Cedar-style value (as a downstream client would send it).
        let from_cedar: PrincipalType =
            serde_json::from_str("\"Workflow\"").expect("Cedar wire value deserializes");
        assert_eq!(from_cedar, PrincipalType::Workflow);

        // snake_case must NOT deserialize (would indicate wire-format mismatch).
        let snake_result = serde_json::from_str::<PrincipalType>("\"workflow\"");
        assert!(
            snake_result.is_err(),
            "snake_case wire value must be rejected to prevent silent Cedar mismatch"
        );
    }

    /// P1 PRRT_kwDOSbSl2s6CnhW0 (effect serde):
    /// `PolicyEffect` must serialize to UPPERCASE values so it is unambiguous on
    /// the wire and does not collide with Cedar reserved lowercase tokens.
    #[test]
    fn policy_effect_serde_uses_uppercase_wire_values() {
        let allow_json =
            serde_json::to_string(&PolicyEffect::Allow).expect("PolicyEffect::Allow serializes");
        assert_eq!(allow_json, "\"ALLOW\"");

        let deny_json =
            serde_json::to_string(&PolicyEffect::Deny).expect("PolicyEffect::Deny serializes");
        assert_eq!(deny_json, "\"DENY\"");

        let roundtrip: PolicyEffect =
            serde_json::from_str("\"ALLOW\"").expect("ALLOW deserializes");
        assert_eq!(roundtrip, PolicyEffect::Allow);
    }

    /// P1 PRRT_kwDOSbSl2s6CnoFA (audit-chain append-only):
    /// Synthetic violation: serializing a decision must not mutate state that
    /// could corrupt an append-only ledger entry if accidentally re-serialized.
    #[test]
    fn authz_decision_default_deny_is_immutable_across_serialization_roundtrip() {
        let d1 = AuthzDecision::default_deny();
        let json = serde_json::to_string(&d1).expect("default_deny serializes");
        let d2: AuthzDecision = serde_json::from_str(&json).expect("default_deny deserializes");
        assert_eq!(d1, d2);
        assert!(!d2.is_allowed());
        assert!(d2.determining_policies.is_empty());
    }

    /// P2 synthetic (EvalLogFilter default limit):
    /// `EvalLogFilter::default()` must yield `limit = 100`, not `0`.
    #[test]
    fn eval_log_filter_default_limit_is_100() {
        assert_eq!(EvalLogFilter::default().limit, 100);

        // Deserializing a payload that omits `limit` must also default to 100.
        let from_partial: EvalLogFilter =
            serde_json::from_str(r#"{"principal_id":null,"effect":null,"resource_type":null}"#)
                .expect("partial EvalLogFilter deserializes");
        assert_eq!(from_partial.limit, 100);
    }

    #[test]
    fn backbone_write_policy_pack_allows_every_implemented_write_action() {
        let mut evaluator =
            CedarRuntimeEvaluator::with_backbone_write_policies(TEST_TENANT_ID).unwrap();

        for operation in BackboneWriteOperation::all() {
            let evaluation = evaluator
                .evaluate(backbone_request(operation, TEST_TENANT_ID), "audit-cedar")
                .unwrap();

            assert!(evaluation.decision.is_allowed());
            assert!(
                evaluation
                    .decision_ref
                    .starts_with(&format!("cedar:allow:{}", operation.policy_id()))
            );
            assert_eq!(
                evaluation.decision.determining_policies,
                vec![operation.policy_id().to_string()]
            );
            assert_eq!(evaluation.log_entry.audit_correlation_id, "audit-cedar");
        }

        assert_eq!(evaluator.log_len(), BackboneWriteOperation::all().len());
    }

    #[test]
    fn backbone_write_policy_pack_denies_wrong_tenant_by_default() {
        let mut evaluator =
            CedarRuntimeEvaluator::with_backbone_write_policies(TEST_TENANT_ID).unwrap();
        let evaluation = evaluator
            .evaluate(
                backbone_request(BackboneWriteOperation::MessengerPostMessage, "ten_other"),
                "audit-cedar",
            )
            .unwrap();

        assert!(!evaluation.decision.is_allowed());
        assert_eq!(evaluation.decision_ref, "cedar:deny:default:1");
        assert!(evaluation.decision.determining_policies.is_empty());
        assert_eq!(evaluation.log_entry.reason, "no matching allow policy");
    }

    #[test]
    fn explicit_deny_policy_wins_over_backbone_allow() {
        let operation = BackboneWriteOperation::MailSubmitMessage;
        let mut versions = backbone_write_policy_versions(TEST_TENANT_ID);
        versions.push(PolicyVersion {
            policy_id: "pol_backbone_mail_submit_freeze".to_string(),
            version: "1.0.0".to_string(),
            scope: tenant_scope(),
            supersedes: None,
            rules: vec![PolicyRuleInput {
                effect: PolicyEffect::Deny,
                principal_role: operation.principal_role().to_string(),
                action: operation.action().to_string(),
                resource_prefix: operation.resource_prefix().to_string(),
                required_attribute: Some(("data_plane".to_string(), "backbone".to_string())),
            }],
        });
        let mut evaluator = CedarRuntimeEvaluator::from_policy_versions(versions).unwrap();

        let evaluation = evaluator
            .evaluate(backbone_request(operation, TEST_TENANT_ID), "audit-cedar")
            .unwrap();

        assert!(!evaluation.decision.is_allowed());
        assert_eq!(
            evaluation.decision.determining_policies,
            vec!["pol_backbone_mail_submit_freeze".to_string()]
        );
        assert_eq!(evaluation.log_entry.reason, "explicit deny policy");
    }

    #[test]
    fn eval_log_filter_selects_effect_principal_and_resource_type() {
        let mut evaluator =
            CedarRuntimeEvaluator::with_backbone_write_policies(TEST_TENANT_ID).unwrap();
        evaluator
            .evaluate(
                backbone_request(BackboneWriteOperation::CommunityCreatePost, TEST_TENANT_ID),
                "audit-1",
            )
            .unwrap();
        evaluator
            .evaluate(
                backbone_request(BackboneWriteOperation::CommunityCastVote, "ten_other"),
                "audit-2",
            )
            .unwrap();

        let allow_filter = EvalLogFilter {
            principal_id: Some("user:u".to_string()),
            effect: Some(PolicyEffect::Allow),
            resource_type: Some("community:space".to_string()),
            limit: 10,
        };
        let allow_entries = evaluator.eval_log(&allow_filter);
        assert_eq!(allow_entries.len(), 1);
        assert_eq!(
            allow_entries[0].determining_policies,
            vec!["pol_backbone_community_post_create".to_string()]
        );

        let deny_filter = EvalLogFilter {
            principal_id: None,
            effect: Some(PolicyEffect::Deny),
            resource_type: None,
            limit: 1,
        };
        let deny_entries = evaluator.eval_log(&deny_filter);
        assert_eq!(deny_entries.len(), 1);
        assert_eq!(deny_entries[0].decision_ref, "cedar:deny:default:2");
    }

    #[test]
    fn runtime_evaluator_rejects_missing_audit_and_invalid_role_context() {
        let mut evaluator =
            CedarRuntimeEvaluator::with_backbone_write_policies(TEST_TENANT_ID).unwrap();
        assert_eq!(
            evaluator.evaluate(
                backbone_request(BackboneWriteOperation::MessengerPostMessage, TEST_TENANT_ID),
                "",
            ),
            Err(CedarRuntimeError::MissingAuditCorrelation)
        );

        let mut request =
            backbone_request(BackboneWriteOperation::MessengerPostMessage, TEST_TENANT_ID);
        request
            .context
            .insert("roles".to_string(), json!({"not": "a-role"}));
        assert_eq!(
            evaluator.evaluate(request, "audit"),
            Err(CedarRuntimeError::InvalidRoleContext)
        );
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn tenant_scope() -> PolicyScope {
        PolicyScope::Tenant(TEST_TENANT_ID.to_string())
    }

    fn policy_version(
        policy_id: &str,
        version: &str,
        scope: PolicyScope,
        supersedes: Option<&str>,
    ) -> PolicyVersion {
        policy_version_with_effect(policy_id, version, scope, supersedes, PolicyEffect::Allow)
    }

    fn policy_version_with_effect(
        policy_id: &str,
        version: &str,
        scope: PolicyScope,
        supersedes: Option<&str>,
        effect: PolicyEffect,
    ) -> PolicyVersion {
        PolicyVersion {
            policy_id: policy_id.to_string(),
            version: version.to_string(),
            scope,
            supersedes: supersedes.map(str::to_string),
            rules: vec![PolicyRuleInput {
                effect,
                principal_role: "tenant-admin".to_string(),
                action: "tenant.settings.update".to_string(),
                resource_prefix: "tenant:".to_string(),
                required_attribute: None,
            }],
        }
    }

    fn backbone_request(operation: BackboneWriteOperation, tenant_id: &str) -> AuthzRequest {
        let mut context = BTreeMap::new();
        context.insert("roles".to_string(), json!([operation.principal_role()]));
        context.insert("data_plane".to_string(), json!("backbone"));
        AuthzRequest {
            tenant_id: tenant_id.to_string(),
            principal_type: PrincipalType::User,
            principal_id: Some("user:u".to_string()),
            action: operation.action().to_string(),
            resource_type: operation.resource_type().to_string(),
            resource_id: Some(sample_resource_id(operation).to_string()),
            context,
        }
    }

    fn sample_resource_id(operation: BackboneWriteOperation) -> &'static str {
        match operation {
            BackboneWriteOperation::MessengerPostMessage => "channel:c",
            BackboneWriteOperation::MailSubmitMessage => "mailbox:b",
            BackboneWriteOperation::SocialPublishPost => "profile:p",
            BackboneWriteOperation::CommunityCreatePost => "space:s",
            BackboneWriteOperation::CommunityCastVote
            | BackboneWriteOperation::CommunityApplyModerationAction => "post:p",
        }
    }
}
