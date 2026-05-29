---
doc_class: APIReference
microservice: audit-chain
version: 1.0.0
status: Accepted
date: 2026-05-20
owner: axis-audit-chain + council-compliance + ops-security
openapi_version: 3.2.0
asyncapi_version: 3.1.0
proto3: true
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# audit-chain API Reference

Canonical REST, gRPC, and AsyncAPI reference for the `audit-chain`
microservice. The service owns immutable audit emission, Merkle proof lookup,
verification, query, export, signed roots, public verification keys, retention
events, DSR redaction receipts, and evidence sealing for regulated workflows.

Contract status legend:

- `contract-bound`: implemented in the current OpenAPI, AsyncAPI, or proto3 file.
- `reference-planned`: canonical API surface derived from the PRD and SDK plan, pending contract promotion.

## Quick Start

Named example: `EmitVerifyAndExportEvidence`.

1. Emit an audit event with `POST /emit`.
2. Retrieve proof material with `GET /events/{event_id}/proof`.
3. Verify and export the evidence set with `POST /verify` and `POST /export`.

Minimum headers:

- `Authorization: Bearer <oidc-token>`
- `X-Tenant-Id: <uuid-v7>`
- `X-Context-Kind: Personal | Professional`
- `Idempotency-Key: <ulid>` on emit and export requests
- `X-Request-Id: <ulid>` for chain traceability
- `Content-Type: application/json`

Example:

```http
POST /emit HTTP/2
Host: audit-chain.oyatie.com
Authorization: Bearer eyJ...
X-Tenant-Id: 018f7a54-3ef5-7c42-a111-a2c4ad7f88f0
Idempotency-Key: 01HYAUDITEMIT0000000000
Content-Type: application/json
```

## Authentication & Authorization

Authentication patterns:

- OIDC bearer for compliance operators and tenant evidence requests.
- SPIFFE SVID mTLS for internal emitters.
- HSM-backed signing key assertions for root and epoch operations.
- Export bundle signatures for regulator and tenant download.
- Webhook HMAC for inbound external WORM, HSM, and regulator callbacks.

Principal types:

- `AuditEmitter`: internal service principal allowed to append events.
- `ComplianceOfficer`: tenant compliance user allowed to query and export.
- `RegulatorViewer`: external auditor with explicit evidence package grant.
- `RetentionOperator`: principal allowed to apply retention policy transitions.
- `DsrProcessor`: principal allowed to record redaction proofs.
- `AuditVerifier`: user or service principal allowed to verify proof material.
- `KeyCustodian`: operator allowed to inspect public key epochs.
- `AuditChainWorker`: internal sealer, export, and retention worker.

Named Cedar policy patterns:

- `audit_chain::tenant_scope_match`: tenant in token, request, and event envelope must match.
- `audit_chain::append_only_emit`: emitters can append but cannot mutate sealed records.
- `audit_chain::proof_read_scope`: proof reads require event ownership or regulator grant.
- `audit_chain::export_four_eyes`: sensitive evidence export requires dual approval.
- `audit_chain::retention_policy_apply`: retention action requires policy and legal hold check.
- `audit_chain::dsr_redaction_receipt`: redaction records must preserve proof verifiability.
- `audit_chain::key_epoch_read`: public verification keys are readable, private keys are never exposed.
- `audit_chain::query_minimization`: query results are minimized by purpose and data class.

Authorization failure shape:

```json
{
  "error": {
    "code": "AUDIT_CHAIN_AUTHZ_DENIED",
    "message": "Cedar policy denied audit-chain action",
    "request_id": "01HYREQ...",
    "details": [{"policy": "audit_chain::export_four_eyes"}]
  }
}
```

## REST Endpoints

### Event Emission

#### POST /emit

- Status: `contract-bound`.
- Operation: `emit`.
- Request schema: `EmitAuditEventRequest`.
- Required fields: `tenant_id`, `event_type`, `subject`, `actor`, `payload_hash`.
- Response schema: `EmitAuditEventResponse`.
- Status codes: `202`, `400`, `401`, `403`, `409`, `422`, `429`, `500`, `503`.
- Error shape: `AUDIT_CHAIN_EVENT_DUPLICATE`.

