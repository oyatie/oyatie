---
doc_class: Standard
title: Capability Tier Matrix Standard
status: Accepted
date: 2026-05-20
owner: council-architecture + axis-foundry
related_oyatie_adrs:
  - ADR-0007
  - ADR-0245
  - ADR-0250
  - ADR-0257
  - ADR-0316
enforced_by:
  - oya-governance-capability-tier-registry-shape
  - oya-governance-capability-tier-cedar-coverage
  - oya-governance-capability-tier-ontology-projection-pin
  - oya-governance-capability-tier-workflow-template-coverage
canonical_paths:
  - registry/capability-tiers/
  - specs/capability-tier-registry-schema.json
  - specs/capability-tier-grant-schema.json
  - docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md
---

# Capability Tier Matrix Standard

Capability tiers are the tenant-visible activation units that replace product
fragmentation. A tier is not a service, package, license SKU, or UI tab. It is a
versioned projection bundle over shared substrate: Cedar permissions, ontology
objects, workflow templates, UX shell manifests, compliance overlays, telemetry,
and cost allocation.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to `registry/capability-tiers/`.

It applies to per-microservice capability tier contribution manifests.

It applies to tenant grants.

It applies to tier migrations.

It applies to tier UI shell registration.

It applies to tier policy registration.

It applies to tier ontology projections.

It applies to tier workflow templates.

It applies to tier compliance overlays.

It applies to tier telemetry and cost dimensions.

It does not create new product services.

It does not replace microservice ownership.

It does not replace Cedar autonomy ceilings.

## Normative Requirements

C-001. Every capability tier MUST have a stable id.

C-002. Every capability tier id MUST use `<microservice>.<capability>.<tier>`.

C-003. Every capability tier MUST declare owning microservice.

C-004. Every capability tier MUST declare tenant grant shape.

C-005. Every capability tier MUST declare lifecycle state.

C-006. Every capability tier MUST declare activation prerequisites.

C-007. Every capability tier MUST declare deactivation behavior.

C-008. Every capability tier MUST declare migration behavior.

C-009. Every capability tier MUST declare Cedar permit sets.

C-010. Every capability tier MUST declare Cedar forbid coverage.

C-011. Every capability tier MUST declare ontology projections or explain workflow-only posture.

C-012. Every capability tier MUST declare workflow templates or explain read-only posture.

C-013. Every interactive capability tier MUST declare UX shell manifest.

C-014. Every API-only capability tier MUST declare API-only posture.

C-015. Every regulated capability tier MUST declare compliance overlays.

C-016. Every capability tier MUST pin ontology schema revisions.

C-017. Every capability tier MUST pin workflow template versions.

C-018. Every capability tier MUST pin policy versions.

C-019. Every capability tier MUST emit audit-chain evidence.

C-020. Every capability tier MUST publish FinOps cost dimensions.

C-021. Every capability tier MUST declare SLO impact.

C-022. Every capability tier MUST declare observability dashboard references.

C-023. Every capability tier MUST declare owner team.

C-024. Every capability tier MUST declare support runbooks.

C-025. Every capability tier MUST declare data classes.

C-026. Every capability tier MUST declare tenant audience.

C-027. Every capability tier MUST declare pack applicability.

C-028. Every capability tier MUST declare sovereign-cell behavior.

C-029. Every capability tier MUST declare on-prem behavior.

C-030. Every capability tier MUST declare BYOC behavior.

C-031. Every capability tier MUST declare API contract references when exposed.

C-032. Every capability tier MUST declare AsyncAPI event references when emitting events.

C-033. Every capability tier MUST declare proto references when using internal gRPC.

C-034. Every capability tier MUST declare rollback behavior.

C-035. Every capability tier MUST declare irreversible side effects.

C-036. Every capability tier MUST declare user-facing display labels separately from machine ids.

C-037. Every capability tier MUST declare localization keys for display labels.

C-038. Every capability tier MUST declare default grant posture.

C-039. Every capability tier MUST declare trial posture if trial-enabled.

C-040. Every capability tier MUST declare billing posture if billable.

C-041. Tier grants MUST be tenant-scoped.

C-042. Tier grants MUST be auditable.

C-043. Tier grants MUST be revocable.

C-044. Tier grants MUST be versioned.

