---
doc_class: Standard
title: Ontology Projection Substrate Standard
status: Accepted
date: 2026-05-20
owner: axis-ontology + council-architecture
related_oyatie_adrs:
  - ADR-0141
  - ADR-0213
  - ADR-0246
  - ADR-0257
  - ADR-0316
enforced_by:
  - oya-governance-ontology-projection-pin
  - oya-governance-capability-tier-ontology-projection-pin
  - oya-governance-cross-microservice-latency-budget
canonical_paths:
  - specs/products/ontology.json
  - microservices/ontology/
  - registry/knowledge-graph-kinetic.json
  - registry/knowledge-graph-dynamic.json
---

# Ontology Projection Substrate Standard

The ontology substrate is the typed projection layer that lets capability tiers
compose across microservices without creating product silos. It is the local
equivalent of the Palantir Foundry ontology pattern: typed objects, typed links,
actions, functions, lineage, and policy-scoped projections over operational
systems.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT,
RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are interpreted
as described in RFC 2119 and RFC 8174 when they appear in all capitals.

## Scope

This standard applies to `microservices/ontology/`.

It applies to `specs/products/ontology.json`.

It applies to ontology object-type definitions.

It applies to ontology link-type definitions.

It applies to ontology action declarations.

It applies to ontology function declarations.

It applies to capability tier ontology projections.

It applies to workflow templates that read or mutate ontology projections.

It applies to Cedar policies that gate ontology actions.

It applies to OpenAPI, AsyncAPI, and proto contracts exposing ontology surfaces.

It does not allow service databases to be replaced by the ontology.

It does not allow synchronous cross-service fan-out on hot paths.

It does not allow product modules to bypass microservice ownership.

## Normative Requirements

O-001. Every ontology object type MUST have a stable id.

O-002. Every ontology object type MUST cite an owning microservice.

O-003. Every ontology object type MUST cite a source system or derived source.

O-004. Every ontology object type MUST declare a schema version.

O-005. Every ontology object type MUST declare data classes.

O-006. Every ontology object type MUST declare tenant scope.

O-007. Every ontology object type MUST declare residency pack behavior.

O-008. Every ontology object type MUST declare lifecycle ownership.

O-009. Every ontology object type MUST declare retention behavior.

O-010. Every ontology object type MUST declare audit events for create, update, delete, and projection refresh.

O-011. Every link type MUST name source object type.

O-012. Every link type MUST name target object type.

O-013. Every link type MUST declare cardinality.

O-014. Every link type MUST declare directionality.

O-015. Every link type MUST declare whether it is materialized or computed.

O-016. Every link type MUST declare stale-read tolerance.

O-017. Every action MUST declare Cedar permit requirements.

O-018. Every action MUST declare Workflow template binding when it mutates state.

O-019. Every action MUST declare idempotency key shape.

O-020. Every action MUST declare audit-chain event shape.

O-021. Every action MUST declare rollback or compensating action.

O-022. Every action MUST declare tenant and principal requirements.

O-023. Every function MUST be side-effect free unless declared as an action.

O-024. Every function MUST declare latency budget.

O-025. Every function MUST declare cacheability.

O-026. Every function MUST declare source freshness.

O-027. Every projection MUST pin source schema versions.

O-028. Every projection MUST pin ontology schema versions.

O-029. Every projection MUST declare refresh trigger.

O-030. Every projection MUST declare replay behavior.

O-031. Every projection MUST declare backfill behavior.

O-032. Every projection MUST declare conflict behavior.

O-033. Every projection MUST declare partial failure behavior.

O-034. Every projection MUST declare observability signals.

O-035. Every projection MUST declare SLO impact.

O-036. Every projection MUST declare cost dimensions.

O-037. Every projection MUST declare pack overlays.

O-038. Every projection MUST declare whether the read path is direct or materialized.

O-039. Direct read paths MUST satisfy `cross-microservice-latency-budget.md`.

O-040. Materialized read paths MUST satisfy freshness SLOs.

O-041. Ontology projections MUST NOT own canonical write state for another microservice.

O-042. Ontology projections MUST NOT bypass source service invariants.

O-043. Ontology projections MUST NOT weaken source service Cedar policy.

O-044. Ontology projections MUST NOT expose regulated attributes without pack mapping.

O-045. Ontology projections MUST NOT create hidden cross-tenant joins.

