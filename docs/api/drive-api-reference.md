---
doc_class: APIReference
microservice: drive
version: 1.0.0
status: Accepted
date: 2026-05-20
owner: axis-drive
openapi_version: 3.2.0
asyncapi_version: 3.1.0
proto3: true
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# drive API Reference

Canonical REST, gRPC, and AsyncAPI reference for the `drive` microservice.
The native API owns files, folders, multipart upload, range download, sync,
share links, permissions, search, previews, scan verdicts, and WORM controls.

## Quick Start

Named example: `UploadShareAndSubscribe`.

1. Initiate a resumable upload with `POST /upload/sessions`.
2. Upload chunks with `PATCH /upload/sessions/{session_id}` and read metadata with `GET /files/{file_id}`.
3. Mint a share link with `POST /share-links` and subscribe to `drive.share.v1`.

Minimum headers:

- `Authorization: Bearer <oidc-token>`
- `X-Tenant-Id: tenant:<hashed-id>` or contract-specific tenant header
- `X-Context-Kind: Personal | Professional`
- `Idempotency-Key: <ulid>` on mutating routes
- `Content-Type: application/json`, `multipart/form-data`, or `application/offset+octet-stream`

Native protocol note:

- OpenAPI covers the native API.
- S3-compatible facade is under `/s3/` and follows AWS SigV4.
- WebDAV facade is under `/dav/` and follows RFC 4918.

## Authentication & Authorization

Authentication patterns:

- OIDC bearer for tenant users and tenant service accounts.
- Per-tenant API key for service integrations.
- AWS SigV4-compatible credentials for the S3 facade.
- Basic-over-TLS tenant credential for WebDAV compatibility.
- SPIFFE-bound mTLS for internal service-to-service calls.

Principal types:

- `DriveUser`: tenant member reading or writing owned files.
- `DriveFolderManager`: principal managing inherited folder permissions.
- `DriveShareIssuer`: principal allowed to mint external share links.
- `DriveSyncClient`: desktop or mobile sync agent bound to a device.
- `DriveComplianceOfficer`: WORM and legal-hold principal.
- `DrivePreviewWorker`: sandboxed render worker principal.
- `DriveScanWorker`: malware and DLP verdict principal.
- `DriveAuditor`: scoped read-only evidence principal.

Named Cedar policy patterns:

- `drive::tenant_scope_match`: tenant header must match token tenant.
- `drive::dual_context_isolation`: Personal and Professional files do not co-mingle.
- `drive::file_read_acl`: file read requires inherited or direct permission.
- `drive::folder_manage_acl`: folder permission mutation requires manager role.
- `drive::share_link_policy`: share link issuance checks DLP, expiry, and audience.
- `drive::worm_immutability_guard`: WORM file cannot be modified or purged.
- `drive::legal_hold_guard`: held file cannot be purged while hold is active.
- `drive::scan_release_gate`: durable promotion waits for malware and DLP verdict.

Authorization failure shape:

```json
{
  "error": {
    "code": "DRIVE_AUTHZ_DENIED",
    "message": "Cedar policy denied drive action",
    "request_id": "01HYREQ...",
    "details": [{"policy": "drive::file_read_acl"}]
  }
}
```

## REST Endpoints

Base URL: `https://drive.{pack}.oyatie.com/v1`.

### Files

#### 1. `POST /files`
- Resource: File collection.
- Request schema: `FileCreateMetadata` plus binary `content` for single-shot upload.
- Response schema: `File`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `413`, `422`, `429`, `503`.
- Error shape: `FILE_TOO_LARGE_FOR_SINGLE_SHOT`, `FILE_SCHEMA_INVALID`, `IDEMPOTENCY_REPLAY_CONFLICT`.