C-045. Tier grants MUST not bypass autonomy ceilings.

C-046. Tier grants MUST not bypass data residency.

C-047. Tier grants MUST not bypass policy-engine evaluation.

C-048. Tier grants MUST not create hidden product modules.

C-049. Tier grants MUST not use marketing names as ids.

C-050. Tier grants MUST not activate unimplemented dependencies.

C-051. Contribution manifests MUST declare no-tier posture when a service exposes no tenant-visible tier.

C-052. Contribution manifests MUST be present for every service named in ADR-0316 Section K.

C-053. Contribution manifests MUST map capabilities to owning code paths.

C-054. Contribution manifests MUST map capabilities to policy paths.

C-055. Contribution manifests MUST map capabilities to ontology paths.

C-056. Contribution manifests MUST map capabilities to workflow paths.

C-057. Contribution manifests MUST map capabilities to UX paths when interactive.

C-058. Contribution manifests MUST map capabilities to tests.

C-059. Contribution manifests MUST map capabilities to runbooks.

C-060. Contribution manifests MUST map capabilities to support owners.

## Worked Examples

### Example 1: CRM customer graph tier

```yaml
id: ontology.customer-graph.professional
owning_microservice: ontology
tier_class: professional
cedar:
  policies:
    - microservices/ontology/policy/customer-graph-professional.cedar
ontology:
  object_types:
    - ontology.object.Account.v1
    - ontology.object.Contact.v1
workflow:
  templates:
    - microservices/workflow-engine/templates/customer-graph-refresh-v1.yaml
ux:
  shell_manifest: microservices/ontology/ux/customer-graph.shell.yaml
```

This passes because the tier composes substrate primitives without a CRM silo.

### Example 2: Read-only analytics tier

```yaml
id: analytics.executive-dashboard.readonly
workflow:
  posture: not_required_read_only
ontology:
  object_types:
    - ontology.object.RevenueMetric.v1
cedar:
  policies:
    - microservices/analytics/policy/executive-dashboard-readonly.cedar
```

This passes because the missing workflow template is justified.

### Example 3: Invalid product module

```yaml
id: sales-cloud.enterprise
service_boundary: sales-cloud
```

This fails because ADR-0316 requires capability-tier activation over shared
microservices, not product service fragmentation.

### Example 4: Regulated tier overlay

```yaml
id: payments.dispute-resolution.financial-services
compliance_overlays:
  - pack-us-msb-mtl
  - pack-eu-dora
data_classes:
  - FINANCIAL_TRANSACTION
  - REGULATED_DECISION
```

This passes because regulated behavior is explicit.

### Example 5: Tenant grant evidence

```json
{
  "event": "EVT-CAPABILITY-TIER-GRANTED-V1",
  "tenant_id_hash": "sha256:...",
  "capability_tier": "workflow-engine.approval-routing.professional",
  "granted_by": "principal:tenant-admin",
  "policy_decision_id": "cedar_dec_01J..."
}
```

This passes because activation is auditable.

## Verification

Primary command:

```bash
oya gate validate capability-tier-registry-shape --scope registry/capability-tiers
```

Required companion commands:

```bash
oya gate validate capability-tier-cedar-coverage --scope microservices
oya gate validate capability-tier-ontology-projection-pin --scope microservices
oya gate validate capability-tier-workflow-template-coverage --scope microservices
oya gate validate capability-tier-ux-shell-coverage --scope microservices
oya gate validate capability-tier-compliance-overlay-coverage --scope microservices
```

The checker MUST reject missing contribution manifests.

The checker MUST reject missing no-tier declarations.

The checker MUST reject missing Cedar permits.

The checker MUST reject missing Cedar forbids.

The checker MUST reject missing ontology projections unless workflow-only is declared.

The checker MUST reject missing workflow templates unless read-only is declared.

The checker MUST reject missing UX shell for interactive tiers.

The checker MUST reject missing compliance overlays for regulated data.

The checker MUST reject missing cost dimensions.

The checker MUST reject missing SLO impact.

The checker MUST reject missing audit events.

The checker MUST reject marketing names as ids.

The checker MUST reject unversioned dependencies.

The checker MUST produce a per-service matrix.

The checker SHOULD produce a tenant-grant fixture.

