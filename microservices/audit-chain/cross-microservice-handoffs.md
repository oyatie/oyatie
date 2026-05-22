---
doc_class: CrossMicroserviceHandoffMatrix
title: "Audit Chain Cross-Microservice Handoff Matrix"
status: Draft
date: 2026-05-20
microservice: audit-chain
owner_team: axis-audit-chain
---

# Audit Chain Cross-Microservice Handoff Matrix

This matrix records concrete handoffs for the `audit-chain` microservice.
REST shapes are from `microservices/audit-chain/contracts/openapi/audit-chain.yaml`.
Async shapes are from `microservices/audit-chain/contracts/asyncapi/audit-events.yaml`.
gRPC names are from `microservices/audit-chain/contracts/proto/audit_chain.proto`.
Cedar policies are from `microservices/audit-chain/policies/`.
Audit-chain is the ADR-0263 receipt authority for microservice state-changing handoffs.
The service owns event emission receipts, proofs, verification, export, signed roots, and public keys.

## Inbound Callers

| # | Calling microservice | Named API on `audit-chain` | Data shape | Cedar permit required | Audit event emitted |
|---|---|---|---|---|---|
| 1 | `api-gateway` | `POST /emit` `emitEvent` | `AuditEvent` with gateway admission or denial payload | `tenant-scope.cedar` action `AuditChain::emit_gateway_event` | `AuditEventAccepted` |
| 2 | `application` | `POST /emit` `emitEvent` | `AuditEvent` with route, auth, module, or tenant-admin payload | `tenant-scope.cedar` action `AuditChain::emit_application_event` | `AuditEventAccepted` |
| 3 | `cell` | `POST /emit` `emitEvent` | `AuditEvent` with assignment, migration, or boundary payload | `tenant-scope.cedar` action `AuditChain::emit_cell_event` | `AuditEventAccepted` |
| 4 | `cloud-iac` | `POST /emit` `emitEvent` | `AuditEvent` with render, apply, rollback, or drift payload | `tenant-scope.cedar` action `AuditChain::emit_iac_event` | `AuditEventAccepted` |
| 5 | `cloud-secrets` | `POST /emit` `emitEvent` | `AuditEvent` with secret reference, rotation, revocation, BYOK payload | `tenant-scope.cedar` action `AuditChain::emit_secret_event` | `AuditEventAccepted` |
| 6 | `developer-sdk` | `POST /emit` `emitEvent` | `AuditEvent` with onboarding, signing-key, sandbox, payout payload | `developer-scope.cedar` action `AuditChain::emit_developer_event` | `AuditEventAccepted` |
| 7 | `payments` | `POST /emit` `emitEvent` | `AuditEvent` with charge, refund, payout, dispute, subscription payload | `tenant-scope.cedar` action `AuditChain::emit_payment_event` | `AuditEventAccepted` |
| 8 | `compliance` | `POST /query` `queryEvents` | `QueryRequest` with tenant, event class, period filter | `auditor-scope.cedar` action `AuditChain::query_events_for_compliance` | `AuditQueryExecuted` |
| 9 | `compliance` | `POST /export` `requestExport` | `ExportRequest` | `auditor-scope.cedar` action `AuditChain::request_export` | `AuditExportRequested` |
| 10 | `ops-dashboard-control-center` | `GET /export/{export_id}` `getExportStatus` | `ExportStatus` | `auditor-scope.cedar` action `AuditChain::read_export_status` | `AuditExportStatusRead` |
| 11 | `api-gateway` | `GET /events/{event_id}/proof` `getProof` | `AuditProof` | `auditor-scope.cedar` action `AuditChain::read_gateway_proof` | `AuditProofRead` |
| 12 | `cell` | `GET /events/{event_id}/proof` `getProof` | `AuditProof` | `auditor-scope.cedar` action `AuditChain::read_cell_proof` | `AuditProofRead` |
| 13 | `application` | `POST /verify` `verifyEvent` | `VerifyRequest` with `event_envelope`, `proof`, `signed_root` | `public-read.cedar` action `AuditChain::verify_event_public` | `AuditVerificationRequested` |
| 14 | `cloud-iac` | `GET /roots/{pack}/{period_id}` `getSignedRoot` | `SignedRoot` | `auditor-scope.cedar` action `AuditChain::read_signed_root` | `AuditSignedRootRead` |
| 15 | `cloud-secrets` | `GET /keys/{pack}/{epoch_id}` `getPublicKey` | `PublicKey` | `public-read.cedar` action `AuditChain::read_public_key` | `AuditPublicKeyRead` |