#### GET /events/{event_id}

- Status: `reference-planned`.
- Operation: `getEvent`.
- Path schema: `event_id` as UUID-v7.
- Response schema: `AuditEventRecord`.
- Status codes: `200`, `401`, `403`, `404`, `410`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_EVENT_NOT_FOUND`.

#### GET /events/{event_id}/proof

- Status: `contract-bound`.
- Operation: `getProof`.
- Path schema: `event_id` as UUID-v7.
- Query schema: `include_root`, `include_key`, `format`.
- Response schema: `AuditProof`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `410`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_PROOF_NOT_READY`.

#### POST /events/{event_id}:annotate

- Status: `reference-planned`.
- Operation: `annotateEvent`.
- Request schema: `AnnotateEventRequest`.
- Required fields: `annotation_type`, `reason`, `annotated_by`.
- Response schema: `AuditAnnotationReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_ANNOTATION_DENIED`.

### Verification

#### POST /verify

- Status: `contract-bound`.
- Operation: `verify`.
- Request schema: `VerifyAuditProofRequest`.
- Required fields: `event_id`, `proof`, `root`, `public_key_epoch`.
- Response schema: `VerifyAuditProofResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_PROOF_INVALID`.

#### POST /integrity:replay

- Status: `reference-planned`.
- Operation: `replayIntegrityRange`.
- Request schema: `ReplayIntegrityRangeRequest`.
- Required fields: `tenant_id`, `from_event_id`, `to_event_id`, `pack`.
- Response schema: `IntegrityReplayReport`.
- Status codes: `202`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_REPLAY_WINDOW_INVALID`.

#### POST /chain:verify-range

- Status: `reference-planned`.
- Operation: `verifyChainRange`.
- Request schema: `VerifyChainRangeRequest`.
- Required fields: `tenant_id`, `from_root`, `to_root`, `public_key_epoch`.
- Response schema: `VerifyChainRangeResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_RANGE_VERIFICATION_FAILED`.

### Query

#### POST /query

- Status: `contract-bound`.
- Operation: `query`.
- Request schema: `QueryAuditEventsRequest`.
- Required fields: `tenant_id`, `purpose`, `filters`.
- Response schema: `QueryAuditEventsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_QUERY_DENIED`.

#### GET /subjects/{subject_id}/events

- Status: `reference-planned`.
- Operation: `listSubjectEvents`.
- Query schema: `tenant_id`, `event_type`, `from_time`, `to_time`, `cursor`, `limit`.
- Response schema: `ListSubjectEventsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_SUBJECT_QUERY_DENIED`.

#### GET /actors/{actor_id}/events

- Status: `reference-planned`.
- Operation: `listActorEvents`.
- Query schema: `tenant_id`, `event_type`, `from_time`, `to_time`, `cursor`, `limit`.
- Response schema: `ListActorEventsResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_ACTOR_QUERY_DENIED`.

### Export

#### POST /export

- Status: `contract-bound`.
- Operation: `export`.
- Request schema: `RequestAuditExportRequest`.
- Required fields: `tenant_id`, `purpose`, `filters`, `format`.
- Response schema: `AuditExportTicket`.
- Status codes: `202`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_EXPORT_REQUIRES_APPROVAL`.

#### GET /export/{export_id}

- Status: `contract-bound`.
- Operation: `getExportStatus`.
- Path schema: `export_id` as UUID-v7.
- Response schema: `AuditExportStatus`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_EXPORT_NOT_FOUND`.

#### GET /exports/{export_id}/download

- Status: `reference-planned`.
- Operation: `downloadExport`.
- Path schema: `export_id` as UUID-v7.
- Response schema: signed archive stream with `ExportManifest`.
- Status codes: `200`, `401`, `403`, `404`, `410`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_EXPORT_EXPIRED`.

