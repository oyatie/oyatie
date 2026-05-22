---
doc_class: CrossMicroserviceHandoffMatrix
title: "API Gateway Cross-Microservice Handoff Matrix"
status: Draft
date: 2026-05-20
microservice: api-gateway
owner_team: axis-network
---

# API Gateway Cross-Microservice Handoff Matrix

This matrix records the concrete handoffs for the `api-gateway` microservice.
REST shapes are from `microservices/api-gateway/contracts/openapi/api-gateway.openapi.yaml`.
Async shapes are from `microservices/api-gateway/contracts/asyncapi/api-gateway.asyncapi.yaml`.
Policy names are from `microservices/api-gateway/policies/`.
Audit-chain handoffs follow ADR-0263 and the gateway audit seal events in the service manifest.
The gateway owns edge admission, policy preflight, route-to-cell resolution, and upstream circuit signalling.

## Inbound Callers

| # | Calling microservice | Named API on `api-gateway` | Data shape | Cedar permit required | Audit event emitted |
|---|---|---|---|---|---|
| 1 | `application` | `POST /edge/admission` `admitEdgeRequest` | `EdgeAdmissionRequest` with `tenant_id`, `cell_id`, `route_id`, `request_id` | `route-authorization.cedar` principal `Service::application` action `Gateway::admit_edge_request` resource `Route::{route_id}` | `ApiGatewayRequestAdmitted` or `ApiGatewayRequestDenied` |
| 2 | `developer-sdk` | `POST /edge/admission` `admitEdgeRequest` | `EdgeAdmissionRequest` for sandbox `route_id` | `developer-scope.cedar` via `route-authorization.cedar` action `Gateway::admit_sandbox_request` | `ApiGatewaySandboxRequestAdmitted` |
| 3 | `payments` | `POST /edge/admission` `admitEdgeRequest` | `EdgeAdmissionRequest` with PSP callback `request_id` | `rate-limit.cedar` action `Gateway::admit_payment_webhook` | `ApiGatewayPaymentWebhookAdmitted` |
| 4 | `audit-chain` | `POST /edge/admission` `admitEdgeRequest` | `EdgeAdmissionRequest` for proof/export endpoints | `auditor-scope.cedar` action `Gateway::admit_audit_endpoint` | `ApiGatewayAuditRouteAdmitted` |
| 5 | `cloud-iac` | `POST /edge/admission` `admitEdgeRequest` | deployment callback admission shape | `ci-scope.cedar` action `Gateway::admit_iac_callback` | `ApiGatewayIacCallbackAdmitted` |
| 6 | `cloud-secrets` | `POST /edge/admission` `admitEdgeRequest` | secret revocation push admission shape | `secret-isolation.md` guard `internal_secret_push_only` | `ApiGatewaySecretPushAdmitted` |
| 7 | `observability` | `POST /edge/admission` `admitEdgeRequest` | synthetic probe `EdgeAdmissionRequest` | `public-read.cedar` action `Gateway::admit_probe` | `ApiGatewaySyntheticProbeAdmitted` |
| 8 | `ops-dashboard-control-center` | `POST /edge/admission` `admitEdgeRequest` | operator route probe shape | `auditor-scope.cedar` action `Gateway::admit_operator_probe` | `ApiGatewayOperatorProbeAdmitted` |
| 9 | `cell` | `POST /edge/cell-route-refresh` | inline `CellRouteRefresh {cell_id, route_epoch}` | `route-authorization.cedar` action `Gateway::refresh_cell_route` | `ApiGatewayCellRouteRefreshAccepted` |
| 10 | `compliance` | `POST /edge/admission` `admitEdgeRequest` | compliance export route shape | `auditor-scope.cedar` action `Gateway::admit_compliance_route` | `ApiGatewayComplianceRouteAdmitted` |
| 11 | `identity` | `POST /edge/admission` `admitEdgeRequest` | login and callback route shape | `route-authorization.cedar` action `Gateway::admit_identity_route` | `ApiGatewayIdentityRouteAdmitted` |
| 12 | `tenancy` | `POST /edge/admission` `admitEdgeRequest` | tenant bootstrap route shape | `tenant-scope.cedar` action `Gateway::admit_tenant_bootstrap` | `ApiGatewayTenantBootstrapAdmitted` |

