---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-meet
microservice: meet
status: Accepted
sales_segment: hero-product (productivity-suite)
tier: hero-product
milestone_first_ship: M02-foundation
net_new: true
bominal_source: []
related_adrs: [ADR-0008, ADR-0056, ADR-0105, ADR-0106, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145), ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-meet
doc_status: published
---

# PRD-meet: Video-meeting platform (Google Meet / Zoom / Microsoft Teams Meetings / Webex / Whereby / Jitsi-class)

## Purpose

The `meet` microservice is oyatie's dedicated video-meeting product. It is net-new per ADR-0135 — there is no legacy `oya-connect-meet-*` predecessor. It owns **named meeting rooms with lobby/registration + calendar-bound meeting instances + breakout rooms + screen-share with presenter-control + cloud recording + live transcription with multi-language captions + post-meeting AI summary + interpretation channels + webinar mode + RTMP live-stream egress + large-audience broadcast (10k+)**.

`meet` is **distinct from the `messenger` huddles BC** (per ADR-MSGR-0001):

| Dimension | `messenger` huddles BC | `meet` µservice |
|---|---|---|
| Entry surface | Ad-hoc inside a channel | Named room URL + calendar invite + scheduled link |
| Max participants | ≤ 30 per session (LiveKit sweet-spot) | ≤ 1000 interactive; ≤ 100 000 broadcast via WHIP/HLS mesh |
| Persistence | Live only; no recording by default | Recording + transcription + summary persisted |
| Lobby / registration | None | Yes (lobby, waiting room, pre-registration, practice session) |
| Webinar mode | n/a | First-class (registration + Q&A moderation + practice session) |
| Live-stream egress | n/a | RTMP → YouTube/Twitch/Vimeo with WHIP fallback |
| Interpretation channels | n/a | Up to 12 simultaneous interpreter audio overlays |
| Owning team | axis-messenger | axis-meet |

`meet` shares the LiveKit 1.6.2 SFU and coturn 0.2.0 substrate with `messenger` huddles per ADR-MSGR-0001's substrate-adapter pattern (LiveKit runs as a sidecar substrate in each µservice's cell; meet does NOT import `oya-messenger-*` crates — substrate adapters live in each µservice).

`meet` is a **hero product**, end-user-facing through Workflow Studio shell + standalone meet clients (web + desktop + iOS + Android). It is consumable as a shared substrate by other oyatie products via the `meet.meeting.v1` Workflow events and the `Meeting` Ontology object type.

## Tenant Value