#### POST /exports/{export_id}:revoke

- Status: `reference-planned`.
- Operation: `revokeExport`.
- Request schema: `RevokeExportRequest`.
- Required fields: `reason`, `revoked_by`.
- Response schema: `ExportRevocationReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_EXPORT_ALREADY_FINAL`.

### Roots and Keys

#### GET /roots/{pack}/{period_id}

- Status: `contract-bound`.
- Operation: `getSignedRoot`.
- Path schema: `pack` and `period_id`.
- Response schema: `SignedMerkleRoot`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_ROOT_NOT_FOUND`.

#### GET /roots/{pack}/{period_id}/manifest

- Status: `reference-planned`.
- Operation: `getRootManifest`.
- Path schema: `pack` and `period_id`.
- Response schema: `RootManifest`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_ROOT_MANIFEST_NOT_FOUND`.

#### GET /keys/{pack}/{epoch_id}

- Status: `contract-bound`.
- Operation: `getPublicKey`.
- Path schema: `pack` and `epoch_id`.
- Response schema: `PublicVerificationKey`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_KEY_EPOCH_NOT_FOUND`.

#### GET /keys/{pack}/{epoch_id}/chain

- Status: `reference-planned`.
- Operation: `getKeyEpochChain`.
- Path schema: `pack` and `epoch_id`.
- Response schema: `KeyEpochChain`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_KEY_CHAIN_NOT_FOUND`.

### Retention and DSR

#### GET /retention/policies

- Status: `reference-planned`.
- Operation: `listRetentionPolicies`.
- Query schema: `tenant_id`, `pack`, `data_class`, `cursor`, `limit`.
- Response schema: `ListRetentionPoliciesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_RETENTION_POLICY_QUERY_DENIED`.

#### POST /retention/runs

- Status: `reference-planned`.
- Operation: `createRetentionRun`.
- Request schema: `CreateRetentionRunRequest`.
- Required fields: `tenant_id`, `policy_id`, `dry_run`.
- Response schema: `RetentionRun`.
- Status codes: `202`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_RETENTION_HOLD_BLOCKED`.

#### GET /retention/runs/{run_id}

- Status: `reference-planned`.
- Operation: `getRetentionRun`.
- Path schema: `run_id` as UUID-v7.
- Response schema: `RetentionRun`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_RETENTION_RUN_NOT_FOUND`.

#### POST /dsr/redactions

