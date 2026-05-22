---
doc_class: CrossMicroserviceHandoffMatrix
title: "Application Cross-Microservice Handoff Matrix"
status: Draft
date: 2026-05-20
microservice: application
owner_team: axis-application
---

# Application Cross-Microservice Handoff Matrix

This matrix records the concrete handoffs for the `application` microservice.
REST shapes are from `microservices/application/contracts/openapi/application.yaml`.
Tenant-admin shapes are from `microservices/application/contracts/openapi/tenant-admin-console.yaml`.
Async shapes are from `microservices/application/contracts/asyncapi/application-events.yaml`.
gRPC names are from `microservices/application/contracts/proto/`.
Cedar policies are from `microservices/application/policies/`.
Audit-chain emission follows ADR-0263 with `source_microservice=application`.

## Inbound Callers

| # | Calling microservice | Named API on `application` | Data shape | Cedar permit required | Audit event emitted |
|---|---|---|---|---|---|
| 1 | `api-gateway` | `GET /routes/resolve` `resolve_route` | `RouteResolveResponse` with `route`, `tenant_scope`, `required_roles`, `pack_residency` | `tenant-scope.cedar` action `Application::resolve_route` | `ApplicationRouteResolved` |
| 2 | `api-gateway` | `GET /modules/{module}/manifest` `get_module_manifest` | `ModuleManifest` | `tenant-scope.cedar` action `Application::read_module_manifest` | `ApplicationModuleManifestRead` |
| 3 | `api-gateway` | `GET /bundles/{module}/pointer` `get_bundle_pointer` | `BundlePointer` | `tenant-scope.cedar` action `Application::read_bundle_pointer` | `ApplicationBundlePointerRead` |
| 4 | `identity` | `POST /auth/start` `start_auth` | `SessionStartRequest` with `provider`, `redirect_uri`, `state`, `nonce`, `code_verifier_sha256` | `tenant-scope.cedar` action `Application::start_auth_session` | `ApplicationAuthStarted` |
| 5 | `identity` | `GET /auth/callback` `auth_callback` | OIDC callback query with `code`, `state` | `tenant-scope.cedar` action `Application::finish_oidc_session` | `ApplicationAuthCallbackAccepted` |
| 6 | `identity` | `POST /auth/saml/callback` `saml_callback` | SAML assertion callback body | `tenant-scope.cedar` action `Application::finish_saml_session` | `ApplicationSamlCallbackAccepted` |
| 7 | `tenant-admin-console` | `POST /tenant-admin/policy-drafts` `createTenantPolicyDraft` | `TenantPolicyDraftRequest` | `tenant-admin-console.cedar` action `CreateTenantPolicyDraft` | `TenantPolicyDraftCreated` |
| 8 | `tenant-admin-console` | `POST /tenant-admin/policy-drafts/{draft_id}/apply` `applyTenantPolicyDraft` | `TenantPolicyDraftApplyRequest` | `tenant-admin-console.cedar` action `ApplyTenantPolicyDraft` | `TenantPolicyDraftApplied` |
| 9 | `cloud-iac` | `POST /modules/{module}/manifest:publish` `publish_module_manifest` | `ModuleManifestPublishRequest` | `ci-scope.cedar` action `Application::publish_module_manifest` | `ApplicationModuleManifestPublished` |
| 10 | `cloud-iac` | `POST /cdn/purge` `purge_cdn` | `CdnPurgeRequest` with `pattern`, `reason` | `ci-scope.cedar` action `Application::purge_cdn` | `ApplicationCdnPurgeRequested` |
| 11 | `developer-sdk` | `GET /modules/{module}/manifest` `get_module_manifest` | `ModuleManifest` for sandbox module | `developer-scope.cedar` action `Application::read_sandbox_manifest` | `ApplicationSandboxManifestRead` |
| 12 | `payments` | `GET /routes/resolve` `resolve_route` | route resolution for checkout and billing flows | `tenant-scope.cedar` action `Application::resolve_payment_route` | `ApplicationPaymentRouteResolved` |
| 13 | `audit-chain` | `GET /auth/session` `get_session` | `SessionSummary` projection | `auditor-scope.cedar` action `Application::read_session_for_audit` | `ApplicationSessionReadForAudit` |
| 14 | `ops-dashboard-control-center` | `GET /status` `get_status` | `ApplicationStatus` | `public-read.cedar` action `Application::read_status` | `ApplicationStatusRead` |