## Outbound Callees

| # | Callee microservice | Named API called by `api-gateway` | Data shape sent or received | Cedar permit required | Audit event consumed |
|---|---|---|---|---|---|
| 1 | `cell` | `GET /tenants/{tenant_id}/assignment` `getCellAssignment` | `CellAssignment` | `tenant-scope.cedar` action `Cell::get_assignment` | consumes `CellAssignmentRead` |
| 2 | `application` | `GET /routes/resolve` `resolve_route` | `RouteResolveResponse` | `route-authorization.cedar` action `Application::resolve_route` | consumes `ApplicationRouteResolved` |
| 3 | `identity` | `POST /service-token/introspect` | inline `ServiceTokenIntrospectionRequest {svid, audience}` | `tls-policy.cedar` action `Identity::introspect_edge_principal` | consumes `IdentityPrincipalIntrospected` |
| 4 | `audit-chain` | `POST /emit` `emitEvent` | `AuditEvent` with `source_microservice=api-gateway` | `tenant-scope.cedar` action `AuditChain::emit_gateway_event` | consumes `ApiGatewayAuditReceiptAccepted` |
| 5 | `cloud-secrets` | `GET /secrets/{tenant}/api-gateway/tls-cert/reference` | `SecretReference` | `secret-isolation.md` guard `gateway_tls_secret_read` | consumes `SecretReferenceRead` |
| 6 | `cloud-iac` | `GET /microservices/api-gateway/apply-state/{environment}` | `ApplyStateRecord` | `ci-scope.cedar` action `CloudIac::read_gateway_apply_state` | consumes `CloudIacApplyStateRead` |
| 7 | `observability` | `POST /metrics/gateway-admission` | inline `GatewayAdmissionMetric {tenant_id_hash, route_id, decision}` | `public-read.cedar` action `Observability::write_gateway_metric` | consumes `MetricAccepted` |
| 8 | `payments` | `POST /v1/webhooks/{psp}/v1` | PSP webhook body routed after admission | `route-authorization.cedar` action `Payments::receive_psp_webhook` | consumes `PaymentWebhookAccepted` |
| 9 | `developer-sdk` | `GET /sdk/families/{family}/{version}/download` | public SDK artifact response | `public-read.cedar` action `DeveloperSdk::download_sdk` | consumes `DeveloperSdkArtifactRead` |
| 10 | `audit-chain` | `GET /events/{event_id}/proof` `getProof` | `AuditProof` for gateway denial replay | `auditor-scope.cedar` action `AuditChain::read_gateway_proof` | consumes `AuditProofRead` |
| 11 | `compliance` | `POST /abuse/evaluate` | inline `AbuseEvaluationRequest {tenant_id, request_id, bot_score}` | `abuse-defence.cedar` action `Compliance::evaluate_abuse` | consumes `AbuseDefenceChallengeIssued` |
| 12 | `ops-dashboard-control-center` | `POST /incidents/gateway-circuit-open` | inline `GatewayCircuitIncident {route_id, upstream, opened_at}` | `auditor-scope.cedar` action `OpsDashboard::open_gateway_incident` | consumes `OpsIncidentOpened` |

## Event Subscriptions

