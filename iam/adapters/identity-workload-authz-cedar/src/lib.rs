//! Cedar authorization gate for workload identities.
//!
//! This adapter evaluates an [`AuthorizationRequest`] (a verified
//! [`WorkloadPrincipal`] plus a PARC action/resource/context) against a set of
//! policies and produces an [`AuthorizationDecision`]. It is the workload-side
//! authz gate referenced by `microservices/identity` IP-085 (principal +
//! authz-gate) and ADR-0002 (tenant+identity kernel).
//!
//! ## Real Cedar engine (ADR-0183 Cedar app-authz)
//!
//! Authorization is delegated to the upstream **`cedar-policy`** crate
//! (AWS, Apache-2.0): the same formally-verified engine named in
//! `iam/identity/manifest.json#consumes_upstream_oss` and exercised
//! by `iam/identity/policy/identity.cedar`. There is no hand-rolled
//! decision algorithm here — Cedar's own [`cedar_policy::Authorizer`] decides.
//!
//! Cedar's properties (arXiv 2403.04651) hold natively and we rely on them:
//! 1. **Deny by default.** With no matching `permit`, the request is denied.
//! 2. **Explicit permit.** Allowed only when a `permit` matches AND no `forbid`
//!    matches.
//! 3. **Forbid wins.** A matching `forbid` overrides every `permit`.
//! 4. **Order independence.** The result does not depend on policy order.
//!
//! A non-operational principal ([`WorkloadState::Active`] is the only
//! operational state) is short-circuited to a deny *before* the engine runs:
//! a suspended/retired workload can never be authorized, mirroring the global
//! `forbid (... ) when { principal.state != "active" }` guardrail in
//! `identity.cedar` and giving callers a distinct
//! [`DecisionReason::PrincipalNotOperational`] reason for the audit chain.
//!
//! ## Two ways to build the authorizer
//!
//! - [`CedarWorkloadAuthorizer::from_cedar_policies`] parses **raw Cedar policy
//!   text** (e.g. the contents of `identity.cedar`) directly into a
//!   `cedar_policy::PolicySet`. This is the production path.
//! - The structured [`Policy`] builder ([`Policy::permit`] / [`Policy::forbid`]
//!   plus the `*Condition` enums) is compiled to equivalent Cedar policy text
//!   and handed to the same engine. This keeps test authoring ergonomic and the
//!   public API stable while still exercising the real Cedar evaluator.
//!
//! See `registry/catalog/oya-identity-workload-authz-cedar-adapter.yaml`.

// ADR-0083 Tier 3: production code stays panic-free; tests may use unwrap/expect.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;
use std::fmt::Write as _;

use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityUid, Policy as CedarPolicy, PolicyId,
    PolicySet, Request, RestrictedExpression,
};
use iam_identity_workload_domain::{
    Action, AuthorizationDecision, AuthorizationRequest, ClaimValue, Resource, WorkloadPrincipal,
};

/// The Cedar entity type used for a workload principal. The bundled
/// `identity.cedar` namespaces this as `Oya::WorkloadPrincipal`; for the
/// structured-builder compilation path we use the unqualified type so the
/// generated policy text and the entity uids line up without a schema.
const PRINCIPAL_ENTITY_TYPE: &str = "Workload";

/// Errors raised while preparing or evaluating a Cedar authorization. The
/// engine itself is total once a request is built; these cover the fallible
/// translation/parse boundary so the request path stays panic-free
/// (ADR-0083 Tier 3 — no `unwrap`/`expect`/`panic`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CedarAuthzError {
    /// A Cedar policy (raw text or compiled from a [`Policy`]) failed to parse.
    PolicyParse(String),
    /// Building the Cedar entity store from the principal/resource failed.
    EntityBuild(String),
    /// Constructing the Cedar [`cedar_policy::Request`] failed.
    RequestBuild(String),
    /// A value (id, claim, scope) could not be represented in Cedar.
    InvalidValue(String),
}

impl fmt::Display for CedarAuthzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PolicyParse(detail) => write!(f, "cedar policy parse failed: {detail}"),
            Self::EntityBuild(detail) => write!(f, "cedar entity build failed: {detail}"),
            Self::RequestBuild(detail) => write!(f, "cedar request build failed: {detail}"),
            Self::InvalidValue(detail) => write!(f, "value not representable in cedar: {detail}"),
        }
    }
}

impl std::error::Error for CedarAuthzError {}

