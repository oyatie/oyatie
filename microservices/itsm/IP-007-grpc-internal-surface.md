---
doc_class: IP
ip_id: IP-007-grpc-internal-surface
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + council-product
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/contracts/itsm-v1.proto
  - microservices/itsm/src/adapter/mod.rs
  - microservices/itsm/src/usecase/mod.rs
  - microservices/itsm/tests/integration.rs
---

# IP-007 ITSM gRPC Internal Surface

## A. Problem
The prior stamped IP did not explain why ITSM needs gRPC. The actual gap is the internal low-latency command surface used by workflow-engine, incident-room, status-update, and postmortem flows when a REST request has already been admitted but downstream services need a typed, versioned command.

ServiceNow and Jira hide these internals behind their platform runtimes. Oyatie needs the internal surface to be explicit so agents and sibling µservices can call ITSM without learning REST route quirks.

## B. Approach
Promote `microservices/itsm/contracts/itsm-v1.proto` from a single `InvokeAction` stub to a small internal command surface that mirrors current usecases:

| RPC | Usecase | Current Rust binding |
|---|---|---|
| `OpenIncident` | open ticket and emit `IncidentOpened` | `OpenIncident::execute` |
| `RecomputeSla` | recompute SLA for open/triaged ticket | `RecomputeSla::execute` |
| `ApproveChange` | approve change after Cedar/freeze gate | `ApproveChange::execute` |

The proto must preserve `tenant_id`, `principal_id`, `purpose`, and idempotency fields. It must not introduce a vendor-style "admin workspace id" authority.

## C. Deliverables
- Replace generic `ItsmActionRequest` with specific request/response messages in `contracts/itsm-v1.proto`.
- Keep `ItsmService` RPC names aligned with `src/usecase/mod.rs`.
- Update `ItsmGrpcHandler` in `src/adapter/mod.rs` with handlers for SLA recompute and change approval, not only incident open.
- Add tests that compare gRPC handler receipts with HTTP handler receipts for the same command.
- Document internal-only audience and HTTP/3/gRPC transport assumption from the manifest.

## D. Implementation
1. Add `OpenIncidentRequest`, `RecomputeSlaRequest`, and `ApproveChangeRequest` messages with explicit tenant and command identifiers.
2. Add response messages that include `tenant_id`, subject id, status, and `audit_event`.
3. Replace or deprecate `rpc InvokeAction` with named RPCs; keep a compatibility alias only if downstream callers still consume it.
4. Extend `ItsmGrpcHandler` with `recompute_sla` and `approve_change` wrappers over `ItsmService`.
5. Ensure every parsed id uses `TenantId::parse`, `TicketId::parse`, and `ChangeId::parse` so invalid internal messages fail before usecase execution.
6. Add an integration test that opens an incident over HTTP and then recomputes SLA over gRPC using the same in-memory ports.
7. Add a test proving a bad tenant id in the gRPC request is rejected with the same validation path as REST.
8. Update this IP's evidence after proto generation is wired; do not invent generated Rust paths until they exist.

## E. Acceptance
- `contracts/itsm-v1.proto` compiles under proto3 and exposes named ITSM commands.
- `cargo test -p oya-itsm-service-management-service grpc` covers incident open, SLA recompute, and change approval once handlers exist.
- No gRPC message authorizes by ServiceNow sys_id, Jira project key, or Freshservice workspace id.
- The gRPC surface remains internal; public tenant calls continue through REST/OpenAPI.

## F. Evidence
- `contracts/itsm-v1.proto` currently defines `ItsmActionRequest`, `ItsmActionAccepted`, and `ItsmService.InvokeAction`.
- `src/adapter/mod.rs` currently has `TicketGrpcRequest`, `TicketGrpcResponse`, and `ItsmGrpcHandler::open_incident`.
- `src/usecase/mod.rs` provides the three command paths this surface should expose.
- `tests/integration.rs` already proves handler/usecase wiring through REST.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow internal platform APIs | Explicit proto contract instead of hidden platform runtime calls |
| Jira Service Management project automation internals | Typed tenant-scoped command surface for sibling µservices |
| Freshservice orchestration hooks | Internal RPCs still carry Cedar/audit receipt semantics |

## H. Cold-start buildability notes
- Start by adding named RPCs beside `InvokeAction`; delete the generic RPC only after callers migrate.
- Keep proto package `oyatie.itsm.v1` stable.
- Reuse `TicketGrpcResponse` until a richer response type is needed.
- Add `request_id` or idempotency only as a required field when the handler actually consumes it.
- Keep REST and gRPC behavior equivalent for incident open.
- Never parse tenant ids manually; route through `TenantId::parse`.
- Add conformance tests before generated client work.
- Keep gRPC internal in docs and ingress policy.
- Do not expose ServiceNow sys_id or Jira issue key as primary ids.
- Record generated-code paths only after generation lands.
- Keep proto evolution additive until the first stable SDK release.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`.proto`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