| # | AsyncAPI channel subscribed | Event class | Handler behavior | Dead-letter policy |
|---|---|---|---|---|
| 1 | `workflow-events/cell.assigned` | `CellAssignedPayload` | updates edge route table for tenant-cell mapping | retry 8 times, then `api-gateway.dlq.cell_assigned` |
| 2 | `workflow-events/cell.rebalanced` | `CellRebalancedPayload` | drains old upstream and warms new upstream pool | retry 8 times, then `api-gateway.dlq.cell_rebalanced` |
| 3 | `workflow-events/host.drain.started` | `HostDrainStartedPayload` | marks cell endpoints as draining and reduces load | retry 6 times, then `api-gateway.dlq.host_drain_started` |
| 4 | `workflow-events/host.drain.completed` | `HostDrainCompletedPayload` | removes drained endpoints from route cache | retry 6 times, then `api-gateway.dlq.host_drain_completed` |
| 5 | `cloud-secrets.secret.rotated` | `SecretLifecyclePayload` | reloads TLS and signing secret references by version | retry 10 times, then `api-gateway.dlq.secret_rotated` |
| 6 | `cloud-secrets.secret.revoked` | `SecretRevokedPayload` | evicts revoked TLS certificate and opens cert breaker | retry 10 times, then `api-gateway.dlq.secret_revoked` |
| 7 | `cloud-iac.apply.completed` | `ApplyCompletedPayload` | refreshes deployed route manifest digest | retry 5 times, then `api-gateway.dlq.iac_apply_completed` |
| 8 | `audit-chain.audit.seal.minted` | `SealMintedPayload` | associates gateway audit ids with sealed periods | retry 10 times, then `api-gateway.dlq.audit_seal_minted` |
| 9 | `payments.payment-events.{tenant_id}` | `ChargeErrored` | applies PSP callback route throttle when charge errors spike | retry 3 times, then `api-gateway.dlq.payment_charge_errored` |
| 10 | `developer-sdk.oya.developer-sdk.sandbox` | `SandboxProvisioned` | creates sandbox route namespace | retry 6 times, then `api-gateway.dlq.sandbox_provisioned` |
| 11 | `application.workflow-events/application.module.rolled.back` | `ModuleRolledBack` | invalidates module bundle routes | retry 6 times, then `api-gateway.dlq.module_rolled_back` |
| 12 | `cell.workflow-events/cell.boundary.violation.detected` | `CellBoundaryViolationDetectedPayload` | refuses cross-boundary edge routing for affected tenant | retry 10 times, then `api-gateway.dlq.cell_boundary_violation` |

## Event Emissions

| # | AsyncAPI channel published | Event class | Payload schema | Downstream consumers |
|---|---|---|---|---|
| 1 | `oya.api_gateway.admission` | `RequestAdmitted` | `api-gateway.asyncapi.yaml#/components/schemas/RequestAdmitted` | `audit-chain`, `observability`, `application` |
| 2 | `oya.api_gateway.admission` | `RequestDenied` | `api-gateway.asyncapi.yaml#/components/schemas/RequestDenied` | `audit-chain`, `observability`, `compliance` |
| 3 | `audit-chain /emit` | `ApiGatewayRequestAdmitted` | `AuditEvent.payload` with `tenant_id`, `request_id`, `route_id`, `cell_id` | `audit-chain` |
| 4 | `audit-chain /emit` | `ApiGatewayRequestDenied` | `AuditEvent.payload` with `tenant_id`, `request_id`, `denial_reason` | `audit-chain`, `compliance` |
| 5 | `audit-chain /emit` | `ApiGatewayWafTriggered` | `AuditEvent.payload` with `route_id`, `bot_score`, `rule_id` | `audit-chain`, `compliance` |
| 6 | `audit-chain /emit` | `ApiGatewayRateLimitExceeded` | `AuditEvent.payload` with `tenant_id`, `route_id`, `limit_key` | `audit-chain`, `observability` |
| 7 | `audit-chain /emit` | `ApiGatewayTlsHandshakeFailed` | `AuditEvent.payload` with `sni_hash`, `tls_policy_id` | `audit-chain`, `cloud-secrets` |
| 8 | `audit-chain /emit` | `ApiGatewayCedarPermitMatched` | `AuditEvent.payload` with `cedar_policy_version`, `action`, `resource_ref` | `audit-chain` |
| 9 | `audit-chain /emit` | `ApiGatewayCedarDenyMatched` | `AuditEvent.payload` with `cedar_policy_version`, `action`, `resource_ref`, `decision` | `audit-chain`, `compliance` |
| 10 | `audit-chain /emit` | `ApiGatewayUpstreamTimeout` | `AuditEvent.payload` with `upstream_microservice`, `route_id`, `deadline_ms` | `audit-chain`, `ops-dashboard-control-center` |
| 11 | `audit-chain /emit` | `ApiGatewayCircuitOpen` | `AuditEvent.payload` with `upstream_microservice`, `breaker_name`, `opened_at` | `audit-chain`, `ops-dashboard-control-center` |
| 12 | `observability.gateway-admission` | `GatewayAdmissionMetricRecorded` | inline `GatewayAdmissionMetric` | `observability` |
| 13 | `cell.gateway-circuit-open` | `GatewayCircuitOpenForCell` | inline `GatewayCircuitOpen {cell_id, route_id, reason}` | `cell` |
| 14 | `application.route-cache-invalidated` | `GatewayRouteCacheInvalidated` | inline `RouteCacheInvalidated {route_id, reason, epoch}` | `application` |

