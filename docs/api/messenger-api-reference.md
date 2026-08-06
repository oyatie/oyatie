---
doc_class: APIReference
microservice: messenger
version: 1.0.0
status: Accepted
date: 2026-05-20
owner: axis-messenger + council-privacy + ops-deliverability
openapi_version: 3.2.0
asyncapi_version: 3.1.0
proto3: true
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# messenger API Reference

Canonical REST, gRPC, and AsyncAPI reference for the `messenger` microservice.
The reference is contract-first and points to the live contract files under
`microservices/messenger/contracts/`.

## Quick Start

Named example: `PostChannelMessageAndSubscribe`.

1. Create or select a channel with `POST /channels`.
2. Post a message with `POST /channels/{channel_id}/messages`.
3. Subscribe to `workflow-events/messenger.message.posted` or WebSocket `ws-message-frame`.

Minimum headers:

- `Authorization: Bearer <oidc-token>`
- `X-Scope-OrgID: tenant:<hashed-id>`
- `X-Context-Kind: Personal | Professional`
- `Idempotency-Key: <ulid>` on state-changing requests
- `X-Request-Id: <ulid>` for trace correlation

Example REST flow:

```http
POST /api/v1/channels HTTP/2
Host: messenger-kr.oyatie.com
Authorization: Bearer eyJ...
X-Scope-OrgID: tenant:2f0a1c0e9b77aa11
X-Context-Kind: Professional
Idempotency-Key: 01HYMSGCREATE000000000000
Content-Type: application/json
```

## Authentication & Authorization

Authentication is OIDC bearer for tenant-facing REST and gRPC metadata.
Internal mesh callers use SPIFFE-bound mTLS where deployed.

Principal types:

- `PersonalMessengerUser`: personal context user, no tenant admin visibility.
- `WorkMessengerMember`: professional tenant member, channel-scoped.
- `ChannelAdmin`: member with channel membership management rights.
- `ComplianceOfficer`: professional context officer for holds and disclosures.
- `MessengerBridgeWorker`: internal bridge principal for mail, workflow, and ontology events.
- `MessengerAuditor`: scoped auditor with read-only event and evidence access.

Named Cedar policy patterns:

- `messenger::tenant_scope_match`: JWT tenant must match `X-Scope-OrgID`.
- `messenger::dual_context_isolation`: Personal and Professional data never co-resolve.
- `messenger::channel_member_read`: read requires channel membership or legal hold scope.
- `messenger::channel_admin_write`: membership edits require channel admin role.
- `messenger::message_author_edit`: edits require author identity and edit window.
- `messenger::compliance_four_eyes_disclosure`: disclosure requires paired approval.
- `messenger::attachment_malware_quarantine`: quarantined attachments are not readable.
- `messenger::retention_hold_override`: hold prevents tombstone purge.

Authorization failure shape:

```json
{
  "error": {
    "code": "MESSENGER_AUTHZ_DENIED",
    "message": "Cedar policy denied messenger action",
    "request_id": "01HYREQ...",
    "details": [{"policy": "messenger::channel_member_read"}]
  }
}
```

## REST Endpoints

Base URL: `https://messenger-{pack}.oyatie.com/api/v1`.

All success responses use `{ "data": ..., "metadata": ... }` unless the OpenAPI
operation returns a typed object directly. All error responses use `{ "error": ... }`.

### Channels

#### 1. `GET /channels`
- Resource: Channel collection.
- Request schema: `ListChannelsRequest` query with `cursor`, `page_size`, `visibility`.
- Response schema: `ListChannelsResponse` containing `channels[]` and `cursor_next`.
- Status codes: `200`, `400`, `401`, `403`, `429`, `503`.
- Error shape: `MESSENGER_CURSOR_INVALID`, `MESSENGER_AUTHZ_DENIED`, `RATE_LIMIT`.