- Status: `reference-planned`.
- Operation: `recordDsrRedaction`.
- Request schema: `RecordDsrRedactionRequest`.
- Required fields: `tenant_id`, `subject_id`, `redaction_scope`, `proof_hash`.
- Response schema: `DsrRedactionReceipt`.
- Status codes: `202`, `400`, `401`, `403`, `409`, `422`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_DSR_REDACTION_INVALID`.

#### GET /dsr/redactions/{redaction_id}

- Status: `reference-planned`.
- Operation: `getDsrRedaction`.
- Path schema: `redaction_id` as UUID-v7.
- Response schema: `DsrRedactionReceipt`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `500`.
- Error shape: `AUDIT_CHAIN_DSR_REDACTION_NOT_FOUND`.

### Health

#### GET /health

- Status: `contract-bound`.
- Operation: `health`.
- Response schema: `HealthStatus`.
- Status codes: `200`, `500`.
- Error shape: standard health probe failure.

#### GET /ready

- Status: `contract-bound`.
- Operation: `ready`.
- Response schema: `ReadinessStatus`.
- Status codes: `200`, `503`.
- Error shape: `AUDIT_CHAIN_SEALER_UNREADY`.

## gRPC Methods

### service AuditChainEmission

```proto
rpc Emit(EmitAuditEventRequest) returns (EmitAuditEventResponse);
```

- Status: `contract-bound`.
- Semantics: appends an event envelope for asynchronous sealing.
- Auth: `audit_chain::append_only_emit`.
- Errors: `ALREADY_EXISTS`, `INVALID_ARGUMENT`, `RESOURCE_EXHAUSTED`.

```proto
rpc GetEvent(GetEventRequest) returns (AuditEventRecord);
```

- Status: `reference-planned`.
- Semantics: returns an immutable event record.
- Auth: `audit_chain::proof_read_scope`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

### service AuditChainProof

```proto
rpc GetProof(GetProofRequest) returns (AuditProof);
```

- Status: `contract-bound`.
- Semantics: returns Merkle proof and optional root/key material.
- Auth: `audit_chain::proof_read_scope`.
- Errors: `NOT_FOUND`, `FAILED_PRECONDITION`.

```proto
rpc Verify(VerifyAuditProofRequest) returns (VerifyAuditProofResponse);
```

- Status: `contract-bound`.
- Semantics: verifies proof against a signed root and public key epoch.
- Auth: `audit_chain::key_epoch_read`.
- Errors: `INVALID_ARGUMENT`, `FAILED_PRECONDITION`.

```proto
rpc VerifyChainRange(VerifyChainRangeRequest) returns (VerifyChainRangeResponse);
```

- Status: `reference-planned`.
- Semantics: verifies continuity between two signed roots.
- Auth: `audit_chain::key_epoch_read`.
- Errors: `INVALID_ARGUMENT`, `FAILED_PRECONDITION`.

### service AuditChainQuery

```proto
rpc Query(QueryAuditEventsRequest) returns (QueryAuditEventsResponse);
```

- Status: `contract-bound`.
- Semantics: runs purpose-bound minimized audit queries.
- Auth: `audit_chain::query_minimization`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc ListSubjectEvents(ListSubjectEventsRequest) returns (ListSubjectEventsResponse);
```

- Status: `reference-planned`.
- Semantics: lists minimized subject events.
- Auth: `audit_chain::query_minimization`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

### service AuditChainExport

```proto
rpc RequestExport(RequestAuditExportRequest) returns (AuditExportTicket);
```

- Status: `contract-bound`.
- Semantics: creates an asynchronous export ticket.
- Auth: `audit_chain::export_four_eyes`.
- Errors: `FAILED_PRECONDITION`, `RESOURCE_EXHAUSTED`.

```proto
rpc GetExportStatus(GetExportStatusRequest) returns (AuditExportStatus);
```

- Status: `contract-bound`.
- Semantics: returns export job state and manifest summary.
- Auth: `audit_chain::export_four_eyes`.
- Errors: `NOT_FOUND`, `PERMISSION_DENIED`.

### service AuditChainRoots

```proto
rpc GetSignedRoot(GetSignedRootRequest) returns (SignedMerkleRoot);
```

- Status: `contract-bound`.
- Semantics: returns signed root for pack and period.
- Auth: `audit_chain::key_epoch_read`.
- Errors: `NOT_FOUND`, `INVALID_ARGUMENT`.

```proto
rpc GetPublicKey(GetPublicKeyRequest) returns (PublicVerificationKey);
```

- Status: `contract-bound`.
- Semantics: returns public verification key for pack and epoch.
- Auth: `audit_chain::key_epoch_read`.
- Errors: `NOT_FOUND`, `INVALID_ARGUMENT`.

### service AuditChainRetention

```proto
rpc CreateRetentionRun(CreateRetentionRunRequest) returns (RetentionRun);
```

- Status: `reference-planned`.
- Semantics: applies retention policy while preserving verification receipts.
- Auth: `audit_chain::retention_policy_apply`.
- Errors: `FAILED_PRECONDITION`, `PERMISSION_DENIED`.

```proto
rpc RecordDsrRedaction(RecordDsrRedactionRequest) returns (DsrRedactionReceipt);
```

- Status: `reference-planned`.
- Semantics: records DSR redaction evidence without breaking root verification.
- Auth: `audit_chain::dsr_redaction_receipt`.
- Errors: `INVALID_ARGUMENT`, `PERMISSION_DENIED`.

```proto
rpc SubscribeVerificationFailed(SubscribeVerificationFailedRequest) returns (stream VerificationFailedEvent);
```

