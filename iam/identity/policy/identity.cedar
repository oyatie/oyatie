// Workload-identity PARC authorization policy
// Authority: ADR-0002 (tenant-and-identity-kernel), ADR-0183 (Cedar app-authz),
//            ADR-0162 (per-tenant audit slicing).
// Bounded context: workload-identity (identity-workload-authz-cedar-adapter).
// Owner:   axis-identity + ops-security
//
// Design grounding — microservices/identity/design/hyperscaler-best-practice-brief.md §10:
//   Cedar's formally-proven properties (arXiv 2403.04651) are the bedrock:
//   default-deny, forbid-overrides-permit, order-independence. This policy set
//   relies on them. The PARC tuple is (principal, action, resource, context).
//
// Isolation primitive (brief §6): tenant == trust domain. Every permit is
// scoped so principal and resource share a tenant; the shared partition holds
// only global `forbid` guardrails.

// =============================================================================
// 1. Default deny. Nothing is permitted unless an explicit permit matches.
//    (Cedar is already default-deny; this is stated for auditor legibility.)
// =============================================================================

forbid (principal, action, resource);

// =============================================================================
// 2. Lifecycle guardrail (GLOBAL forbid — overrides every permit below).
//    A non-active WorkloadPrincipal can never be authorized. This is the
//    fast-revocation enforcement point: suspend/retire writes the principal to
//    the denylist and this forbid fires at evaluate time (brief §1, §10).
//    Policy id: forbid-suspended-principal
// =============================================================================

forbid (
  principal is Oya::WorkloadPrincipal,
  action,
  resource
) when {
  principal.state != "active"
};

// =============================================================================
// 3. Tenant isolation guardrail (GLOBAL forbid). A principal may never act on a
//    resource outside its own trust domain, regardless of any permit. This is
//    belt-and-braces over the per-permit tenant check in §5 (brief §6).
//    Policy id: forbid-cross-trust-domain
// =============================================================================

forbid (
  principal is Oya::WorkloadPrincipal,
  action,
  resource
) when {
  principal.trust_domain != resource.trust_domain
};

// =============================================================================
// 4. Confused-deputy / audience guardrail (GLOBAL forbid). If the request
//    context carries a token audience, it MUST match the principal's bound
//    audience. Defends the confused-deputy class (brief §5).
//    Policy id: forbid-audience-mismatch
// =============================================================================

forbid (
  principal is Oya::WorkloadPrincipal,
  action,
  resource
) when {
  context has token_audience &&
  context.token_audience != principal.bound_audience
};

// =============================================================================
// 5. Capability grant (PERMIT). An active, same-trust-domain principal may
//    perform an action whose required capability it holds, within its tenant.
//    Least-privilege: the action's capability must be in principal.capabilities
//    (brief §5 privilege-escalation defense).
//    Policy id: permit-capability-scoped-action
// =============================================================================

permit (
  principal is Oya::WorkloadPrincipal,
  action,
  resource
) when {
  principal.state == "active" &&
  principal.trust_domain == resource.trust_domain &&
  principal.tenant_id == resource.tenant_id &&
  action.required_capability in principal.capabilities
};

// =============================================================================
// 6. Scope-narrowed read (PERMIT). Read-class actions additionally require the
//    resource's scope to be in the principal's granted scopes. Lets a principal
//    that holds a broad capability still be narrowed per-resource by scope.
//    Policy id: permit-scope-narrowed-read
// =============================================================================

permit (
  principal is Oya::WorkloadPrincipal,
  action in [Oya::Action::"read", Oya::Action::"list", Oya::Action::"describe"],
  resource
) when {
  principal.state == "active" &&
  principal.trust_domain == resource.trust_domain &&
  resource.scope in principal.scopes
};

// =============================================================================
// 7. High-assurance write guardrail (GLOBAL forbid). Mutating actions on
//    resources tagged sensitive require mfa_present in context. Pairs with the
//    permit in §5 — forbid wins, so a sensitive write without MFA is denied
//    even if the capability is held (brief §5, §10 fail-closed).
//    Policy id: forbid-sensitive-write-without-mfa
// =============================================================================

forbid (
  principal is Oya::WorkloadPrincipal,
  action in [Oya::Action::"write", Oya::Action::"delete", Oya::Action::"rotate"],
  resource
) when {
  resource has sensitivity &&
  resource.sensitivity == "high"
} unless {
  context has mfa_present &&
  context.mfa_present == true
};