/// A condition a policy places on the *principal* of a request. All conditions
/// in a policy must hold (logical AND) for the policy to match. Each compiles
/// to a Cedar `when { ... }` sub-expression over the principal entity.
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
    /// Emit the Cedar `when`-clause fragment for this condition. `has` guards
    /// keep evaluation total when an attribute is absent.
    fn to_cedar_clause(&self) -> Result<String, CedarAuthzError> {
        match self {
            Self::TenantIs(tenant) => {
                Ok(format!("principal.tenant_id == {}", cedar_string(tenant)?))
            }
            Self::OwningCapabilityIs(capability) => Ok(format!(
                "principal.owning_capability == {}",
                cedar_string(capability)?
            )),
            Self::HasScope(scope) => Ok(format!(
                "principal.scopes.contains({})",
                cedar_string(scope)?
            )),
            Self::ClaimContains { claim, needle } => {
                let attr = claim_attr_name(claim);
                // Every claim is projected to a Cedar `Set` of strings (see
                // `claim_set_attribute`): a text/bool claim becomes a singleton
                // set, a list claim keeps its elements. So membership is the
                // single, unambiguous `.contains` operator — Cedar's `in` is
                // entity-hierarchy membership, NOT set membership, and would
                // type-error here. `has` guards the absent-attribute case so
                // evaluation stays total (no error -> no silent deny surprise).
                Ok(format!(
                    "(principal has {attr} && principal.{attr}.contains({needle}))",
                    attr = attr,
                    needle = cedar_string(needle)?
                ))
            }
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
    /// Cedar policy-head scope for the action.
    fn to_cedar_head(&self) -> Result<String, CedarAuthzError> {
        match self {
            Self::Equals(expected) => Ok(format!("action == Action::{}", cedar_string(expected)?)),
            Self::Any => Ok("action".to_string()),
        }
    }
}

/// A condition a policy places on the targeted *resource*.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceCondition {
    /// Resource type must equal this.
    TypeIs(String),
    /// Resource type must equal this and `resource.tenant_id` must match
    /// `principal.tenant_id` at request evaluation time.
    SameTenantAsPrincipal {
        /// Required resource type.
        resource_type: String,
    },
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
    /// Cedar policy-head scope for the resource.
    fn to_cedar_head(&self) -> Result<String, CedarAuthzError> {
        match self {
            Self::TypeIs(resource_type) => {
                Ok(format!("resource is {}", cedar_type_name(resource_type)?))
            }
            Self::SameTenantAsPrincipal { resource_type } => {
                Ok(format!("resource is {}", cedar_type_name(resource_type)?))
            }
            Self::Is {
                resource_type,
                resource_id,
            } => Ok(format!(
                "resource == {}::{}",
                cedar_type_name(resource_type)?,
                cedar_string(resource_id)?
            )),
            Self::Any => Ok("resource".to_string()),
        }
    }

    /// Cedar `when`-clause fragment for resource conditions that need request
    /// attributes rather than policy-head matching.
    fn to_cedar_clause(&self) -> Option<String> {
        match self {
            Self::SameTenantAsPrincipal { .. } => Some(
                "(principal has tenant_id && resource has tenant_id && \
                 principal.tenant_id == resource.tenant_id)"
                    .to_string(),
            ),
            Self::TypeIs(_) | Self::Is { .. } | Self::Any => None,
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
///
/// It is compiled to equivalent Cedar policy text and evaluated by the real
/// `cedar-policy` engine — there is no separate in-crate matcher.
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

    /// Compile this structured policy into a single Cedar policy statement.
    /// The caller's human id is emitted as the `@id(...)` annotation so it
    /// surfaces in the decision reason; the engine-internal id that guarantees
    /// set uniqueness is assigned separately at insert time via `new_id`.
    fn to_cedar_text(&self) -> Result<String, CedarAuthzError> {
        let effect = match self.effect {
            PolicyEffect::Permit => "permit",
            PolicyEffect::Forbid => "forbid",
        };
        let principal_head = format!("principal is {PRINCIPAL_ENTITY_TYPE}");
        let action_head = self.action.to_cedar_head()?;
        let resource_head = self.resource.to_cedar_head()?;

        let mut text = String::new();
        // @id annotation is echoed back by the engine in diagnostics().reason().
        writeln!(text, "@id({})", cedar_string(&self.id)?)
            .map_err(|e| CedarAuthzError::PolicyParse(e.to_string()))?;
        writeln!(text, "{effect} (").map_err(|e| CedarAuthzError::PolicyParse(e.to_string()))?;
        writeln!(text, "  {principal_head},")
            .map_err(|e| CedarAuthzError::PolicyParse(e.to_string()))?;
        writeln!(text, "  {action_head},")
            .map_err(|e| CedarAuthzError::PolicyParse(e.to_string()))?;
        writeln!(text, "  {resource_head}")
            .map_err(|e| CedarAuthzError::PolicyParse(e.to_string()))?;
        write!(text, ")").map_err(|e| CedarAuthzError::PolicyParse(e.to_string()))?;

        let mut clauses: Vec<String> = self
            .principal
            .iter()
            .map(PrincipalCondition::to_cedar_clause)
            .collect::<Result<_, _>>()?;
        if let Some(resource_clause) = self.resource.to_cedar_clause() {
            clauses.push(resource_clause);
        }

        if clauses.is_empty() {
            write!(text, ";").map_err(|e| CedarAuthzError::PolicyParse(e.to_string()))?;
        } else {
            write!(text, "\nwhen {{ {} }};", clauses.join(" && "))
                .map_err(|e| CedarAuthzError::PolicyParse(e.to_string()))?;
        }
        Ok(text)
    }
}

/// The abstraction the rest of the system depends on. Backed by the real
/// `cedar-policy` engine; callers only see [`AuthorizationDecision`].
pub trait WorkloadAuthorizer {
    /// Evaluate the request and return a decision (always total; deny-by-default).
    fn authorize(&self, request: &AuthorizationRequest) -> AuthorizationDecision;
}

/// A Cedar-backed workload authorizer. Holds a parsed `cedar_policy::PolicySet`
/// and evaluates every request with the real `cedar_policy::Authorizer`
/// (deny-by-default, forbid-overrides-permit, order-independent — Cedar's own
/// semantics, not re-implemented here).
#[derive(Clone, Debug)]
pub struct CedarWorkloadAuthorizer {
    policy_set: PolicySet,
    authorizer_policy_count: usize,
}

impl Default for CedarWorkloadAuthorizer {
    fn default() -> Self {
        Self {
            policy_set: PolicySet::new(),
            authorizer_policy_count: 0,
        }
    }
}

impl CedarWorkloadAuthorizer {
    /// Build an empty authorizer (Cedar denies everything by default until
    /// policies are added).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build from raw Cedar policy text (the production path — e.g. the
    /// contents of `iam/identity/policy/identity.cedar`).
    ///
    /// # Errors
    /// Returns [`CedarAuthzError::PolicyParse`] if the text is not a valid
    /// Cedar policy set.
    pub fn from_cedar_policies(policy_text: &str) -> Result<Self, CedarAuthzError> {
        let policy_set: PolicySet = policy_text
            .parse()
            .map_err(|e: cedar_policy::ParseErrors| CedarAuthzError::PolicyParse(e.to_string()))?;
        let count = policy_set.policies().count();
        Ok(Self {
            policy_set,
            authorizer_policy_count: count,
        })
    }

    /// Build from a structured [`Policy`] set. Each policy is compiled to Cedar
    /// text and parsed into the real engine's `PolicySet`.
    ///
    /// # Errors
    /// Returns [`CedarAuthzError`] if any policy cannot be compiled or parsed.
    pub fn with_policies(policies: Vec<Policy>) -> Result<Self, CedarAuthzError> {
        let mut policy_set = PolicySet::new();
        for (slot, policy) in policies.iter().enumerate() {
            let text = policy.to_cedar_text()?;
            let parsed: CedarPolicy = text.parse().map_err(|e: cedar_policy::ParseErrors| {
                CedarAuthzError::PolicyParse(format!("policy '{}': {e}", policy.id))
            })?;
            // Re-id with a stable slot so duplicate human ids never collide in
            // the set; the original id is preserved in the @id annotation and
            // surfaced via the policy annotation on a hit.
            let parsed = parsed.new_id(stable_policy_id(slot));
            policy_set.add(parsed).map_err(|e| {
                CedarAuthzError::PolicyParse(format!("policy '{}': {e}", policy.id))
            })?;
        }
        let count = policy_set.policies().count();
        Ok(Self {
            policy_set,
            authorizer_policy_count: count,
        })
    }

    /// Add a structured policy (builder). Compiles + parses it into the set.
    ///
    /// # Errors
    /// Returns [`CedarAuthzError`] if the policy cannot be compiled or parsed.
    pub fn add_policy(mut self, policy: Policy) -> Result<Self, CedarAuthzError> {
        let slot = self.authorizer_policy_count;
        let text = policy.to_cedar_text()?;
        let parsed: CedarPolicy = text
            .parse::<CedarPolicy>()
            .map_err(|e: cedar_policy::ParseErrors| {
                CedarAuthzError::PolicyParse(format!("policy '{}': {e}", policy.id))
            })?
            .new_id(stable_policy_id(slot));
        self.policy_set
            .add(parsed)
            .map_err(|e| CedarAuthzError::PolicyParse(format!("policy '{}': {e}", policy.id)))?;
        self.authorizer_policy_count = self.policy_set.policies().count();
        Ok(self)
    }

    /// Number of policies in the set.
    #[must_use]
    pub fn len(&self) -> usize {
        self.authorizer_policy_count
    }

    /// Whether the policy set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.authorizer_policy_count == 0
    }

    /// Fallible evaluation. Translates the request into Cedar values, runs the
    /// real engine, and maps the Cedar [`Decision`] back to an
    /// [`AuthorizationDecision`]. Used by the infallible
    /// [`WorkloadAuthorizer::authorize`] trait method, which converts any
    /// translation error into a fail-closed default-deny.
    ///
    /// # Errors
    /// Returns [`CedarAuthzError`] only when the request itself cannot be
    /// represented in Cedar (malformed ids etc.); a well-formed request that
    /// the policies simply do not permit yields `Ok(default_deny())`.
    pub fn try_authorize(
        &self,
        request: &AuthorizationRequest,
    ) -> Result<AuthorizationDecision, CedarAuthzError> {
        // Lifecycle precondition: a non-operational principal cannot be
        // authorized regardless of policy. Short-circuited before the engine so
        // a suspended/retired workload yields a distinct, audit-legible reason.
        if !request.principal.state().is_operational() {
            return Ok(AuthorizationDecision::principal_not_operational(
                request.principal.state(),
            ));
        }

        let principal_entity = principal_entity(&request.principal)?;
        let resource_entity = resource_entity(&request.resource)?;
        let principal_uid = principal_entity.uid();
        let resource_uid = resource_entity.uid();
        let action_uid = action_uid(&request.action)?;
        let context = request_context(&request.context)?;

        let entities = Entities::empty()
            .add_entities([principal_entity, resource_entity], None)
            .map_err(|e| CedarAuthzError::EntityBuild(e.to_string()))?;

        let cedar_request = Request::new(principal_uid, action_uid, resource_uid, context, None)
            .map_err(|e| CedarAuthzError::RequestBuild(e.to_string()))?;

        let authorizer = Authorizer::new();
        let response = authorizer.is_authorized(&cedar_request, &self.policy_set, &entities);

        Ok(self.map_decision(&response))
    }

    /// Map a Cedar [`cedar_policy::Response`] onto an [`AuthorizationDecision`],
    /// preserving the determining policy id (the original `@id` annotation when
    /// present) for the audit chain.
    fn map_decision(&self, response: &cedar_policy::Response) -> AuthorizationDecision {
        match response.decision() {
            Decision::Allow => {
                let policy_id = self.first_reason_label(response.diagnostics().reason());
                AuthorizationDecision::permit(policy_id)
            }
            Decision::Deny => {
                // Cedar denies either because a forbid fired or because nothing
                // permitted. If the reason set is non-empty on a Deny, a forbid
                // determined it (only forbids contribute to a Deny reason set).
                let mut reasons = response.diagnostics().reason();
                if let Some(first) = reasons.next() {
                    let label = self.reason_label(first);
                    AuthorizationDecision::forbid(label)
                } else {
                    AuthorizationDecision::default_deny()
                }
            }
        }
    }

    /// Resolve the first determining policy's human-facing id label.
    fn first_reason_label<'a>(&self, mut reasons: impl Iterator<Item = &'a PolicyId>) -> String {
        match reasons.next() {
            Some(id) => self.reason_label(id),
            None => "permit".to_string(),
        }
    }

    /// Prefer the policy's `@id` annotation (the caller-supplied id) over the
    /// engine-internal slot id when reporting which policy decided.
    fn reason_label(&self, policy_id: &PolicyId) -> String {
        if let Some(policy) = self.policy_set.policy(policy_id)
            && let Some(annotated) = policy.annotation("id")
        {
            return annotated.to_string();
        }
        policy_id.to_string()
    }
}

