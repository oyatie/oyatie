# Oyatie — Product PRD: Workspace / Productivity Suite

> **Status:** draft v0.1 — 2026-05-09. New axis added per user directive.
> **Owning team:** `axis-workspace` (new team — needs charter)
> **Owning axis:** `workspace` (axis 2)
> **Catalog reference:** `crates/oya-workspace-*` (planned)

## 1. North star

Oyatie Workspace is the **canonical end-user productivity suite** for every Oyatie tenant — Mail, Calendar, Docs, Sheets, Slides, Drive (Cloud Storage), Meet (video / audio / screen share), Chat, Forms, Sites, Tasks, Notes, Translate, Recordings. Reference benchmarks: Google Workspace, Naver Works, Microsoft 365 Business, AWS Productivity (WorkMail / WorkDocs / Chime), Kakao Work. Why it can only exist as part of Oyatie's ecosystem: every productivity surface plugs *natively* into the SaaS platform's tenancy + identity + workflows, the Foundry agent runtime (compose / summarize / triage / schedule), the Search engine (every Doc + Mail is searchable per consent tier), the Cloud provider (Drive backed by KMS-shred object storage; Meet on cloud SFU), the Vertical industry cloud (per-vertical compliance: HIPAA Mail, attorney-client privileged Docs, KYC-bound Calendar). The cohesion eliminates the integration tax that every multi-vendor productivity stack pays.

## 2. Target users

| Persona | What they get | What they pay for |
|---|---|---|
| Tenant employee | Daily-driver Mail + Calendar + Docs + Drive on Oyatie tenancy with Foundry-agent assistance built in | Per-seat workspace subscription |
| Tenant admin | Org-wide Mail / Doc / Drive policy, retention, DLP, e-discovery, legal hold, DSR cascade | Per-seat |
| Tenant builder | Customize Mail rules, Doc templates, Calendar workflows; build Workspace plugins | Per-seat (builder tier) |
| External collaborator | Per-document share access without an Oyatie account; consent-bound | Free; tenant pays for hosting |
| Migrator | Bulk import from Google Workspace / Microsoft 365 / Naver Works / Kakao Work / Notion / Slack | Migration assistance billed once |
| Auditor | Per-Mail / per-Doc audit trail; e-discovery export; retention proof | Bundled with tenant |
| Foundry agent | Mail-triage capabilities, Doc-draft capabilities, Calendar-scheduling capabilities, Meet-summarize capabilities | (internal — capabilities consumed by SaaS workflows) |

## 3. In-scope / out-of-scope

### 3.1 In-scope by wave

| Wave | Capabilities | Surfaces exposed |
|---|---|---|
| W-Workspace-Preview | Mail (SMTP / IMAP / JMAP server + per-tenant domain or `<tenant>.oyatiemail.com`), Calendar (CalDAV + scheduling primitives), Drive (object storage + folder/permission semantics; web + sync clients), Docs (CRDT-based real-time collaborative editor; export to PDF / DOCX / HWPX / Markdown), Foundry-agent capabilities (compose, summarize, triage, schedule) | REST + GraphQL + IMAP/SMTP/JMAP/CalDAV/WebDAV |
| W-Workspace-Stable | Sheets (collaborative spreadsheet + formula engine + Foundry what-if), Slides (collaborative presentation + templates + export PPTX/PDF/HTML), Meet (WebRTC SFU + recording + Foundry transcription + AI summaries), Chat (DM + group + channels + threading + bots) | + WebRTC + WSS chat + Slides export |
| W-Workspace-GA | Forms (Object-Graph-routed data collection), Sites (lightweight intranet / wiki), Tasks + Notes + Keep, Translate (50+ languages via Foundry adapter), Recordings archive, Org Address Book (cross-tenant directory under consent), DLP, retention + legal hold, e-discovery export, DSR cascade integration | + Forms + Sites publishing + Address-Book API |
| Cross-region rollout | Per-region pack: KR mail integration (KR mail-security controls / 메일 보안 / 본인확인); JP integration (マイナンバー-aware Mail); EU eIDAS-signed Docs; US-FedRAMP for Workspace (gov tier); etc. | per-pack per region |

### 3.2 Out-of-scope (anti-scope)

- **Public consumer mail / calendar / docs** (Gmail-class B2C). Workspace is B2B/B2G only; consumer launch reconsidered only on substantial evidence.
- **Free tier with ads.** Workspace is paid; Connect Personal stays separate (per `LEDG-021` decision).
- **Game / streaming / consumer-social products bundled in.** Out-of-scope.
- **Generic file-sync without enterprise controls** (Dropbox-class consumer sync).

