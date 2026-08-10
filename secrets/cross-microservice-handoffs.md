---
doc_class: CrossMicroserviceHandoffMatrix
title: "Cloud Secrets Cross-Microservice Handoff Matrix"
status: Draft
date: 2026-05-20
microservice: cloud-secrets
owner_team: axis-cloud-secrets
---

# Cloud Secrets Cross-Microservice Handoff Matrix

This matrix records concrete handoffs for the `cloud-secrets` microservice.
REST shapes are from `secrets/contracts/openapi/cloud-secrets.yaml`.
Async shapes are from `secrets/contracts/asyncapi/cloud-secrets-events.yaml`.
gRPC names are from `secrets/contracts/proto/cloud-secrets.proto`.
Cedar policies are from `secrets/policy/`.
Audit-chain emission follows ADR-0263 with `source_microservice=cloud-secrets`.
The service owns secret references, rotation, revocation, tenant namespaces, BYOK, KEK attestation, and revocation pushes.

## Inbound Callers

| # | Calling microservice | Named API on `cloud-secrets` | Data shape | Cedar permit required | Audit event emitted |
|---|---|---|---|---|---|
| 1 | `api-gateway` | `GET /secrets/{tenant}/api-gateway/tls-cert/reference` `getSecretReference` | `SecretReference` with `path`, `version`, `data_class`, `rotation_policy_id` | `secret-isolation.md` guard `gateway_tls_secret_read` | `CloudSecretsReferenceRead` |
| 2 | `application` | `GET /secrets/{tenant}/application/session-signer/reference` `getSecretReference` | `SecretReference` | `secret-isolation.md` guard `application_session_secret_read` | `CloudSecretsReferenceRead` |
| 3 | `cloud-iac` | `GET /secrets/{tenant}/cloud-iac/gitops-signer/reference` `getSecretReference` | `SecretReference` | `secret-isolation.md` guard `cloud_iac_signer_read` | `CloudSecretsReferenceRead` |
| 4 | `cloud-iac` | `GET /secrets/{tenant}/cloud-iac/kubeconfig/reference` `getSecretReference` | `SecretReference` | `secret-isolation.md` guard `cloud_iac_kubeconfig_read` | `CloudSecretsReferenceRead` |
| 5 | `cell` | `GET /secrets/{tenant}/cell/cell-signer/reference` `getSecretReference` | `SecretReference` | `secret-isolation.md` guard `cell_signer_read` | `CloudSecretsReferenceReadForCell` |
| 6 | `payments` | `GET /secrets/{tenant}/payments/psp-token/reference` `getSecretReference` | `SecretReference` | `secret-isolation.md` guard `payments_psp_token_read` | `CloudSecretsReferenceReadForPayments` |
| 7 | `developer-sdk` | `GET /secrets/{tenant}/developer-sdk/signing-key/reference` `getSecretReference` | `SecretReference` | `developer-scope.cedar` action `CloudSecrets::read_developer_signing_reference` | `CloudSecretsDeveloperSigningReferenceRead` |
| 8 | `audit-chain` | `GET /secrets/{tenant}/audit-chain/seal-key/reference` `getSecretReference` | `SecretReference` | `auditor-scope.cedar` action `CloudSecrets::read_audit_seal_reference` | `CloudSecretsAuditSealReferenceRead` |
| 9 | `tenancy` | `GET /tenants/{tenant}/namespace` `getTenantNamespace` | `TenantNamespace` | `tenant-scope.cedar` action `CloudSecrets::read_tenant_namespace` | `CloudSecretsNamespaceRead` |
| 10 | `tenancy` | `POST /tenants/{tenant}/byok` `uploadByok` | `BYOKUploadRequest` | `tenant-scope.cedar` action `CloudSecrets::upload_byok` | `CloudSecretsByokUploaded` |
| 11 | `compliance` | `GET /attestation/{pack}/reports` `listAttestationReports` | `AttestationReport[]` | `auditor-scope.cedar` action `CloudSecrets::read_attestation_reports` | `CloudSecretsAttestationReportsRead` |
| 12 | `ops-dashboard-control-center` | `GET /rotation-policies/{policy_id}` `getRotationPolicy` | `RotationPolicy` | `auditor-scope.cedar` action `CloudSecrets::read_rotation_policy_operator` | `CloudSecretsRotationPolicyRead` |
| 13 | `api-gateway` | `GET /health` and `GET /ready` | `HealthResponse`, `ReadyResponse` | `public-read.cedar` action `CloudSecrets::read_health` | `CloudSecretsHealthChecked` |
| 14 | `audit-chain` | `GET /audit/query` `queryAudit` | `SecretAuditQueryResult` | `auditor-scope.cedar` action `CloudSecrets::query_secret_audit` | `CloudSecretsAuditQueried` |

