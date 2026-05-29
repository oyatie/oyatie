---
doc_class: IP
ip_id: IP-008-policy-eval-library-binding
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + axis-policy-engine
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/policy/service-management-authorization.cedar
  - microservices/itsm/policies/local-change-approval-window.cedar
  - microservices/itsm/src/usecase/mod.rs
  - microservices/itsm/src/domain/mod.rs
---

# IP-008 ITSM Policy Evaluation Library Binding

## A. Problem
ITSM mutations cannot be trusted if the policy check is hidden in the HTTP adapter or described only in prose. The service already has `PolicyAuthorizer` in `src/usecase/mod.rs`; this IP turns that port into the explicit binding point for Cedar evaluation across incident, SLA, change, CMDB, service catalog, and knowledge publish flows.

ServiceNow ACLs, Jira project permissions, and Freshservice roles are feature-rich but vendor-local. Oyatie's requirement is stronger: every ITSM action evaluates the shared Cedar corpus and emits denial evidence that downstream audit tools can replay.

## B. Approach
Bind usecase capability enums to Cedar action names:

| Capability | Cedar action candidate | Local policy evidence |
|---|---|---|
| `IncidentOpen` | `itsm.incident.open` | `policies/local-incident-ticket-scope.cedar` |
| `SlaRecompute` | `itsm.sla.recompute` | `policies/local-sla-recompute-guard.cedar` |
| `ProblemLink` | `itsm.problem.link` | `policies/local-problem-link-control.cedar` |
| `ChangeApprove` | `itsm.change.approve` | `policies/local-change-approval-window.cedar` |
| `CmdbSync` | `itsm.cmdb.sync` | `policies/local-cmdb-relation-write.cedar` |
| `ServiceCatalogPublish` | `itsm.service_catalog.publish` | `policy/service-management-authorization.cedar` |

The implementation should evaluate before domain mutation and before audit publication. Denials must be explicit outcomes, not absent events.

## C. Deliverables
- A mapping function from `Capability::action_slug()` to Cedar action names in `src/domain/mod.rs`.
- A production `PolicyAuthorizer` adapter, likely under `src/adapter/mod.rs` or a future policy adapter module.
- Tests that prove `OpenIncident`, `RecomputeSla`, and `ApproveChange` call `authorize` before repository writes.
- A README or inline evidence row in this IP linking each local Cedar file to one capability.
- Denial audit event taxonomy aligned with ADR-0263.

## D. Implementation
1. Audit `Capability::action_slug()` in `src/domain/mod.rs` and ensure every usecase action has one stable slug.
2. Add a Cedar request context containing `tenant_id`, `principal_id`, `audience_type`, `data_class`, `home_cell`, and `jurisdiction_code`.
3. Implement a concrete authorizer adapter that receives `TenantId` + `Capability` from `PolicyAuthorizer::authorize`.
4. Wire local Cedar fragments from `microservices/itsm/policy/` and `microservices/itsm/policies/` into the adapter load list.
5. Add unit tests that inject a denying authorizer and assert no repository write or audit publish occurs.
6. Add an allow-path test for `OpenIncident::execute` using `InMemoryItsmPorts`.
7. Add a denial evidence event name such as `itsm.policy.denied` without overloading success events.
8. Update `contracts/openapi-v1.yaml` error responses once the error body exists.

## E. Acceptance
- Every mutating usecase calls `PolicyAuthorizer::authorize` before state change.
- Denied actions return deterministic errors and emit or queue denial evidence.
- Local Cedar files named in this IP resolve.
- Tests cover both allowed and denied paths for at least `IncidentOpen` and `ChangeApprove`.

## F. Evidence
- `src/usecase/mod.rs` already requires `PolicyAuthorizer` for the service port.
- `src/domain/mod.rs` already defines `Capability::{IncidentOpen,SlaRecompute,ProblemLink,ChangeApprove,CmdbSync,ServiceCatalogPublish}`.
- `policy/service-management-authorization.cedar` and local Cedar fragments provide concrete policy surfaces.
- ADR-0243 is the shared Cedar gate doctrine; ADR-0244 supplies tenant scope.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow ACL / role rules | Shared Cedar policy rather than service-local ACL sprawl |
| Jira Service Management project permissions | Tenant and audience rules checked at usecase boundary |
| Freshservice agent/requester roles | Denials become replayable audit evidence |

## H. Cold-start buildability notes
- Use `Capability::action_slug()` as the single source for action names.
- Add a denying fake authorizer before wiring a production Cedar engine.
- Check repository write counts in tests to prove deny-before-mutate ordering.
- Keep denial errors distinct from budget and capacity denials.
- Keep local policy fragments in their existing `policy/` and `policies/` locations until a migration plan exists.
- Add principal and audience fields only when caller context can supply them.
- Avoid embedding Cedar snippets in Rust tests unless they are parsed by the same library used in production.
- Treat missing Cedar entity types as a blocker, not prose work.
- Emit denial evidence with redacted resource identifiers.
- Preserve the existing `ItsmPorts` trait shape unless multiple adapters need a split.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