#### 2. `POST /channels`
- Resource: Channel collection.
- Request schema: `CreateChannelRequest` with `name`, `channel_kind`, `context_kind`, `member_refs[]`.
- Response schema: `Channel`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`, `503`.
- Error shape: `CHANNEL_NAME_CONFLICT`, `CONTEXT_KIND_MISMATCH`, `IDEMPOTENCY_REPLAY_CONFLICT`.

#### 3. `GET /channels/{channel_id}`
- Resource: Channel entity.
- Request schema: path `channel_id`.
- Response schema: `Channel`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `CHANNEL_NOT_FOUND`, `CHANNEL_READ_DENIED`.

#### 4. `DELETE /channels/{channel_id}`
- Resource: Channel entity.
- Request schema: path `channel_id`, optional retention reason.
- Response schema: `ArchiveChannelResponse` or empty success envelope.
- Status codes: `202`, `401`, `403`, `404`, `409`, `429`, `503`.
- Error shape: `CHANNEL_HOLD_ACTIVE`, `CHANNEL_ARCHIVE_DENIED`.

### Members

#### 5. `GET /channels/{channel_id}/members`
- Resource: Channel member collection.
- Request schema: path `channel_id`, pagination query.
- Response schema: `ListChannelMembersResponse`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `CHANNEL_NOT_FOUND`, `MEMBER_LIST_DENIED`.

#### 6. `POST /channels/{channel_id}/members`
- Resource: Channel member collection.
- Request schema: `AddChannelMemberRequest` with `user_ref`, `role`, `expires_at`.
- Response schema: `ChannelMember`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `MEMBER_ALREADY_PRESENT`, `CHANNEL_ADMIN_REQUIRED`.

#### 7. `DELETE /channels/{channel_id}/members/{user_ref}`
- Resource: Channel member entity.
- Request schema: path `channel_id`, `user_ref`.
- Response schema: empty success envelope.
- Status codes: `204`, `401`, `403`, `404`, `409`, `429`.
- Error shape: `MEMBER_NOT_FOUND`, `LAST_OWNER_REMOVAL_FORBIDDEN`.

### Messages

#### 8. `GET /channels/{channel_id}/messages`
- Resource: Message collection.
- Request schema: path `channel_id`, `cursor`, `page_size`, `around_message_id`.
- Response schema: `ListMessagesResponse`.
- Status codes: `200`, `401`, `403`, `404`, `422`, `429`, `503`.
- Error shape: `MESSAGE_CURSOR_INVALID`, `CHANNEL_READ_DENIED`.

#### 9. `POST /channels/{channel_id}/messages`
- Resource: Message collection.
- Request schema: `PostMessageRequest` with `body`, `blocks[]`, `attachment_ids[]`, `thread_id`.
- Response schema: `Message`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`, `503`.
- Error shape: `MESSAGE_VALIDATION_FAILED`, `ATTACHMENT_QUARANTINED`, `IDEMPOTENCY_REPLAY_CONFLICT`.

#### 10. `GET /channels/{channel_id}/messages/{message_id}`
- Resource: Message entity.
- Request schema: path `channel_id`, `message_id`.
- Response schema: `Message`.
- Status codes: `200`, `401`, `403`, `404`, `410`, `429`, `503`.
- Error shape: `MESSAGE_NOT_FOUND`, `MESSAGE_TOMBSTONED`.

#### 11. `PATCH /channels/{channel_id}/messages/{message_id}`
- Resource: Message entity.
- Request schema: `EditMessageRequest` with `body`, `blocks[]`, `expected_version`.
- Response schema: `Message`.
- Status codes: `200`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `MESSAGE_EDIT_WINDOW_EXPIRED`, `MESSAGE_VERSION_CONFLICT`.

#### 12. `DELETE /channels/{channel_id}/messages/{message_id}`
- Resource: Message entity.
- Request schema: path identifiers and optional delete reason.
- Response schema: empty success envelope.
- Status codes: `204`, `401`, `403`, `404`, `409`, `410`, `429`.
- Error shape: `MESSAGE_DELETE_DENIED`, `MESSAGE_HOLD_ACTIVE`.