## Common Anti-Patterns

Creating a `crm` service for every CRM concept is an anti-pattern.

Creating a `sales-cloud` module is an anti-pattern.

Putting licensing SKU names in tier ids is an anti-pattern.

Granting a tier without Cedar policy is an anti-pattern.

Granting a tier without audit emission is an anti-pattern.

Granting a tier without ontology or workflow posture is an anti-pattern.

Treating UX navigation as the tier authority is an anti-pattern.

Treating compliance overlays as sales packaging is an anti-pattern.

Treating cost allocation as post-launch work is an anti-pattern.

Treating tier migration as a data-only migration is an anti-pattern.

Treating tier deactivation as account deletion is an anti-pattern.

Treating no-tier services as undocumented is an anti-pattern.

Treating pack labels as optional metadata is an anti-pattern.

Treating tenant grants as Boolean flags is an anti-pattern.

Treating ADR-0316 Section K as advisory is an anti-pattern.

## Cross-References

`docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md` is the binding doctrine.

`docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md` binds autonomy ceilings.

`docs/decisions/ADR-0245-substrate-vs-product-layering.md` binds substrate layering.

`docs/standards/ontology-projection-substrate.md` binds projection requirements.

`docs/standards/workflow-substrate-engine.md` binds workflow template requirements.

`docs/standards/cedar-policy-authoring.md` binds policy requirements.

`docs/standards/openslo-authoring.md` binds SLO requirements.

`registry/capability-tiers/` is the machine-readable registry root.

## Substance Bar Compliance Checklist

CT-SB-001. Verify capability tier id shape.

CT-SB-002. Verify owning microservice.

CT-SB-003. Verify tenant grant schema.

CT-SB-004. Verify lifecycle state.

CT-SB-005. Verify activation prerequisites.

CT-SB-006. Verify deactivation behavior.

CT-SB-007. Verify migration behavior.

CT-SB-008. Verify Cedar permit set.

CT-SB-009. Verify Cedar forbid coverage.

CT-SB-010. Verify ontology projection posture.

CT-SB-011. Verify workflow template posture.

CT-SB-012. Verify UX shell posture.

CT-SB-013. Verify API-only posture.

CT-SB-014. Verify compliance overlay.

CT-SB-015. Verify ontology schema pin.

CT-SB-016. Verify workflow version pin.

CT-SB-017. Verify policy version pin.

CT-SB-018. Verify audit-chain evidence.

CT-SB-019. Verify FinOps dimensions.

CT-SB-020. Verify SLO impact.

CT-SB-021. Verify dashboard reference.

CT-SB-022. Verify runbook reference.

CT-SB-023. Verify owner team.

CT-SB-024. Verify data classes.

CT-SB-025. Verify tenant audience.

CT-SB-026. Verify pack applicability.

CT-SB-027. Verify sovereign-cell behavior.

CT-SB-028. Verify on-prem behavior.

CT-SB-029. Verify BYOC behavior.

CT-SB-030. Verify OpenAPI references.

CT-SB-031. Verify AsyncAPI references.

CT-SB-032. Verify proto references.

CT-SB-033. Verify rollback behavior.

CT-SB-034. Verify irreversible side effects.

CT-SB-035. Verify display label indirection.

CT-SB-036. Verify localization keys.

CT-SB-037. Verify default grant posture.

CT-SB-038. Verify trial posture.

CT-SB-039. Verify billing posture.

CT-SB-040. Verify grant revocation behavior.

CT-SB-041. Check `ontology.customer-graph.professional`.

CT-SB-042. Check `workflow-engine.approval-routing.professional`.

CT-SB-043. Check `analytics.executive-dashboard.readonly`.

CT-SB-044. Check `payments.dispute-resolution.financial-services`.

CT-SB-045. Check `mail.campaign-delivery.professional`.

CT-SB-046. Check `messenger.personal-e2ee.consumer`.

CT-SB-047. Check `tenancy.sub-scope-registry.enterprise`.

CT-SB-048. Check `policy-engine.cedar-gate.enterprise`.

CT-SB-049. Check `observability.slo-promotion.platform`.

CT-SB-050. Check `foundry.capability-publish.platform`.

CT-SB-051. Reject product service fragmentation.

CT-SB-052. Reject marketing label as id.