## 4. Architecture overview

### 4.1 Bounded context

`crates/oya-workspace-*` per surface (mail, calendar, docs, sheets, slides, drive, meet, chat, forms, sites, tasks, notes, translate, recordings, address-book). Each surface is its own bounded context with clean-arch layers (kernel / domain / app / adapter / api / worker / runtime).

### 4.2 Layered structure (per surface, 4 layers per ADR-0015)

For Mail, illustrative:
- `oya-workspace-mail-kernel` — `Mailbox`, `Message`, `Thread`, `Folder`, `Filter`, `MimeBody` entities with `data_class: PII_IDENTIFYING` default and per-folder override.
- `oya-workspace-mail-domain` — use cases: send, receive, classify (Foundry-driven), search-index, retain, legal-hold, DSR-cascade.
- `oya-workspace-mail-app` — orchestration: SMTP receive → spam scan → DLP → classify → Foundry-triage capability invocation → store.
- `oya-workspace-mail-adapter-{smtp,imap,jmap,storage,kms,oya-search,oya-foundry,oya-eventing}-*`.
- `oya-workspace-mail-{api,worker,runtime}-*`.

Repeat the pattern per surface.

Initial Drive implementation crates:
- `oya-workspace-drive-kernel` — typed Drive object, folder, ACL, path, KMS-shred, and data-class records.
- `oya-workspace-drive-api` — REST boundary for `workspace.drive.put` / `workspace.drive.get`, backed by `contracts/openapi/workspace/workspace-drive-v1.yaml`.

Initial Meet implementation crates:
- `oya-workspace-meet-kernel` — typed Meet session, participant, recording, transcript, KMS-shred, and data-class records.
- `oya-workspace-meet-api` — REST boundary for `workspace.meet.session.start`, backed by `contracts/openapi/workspace/workspace-meet-v1.yaml`.

Initial Chat implementation crates:
- `oya-workspace-chat-kernel` — typed Chat channel, participant, message, attachment, thread, bot, and data-class records.
- `oya-workspace-chat-api` — REST boundary for `workspace.chat.message.send`, backed by `contracts/openapi/workspace/workspace-chat-v1.yaml`.

Initial Forms implementation crates:
- `oya-workspace-forms-kernel` — typed form schema, field, answer, submission, Object Graph route, and data-class records.
- `oya-workspace-forms-api` — REST boundary for `workspace.forms.submission.ingest`, backed by `contracts/openapi/workspace/workspace-forms-v1.yaml`.

### 4.3 External-facing surfaces

| Surface | Contract | Plane | SLO |
|---|---|---|---|
| Mail SMTP / IMAP / JMAP | RFC 5321 / 3501 / RFC 8620 | data | 99.9% deliverability |
| Calendar CalDAV + REST | RFC 4791 + Oyatie REST | data | 99.95% |
| Drive WebDAV + REST + sync | RFC 4918 + WebSocket sync + `contracts/openapi/workspace/workspace-drive-v1.yaml` | data | 99.95% |
| Docs CRDT / WebSocket | Y.js / Automerge wire format | data | 99.9% |
| Sheets / Slides | similar to Docs | data | 99.9% |
| Meet WebRTC SFU | WHIP/WHEP + Foundry transcription + `contracts/openapi/workspace/workspace-meet-v1.yaml` | data | 99.9% |
| Chat WSS + REST | Oyatie chat schema + Matrix-compatible (optional) + `contracts/openapi/workspace/workspace-chat-v1.yaml` | data | 99.9% |
| Forms REST | Oyatie Forms schema → Object Graph + `contracts/openapi/workspace/workspace-forms-v1.yaml` | data + control | 99.95% |
| Sites HTTP | static + dynamic blocks | data | 99.99% (read) |
| Address Book CardDAV | RFC 6352 | data | 99.95% |

### 4.4 Internal seams (depended on by other axes)

