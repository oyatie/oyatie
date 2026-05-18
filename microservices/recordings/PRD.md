---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-recordings
microservice: recordings
status: Accepted
sales_segment: cross-suite-substrate-and-hero-product
tier: hero-product
milestone_first_ship: M02-foundation
bominal_source: [ADR-0029-workspace-recordings-adjunct.md, ADR-0215-connect-retention-legal-hold-dual-context.md]
related_adrs:
  - ADR-0008
  - ADR-0056
  - ADR-0105
  - ADR-0106
  - ADR-0135
  - ADR-0139
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-RECORDINGS-0001
  - ADR-RECORDINGS-0002
  - ADR-RECORDINGS-0003
  - ADR-RECORDINGS-0004
  - ADR-RECORDINGS-0005
  - ADR-RECORDINGS-0006
  - ADR-RECORDINGS-0007
related_specs:
  - /specs/microservices/recordings.json
  - /specs/per-microservice-flat-layout.json
  - /specs/agentic-slo-gated-promotion.json
date: 2026-05-17
owner_team: axis-recordings
doc_status: published
---

# PRD-recordings: Centralised Recording Store + Transcript + Searchable Archive

## Purpose

The `recordings` µservice is the **centralised, audit-grade recording store**
for every audio/video/screen-capture artifact produced across oyatie:
`meet` sessions, `messenger` huddles, live-broadcast streams, manual audio/video
uploads, and standalone screen captures. It is the single µservice in oyatie
where a recording **lives** (durable storage + transcript + redaction +
retention + legal-hold + eDiscovery + export); the producing µservices
(`meet`, `messenger`, future live-broadcast) only **record** and emit to this
µservice via durable ingest contract.

This µservice ships under ADR-0132 (no-suite forward-policy) as a stand-alone
µservice at the architecture layer, factored out of the legacy Connect-suite
by parallel ADR-0135. Bominal predecessor: `workspace-recordings` adjunct from
Bominal ADR-0029.

This µservice is **a hero product** end-user-facing through Workflow Studio
shell (Recordings tab) and standalone clients (web + desktop + mobile), and a
**shared substrate** consumed by other oyatie products via the
`recordings.recording.v1` / `recordings.transcript.v1` Workflow events and the
`Recording` / `Transcript` Ontology object types.

### Critical differentiation — recordings vs. meet/messenger huddles

`meet` and `messenger` (huddles) **produce** recordings as a side-effect of a
live session. The recording lifecycle (durable storage, transcript,
redaction, retention policy enforcement, legal-hold seal, export, eDiscovery)
is owned **here**, not in the producing µservice. The producing µservice
emits via the `recording.ingest.v1` durable contract (per ADR-RECORDINGS-0007)
and stops caring; the recording lives in the recordings µservice from that
point forward.

## Tenant Value

- **Tenant Outcome 1 — One archive for every recording, regardless of source.**
  Tenants get Otter.ai-class transcript search + Zoom-Cloud-Recording-class
  playback + Microsoft-Stream-class enterprise archive — across meet,
  messenger huddles, live-broadcast, manual uploads — in one searchable surface.
- **Tenant Outcome 2 — Audit-grade retention + legal hold.** Every recording
  carries a tenant-tier-bound retention policy with per-pack defaults (SEC
  17a-4(f) WORM for pack-us-financial, HIPAA 6y for pack-us-healthcare, KR
  전자문서법 for pack-kr). Legal-hold engagement is **load-bearing**:
  100 % correctness, p99 ≤ 1s engagement latency, audit-chain seal per event.
- **Tenant Outcome 3 — Transcript-native search across the archive.** Speaker-
  diarised transcript with timestamps + per-segment confidence. Cross-recording
  search via Meilisearch returns Cedar-permitted results only; p99 ≤ 300ms
  across a 1k-hour archive.
- **Tenant Outcome 4 — Auto-redaction of PII + manual redaction overlay.**
  Whisper transcription emits PII candidates; per-segment redaction overlay
  hides PII at playback without mutating source media (GDPR Art. 25 + HIPAA
  Safe Harbor §164.514). Manual redaction layer addable per court order.
- **Tenant Outcome 5 — eDiscovery export with chain-of-custody.** Court-order
  workflow emits a tar.gz bundle with media + transcript + redaction overlay
  + audit-chain seal + Merkle root, signed by an export-worker SPIFFE
  identity (Ed25519). Conforms to FRCP Rule 26(f)/34 + Sedona Conference + ISO
  27037:2012 digital evidence handling.
