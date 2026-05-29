---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-messenger
microservice: messenger
status: Accepted
sales_segment: shared-substrate-and-product
tier: hero-product
tier_subtype: product-consumer-messenger
service_classification_rationale: |
  The messenger µservice is a hero product launching B2C (personal) + B2B (work)
  day-one. It is both an end-user surface (Signal/Telegram/KakaoTalk/Line/
  WhatsApp/Instagram-DM/FB-Messenger/Discord parity for personal; Slack/Teams
  parity for work) and a consumable substrate (cross-product mention surface,
  workflow trigger surface, mail-bridge target). Per ADR-0245 §D-3.B the
  product surface dominates; substrate consumption by sibling products
  (community, mail, calendar, workflow) goes through Workflow events +
  Ontology object writes only.
tier_certified_at: 2026-05-20
launch_modes: [B2C-personal, B2B-work, oyatie-internal-tenant]
milestone_first_ship: M02-foundation
bominal_source: [ADR-0208-connect-dual-context-unified-channel-hub.md, ADR-0215-retention-legal-hold-dual-context.md]
related_adrs:
  - ADR-0008
  - ADR-0028
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0117
  - ADR-0123
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-0135
  - ADR-0139
  - ADR-0140
  - ADR-0145
  - ADR-0148
  - ADR-0150
  - ADR-0172
  - ADR-0208
  - ADR-0215
  - ADR-0238
  - ADR-0240
  - ADR-0241
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0251
  - ADR-0255
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
  - ADR-MSGR-0001
  - ADR-MSGR-0002
  - ADR-MSGR-0003
related_specs:
  - /specs/microservices/messenger.json
  - /specs/per-microservice-flat-layout.json
  - /specs/agentic-slo-gated-promotion.json
  - /specs/tenant-model.json
date: 2026-05-20
owner_team: axis-messenger + council-privacy + ops-deliverability
doc_status: published
tenant_scoped: true
audience_modes:
  - B2C-personal
  - B2B-work
  - oyatie-internal-tenant
benchmarks:
  - signal
  - telegram
  - kakaotalk
  - line
  - whatsapp
  - instagram-dm
  - facebook-messenger
  - discord
  - slack
  - microsoft-teams
  - element-matrix
  - imessage
---

# PRD-messenger: Personal messenger + Work messenger as two surfaces of one messaging substrate

> Hero product. Launches B2C personal + B2B work + `oyatie.*` internal-tenant on day one. Signal/Telegram/KakaoTalk/Line/WhatsApp/Instagram-DM/FB-Messenger/Discord parity for personal; Slack/Teams/Discord-Stage parity for work with native Mail + Calendar + Meet + Workflow Engine integration. Per ADR-0242 oyatie is a tenant of its own platform; per ADR-0245 every messenger surface is built on the same substrate and differentiated by Cedar-gated localisation overlays.

---

## Strict personal/professional separation

Messenger is a concrete microservice, not part of any retired grouping. Personal DMs/groups and professional tenant channels share UX primitives but not tenancy or RBAC authority. Personal messaging lives under the user's personal tenant scope; professional messaging lives under the organization/workspace tenant scope. The default cross-context decision is **deny**: org admins, legal hold, retention, DLP, search, export, notification routing, workflow bridges, huddles, and automation for a professional tenant cannot read, infer, mutate, or route personal messages. Any explicit user-mediated cross-context action must carry tenant id, RBAC scope, policy decision id, data-class check, and audit-chain evidence before execution.

## 1. Purpose

The `messenger` µservice is oyatie's unified real-time messaging surface. It speaks Matrix Client-Server + Server-Server APIs, WebSocket/QUIC native transport, MLS (RFC 9420) E2E key agreement, ActivityPub for federation, and HTTP/3/QUIC at the edge. It stores conversations in a per-tenant Postgres + Citus + SeaweedFS + ClickHouse + Tantivy/Meilisearch stack, and is differentiated at the application layer into two products that share the same substrate:

- **Personal Messenger (B2C).** Standalone consumer messenger account (`alice@oyatie.app` handle, phone-number optional). Best-in-class 1:1 + group chat, voice + video calls, stickers + custom emoji + GIFs, status/stories, multi-device sync, full MLS E2E by default. Targets Signal / Telegram / KakaoTalk / Line / WhatsApp / Instagram DM / Facebook Messenger / Discord / iMessage switchers.
- **Work Messenger (B2B).** Enterprise messenger attached to a tenant (`alice@acme.com` on `acme` tenant). Adds channels, threads, mentions, Workflow-Engine slash commands, mail-to-messenger bridge, calendar + meet integration, huddles (per ADR-MSGR-0001), clocking-in via `/in`, e-signing flows, approval workflows, DLP, eDiscovery, audit-chain, retention, federation via Slack + Matrix bridges. Targets Slack / Microsoft Teams / Mattermost / Rocket.Chat / Naver Works / Line Works switchers.

A single user can hold any number of personal handles and be a member of any number of work tenants. The kernel-layer **dual-context isolation** invariant (ADR-0135 parallel + Bominal ADR-0208) guarantees a user's personal DMs are structurally invisible to any org admin, even when both contexts share a physical cluster, and that a personal DM cannot become a professional channel reply.

Both surfaces are products in their own right. The substrate (`oya-messenger-*` crates) is also reusable by any other oyatie product that needs messaging primitives (e.g., Community's cross-product mention surface, Mail's action-card carrier, the Workflow Studio's run-status channel binding, Plugin App Store's app-installation receipt notifier). Substrate-vs-product layering follows ADR-0245.

Per ADR-MSGR-0001 the canonical voice/video huddle composition is LiveKit 2024.x SFU + WebRTC + ICE/STUN/TURN + Opus + AV1. Per ADR-MSGR-0002 the E2E group-key agreement is MLS (RFC 9420) for both personal and work. Per ADR-MSGR-0003 the federation wire format is Matrix Client-Server r0.6.1 + Server-Server r0.1.4 as the canonical-base, with ActivityPub bridges for fediverse interop on B2C personal opt-in only.

---

## 2. Audience and Tenant Modes

### 2.1 Tenant modes

| Mode | Tenant | Authoritative identity | Primary surface | Differentiated UX | Compliance posture |
|---|---|---|---|---|---|
| **B2C Personal** | implicit per-user (`__personal__/<user_id>`) | Zitadel personal IdP (email-password + WebAuthn + phone-number + Apple/Google/Apple-Sign-In/Google-One-Tap federation) | Personal Messenger web/mobile/desktop apps | Signal-class E2E, KakaoTalk-class stickers, Discord-class voice, Telegram-class channels (read-only broadcast) | GDPR + CCPA + KR-PIPA personal-data subject |
| **B2B Work** | per-org (`acme`, `naver`, etc.) | Zitadel tenant IdP (SAML/OIDC SSO, SCIM provision, MFA enforced) | Work Messenger web/mobile/desktop + admin console | Slack-class channels + huddles, workflow triggers, mail-bridge, clock-in flows | GDPR/HIPAA/SOC2/ISO-27001 processor; per-tenant retention floors |
| **`oyatie.*` internal** | `oyatie-corp` (axis-messenger, council-*, ops-*) | Zitadel internal IdP + GitHub federation | Work Messenger with internal admin overlay | Same as B2B with dogfooding flags surfaced | Same as B2B; per ADR-0242 we ARE a tenant |

Per ADR-0242 (oyatie-is-a-tenant doctrine): the oyatie company itself runs entirely on the `oyatie-corp` tenant with no special-case code paths. Every dogfooding bug found is a paying-customer bug found.

### 2.2 Cedar gating across modes

The same Cedar policy engine (per ADR-0140 / ADR-0255) gates feature availability per mode:

- B2C: `Messenger::FeatureClass::Personal` policies — `e2e_mls=enforce`, `stickers=allow`, `voice_video_call=allow`, `stories=allow`, `channels_broadcast=allow`, `federation_activitypub=user_opt_in`, `dlp=deny`, `legal_hold=deny`, `workflow_trigger=deny`.
- B2B: `Messenger::FeatureClass::Work` policies — `channels_team=allow`, `threads=allow`, `huddles=allow`, `workflow_trigger=allow_with_explicit_consent`, `mail_bridge=allow`, `dlp=allow`, `legal_hold=allow`, `e2e_mls=tenant_opt_in_with_recovery_key_escrow`, `federation_matrix=tenant_configurable`, `federation_slack_connect=tenant_configurable`.
- Internal: B2B policy set + `dogfooding_flag=true` + `incident_redaction=on`.

Cedar fragments live under `microservices/messenger/policy/cedar/{personal,work,internal}.cedar` and are versioned with PR-review.

### 2.3 User journey

A new oyatie user signs up with `alice@gmail.com`, gets a personal messenger handle `@alice` resolvable as `alice@oyatie.app`, and starts using Personal Messenger to chat with friends, join group chats, follow Telegram-style broadcast channels, post stories, and make voice/video calls — all MLS E2E by default. Months later her employer Acme adopts oyatie; her admin invites `alice@acme.com`. The Acme work messenger appears in her client as a second workspace. The two are kernel-isolated: Acme's compliance officer cannot legal-hold or eDiscover anything in her personal DMs; Alice cannot accidentally forward an Acme-confidential message into a personal group (cross-context routing is refused by `policy/dual-context-isolation.cedar` and the `oya-check-dual-context-isolation` lane).

### 2.4 Tenant_class adoption

Per ADR-0330 and ADR-0331, messenger supports `tenant_class_eligibility = ["demo_trial", "paid"]`. Tenant class is orthogonal to B2C personal, B2B work, and `oyatie.*` internal modes: every mode consumes the same cloud-iam principal claim and the same Cedar gate rather than a messenger-local tenant-class resolver.

- **Principal claim binding.** cloud-iam / identity issues `principal.tenant_class`, `principal.billing_components`, `principal.cap_breached`, and `principal.demo_trial_expires_at` at session issuance. The messenger gateway passes those claims to `policy/tenant-class.cedar`; messenger never re-queries cloud-billing per request.
- **Paid billing components emitted.** Messenger emits `paid_billing_components_emitted = ["per_seat", "per_usage"]`. Per-seat meters count active messenger seats and per-usage meters count message sends, active channels, huddle minutes, attachment bytes, MLS KeyPackage uploads, search queries, mention fanout, and workflow triggers.
- **demo_trial caps.** Demo trial tenants get the same capability surface and security posture, with capped usage: 25 monthly active users, 10 active channels, 50,000 messages per month, 30-day message retention, 120 huddle minutes per month, and 2 GB attachment storage. Cap breach preserves read access and refuses new write-heavy actions until conversion or grace-window resolution.
- **Compliance pack gate.** Compliance pack activation, HSM-anchored backup-key escrow, work-mode MLS recovery-key escrow, BYOK, cross-tenant Slack pairing, and retention overrides above 30 days require `principal.tenant_class == "paid"` and the relevant billing component. Demo trial tenants cannot activate compliance packs.
- **Conversion.** cloud-billing owns demo_trial to paid conversion. Messenger observes the refreshed principal claim at token refresh, clears cap-based write denies, and preserves message/channel state without retroactive billing.

---

## 3. Feature Matrix vs Benchmarks

Legend:
- `Y` = fully supported (parity).
- `P` = partial (paid SKU, limited region, gimped, or behind opt-in).
- `N` = not supported.
- `Y+` = supported and oyatie target exceeds (deeper, more controllable, cheaper, or open).

Sources cited inline in §14. Snapshot date: 2026-05.

### 3.1 1:1, group, channel, thread topology

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | Instagram DM | FB Messenger | Discord | iMessage | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|
| 1:1 direct messaging | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Group chat (small ≤ 250) | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Large group chat (≥ 1k members) | Y (1k) | Y (200k) | Y (1.5k) | Y (500) | Y (1024) | N | P | Y (500 / server-channel) | P | **Y+** (10k MLS group target; Matrix-class scale) |
| Mega-group / supergroup | N | Y | N | N | N | N | N | Y | N | **Y** (Telegram-style supergroup) |
| Broadcast channel (read-only) | N | Y | Y (PlusFriend) | Y (Official) | Y (Channels 2024) | Y (Broadcast) | Y (Broadcast list) | Y (Announcement) | N | **Y** (Telegram-style channels + WhatsApp Channels 2024 parity) |
| Team / workspace (B2B) | N | N | Work (KakaoWork) | Y (Line Works) | N | N | N | Y (Servers) | N | **Y** (Slack-class workspace) |
| Channels within workspace | N | N | P | Y | N | N | N | Y | N | **Y** (Slack/Discord-class) |
| Channel types (text, voice, forum, announcement, stage) | N | N | N | P | N | N | N | Y | N | **Y** (Discord-parity + Q&A added) |
| Threads (inline reply chain) | N | P | N | N | Y | Y (since 2024) | Y | Y | N | **Y** (Slack-class threading) |
| Mentions (`@user`, `@channel`, `@here`, `@everyone`) | P | Y | Y | Y | Y | Y | Y | Y | P | **Y+** (cross-product via Ontology) |
| Role mentions (`@role`) | N | N | N | N | N | N | N | Y | N | **Y** |
| Cross-workspace DM (Slack Connect-style) | N | N | N | N | N | N | N | N | N | **Y** (federation via Matrix + Slack-adapter) |

### 3.2 Reactions, edit, delete, scheduling, voice msg

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|
| Reactions (emoji) | Y | Y | Y | Y | Y | Y | Y | Y | Y (Tapbacks) | **Y** |
| Reactions (custom emoji) | N | Y (Premium) | Y | Y | N | Y | Y | Y | N | **Y** (per-tenant + personal emoji libraries) |
| Reaction tally visible | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Edit message | Y | Y (48h) | Y (5min) | N | Y (15min) | Y (15min) | Y (15min) | Y | Y (15min) | **Y+** (configurable per-channel; default 24h personal, tenant-configurable work) |
| Delete for self | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Delete for everyone | Y (3h) | Y | Y (5min) | Y (24h) | Y (~2 days) | Y (10min) | Y (10min) | Y (mod) | Y (15min) | **Y** (configurable; admin override audit-chained on work) |
| Self-destruct (timed disappear) | Y | Y (Secret Chat) | N | N | Y | Y (Vanish Mode) | Y (Vanish Mode) | N | N | **Y+** (per-conversation + per-message TTL; range 5s–90d) |
| Schedule send | N | Y | N | P | N | N | N | N | Y (Scheduled Send 2024) | **Y+** (timezone-aware) |
| Voice messages | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (Opus 48kHz; waveform; transcription opt-in) |
| Voice message transcription | N | Y (Premium) | N | N | Y (since 2024) | P | P | N | Y | **Y** (Intelligence substrate; opt-in; per-tenant model) |
| Video messages (short clips) | N | Y (Round) | N | N | N | N | N | N | N | **Y** (Telegram-round + Instagram-style) |
| Pinning messages | Y (limit 4) | Y | Y | N | Y | Y | Y | Y | N | **Y** |

### 3.3 Voice + video calls

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage (FaceTime) | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|
| 1:1 voice call | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| 1:1 video call | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Group voice call | Y (50) | Y (200) | Y (200) | Y (500 audio) | Y (32) | Y (8) | Y (50) | Y (99 + Stage 10k) | Y (32 FaceTime) | **Y+** (1k participants via LiveKit SFU; per ADR-MSGR-0001) |
| Group video call | Y (40) | Y (30) | Y (8 grid) | Y (200) | Y (32) | Y (8) | Y (50) | Y (25) | Y (32) | **Y+** (100 active + 1k view-only) |
| Huddle (drop-in audio) | N | N | N | N | N | N | N | Y (Voice Channel) | N | **Y** (Slack-Huddle + Discord-Voice hybrid; per ADR-MSGR-0001) |
| Stage / town-hall channel | N | Y (Video Chat) | N | N | N | N | N | Y (Stage) | N | **Y** (Discord-Stage parity; speakers + listeners) |
| Screen share | N | Y | Y | Y | Y (since 2024) | N | Y | Y | Y | **Y** (WebRTC; per-participant; resolution + frame-rate adaptive) |
| Camera switch / front-back | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Background blur / replace | N | N | N | Y | Y | Y | Y | Y (Krisp) | Y (Studio Light) | **Y** (MediaPipe segmentation; on-device first) |
| Noise suppression | Y | Y | Y | P | Y | P | Y | Y (Krisp) | Y (Voice Isolation) | **Y** (RNNoise + Krisp-class; on-device first) |
| Spatial audio | N | N | N | N | N | N | N | N | Y | **Roadmap** (M04) |
| Recording (call) | N | N | Y | Y | N | N | Y | Y (server-bot) | P | **Y** (Cedar-gated; explicit-consent every participant; audit-chained) |
| Live captions / subtitles | N | N | N | N | N | N | N | Y | Y | **Y** (Intelligence substrate; opt-in) |
| Live translation | N | N | N | P | N | N | N | N | Y (Live 2024) | **Y** (M03; per ADR-MSGR-0001 §scope-2) |
| End-to-end encrypted call | Y | P (Secret Chat) | P | P | Y | P | P | N | Y | **Y** (DTLS-SRTP + MLS-based shared key per ADR-MSGR-0002) |