- Status: `contract-bound`.
- Semantics: streams verification-failure events to operators.
- Auth: `audit_chain::key_epoch_read`.
- Errors: `UNAVAILABLE`, `RESOURCE_EXHAUSTED`.

## AsyncAPI Channels

### audit-chain.audit-emitted

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `AuditEmitted`.
- Delivery semantics: at-least-once, deduplicate by `event_id`.
- Consumers: governance, analytics, compliance dashboards.

### audit-chain.seal-minted

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `SealMinted`.
- Delivery semantics: ordered per `pack` and `period_id`.
- Consumers: governance, regulator export, key transparency.

### audit-chain.verification-failed

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `VerificationFailed`.
- Delivery semantics: at-least-once with incident severity.
- Consumers: governance, security operations, pager.

### audit-chain.retention-applied

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `RetentionApplied`.
- Delivery semantics: at-least-once, partitioned by `tenant_id`.
- Consumers: governance, privacy operations, evidence export.

### audit-chain.key-rotated

- Direction: publish.
- Status: `contract-bound`.
- Payload schema: `KeyRotated`.
- Delivery semantics: compacted by `pack` and `epoch_id`.
- Consumers: verifier clients, governance, security.

### audit-chain.export-completed

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `ExportCompleted`.
- Delivery semantics: at-least-once with signed manifest reference.
- Consumers: compliance portal, regulator bridge, governance.

### audit-chain.dsr-redaction-recorded

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `DsrRedactionRecorded`.
- Delivery semantics: at-least-once.
- Consumers: privacy operations, governance, subject access service.

### audit-chain.integrity-replay-completed

- Direction: publish.
- Status: `reference-planned`.
- Payload schema: `IntegrityReplayCompleted`.
- Delivery semantics: at-least-once.
- Consumers: governance, security operations.

### services.audit-event-requested

- Direction: subscribe.
- Status: `contract-bound`.
- Payload schema: `AuditEventRequested`.
- Delivery semantics: at-least-once from service emitters.
- Handler: validate, hash, append, and seal.

### governance.retention-policy-updated

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `RetentionPolicyUpdated`.
- Delivery semantics: compacted by `policy_id`.
- Handler: update retention policy cache.

### privacy.dsr-erasure-approved

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `DsrErasureApproved`.
- Delivery semantics: at-least-once.
- Handler: create DSR redaction receipt.

### security.key-rotation-completed

- Direction: subscribe.
- Status: `reference-planned`.
- Payload schema: `KeyRotationCompleted`.
- Delivery semantics: ordered per `pack`.
- Handler: publish public key epoch and key chain record.

## Webhooks Inbound

### webhook.service.audit-event

- Source: all internal microservices.
- Event: `audit.event`.
- Payload schema: `AuditEventWebhook`.
- Semantics: validates event envelope and appends it to the chain.

### webhook.hsm.key-rotated

- Source: HSM controller.
- Event: `hsm.key_rotated`.
- Payload schema: `HsmKeyRotatedWebhook`.
- Semantics: publishes public epoch metadata and starts root rollover.

### webhook.worm.write-completed

- Source: object storage WORM controller.
- Event: `worm.write_completed`.
- Payload schema: `WormWriteCompletedWebhook`.
- Semantics: confirms export bundle immutability.

### webhook.governance.retention-policy-updated

- Source: governance.
- Event: `governance.retention_policy.updated`.
- Payload schema: `RetentionPolicyUpdatedWebhook`.
- Semantics: refreshes retention policy and hold cache.

### webhook.privacy.dsr-erasure-approved

- Source: privacy service.
- Event: `privacy.dsr_erasure.approved`.
- Payload schema: `DsrErasureApprovedWebhook`.
- Semantics: records redaction evidence and verifies proof continuity.

### webhook.regulator.export-accessed

- Source: regulator portal.
- Event: `regulator.export_accessed`.
- Payload schema: `RegulatorExportAccessedWebhook`.
- Semantics: records evidence package access.

