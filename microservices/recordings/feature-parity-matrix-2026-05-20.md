# recordings feature-parity matrix - 2026-05-20

- µservice: `recordings`
- Deliverable status: substantive audit deliverable 2 of 3.
- Counterpart 1: Zoom Cloud Recording.
- Counterpart 2: Gong.io.
- Counterpart 3: Otter.ai.
- Audit bar: union coverage, not a lowest-common-denominator match.
- Current doctrine: one industry-leader quality bar across tenant classes.
- Tier-retirement note: this document does not introduce feature tiers.
- Primary local anchors: `PRD.md`, `contracts/openapi/recordings.yaml`, `contracts/asyncapi/recordings-events.yaml`, `contracts/proto/recordings.proto`, `competitor-parity-matrix.md`, `capacity-model.md`, and SLO files.
- External source anchor Zoom: `https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0062627&trk=s-bl`.
- External source anchor Zoom AI: `https://library.zoom.com/zoom-workplace/artificial-intelligence/artificial-intelligence-bluepaper/ai-companion/ai-companion-features/zoom-recordings`.
- External source anchor Gong: `https://www.gong.io/conversation-intelligence`.
- External source anchor Gong help: `https://help.gong.io/docs/understanding-call-recording`.
- External source anchor Otter: `https://help.otter.ai/hc/en-us/articles/360047872833-Otter-ai-features`.
- External source anchor OtterPilot: `https://otter.ai/blog/otter-surpasses-1-billion-meetings-transcribed-and-launches-otterpilot-tm-the-smart-ai-meeting-assistant-to-eliminate-note-taking-and-automate-meeting-summaries`.

## §1 Counterpart 1 - Zoom Cloud Recording capability surface

1. Zoom records meeting video, audio, and chat text to the Zoom cloud, per Zoom support `KB0062627:2`.
2. Zoom recordings can be downloaded locally or streamed from a browser, per Zoom support `KB0062627:2`.
3. Zoom supports multiple recording layouts, including active speaker, gallery view, and shared screen, per Zoom support `KB0062627:3`.
4. Zoom cloud recording is available to licensed users on paid account classes, per Zoom support `KB0062627:5-13`.
5. Zoom cloud recordings have session-file generation constraints, including a 150-file limit for a live session, per Zoom support `KB0062627:14-18`.
6. Zoom allows hosts and co-hosts to start cloud recordings, per Zoom support `KB0062627:27-37`.
7. Zoom processes recordings after a meeting ends and places them in Recording & Transcripts, per Zoom support `KB0062627:57`.
8. Zoom storage capacity is plan-dependent: Pro/Business 10 GB per licensed user, Business Plus 15 GB, Enterprise unlimited, and Education Core 0.5 GB, per Zoom support `KB0067670:17-24`.
9. Zoom warns billing admins at 80 percent of subscribed storage capacity, per Zoom support `KB0067670:2-4`.
10. Zoom continues a recording that is already in progress when storage limit is reached, but prevents additional cloud recordings after limit exhaustion, per Zoom support `KB0060832` search evidence.
11. Zoom Smart Recording generates structured summaries, highlighted moments, action items, searchable time-stamped highlights, chapters, and speaker insights, per Zoom library `zoom-recordings:108-117`.
12. Zoom Voice Recorder generates audio files, searchable transcripts with speaker labels, and AI-generated summaries for in-person conversations, per Zoom library `zoom-recordings:122-135`.
13. Zoom has a strong native-meeting capture path but weaker service-local eDiscovery evidence than Oyatie's intended legal-hold/export surface.
14. Zoom has strong user-facing playback and share workflows, but product docs do not expose the same cross-pack retention/legal-hold policy depth as `PRD.md:140-169`.
15. Zoom is a baseline for capture, processing, layout inventory, cloud streaming, transcript access, storage governance, and AI meeting recall.

### §1.1 Zoom parity observations against recordings

