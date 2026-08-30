//! The published-policy store and its evaluation entry point.

use std::collections::BTreeMap;

use crate::authorization::{AuthorizationDecision, AuthorizationQuery, PolicyError};
use crate::policy::{PolicyEffect, PolicyRule, PolicyScope, PolicyVersion, PublishedPolicy};
use crate::policy::{parse_semver, validate_policy_id};

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
                    // Forbid-wins: Deny suppresses all annotations.
                    return AuthorizationDecision {
                        allowed: false,
                        reason: "explicit deny policy".to_string(),
                        matched_policy: Some(policy.policy_id.clone()),
                        annotations: Vec::new(),
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
                        annotations: rule.annotations.clone(),
                    };
                }
            }
        }
        AuthorizationDecision {
            allowed: false,
            reason: "no matching allow policy".to_string(),
            matched_policy: None,
            annotations: Vec::new(),
        }
    }
}