## Synchronous vs Asynchronous Boundaries

| # | Boundary | Mode | Reasoning |
|---|---|---|---|
| 1 | edge caller to `POST /edge/admission` | synchronous | callers need permit, deny, or throttle before forwarding |
| 2 | gateway to `cell getCellAssignment` | synchronous | upstream target is unknowable without cell assignment |
| 3 | gateway to `application resolve_route` | synchronous | route policy and required roles must be known before request proxy |
| 4 | gateway to `identity introspect` | synchronous | principal SVID is part of Cedar context |
| 5 | gateway to `audit-chain emitEvent` for deny decisions | synchronous | denial must have ADR-0263 audit receipt before response body leaves edge |
| 6 | gateway to `cloud-secrets getSecretReference` | synchronous at boot and rotation | TLS reload cannot complete without versioned reference |
| 7 | `RequestAdmitted` event | asynchronous | admission result is already returned; observability can consume after |
| 8 | `RequestDenied` event | asynchronous with audit receipt | compliance can react after denial response |
| 9 | `cell.assigned` subscription | asynchronous | route table can converge without blocking tenancy creation |
| 10 | `host.drain.started` subscription | asynchronous | edge load shedding uses cache update and does not block drain |
| 11 | PSP webhook proxy to `payments` | synchronous | PSP expects HTTP acknowledgement from payments path |
| 12 | `cloud-iac.apply.completed` subscription | asynchronous | deployment digest refresh can lag behind route serving |
| 13 | abuse evaluation to `compliance` | synchronous for challenge, asynchronous for metrics | challenge response must be immediate; trend analysis can lag |
| 14 | circuit-open incident to `ops-dashboard` | asynchronous | operator alert cannot block edge path |
| 15 | audit seal minted subscription | asynchronous | seal finality is replayable after emit receipt |

## Failure Mode Cascade

| # | Failure in `api-gateway` | Upstream impact | Circuit breaker | Retry policy |
|---|---|---|---|---|
| 1 | admission timeout | `application`, `payments`, and `developer-sdk` receive no edge decision | caller-side `gateway-admission` breaker trips after 5 failures | clients retry only idempotent requests with same `request_id` |
| 2 | cell lookup failure | edge cannot route tenant traffic | `cell-assignment` breaker opens per tenant | 3 gateway retries, then 503 with `Retry-After` |
| 3 | identity introspection failure | authenticated routes fail closed | `identity-introspection` breaker blocks protected routes | no fallback to anonymous principal |
| 4 | audit emit failure on deny | gateway cannot return policy denial | `audit-deny` breaker holds response until deadline | retry 10 times, then 503 audit unavailable |
| 5 | rate limiter unavailable | abuse-sensitive routes fail closed | `rate-limit` breaker moves route to conservative deny | no retry for unsafe methods |
| 6 | WAF engine unavailable | public routes enter challenge mode | `abuse-defence` breaker emits `AbuseDefenceVendorOutage` | retry rule engine reload every 30 seconds |
| 7 | TLS secret revoked | route serving stops for affected SNI | `tls-cert` breaker refuses handshakes | reload next version from `cloud-secrets` |
| 8 | upstream timeout to `payments` | PSP callback may retry externally | `payments-upstream` breaker opens by PSP and tenant | retry safe PSP callbacks using idempotency key |
| 9 | route cache corruption | application bundle routes may misroute | route cache breaker purges and rebuilds | refetch from `application` and `cell` |
| 10 | admission event bus down | observability delayed | outbox breaker spools events locally | replay by `event_id` after broker returns |
| 11 | DLQ saturation | cell and route changes lag | gateway stops noncritical route refreshes | manual replay oldest-first |
| 12 | abuse challenge false positive | user blocked at edge | compliance review channel receives denial event | no automatic retry without new challenge token |