## Outbound Callees

| # | Callee microservice | Named API called by `audit-chain` | Data shape sent or received | Cedar permit required | Audit event consumed |
|---|---|---|---|---|---|
| 1 | `cloud-secrets` | `GET /secrets/{tenant}/audit-chain/seal-key/reference` `getSecretReference` | `SecretReference` | `auditor-scope.cedar` action `CloudSecrets::read_audit_seal_reference` | consumes `CloudSecretsAuditSealReferenceRead` |
| 2 | `cloud-secrets` | `GET /secrets/{tenant}/audit-chain/export-key/reference` | `SecretReference` | `auditor-scope.cedar` action `CloudSecrets::read_audit_export_reference` | consumes `CloudSecretsReferenceRead` |
| 3 | `cell` | `GET /tenants/{tenant_id}/assignment` `getCellAssignment` | `CellAssignment.audit_partition` | `auditor-scope.cedar` action `Cell::read_audit_partition` | consumes `CellAuditPartitionRead` |
| 4 | `cloud-iac` | `GET /microservices/audit-chain/apply-state/{environment}` `getApplyState` | `ApplyStateRecord` | `ci-scope.cedar` action `CloudIac::read_audit_chain_apply_state` | consumes `CloudIacApplyStateRead` |
| 5 | `observability` | `POST /metrics/audit-chain` | inline `AuditChainMetric {pack, period_id, accepted_count, sealed_count}` | `public-read.cedar` action `Observability::write_audit_metric` | consumes `MetricAccepted` |
| 6 | `ops-dashboard-control-center` | `POST /incidents/audit-verification-failed` | inline `AuditVerificationIncident {event_id, period_id, reason}` | `auditor-scope.cedar` action `OpsDashboard::open_audit_incident` | consumes `OpsIncidentOpened` |
| 7 | `compliance` | `POST /retention/apply` | inline `RetentionApplyRequest {tenant_id, period_id, policy_id}` | `auditor-scope.cedar` action `Compliance::apply_retention_policy` | consumes `RetentionDecisionRecorded` |
| 8 | `api-gateway` | `POST /edge/admission` `admitEdgeRequest` | `EdgeAdmissionRequest` for proof and export routes | `route-authorization.cedar` action `Gateway::admit_audit_endpoint` | consumes `ApiGatewayAuditRouteAdmitted` |
| 9 | `identity` | `POST /service-token/introspect` | inline `ServiceTokenIntrospectionRequest {svid, audience}` | `tenant-scope.cedar` action `Identity::introspect_audit_caller` | consumes `IdentityPrincipalIntrospected` |
| 10 | `application` | `GET /auth/session` `get_session` | `SessionSummary` for export actor context | `auditor-scope.cedar` action `Application::read_session_for_audit` | consumes `ApplicationSessionReadForAudit` |
| 11 | `payments` | `GET /v1/charges/{charge_id}` `getCharge` | `Charge` projection for payment event verification | `auditor-scope.cedar` action `Payments::read_charge_for_audit` | consumes `PaymentChargeReadForAudit` |
| 12 | `developer-sdk` | `GET /developers/{developerId}/signing-keys` `listSigningKeys` | `SigningKey[]` public fingerprint projection | `auditor-scope.cedar` action `DeveloperSdk::read_signing_keys_for_audit` | consumes `DeveloperSigningKeysReadForAudit` |

## Event Subscriptions