### webhook.security.incident-opened

- Source: security operations.
- Event: `security.incident_opened`.
- Payload schema: `SecurityIncidentWebhook`.
- Semantics: pins relevant chain periods for hold.

### webhook.tenant.legal-hold-changed

- Source: tenant compliance.
- Event: `tenant.legal_hold.changed`.
- Payload schema: `LegalHoldChangedWebhook`.
- Semantics: blocks retention runs for held subjects or periods.

## SDK Quick Reference

### Rust

```rust
let emitted = audit_chain::emit(client, event).await?;
let proof = audit_chain::get_proof(client, emitted.event_id).await?;
let verified = audit_chain::verify(client, proof).await?;
let export = audit_chain::request_export(client, export_request).await?;
let root = audit_chain::get_signed_root(client, pack, period_id).await?;
```

Named functions:

- `emit`
- `get_event`
- `get_proof`
- `verify`
- `query`
- `request_export`
- `get_export_status`
- `download_export`
- `get_signed_root`
- `get_public_key`
- `create_retention_run`
- `record_dsr_redaction`

### TypeScript

```ts
const audit = new AuditChainClient({ tenantId, token });
const emitted = await audit.emit(event);
const proof = await audit.getProof(emitted.eventId);
await audit.verify({ eventId: emitted.eventId, proof });
const ticket = await audit.requestExport({ purpose, filters, format: "zip" });
await audit.getSignedRoot({ pack, periodId });
```

Named functions:

- `emit`
- `getEvent`
- `getProof`
- `verify`
- `query`
- `requestExport`
- `getExportStatus`
- `downloadExport`
- `getSignedRoot`
- `getPublicKey`
- `createRetentionRun`
- `recordDsrRedaction`

### Python

```python
audit = AuditChainClient(tenant_id=tenant_id, token=token)
emitted = audit.emit(event)
proof = audit.get_proof(emitted.event_id)
audit.verify(event_id=emitted.event_id, proof=proof)
ticket = audit.request_export(purpose=purpose, filters=filters, format="zip")
root = audit.get_signed_root(pack=pack, period_id=period_id)
```

Named functions:

- `emit`
- `get_event`
- `get_proof`
- `verify`
- `query`
- `request_export`
- `get_export_status`
- `download_export`
- `get_signed_root`
- `get_public_key`
- `create_retention_run`
- `record_dsr_redaction`

## Error Catalogue

### AUDIT_CHAIN_AUTHZ_DENIED

- Meaning: Cedar denied audit-chain operation.
- Retry policy: do not retry without changing scope or approval.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### AUDIT_CHAIN_EVENT_DUPLICATE

- Meaning: idempotency key or event id already emitted.
- Retry policy: safe to fetch the existing event or proof.
- HTTP mapping: `409`.
- gRPC mapping: `ALREADY_EXISTS`.

### AUDIT_CHAIN_PROOF_NOT_READY

- Meaning: event is accepted but not yet sealed into a root.
- Retry policy: retry with exponential backoff until proof readiness SLA.
- HTTP mapping: `409`.
- gRPC mapping: `FAILED_PRECONDITION`.

### AUDIT_CHAIN_PROOF_INVALID

- Meaning: proof failed root, hash, or key verification.
- Retry policy: do not retry; escalate as integrity incident.
- HTTP mapping: `422`.
- gRPC mapping: `FAILED_PRECONDITION`.

### AUDIT_CHAIN_QUERY_DENIED

- Meaning: purpose, filter, or principal violates minimization policy.
- Retry policy: do not retry without changing purpose or approval.
- HTTP mapping: `403`.
- gRPC mapping: `PERMISSION_DENIED`.

### AUDIT_CHAIN_EXPORT_REQUIRES_APPROVAL

- Meaning: export requires a missing approval or legal basis.
- Retry policy: retry after approval state changes.
- HTTP mapping: `409`.
- gRPC mapping: `FAILED_PRECONDITION`.