## Outbound Callees

| # | Callee microservice | Named API called by `application` | Data shape sent or received | Cedar permit required | Audit event consumed |
|---|---|---|---|---|---|
| 1 | `api-gateway` | `POST /edge/admission` `admitEdgeRequest` | `EdgeAdmissionRequest` | `route-authorization.cedar` action `Gateway::admit_edge_request` | consumes `ApiGatewayRequestAdmitted` |
| 2 | `cell` | `GET /tenants/{tenant_id}/assignment` `getCellAssignment` | `CellAssignment` | `tenant-scope.cedar` action `Cell::resolve_runtime_assignment` | consumes `CellAssignmentResolvedForApplication` |
| 3 | `identity` | `POST /service-token/introspect` | inline `ServiceTokenIntrospectionRequest {svid, audience}` | `tenant-scope.cedar` action `Identity::introspect_application_principal` | consumes `IdentityPrincipalIntrospected` |
| 4 | `audit-chain` | `POST /emit` `emitEvent` | `AuditEvent` with `source_microservice=application` | `tenant-scope.cedar` action `AuditChain::emit_application_event` | consumes `ApplicationAuditReceiptAccepted` |
| 5 | `cloud-secrets` | `GET /secrets/{tenant}/application/session-signer/reference` | `SecretReference` | `secret-isolation.md` guard `application_session_secret_read` | consumes `SecretReferenceRead` |
| 6 | `cloud-iac` | `GET /charts/{digest}/provenance` `getProvenance` | `Provenance` | `ci-scope.cedar` action `CloudIac::read_application_bundle_provenance` | consumes `CloudIacProvenanceRead` |
| 7 | `developer-sdk` | `GET /submissions/{submissionId}/status` `streamSubmissionStatus` | `VettingStageEvent` | `developer-scope.cedar` action `DeveloperSdk::read_submission_status` | consumes `DeveloperSubmissionStatusRead` |
| 8 | `payments` | `GET /v1/subscriptions/{subscription_id}` | inline `Subscription` projection from payments contract | `tenant-scope.cedar` action `Payments::read_subscription_for_application` | consumes `PaymentSubscriptionRead` |
| 9 | `observability` | `POST /metrics/application-route` | inline `ApplicationRouteMetric {tenant_id_hash, route, result}` | `public-read.cedar` action `Observability::write_application_metric` | consumes `MetricAccepted` |
| 10 | `compliance` | `POST /policy/simulate` | inline `PolicySimulationRequest {tenant_id, draft_id, fragment}` | `tenant-admin-console.cedar` action `SimulateTenantPolicyDraft` | consumes `TenantPolicyDraftSimulated` |
| 11 | `api-gateway` | `POST /edge/cell-route-refresh` | inline `CellRouteRefresh {cell_id, route_epoch}` when module route changes | `route-authorization.cedar` action `Gateway::refresh_cell_route` | consumes `ApiGatewayCellRouteRefreshAccepted` |
| 12 | `ops-dashboard-control-center` | `POST /incidents/module-load-rejected` | inline `ModuleLoadIncident {module, tenant_id_hash, reason}` | `auditor-scope.cedar` action `OpsDashboard::open_application_incident` | consumes `OpsIncidentOpened` |

## Event Subscriptions