O-046. Ontology projections MUST NOT invent business terms absent from `docs/GLOSSARY.md`.

O-047. Capability tiers MUST reference ontology projections by id.

O-048. Capability tiers MUST declare why no projection exists when they are workflow-only.

O-049. Workflow templates MUST reference ontology object and action ids.

O-050. Workflow templates MUST not embed raw database table names as ontology substitutes.

O-051. OpenAPI responses SHOULD use ontology ids for tenant-facing object references.

O-052. AsyncAPI events SHOULD carry ontology object ids when events update projections.

O-053. Proto messages SHOULD carry ontology object ids for internal graph operations.

O-054. Audit events MUST include ontology projection ids when projection state changes.

O-055. Projection errors MUST be typed.

O-056. Projection stale reads MUST be visible in telemetry.

O-057. Projection refresh MUST be replayable.

O-058. Projection refresh MUST be resumable.

O-059. Projection deletion MUST respect source service retention.

O-060. Projection redaction MUST follow data-class policy.

## Worked Examples

### Example 1: Approval route projection

```yaml
id: ontology.object.ApprovalRoute.v1
owner_microservice: workflow-engine
source:
  service: workflow-engine
  contract: microservices/workflow-engine/contracts/state-machine-v1.proto
data_classes:
  - BEHAVIORAL_TENANT_PRODUCT
tenant_scope: tenant
actions:
  - ontology.action.ApprovalRoute.approve.v1
  - ontology.action.ApprovalRoute.reject.v1
```

This passes because workflow owns the state and ontology exposes projection.

### Example 2: Customer graph capability tier

```yaml
tenant_class: ["demo_trial", "paid"]
ontology_projection:
  object_types:
    - ontology.object.Account.v1
    - ontology.object.Contact.v1
  link_types:
    - ontology.link.AccountContact.v1
cedar_policy: microservices/ontology/policy/customer-graph-professional.cedar
workflow_template: microservices/workflow-engine/templates/customer-graph-refresh.v1.yaml
```

This keeps CRM as a tier over ontology, community, mail, and workflow rather
than a new product silo.

### Example 3: Invalid direct database join

```sql
SELECT * FROM mail.messages m
JOIN tenancy.tenants t ON t.id = m.tenant_id;
```

This fails because ontology projections cannot bypass service ownership and
tenant-scoped APIs.

### Example 4: Materialized projection refresh

```yaml
projection: ontology.object.MailThread.v1
refresh:
  trigger: EVT-MAIL-THREAD-UPDATED-V1
  replay_topic: mail.thread.events.v1
  freshness_slo: 60s
  backfill: microservices/ontology/runbooks/mail-thread-backfill.md
```

This passes because refresh, replay, and freshness are explicit.

### Example 5: Action binding

```yaml
action: ontology.action.Case.escalate.v1
cedar:
  policy: microservices/ontology/policy/tenant-scope.cedar
workflow:
  template: microservices/workflow-engine/templates/case-escalation-v1.yaml
audit:
  event: EVT-ONTOLOGY-CASE-ESCALATED-V1
```

The action is policy-gated, workflow-backed, and auditable.

## Verification

Primary command:

```bash
oya gate validate ontology-projection-pin --scope microservices
```

The checker MUST parse `specs/products/ontology.json`.

The checker MUST parse capability tier records.

The checker MUST parse workflow template references.

The checker MUST parse Cedar policy references.

The checker MUST parse OpenAPI schema references.

The checker MUST parse AsyncAPI message references.

The checker MUST parse proto message references.

The checker MUST reject unpinned source schema versions.

The checker MUST reject unpinned ontology schema versions.

The checker MUST reject missing source service ownership.

The checker MUST reject missing tenant scope.

The checker MUST reject missing data class.

The checker MUST reject missing Cedar policy on actions.

The checker MUST reject missing workflow binding on mutating actions.

The checker MUST reject missing audit event binding.

The checker MUST reject cross-tenant joins.

The checker MUST reject direct source database references in public projection docs.

The checker MUST reject stale-read tolerance without telemetry.

The checker MUST reject regulated attributes without pack overlay mapping.

The checker SHOULD emit graph traversal evidence.

## Common Anti-Patterns

Treating ontology as the source database is an anti-pattern.

Treating ontology as a cache without schema pins is an anti-pattern.

Treating ontology actions as unaudited helper functions is an anti-pattern.