### 3.4 Stickers, emoji, GIF, polls, location, contacts

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|
| Built-in stickers | Y | Y | Y+ | Y+ | Y | Y | Y | Y | Y | **Y** |
| Custom sticker upload | Y | Y | Y | Y | Y | Y | N | Y | Y (in iOS 17+) | **Y** |
| Sticker store / marketplace | N | Y | Y+ (paid) | Y+ (paid) | N | N | N | Y (Nitro) | Y (App Store) | **Y** (Plugin App Store integration; free + paid sticker packs) |
| Animated stickers (Lottie / WebP) | N | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (Lottie + animated WebP + APNG) |
| Custom emoji (server / tenant) | N | Y (Premium) | N | N | N | N | N | Y | N | **Y** (Discord-class custom emoji; per-tenant + personal) |
| Standard Unicode emoji | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (Emoji 15.1; Unicode 15.1) |
| Skin-tone modifier | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| GIF picker (Giphy / Tenor) | N | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (Tenor primary; Giphy fallback; tenant policy can disable) |
| GIF auto-loop | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Polls | N | Y | Y | Y | Y (since 2024) | N | Y | Y (via bot) | N | **Y** (single + multi + ranked-choice; closing date) |
| Poll anonymity | N | Y | N | N | N | N | N | N | N | **Y** (audit-sealed but not surfaced when anonymous) |
| Location share (one-shot) | Y | Y | Y | Y | Y | Y | Y | N | Y | **Y** |
| Live location (timed) | Y | Y | Y | Y | Y | Y | N | N | Y (Find My) | **Y** (up to 8h; per-conversation; explicit-consent each share) |
| Contact share (vcard) | Y | Y | Y | Y | Y | Y | Y | N | Y | **Y** (RFC 6350 vCard 4.0) |

### 3.4.B Mobile-app-bundle coordination

Per the 2026-05-21 mobile-bundle directive, messenger, mail, social, and community ship as one client binary per platform while remaining four separate backend µservices.

- **Cross-handoffs.** Messenger invokes mail, social, and community through direct gRPC handoffs with typed intents: `mail.compose_from_thread`, `social.share_message_snapshot`, `community.publish_channel_announcement`, and `community.resolve_member_context`. Handoffs carry tenant scope, `principal.tenant_class`, data-class labels, and audit-chain correlation IDs.
- **Unified auth.** All four panes share a single cloud-iam session and principal claim set. Pane switching never mints a messenger-local identity, and Cedar receives the same `tenant_id`, `tenant_class`, `billing_components`, and `cap_breached` claims across messenger, mail, social, and community.
- **Unified push.** APNs, FCM, Windows Push Notification Services, and Web Push route through one notification surface with per-pane payload classification. The push envelope names the source pane, preserves MLS sealed-sender constraints for messenger content, and deduplicates mention/mail/social/community notifications before display.
- **Forbidden anti-patterns.** The bundle must not become a LinkedIn-style engagement feed, influencer-monetization surface, sponsored-post-promotion surface, or metric-chasing social graph. Messenger notifications prioritize user intent, safety, delivery, and tenant policy over engagement loops.
- **Per-platform clients.** iOS and macOS use Swift native clients; Android uses Kotlin; Windows uses WinUI 3; web uses Leptos SSR with selective hydration. These clients are panes over the same bundle shell, not separate messenger-only applications.

### 3.5 Status, stories, presence, read receipts

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|
| Status (one-line; persistent) | N | Y | Y | Y | Y | N | N | Y (Custom Status) | N | **Y** |
| Stories (24h ephemeral) | Y (Note 2024) | Y | Y (Story) | Y (Timeline) | Y (Status) | Y | Y | N | N | **Y** (24h default; per-story TTL configurable) |
| Story reactions / replies | Y | Y | Y | Y | Y | Y | Y | N | N | **Y** |
| Stories privacy (audience scope) | Y | Y | Y | Y | Y | Y | Y | N | N | **Y+** (per-list + Cedar-gated; close-friends-style) |
| Presence (online / away / dnd) | P | Y | Y | Y | Y (last seen) | Y | Y | Y | N | **Y** (online + away + DND + custom) |
| Last-seen / last-active | N | Y (configurable) | Y | Y | Y (configurable) | Y | Y | N | N | **Y** (per-user configurable; default off in B2C) |
| Typing indicator | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Read receipts (per-recipient in group) | Y (configurable) | Y | Y | Y | Y (configurable) | Y | Y | N | Y (configurable) | **Y** (per-user toggle; symmetric — if you turn off, you don't see others') |
| Read-receipt for channel (broadcast) | N | Y (view count) | Y (view count) | N | N | N | N | Y | N | **Y** (view-count for channels; per-message receipts for DMs) |
| Delivered receipt | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Reply quoting (inline) | Y | Y | Y | Y | Y | Y | Y | Y | Y (inline reply) | **Y** |
| Reply quoting (cross-channel) | N | Y (forward) | P | P | P | P | P | P | N | **Y** |
| Message forwarding | Y | Y | Y | Y | Y | Y | Y | N (re-post) | Y | **Y** (with "forwarded-from" indicator; Cedar-gated on B2B) |

### 3.6 Files, photos, video, drive

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|
| Photo / image attachment | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Compressed vs original quality | Y | Y | Y | Y | Y (HD opt-in 2024) | Y | Y | Y | Y | **Y+** (per-attachment toggle; HD default on Wi-Fi) |
| Video attachment | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (up to 5 GB per file; H.264 + AV1) |
| File attachment (any MIME) | Y | Y | Y | Y | Y | P (limited) | Y | Y | Y | **Y** (up to 5 GB; tenant-configurable to 100 GB SeaweedFS-link) |
| Multipart resumable upload | N | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (tus.io 1.0.0 protocol; resumable across network drops) |
| Inline image preview | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Inline video player | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (DASH + HLS adaptive bitrate) |
| Drag-and-drop attach (desktop) | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Paste image from clipboard | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Drive-integration share (file picker) | N | N | P | P | P | N | P | P | P | **Y+** (oyatie `drive` µservice + Google Drive / Dropbox / OneDrive adapters) |
| Attachment malware scan | N | P | P | P | P | P | P | P | Y (cloud) | **Y** (ClamAV + OPSWAT inline) |
| Attachment retention TTL | N | N | P | P | P | N | P | P | Y | **Y** (per-conversation TTL; per-tenant override; audit-chained) |

### 3.7 E2E encryption, multi-device, backups

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|
| E2E encryption by default (1:1) | Y | P (Secret Chat) | P (Secret Chat) | Y (Letter Sealing) | Y | Y (2024) | Y (2024) | N | Y | **Y** (MLS RFC 9420 by default; personal mode) |
| E2E encryption (group) | Y (Sender Keys) | P | P | Y | Y (Sender Keys) | Y | Y | N | Y | **Y** (MLS RFC 9420 group; scales to 10k members) |
| Signal Protocol / Double Ratchet | Y | N | N | N | Y | Y | Y | N | P | **Y** (used as MLS PSK input for cross-vendor bridges) |
| MLS RFC 9420 | N | N | N | N | N | N | N | N | N | **Y+** (canonical; first-class messenger using MLS at consumer scale) |
| Per-device key | Y | N (one cloud key) | N | N | Y | Y | Y | N | Y | **Y** (each device = MLS LeafNode) |
| Forward secrecy | Y | P | P | Y | Y | Y | Y | N | Y | **Y** (MLS commit-based key rotation per epoch) |
| Post-compromise security | Y | N | N | P | Y | Y | Y | N | Y | **Y** (MLS PCS by design) |
| Sealed sender (metadata-private) | Y | N | N | N | P | N | N | N | N | **Y** (per ADR-MSGR-0002 §metadata-minimisation) |
| Multi-device sync (same E2E account) | Y | Y (cloud) | Y (cloud) | Y | Y | Y | Y | Y (cloud) | Y | **Y** (MLS multi-LeafNode; no key escrow needed) |
| Web / desktop client (E2E-safe) | Y (linked device) | Y (cloud or secret) | Y | Y | Y | Y | Y | Y (cloud) | Y (Continuity) | **Y** (linked-device pattern; sealed sender at server) |
| Encrypted cloud backup | Y (passphrase) | Y | N | Y | Y (since 2023) | P | P | N | Y (Advanced Data Protection) | **Y** (user-controlled passphrase; HKDF + Argon2id; per ADR-0255) |
| Backup-key escrow / recovery | Y | Y (cloud) | N | Y | Y | P | P | N | Y (HSA recovery) | **Y** (opt-in per ADR-MSGR-0002; Shamir-split or HSM-anchored) |
| Key transparency (CONIKS / SEEMless) | N | N | N | N | N | N | N | N | Y (KT 2024) | **Y** (CONIKS-class log; per ADR-MSGR-0002 §key-transparency) |
| Verification (safety number / QR) | Y | Y | N | Y (Letter Sealing key) | Y | Y | Y | N | Y | **Y** (QR + 60-digit safety number) |
| encryption-BYOK (customer KMS for work tenant) | N | N | N | N | N | N | N | N | N | **Y+** (per ADR-0251; tenant KEK in tenant KMS region) |

### 3.8 Bots, slash commands, integrations, workflows

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage | Slack | Teams | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Bot API | N | Y+ | Y | Y | Y (Business API) | P | Y | Y | P (Business Chat) | Y | Y | **Y+** (Matrix bot API + native oyatie SDK; bot personas) |
| Slash commands | N | Y (`/`) | N | N | N | N | N | Y (`/`) | N | Y (`/`) | Y (`/`) | **Y** (built-in + tenant-defined + plugin-defined) |
| Workflow triggers (native) | N | P | N | N | N | N | N | N | N | Y (Workflow Builder) | Y (Power Automate) | **Y+** (native Workflow-Engine emission; Cedar-gated) |
| App / plugin store | N | Y (Mini Apps) | Y (Channels) | Y (LIFF) | N | N | N | Y (App Directory) | P | Y (App Directory) | Y (App Store) | **Y** (oyatie Plugin App Store integration) |
| Webhook (outbound) | N | Y | P | Y | Y | P | Y | Y | N | Y | Y | **Y** (HMAC-signed; per-channel) |
| Webhook (inbound) | N | Y | P | Y | Y | P | Y | Y | N | Y | Y | **Y** |
| Mini-app (in-conversation web app) | N | Y | Y (KakaoBiz) | Y (LIFF) | N | N | N | Y (Activities) | N | Y (Block Kit) | Y (Adaptive Cards) | **Y** (oyatie Action Cards + MCP-server-backed mini-apps) |
| Action buttons in message | N | Y (Inline buttons) | Y | Y | Y (Interactive Buttons) | N | Y | Y | N | Y (Block Kit) | Y (Adaptive Cards) | **Y** (typed; Cedar-gated) |
| Mail-to-messenger bridge | N | P (forwarders) | N | N | N | N | N | N | N | Y (Email Channel) | Y (Email Forward) | **Y+** (native via Mail µservice + Workflow-Engine) |
| Calendar integration | N | P | P | P | N | N | N | N | Y | Y (Outlook) | Y (Outlook) | **Y** (native via Calendar µservice; iMIP) |
| Video-meeting launch from chat | N | N | N | N | N | N | N | Y (Activities) | Y (FaceTime) | Y (Huddles) | Y | **Y** (native Meet µservice via ADR-MSGR-0001) |
| Issue-tracker integration | N | P | N | N | N | N | N | Y | N | Y | Y | **Y** (Jira/Linear/GitHub adapters via Plugin Store) |
| Clocking-in / HR | N | N | P (KakaoWork) | Y (Line Works) | N | N | N | N | N | P | P | **Y+** (native via `/in` slash; per ADR-0245 cross-product via Workflow + Ontology only) |
| E-signing in chat | N | N | N | N | N | N | N | N | N | P | P | **Y** (native via Workflow `oya-messenger-esignature`) |
| Approval workflows | N | N | N | N | N | N | N | N | N | Y (Workflow Builder) | Y (Approvals) | **Y+** (structured-card action; audit-chained) |

### 3.9 Search, archive, retention, history

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage | Slack | Teams | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Full-text search (own messages) | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (Meilisearch + Tantivy per ADR-COMM-0004 shared) |
| Search-as-you-type | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Advanced filters (from, has-file, date) | P | Y | Y | Y | Y | P | P | Y | Y | Y | Y | **Y** |
| Pin conversation | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Archive conversation | N | Y | N | Y | Y | Y | Y | N | N | Y | Y | **Y** |
| Mute conversation | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Block contact | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Unread / mark-as-unread | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Mention inbox / activity feed | N | Y | P | P | P | Y | Y | Y | N | Y | Y | **Y** |
| Retention policy (per-conversation) | N | N | P | P | P | N | P | N | N | Y | Y | **Y** (per-conversation + per-tenant override) |
| Legal hold (litigation hold) | N | N | N | N | N | N | N | N | N | Y | Y (Purview) | **Y+** (per ADR-0215; four-eyes plaintext disclosure) |
| eDiscovery export | N | N | N | N | N | N | N | N | N | Y (DLP) | Y (Purview) | **Y+** (sealed bundle; Ed25519 chain-of-custody) |
| DLP scan (outbound) | N | N | N | N | N | N | N | N | N | Y | Y | **Y+** (Cedar policy + content scan + DSAR-compliant logging) |
| Audit log | N | N | P | P | N | N | N | P | N | Y | Y | **Y+** (audit-chain Merkle + Ed25519 per ADR-0028) |

### 3.10 Federation, multi-device, multi-platform

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage | Slack | Element/Matrix | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Federation (cross-server) | N | N | N | N | N | N | N | N | N | P (Slack Connect) | Y (Matrix) | **Y** (Matrix r0.6.1 federation + Slack-adapter) |
| ActivityPub fediverse | N | N | N | N | N | N | N | N | N | N | P | **Y** (B2C personal opt-in; channel-broadcast as ActivityPub Group actor) |
| iOS native client | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (Swift native bundle pane) |
| Android native client | Y | Y | Y | Y | Y | Y | Y | Y | N | Y | Y | **Y** (Kotlin native bundle pane) |
| macOS desktop | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (Swift native bundle pane) |
| Windows desktop | Y | Y | Y | Y | Y | Y | Y | Y | N | Y | Y | **Y** (WinUI 3 native bundle pane) |
| Linux desktop | Y | Y | N | N | P (Wine) | N | N | Y | N | Y | Y | **Y** (Leptos web app installable as PWA) |
| Web client | N | Y | Y | N | Y | Y | Y | Y | N | Y | Y | **Y** (Leptos SSR + selective hydration + Web Push) |
| iPad-optimised | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** |
| Apple Watch | Y | Y | Y | Y | Y | Y | Y | N | Y | Y | Y | **Y** |
| Wear OS | N | Y | Y | Y | Y | Y | Y | N | N | Y | Y | **Y** |
| CarPlay / Android Auto | N | N | Y | Y | Y | N | Y | N | Y | N | N | **Y** (read + voice-reply) |
| Multi-account in client | N | Y | Y | Y | Y (Business + Personal) | Y | Y | Y | N | Y | Y | **Y** (mix personal + work in one client) |
| Offline mode | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (CRDT + outbox + sync on reconnect) |
| Push notifications | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (APNs + FCM + Web Push; sealed sender for E2E) |

### 3.11 Themes, accessibility, i18n

| Feature | Signal | Telegram | KakaoTalk | Line | WhatsApp | IG DM | FB Msgr | Discord | iMessage | Slack | Teams | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Dark mode | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (system-aware default) |
| Custom themes | P | Y | Y | Y | Y (2024 chat themes) | Y | Y | Y (Nitro) | P | Y | Y | **Y** (per-conversation + per-tenant; CSS variables) |
| Wallpaper per chat | N | Y | Y | Y | Y | N | Y | N | N | N | N | **Y** |
| Font size scaling | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (respects OS settings) |
| Screen-reader support (VO/TB) | Y | Y | P | P | Y | Y | Y | Y | Y | Y | Y | **Y** (WCAG 2.2 AA; ARIA-live for incoming msgs) |
| WCAG 2.2 AA | P | P | P | P | P | P | P | P | Y | P | Y | **Y** (CI lane verifies) |
| Keyboard nav (full) | P | Y | P | P | P | P | P | Y | Y | Y | Y | **Y** |
| High contrast mode | P | Y | P | P | P | P | Y | Y | Y | Y | Y | **Y** |
| Reduced motion | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (prefers-reduced-motion) |
| RTL languages | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | **Y** (Arabic, Hebrew, Persian, Urdu) |
| CJK ideographs + IME | Y | Y | Y+ | Y+ | Y | Y | Y | Y | Y | Y | Y | **Y+** (ko/ja/zh-Hans/zh-Hant first-class) |
| Built-in translation | N | N | Y (Papago) | P | N | Y (2024) | Y | N | Y (Live 2024) | P | Y | **Y** (Intelligence substrate; opt-in; per-tenant model) |
| i18n (13+ languages day-one) | Y | Y (~70) | Y (15+) | Y (20+) | Y (~70) | Y (~80) | Y (~80) | Y (~30) | Y (~40) | Y (~30) | Y (~40) | **Y** (en, ko, ja, zh-Hans, zh-Hant, es, pt, fr, de, it, nl, ru, ar, hi, id, vi, th — 17 day-one) |

### 3.12 Enterprise admin and compliance (B2B comparators)