#### 2. `GET /files`
- Resource: File collection.
- Request schema: query `folder_id`, `cursor`, `page_size`, `context_kind`.
- Response schema: `FilePage`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `422`, `429`, `503`.
- Error shape: `FOLDER_NOT_FOUND`, `DRIVE_CURSOR_INVALID`.

#### 3. `GET /files/{file_id}`
- Resource: File entity.
- Request schema: path `file_id`.
- Response schema: `File`.
- Status codes: `200`, `401`, `403`, `404`, `410`, `423`, `429`, `503`.
- Error shape: `FILE_NOT_FOUND`, `FILE_UNDER_LEGAL_HOLD`.

#### 4. `PATCH /files/{file_id}`
- Resource: File metadata.
- Request schema: `FilePatchRequest` with `name`, `parent_folder_id`, `labels`, `expected_version`.
- Response schema: `File`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `FILE_VERSION_CONFLICT`, `FILE_IMMUTABLE`.

#### 5. `DELETE /files/{file_id}`
- Resource: File entity.
- Request schema: path `file_id`, optional `delete_reason`.
- Response schema: empty success envelope.
- Status codes: `204`, `401`, `403`, `404`, `409`, `423`, `429`.
- Error shape: `FILE_IMMUTABLE`, `LEGAL_HOLD_ACTIVE`.

#### 6. `GET /files/{file_id}/versions`
- Resource: File version collection.
- Request schema: path `file_id`, pagination query.
- Response schema: `FileVersionPage`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `FILE_NOT_FOUND`, `VERSION_HISTORY_DENIED`.

### Folders