- **Tenant Outcome 1 — Sovereign video conferencing.** Tenants get Google Meet / Zoom / Microsoft Teams Meetings-class video conferencing on tenant-pinned residency. No vendor lock-in; no cross-border data flow without SCC.
- **Tenant Outcome 2 — Calendar-bound meeting flow.** Meet rooms bind to calendar events in the `calendar` µservice; invitees follow a single link from invite to lobby to room.
- **Tenant Outcome 3 — Compliance-grade recording + retention.** Recordings + live transcripts + AI summaries persisted under tenant-DEK envelope encryption (Bominal ADR-0111); per-pack retention floors (HIPAA 6y for pack-us-healthcare; SEC Rule 17a-4 + FINRA 4511 for pack-us-financial; KR PIPA Art. 21 for pack-kr).
- **Tenant Outcome 4 — Multi-language meetings.** Live captions in 60+ languages via Whisper-large; live interpretation channels for up to 12 languages with human or AI interpreters; transcript translated post-meeting.
- **Tenant Outcome 5 — Webinar + large-audience broadcast.** First-class webinar mode (registration, practice session, Q&A moderation, attendee analytics); RTMP egress to YouTube/Twitch; ≥ 10 000 broadcast attendees via WHIP/HLS mesh.
- **Tenant Outcome 6 — Optional E2E encryption.** Tenant-tier opt-in MLS RFC 9420 + Insertable Streams (W3C) for end-to-end-encrypted meetings; default off because E2E disables server-side recording + transcription + AI features.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | meeting host | to create a named room with a stable URL + lobby + waiting room | recurring meetings have a permanent home | meeting-room | Must |
| FR-02 | meeting host | to bind a meet room to a calendar event in `calendar` µservice | invitees use one link end-to-end | meeting-instance | Must |
| FR-03 | meeting host | to start a recording (cloud) with transcription | absent attendees + audit-retention work | recording | Must |
| FR-04 | meeting host | to mute / unmute / spotlight / remove participants | host can moderate | participant | Must |
| FR-05 | attendee | to join from web + desktop + iOS + Android + dial-in (M03-onward) | barrier-free join | meeting-instance | Must |
| FR-06 | presenter | to share screen with remote-control optional | demos work | screen-share | Must |
| FR-07 | attendee | to see live captions in their language | accessibility + multilingual meetings work | transcription | Must |
| FR-08 | host | to run a poll, Q&A, or whiteboard | engagement features | meeting-instance | Must |
| FR-09 | host | to split attendees into breakout rooms then re-merge | workshop-style works | breakout-rooms | Must |
| FR-10 | webinar host | to enable practice-session + registration + Q&A-moderation + attendee report | webinars work end-to-end | webinar | Must |
| FR-11 | host | to live-stream the meeting to YouTube/Twitch via RTMP | broadcast to large audiences | live-stream-egress | Must |
| FR-12 | broadcast attendee (read-only) | to join a 10 000+ person broadcast | large-scale fan-out works | large-audience-broadcast | Must |
| FR-13 | interpreter | to speak on a language-specific overlay audio channel | multi-language live interpretation works | interpretation-channels | Should |
| FR-14 | tenant compliance officer | to put a recording under legal hold (eDiscovery) | regulatory works | recording | Must |
| FR-15 | end-user | to opt into an end-to-end-encrypted meeting (tenant-tier) | sensitive meetings (legal, M&A) work | e2e-encryption | Should |
| FR-16 | tenant operator | to receive a post-meeting AI summary + action-item list | productivity loop closes | foundry-runtime AI summary | Should |
| FR-17 | Workflow Studio | to consume `MeetingStarted` / `MeetingEnded` / `RecordingProduced` events | downstream automation works | meeting-instance | Must |
| FR-18 | guest (no tenant account) | to join via lobby + waiting room + host-approval | external-attendee scenarios work | participant | Must |
| FR-19 | host | to enable virtual background + blur + noise-suppression | quality of life | video / audio | Should |
| FR-20 | screen-reader user | to navigate captions + chat + participants list | WCAG 2.2 AA | (cross-cutting) | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | Notes |
|---|---|---|---|---|
| Room create | ≤ 30 ms | ≤ 80 ms | ≤ 100 ms | Postgres insert + Valkey registry |
| Participant join (1st media frame) | ≤ 800 ms | ≤ 1.5 s | ≤ 2.0 s | SDP + ICE + DTLS + LiveKit room join |
| Media glass-to-glass (intra-region) | ≤ 80 ms | ≤ 150 ms | ≤ 200 ms | LiveKit SFU intra-region |
| Media glass-to-glass (inter-region) | ≤ 130 ms | ≤ 250 ms | ≤ 350 ms | Cross-region SFU mesh |
| Screen-share start | ≤ 400 ms | ≤ 800 ms | ≤ 1.2 s | Track publish + simulcast |
| Recording start | ≤ 500 ms | ≤ 800 ms | ≤ 1.0 s | Egress worker spawn + ffmpeg mux |
| Live caption (audio → caption) | ≤ 300 ms | ≤ 450 ms | ≤ 500 ms | Whisper-medium streaming + WS push |
| Meeting summary post-end | — | ≤ 40 s | ≤ 60 s | 60-min meeting; Whisper-large + LLM |
| Webinar 10k-attendee fan-out | — | ≤ 3 s | ≤ 5 s | WHIP/HLS mesh + edge cache |
| E2E MLS handshake (≤ 12 participants) | ≤ 400 ms | ≤ 700 ms | ≤ 1.0 s | MLS RFC 9420 epoch advance |

Reference: ITU-T G.107 (E-model MOS) for voice quality target ≥ 4.0 mean; ITU-T Y.1541 IPTV class for video frame timing.

