//! Cedar-shaped authorization policy kernel.
//!
//! This is deliberately pure: it stores versioned policy records and evaluates
//! role + attribute predicates without network, storage, or runtime side effects.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

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

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY_ID: &str = "pol_tenant_admin";

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
            .publish(policy_version(
                POLICY_ID,
                "1.0.0",
                PolicyScope::Tenant("ten_kr".to_string()),
                None,
            ))
            .expect("tenant policy publishes");

        assert_eq!(global.scope, PolicyScope::Global);
        assert_eq!(tenant.scope, PolicyScope::Tenant("ten_kr".to_string()));
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
                tenant_id: "ten_kr".to_string(),
                roles: vec!["tenant-admin".to_string()],
            },
            action: "tenant.settings.update".to_string(),
            resource: "tenant:ten_kr:settings".to_string(),
            attributes: BTreeMap::new(),
        });

        assert!(!decision.allowed);
        assert_eq!(decision.matched_policy.as_deref(), Some(POLICY_ID));
    }

    fn tenant_scope() -> PolicyScope {
        PolicyScope::Tenant("ten_kr".to_string())
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
}
