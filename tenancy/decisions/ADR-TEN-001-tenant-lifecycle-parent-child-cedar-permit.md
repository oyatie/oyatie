---
id: ADR-TEN-001
title: Tenant Lifecycle State Machine and Parent-Child Cedar Permit Model
status: Accepted
date: 2026-05-20
microservice: tenancy
related_oyatie_adrs:
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
decision_owner: axis-tenancy
---

# ADR-TEN-001: Tenant Lifecycle State Machine and Parent-Child Cedar Permit Model

## Context

- Tenancy is a substrate microservice that owns tenant lifecycle, tenant scope, RLS policy emission, cell assignment, DSR cascade, quota, DR pairing, KYB/KYC, lifecycle locks, and sub-scope registry.
- Existing IPs include `IP-002-tenant-lifecycle-kernel.md`, `IP-004-tenant-lifecycle-usecase.md`, `IP-006-isolation-policy-rls-generator.md`, `IP-016-sub-scope-registry-kernel.md`, and `IP-021-lifecycle-locks-kernel.md`.
- Existing policies include `tenant-scope.cedar`, `lifecycle.md`, `rls-isolation.md`, `action-authorization.cedar`, and `data-residency.cedar`.
- ADR-0313 introduces conglomerate tenant hierarchy and sovereign children; this ADR binds the tenancy-local state machine and permit enforcement.
- Named precedent: AWS Organizations accounts and organizational units separate parent management authority from child account data-plane access.
- Named precedent: Stripe separates platform facilitation from connected-account ownership and capabilities.
- Named precedent: Google Cloud Resource Manager folders and projects encode hierarchy while IAM controls delegation.
- Constraint TEN-C1: Tenant is a universal substrate primitive per ADR-0002 and must not be redefined per product.
- Constraint TEN-C2: every lifecycle transition and parent-child grant mutation must emit audit evidence per ADR-0003.
- Constraint TEN-C3: parent-child authority is represented as Cedar permits, not as implicit SQL joins or inherited admin booleans, per ADR-0007.
- Constraint TEN-C4: cell assignment and region binding follow ADR-0009 and cannot be bypassed by parent hierarchy.
- Constraint TEN-C5: ADR-0313 requires sovereign children that can be controlled but not data-pierced by parents without explicit grants.
- Constraint TEN-C6: tenant creation must support personal, work, marketplace, healthcare, regulated, and internal substrate contexts.
- Constraint TEN-C7: lifecycle states must be monotonic where compliance demands it, especially suspension, offboarding, and legal hold.
- Constraint TEN-C8: offboarding must cascade to data-bearing services without leaving hidden active grants.
- Constraint TEN-C9: parent-child relationships must be time-bounded, purpose-bounded, and revocable.
- Constraint TEN-C10: a parent tenant can administer billing or policy for a child without automatically reading child data.
- Constraint TEN-C11: child tenant sovereignty must win over parent convenience when packs conflict.
- Constraint TEN-C12: tenant lifecycle must be observable enough for Ops Dashboard and compliance evidence.
- Constraint TEN-C13: tenant lifecycle changes must be idempotent and replayable across services.
- Constraint TEN-C14: trial, KYB/KYC, DR pair, quota, and residency gates must be state-machine guards.
- Constraint TEN-C15: lifecycle locks must freeze risky transitions during incidents.
- The architecture must support parent-child acquisition, divestiture, regional subsidiary, franchise, agency, benefits-provider, and engagement scopes.
- The architecture must support both service-local RLS and Cedar library evaluation.
- The architecture must make junior implementers able to build the transition table without reading ADR-0313 prose alone.

## Decision

- Define one authoritative tenant lifecycle state machine inside tenancy.
- Use states: `requested`, `kyb_pending`, `provisioning`, `active`, `restricted`, `suspended`, `offboarding`, `retained`, `cryptoshredded`, and `retired`.
- Use explicit transition commands; no service may mutate tenant lifecycle by direct row update.
- Require every transition command to evaluate Cedar and lifecycle guards before writing.
- Represent parent-child relationship as `TenantRelationship` plus `TenantPermitGrant`.
- Support relationship types: `owns`, `controls`, `manages_billing`, `manages_policy`, `audits`, `services`, and `delegated_operator`.
- Store parent-child edges as purpose-scoped, time-scoped, pack-scoped grants.
- Keep data-plane access separate from management-plane access.
- Make parent data access deny-by-default even for `owns`.
- Require a separate Cedar permit for child data read, child policy mutation, child billing read, child user admin, and child export.
- Apply higher-restriction-wins when parent and child packs conflict.
- For sovereign children, deny parent data-plane access unless the child pack explicitly permits the action and the child tenant grants it.
- Use lifecycle locks to prevent transitions while investigations, legal holds, incident freezes, or regulator holds are active.
- Assign tenant home cell during `provisioning` and make it immutable after `active` except through migration ceremony.
- Publish tenant lifecycle changes through the canonical eventing backbone.
- Use DSR cascade events when a tenant enters `offboarding`.
- Use cryptoshred plan events when a tenant moves from `retained` to `cryptoshredded`.
- Keep `retired` tenant metadata for audit, invoices, and regulator evidence.
- Generate RLS policies from the same `tenant_id` and relationship model but never rely on RLS as the only guard.
- Make Cedar the primary authorization guard; RLS is defense-in-depth.
- Expose a typed API for parent-child grant request, approval, revocation, and audit.
- Make grant revocation immediate for new actions and eventually consistent for long-running workflows through cancellation events.
- Make all relationship and lifecycle commands idempotent by command id.
- Make offboarding resumable from a transition ledger.

