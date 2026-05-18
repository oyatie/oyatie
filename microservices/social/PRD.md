---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-social
microservice: social
status: Accepted
sales_segment: connect-suite-product
tier: hero-product
milestone_first_ship: M02-foundation
bominal_source: [ADR-0208-connect-dual-context-unified-channel-hub.md]
related_adrs: [ADR-0008, ADR-0056, ADR-0105, ADR-0106, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/microservices/social.json, /specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-social
doc_status: published
---

# PRD-social: First-Party Social Platform (Profiles + Posts + Feed + Follow-Graph + Reactions + Notifications)

## Purpose

The `social` microservice is oyatie's native Twitter/X-class first-party social platform. Per parallel-session ADR-0135 (Connect dissolution), it is one of the 8 first-class µservices factored out of the legacy Connect umbrella. It owns **user-profile + follow-graph + chronological-and-algorithmic feed + post-composition + reactions + comments + reposts/quote-posts + mentions + hashtags + trending-topics + content-discovery + content-moderation + blocking + muting + lists + bookmarks + people-and-content-search + real-time-and-digest notifications + cross-context dual-pillar isolation (Personal B2C vs Professional B2B) + optional ActivityPub federation + content-moderation classifier + ranking model + abuse-reporting + appeal-workflow + age-verification + accessibility-alt-text + ads-substrate-stub (T2 capability, off by default)** across the 11 oyatie regulatory packs.

This µservice is **a hero product**, end-user-facing through Workflow Studio shell and standalone social clients (web + desktop + mobile). It is also consumable as a shared substrate by other oyatie products via the `social.post.v1` Workflow events and the `Person`, `Post`, `Topic` Ontology object types.

Bominal predecessor: the `connect-social` slice of Bominal's unified Connect suite. Per parallel ADR-0135, that monolithic suite is dissolved into per-surface µservices; this PRD is the canonical social landing in oyatie. **social is NET-NEW** — no `oya-connect-social-*` crates exist; there is no migration-from-connect.md or deprecation-notice.md.

## Tenant Value

- **Tenant Outcome 1 — Native social presence without identity fragmentation.** Tenants and their end-users get Twitter/X-class profile + follow-graph + feed UX inside the same shell as mail, messenger, calendar, workflow studio — switching personal/professional context without leaving the surface.
- **Tenant Outcome 2 — Dual-context-safe social.** Personal (B2C) profiles never cross into professional (B2B) audit scope per parallel ADR-0135; Professional posts carry tenant-DEK encryption when configured and four-eyes audit disclosure inherited from Bominal ADR-0215. Personal-tier never federates.
- **Tenant Outcome 3 — Real-time feed + notification delivery.** Feed-render p95 ≤ 200ms (top 50 posts); notification-fanout p99 ≤ 2s for 10k-follower accounts; post-create p95 ≤ 100ms.
- **Tenant Outcome 4 — Moderation that is auditable.** Every moderation verdict + appeal-action emits an audit-chain record (Merkle / Ed25519); EU AI Act high-risk classification for content-moderation + ranking model carries transparency obligations per Art. 50.
- **Tenant Outcome 5 — Cross-product mention-and-link integration.** Posts cross-link to messenger DMs (deep-link to-messenger bridge), community discussions, ontology entities, and workflow events natively; competitors expose webhooks only.
- **Tenant Outcome 6 — Multi-pack residency by design.** 11 region-pinned packs; Personal-tier residency follows the user; Professional-tier follows the tenant.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | end-user | to create a profile (handle, display-name, bio, avatar, header) | I have a social identity | user-profile | Must |
| FR-02 | end-user | to follow / unfollow / mute / block another user | I curate my feed | follow-graph | Must |
| FR-03 | end-user | to publish a post (text ≤ 4096 chars + media + link-preview) | I broadcast to my followers | post-composition | Must |
| FR-04 | end-user | to repost / quote-post another post | I amplify or commentary-add | post-composition | Must |
| FR-05 | end-user | to comment on a post / reply in a thread | conversation depth is preserved | post-composition | Must |
| FR-06 | end-user | to react inline (like / laugh / wow / etc) | low-overhead acknowledgement | reactions | Must |
| FR-07 | end-user | to see a chronological + an algorithmic feed | I choose how I consume | feed-timeline | Must |
| FR-08 | end-user | to @mention people, teams, channels, hashtags | recipients are notified + linked | mentions | Must |
| FR-09 | end-user | to use #hashtags for discoverability | content is grouped by topic | hashtags | Must |
| FR-10 | end-user | to see trending topics within my tenant + pack | discovery surfaces broader context | trending-topics | Must |
| FR-11 | end-user | to search for people and content I can read | I recover and discover context | search | Must |
| FR-12 | end-user | to bookmark a post | I save for later | bookmarks | Must |
| FR-13 | end-user | to create / curate lists of users | I segment my feed | lists | Must |
| FR-14 | end-user | to report abuse / spam / impersonation | community safety is maintained | abuse-reporting | Must |
| FR-15 | end-user | to appeal a moderation verdict | due-process is honoured | appeal-workflow | Must |
| FR-16 | end-user | to receive real-time + digest notifications | I stay informed | notifications | Must |
| FR-17 | end-user | to share a post deep-link into messenger DM | cross-product flow works | post-composition + messenger bridge | Must |
| FR-18 | end-user | to set per-post visibility (public, followers, list, private) | I control scope | post-composition | Must |
| FR-19 | end-user | to add alt-text to media for accessibility | screen-reader users can perceive | post-composition | Must |
| FR-20 | end-user | to flag content with content-warnings (sensitive media, spoiler) | viewers can opt-in to see | post-composition | Must |
| FR-21 | end-user | to switch personal/professional persona | dual-context isolation preserved | user-profile | Must |
| FR-22 | tenant-admin | to configure pack-aware retention + moderation policy | regulatory bounds hold | user-profile + content-moderation | Must |
| FR-23 | compliance-officer | to issue eDiscovery hold on professional posts | regulatory request is satisfied | post-composition + audit-chain | Must |
| FR-24 | tenant-admin | to verify a profile (blue-check equivalent) | authenticity signals exist | profile-verification | Must |
| FR-25 | Workflow Studio | to consume `PostPublished` / `MentionEmitted` / `ReactionAdded` events | downstream automation works | post-composition + reactions | Must |
| FR-26 | messenger µservice | to resolve user @-handles via social mention-resolution | identity registry is shared | user-profile | Must |
| FR-27 | tenant-operator | to query profile + feed + moderation metrics | I plan capacity + verify SLAs | observability | Must |
| FR-28 | end-user (per pack regulation) | to attest age at signup | age-gate is enforced | age-verification | Must |
| FR-29 | end-user | to opt-in their tenant to ActivityPub federation (Professional only) | external interop where wanted | federation-gateway | Should |
| FR-30 | tenant-admin | to disable ads-substrate-stub T2 capability (off by default) | tenants choose monetisation | ads-substrate-stub | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | p999 | Notes |
|---|---|---|---|---|---|
| Feed-render latency (top 50 posts) | ≤ 60ms | ≤ 200ms | ≤ 400ms | ≤ 1s | Redis hot-feed cache; warm hit on read |
| Profile-render latency | ≤ 40ms | ≤ 150ms | ≤ 350ms | ≤ 800ms | Postgres + Redis cache |
| Post-create latency (post → fanout-ack) | ≤ 30ms | ≤ 100ms | ≤ 250ms | ≤ 700ms | Postgres insert + async fanout |
| Follow-action latency | ≤ 20ms | ≤ 50ms | ≤ 150ms | ≤ 400ms | Postgres adjacency-list write |
| Search-people query (≤ 25 results) | ≤ 80ms | ≤ 300ms | ≤ 600ms | ≤ 1.2s | Meilisearch + Cedar filter |
| Search-content query (≤ 25 results) | ≤ 150ms | ≤ 500ms | ≤ 1s | ≤ 2s | Meilisearch + Cedar filter |
| Notification fanout (10k followers) | ≤ 200ms | ≤ 1s | ≤ 2s | ≤ 5s | per-recipient async via Redis Streams |
| Notification fanout (100k followers) | ≤ 500ms | ≤ 2s | ≤ 5s | ≤ 15s | sharded fanout workers |
| Comment / reply create | ≤ 30ms | ≤ 100ms | ≤ 250ms | ≤ 700ms | Postgres insert |
| Reaction add | ≤ 15ms | ≤ 50ms | ≤ 120ms | ≤ 300ms | Redis-buffered + Postgres flush |
| Trending-topic compute | n/a | n/a | n/a | n/a | batched 5min windowed |
| Media transcode (image, ≤ 10MB) | ≤ 800ms | ≤ 2s | ≤ 4s | ≤ 10s | ImageMagick |
| Media transcode (video, ≤ 200MB) | ≤ 30s | ≤ 90s | ≤ 180s | ≤ 300s | ffmpeg HLS segmentation |

### Security

- Profile + post + follow-graph reads enforced server-side via Cedar policy (`policy/tenant-scope.cedar` + `policy/public-read.cedar`); client never trusted.
- Personal-tier profiles + posts: tenant operators + oyatie operators MUST NOT have plaintext disclosure access (inherited from Bominal ADR-0208).
- Professional-tier posts tenant-DEK encrypted (envelope encryption per Bominal ADR-0111); admin disclosure requires four-eyes audit trail per Bominal ADR-0215.
- Media uploads scanned via OPSWAT MetaDefender or ClamAV before publication; quarantine bucket pattern (`runbooks/media-malware-quarantine.md` referenced in attachment lifecycle; same shape as messenger).
- All WebSocket connections mTLS-terminated; per-tenant API token bound at OpenBao with rotation 30d.
- Search index excludes redacted PII / PHI per `policy/redaction-phi.md` (pack-us-healthcare overlay; same pattern as messenger).
- Cross-context routing forbidden: a Personal post cannot become a Professional post; enforced by `policy/dual-context-isolation.md`.
- Federation egress (ActivityPub): only Professional-tier with tenant opt-in; Personal-tier NEVER federates.

### Audit + Compliance

- Every post-create / post-delete / moderation-verdict / appeal-action / four-eyes-disclosure / hold event writes an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028).
- Professional-tier disclosure (admin reads PII or quote-original on a moderated post) requires two distinct approving principals + reason code (per Bominal ADR-0215).
- Retention: per-pack bounds in `policy/data-residency.md`. KR PIPA work-record floor satisfied. GDPR storage-limitation honored. HIPAA pack: PHI retention 6y where applicable.
- DSA Art. 23 (EU): every moderation action emits to a per-tenant transparency log; tenant publishes per Art. 24 obligations.
- EU AI Act 2024/1689 high-risk classification applies to (a) content-moderation classifier, (b) ranking model. Transparency + risk-management + post-deployment monitoring obligations per Arts. 9–15 satisfied via `capabilities/T2-auto.yaml` evidence pipeline + ADR-SOC-0003.

