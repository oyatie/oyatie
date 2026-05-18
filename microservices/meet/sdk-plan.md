---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-meet + gtm-customer-success
deciders: axis-meet, council-architecture
related_adrs: [ADR-0126, ADR-0131, ADR-0132]
related_artifacts:
  - microservices/meet/contracts/openapi/meet.yaml
  - microservices/meet/contracts/asyncapi/meet-events.yaml
  - microservices/meet/contracts/proto/meet.proto
  - microservices/meet/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (meet µservice)

## Purpose

Tenants embed meet functionality in their own surfaces (mobile / desktop / embedded video / bot APIs) via first-party SDKs. Per Zoom Video SDK, Google Meet SDK (limited), Microsoft Teams JS / Bot SDK, Webex Embedded App, Daily.co, Whereby Embedded, LiveKit Client SDKs precedent: each ships per-language SDKs that handle auth, WebSocket signaling reconnection, WebRTC peer connection lifecycle, recording-consent UX, encryption (where applicable), and ergonomic patterns idiomatic to each language.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M02 (primary; oyatie's own language) | First-party authored (`oya-meet-*-sdk` crates per BC) | axis-meet |
| **TypeScript + Browser SDK** | M02 (web-client + ts-server) | OpenAPI + AsyncAPI generation + first-party ergonomic + WebRTC peer-conn helpers + LiveKit-client re-export | axis-meet + gtm |
| **Kotlin (Android) + Swift (iOS)** | M02+1 | gRPC-generated + first-party ergonomic + native WebSocket + WebRTC bindings + LiveKit native SDKs | axis-meet + gtm |
| **Python** | M03 | OpenAPI-generated + ergonomic (bot-only; no native WebRTC) | axis-meet + gtm |
| **Go** | M03 | gRPC-generated + ergonomic (bot-only) | axis-meet + gtm |
| **JVM (Java/Kotlin)** | M03+1 | gRPC-generated + ergonomic; published to Maven Central | axis-meet + gtm |
| **C# / .NET** | M04+ | OpenAPI-generated + ergonomic; published to NuGet | axis-meet + gtm |
| **C++ (embedded / desktop)** | M05+ (only if tenant demand surfaces) | gRPC-generated | axis-meet |
| **Dart (Flutter)** | M03 | OpenAPI-generated + WebRTC wrapper + LiveKit Flutter SDK | axis-meet + gtm |

Driver: mobile clients lead because video usage is heavily mobile. Web SDK ties to Workflow Studio shell + standalone meet web client. Server-language SDKs follow for bot scenarios (e.g., recording-bot that joins meetings programmatically).

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/meet/src/crates/oya-meet-*-sdk/`.

- `Client::new(opts) -> Client; client.create_meeting_room(...) -> Result<MeetingRoom>`.
- WebSocket subscription: `client.subscribe_meeting(instance_id) -> impl Stream<Item=MeetingFrame>`.
- WebRTC peer-connection: re-exports `livekit-client` (Rust client SDK).
- Recording-consent UX helper: built-in modal-banner emission for consent acknowledgement.
- E2E (opt-in tier): client-side MLS (RFC 9420) + Insertable Streams helper.

### TypeScript + Browser SDK

`@oyatie/meet-sdk` (npm). Sub-packages:
- `@oyatie/meet-sdk` — node + browser core.
- `@oyatie/meet-sdk-react` — React hooks (`useMeeting`, `useParticipants`, `useRecording`).
- `@oyatie/meet-sdk-webrtc` — LiveKit Browser SDK + RFC 8825 wrappers.
- `@oyatie/meet-sdk-livecaption` — WebSocket live-caption stream consumer.

### Mobile SDKs (iOS + Android)

- iOS: Swift Package via SwiftPM; binary XCFramework; supports iOS 15+; embeds LiveKit iOS SDK + WebRTC framework.
- Android: Maven Central + Gradle; AAR; minSDK 26 (Android 8.0); embeds LiveKit Android SDK + native WebRTC.
- Both bind to OS-native screen-share (ReplayKit / MediaProjection) and camera/mic permissions.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| List meeting rooms | `listMeetingRooms(opts)` | `MeetingRoom[]` (paginated) |
| Create meeting room | `createMeetingRoom(req)` | `MeetingRoom` |
| Start instance | `startMeetingInstance(roomId, opts)` | `MeetingInstance` |
| Join (host or guest) | `joinMeeting(instanceId, joinToken)` | `JoinSession` (publishes media + signals) |
| Subscribe to lobby | `subscribeLobby(instanceId) -> AsyncIterable<LobbyEvent>` | streaming |
| Approve / deny lobby | `approveLobbyMember(instanceId, userRef)` | `void` |
| Start / stop recording | `startRecording(instanceId)` / `stopRecording(...)` | `RecordingHandle` |
| Subscribe live captions | `subscribeCaptions(instanceId, language) -> AsyncIterable<Caption>` | streaming |
| Start screen-share | `startScreenShare(opts)` | `ScreenShareHandle` |
| Grant remote-control | `grantRemoteControl(participantRef)` | `void` |
| Create breakout rooms | `createBreakoutRooms(instanceId, config)` | `BreakoutRoom[]` |
| Send / receive chat-in-meeting | `sendChat(instanceId, body)` / `subscribeChat(...)` | streaming |
| Run poll | `createPoll(instanceId, opts)` | `Poll` |
| Submit Q&A question | `submitQuestion(instanceId, body)` | `Question` |
| Start live-stream egress | `startLiveStreamEgress(instanceId, destination)` | `EgressHandle` |
| Retrieve recording + transcript + summary | `getRecording(recordingId)` / `getTranscript(...)` / `getSummary(...)` | `Recording` / `Transcript` / `Summary` |
| Update presence | `updatePresence(state)` | `Presence` |
| E2E mode init | `initMlsGroup(instanceId, keyPackages)` | `MlsGroupHandle` (opt-in) |

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue meet API key + LiveKit-token-issuance API key via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start per language | gtm-customer-success |
| Sample workflow: build a 50-line "recording-bot that auto-joins scheduled meetings" in TS / Python | axis-meet |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-meet |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice |
| LiveKit upstream SDK upgrade with breaking change | upstream-coupled | follow LiveKit upstream + 3mo migration window |
| Breaking API change | per-release | major version bump; backwards-compatible adapter for 1 prior major |

## Versioning

- meet µservice version: semver.
- SDK version per language matches meet major.minor; SDK patch independent.
- LiveKit SDK version coupling: pinned LTS (1.6.2); upgrade IP quarterly.
- Compatibility matrix: published per-language; CI lane verifies SDK N+1 ↔ meet N-1..N+1.

## E2E (opt-in) considerations

- E2E SDK ships MLS (RFC 9420) client library bindings (web: `mlspp.js`; native: `mls-rs`).
- W3C Insertable Streams API for per-frame encryption (`SFrame` for media-frame encryption).
- Key management: per-device key bundle + key escrow optional (council-privacy decision pending; mirrors ADR-MSGR-0002 tier-split).
- Server never sees plaintext; SDK enforces ciphertext-only marker `media_client_only: ()`.
- Recording + transcription + AI summary structurally disabled when E2E mode active (Cedar deny).

## Bot / Webhook surface

Beyond per-language SDKs, meet ships:
- Outgoing webhook signing (HMAC-SHA256 over body + timestamp; replay-resistant).
- Bot API: REST endpoint subset (create-meeting + start-recording + retrieve-transcript); rate-limited; per-bot OpenBao token.
- Recording-Bot SDK helper: programmatic participant that joins a meeting solely to drive recording (useful for compliance archival, transcription augmentation).

## Open-Source Decision

Defer per-SDK open-source decision until 6mo stability. Default: closed until tenant or competitive pressure shifts. Open-source-when-stable matches Stripe + Twilio + LiveKit (already OSS) precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK N+1 works against meet versions N-1..N+1.
- WebRTC interop test: SDK joins a LiveKit-hosted room with media flow; setup p95 ≤ 1.5s.
- Annual SDK telemetry review: usage per SDK; underused flagged for sunset.

## References

- `microservices/meet/contracts/openapi/meet.yaml`.
- `microservices/meet/contracts/asyncapi/meet-events.yaml`.
- `microservices/meet/contracts/proto/meet.proto`.
- ADR-0105 (13-layer enum; `sdk` is one canonical layer).
- Zoom Video SDK `developers.zoom.us/docs/video-sdk/`.
- Microsoft Teams JavaScript Client SDK `learn.microsoft.com/microsoftteams/platform/`.
- Google Meet SDK (limited; Live Sharing) `developers.google.com/meet/`.
- Webex Embedded App `developer.webex.com/docs/embedded-apps`.
- LiveKit Client SDKs `docs.livekit.io/realtime/client/`.
- Daily.co client SDKs `docs.daily.co/reference`.
- W3C WebRTC + Insertable Streams.
- MLS protocol RFC 9420.
- RFC 6455 (WebSocket); RFC 8825 (WebRTC); RFC 8866 (SDP); RFC 8445 (ICE); RFC 5766 (TURN); RFC 5389 (STUN).
- WHIP/WHEP IETF drafts.
