//! Cedar-shaped authorization policy kernel.
//!
//! This is deliberately pure: it stores versioned policy records and evaluates
//! role + attribute predicates without network, storage, or runtime side effects.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PolicyScope {
    Global,
    Tenant(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PolicySet {
    policies: BTreeMap<(String, String), PublishedPolicy>,
}

impl PolicySet {
    pub fn publish(&mut self, version: PolicyVersion) -> Result<PublishedPolicy, PolicyError> {
        let key = (version.policy_id.clone(), version.version.clone());
        if self.policies.contains_key(&key) {
            return Err(PolicyError::VersionAlreadyExists);
        }
        validate_policy_id(&version.policy_id)?;
        validate_semver(&version.version)?;
        if version.rules.is_empty() {
            return Err(PolicyError::EmptyRules);
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

    pub fn authorize(&self, query: &AuthorizationQuery) -> AuthorizationDecision {
        let mut scoped_policies = self
            .policies
            .values()
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

fn validate_semver(version: &str) -> Result<(), PolicyError> {
    let parts = version.split('.').collect::<Vec<_>>();
    if parts.len() == 3 && parts.iter().all(|part| part.parse::<u64>().is_ok()) {
        Ok(())
    } else {
        Err(PolicyError::InvalidSemver)
    }
}