#### 13. `POST /channels/{channel_id}/messages/{message_id}/reactions`
- Resource: Message reaction collection.
- Request schema: `AddReactionRequest` with `emoji`, `skin_tone`, `client_nonce`.
- Response schema: `Reaction`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `REACTION_ALREADY_EXISTS`, `REACTION_EMOJI_BLOCKED`.

#### 14. `DELETE /channels/{channel_id}/messages/{message_id}/reactions/{emoji}`
- Resource: Message reaction entity.
- Request schema: path identifiers.
- Response schema: empty success envelope.
- Status codes: `204`, `401`, `403`, `404`, `429`.
- Error shape: `REACTION_NOT_FOUND`, `REACTION_REMOVE_DENIED`.

#### 15. `POST /channels/{channel_id}/messages/{message_id}/read`
- Resource: Read receipt cursor.
- Request schema: `MarkReadRequest` with `read_at`, `device_id`.
- Response schema: `ReadReceipt`.
- Status codes: `200`, `401`, `403`, `404`, `409`, `429`.
- Error shape: `READ_CURSOR_REGRESSION`, `CHANNEL_READ_DENIED`.

### Threads

#### 16. `GET /threads/{thread_id}/replies`
- Resource: Thread reply collection.
- Request schema: path `thread_id`, pagination query.
- Response schema: `ListThreadRepliesResponse`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `THREAD_NOT_FOUND`, `THREAD_READ_DENIED`.

#### 17. `POST /threads/{thread_id}/replies`
- Resource: Thread reply collection.
- Request schema: `PostThreadReplyRequest` with `body`, `blocks[]`, `attachment_ids[]`.
- Response schema: `Message`.
- Status codes: `201`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `THREAD_LOCKED`, `MESSAGE_VALIDATION_FAILED`.

### Attachments

#### 18. `POST /attachments`
- Resource: Attachment upload session.
- Request schema: `InitiateUploadRequest` with `filename`, `content_type`, `byte_size`, `sha256`.
- Response schema: `UploadSession`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `413`, `422`, `429`.
- Error shape: `ATTACHMENT_TOO_LARGE`, `ATTACHMENT_TYPE_DENIED`.

#### 19. `GET /attachments/{attachment_id}`
- Resource: Attachment entity.
- Request schema: path `attachment_id`.
- Response schema: `Attachment` plus signed URL metadata when readable.
- Status codes: `200`, `401`, `403`, `404`, `409`, `423`, `429`.
- Error shape: `ATTACHMENT_QUARANTINED`, `ATTACHMENT_READ_DENIED`.

### Search And Presence

#### 20. `GET /search`
- Resource: Message search index.
- Request schema: query `q`, `channel_id`, `from`, `to`, `cursor`, `page_size`.
- Response schema: `SearchMessagesResponse`.
- Status codes: `200`, `400`, `401`, `403`, `422`, `429`, `503`.
- Error shape: `SEARCH_QUERY_INVALID`, `SEARCH_INDEX_DEGRADED`.

#### 21. `GET /presence/{user_ref}`
- Resource: Presence state.
- Request schema: path `user_ref`.
- Response schema: `PresenceState`.
- Status codes: `200`, `401`, `403`, `404`, `429`, `503`.
- Error shape: `PRESENCE_SCOPE_DENIED`, `PRESENCE_NOT_VISIBLE`.

#### 22. `PUT /presence/me`
- Resource: Own presence state.
- Request schema: `UpdateOwnPresenceRequest` with `state`, `expires_at`, `device_id`.
- Response schema: `PresenceState`.
- Status codes: `200`, `400`, `401`, `409`, `422`, `429`, `503`.
- Error shape: `PRESENCE_STATE_INVALID`, `DEVICE_SESSION_EXPIRED`.

### Compliance