impl WorkloadAuthorizer for CedarWorkloadAuthorizer {
    fn authorize(&self, request: &AuthorizationRequest) -> AuthorizationDecision {
        // Fail closed: a request that cannot be represented in Cedar is denied,
        // never allowed and never panicked (ADR-0083 Tier 3).
        self.try_authorize(request)
            .unwrap_or_else(|_| AuthorizationDecision::default_deny())
    }
}

// ---- Cedar value construction (request path; panic-free) -------------------

/// Build the Cedar principal entity carrying the workload's authz-relevant
/// attributes (tenant, owning capability, lifecycle state, scopes, claims).
fn principal_entity(principal: &WorkloadPrincipal) -> Result<Entity, CedarAuthzError> {
    let uid = entity_uid(PRINCIPAL_ENTITY_TYPE, principal.workload_id().as_str())?;

    let mut attrs = std::collections::HashMap::new();
    attrs.insert(
        "tenant_id".to_string(),
        RestrictedExpression::new_string(principal.tenant_id().as_str().to_string()),
    );
    attrs.insert(
        "owning_capability".to_string(),
        RestrictedExpression::new_string(principal.owning_capability().as_str().to_string()),
    );
    attrs.insert(
        "state".to_string(),
        RestrictedExpression::new_string(state_label(principal).to_string()),
    );
    let scopes = principal
        .scopes()
        .iter()
        .map(|s| RestrictedExpression::new_string(s.clone()));
    attrs.insert("scopes".to_string(), RestrictedExpression::new_set(scopes));

    for (name, value) in principal.claims() {
        attrs.insert(claim_attr_name(name), claim_set_attribute(value));
    }

    Entity::new(uid, attrs, std::collections::HashSet::new())
        .map_err(|e| CedarAuthzError::EntityBuild(e.to_string()))
}