1. Oyatie recordings has explicit ingest sources for Meet, Messenger calls, Live streams, manual uploads, screen captures, and external webhooks, per `PRD.md:37-48` and `ADR-RECORDINGS-0007-source-ingest-contract.md:31-56`.
2. Oyatie recordings has playback APIs, transcript APIs, search APIs, share-link APIs, legal-hold APIs, and export APIs, per `contracts/openapi/recordings.yaml:147-401`.
3. Oyatie recordings has explicit playback start SLOs, per `slos/playback-start-p99.openslo.yaml:5-16`.
4. Oyatie recordings has explicit transcript render and search SLOs, per `slos/transcript-render-p99.openslo.yaml:5-15` and `slos/transcript-search-p99.openslo.yaml:5-17`.
5. Oyatie recordings has explicit eDiscovery export SLOs, per `slos/ediscovery-export-mp4-p99.openslo.yaml:5-16` and `slos/ediscovery-export-transcript-pdf-p99.openslo.yaml:5-15`.
6. Oyatie recordings has stronger legal-hold evidence-chain intent than the public Zoom documentation reviewed, per `ADR-RECORDINGS-0002-retention-legal-hold-ediscovery.md:88-123`.
7. Oyatie recordings currently lacks explicit layout/file-fragment modeling equivalent to Zoom's active speaker, gallery, shared screen, and 150-file live-session semantics.
8. Oyatie recordings currently lacks explicit storage-capacity warning semantics equivalent to Zoom's 80 percent billing-admin alert.
9. Oyatie recordings currently lacks a first-class "Recording & Transcripts" portal contract naming convention, although UI surfaces exist under `ux/`.
10. Oyatie recordings should add explicit recording-processing lifecycle statuses matching ingest, queued, processing, transcript-ready, redaction-ready, published, held, expired, and deleted states.
11. Oyatie recordings should add explicit host/co-host delegation semantics for source services rather than relying on generic authorization language.
12. Oyatie recordings should add explicit cloud recording continuation behavior when storage caps are hit for demo-trial and paid contexts.
13. Oyatie recordings should separate recording source, layout variant, track variant, language interpreter view, chat transcript, and generated artifact records.
14. Oyatie recordings should define how in-person voice recordings enter the same evidence chain as meeting recordings.
15. Oyatie recordings should define AI summary, chapters, highlights, action items, and speaker insights in the contract if Zoom Smart Recording parity is desired.

## §2 Counterpart 2 - Gong.io capability surface

1. Gong positions conversation intelligence around automatic call recording and transcription, per Gong `conversation-intelligence:32-35`.
2. Gong includes AI-powered keyword and topic detection, per Gong `conversation-intelligence:34-35`.
3. Gong includes sentiment and talk-ratio analysis, per Gong `conversation-intelligence:36`.
4. Gong ties conversation data to deal and pipeline tracking, per Gong `conversation-intelligence:37`.
5. Gong integrates with CRM and sales engagement platforms, per Gong `conversation-intelligence:38`.
6. Gong use cases include sales coaching, pipeline risk detection, messaging optimization, compliance, and quality assurance, per Gong `conversation-intelligence:40-44`.
7. Gong claims capture of customer interactions across calls, emails, meetings, and more, per Gong `conversation-intelligence:56-63`.
8. Gong emphasizes real-time risk, next-step, and coaching insights, per Gong `conversation-intelligence:94-98`.
9. Gong's call-recording guide names secure cloud storage, retention policies, searchable transcripts, keyword spotting, role-based access controls, audit trails, and CRM integration, per Gong `call-recording-software:34-40`.
10. Gong can automatically join scheduled calls or record inbound/outbound phone conversations, per Gong `call-recording-software:87-99`.
11. Gong indexes recordings and transcripts for filtering by rep, account, deal stage, or keyword, per Gong `call-recording-software:99-106`.
12. Gong supports native recording with supported providers and assistant recording where native recording is unavailable, per Gong help `understanding-call-recording:73-90`.
13. Gong consent behavior is configurable by company settings, per Gong help `understanding-call-recording:92-120`.
14. Gong documents long-call processing constraints, including a six-hour processing limit and recommended splitting, per Gong help `how-to-record-calls:130-141`.
15. Gong documents redaction for numeric data and PHI in both audio and transcripts, per Gong help `redact-sensitive-information:21-27`.

### §2.1 Gong parity observations against recordings

1. Oyatie recordings has automatic ingest contract intent across internal sources, per `ADR-RECORDINGS-0007-source-ingest-contract.md:31-56`.
2. Oyatie recordings has compliance and quality assurance surfaces through legal hold, retention, redaction, eDiscovery, and audit-chain dependencies, per `PRD.md:140-169`.
3. Oyatie recordings has searchable transcript intent, per `PRD.md:102-120` and `contracts/openapi/recordings.yaml:242-259`.
4. Oyatie recordings has role/policy controls through tenant headers, SPIFFE, and Cedar, per `contracts/openapi/recordings.yaml:11-13`.
5. Oyatie recordings has redaction rendering, per `PRD.md:116`, `ADR-RECORDINGS-0004-redaction-rendering-and-evidence-chain.md`, and `slos/redaction-render-p99.openslo.yaml:5-16`.
6. Oyatie recordings does not currently define deal, pipeline, account, rep, opportunity, competitor mention, objection, talk ratio, or sentiment entities in its core contracts.
7. Oyatie recordings does not currently define CRM integrations or sales-engagement writebacks.
8. Oyatie recordings does not currently define call coaching cards, curated best-call libraries, enablement snippets, or methodology adoption scoring.
9. Oyatie recordings does not currently define real-time revenue-risk flags.
10. Oyatie recordings does not currently define conversation intelligence as a revenue operating-system data plane.
11. The correct ownership decision may be to emit normalized conversation facts from recordings while another µservice owns revenue intelligence.
12. If recordings keeps Gong union coverage, OpenAPI must gain analytics surfaces or AsyncAPI must emit facts for analytics consumers.
13. If another µservice owns Gong-style features, recordings still must own capture fidelity, transcript provenance, consent evidence, redaction chain, retention policy, and exportability.
14. Gong parity is therefore a product-boundary decision, not just a feature backlog.
15. The current artifacts do not yet record that decision.