CT-SB-053. Reject Boolean tenant flag grant.

CT-SB-054. Reject grant without audit.

CT-SB-055. Reject grant without Cedar.

CT-SB-056. Reject tier without projection posture.

CT-SB-057. Reject tier without workflow posture.

CT-SB-058. Reject interactive tier without UX shell.

CT-SB-059. Reject regulated tier without pack overlay.

CT-SB-060. Reject no-tier service without declaration.

CT-SB-061. Emit capability tier count.

CT-SB-062. Emit grant schema count.

CT-SB-063. Emit contribution manifest count.

CT-SB-064. Emit no-tier declaration count.

CT-SB-065. Emit Cedar coverage count.

CT-SB-066. Emit ontology projection count.

CT-SB-067. Emit workflow template count.

CT-SB-068. Emit UX shell count.

CT-SB-069. Emit compliance overlay count.

CT-SB-070. Emit cost dimension count.

CT-SB-071. Preserve ADR-0316 product-fragmentation refusal.

CT-SB-072. Preserve tenant-scoped activation.

CT-SB-073. Preserve policy-first grant decisions.

CT-SB-074. Preserve ontology projection pins.

CT-SB-075. Preserve workflow template pins.

CT-SB-076. Preserve UX as shell, not authority.

CT-SB-077. Preserve compliance overlays.

CT-SB-078. Preserve audit evidence.

CT-SB-079. Preserve FinOps attribution.

CT-SB-080. Preserve revocation path.

## Extended Worked Example: Capability Tier Entry

```yaml
capability_id: cap.foundry.provider.invoke.t2
title: Foundry provider invocation with human-gated regulated output
owning_microservice: foundry
implementation_paths:
  - microservices/foundry/contracts/provider-invocation-v1.openapi.yaml
  - microservices/foundry/contracts/provider-events-v1.asyncapi.yaml
  - crates/oya-foundry-provider-invocation-usecase
  - crates/oya-foundry-provider-invocation-adapter-openai
  - microservices/foundry/policy/provider-invocation.cedar
related_adrs:
  - docs/decisions/ADR-0243-cedar-as-universal-gate.md
  - docs/decisions/ADR-0255-provider-byok-credential-envelope.md
  - docs/decisions/ADR-0308-ml-model-lifecycle-ai-act-compliance.md
tier: T2
allowed_autonomy:
  - draft_output
  - classify_request
  - prepare_tool_plan
forbidden_autonomy:
  - send_regulated_decision_without_human
  - mutate_customer_record_without_policy
  - spend_provider_budget_without_limit
required_controls:
  cedar_action: Action::"foundry.provider.invoke.t2"
  budget_policy: provider_budget_limit_v1
  audit_events:
    - EVT-FOUNDRY-PROVIDER-INVOCATION-REQUESTED
    - EVT-FOUNDRY-PROVIDER-INVOCATION-APPROVED
    - EVT-FOUNDRY-PROVIDER-INVOCATION-DENIED
  slo_refs:
    - docs/slos/foundry-provider-latency.openslo.yaml
verification:
  - cargo run -p oya-check-capability-tier-matrix --quiet
  - cargo run -p oya-check-cedar-action-coverage --quiet
  - cargo run -p oya-check-autonomy-ceiling --quiet
```

## Extended Capability Matrix