### Security

- All meeting signaling mTLS-terminated; per-tenant API token bound at OpenBao with rotation 30d.
- Per-participant LiveKit access token short-TTL (≤ 1 h); scoped to single room.
- Cedar v4.2 default-deny on every read/write (`policy/*.cedar`); client never trusted.
- Recording blobs encrypted at rest under tenant-DEK (envelope encryption per Bominal ADR-0111); transcripts likewise.
- Lobby + waiting room: non-members refused until host approval; guest tokens bound to single meeting-instance.
- E2E mode (opt-in): MLS RFC 9420 + W3C Insertable Streams; server sees ciphertext only; recording + transcription + AI features structurally disabled.
- Screen-share remote-control gated by per-participant Cedar `Action::"grant_remote_control"`; revocable in 1 click.
- ffmpeg transcode workers run under gVisor sandbox to contain media-parser CVEs.
- Whisper transcription workers run with GPU node selector + tenant-segregated batches; transcript content-hash signed.

### Audit + Compliance

- Every `MeetingStarted / MeetingEnded / ParticipantJoined / ParticipantLeft / RecordingStarted / RecordingFinalized / TranscriptionSealed / FourEyesDisclosureExecuted / EDiscoveryHoldOpened` event emits an audit-chain record (Merkle + Ed25519 per Bominal ADR-0028).
- Recording retention floors per pack (pack-us-healthcare 6y; pack-us-financial 3-7y per SEC 17a-4 / FINRA 4511; pack-eu MiFID II 5-7y for investment-firm communications; pack-kr 1-5y per labor + 전자문서법).
- KR PIPA Art. 15 (recording consent): tenants who enable recording attest that participant-consent flow runs at join — modal banner ("This meeting is being recorded — your participation constitutes consent under KR PIPA Art. 15") + audit-chain record per participant.
- EU AI Act risk classification: transcription = low-risk; live-translate = medium-risk (per ADR-MEET-0006); summary = low-risk.
- ePrivacy Directive Art. 5(3) embedded analytics gated; default off.
- Per-tenant DPA + BAA (pack-us-healthcare) at `legal/`.

### Availability + SLO

- Availability target: 99.95 % monthly for room-create + participant-join.
- Media plane: 99.9 % monthly (LiveKit SFU + coturn TURN).
- RTO ≤ 60 min for meeting-room metadata + recording manifest per manifest `dr.rto_p99_seconds=3600`; live-room SLOs still fail over faster through SFU/TURN degradation paths.
- RPO ≤ 5 min (cross-AZ Postgres replication + S3 versioning for recordings).

### Data residency