| # | AsyncAPI channel subscribed | Event class | Handler behavior | Dead-letter policy |
|---|---|---|---|---|
| 1 | `workflow-events/cell.assigned` | `CellAssignedPayload` | updates tenant route resolver with current cell | retry 8 times, then `application.dlq.cell_assigned` |
| 2 | `workflow-events/cell.rebalanced` | `CellRebalancedPayload` | invalidates route and bundle caches for tenant | retry 8 times, then `application.dlq.cell_rebalanced` |
| 3 | `api-gateway.route-cache-invalidated` | `GatewayRouteCacheInvalidated` | refreshes application-side route epoch | retry 6 times, then `application.dlq.gateway_route_cache_invalidated` |
| 4 | `cloud-secrets.secret.rotated` | `SecretLifecyclePayload` | reloads session signer and module signing secret references | retry 10 times, then `application.dlq.secret_rotated` |
| 5 | `cloud-iac.apply.completed` | `ApplyCompletedPayload` | marks module deployment as current | retry 6 times, then `application.dlq.iac_apply_completed` |
| 6 | `developer-sdk.oya.developer-sdk.codegen` | `SdkCodegenEmitted` | verifies generated SDK manifests against module contract | retry 4 times, then `application.dlq.sdk_codegen_emitted` |
| 7 | `developer-sdk.oya.developer-sdk.onboarding` | `DeveloperKycPassed` | enables sandbox module publishing for developer tenant | retry 6 times, then `application.dlq.developer_kyc_passed` |
| 8 | `payments.payment-events.{tenant_id}` | `SubscriptionCancelled` | disables paid module entitlements | retry 5 times, then `application.dlq.subscription_cancelled` |
| 9 | `audit-chain.audit.seal.minted` | `SealMintedPayload` | closes application audit receipt loop | retry 10 times, then `application.dlq.audit_seal_minted` |
| 10 | `cell.workflow-events/cell.boundary.violation.detected` | `CellBoundaryViolationDetectedPayload` | refuses affected route resolution until compliance clears boundary | retry 10 times, then `application.dlq.cell_boundary_violation` |

## Event Emissions

| # | AsyncAPI channel published | Event class | Payload schema | Downstream consumers |
|---|---|---|---|---|
| 1 | `workflow-events/application.session.started` | `SessionStarted` | `application-events.yaml#/components/schemas/SessionStartedPayload` | `api-gateway`, `audit-chain`, `observability` |
| 2 | `workflow-events/application.session.ended` | `SessionEnded` | `SessionEndedPayload` | `api-gateway`, `audit-chain` |
| 3 | `workflow-events/application.module.loaded` | `ModuleLoaded` | `ModuleLoadedPayload` | `api-gateway`, `developer-sdk`, `observability` |
| 4 | `workflow-events/application.module.load.rejected` | `ModuleLoadRejected` | `ModuleLoadRejectedPayload` | `developer-sdk`, `audit-chain`, `ops-dashboard-control-center` |
| 5 | `workflow-events/application.route.access.denied` | `RouteAccessDenied` | `RouteAccessDeniedPayload` | `api-gateway`, `compliance`, `audit-chain` |
| 6 | `workflow-events/application.cdn.purge.requested` | `CdnPurgeRequested` | `CdnPurgeRequestedPayload` | `cloud-iac`, `api-gateway` |
| 7 | `workflow-events/application.module.rolled.back` | `ModuleRolledBack` | `ModuleRolledBackPayload` | `api-gateway`, `developer-sdk`, `observability` |
| 8 | `audit-chain /emit` | `ApplicationRouteResolved` | `AuditEvent.payload` with `route`, `tenant_id`, `cell_id` | `audit-chain` |
| 9 | `audit-chain /emit` | `ApplicationAuthStarted` | `AuditEvent.payload` with `provider`, `tenant_id`, `state_hash` | `audit-chain`, `identity` |
| 10 | `audit-chain /emit` | `TenantPolicyDraftCreated` | `AuditEvent.payload` with `draft_id`, `draft_kind`, `admin_principal_id` | `audit-chain`, `compliance` |
| 11 | `audit-chain /emit` | `TenantPolicyDraftApplied` | `AuditEvent.payload` with `draft_id`, `policy_fragment_id`, `evaluation_id` | `audit-chain`, `compliance` |
| 12 | `audit-chain /emit` | `ApplicationModuleManifestPublished` | `AuditEvent.payload` with `module`, `version`, `sri_hash`, `signer_key_id` | `audit-chain`, `developer-sdk` |
| 13 | `observability.application-route` | `ApplicationRouteMetricRecorded` | inline `ApplicationRouteMetric` | `observability` |

## Synchronous vs Asynchronous Boundaries