| # | AsyncAPI channel subscribed | Event class | Handler behavior | Dead-letter policy |
|---|---|---|---|---|
| 1 | `workflow-events/secret.rotated` | `SecretLifecyclePayload` | rotates seal or export key references by version | retry 10 times, then `audit-chain.dlq.secret_rotated` |
| 2 | `workflow-events/secret.revoked` | `SecretRevokedPayload` | disables affected seal/export key epoch | retry 10 times, then `audit-chain.dlq.secret_revoked` |
| 3 | `workflow-events/cell.assigned` | `CellAssignedPayload` | refreshes tenant audit partition routing | retry 8 times, then `audit-chain.dlq.cell_assigned` |
| 4 | `cloud-iac.apply.completed` | `ApplyCompletedPayload` | records current deployment sha for verifier provenance | retry 6 times, then `audit-chain.dlq.iac_apply_completed` |
| 5 | `api-gateway.oya.api_gateway.admission` | `RequestDenied` | correlates denied request event with emitted audit envelope | retry 8 times, then `audit-chain.dlq.gateway_request_denied` |
| 6 | `application.workflow-events/application.route.access.denied` | `RouteAccessDenied` | checks route denial audit event presence | retry 8 times, then `audit-chain.dlq.application_route_denied` |
| 7 | `payments.payment-events.{tenant_id}` | `ChargeErrored` | verifies payment error audit envelope and sequence | retry 8 times, then `audit-chain.dlq.payment_charge_errored` |
| 8 | `developer-sdk.oya.developer-sdk.signing-key` | `SigningKeyRevoked` | verifies signing-key revocation receipt exists | retry 8 times, then `audit-chain.dlq.signing_key_revoked` |
| 9 | `cell.workflow-events/cell.boundary.violation.detected` | `CellBoundaryViolationDetectedPayload` | verifies boundary violation audit envelope exists | retry 10 times, then `audit-chain.dlq.cell_boundary_violation` |
| 10 | `cloud-secrets.workflow-events/rotation.overdue` | `RotationOverduePayload` | verifies overdue rotation incident is sealed | retry 8 times, then `audit-chain.dlq.rotation_overdue` |

## Event Emissions

| # | AsyncAPI channel published | Event class | Payload schema | Downstream consumers |
|---|---|---|---|---|
| 1 | `workflow-events/audit.emitted` | `AuditEmittedPayload` | `audit-events.yaml#/components/schemas/AuditEmittedPayload` | emitting service, `compliance`, `observability` |
| 2 | `workflow-events/audit.seal.minted` | `SealMintedPayload` | `SealMintedPayload` | all emitters, `compliance`, `ops-dashboard-control-center` |
| 3 | `workflow-events/audit.verification.failed` | `VerificationFailedPayload` | `VerificationFailedPayload` | `ops-dashboard-control-center`, `compliance` |
| 4 | `workflow-events/audit.retention.applied` | `RetentionAppliedPayload` | `RetentionAppliedPayload` | `compliance`, `cell`, `cloud-secrets` |
| 5 | `workflow-events/audit.key.rotated` | `KeyRotatedPayload` | `KeyRotatedPayload` | all emitters, `cloud-secrets`, `compliance` |
| 6 | `audit-chain /emit` | `AuditQueryExecuted` | self-audit `AuditEvent.payload` with query filters and actor | `audit-chain`, `compliance` |
| 7 | `audit-chain /emit` | `AuditExportRequested` | self-audit `AuditEvent.payload` with `export_id`, `tenant_id`, `period_id` | `audit-chain`, `compliance` |
| 8 | `audit-chain /emit` | `AuditProofRead` | self-audit `AuditEvent.payload` with `event_id`, `period_id`, `requester` | `audit-chain` |
| 9 | `audit-chain /emit` | `AuditVerificationRequested` | self-audit `AuditEvent.payload` with `event_id`, `verdict` | `audit-chain`, `ops-dashboard-control-center` |
| 10 | `observability.audit-chain` | `AuditChainMetricRecorded` | inline `AuditChainMetric` | `observability` |
| 11 | `ops-dashboard.audit-verification-failed` | `AuditVerificationIncidentOpened` | inline `AuditVerificationIncident` | `ops-dashboard-control-center` |
| 12 | `compliance.retention-apply` | `AuditRetentionApplyRequested` | inline `RetentionApplyRequest` | `compliance` |

## Synchronous vs Asynchronous Boundaries