## Outbound Callees

| # | Callee microservice | Named API called by `cloud-secrets` | Data shape sent or received | Cedar permit required | Audit event consumed |
|---|---|---|---|---|---|
| 1 | `audit-chain` | `POST /emit` `emitEvent` | `AuditEvent` with `source_microservice=cloud-secrets` | `tenant-scope.cedar` action `AuditChain::emit_secret_event` | consumes `CloudSecretsAuditReceiptAccepted` |
| 2 | `cell` | `GET /tenants/{tenant_id}/assignment` `getCellAssignment` | `CellAssignment` for residency and namespace placement | `tenant-scope.cedar` action `Cell::resolve_secret_namespace_cell` | consumes `CellResidencyReadForSecrets` |
| 3 | `cloud-iac` | `GET /charts/{digest}/provenance` `getProvenance` | `Provenance` for secret controller chart | `ci-scope.cedar` action `CloudIac::read_secret_chart_provenance` | consumes `CloudIacSecretChartProvenanceRead` |
| 4 | `cloud-k8s` | `POST /namespaces/{tenant}/secret-sync` | inline `SecretSyncRequest {tenant, namespace, version}` | `ci-scope.cedar` action `CloudK8s::sync_secret_reference` | consumes `K8sSecretSyncAccepted` |
| 5 | `api-gateway` | `POST /revocation-push/secret/{path}` | `RevocationPushPayload` | `secret-isolation.md` guard `push_gateway_secret_revocation` | consumes `ApiGatewaySecretRevocationAccepted` |
| 6 | `application` | `POST /revocation-push/secret/{path}` | `RevocationPushPayload` for session signer | `secret-isolation.md` guard `push_application_secret_revocation` | consumes `ApplicationSecretRevocationAccepted` |
| 7 | `payments` | `POST /revocation-push/secret/{path}` | `RevocationPushPayload` for PSP token | `secret-isolation.md` guard `push_payments_secret_revocation` | consumes `PaymentsSecretRevocationAccepted` |
| 8 | `developer-sdk` | `POST /developers/{developerId}/signing-keys/{keyId}/revoke` | signing key revocation request | `developer-scope.cedar` action `DeveloperSdk::revoke_signing_key` | consumes `SigningKeyRevoked` |
| 9 | `observability` | `POST /metrics/secret-rotation` | inline `SecretRotationMetric {tenant_hash, microservice, policy_id}` | `public-read.cedar` action `Observability::write_secret_metric` | consumes `MetricAccepted` |
| 10 | `ops-dashboard-control-center` | `POST /incidents/rotation-overdue` | inline `RotationOverdueIncident {policy_id, secret_path_hash}` | `auditor-scope.cedar` action `OpsDashboard::open_secret_incident` | consumes `OpsIncidentOpened` |
| 11 | `compliance` | `POST /residency/evaluate` | inline `SecretResidencyRequest {tenant, path, data_class, pack}` | `data-residency.md` guard `secret_residency_allowed` | consumes `ResidencyDecisionRecorded` |
| 12 | `identity` | `POST /service-token/introspect` | inline `ServiceTokenIntrospectionRequest {svid, audience}` | `tenant-scope.cedar` action `Identity::introspect_secret_caller` | consumes `IdentityPrincipalIntrospected` |

## Event Subscriptions