#### 23. `POST /holds`
- Resource: eDiscovery hold.
- Request schema: `OpenEDiscoveryHoldRequest` with `scope`, `reason`, `expires_at`.
- Response schema: `EDiscoveryHold`.
- Status codes: `201`, `400`, `401`, `403`, `409`, `422`, `429`.
- Error shape: `COMPLIANCE_OFFICER_REQUIRED`, `HOLD_SCOPE_INVALID`.

#### 24. `POST /disclosures`
- Resource: Four-eyes disclosure request.
- Request schema: `RequestDisclosureRequest` with `message_ids[]`, `approver_ref`, `reason`.
- Response schema: `DisclosureRequest`.
- Status codes: `202`, `400`, `401`, `403`, `404`, `409`, `422`, `429`.
- Error shape: `DISCLOSURE_APPROVER_REQUIRED`, `DISCLOSURE_SCOPE_DENIED`.

### Health

#### 25. `GET /health`
- Resource: Liveness probe.
- Request schema: none.
- Response schema: `HealthStatus`.
- Status codes: `200`, `503`.
- Error shape: `SERVICE_UNHEALTHY`.

#### 26. `GET /ready`
- Resource: Readiness probe.
- Request schema: none.
- Response schema: `ReadyStatus` with Postgres, Valkey, and S3 reachability.
- Status codes: `200`, `503`.
- Error shape: `DEPENDENCY_UNAVAILABLE`.

## gRPC Methods

Package: `oya.messenger.v1`.

### `ChannelStore`

- `rpc CreateChannel(CreateChannelRequest) returns (Channel);`
- `rpc GetChannel(GetChannelRequest) returns (Channel);`
- `rpc ListChannels(ListChannelsRequest) returns (ListChannelsResponse);`
- `rpc ArchiveChannel(ArchiveChannelRequest) returns (google.protobuf.Empty);`
- `rpc AddChannelMember(AddChannelMemberRequest) returns (google.protobuf.Empty);`
- `rpc RemoveChannelMember(RemoveChannelMemberRequest) returns (google.protobuf.Empty);`

### `MessageStream`

- `rpc PostMessage(PostMessageRequest) returns (Message);`
- `rpc ListMessages(ListMessagesRequest) returns (ListMessagesResponse);`
- `rpc EditMessage(EditMessageRequest) returns (Message);`
- `rpc DeleteMessage(DeleteMessageRequest) returns (google.protobuf.Empty);`
- `rpc StreamMessages(StreamMessagesRequest) returns (stream Message);`

### `ThreadTree`

- `rpc GetThread(GetThreadRequest) returns (Thread);`
- `rpc PostThreadReply(PostThreadReplyRequest) returns (Message);`

### `ReadReceiptTracker`

- `rpc MarkRead(MarkReadRequest) returns (google.protobuf.Empty);`

### `Presence`

- `rpc StreamPresence(StreamPresenceRequest) returns (stream PresenceState);`

## AsyncAPI Channels

Delivery defaults:

- Workflow bus: AMQP/NATS, at-least-once, idempotent consumers by `event_id`.
- WebSocket gateway: session-scoped, ordered per connection, replay gap by cursor.
- All payloads carry `tenant_id`, `context_kind`, `principal_ref`, `occurred_at`, and `audit_chain_ref`.

Publish channels:

- `workflow-events/messenger.message.posted`: payload `MessagePosted`, delivery at-least-once.
- `workflow-events/messenger.message.edited`: payload `MessageEdited`, delivery at-least-once.
- `workflow-events/messenger.message.deleted`: payload `MessageDeleted`, delivery at-least-once.
- `workflow-events/messenger.message.reaction`: payload `MessageReaction`, delivery at-least-once.
- `workflow-events/messenger.presence.changed`: payload `PresenceChanged`, delivery best-effort for realtime plus durable summary.
- `workflow-events/messenger.file.attached`: payload `FileAttached`, delivery at-least-once after malware verdict.
- `workflow-events/messenger.mention.emitted`: payload `MentionEmitted`, delivery at-least-once to workflow and notification consumers.
- `workflow-events/messenger.channel.created`: payload `ChannelCreated`, delivery at-least-once.
- `workflow-events/messenger.channel.member.changed`: payload `ChannelMemberChanged`, delivery at-least-once.
- `workflow-events/messenger.ediscovery.hold`: payload `EDiscoveryHold`, delivery exactly-once effect by hold id.
- `workflow-events/messenger.four-eyes.disclosure`: payload `FourEyesDisclosure`, delivery exactly-once effect by disclosure id.