| # | Boundary | Mode | Reasoning |
|---|---|---|---|
| 1 | `POST /emit` | synchronous | emitters require `EmitReceipt` before state changes are durable |
| 2 | `GET /events/{event_id}/proof` | synchronous | callers need proof response for verification or export |
| 3 | `POST /verify` | synchronous | verifier must return a verdict for the supplied envelope and proof |
| 4 | `POST /query` | synchronous | compliance workflows need exact query result page |
| 5 | `POST /export` | synchronous admission, asynchronous export build | caller needs `export_id`; export can build later |
| 6 | `GET /export/{export_id}` | synchronous | operators poll status directly |
| 7 | `GET /roots/{pack}/{period_id}` | synchronous | verifiers need signed root before proof validation |
| 8 | `GET /keys/{pack}/{epoch_id}` | synchronous | public key retrieval blocks signature verification |
| 9 | `audit.emitted` emission | asynchronous | emit receipt has already been returned |
| 10 | `audit.seal.minted` emission | asynchronous | period sealing is batch-oriented and replayable |
| 11 | `audit.verification.failed` emission | asynchronous urgent | incident consumers react without blocking verify response |
| 12 | `audit.retention.applied` emission | asynchronous | retention effects propagate after policy execution |
| 13 | seal-key reference read | synchronous | signing cannot proceed without current key reference |
| 14 | cell assignment read | synchronous | tenant partition routing requires current cell |
| 15 | observability metric write | asynchronous | metrics cannot block audit receipt issuance |

## Failure Mode Cascade

| # | Failure in `audit-chain` | Upstream impact | Circuit breaker | Retry policy |
|---|---|---|---|---|
| 1 | `POST /emit` unavailable | state-changing calls in all emitting services fail closed | each emitter opens `audit-chain-emit` breaker | retry 10 times with same `event_id` |
| 2 | receipt persistence failure | emitters cannot commit | `receipt-store` breaker stops accepts | no receipt returned until persisted |
| 3 | proof read failure | compliance and services cannot verify events | `proof-read` breaker returns 503 | retry by `event_id` |
| 4 | verification failure | callers get negative verdict and incident opens | `verify` breaker isolates bad period | no automatic permit from failed verify |
| 5 | export builder failure | compliance export delayed | `audit-export` breaker holds export in failed state | retry export by `export_id` |
| 6 | seal key unavailable | new periods cannot be sealed | `seal-key` breaker stops period finalization | retry versioned key reference read |
| 7 | key rotation event failure | emitters may hold old key metadata | `key-rotation-outbox` breaker spools event | replay by `epoch_id` |
| 8 | cell partition read failure | tenant event routing may pause | `audit-partition` breaker fails closed | retry cell assignment read |
| 9 | event bus unavailable | seal minted and verification failed events lag | outbox breaker spools events | replay by `event_id` |
| 10 | DLQ saturation | emitters may not receive seal finality | `audit-dlq` breaker raises operator incident | manual replay by period |
| 11 | retention policy unavailable | exports may include retained periods only | `retention` breaker fails closed | retry compliance retention call |
| 12 | public key endpoint failure | external verification cannot complete | `public-key-read` breaker returns 503 | caller retries with same epoch |

## Cross-tenant Coordination

| # | Scenario | Cedar guard pattern | Audit-mirror requirement |
|---|---|---|---|
| 1 | conglomerate parent queries child audit partition | `auditor-scope.cedar` with active parent-child grant | mirror `ConglomerateParentReadAction` to parent and child partitions |
| 2 | export spans jurisdictions | `data-residency.md` with export destination and pack | mirror `ConglomerateCrossJurisdictionResidencyEnforced` |
| 3 | office scoped proof read | `OfficeBoundaryAttemptEvaluated` with `sub_scope_path` | mirror final office allow or deny |
| 4 | information-barrier audit query | `auditor-scope.cedar` carries barrier tags | mirror `ConglomerateInformationBarrierCrossingRefused` when denied |
| 5 | personal context attempts audit export | `auditor-scope.cedar` forbids personal context | mirror `ConglomeratePersonalTenantBoundaryRefused` |

## Data Shape Ledger