| ID | Capability | Tier | Authority path | Required gate |
|---|---|---|---|---|
| CT-MAT-001 | Tenant creation | T1 | `microservices/tenancy` | Cedar tenant-admin |
| CT-MAT-002 | Tenant deletion | T2 | `microservices/tenancy` | two-person approval |
| CT-MAT-003 | Pack attachment | T2 | `regional-packs/*` | pack owner approval |
| CT-MAT-004 | Provider invocation | T2 | `microservices/foundry` | provider budget gate |
| CT-MAT-005 | Regulated auto decision | T1 | `microservices/foundry` | human review required |
| CT-MAT-006 | Workflow template publish | T2 | `microservices/workflow-engine` | template review |
| CT-MAT-007 | Workflow execution start | T1 | `microservices/workflow-engine` | tenant policy |
| CT-MAT-008 | Workflow execution cancel | T1 | `microservices/workflow-engine` | actor policy |
| CT-MAT-009 | Billing invoice issue | T2 | `microservices/billing` | finance approval |
| CT-MAT-010 | Billing tax override | T3 | `microservices/billing` | finance plus compliance |
| CT-MAT-011 | Mail send | T1 | `microservices/mail` | sender policy |
| CT-MAT-012 | Mail legal hold release | T3 | `microservices/mail` | legal approval |
| CT-MAT-013 | Messenger room create | T1 | `microservices/messenger` | tenant policy |
| CT-MAT-014 | Messenger key reset | T3 | `microservices/messenger` | security ceremony |
| CT-MAT-015 | Recording export | T2 | `microservices/recordings` | data-class approval |
| CT-MAT-016 | Search index rebuild | T2 | `microservices/search` | data-class scan |
| CT-MAT-017 | Ads audience export | T3 | `microservices/ads` | privacy approval |
| CT-MAT-018 | Audit evidence read | T2 | `microservices/audit` | auditor JIT policy |
| CT-MAT-019 | Audit evidence delete | forbidden | `microservices/audit` | not allowed |
| CT-MAT-020 | Policy fragment publish | T3 | `microservices/policy-engine` | signed fragment |
| CT-MAT-021 | Cedar schema update | T2 | `microservices/policy-engine` | compatibility check |
| CT-MAT-022 | OpenBao secret rotate | T2 | `microservices/secrets` | rotation runbook |
| CT-MAT-023 | HSM root ceremony | T4 | `docs/runbooks/cedar-hsm-root-key-ceremony.md` | ceremony quorum |
| CT-MAT-024 | Region failover | T3 | `microservices/cell` | disaster mode gate |
| CT-MAT-025 | SLO threshold change | T1 | `docs/slos` | SRE approval |
| CT-MAT-026 | Public API deprecation | T2 | `docs/standards/openapi-3-2-authoring.md` | client impact review |
| CT-MAT-027 | Async event removal | T2 | `docs/standards/asyncapi-3-1-authoring.md` | consumer compatibility |
| CT-MAT-028 | Proto field removal | T2 | `docs/standards/proto3-authoring.md` | major version bump |
| CT-MAT-029 | Regulatory overlay publish | T3 | `docs/standards/regulatory-pack-authzpolicy-overlays.md` | compliance approval |
| CT-MAT-030 | Promotion to dev | T1 | `oya-vcs-admission` | verify evidence |

## Extended Tier Verification Checklist

CT-REV-001. Confirm every capability has a stable `cap.*` id.

CT-REV-002. Confirm the owning µservice is named.

CT-REV-003. Confirm implementation paths exist or are declared planned.

CT-REV-004. Confirm tier escalation is justified by side-effect risk.

CT-REV-005. Confirm T0 actions are read-only or draft-only.

CT-REV-006. Confirm T1 actions are reversible or policy-gated.

CT-REV-007. Confirm T2 actions have audit events and rollback paths.

CT-REV-008. Confirm T3 actions have human approval and compliance owner.

CT-REV-009. Confirm T4 actions have ceremony or executive authority.

CT-REV-010. Confirm forbidden actions are explicitly marked forbidden, not T4.

CT-REV-011. Confirm Cedar actions match capability ids.

CT-REV-012. Confirm OpenAPI operation ids cite capability ids.

CT-REV-013. Confirm AsyncAPI events cite capability ids.

CT-REV-014. Confirm Proto RPCs cite capability ids.

CT-REV-015. Confirm SLOs exist for user-visible capabilities.

CT-REV-016. Confirm FinOps attribution exists for spend capabilities.

CT-REV-017. Confirm provider-BYOK controls exist for provider capabilities.

CT-REV-018. Confirm revocation is possible for delegated authority.

CT-REV-019. Confirm pack overlays cannot silently raise tier ceilings.

CT-REV-020. Confirm promote evidence cites `oya-check-capability-tier-matrix`.

## Extended Capability Evidence Ledger

CT-EVID-001. Record capability id.

CT-EVID-002. Record capability title.

CT-EVID-003. Record owning µservice.

CT-EVID-004. Record owning product axis.

CT-EVID-005. Record assigned tier.

CT-EVID-006. Record previous tier if changed.

CT-EVID-007. Record tier-change rationale.