| # | AsyncAPI channel subscribed | Event class | Handler behavior | Dead-letter policy |
|---|---|---|---|---|
| 1 | `workflow-events/tenant.onboarded` | `TenantOnboarded` | provisions tenant namespace and emits `namespace.provisioned` | retry 8 times, then `cloud-secrets.dlq.tenant_onboarded` |
| 2 | `workflow-events/tenant.deprovisioned` | `TenantDeprovisioned` | seals namespace and schedules destruction after retention | retry 8 times, then `cloud-secrets.dlq.tenant_deprovisioned` |
| 3 | `workflow-events/cell.assigned` | `CellAssignedPayload` | pins namespace placement to assigned cell and residency pack | retry 8 times, then `cloud-secrets.dlq.cell_assigned` |
| 4 | `workflow-events/cell.rebalanced` | `CellRebalancedPayload` | migrates namespace reference metadata, not raw secret material | retry 8 times, then `cloud-secrets.dlq.cell_rebalanced` |
| 5 | `cloud-iac.apply.completed` | `ApplyCompletedPayload` | refreshes controller deployment digest | retry 6 times, then `cloud-secrets.dlq.iac_apply_completed` |
| 6 | `audit-chain.audit.seal.minted` | `SealMintedPayload` | closes secret audit receipt state | retry 10 times, then `cloud-secrets.dlq.audit_seal_minted` |
| 7 | `developer-sdk.oya.developer-sdk.signing-key` | `SigningKeyIssued` | records developer signing-key reference metadata | retry 6 times, then `cloud-secrets.dlq.signing_key_issued` |
| 8 | `payments.payment-events.{tenant_id}` | `SubMerchantRestricted` | rotates PSP token for restricted sub-merchant scope | retry 5 times, then `cloud-secrets.dlq.submerchant_restricted` |
| 9 | `api-gateway.upstream.circuit-open` | `GatewayCircuitOpen` | suppresses revocation push retries to unavailable gateway path | retry 4 times, then `cloud-secrets.dlq.gateway_circuit_open` |
| 10 | `cell.workflow-events/cell.boundary.violation.detected` | `CellBoundaryViolationDetectedPayload` | blocks secret reference reads for affected tenant-cell pair | retry 10 times, then `cloud-secrets.dlq.cell_boundary_violation` |

## Event Emissions

| # | AsyncAPI channel published | Event class | Payload schema | Downstream consumers |
|---|---|---|---|---|
| 1 | `workflow-events/secret.created` | `SecretLifecyclePayload` | `cloud-secrets-events.yaml#/components/schemas/SecretLifecyclePayload` | `audit-chain`, `observability`, owning microservice |
| 2 | `workflow-events/secret.rotated` | `SecretLifecyclePayload` | `SecretLifecyclePayload` | `api-gateway`, `application`, `cloud-iac`, `payments` |
| 3 | `workflow-events/secret.revoked` | `SecretRevokedPayload` | `SecretRevokedPayload` | `api-gateway`, `application`, `cloud-iac`, `payments`, `developer-sdk` |
| 4 | `workflow-events/secret.accessed` | `SecretAccessedPayload` | `SecretAccessedPayload` | `audit-chain`, `compliance` |
| 5 | `workflow-events/namespace.provisioned` | `NamespaceLifecyclePayload` | `NamespaceLifecyclePayload` | `tenancy`, `cell`, `audit-chain` |
| 6 | `workflow-events/namespace.sealed` | `NamespaceLifecyclePayload` | `NamespaceLifecyclePayload` | `tenancy`, `cell`, `audit-chain` |
| 7 | `workflow-events/kek.attested` | `KekAttestedPayload` | `KekAttestedPayload` | `compliance`, `audit-chain` |
| 8 | `workflow-events/rotation.overdue` | `RotationOverduePayload` | `RotationOverduePayload` | `ops-dashboard-control-center`, `audit-chain` |
| 9 | `revocation-push/secret/{path}` | `RevocationPushPayload` | `RevocationPushPayload` | owning microservice, `api-gateway`, `application`, `payments` |
| 10 | `audit-chain /emit` | `CloudSecretsReferenceRead` | `AuditEvent.payload` with `secret_path_hash`, `tenant_hash`, `microservice`, `version` | `audit-chain` |
| 11 | `audit-chain /emit` | `CloudSecretsSecretRotated` | `AuditEvent.payload` with `secret_path_hash`, `version`, `rotation_policy_id` | `audit-chain`, `compliance` |
| 12 | `audit-chain /emit` | `CloudSecretsSecretRevoked` | `AuditEvent.payload` with `secret_path_hash`, `reason`, `version` | `audit-chain`, owning microservice |
| 13 | `audit-chain /emit` | `CloudSecretsByokUploaded` | `AuditEvent.payload` with `tenant`, `wrap_algorithm`, `jit_token_hash` | `audit-chain`, `compliance` |
| 14 | `observability.secret-rotation` | `SecretRotationMetricRecorded` | inline `SecretRotationMetric` | `observability` |

## Synchronous vs Asynchronous Boundaries