Treating CRM as a new service when it is a projection tier is an anti-pattern.

Treating object names as free-form labels is an anti-pattern.

Treating link direction as obvious is an anti-pattern.

Treating direct SQL joins as faster than service-owned projections is an anti-pattern.

Treating stale projections as invisible is an anti-pattern.

Treating redaction as UI-only is an anti-pattern.

Treating workflow templates as optional for mutating actions is an anti-pattern.

Treating Cedar policy as an edge-only concern is an anti-pattern.

Treating data-class labels as documentation-only is an anti-pattern.

Treating pack behavior as later compliance work is an anti-pattern.

Treating replay as a best-effort maintenance job is an anti-pattern.

Treating ontology schemas as unversioned JSON examples is an anti-pattern.

## Cross-References

`docs/decisions/ADR-0709-general-live-apex.md` binds direct read path limits.

`docs/decisions/ADR-0709-general-live-apex.md` binds projection pinning.

`docs/decisions/ADR-0709-general-live-apex.md` binds capability tier projection.

`docs/standards/workflow-substrate-engine.md` binds mutating workflow templates.

`docs/standards/capability-tier-matrix.md` binds tier projection manifests.

`docs/standards/cedar-policy-authoring.md` binds action authorization.

`docs/standards/cross-microservice-latency-budget.md` binds direct query budgets.

`specs/products/ontology.json` is the machine-readable ontology source.

## Substance Bar Compliance Checklist

ONT-SB-001. Verify object type id is stable.

ONT-SB-002. Verify object type owner microservice exists.

ONT-SB-003. Verify object type source system exists.

ONT-SB-004. Verify object type schema version is pinned.

ONT-SB-005. Verify object type data classes are declared.

ONT-SB-006. Verify object type tenant scope is declared.

ONT-SB-007. Verify object type residency behavior is declared.

ONT-SB-008. Verify object type lifecycle owner is declared.

ONT-SB-009. Verify object type retention is declared.

ONT-SB-010. Verify object type audit events are declared.

ONT-SB-011. Verify link source object exists.

ONT-SB-012. Verify link target object exists.

ONT-SB-013. Verify link cardinality is declared.

ONT-SB-014. Verify link directionality is declared.

ONT-SB-015. Verify link materialization mode is declared.

ONT-SB-016. Verify action Cedar permit exists.

ONT-SB-017. Verify action Workflow template exists for mutation.

ONT-SB-018. Verify action idempotency key exists.

ONT-SB-019. Verify action audit event exists.

ONT-SB-020. Verify action rollback or compensation exists.

ONT-SB-021. Verify function side-effect posture.

ONT-SB-022. Verify function latency budget.

ONT-SB-023. Verify function cacheability.

ONT-SB-024. Verify function freshness source.

ONT-SB-025. Verify projection source schema pin.

ONT-SB-026. Verify projection ontology schema pin.

ONT-SB-027. Verify projection refresh trigger.

ONT-SB-028. Verify projection replay behavior.

ONT-SB-029. Verify projection backfill behavior.

ONT-SB-030. Verify projection conflict behavior.

ONT-SB-031. Verify projection partial failure behavior.

ONT-SB-032. Verify projection telemetry.

ONT-SB-033. Verify projection SLO impact.

ONT-SB-034. Verify projection cost dimension.

ONT-SB-035. Verify projection pack overlays.

ONT-SB-036. Verify direct read path latency budget.

ONT-SB-037. Verify materialized read path freshness SLO.

ONT-SB-038. Verify no source database ownership drift.

ONT-SB-039. Verify no source invariant bypass.

ONT-SB-040. Verify no Cedar policy weakening.

ONT-SB-041. Check `ontology.object.ApprovalRoute.v1`.

ONT-SB-042. Check `ontology.object.Account.v1`.

ONT-SB-043. Check `ontology.object.Contact.v1`.

ONT-SB-044. Check `ontology.object.MailThread.v1`.

ONT-SB-045. Check `ontology.object.TenantGrant.v1`.

ONT-SB-046. Check `ontology.link.AccountContact.v1`.

ONT-SB-047. Check `ontology.action.ApprovalRoute.approve.v1`.

ONT-SB-048. Check `ontology.action.Case.escalate.v1`.

ONT-SB-049. Check `workflow-engine.approval-routing.professional`.