## Alternatives Considered

### Simple Active / Suspended / Deleted States

- Pros: easy to implement.
- Pros: matches many SaaS account systems.
- Pros: fewer support states.
- Cons: cannot model KYB, provisioning, offboarding, retained, cryptoshredded, and retired obligations.
- Cons: offboarding and legal hold become hidden flags.
- Cons: product services invent local lifecycle semantics.
- Rejected because tenancy is substrate and lifecycle must drive compliance and data operations.

### SQL Parent ID with Implicit Inheritance

- Pros: simple hierarchy queries.
- Pros: easy UI for conglomerates.
- Pros: conventional enterprise account model.
- Cons: parent authority becomes ambiguous and overbroad.
- Cons: child sovereignty cannot be represented safely.
- Cons: auditing why access was allowed becomes difficult.
- Rejected because ADR-0313 requires explicit parent-child grants and sovereign children.

### Separate Tenant Graph Service

- Pros: graph traversal becomes specialized.
- Pros: complex conglomerate modeling can evolve independently.
- Pros: may scale relationship reads separately.
- Cons: splits tenant lifecycle from tenant relationship authority.
- Cons: creates a second substrate source of truth.
- Cons: cross-service consistency and bootstrapping become harder.
- Rejected for Batch A; relationship graph stays in tenancy with projection APIs.

### Product-Service Local Tenant Lifecycle

- Pros: product teams move faster.
- Pros: product-specific states can be modeled directly.
- Pros: fewer substrate dependencies initially.
- Cons: violates universal tenancy primitive.
- Cons: produces inconsistent offboarding and compliance behavior.
- Cons: parent-child grants drift by product.
- Rejected because all products must consume one tenancy substrate.

## Consequences

- Positive: every service sees the same tenant lifecycle state and transition evidence.
- Positive: parent-child authority is explicit, revocable, and audit-friendly.
- Positive: sovereign child tenants can be controlled administratively without automatic data access.
- Positive: offboarding, retained, cryptoshredded, and retired states become observable.
- Positive: RLS and Cedar derive from one substrate model.
- Positive: higher-restriction-wins is enforceable during conglomerate conflict.
- Positive: lifecycle locks give incident response a safe freeze primitive.
- Positive: services can replay transition events to repair projections.
- Negative: tenant onboarding has more states and can take longer.
- Negative: management UIs must explain relationship grants rather than a simple parent admin toggle.
- Negative: relationship queries may need caching for large conglomerates.
- Negative: grant revocation during long-running workflows needs cancellation handling.
- Negative: migration from any local tenant flags requires cleanup.
- Neutral: products can add local readiness states, but they cannot replace substrate lifecycle.
- Neutral: billing hierarchy and data hierarchy are deliberately separate.
- Neutral: child tenants can delegate policy management without delegating data access.
- Neutral: tenant migration remains a separate council-approved ceremony.
- Neutral: retained and retired records persist for audit even after cryptoshred.

## Implementation Notes

