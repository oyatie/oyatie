---
doc_class: SdkPlan
template_id: TPL-SDK-PLAN
microservice: community
status: Accepted
date: 2026-05-17
owner_team: axis-community + axis-sdk
related_adrs: [ADR-0056, ADR-0105, ADR-0126, ADR-0131]
doc_status: published
---

# SDK Plan: community µservice

## Target Languages

| Language | Status | Rationale |
|---|---|---|
| Rust | Live (oya-community-*-sdk) | Native + workspace consumer |
| TypeScript / JavaScript | M02 ship | Web app, Workflow Studio editor |
| Python | M02 ship | Foundry-runtime + agent integrations |
| Go | M03 | Cloud-native ecosystem |
| Java / Kotlin | M03 | Enterprise tenant integrations |
| Swift | M04 | iOS apps |
| Kotlin (Android) | M04 | Android apps |

## SDK Surfaces

### post-store SDK

- `community.posts.create(input) → Post`
- `community.posts.edit(id, input) → Post`
- `community.posts.delete(id) → void`
- `community.posts.read(id) → Post`
- `community.posts.list(query) → PostList`
- Streaming: `community.posts.subscribe(filter) → AsyncIterator<Event>` (server-sent events)

### thread-tree SDK

- `community.threads.listReplies(postId, depth) → ThreadNodeList`
- `community.threads.postReply(postId, parentNodeId, body) → ThreadNode`

### voting-engine SDK

- `community.votes.cast(postId, direction, idempotencyKey) → VoteTally`
- `community.votes.tally(postId) → VoteTally`
- `community.votes.acceptAnswer(postId, replyId) → void`

### moderation-queue SDK

- `community.moderation.queue.list(filter) → QueueList`
- `community.moderation.actions.apply(input) → ModerationAction`
- `community.moderation.flags.raise(target, reason, note) → Flag`

### kb-article-store SDK

- `community.kb.articles.create(input) → KbArticle`
- `community.kb.articles.edit(id, input) → KbArticle`
- `community.kb.articles.publish(id) → KbArticle`
- `community.kb.articles.read(id) → KbArticle`
- `community.kb.articles.uploadAttachment(id, file) → Attachment` (resumable)

### search-index SDK

- `community.search(q, scope) → SearchResults`
- `community.search.subscribe(q) → AsyncIterator<SearchHit>` (live results)

## Generation Pipeline

- Source: `contracts/openapi/community.yaml` + `contracts/proto/community.proto` + `contracts/asyncapi/community-events.yaml`.
- Generator: `openapi-generator` (TS/Java/Swift/Kotlin), `python-openapi-codegen` (Python), `buf` (Go from proto), oyatie's `cargo-codegen` (Rust).
- CI lane: per-language generation + smoke + publish to registry.

## Versioning

- SDK version = `<api_major>.<sdk_minor>.<patch>`.
- API major bumps require migration guide.
- 6-month deprecation window per `feedback_no_silent_regression.md`.

## Auth

- Per-language idiomatic JWT bearer.
- Token refresh: pluggable provider.
- mTLS: opt-in for service-to-service.

## Streaming

- Server-sent events for post / vote / moderation streams.
- Per-tenant subject pattern: `community.<tenant_id>.<bc>.*`.
- Backpressure: client-side ack window.

## Pagination

- Cursor-based; opaque token.
- Default limit: 50; max 200.

## Error Model

- Structured error: `{ code, message, retry_after?, details? }`.
- HTTP mapping: 400 invalid, 401 unauth, 403 forbidden, 404 not_found, 409 conflict, 429 rate_limited, 500 internal, 503 unavailable.

## Observability

- Auto-instrumented OTel spans per call.
- Per-call trace context propagation.
- Metric emission: client-side latency + error.

## Distribution

- npm: `@oyatie/community-sdk`
- PyPI: `oya-community-sdk`
- Maven: `io.oyatie:community-sdk`
- Crates: `oya-community-<bc>-sdk`

## Roadmap

| Milestone | Languages | Surfaces |
|---|---|---|
| M02 | TS / Python / Rust | post-store, thread-tree, voting-engine, moderation-queue, kb-article-store, search-index |
| M03 | + Go / Java | full parity |
| M04 | + Swift / Kotlin | full parity + offline-first |