## §3 Counterpart 3 - Otter.ai capability surface

1. Otter centers on real-time transcription for conversations and meetings, per Otter help `Record a conversation` search/open evidence.
2. Otter AI Chat lets participants ask questions and collaborate with the transcription during or after a meeting, per Otter help `features:45-52`.
3. OtterPilot auto-joins meetings through Microsoft or Google calendar connections, per Otter blog `OtterPilot:87-96`.
4. OtterPilot writes live meeting notes in real time, per Otter blog `OtterPilot:96`.
5. OtterPilot can auto-share meeting notes, per Otter blog `OtterPilot:87-96`.
6. OtterPilot captures shared slides into meeting notes, per Otter blog `OtterPilot:98-100`.
7. OtterPilot sends automated summaries with links to key moments and slide captures, per Otter blog `OtterPilot:101-105`.
8. Otter positions itself as a collaborative, accessible meeting knowledge surface, per Otter blog `OtterPilot:112-113`.
9. Otter imports audio or video files under 5 GB, subject to plan limits, per Otter help `Import an audio or video file:33-34`.
10. Otter supports common audio formats: AAC, MP3, M4A, WAV, WMA, and OGG, per Otter help `Import an audio or video file:36-44`.
11. Otter supports common video formats: AVI, MOV, MPEG, MP4, WMV, MPG, MKV, M4P, and 3GP, per Otter help `Import an audio or video file:46-56`.
12. Otter Basic has 300 transcription minutes per month, 30 minutes per transcription, three file imports, and 25 recent conversations visible, per Otter help `Conversation, import, and app limits:34-55`.
13. Otter Notetaker can be removed by meeting participants typing a stop phrase, per Otter help `Remove Otter Notetaker` search evidence.
14. Otter supports calendar-mediated auto-join controls, per Otter help `Automatically add Otter Notetaker` search evidence.
15. Otter is a baseline for live collaborative transcript UX, AI chat over meeting content, slide capture, auto summaries, and simple import limits.

### §3.1 Otter parity observations against recordings

1. Oyatie recordings has transcription and diarization substrate intent, per `ADR-RECORDINGS-0001-transcription-and-diarization-substrate.md:68-94`.
2. Oyatie recordings has transcript render and search SLOs, per `slos/transcript-render-p99.openslo.yaml:5-15` and `slos/transcript-search-p99.openslo.yaml:5-17`.
3. Oyatie recordings has summary/action item workflow intent, per `workflows/WF-summary-action-items.md`.
4. Oyatie recordings has transcript translation workflow intent, per `workflows/WF-transcript-translation.md`.
5. Oyatie recordings has manual upload workflow intent, per `workflows/WF-manual-upload-ingest.md`.
6. Oyatie recordings has external webhook source workflow intent, per `workflows/WF-webhook-recording-source.md`.
7. Oyatie recordings has upload/source diversity, but OpenAPI currently does not enumerate Otter's explicit file-format list or 5 GB import limit equivalent.
8. Oyatie recordings does not currently define live collaborative transcript comments, highlights, or in-meeting chat with recording context.
9. Oyatie recordings does not currently define calendar auto-join behavior.
10. Oyatie recordings does not currently define a participant-level immediate removal command for recording assistants.
11. Oyatie recordings does not currently define slide-capture artifacts as first-class children of a recording.
12. Oyatie recordings does not currently define per-tenant import minute caps in tenant-class terms.
13. Oyatie recordings should add AI chat over a transcript only if privacy, legal hold, and redaction policy can constrain generated answers.
14. Oyatie recordings should model summary and action items as derived artifacts with provenance back to transcript spans and redaction state.
15. Otter parity is strongest for UX collaboration, while Oyatie's current edge is compliance chain and multi-source ingest.

## §4 Union-coverage matrix

