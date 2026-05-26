//! Cedar authorization gate for workload identities.
//!
//! This adapter evaluates an [`AuthorizationRequest`] (a verified
//! [`WorkloadPrincipal`] plus a PARC action/resource/context) against a set of
//! policies and produces an [`AuthorizationDecision`]. It is the workload-side
//! authz gate referenced by `microservices/identity` IP-085 (principal +
//! authz-gate) and ADR-0002 (tenant+identity kernel).
//!
//! ## Cedar semantics implemented
//!
//! The evaluator implements Cedar's decision algorithm faithfully:
//! 1. **Deny by default.** With no matching policy, the request is denied.
//! 2. **Explicit permit.** A request is allowed only if at least one `permit`
//!    policy matches AND no `forbid` policy matches.
//! 3. **Forbid wins.** A matching `forbid` overrides any number of matching
//!    `permit`s. This is Cedar's defining safety property.
//! 4. **Lifecycle precondition.** A principal that is not in an operational
//!    state ([`WorkloadState::Active`]) is denied before any policy runs.
//!
//! A policy condition can constrain the principal's tenant, owning capability,
//! lifecycle state, held scopes, and individual claim values, plus the
//! requested action and resource type/id — the same surface a Cedar policy
//! expresses with `principal`, `action`, `resource`, and `when {{ ... }}`.
//!
//! ## Real-`cedar-policy` swap seam (documented, intentional)
//!
//! The upstream `cedar-policy` crate (Apache-2.0, OSI-clean, the engine named
//! in `microservices/identity/manifest.json#consumes_upstream_oss`) is the
//! production target. It is not vendored into this offline workspace, so this
//! adapter ships a faithful in-crate evaluator behind the
//! [`WorkloadAuthorizer`] trait. Swapping in `cedar-policy` is a drop-in: add
//! the dependency, implement [`WorkloadAuthorizer`] over a
//! `cedar_policy::Authorizer` + `PolicySet`, and translate
//! [`AuthorizationRequest`] into a `cedar_policy::Request` + `Entities`. The
//! domain types and this crate's public trait do not change. See
//! `registry/catalog/oya-identity-workload-authz-cedar-adapter.yaml`.

// ADR-0083 Tier 3: production code stays panic-free; tests may use unwrap/expect.
#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use oya_identity_workload_domain::{
    Action, AuthorizationDecision, AuthorizationRequest, ClaimValue, Resource, WorkloadPrincipal,
};

/// A condition a policy places on the *principal* of a request. All conditions
/// in a policy must hold (logical AND) for the policy to match.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrincipalCondition {
    /// Principal must belong to this tenant (`ten_<slug>`).
    TenantIs(String),
    /// Principal's owning capability must equal this (`cap.<dotted>`).
    OwningCapabilityIs(String),
    /// Principal must currently hold this scope.
    HasScope(String),
    /// Principal must carry a claim whose value contains the needle (text
    /// equality, list membership, or `Bool(true)` when needle is `"true"`).
    ClaimContains {
        /// Claim name.
        claim: String,
        /// Needle the claim value must contain.
        needle: String,
    },
}

impl PrincipalCondition {
    fn holds_for(&self, principal: &WorkloadPrincipal) -> bool {
        match self {
            Self::TenantIs(tenant) => principal.tenant_id().as_str() == tenant,
            Self::OwningCapabilityIs(capability) => {
                principal.owning_capability().as_str() == capability
            }
            Self::HasScope(scope) => principal.has_scope(scope),
            Self::ClaimContains { claim, needle } => principal
                .claim(claim)
                .map(|value| claim_matches(value, needle))
                .unwrap_or(false),
        }
    }
}

/// A condition a policy places on the requested *action*.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionCondition {
    /// Action must equal this exact string.
    Equals(String),
    /// Action is unconstrained (matches anything).
    Any,
}

impl ActionCondition {
    fn holds_for(&self, action: &Action) -> bool {
        match self {
            Self::Equals(expected) => action.as_str() == expected,
            Self::Any => true,
        }
    }
}

/// A condition a policy places on the targeted *resource*.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceCondition {
    /// Resource type must equal this.
    TypeIs(String),
    /// Resource type and id must both equal these.
    Is {
        /// Required resource type.
        resource_type: String,
        /// Required resource id.
        resource_id: String,
    },
    /// Resource is unconstrained (matches anything).
    Any,
}

impl ResourceCondition {
    fn holds_for(&self, resource: &Resource) -> bool {
        match self {
            Self::TypeIs(resource_type) => resource.resource_type() == resource_type,
            Self::Is {
                resource_type,
                resource_id,
            } => resource.resource_type() == resource_type && resource.resource_id() == resource_id,
            Self::Any => true,
        }
    }
}