| # | Boundary | Mode | Reasoning |
|---|---|---|---|
| 1 | `getSecretReference` | synchronous | caller cannot use secret without current versioned reference |
| 2 | `listSecretReferences` | synchronous | controllers need exact reference inventory before reconciliation |
| 3 | `rotateSecret` | synchronous admission, asynchronous propagation | caller needs rotation job id; consumers update through events |
| 4 | `revokeSecret` | synchronous admission, asynchronous revocation push | revocation must be accepted before push fan-out |
| 5 | `getTenantNamespace` | synchronous | tenancy and controllers need namespace state before creating dependent resources |
| 6 | `uploadByok` | synchronous | BYOK must be wrapped and audited before accepted |
| 7 | `listAttestationReports` | synchronous | compliance must block on attestation evidence |
| 8 | `queryAudit` | synchronous | auditors expect direct query response |
| 9 | `secret.rotated` emission | asynchronous | consumers reload references after rotation commit |
| 10 | `secret.revoked` emission | asynchronous urgent | consumers must converge quickly but not block revocation commit |
| 11 | `revocation-push/secret/{path}` | asynchronous urgent | direct fan-out reduces stale credential windows |
| 12 | `namespace.provisioned` emission | asynchronous | tenancy and cell can observe after namespace commit |
| 13 | `audit-chain emitEvent` for access and mutation | synchronous | ADR-0263 receipt required before responding to caller |
| 14 | `cell getCellAssignment` | synchronous | residency and namespace placement require current cell |
| 15 | `cloud-k8s secret-sync` | asynchronous | Kubernetes reconciliation can proceed after secret metadata commit |

## Failure Mode Cascade

| # | Failure in `cloud-secrets` | Upstream impact | Circuit breaker | Retry policy |
|---|---|---|---|---|
| 1 | reference read timeout | gateway, application, payments, and cloud-iac cannot reload credentials | caller opens `secret-reference` breaker per secret path | 3 retries, then fail closed |
| 2 | rotation job failure | consumers remain on old version | `secret-rotation` breaker blocks new rotations for policy | retry with same rotation id |
| 3 | revocation push failure | consumers may retain revoked reference | `revocation-push` breaker tracks target microservice | retry urgent fan-out until ack or manual quarantine |
| 4 | namespace provisioning failure | tenancy cannot finish tenant bootstrap | `namespace-provision` breaker fails closed | retry by tenant namespace id |
| 5 | BYOK upload failure | tenant key adoption blocks | `byok-upload` breaker rejects dependent secrets | retry with same wrapped key digest |
| 6 | audit emit failure | secret access or mutation is refused | `secret-audit` breaker fails closed | retry 10 times, then hold in `cloud_secrets.audit_pending` |
| 7 | cell assignment failure | namespace residency cannot be proven | `cell-residency` breaker fails closed | retry until cell assignment deadline |
| 8 | attestation report unavailable | compliance cannot approve pack | `kek-attestation` breaker marks pack untrusted | retry report fetch by pack |
| 9 | event bus unavailable | rotations and revocations lag | outbox breaker spools events | replay by `event_id` |
| 10 | DLQ saturation | namespace or revocation convergence delayed | `secret-dlq` breaker pauses noncritical rotations | replay oldest-first after operator review |
| 11 | secret isolation policy error | reference reads fail | `cedar-secret-isolation` breaker denies all reads | no fallback permit |
| 12 | data residency denial | secret namespace cannot be created | residency breaker blocks namespace commit | no retry until policy or pack changes |

## Cross-tenant Coordination

| # | Scenario | Cedar guard pattern | Audit-mirror requirement |
|---|---|---|---|
| 1 | conglomerate parent reads child tenant secret reference metadata | `secret-isolation.md` plus active grant in `tenant-scope.cedar` | mirror `ConglomerateParentReadAction` to parent and child partitions |
| 2 | BYOK applies to child tenant under parent grant | `CloudSecrets::upload_byok` with child `Tenant::{tenant}` resource | mirror `CloudSecretsByokUploaded` and `ConglomerateGrantCreated` reference |
| 3 | secret namespace crosses jurisdiction | `data-residency.md` guard `secret_residency_allowed` | mirror `ConglomerateCrossJurisdictionResidencyEnforced` |
| 4 | office pack secret overlay | `OfficePackOverlayActivated` with `sub_scope_path` | mirror `OfficePackOverlayChanged` after rotation |
| 5 | personal context attempts secret reference read | `secret-isolation.md` forbids personal context for work secret | mirror `ConglomeratePersonalTenantBoundaryRefused` |

## Data Shape Ledger

