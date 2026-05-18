---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: network
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-network + gtm-customer-success
deciders: axis-network, council-architecture
related_adrs: [ADR-0126, ADR-0131, ADR-0132]
related_artifacts:
  - microservices/network/contracts/openapi/network.yaml
  - microservices/network/contracts/asyncapi/network-events.yaml
  - microservices/network/contracts/proto/network.proto
  - microservices/network/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (network µservice)

## Purpose

Tenants embed `network` functionality in their own surfaces (B2B portals / mobile / desktop / embedded Professional widget / HRIS integration / ATS adapters / Workflow Studio templates) via first-party SDKs. Per LinkedIn API + Xing + Wantedly + Hashnode + Hubspot precedent: each ships per-language SDKs that handle auth, WebSocket reconnection, event deserialisation, multipart media-upload, profile-export, and ergonomic patterns idiomatic to each language. The `network` SDK suite additionally ships per-language helpers for EU AI Act Art. 50 transparency, NYC LL144 candidate-notice formatting, GDPR Art. 22 opt-out surfaces, and vCard 4.0 (RFC 6350) + JSON Resume portable-export emission.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M02 (primary; oyatie's own language) | First-party authored (`oya-network-*-sdk` crates per BC) | axis-network |
| **TypeScript + Browser SDK** | M02 (web-client + ts-server) | OpenAPI + AsyncAPI generation + first-party ergonomic + WebSocket reconnect | axis-network + gtm |
| **Kotlin (Android) + Swift (iOS)** | M02+1 | gRPC-generated + first-party ergonomic + native WebSocket + multipart-upload | axis-network + gtm |
| **Python** | M03 | OpenAPI-generated + ergonomic (most-requested for HR/people-analytics tooling) | axis-network + gtm |
| **Go** | M03 | gRPC-generated + ergonomic | axis-network + gtm |
| **JVM (Java/Kotlin)** | M03+1 | gRPC-generated + ergonomic; published to Maven Central; ATS-integration sample | axis-network + gtm |
| **C# / .NET** | M04+ | OpenAPI-generated + ergonomic; published to NuGet; HRIS-integration sample | axis-network + gtm |
| **Dart (Flutter)** | M03 | OpenAPI-generated + multipart-upload helpers | axis-network + gtm |

Driver: enterprise HR + ATS + HRIS integrations skew Java + .NET + Python; mobile Professional network clients skew iOS + Android (per Microsoft + LinkedIn Engineering precedent). Web SDK ties to Workflow Studio shell. Server-language SDKs follow.

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/network/src/crates/oya-network-*-sdk/`.

- `Client::new(opts) -> Client; client.publish_post(...) -> Result<Post>`.
- WebSocket subscription: `client.subscribe_feed() -> impl Stream<Item=WsFrame>`.
- Reconnection: built-in exponential backoff + jitter; resume from last seen `seq`.
- Media + document upload: `client.upload_media(file, opts) -> Result<MediaRef>` with multipart progress.
- Context awareness: `ContextKind::Professional` is hard-coded for `network`; cross-context calls refused at compile-time.
- Endorsement signing: SDK provides `endorse(skill_id, signing_key) -> Result<Endorsement>`; per-endorser Ed25519 (ADR-NET-0005).

### TypeScript + Browser SDK

`@oyatie/network-sdk` (npm). Sub-packages:
- `@oyatie/network-sdk` — node + browser core.
- `@oyatie/network-sdk-react` — React hooks (`useProfile`, `useConnectionGraph`, `useFeed`, `useEndorse`, `useInMail`).
- `@oyatie/network-sdk-media` — multipart media + document upload + progress helpers.
- `@oyatie/network-sdk-export` — vCard 4.0 + JSON Resume + GDPR-Art-20 portable-export client.

### Mobile SDKs (iOS + Android)

- iOS: Swift Package via SwiftPM; binary XCFramework; supports iOS 15+.
- Android: Maven Central + Gradle; AAR; minSDK 26 (Android 8.0).
- Both bind to OS-native WebSocket + multipart-upload stacks.
- Push notification: APNs / FCM bound to per-device token; oyatie does not store device-tokens long-term beyond hash.
- iOS: ASWebAuthenticationSession for OIDC redirect; Android: Custom Tabs.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Get my profile | `getMyProfile()` | `Profile` |
| Update my profile | `updateMyProfile(req)` | `Profile` |
| Get profile by handle | `getProfile(handle)` | `Profile` |
| Export profile (vCard 4.0 / JSON Resume / GDPR Art. 20 JSON) | `exportProfile(format)` | `Bytes` |
| Send connection request | `sendConnectionRequest(target, note?)` | `ConnectionRequestId` |
| Accept / reject / ignore connection request | `respondToRequest(id, verdict)` | `void` |
| Follow / unfollow (asymmetric) | `follow(target)` / `unfollow(target)` | `void` |
| Block / restrict / disconnect | `block(target)` / `restrict(target)` / `disconnect(target)` | `void` |
| Get connection degree | `getDegreeOfSeparation(target)` | `1 \| 2 \| 3 \| Out` |
| Publish post (article / status / document / poll / carousel) | `publishPost(req)` | `Post` |
| Repost / share / quote-post | `repost(post_id)` / `quotePost(post_id, body)` | `Post` |
| Edit post (within edit-window) | `editPost(post_id, body)` | `Post` |
| Delete post | `deletePost(post_id)` | `void` (tombstone) |
| React | `addReaction(post_id, kind)` / `removeReaction(post_id, kind)` | `void` |
| Comment | `comment(post_id, body)` | `Post` |
| Render feed (chronological / algorithmic) | `renderFeed(opts)` | `Feed` |
| Subscribe real-time feed | `subscribeFeed() -> AsyncIterable<WsFrame>` | streaming |
| Search people | `searchPeople(q, opts)` | `Profile[]` |
| Search content | `searchContent(q, opts)` | `Post[]` |
| Search skills / jobs / companies / events | `searchSkills(q)` / `searchJobs(q)` / `searchCompanies(q)` / `searchEvents(q)` | typed result sets |
| Send InMail (Professional-tier-only bridge to messenger) | `sendInMail(target, subject, body)` | `InMailId` |
| Endorse skill (Ed25519 signed) | `endorseSkill(connection_id, skill_id)` | `EndorsementId` |
| Revoke endorsement | `revokeEndorsement(endorsement_id)` | `void` |
| Publish recommendation | `publishRecommendation(connection_id, body)` | `RecommendationId` |
| Take skill assessment | `submitSkillAssessment(skill_id, attempt)` | `AssessmentResult` |
| Request profile verification | `requestVerification(method)` | `VerificationRequestId` |
| Create / join / leave group | `createGroup`, `joinGroup`, `leaveGroup` | `Group` / `void` |
| Create / RSVP event | `createEvent(...)`, `rsvpEvent(...)` | `Event` / `RSVP` |
| Follow Page | `followPage(page_id)` | `void` |
| Publish job posting (handoff to ATS) | `publishJobPosting(req)` | `JobPostingId` |
| Get notifications | `getNotifications(opts)` | `Notification[]` |
| Report abuse / harassment / 직장 갑질 | `reportAbuse(post_id, category)` | `AbuseReportId` |
| Open appeal | `openAppeal(verdict_id, reason)` | `AppealId` |
| Opt out of high-risk automated decisions (GDPR Art. 22) | `setAutomatedDecisionPreference(opt_out: bool)` | `void` |

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue network API key + WebSocket auth token via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start per language | gtm-customer-success |
| Sample workflow: build a 50-line "post-on-employee-onboarding" bot in TS / Python | axis-network |
| Sample workflow: build a 100-line Professional profile widget in React / Flutter | axis-network |
| Sample workflow: build a 200-line ATS-integration adapter in Java / .NET | axis-network |
| Sample workflow: HRIS bulk-profile-sync (Workday / SAP SuccessFactors / BambooHR pattern) | axis-network |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-network |
| EU AI Act Art. 50 transparency-label SDK helper (required for tenants surfacing classifier verdicts) | axis-network + council-privacy |
| NYC LL144 §20-871 candidate-notice SDK helper (required for NYC tenants using recruiter-stub) | axis-network + ops-compliance |
| GDPR Art. 22 opt-out UI helper | axis-network + council-privacy |

## EU AI Act SDK Helpers (mandatory for EU tenants)

- `formatHighRiskDecisionLabel(decision)`: returns localised "AI-assessed decision (high-risk; employment context)" label per Annex III §4.
- `getRankerExplanation(post_id|candidate_ref)`: returns ranking signals contributing to current placement (Art. 27 + Art. 50 transparency).
- `getRecruiterDecisionExplanation(candidate_ref)`: returns recruiter-stub ranker contributing signals + bias-audit summary URL.
- `openAppealUI(decision_id)`: launches in-app appeal flow (Art. 14 human oversight surface).
- `getHumanReviewOption(decision_id)`: surfaces GDPR Art. 22 right-to-human-review entry point.

## NYC LL144 + CA AB-331 + CO SB 24-205 Helpers (employment-context, US)

- `formatLL144CandidateNotice(tenant_id, locale)`: returns localised candidate-notice text + bias-audit-summary URL per NYC Admin Code §20-871.
- `getLL144BiasAuditSummary(tenant_id)`: returns latest annual bias-audit summary URL (DCWP-compliant format).
- `formatAB331ConsumerNotice(tenant_id)`: per CA AB-331 §22756.3.
- `formatCOSB205Notice(tenant_id)`: per CO SB 24-205.

## Profile-Export Helpers (GDPR Art. 20)

- `exportProfileVCard()`: emits RFC 6350 vCard 4.0.
- `exportProfileJsonResume()`: emits open JSON Resume schema.
- `exportProfileGdprArt20()`: emits GDPR Art. 20 portable-JSON bundle (includes connection-graph references, endorsement-chain references, post references; no media blobs — separate signed-URL list).

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice |
| Breaking API change | per-release | major version bump; backwards-compatible adapter for 1 prior major |

## Versioning

- network µservice version: semver.
- SDK version per language matches network major.minor; SDK patch independent.
- Compatibility matrix: published per-language; CI lane verifies SDK N+1 ↔ network N-1..N+1.

## Bot / Webhook Surface

Beyond per-language SDKs, network ships:
- LinkedIn-API-compatible incoming-webhook URL shape (for migration ease from LinkedIn-integration tools).
- Outgoing webhook signing (HMAC-SHA256 over body + timestamp; replay-resistant).
- Bot API: REST endpoint subset (publish-post + react + read-feed + connect + endorse); rate-limited; per-bot OpenBao token.
- ATS-integration webhook contract (signed; idempotent; replay-safe) for jobs-handoff.
- HRIS-integration webhook contract (signed) for profile bulk-sync.

## Open-Source Decision

Defer per-SDK open-source decision until 6mo stability. Default: closed until tenant or competitive pressure shifts. Open-source-when-stable matches Stripe + Twilio + Slack-Bolt precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK N+1 works against network versions N-1..N+1.
- Per-SDK NYC LL144 / EU AI Act / GDPR Art. 22 helper conformance test.
- Annual SDK telemetry review: usage per SDK; underused flagged for sunset.

## References

- `microservices/network/contracts/openapi/network.yaml`.
- `microservices/network/contracts/asyncapi/network-events.yaml`.
- `microservices/network/contracts/proto/network.proto`.
- `microservices/social/sdk-plan.md` (sibling reference).
- ADR-0105 (13-layer enum; `sdk` is one canonical layer).
- LinkedIn API precedent `learn.microsoft.com/linkedin`.
- Xing API `dev.xing.com`.
- JSON Resume open schema `jsonresume.org`.
- vCard 4.0 RFC 6350.
- EU AI Act Arts. 13 + 27 + 50; GDPR Art. 22 + Art. 20.
- NYC Admin Code §§20-870, 20-871, 20-872; CA AB-331; CO SB 24-205.