- Data shape `TenantLifecycle`: `{tenant_id, state, state_version, home_cell, residency_class, pack_set, lifecycle_lock_ids[], updated_at}`.
- Data shape `TenantTransitionCommand`: `{command_id, tenant_id, from_state, to_state, requested_by, reason, guard_snapshot_hash, idempotency_key}`.
- Data shape `TenantRelationship`: `{relationship_id, parent_tenant_id, child_tenant_id, relationship_type, starts_at, ends_at, pack_scope, state}`.
- Data shape `TenantPermitGrant`: `{grant_id, relationship_id, action_namespace, resource_scope, purpose, expires_at, cedar_policy_ref, state}`.
- Data shape `LifecycleLock`: `{lock_id, tenant_id, lock_type, reason, created_by, expires_at, release_policy_ref}`.
- REST endpoint `POST /v1/tenancy/tenants` creates a requested tenant.
- REST endpoint `POST /v1/tenancy/tenants/{tenant_id}/transitions` runs lifecycle transition.
- REST endpoint `GET /v1/tenancy/tenants/{tenant_id}/lifecycle` reads lifecycle state.
- REST endpoint `POST /v1/tenancy/relationships` creates parent-child relationship proposal.
- REST endpoint `POST /v1/tenancy/relationships/{relationship_id}/permits` creates a scoped permit.
- REST endpoint `DELETE /v1/tenancy/permits/{grant_id}` revokes a scoped permit.
- REST endpoint `POST /v1/tenancy/tenants/{tenant_id}/locks` creates lifecycle lock.
- REST endpoint `DELETE /v1/tenancy/locks/{lock_id}` releases lifecycle lock.
- AsyncAPI channel `tenancy.lifecycle.transitioned.v1` publishes state changes.
- AsyncAPI channel `tenancy.relationship.created.v1` publishes hierarchy edge creation.
- AsyncAPI channel `tenancy.permit.granted.v1` publishes scoped grant.
- AsyncAPI channel `tenancy.permit.revoked.v1` publishes revocation.
- AsyncAPI channel `tenancy.offboarding.cascade.requested.v1` publishes product-service offboarding commands.
- Cedar permit `tenancy::lifecycle::transition` requires current state, requested state, actor role, and clear locks.
- Cedar forbid `tenancy::relationship::data_read` unless grant purpose explicitly includes data access.
- Cedar permit `tenancy::relationship::policy_manage` can exist without `data_read`.
- Cedar forbid `tenancy::relationship::parent_override_sovereign_child` when child pack denies parent access.
- Cedar permit `tenancy::lock::release` requires lock owner or incident commander role.
- Audit event `EVT-TEN-LIFECYCLE-TRANSITIONED` includes old state, new state, guards, and command id.
- Audit event `EVT-TEN-RELATIONSHIP-CREATED` includes relationship type and pack scope.
- Audit event `EVT-TEN-PERMIT-GRANTED` includes action namespace, purpose, and expiry.
- Audit event `EVT-TEN-PERMIT-REVOKED` includes cancellation policy.
- Metric `tenancy_lifecycle_transition_latency_ms` tracks transition command latency.
- Metric `tenancy_relationship_permit_count` tracks active grants by type.
- Metric `tenancy_lifecycle_lock_active_total` tracks active locks.
- Metric `tenancy_offboarding_cascade_lag_seconds` tracks downstream projection completion.
- Capacity math: a conglomerate with 50k child tenants and 10 permits each has 500k permit rows; cache permit summaries by parent and child to keep decision prefetch below 50 ms.
- Capacity math: offboarding 10k tenants/day with 20 downstream services means 200k cascade commands/day; Kafka partition by tenant to preserve per-tenant order.
- Rollback path: invalid relationship grant revocation is immediate; prior allowed actions are preserved only as audit evidence.
- Rollback path: accidental transition to `suspended` can transition back to `restricted` or `active` only if guards still pass.
- Multi-region path: lifecycle source of truth stays in tenant home cell; read projections replicate metadata only.
- Sovereign path: child tenant pack restrictions are evaluated before parent permit grants.
- Versioning: lifecycle state machine version is `tenant-lifecycle-v1`.
- Deprecation: states are never removed; old states can be terminally mapped through migration events.

## Verification

- Unit test `active_transition_requires_kyb_complete_and_home_cell` validates provisioning guards.
- Unit test `parent_owns_does_not_imply_data_read` validates grant separation.
- Unit test `sovereign_child_denies_parent_override` validates ADR-0313 pressure.
- Unit test `lifecycle_lock_blocks_offboarding` validates freeze behavior.
- Unit test `transition_command_idempotent` validates command replay.
- Property test `state_machine_has_no_unapproved_back_edges` generates transition paths.
- Property test `permit_expiry_removes_authority` checks time-bound grants.
- Fuzz test `relationship_graph_rejects_cycles` prevents hierarchy loops.
- Integration test `offboarding_publishes_cascade_events` verifies downstream handoff.
- Integration test `grant_revocation_publishes_cancellation` verifies long-running workflow cancellation.
- Integration test `rls_projection_matches_cedar_scope` verifies defense-in-depth parity.
- Integration test `retained_to_cryptoshredded_requires_clearance` verifies compliance guards.
- Load test `permit_prefetch_50k_children` keeps p95 below 50 ms.
- Load test `offboarding_10k_tenants_per_day` keeps cascade lag below 15 minutes.
- Chaos test `audit_chain_unavailable_blocks_lifecycle_transition` proves evidence-first behavior.
- Chaos test `relationship_projection_rebuild_from_events` proves replayability.
- Metric SLO: `tenancy_lifecycle_transition_latency_ms` p95 below 100 ms.
- Metric SLO: `tenancy_offboarding_cascade_lag_seconds` p95 below 900 seconds.
- Metric SLO: active lifecycle locks older than expiry equals zero.
- Audit check: every transition has one `EVT-TEN-LIFECYCLE-TRANSITIONED`.
- Audit check: every active permit has a relationship id, purpose, expiry, and Cedar policy reference.
- Static check: product services cannot define their own tenant lifecycle enums as authoritative.
- Static check: parent-child SQL joins are not used as authorization decisions without Cedar.
- Contract check: OpenAPI documents all lifecycle states and transition errors.