| Seam | Trait / interface | Consumers |
|---|---|---|
| `MailReader` (pull mail content for Foundry triage) | `oya-workspace-mail-kernel::MailReader` | Foundry agent capabilities |
| `DocReader` (pull doc content for Search indexing per consent) | `oya-workspace-docs-kernel::DocReader` | Search axis |
| `CalendarSlotPicker` (find open slots for scheduling) | `oya-workspace-calendar-kernel::SlotPicker` | Foundry; Vertical-corporate scheduling |
| `DrivePathProvider` (URL → object) | `oya-workspace-drive-kernel::DrivePathProvider` + `oya-workspace-drive-api::{put_workspace_drive_object_from_api,get_workspace_drive_object_from_api}` | Search; Foundry RAG; audit-chain readers |
| `MeetTranscriptStream` (live + batch transcription) | `oya-workspace-meet-kernel::TranscriptStream` + `oya-workspace-meet-api::start_workspace_meet_session_from_api` | Foundry; Search; audit-chain readers |
| `ChatMessageReader` (per-channel message stream for triage/search) | `oya-workspace-chat-kernel::ChatMessageReader` + `oya-workspace-chat-api::send_workspace_chat_message_from_api` | Foundry; Search; audit-chain readers |
| `FormSubmissionReader` (per-form submission stream for workflows / Object Graph) | `oya-workspace-forms-kernel::FormSubmissionReader` + `oya-workspace-forms-api::ingest_workspace_forms_submission_from_api` | Object Graph; Foundry; audit-chain readers |

### 4.5 Cross-axis contracts consumed

| Contract | Owner axis | Where it lives | Change-review |
|---|---|---|---|
| Tenant kernel | SaaS | `oya-platform-tenant-kernel` | All-axis review |
| Identity / RBAC / Cedar | SaaS | `oya-platform-identity-kernel` | Cross-axis + security |
| Object Graph property tiers | SaaS | `oya-platform-object-graph-kernel` | OG review |
| Audit chain | Foundry (audit) | `oya-platform-audit-chain-kernel` | Audit + downstream |
| Foundry capability invocation | Foundry | `oya-foundry-api` | Foundry review |
| Cloud KMS-shred (per-record encryption for Mail / Docs / Drive) | Cloud | `oya-cloud-kms-kernel` | Cloud + security |
| Cloud object storage (Drive backing) | Cloud | `oya-cloud-storage-object-kernel` | Cloud review |
| Search index lifecycle (per-tenant private + cross-tenant per-consent) | Search | `oya-search-index-kernel` | Search + downstream |
| Data Use Boundary (Workspace data classes feed Search + Analytics under DUB) | Privacy | `decisions/ADR-0008-data-use-boundary.md` | Privacy review |
| Regional pack seams (per-region mail rules, holiday calendar, language pack, e-invoicing for billing) | Regional packs | `oya-platform-regional-pack-kernel` | per-pack review |

## 5. Data structures

### 5.1 Kernel entities (illustrative; full set ~50+ entities across Workspace surfaces)

```rust
// oya-workspace-mail-kernel
pub struct Mailbox {
    pub id: MailboxId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub data_class: DataClass,        // PII_IDENTIFYING default; per-folder override
    pub subject_attributes: RecordAttributes,
    pub address: EmailAddress,        // user@<tenant>.oyatiemail.com or per-tenant domain
    pub quota_bytes: u64,
    pub retention_policy: RetentionPolicyId,
    pub legal_hold: Option<LegalHoldId>,
    pub created_at: DateTime<Utc>,
    pub schema_version: u32,
}

pub struct Message {
    pub id: MessageId,
    pub mailbox_id: MailboxId,
    pub thread_id: ThreadId,
    pub data_class: DataClass,        // inherits from mailbox; per-message override
    pub headers: MimeHeaders,
    pub body: MimeBody,               // stored encrypted (KMS-shred per ADR-0043)
    pub attachments: Vec<AttachmentRef>,
    pub classifications: Vec<ClassificationLabel>,  // spam / phishing / DLP / vertical-class
    pub received_at: DateTime<Utc>,
    pub indexed_at: Option<DateTime<Utc>>,  // when consent-tier indexing happened
    pub schema_version: u32,
}

pub struct CalendarEvent {
    pub id: CalendarEventId,
    pub calendar_id: CalendarId,
    pub tenant_id: TenantId,
    pub data_class: DataClass,
    pub subject_attributes: RecordAttributes,
    pub title: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub recurrence: Option<RecurrenceRule>,
    pub attendees: Vec<Attendee>,
    pub location: Option<Location>,
    pub videoconference: Option<MeetSessionId>,
    pub schema_version: u32,
}

pub struct Doc {
    pub id: DocId,
    pub drive_path: DrivePath,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub data_class: DataClass,
    pub crdt_state: CrdtSnapshotRef,  // Y.js / Automerge state
    pub content_type: ContentType,    // doc / sheet / slide
    pub permissions: PermissionSet,
    pub version_history: Vec<VersionRef>,
    pub indexed_at: Option<DateTime<Utc>>,
    pub schema_version: u32,
}

pub struct DriveObject {
    pub id: DriveObjectId,
    pub path: DrivePath,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub data_class: DataClass,
    pub object_storage_key: CloudObjectKey,  // backed by oya-cloud-storage-object
    pub size_bytes: u64,
    pub mime_type: String,
    pub kms_shred_key_id: KmsKeyId,           // per-record DEK
    pub permissions: PermissionSet,
    pub schema_version: u32,
}

pub struct MeetSession {
    pub id: MeetSessionId,
    pub tenant_id: TenantId,
    pub region: RegionCode,
    pub started_at: DateTime<Utc>,
    pub participants: Vec<ParticipantRef>,
    pub recording: Option<RecordingRef>,
    pub transcript_session_id: Option<TranscriptSessionId>,  // Foundry-driven
    pub summary_id: Option<SummaryRef>,
    pub schema_version: u32,
}

// ... plus Sheet, Slide, ChatMessage, ChatChannel, Form, FormSubmission, Site, Page, Task, Note, AddressBookEntry, TranslationJob, RecordingArtifact
```