- Per-tenant pack pinning per ADR-0117; recordings + transcripts + summaries follow tenant pack.
- Cross-pack meeting attendance allowed (a pack-eu user can join a pack-us tenant's meeting) but media routes through inter-region SFU mesh with tenant-attestation; recording stays in host-tenant pack.
- Guest tokens enforce cell isolation; anonymous guests join lobby but cannot pivot tenants.

### DR Posture (ADR-0343)

- RTO/RPO target: manifest `dr` declares `rto_p99_seconds=3600` and `rpo_p99_seconds=300`. HIPAA-2024 (3600s/300s), SOC2-T2 (14400s/900s), KR-CSAP-v3.1 (3600s/900s), and ISO27001-2022 (14400s/3600s) leave the effective meet bound at 3600s RTO and 300s RPO.
- failover_runbook: `runbooks/dr-failover.md`; manifest backup substrate is `postgres_wal_g`, `object_storage_versioned`, `valkey_cluster`, and `audit_chain_merkle_seal`.
- multi_region_active_active: true, with manifest replication shape `active-active-multi-az-cross-region-warm`; live media still routes through SFU/TURN degradation runbooks for faster operational recovery.
- WHY: this target lets a tenant survive a regional media/control-plane fault during a live customer meeting while keeping recording and transcript evidence recoverable under the D-2 manifest contract.

### Capacity Model (ADR-0340)

- Per-tenant baseline: manifest `capacity_model` declares 0.28 vCPU, 768Mi RAM, 4Gi storage, 4 Valkey connections, 3 Postgres connections, and 8 outbound HTTP connections per tenant.
- Scaling dimension: `per_user`; participant concurrency drives room create/join, participant/lobby, captions, SFU signaling, screen-share, and recording-start pressure.
- Cell placement class: Tier-3, matching manifest `capacity_model.cell_placement_class`, because media pressure is high but the service remains a product application surface rather than tenant-customer code execution.
- Autoscaling boundaries: room-control min 4 / max 100 replicas, LiveKit adapter min 6 / max 200, GPU transcription pool on reserved GPU nodes; per-tenant throttles cap webinar/broadcast bursts before shared SFU pools saturate.
- WHY: meet's load profile is participant-spiky rather than CRUD-spiky, so capacity follows concurrent publishers/subscribers and caption sessions instead of account count alone.

### Sustainability + Cost Attribution (ADR-0344)

- Every audit-chain row emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` alongside the existing meeting, recording, transcription, eDiscovery, and disclosure seals.
- Provider routing affected by carbon: no for live media, telehealth/emergency, recording consent, and regulated retention paths; yes only for asynchronous summary, batch transcription, and archive search jobs when the tenant pack and SLO leave slack.
- Per-tenant transparency surface: FinOps portal exposes meeting-minutes, recording GB-months, live-caption GPU seconds, transcript-search index size, and broadcast egress by tenant/capability/provider/cell/compliance_pack.
- WHY: meet combines high-emission real-time media with regulated evidence artifacts, so CSRD, SB-253, and SEC climate disclosures need tenant-attributed emissions without letting carbon routing violate HIPAA emergency or live meeting guarantees.

### API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet via `Oyatie-Version` header, `/v/<YYYY-MM-DD>` URL prefix, and proto3 `oyatie_version` field for `meet.meeting.v1`, recording handoff, webinar, and event contracts.
- SDK semver model: major.minor.patch, with generated web/desktop/mobile SDKs mapping semver releases to the date-versioned public contracts.
- Support window: last N=3 public API versions for at least 180 days; deprecation is audit-visible for tenant admins.
- Per-tenant pinning supported: yes, so regulated tenants can freeze meeting/webinar contracts during validation windows.
- Internal-mesh exemption: yes; direct gRPC between first-party services remains exempt under ADR-0145 when it does not expose a public contract.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase`). Eleven primary BCs.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `meeting-room` | `oya-meet-meeting-room-{kernel,domain,usecase,api,adapter-postgres,rest,sdk,app}` | Named room CRUD; URL + lobby + waiting-room policy; tenant-bound | `MeetingRoom`, `LobbyPolicy`, `WaitingRoomPolicy` |
| `meeting-instance` | `oya-meet-meeting-instance-{kernel,domain,usecase,api,adapter-postgres,adapter-livekit,rest,worker,sdk,app}` | Per-occurrence session; calendar binding; lifecycle (created→active→ended) | `MeetingInstance`, `CalendarBinding`, `MeetingLifecycle` |
| `participant` | `oya-meet-participant-{kernel,domain,usecase,api,adapter-postgres,adapter-valkey,rest,worker,sdk,app}` | Per-participant state; role (host/co-host/presenter/attendee/guest); presence | `Participant`, `ParticipantRole`, `JoinTicket`, `LobbyMembership` |
| `audio` | `oya-meet-audio-{kernel,domain,usecase,adapter-livekit,worker,sdk}` | Audio track lifecycle; mute/echo-cancel/noise-suppression via LiveKit | `AudioTrack`, `MuteState`, `NoiseSuppressionProfile` |
| `video` | `oya-meet-video-{kernel,domain,usecase,adapter-livekit,worker,sdk}` | Video track lifecycle; virtual-background/blur/HD/4K/spotlight | `VideoTrack`, `Spotlight`, `BackgroundPolicy` |
| `screen-share` | `oya-meet-screen-share-{kernel,domain,usecase,adapter-livekit,worker,sdk}` | Screen-share track; presenter-control; remote-control grants | `ScreenShareTrack`, `PresenterControlState`, `RemoteControlGrant` |
| `recording` | `oya-meet-recording-{kernel,domain,usecase,api,adapter-postgres,adapter-s3,adapter-ffmpeg,rest,worker,sdk,app}` | Cloud recording lifecycle; ffmpeg mux; retention; legal hold | `Recording`, `RecordingManifest`, `RetentionBinding`, `Hold` |
| `transcription` | `oya-meet-transcription-{kernel,domain,usecase,api,adapter-postgres,adapter-s3,adapter-whisper,adapter-meilisearch,rest,worker,sdk,app}` | Live captions + post-meeting transcript via Whisper; per-language; searchable | `Transcript`, `CaptionStream`, `LanguageChannel`, `TranscriptSeal` |
| `webinar` | `oya-meet-webinar-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}` | Registration + practice-session + Q&A moderation + attendee report | `WebinarRegistration`, `PracticeSession`, `QnaQueue`, `AttendeeReport` |
| `live-stream-egress` | `oya-meet-live-stream-egress-{kernel,domain,usecase,api,adapter-srs,adapter-ffmpeg,worker,sdk}` | RTMP egress to YouTube/Twitch/Vimeo + WHIP fallback | `EgressTarget`, `RtmpStream`, `WhipStream` |
| `e2e-encryption` | `oya-meet-e2e-encryption-{kernel,domain,usecase,adapter-mls,sdk}` | Opt-in MLS RFC 9420 + W3C Insertable Streams | `MlsGroupState`, `EpochAdvance`, `KeyPackageBundle` |