| Feature | Slack | Microsoft Teams | Discord (Nitro Enterprise) | Mattermost | Rocket.Chat | Naver Works | Line Works | **oyatie target** |
|---|---|---|---|---|---|---|---|---|
| SSO (SAML / OIDC) | Y | Y | P | Y | Y | Y | Y | **Y** (Zitadel; SAML + OIDC + SCIM) |
| SCIM user provisioning | Y | Y | N | Y | Y | Y | Y | **Y** |
| MFA enforcement | Y | Y | Y | Y | Y | Y | Y | **Y** (passkey-preferred) |
| DLP (data-loss prevention) | Y | Y (Purview) | N | Y | Y | P | P | **Y+** (Cedar policy + content scan) |
| Retention policy per channel | Y | Y | N | Y | Y | Y | Y | **Y** |
| Legal hold | Y | Y (Purview) | N | Y | P | P | P | **Y+** (per ADR-0215; four-eyes plaintext) |
| eDiscovery export | Y | Y (Purview) | N | Y | P | P | P | **Y+** (sealed bundle; Ed25519 chain-of-custody) |
| Audit log | Y | Y | P | Y | Y | Y | Y | **Y+** (Merkle + Ed25519 per ADR-0028) |
| Per-jurisdiction data residency | P | Y | N | Y (self-host) | Y (self-host) | Y (KR) | Y (JP) | **Y+** (pack-kr/eu/us/jp/sg/au/in/br/ae/ksa per ADR-0117) |
| encryption-BYOK customer KMS | Y (EKM) | Y (encryption-BYOK) | N | Y | Y | N | N | **Y** (per ADR-0251) |
| Compliance exports (CSV, JSON, EDRM-XML) | Y | Y | N | Y | Y | P | P | **Y** |
| HIPAA / BAA | Y (Enterprise+) | Y | N | Y | Y | N | N | **Y** (pack-us-healthcare overlay) |
| FedRAMP / IL5 | Y (GovSlack) | Y (GCC-High) | N | Y (USG) | Y | N | N | **Roadmap** (M06) |
| Slack (cross-workspace DM) | Y | P (Federation) | N | P | P | N | N | **Y** (Slack-adapter; Matrix-bridged) |

### 3.13 Aggregate parity scorecard

Aggregate target: oyatie messenger meets-or-beats every `Y` in the above tables; closes the gaps that competitors leave open by combining the union of:

- **Signal**'s E2E rigour + MLS at scale.
- **Telegram**'s big-group + channels + bot ecosystem (without the metadata-leak).
- **KakaoTalk/Line**'s sticker-first UX + KR/JP localisation depth.
- **WhatsApp**'s simplicity + universal multi-device.
- **Instagram DM / FB Messenger**'s stories + reactions + ephemeral.
- **Discord**'s voice channels + huddles + custom emoji + roles.
- **iMessage**'s Tapbacks + sealed-sender + key-transparency.
- **Slack/Teams**'s channels + threads + workflow + admin + eDiscovery.
- **Matrix/Element**'s federation + open protocol.

Aggregate gaps closed at M02 launch:

1. **MLS at consumer scale** — no major consumer messenger uses MLS yet; we ship first.
2. **Sealed sender on metadata + key transparency** — only iMessage 2024 ships KT; we ship both.
3. **Native cross-product Workflow + Ontology integration** — no competitor exposes typed workflow events natively.
4. **Per-pack residency + per-jurisdiction regulatory overlay** — competitors are region-coarse; we are pack-pinned.
5. **Dual-context isolation by data-model invariant** — Slack/Teams blur via shared identity; we enforce in the kernel + LEAN-lane CI.

---

## 4. Personal Mode (B2C) — Feature Deep-Dive

### 4.1 Account creation

Sign-up via Zitadel personal IdP: phone-number-first (KakaoTalk/WhatsApp pattern) OR email-password (Signal/Telegram email-fallback pattern) OR passkey-first (iMessage-class) — user picks. WebAuthn passkey strongly recommended. Optional Apple/Google/Facebook federation. Auto-provisions `@<chosen>` handle (resolvable as `<chosen>@oyatie.app`) and a primary inbox in `__personal__/<user_id>` tenant. Custom-handle paid tier (e.g., `@alice` premium). Phone-number-to-handle directory opt-in (Signal-style).

### 4.2 MLS (RFC 9420) end-to-end encryption (per ADR-MSGR-0002)

The personal messenger uses **MLS RFC 9420** as the canonical group key agreement for both 1:1 and group conversations. Each device is a separate MLS LeafNode in the user's tree; the group has its own tree across all member devices. Key properties:

- **Forward secrecy** via per-epoch commit; old keys cannot decrypt new messages even if device compromised.
- **Post-compromise security**: a malicious actor who compromises a device loses access after the next commit (typically within seconds for active groups, hours for inactive).
- **Scales to 10k members** with O(log N) commit complexity — Signal's Sender Keys are O(N); MLS is the scalability win.
- **Authentication via Authentication Service** (AS) per RFC 9420 §5.3 — oyatie's `identity` µservice issues KeyPackages signed by user's master key.
- **Delivery Service** (DS) per RFC 9420 §5.4 — oyatie's `message-stream` BC routes commits + application messages; never sees plaintext.

Cross-vendor bridges (e.g., Slack adapter, Matrix federation): MLS is mandatory inside oyatie; bridge endpoints decrypt MLS, transcode to the bridge's wire format (Signal Protocol, Olm/Megolm for Matrix r0.6.1, Slack's TLS-only), and re-encrypt. The bridge run is audit-chained as a `CrossVendorBridgeDecryptionEvent`.

Sealed sender: per ADR-MSGR-0002 §metadata-minimisation, the server sees only the recipient device's wrapped key blob; it does NOT see who the sender is. Implementation: sender attaches a sealed envelope encrypted to recipient's device key containing sender's identity proof.

Key transparency: oyatie runs a **CONIKS-class transparency log** (per ADR-MSGR-0002 §key-transparency) for all KeyPackages. Clients audit the log on every conversation start (gossip protocol). Equivocation by the server is detectable.

### 4.3 Multi-device (per RFC 9420 multi-LeafNode pattern)

When Alice adds a second device (iPad alongside iPhone):
1. iPad generates its own MLS KeyPackage; publishes to identity µservice.
2. iPhone (linked device) verifies KeyPackage out-of-band via QR code + safety number.
3. iPhone issues an MLS commit adding the iPad LeafNode to all of Alice's conversations.
4. Pending messages re-sealed to the new tree on next epoch.
5. iPad receives full history forward from its add-epoch; historical-back-to-zero requires explicit "share history" UX (Signal-pattern; can refuse).

Each device holds its own keys; oyatie operators never see them. No cloud key escrow by default. Optional opt-in passphrase-protected encrypted backup to SeaweedFS (HKDF + Argon2id).

### 4.4 Voice + video calls (per ADR-MSGR-0001 LiveKit + WebRTC)

**Architecture.** LiveKit 2024.x SFU per region (selective forwarding unit). WebRTC PeerConnection client-side. STUN/TURN for NAT traversal via `coturn` per cell. ICE for path selection. DTLS-SRTP for media encryption with the MLS-derived group key as PSK input (per ADR-MSGR-0002 §call-key-binding).

**Codec.** Opus 48 kHz mono/stereo for voice; AV1 for video (with H.264 fallback for older clients). Per-stream simulcast (180p / 360p / 720p / 1080p) so SFU can downgrade to slow networks without re-encoding.

**1:1 calls.** Direct P2P attempted first (DTLS-SRTP via ICE); falls back to SFU if path fails (NAT, firewall, etc.). P99 call-start latency ≤ 1s.

**Group calls.** SFU mandatory. Up to 100 active speakers + 1k view-only participants. Active-speaker detection via WebRTC `RTCRtpReceiver` audio-level stats.

**Huddles (Slack-Huddle parity).** Drop-in audio channel attached to a conversation or workspace channel. No invite required for members of the parent channel; "join" is one-tap. UX: bottom-bar pill always visible while in huddle; pop-out window optional.

**Stage / town-hall channel (Discord-Stage parity).** Multi-thousand-listener mode; speakers + listeners separated by role; "raise hand" → host promotes to speaker.

**Background blur / replace.** MediaPipe Selfie Segmentation on-device first; cloud fallback for older hardware. Per ADR-MSGR-0001 §privacy: blur runs on-device by default; cloud only with explicit user toggle + audit.

**Noise suppression.** RNNoise on-device first; Krisp-class cloud model optional.

**Screen share.** WebRTC `getDisplayMedia`. Per-application or full-screen. Audio capture optional.

**Recording.** Off by default. When enabled: every participant sees "Recording" banner; explicit-consent prompt at start; participants who decline → call ends or recording disabled; per ADR-MSGR-0001 §consent. Recording stored in SeaweedFS encrypted with the conversation MLS key; ACL'd per channel; eDiscovery-able for work.

**Live captions / live translation.** Intelligence substrate; on-device first (Whisper-Small); cloud-Whisper opt-in. Per ADR-MSGR-0001 §scope-2 live translation roadmap M03.

### 4.5 Stickers, custom emoji, GIFs

**Stickers.**
- Built-in pack (oyatie default) + Plugin App Store sticker marketplace.
- User can upload custom stickers (Lottie + WebP + APNG + static PNG up to 512×512).
- Sticker tray: recent + favourites + categories.
- Tap-to-send; hold-to-preview.
- Stickers are E2E-encrypted alongside other content (URL ref + content-hash; content fetched via signed CDN URL with E2E-derived path).

**Custom emoji.**
- Per-tenant + per-personal-namespace.
- Discord-class `:name:` resolution.
- Upload via Settings → Emoji or `/emoji add <name>` slash.

**GIFs.**
- Tenor primary API; Giphy fallback (per ADR-MSGR-0001 §gif-providers).
- Search-as-you-type in the GIF picker.
- Auto-loop on render.
- Tenant policy can disable GIFs (e.g., for compliance-locked B2B tenants).

### 4.6 Stories (24h ephemeral, per ADR-MSGR-0001 §stories)

- 24h default TTL; user can set 1h / 6h / 12h / 24h.
- Audience scope per story: All contacts / Selected list / Close friends / Exclude list (KakaoTalk + Instagram pattern combined).
- Story types: photo, video (≤ 60s), text-with-background, voice-message-with-waveform.
- Story reactions: emoji + reply-to-story (which becomes a 1:1 message).
- Story view receipts: who viewed.
- Stories are E2E-encrypted; each story is a one-message MLS group keyed to the audience scope.

### 4.7 Channels (broadcast, Telegram + WhatsApp Channels 2024 parity)

- Public read-only channels: anyone with the link can follow.
- Channel owner posts; followers can react + comment (if owner enables).
- View count visible to owner; per-follower view-receipt configurable.
- Channels can federate via Matrix or ActivityPub (per ADR-MSGR-0003); follower can be a Mastodon/Lemmy actor.
- Channel feed appears in messenger Inbox alongside DMs; user can pin or mute.

### 4.8 Federation (per ADR-MSGR-0003)

- **Matrix r0.6.1 Client-Server + r0.1.4 Server-Server**: canonical-base federation. Oyatie homeserver is a Matrix homeserver; users can interact with `@user:matrix.org`, `@user:element.io`, etc.
- **ActivityPub** (B2C personal opt-in only): WebFinger discovery; `Note` for posts; channels as `Group` actor. Mastodon/Lemmy follower can follow an oyatie channel.
- **Slack adapter**: per-tenant opt-in; bridges to an external Slack workspace by tenant admin pairing.
- **Bridge security**: cross-vendor decryption events are audit-chained; tenant admin must approve federation at the tenant level + every cross-vendor channel pairing.

### 4.9 Status, stories, presence

- **Custom status**: one-line text + emoji + expiry. Visible to contacts.
- **Presence**: online / away / DND / invisible. DND silences notifications but messages still queue. Invisible: server reports last-seen as configured (off / contacts-only / everyone).
- **Last-seen**: configurable; default off in B2C (Signal-default).
- **Typing indicator**: per-conversation; per-user toggle.
- **Read receipts**: symmetric toggle — if off, you don't see others' either.

### 4.10 Migration from competitors

Import from Signal / Telegram / KakaoTalk / Line / WhatsApp / Instagram DM / FB Messenger / Discord / iMessage / Slack / Teams:

- Per-source adapter (each in `oya-messenger-import-<source>` worker crate per ADR-MSGR-0001 §import).
- Chat-history export → MIME-based archive (Signal `.tar`, Telegram JSON, WhatsApp `.txt`+attachments, Slack export, etc.).
- Preserves: message timestamps, sender, body, attachments, reactions (where exported), thread structure.
- Audit-chain emission per batch.
- Reverse-export: full conversation archive in oyatie-native format (JSON + attachments) for GDPR Art. 20 portability.

---

## 5. Work Mode (B2B) — Feature Deep-Dive

### 5.1 Workspaces, channels, threads

- **Workspace** (`acme` tenant): top-level container; per ADR-0244 one tenant = one workspace.
- **Channels** within workspace: text (`#general`, `#engineering`), voice (`#standup-room`), forum (`#design-debates`), announcement (`#all-hands`), Q&A (`#dev-help`), stage (`#town-hall`).
- **Public vs private channels**: public visible to all workspace members; private invite-only.
- **Threads**: every message can spawn a thread; thread is its own conversation surface with own read-cursor.
- **Pinned messages**: per channel; up to 100.
- **Channel bookmarks**: links + docs pinned per-channel.

### 5.2 Mentions

- `@user`: notifies user via push + in-app.
- `@here`: notifies online members of the channel.
- `@channel`: notifies all members (Cedar-gated; tenant policy may restrict to channel-owner or above).
- `@everyone`: workspace-wide (Cedar-gated; usually exec-only).
- `@role` (e.g., `@on-call`, `@engineers`): tenant-defined Cedar role.
- Cross-product mention resolution: `@user` resolves to the user's Ontology `Person` object; same identity surfaces in Mail, Community, Calendar, Drive, etc.

### 5.3 Slash commands

Built-in slash commands ship day-one:

| Command | Effect |
|---|---|
| `/me <action>` | Italic third-person action ("Alice is heading out") |
| `/away`, `/dnd`, `/online` | Set presence |
| `/in [reason]` | Clock-in via Workflow → HR/Payroll µservice; emits timesheet entry |
| `/out [reason]` | Clock-out |
| `/break <minutes>` | Break-time toggle |
| `/poll <question>` | Open a poll in-channel |
| `/remind <when> <what>` | Self or @-target reminder |
| `/giphy <query>` | Insert GIF |
| `/shrug` | Insert ¯\\\_(ツ)\_/¯ |
| `/topic <text>` | Set channel topic |
| `/purpose <text>` | Set channel purpose |
| `/invite @user` | Invite user to channel |
| `/leave` | Leave channel |
| `/mute <duration>` | Mute channel |
| `/dm @user` | Start DM |
| `/huddle` | Start huddle in current channel (ADR-MSGR-0001) |
| `/call @user` | Start 1:1 voice call |
| `/video @user` | Start 1:1 video call |
| `/meet [topic]` | Schedule a Meet (via Calendar µservice) |
| `/mail <subject>` | Open mail composer addressed to channel members |
| `/sign <attachment>` | Open e-sign flow on attached PDF (Workflow µservice) |
| `/approve` | Cast vote on an active approval card |
| `/reject [reason]` | Reject an active approval card |
| `/workflow <name>` | Trigger a tenant-defined workflow |
| `/integration <name>` | Manage plugin/bot integration |
| `/feedback <text>` | Submit feedback to tenant admin |
| `/help` | Open help |

Tenant-defined custom slash commands: defined via Workflow Studio + plugin spec; Cedar-gated; appear in slash autocomplete.

### 5.4 Workflow triggers

Every message + reaction + channel event can be a Workflow Engine trigger. Per ADR-0245 cross-product flows ALWAYS pass through Workflow + Ontology (never direct cross-product imports).

Examples:
- Mention `@on-call` in `#incidents` → triggers PagerDuty-equivalent workflow.
- Reaction `:approved:` on an approval card → advances workflow state.
- New message in `#tickets` → triggers ticket-spawn workflow.
- `/in` slash → triggers `oya-hr-payroll-timesheet-clockin` workflow.
- KB-article-publish event in `community` → posts to `#announcements` channel (subscribed).

Workflow trigger audit: every trigger emits `WorkflowTriggerRequested{source_msg_id, principal, policy_basis, workflow_item_id}` to audit-chain.

### 5.5 Mail-to-messenger bridge

- Tenant can configure: incoming mail to `<alias>@acme.com` becomes a thread in `#<channel>`.
- Outbound: a message in `#<channel>` can be mirrored as mail to a mailing-list (per Community PRD §6).
- Mail action-cards (AsyncAPI 2.6 contract) ingested into messenger as inline cards.

### 5.6 Calendar + Meet integration

- Receive iCalendar invites inline in messenger (rendered as cards).
- Accept / Decline → calendar µservice iMIP REPLY.
- `/meet` slash → calendar µservice creates meeting + posts join link to channel.
- Meeting reminders post to channel 15min before.
- Per ADR-MSGR-0001 huddles complement Meet: huddles for ad-hoc/persistent; Meet for scheduled.

### 5.7 Huddles (per ADR-MSGR-0001)

- One-tap drop-in audio in any channel.
- Persistent until last participant leaves.
- "Speakers" + "listeners" in stage-mode.
- Recording toggle (consent-gated).
- Live captions via Intelligence.
- Screen share.
- Per-channel huddle limit (Cedar-gated; default 1 per channel concurrent).

### 5.8 Clocking-in via `/in` slash