### 5.2 Aggregate boundaries

- `Mailbox + Message + Thread + Folder + Filter` is one aggregate per mailbox.
- `Calendar + CalendarEvent + Attendee` per calendar.
- `Doc + CrdtSnapshot + VersionHistory + Permission` per doc.
- `DriveFolder + DriveObject + Permission` per folder.
- `MeetSession + Participants + Recording + Transcript` per session.

### 5.3 Persistence layout

| Aggregate | Store | Sharding key | Partition | Replication | Retention |
|---|---|---|---|---|---|
| Mailbox / Messages | Postgres + object storage for bodies | `tenant_id + mailbox_id` | per-tenant shard | 3-replica + cross-AZ | per retention_policy + legal_hold override |
| Calendar | Postgres | `tenant_id + calendar_id` | per-tenant | 3-replica | indefinite + DSR cascade |
| Docs CRDT | Postgres + object-storage snapshots + Redis hot state | `tenant_id + doc_id` | per-tenant | 3-replica + Redis HA | indefinite + DSR cascade |
| Drive objects | Cloud object store (oya-cloud-storage-object) | `tenant_id + path` | per-tenant cell | per region replication policy | indefinite + KMS-shred on delete |
| Meet recordings | Cloud cold tier (oya-cloud-storage-archive) | `tenant_id + session` | per-tenant | per region | per retention_policy |

### 5.4 Event schemas (events emitted)

| Event | Topic | Schema | Consumers | Retention | Idempotency key |
|---|---|---|---|---|---|
| `oya.workspace.mail.received` | `oya.workspace.mail.events` | proto | Foundry triage; Search indexer (per consent); audit chain | 90d | `(message_id)` |
| `oya.workspace.mail.classified` | same | proto | DLP; legal hold; audit chain | 90d | `(message_id, classification_id)` |
| `oya.workspace.calendar.event_changed` | `oya.workspace.calendar.events` | proto | Notifier; Foundry scheduling; Search per consent | 90d | `(event_id, version)` |
| `oya.workspace.doc.published` | `oya.workspace.docs.events` | proto | Search indexer; audit chain | 90d | `(doc_id, version)` |
| `oya.workspace.drive.object_uploaded` | `oya.workspace.drive.events` | proto | Virus scan; DLP; Search; audit chain | 90d | `(object_id)` |
| `oya.workspace.meet.session_ended` | `oya.workspace.meet.events` | proto | Recording archiver; Foundry transcription/summary; audit chain | 90d | `(session_id)` |
| `oya.workspace.dsr.cascade_started` | `oya.workspace.dsr.events` | proto | Mail purge; Calendar purge; Doc purge; Drive shred; Meet recording purge | per-DSR | `(dsr_id)` |

### 5.5 Search-index touchpoints (per Data Use Boundary)

| Entity field | Index | Class allowed | DSR cascade |
|---|---|---|---|
| `Message.body` | per-tenant private index | `PII_IDENTIFYING` (default; opt-out per mailbox) | yes |
| `Message.headers` | per-tenant private index | `PII_QUASI_IDENTIFIER` | yes |
| `Doc.crdt_state.text` | per-tenant private index | `PII_IDENTIFYING` (default) or higher per doc | yes |
| `CalendarEvent.title` | per-tenant private | `PII_QUASI` | yes |
| `DriveObject.metadata` | per-tenant private | `PII_QUASI` | yes |
| `MeetTranscript.text` | per-tenant private | `PII_IDENTIFYING` | yes |