## Cross-tenant Coordination

| # | Scenario | Cedar guard pattern | Audit-mirror requirement |
|---|---|---|---|
| 1 | conglomerate parent route reads child tenant dashboard | `route-authorization.cedar` plus `ConglomerateGrantCreated` scope in request context | mirror `ConglomerateParentReadAction` to parent and child partitions |
| 2 | cross-jurisdiction edge route selected | `sov-cloud-overlay.cedar` checks `jurisdiction_code` and `cell_id` | mirror `ConglomerateCrossJurisdictionResidencyEnforced` |
| 3 | office sub-scope route access | `route-authorization.cedar` action includes `sub_scope_path` | mirror `OfficeBoundaryAttemptEvaluated` and final allow/deny |
| 4 | personal tenant attempts work route | `tenant-scope.cedar` forbids `context_type=personal` on work resource | mirror `ConglomeratePersonalTenantBoundaryRefused` |
| 5 | information-barrier route crossing | `route-authorization.cedar` checks `barrier_tags` | mirror `ConglomerateInformationBarrierCrossingRefused` |

## Data Shape Ledger

| # | Shape | Source | Required handoff fields |
|---|---|---|---|
| 1 | `EdgeAdmissionRequest` | `contracts/openapi/api-gateway.openapi.yaml` | `tenant_id`, `cell_id`, `route_id`, `request_id` |
| 2 | `RequestAdmitted` | `contracts/asyncapi/api-gateway.asyncapi.yaml` | `tenant_id`, `request_id`, `route_id` |
| 3 | `RequestDenied` | `contracts/asyncapi/api-gateway.asyncapi.yaml` | `tenant_id`, `request_id`, `denial_reason` |
| 4 | `CellAssignment` | `microservices/cell/contracts/openapi/cell.yaml` | `tenant_id`, `cell_id`, `pack`, `assigned_at`, `scope` |
| 5 | `RouteResolveResponse` | `microservices/application/contracts/openapi/application.yaml` | `route`, `tenant_scope`, `required_roles`, `pack_residency` |
| 6 | `SecretReference` | `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml` | `path`, `version`, `data_class`, `rotation_policy_id` |
| 7 | `AuditEvent` | `microservices/audit-chain/contracts/openapi/audit-chain.yaml` | `tenant_id`, `source_microservice`, `event_class`, `payload`, `payload_data_class` |
| 8 | `ApplyStateRecord` | `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml` | `microservice`, `pack`, `environment`, `current_sha` |
| 9 | `GatewayAdmissionMetric` | inline gateway observability contract | `tenant_id_hash`, `route_id`, `decision`, `latency_ms` |
| 10 | `GatewayCircuitIncident` | inline ops-dashboard contract | `route_id`, `upstream`, `opened_at`, `failure_count` |

## Cedar Guard Ledger