| # | Shape | Source | Required handoff fields |
|---|---|---|---|
| 1 | `SecretReference` | `openapi/cloud-secrets.yaml` | `path`, `version`, `data_class`, `rotation_policy_id` |
| 2 | `SecretRotateRequest` | `openapi/cloud-secrets.yaml` | `path` |
| 3 | `SecretRevokeRequest` | `openapi/cloud-secrets.yaml` | `path`, `reason` |
| 4 | `BYOKUploadRequest` | `openapi/cloud-secrets.yaml` | `tenant`, `wrapped_byok`, `wrap_algorithm`, `jit_token` |
| 5 | `SecretLifecyclePayload` | `asyncapi/cloud-secrets-events.yaml` | `event_id`, `secret_path_hash`, `tenant_hash`, `microservice`, `version`, `occurred_at`, `signature` |
| 6 | `SecretRevokedPayload` | `asyncapi/cloud-secrets-events.yaml` | `event_id`, `secret_path_hash`, `version`, `reason`, `occurred_at` |
| 7 | `SecretAccessedPayload` | `asyncapi/cloud-secrets-events.yaml` | `event_id`, `secret_path_hash`, `tenant_hash`, `microservice`, `principal_svid` |
| 8 | `NamespaceLifecyclePayload` | `asyncapi/cloud-secrets-events.yaml` | `tenant_hash`, `namespace`, `state`, `occurred_at` |
| 9 | `KekAttestedPayload` | `asyncapi/cloud-secrets-events.yaml` | `pack`, `kek_id`, `attestation_digest`, `occurred_at` |
| 10 | `RevocationPushPayload` | `asyncapi/cloud-secrets-events.yaml` | `path`, `version`, `reason`, `deadline_at` |

## Cedar Guard Ledger

| # | Policy file | Principal | Action | Resource |
|---|---|---|---|---|
| 1 | `secret-isolation.md` | `Service::api-gateway` | `CloudSecrets::read_reference` | `SecretReference::{tenant}/api-gateway/tls-cert` |
| 2 | `secret-isolation.md` | `Service::application` | `CloudSecrets::read_reference` | `SecretReference::{tenant}/application/session-signer` |
| 3 | `secret-isolation.md` | `Service::cloud-iac` | `CloudSecrets::read_reference` | `SecretReference::{tenant}/cloud-iac/*` |
| 4 | `secret-isolation.md` | `Service::payments` | `CloudSecrets::read_reference` | `SecretReference::{tenant}/payments/psp-token` |
| 5 | `tenant-scope.cedar` | `Service::tenancy` | `CloudSecrets::read_tenant_namespace` | `Tenant::{tenant}` |
| 6 | `tenant-scope.cedar` | `Service::tenancy` | `CloudSecrets::upload_byok` | `Tenant::{tenant}` |
| 7 | `developer-scope.cedar` | `Service::developer-sdk` | `CloudSecrets::read_developer_signing_reference` | `Developer::{developerId}` |
| 8 | `auditor-scope.cedar` | `Service::compliance` | `CloudSecrets::read_attestation_reports` | `Pack::{pack}` |
| 9 | `public-read.cedar` | `Service::api-gateway` | `CloudSecrets::read_health` | `Health::cloud-secrets` |
| 10 | `data-residency.md` | `Service::cloud-secrets` | `CloudSecrets::place_namespace` | `Namespace::{tenant}` |

## Audit Event Class Ledger

| # | Audit class | Emitting handoff | ADR-0263 envelope fields that must be present |
|---|---|---|---|
| 1 | `CloudSecretsReferenceRead` | `getSecretReference` | `tenant_id`, `secret_path_hash`, `microservice`, `version`, `audit_id` |
| 2 | `CloudSecretsSecretRotated` | `rotateSecret` | `secret_path_hash`, `version`, `rotation_policy_id`, `audit_id` |
| 3 | `CloudSecretsSecretRevoked` | `revokeSecret` | `secret_path_hash`, `reason`, `version`, `audit_id` |
| 4 | `CloudSecretsSecretAccessed` | `secret.accessed` | `principal_svid`, `secret_path_hash`, `decision`, `audit_id` |
| 5 | `CloudSecretsNamespaceProvisioned` | namespace create | `tenant_id`, `namespace`, `cell_id`, `audit_id` |
| 6 | `CloudSecretsNamespaceSealed` | tenant deprovision | `tenant_id`, `namespace`, `sealed_at`, `audit_id` |
| 7 | `CloudSecretsByokUploaded` | BYOK upload | `tenant_id`, `wrap_algorithm`, `jit_token_hash`, `audit_id` |
| 8 | `CloudSecretsKekAttested` | KEK attestation | `pack`, `kek_id`, `attestation_digest`, `audit_id` |
| 9 | `ConglomerateCrossJurisdictionResidencyEnforced` | namespace placement | `jurisdiction_code`, `policy_pack`, `resource_ref`, `decision` |