/// The effect a policy produces when it matches: Cedar's `permit` / `forbid`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyEffect {
    /// Matching contributes an allow (subject to forbid-wins).
    Permit,
    /// Matching forces a deny that overrides every permit.
    Forbid,
}

/// A single workload-authz policy in PARC form. Conceptually:
///
/// ```text
/// <effect>(principal, action, resource)
/// when { <all principal/action/resource conditions hold> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Policy {
    /// Stable identifier surfaced in the decision reason / audit chain.
    pub id: String,
    /// Permit or forbid.
    pub effect: PolicyEffect,
    /// Conditions on the principal (all must hold).
    pub principal: Vec<PrincipalCondition>,
    /// Condition on the action.
    pub action: ActionCondition,
    /// Condition on the resource.
    pub resource: ResourceCondition,
}

impl Policy {
    /// Construct a `permit` policy with the given id and unconstrained
    /// action/resource. Builder methods narrow it.
    #[must_use]
    pub fn permit(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            effect: PolicyEffect::Permit,
            principal: Vec::new(),
            action: ActionCondition::Any,
            resource: ResourceCondition::Any,
        }
    }

    /// Construct a `forbid` policy with the given id and unconstrained
    /// action/resource.
    #[must_use]
    pub fn forbid(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            effect: PolicyEffect::Forbid,
            principal: Vec::new(),
            action: ActionCondition::Any,
            resource: ResourceCondition::Any,
        }
    }

    /// Add a principal condition (builder).
    #[must_use]
    pub fn when_principal(mut self, condition: PrincipalCondition) -> Self {
        self.principal.push(condition);
        self
    }

    /// Set the action condition (builder).
    #[must_use]
    pub fn for_action(mut self, condition: ActionCondition) -> Self {
        self.action = condition;
        self
    }

    /// Set the resource condition (builder).
    #[must_use]
    pub fn for_resource(mut self, condition: ResourceCondition) -> Self {
        self.resource = condition;
        self
    }

    fn matches(&self, request: &AuthorizationRequest) -> bool {
        self.action.holds_for(&request.action)
            && self.resource.holds_for(&request.resource)
            && self
                .principal
                .iter()
                .all(|condition| condition.holds_for(&request.principal))
    }
}

/// The abstraction the rest of the system depends on. Swapping the in-crate
/// evaluator for the real `cedar-policy` engine means implementing this trait
/// over a `cedar_policy::Authorizer`; callers do not change.
pub trait WorkloadAuthorizer {
    /// Evaluate the request and return a decision (always total; deny-by-default).
    fn authorize(&self, request: &AuthorizationRequest) -> AuthorizationDecision;
}

/// A policy set evaluated with Cedar semantics (deny-by-default, forbid-wins).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CedarWorkloadAuthorizer {
    policies: Vec<Policy>,
}

impl CedarWorkloadAuthorizer {
    /// Build an empty authorizer (denies everything until policies are added).
    #[must_use]
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    /// Build from a policy set.
    #[must_use]
    pub fn with_policies(policies: Vec<Policy>) -> Self {
        Self { policies }
    }

    /// Add a policy (builder).
    #[must_use]
    pub fn add_policy(mut self, policy: Policy) -> Self {
        self.policies.push(policy);
        self
    }

    /// Number of policies in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.policies.len()
    }

    /// Whether the policy set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }
}

impl WorkloadAuthorizer for CedarWorkloadAuthorizer {
    fn authorize(&self, request: &AuthorizationRequest) -> AuthorizationDecision {
        // Lifecycle precondition: a non-operational principal cannot be
        // authorized regardless of policy. This is evaluated before policies so
        // a suspended/retired workload is denied even if a stale permit exists.
        if !request.principal.state().is_operational() {
            return AuthorizationDecision::principal_not_operational(request.principal.state());
        }

        // Forbid-wins: a single matching forbid denies the request outright.
        if let Some(forbid) = self
            .policies
            .iter()
            .find(|policy| policy.effect == PolicyEffect::Forbid && policy.matches(request))
        {
            return AuthorizationDecision::forbid(&forbid.id);
        }

        // Otherwise allow iff some permit matches.
        if let Some(permit) = self
            .policies
            .iter()
            .find(|policy| policy.effect == PolicyEffect::Permit && policy.matches(request))
        {
            return AuthorizationDecision::permit(&permit.id);
        }

        // Deny by default.
        AuthorizationDecision::default_deny()
    }
}