| # | Policy file | Principal | Action | Resource |
|---|---|---|---|---|
| 1 | `route-authorization.cedar` | `Service::application` | `Gateway::admit_edge_request` | `Route::{route_id}` |
| 2 | `route-authorization.cedar` | `Service::identity` | `Gateway::admit_identity_route` | `Route::identity` |
| 3 | `rate-limit.cedar` | `Service::payments` | `Gateway::admit_payment_webhook` | `Route::payments_webhook` |
| 4 | `abuse-defence.cedar` | `Service::api-gateway` | `Compliance::evaluate_abuse` | `Request::{request_id}` |
| 5 | `tls-policy.cedar` | `Service::api-gateway` | `Identity::introspect_edge_principal` | `Principal::{svid}` |
| 6 | `sov-cloud-overlay.cedar` | `Service::api-gateway` | `Gateway::route_cross_jurisdiction` | `Cell::{cell_id}` |
| 7 | `tenant-scope.cedar` | `Service::tenancy` | `Gateway::admit_tenant_bootstrap` | `Tenant::{tenant_id}` |
| 8 | `ci-scope.cedar` | `Service::cloud-iac` | `Gateway::admit_iac_callback` | `Deployment::{environment}` |
| 9 | `auditor-scope.cedar` | `Service::audit-chain` | `Gateway::admit_audit_endpoint` | `AuditRoute::*` |
| 10 | `public-read.cedar` | `Service::observability` | `Gateway::admit_probe` | `Route::health` |

## Audit Event Class Ledger

| # | Audit class | Emitting handoff | ADR-0263 envelope fields that must be present |
|---|---|---|---|
| 1 | `ApiGatewayRequestAdmitted` | `POST /edge/admission` permit | `tenant_id`, `event_id`, `trace_id`, `audit_id`, `source_microservice` |
| 2 | `ApiGatewayRequestDenied` | `POST /edge/admission` deny | `tenant_id`, `decision`, `action`, `resource_ref`, `cedar_policy_version` |
| 3 | `ApiGatewayWafTriggered` | WAF denial | `tenant_id`, `request_id`, `policy_fragment_id`, `decision` |
| 4 | `ApiGatewayRateLimitExceeded` | rate limiter denial | `tenant_id`, `resource_ref`, `limit_key`, `audit_id` |
| 5 | `ApiGatewayTlsHandshakeFailed` | TLS ingress | `sni_hash`, `cell_id`, `jurisdiction_code`, `audit_id` |
| 6 | `ApiGatewayCedarPermitMatched` | route permit | `action`, `resource_ref`, `evaluation_id`, `decision` |
| 7 | `ApiGatewayCedarDenyMatched` | route denial | `action`, `resource_ref`, `evaluation_id`, `decision` |
| 8 | `ApiGatewayUpstreamTimeout` | proxy timeout | `upstream_microservice`, `deadline_ms`, `request_id`, `audit_id` |
| 9 | `ApiGatewayCircuitOpen` | breaker opens | `upstream_microservice`, `breaker_name`, `opened_at`, `audit_id` |
| 10 | `AbuseDefenceChallengeIssued` | compliance challenge response | `tenant_id`, `event_id`, `source_microservice`, `audit_id` |

## Handoff Control Checklist