- Employee in `#clockin` channel types `/in` (optional `/in <reason>` or `/in --geo`).
- Workflow Engine fires `TimesheetEntryRequested{user, kind=clock-in, at, geo?}`.
- `oya-hr-payroll` µservice records entry (cross-product via Workflow event per ADR-0245).
- Employee receives confirmation card in-channel.
- `/out` for clock-out; `/break 30` for break.

### 5.9 E-signing in chat

- Attachment of PDF in a message → toolbar shows `/sign` option.
- `/sign @user1 @user2` triggers `oya-messenger-esignature` Workflow.
- AdES-compliant (PAdES per ETSI EN 319 142).
- Signed envelope posted back to channel; copies mailed to signers; audit-chain seals.

### 5.10 Approval workflows

- Workflow Engine can post an "approval card" message (typed Action Card).
- Card has Approve/Reject/Need-info buttons.
- Tap → emits `WorkflowApprovalCast{principal, decision, reason}`.
- Two-eyes / multi-approver flows supported.
- Audit trail on every cast.

### 5.11 DLP (data-loss prevention)

- Outbound scan on every message + attachment.
- Pattern classes: PII (SSN/RRN/credit-card/IBAN/passport), PHI (HIPAA pack), classification labels (Confidential / Restricted), tenant keyword lists.
- Cedar gates: `policy/dlp.cedar` declares allow / warn / block per pattern × recipient-scope.
- Block: send refused at submission; user shown DLP reason.
- Warn: user shown override prompt; override requires reason; audit-chained.
- Quarantine: held for tenant-admin review.

### 5.12 eDiscovery + legal hold

- Per ADR-0215 four-eyes for plaintext disclosure.
- Hold scoped (by channel, user, date range, query).
- Hold-before-purge invariant.
- Export bundle: messages + threads + attachments + audit-chain seal; Ed25519 chain-of-custody.
- Re-derivable digest.

### 5.13 SSO + provisioning

- Zitadel SAML 2.0 + OIDC (RFC 6749 + OIDC 1.0).
- SCIM 2.0 (RFC 7644).
- MFA enforced per tenant.
- JIT vs pre-provisioned configurable.

### 5.14 Admin console

- Tenant admin UI at `https://admin.<tenant>.oyatie.app/messenger`.
- Surfaces: users, channels, retention, legal holds, eDiscovery jobs, DLP rules, slash command library, workflow integrations, audit-log search, federation pairings (Slack Connect, Matrix), quota usage, FinOps, federation blocklist.

### 5.15 Federation (Slack + Matrix)

- Slack adapter: per ADR-MSGR-0001 + ADR-MSGR-0003. Tenant pairs with an external Slack workspace; cross-tenant DMs + channels route through the adapter.
- Matrix federation: r0.6.1 Client-Server + r0.1.4 Server-Server (per ADR-MSGR-0003). Tenant can pair with Matrix homeservers (Element, internal Synapse, etc.).
- Federation default OFF for B2B; tenant admin must enable + each pairing requires four-eyes audit.

---

## 6. User Stories (20+: 10 Personal + 10 Work)

### Story P-1 — Alice (Personal) starts an MLS-encrypted group chat with 5 friends

**Precondition.** Alice has a personal handle `@alice`. She's on iOS Personal Messenger app.

**Steps.**
1. Alice taps `+` → `New Group`.
2. Selects 5 contacts (Bob, Carol, Dan, Eve, Frank) from contact list. UX latency: <100ms per tap.
3. Names the group `Trip to Seoul`; taps `Create`.
4. Client constructs an MLS group: each member's primary device LeafNode + Alice's own iPhone LeafNode (6 total). Welcome messages dispatched.
5. Members' devices receive Welcome → MLS group bootstrapped; group epoch 0.
6. Alice types `Hey, who's free next weekend?` → MLS-encrypted message sealed to group → delivered to message-stream → fanned out to all 5 devices.
7. Bob's iPhone receives at p99 ≤ 100ms in-region (cross-region < 500ms).
8. Bob reacts `:plus_one:`; reaction is MLS-encrypted; tally rendered on Alice's device within 250ms.

**Expected.** Group is end-to-end encrypted; oyatie servers never see plaintext; group scales to 6 members but the same code path scales to 10k.

**Edge cases.**
- One member (Frank) is on Android with no oyatie account: invite via SMS (Signal-style) with sign-up link; group remains 5-person until Frank joins; on Frank join, MLS commit adds his LeafNode.
- Bob has 2 devices (iPhone + Mac): both LeafNodes added; both decrypt independently.
- Alice's iPhone runs out of battery mid-conversation: messages queue on the server (ciphertext); on reconnect, iPhone catches up via MLS application messages.

**Error cases.**
- Key transparency log returns an unexpected hash for Bob's key (possible MITM): client warns "Bob's safety number changed"; Alice must re-verify before continuing.
- MLS commit fails (network drop during commit): client retries with exponential backoff; conversation enters degraded mode (cannot add/remove members) but messages still send via current epoch.

### Story P-2 — Bob (Personal) makes a 1:1 video call to his mother

**Precondition.** Bob has his mother (`@mom`) in contacts. Bob is on home Wi-Fi; mom is on 4G mobile.

**Steps.**
1. Bob opens chat with mom; taps the video-call icon.
2. Client emits `CallInitiate{caller=@bob, callee=@mom, kind=video}`.
3. LiveKit SFU at nearest region issues room token; STUN/TURN credentials provisioned.
4. Mom's device rings via APNs CallKit (iOS) push (per ADR-MSGR-0001 §push-callkit).
5. Mom taps Accept; WebRTC PeerConnection setup begins.
6. ICE candidates exchanged via LiveKit signaling.
7. DTLS-SRTP handshake completes; media flows; MLS-derived PSK binds the call key to the conversation E2E key (per ADR-MSGR-0002 §call-key-binding).
8. Call connects in p99 ≤ 1s end-to-end.
9. Mom's video stream (720p simulcast, mobile-network adaptive) renders on Bob's screen.
10. Call ends after 10 min when mom taps End.
11. Call metadata (duration, codec, jitter, packet-loss) written to ClickHouse for QoE analytics; call body never recorded.

**Expected.** Call connects <1s, stays connected, ends cleanly. No oyatie operator ever has access to the media.

