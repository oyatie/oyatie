---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-social + gtm-customer-success
deciders: axis-social, council-architecture
related_adrs: [ADR-0135, ADR-0131, ADR-0132]
related_artifacts:
  - microservices/social/contracts/openapi/social.yaml
  - microservices/social/contracts/asyncapi/social-events.yaml
  - microservices/social/contracts/proto/social.proto
  - microservices/social/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (social µservice)

## Purpose

Tenants embed social functionality in their own surfaces (mobile / desktop / embedded social widget / bot APIs) via first-party SDKs. Per Twitter/X, Mastodon, Bluesky AT-Proto, Threads precedent: each ships per-language SDKs that handle auth, WebSocket reconnection, event deserialisation, media-upload progress, and ergonomic patterns idiomatic to each language.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M02 (primary; oyatie's own language) | First-party authored (`oya-social-*-sdk` crates per BC) | axis-social |
| **TypeScript + Browser SDK** | M02 (web-client + ts-server) | OpenAPI + AsyncAPI generation + first-party ergonomic + WebSocket reconnect | axis-social + gtm |
| **Kotlin (Android) + Swift (iOS)** | M02-onward1 | gRPC-generated + first-party ergonomic + native WebSocket + multipart-upload | axis-social + gtm |
| **Python** | M03 | OpenAPI-generated + ergonomic | axis-social + gtm |
| **Go** | M03 | gRPC-generated + ergonomic | axis-social + gtm |
| **JVM (Java/Kotlin)** | M03-onward1 | gRPC-generated + ergonomic; published to Maven Central | axis-social + gtm |
| **C# / .NET** | M04-onward | OpenAPI-generated + ergonomic; published to NuGet | axis-social + gtm |
| **Dart (Flutter)** | M03 | OpenAPI-generated + multipart-upload helpers | axis-social + gtm |

Driver: mobile clients lead because most social usage is mobile (per industry data: ~80% of Twitter/X + Instagram + Threads usage is mobile). Web SDK ties to Workflow Studio shell. Server-language SDKs follow.

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/social/src/crates/oya-social-*-sdk/`.

- `Client::new(opts) -> Client; client.publish_post(...) -> Result<Post>`.
- WebSocket subscription: `client.subscribe_feed() -> impl Stream<Item=WsFrame>`.
- Reconnection: built-in exponential backoff + jitter; resume from last seen `seq`.
- Media upload: `client.upload_media(file, opts) -> Result<MediaRef>` with multipart progress.
- Context awareness: `ContextKind::{Personal, Professional}` set per Client; cross-context calls refused at compile-time.

### TypeScript + Browser SDK

`@oyatie/social-sdk` (npm). Sub-packages:
- `@oyatie/social-sdk` — node + browser core.
- `@oyatie/social-sdk-react` — React hooks (`useFeed`, `useProfile`, `usePost`).
- `@oyatie/social-sdk-media` — multipart media-upload + progress helpers.

### Mobile SDKs (iOS + Android)

- iOS: Swift Package via SwiftPM; binary XCFramework; supports iOS 15+.
- Android: Maven Central + Gradle; AAR; minSDK 26 (Android 8.0).
- Both bind to OS-native WebSocket + multipart-upload stacks.
- Push notification: APNs / FCM bound to per-device token; oyatie does not store device-tokens long-term beyond hash.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Get my profile | `getMyProfile()` | `Profile` |
| Update my profile | `updateMyProfile(req)` | `Profile` |
| Get profile by handle | `getProfile(handle)` | `Profile` |
| Follow / unfollow | `follow(user_ref)` / `unfollow(user_ref)` | `void` |
| Block / unblock | `block(user_ref)` / `unblock(user_ref)` | `void` |
| Mute / unmute | `mute(user_ref)` / `unmute(user_ref)` | `void` |
| Publish post | `publishPost(req)` | `Post` |
| Repost | `repost(post_id)` | `Post` |
| Quote-post | `quotePost(post_id, body)` | `Post` |
| Edit post | `editPost(post_id, body)` | `Post` (within edit-window) |
| Delete post | `deletePost(post_id)` | `void` (tombstone) |
| React | `addReaction(post_id, emoji)` / `removeReaction(post_id, emoji)` | `void` |
| Comment | `comment(post_id, body)` | `Post` (comment is a Post sub-kind) |
| Bookmark / unbookmark | `bookmark(post_id)` / `unbookmark(post_id)` | `void` |
| Render feed | `renderFeed(opts)` | `Feed` (chronological or algorithmic) |
| Subscribe real-time feed | `subscribeFeed() -> AsyncIterable<WsFrame>` | streaming |
| Search people | `searchPeople(q, opts)` | `Profile[]` |
| Search content | `searchContent(q, opts)` | `Post[]` |
| Get trending | `getTrending(pack, opts)` | `Topic[]` |
| Upload media | `uploadMedia(blob, opts)` | `MediaRef` (multipart) |
| Get notifications | `getNotifications(opts)` | `Notification[]` |
| Report abuse | `reportAbuse(post_id, reason)` | `AbuseReportId` |
| Open appeal | `openAppeal(verdict_id, reason)` | `AppealId` |
| Manage lists | `createList`, `addToList`, `removeFromList`, `getList`, `getListFeed` | `List` / `Feed` |

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue social API key + WebSocket auth token via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start per language | gtm-customer-success |
| Sample workflow: build a 50-line "post-on-event" bot in TS / Python | axis-social |
| Sample workflow: build a 100-line feed widget in React / Flutter | axis-social |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-social |
| EU AI Act Art. 50 transparency-label SDK helper (required for tenants surfacing classifier verdicts) | axis-social + council-privacy |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice |
| Breaking API change | per-release | major version bump; backwards-compatible adapter for 1 prior major |

## Versioning

- social µservice version: semver.
- SDK version per language matches social major.minor; SDK patch independent.
- Compatibility matrix: published per-language; CI lane verifies SDK N+1 ↔ social N-1..N+1.

## Bot / Webhook surface

Beyond per-language SDKs, social ships:
- Twitter-API-compatible incoming-webhook URL shape (for migration ease from Twitter / X bots).
- Outgoing webhook signing (HMAC-SHA256 over body + timestamp; replay-resistant).
- Bot API: REST endpoint subset (publish-post + react + read-feed + follow); rate-limited; per-bot OpenBao token.
- ActivityPub-compatible inbox endpoint (Professional-tier only) for federation peers.

## EU AI Act SDK Helpers (mandatory for EU tenants)

- `formatModerationVerdictLabel(verdict)`: returns localised "AI-assessed" label.
- `getRankingExplanation(post_id)`: returns ranking signals contributing to current placement (Art. 27 transparency).
- `openAppealUI(verdict_id)`: launches in-app appeal flow.

## Open-Source Decision

Defer per-SDK open-source decision until 6mo stability. Default: closed until tenant or competitive pressure shifts. Open-source-when-stable matches Stripe + Twilio + Slack-Bolt precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK N+1 works against social versions N-1..N+1.
- Annual SDK telemetry review: usage per SDK; underused flagged for sunset.

## References

- `microservices/social/contracts/openapi/social.yaml`.
- `microservices/social/contracts/asyncapi/social-events.yaml`.
- `microservices/social/contracts/proto/social.proto`.
- ADR-0105 (13-layer enum; `sdk` is one canonical layer).
- Twitter / X API precedent `developer.x.com`.
- Mastodon API `docs.joinmastodon.org/api/`.
- Bluesky AT Protocol `docs.bsky.app`.
- ActivityPub spec W3C Rec 2018 `www.w3.org/TR/activitypub/`.
- RFC 9421 HTTP Signatures.
- EU AI Act Arts. 13 + 27 + 50.
