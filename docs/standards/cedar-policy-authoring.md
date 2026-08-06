---
doc_class: Standard
title: Cedar Policy Authoring Standard
status: Accepted
date: 2026-05-20
owner: council-security + axis-policy-engine
related_oyatie_adrs:
  - ADR-0007
  - ADR-0183
  - ADR-0243
  - ADR-0246
  - ADR-0316
enforced_by:
  - oya-governance-cedar-policy-authoring
  - oya-governance-cedar-structural-validator
  - oya-governance-capability-tier-cedar-coverage
canonical_paths:
  - docs/standards/cedar-policy-discipline.md
  - specs/cedar-fragment-schema.json
  - specs/policy/cedar-scope-schema.md
  - microservices/*/policy/*.cedar
external_reference:
  - https://docs.cedarpolicy.com/
---

# Cedar Policy Authoring Standard

This standard is the full authoring bar for Cedar policy fragments in Oyatie.
Cedar is the authorization policy language used for RBAC, ABAC, capability tier
grants, autonomy ceilings, tenant boundaries, CI actors, auditor scopes, and
defense-in-depth forbids. `cedar-policy-discipline.md` remains the short policy
discipline; this document is the implementation-grade standard.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to every `microservices/<ms>/policy/*.cedar` file.

It applies to canonical imports under `microservices/governance/policy/`.

It applies to Cedar schemas.

It applies to Cedar fragment manifests.

It applies to capability tier policy coverage.

It applies to autonomy ceiling policy coverage.

It applies to CI actor policy coverage.

It applies to auditor JIT policy coverage.

It applies to public anonymous allow-list policy coverage.

It applies to regulatory pack overlays that compile into Cedar or reference Cedar intent.

It does not make Cedar a replacement for Kubernetes admission.

It does not make Cedar a replacement for database row-level security.

It does not allow policy to bypass service-owned domain invariants.

## Normative Requirements

P-001. Every microservice MUST include `tenant-scope.cedar`.

P-002. Every microservice MUST include `ci-scope.cedar`.

P-003. Every microservice MUST include `auditor-scope.cedar`.

P-004. Every microservice MUST include `public-read.cedar`.

P-005. Additional policy files MUST cite an ADR or capability tier record.

P-006. Every policy file MUST include the standard header.

P-007. Every policy file MUST cite canonical imports.

P-008. Every policy file MUST cite related threat model or DPIA when regulated data is involved.

P-009. Every policy file MUST have a PERMITS section.

P-010. Every policy file MUST have a FORBIDS section.

P-011. Every policy file MUST have a SCHEMA HINTS section.

P-012. Every policy file MUST have a FRAMEWORK MAPPING section.

P-013. Every permit MUST have a stable rule id.

P-014. Every forbid MUST have a stable rule id.

P-015. Every rule id MUST be unique within the file.

P-016. Every permit MUST name principal, action, and resource shape.

P-017. Every forbid MUST name principal, action, and resource shape.

P-018. Every policy MUST implement cross-tenant refusal.

P-019. Every policy MUST implement cross-pack refusal.

P-020. Every policy MUST implement expired-token refusal.

P-021. Every policy MUST implement least-privilege anonymous refusal.

P-022. Every tenant-scoped permit MUST compare principal tenant to resource tenant.

P-023. Every regulated-data permit MUST test data class or pack scope.

P-024. Every capability-tier permit MUST test tier grant.

P-025. Every autonomy-tier permit MUST test autonomy ceiling.

P-026. Every CI permit MUST test lane actor and repository scope.

P-027. Every auditor permit MUST test JIT expiry.

P-028. Every public-read permit MUST enumerate allowed resource classes.

P-029. Every destructive action MUST require explicit authority context.

P-030. Every break-glass action MUST require break-glass event id.

P-031. Every production destructive action MUST require acknowledgement context.

P-032. Every policy schema MUST include entity types.

P-033. Every policy schema MUST include action types.

P-034. Every policy schema MUST include context attributes.

P-035. Every policy schema MUST avoid unbounded `Any`-style context.

P-036. Every policy schema MUST reserve implementation-private attributes.

P-037. Every policy decision MUST be auditable by decision id.

P-038. Every policy denial SHOULD produce a typed reason.

P-039. Every policy update MUST include fixtures.

P-040. Every policy update MUST include positive tests.

P-041. Every policy update MUST include negative tests.

P-042. Every policy update MUST include cross-tenant negative tests.

P-043. Every policy update MUST include expired-token negative tests.

P-044. Every policy update MUST include anonymous negative tests when public read exists.

P-045. Policy authors MUST prefer explicit conditions over broad group membership.

P-046. Policy authors MUST prefer forbids for defense-in-depth invariants.

P-047. Policy authors MUST not rely on rule order for safety.

P-048. Policy authors MUST not encode business workflows in Cedar.

P-049. Policy authors MUST not encode pricing rules in Cedar.

P-050. Policy authors MUST not encode UI navigation in Cedar.

P-051. Policy authors MUST not encode secrets in Cedar.

P-052. Policy authors MUST not use wildcard actions in permits.

P-053. Policy authors MUST not use wildcard resources in permits.

P-054. Policy authors MUST not create tenant-admin superuser bypass.

P-055. Policy authors MUST not create platform-owner bypass outside explicit break-glass.

P-056. Policy authors MUST not treat CI actors as humans.

P-057. Policy authors MUST not treat auditors as tenant admins.

P-058. Policy authors MUST not treat anonymous public read as unauthenticated mutation.

P-059. Policy authors MUST not compile policy fragments from unreviewed tenant input.

P-060. Policy authors MUST version policy schemas when changing entity shape.

## Worked Examples

### Example 1: Tenant-scoped permit

```cedar
// P1 - tenant member may read own tenant task
permit (
  principal in Role::"tenant_member",
  action == Action::"ReadTask",
  resource
) when {
  principal.tenant_id == resource.tenant_id &&
  resource.data_class in principal.allowed_data_classes
};
```

This passes because tenant and data class are explicit.

### Example 2: Cross-tenant forbid

```cedar
// F1 - cross-tenant access is always refused
forbid (
  principal,
  action,
  resource
) when {
  principal.tenant_id != resource.tenant_id
};
```

This passes because the invariant is fail-closed.

### Example 3: Capability tier permit

```cedar
permit (
  principal in Role::"tenant_admin",
  action == Action::"GrantCapabilityTier",
  resource
) when {
  resource.capability_tier == "workflow-engine.approval-routing.professional" &&
  principal.grants.contains("capability-tier-admin")
};
```

This passes only when the capability tier registry has the referenced tier.

### Example 4: Auditor JIT access

```cedar
permit (
  principal in Role::"external_auditor",
  action == Action::"ReadAuditEvidence",
  resource
) when {
  principal.jit_grant_expires_at > context.now &&
  resource.audit_scope_id == principal.audit_scope_id
};
```

This passes because access expires and scope is explicit.

### Example 5: Invalid wildcard permit

```cedar
permit (principal, action, resource);
```

This fails because it grants every action on every resource.

A permit that constrains the action but leaves bare `resource` with no resource/scope predicate also fails for deployed production policy because it grants that action across every resource:
```cedar
permit (principal, action == Action::"ReadDocument", resource);
```

## Verification

Merge/promotion authority is the cloud-ci/oya-ci Rust gate packet for Cedar policy authoring,
Cedar structural validation, capability-tier Cedar coverage, and autonomy-ceiling coverage.
Legacy `oya gate` invocations are transitional/local feedback only; they are not merge
authority, promotion authority, or a required context.

Local feedback commands while the cloud-ci/Rust gates are being cut over:
- `oya gate validate cedar-policy-authoring --scope microservices`
- `oya gate validate cedar-structural-validator --scope microservices`
- `oya gate validate capability-tier-cedar-coverage --scope microservices`
- `oya gate validate autonomy-ceiling --scope capabilities`

The checker MUST parse every Cedar file.

The checker MUST parse Cedar schema files.

The checker MUST parse fragment manifests.

The checker MUST parse capability tier records.

The checker MUST detect missing canonical archetypes.

The checker MUST detect missing headers.

The checker MUST detect missing imports.

The checker MUST detect missing PERMITS section.

The checker MUST detect missing FORBIDS section.

The checker MUST detect missing SCHEMA HINTS section.

The checker MUST detect missing FRAMEWORK MAPPING section.

The checker MUST detect missing F-CROSS-TENANT.

The checker MUST detect missing F-CROSS-PACK.

The checker MUST detect missing F-EXPIRED-TOKEN.

The checker MUST detect missing F-LEAST-PRIVILEGE.

The checker MUST detect wildcard permits.

The checker MUST detect untested policy updates.

The checker MUST detect capability tiers without Cedar coverage.

The checker MUST detect Cedar references to missing actions.

The checker MUST emit positive and negative fixture counts.

## Common Anti-Patterns

One `auth.cedar` file per service is an anti-pattern.

Wildcard permits are an anti-pattern.

Tenant admin superuser bypass is an anti-pattern.

Platform owner invisible bypass is an anti-pattern.

CI actor treated as human principal is an anti-pattern.

Auditor treated as tenant admin is an anti-pattern.

Public read treated as public mutation is an anti-pattern.

Policy without schema is an anti-pattern.

Schema without tests is an anti-pattern.

Permits without forbids are an anti-pattern.

Forbids without purpose comments are an anti-pattern.

Encoding workflow state transitions in Cedar is an anti-pattern.

Encoding billing plans in Cedar is an anti-pattern.

Referencing capability tiers by display label is an anti-pattern.

Using policy as a substitute for domain validation is an anti-pattern.

## Cross-References

External authority: `https://docs.cedarpolicy.com/`.

`docs/standards/cedar-policy-discipline.md` is the short canonical discipline.

`docs/decisions/ADR-0702-identity-authz-live-apex.md` binds Cedar and autonomy tiers.

`docs/decisions/ADR-0709-general-live-apex.md` separates Cedar from admission policy.

`docs/decisions/ADR-0700-ci-admission-live-apex.md` binds Cedar as universal gate.

`docs/decisions/ADR-0701-monorepo-capability-live-apex.md` binds policy substrate.

`docs/standards/capability-tier-matrix.md` binds tier policy coverage.

`docs/standards/regulatory-pack-authzpolicy-overlays.md` binds regulatory overlays.

## Substance Bar Compliance Checklist

CEDAR-SB-001. Verify `tenant-scope.cedar` exists.

CEDAR-SB-002. Verify `ci-scope.cedar` exists.

CEDAR-SB-003. Verify `auditor-scope.cedar` exists.

CEDAR-SB-004. Verify `public-read.cedar` exists.

CEDAR-SB-005. Verify additional policy file has ADR or tier record.

CEDAR-SB-006. Verify standard header.

CEDAR-SB-007. Verify canonical import reference.

CEDAR-SB-008. Verify threat-model reference when regulated.

CEDAR-SB-009. Verify PERMITS section.

CEDAR-SB-010. Verify FORBIDS section.

CEDAR-SB-011. Verify SCHEMA HINTS section.

CEDAR-SB-012. Verify FRAMEWORK MAPPING section.

CEDAR-SB-013. Verify permit rule ids.

CEDAR-SB-014. Verify forbid rule ids.

CEDAR-SB-015. Verify rule id uniqueness.

CEDAR-SB-016. Verify principal shape.

CEDAR-SB-017. Verify action shape.

CEDAR-SB-018. Verify resource shape.

CEDAR-SB-019. Verify F-CROSS-TENANT.

CEDAR-SB-020. Verify F-CROSS-PACK.

CEDAR-SB-021. Verify F-EXPIRED-TOKEN.

CEDAR-SB-022. Verify F-LEAST-PRIVILEGE.

CEDAR-SB-023. Verify tenant equality check.

CEDAR-SB-024. Verify data class check.

CEDAR-SB-025. Verify capability tier grant check.

CEDAR-SB-026. Verify autonomy ceiling check.

CEDAR-SB-027. Verify CI actor scope.

CEDAR-SB-028. Verify auditor JIT expiry.

CEDAR-SB-029. Verify public read allow-list.

CEDAR-SB-030. Verify destructive action authority.

CEDAR-SB-031. Verify break-glass event id.

CEDAR-SB-032. Verify production acknowledgement.

CEDAR-SB-033. Verify entity schema.

CEDAR-SB-034. Verify action schema.

CEDAR-SB-035. Verify context schema.

CEDAR-SB-036. Verify no unbounded context.

CEDAR-SB-037. Verify policy decision audit id.

CEDAR-SB-038. Verify denial reason shape.

CEDAR-SB-039. Verify positive fixtures.

CEDAR-SB-040. Verify negative fixtures.

CEDAR-SB-041. Check `microservices/tenancy/policy/tenant-scope.cedar`.

CEDAR-SB-042. Check `microservices/policy-engine/policy/tenant-scope.cedar`.

CEDAR-SB-043. Check `microservices/workflow-engine/policy/tenant-scope.cedar`.

CEDAR-SB-044. Check `microservices/ontology/policy/tenant-scope.cedar`.

CEDAR-SB-045. Check `microservices/messenger/policy/public-read.cedar`.

CEDAR-SB-046. Check `microservices/governance/policy/cedar-canonical-imports.cedar`.

CEDAR-SB-047. Check `specs/cedar-fragment-schema.json`.

CEDAR-SB-048. Check `specs/policy/cedar-scope-schema.md`.

CEDAR-SB-049. Check `workflow-engine.approval-routing.professional`.

CEDAR-SB-050. Check `policy-engine.cedar-gate.enterprise`.

CEDAR-SB-051. Reject wildcard permit.

CEDAR-SB-052. Reject tenant admin superuser bypass.

CEDAR-SB-053. Reject platform owner invisible bypass.

CEDAR-SB-054. Reject CI actor as human principal.

CEDAR-SB-055. Reject auditor as tenant admin.

CEDAR-SB-056. Reject public mutation in public read file.

CEDAR-SB-057. Reject policy without schema.

CEDAR-SB-058. Reject permits without forbids.

CEDAR-SB-059. Reject workflow encoded in Cedar.

CEDAR-SB-060. Reject billing encoded in Cedar.

CEDAR-SB-061. Emit policy file count.

CEDAR-SB-062. Emit canonical archetype count.

CEDAR-SB-063. Emit permit count.

CEDAR-SB-064. Emit forbid count.

CEDAR-SB-065. Emit schema entity count.

CEDAR-SB-066. Emit schema action count.

CEDAR-SB-067. Emit fixture count.

CEDAR-SB-068. Emit cross-tenant negative count.

CEDAR-SB-069. Emit capability tier coverage count.

CEDAR-SB-070. Emit autonomy ceiling coverage count.

CEDAR-SB-071. Preserve Cedar as application authorization.

CEDAR-SB-072. Preserve Kyverno as admission policy.

CEDAR-SB-073. Preserve RLS as database defense.

CEDAR-SB-074. Preserve domain invariants in code.

CEDAR-SB-075. Preserve default deny.

CEDAR-SB-076. Preserve explicit forbid priority.

CEDAR-SB-077. Preserve tenant isolation.

CEDAR-SB-078. Preserve pack isolation.

CEDAR-SB-079. Preserve JIT auditor expiry.

CEDAR-SB-080. Preserve auditability of every policy decision.

## Extended Worked Example: Tenant-Scoped Capability Policy

```cedar
// File: microservices/intelligence/policy/provider-invocation.cedar
permit(
  principal in Tenant::"tenant_01",
  action == Action::"foundry.provider.invoke.t2",
  resource in Capability::"cap.foundry.provider.invoke.t2"
)
when {
  principal.account_status == "active" &&
  principal.capability_ceiling >= 2 &&
  resource.provider_budget_remaining_cents > 0 &&
  context.request_data_class in ["internal", "tenant_confidential"] &&
  context.regulatory_pack not in ["pack-eu-high-risk-employment"]
};

forbid(
  principal,
  action == Action::"foundry.provider.invoke.t2",
  resource
)
when {
  context.regulatory_pack == "pack-eu-high-risk-employment" &&
  context.human_review_attached == false
};
```

```cedarschema
namespace Oyatie {
  entity Tenant = {
    account_status: String,
    capability_ceiling: Long,
  };

  entity Capability = {
    provider_budget_remaining_cents: Long,
  };

  action "foundry.provider.invoke.t2" appliesTo {
    principal: Tenant,
    resource: Capability,
    context: {
      request_data_class: String,
      regulatory_pack: String,
      human_review_attached: Bool,
      trace_id: String,
    }
  };
}
```

## Extended Cedar Policy Matrix

| ID | Concern | Requirement | Example path | Checker |
|---|---|---|---|---|
| CEDAR-MAT-001 | Default deny | No blanket permit | `policy/*.cedar` | `oya-check-cedar-default-deny` |
| CEDAR-MAT-002 | Forbid priority | Explicit forbids for regulated denial | `policy/*.cedar` | `oya-check-cedar-forbid-priority` |
| CEDAR-MAT-003 | Action name | Dotted action id | `Action::"foundry.provider.invoke.t2"` | `oya-check-cedar-action-names` |
| CEDAR-MAT-004 | Principal | Tenant or actor entity | schema | `oya-check-cedar-schema` |
| CEDAR-MAT-005 | Resource | Capability or object entity | schema | `oya-check-cedar-schema` |
| CEDAR-MAT-006 | Context | trace id required | schema | `oya-check-policy-context` |
| CEDAR-MAT-007 | Data class | context carries data class | schema | `oya-check-data-class` |
| CEDAR-MAT-008 | Pack | context carries regulatory pack | schema | `oya-check-pack-policy` |
| CEDAR-MAT-009 | Capability | action maps to capability tier | capability matrix | `oya-check-capability-tier-matrix` |
| CEDAR-MAT-010 | Budget | spend action checks budget | policy | `oya-check-provider-budget-policy` |
| CEDAR-MAT-011 | Human gate | regulated action requires human flag | policy | `oya-check-human-gate-policy` |
| CEDAR-MAT-012 | Tenant isolation | no cross-tenant wildcard | policy | `oya-check-tenant-isolation` |
| CEDAR-MAT-013 | Auditor | JIT expiry for audit reads | policy | `oya-check-auditor-jit` |
| CEDAR-MAT-014 | Tests | allow/deny fixtures | `policy/tests` | `oya-check-cedar-fixtures` |
| CEDAR-MAT-015 | Signing | fragment signature | registry | `oya-check-cedar-fragment-signature` |
| CEDAR-MAT-016 | Schema | no implicit unknown fields | schema | `cedar validate` |
| CEDAR-MAT-017 | Simulation | decision fixture hash | simulator output | `oya-check-cedar-simulation` |
| CEDAR-MAT-018 | Audit | decision emits audit event | audit chain | `oya-check-audit-emission` |
| CEDAR-MAT-019 | Version | bundle version bump | policy bundle | `oya-check-policy-version` |
| CEDAR-MAT-020 | Promote | checker output in evidence | VCS bundle | `oya-vcs-admission` |

## Extended Cedar Evidence Ledger

CEDAR-EVID-001. Record policy file path.

CEDAR-EVID-002. Record schema file path.

CEDAR-EVID-003. Record policy bundle id.

CEDAR-EVID-004. Record policy bundle version.

CEDAR-EVID-005. Record action ids.

CEDAR-EVID-006. Record principal entity types.

CEDAR-EVID-007. Record resource entity types.

CEDAR-EVID-008. Record context fields.

CEDAR-EVID-009. Record data-class fields.

CEDAR-EVID-010. Record regulatory-pack fields.

CEDAR-EVID-011. Record forbid statement count.

CEDAR-EVID-012. Record permit statement count.

CEDAR-EVID-013. Record default-deny validation.

CEDAR-EVID-014. Record tenant-isolation validation.

CEDAR-EVID-015. Record capability-tier validation.

CEDAR-EVID-016. Record human-gate validation.

CEDAR-EVID-017. Record budget-policy validation.

CEDAR-EVID-018. Record JIT-auditor validation.

CEDAR-EVID-019. Record allow fixture count.

CEDAR-EVID-020. Record deny fixture count.

CEDAR-EVID-021. Record simulation hash.

CEDAR-EVID-022. Record fragment signature.

CEDAR-EVID-023. Record signer certificate id.

CEDAR-EVID-024. Record audit event names.

CEDAR-EVID-025. Record checker crate version.

CEDAR-EVID-026. Record VCS changeset id.

CEDAR-EVID-027. Record promote bundle id.

CEDAR-EVID-028. Record residual policy risks.

## Extended Cedar Anti-Patterns

CEDAR-APX-001. Blanket permit with context-only denial.

CEDAR-APX-002. Missing explicit forbid for regulated action.

CEDAR-APX-003. Stringly typed action id outside registry.

CEDAR-APX-004. Policy fixture covers allow path only.

CEDAR-APX-005. Cross-tenant wildcard principal.

CEDAR-APX-006. Auditor access without JIT expiry.

CEDAR-APX-007. Spend action without budget context.

CEDAR-APX-008. Policy bundle unsigned at promotion.

CEDAR-APX-009. Human-review flag trusted without actor evidence.

CEDAR-APX-010. Decision audit event lacks policy bundle version.

## Extended Promotion Review Checklist

CEDAR-PROMOTE-001. Policy file path is stable.

CEDAR-PROMOTE-002. Schema file path is stable.

CEDAR-PROMOTE-003. Policy bundle id is stable.

CEDAR-PROMOTE-004. Policy bundle version is bumped.

CEDAR-PROMOTE-005. Action ids are registered.

CEDAR-PROMOTE-006. Principal entity types are declared.

CEDAR-PROMOTE-007. Resource entity types are declared.

CEDAR-PROMOTE-008. Context fields are declared.

CEDAR-PROMOTE-009. Data-class fields are declared.

CEDAR-PROMOTE-010. Regulatory-pack fields are declared.

CEDAR-PROMOTE-011. Forbid statement count is recorded.

CEDAR-PROMOTE-012. Permit statement count is recorded.

CEDAR-PROMOTE-013. Default-deny validation passes.

CEDAR-PROMOTE-014. Tenant-isolation validation passes.

CEDAR-PROMOTE-015. Capability-tier validation passes.

CEDAR-PROMOTE-016. Human-gate validation passes.

CEDAR-PROMOTE-017. Budget-policy validation passes.

CEDAR-PROMOTE-018. JIT-auditor validation passes.

CEDAR-PROMOTE-019. Allow fixture count is recorded.

CEDAR-PROMOTE-020. Deny fixture count is recorded.

CEDAR-PROMOTE-021. Simulation hash is recorded.

CEDAR-PROMOTE-022. Fragment signature is recorded.

CEDAR-PROMOTE-023. Signer certificate id is recorded.

CEDAR-PROMOTE-024. Audit event names are recorded.

CEDAR-PROMOTE-025. Checker crate version is recorded.

CEDAR-PROMOTE-026. VCS changeset id is recorded.

CEDAR-PROMOTE-027. Promote bundle id is recorded.

CEDAR-PROMOTE-028. Residual policy risks are recorded.

CEDAR-PROMOTE-029. No blanket permit exists.

CEDAR-PROMOTE-030. No cross-tenant wildcard exists.

CEDAR-PROMOTE-031. No spend action lacks budget.

CEDAR-PROMOTE-032. No auditor action lacks expiry.

CEDAR-PROMOTE-033. No regulated action lacks forbid.

CEDAR-PROMOTE-034. No human-review flag is trusted without actor evidence.

CEDAR-PROMOTE-035. No policy fragment is unsigned.

CEDAR-PROMOTE-036. No schema field is implicit.

CEDAR-PROMOTE-037. No action id is unregistered.

CEDAR-PROMOTE-038. No test fixture uses real tenant data.

CEDAR-PROMOTE-039. No denial fixture is missing.

CEDAR-PROMOTE-040. No allow fixture is missing.

CEDAR-PROMOTE-041. Policy simulator output is attached.

CEDAR-PROMOTE-042. Cedar formatter output is attached.

CEDAR-PROMOTE-043. Cedar schema validator output is attached.

CEDAR-PROMOTE-044. Capability matrix checker output is attached.

CEDAR-PROMOTE-045. Audit checker output is attached.

CEDAR-PROMOTE-046. Fragment signature checker output is attached.

CEDAR-PROMOTE-047. Pack-overlay checker output is attached.

CEDAR-PROMOTE-048. Data-class checker output is attached.

CEDAR-PROMOTE-049. Tenant-isolation checker output is attached.

CEDAR-PROMOTE-050. Promotion evidence includes Cedar checker output.

## Extended Cedar Residual-Risk Register

CEDAR-RISK-001. Residual risk: policy author misunderstands entity hierarchy; mitigation is schema fixture review.

CEDAR-RISK-002. Residual risk: pack overlay changes action semantics; mitigation is pack-overlay diff gate.

CEDAR-RISK-003. Residual risk: simulator fixtures drift from runtime context; mitigation is runtime-context snapshot test.

CEDAR-RISK-004. Residual risk: default-deny passes while required permits are missing; mitigation is allow-fixture coverage.

CEDAR-RISK-005. Residual risk: explicit forbid blocks emergency access; mitigation is break-glass ADR and JIT expiry.

CEDAR-RISK-006. Residual risk: budget context is stale; mitigation is budget projection freshness SLO.

CEDAR-RISK-007. Residual risk: human-review context is forged; mitigation is actor-bound approval event verification.

CEDAR-RISK-008. Residual risk: unsigned local bundle used in dev; mitigation is dev bundle signature requirement.

CEDAR-RISK-009. Residual risk: policy result is logged without bundle version; mitigation is audit schema enforcement.

CEDAR-RISK-010. Residual risk: reviewer reads policy without schema; mitigation is combined policy/schema review artifact.