### Availability + SLO

- Availability target: 99.95 % monthly for feed-render + post-create + profile-render.
- Notification fanout is best-effort; 99.9 % monthly.
- Search availability 99.9 % monthly.
- RTO: ≤ 15 min for post-store. RPO: ≤ 5 min (cross-region replication for professional store).

### Data residency

- Per-tenant pack pinning per ADR-0117. Personal-tier user data follows the personal-residency model (per-user); professional follows tenant-residency.
- Federation egress is per-tenant opt-in for the Professional tier only; subject to SCC + tenant attestation; Personal tier forbidden.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename). Layers used: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-redis`, `adapter-s3`, `adapter-meilisearch`, `adapter-clamav`, `adapter-opswat`, `adapter-activitypub`, `adapter-imagemagick`, `adapter-ffmpeg`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `user-profile` | `oya-social-user-profile-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Profile CRUD; handle uniqueness per (tenant, context); avatar + header media refs; verification badge; persona switch | `Profile`, `Handle`, `VerificationBadge`, `PersonaContext` |
| `follow-graph` | `oya-social-follow-graph-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | Directed follow edges; mutual-follow = friend; block / mute lists; adjacency-list storage | `FollowEdge`, `BlockEdge`, `MuteEdge`, `FriendDerivation` |
| `post-composition` | `oya-social-post-composition-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,adapter-imagemagick,adapter-ffmpeg,rest,worker,sdk,app}` | Post + repost + quote-post + comment-reply CRUD; media upload + transcode; link-preview; visibility scope; cross-link to messenger | `Post`, `Repost`, `QuotePost`, `Comment`, `Media`, `LinkPreview`, `Visibility`, `ContentWarning` |
| `feed-timeline` | `oya-social-feed-timeline-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk,app}` | Chronological + algorithmic feed materialisation; fanout-on-write for hot-tier accounts; fanout-on-read for cold-tier; ranking | `FeedEntry`, `RankingSignal`, `FanoutPlan`, `RankSnapshot` |
| `reactions` | `oya-social-reactions-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Inline reactions (emoji set bounded); conflict-free counter; per-user reaction record | `Reaction`, `ReactionTally`, `UserReactionRecord` |
| `mentions` | `oya-social-mentions-{kernel,domain,usecase,api,adapter,worker,sdk}` | @mention parse; Ontology lookup; fanout to notifications + cross-µservice (messenger bridge) | `Mention`, `MentionTarget`, `MentionFanoutPlan` |
| `hashtags` | `oya-social-hashtags-{kernel,domain,usecase,api,adapter,adapter-postgres,worker,sdk}` | #tag parse; per-tag corpus; trending input emission | `Hashtag`, `HashtagCorpus`, `HashtagEmission` |
| `trending-topics` | `oya-social-trending-topics-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk}` | Windowed trend compute over hashtags + entities; per-tenant per-pack ranking | `TrendingTopic`, `TrendWindow`, `TrendRank` |
| `notifications` | `oya-social-notifications-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-redis,worker,sdk,app}` | Real-time + digest notification delivery; per-recipient idempotent; backpressure-coalesced | `Notification`, `DigestBucket`, `RealtimeFrame` |
| `content-moderation` | `oya-social-content-moderation-{kernel,domain,usecase,api,adapter,adapter-clamav,adapter-opswat,worker,sdk}` | AI-classifier verdicts; manual reviewer queue; appeal workflow input; abuse-report ingestion; EU AI Act high-risk | `ModerationVerdict`, `AbuseReport`, `Appeal`, `ClassifierVersion` |
| `bookmarks` | `oya-social-bookmarks-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Per-user bookmark list; private to user | `Bookmark`, `BookmarkFolder` |
| `lists` | `oya-social-lists-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | User-curated lists of accounts; per-list feed view; private or public | `List`, `ListMembership`, `ListVisibility` |
| `search` | `oya-social-search-{kernel,domain,usecase,api,adapter,adapter-meilisearch,worker,sdk}` | People + content + hashtag search; Cedar-filtered; PHI-redacted in pack-us-healthcare | `SearchDoc`, `SearchQuery`, `SearchResultSet` |
| `profile-verification` | `oya-social-profile-verification-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Verification badge issuance; per-tenant policy; revocation | `VerificationRequest`, `VerificationBadge`, `RevocationEvent` |
| `age-verification` | `oya-social-age-verification-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Pack-aware age-gate; signup attestation; minor-protection routing | `AgeAttestation`, `AgeBracket`, `MinorProtectionPolicy` |
| `federation-gateway` | `oya-social-federation-gateway-{kernel,domain,usecase,api,adapter,adapter-activitypub,worker,sdk}` | Optional ActivityPub egress + ingress; per-tenant opt-in; Professional-only | `FederationPeer`, `ActivityInbox`, `ActivityOutbox`, `FederationOptIn` |