| # | Shape | Source | Required handoff fields |
|---|---|---|---|
| 1 | `AuditEvent` | `openapi/audit-chain.yaml` | `tenant_id`, `source_microservice`, `event_class`, `payload`, `payload_data_class`, `emitted_at` |
| 2 | `EmitReceipt` | `openapi/audit-chain.yaml` | `event_id`, `period_id`, `pack`, `tenant_partition`, `accepted_at`, `sealed` |
| 3 | `VerifyRequest` | `openapi/audit-chain.yaml` | `event_envelope`, `proof`, `signed_root` |
| 4 | `Verdict` | `openapi/audit-chain.yaml` | `event_id`, `valid`, `reason`, `verified_at` |
| 5 | `QueryRequest` | `openapi/audit-chain.yaml` | `tenant_id`, `event_class`, `period_id`, `limit` |
| 6 | `ExportRequest` | `openapi/audit-chain.yaml` | `tenant_id`, `period_id`, `format`, `destination` |
| 7 | `AuditEmittedPayload` | `asyncapi/audit-events.yaml` | `event_id`, `period_id`, `tenant_partition`, `source_microservice` |
| 8 | `SealMintedPayload` | `asyncapi/audit-events.yaml` | `period_id`, `pack`, `signed_root`, `sealed_at` |
| 9 | `VerificationFailedPayload` | `asyncapi/audit-events.yaml` | `event_id`, `period_id`, `reason`, `detected_at` |
| 10 | `KeyRotatedPayload` | `asyncapi/audit-events.yaml` | `pack`, `epoch_id`, `public_key_id`, `rotated_at` |

## Cedar Guard Ledger

| # | Policy file | Principal | Action | Resource |
|---|---|---|---|---|
| 1 | `tenant-scope.cedar` | `Service::api-gateway` | `AuditChain::emit_gateway_event` | `AuditStream::{tenant_id}` |
| 2 | `tenant-scope.cedar` | `Service::application` | `AuditChain::emit_application_event` | `AuditStream::{tenant_id}` |
| 3 | `tenant-scope.cedar` | `Service::cell` | `AuditChain::emit_cell_event` | `AuditStream::{tenant_id}` |
| 4 | `tenant-scope.cedar` | `Service::cloud-iac` | `AuditChain::emit_iac_event` | `AuditStream::{tenant_id}` |
| 5 | `tenant-scope.cedar` | `Service::cloud-secrets` | `AuditChain::emit_secret_event` | `AuditStream::{tenant_id}` |
| 6 | `developer-scope.cedar` | `Service::developer-sdk` | `AuditChain::emit_developer_event` | `AuditStream::{developer_id}` |
| 7 | `auditor-scope.cedar` | `Service::compliance` | `AuditChain::query_events_for_compliance` | `AuditPartition::{tenant_id}` |
| 8 | `auditor-scope.cedar` | `Service::compliance` | `AuditChain::request_export` | `AuditPartition::{tenant_id}` |
| 9 | `public-read.cedar` | `Anonymous` | `AuditChain::verify_event_public` | `AuditProof::{event_id}` |
| 10 | `seal-integrity.md` | `Service::audit-chain` | `AuditChain::mint_seal` | `AuditPeriod::{period_id}` |

## Audit Event Class Ledger

| # | Audit class | Emitting handoff | ADR-0263 envelope fields that must be present |
|---|---|---|---|
| 1 | `AuditEventAccepted` | `POST /emit` | `tenant_id`, `source_microservice`, `event_class`, `audit_id` |
| 2 | `AuditQueryExecuted` | `POST /query` | `tenant_id`, `query_hash`, `actor`, `audit_id` |
| 3 | `AuditExportRequested` | `POST /export` | `tenant_id`, `export_id`, `period_id`, `destination_hash` |
| 4 | `AuditProofRead` | `GET /events/{event_id}/proof` | `event_id`, `period_id`, `requester`, `audit_id` |
| 5 | `AuditVerificationRequested` | `POST /verify` | `event_id`, `verdict`, `signed_root`, `audit_id` |
| 6 | `AuditSignedRootRead` | `GET /roots/{pack}/{period_id}` | `pack`, `period_id`, `requester`, `audit_id` |
| 7 | `AuditPublicKeyRead` | `GET /keys/{pack}/{epoch_id}` | `pack`, `epoch_id`, `public_key_id`, `audit_id` |
| 8 | `AuditRetentionApplied` | retention handoff | `tenant_id`, `period_id`, `policy_id`, `audit_id` |
| 9 | `AuditKeyRotated` | key rotation | `pack`, `epoch_id`, `public_key_id`, `audit_id` |