1. Admission accepts only `EdgeAdmissionRequest` objects with non-empty `tenant_id`.
2. Admission accepts only `EdgeAdmissionRequest` objects with non-empty `request_id`.
3. Admission rejects any route without a `Route::{route_id}` Cedar resource.
4. Admission records the `cedar_policy_version` for every permit.
5. Admission records the `cedar_policy_version` for every denial.
6. The gateway calls `cell getCellAssignment` before choosing an upstream pool.
7. The gateway calls `application resolve_route` before forwarding module routes.
8. The gateway calls `identity introspect` before protected route authorization.
9. The gateway emits `ApiGatewayRequestDenied` before returning a Cedar denial.
10. The gateway emits `ApiGatewayRateLimitExceeded` before returning a 429.
11. The gateway emits `ApiGatewayWafTriggered` before returning an abuse challenge.
12. The gateway emits `ApiGatewayTlsHandshakeFailed` for rejected TLS handshakes.
13. The gateway tags PSP webhooks with `Payments::receive_psp_webhook`.
14. The gateway preserves PSP idempotency keys when proxying to `payments`.
15. The gateway never sends raw TLS secret material to `observability`.
16. The gateway consumes only versioned TLS references from `cloud-secrets`.
17. The gateway updates route cache from `workflow-events/cell.assigned`.
18. The gateway removes drained endpoints after `workflow-events/host.drain.completed`.
19. The gateway applies `sov-cloud-overlay.cedar` before cross-jurisdiction routing.
20. The gateway mirrors parent-child route reads into both audit partitions.
21. The gateway stores `RequestAdmitted.event_id` in the local outbox.
22. The gateway stores `RequestDenied.event_id` in the local outbox.
23. The gateway replays outbox events in `event_id` order.
24. The gateway DLQ uses channel-specific names, not a shared global queue.
25. The gateway opens per-upstream breakers, not a single global breaker.
26. The `cell-assignment` breaker is keyed by `tenant_id`.
27. The `payments-upstream` breaker is keyed by PSP and tenant.
28. The `identity-introspection` breaker fails closed.
29. The abuse vendor breaker emits `AbuseDefenceVendorOutage`.
30. The rate-limit breaker fails closed for unsafe methods.
31. The public SDK download path remains covered by `public-read.cedar`.
32. The audit proof path remains covered by `auditor-scope.cedar`.
33. The operator probe path remains covered by `auditor-scope.cedar`.
34. The synthetic probe path remains covered by `public-read.cedar`.
35. The route refresh path accepts only `Service::cell`.
36. The gateway treats `cell_id` from clients as untrusted input.
37. The gateway overwrites client-supplied `cell_id` with `cell` assignment when needed.
38. The gateway scrubs request bodies before audit emission.
39. The gateway hashes tenant identifiers in metric payloads.
40. The gateway keeps audit payloads PII-scrubbed per ADR-0263.
41. The gateway includes `trace_id` on every audit event.
42. The gateway includes `span_id` on every audit event.
43. The gateway includes `audit_id` on every state-changing response.
44. The gateway includes `cell_id` on route decisions.
45. The gateway includes `jurisdiction_code` for residency-sensitive routes.
46. The gateway rejects personal-context calls to work-context routes.
47. The gateway rejects information-barrier crossings without an approved grant.
48. The gateway logs canary routing as `ApiGatewayCanaryRouted`.
49. The gateway logs blue-green routing as `ApiGatewayBlueGreenRouted`.
50. The gateway logs certificate rotation as `ApiGatewayTlsCertRotated`.
51. The gateway logs ECH rotation as `ApiGatewayEchConfigRotated`.
52. The gateway logs PQC handshakes as `ApiGatewayPqcHandshakeCompleted`.
53. The gateway logs honeypot activation as `ApiGatewayHoneypotActivated`.
54. The gateway logs cell depool as `ApiGatewayCellDepooled`.
55. The gateway logs cell repool as `ApiGatewayCellRepooled`.
56. The gateway logs DDoS scrub as `ApiGatewayDdosScrubEngaged`.
57. The gateway logs cache poisoning attempts as `ApiGatewayCachePoisoningAttempt`.
58. The gateway does not publish admission events without audit receipt for denies.
59. The gateway may publish permit metrics after response completion.
60. The gateway must preserve route epoch monotonicity.
61. The gateway must not accept stale `CellRouteRefresh.route_epoch`.
62. The gateway must reject route refresh for unknown `cell_id`.
63. The gateway must reopen circuits only after a successful health probe.
64. The gateway must not reopen circuits from event messages alone.
65. The gateway must report DLQ age to `observability`.
66. The gateway must attach `source_microservice=api-gateway` to audit events.
67. The gateway must not emit audit classes outside the manifest allowlist without an ADR.
68. The gateway must keep upstream timeout budgets below caller deadlines.
69. The gateway must reject unsigned internal callbacks.
70. The gateway must keep this matrix updated when `api-gateway.openapi.yaml` changes.

## Checkpoint

- Authored for `api-gateway` on 2026-05-20.
- Source contracts checked: `api-gateway.openapi.yaml`, `api-gateway.asyncapi.yaml`, and gateway proto.
- Source policies checked: `route-authorization.cedar`, `rate-limit.cedar`, `abuse-defence.cedar`, `tls-policy.cedar`, `sov-cloud-overlay.cedar`, `tenant-scope.cedar`.
- No in-flight microservice directories were edited.
- Oya VCS scope: `microservices`.