/// Build the Cedar resource entity (type + id + typed attributes).
fn resource_entity(resource: &Resource) -> Result<Entity, CedarAuthzError> {
    let uid = entity_uid(
        &sanitize_type_name(resource.resource_type())?,
        resource.resource_id(),
    )?;
    let attrs = resource
        .attributes()
        .iter()
        .map(|(key, value)| (key.clone(), policy_scalar_attribute(value)))
        .collect();
    Entity::new(uid, attrs, std::collections::HashSet::new())
        .map_err(|e| CedarAuthzError::EntityBuild(e.to_string()))
}

/// The `Action::"..."` uid for the requested action.
fn action_uid(action: &Action) -> Result<EntityUid, CedarAuthzError> {
    entity_uid("Action", action.as_str())
}

/// Build a Cedar [`Context`] from the request's typed context attributes.
fn request_context(
    context: &std::collections::BTreeMap<String, ClaimValue>,
) -> Result<Context, CedarAuthzError> {
    let pairs = context
        .iter()
        .map(|(key, value)| (key.clone(), policy_scalar_attribute(value)));
    Context::from_pairs(pairs).map_err(|e| CedarAuthzError::RequestBuild(e.to_string()))
}

/// Parse a `Type::"id"` Cedar entity uid from a type name and an id, escaping
/// the id so arbitrary ids (with quotes etc.) are represented safely.
fn entity_uid(type_name: &str, id: &str) -> Result<EntityUid, CedarAuthzError> {
    let literal = format!("{type_name}::{}", cedar_string(id)?);
    literal
        .parse()
        .map_err(|e: cedar_policy::ParseErrors| CedarAuthzError::InvalidValue(e.to_string()))
}