Naming justification — `user-profile`:

```
NAME: oya-social-user-profile-<layer>
JUSTIFICATION:
- microservice = social: per ADR-0131 per-microservice flat layout.
- bc-tokens = user-profile: primary BC. ADR-0056 v4.1 BC-optionality rule honoured.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-postgres / -adapter-s3 / -adapter-meilisearch /
  -adapter-clamav / -adapter-opswat / -adapter-activitypub / -adapter-imagemagick /
  -adapter-ffmpeg are canonical *-adapter-<backend> per ADR-0105 Amendment 3.
```

Total crates introduced: **~115** (16 BCs × 7-9 layers per BC depending on backend variety).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `ProfileRepository` | `oya-social-user-profile-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `FollowGraphRepository` | `oya-social-follow-graph-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `RELATIONSHIP_GRAPH` |
| `PostStore` | `oya-social-post-composition-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING`, `PHI` (pack-us-healthcare) |
| `MediaBlobStore` | `oya-social-post-composition-kernel` | `-adapter-s3` | `BEHAVIORAL_TENANT_PRODUCT`, sometimes `PII_IDENTIFYING` |
| `ImageTranscoder` | `oya-social-post-composition-kernel` | `-adapter-imagemagick` | `INTERNAL_ONLY` |
| `VideoTranscoder` | `oya-social-post-composition-kernel` | `-adapter-ffmpeg` | `INTERNAL_ONLY` |
| `MalwareScanner` | `oya-social-content-moderation-kernel` | `-adapter-opswat` / `-adapter-clamav` | `INTERNAL_ONLY` |
| `FeedCache` | `oya-social-feed-timeline-kernel` | `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ReactionCounter` | `oya-social-reactions-kernel` | `-adapter-redis` + `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `MentionResolver` | `oya-social-mentions-kernel` | `-adapter` (Ontology client) | `PII_IDENTIFYING` |
| `TrendStore` | `oya-social-trending-topics-kernel` | `-adapter-postgres` + `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `NotificationStore` | `oya-social-notifications-kernel` | `-adapter-postgres` + `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `SearchIndex` | `oya-social-search-kernel` | `-adapter-meilisearch` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ModerationClassifier` | `oya-social-content-moderation-kernel` | `-adapter` (foundry-runtime client; T2) | `INTERNAL_ONLY` |
| `ActivityPubGateway` | `oya-social-federation-gateway-kernel` | `-adapter-activitypub` | `BEHAVIORAL_TENANT_PRODUCT` (Professional only) |
| `CedarSocialPolicy` | `oya-social-user-profile-kernel` (cross-BC) | `-adapter` (Cedar evaluator) | `INTERNAL_ONLY` |
| `AuditChainClient` | (cross-BC) `-kernel` per BC | (cross-BC) `-adapter` to audit-chain µservice | `AUDIT` |