Cross-tenant search index NEVER receives Workspace data unless explicit per-record consent + class allows (rare; e.g., a tenant publishes a doc to public corpus via the Sites surface).

### 5.6 Audit-chain emission

Every regulated capability invocation emits per ADR-0003:
- Mail send/receive (per-message envelope hash, classification, recipient(s) opted)
- Doc share (per-permission-grant)
- Drive download (per-object access)
- Meet recording start/stop (per-session)
- DSR cascade execution (proof-of-erasure per affected record)
- Legal hold imposition / release
- Foundry agent invocation on Workspace data (capability + autonomy tier + class touched)

### 5.7 Schema migration policy

Per [TOOLCHAIN.md](../../TOOLCHAIN.md) §3 schema-evolution; CRDT migrations require explicit version-vector compatibility tests; Mail / Calendar / Drive schema migrations require backward-read for ≥ 2 versions.

## 6. Optimization practices

| Practice | Implementation |
|---|---|
| Cell routing | Per-tenant cell binding; cross-cell traffic only via published replication |
| Sharding | Mail / Calendar / Doc / Drive sharded by `tenant_id`; Meet sharded by `session_id` for SFU placement nearest to median participant |
| Caching | Hot-message cache (Redis) for active mailboxes; doc CRDT snapshot cache; drive metadata cache |
| Bulk endpoints | `messages.batchSend`, `events.batchUpdate`, `drive.batchUpload`, `docs.batchExport` |
| Pagination | Cursor-based across all list endpoints |
| Idempotency | `Idempotency-Key` header on every write; deduped at API gate |
| Batch dispatch | Mail send batches under 1000; Drive uploads chunked at 8 MiB |
| Backpressure | SMTP receive applies sender-rate-limit; Doc CRDT applies edit-rate-limit |
| Hot-path benchmarks | Mail send p99 < 100ms; Doc edit propagation p99 < 80ms; Meet RTT p95 < 150ms intra-region |
| Agent-driven optimization | Foundry-driven mail-classification re-tuning; Foundry-driven calendar-scheduling optimization; Foundry-driven Doc-template suggestions |
| FinOps | Per-tenant per-surface unit-cost: storage GB, mail messages/month, doc edits, Meet minutes, transcription minutes |
| Build-cache | Per-Workspace-surface flat crate is independently cached; affected-graph testing on per-surface changes |

## 7. Regional pack interactions

| Seam | Per-pack impl needed? | Tested with |
|---|---|---|
| Mail server / outbound IP / SPF / DKIM / DMARC reputation per-region | yes | KR / JP / US / EU initial |
| Holiday calendar | yes | every pack |
| Language pack (Mail compose / Doc spell-check / Calendar locale) | yes | every pack |
| Per-region tax-invoice on Workspace billing | inherits from Cloud billing seam | every pack |
| Per-region identity provider (Workspace SSO) | inherits from Cloud IAM seam | every pack |
| Per-region content moderation (Chat / Sites / Forms) | yes | every pack |
| Per-region DLP rules (KR-RRN / JP-マイナンバー / US-SSN / EU-NI / IN-Aadhaar / BR-CPF) | yes | every pack |
| Per-region mail-security gate (KR has unique 메일 보안 controls; JP has spam/phishing reporting requirements; EU has DMARC enforcement) | yes | every pack |

## 8. In-house vs external dep posture

| Dependency | Maturity tier | License | In-house alternative? | Decision |
|---|---|---|---|---|
| `lettre` (Rust SMTP client) | mature | Apache-2 / MIT | yes | adopt |
| `imap` (Rust IMAP client) | mature | Apache-2 | yes | adopt |
| `caldav-rs` / hand-roll | maturing | varies | partial | hand-roll on top of HTTP |
| Y.js / Automerge for CRDT | mature | various (Y.js MIT; Automerge MIT) | partial — Yrs (Rust port of Y.js, MIT) is preferred | adopt Yrs |
| `webrtc-rs` for Meet | maturing | Apache-2 / MIT | partial — kernel-grade lib accepted | adopt with in-house SFU |
| Tesseract / OCR for Drive PDF preview | mature | Apache-2 | partial | adopt; consider in-house |
| ffmpeg for Meet recording transcoding | mature | LGPL | NO — license drift risk | replace with `gstreamer-rs` (LGPL same risk) OR build narrow in-house transcoder for the formats we ship; council decision |
| Whisper-class transcription | mature | MIT (whisper.cpp) | yes | adopt whisper.cpp + Foundry routing |
| Postgres (mail / calendar / doc metadata) | kernel-grade | PostgreSQL License | n/a | adopt |
| Redis (hot state) | mature pre-7.4 | BSD-3 (pre-7.4) / RSAL (post) | yes — Valkey | pin pre-7.4 OR migrate to Valkey |
| ClickHouse for usage analytics | mature | Apache-2 (own) / commercial vs YT-ClickHouse fork | yes — DataFusion | pin Apache-2 path; in-house long-term |
| Migration libs (Google Workspace Admin SDK / Microsoft Graph / Naver Works API) | mature | each provider's commercial license | partial — direct API call without SDK | direct REST per provider |

