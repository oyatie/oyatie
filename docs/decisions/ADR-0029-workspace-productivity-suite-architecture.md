# ADR-0029: Workspace axis — productivity suite architecture as cohesion-bound replacement for Google Workspace / M365 / Naver Works / Kakao Work

> **Status:** Proposed
> **Supersedes:** -
> **Superseded-by:** -
> **Owner:** `axis-workspace`
> **Date:** 2026-05-09
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0007, ADR-0011, ADR-0028, ADR-0035, ADR-0038, ADR-0043, ADR-0049

---

## Context

Axis 2 (Workspace) is the daily-use surface for every employee of every tenant. It is the place users *live*; if Workspace is unfamiliar, fragmented, or laggy, the entire ecosystem moat collapses because users keep their incumbent suite (Google Workspace, M365, Naver Works, Kakao Work) and bypass us. The pack-of-19 foundation ADRs named Workspace as a separate axis but did not pin its product surface or its substrate-consumption pattern. The cohesion thesis (ADR-0001) says Workspace must consume the same Tenant / Identity / Audit / Capability / Runtime / Autonomy substrates that every other axis consumes — so Workspace cannot ship its own auth, its own audit chain, or its own consent model.

The competitive bar is set by mature suites that have shipped for 15-20 years; we cannot win on feature parity alone. We win on three differentiators: (a) tenant-cell isolation per Data Use Boundary (ads cannot be sourced from Workspace data unless tenant explicitly opts in), (b) Foundry agent integration that is native rather than bolted-on, and (c) per-tenant domain-bound mail/calendar with KMS-shred per object. This ADR pins the surface, the per-app architecture, the per-tenant cell isolation pattern, the migration tooling from incumbent suites, and the KMS-shred guarantee.

---

## Decision

We adopt a **suite-of-twelve canonical apps**, each implemented as its own bounded context under `crates/oya-workspace-<app>-*`, sharing the six substrates from ADR-0001 plus a Workspace-internal **document-format kernel** (used by Docs/Sheets/Slides only) and a Workspace-internal **collab-runtime kernel** (Yrs CRDT, used by all real-time-collaborative surfaces).

### The twelve apps

| App | Bounded context | Primary protocol surface | Per-tenant isolation unit |
|---|---|---|---|
| **Mail** | `crates/oya-workspace-mail-*` | SMTP / IMAP4 / JMAP per-tenant domain | Per-tenant MX + per-tenant DKIM |
| **Calendar** | `crates/oya-workspace-calendar-*` | CalDAV + iCalendar + Foundry scheduling agent | Per-tenant calendar database |
| **Docs** | `crates/oya-workspace-docs-*` | Yrs CRDT WebSocket + REST | Per-tenant document store |
| **Sheets** | `crates/oya-workspace-sheets-*` | Yrs CRDT + cell-graph kernel | Per-tenant sheet store |
| **Slides** | `crates/oya-workspace-slides-*` | Yrs CRDT + slide-graph kernel | Per-tenant deck store |
| **Drive** | `crates/oya-workspace-drive-*` | S3-compatible + folder/permission graph + sync clients | Per-tenant object store |
| **Meet** | `crates/oya-workspace-meet-*` | WebRTC SFU + recording + Foundry transcription | Per-tenant SFU pool |
| **Chat** | `crates/oya-workspace-chat-*` | XMPP-derived + DM/group/channel/thread/bot surface | Per-tenant message store |
| **Forms** | `crates/oya-workspace-forms-*` | Object-Graph routed | Per-tenant form schema + responses |
| **Sites** | `crates/oya-workspace-sites-*` | Static-site generator + Yrs CRDT editor | Per-tenant site store |
| **Tasks** | `crates/oya-workspace-tasks-*` | Tasks-graph kernel + Foundry agent task runtime | Per-tenant task store |
| **Notes** | `crates/oya-workspace-notes-*` | Yrs CRDT + tag/folder graph | Per-tenant note store |

Plus three **adjunct surfaces** that ship in W4..W8: **Translate** (per-tenant locale + glossary), **Recordings** (transcoded archive of Meet recordings with retention policy per Data Use Boundary), and **Address Book** (per-tenant + per-user contacts; integrates with Mail and Calendar).

### Per-tenant cell isolation

Every Workspace app is deployed into a per-tenant cell (per ADR-0028 cells). A cell is the unit of:

- **Data residency.** All app state for a tenant lives in cells that satisfy that tenant's residency class (per ADR-0049).
- **KMS-shred boundary.** Per-tenant KMS key (ADR-0043) wraps every Drive object's CEK, every Mail body's encryption key, and every Meet recording's encryption key. Tenant deletion (or DSR erase, per ADR-0038) shreds the per-tenant KMS key, rendering all per-object CEKs unrecoverable.
- **Quota and isolation.** Per-cell CPU / memory / storage / network / SFU pool quotas are enforced; one tenant cannot starve another.
- **Audit emission.** Every state-changing API call emits to the audit chain (ADR-0003) tagged with the cell ID.

### Document-format kernel + collab-runtime kernel

```rust
// crates/oya-workspace-collab-runtime
pub struct CollabRuntime {
    pub doc: yrs::Doc,
    pub awareness: yrs::sync::Awareness,
    pub persistence: PersistenceAdapter,
    pub access_control: AccessControlAdapter,
}

// crates/oya-workspace-document-format
pub trait DocumentExporter {
    fn export(&self, doc: &CollabRuntime, format: ExportFormat) -> Result<Bytes>;
}

pub enum ExportFormat {
    Pdf,
    Docx, Xlsx, Pptx,   // M365 family
    Hwpx,               // Hancom (KR-mandatory for public-sector tenders)
    Odt, Ods, Odp,      // OpenDocument family
    Markdown, Csv, Tsv,
}
```

Both kernels are Workspace-internal (not promoted to a six-substrate). They are the only place real-time collaborative state lives, and the only place exporters live; per-app crates wrap them with app-specific schema (sheet cells, slide layouts, doc paragraphs, etc.).

### Mail security

- **Per-tenant phishing defense.** Inbound SPF/DKIM/DMARC validation; URL-rewrite + safe-link; attachment sandboxing in WASM (ADR-0036).
- **DLP.** Per-tenant DLP rules; outbound mail scanned against tenant policy; PHI/PCI/financial-credit data classifications (per ADR-0034) trigger admin-review hold.
- **Classify.** Mail classified at delivery into per-tenant data-class buckets; classification recorded in audit chain.
- **PIPA + KISA-MVP.** KR cells satisfy both; per-tenant DPO can issue legal-hold per ADR-0038.

### Calendar smart scheduling

The Foundry-driven scheduling agent (capability `workflow.workspace.schedule`) consumes per-attendee free/busy + per-room availability + per-tenant policy (e.g. "no meetings during Korean lunch 12:00-13:00") and proposes optimal slots. The agent runs at the configured persona-tier autonomy ceiling (per ADR-0007) — at `coworker` tier or above it can hold provisional slots; at `assistant` tier it only proposes.

### Migration tooling from incumbent suites

A standalone tool `oya-workspace-migrate` ships per source platform:

- `--from google-workspace` (Gmail / Calendar / Drive / Docs / Sheets / Slides via Workspace Migration API)
- `--from m365` (Exchange / Outlook / OneDrive / Word / Excel / PowerPoint via Graph API)
- `--from naver-works` (KR-specific; per Naver Cloud Platform mail/calendar API)
- `--from kakao-work` (KR-specific; per Kakao Work admin API)

Each adapter is read-only against the source, produces a per-user migration manifest (mailboxes / calendar events / files / docs), and validates on the destination side via the Workspace ingest API. Per-tenant migration sessions are audit-chained.

### Anti-scope

Workspace does not ship a database (uses ADR-0045 tier), does not ship its own observability stack (uses ADR-0042), does not ship its own auth (uses ADR-0002 identity kernel), does not ship plugins outside the WASM substrate (ADR-0036).

---

## Consequences

### Positive

- Tenant-cell isolation makes the Data Use Boundary mechanically enforceable: ads cannot be sourced from Workspace state without an explicit per-tenant grant chained to consent.
- Foundry agent integration is native: scheduling, summarization, translation, and meeting transcription all flow through the same capability registry (ADR-0011), with the same autonomy ceiling, the same audit chain.
- Migration tooling lowers switching cost from incumbent suites — the dominant adoption barrier for KR enterprise + public-sector tenants.
- KMS-shred per object gives a defensible answer to PIPA Art 21 (right to erasure) that incumbent suites cannot match without architectural rework.

### Negative