Subscribe channels:

- `ws-message-frame`: client subscribes to readable messages; payload `MessageFrame`.
- `ws-presence-frame`: client subscribes to visible presence; payload `PresenceFrame`.
- `ws-reaction-frame`: client subscribes to reaction deltas; payload `ReactionFrame`.
- `mail.action-card.created`: bridge consumes mail action cards; payload `MailActionCard`.
- `ontology.entity.changed`: bridge consumes entity mention target updates; payload `OntologyEntityChanged`.
- `tenancy.retention-policy.updated`: consumes retention policy changes; payload `RetentionPolicyUpdated`.
- `audit-chain.sealed`: consumes seal confirmation; payload `AuditChainSealed`.

## Webhooks Inbound

Inbound webhook handlers verify HMAC or mTLS identity before conversion to internal events.

- `mail.action_card.created`: payload `MailActionCardWebhook`, creates message action card.
- `drive.file_scan.completed`: payload `DriveScanCompletedWebhook`, releases or quarantines attachment.
- `workflow.run.status_changed`: payload `WorkflowRunStatusWebhook`, posts workflow status message.
- `calendar.meeting.started`: payload `CalendarMeetingStartedWebhook`, posts huddle prompt.
- `tenancy.member.deprovisioned`: payload `MemberDeprovisionedWebhook`, removes work memberships.
- `governance.retention_hold.changed`: payload `RetentionHoldChangedWebhook`, updates hold state.
- `audit-chain.seal.failed`: payload `AuditSealFailedWebhook`, marks compliance export degraded.

## SDK Quick Reference

Rust crate names are planned around `oya-messenger-*-sdk` layer crates.

Rust:

```rust
let client = MessengerClient::connect(endpoint, token)?;
let channel = client.create_channel(CreateChannelRequest::work("incident-room")).await?;
let msg = client.post_message(channel.id(), "SEV-2 opened").await?;
client.mark_read(channel.id(), msg.id()).await?;
```

TypeScript:

```ts
const client = new MessengerClient({ endpoint, token, tenantId });
const channel = await client.createChannel({ name: "incident-room", contextKind: "Professional" });
await client.postMessage({ channelId: channel.channelId, body: "SEV-2 opened" });
for await (const frame of client.streamMessages({ channelId: channel.channelId })) {}
```

Python:

```python
client = MessengerClient(endpoint=endpoint, token=token, tenant_id=tenant_id)
channel = client.create_channel(name="incident-room", context_kind="Professional")
client.post_message(channel_id=channel.channel_id, body="SEV-2 opened")
client.mark_read(channel_id=channel.channel_id, message_id="msg_...")
```

Named SDK functions:

- `create_channel(input)`
- `list_channels(cursor=None, page_size=100)`
- `post_message(channel_id, body, attachments=None)`
- `edit_message(channel_id, message_id, expected_version, body)`
- `stream_messages(channel_id, cursor=None)`
- `initiate_attachment_upload(metadata)`
- `update_presence(state)`
- `open_ediscovery_hold(scope, reason)`

## Error Catalogue