Naming justification — `meeting-room`:

```
NAME: oya-meet-meeting-room-<layer>
JUSTIFICATION:
- microservice = meet: per ADR-0131 per-microservice flat layout.
- bc-tokens = meeting-room: primary BC. ADR-0056 v4.1 BC-optionality honoured.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-livekit, -adapter-whisper, -adapter-ffmpeg, -adapter-srs,
  -adapter-postgres, -adapter-valkey, -adapter-s3, -adapter-meilisearch, -adapter-mls
  are canonical *-adapter-<backend> per ADR-0105 Amendment 3.
```

Total crates introduced: **~80** across 11 BCs.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `MeetingRoomRepository` | `oya-meet-meeting-room-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `MeetingInstanceStore` | `oya-meet-meeting-instance-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `MeetingSfuClient` | `oya-meet-meeting-instance-kernel` | `-adapter-livekit` | `BEHAVIORAL_TENANT_PRODUCT`, `SECRET` (per-participant LiveKit tokens) |
| `ParticipantRegistry` | `oya-meet-participant-kernel` | `-adapter-valkey` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` (guest display name) |
| `RecordingStore` | `oya-meet-recording-kernel` | `-adapter-s3` | `BEHAVIORAL_TENANT_PRODUCT`, sometimes `PII_IDENTIFYING` / `PHI` |
| `RecordingMux` | `oya-meet-recording-kernel` | `-adapter-ffmpeg` (gVisor sandbox) | `BEHAVIORAL_TENANT_PRODUCT` |
| `TranscriptionEngine` | `oya-meet-transcription-kernel` | `-adapter-whisper` (Whisper-large; GPU pool) | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING`, `PHI` (pack-us-healthcare) |
| `TranscriptSearchIndex` | `oya-meet-transcription-kernel` | `-adapter-meilisearch` | `BEHAVIORAL_TENANT_PRODUCT` |
| `LiveStreamEgress` | `oya-meet-live-stream-egress-kernel` | `-adapter-srs` (SRS RTMP server) + `-adapter-ffmpeg` | `BEHAVIORAL_TENANT_PRODUCT` |
| `MlsGroup` | `oya-meet-e2e-encryption-kernel` | `-adapter-mls` (mls-rs RFC 9420) | `SECRET` (key packages); ciphertext only at rest |
| `CedarMeetingPolicy` | every BC `-kernel` | `-adapter` (Cedar v4.2 evaluator) | `INTERNAL_ONLY` |
| `AuditChainClient` | (cross-BC) `-kernel` per BC | (cross-BC) `-adapter` to audit-chain µservice | `AUDIT` |