Data-class enforcement: `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `social` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice social` — dependency-direction
- `oya gate validate lean-a2 --microservice social` — cross-product-refusal
- `oya gate validate port-location --microservice social`
- `oya gate validate layer-correctness --microservice social`
- `oya gate validate per-microservice-layout --microservice social`
- `oya gate validate statelessness --microservice social`
- `oya gate validate shardability --microservice social`
- `oya gate validate authority-cohesion --microservice social` (HG-SOCIAL)
- `oya gate validate dual-context-isolation --microservice social` (per parallel ADR-0135)

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `PostPublished` | end-user publishes post | search-index, mentions, feed-timeline, trending-topics, audit-chain, downstream Workflow engines | append-only |
| `PostEdited` | end-user edits within edit-window | search-index, audit-chain | append-only delta |
| `PostDeleted` | end-user / admin deletes | search-index, audit-chain, retention-purge worker | tombstone |
| `RepostCreated` | end-user reposts | feed-timeline, audit-chain | append-only |
| `QuotePostCreated` | end-user quote-posts | feed-timeline, mentions, audit-chain | append-only |
| `CommentPublished` | end-user comments | feed-timeline, mentions, audit-chain | append-only |
| `ReactionAdded` / `ReactionRemoved` | end-user reacts | feed-timeline, downstream engines | append-only |
| `FollowEdgeAdded` / `FollowEdgeRemoved` | end-user follows / unfollows | feed-timeline, notifications, audit-chain | append-only |
| `BlockEdgeAdded` / `MuteEdgeAdded` | end-user blocks / mutes | feed-timeline, notifications | append-only |
| `MentionEmitted` | mentions BC resolves a mention | notifications, messenger bridge, action-card consumer (Workflow Studio) | append-only |
| `HashtagEmission` | post carries hashtags | trending-topics, search-index | append-only |
| `ModerationVerdictEmitted` | classifier or reviewer issues verdict | feed-timeline (hide/show), notifications (sender), audit-chain | append-only |
| `AppealOpened` / `AppealResolved` | end-user appeals; reviewer resolves | audit-chain, notifications | append-only |
| `AbuseReportFiled` | end-user files abuse report | content-moderation, audit-chain | append-only |
| `VerificationBadgeIssued` / `Revoked` | tenant-admin verifies / revokes | feed-timeline, search-index, audit-chain | append-only |
| `ProfileCreated` / `ProfileUpdated` / `ProfileDeleted` | user signup / edit / delete | ontology (`Person` write), audit-chain | append-only |
| `EDiscoveryHoldOpened` / `Closed` | compliance-officer action | audit-chain, retention-purge worker | append-only |
| `FourEyesDisclosureExecuted` | tenant-admin pair approves Professional PII read | audit-chain | append-only |
| `FederationOptInGranted` / `Revoked` | tenant opts in / out of ActivityPub | federation-gateway, audit-chain | append-only |
| `FederationActivityIngested` / `FederationActivityEgressed` | ActivityPub inbox/outbox | feed-timeline, audit-chain | append-only |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `OntologyEntityChanged` (Person/Team/Topic) | ontology | mentions | refresh mention-resolution cache |
| `MessengerDeepLinkRequested` | messenger | post-composition | resolve post URL → embed metadata |
| `TenantRetentionPolicyUpdated` | tenancy | post-composition | reassign post retention bounds |
| `AuditChainSealed` | audit-chain | (read-only) | confirm audit-write durability |
| `WorkflowStudioRunStarted/Completed` | workflow-engine | notifications | post status into bound profile |
| `MailActionCardEmitted` | mail | post-composition | post action-card as a quote-post comment (Professional-tier only) |