/// Project a principal [`ClaimValue`] onto a Cedar **Set of strings**, so the
/// authz layer tests every claim with the single `.contains` operator
/// regardless of the underlying shape:
/// - `Text("prod")`   -> `["prod"]`              (matches needle "prod")
/// - `Bool(true)`     -> `["true"]`              (matches needle "true")
/// - `Int(7)`         -> `["7"]`                 (matches needle "7")
/// - `TextList([..])` -> the list itself
///
/// This mirrors the domain's own [`ClaimValue::contains`] semantics (text
/// equality OR list membership) while keeping Cedar evaluation type-stable.
fn claim_set_attribute(value: &ClaimValue) -> RestrictedExpression {
    let members: Vec<String> = match value {
        ClaimValue::Text(text) => vec![text.clone()],
        ClaimValue::Bool(flag) => vec![if *flag { "true" } else { "false" }.to_string()],
        ClaimValue::Int(int) => vec![int.to_string()],
        ClaimValue::TextList(items) => items.clone(),
    };
    RestrictedExpression::new_set(members.into_iter().map(RestrictedExpression::new_string))
}

/// Project a policy-visible [`ClaimValue`] onto its natural Cedar scalar so
/// raw/structured Cedar policies can test it with the idiomatic operator
/// (`context.mfa_present == "true"`, `context.tier > 2`, list membership).
/// Unlike principal claims, context/resource attributes are read by Cedar
/// policy text, so we preserve the shape the policy author expects:
/// `Bool` -> the string "true"/"false", `Int` -> a Long, list -> a Set.
fn policy_scalar_attribute(value: &ClaimValue) -> RestrictedExpression {
    match value {
        ClaimValue::Text(text) => RestrictedExpression::new_string(text.clone()),
        ClaimValue::Bool(flag) => {
            RestrictedExpression::new_string(if *flag { "true" } else { "false" }.to_string())
        }
        ClaimValue::Int(int) => RestrictedExpression::new_long(*int),
        ClaimValue::TextList(items) => RestrictedExpression::new_set(
            items
                .iter()
                .map(|item| RestrictedExpression::new_string(item.clone())),
        ),
    }
}