| # | Boundary | Mode | Reasoning |
|---|---|---|---|
| 1 | `api-gateway` to `resolve_route` | synchronous | edge cannot proxy until application returns route policy and pack residency |
| 2 | `api-gateway` to `get_module_manifest` | synchronous | bundle signature and SRI hash must be known before serving module |
| 3 | auth start and callback | synchronous | user session cannot be created without immediate identity response |
| 4 | tenant-admin draft create | synchronous | admin UI needs draft id and simulation preconditions |
| 5 | tenant-admin draft apply | synchronous | policy mutation must return final audit id and status |
| 6 | application to `cell getCellAssignment` | synchronous | route resolution depends on current cell |
| 7 | application to `audit-chain emitEvent` for policy apply | synchronous | tenant policy mutation is not durable without receipt |
| 8 | application to `cloud-secrets getSecretReference` | synchronous at boot and rotation | session signing key reference is required before issuing sessions |
| 9 | session started event | asynchronous | consumers can process after session response |
| 10 | module loaded event | asynchronous | observability and developer SDK state can lag behind successful load |
| 11 | module load rejected event | asynchronous urgent | developer notification should not block rejection response |
| 12 | route access denied event | asynchronous with audit receipt | denial is returned after audit receipt, downstream analysis can lag |
| 13 | CDN purge request | asynchronous | purge execution is external to application request path |
| 14 | module rollback event | asynchronous | route invalidation and SDK notification can replay from event |
| 15 | subscription cancellation subscription | asynchronous | entitlement revocation can be replayed and is not part of payment commit |

## Failure Mode Cascade

| # | Failure in `application` | Upstream impact | Circuit breaker | Retry policy |
|---|---|---|---|---|
| 1 | `resolve_route` timeout | `api-gateway` cannot proxy tenant route | gateway opens `application-route` breaker by route | 2 retries inside gateway deadline, then 503 |
| 2 | module manifest read failure | bundles cannot be served | gateway opens `module-manifest` breaker | retry on cache miss only |
| 3 | auth callback failure | identity cannot complete session | `auth-callback` breaker fails closed | no retry for callback without matching `state` |
| 4 | policy draft apply failure | tenant-admin UI cannot commit policy | `tenant-policy-apply` breaker blocks further applies for draft | retry with same `draft_id` idempotency key |
| 5 | audit emit failure | state-changing application action remains uncommitted | application refuses to publish event | retry 10 times into `application.audit_pending` |
| 6 | cloud-secrets signer unavailable | new sessions cannot be issued | `session-signer` breaker fails closed | retry reference fetch with version pin |
| 7 | cell assignment unavailable | route resolution fails | `cell-assignment` breaker on tenant | short retry, then deny route resolution |
| 8 | CDN purge outbox failure | stale assets may remain | `cdn-purge` breaker marks purge pending | replay purge outbox by `event_id` |
| 9 | developer SDK status unavailable | module submission UI shows pending | `developer-sdk-status` breaker returns cached stage | retry background poll |
| 10 | payment entitlement event delayed | paid modules may remain enabled briefly | entitlement breaker uses last known paid state | replay payment events by sequence |
| 11 | module load rejection DLQ growth | developer feedback delayed | `module-rejection` breaker pauses new publishes | replay DLQ after schema fix |
| 12 | route cache invalidation failure | gateway may use stale route | application emits explicit route epoch bump | gateway rejects stale epoch |

## Cross-tenant Coordination

| # | Scenario | Cedar guard pattern | Audit-mirror requirement |
|---|---|---|---|
| 1 | tenant admin applies policy for child tenant | `tenant-admin-console.cedar` with active work context and grant check | mirror `ConglomerateParentReadAction` plus `TenantPolicyDraftApplied` to parent and child |
| 2 | route resolves across jurisdictional cell | `tenant-scope.cedar` and `pack_residency` check | mirror `ConglomerateCrossJurisdictionResidencyEnforced` |
| 3 | office-scoped route uses `sub_scope_path` | `OfficeBoundaryAttemptEvaluated` guard in route context | mirror final `OfficeBoundaryAttemptAllowed` or `OfficeBoundaryAttemptDenied` |
| 4 | information-barrier module route | module manifest carries barrier tags into Cedar context | mirror `ConglomerateInformationBarrierCrossingRefused` on denial |
| 5 | personal context attempts tenant-admin policy apply | `tenant-admin-console.cedar` forbid personal context | mirror `ConglomeratePersonalTenantBoundaryRefused` |

## Data Shape Ledger