- **Tenant Outcome 6 — Cross-µservice publish bridge.** Workflow events let a
  finished recording auto-summarise via foundry-runtime, auto-translate
  transcript via the `translate` µservice, auto-publish to social/shorts feeds,
  and auto-share via mail — gated by Cedar policy + tenant config.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | source µservice (meet/messenger/live-broadcast/manual upload) | to emit a recording to recordings via durable ingest contract | the recording archives reliably | recording-ingest | Must |
| FR-02 | end-user | to list my recordings filtered by source / date / participant | I can find a past recording | recording | Must |
| FR-03 | end-user | to play a recording with chapter-skip + caption-toggle + speaker-filter + 2x-speed | I review efficiently | playback | Must |
| FR-04 | end-user | to read the speaker-diarised transcript with timestamps + confidence per segment | I navigate audio quickly | transcript | Must |
| FR-05 | end-user | to search across recordings + transcripts | I recover old context | search | Must |
| FR-06 | end-user | to download the recording as MP4 / MP3 / WAV + transcript as VTT / SRT / PDF / DOCX | I share or archive offline | export | Must |
| FR-07 | end-user | to share a recording with signed-URL + password + view-count cap + expiry | I share securely | share-link | Must |
| FR-08 | compliance-officer | to engage a legal hold on a recording or tenant scope | regulatory request is satisfied | retention-policy + legal-hold | Must |
| FR-09 | compliance-officer | to manually redact a segment (visually + audibly) without mutating source media | court-ordered redaction works | redaction | Must |
| FR-10 | compliance-officer | to issue an eDiscovery export bundle with chain-of-custody Merkle seal | FRCP / SEC / FINRA / KR 전자문서법 satisfied | ediscovery | Must |
| FR-11 | tenant-admin | to set per-tenant retention policy override (within pack-default ceiling) | tenant policy is honoured | retention-policy | Must |
| FR-12 | end-user | to view auto-extracted chapter markers + auto-generated summary | I skim long recordings | chapter-marker + summary | Should |
| FR-13 | end-user | to view auto-translated transcript in N other languages | cross-language accessibility works | translation | Should |
| FR-14 | Workflow Studio | to trigger auto-publish-to-feed / auto-share-via-mail / auto-task-extract post-recording | post-recording workflows fire | recording (event emit) | Must |
| FR-15 | Workflow Studio | to consume `RecordingPublished` / `TranscriptReady` / `LegalHoldEngaged` events | downstream automation works | recording (event emit) | Must |
| FR-16 | tenant-operator | to apply dynamic per-viewer watermark to sensitive recordings | leak attribution works | watermarking | Must |
| FR-17 | external API consumer | to ingest a manual recording (audio/video file) with a presigned upload URL + post-ingest scan | external workflows can add to the archive | recording-ingest | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Recording list (100 recordings) | ≤ 50ms | ≤ 200ms | ≤ 500ms | Postgres + Redis cache |
| Playback-start latency (warm CDN) | ≤ 150ms | ≤ 400ms | ≤ 800ms | HLS manifest fetch + first segment |
| Playback-start latency (cold) | ≤ 400ms | ≤ 1s | ≤ 2s | CDN miss; fallback to S3 + transcode hint |
| Transcript-search across 1k-hour archive | ≤ 80ms | ≤ 300ms | ≤ 800ms | Meilisearch 0.10.0 |
| Transcript-render (10-min recording) | ≤ 150ms | ≤ 500ms | ≤ 1.2s | JSON → DOM render |
| Redaction-render per segment | ≤ 300ms | ≤ 1s | ≤ 2s | overlay apply + transcode-on-the-fly |
| Export MP4 (transcode) | ≤ duration × 0.15 | ≤ duration × 0.3 | ≤ duration × 0.5 | ffmpeg 7.x gVisor sandbox |
| Export transcript-PDF | ≤ 1s | ≤ 3s | ≤ 6s | Pandoc 3.x |
| Legal-hold engagement | ≤ 500ms | ≤ 1s | ≤ 2s | **load-bearing 100% correctness invariant** |

### Security

- Cedar v4 default-deny across every action (`policy/cedar/*.cedar`).
- All media + transcript + redaction overlay encrypted at rest under tenant
  DEK envelope encryption (per Bominal ADR-0111). KMS-shred on retention
  expiry (per ADR-RECORDINGS-0002).