CT-EVID-008. Record Cedar action id.

CT-EVID-009. Record OpenAPI operation id.

CT-EVID-010. Record AsyncAPI message id.

CT-EVID-011. Record Proto RPC id.

CT-EVID-012. Record implementation crate path.

CT-EVID-013. Record policy file path.

CT-EVID-014. Record SLO file path.

CT-EVID-015. Record runbook file path.

CT-EVID-016. Record audit event names.

CT-EVID-017. Record human-approval requirement.

CT-EVID-018. Record budget policy requirement.

CT-EVID-019. Record revocation mechanism.

CT-EVID-020. Record rollback mechanism.

CT-EVID-021. Record pack overlay constraints.

CT-EVID-022. Record residency constraints.

CT-EVID-023. Record data-class constraints.

CT-EVID-024. Record provider-BYOK constraints.

CT-EVID-025. Record FinOps attribution key.

CT-EVID-026. Record checker crate version.

CT-EVID-027. Record denied capability count.

CT-EVID-028. Record missing-policy count.

CT-EVID-029. Record VCS changeset id.

CT-EVID-030. Record promote bundle id.

## Extended Tier Failure Modes

CT-FAIL-001. Capability has no stable id.

CT-FAIL-002. Capability tier is lower than its side-effect risk.

CT-FAIL-003. Capability omits Cedar action.

CT-FAIL-004. Capability omits audit event.

CT-FAIL-005. Capability omits revocation path.

CT-FAIL-006. Capability permits regulated decision without human gate.

CT-FAIL-007. Capability spends external-provider budget without limit.

CT-FAIL-008. Capability uses marketing name as authority.

CT-FAIL-009. Capability tier differs across docs and policy.

CT-FAIL-010. Capability promote evidence omits checker output.

## Extended Promotion Review Checklist

CT-PROMOTE-001. Capability id is stable.

CT-PROMOTE-002. Capability title is explicit.

CT-PROMOTE-003. Owning µservice is cited.

CT-PROMOTE-004. Product axis is cited.

CT-PROMOTE-005. Assigned tier is justified.

CT-PROMOTE-006. Previous tier is recorded when changed.

CT-PROMOTE-007. Tier-change rationale is recorded.

CT-PROMOTE-008. Cedar action id is present.

CT-PROMOTE-009. OpenAPI operation id is present.

CT-PROMOTE-010. AsyncAPI message id is present.

CT-PROMOTE-011. Proto RPC id is present.

CT-PROMOTE-012. Implementation crate path is present.

CT-PROMOTE-013. Policy file path is present.

CT-PROMOTE-014. SLO file path is present.

CT-PROMOTE-015. Runbook file path is present.

CT-PROMOTE-016. Audit event names are present.

CT-PROMOTE-017. Human approval requirement is explicit.

CT-PROMOTE-018. Budget policy requirement is explicit.

CT-PROMOTE-019. Revocation mechanism is explicit.

CT-PROMOTE-020. Rollback mechanism is explicit.

CT-PROMOTE-021. Pack overlay constraints are explicit.

CT-PROMOTE-022. Residency constraints are explicit.

CT-PROMOTE-023. Data-class constraints are explicit.

CT-PROMOTE-024. Provider-BYOK constraints are explicit.

CT-PROMOTE-025. FinOps attribution key is explicit.

CT-PROMOTE-026. Denied capability count is recorded.

CT-PROMOTE-027. Missing-policy count is zero.

CT-PROMOTE-028. Capability checker output is attached.

CT-PROMOTE-029. VCS changeset id is recorded.

CT-PROMOTE-030. Promote bundle id is recorded.

CT-PROMOTE-031. T4 actions cite ceremony authority.

CT-PROMOTE-032. Forbidden actions are not assigned a tier.

CT-PROMOTE-033. Pack overlays do not raise autonomy silently.

CT-PROMOTE-034. Service manifest names the capability.

CT-PROMOTE-035. Documentation cross-references are current.

CT-PROMOTE-036. Policy tests cover allow and deny.

CT-PROMOTE-037. Regulated decisions require human gate.

CT-PROMOTE-038. External spend has budget guard.

CT-PROMOTE-039. Delegated authority has expiry.

CT-PROMOTE-040. Promotion evidence is reproducible.