| # | Shape | Source | Required handoff fields |
|---|---|---|---|
| 1 | `SessionStartRequest` | `openapi/application.yaml` | `provider`, `redirect_uri`, `state`, `nonce`, `code_verifier_sha256` |
| 2 | `RouteResolveResponse` | `openapi/application.yaml` | `route`, `tenant_scope`, `required_roles`, `pack_residency` |
| 3 | `ModuleManifest` | `openapi/application.yaml` | `module`, `version`, `sri_hash`, `signer_key_id`, `signature`, `routes` |
| 4 | `CdnPurgeRequest` | `openapi/application.yaml` | `pattern`, `reason` |
| 5 | `TenantPolicyDraftRequest` | `openapi/tenant-admin-console.yaml` | `tenant_id`, `admin_principal_id`, `active_context_id`, `draft_kind`, `proposed_fragment` |
| 6 | `CellAssignment` | `microservices/cell/contracts/openapi/cell.yaml` | `tenant_id`, `cell_id`, `pack`, `scope` |
| 7 | `SecretReference` | `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml` | `path`, `version`, `data_class`, `rotation_policy_id` |
| 8 | `AuditEvent` | `microservices/audit-chain/contracts/openapi/audit-chain.yaml` | `tenant_id`, `source_microservice`, `event_class`, `payload`, `payload_data_class` |
| 9 | `Provenance` | `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml` | `artifact_digest`, `slsa_level`, `sigstore_signature`, `builder_id` |
| 10 | `VettingStageEvent` | `microservices/developer-sdk/contracts/openapi/developer-sdk.yaml` | `submission_id`, `stage`, `status`, `emitted_at` |

## Cedar Guard Ledger

| # | Policy file | Principal | Action | Resource |
|---|---|---|---|---|
| 1 | `tenant-scope.cedar` | `Service::api-gateway` | `Application::resolve_route` | `Route::{route}` |
| 2 | `tenant-scope.cedar` | `Service::api-gateway` | `Application::read_module_manifest` | `Module::{module}` |
| 3 | `tenant-admin-console.cedar` | `Role::tenant-admin` | `CreateTenantPolicyDraft` | `Tenant::{tenant_id}` |
| 4 | `tenant-admin-console.cedar` | `Role::tenant-admin` | `ApplyTenantPolicyDraft` | `PolicyDraft::{draft_id}` |
| 5 | `ci-scope.cedar` | `Service::cloud-iac` | `Application::publish_module_manifest` | `Module::{module}` |
| 6 | `ci-scope.cedar` | `Service::cloud-iac` | `Application::purge_cdn` | `CdnPattern::{pattern}` |
| 7 | `developer-scope.cedar` | `Service::developer-sdk` | `Application::read_sandbox_manifest` | `Module::{module}` |
| 8 | `auditor-scope.cedar` | `Service::audit-chain` | `Application::read_session_for_audit` | `Session::{session_id}` |
| 9 | `public-read.cedar` | `Service::ops-dashboard-control-center` | `Application::read_status` | `Status::application` |
| 10 | `secret-isolation.md` | `Service::application` | `CloudSecrets::read_reference` | `SecretReference::{tenant}/application/*` |

## Audit Event Class Ledger

| # | Audit class | Emitting handoff | ADR-0263 envelope fields that must be present |
|---|---|---|---|
| 1 | `ApplicationRouteResolved` | `GET /routes/resolve` | `tenant_id`, `route`, `cell_id`, `audit_id` |
| 2 | `ApplicationAuthStarted` | `POST /auth/start` | `tenant_id`, `provider`, `state_hash`, `trace_id` |
| 3 | `ApplicationAuthCallbackAccepted` | `GET /auth/callback` | `tenant_id`, `principal_svid`, `audit_id` |
| 4 | `ApplicationModuleManifestPublished` | `POST /modules/{module}/manifest:publish` | `module`, `version`, `sri_hash`, `signer_key_id` |
| 5 | `TenantPolicyDraftCreated` | tenant-admin draft create | `tenant_id`, `draft_id`, `draft_kind`, `admin_principal_id` |
| 6 | `TenantPolicyDraftApplied` | tenant-admin draft apply | `tenant_id`, `draft_id`, `policy_fragment_id`, `evaluation_id` |
| 7 | `ApplicationRouteAccessDenied` | route denial event | `action`, `resource_ref`, `decision`, `cedar_policy_version` |
| 8 | `OfficeBoundaryAttemptEvaluated` | office scoped route decision | `sub_scope_path`, `action`, `resource_ref`, `decision` |
| 9 | `ConglomeratePersonalTenantBoundaryRefused` | personal context policy refusal | `tenant_id`, `context_type`, `resource_ref`, `audit_id` |