- Signed-URL share links: short-lived (default 24h, max 7d), HMAC-signed by
  per-tenant secret resolved at request time from `${openbao:secret/recordings/<tenant>/share-link-hmac}`.
- Per-viewer dynamic watermark (PRD FR-16): visible + invisible (steganographic)
  watermarks for sensitive recordings; mitigates screen-capture leak.
- Upload-time malware scan via OPSWAT/ClamAV adapter; refused if positive.
- Pre-recording phase: ingest contract validates SPIFFE identity of producer
  (meet/messenger/live-broadcast).
- Transcript Whisper runs in foundry-runtime gVisor sandbox; ffmpeg transcode
  runs in gVisor sandbox; pyannote diarization runs in foundry-runtime gVisor.
- Per ADR-RECORDINGS-0003: redaction overlay does **not** mutate source media;
  source media is immutable post-ingest (WORM where pack requires).

### Audit + Compliance

- Every playback / share / redact / hold-engage / hold-release / export
  emits an audit-chain Ed25519-signed event with a Merkle commitment over
  prior events (Bominal ADR-0028). Audit-chain seals are cross-pack-portable
  (no PII).
- **Legal-hold chain-of-custody is a load-bearing 100 % correctness
  invariant** (SLO `legal-hold-chain-of-custody-correctness` at zero
  error-budget; any breach pages the on-call axis-recordings lead).
- **Retention-policy correctness is a load-bearing 100 % correctness
  invariant** (SLO `retention-policy-correctness` at zero error-budget; any
  breach is a Sev-1).
- Per-pack retention defaults + ceilings + legal-hold semantics specified in
  ADR-RECORDINGS-0002 + `policy/data-residency.md`.

### Availability + SLO

- Availability target: 99.95 % monthly for recording-list + playback-start.
- 99.99 % for legal-hold engagement (load-bearing).
- RTO: ≤ 15 min for recording metadata; ≤ 30 min for media (S3 cross-AZ).
- RPO: ≤ 1 min for metadata; ≤ 5 min for media (chunked upload resumable).

### Data residency

- Per-tenant pack pinning per ADR-0117. Recording + transcript + redaction
  overlay reside in the tenant's pack region; cross-pack replication
  forbidden by default. Audit-chain seals are pack-portable.