| # | Capability | Zoom | Gong | Otter | Current recordings evidence | Status |
|---:|---|---|---|---|---|
| 1 | Cloud recording capture | Yes | Yes | Partial via notetaker/import | `PRD.md:37-48`; `ADR-0007:31-56` | Covered with source diversity |
| 2 | Meeting video/audio/chat capture | Yes | Yes for supported calls | Yes for meeting notes/transcripts | `PRD.md:102-120`; `contracts/openapi:147-401` | Mostly covered |
| 3 | Multiple recording layouts | Yes | Provider-dependent | Not primary | No explicit local layout schema | Gap |
| 4 | Live-session file count governance | Yes | Not public primary | Not primary | No explicit local file-fragment cap | Gap |
| 5 | Browser streaming playback | Yes | Yes | Yes | `contracts/openapi:182-209`; `slos/playback-start-p99` | Covered |
| 6 | Local download/export | Yes | Yes | Yes | `contracts/openapi:322-353` | Covered |
| 7 | Storage quota alerts | Yes | Enterprise admin | Plan-based limits | `cost-budget.md`; no quota alert contract | Gap |
| 8 | Transcript generation | Yes | Yes | Yes | `ADR-0001:68-94`; `contracts/openapi:210-220` | Covered |
| 9 | Speaker labels/diarization | Yes in AI docs | Yes | Yes | `ADR-0001:68-94` | Covered by design |
| 10 | Time-stamped transcript search | Yes | Yes | Yes | `contracts/openapi:242-259`; `slos/transcript-search-p99` | Covered |
| 11 | AI summaries | Yes | Yes | Yes | `workflows/WF-summary-action-items.md` | Covered by workflow, thin in API |
| 12 | Chapters/highlights | Yes | Moments/highlights | Key moments | No explicit chapter artifact in OpenAPI | Gap |
| 13 | Action items | Yes | Yes | Yes | `WF-summary-action-items.md` | Covered by workflow, thin in API |
| 14 | AI chat over recording | Not core in source read | Ask Anything | Otter AI Chat | No explicit local API | Gap |
| 15 | Live collaborative transcript | Limited | Not core | Yes | UI/workflow not explicit | Gap |
| 16 | Slide capture | Not core in Zoom source read | Not core | Yes | No explicit slide artifact | Gap |
| 17 | Calendar auto-join | Meeting native | Yes | Yes | No calendar auto-join contract | Gap |
| 18 | Assistant recording bot | Not needed for native Zoom | Yes | Yes | Source webhook possible, not assistant-specific | Gap |
| 19 | Assistant removal/stop command | Meeting controls | Consent/config | Yes | No participant stop command | Gap |
| 20 | Consent before recording | Yes through meeting UX | Yes | Yes | `PRD.md:140-154`; `ADR-0007:99-117` | Covered, needs command-level UX |
| 21 | Policy-driven recording suppression | Account settings | Yes | Auto-join settings | Cedar intent, no explicit suppression list | Partial |
| 22 | Legal hold | Enterprise export/admin | Not core public feature | Not primary | `ADR-0002:88-123`; `slos/legal-hold-*` | Strong |
| 23 | eDiscovery bundle | Enterprise adjacent | Compliance adjacent | Not primary | `contracts/openapi:322-339`; SLOs | Strong |
| 24 | Immutable evidence chain | Not public core | Audit trail | Not primary | `ADR-0002`; `ADR-0004`; audit-chain dep | Strong |
| 25 | Retention policy | Yes admin | Yes admin | Plan/archive | `ADR-0002:68-86`; SLO retention correctness | Strong |
| 26 | DSR/delete cascade | Privacy admin | Privacy admin | Account deletion | `WF-dsr-delete-cascade.md`; `WF-retention-expiry.md` | Covered |
| 27 | Redaction of audio/transcript | Limited | Yes numeric/PHI | Editing/export features | `ADR-0004`; `slos/redaction-render-p99` | Strong |
| 28 | CRM integration | Not core | Yes | Not core | No explicit local CRM surface | Gap |
| 29 | Sales coaching | Not core | Yes | Not core | No local coaching entity | Gap |
| 30 | Pipeline risk detection | Not core | Yes | Not core | No local revenue-risk entity | Gap |
| 31 | Talk ratio analysis | Not core | Yes | Not core | No local metric entity | Gap |
| 32 | Sentiment analysis | Not core | Yes | Not core | No local metric entity | Gap |
| 33 | Competitor mention detection | Not core | Yes | Possible search | No local revenue ontology | Gap |
| 34 | Call library for onboarding | Sharing/playback | Yes | Conversation sharing | Share links exist, no curated library | Partial |
| 35 | Role-based access controls | Yes | Yes | Workspace controls | SPIFFE/Cedar intent | Covered |
| 36 | Audit trails | Yes admin | Yes | Workspace history | audit-chain dependency | Covered |
| 37 | Import audio/video files | Local/cloud imports | Manual upload possible | Yes | `contracts/openapi:355-401`; `WF-manual-upload-ingest.md` | Covered |
| 38 | Published file-format matrix | Not central | Upload rules | Yes | No explicit local list | Gap |
| 39 | Import file-size cap | Storage dependent | Long-call guidance | 5 GB file import | No tenant-class cap | Gap |
| 40 | Long recording handling | Session constraints | Six-hour processing limit | Plan limits | No explicit split/retry policy | Gap |
| 41 | Processing lifecycle status | Yes | Yes | Yes | Events exist, lifecycle enum needs clarity | Partial |
| 42 | Webhook ingestion | API ecosystem | Integrations | Zapier/import | `WF-webhook-recording-source.md` | Covered |
| 43 | Translation | Zoom AI adjacent | Not core | language support | `WF-transcript-translation.md` | Covered |
| 44 | Compliance packs | Enterprise compliance | Enterprise compliance | Workspace controls | `PRD.md:140-169`; `ADR-0002` | Strong |
| 45 | Multi-region policy | Enterprise | Enterprise | SaaS managed | `multi-region.md`; `PRD.md:173-176` | Covered by docs |
| 46 | Tenant isolation | Enterprise admin | Enterprise admin | Workspace | `contracts/openapi:11-13`; manifest invariants | Covered |
| 47 | Cross-service workflow events | Not product-facing | Integrations | Integrations | `AsyncAPI:18-214` | Strong |
| 48 | Open API export | Zoom APIs | APIs | Exports | `OpenAPI:322-401` | Covered |
| 49 | Watermarked playback | Enterprise adjacent | Not primary | Not primary | `ADR-0003` | Covered by ADR |
| 50 | Abuse controls for share links | Admin controls | Sharing controls | Sharing controls | `runbooks/expiring-share-link-abuse.md` | Covered |
| 51 | Transcript privacy controls | Admin settings | Redaction/privacy | Workspace sharing | `ADR-0005` | Covered by ADR |
| 52 | Capacity model | Public limits | Private SaaS | Public plan limits | `capacity-model.md:17-120` | Covered locally |
| 53 | OCI Always Free profile overlay | Not applicable | Not applicable | Basic free plan analogue | Missing local module | Gap |
| 54 | Tenant-class quota semantics | Plan-based | Seat-based | Plan-based | Missing tenant_class semantics | Gap |
| 55 | OS support matrix | SaaS hidden | SaaS hidden | SaaS hidden | Missing `supported-oses.json` | Gap |
| 56 | OpenTofu deployment modules | SaaS hidden | SaaS hidden | SaaS hidden | Missing canonical modules | Gap |
| 57 | On-prem deployability | Not product default | Enterprise managed | Not product default | no context module/N/A manifest | Gap |
| 58 | Colo deployability | Not product default | Enterprise managed | Not product default | no context module/N/A manifest | Gap |
| 59 | Oyatie-as-cloud-provider deployability | Not comparable | Not comparable | Not comparable | no `iac/oyatie-iaas/` | Gap |
| 60 | Uniform quality bar across tenant classes | Plan-dependent | Seat/platform-dependent | Plan-dependent | not represented | Gap |

