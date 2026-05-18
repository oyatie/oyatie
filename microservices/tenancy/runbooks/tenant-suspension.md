---
doc_class: Runbook
title: Tenant suspension (operational, reversible)
microservice: tenancy
severity: "Sev-3 (single-tenant) / Sev-2 (propagation lag)"
status: Accepted
owner_team: axis-tenancy
date: 2026-05-17
related_artifacts:
  - microservices/tenancy/failure-modes.md (FM-10 suspension propagation lag)
  - microservices/tenancy/incident-response.md
  - microservices/tenancy/PRD.md (FR-04)
doc_status: published
---

# Runbook: Tenant suspension

## Purpose

Suspension is a **non-destructive operational state change**: the tenant's data is preserved; new requests against the tenant are blocked at every µservice via Cedar deny + JWT-claim check; the tenant can be `Resumed` cleanly with no data migration. Distinct from deletion (which is the irreversible DSR cascade per `tenant-deletion-dsr-cascade.md`).

## Trigger

ONE of:
- Tenant non-payment / policy violation: platform-operator initiated.
- Tenant operator self-suspends for maintenance / contractual pause.
- Security incident: ops-security suspends pending forensic investigation.
- Sev-2 propagation lag detected: some µservice still allowing requests after suspension event.

## Severity

- Routine self-suspend or operator-initiated suspend: Sev-3 (or not an incident at all; tracked as ordinary lifecycle).
- Propagation-lag incident: Sev-2.
- Security-incident suspension: Sev-1 (handled per `incident-response.md`).

## Steps (normal suspend)

| Step | Action | Time budget |
|---|---|---|
| 1 | Operator: `POST /tenants/<id>/suspend` (with reason) to tenant-lifecycle-rest | – |
| 2 | tenant-lifecycle-rest: Cedar policy verifies (`suspend_tenant` action permitted for the principal) | ≤ 100 ms |
| 3 | tenant-lifecycle-usecase: state machine `Activated → Suspended`; reason recorded | ≤ 1 s |
| 4 | tenant-lifecycle-adapter-postgres: UPDATE tenant row; audit-chain emit | ≤ 1 s |
| 5 | Workflow `TenantSuspended` event emitted; consumed by every µservice | ≤ 5 s end-to-end propagation |
| 6 | Each µservice's consumer updates its in-memory tenant-status cache; new requests for this tenant_id are denied at REST layer | ≤ 10 s |
| 7 | Status page / tenant operator portal reflects suspended state | ≤ 30 s |

## Recovery Path A — Propagation lag (FM-10)

Cause: some µservice's Workflow event consumer is down / lagging; suspension event not delivered.

| Step | Action |
|---|---|
| 1 | Verify event propagation: `oya_tenancy_event_propagation_lag_seconds{microservice=<>, event="TenantSuspended"} > 60`. |
| 2 | Identify the lagging consumer; engage that µservice's on-call. |
| 3 | If consumer crash-looping: restart pods + clear backlog. |
| 4 | If consumer healthy but lagging: scale consumer replicas; verify backlog drain. |
| 5 | Verify all µservices have processed the event (consult per-µservice lag metric). |
| 6 | If immediate enforcement needed (security-incident): use the global short-circuit via Cedar policy hot-reload to deny by tenant_id at every ingress. |

## Recovery Path B — Suspension reversal (resume)

| Step | Action |
|---|---|
| 1 | Operator: `POST /tenants/<id>/resume`. |
| 2 | Cedar policy verifies same principal class (`resume_tenant` action). |
| 3 | tenant-lifecycle-usecase: state machine `Suspended → Activated`; reason recorded; audit-chain emit. |
| 4 | Workflow `TenantResumed` event emitted; consumed by every µservice (cache update). |
| 5 | Tenant operator portal reflects active state within 30s. |

## Recovery Path C — Erroneous suspension (immediate undo)

If a suspension was issued in error (e.g., wrong tenant ID by an operator):

| Step | Action |
|---|---|
| 1 | Operator immediately initiates `resume` (no penalty; both states logged). |
| 2 | Postmortem on the erroneous-suspension cause (audit-chain log carries both events). |
| 3 | If operator-error pattern recurs: tighten Cedar policy (e.g., require typing the tenant name; require 2-person rule for production-tier suspend). |

## Recovery Path D — Suspension during active DSR

If a tenant is mid-DSR cascade when suspension is initiated:

| Step | Action |
|---|---|
| 1 | DSR continues (not blocked by suspension; deletion is the terminal state). |
| 2 | New non-DSR requests blocked per normal suspension. |
| 3 | If DSR succeeds, terminal state becomes `Deleted` (overrides `Suspended`). |
| 4 | If DSR aborted, tenant remains `Suspended`. |

## Verification

After completion:
- Tenant status reflects expected state in tenant-lifecycle DB.
- All µservices have processed the Workflow event (lag metric ≤ 60s).
- Tenant operator portal reflects state.
- Audit-chain seal log captures the event(s).

## References

- `microservices/tenancy/PRD.md` FR-04.
- `microservices/tenancy/failure-modes.md` FM-10.
- `microservices/tenancy/policy/tenant-scope.cedar` (PERMIT 2).
- `microservices/tenancy/incident-response.md`.