- Legal-hold cells (per `cell` µservice) are isolated to prevent
  same-substrate noisy-neighbour from a held recording.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` →
`usecase` rename). Layers used: `kernel`, `domain`, `usecase`, `api`,
`adapter`, `adapter-postgres`, `adapter-redis`, `adapter-s3`, `adapter-cdn`,
`adapter-meilisearch`, `adapter-whisper`, `adapter-pyannote`, `adapter-ffmpeg`,
`adapter-pandoc`, `adapter-clamav`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `recording` | `oya-recordings-recording-{kernel,domain,usecase,api,adapter-postgres,adapter-s3,rest,worker,sdk,app}` | Recording asset metadata + manifest + chapter index + speaker index | `Recording`, `RecordingManifest`, `ChapterIndex`, `SpeakerIndex` |
| `media-segment` | `oya-recordings-media-segment-{kernel,domain,usecase,adapter-s3,adapter-cdn-cloudfront-stub-or-self,adapter-ffmpeg,worker,app}` | HLS/DASH chunked segments + thumbnail keyframes + multi-bitrate ladder | `MediaSegment`, `ThumbnailKeyframe`, `BitrateLadder` |
| `transcript` | `oya-recordings-transcript-{kernel,domain,usecase,api,adapter-postgres,adapter-whisper,adapter-pyannote,rest,worker,sdk,app}` | Speaker-diarised transcript + timestamps + per-segment confidence | `Transcript`, `TranscriptSegment`, `SpeakerCluster`, `Diarization` |
| `translation` | `oya-recordings-translation-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | Cross-µservice translation handoff to `translate` µservice | `TranslationRequest`, `TranslatedTranscript` |
| `redaction` | `oya-recordings-redaction-{kernel,domain,usecase,api,adapter-postgres,adapter-ffmpeg,rest,worker,sdk,app}` | PII auto-redact + manual redaction overlay; immutable source | `Redaction`, `RedactionOverlay`, `RedactionSpan` |
| `chapter-marker` | `oya-recordings-chapter-marker-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}` | Auto-via-diarization + manual chapter markers | `ChapterMarker`, `ChapterSet` |
| `summary` | `oya-recordings-summary-{kernel,domain,usecase,api,adapter-postgres,adapter-whisper,rest,worker,sdk,app}` | Auto-summary (semantic + chronological flavors) via foundry-runtime | `Summary`, `SummaryDraft`, `SummaryFlavor` |
| `thumbnail-pack` | `oya-recordings-thumbnail-pack-{kernel,domain,usecase,adapter-s3,adapter-ffmpeg,worker,app}` | Auto-extracted thumbnail per chapter | `ThumbnailPack`, `ThumbnailFrame` |
| `search` | `oya-recordings-search-{kernel,domain,usecase,api,adapter-meilisearch,rest,sdk,app}` | Cross-recording + cross-transcript search via Meilisearch | `SearchIndex`, `SearchHit`, `SearchFacet` |
| `retention-policy` | `oya-recordings-retention-policy-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}` | Per-pack default + tenant-tier override + legal-hold override; KMS-shred orchestration | `RetentionPolicy`, `RetentionEffectiveBound`, `KmsShredKeyRef` |
| `legal-hold` | `oya-recordings-legal-hold-{kernel,domain,usecase,api,adapter-postgres,rest,worker,sdk,app}` | Court-order workflow with chain-of-custody seal; load-bearing | `LegalHold`, `LegalHoldEngagement`, `ChainOfCustodyEvent` |
| `export` | `oya-recordings-export-{kernel,domain,usecase,api,adapter-ffmpeg,adapter-pandoc,worker,sdk,app}` | MP4/MP3/WAV media + VTT/SRT/PDF/DOCX transcript export bundles | `ExportRequest`, `ExportBundle`, `ExportManifest` |
| `share-link` | `oya-recordings-share-link-{kernel,domain,usecase,api,adapter-postgres,adapter-redis,rest,worker,sdk,app}` | Signed-URL + password + view-count cap + expiry | `ShareLink`, `ShareLinkPolicy`, `ViewSession` |
| `playback` | `oya-recordings-playback-{kernel,domain,usecase,api,adapter-cdn-cloudfront-stub-or-self,adapter-redis,rest,sdk,app}` | Chapter-skip + caption-toggle + speaker-filter + 2x-speed | `PlaybackSession`, `PlaybackCheckpoint`, `CaptionRendition` |
| `ediscovery` | `oya-recordings-ediscovery-{kernel,domain,usecase,api,adapter-postgres,worker,sdk,app}` | Court-order workflow + chain-of-custody seal per audit-chain | `EDiscoveryHold`, `EDiscoveryExport`, `CourtOrderRef` |
| `watermarking` | `oya-recordings-watermarking-{kernel,domain,usecase,adapter-ffmpeg,worker,app}` | Per-viewer dynamic visible + steganographic watermark | `WatermarkPolicy`, `WatermarkOverlay` |
| `drm-stub` | `oya-recordings-drm-stub-{kernel,domain,usecase,adapter,app}` | DRM hook (per shorts ADR-SHORTS-0004); Widevine/Fairplay/PlayReady stub | `DrmPolicy`, `DrmLicenseRef` |
| `audio-loudness` | `oya-recordings-audio-loudness-{kernel,domain,usecase,adapter-ffmpeg,worker,app}` | EBU R128 loudness normalisation | `LoudnessTarget`, `LoudnessMeasurement` |
| `video-encode-ladder` | `oya-recordings-video-encode-ladder-{kernel,domain,usecase,adapter-ffmpeg,worker,app}` | HLS multi-bitrate ladder + CMAF segmentation | `BitrateRung`, `LadderManifest` |
| `accessibility-captions` | `oya-recordings-accessibility-captions-{kernel,domain,usecase,api,adapter-postgres,rest,sdk,app}` | Live-captions storage + descriptive-audio metadata | `CaptionTrack`, `DescriptiveAudioTrack` |
| `recording-ingest` | `oya-recordings-recording-ingest-{kernel,domain,usecase,api,adapter,adapter-s3,rest,worker,sdk,app}` | Multi-source durable ingest contract (per ADR-RECORDINGS-0007) | `IngestRequest`, `IngestSession`, `IngestSourceKind` |

Naming justification — `recording`:

```
NAME: oya-recordings-recording-<layer>
JUSTIFICATION:
- microservice = recordings: per ADR-0131 per-microservice flat layout.
- bc-tokens = recording: primary BC; bc differs from microservice (singular).
  ADR-0056 v4.1 BC-optionality rule honoured (BC token required because
  microservice plural ≠ BC singular).
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-postgres / -adapter-s3 / -adapter-cdn-* /
  -adapter-meilisearch / -adapter-whisper / -adapter-pyannote /
  -adapter-ffmpeg / -adapter-pandoc / -adapter-clamav are canonical
  *-adapter-<backend> per ADR-0105 Amendment 3.
```

Port traits declared in each kernel (zero business logic; zero I/O):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `RecordingRepository` | `oya-recordings-recording-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING`, `AUDIT` |
| `MediaBlobStore` | `oya-recordings-media-segment-kernel` | `-adapter-s3` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` (sometimes), `PHI` (pack-us-healthcare) |
| `CdnPurger` | `oya-recordings-media-segment-kernel` | `-adapter-cdn-cloudfront-stub-or-self` | `INTERNAL_ONLY` |
| `TranscriptStore` | `oya-recordings-transcript-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING`, `PHI` |
| `SpeechRecogniser` | `oya-recordings-transcript-kernel` | `-adapter-whisper` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING`, `PHI` |
| `SpeakerDiariser` | `oya-recordings-transcript-kernel` | `-adapter-pyannote` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `RedactionOverlayStore` | `oya-recordings-redaction-kernel` | `-adapter-postgres` | `INTERNAL_ONLY` (overlay is metadata; not media) |
| `MediaTranscoder` | `oya-recordings-export-kernel`, `-watermarking-kernel`, `-video-encode-ladder-kernel`, `-audio-loudness-kernel` | `-adapter-ffmpeg` (shared sandbox) | `BEHAVIORAL_TENANT_PRODUCT` |
| `TranscriptDocumentRenderer` | `oya-recordings-export-kernel` | `-adapter-pandoc` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` |
| `MalwareScanner` | `oya-recordings-recording-ingest-kernel` | `-adapter-clamav` | `INTERNAL_ONLY` |
| `SearchIndex` | `oya-recordings-search-kernel` | `-adapter-meilisearch` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ShareLinkCache` | `oya-recordings-share-link-kernel` | `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `PlaybackSessionCache` | `oya-recordings-playback-kernel` | `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `LegalHoldStore` | `oya-recordings-legal-hold-kernel` | `-adapter-postgres` | `AUDIT`, `BEHAVIORAL_TENANT_PRODUCT` |
| `RetentionPolicyStore` | `oya-recordings-retention-policy-kernel` | `-adapter-postgres` | `AUDIT` |
| `CedarRecordingPolicy` | `oya-recordings-recording-kernel` | `-adapter` (Cedar evaluator) | `INTERNAL_ONLY` |
| `AuditChainClient` | (cross-BC) `-kernel` per BC | (cross-BC) `-adapter` to audit-chain µservice | `AUDIT` |

Data-class enforcement: `oya-check-data-class` LEAN lane refuses unannotated
fields.