#### 7. `POST /folders`
- Resource: Folder collection.
- Request schema: `FolderCreateRequest` with `name`, `parent_folder_id`, `context_kind`.
- Response schema: `Folder`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`.
- Error shape: `FOLDER_NAME_CONFLICT`, `CONTEXT_KIND_MISMATCH`.

#### 8. `GET /folders/{folder_id}`
- Resource: Folder entity.
- Request schema: path `folder_id`.
- Response schema: `Folder`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `FOLDER_NOT_FOUND`, `FOLDER_READ_DENIED`.

### Upload

#### 9. `POST /upload/sessions`
- Resource: Upload session collection.
- Request schema: `CreateUploadSessionRequest` with `filename`, `byte_size`, `chunking`, `sha256`.
- Response schema: `UploadSession`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `413`, `422`, `429`, `503`.
- Error shape: `UPLOAD_SIZE_EXCEEDS_TIER`, `UPLOAD_HASH_REQUIRED`.

#### 10. `HEAD /upload/sessions/{session_id}`
- Resource: Upload session state.
- Request schema: path `session_id`.
- Response schema: tus headers including `Upload-Offset`.
- Status codes: `200`, `401`, `403`, `404`, `409`, `410`, `429`, `503`.
- Error shape: `UPLOAD_SESSION_NOT_FOUND`, `UPLOAD_SESSION_EXPIRED`.

#### 11. `PATCH /upload/sessions/{session_id}`
- Resource: Upload session chunk stream.
- Request schema: binary chunk with `Upload-Offset` and `Content-Type: application/offset+octet-stream`.
- Response schema: `UploadChunkResponse`.
- Status codes: `204`, `400`, `401`, `403`, `404`, `409`, `413`, `422`, `429`, `503`.
- Error shape: `UPLOAD_OFFSET_MISMATCH`, `UPLOAD_CHUNK_TOO_LARGE`.

#### 12. `DELETE /upload/sessions/{session_id}`
- Resource: Upload session entity.
- Request schema: path `session_id`.
- Response schema: empty success envelope.
- Status codes: `204`, `401`, `403`, `404`, `409`, `429`.
- Error shape: `UPLOAD_SESSION_ALREADY_COMPLETED`, `UPLOAD_ABORT_DENIED`.

### Download

#### 13. `GET /download/{file_id}`
- Resource: File bytes.
- Request schema: path `file_id`, optional `Range` header.
- Response schema: binary stream with content headers.
- Status codes: `200`, `206`, `304`, `401`, `403`, `404`, `416`, `429`, `503`.
- Error shape: `RANGE_NOT_SATISFIABLE`, `FILE_DOWNLOAD_DENIED`.

#### 14. `POST /download/{file_id}/signed-url`
- Resource: Signed download URL.
- Request schema: `MintSignedUrlRequest` with `expires_in_seconds`, `audience`, `range_allowed`.
- Response schema: `SignedUrlResponse`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `SIGNED_URL_POLICY_DENIED`, `DLP_SHARE_BLOCKED`.

### Sync

#### 15. `POST /sync/sessions`
- Resource: Delta sync session.
- Request schema: `OpenSyncSessionRequest` with `device_id`, `root_folder_id`, `last_watermark`.
- Response schema: `SyncSession`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `SYNC_DEVICE_REVOKED`, `SYNC_WATERMARK_INVALID`.

#### 16. `POST /sync/sessions/{session_id}/manifest`
- Resource: Sync manifest submission.
- Request schema: `SubmitManifestRequest` with `chunks[]`, `file_versions[]`.
- Response schema: `SyncDelta`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `503`.
- Error shape: `SYNC_CONFLICT_DETECTED`, `MANIFEST_SCHEMA_INVALID`.

### Share Links

#### 17. `POST /share-links`
- Resource: Share link collection.
- Request schema: `MintShareLinkRequest` with `resource_id`, `expires_at`, `password_policy`, `view_limit`.
- Response schema: `ShareLink`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `SHARE_LINK_DLP_BLOCKED`, `SHARE_LINK_POLICY_DENIED`.

#### 18. `DELETE /share-links/{link_id}`
- Resource: Share link entity.
- Request schema: path `link_id`.
- Response schema: empty success envelope.
- Status codes: `204`, `401`, `403`, `404`, `409`, `429`.
- Error shape: `SHARE_LINK_NOT_FOUND`, `SHARE_LINK_REVOKE_DENIED`.

#### 19. `POST /share-links/{link_id}/access`
- Resource: Public share access.
- Request schema: `ShareLinkAccessRequest` with `password`, `client_fingerprint`.
- Response schema: `ShareLinkAccessResponse`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `410`, `423`, `429`.
- Error shape: `SHARE_LINK_EXPIRED`, `SHARE_LINK_PASSWORD_INVALID`, `SHARE_VIEW_LIMIT_EXCEEDED`.

### Permissions

#### 20. `POST /permissions`
- Resource: Permission collection.
- Request schema: `GrantPermissionRequest` with `resource_id`, `principal_ref`, `role`, `expires_at`.
- Response schema: `Permission`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `PERMISSION_ALREADY_EXISTS`, `PERMISSION_SCOPE_DENIED`.

#### 21. `DELETE /permissions/{permission_id}`
- Resource: Permission entity.
- Request schema: path `permission_id`.
- Response schema: empty success envelope.
- Status codes: `204`, `401`, `403`, `404`, `409`, `429`.
- Error shape: `PERMISSION_NOT_FOUND`, `OWNER_PERMISSION_REQUIRED`.

### Search, Preview, Scan, Immutability

#### 22. `GET /search`
- Resource: Search index.
- Request schema: query `q`, `folder_id`, `mime_type`, `cursor`, `page_size`.
- Response schema: `SearchResponse`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `503`.
- Error shape: `SEARCH_QUERY_INVALID`, `SEARCH_INDEX_DEGRADED`.

#### 23. `GET /preview/{file_id}`
- Resource: Preview artifact.
- Request schema: path `file_id`, query `kind=thumbnail|first-page|video-frame`.
- Response schema: binary preview stream or `PreviewArtifact`.
- Status codes: `200`, `202`, `401`, `403`, `404`, `409`, `415`, `429`.
- Error shape: `PREVIEW_NOT_READY`, `PREVIEW_UNSUPPORTED_TYPE`.

#### 24. `GET /scan/{file_id}/verdict`
- Resource: Malware and DLP verdict.
- Request schema: path `file_id`.
- Response schema: `ScanVerdictResponse`.
- Status codes: `200`, `202`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `SCAN_PENDING`, `SCAN_BACKEND_UNAVAILABLE`.

#### 25. `POST /immutability/files/{file_id}`
- Resource: WORM immutability record.
- Request schema: `ElectWormRequest` with `retention_until`, `reason`, `policy_ref`.
- Response schema: `ImmutabilityRecord`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `WORM_POLICY_INVALID`, `FILE_ALREADY_IMMUTABLE`.

#### 26. `POST /immutability/files/{file_id}/legal-hold`
- Resource: Legal hold.
- Request schema: `OpenLegalHoldRequest` with `hold_reason`, `case_ref`, `expires_at`.
- Response schema: `LegalHold`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `LEGAL_HOLD_SCOPE_INVALID`, `COMPLIANCE_OFFICER_REQUIRED`.

## gRPC Methods

Package: `oya.drive.v1`.

### `FileStore`

- `rpc CreateFile(CreateFileRequest) returns (File);`
- `rpc GetFile(GetFileRequest) returns (File);`
- `rpc UpdateFile(UpdateFileRequest) returns (File);`
- `rpc TrashFile(TrashFileRequest) returns (google.protobuf.Empty);`
- `rpc ListFiles(ListFilesRequest) returns (ListFilesResponse);`

### `FolderHierarchy`

- `rpc CreateFolder(CreateFolderRequest) returns (Folder);`
- `rpc GetFolder(GetFolderRequest) returns (Folder);`

### `Upload`

- `rpc CreateSession(CreateUploadSessionRequest) returns (UploadSession);`
- `rpc UploadChunk(stream UploadChunkRequest) returns (UploadChunkResponse);`
- `rpc CompleteSession(CompleteSessionRequest) returns (File);`

### `Download`

- `rpc Get(GetDownloadRequest) returns (stream GetDownloadResponse);`
- `rpc MintSignedUrl(MintSignedUrlRequest) returns (SignedUrlResponse);`

### `ShareLinkService`

- `rpc Mint(MintShareLinkRequest) returns (ShareLink);`
- `rpc Revoke(RevokeShareLinkRequest) returns (google.protobuf.Empty);`

### `Search`

- `rpc Query(SearchQueryRequest) returns (SearchResponse);`

## AsyncAPI Channels

Delivery defaults:

- NATS workflow event bus.
- CloudEvents JSON envelope.
- At-least-once delivery with idempotent consumers by `event_id`.
- Every event carries `tenant_id`, `context_kind`, `resource_id`, `content_hash`, and `audit_chain_ref`.

Publish channels:

- `drive.file.lifecycle.v1`: payloads `FileUploaded`, `FileUpdated`, `FileMovedToTrash`, `FilePurged`.
- `drive.file.access.v1`: payload `FileDownloaded`.
- `drive.share.v1`: payloads `ShareLinkIssued`, `ShareLinkAccessed`, `ShareLinkRevoked`.
- `drive.permissions.v1`: payload `PermissionChanged`.
- `drive.sync.v1`: payloads `SyncDeltaApplied`, `SyncConflictDetected`.
- `drive.scan.v1`: payloads `VirusDetected`, `DlpFlagRaised`.
- `audit.drive.immutability.v1`: payload `ImmutabilityTierEntered`.
- `audit.drive.legal_hold.v1`: payloads `LegalHoldApplied`, `LegalHoldReleased`.
- `drive.quota.v1`: payload `QuotaThresholdCrossed`.

Subscribe channels:

- `tenancy.member.deprovisioned`: revoke device and share access; payload `MemberDeprovisioned`.
- `governance.retention_policy.changed`: update WORM and legal-hold policy; payload `RetentionPolicyChanged`.
- `audit-chain.seal.confirmed`: mark emitted file events sealed; payload `AuditSealConfirmed`.
- `workflow.file_action.requested`: run file automation; payload `WorkflowFileActionRequested`.
- `messenger.attachment.requested`: create shareable attachment target; payload `MessengerAttachmentRequested`.

## Webhooks Inbound

- `workflow.file_action.requested`: payload `WorkflowFileActionWebhook`, invokes copy, move, or classification action.
- `messenger.attachment.promote_requested`: payload `MessengerAttachmentWebhook`, promotes upload into drive storage.
- `mail.attachment.archive_requested`: payload `MailAttachmentArchiveWebhook`, stores mail attachment record.
- `governance.legal_hold.changed`: payload `LegalHoldChangedWebhook`, opens or releases file holds.
- `audit-chain.seal.failed`: payload `AuditSealFailedWebhook`, marks chain state degraded.
- `tenancy.quota.changed`: payload `TenantQuotaChangedWebhook`, adjusts quota enforcement.
- `intelligence.classification.completed`: payload `ClassificationCompletedWebhook`, writes labels and DLP metadata.

## SDK Quick Reference

Rust:

```rust
let client = DriveClient::connect(endpoint, token)?;
let session = client.create_upload_session(CreateUploadSessionRequest::new("evidence.pdf")).await?;
client.upload_chunk(session.id(), 0, bytes).await?;
let file = client.complete_upload(session.id()).await?;
let link = client.mint_share_link(file.id(), SharePolicy::expires_in_hours(24)).await?;
```

TypeScript:

```ts
const drive = new DriveClient({ endpoint, token, tenantId });
const session = await drive.createUploadSession({ filename: "evidence.pdf", byteSize });
await drive.uploadChunk({ sessionId: session.sessionId, offset: 0, body });
const file = await drive.completeUpload({ sessionId: session.sessionId });
await drive.mintShareLink({ resourceId: file.fileId, expiresInSeconds: 86400 });
```

Python:

```python
drive = DriveClient(endpoint=endpoint, token=token, tenant_id=tenant_id)
session = drive.create_upload_session(filename="evidence.pdf", byte_size=size)
drive.upload_chunk(session_id=session.session_id, offset=0, body=chunk)
file = drive.complete_upload(session_id=session.session_id)
drive.mint_share_link(resource_id=file.file_id, expires_in_seconds=86400)
```

Named SDK functions:

- `create_file(metadata, content)`
- `list_files(folder_id=None, cursor=None)`
- `create_upload_session(input)`
- `upload_chunk(session_id, offset, body)`
- `complete_upload(session_id)`
- `download(file_id, range=None)`
- `mint_signed_url(file_id, policy)`
- `open_sync_session(device_id, root_folder_id)`
- `mint_share_link(resource_id, policy)`
- `grant_permission(resource_id, principal_ref, role)`
- `render_preview(file_id, kind)`
- `get_scan_verdict(file_id)`
- `elect_worm(file_id, retention_until)`
- `open_legal_hold(file_id, reason)`

## Error Catalogue

- `DRIVE_AUTHN_MISSING`: missing bearer, API key, or SigV4 credential; do not retry unchanged.
- `DRIVE_AUTHZ_DENIED`: Cedar denied action; do not retry unchanged.
- `TENANT_SCOPE_MISMATCH`: token tenant differs from header; fix credentials.
- `CONTEXT_KIND_MISMATCH`: Personal/Professional boundary violation; do not retry.
- `FILE_NOT_FOUND`: file missing or hidden; do not retry unchanged.
- `FILE_TOO_LARGE_FOR_SINGLE_SHOT`: use multipart upload.
- `FILE_VERSION_CONFLICT`: optimistic concurrency conflict; fetch latest and retry.
- `FILE_IMMUTABLE`: WORM policy blocks mutation; do not retry.
- `LEGAL_HOLD_ACTIVE`: held file cannot be purged; wait for hold release.
- `UPLOAD_OFFSET_MISMATCH`: client and server offsets differ; HEAD session then retry.
- `UPLOAD_SESSION_EXPIRED`: create a new session.
- `DLP_SHARE_BLOCKED`: DLP policy blocks share; resolve classification.
- `SHARE_LINK_EXPIRED`: create a new link if authorized.
- `SHARE_VIEW_LIMIT_EXCEEDED`: view cap reached; do not retry.
- `SYNC_CONFLICT_DETECTED`: surface conflict resolution; do not auto-overwrite.
- `SEARCH_INDEX_DEGRADED`: retry with exponential backoff or degrade to metadata search.
- `PREVIEW_UNSUPPORTED_TYPE`: unsupported renderer; do not retry.
- `SCAN_PENDING`: retry after backoff or subscribe to scan channel.
- `WORM_POLICY_INVALID`: correct retention policy.
- `RATE_LIMIT`: retry after `Retry-After`.
- `DEPENDENCY_UNAVAILABLE`: object store, scan, or index dependency down; retry with jitter.

## Pagination

Cursor pattern name: `drive_resource_cursor_v1`.

- Cursor format: opaque, signed, tenant-bound token.
- Default page size: `100`.
- Maximum file list page size: `500`.
- Maximum folder child page size: `500`.
- Maximum version page size: `200`.
- Maximum search page size: `100`.
- Maximum permission page size: `500`.
- Stable ordering: folder listings by normalized name, search by relevance then update time.
- Mutation safety: cursors bind to a folder or query snapshot watermark.
- Bulk lookup maximum: `100` ids per batch reference where supported by SDK.

## Rate Limits per Tier

ADR-0316 capability tiers control throughput and storage envelopes.

| Tier | Metadata reads | Metadata writes | Upload ingress | Download egress | Notes |
|---|---:|---:|---:|---:|---|

Rate-limit headers:

- `Retry-After`
- `throttle-class`
- `throttle-user-headroom`
- `throttle-tenant-headroom`
- `Upload-Offset` for resumable upload recovery

## OpenAPI 3.2.0 Schema

Contract file: [`microservices/drive/contracts/openapi/drive.yaml`](../../microservices/drive/contracts/openapi/drive.yaml).

Compatibility projection: [`contracts/openapi/workspace/workspace-drive-v1.yaml`](../../contracts/openapi/workspace/workspace-drive-v1.yaml).

## AsyncAPI 3.1.0 Schema

Contract file: [`microservices/drive/contracts/asyncapi/drive-events.yaml`](../../microservices/drive/contracts/asyncapi/drive-events.yaml).

## proto3 Schema

Contract file: [`microservices/drive/contracts/proto/drive.proto`](../../microservices/drive/contracts/proto/drive.proto).

## Cross-References

- PRD: [`microservices/drive/PRD.md`](../../microservices/drive/PRD.md).
- Architecture: [`microservices/drive/ARCHITECTURE.md`](../../microservices/drive/ARCHITECTURE.md).
- SDK plan: [`microservices/drive/sdk-plan.md`](../../microservices/drive/sdk-plan.md).
- Capability tiers: [`microservices/drive/capability-tiers/tier-matrix.md`](../../microservices/drive/capability-tiers/tier-matrix.md).
- Policies: [`microservices/drive/policy/`](../../microservices/drive/policy/).
- Runbooks: [`microservices/drive/runbooks/`](../../microservices/drive/runbooks/).
- API standard: [`docs/standards/api-design.md`](../standards/api-design.md).
- Throttling standard: [`docs/standards/throttling-tiers.md`](../standards/throttling-tiers.md).
- ADR-0316: [`docs/decisions/ADR-0709-general-live-apex.md`](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md).