## §5 Family summary

1. Capture family: Oyatie recordings is strong on multi-source ingest but needs explicit layout and live-session artifact governance to match Zoom.
2. Processing family: Oyatie recordings has strong transcription/diarization ADR intent and workflow depth, but AI chapter/highlight/chat artifacts need contract-level expression.
3. Search family: Oyatie recordings is strong because transcript search is in OpenAPI and has a p99 SLO.
4. Playback family: Oyatie recordings is strong on playback API and p99 objective, but needs storage/quota alert semantics.
5. Sharing family: Oyatie recordings has share-link APIs and abuse runbooks, but not Otter-style collaborative transcript controls.
6. Compliance family: Oyatie recordings is stronger than the public meeting-assistant baseline because legal hold, eDiscovery, DSR, retention, redaction, and evidence-chain concepts are explicit.
7. Revenue-intelligence family: Oyatie recordings is weak against Gong unless it either owns revenue analytics or emits conversation facts to another owner.
8. Consent family: Oyatie recordings has policy and consent doctrine, but lacks participant-level assistant-removal controls and provider-specific consent flows.
9. Import family: Oyatie recordings supports manual ingest by design, but file-format and file-size rules are not explicit.
10. Deployment family: Oyatie recordings is behind canonical Oyatie requirements because the six-context OpenTofu/OCI/OS control surfaces are absent.
11. Tenant-class family: Oyatie recordings is behind current doctrine because no `tenant_class` semantics are encoded.
12. Documentation family: Oyatie recordings has many docs, but the older counterpart map and retired feature-tier artifacts make it internally inconsistent.

## §6 Headline gap analysis