fn claim_matches(value: &ClaimValue, needle: &str) -> bool {
    if value.contains(needle) {
        return true;
    }
    // Allow `ClaimContains { needle: "true" }` to match a `Bool(true)` claim,
    // mirroring how a Cedar policy would test a boolean context attribute.
    matches!((value, needle), (ClaimValue::Bool(true), "true"))
        || matches!((value, needle), (ClaimValue::Bool(false), "false"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_identity_workload_domain::{Effect, WorkloadState};

    fn active_principal() -> WorkloadPrincipal {
        let mut principal =
            WorkloadPrincipal::provision("ten_acme", "wl_deployer", "cap.cloud.deploy")
                .expect("valid");
        principal.transition_to(WorkloadState::Active).expect("activate");
        principal
            .with_scope("cloud.deploy.write")
            .expect("scope ok")
            .with_claim("env", ClaimValue::Text("prod".into()))
            .expect("claim ok")
    }

    fn deploy_request(principal: WorkloadPrincipal) -> AuthorizationRequest {
        AuthorizationRequest::new(
            principal,
            Action::new("cloud.deploy.Apply"),
            Resource::new("Deployment", "checkout-svc"),
        )
    }

    #[test]
    fn empty_policy_set_denies_by_default() {
        let authorizer = CedarWorkloadAuthorizer::new();
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(authorizer.is_empty());
    }

    #[test]
    fn matching_permit_allows() {
        let authorizer = CedarWorkloadAuthorizer::new().add_policy(
            Policy::permit("allow-acme-deploy")
                .when_principal(PrincipalCondition::TenantIs("ten_acme".into()))
                .when_principal(PrincipalCondition::HasScope("cloud.deploy.write".into()))
                .for_action(ActionCondition::Equals("cloud.deploy.Apply".into()))
                .for_resource(ResourceCondition::TypeIs("Deployment".into())),
        );
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert!(decision.is_allow());
    }

    #[test]
    fn forbid_overrides_permit() {
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("allow-acme-deploy")
                    .when_principal(PrincipalCondition::TenantIs("ten_acme".into())),
            )
            .add_policy(
                // Break-glass freeze: forbid all writes to checkout-svc.
                Policy::forbid("freeze-checkout").for_resource(ResourceCondition::Is {
                    resource_type: "Deployment".into(),
                    resource_id: "checkout-svc".into(),
                }),
            );
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert_eq!(decision.effect(), Effect::Deny);
    }

    #[test]
    fn permit_for_other_tenant_does_not_match() {
        let authorizer = CedarWorkloadAuthorizer::new().add_policy(
            Policy::permit("allow-globex")
                .when_principal(PrincipalCondition::TenantIs("ten_globex".into())),
        );
        // active_principal() is ten_acme; cross-tenant permit must not apply.
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert_eq!(decision.effect(), Effect::Deny);
    }

    #[test]
    fn missing_scope_denies_even_with_tenant_match() {
        let authorizer = CedarWorkloadAuthorizer::new().add_policy(
            Policy::permit("needs-admin-scope")
                .when_principal(PrincipalCondition::TenantIs("ten_acme".into()))
                .when_principal(PrincipalCondition::HasScope("cloud.deploy.admin".into())),
        );
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert_eq!(decision.effect(), Effect::Deny);
    }

    #[test]
    fn claim_condition_matches_text_and_bool() {
        let principal = active_principal()
            .with_claim("mfa", ClaimValue::Bool(true))
            .expect("claim ok");
        let authorizer = CedarWorkloadAuthorizer::new().add_policy(
            Policy::permit("prod-mfa")
                .when_principal(PrincipalCondition::ClaimContains {
                    claim: "env".into(),
                    needle: "prod".into(),
                })
                .when_principal(PrincipalCondition::ClaimContains {
                    claim: "mfa".into(),
                    needle: "true".into(),
                }),
        );
        assert!(authorizer.authorize(&deploy_request(principal)).is_allow());
    }

    #[test]
    fn suspended_principal_denied_before_policies() {
        let mut principal = active_principal();
        principal.transition_to(WorkloadState::Suspended).expect("suspend");
        // A permit that WOULD match if active.
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(Policy::permit("allow-all-acme")
                .when_principal(PrincipalCondition::TenantIs("ten_acme".into())));
        let decision = authorizer.authorize(&deploy_request(principal));
        assert!(matches!(
            decision.reason(),
            oya_identity_workload_domain::DecisionReason::PrincipalNotOperational { .. }
        ));
    }
}
