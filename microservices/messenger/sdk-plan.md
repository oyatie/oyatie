---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-messenger + gtm-customer-success
deciders: axis-messenger, council-architecture
related_adrs: [ADR-0135, ADR-0131, ADR-0132]
related_artifacts:
  - microservices/messenger/contracts/openapi/messenger.yaml
  - microservices/messenger/contracts/asyncapi/messenger-events.yaml
  - microservices/messenger/contracts/proto/messenger.proto
  - microservices/messenger/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (messenger µservice)

## Purpose

Tenants embed messenger functionality in their own surfaces (mobile / desktop
/ embedded chat / bot APIs) via first-party SDKs. Per Slack, Microsoft Teams,
Discord, Matrix, Zulip precedent: each ships per-language SDKs that handle
auth, WebSocket reconnection, encryption (where applicable), event
deserialisation, and ergonomic patterns idiomatic to each language.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M02 (primary; oyatie's own language) | First-party authored (`oya-messenger-*-sdk` crates per BC) | axis-messenger |
| **TypeScript + Browser SDK** | M02 (web-client + ts-server) | OpenAPI + AsyncAPI generation + first-party ergonomic + WebRTC peer-conn helpers | axis-messenger + gtm |
| **Kotlin (Android) + Swift (iOS)** | M02-onward1 | gRPC-generated + first-party ergonomic + native WebSocket + WebRTC bindings | axis-messenger + gtm |
| **Python** | M03 | OpenAPI-generated + ergonomic | axis-messenger + gtm |
| **Go** | M03 | gRPC-generated + ergonomic | axis-messenger + gtm |
| **JVM (Java/Kotlin)** | M03-onward1 | gRPC-generated + ergonomic; published to Maven Central | axis-messenger + gtm |
| **C# / .NET** | M04-onward | OpenAPI-generated + ergonomic; published to NuGet | axis-messenger + gtm |
| **C++ (embedded / desktop)** | M05-onward (only if tenant demand surfaces) | gRPC-generated | axis-messenger |
| **Dart (Flutter)** | M03 | OpenAPI-generated + WebRTC wrapper | axis-messenger + gtm |

Driver: mobile clients lead because most messenger usage is mobile. Web SDK
ties to Workflow Studio shell. Server-language SDKs follow.

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/messenger/src/crates/oya-messenger-*-sdk/`.

- `Client::new(opts) -> Client; client.post_message(...) -> Result<Message>`.
- WebSocket subscription: `client.subscribe_channel(channel_id) -> impl Stream<Item=WsFrame>`.
- Reconnection: built-in exponential backoff + jitter; resume from last seen `seq`.
- E2E (Personal-context DMs): client-side MLS (RFC 9420) key management;
  body encrypted before send; never plaintext server-side.
- WebRTC helpers: re-exports `livekit-client` for huddles.

### TypeScript + Browser SDK

`@oyatie/messenger-sdk` (npm). Sub-packages:
- `@oyatie/messenger-sdk` — node + browser core.
- `@oyatie/messenger-sdk-react` — React hooks (`useChannel`, `usePresence`).
- `@oyatie/messenger-sdk-webrtc` — LiveKit + RFC 8825 wrappers.

### Mobile SDKs (iOS + Android)

- iOS: Swift Package via SwiftPM; binary XCFramework; supports iOS 15+.
- Android: Maven Central + Gradle; AAR; minSDK 26 (Android 8.0).
- Both bind to OS-native WebSocket + WebRTC stacks (WKWebView / WebView).

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| List channels | `listChannels(opts)` | `Channel[]` (paginated) |
| Create channel | `createChannel(req)` | `Channel` |
| Post message | `postMessage(channelId, body, opts)` | `Message` |
| Edit message | `editMessage(channelId, messageId, body)` | `Message` |
| Delete message | `deleteMessage(channelId, messageId)` | `void` |
| React to message | `addReaction(channelId, messageId, emoji)` | `void` |
| Search messages | `search(q, opts)` | `SearchResult` |
| Update presence | `updatePresence(state, custom?)` | `Presence` |
| Subscribe channel | `subscribe(channelId) -> AsyncIterable<WsFrame>` | streaming |
| Start huddle | `startHuddle(channelId)` | `HuddleSession` |
| Upload attachment | `uploadAttachment(channelId, blob)` | `Attachment` (multipart) |

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue messenger API key + WebSocket auth token via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start per language | gtm-customer-success |
| Sample workflow: build a 50-line "post-message-on-event" bot in TS / Python | axis-messenger |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-messenger |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice |
| Breaking API change | per-release | major version bump; backwards-compatible adapter for 1 prior major |

## Versioning

- messenger µservice version: semver.
- SDK version per language matches messenger major.minor; SDK patch independent.
- Compatibility matrix: published per-language; CI lane verifies SDK N+1 ↔ messenger N-1..N+1.

## E2E (Personal-context DM) considerations

- E2E SDK ships MLS (RFC 9420) client library bindings (web: `mlspp.js`; native: `mls-rs`).
- Key management: per-device key bundle + key escrow optional (council-privacy
  decision pending; see PRD Open Question 5).
- Server never sees plaintext; SDK enforces ciphertext-only marker `body_client_only: ()`.

## Bot / Webhook surface

Beyond per-language SDKs, messenger ships:
- Slack-compatible Incoming Webhook URL shape (for migration ease).
- Outgoing webhook signing (HMAC-SHA256 over body + timestamp; replay-resistant).
- Bot API: REST endpoint subset (post-message + react + read-membership);
  rate-limited; per-bot OpenBao token.

## Open-Source Decision

Defer per-SDK open-source decision until 6mo stability. Default: closed
until tenant or competitive pressure shifts. Open-source-when-stable matches
Stripe + Twilio + Slack-Bolt precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK N+1 works against messenger versions N-1..N+1.
- Annual SDK telemetry review: usage per SDK; underused flagged for sunset.

## References

- `microservices/messenger/contracts/openapi/messenger.yaml`.
- `microservices/messenger/contracts/asyncapi/messenger-events.yaml`.
- `microservices/messenger/contracts/proto/messenger.proto`.
- ADR-0105 (13-layer enum; `sdk` is one canonical layer).
- Slack SDK precedent `api.slack.com/start/building/bolt-js`.
- Matrix Client-Server API `spec.matrix.org`.
- Discord Bot API `discord.com/developers/docs`.
- LiveKit client SDKs `docs.livekit.io/realtime/client/`.
- MLS protocol RFC 9420.
- RFC 6455 (WebSocket); RFC 8825 (WebRTC); RFC 8866 (SDP); RFC 8445 (ICE);
  RFC 5766 (TURN); RFC 5389 (STUN).