## Handoff Control Checklist

1. `POST /emit` must require `tenant_id`.
2. `POST /emit` must require `source_microservice`.
3. `POST /emit` must require `event_class`.
4. `POST /emit` must require `payload_data_class`.
5. `POST /emit` must return `EmitReceipt`.
6. `EmitReceipt` must include `event_id`.
7. `EmitReceipt` must include `period_id`.
8. `EmitReceipt` must include `tenant_partition`.
9. `EmitReceipt` must include `accepted_at`.
10. `GET /proof` must verify caller audit scope.
11. `POST /verify` must validate signed root.
12. `POST /verify` must validate proof membership.
13. `POST /query` must hash query filters in self-audit.
14. `POST /export` must create stable `export_id`.
15. `GET /export/{export_id}` must not reveal export payload.
16. `GET /roots` must return signed root by pack and period.
17. `GET /keys` must return public key only.
18. Seal minting must use versioned seal-key reference.
19. Export creation must use versioned export-key reference.
20. Cell assignment must determine audit partition.
21. Gateway emit permits must use `tenant-scope.cedar`.
22. Application emit permits must use `tenant-scope.cedar`.
23. Cell emit permits must use `tenant-scope.cedar`.
24. Cloud-IAC emit permits must use `tenant-scope.cedar`.
25. Cloud-secrets emit permits must use `tenant-scope.cedar`.
26. Developer emit permits must use `developer-scope.cedar`.
27. Payments emit permits must use `tenant-scope.cedar`.
28. Query permits must use `auditor-scope.cedar`.
29. Export permits must use `auditor-scope.cedar`.
30. Public verification must use `public-read.cedar`.
31. Audit event payloads must remain PII-scrubbed.
32. Audit event payloads must preserve `trace_id`.
33. Audit event payloads must preserve `span_id`.
34. Audit event payloads must preserve `audit_id`.
35. Audit event payloads must preserve `schema_version`.
36. Cedar decision events must include `cedar_policy_version`.
37. Cedar decision events must include `evaluation_id`.
38. Cedar decision events must include `action`.
39. Cedar decision events must include `resource_ref`.
40. Cedar decision events must include `decision`.
41. `audit.emitted` must publish after receipt persistence.
42. `audit.seal.minted` must publish after signed root persistence.
43. `audit.verification.failed` must publish after negative verdict.
44. `audit.retention.applied` must publish after compliance retention ack.
45. `audit.key.rotated` must publish after public key availability.
46. Emit retries must reuse the same `event_id`.
47. Export retries must reuse the same `export_id`.
48. Seal retries must reuse the same `period_id`.
49. Verification incidents must include failure reason.
50. Observability metrics must aggregate by pack and period.
51. DLQ replay must preserve period order.
52. DLQ replay must not mint seals before events are persisted.
53. Secret revocation must disable affected key epoch.
54. Secret rotation must update key reference by version.
55. Public key reads must not require tenant context.
56. Proof reads must require tenant or auditor context.
57. Cross-tenant audit queries must mirror both partitions.
58. Cross-jurisdiction exports must mirror residency enforcement.
59. Office scoped reads must include `sub_scope_path`.
60. Personal-context export attempts must be refused.
61. Information-barrier denials must be self-audited.
62. Retention application must be self-audited.
63. Query results must page deterministically.
64. Export payloads must be encrypted when written.
65. Signed roots must be immutable per period.
66. Public keys must be immutable per epoch.
67. Verification must fail closed on malformed proof.
68. Verification must fail closed on mismatched root.
69. `audit-chain` must update this matrix when `audit-chain.yaml` changes.
70. `audit-chain` must update this matrix when `audit-events.yaml` changes.

## Checkpoint

- Authored for `audit-chain` on 2026-05-20.
- Source contracts checked: `audit-chain.yaml`, `audit-events.yaml`, and audit-chain proto.
- Source policies checked: `seal-integrity.md`, `tenant-scope.cedar`, `public-read.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`.
- No in-flight microservice directories were edited.
- Oya VCS scope: `microservices`.