/// Lifecycle-state string mirroring `identity.cedar`'s `principal.state`.
fn state_label(principal: &WorkloadPrincipal) -> &'static str {
    use iam_identity_workload_domain::WorkloadState;
    match principal.state() {
        WorkloadState::Provisioned => "provisioned",
        WorkloadState::Active => "active",
        WorkloadState::Suspended => "suspended",
        WorkloadState::Retired => "retired",
    }
}

// ---- Cedar text helpers (policy-compilation path) --------------------------

/// Render a Cedar string literal, escaping `\` and `"` so the value is a valid,
/// injection-safe Cedar string.
fn cedar_string(value: &str) -> Result<String, CedarAuthzError> {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => {
                return Err(CedarAuthzError::InvalidValue(
                    "nul byte in value".to_string(),
                ));
            }
            other => out.push(other),
        }
    }
    out.push('"');
    Ok(out)
}

/// Validate that a resource type name is a usable Cedar entity-type identifier.
/// Cedar type names are `Ident(::Ident)*`; we accept ASCII alphanumeric + `_`
/// segments separated by `::` and reject anything else (fail closed).
fn cedar_type_name(value: &str) -> Result<String, CedarAuthzError> {
    sanitize_type_name(value)
}

/// Shared validator/normalizer for entity type names.
fn sanitize_type_name(value: &str) -> Result<String, CedarAuthzError> {
    if value.is_empty() {
        return Err(CedarAuthzError::InvalidValue(
            "empty entity type name".to_string(),
        ));
    }
    let valid = value.split("::").all(|segment| {
        !segment.is_empty()
            && segment
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
    });
    if valid {
        Ok(value.to_string())
    } else {
        Err(CedarAuthzError::InvalidValue(format!(
            "invalid entity type name: {value}"
        )))
    }
}

/// Cedar attribute name for a claim. Claim names are already validated by the
/// domain (no whitespace/control chars); Cedar attribute keys are quoted in the
/// `has`/`.` access we emit, so we pass them through unchanged.
fn claim_attr_name(claim: &str) -> String {
    format!("claim_{claim}")
}