- `MESSENGER_AUTHN_MISSING`: no bearer or mTLS identity; do not retry without credentials.
- `MESSENGER_AUTHZ_DENIED`: Cedar denied the action; do not retry until policy or principal changes.
- `TENANT_SCOPE_MISMATCH`: token tenant differs from header; do not retry unchanged.
- `CONTEXT_KIND_MISMATCH`: personal/professional boundary violation; do not retry unchanged.
- `CHANNEL_NOT_FOUND`: channel id is absent or hidden; do not retry unchanged.
- `CHANNEL_NAME_CONFLICT`: duplicate channel name; retry with a different name.
- `CHANNEL_ADMIN_REQUIRED`: member mutation requires admin; do not retry unchanged.
- `MESSAGE_VALIDATION_FAILED`: body or block schema invalid; fix request.
- `MESSAGE_VERSION_CONFLICT`: optimistic concurrency conflict; fetch latest then retry.
- `MESSAGE_EDIT_WINDOW_EXPIRED`: edit window closed; do not retry.
- `MESSAGE_HOLD_ACTIVE`: delete/archive blocked by hold; do not retry until hold changes.
- `ATTACHMENT_TOO_LARGE`: upload exceeds tenant tier; split or upgrade.
- `ATTACHMENT_QUARANTINED`: malware/DLP hold active; retry after scan release only.
- `SEARCH_INDEX_DEGRADED`: search backend stale; retry with exponential backoff.
- `READ_CURSOR_REGRESSION`: cursor moved backwards; refresh state.
- `DISCLOSURE_APPROVER_REQUIRED`: four-eyes approver missing; resubmit with approver.
- `RATE_LIMIT`: request bucket exhausted; retry after `Retry-After`.
- `DEPENDENCY_UNAVAILABLE`: storage, Valkey, or search unavailable; retry with jitter.
- `AUDIT_SEAL_PENDING`: write accepted but seal delayed; poll or wait for async confirmation.

## Pagination

Cursor pattern name: `messenger_channel_message_cursor_v1`.

- Cursor format: opaque, signed, tenant-bound token.
- Default page size: `100`.
- Maximum channel page size: `500`.
- Maximum message page size: `200`.
- Maximum member page size: `500`.
- Maximum search page size: `100`.
- Stable ordering: channels by last activity, messages by append sequence.
- Mutation safety: cursors bind to snapshot watermark and never expose offset.
- Backward traversal: use `cursor_prev` where returned by the contract.

## Rate Limits per Tier

ADR-0316 tier names map to tenant-visible capability tiers, not separate services.

| Tier | REST reads | REST writes | gRPC streams | WebSocket fanout | Notes |
|---|---:|---:|---:|---:|---|

Rate-limit headers:

- `Retry-After`
- `oya-throttle-class`
- `oya-throttle-user-headroom`
- `oya-throttle-tenant-headroom`

## OpenAPI 3.2.0 Schema

Contract file: [`microservices/messenger/contracts/openapi/messenger.yaml`](../../microservices/messenger/contracts/openapi/messenger.yaml).

## AsyncAPI 3.1.0 Schema

Contract file: [`microservices/messenger/contracts/asyncapi/messenger-events.yaml`](../../microservices/messenger/contracts/asyncapi/messenger-events.yaml).

## proto3 Schema

Contract file: [`microservices/messenger/contracts/proto/messenger.proto`](../../microservices/messenger/contracts/proto/messenger.proto).

## Cross-References

- PRD: [`microservices/messenger/PRD.md`](../../microservices/messenger/PRD.md).
- Architecture: [`microservices/messenger/ARCHITECTURE.md`](../../microservices/messenger/ARCHITECTURE.md).
- SDK plan: [`microservices/messenger/sdk-plan.md`](../../microservices/messenger/sdk-plan.md).
- Capability tiers: [`microservices/messenger/capability-tiers/tier-matrix.md`](../../microservices/messenger/capability-tiers/tier-matrix.md).
- Policies: [`microservices/messenger/policy/`](../../microservices/messenger/policy/).
- API standard: [`docs/standards/api-design.md`](../standards/api-design.md).
- Throttling standard: [`docs/standards/throttling-tiers.md`](../standards/throttling-tiers.md).
- ADR-0316: [`docs/decisions/ADR-0709-general-live-apex.md`](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md).