## Handoff Control Checklist

1. `resolve_route` must return `pack_residency`.
2. `resolve_route` must include the active `cell_id`.
3. `resolve_route` must include route-required roles.
4. `resolve_route` must attach `cedar_policy_version`.
5. `resolve_route` must reject unknown route ids.
6. `get_module_manifest` must return `sri_hash`.
7. `get_module_manifest` must return `signer_key_id`.
8. `get_module_manifest` must return `signature`.
9. `publish_module_manifest` must validate provenance before storing.
10. `publish_module_manifest` must emit `ApplicationModuleManifestPublished`.
11. `start_auth` must hash `state` before audit emission.
12. `start_auth` must store `nonce` binding.
13. `auth_callback` must match stored `state`.
14. `saml_callback` must verify assertion audience.
15. `delete_session` must emit `SessionEnded`.
16. `get_session` for audit must use `auditor-scope.cedar`.
17. `purge_cdn` must require a reason.
18. `purge_cdn` must publish `CdnPurgeRequested`.
19. Tenant policy draft creation must require `active_context_id`.
20. Tenant policy draft creation must reject personal context.
21. Tenant policy draft apply must require a successful simulation.
22. Tenant policy draft apply must include `policy_fragment_id`.
23. Tenant policy draft apply must emit audit before policy activation.
24. `cell.assigned` handling must refresh route resolver state.
25. `cell.rebalanced` handling must invalidate affected route caches.
26. `gateway route-cache-invalidated` handling must update route epoch.
27. `secret.rotated` handling must reload signer references by version.
28. `cloud-iac.apply.completed` handling must update deployment digest.
29. Developer KYC event handling must affect sandbox only.
30. Payment subscription cancellation must update entitlements.
31. Audit seal minted handling must close receipt status.
32. Boundary violation handling must refuse affected route resolution.
33. Application route metrics must hash tenant identifiers.
34. Module load rejection incidents must not include module source code.
35. Module manifest reads must not disclose unpublished versions.
36. Bundle pointer reads must not bypass route authorization.
37. Session events must include `audit_id`.
38. Route denial events must include Cedar action and resource.
39. Module rollback events must include previous and target versions.
40. CDN purge events must include pattern and reason.
41. The outbox must key events by `event_id`.
42. The outbox must preserve tenant ordering.
43. The DLQ must be channel-specific.
44. The `route-resolution` breaker must fail closed.
45. The `session-signer` breaker must fail closed.
46. The `module-manifest` breaker may return cached signed manifests only.
47. Cached manifests must not outlive signer revocation.
48. Cached route responses must not outlive cell assignment epoch.
49. Policy draft retries must use `draft_id`.
50. Auth callback retries must not create duplicate sessions.
51. Application audit events must set `source_microservice=application`.
52. Application audit events must include `tenant_id`.
53. Application audit events must include `trace_id`.
54. Application audit events must include `span_id`.
55. Application audit events must include `audit_id`.
56. Application audit events must be PII-scrubbed.
57. Cross-tenant route reads must mirror parent and child partitions.
58. Office-scoped routes must include `sub_scope_path`.
59. Information-barrier tags must be passed into Cedar context.
60. Personal-context route attempts must be refused.
61. Developer sandbox manifests must not resolve production entitlements.
62. Payment entitlement reads must not expose payment instruments.
63. Cloud-IAC provenance reads must require SLSA fields.
64. Cloud-secrets references must remain version-pinned.
65. Gateway route refreshes must include route epoch.
66. Gateway route refreshes must reject stale epoch responses.
67. Observability metrics must not include raw session ids.
68. Operator incidents must include module and reason only.
69. `application` must update this matrix when `application.yaml` changes.
70. `application` must update this matrix when `tenant-admin-console.yaml` changes.

## Checkpoint

- Authored for `application` on 2026-05-20.
- Source contracts checked: `application.yaml`, `tenant-admin-console.yaml`, `application-events.yaml`, and application proto.
- Source policies checked: `tenant-scope.cedar`, `tenant-admin-console.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`.
- No in-flight microservice directories were edited.
- Oya VCS scope: `microservices`.
