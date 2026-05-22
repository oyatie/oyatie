---
doc_class: IP
ip_id: IP-016-backfill-replay-worker
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + data-migration
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/src/usecase/mod.rs
  - microservices/itsm/src/adapter/mod.rs
  - microservices/itsm/contracts/asyncapi-v1.yaml
  - microservices/itsm/ARCHITECTURE.md
---

# IP-016 ITSM Backfill Replay Worker

## A. Problem
Tenant migrations from ServiceNow, Jira Service Management, or Freshservice require replaying tickets, changes, CMDB relations, KB articles, and postmortems without corrupting live tenant state. The stamped IP never named source systems, replay units, idempotency, or rollback.

This IP defines an async worker that imports and replays historical ITSM records as audited, idempotent batches.

## B. Approach
Backfill is not a direct database write. It is a worker path that converts source records into normal ITSM commands where possible and emits rejected-row evidence where not possible.

| Source object | Oyatie target |
|---|---|
| ServiceNow incident/problem/change | `IncidentTicket` plus linked problem/change state |
| Jira JSM issue/request | incident or service request with requester id |
| Freshservice asset | CMDB CI relation candidate |
| ServiceNow KB article | knowledge-base draft state |
| post-incident review | postmortem draft/action items |

## C. Deliverables
- Worker batch manifest shape with source system, source id, tenant id, dry-run flag, and replay id.
- Idempotency key strategy for every imported ticket and relation.
- Rejection report format naming row, field, transform, data class, and reason.
- AsyncAPI event for batch accepted, row rejected, row replayed, and rollback completed.
- Tests for duplicate replay and tenant mismatch.

## D. Implementation
1. Define backfill batch DTOs under a worker module rather than overloading public REST action payloads.
2. Parse source records into domain value objects using `TenantId::parse`, `TicketId::parse`, and `RequesterId::parse`.
3. Route valid incident rows through `OpenIncident::execute` so policy and audit behavior is reused.
4. For changes and SLA recompute, reuse `ApproveChange` and `RecomputeSla` where source history maps safely.
5. Store source-system aliases separately; source ids never become Oyatie authority.
6. Emit rejection evidence for unknown field, invalid tenant, unsupported status, or pack residency block.
7. Make replay idempotent by `(tenant_id, source_system, source_object_type, source_id, source_version)`.
8. Implement rollback as detaching imported aliases and replayed projections, not deleting signed audit history.

## E. Acceptance
- Replaying the same batch twice produces no duplicate tickets or audit success events.
- A dry run produces rejection evidence without mutating state.
- ServiceNow/Jira/Freshservice ids are aliases only.
- Pack residency failures stop row import and produce redacted denial evidence.

## F. Evidence
- `ARCHITECTURE.md` already names source-system import drift and replay flows.
- `src/usecase/mod.rs` exposes reusable command handlers.
- `contracts/asyncapi-v1.yaml` is the event surface for replay status.
- ADR-0244 and ADR-0263 govern tenant scoping and audit evidence.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow migration/import sets | Idempotent replay through normal ITSM usecases |
| Jira Service Management issue import | Source ids remain aliases, not authority |
| Freshservice asset/ticket import | Row-level rejection evidence and rollback bundle |

## H. Cold-start buildability notes
- Start with dry-run only; enable mutation after rejection reports are stable.
- Use `(tenant_id, source_system, source_id, source_version)` for idempotency.
- Route valid incident rows through current usecases.
- Keep source ids as aliases in every structure.
- Add duplicate replay tests before large source fixtures.
- Treat residency denial as row rejection, not batch success.
- Preserve signed audit history during rollback.
- Separate import credentials from ticket payloads.
- Add source-system enums for ServiceNow, Jira, and Freshservice only when adapters exist.
- Keep progress events typed through AsyncAPI.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`asyncapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