## Handoff Control Checklist

1. `getSecretReference` must never return secret material.
2. `getSecretReference` must return only `path`, `version`, `data_class`, and rotation policy.
3. `getSecretReference` must audit every read.
4. `listSecretReferences` must filter by tenant and microservice.
5. `rotateSecret` must create a new immutable version.
6. `rotateSecret` must emit `secret.rotated`.
7. `rotateSecret` must not delete the previous version before consumer ack.
8. `revokeSecret` must emit `secret.revoked`.
9. `revokeSecret` must enqueue `revocation-push/secret/{path}`.
10. `getTenantNamespace` must verify tenant-cell residency.
11. `uploadByok` must require a JIT token.
12. `uploadByok` must hash the JIT token before audit.
13. `listAttestationReports` must be auditor-scoped.
14. `queryAudit` must never expose raw secret values.
15. `getRotationPolicy` must include rotation deadline.
16. `secret.created` must include secret path hash.
17. `secret.rotated` must include version.
18. `secret.revoked` must include reason.
19. `secret.accessed` must include principal SVID.
20. `namespace.provisioned` must include namespace state.
21. `namespace.sealed` must include retention marker.
22. `kek.attested` must include attestation digest.
23. `rotation.overdue` must include policy id.
24. Revocation push must include deadline.
25. API gateway TLS references must be gateway-scoped.
26. Application signer references must be application-scoped.
27. Cloud-IAC kubeconfig references must be IAC-scoped.
28. Payments PSP token references must be payments-scoped.
29. Developer signing references must be developer-scoped.
30. Audit-chain seal-key references must be auditor-scoped.
31. Namespace provisioning must consume current cell assignment.
32. Cell rebalancing must move metadata only.
33. Secret material must not move through cell events.
34. Secret rotation metrics must hash tenant ids.
35. Rotation overdue incidents must hash secret paths.
36. Cloud-k8s sync requests must carry version only.
37. Cloud-k8s sync requests must not carry secret values.
38. Revocation pushes must be idempotent by path and version.
39. Reference reads must fail closed when Cedar is unavailable.
40. Reference reads must fail closed when residency denies.
41. BYOK upload must fail closed when audit-chain is unavailable.
42. Rotation must fail closed when audit-chain is unavailable.
43. Revocation must fail closed when audit-chain is unavailable.
44. Event outbox must key by `event_id`.
45. Event outbox must preserve order per secret path.
46. DLQ replay must process revocations before rotations for same path.
47. Gateway circuit events must pause gateway revocation pushes only.
48. Application revocation pushes must continue when gateway is down.
49. Payments revocation pushes must include PSP scope.
50. Developer signing-key revocation must call developer-sdk revoke API.
51. Secret audit events must include `source_microservice=cloud-secrets`.
52. Secret audit events must include `tenant_id` or `tenant_hash`.
53. Secret audit events must include `trace_id`.
54. Secret audit events must include `span_id`.
55. Secret audit events must include `audit_id`.
56. Secret audit events must include `payload_data_class`.
57. Cross-tenant secret reads must mirror both tenant partitions.
58. Cross-jurisdiction namespace placement must mirror residency enforcement.
59. Office overlays must include `sub_scope_path`.
60. Personal-context reads must be refused.
61. Information-barrier taints must be preserved on secret metadata.
62. Secret data class must be part of Cedar context.
63. Rotation policy id must be part of Cedar context.
64. The service must reject unsigned internal callers.
65. The service must keep health and ready public-read only.
66. The service must never expose secret values in metrics.
67. The service must never expose secret values in audit payloads.
68. The service must record policy version on every permit or deny.
69. `cloud-secrets` must update this matrix when `cloud-secrets.yaml` changes.
70. `cloud-secrets` must update this matrix when `cloud-secrets-events.yaml` changes.

## Checkpoint

- Authored for `cloud-secrets` on 2026-05-20.
- Source contracts checked: `cloud-secrets.yaml`, `cloud-secrets-events.yaml`, and cloud-secrets proto.
- Source policies checked: `secret-isolation.md`, `tenant-scope.cedar`, `public-read.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `data-residency.md`.
- No in-flight microservice directories were edited.
- Oya VCS scope: `microservices`.