ONT-SB-050. Check `crm.customer-graph.professional`.

ONT-SB-051. Reject direct SQL joins across services.

ONT-SB-052. Reject unversioned object types.

ONT-SB-053. Reject missing data classes.

ONT-SB-054. Reject hidden cross-tenant joins.

ONT-SB-055. Reject UI-only redaction.

ONT-SB-056. Reject projection refresh without replay.

ONT-SB-057. Reject mutable action without Workflow.

ONT-SB-058. Reject action without Cedar.

ONT-SB-059. Reject capability tier without projection posture.

ONT-SB-060. Reject regulated attribute without pack overlay.

ONT-SB-061. Emit object type count.

ONT-SB-062. Emit link type count.

ONT-SB-063. Emit action count.

ONT-SB-064. Emit function count.

ONT-SB-065. Emit projection count.

ONT-SB-066. Emit direct-read count.

ONT-SB-067. Emit materialized-read count.

ONT-SB-068. Emit stale-read tolerance count.

ONT-SB-069. Emit capability tier reference count.

ONT-SB-070. Emit policy binding count.

ONT-SB-071. Preserve source service ownership.

ONT-SB-072. Preserve ontology as projection, not database.

ONT-SB-073. Preserve Cedar policy at action boundary.

ONT-SB-074. Preserve Workflow template at mutation boundary.

ONT-SB-075. Preserve audit event at projection mutation.

ONT-SB-076. Preserve data class at every object type.

ONT-SB-077. Preserve pack overlay at regulated attribute.

ONT-SB-078. Preserve replay for projection refresh.

ONT-SB-079. Preserve schema pins for every projection.

ONT-SB-080. Preserve cross-service latency budget for direct reads.

## Extended Worked Example: Tenant Ontology Projection

The following projection turns canonical tenant facts into read models used by
policy, billing, search, and workflow routing. The projection is derived; it is
not the authority for tenant state.

```yaml
projection_id: tenant-access-context-v1
authority:
  service: tenancy
  aggregate: TenantAccount
  event_stream: tenant.scope.updated.v1
derived_for:
  - service: policy-engine
    path: microservices/policy-engine/projections/tenant-access-context-v1.yaml
  - service: workflow-engine
    path: microservices/workflow-engine/projections/tenant-access-context-v1.yaml
  - service: observability
    path: microservices/observability/projections/tenant-access-context-v1.yaml
related_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/adr-archive/ADR-0258-api-versioning-model.md
  - docs/decisions/ADR-0709-general-live-apex.md
source_events:
  - TenantCreatedV1
  - TenantPackAttachedV1
  - TenantResidencyChangedV1
  - TenantCapabilityTierChangedV1
fields:
  tenant_id:
    type: TenantId
    data_class: internal
    source: TenantCreatedV1.tenant_id
  active_packs:
    type: list<PackId>
    data_class: regulated
    source: TenantPackAttachedV1.pack_id
  residency:
    type: ResidencyRegion
    data_class: regulated
    source: TenantResidencyChangedV1.region
  capability_ceiling:
    type: CapabilityTier
    data_class: internal
    source: TenantCapabilityTierChangedV1.tier
rebuild:
  command: cargo run -p oya-projection-rebuild -- tenant-access-context-v1
  checkpoint_store: projections.tenant_access_context_checkpoint
  replay_order: event_time_then_sequence
```

## Extended Projection Compliance Matrix