Data-class enforcement: `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `meet` MUST NOT import any other product µservice crate at any layer. Calendar binding flows through Workflow events (`calendar.event.v1`) + Ontology reads (`Meeting`/`CalendarEvent` object types). LEAN-A2 lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice meet` — dependency-direction
- `oya gate validate lean-a2 --microservice meet` — cross-product-refusal
- `oya gate validate port-location --microservice meet`
- `oya gate validate layer-correctness --microservice meet`
- `oya gate validate per-microservice-layout --microservice meet`
- `oya gate validate statelessness --microservice meet`
- `oya gate validate shardability --microservice meet`
- `oya gate validate authority-cohesion --microservice meet` (HG-MEET)
- `oya gate validate hyperscaler-maturity-claims --microservice meet` (HG-MEET)

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `MeetingRoomCreated` | host creates a named room | ontology (`Meeting` write), audit-chain | append-only |
| `MeetingInstanceStarted` | first participant joins | calendar (event-status update via Workflow), workflow-engine, audit-chain | append-only |
| `MeetingInstanceEnded` | host ends or last participant leaves | foundry-runtime (post-meeting summary T1), workflow-engine, audit-chain | append-only |
| `ParticipantJoined` / `ParticipantLeft` | join/leave | audit-chain, presence | append-only |
| `RecordingStarted` / `RecordingFinalized` | host or auto-start; finalize on end | ontology (`Recording` write), audit-chain, retention worker | append-only |
| `TranscriptionStreamed` (rate-limited) | live caption tick | (internal only; not externally consumed for cardinality) | append-only |
| `TranscriptSealed` | post-meeting transcript ready | foundry-runtime (translate/summary), search-index, audit-chain | append-only |
| `SummaryProduced` | T1 AI summary ready | mail (post-meeting digest), workflow-engine, audit-chain | append-only |
| `BreakoutRoomCreatedDissolved` | host opens/closes breakouts | audit-chain | append-only |
| `LiveStreamStartedStopped` | RTMP egress lifecycle | audit-chain | append-only |
| `WebinarRegistrationOpenedClosed` | host opens/closes registration | audit-chain | append-only |
| `MlsEpochAdvanced` (E2E mode) | epoch rotation | audit-chain | append-only |
| `EDiscoveryHoldOpenedClosed` | compliance-officer action | audit-chain, retention worker | append-only |
| `FourEyesDisclosureExecuted` | tenant-admin pair approves recording disclosure | audit-chain | append-only |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `CalendarEventCreated` / `CalendarEventUpdated` | calendar | meeting-room | bind meet-link into calendar event |
| `OntologyEntityChanged` (Person/Team) | ontology | participant | refresh participant directory cache |
| `TenantRetentionPolicyUpdated` | tenancy | recording | reassign recording retention bounds |
| `AuditChainSealed` | audit-chain | (read-only) | confirm audit-write durability |
| `MessengerHuddleGraduateRequest` | messenger | meeting-instance | promote an ad-hoc huddle to a `meet` named room |

### Ontology writes