/// Deterministic, collision-free internal policy id for a compiled slot.
fn stable_policy_id(slot: usize) -> PolicyId {
    // PolicyId::from_str is infallible for this charset; build directly.
    PolicyId::new(format!("p{slot}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use iam_identity_workload_domain::{Effect, WorkloadState};

    fn active_principal() -> WorkloadPrincipal {
        let mut principal =
            WorkloadPrincipal::provision("ten_acme", "wl_deployer", "cap.cloud.deploy")
                .expect("valid");
        principal
            .transition_to(WorkloadState::Active)
            .expect("activate");
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
        assert!(matches!(
            decision.reason(),
            iam_identity_workload_domain::DecisionReason::DefaultDeny
        ));
    }

    #[test]
    fn matching_permit_allows() {
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("allow-acme-deploy")
                    .when_principal(PrincipalCondition::TenantIs("ten_acme".into()))
                    .when_principal(PrincipalCondition::HasScope("cloud.deploy.write".into()))
                    .for_action(ActionCondition::Equals("cloud.deploy.Apply".into()))
                    .for_resource(ResourceCondition::TypeIs("Deployment".into())),
            )
            .expect("policy compiles");
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert!(decision.is_allow());
        assert_eq!(authorizer.len(), 1);
    }

    #[test]
    fn permit_reason_carries_policy_id() {
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("allow-acme-deploy")
                    .when_principal(PrincipalCondition::TenantIs("ten_acme".into())),
            )
            .expect("policy compiles");
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert!(matches!(
            decision.reason(),
            iam_identity_workload_domain::DecisionReason::ExplicitPermit { policy_id }
                if policy_id == "allow-acme-deploy"
        ));
    }

    #[test]
    fn forbid_overrides_permit() {
        let authorizer = CedarWorkloadAuthorizer::with_policies(vec![
            Policy::permit("allow-acme-deploy")
                .when_principal(PrincipalCondition::TenantIs("ten_acme".into())),
            // Break-glass freeze: forbid all writes to checkout-svc.
            Policy::forbid("freeze-checkout").for_resource(ResourceCondition::Is {
                resource_type: "Deployment".into(),
                resource_id: "checkout-svc".into(),
            }),
        ])
        .expect("policies compile");
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(
            decision.reason(),
            iam_identity_workload_domain::DecisionReason::ExplicitForbid { policy_id }
                if policy_id == "freeze-checkout"
        ));
    }

    #[test]
    fn permit_for_other_tenant_does_not_match() {
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("allow-globex")
                    .when_principal(PrincipalCondition::TenantIs("ten_globex".into())),
            )
            .expect("policy compiles");
        // active_principal() is ten_acme; cross-tenant permit must not apply.
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert_eq!(decision.effect(), Effect::Deny);
    }

    #[test]
    fn same_tenant_resource_condition_matches_request_tenant_field() {
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("allow-own-quota")
                    .when_principal(PrincipalCondition::HasScope("quota:read".into()))
                    .for_action(ActionCondition::Equals("quota:Read".into()))
                    .for_resource(ResourceCondition::SameTenantAsPrincipal {
                        resource_type: "QuotaRecord".into(),
                    }),
            )
            .expect("policy compiles");
        let request = AuthorizationRequest::new(
            active_principal()
                .with_scope("quota:read")
                .expect("scope ok"),
            Action::new("quota:Read"),
            Resource::new("QuotaRecord", "ten_acme")
                .with_attribute("tenant_id", ClaimValue::Text("ten_acme".into())),
        );

        let decision = authorizer.authorize(&request);

        assert!(decision.is_allow(), "expected allow, got {decision:?}");
    }

    #[test]
    fn same_tenant_resource_condition_denies_cross_tenant_resource() {
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("allow-own-quota")
                    .when_principal(PrincipalCondition::HasScope("quota:read".into()))
                    .for_action(ActionCondition::Equals("quota:Read".into()))
                    .for_resource(ResourceCondition::SameTenantAsPrincipal {
                        resource_type: "QuotaRecord".into(),
                    }),
            )
            .expect("policy compiles");
        let request = AuthorizationRequest::new(
            active_principal()
                .with_scope("quota:read")
                .expect("scope ok"),
            Action::new("quota:Read"),
            Resource::new("QuotaRecord", "ten_globex")
                .with_attribute("tenant_id", ClaimValue::Text("ten_globex".into())),
        );

        let decision = authorizer.authorize(&request);

        assert_eq!(decision.effect(), Effect::Deny);
    }

    #[test]
    fn same_tenant_resource_condition_fails_closed_without_resource_tenant() {
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("allow-own-quota")
                    .when_principal(PrincipalCondition::HasScope("quota:read".into()))
                    .for_action(ActionCondition::Equals("quota:Read".into()))
                    .for_resource(ResourceCondition::SameTenantAsPrincipal {
                        resource_type: "QuotaRecord".into(),
                    }),
            )
            .expect("policy compiles");
        let request = AuthorizationRequest::new(
            active_principal()
                .with_scope("quota:read")
                .expect("scope ok"),
            Action::new("quota:Read"),
            Resource::new("QuotaRecord", "ten_acme"),
        );

        let decision = authorizer.authorize(&request);

        assert_eq!(decision.effect(), Effect::Deny);
    }

    #[test]
    fn missing_scope_denies_even_with_tenant_match() {
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("needs-admin-scope")
                    .when_principal(PrincipalCondition::TenantIs("ten_acme".into()))
                    .when_principal(PrincipalCondition::HasScope("cloud.deploy.admin".into())),
            )
            .expect("policy compiles");
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert_eq!(decision.effect(), Effect::Deny);
    }

    #[test]
    fn claim_condition_matches_text_and_bool() {
        let principal = active_principal()
            .with_claim("mfa", ClaimValue::Bool(true))
            .expect("claim ok");
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("prod-mfa")
                    .when_principal(PrincipalCondition::ClaimContains {
                        claim: "env".into(),
                        needle: "prod".into(),
                    })
                    .when_principal(PrincipalCondition::ClaimContains {
                        claim: "mfa".into(),
                        needle: "true".into(),
                    }),
            )
            .expect("policy compiles");
        assert!(authorizer.authorize(&deploy_request(principal)).is_allow());
    }

    #[test]
    fn claim_condition_matches_list_membership() {
        let principal = active_principal()
            .with_claim(
                "groups",
                ClaimValue::TextList(vec!["deployers".into(), "readers".into()]),
            )
            .expect("claim ok");
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(Policy::permit("group-gated").when_principal(
                PrincipalCondition::ClaimContains {
                    claim: "groups".into(),
                    needle: "deployers".into(),
                },
            ))
            .expect("policy compiles");
        assert!(authorizer.authorize(&deploy_request(principal)).is_allow());
    }

    #[test]
    fn suspended_principal_denied_before_policies() {
        let mut principal = active_principal();
        principal
            .transition_to(WorkloadState::Suspended)
            .expect("suspend");
        // A permit that WOULD match if active.
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("allow-all-acme")
                    .when_principal(PrincipalCondition::TenantIs("ten_acme".into())),
            )
            .expect("policy compiles");
        let decision = authorizer.authorize(&deploy_request(principal));
        assert!(matches!(
            decision.reason(),
            iam_identity_workload_domain::DecisionReason::PrincipalNotOperational { .. }
        ));
    }

    // ---- Raw Cedar policy text path (production grounding) -----------------

    #[test]
    fn raw_cedar_permit_allows_matching_request() {
        // A real Cedar policy in text form, evaluated by the real engine.
        let authorizer = CedarWorkloadAuthorizer::from_cedar_policies(
            r#"
            @id("permit-acme-deploy")
            permit (
              principal is Workload,
              action == Action::"cloud.deploy.Apply",
              resource is Deployment
            ) when {
              principal.tenant_id == "ten_acme" &&
              principal.scopes.contains("cloud.deploy.write")
            };
            "#,
        )
        .expect("cedar parses");
        assert_eq!(authorizer.len(), 1);
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert!(decision.is_allow());
    }

    #[test]
    fn raw_cedar_forbid_overrides_permit() {
        let authorizer = CedarWorkloadAuthorizer::from_cedar_policies(
            r#"
            @id("permit-all-acme")
            permit ( principal is Workload, action, resource )
            when { principal.tenant_id == "ten_acme" };

            @id("forbid-checkout-freeze")
            forbid ( principal is Workload, action, resource is Deployment )
            when { resource == Deployment::"checkout-svc" };
            "#,
        )
        .expect("cedar parses");
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(
            decision.reason(),
            iam_identity_workload_domain::DecisionReason::ExplicitForbid { policy_id }
                if policy_id == "forbid-checkout-freeze"
        ));
    }

    #[test]
    fn raw_cedar_default_deny_when_no_policy_matches() {
        let authorizer = CedarWorkloadAuthorizer::from_cedar_policies(
            r#"
            @id("permit-other-action")
            permit ( principal is Workload, action == Action::"cloud.deploy.Delete", resource );
            "#,
        )
        .expect("cedar parses");
        // Request asks for Apply, policy only permits Delete -> default deny.
        let decision = authorizer.authorize(&deploy_request(active_principal()));
        assert_eq!(decision.effect(), Effect::Deny);
        assert!(matches!(
            decision.reason(),
            iam_identity_workload_domain::DecisionReason::DefaultDeny
        ));
    }

    #[test]
    fn malformed_cedar_policy_is_rejected() {
        // Not valid Cedar: missing effect keyword + unbalanced.
        let result = CedarWorkloadAuthorizer::from_cedar_policies("this is not cedar (");
        assert!(matches!(result, Err(CedarAuthzError::PolicyParse(_))));
    }

    #[test]
    fn context_attribute_is_visible_to_policy() {
        // Confused-deputy style: require a context flag the request carries.
        let authorizer = CedarWorkloadAuthorizer::from_cedar_policies(
            r#"
            @id("require-mfa-context")
            permit ( principal is Workload, action, resource )
            when { context has mfa_present && context.mfa_present == "true" };
            "#,
        )
        .expect("cedar parses");

        let allowed =
            deploy_request(active_principal()).with_context("mfa_present", ClaimValue::Bool(true));
        assert!(authorizer.authorize(&allowed).is_allow());

        // Without the context flag -> default deny.
        let denied = deploy_request(active_principal());
        assert_eq!(authorizer.authorize(&denied).effect(), Effect::Deny);
    }

    #[test]
    fn malformed_resource_type_fails_closed() {
        // A resource type that is not a valid Cedar identifier must deny, never
        // panic (ADR-0083 Tier 3 fail-closed on the request path).
        let authorizer = CedarWorkloadAuthorizer::new()
            .add_policy(
                Policy::permit("allow-all")
                    .when_principal(PrincipalCondition::TenantIs("ten_acme".into())),
            )
            .expect("policy compiles");
        let request = AuthorizationRequest::new(
            active_principal(),
            Action::new("cloud.deploy.Apply"),
            Resource::new("123-not-an-ident", "x"),
        );
        // try_authorize surfaces the typed error...
        assert!(matches!(
            authorizer.try_authorize(&request),
            Err(CedarAuthzError::InvalidValue(_))
        ));
        // ...and the infallible trait method fails closed to deny.
        assert_eq!(authorizer.authorize(&request).effect(), Effect::Deny);
    }
}