1. Gap A: Gong union coverage is the largest product-surface decision.
2. Evidence A: Gong's public surface includes CRM integration, pipeline risk, coaching, sentiment, talk ratio, and revenue workflow analytics.
3. Evidence A local: recordings contracts focus on media, transcript, legal hold, search, share, and export.
4. Decision A: either recordings owns conversation intelligence facts or emits them to the intelligence/analytics surface.
5. Gap B: Zoom layout and artifact-fragment semantics are absent.
6. Evidence B: Zoom publicly names active speaker, gallery view, shared screen, and a live-session file generation limit.
7. Evidence B local: recordings has source kinds but no layout variant or file-fragment inventory schema.
8. Decision B: add artifact-variant and layout schemas for recording children.
9. Gap C: Otter live collaboration is not represented.
10. Evidence C: Otter supports AI Chat during/after meetings, live notes, slide capture, and summary sharing.
11. Evidence C local: recordings has summary and translation workflows, but not live collaborative transcript comments/highlights/chat.
12. Decision C: define transcript collaboration as either recordings-owned UI/API or a docs/notes integration.
13. Gap D: tenant-class controls are absent.
14. Evidence D: search found no `tenant_class`, `demo_trial`, or `revenue_share`.
15. Decision D: encode tenant-class caps for capture, storage, transcription, export, legal hold, retention, share bandwidth, and AI-derived artifacts.
16. Gap E: deployability controls are absent.
17. Evidence E: missing canonical OpenTofu modules and `supported-oses.json`.
18. Decision E: no "all six deployable" claim should stand until those modules or N/A manifests exist.
19. Gap F: retired feature-tier docs remain.
20. Evidence F: 148 matching retired-term lines under this service.
21. Decision F: Wave 15J should retire or replace the entire `capability-tiers/` directory, not patch individual sentences.
22. Gap G: storage quota and account alert behavior is under-modeled.
23. Evidence G: Zoom has explicit storage capacity and alert behavior; recordings has a capacity model but no quota event contract.
24. Decision G: add quota thresholds, warning events, over-limit recording continuation rules, and tenant-class enforcement behavior.
25. Gap H: long-call handling is under-modeled.
26. Evidence H: Gong documents a six-hour processing ceiling; Otter has plan duration limits; Zoom has live-session file limits.
27. Decision H: add long-recording split, resumable processing, and export chunking rules.
28. Gap I: redaction is strong but must be bound to AI summaries/chat.
29. Evidence I: Gong notes redaction can affect AI insights; recordings has redaction renderer and summary workflows.
30. Decision I: every AI artifact must carry redaction-source version and legal-hold state.

## §7 Additive surface for industry-leader coverage

1. Add `RecordingArtifactVariant` with fields for layout, media track, transcript, chat transcript, slide image, summary, chapter, highlight, action item, translation, redaction render, and export bundle.
2. Add `RecordingProcessingLifecycle` with states for captured, uploaded, normalized, transcribing, diarizing, indexing, redaction_pending, playable, shared, held, export_packaging, expired, deleted, and error.
3. Add `RecordingQuotaEvent` with warning, hard cap, over-limit continuation, and enforcement actions.
4. Add `tenant_class` policy binding for `demo_trial`, `paid`, and `revenue_share`.
5. Add `demo_trial` caps for retained media hours, transcript minutes, export count, playback bandwidth, share links, and compliance-pack exclusion.
6. Add `paid` scaling semantics for contractual SLO, compliance packs, BYOK, and usage-based storage/transcription/export billing.
7. Add `revenue_share` scaling semantics for at-cost substrate and revenue reporting hooks without lowering product quality.
8. Add `ConversationFactEmitted` event for normalized talk-time, topic, keyword, sentiment, objection, competitor, action, risk, and coaching facts if Gong coverage is delegated.
9. Add `RevenueConversationAnalytics` API only if recordings is chosen to own Gong-style analytics directly.
10. Add `TranscriptCollaboration` surfaces for comments, highlights, mentions, action items, and live Q&A if recordings owns Otter-style collaboration.
11. Add `RecordingAssistantControl` for auto-join, manual invite, participant removal, stop command, recording suppression, and consent failure.
12. Add `CalendarRecordingIntent` if recordings owns calendar-based auto-join policy.
13. Add `ConsentEvidence` record with source service, provider, participant notification mode, explicit consent, implicit consent, denial, and legal basis.
14. Add `LayoutCapturePolicy` for provider-specific layout selection and artifact count limits.
15. Add `LongRecordingPolicy` for split recommendations, automatic chunking, continuation, and export packaging.
16. Add `ImportFormatPolicy` listing accepted audio/video types and size caps by tenant class.
17. Add `RedactionAffectsDerivedArtifact` event when redaction invalidates summary, search index, translation, or AI chat embedding.
18. Add `LegalHoldBlocksDeletion` event with immutable evidence chain and tenant/policy references.
19. Add `PlaybackWatermarkPolicy` for legal, compliance, and external share contexts.
20. Add `ShareLinkAbuseSignal` for suspicious download, geography, token reuse, and link spraying.
21. Add `ExportBundleManifest` with media, transcript, redaction, hash chain, audit events, legal-hold state, and retention policy version.
22. Add six-context OpenTofu modules or explicit N/A manifests before shipping deployment claims.
23. Add `supported-oses.json` with the canonical 13+2+6 OS/arch matrix.
24. Add an OCI Always Free profile module for demo-trial infrastructure budgets.
25. Add root README or machine-readable replacement that points to PRD, architecture, contracts, workflows, SLOs, handoffs, deployment contexts, and tenant-class policies.
26. Add `cross-microservice-handoffs.md` or machine-readable handoff registry for Meet, Messenger, Live, docs, identity, tenancy, policy-engine, audit-chain, observability, cell, and cloud-iac.
27. Retire the existing feature-tier docs as Wave 15J artifacts instead of evolving them.
28. Replace existing benchmark language with a single target set plus deployment-context and tenant-class overlays.
29. Refresh local competitor matrix so the top-3 audit set is Zoom Cloud Recording, Gong.io, and Otter.ai.
30. Treat compliance-grade evidence chain as Oyatie's differentiator, not as an excuse to skip meeting-assistant and revenue-intelligence union coverage.