| Object Type | Written by BC | Audit trail |
|---|---|---|
| `Meeting{meeting_room_id, tenant_id, name, lobby_policy, members, retention_policy_id}` | `meeting-room` | Ed25519 |
| `MeetingInstance{instance_id, room_id, start_at, end_at, attendee_refs, recording_ref?}` | `meeting-instance` | Ed25519 |
| `Recording{recording_id, instance_id, blob_ref, content_hash, retention_bound}` | `recording` | Ed25519 |
| `Transcript{transcript_id, instance_id, languages, content_hash, seal}` | `transcription` | Ed25519 |
| `WebinarRegistration{registration_id, webinar_id, attendee_ref, status}` | `webinar` | Ed25519 |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Person`, `Team` | `participant` | `find_by(user_ref, tenant_id)` |
| `CalendarEvent` | `meeting-instance` | `lookup(event_id)` — binding side |
| `RetentionPolicy` | `recording`, `transcription` | `lookup(tenant_id, context_kind)` |
| `TenantContext` | every BC | tenant scope + pack jurisdiction lookup |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Google Meet | Workspace-bundled video meetings | calendar integration; captions; recording; large audience | `support.google.com/meet` |
| Zoom | Free/Pro/Business/Enterprise/Webinars/Events | webinar; breakout rooms; live-translation; recording cloud; large-scale | `support.zoom.us` |
| Microsoft Teams Meetings | M365 meetings + webinars + Town Hall | enterprise SSO; eDiscovery; HIPAA via BAA; large-scale events | `learn.microsoft.com/microsoftteams` |
| Cisco Webex (Meet/Events/Webinars) | enterprise + government tier | large-audience webinar; FedRAMP | `help.webex.com` |
| GoToMeeting / GoToWebinar | mid-market webinar leader | webinar focus; recording; registration | `support.goto.com` |
| Whereby | browser-first; embeddable rooms | embeddable URLs; no-install; small group | `whereby.com/information` |
| Jitsi Meet | OSS reference + self-hosted | OSS substrate; no vendor lock | `jitsi.org` |
| Daily.co | API-first developer platform | embeddable; simple SDK | `docs.daily.co` |
| Vonage Meet API | telco API focus | programmable rooms; PSTN dial-in | `developer.vonage.com` |
| 100ms | low-latency video infra | sub-second live; large events | `100ms.live` |
| Around | small-team UX-focus | overlay video; novel UX | `around.co` |
| Mmhmm | presenter-focused (slides+camera composite) | streamer-style presentation | `mmhmm.app` |
| Vimeo Live (subset) | live-streaming-only | RTMP + HLS only; not interactive | `vimeo.com/live` |

Key parity gaps to close (ordered by priority):

1. **Sovereign-tenant residency by data-model** — none of Google Meet / Zoom / Teams enforce per-pack residency natively; oyatie does (11 packs at GA-readiness).
2. **OpenSLO-gated promotion of meeting features** — ADR-0139 means new meet features cannot ship while burn-rate is hot; competitors have no SLO-promotion gate.
3. **Cedar v4.2 fine-grained policy on participants + tracks + recordings** — competitors expose only host/co-host/attendee role flags.
4. **First-class audit-chain Ed25519 over every recording + transcript + disclosure** — competitors emit opaque vendor logs.
5. **Interpretation channels via standardised LiveKit audio overlays** — Zoom Webex have proprietary equivalents; oyatie uses LiveKit + RTP packetisation per RFC 3550.
6. **E2E-by-tenant-opt-in (MLS RFC 9420 + Insertable Streams)** — Zoom's E2E is opt-in; Google Meet has E2E for client-to-client video calls only; oyatie + MLS unifies signaling + recording-gating policy.

## Performance Targets

(See Performance NFR table above.)

Error budget:
- Monthly error budget for room-create + participant-join: 0.05 % (≈ 22 min/month).
- Monthly error budget for media-plane (in-call): 0.1 % (≈ 43 min/month).
- Burn-rate alarm on `meet.room-create.availability` is 14.4× burn rate over 1h.
- Error budget policy: `microservices/meet/runbooks/error-budget-policy.md` (Slice B).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres for meeting-room metadata + participant log + recording manifest; Valkey for per-meeting presence + signaling-session-state + lobby queues; S3 for recordings + transcripts + summaries; Meilisearch for transcript search; LiveKit SFU cluster stateless beyond room registry; SRS for RTMP egress.

**Active-active compatibility**: stateless `meet-rest` + LiveKit SFU + SRS; Postgres logical-replicated within pack; Valkey primary-replica HA; S3 cross-AZ replication.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Concurrent meetings | 5 000 | 50 000 | LiveKit pod CPU > 70% |
| Concurrent participants | 50 000 | 500 000 | LiveKit cluster scale-out |
| Recordings/day | 50 000 | 500 000 | S3 PUT rate > 70% provisioned |
| Webinar attendees (single event) | 1 000 interactive | 10 000 (interactive) / 100 000 (broadcast via WHIP/HLS mesh) | egress pod scale-out |
| Transcripts produced/day | 50 000 | 500 000 | Whisper GPU pool depth |
| Live caption sessions concurrent | 5 000 | 50 000 | Whisper streaming GPU |

Scale-out policy:
- HPA on `meet-meeting-instance-rest`: CPU > 70 %, min 6, max 100 replicas.
- LiveKit StatefulSet sharded by `(tenant_id, room_id) mod N`; HPA-bounded by GPU node availability for transcription.
- Whisper transcription pool: GPU node selector with `nvidia.com/gpu` request; reserved per pack region.
- ffmpeg recording workers under gVisor sandbox; HPA on queue depth > 5s.

Sharding:
- Meeting metadata partitions by `(tenant_id, room_id mod N)`.
- Recording manifest partitions by `(tenant_id, year-month)`.
- Transcript search index per-tenant.
- `oya-check-shardability-cli` lane verifies partition keys present in every kernel struct.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Create named room + join + record + transcribe roundtrip completes within performance budget | `microservices/meet/tests/e2e/room-record-transcribe.rs` |
| AC-02 | Calendar event creation produces a meet link bound to that event | `tests/e2e/calendar-bound-meeting.rs` |
| AC-03 | Recording legal hold preserves recording + transcript past retention | `tests/e2e/hold-recording.rs` |
| AC-04 | Webinar mode: 10k attendees fan-out p99 ≤ 5s with WHIP mesh | `tests/e2e/webinar-fanout.rs` |
| AC-05 | E2E (MLS) mode disables server-side recording + transcription by Cedar deny | `tests/e2e/e2e-mode-blocks-recording.rs` |
| AC-06 | Lobby + waiting room: guest is held until host approves | `tests/e2e/lobby-approval.rs` |
| AC-07 | Breakout rooms create, route participants, re-merge cleanly | `tests/e2e/breakout-rooms.rs` |
| AC-08 | Live caption p99 ≤ 500ms; multi-language overlay | `tests/e2e/live-caption-latency.rs` |
| AC-09 | RTMP egress to YouTube smoke + WHIP fallback handshake | `tests/e2e/live-stream-egress.rs` |
| AC-10 | Four-eyes disclosure on recording requires two distinct principals + audit-chain seal | `tests/e2e/four-eyes-recording-disclosure.rs` |
| AC-11 | `oya gate validate per-microservice-layout --microservice meet` exits 0 | ADR-0131 lane |
| AC-12 | `oya gate validate authority-cohesion --microservice meet` exits 0; HG-MEET registered | ADR-0123 lane |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | PSTN dial-in (Twilio Voice / Vonage) — phase or out-of-scope? | axis-meet + gtm | successor-IP ADR |
| 2 | Federation: accept incoming SIP / Matrix calls from external systems? | council-architecture | ADR after S-tier |
| 3 | Interpretation channels: human-only, AI-only, or both? | axis-meet + axis-foundry-runtime | ADR-MEET-0007 (next sprint) |
| 4 | Self-observability emission posture: one tenant or per-pack? | axis-meet + axis-observability | resolved in IP-007 |
| 5 | Live whiteboard collaborative-editing: own BC or use `slides` µservice's draft surface? | council-architecture | successor-IP ADR |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0008 | Data Use Boundary | data-class invariants |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application → usecase | layer naming |
| ADR-0135 | Connect dual-context (parallel) | net-new µservice authorisation |
| ADR-0139 | Agentic SLO-gated promotion | gates meet releases |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | Suite-and-bundle dissolution | new µservices are single-concern + flat |
| ADR-0133 | Industry best-practice conformance | HG-MEET under this |
| ADR-MEET-0001 | SFU substrate selection | LiveKit |
| ADR-MEET-0002 | Recording + transcription pipeline | Whisper + ffmpeg + gVisor |
| ADR-MEET-0003 | E2E encryption for meetings | MLS RFC 9420 + Insertable Streams |
| ADR-MEET-0004 | Live-streaming egress policy | RTMP + WHIP |
| ADR-MEET-0005 | Large-audience + webinar architecture | SFU mesh + MCU mix-down + WHIP/HLS |
| ADR-MEET-0006 | AI feature bounds | EU AI Act classification |
| ADR-MSGR-0001 | Huddles placement | substrate-sharing boundary (LiveKit + coturn) |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