- Twelve apps is a lot of surface; each must reach feature-parity-enough to keep users from re-installing the incumbent.
- Real-time collaborative editing (Yrs CRDT) at scale is non-trivial; the collab-runtime kernel becomes a hot reliability surface.
- HWPX export for public-sector KR is a specific, fiddly format; we own the parser/serializer tax.
- Mail deliverability is its own discipline; we must invest in IP reputation, DKIM key rotation, and abuse-handling early.

### Operational

- Per-app SLO catalog (per ADR-0042) tracks: Mail delivery latency, Calendar event-write latency, Docs collab edit latency (P50 < 80ms), Drive object-PUT latency, Meet join latency, Chat message-send latency.
- Per-tenant DPO console exposes DSR cascade across all twelve apps in one workflow (per ADR-0038).
- Per-cell capacity headroom monitored separately from generic Cloud cells; Workspace cells have tighter latency SLOs.
- KMS key rotation drill quarterly per cell (ADR-0043).
- Incident classification: a Mail outage in a paying-tenant cell is Sev-1; a Notes outage is Sev-2.

---

## Alternatives considered

### Alternative A — White-label an existing open-source suite (Nextcloud / Zimbra / Mattermost)

- **Pros:** faster to ship; existing migration tooling.
- **Cons:** does not consume the six substrates; does not satisfy per-tenant cell isolation; KMS-shred would be bolted on; cohesion thesis violated.
- **Rejected because:** the cohesion moat is the whole point.

### Alternative B — Three apps (Mail/Calendar/Drive) only at launch, defer the other nine

- **Pros:** smaller initial surface; faster GA.
- **Cons:** users churn back to incumbent suites for the missing apps; the per-tenant-cell pattern fragments because three of twelve apps live in our cells.
- **Rejected because:** the moat requires *daily-use* density; three apps is not enough.

### Alternative C — Per-app independent runtime (no shared collab-runtime kernel)

- **Pros:** simpler per-app teams.
- **Cons:** Yrs CRDT integration drift across Docs/Sheets/Slides/Notes/Sites becomes a perpetual integration cost; the document-format exporter has to be re-implemented per app.
- **Rejected because:** the same drift the cohesion thesis exists to prevent.

### Alternative D — Defer migration tooling to a partner

- **Pros:** less code we own.
- **Cons:** switching cost from incumbents is the #1 adoption barrier; outsourcing it means losing the deal.
- **Rejected because:** the migration tool is a sales tool, not a feature.

---

## Open questions

1. **Q1.** Does Sheets ship with a Foundry-driven formula authoring agent at GA, or in W+12? Default: in-W+12. → ADR-0035 (workflow engine integration).
2. **Q2.** Does Meet record by default for tenants on regulated data classes, or opt-in per meeting? Default: opt-in per meeting; per-tenant policy can flip to default-on. → ADR-0034.
3. **Q3.** Mail server runtime — own SMTP daemon (Rust) or wrap an existing MTA? Default: own (Rust); existing MTAs (Postfix/Exim/Haraka) are LGPL/GPL-tainted or unmaintained for our scale model. → ADR pending in product-license-policy.
4. **Q4.** WebRTC SFU build-vs-buy — own (mediasoup port to Rust) or LiveKit OSS? Default: LiveKit (Apache-2) at Phase 1 with adapter; in-house at Phase 3. → ADR pending in build-vs-buy-policy.
5. **Q5.** Do we ship a per-tenant DLP rule editor at launch, or defer to Foundry workflow studio? Default: defer to Foundry workflow studio (consistent UX). → ADR-0035.

---

## References

- `docs/PRD.md` §7 (workspace axis), §11 (per-tenant residency)
- `docs/DESIGN.md` §4 (workspace architecture), §10 (cross-axis contracts), §11 (DSR cascade)
- KR 「개인정보보호법」 (PIPA) Art 21 (right to erasure), Art 24 (sensitive data); 「정보통신망법」 (NIA Act); KISA 클라우드보안인증 (CSAP)
- Yjs / Yrs CRDT spec; iCalendar RFC 5545; CalDAV RFC 4791; JMAP RFC 8620; WebRTC RFC 8825
- ADR-0001 (cohesion thesis), ADR-0002 (tenant + identity), ADR-0003 (audit chain), ADR-0007 (Cedar policy + persona tier), ADR-0011 (capability registry), ADR-0028 (cloud), ADR-0035 (workflow engine), ADR-0038 (trust framework + DSR cascade), ADR-0043 (HSM + KMS), ADR-0049 (residency)