Cross-product rule: `recordings` MUST NOT import any other product µservice
crate at any layer. Cross-product flows go through Workflow (events) or
Ontology (entity reads/writes). LEAN-A2 lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice recordings` — dependency-direction
- `oya gate validate lean-a2 --microservice recordings` — cross-product-refusal
- `oya gate validate port-location --microservice recordings`
- `oya gate validate layer-correctness --microservice recordings`
- `oya gate validate per-microservice-layout --microservice recordings`
- `oya gate validate statelessness --microservice recordings`
- `oya gate validate shardability --microservice recordings`
- `oya gate validate authority-cohesion --microservice recordings` (HG-RECORDINGS)
- `oya gate validate retention-policy-correctness --microservice recordings` (NEW; load-bearing)
- `oya gate validate legal-hold-chain-of-custody-correctness --microservice recordings` (NEW; load-bearing)

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `RecordingIngested` | producer (meet/messenger/etc.) emits via ingest contract | transcript-worker, audit-chain, retention-policy-assigner | append-only |
| `RecordingPublished` | transcript + transcode + thumbnail-pack complete | mention notification, workflow-engine, ontology | append-only |
| `RecordingPlayed` | end-user plays a recording | audit-chain | append-only |
| `RecordingShared` | end-user creates share-link | audit-chain, retention-policy (extends retention if pinned) | append-only |
| `RecordingRedacted` | compliance-officer adds redaction overlay | audit-chain, search-index re-emit | append-only |
| `RecordingDeleted` | end-user / retention purge | audit-chain, search-index purge, KMS-shred worker | tombstone |
| `TranscriptReady` | transcription + diarization complete | translation worker (if enabled), summary worker, search-index | append-only |
| `TranslationReady` | `translate` µservice returns translated transcript | search-index, ontology | append-only |
| `SummaryReady` | summary worker emits | workflow-engine, ontology | append-only |
| `LegalHoldEngaged` | compliance-officer engages hold | audit-chain, retention-policy suspend, ediscovery readiness | append-only |
| `LegalHoldReleased` | court-order / compliance-officer releases hold | audit-chain, retention-policy resume | append-only |
| `EDiscoveryExportExecuted` | ediscovery export bundle sealed | audit-chain | append-only |
| `RetentionPolicyApplied` | retention worker enforces purge or KMS-shred | audit-chain | append-only |
| `WatermarkRotated` | per-viewer watermark key rotated | audit-chain | append-only |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `MeetSessionEnded` | `meet` | recording-ingest | finalise meet recording into archive (per ADR-RECORDINGS-0007) |
| `MessengerHuddleEnded` | `messenger` | recording-ingest | finalise huddle recording into archive |
| `LiveBroadcastEnded` | (future) `live-broadcast` | recording-ingest | finalise live-broadcast recording into archive |
| `OntologyEntityChanged` (Person/Team) | `ontology` | transcript | refresh speaker-name binding |
| `TenantRetentionPolicyUpdated` | `tenancy` | retention-policy | reassign retention bounds |
| `TranslateResultReady` | `translate` | translation | persist translated transcript |
| `AuditChainSealed` | `audit-chain` | (read-only) | confirm audit-write durability |
| `WorkflowStudioRunStarted/Completed` | `workflow-engine` | recording-ingest | record workflow-emitted recording (e.g., terminal capture) |

### Ontology writes

| Object Type | Written by BC | Audit trail |
|---|---|---|
| `Recording{recording_id, tenant_id, source_kind, context_kind, retention_policy_id, duration_seconds, content_hash}` | `recording` | Ed25519 |
| `Transcript{transcript_id, recording_id, lang_code, content_hash, speaker_count, segment_count}` | `transcript` | Ed25519 |
| `Speaker{speaker_id, recording_id, cluster_label, named_ref?}` | `transcript` (diarization) | Ed25519 |
| `Redaction{redaction_id, recording_id, span_start_ms, span_end_ms, reason}` | `redaction` | Ed25519 |
| `LegalHold{hold_id, tenant_id, scope, engaged_at, released_at?, court_order_ref?}` | `legal-hold` | Ed25519 |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Person`, `Team` | `transcript` (speaker naming), `share-link` (recipient resolve) | `find_by(ref, tenant_id)` |
| `RetentionPolicy` | `retention-policy` | `lookup(tenant_id, source_kind, pack_id)` |
| `TenantContext` | every BC | tenant scope + pack jurisdiction lookup |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Otter.ai | Transcript-focused recording archive | speaker-diarised transcript; live transcription; semantic search; auto-summary; speaker training | `otter.ai/api-docs` |
| Rev.com | Human + AI transcription | high-accuracy transcription; verbatim option; legal-grade attestation | `rev.com/help` |
| Descript | Post-edit + transcription | text-based editing of audio/video; overdub; transcript-as-timeline | `help.descript.com` |
| Fireflies.ai | Meeting recording + summary | meeting-specific integration; auto-summary; CRM hand-off | `fireflies.ai/docs` |
| Tactiq | Live meeting transcript | live in-meeting transcript; quote-bookmarking | `tactiq.io/learn` |
| Sembly AI | Meeting AI agent | task extraction; meeting-minutes generation; multilingual | `sembly.ai/docs` |
| Read.ai | Meeting metrics + summary | sentiment; engagement scoring | `read.ai/help` |
| Krisp recording | Noise-cancellation + recording | per-speaker isolation; on-device privacy | `krisp.ai/docs` |
| Zoom Cloud Recording | Enterprise meeting archive | cloud + local recording; per-tenant storage; ECM connector | `support.zoom.com` |
| Microsoft Stream | Enterprise video archive | Stream-on-SharePoint; SSO + RBAC; e-discovery | `learn.microsoft.com/stream` |
| Google Meet recordings | Drive-backed meet recordings | Drive + Meet integration; auto-transcription | `support.google.com/meet` |
| Loom | Async screen recording + share | screen + cam capture; share-link analytics | `loom.com/help` |
| Bubbles | Async video collab | inline comments + reactions on the video timeline | `usebubbles.com/help` |
| Vidcast (Webex) | Async video + transcript | enterprise integration; teams chat | `help.webex.com` |
| Vimeo Record | Cam + screen + livestream | OTT-style hosting; private-show controls | `help.vimeo.com` |
| mmhmm | Presenter overlay + recording | presenter-mode; backgrounds; share-as-video | `mmhmm.app/help` |
| Veed.io | Post-edit + subtitles | browser-based edit; auto-subtitles | `veed.io/help` |