### Ontology writes

| Object Type | Written by BC | Audit trail |
|---|---|---|
| `Person{user_id, tenant_id, context_kind, handle, verification_state}` | `user-profile` | Ed25519 |
| `Post{post_id, author_ref, tenant_id, context_kind, visibility, posted_at, data_class}` | `post-composition` | Ed25519 |
| `Topic{topic_id, hashtag, pack, first_seen_at}` (hashtag → topic promotion) | `hashtags` + `trending-topics` | Ed25519 |
| `FollowRelation{follower_ref, followee_ref, established_at}` | `follow-graph` | Ed25519 |
| `Mention{post_id, target_ref, mention_kind}` | `mentions` | Ed25519 |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Person`, `Team`, `Topic` | `mentions` | `find_by(@-handle, tenant_id)` for mention resolution |
| `RetentionPolicy` | `post-composition` | `lookup(tenant_id, context_kind)` |
| `TenantContext` | every BC | tenant scope + pack jurisdiction lookup |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Twitter / X | global microblog + follow-graph + algorithmic feed | full feature parity; ranking; mentions; reposts | `developer.twitter.com` (legacy) + `developer.x.com` |
| Bluesky | AT Protocol microblog; decentralised | OSS-decentralised; algorithmic-feed-marketplace | `docs.bsky.app` |
| Mastodon | ActivityPub federated microblog | federation; chronological-only by default | `docs.joinmastodon.org` |
| Threads (Meta) | Instagram-tied microblog | mobile-first; Meta-graph leverage | `developers.facebook.com/docs/threads` |
| Facebook | full social network | profile + feed + groups + events | `developers.facebook.com` |
| Instagram | photo + reels + stories | media-first; reels; stories | `developers.facebook.com/docs/instagram-platform` |
| LinkedIn | professional social network | dual-context (professional-only); rich profiles | `learn.microsoft.com/linkedin` |
| TikTok (timeline) | short-video feed | algorithmic-feed sophistication; ranking | (proprietary; benchmark via 3rd-party studies) |
| Pinterest | visual-discovery board | image-first; collection model | `developers.pinterest.com` |
| Reddit | community-forum + threaded discussion | subreddit model; voting | `www.reddit.com/dev/api` |
| Lemmy | OSS federated Reddit-alike | ActivityPub federation | `join-lemmy.org/docs/` |
| Truth Social | Mastodon-fork microblog | (US-political niche; included for completeness) | (limited public docs) |
| Hive Social | Twitter-alike; minimal moderation | (smaller scale; included for completeness) | (limited public docs) |

Key parity gaps to close (ordered by priority):

1. **Dual-context isolation by data-model** — none of the competitors enforce personal/professional context as a data-model invariant (LinkedIn is professional-only; Twitter/X blurs via shared identity). Target: compile-time + LEAN-lane enforcement.
2. **Four-eyes admin disclosure on professional reads** — Twitter/X/Meta allow admin discovery without two-party approval. Target: Bominal ADR-0215 four-eyes pattern.
3. **Native Workflow + Ontology integration** — competitors expose webhooks/Graph APIs; oyatie exposes typed Workflow events + Ontology object writes natively.
4. **OpenSLO + agentic gate** — none gate feature rollouts on SLO compliance; oyatie does (per ADR-0139).
5. **Multi-pack residency + per-pack regulatory overlays** — competitors are SaaS-region-coarse; oyatie is per-pack jurisdiction-pinned.
6. **EU AI Act high-risk transparency** — competitors lag on Art. 50 transparency labels; oyatie ships from day-1 per `capabilities/T2-auto.yaml`.

## Performance Targets

(See Performance NFR table above.)

Error budget:
- Monthly error budget for post-create + feed-render: 0.05 % (≈ 22 min/month).
- Burn-rate alarm on `social.post-create.availability` is 14.4× burn rate over 1h.
- Error budget policy: `microservices/social/runbooks/error-budget-policy.md` (Slice B).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres for profiles + posts + follow-graph; Redis for feed cache + reactions + presence; S3 for media; Meilisearch for search; ActivityPub gateway stateless beyond peer registry.

**Active-active compatibility**: stateless REST + worker pods + Postgres logical-replicated within pack; Redis primary-replica HA; S3 cross-AZ replication.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active users / cell | 500k | 5M | feed-render p99 > 200ms |
| Posts/sec sustained | 1k | 25k | Postgres write IOPS > 70% |
| Profiles per tenant | 1k | 1M | per-tenant cardinality limit hit |
| Media/day | 100k | 5M | S3 PUT rate > 70% provisioned |
| Search index size | 100GB | 5TB | shard count exceeded |
| Follow edges per cell | 100M | 5B | adjacency-list shard saturation |

Scale-out policy:
- HPA on REST pods: CPU > 70 %, min 6, max 200 replicas.
- Postgres shard-by-tenant once cell hits 25k posts/sec aggregate.
- Redis cluster sharding by `(tenant_id, post_id) mod N`.
- Follow-graph: adjacency-list sharded by `(follower_tenant, follower_user mod N)`.

Sharding:
- Post store partitions by `(tenant_id, author_ref, year-month)`.
- Follow-graph partitions by `(tenant_id, follower_ref mod N)`.
- Feed cache partitions by `(tenant_id, user_ref mod N)`.
- Reaction store partitions by `(tenant_id, post_id mod N)`.
- `oya-check-shardability-cli` lane verifies partition keys are present in every kernel struct.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | A profile-create + post + repost + reaction + comment + follow roundtrip completes within p99 < 250ms post-create | `microservices/social/tests/e2e/profile-post-repost.rs` |
| AC-02 | Personal-context profile cannot post under Professional tenant context | `tests/e2e/dual-context-isolation.rs` |
| AC-03 | Professional Post admin disclosure requires two distinct approving principals + audit-chain seal | `tests/e2e/four-eyes-disclosure.rs` |
| AC-04 | Media upload, scan, transcode, finalize, then revoke after retention TTL | `tests/e2e/media-lifecycle.rs` |
| AC-05 | @mention of Person resolves via Ontology and emits `MentionEmitted` within 250ms | `tests/e2e/mention-emit.rs` |
| AC-06 | Notification fanout to 10k followers within 2s p99 | `tests/e2e/notification-fanout.rs` |
| AC-07 | People + content search returns only Cedar-permitted results | `tests/e2e/search-cedar-scope.rs` |
| AC-08 | Federation: Personal-tier post attempted on ActivityPub egress is REFUSED | `tests/e2e/federation-personal-tier-refused.rs` |
| AC-09 | Moderation classifier verdict → audit-chain seal within 2s + appeal-workflow opens | `tests/e2e/moderation-appeal.rs` |
| AC-10 | Age-gate: minor signup on pack-eu requires parental consent attestation | `tests/e2e/age-gate-pack-eu.rs` |
| AC-11 | `oya gate validate per-microservice-layout --microservice social` exit 0 | ADR-0131 lane |
| AC-12 | `oya gate validate authority-cohesion --microservice social` exit 0 | ADR-0123 lane; HG-SOCIAL registered |
| AC-13 | `oya gate validate dual-context-isolation --microservice social` exit 0 | per parallel ADR-0135 |
| AC-14 | EU AI Act transparency label appears on every classifier verdict on pack-eu | `tests/e2e/eu-ai-act-transparency.rs` |
| AC-15 | Ads-substrate-stub T2 capability is disabled by default; tenant-admin opt-in required | `tests/e2e/ads-substrate-default-off.rs` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Ranking-model openness: closed-weights vs published-weights for trust + EU AI Act audit transparency | axis-social + council-privacy | ADR-SOC successor-IP |
| 2 | Federation: ActivityPub only, or also AT Protocol (Bluesky)? | axis-social + council-architecture | successor-IP ADR after federation minimum-shippable-tier |
| 3 | Ads-substrate-stub: keep interface-only-pending-impl indefinitely vs delete vs activate-with-tenant-opt-in | council-architecture + gtm | ADR successor-IP after M03 |
| 4 | Self-observability: social emits to observability µservice as one tenant or per-pack? | axis-social + axis-observability | resolved in IP-014 |
| 5 | Verified-handle uniqueness scope: per-tenant or per-pack or global? | axis-social + gtm | ADR-SOC successor-IP |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0008 | Data Use Boundary | personal/professional data-use invariants |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum + Amendment 3 | layer + backend-qualified authority |
| ADR-0106 | application → usecase | layer naming |
| ADR-0135 | Connect dissolution (parallel) | dual-context isolation source; social as a sibling µservice |
| ADR-0139 | Agentic SLO-gated promotion | gates social releases |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | Suite-and-bundle dissolution | factored Connect into surfaces |
| ADR-0133 | Industry best-practice conformance | HG-SOCIAL under this |
| ADR-SOC-0001 | Feed-ranking-algorithm | this µservice |
| ADR-SOC-0002 | Follow-graph-storage | this µservice |
| ADR-SOC-0003 | Content-moderation-classifier-bounds | this µservice |
| ADR-SOC-0004 | Federation-posture | this µservice |
| ADR-SOC-0005 | Dual-context-feed-isolation | this µservice |
| ADR-SOC-0006 | Media-transcode-and-storage | this µservice |
| Bominal ADR-0208 | Connect dual-context unified channel hub | inherited |
| Bominal ADR-0215 | Connect retention legal-hold dual-context | inherited |
| Bominal ADR-0028 | Audit-chain Merkle + Ed25519 | inherited |
| Bominal ADR-0111 | Ciphertext property type + envelope encryption | inherited |