## §8 Coverage decision summary

1. Zoom parity is reachable with focused additions to layout, artifact variants, storage warnings, processing lifecycle, and AI recording recall artifacts.
2. Gong parity requires an ownership decision because it reaches beyond recordings into CRM and revenue operations.
3. Otter parity requires live collaboration, calendar auto-join, Notetaker controls, slide capture, and AI chat over transcripts.
4. Oyatie's strongest existing differentiator is compliance-grade lifecycle control: legal hold, eDiscovery, redaction, retention, DSR, audit-chain, and evidence manifests.
5. Oyatie's weakest current differentiator is deployment-control evidence because canonical OpenTofu, OCI Always Free, supported OS, and tenant-class artifacts are missing.
6. The next artifact refresh should avoid prose-only claims and add machine-readable policy/control surfaces wherever canonical direction requires them.

## §9 Counterpart-driven implementation backlog

1. Backlog Z-01: add recording layout variants for active speaker, gallery, shared screen, and audio-only children.
2. Backlog Z-02: add chat transcript as a first-class artifact child rather than treating it as transcript text.
3. Backlog Z-03: add per-session artifact count governance so live-session fanout can be validated against a 150-file comparison bar.
4. Backlog Z-04: add host/co-host recording-start provenance fields.
5. Backlog Z-05: add recording pause/resume provenance fields.
6. Backlog Z-06: add recording stop reason fields for host stop, meeting end, participant leave, quota exhaustion, and consent failure.
7. Backlog Z-07: add storage-quota warning event at 80 percent budget consumption.
8. Backlog Z-08: add over-quota in-progress continuation policy.
9. Backlog Z-09: add over-quota new-recording denial policy.
10. Backlog Z-10: add account/admin notification records for quota events.
11. Backlog Z-11: add browser-streaming capability tests for warm playback.
12. Backlog Z-12: add browser-streaming capability tests for cold playback.
13. Backlog Z-13: add structured recording summary artifact.
14. Backlog Z-14: add chapter artifact with transcript-span provenance.
15. Backlog Z-15: add highlight artifact with transcript-span provenance.
16. Backlog Z-16: add action-item artifact with assignee, due date, confidence, and source span.
17. Backlog Z-17: add speaker-insight artifact with privacy controls.
18. Backlog Z-18: add original-audio artifact for in-person voice recordings.
19. Backlog Z-19: add recording-law compliance prompt evidence for in-person voice recordings.
20. Backlog Z-20: add export behavior for every derived AI artifact.
21. Backlog G-01: add normalized customer-interaction fact emission.
22. Backlog G-02: add topic detection facts if recordings owns raw fact extraction.
23. Backlog G-03: add keyword detection facts if recordings owns raw fact extraction.
24. Backlog G-04: add sentiment facts only after privacy/legal review.
25. Backlog G-05: add talk-ratio facts only after participant consent policy is explicit.
26. Backlog G-06: add competitor-mention facts if revenue analytics is not delegated elsewhere.
27. Backlog G-07: add objection-mention facts if revenue analytics is not delegated elsewhere.
28. Backlog G-08: add buying-signal facts if revenue analytics is not delegated elsewhere.
29. Backlog G-09: add pipeline-risk handoff event if another µservice owns deal scoring.
30. Backlog G-10: add CRM-object reference fields only through a governed integration boundary.
31. Backlog G-11: add rep/account/deal-stage filters only if recordings owns revenue search.
32. Backlog G-12: add coaching-library metadata if recordings owns call library curation.
33. Backlog G-13: add enablement clip packaging if recordings owns coaching workflows.
34. Backlog G-14: add numeric redaction verification for transcript and audio.
35. Backlog G-15: add PHI redaction verification for transcript and audio.
36. Backlog G-16: add "redaction cannot be recovered" evidence semantics for irreversible redactions.
37. Backlog G-17: add native-provider recording mode field.
38. Backlog G-18: add assistant-participant recording mode field.
39. Backlog G-19: add consent profile reference.
40. Backlog G-20: add consent-denied blocked-recording event.
41. Backlog O-01: add live transcript collaboration comments.
42. Backlog O-02: add live transcript highlights.
43. Backlog O-03: add live transcript shared notes.
44. Backlog O-04: add transcript Q&A API only after redaction/legal-hold policy is binding.
45. Backlog O-05: add meeting calendar auto-join intent.
46. Backlog O-06: add auto-join suppression policy.
47. Backlog O-07: add participant stop command semantics.
48. Backlog O-08: add assistant removal event.
49. Backlog O-09: add slide-capture artifact with image hash and source timestamp.
50. Backlog O-10: add summary email/share event if recordings owns post-meeting distribution.
51. Backlog O-11: add import audio-format matrix.
52. Backlog O-12: add import video-format matrix.
53. Backlog O-13: add import file-size cap by tenant class.
54. Backlog O-14: add import duration cap by tenant class.
55. Backlog O-15: add monthly transcription-minute cap by tenant class.
56. Backlog O-16: add conversation-history visibility cap by tenant class.
57. Backlog O-17: add auto-share controls for team/workspace channels.
58. Backlog O-18: add meeting-summary recipient controls.
59. Backlog O-19: add key-moment link generation.
60. Backlog O-20: add slide-to-transcript search join.
61. Backlog C-01: add `tenant_class` field to service-local policy artifacts.
62. Backlog C-02: add `demo_trial` quota pack.
63. Backlog C-03: add `paid` quota and billing pack.
64. Backlog C-04: add `revenue_share` cost/accounting pack.
65. Backlog C-05: add uniform quality-bar statement that applies across tenant classes.
66. Backlog C-06: add compliance-pack exclusion semantics for demo-trial tenants.
67. Backlog C-07: add BYOK exclusion semantics for demo-trial tenants.
68. Backlog C-08: add contractual SLO enablement semantics for paid tenants.
69. Backlog C-09: add at-cost substrate accounting semantics for revenue-share tenants.
70. Backlog C-10: add usage-exhaustion response codes.
71. Backlog D-01: add `iac/oyatie-public-cloud/` OpenTofu module.
72. Backlog D-02: add `iac/guest-on-aws/` OpenTofu module.
73. Backlog D-03: add `iac/oci-guest/` OpenTofu module.
74. Backlog D-04: add `iac/oci-guest/always-free/` OpenTofu module.
75. Backlog D-05: add `iac/on-prem/` OpenTofu or N/A artifact.
76. Backlog D-06: add `iac/colo/` OpenTofu or N/A artifact.
77. Backlog D-07: add `iac/oyatie-iaas/` OpenTofu module.
78. Backlog D-08: migrate `iac/terraform/grafana-rbac.tf` into an approved OpenTofu/GitOps path.
79. Backlog D-09: replace Terraform command references with OpenTofu command references.
80. Backlog D-10: add state-backend posture for each deployment context.
81. Backlog S-01: add `supported-oses.json`.
82. Backlog S-02: add Talos runtime support evidence.
83. Backlog S-03: add Ubuntu LTS runtime support evidence.
84. Backlog S-04: add Debian runtime support evidence.
85. Backlog S-05: add RHEL runtime support evidence.
86. Backlog S-06: add Oracle Linux runtime support evidence.
87. Backlog S-07: add SUSE Linux Enterprise support evidence.
88. Backlog S-08: add Rocky/Alma/CentOS Stream support evidence.
89. Backlog S-09: add Amazon Linux support evidence.
90. Backlog S-10: add Flatcar/Photon support evidence.
91. Backlog S-11: add macOS Apple Silicon development/build evidence.
92. Backlog S-12: add s390x and ppc64le support or explicit N/A evidence.
93. Backlog X-01: add root README or machine-readable equivalent.
94. Backlog X-02: add cross-microservice handoff registry.
95. Backlog X-03: add Meet handoff details.
96. Backlog X-04: add Messenger handoff details.
97. Backlog X-05: add Live handoff details.
98. Backlog X-06: add docs/notes handoff details for transcript collaboration.
99. Backlog X-07: add policy-engine handoff details.
100. Backlog X-08: add audit-chain handoff details.
101. Backlog X-09: add identity/tenancy authorization handoff details.
102. Backlog X-10: add observability handoff details.
103. Backlog R-01: retire the feature-tier directory under Wave 15J.
104. Backlog R-02: retire old feature-segmented benchmark rows.
105. Backlog R-03: retire old pricing language that implies product-quality stratification.
106. Backlog R-04: migrate remaining useful facts into tenant-class and deployment-context docs.
107. Backlog R-05: keep vendor comparison facts but reframe them around union coverage.
108. Backlog R-06: preserve compliance-grade requirements when retiring old docs.
109. Backlog R-07: preserve Rust reference implementation examples when retiring old tier labels.
110. Backlog R-08: preserve migration playbook commands while removing stale segmentation terms.
111. Backlog R-09: preserve benchmark workload fixtures while changing methodology.
112. Backlog R-10: preserve FAQ answers that remain accurate after doctrine updates.
113. Backlog M-01: add measured benchmark harness for ingestion.
114. Backlog M-02: add measured benchmark harness for transcript generation.
115. Backlog M-03: add measured benchmark harness for playback.
116. Backlog M-04: add measured benchmark harness for search.
117. Backlog M-05: add measured benchmark harness for redaction.
118. Backlog M-06: add measured benchmark harness for eDiscovery.
119. Backlog M-07: add measured benchmark harness for legal-hold correctness.
120. Backlog M-08: add measured benchmark harness for tenant-class quota enforcement.