## 9. Success metrics

| Metric | Preview | Stable | GA |
|---|---|---|---|
| Mail deliverability | ≥ 99% | ≥ 99.5% | ≥ 99.9% |
| Mail spam-classification F1 | ≥ 0.85 | ≥ 0.90 | ≥ 0.95 |
| Doc edit-propagation p99 | < 200ms intra-region | < 100ms | < 80ms |
| Drive sync conflict rate | < 0.5% | < 0.1% | < 0.05% |
| Meet RTT p95 | < 250ms intra-region | < 200ms | < 150ms |
| Foundry-agent capability success on Workspace data | ≥ 85% | ≥ 92% | ≥ 95% |
| Per-tenant DSR cascade SLA | 30d | 14d | 7d |
| Migration ingest from Google Workspace / M365 / Naver Works | per-tenant N hours | < 24h | < 12h |
| Cross-axis-contract violations | 0 | 0 | 0 |

## 10. Risks + mitigations

| Risk | Severity | Mitigation | Owner |
|---|---|---|---|
| Mail-deliverability reputation collapse from outbound IP block | catastrophic | per-region warm IP pool; SPF/DKIM/DMARC right out of box; sender-reputation monitoring; per-tenant rate cap | `axis-workspace` |
| CRDT divergence on Doc | high | Yrs proven CRDT; per-doc state-vector contract test; deterministic merge replay | `axis-workspace` |
| Drive permission escalation | catastrophic | Cedar policy + per-object permission set; quarterly red-team | `axis-workspace` + `ops-security` |
| Meet recording leak | catastrophic | KMS-shred per recording; trust-portal access only; audit-chain per access | `axis-workspace` + `council-privacy` |
| DLP / phishing / malware miss in Mail | high | DLP gate + classifier + sandboxed attachment scan; per-vertical override (HIPAA tighter) | `axis-workspace` + `ops-security` |
| Foundry agent leaks mail content cross-tenant | catastrophic | per-tenant capability scope; Data Use Boundary enforcement; agent step audit | `axis-foundry` + `council-privacy` |

## 11. Open questions

1. Per-tenant mail domain naming (`<tenant>.oyatiemail.com` vs custom per-tenant domain — both supported but default?)
2. Meet maximum participants per session (50 / 200 / 500 / 1000?)
3. Workspace pricing model — per-seat fixed, per-seat usage-based, or per-bundle?
4. CRDT vs OT for Docs — Yrs (CRDT) recommended; OT alternative considered if performance demands?
5. Migration from Notion/Slack — first-class or third-party connector?

## 12. Decision log

| Date | Decision | Rationale |
|---|---|---|
| 2026-05-09 | Workspace added as Axis 2 | Comprehensive end-user productivity is essential to the cohesive-behemoth thesis |
| 2026-05-09 | Yrs (Rust port of Y.js) for CRDT | Best-in-class + Apache-2/MIT + same lang as backend |
| 2026-05-09 | In-house SFU for Meet | Latency + per-region cell-routing + cost |

## 13. Sources scanned

- Google Workspace product pages
- Naver Works product pages
- Microsoft 365 Business documentation
- AWS WorkMail / WorkDocs / Chime documentation
- Kakao Work documentation
- ProseMirror / Yjs / Automerge / Yrs documentation
- IETF RFCs: 5321 (SMTP), 3501 (IMAP), 8620 (JMAP), 4791 (CalDAV), 4918 (WebDAV), 6352 (CardDAV)
- WHIP / WHEP draft (WebRTC ingestion)
- Existing Oyatie consolidated docs (PRD, DESIGN, PRIVACY-PROGRAM, COMPLIANCE-MATRIX)