Key parity gaps to close (ordered by priority):

1. **One archive for every recording, regardless of source** — none of the
   competitors centralise meet/huddle/live-broadcast/manual upload in one
   tenant-scoped Cedar-policy-evaluated archive. Target: ADR-RECORDINGS-0007
   durable ingest contract.
2. **Legal-hold load-bearing 100 % correctness** — Zoom Cloud Recording +
   Stream + Drive recordings support hold, but with eventual-consistency
   semantics. Target: this PRD's load-bearing SLO + chain-of-custody seal.
3. **Auto-redaction without source mutation** — competitors either don't
   redact or destructively rewrite. Target: ADR-RECORDINGS-0003 overlay model.
4. **Per-viewer dynamic + steganographic watermark** — competitors offer
   static visible watermark only. Target: ADR-RECORDINGS-0004 watermarking.
5. **OpenSLO + agentic gate** — none gate recording feature rollouts on SLO
   compliance; oyatie does (per ADR-0139).
6. **Multi-pack residency + per-pack regulatory overlays** — competitors are
   SaaS-region-coarse; oyatie is per-pack jurisdiction-pinned.
7. **eDiscovery chain-of-custody Merkle seal** — competitors emit a zip with
   no cryptographic chain. Target: ISO 27037:2012 + Sedona Conference.

## Performance Targets

(See Performance NFR table above.)

Error budget:
- Monthly error budget for playback-start availability: 0.05 % (≈ 22 min/month).
- Burn-rate alarm on `recordings.playback-start.availability` is 14.4× burn rate over 1h.
- Zero error budget on legal-hold-engagement availability + retention-policy correctness.
- Error budget policy: `microservices/recordings/runbooks/error-budget-policy.md` (Slice B).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres for
metadata + transcript + redaction overlay; S3 for media (multi-bitrate + raw
source); Redis for share-link + playback session; Meilisearch for search;
CDN for hot playback; foundry-runtime for Whisper + pyannote + summary.

**Active-active compatibility**: stateless REST + WebSocket gateway; Postgres
logical-replicated within pack; Redis primary-replica HA; S3 cross-AZ
replication; Meilisearch primary-only with snapshot DR.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Recordings / day | 50k | 1M | Postgres write IOPS > 70% |
| Hours of media / day | 10k | 200k | S3 PUT rate > 70% provisioned |
| Concurrent playbacks | 5k | 100k | CDN cache-miss > 30% |
| Search QPS | 100 | 5k | Meilisearch CPU > 70% |
| Transcript hours / day | 10k | 200k | foundry-runtime Whisper GPU queue > 60% |
| Active legal holds | 100 | 50k | per-cell cell-isolation breach risk |

Scale-out policy:
- HPA on recording-rest pods: CPU > 70 %, min 4, max 200 replicas.
- Postgres shard-by-tenant once cell hits 1M recordings/year aggregate.
- Redis cluster sharding by `(tenant_id, recording_id) mod N`.
- Meilisearch sharded by `tenant_id`.

Sharding:
- Recording metadata partitions by `(tenant_id, year-month)`.
- Transcript partitions by `(tenant_id, recording_id)`.
- Redaction overlay partitions by `(tenant_id, recording_id)`.
- Share-link partitions by `(tenant_id, share_link_id)`.
- `oya-check-shardability-cli` lane verifies partition keys are present in
  every kernel struct.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | A meet session ends + ingest contract finalises + transcription + transcode + thumbnail-pack + summary complete within ingest p99 budgets | `tests/e2e/meet-ingest-pipeline.rs` |