| ID | Projection concern | Requirement | Example path | Checker |
|---|---|---|---|---|
| ONT-MAT-001 | Authority | Projection cites owning aggregate | `TenantAccount` | `oya-check-projection-authority` |
| ONT-MAT-002 | Authority | Projection cites event stream | `tenant.scope.updated.v1` | `oya-check-event-source-linkage` |
| ONT-MAT-003 | Authority | Projection rejects direct writes | repository adapter | `oya-check-projection-write-paths` |
| ONT-MAT-004 | Schema | Every field has data class | projection YAML | `oya-check-data-class` |
| ONT-MAT-005 | Schema | Every field has source event | projection YAML | `oya-check-projection-field-source` |
| ONT-MAT-006 | Schema | Every derived enum has version | projection YAML | `oya-check-projection-versioning` |
| ONT-MAT-007 | Replay | Rebuild command is declared | projection manifest | `oya-check-projection-rebuild` |
| ONT-MAT-008 | Replay | Checkpoint store is declared | projection manifest | `oya-check-projection-checkpoint` |
| ONT-MAT-009 | Replay | Replay order is deterministic | projection manifest | `oya-check-projection-replay-order` |
| ONT-MAT-010 | Consistency | Lag SLO is declared | OpenSLO file | `oya-check-projection-slo` |
| ONT-MAT-011 | Consistency | Stale reads are labeled | API response | `oya-check-staleness-label` |
| ONT-MAT-012 | Consistency | Strong-read fallback is explicit | direct gRPC rubric | `oya-check-direct-read-rubric` |
| ONT-MAT-013 | Policy | Cedar schema imports projection shape | `policy/schema.cedarschema` | `oya-check-cedar-projection` |
| ONT-MAT-014 | Policy | Policy denies stale regulated fields | Cedar policy | `oya-check-stale-policy-deny` |
| ONT-MAT-015 | Search | Search indexes derived fields only when allowed | search mapping | `oya-check-search-data-class` |
| ONT-MAT-016 | Billing | Billing projection cites tariff authority | billing manifest | `oya-check-billing-projection` |
| ONT-MAT-017 | Workflow | Workflow reads projection through port | usecase port | `oya-check-workflow-projection-port` |
| ONT-MAT-018 | Observability | Projection lag metric is emitted | metric name | `oya-check-projection-metrics` |
| ONT-MAT-019 | Audit | Rebuild emits audit event | `EVT-PROJECTION-REBUILT` | `oya-check-audit-emission` |
| ONT-MAT-020 | Migration | Breaking shape change bumps version | projection id | `oya-check-projection-versioning` |
| ONT-MAT-021 | Fixture | Fixture includes source events | fixtures path | `oya-check-projection-fixtures` |
| ONT-MAT-022 | Test | Replay test covers reordering | test module | `oya-check-projection-tests` |
| ONT-MAT-023 | Docs | Cross-reference cites standard | ADR/PRD | `oya-check-doc-links` |
| ONT-MAT-024 | Rollback | Old projection is retained during migration | deployment plan | `oya-check-projection-rollback` |
| ONT-MAT-025 | Retention | Projection retention matches source data class | retention file | `oya-check-retention-parity` |
| ONT-MAT-026 | Residency | Projection storage region matches source | cell manifest | `oya-check-residency-parity` |
| ONT-MAT-027 | Tenant | Tenant id is mandatory on tenant projections | schema | `oya-check-tenant-boundary` |
| ONT-MAT-028 | Pack | Pack overlays are explicit | projection manifest | `oya-check-pack-overlay` |
| ONT-MAT-029 | Consumer | Consumer compatibility is tested | contract test | `oya-check-consumer-compat` |
| ONT-MAT-030 | Promote | Evidence names changed projections | VCS bundle | `oya-vcs-admission` |

## Extended Review Questions

ONT-REV-001. Is the projection clearly marked derived?

ONT-REV-002. Is the source aggregate named?

ONT-REV-003. Is the source event stream named?

ONT-REV-004. Is every field mapped to a source event field?

ONT-REV-005. Is every regulated field labeled with data class?

ONT-REV-006. Is replay deterministic under duplicate events?

ONT-REV-007. Is replay deterministic under out-of-order delivery?

ONT-REV-008. Is projection lag observable?

ONT-REV-009. Is the stale-read contract visible to callers?

ONT-REV-010. Is direct read fallback justified by latency budget?

ONT-REV-011. Does Cedar policy consume the projection through schema?

ONT-REV-012. Does workflow logic consume projection through a port?

ONT-REV-013. Does search indexing respect data-class restrictions?

ONT-REV-014. Does billing use only tariff-authoritative fields?

ONT-REV-015. Does rebuild emit audit evidence?

ONT-REV-016. Does migration keep old and new projections side by side?

ONT-REV-017. Does rollback preserve checkpoint compatibility?

ONT-REV-018. Does retention match source event retention?

ONT-REV-019. Does residency match tenant pack requirements?

ONT-REV-020. Does promote evidence include projection checker output?

## Extended Projection Evidence Ledger

ONT-EVID-001. Record projection id.

ONT-EVID-002. Record projection version.

ONT-EVID-003. Record source aggregate.

ONT-EVID-004. Record source event stream.

ONT-EVID-005. Record source event schema version.