### AUDIT_CHAIN_ROOT_NOT_FOUND

- Meaning: requested pack and period root does not exist.
- Retry policy: retry only if period is still sealing.
- HTTP mapping: `404`.
- gRPC mapping: `NOT_FOUND`.

### AUDIT_CHAIN_RETENTION_HOLD_BLOCKED

- Meaning: legal hold prevents retention run.
- Retry policy: do not retry until hold is lifted.
- HTTP mapping: `409`.
- gRPC mapping: `FAILED_PRECONDITION`.

### AUDIT_CHAIN_DSR_REDACTION_INVALID

- Meaning: DSR redaction request would break verification continuity.
- Retry policy: correct redaction proof and retry.
- HTTP mapping: `422`.
- gRPC mapping: `INVALID_ARGUMENT`.

### AUDIT_CHAIN_SEALER_UNREADY

- Meaning: sealer, HSM, or WORM dependency is unavailable.
- Retry policy: backoff; emitters may queue durable outbox entries.
- HTTP mapping: `503`.
- gRPC mapping: `UNAVAILABLE`.

## Pagination

Cursor pattern name: `audit_chain_period_cursor`.

Cursor fields:

- `tenant_id`
- `pack`
- `period_id`
- `resource_kind`
- `sort_key`
- `last_seen_id`
- `issued_at`
- `signature`

Rules:

- Cursor values are opaque and signed.
- Audit event queries sort by event time plus UUID-v7 tiebreaker.
- Export and retention views sort by creation time.
- Cursor TTL is 24 hours for evidence queries and 15 minutes for operational views.
- Redacted events remain represented by receipts where policy allows.
- Invalid cursors return `AUDIT_CHAIN_CURSOR_INVALID`.

Max page-size limits:

- Query events: `500`.
- Subject events: `500`.
- Actor events: `500`.
- Exports: `100`.
- Retention policies: `200`.
- Retention runs: `200`.
- DSR redactions: `200`.
- Default page size: `100`.

## Rate Limits per Tier

Per ADR-0316, audit-chain uses capability-tier throttles rather than
product-fragmented limits.

| Tier | REST requests per second | gRPC requests per second | Async publishes per second | Burst |
| --- | ---: | ---: | ---: | ---: |

Special limits:


## OpenAPI 3.2.0 Schema

Actual contracts file:

- [audit-chain.yaml](../../microservices/audit-chain/contracts/openapi/audit-chain.yaml)

Design references:

- [audit-chain PRD](../../microservices/audit-chain/PRD.md)
- [audit-chain SDK plan](../../microservices/audit-chain/sdk-plan.md)
- [API design standard](../standards/api-design.md)

## AsyncAPI 3.1.0 Schema

Actual contracts file:

- [audit-events.yaml](../../microservices/audit-chain/contracts/asyncapi/audit-events.yaml)

Delivery notes:

- Emit events are accepted asynchronously and sealed later.
- Root events are ordered by `pack` and `period_id`.
- Verification failures are never compacted.
- Consumers must deduplicate by `event_id`.

## proto3 Schema

Actual contracts file:

- [audit-chain.proto](../../microservices/audit-chain/contracts/proto/audit-chain.proto)

Proto package expectations:

- Use proto3 syntax.
- Keep event payload hashes stable across language SDKs.
- Map proof-not-ready to `FAILED_PRECONDITION`.
- Map integrity failure to `FAILED_PRECONDITION` and emit verification event.

## Cross-References

- [audit-chain PRD](../../microservices/audit-chain/PRD.md)
- [audit-chain SDK plan](../../microservices/audit-chain/sdk-plan.md)
- [ADR-0316 capability tier over product fragmentation](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md)
- [API design standard](../standards/api-design.md)
- [Throttling tiers](../standards/throttling-tiers.md)
- [Governance API reference](governance-api-reference.md)
- [Intelligence API reference](intelligence-api-reference.md)
- [Payments API reference](payments-api-reference.md)
- [Ontology API reference](ontology-api-reference.md)