| AC-02 | A messenger huddle ends + ingest contract finalises | `tests/e2e/messenger-huddle-ingest.rs` |
| AC-03 | A manual upload (via presigned S3 URL + scan) ingests cleanly | `tests/e2e/manual-upload-ingest.rs` |
| AC-04 | Playback-start p99 ≤ 400ms (warm CDN) / 1s (cold) | `tests/e2e/playback-start-latency.rs` |
| AC-05 | Transcript-search p99 ≤ 300ms across 1k-hour archive | `tests/e2e/transcript-search-latency.rs` |
| AC-06 | Redaction overlay does not mutate source media | `tests/e2e/redaction-immutability.rs` |
| AC-07 | Export MP4 + transcript-PDF + audit-manifest bundle verifies under Ed25519 + Merkle | `tests/e2e/export-bundle-verify.rs` |
| AC-08 | Legal-hold engagement p99 ≤ 1s; 100 % correctness across 1000 holds | `tests/e2e/legal-hold-correctness.rs` |
| AC-09 | eDiscovery export bundle Merkle root validates against audit-chain seal | `tests/e2e/ediscovery-merkle-seal.rs` |
| AC-10 | Retention-policy purge respects legal-hold (held recordings never purged) | `tests/e2e/retention-vs-hold.rs` |
| AC-11 | `oya gate validate per-microservice-layout --microservice recordings` exit 0 | ADR-0131 lane |
| AC-12 | `oya gate validate authority-cohesion --microservice recordings` exit 0 | ADR-0123 lane; HG-RECORDINGS registered |
| AC-13 | `oya gate validate retention-policy-correctness --microservice recordings` exit 0 | NEW; load-bearing |
| AC-14 | `oya gate validate legal-hold-chain-of-custody-correctness --microservice recordings` exit 0 | NEW; load-bearing |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Whisper-large vs. Whisper-medium default — accuracy/latency tradeoff per pack | axis-recordings + axis-foundry-runtime | resolved in ADR-RECORDINGS-0001 |
| 2 | CDN backend pick: CloudFront vs. self-host (Bunny + Fastly + nginx-vod) for pack-cn/pack-ksa | ops-sre-reliability | resolved in ADR-RECORDINGS-0004; backend-qualified -adapter-cdn |
| 3 | Hot S3 + S3-Glacier-class cold tier — automatic age-down policy per pack | axis-recordings + ops-finops | resolved in ADR-RECORDINGS-0005 |
| 4 | Steganographic watermark detector — own µservice or recordings BC? | council-architecture | successor-IP ADR |
| 5 | DRM (Widevine + Fairplay + PlayReady) — own µservice or recordings BC? | council-architecture | successor-IP ADR (current: stub per shorts ADR-SHORTS-0004) |
| 6 | EU AI Act Annex III — when does auto-summary become high-risk (employment / legal context)? | council-privacy + ops-compliance | resolved in ADR-RECORDINGS-0006 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0008 | Data Use Boundary | data-class invariants |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application → usecase | layer naming |
| ADR-0135 | Connect dual-context (parallel) | dual-context isolation source |
| ADR-0139 | Agentic SLO-gated promotion | gates recordings releases |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | Suite-and-bundle dissolution | factored Connect into surfaces |
| ADR-0133 | Industry best-practice conformance | HG-RECORDINGS under this |
| ADR-RECORDINGS-0001 | Transcription + diarization pipeline | Whisper-large + pyannote |
| ADR-RECORDINGS-0002 | Retention + legal-hold policy | SEC 17a-4 + HIPAA + KR + EU |
| ADR-RECORDINGS-0003 | Redaction + PII policy | overlay model; immutable source |
| ADR-RECORDINGS-0004 | Playback + CDN strategy | HLS + DRM + watermark |
| ADR-RECORDINGS-0005 | Storage substrate tiered | hot s3 + cold s3-glacier |
| ADR-RECORDINGS-0006 | AI feature bounds | EU AI Act + transparency |
| ADR-RECORDINGS-0007 | Multi-source ingest contract | meet + huddle + live + manual |
| Bominal ADR-0029 | Workspace recordings adjunct | inherited Workspace-GA shape |
| Bominal ADR-0215 | Connect retention legal-hold dual-context | inherited |
| Bominal ADR-0028 | Audit-chain Merkle + Ed25519 | inherited |
| Bominal ADR-0111 | Ciphertext property type + envelope encryption | inherited |