ONT-EVID-006. Record target µservice.

ONT-EVID-007. Record target storage path.

ONT-EVID-008. Record replay command.

ONT-EVID-009. Record replay fixture path.

ONT-EVID-010. Record checkpoint store name.

ONT-EVID-011. Record deterministic ordering rule.

ONT-EVID-012. Record data-class coverage percentage.

ONT-EVID-013. Record regulated-field count.

ONT-EVID-014. Record pack-overlay count.

ONT-EVID-015. Record residency binding.

ONT-EVID-016. Record retention binding.

ONT-EVID-017. Record projection lag SLI.

ONT-EVID-018. Record projection lag SLO.

ONT-EVID-019. Record stale-read label.

ONT-EVID-020. Record direct-read fallback.

ONT-EVID-021. Record Cedar schema import.

ONT-EVID-022. Record workflow port import.

ONT-EVID-023. Record search-index allowlist.

ONT-EVID-024. Record billing projection allowlist.

ONT-EVID-025. Record audit event name.

ONT-EVID-026. Record rebuild audit event id.

ONT-EVID-027. Record migration source version.

ONT-EVID-028. Record migration target version.

ONT-EVID-029. Record rollback projection id.

ONT-EVID-030. Record consumer compatibility test id.

ONT-EVID-031. Record replay duplicate-event test id.

ONT-EVID-032. Record replay reorder-event test id.

ONT-EVID-033. Record missing-source-field findings.

ONT-EVID-034. Record unknown-data-class findings.

ONT-EVID-035. Record unauthorized-write findings.

ONT-EVID-036. Record projection owner team.

ONT-EVID-037. Record source service owner team.

ONT-EVID-038. Record runbook link.

ONT-EVID-039. Record checker crate version.

ONT-EVID-040. Record VCS changeset id.

## Extended Projection Failure Modes

ONT-FAIL-001. Projection writes become source of truth.

ONT-FAIL-002. Projection field lacks source event.

ONT-FAIL-003. Projection stores regulated field without data class.

ONT-FAIL-004. Projection replay depends on wall-clock time.

ONT-FAIL-005. Projection replay depends on broker partition order only.

ONT-FAIL-006. Projection lag has no SLO.

ONT-FAIL-007. Projection stale read is invisible to caller.

ONT-FAIL-008. Projection migrates without old-version support.

ONT-FAIL-009. Projection rebuild skips audit emission.

ONT-FAIL-010. Projection consumer treats derived data as canonical.

## Extended Promotion Review Checklist

ONT-PROMOTE-001. Projection id is stable.

ONT-PROMOTE-002. Projection version is explicit.

ONT-PROMOTE-003. Source aggregate is cited.

ONT-PROMOTE-004. Source event stream is cited.

ONT-PROMOTE-005. Source schema version is cited.

ONT-PROMOTE-006. Target service is cited.

ONT-PROMOTE-007. Target storage is cited.

ONT-PROMOTE-008. Replay command is executable.

ONT-PROMOTE-009. Checkpoint store is declared.

ONT-PROMOTE-010. Replay order is deterministic.

ONT-PROMOTE-011. Data-class labels are complete.

ONT-PROMOTE-012. Regulated fields are enumerated.

ONT-PROMOTE-013. Pack overlays are enumerated.

ONT-PROMOTE-014. Residency binding is explicit.

ONT-PROMOTE-015. Retention binding is explicit.

ONT-PROMOTE-016. Projection lag metric exists.

ONT-PROMOTE-017. Projection lag SLO exists.

ONT-PROMOTE-018. Stale-read behavior is documented.

ONT-PROMOTE-019. Direct-read fallback is justified.

ONT-PROMOTE-020. Cedar schema import is tested.

ONT-PROMOTE-021. Workflow port import is tested.

ONT-PROMOTE-022. Search allowlist is tested.

ONT-PROMOTE-023. Billing allowlist is tested.

ONT-PROMOTE-024. Rebuild audit event is tested.

ONT-PROMOTE-025. Migration keeps prior version.

ONT-PROMOTE-026. Rollback uses prior checkpoint.

ONT-PROMOTE-027. Consumer fixture exists.

ONT-PROMOTE-028. Duplicate replay fixture exists.

ONT-PROMOTE-029. Reordered replay fixture exists.

ONT-PROMOTE-030. VCS evidence includes projection checker output.