**Edge cases.**
- Mom's network drops mid-call: WebRTC ICE-restart attempts to recover (default 3 attempts × 5s); if unrecoverable, call ends with "Network lost" UX.
- Mom's iPhone is on a CarPlay call: video falls back to audio-only mode; CarPlay-friendly UI surfaces.
- Mom is in DND but call is from a contact-list-favourite: call rings anyway (per Bob's pre-config); other DND-overrides configurable.

**Error cases.**
- Mom declines: Bob sees `Declined`; call entry persists in history.
- TURN server unreachable (rare; coturn outage): client surfaces "Connection failed"; retry button.

### Story P-3 — Carol (Personal) sends a self-destructing photo with 10s TTL

**Precondition.** Carol is chatting with a friend; wants to send a sensitive photo.

**Steps.**
1. Carol taps the camera icon, snaps a photo.
2. Before send: she taps the timer-icon → sets TTL `10 seconds after view`.
3. Sends.
4. Friend's device receives the message; sees thumbnail with a "self-destructing" icon overlay.
5. Friend taps to view; photo opens full-screen; 10-second countdown.
6. At 10s: photo deleted from friend's local store + server-side ciphertext purged.
7. Audit (personal-context only): TTL-expire event logged locally.

**Expected.** Photo disappears for both parties after 10s view.

**Edge cases.**
- Friend takes a screenshot: Carol notified ("Friend took a screenshot at 11:42"); per ADR-MSGR-0001 §screenshot-notify.
- Friend doesn't view within 7 days: message TTL expires anyway (per ADR-MSGR-0001 §dead-letter-TTL = 7 days).
- Carol unsends within 3h: standard delete-for-everyone applies.

### Story P-4 — Dan (Personal) follows a Telegram-style broadcast channel

**Precondition.** Dan wants to follow the "NYC Cycling News" channel published by another user.

**Steps.**
1. Dan opens search; types `NYC Cycling News`.
2. Channel result; tap Follow.
3. Channel added to Inbox under "Channels" section.
4. Owner posts a new message → Dan's device receives push.
5. Dan taps `+1` reaction; emoji tally increments on owner's view.
6. Dan can't reply (channel is read-only) but can DM the owner if owner allows.

**Expected.** Channel acts as read-only broadcast; Dan receives all posts.

**Edge cases.**
- Channel federates via Matrix: a Matrix user can also follow.
- Channel federates via ActivityPub (B2C-personal opt-in): a Mastodon user can also follow.
- Channel owner mutes Dan: Dan continues to see posts but cannot react/comment.

### Story P-5 — Erin (Personal) joins a Discord-style stage town-hall

**Precondition.** Erin is a member of a hobby community; the community uses oyatie Messenger personal-server (à la Discord).

**Steps.**
1. Owner announces a Saturday 8pm AMA stage event.
2. Saturday at 7:55pm, Erin taps the `#town-hall` stage channel.
3. She enters as Listener (default; speakers are pre-assigned).
4. Live captions appear (Intelligence substrate, opt-in).
5. She "raises hand"; host promotes her to Speaker.
6. She speaks; her audio joins the LiveKit SFU stream.
7. Question over, host re-demotes to Listener.

**Expected.** Stage event scales to 1000s of listeners; speakers are ≤ 10 active.

### Story P-6 — Frank (Personal) posts a 24h story with close-friends scope

**Precondition.** Frank has a close-friends list of 12 contacts.

**Steps.**
1. Frank taps Stories → camera → snaps a photo.
2. Selects audience: Close Friends.
3. Adds caption "Beach day"; posts.
4. Story uploaded; MLS-encrypted to a one-shot group of 12 LeafNodes.
5. Close-friend Grace opens stories; sees Frank's story; viewer logged.
6. At 24h, story TTL expires; ciphertext purged; viewer-list retained 30 days then purged.

**Edge cases.**
- Frank deletes story manually before 24h: immediately purged.
- Grace replies to story: reply becomes a 1:1 DM with Frank quoting the story (story expires but reply persists).

### Story P-7 — Gina (Personal) uses live location share for a 2h meetup

**Precondition.** Gina is meeting Henry at Central Park.

**Steps.**
1. Gina opens chat; attaches → Live Location → 2 hours.
2. Henry's device receives live-location card; map shows Gina's pin updating every 30s.
3. Gina's location stream is MLS-encrypted (location is in-message-body; not metadata).
4. After 2h, share auto-stops; pin disappears.

**Edge cases.**
- Gina stops sharing early via "Stop sharing" button.
- Gina loses GPS: pin shows "Location unavailable" until restored.
- Gina sends to a group (e.g., to family): all family devices see her live pin.

### Story P-8 — Hannah (Personal) custom-emoji-reacts in a 250-person group

**Precondition.** Hannah is in a 250-person group for her university alumni network.

**Steps.**
1. Group has uploaded a custom `:alumni-cheer:` emoji.
2. Someone posts "I just got accepted to grad school!".
3. Hannah long-presses → reaction picker → searches "alum" → `:alumni-cheer:` tap.
4. Reaction MLS-encrypted; fanned out; tally renders on all 250 devices within 1s.
5. By end of day, 187 reactions tallied.

**Edge cases.**
- One member is on a slow network: reaction tally lags; eventually consistent.
- Custom emoji is renamed by group owner: existing reactions still render; new ones use new name.

### Story P-9 — Ivan (Personal) federates a chat to a Mastodon friend via Matrix

**Precondition.** Ivan's friend Jose only uses Mastodon (`@jose@mastodon.social`).

**Steps.**
1. Ivan opens search; types `@jose@mastodon.social`.
2. Matrix bridge resolves via WebFinger; offers Federated Chat option.
3. Ivan starts chat; first message routes via Matrix r0.6.1 federation.
4. Jose receives via his Mastodon DM surface; replies route back through Matrix.
5. Both see "Federated" badge in the chat header.

**Edge cases.**
- E2E unavailable across federation (Mastodon doesn't support MLS): Ivan warned "End-to-end not available outside oyatie."
- Jose's instance blocks federation with oyatie: Ivan sees "Federation blocked" error.

### Story P-10 — Jack (Personal) migrates from KakaoTalk

**Precondition.** Jack has 6 years of KakaoTalk history.

**Steps.**
1. Jack opens Settings → Import → KakaoTalk.
2. Walks through KakaoTalk's backup-to-file flow (per KakaoTalk's documented export).
3. Uploads the export file to oyatie.
4. Import worker spins up; estimates 480k messages / 12 GB / 3h.
5. Progress dashboard.
6. On completion: chats appear in oyatie Inbox with original timestamps; stickers translated to oyatie's sticker engine (compatible packs auto-mapped; non-compatible become PNG fallback).
7. Original KakaoTalk account unaffected (user-controlled cutover).

**Edge cases.**
- Some KakaoTalk-exclusive stickers don't translate: rendered as PNG with `[KakaoTalk sticker]` caption.
- Voice messages: re-encoded to Opus from KakaoTalk's AAC.

### Story W-1 — Kim (Work) creates a `#engineering` channel and invites the team

**Precondition.** Kim is Acme's engineering lead. Acme has 50 employees, SCIM-provisioned.

**Steps.**
1. Kim opens Acme workspace → `+ Channel`.
2. Names `#engineering`; visibility Public-in-workspace; topic "Engineering team discussion".
3. Cedar role `Engineering-Team` auto-binds via SCIM → channel members provisioned.
4. Kim sets channel retention to "365 days" (within tenant policy floor).
5. Kim pins the `Engineering Onboarding KB` link.
6. Posts welcome message + `@here`.
7. 25 engineers receive push notifications within 5s.
8. Audit-chain: `ChannelCreated`, `ChannelMembersGranted × 25`, `MessageSent`.

**Expected.** Channel is live in <10s; team is in.

**Edge cases.**
- One member is on PTO: notification queued; delivered on return.
- Cedar policy denies `@here`: Kim sees "Use `@channel-engineers` instead" prompt.

### Story W-2 — Liam (Work) starts a huddle with two teammates mid-flow

**Precondition.** Liam is debugging a P0 incident in `#incidents`. He needs synchronous help.

**Steps.**
1. Liam types `/huddle`.
2. Huddle starts in `#incidents` channel; persistent bottom-bar pill appears.
3. Two teammates (Mia, Noah) tap the pill to join.
4. Huddle = audio-first; Liam shares screen via `/screenshare`.
5. Live captions auto-on (tenant default).
6. Mia drops a code snippet in channel; rendered alongside huddle.
7. 30 min later, problem solved; Liam ends huddle.
8. Audit: `HuddleStarted`, `HuddleParticipantJoined × 2`, `ScreenShareStarted`, `HuddleEnded` with duration.

**Edge cases.**
- A fourth person tries to join after 30 min: huddle still active; they join freely.
- Mia has tenant DLP that blocks screen-share to certain client domains: warning shown before her share starts.

### Story W-3 — Oliver (Work) sends an approval card via Workflow

**Precondition.** Oliver is requesting expense approval >$1000; tenant has an `expense-approval` Workflow.

**Steps.**
1. Oliver opens chat with his manager Paula.
2. Types `/workflow expense-approval`.
3. Form pops up: amount, vendor, receipt PDF (attached); category.
4. Submits.
5. Workflow Engine posts an Approval Card to Paula's DM with Oliver: "Approve $1200 to Vendor X?"
6. Paula taps Approve.
7. Workflow advances; finance team notified; Oliver receives confirmation card.
8. Audit-chain: `WorkflowApprovalRequested`, `WorkflowApprovalCast{decision=approve}`.

**Edge cases.**
- Amount >$5000 → workflow requires two approvers (Paula + CFO Quincy); card posts to both; advance only after both approve.
- Paula rejects with reason: Oliver sees the rejection card; can edit + resubmit.

### Story W-4 — Rita (Work) clocks in via `/in` from her field site

**Precondition.** Rita works at a remote site; tenant uses messenger for time tracking.

**Steps.**
1. Rita opens Acme workspace mobile app at 07:42 local.
2. In `#clockin` channel: `/in starting morning shift`.
3. Workflow Engine fires `TimesheetEntryRequested{user=rita, kind=clock-in, at=07:42, reason="starting morning shift", geo=optional}`.
4. `oya-hr-payroll` (consumed via Workflow event per ADR-0245) records.
5. Rita receives confirmation card: "Clocked in at 07:42 EST".
6. End of day: `/out` clocks out.

**Edge cases.**
- Geo required by tenant policy: client requests location; if denied, entry rejected with "Geo required" message.
- Late clock-in (after 09:00 grace): workflow flags for supervisor approval.

### Story W-5 — Sam (Work) e-signs a contract attached in a DM

**Precondition.** Sam (legal) receives a contract from external counsel via federated DM.

**Steps.**
1. Sam opens DM; sees PDF attachment `MSA-FooCorp.pdf`.
2. Taps PDF → preview; toolbar shows "Send for e-signature".
3. Tap → `/sign` workflow opens; Sam adds signature field; counterparty field; selects `legal@foocorp.com` as counterparty.
4. Submits.
5. Workflow creates signed envelope (PAdES-compliant per ETSI EN 319 142).
6. Counterparty receives signing-link via mail.
7. Counterparty signs.
8. Both parties receive countersigned PDF in their DMs.

**Edge cases.**
- Counterparty doesn't sign in 7 days: reminder sent.
- Counterparty rejects: workflow ends; both notified; audit-chained.

### Story W-6 — Tara (Work) outbound message hits DLP

**Precondition.** Tara accidentally tries to paste a credit-card number into `#general`.

**Steps.**
1. Tara composes: "Customer card: 4532-1234-5678-9012".
2. Tara presses Send.
3. DLP scan fires: PII pattern (credit-card Luhn match) → block (per tenant policy).
4. Tara sees: "Message blocked — Card number detected. [Edit] [Override (requires reason)]"
5. Tara clicks Override → enters "False positive; this is a test card."
6. Cedar policy requires admin approval for override → message held for review.
7. Audit: `DlpBlockApplied`, `DlpOverrideRequested`.

**Edge cases.**
- Tenant policy allows override without admin approval (lower-tier classification): message sends with audit-chained override reason.
- Tenant policy is `quarantine`: message held for tenant-admin review; Tara notified.

### Story W-7 — Uma (Work) sets a vacation status + delegates `@on-call`

**Precondition.** Uma is going on PTO 2 weeks.

**Steps.**
1. Uma opens Settings → Status → Vacation.
2. Configures: 2026-07-01 to 2026-07-14; status `On vacation; reach @vincent`.
3. Toggles "Auto-reply to DMs" with same message.
4. Reassigns `@on-call` Cedar role to Vincent for the period.
5. Audit: `StatusSet`, `RoleDelegatedForPeriod`.
6. Mid-vacation, someone DMs Uma → auto-reply fires + Vincent gets `@on-call` notification.

**Edge cases.**
- Uma sends a manual reply during vacation: auto-reply not re-sent for that sender (per RFC 3834 rate-limit).
- Vincent acks the on-call notification: workflow tracks acknowledgement.

### Story W-8 — Veronica (Work) executes a legal hold on a departing employee's DMs

**Precondition.** Compliance officer Veronica has a litigation matter; needs to hold employee Walt's communications.

**Steps.**
1. Veronica opens Admin Console → Legal Holds → New Hold.
2. Scopes: principal `walt@acme.com`; date range `2026-01-01 to present`; channels `*`; DMs `*`.
3. Configures reason, matter-id; submits.
4. Cedar policy + four-eyes: Veronica + co-counsel Xander must both approve.
5. Xander approves.
6. Hold engages; future purge events for Walt's messages refused.
7. Audit: `LegalHoldEngaged{matter, scope, principals}`.
8. Walt's messages preserved beyond their normal retention.
9. Later: Veronica issues eDiscovery export; bundle returned with Ed25519 chain-of-custody.

**Edge cases.**
- Walt's messages include MLS-encrypted personal DMs: hold preserves ciphertext only; plaintext disclosure requires separate four-eyes per ADR-0215.
- Hold release: same four-eyes pattern.

### Story W-9 — Yvonne (Work) federates with an external Slack workspace via Slack adapter

**Precondition.** Acme works with vendor BarCorp; both have messenger workspaces. BarCorp uses Slack.

**Steps.**
1. Yvonne (Acme admin) and Zane (BarCorp admin) coordinate: Yvonne enables Slack adapter in admin console; pairs Acme tenant with BarCorp Slack workspace.
2. Per ADR-MSGR-0003 federation pairing: four-eyes (Yvonne + co-admin Aaron approve).
3. Cross-workspace channel `acme-barcorp-collab` created.
4. Members from both sides join.
5. Messages flow bidirectionally; adapter decrypts oyatie-side MLS, re-encrypts to Slack TLS; audit-chained.
6. Slack-side users see standard Slack channel; oyatie-side users see federation badge.

**Edge cases.**
- BarCorp doesn't support file types Acme uses: adapter rejects or transcodes (e.g., AV1 video → H.264 for Slack compat).
- Federation paused mid-flow: existing messages persist; new messages queue.

### Story W-10 — Bea (Work) sets up a mail-to-channel bridge for `#announcements`

**Precondition.** Acme has an internal mailing list `announcements@acme.com`. Bea wants new mails to appear in `#announcements`.

**Steps.**
1. Bea opens `#announcements` settings → Integrations → Mail Bridge.
2. Configures: inbound from `announcements@acme.com` (Mail µservice alias) → post to this channel.
3. Cedar policy allows; saves.
4. A new mail arrives at `announcements@acme.com`.
5. Mail µservice (per ADR-0245 cross-product via Workflow + Ontology) emits `MailReceived` event.
6. Messenger ingests action-card; posts a card in `#announcements` with sender, subject, link to full mail.
7. Tap card → opens full mail in side panel.

**Edge cases.**
- Mail has 50 MB PDF: link to drive substrate (auto-uploaded); card shows preview thumbnail.
- Mail is encrypted (PGP): card shows "Encrypted; click to view in Mail."

---

## 7. UX Strive / Avoid

### 7.1 Strive

1. **Signal-grade E2E by default**: MLS RFC 9420 always on for personal; conversation never enters plaintext at the server.
2. **Telegram-class scale + speed**: large groups (10k members), supergroups, broadcast channels, all with <100ms p99 send.
3. **KakaoTalk-grade sticker UX**: sticker tray with categories, recents, favourites; tap-to-send; hold-to-preview.
4. **Line-class regional polish**: KR/JP first-class localisation; CJK IME perfect; regional payment integrations roadmap.
5. **WhatsApp-class simplicity**: one-tap call, one-tap reply, no friction onboarding via phone-number-first.
6. **Instagram-DM-grade stories + vanish mode**: 24h ephemeral; close-friends; vanish mode for one-shot disposable chats.
7. **Discord-grade voice channels + huddles + custom emoji + roles**: voice channels per server; drop-in audio; per-tenant emoji marketplace.
8. **iMessage-grade reactions + Continuity**: Tapbacks; multi-device with linked-device pattern; key-transparency.
9. **Slack-grade channels + threads + mentions + workflow**: workspace shape; threaded conversation per message; native Workflow Engine triggers.
10. **Matrix-grade federation**: open protocol; cross-vendor; sovereign hosting available via self-hosted oyatie homeserver.
11. **Touch-first AND keyboard-first**: full keyboard nav for power users; touch ergonomics for mobile.
12. **Real-time presence + typing without battery drain**: WebSocket coalescing; <1% battery/h baseline.
13. **WCAG 2.2 AA from day one**: not retrofitted; CI-lane verified.
14. **17 day-one languages**: en, ko, ja, zh-Hans, zh-Hant, es, pt, fr, de, it, nl, ru, ar, hi, id, vi, th.

### 7.2 Avoid

1. **Metadata-leak via cleartext envelope**: sealed sender on every personal message; per ADR-MSGR-0002 §metadata-minimisation.
2. **Slack-style ephemerality where users expect persistence**: messages persist by default; ephemeral mode is opt-in per conversation.
3. **Discord-style notification overload**: `@everyone` Cedar-gated by default in B2B; default off in personal servers >100 members.
4. **Read-receipt asymmetry**: if user opts out, they don't see others' either (Signal-pattern; prevents one-sided surveillance).
5. **Phone-number tracking**: phone number is one identity option, not mandatory; handle (`@alice`) is canonical.
6. **WhatsApp/Meta-style data-sharing**: oyatie is the platform-tenant per ADR-0242; no data-shared-with-affiliates pattern; user owns their data.
7. **iMessage-style platform lock-in**: cross-platform (iOS/Android/Web/Desktop) at day-one; federation via Matrix for cross-vendor.
8. **Channel proliferation chaos**: tenant-level channel-naming convention + Cedar role-gated channel creation; archive-old-channels nudge.
9. **Notification noise**: thread/conversation muting; notification scheduling (Do Not Disturb hours); smart-summary catchup.
10. **Hidden block / mute**: blocked-user UX is explicit; "you have blocked this user" badge surfaced; appeal flow.
11. **Server-stored history without E2E**: even in B2B, channel history is tenant-DEK encrypted at rest; admin reads require four-eyes.
12. **AI-trained-on-your-messages**: any Intelligence feature (transcription, translation, smart-reply) is per-user opt-in; tenant-cell model only; DP-noise; no cross-tenant training.

---

## 8. Substrate Dependencies

Per ADR-0245 §D-4 cross-tier dependency rules: `messenger` (hero product) consumes substrates only; never imports another product directly. Cross-product flows go through Workflow events + Ontology object reads/writes.

| Substrate | Purpose | SLO floor consumed |
|---|---|---|
| `identity` | Zitadel OIDC + WebAuthn + phone-IdP; MLS AS for KeyPackages | 99.99% |
| `tenancy` | Tenant + sub-scope resolution; dual-context isolation root | 99.99% |
| `cell` | Per-tenant + per-region cell provisioning; LiveKit SFU per cell | 99.99% |
| `audit-chain` | Merkle/Ed25519 seal on every state change | 99.99% |
| `policy-engine` | Cedar evaluation at every action; per-hop in moderation pipeline | 99.99% |
| `observability` | Metrics, traces, dashboards, OpenSLO authoring | 99.99% |
| `cloud-secrets` | KMS / OpenBao for tenant DEKs + MLS PSK escrow | 99.99% |
| `consent-graph` | DSAR cascade; consent state; Cedar consent gates | 99.99% |
| `governance` | Fitness gates; oya-check-* lanes | 99.99% |
| `compliance` | Per-pack overlay enforcement (KR/EU/US-HC/etc.) | 99.99% |
| `api-gateway` | Request routing, per-tenant rate limit, QUIC termination | 99.99% |
| `network` | Service mesh, NetworkPolicy authoring | 99.99% |
| `cloud-k8s` | Compute scheduling | 99.99% |
| `cloud-iac` | Helm chart + IaC module registry | 99.99% |
| **Product peers (via Workflow + Ontology only)** | | |
| `workflow-engine` | Durable workflows (huddles, e-signing, approvals, clock-in) | 99.95% |
| `intelligence` | Transcription, translation, smart-reply, summarisation, classifier | 99.95% |
| `ontology` | Cross-product entity links; mention resolution; Person/Team/Channel/MessageThread | 99.99% |
| `mail` | Mail-to-channel bridge; action-card carrier | 99.95% |
| `calendar` | Meeting invite render; iMIP; `/meet` slash | 99.95% |
| `meet` | Scheduled meetings; complements huddles | 99.95% |
| `drive` | File attachment cross-link; file picker | 99.95% |
| `community` | Cross-product mention resolution; KB-publish-to-channel | 99.95% |
| `hr-payroll` | Clock-in/out + timesheet entries (via Workflow event) | 99.95% |
| `notifications` | Push (APNs + FCM + Web Push); in-app | 99.95% |

---

## 9. Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | end-user | to send 1:1 DM with MLS E2E by default | conversation is end-to-end encrypted | direct-messaging, e2e-encryption | Must |
| FR-02 | end-user | to send group message with MLS E2E by default | group is end-to-end encrypted | group-messaging, e2e-encryption | Must |
| FR-03 | end-user | to create a large group (up to 10k) without performance cliff | scale doesn't break UX | group-messaging | Must |
| FR-04 | end-user | to start/follow a broadcast channel | I can publish + consume one-to-many | channels | Must |
| FR-05 | end-user | to reply in a thread off a parent message | conversation stays scoped | threads | Must |
| FR-06 | end-user | to @mention people, channels, roles | recipients are notified + linked | direct-messaging, group-messaging | Must |
| FR-07 | end-user | to react inline with emoji + custom emoji | low-overhead acknowledgement | direct-messaging, group-messaging | Must |
| FR-08 | end-user | to edit my message within window | typos fixable | direct-messaging, group-messaging | Must |
| FR-09 | end-user | to delete my message for self or for everyone | mistakes reversible | direct-messaging, group-messaging | Must |
| FR-10 | end-user | to set self-destruct TTL on a conversation or message | sensitive content auto-purges | direct-messaging | Must |
| FR-11 | end-user | to schedule a message for future send | timezone-friendly | direct-messaging, group-messaging | Should |
| FR-12 | end-user | to send a voice message with waveform + transcription | hands-free messaging | direct-messaging, group-messaging | Must |
| FR-13 | end-user | to start a 1:1 voice call | synchronous comms work | voice-calls | Must |
| FR-14 | end-user | to start a 1:1 video call | synchronous video works | video-calls | Must |
| FR-15 | end-user | to start a group voice/video call (up to 100 active + 1k view) | team meets without scheduling | voice-calls, video-calls | Must |
| FR-16 | end-user | to drop into a huddle in a channel (per ADR-MSGR-0001) | ad-hoc team sync | huddles | Must |
| FR-17 | end-user | to host a stage / town-hall channel (1000s of listeners) | broadcast events work | huddles | Must |
| FR-18 | end-user | to share my screen during a call | walk-through works | voice-calls, video-calls, huddles | Must |
| FR-19 | end-user | to use background blur / noise suppression | privacy + clarity | voice-calls, video-calls | Must |
| FR-20 | end-user | to attach files (up to 5 GB; up to 100 GB via drive link) | rich collaboration | attachments | Must |
| FR-21 | end-user | to attach photo / video with quality toggle | media-rich messaging | attachments | Must |
| FR-22 | end-user | to use built-in + custom stickers | expressive messaging | stickers-emoji | Must |
| FR-23 | end-user | to insert GIF via Tenor/Giphy picker | popular UX | stickers-emoji | Must |
| FR-24 | end-user | to create polls (single / multi / ranked) | sentiment + decisions | direct-messaging, group-messaging | Must |
| FR-25 | end-user | to share location (one-shot or live timed) | meetups work | direct-messaging | Must |
| FR-26 | end-user | to share contact (vCard) | introductions work | direct-messaging | Must |
| FR-27 | end-user | to post a 24h story with audience scope | ephemeral broadcast | direct-messaging | Should |
| FR-28 | end-user | to see read receipts (configurable per-user) | I know who has seen | direct-messaging, group-messaging | Must |
| FR-29 | end-user | to see typing indicator | conversation rhythm | direct-messaging, group-messaging | Must |
| FR-30 | end-user | to see presence (online / away / dnd / custom) | I time my comms | presence-status | Must |
| FR-31 | end-user | to set my status (text + emoji + expiry) | I communicate availability | presence-status | Must |
| FR-32 | end-user | to use full MLS E2E with multi-device | privacy with convenience | e2e-encryption, multi-device-sync | Must |
| FR-33 | end-user | to back up my history with passphrase-encrypted bundle | I can recover after device loss | multi-device-sync | Should |
| FR-34 | end-user | to use mobile + web + desktop client | multi-device works | multi-device-sync | Must |
| FR-35 | end-user | to use bots + slash commands + plugins | rich integrations | direct-messaging, group-messaging | Must |
| FR-36 | end-user | to search messages I can read (Cedar-filtered) | recover context | search | Must |
| FR-37 | end-user | to archive a conversation | inbox cleanliness | archive-retention | Must |
| FR-38 | end-user | to mute a conversation | notification control | notifications | Must |
| FR-39 | end-user | to block a contact | personal safety | direct-messaging | Must |
| FR-40 | end-user | to pin a conversation | quick access | direct-messaging, group-messaging | Must |
| FR-41 | end-user | to use dark mode + custom themes + chat wallpapers | aesthetic control | direct-messaging, group-messaging | Should |
| FR-42 | end-user | to use accessibility (screen reader + keyboard nav + WCAG 2.2 AA) | universal access | direct-messaging, group-messaging | Must |
| FR-43 | end-user | to use the messenger in 17 languages day-one | global UX | direct-messaging, group-messaging | Must |
| FR-44 | end-user | to federate (Matrix) with external users | cross-vendor comms | federation | Should (M02-pack) |
| FR-45 | B2C personal | to federate (ActivityPub) for channels | fediverse interop | federation | Should (M02-pack) |
| FR-46 | tenant operator | to enable Slack adapter | cross-vendor B2B comms | federation | Should (M02-pack) |
| FR-47 | tenant operator | to configure SSO (SAML/OIDC) + SCIM provisioning | enterprise identity | direct-messaging, group-messaging, channels | Must (B2B) |
| FR-48 | tenant operator | to define custom roles + Cedar permissions | tenant-tuned authorization | channels | Must (B2B) |
| FR-49 | tenant operator | to set per-channel + per-tenant retention | regulatory compliance | archive-retention | Must (B2B) |
| FR-50 | compliance-officer | to issue legal hold + eDiscovery export | regulatory satisfied | archive-retention, dlp | Must (B2B) |
| FR-51 | tenant operator | to define DLP rules (PII/PHI/keyword) | data-loss prevention | dlp | Must (B2B) |
| FR-52 | tenant operator | to integrate mail-to-channel bridge | unified ops surface | direct-messaging, group-messaging | Should (B2B) |
| FR-53 | tenant member | to trigger workflows via slash, mention, or reaction | workflow integration native | workflow-triggers | Must (B2B) |
| FR-54 | tenant member | to clock-in via `/in` slash | timesheet via messenger | workflow-triggers | Should (B2B) |
| FR-55 | tenant member | to e-sign a PDF in chat | contracts without leaving messenger | workflow-triggers | Should (B2B) |
| FR-56 | tenant member | to send/receive approval cards | structured workflow advances | workflow-triggers | Should (B2B) |
| FR-57 | DSAR requester | to exercise GDPR Art. 17 / PIPA Art. 36 erasure | data removed per ADR-0242 cascade | archive-retention | Must |
| FR-58 | DSAR requester | to export full personal data (DSAR Art. 20) | portability | archive-retention | Must |
| FR-59 | webhook subscriber | to receive outbound webhooks for message/reaction/channel events | external integrations | direct-messaging, group-messaging | Should |
| FR-60 | tenant member | to migrate import from Signal/Telegram/KakaoTalk/Line/WhatsApp/IG/FB/Discord/iMessage/Slack/Teams | switchers move in | archive-retention | Should (M02) |

---

## 10. Non-Functional Requirements

### 10.1 Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Message send (1:1 in-region) | ≤ 25 ms | ≤ 100 ms | ≤ 300 ms | Within-region; sealed-sender envelope construction included |
| Message send (group ≤ 50) | ≤ 50 ms | ≤ 150 ms | ≤ 500 ms | MLS encrypt + fan-out |
| Group fan-out (1k members) | ≤ 60 ms | ≤ 200 ms | ≤ 700 ms | MLS application-message fan-out |
| Group fan-out (10k members) | ≤ 200 ms | ≤ 1 s | ≤ 3 s | MLS at scale |
| Cross-region delivery | ≤ 200 ms | ≤ 500 ms | ≤ 1.5 s | Multi-region async replication |
| Voice/video call start (1:1) | ≤ 400 ms | ≤ 1 s | ≤ 2.5 s | ICE + DTLS-SRTP + MLS PSK |
| Voice/video call start (group ≤ 50) | ≤ 700 ms | ≤ 1.5 s | ≤ 3 s | LiveKit SFU room provision |
| Huddle start | ≤ 400 ms | ≤ 1 s | ≤ 2.5 s | Per ADR-MSGR-0001 |
| Presence update propagation | ≤ 200 ms | ≤ 1 s | ≤ 3 s | Valkey pub/sub + WebSocket |
| Typing indicator | ≤ 100 ms | ≤ 500 ms | ≤ 1.5 s | Coalesced 250ms windows |
| Read-receipt fan-out | ≤ 50 ms | ≤ 150 ms | ≤ 500 ms | Coalesced under 250ms windows |
| File-attachment upload init (5 MB chunk) | ≤ 100 ms | ≤ 300 ms | ≤ 1 s | tus.io multipart |
| File-attachment full upload (100 MB; Wi-Fi) | ≤ 10 s | ≤ 30 s | ≤ 90 s | parallel chunk |
| Message search (≤ 50 results, single tenant) | ≤ 80 ms | ≤ 350 ms | ≤ 1 s | Meilisearch primary; Tantivy fallback |
| @mention resolution + notification | ≤ 80 ms | ≤ 250 ms | ≤ 1 s | Ontology lookup + fanout |
| Audit-chain seal latency | ≤ 200 ms | ≤ 800 ms | ≤ 1.5 s | Per ADR-0028 |
| Sticker / GIF render | ≤ 30 ms | ≤ 100 ms | ≤ 300 ms | Local cache + CDN |
| Story upload + delivery | ≤ 300 ms | ≤ 1 s | ≤ 3 s | MLS to audience |
| Workflow trigger emission | ≤ 50 ms | ≤ 200 ms | ≤ 700 ms | Workflow Engine ingest |
| Per-cell QPS | — | 50k | — | 50k messages/sec/cell sustained |
| Concurrent WebSocket/cell | — | 1M | — | 1M concurrent connections/cell |

**Default protocol: HTTP/3 / QUIC** at the edge for all client connections; falls back to HTTP/2 over TLS 1.3 for legacy clients. WebSocket connections upgraded over HTTP/3 (per ADR-MSGR-0001 §transport).

### 10.2 Availability + SLO

Per ADR-0241 DR tier classification:
- Message send (read + write): **T1** (≤ 15 min RTO; cross-region active-active); **99.95% monthly**.
- Voice/video call signalling: **T1**; **99.95% monthly**.
- Voice/video media (LiveKit SFU): **T2** (≤ 1 h RTO); **99.9% monthly**.
- Presence / typing: **T2**; **99.9% monthly**.
- Federation paths: **T2**; **99.5% monthly** (best-effort).
- Search index: **T2**; **99.9% monthly**.
- RTO: ≤ 15 min for message-store; RPO ≤ 30 s.

### 10.3 Scalability

- Per-tenant Postgres + Citus (per ADR-0150 sharding).
- Per-tenant SeaweedFS prefix (attachments + recordings).
- Per-tenant ClickHouse (analytics + QoE).
- Per-tenant Tantivy / Meilisearch index.
- Per-cell LiveKit SFU cluster.
- Per-tenant Valkey cluster (presence + read-receipt + ephemeral cache).
- Horizontal scale-out: WebSocket gateway HPA on CPU > 70% + queue-depth; Postgres shard-by-tenant at 1M msgs/sec aggregate per cell; Valkey cluster sharding by `(tenant_id, channel_id) mod N`.
- Per-cell capacity envelope:

| Dimension | Baseline | Max | Scale-out trigger |
|---|---|---|---|
| Active WebSocket connections | 100k | 1M | gateway CPU > 70% or queue depth > 5s |
| Messages/sec | 5k | 50k | Postgres write IOPS > 70% |
| Channels per tenant | 1k | 50k | Cardinality limit |
| Attachments/day | 100k | 1M | S3 PUT rate > 70% provisioned |
| Search index size | 100GB | 5TB | Shard count exceeded |
| Concurrent LiveKit rooms | 10k | 100k | SFU cluster CPU > 60% |

### 10.4 Security (per ADR-0243 Cedar-as-universal-gate + ADR-MSGR-0002)

- All authenticated writes carry Zitadel JWT.
- Cedar evaluated at every action; per-hop in moderation pipeline.
- Personal-context: MLS E2E by default; oyatie operators MUST NOT have plaintext access (inherited from Bominal ADR-0208).
- Work-context channels: tenant-DEK envelope encryption per ADR-0255; admin disclosure requires four-eyes audit trail per Bominal ADR-0215.
- File attachments: ClamAV + OPSWAT scan before persistence; per `runbooks/attachment-malware-quarantine.md`.
- WebSocket: mTLS-terminated; per-tenant API token bound at OpenBao with 30d rotation.
- Search index excludes redacted PII / PHI per `policy/redaction-phi.md` (pack-us-healthcare overlay).
- Cross-context routing forbidden: personal DM cannot become professional channel reply; enforced by `policy/dual-context-isolation.cedar` + `oya-check-dual-context-isolation` lane.
- Cross-tenant mention-resolution forbidden by Cedar fragment.
- Per-tenant rate limits:
  - message send ≤ 240/min/member
  - reactions ≤ 600/min/member
  - calls ≤ 30/min/member
  - file uploads ≤ 60/min/member
- All state-changing events emit Merkle/Ed25519-sealed audit records.
- All federation activities verified against signed actor key.
- Encryption: at-rest via tenant-DEK envelope; in-transit mTLS + DTLS-SRTP for media per ADR-0148.
- encryption-BYOK per-tenant via cloud-secrets / OpenBao.
- Key transparency: CONIKS-class log; clients audit on every conversation start.
- Sealed sender: server sees recipient-wrap only, not sender identity.

### 10.5 Privacy + DSAR

- Per ADR-0242 §D-4 uniform DSAR cascade.
- GDPR Art. 17: erasure tombstones author_id, purges body ciphertext + attachments; audit-chain record retained per legal floor.
- KR PIPA Art. 36: equivalent erasure path.
- GDPR Art. 20: full personal data export (JSON + attachments).
- HIPAA pack: PHI retention 6 years; PHI handling logs.
- Metadata minimisation: sealed sender; minimal-metadata storage; per-user pseudonym-rotation for analytics.

### 10.6 Data residency

- Per ADR-0117 pack-pinned: pack-kr/eu/us/jp/sg/au/in/br/ae/ksa/us-healthcare.
- B2C personal: per-user residency (user pins on signup; default by IP-geo).
- B2B work: per-tenant residency (set on tenant creation; locked thereafter unless ops-compliance approves move).
- Cross-pack routing forbidden by default; tenant-scoped federation seam in `multi-region.md`.

### 10.7 Storage backends per BC

| BC | Primary | Hot path | Search | Analytics |
|---|---|---|---|---|
| direct-messaging, group-messaging | Postgres + Citus | Valkey | Meilisearch + Tantivy fallback | ClickHouse |
| channels (broadcast) | Postgres + Citus | Valkey | Meilisearch | ClickHouse |
| threads | Postgres + Citus (materialised path) | Valkey | Meilisearch | ClickHouse |
| voice-calls, video-calls | Postgres (session log) | LiveKit SFU (transient) | n/a | ClickHouse (QoE) |
| huddles | same as calls | same | n/a | ClickHouse |
| attachments | SeaweedFS | Valkey (metadata) | Tantivy (PDF/Office text extract) | n/a |
| stickers-emoji | SeaweedFS + Postgres (catalog) | CDN edge | n/a | n/a |
| presence-status | Valkey (per-user) | — | n/a | ClickHouse (aggregate) |
| notifications | Postgres (outbox) + APNs/FCM/Web Push | — | n/a | ClickHouse |
| federation | Postgres + Matrix homeserver state | Valkey | n/a | ClickHouse |
| workflow-triggers | Postgres (event log) | — | n/a | ClickHouse |
| dlp | Postgres (rule + verdict) | Valkey | n/a | ClickHouse |
| archive-retention | Postgres (policy) + SeaweedFS (archive blobs) | — | n/a | n/a |
| search | Meilisearch primary; Tantivy embedded fallback | — | — | n/a |
| e2e-encryption | Identity-µservice (KeyPackages); Postgres (transparency log) | — | — | n/a |
| multi-device-sync | Postgres (LeafNode registry) + Valkey (sync cursor) | — | n/a | n/a |

### 10.8 DR posture (per ADR-0343)

- Manifest target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_active_active=true`, `replication_shape=active-active-multi-az-cross-region-warm`. The older §10.2 15-minute/30-second objective remains a stretch SLO, not the ADR-0343 manifest contract.
- Applicable pack floors from `specs/compliance-pack-floors.json`: HIPAA-2024 `3600s/300s` with multi-region required; KR-PIPA-2023 default `14400s/900s`; SOC2-T2 `14400s/900s`; ISO27001-2022 `14400s/3600s`; KR-CSAP-v3.1 `3600s/900s` with multi-region required. The effective maximum pack floor is ISO27001 `14400s/3600s`; messenger keeps the stricter product target because live conversations are tenant-visible.
- `failover_runbook=runbooks/dr-failover.md`, resolved at `microservices/messenger/runbooks/dr-failover.md`; backup substrates are `valkey_cluster`, `postgres_wal_g`, `object_storage_versioned`, `openbao_seal_unseal`, and `audit_chain_merkle_seal`.
- `multi_region_active_active=true` for message send, WebSocket fan-out, and call signalling; media SFU failover may degrade to reconnection rather than preserving in-flight RTP.
- Why: tenants see "message sent", read receipts, and emergency/work channels as live commitments; the 15-minute/30-second target prevents cross-context safety, audit, and operational threads from disappearing during a regional event.

### 10.9 Capacity model (per ADR-0340)

- Per-tenant baseline: `0.20 vCPU`, `384 MiB RAM`, `8 GiB storage`, `connections_per_tenant={valkey:6, postgres:4, outbound_http:6}`.
- Scaling dimension: `per_message` for fan-out, receipts, MLS recovery, huddle signalling, and attachment workflows.
- Cell placement class: `Tier-1` with manifest `pod_runtime_tier=1`, because messenger's MLS/key-recovery and real-time message paths are stronger than a plain app tier while media/search workers can shed independently.
- Autoscaling boundaries: min `3` gateway/api replicas per tenant-cell, max `80` message-path replicas before shard split; huddle SFU pools scale separately on room CPU and packet-loss SLOs.
- Why: the load profile is bursty fan-out with many idle sockets, so baseline reserves keep presence stable while shard triggers absorb channel storms and emergency/work-room spikes.

### 10.10 Sustainability + cost attribution (per ADR-0344)

- Each message, attachment, huddle, search, moderation, and notification audit row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside tenant, product, capability, provider, cell, and compliance-pack dimensions.
- Provider routing is carbon-aware for attachment processing, search rebuilds, media transcode, and analytics backfill; it is not carbon-routed for emergency messages, HIPAA incident channels, or real-time call signalling where tenant-visible safety and latency dominate.
- Tenant cost transparency surface: tenant-admin messenger usage shows message fan-out, attachment storage, huddle minutes, search index size, and moderation scans; aggregate rollups land in finops-portal within the ADR-0344 freshness target.
- Why: messenger produces high-volume audit and storage activity, so CSRD, SB-253, and SEC climate-disclosure exports need cost and emissions per conversation capability rather than platform-average estimates.

### 10.11 API versioning posture (per ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet using `Oyatie-Version` header, `/v/YYYY-MM-DD/` REST/WebSocket URL prefix, and proto3 field `string oyatie_version = 8001` on public contracts.
- SDK semver model: messenger SDKs publish `major.minor.patch`; public contract compatibility is controlled by the date carrier, not by SDK patch cadence.
- Support window: last `N=3` public versions for at least `180` days after deprecation.
- Per-tenant pinning: yes for workspace clients, bots, federation bridges, and regulated tenants that must freeze client contracts during audit windows.
- Internal-mesh exemption: yes; ADR-0145 direct gRPC over HTTP/3 remains tag-compatible and does not carry public version routing.

---

## 11. Bounded Contexts (18 BCs)

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename). Layers used per BC: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-valkey` (Valkey), `adapter-s3` (SeaweedFS), `adapter-websocket`, `adapter-search`, `adapter-livekit`, `adapter-clickhouse`, `rest`, `worker`, `sdk`, `app`.

| BC | Purpose | Key entities |
|---|---|---|
| **direct-messaging** | 1:1 DM CRUD; MLS group of 2 LeafNode-sets; per-conversation TTL; delete-for-everyone window | `DirectConversation`, `DirectMessage`, `MlsLeafNode`, `ConversationTtl` |
| **group-messaging** | Group chat (≤ 10k members); MLS group commit + application; member add/remove | `Group`, `GroupMessage`, `GroupMember`, `MlsCommit`, `MlsEpoch` |
| **channels** | Broadcast channels (read-only public/private); workspace channels (B2B Slack-shape); channel types | `Channel`, `ChannelMember`, `ChannelType`, `BroadcastFollower` |
| **threads** | Thread reply chains; parent-child traversal; thread participant tracking; per-thread read-cursor | `Thread`, `ThreadReply`, `ThreadParticipant`, `ThreadReadCursor` |
| **voice-calls** | 1:1 + group voice call signalling; ICE/STUN/TURN candidates; DTLS-SRTP setup; MLS PSK binding | `VoiceCall`, `CallParticipant`, `IceCandidate`, `DtlsSrtpKey` |
| **video-calls** | 1:1 + group video signalling; simulcast (180p/360p/720p/1080p); active-speaker detection | `VideoCall`, `SimulcastLayer`, `ActiveSpeakerStat` |
| **huddles** | Drop-in audio channel (per ADR-MSGR-0001); stage/town-hall mode; speakers + listeners | `Huddle`, `HuddleSpeaker`, `HuddleListener`, `StageEvent` |
| **attachments** | File upload (tus.io multipart); SeaweedFS blob; malware scan; preview generation; per-conversation TTL | `Attachment`, `BlobRef`, `PreviewVariant`, `MalwareScanResult`, `AttachmentTtl` |
| **stickers-emoji** | Sticker pack catalog; custom emoji registry; GIF picker (Tenor/Giphy proxy) | `StickerPack`, `Sticker`, `CustomEmoji`, `GifReference` |
| **presence-status** | Online/away/dnd/custom status; last-seen; typing indicator | `Presence`, `Status`, `LastSeenCursor`, `TypingIndicator` |
| **notifications** | Push (APNs + FCM + Web Push); in-app; mention inbox; notification preferences | `NotificationPreference`, `PushTarget`, `MentionInboxEntry`, `BatchedNotification` |
| **federation** | Matrix homeserver state; ActivityPub actor; Slack adapter; cross-vendor bridge | `MatrixHomeserverState`, `ActivityPubActor`, `SlackConnectPairing`, `BridgeDecryptionEvent` |
| **workflow-triggers** | Workflow Engine event emission; slash commands; approval cards; action cards | `SlashCommandRegistration`, `ApprovalCard`, `ActionCardEnvelope`, `WorkflowTriggerEvent` |
| **dlp** | Pattern scan; PII/PHI/keyword detection; block/warn/quarantine verdict (work-only) | `DlpRule`, `DlpVerdict`, `DlpOverrideRecord` |
| **archive-retention** | Per-conversation + per-tenant retention policy; legal hold; eDiscovery export | `RetentionPolicy`, `Hold`, `EDiscoveryJob`, `ArchiveBundle` |
| **search** | Cross-BC ranked search; Cedar-filtered server-side | `SearchIndex`, `SearchQuery`, `SearchHit`, `SavedSearch` |
| **e2e-encryption** | MLS RFC 9420 implementation; KeyPackage registry; transparency log; key escrow (opt-in) | `KeyPackage`, `MlsCredential`, `TransparencyLogEntry`, `EscrowedRecoveryKey` |
| **multi-device-sync** | LeafNode registry per user; device-add MLS commit; history-share negotiation; per-device read-cursor | `Device`, `DeviceLeafNode`, `DeviceLink`, `SyncCursor`, `HistoryShareDecision` |

Total crates introduced (18 BCs × ~6 layers avg + workers + apps): **~120 crates** within `microservices/messenger/`.

Naming justification — `direct-messaging`:

```
NAME: oya-messenger-direct-messaging-<layer>
JUSTIFICATION:
- microservice = messenger: per ADR-0131 per-microservice flat layout.
- bc-tokens = direct-messaging: primary BC; v4.1 BNF allows hyphenated multi-token BCs.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-postgres / -adapter-livekit / -adapter-websocket
  are canonical *-adapter-<backend> per ADR-0105 Amendment 3.
```

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `DirectConversationRepository` | `oya-messenger-direct-messaging-kernel` | `-adapter-postgres` | `BEHAVIORAL_PERSONAL` |
| `GroupRepository` | `oya-messenger-group-messaging-kernel` | `-adapter-postgres` | `BEHAVIORAL_PERSONAL` or `BEHAVIORAL_TENANT_PRODUCT` |
| `ChannelRepository` | `oya-messenger-channels-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `ThreadRepository` | `oya-messenger-threads-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `MessageStore` | (shared in direct + group + channels) `-kernel` | `-adapter-postgres` | `BEHAVIORAL_*`, sometimes `PII_IDENTIFYING`, `PHI` (pack-us-healthcare) |
| `MessageSearchIndex` | `-search-kernel` | `-adapter-search` (Meilisearch + Tantivy backends) | `BEHAVIORAL_TENANT_PRODUCT` |
| `RealtimeBroadcaster` | `direct-messaging-kernel`, `group-messaging-kernel`, `channels-kernel`, `threads-kernel` | `-adapter-websocket` | `BEHAVIORAL_*` |
| `CallSignaller` | `voice-calls-kernel`, `video-calls-kernel`, `huddles-kernel` | `-adapter-livekit` | `BEHAVIORAL_TENANT_PRODUCT` |
| `AttachmentBlobStore` | `attachments-kernel` | `-adapter-s3` (SeaweedFS) | `BEHAVIORAL_*`, sometimes `PII_IDENTIFYING` / `PHI` |
| `MalwareScanner` | `attachments-kernel` | `-adapter` (OPSWAT / ClamAV) | `INTERNAL_ONLY` |
| `MentionResolver` | (shared) `-kernel` | `-adapter` (Ontology client) | `PII_IDENTIFYING` |
| `PresenceStore` | `presence-status-kernel` | `-adapter-valkey` (Valkey) | `BEHAVIORAL_*` |
| `NotificationDispatcher` | `notifications-kernel` | `-adapter` (APNs + FCM + Web Push) | `BEHAVIORAL_*` |
| `MlsKeyAgreement` | `e2e-encryption-kernel` | `-adapter` (OpenMLS Rust) | `INTERNAL_ONLY` (keys never leave device for personal) |
| `KeyTransparencyLog` | `e2e-encryption-kernel` | `-adapter-postgres` + `-adapter-merkle` | `AUDIT` |
| `DeviceRegistry` | `multi-device-sync-kernel` | `-adapter-postgres` | `BEHAVIORAL_*` |
| `WorkflowEventEmitter` | `workflow-triggers-kernel` | `-adapter` (Workflow Engine client) | `BEHAVIORAL_TENANT_PRODUCT` |
| `DlpScanner` | `dlp-kernel` | `-adapter` (rule engine) | `INTERNAL_ONLY` |
| `RetentionEnforcer` | `archive-retention-kernel` | `-adapter-postgres` + `-adapter-s3` | `AUDIT` |
| `FederationBridge` | `federation-kernel` | `-adapter` (Matrix + ActivityPub + Slack-Connect) | `BEHAVIORAL_*` |
| `CedarChannelPolicy` | `channels-kernel`, others | `-adapter` (Cedar evaluator) | `INTERNAL_ONLY` |
| `AuditChainClient` | every BC `-kernel` | `-adapter` to audit-chain µservice | `AUDIT` |

Data-class enforcement: `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `messenger` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice messenger` — dependency-direction
- `oya gate validate lean-a2 --microservice messenger` — cross-product-refusal
- `oya gate validate port-location --microservice messenger`
- `oya gate validate layer-correctness --microservice messenger`
- `oya gate validate per-microservice-layout --microservice messenger`
- `oya gate validate statelessness --microservice messenger`
- `oya gate validate shardability --microservice messenger`
- `oya gate validate authority-cohesion --microservice messenger` (HG-MESSENGER)
- `oya gate validate dual-context-isolation --microservice messenger` (per ADR-0238)
- `oya gate validate mls-rfc-9420-conformance --microservice messenger` (per ADR-MSGR-0002)
- `oya gate validate webrtc-spec-conformance --microservice messenger` (per ADR-MSGR-0001)
- `oya gate validate cedar-policy-coverage --microservice messenger`
- `oya gate validate data-class-annotated --microservice messenger`
- `oya gate validate openslo-authored --microservice messenger`

---

## 12. Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | A 1:1 MLS-encrypted DM send + delivery completes p99 ≤ 100 ms in-region | `microservices/messenger/tests/e2e/dm-send-latency.rs` |
| AC-02 | A group MLS DM with 1000 members fans out p99 ≤ 200 ms | `tests/e2e/group-fanout-1k.rs` |
| AC-03 | A group MLS DM with 10000 members fans out p99 ≤ 1 s | `tests/e2e/group-fanout-10k.rs` |
| AC-04 | Personal-context DM cannot become reply on professional channel | `tests/e2e/dual-context-isolation.rs` |
| AC-05 | Professional channel admin read of bodies requires two distinct approving principals + audit-chain seal | `tests/e2e/four-eyes-disclosure.rs` |
| AC-06 | A 1:1 voice call connects p99 ≤ 1 s and binds MLS PSK to DTLS-SRTP | `tests/e2e/voice-call-start.rs` |
| AC-07 | A 1:1 video call connects p99 ≤ 1 s with simulcast | `tests/e2e/video-call-start.rs` |
| AC-08 | A group voice call scales to 100 active speakers + 1000 view-only participants | `tests/e2e/group-voice-scale.rs` |
| AC-09 | A huddle starts p99 ≤ 1 s and persists until last participant leaves | `tests/e2e/huddle-lifecycle.rs` |
| AC-10 | A stage event scales to 1000 listeners + 10 speakers | `tests/e2e/stage-event-scale.rs` |
| AC-11 | A file attachment uploads via tus.io resumable across a forced network drop | `tests/e2e/attachment-resumable.rs` |
| AC-12 | A self-destructing message with 10s view-TTL is purged on both ends within 11s of view | `tests/e2e/self-destruct-ttl.rs` |
| AC-13 | A scheduled-send message delivers within 1 minute of the scheduled time | `tests/e2e/schedule-send.rs` |
| AC-14 | A voice message transcribes via Intelligence substrate (opt-in) and surfaces transcript | `tests/e2e/voice-transcript.rs` |
| AC-15 | A 24h story posts, accumulates view-receipts, and purges on expiry | `tests/e2e/story-lifecycle.rs` |
| AC-16 | A custom emoji uploaded by user renders in all client platforms | `tests/e2e/custom-emoji-render.rs` |
| AC-17 | A poll closes at the scheduled date and tallies ranked-choice via Borda count | `tests/e2e/poll-rcv.rs` |
| AC-18 | A live-location share streams for 2h and terminates automatically | `tests/e2e/live-location.rs` |
| AC-19 | A multi-device user (3 devices) receives every message on every device | `tests/e2e/multi-device-sync.rs` |
| AC-20 | A passphrase-encrypted backup can be restored on a new device | `tests/e2e/encrypted-backup-restore.rs` |
| AC-21 | A key-transparency-log audit detects an equivocated KeyPackage | `tests/e2e/key-transparency-equivocation.rs` |
| AC-22 | A federated chat with a Matrix user (r0.6.1) round-trips messages | `tests/e2e/matrix-federation.rs` |
| AC-23 | An ActivityPub follow from a Mastodon user receives broadcast-channel posts | `tests/e2e/activitypub-channel-follow.rs` |
| AC-24 | A Slack Connect-paired channel routes messages bidirectionally | `tests/e2e/slack-connect-bridge.rs` |
| AC-25 | A slash command `/in` records a timesheet entry via Workflow Engine | `tests/e2e/clock-in-slash.rs` |
| AC-26 | An e-sign workflow on a PDF attachment completes with PAdES-compliant signature | `tests/e2e/esignature-pades.rs` |
| AC-27 | An approval card with two required approvers advances only after both approve | `tests/e2e/two-approver-card.rs` |
| AC-28 | A DLP block on a credit-card pattern refuses outbound send | `tests/e2e/dlp-credit-card-block.rs` |
| AC-29 | A legal hold preserves messages beyond their retention TTL | `tests/e2e/legal-hold-preserve.rs` |
| AC-30 | An eDiscovery export bundles message + attachment + audit-chain seal with Ed25519 chain-of-custody | `tests/e2e/ediscovery-export.rs` |
| AC-31 | A GDPR Art. 17 erasure tombstones author_id across all conversations | `tests/e2e/gdpr-erasure.rs` |
| AC-32 | A GDPR Art. 20 export delivers complete personal data archive | `tests/e2e/gdpr-portability.rs` |
| AC-33 | A search query returns only Cedar-permitted results | `tests/e2e/search-cedar-scope.rs` |
| AC-34 | A federation pairing requires four-eyes approval | `tests/e2e/federation-four-eyes.rs` |
| AC-35 | A WCAG 2.2 AA audit of the web client passes | `tests/a11y/wcag-22-aa.rs` |
| AC-36 | The mobile + web + desktop clients render in all 17 day-one languages | `tests/i18n/17-languages.rs` |
| AC-37 | `oya gate validate per-microservice-layout --microservice messenger` exit 0 | ADR-0131 lane |
| AC-38 | `oya gate validate authority-cohesion --microservice messenger` exit 0 | ADR-0123 lane; HG-MESSENGER registered |
| AC-39 | `oya gate validate dual-context-isolation --microservice messenger` exit 0 | ADR-0238 lane |
| AC-40 | `oya gate validate mls-rfc-9420-conformance --microservice messenger` exit 0 | ADR-MSGR-0002 lane |
| AC-41 | `oya gate validate webrtc-spec-conformance --microservice messenger` exit 0 | ADR-MSGR-0001 lane |
| AC-42 | HTTP/3 / QUIC is the default transport for all client connections | `tests/e2e/transport-quic-default.rs` |
| AC-43 | Per-cell QPS sustains 50k messages/sec without queue depth >5s | `tests/load/per-cell-50k-qps.rs` |
| AC-44 | Per-cell concurrent WebSocket count sustains 1M connections | `tests/load/per-cell-1m-ws.rs` |
| AC-45 | 99.95% T2 monthly availability on T1 surfaces (message send, call signal) | OpenSLO reports |

---

## 13. Integration Points

### 13.1 Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `MessagePosted` | end-user posts message | search-index, mention-router, downstream Workflow engines, audit-chain | append-only |
| `MessageEdited` | end-user edits within edit-window | search-index, audit-chain | append-only delta |
| `MessageDeleted` | end-user / admin deletes | search-index, audit-chain, retention-purge worker | tombstone |
| `MessageReactionAdded` / `Removed` | end-user reacts | downstream engines | append-only |
| `MessageSelfDestructed` | TTL expiry | retention-purge worker, audit-chain | tombstone |
| `PresenceChanged` | client heartbeat / disconnect | tenant presence subscribers | append-only |
| `StatusUpdated` | user sets status | tenant subscribers | append-only |
| `TypingStarted` / `Stopped` | client typing | local fanout (Valkey) | ephemeral |
| `FileAttached` | attachment upload finalised | malware-scanner worker, search-index, audit-chain | append-only |
| `MentionEmitted` | mention-router resolves a mention | notification fanout, action-card consumer | append-only |
| `ChannelCreated` / `Deleted` | channel-admin action | audit-chain, ontology (`Channel` write) | append-only |
| `ChannelMemberGranted` / `Revoked` | channel-admin action | audit-chain, ontology | append-only |
| `ThreadStarted` | end-user opens thread | search-index, audit-chain, ontology | append-only |
| `CallStarted` / `Ended` | voice/video call lifecycle | audit-chain, QoE | append-only |
| `CallParticipantJoined` / `Left` | participant lifecycle | audit-chain | append-only |
| `HuddleStarted` / `Ended` | huddle lifecycle | audit-chain | append-only |
| `StageEventStarted` / `Ended` | stage event lifecycle | audit-chain | append-only |
| `StoryPosted` / `Expired` | story lifecycle | audit-chain, retention-purge | append-only |
| `LiveLocationStarted` / `Stopped` | live-location lifecycle | audit-chain | append-only |
| `SlashCommandInvoked` | end-user types `/...` | workflow-engine | append-only |
| `ApprovalCardPosted` | workflow posts card | downstream workflows | append-only |
| `ApprovalCastSubmitted` | end-user approves/rejects | workflow-engine | append-only |
| `EDiscoveryHoldOpened` / `Closed` | compliance-officer action | audit-chain, retention-purge worker | append-only |
| `FourEyesDisclosureExecuted` | tenant-admin pair approves PII read | audit-chain | append-only |
| `DlpBlockApplied` / `DlpOverrideUsed` | DLP verdict | audit-chain, tenant admin | append-only |
| `FederationPairingEstablished` / `Removed` | tenant admin pairs federation | audit-chain | append-only |
| `KeyTransparencyAuditFailed` | client detects equivocation | audit-chain, ops-security | append-only |

### 13.2 Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `OntologyEntityChanged` (Person/Team/Channel/MessageThread) | ontology | direct-messaging, group-messaging, channels | refresh mention-resolution cache |
| `MailActionCardEmitted` | mail | workflow-triggers | post action-card into target channel |
| `MailReceivedForChannelAlias` | mail | channels | post card in channel |
| `CalendarInviteReceived` | calendar | direct-messaging, group-messaging, channels | render iCal card |
| `CalendarMeetingStarted` / `Ended` | meet | huddles, voice-calls, video-calls | optional join nudge |
| `TenantRetentionPolicyUpdated` | tenancy | archive-retention | reassign channel retention bounds |
| `AuditChainSealed` | audit-chain | (read-only) | confirm audit-write durability |
| `WorkflowStudioRunStarted` / `Completed` | workflow-engine | workflow-triggers | post status into bound channel |
| `IntelligenceTranscriptReady` | intelligence | direct-messaging, group-messaging, channels | attach transcript to voice msg / call recording |
| `HrTimesheetEntryRecorded` | hr-payroll (via Workflow) | workflow-triggers | confirmation card in `/in` flow |
| `ConsentGraphConsentRevoked` | consent-graph | direct-messaging, group-messaging, channels | terminate ongoing data flows for user |

### 13.3 Ontology writes

| Object Type | Written by BC | Audit trail |
|---|---|---|
| `Channel{channel_id, tenant_id, context_kind, members, retention_policy_id}` | `channels` | Ed25519 |
| `MessageThread{thread_id, channel_id, parent_message_id, participant_refs}` | `threads` | Ed25519 |
| `DirectConversation{conv_id, party_ids, e2e_protocol}` | `direct-messaging` | Ed25519 |
| `Group{group_id, tenant_id?, mls_epoch, member_count}` | `group-messaging` | Ed25519 |
| `MessagePosted{message_id, channel_or_conv_id, author_ref, ttl, data_class}` (link-event) | `direct-messaging` / `group-messaging` / `channels` | Ed25519 |
| `Mention{message_id, target_ref, mention_kind}` | (shared mention-router) | Ed25519 |
| `CallSession{call_id, kind, participants, started_at, ended_at, qoe_summary}` | `voice-calls` / `video-calls` | Ed25519 |
| `HuddleSession{huddle_id, channel_id, participants}` | `huddles` | Ed25519 |
| `KeyPackagePublished{user_id, device_id, key_package_hash, log_seq}` | `e2e-encryption` | Ed25519 |
| `FederationPairing{tenant_id, peer_kind, peer_id}` | `federation` | Ed25519 |

### 13.4 Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Person`, `Team`, `Channel`, `Role` | mention-router (shared) | `find_by(@-handle, tenant_id)` |
| `RetentionPolicy` | `archive-retention` | `lookup(tenant_id, context_kind)` |
| `TenantContext` | every BC | tenant scope + pack jurisdiction lookup |
| `Contact` (cross-product) | `direct-messaging` | `lookup_by_phone_or_handle(query)` |

### 13.5 Protocol surfaces

| Surface | Spec | Pinned version | Notes |
|---|---|---|---|
| Client-to-server transport (default) | **HTTP/3 / QUIC** (RFC 9000 + RFC 9114) | RFC 9000 (final) + RFC 9114 (final) | Canonical default. Fallback to HTTP/2 over TLS 1.3 for legacy clients. |
| WebSocket | RFC 6455 over HTTP/3 (RFC 9220) | RFC 6455 (final) | mTLS-terminated; 30d token rotation |
| Federated client-server | Matrix Client-Server API | r0.6.1 LTS | Per ADR-MSGR-0003. Major-version upgrade requires ADR. |
| Federated server-server | Matrix Server-Server API | r0.1.4 LTS | Per ADR-MSGR-0003. Cross-pack routing default-deny. |
| Fediverse | ActivityPub (W3C 2018; living spec 2024) | W3C Rec 2018-01-23 + 2024 living spec | B2C personal opt-in only. WebFinger discovery. NodeInfo 2.1. |
| Slack bridge | Slack RTM API + WebClient | adapter-pinned | Per-tenant pairing. |
| E2E key agreement | MLS (RFC 9420) | RFC 9420 (final 2023-07) | Canonical for personal + work. Per ADR-MSGR-0002. |
| E2E cross-vendor (Matrix bridge) | Olm 3.x + Megolm 1.x | Olm 3.x + Megolm 1.x | Bridge endpoints decrypt MLS → re-encrypt to Olm/Megolm. |
| E2E cross-vendor (Signal Protocol) | Signal X3DH + Double Ratchet | X3DH 2021 spec + DR 2016 spec | For Signal-compat bridges only. |
| Real-time media | WebRTC (W3C + IETF) | W3C Rec 2021-01 + IETF WG-shipped 2024 | DTLS-SRTP + ICE + STUN (RFC 5389) + TURN (RFC 5766) |
| Voice codec | Opus (RFC 6716) | RFC 6716 (final) | 48 kHz mono/stereo |
| Video codec | AV1 (AOMedia 2018 + 2024 profiles) + H.264 (ITU-T H.264 fallback) | AV1 Main Profile + H.264 Baseline/High | Simulcast 180p/360p/720p/1080p |
| Action card carrier | AsyncAPI 2.6 | `contracts/asyncapi/action-cards.yaml` | Mail → messenger + workflow → messenger ingest |
| Search index | Meilisearch 0.10.0 LTS primary + Tantivy 0.22.x embedded fallback | per ADR-COMM-0004 shared backend choice | Cedar-policy-filtered server-side |
| File upload | tus.io (resumable upload) 1.0.0 | tus 1.0.0 | Multipart resumable |
| Push notifications | APNs (Apple) + FCM HTTP v1 (Google) + Web Push (RFC 8030) | latest stable | Sealed sender for E2E |
| Key transparency | CONIKS-class log (per ADR-MSGR-0002) | tree-based Merkle | Auditable equivocation detection |
| Vcard (contact share) | RFC 6350 vCard 4.0 | RFC 6350 (final) | Contact-share payload |
| iCalendar (meeting card) | RFC 5545 + RFC 6047 iMIP | RFC 5545 (final) | For calendar-integration |
| Webhook signing | HMAC-SHA256 + timestamp | per ADR-MSGR-0001 | Outbound webhooks |
| OAuth 2.1 / OIDC 1.0 | RFC 6749 + OIDC 1.0 + OAuth 2.1 draft | latest | Bot / integration auth |
| SCIM 2.0 | RFC 7644 | RFC 7644 (final) | User provisioning |
| SAML 2.0 | OASIS SAML 2.0 | OASIS 2.0 | SSO |

Per ADR-0064 canonical-base + localization, per-pack overlays MAY pin a newer minor (e.g., r0.6.1 → r0.6.1+pack-eu-erasure-extension) but MUST NOT drift the major.

---

## 14. References

### 14.1 Encryption + protocols

- **MLS RFC 9420** (Messaging Layer Security): IETF, 2023-07. https://datatracker.ietf.org/doc/rfc9420/
- **Signal Protocol** (Open Whisper Systems): X3DH key agreement (2016, rev. 2021) + Double Ratchet (2016). https://signal.org/docs/
- **Matrix Spec 2024** (Matrix Foundation): Client-Server r0.6.1 + Server-Server r0.1.4 + Olm/Megolm. https://spec.matrix.org/
- **Olm + Megolm** (Matrix.org): Double-ratchet + group key ratchet (2024 rev). https://gitlab.matrix.org/matrix-org/olm
- **WebRTC** (W3C + IETF): W3C Rec 2021-01; IETF RFCs 8825-8843 (2021); 2024 active-WG drafts. https://www.w3.org/TR/webrtc/
- **DTLS-SRTP** (RFC 5764). https://datatracker.ietf.org/doc/rfc5764/
- **ICE** (RFC 8445), **STUN** (RFC 5389), **TURN** (RFC 5766).
- **HTTP/3** (RFC 9114) + **QUIC** (RFC 9000) + **WebSocket over HTTP/3** (RFC 9220).
- **ActivityPub** (W3C Recommendation 2018-01-23 + 2024 living spec). https://www.w3.org/TR/activitypub/
- **CONIKS** (USENIX Security 2015) + **SEEMless** (CCS 2019) + Apple iMessage Key Transparency 2024 (Apple security blog 2024). https://security.apple.com/blog/imessage-contact-key-verification/
- **PAdES** (ETSI EN 319 142): PDF Advanced Electronic Signatures.

### 14.2 Codecs + media

- **Opus** (RFC 6716): IETF, 2012. https://datatracker.ietf.org/doc/rfc6716/
- **AV1** (Alliance for Open Media + AOMedia 2024 Profile Pack). https://aomediacodec.github.io/av1-spec/
- **H.264** (ITU-T H.264). https://www.itu.int/rec/T-REC-H.264
- **MediaPipe Selfie Segmentation** (Google AI 2022 + 2024 update). https://ai.google.dev/edge/mediapipe/solutions/vision/image_segmenter
- **RNNoise** (Mozilla / Xiph 2017 + 2024 model). https://github.com/xiph/rnnoise
- **Krisp** (commercial; reference SDK pinned 2024).

### 14.3 Real-time backend

- **LiveKit 2024.x** (open-source SFU). https://docs.livekit.io/
- **coturn** (open-source STUN/TURN server). https://github.com/coturn/coturn
- **Pion WebRTC** (Go reference impl); **webrtc-rs** (Rust impl). https://github.com/webrtc-rs/webrtc

### 14.4 Competitor engineering blogs (2024)

- **Discord engineering 2024**: "How Discord Stores Trillions of Messages" (2024 ScyllaDB migration). https://discord.com/blog/how-discord-stores-trillions-of-messages
- **KakaoTalk engineering 2024** (Kakao tech blog). https://tech.kakao.com/
- **Line engineering 2024**: "Line's E2E encryption" + "Line's mobile chat architecture". https://engineering.linecorp.com/
- **WhatsApp engineering 2024**: WhatsApp E2E whitepaper rev. 2024-03 + Multi-device whitepaper 2021 rev. 2024. https://www.whatsapp.com/security/
- **Telegram MTProto 2.0**: https://core.telegram.org/mtproto
- **Apple iMessage Contact Key Verification (CKV) 2024**: https://security.apple.com/blog/imessage-contact-key-verification/
- **Signal engineering 2024**: "Sealed Sender" (2018) + "Private Group System" (2020) + MLS exploratory posts 2024. https://signal.org/blog/
- **Slack engineering 2024**: "Slack's WebSocket gateway" + "Block Kit". https://slack.engineering/
- **Microsoft Teams 2024**: Teams architecture + Purview eDiscovery + DLP. https://learn.microsoft.com/microsoftteams/

### 14.5 Standards

- **RFC 5322** (Internet Message Format) — for thread parent reference compat.
- **RFC 5545** (iCalendar) + **RFC 6047** (iMIP) — for meeting-invite cards.
- **RFC 6350** (vCard 4.0) — for contact-share.
- **RFC 6749** (OAuth 2.0) + OAuth 2.1 draft + **OIDC 1.0** — for SSO.
- **RFC 7644** (SCIM 2.0) — for user provisioning.
- **OASIS SAML 2.0** — for SSO.
- **WCAG 2.2 AA** (W3C 2023-10-05). https://www.w3.org/TR/WCAG22/
- **eIDAS 910/2014** (EU AdES) — for e-sign in pack-eu.
- **GDPR Art. 17 (Erasure)** + **Art. 20 (Portability)**.
- **KR PIPA Art. 21, 22, 36** — for pack-kr.
- **HIPAA + HITECH** — for pack-us-healthcare.
- **DSA Art. 14** (EU Digital Services Act) — for appeal flow.
- **EU AI Act Annex III** — for AI-classifier governance.

### 14.6 Internal references

- ADR-0008 Data Use Boundary
- ADR-0028 Audit-chain Merkle + Ed25519 (inherited from Bominal)
- ADR-0056 BNF v4.1 naming authority
- ADR-0105 13-layer canonical enum
- ADR-0106 application → usecase rename
- ADR-0117 Data residency packs
- ADR-0123 Authority cohesion (HG-MESSENGER)
- ADR-0131 Per-microservice flat layout
- ADR-0132 Suite-and-bundle dissolution
- ADR-0133 Industry best-practice conformance
- ADR-0135 dual-context (parallel)
- ADR-0139 Agentic SLO-gated promotion
- ADR-0140 Cedar policy engine
- ADR-0145 Cross-product flows via Workflow + Ontology only
- ADR-0148 mTLS service mesh
- ADR-0150 Postgres + Citus sharding
- ADR-0172 Read replicas + CQRS for high-read BCs
- ADR-0208 dual-context unified channel hub (Bominal inherited)
- ADR-0215 retention legal-hold dual-context (Bominal inherited)
- ADR-0238 platform parallel dissolution
- ADR-0240 Sovereign-cloud-per-pack
- ADR-0241 DR tiers (T0/T1/T2)
- ADR-0242 oyatie-is-a-tenant doctrine
- ADR-0243 Cedar-as-universal-gate
- ADR-0244 Tenant audience-type
- ADR-0245 Substrate-vs-product layering
- ADR-0251 Microservice tier classification
- ADR-0251 encryption-BYOK + tenant DEK envelope
- ADR-MSGR-0001 Voice/video huddle composition (LiveKit + WebRTC)
- ADR-MSGR-0002 MLS RFC 9420 + sealed sender + key transparency
- ADR-MSGR-0003 Federation wire format (Matrix r0.6.1 + ActivityPub bridge)

### 14.7 Companion specs

- `/specs/microservices/messenger.json`
- `/specs/per-microservice-flat-layout.json`
- `/specs/agentic-slo-gated-promotion.json`
- `/specs/tenant-model.json`
- `/specs/microservice-tier-classification.json`
- `/specs/microservice-dependency-dag.json`

---

## 15. Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Search backend: Meilisearch primary + Tantivy fallback (per ADR-COMM-0004) — confirm for messenger or split? | axis-messenger | resolve in IP-001-search-backend |
| 2 | MLS key escrow: none (Signal-default) or opt-in passphrase + Shamir-split for personal + HSM-anchored for work? | council-privacy | ADR-MSGR-0002 §key-escrow |
| 3 | Live translation backend: on-device only (Whisper-Small + NLLB-200) or cloud-Whisper opt-in? | axis-intelligence | ADR-MSGR-0001 §live-translation |
| 4 | Federation default for B2B: opt-in per tenant (current) or pack-allowlist? | ops-compliance | ADR-MSGR-0003 §federation-default |
| 5 | Slack bridge: build in-house or partner with Element's bridge stack? | axis-messenger + ops-security | RFC-MSGR-001 |
| 6 | Recording storage: encrypted to MLS group key (current plan) or tenant-DEK only for compliance work mode? | council-privacy | ADR-MSGR-0001 §recording |
| 7 | E2E for calls under federation (Matrix bridge): degrade to TLS-only with explicit warning or refuse? | council-privacy | ADR-MSGR-0003 §federated-call-e2e |
| 8 | Sticker store revenue model: free-tier only at M02, paid via Plugin App Store at M03? | sales-strategy | IP-PLUGIN-STORE |
| 9 | Voice-message transcription model: per-tenant cell (current) vs per-pack shared with DP-noise? | axis-intelligence | ADR-INT-0007 |
| 10 | Self-observability emission: messenger emits to observability µservice as one tenant or per-pack? | axis-messenger + axis-observability | resolved in IP-007 |

---

## 16. Milestones + Phasing

| Phase | Milestone | Scope | Date target |
|---|---|---|---|
| M02-foundation | First ship | All 18 BCs in skeletal form; 1:1 + group DM with MLS; voice/video 1:1; channels; threads; reactions; mentions; basic admin | 2026-Q3 |
| M02-pack-personal | B2C launch | Stories, channels (broadcast), self-destruct, custom emoji, GIF, polls, location, contacts, multi-device, full MLS, 17 languages | 2026-Q3 |
| M02-pack-work | B2B launch | Workspaces, threads, mentions, slash commands, workflow triggers, mail-bridge, calendar/meet integration, huddles, clock-in, e-sign, approvals, DLP, eDiscovery, audit-chain, SSO, SCIM | 2026-Q3 |
| M02-pack-federation | Federation packs | Matrix r0.6.1 federation; ActivityPub channels; Slack adapter | 2026-Q4 |
| M03-pack-intel | Intelligence packs | Live captions, voice-msg transcription, smart-reply, AI-summarisation, live translation | 2027-Q1 |
| M04-spatial | Spatial audio + AR | Spatial audio for calls; AR effects roadmap | 2027-Q2 |
| M05-active-sync | ActiveSync | EAS sync (for legacy Exchange ecosystems if demanded) | 2027-Q3 |
| M06-govcloud | FedRAMP / GovSlack-class | IL5 / FedRAMP High / KR-GovCloud | 2027-Q4 |

---

## 17. Risks + Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| MLS at consumer scale unproven | Medium | High | Reference OpenMLS Rust impl; prototype 10k group at M02-pre; benchmark vs Sender-Keys for sanity |
| LiveKit SFU cost balloons | Medium | Medium | Per-cell SFU; auto-scale + cost cap per tenant; FinOps dashboard |
| WebRTC NAT traversal fails for some users | Medium | Medium | TURN fallback; bandwidth-tested coturn cluster; per-region anchoring |
| Federation security surface | Medium | High | Four-eyes on every pairing; per-pack federation default-deny; bridge endpoints audit-chained |
| Sealed sender + KT complex; bugs leak metadata | Medium | High | External pen-test (Trail of Bits or similar); MLS RFC 9420 spec-conformance lane |
| 17-language i18n drift | Low | Medium | i18n CI lane; locale-bundle versioning; pseudo-locale testing |
| WCAG 2.2 AA regression on new features | Medium | Medium | a11y CI lane; per-PR snapshot diff via axe-core |
| Customer migration friction (Signal/Telegram/etc.) | Medium | Medium | Per-source import workers; staff-supported white-glove for top-100 customers |
| Compliance pack drift (KR PIPA, EU AI Act) | Low | High | Per-pack overlay tests; quarterly compliance audit |
| DLP false-positive | High | Low | Override-with-reason flow; audit-chained; weekly tenant-admin review queue |

---

## 18. Glossary

- **MLS (Messaging Layer Security)** — IETF RFC 9420; group key agreement for E2E messaging with PCS, FS, scalability.
- **Sealed Sender** — server sees only recipient envelope; sender identity is encrypted inside.
- **Key Transparency (KT)** — Merkle-tree-backed append-only log of all KeyPackages; clients audit to detect server equivocation.
- **PSK Binding** — in MLS, a pre-shared key derived from the conversation key binds the WebRTC call key, ensuring call media decryption requires conversation membership.
- **LeafNode** — in MLS, one device's key material; a user's tree has one LeafNode per device.
- **Commit** — in MLS, an atomic update to the group key tree (add/remove member; update key).
- **Epoch** — in MLS, the current state of the group key tree; advances on every Commit.
- **SFU (Selective Forwarding Unit)** — voice/video router that doesn't re-encode (low CPU); LiveKit is one.
- **STUN / TURN** — NAT-traversal protocols for WebRTC; coturn is the reference implementation.
- **DTLS-SRTP** — WebRTC media encryption.
- **Huddle** — Slack-Huddle parity: drop-in audio in a channel.
- **Stage** — Discord-Stage parity: speakers + listeners broadcast room.
- **Federation** — cross-vendor / cross-homeserver routing; Matrix is the canonical protocol.
- **ActivityPub** — W3C fediverse protocol; for broadcast channels with Mastodon/Lemmy interop.
- **Four-eyes** — per Bominal ADR-0215; two distinct approving principals required for high-impact action (legal hold, plaintext disclosure, federation pairing, ban affecting >100 posts).
- **Dual-context isolation** — kernel invariant: personal-context DMs cannot become professional-channel replies and vice versa; per ADR-0238.
- **Action Card** — typed inline message with buttons (approve/reject/etc.) backed by Workflow Engine state.
- **DSAR** — Data Subject Access Request; GDPR Art. 15-22.

---

## 19. Document Provenance

- **Source PRD sibling models**: `microservices/mail/PRD.md` (2026-05-20), `microservices/community/PRD.md` (2026-05-20).
- **Prior messenger PRD**: superseded; this PRD expands B2C scope (Signal/Telegram/KakaoTalk/Line/WhatsApp/IG-DM/FB-Msgr/Discord/iMessage parity) and B2B scope (Slack/Teams/Matrix parity), aligns to ADR-0245 substrate-vs-product layering, adds 18 BCs, adds MLS RFC 9420, adds LiveKit + WebRTC composition, adds HTTP/3/QUIC as canonical transport, adds key-transparency, adds federation via Matrix + ActivityPub + Slack Connect.
- **Bominal inheritance**: ADR-0208 dual-context unified channel hub; ADR-0215 retention legal-hold dual-context; ADR-0028 audit-chain Merkle + Ed25519; ADR-0111 ciphertext envelope.
- **New oyatie ADRs introduced by this PRD**: ADR-MSGR-0001 (huddle composition), ADR-MSGR-0002 (MLS + sealed sender + KT), ADR-MSGR-0003 (federation wire format).
- **CI lanes new for messenger**: `dual-context-isolation`, `mls-rfc-9420-conformance`, `webrtc-spec-conformance`.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `messenger` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `messenger` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 10 module pin(s) across 5 context(s).
- Scaling input: `per_message` with cell placement `Tier-1` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
