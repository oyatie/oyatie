---
id: ADR-0029
status: Superseded
superseded_by: [ADR-700]
doc_status: published
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0029: microservice — dual-context communications (Professional + Personal) as cohesion-bound replacement for Google Workspace / M365 / Naver Works / Kakao Work

> **Status:** Accepted
> **Owner:** `oya-connect`
> **Date:** 2026-05-09 (rewritten 2026-05-13 — "Workspace" product renamed to "Connect"; flat µservice framing)
> **Related:** ADR-0001, ADR-0002, ADR-0003, ADR-0007, ADR-0011, ADR-0028, ADR-0035, ADR-0038, ADR-0043, ADR-0049, ADR-0055, ADR-0058, ADR-0060

---

## Context

is the communications + community microservice in the flat catalog. It carries two contexts:

- **Professional** — B2B; lives inside the Application shell; corporate mail, messenger, community for tenant employees. Ships in M03 alongside Enterprise microservices per `[[feedback-flat-product-catalog]]` M03 scope.
- **Personal** — B2C; person-pillar; does NOT go through the B2B Application shell. Separate entry path.

Prior to the 2026-05-13 session, this microservice was named "Workspace" and modeled as an axis. Both terms are retired. The product is now **Connect** (per `[[feedback-flat-product-catalog]]` override #7; Bominal ADR-0208 model). Professional implements dual-context enforcement, legal hold, and eDiscovery per Bominal ADR-0208 / ADR-0215 (inherited).

wins on three differentiators: (a) tenant-cell isolation so ads cannot be sourced from data without explicit per-tenant consent, (b) Foundry agent integration that is native, (c) per-tenant domain-bound mail/calendar with KMS-shred per object.

---

## Decision

We adopt as a **suite of twelve canonical apps** plus three adjunct surfaces, each its own bounded context under `oya-connect-<app>-*`, sharing the six substrates from ADR-0001 plus a Connect-internal **document-format kernel** and **collab-runtime kernel**.

**Naming justification (BNF v4.1, ADR-0056):**
- `oya-mail-kernel`: slot2 = `connector` (registered µservice); slot3 = `mail` (BC); slot4 = `kernel`
- `oya-collab-runtime-kernel`: slot2 = `connector`; slot3 = `collab-runtime` (multi-token BC); slot4 = `kernel`

### The twelve apps

| App | Crate prefix | Primary protocol | Per-tenant isolation unit |
|---|---|---|---|
| **Mail** | `oya-mail-*` | SMTP / IMAP4 / JMAP per-tenant domain | Per-tenant MX + per-tenant DKIM |
| **Calendar** | `oya-calendar-*` | CalDAV + iCalendar + Foundry scheduling agent | Per-tenant calendar database |
| **Docs** | `oya-docs-*` | Yrs CRDT WebSocket + REST | Per-tenant document store |
| **Sheets** | `oya-sheets-*` | Yrs CRDT + cell-graph kernel | Per-tenant sheet store |
| **Slides** | `oya-slides-*` | Yrs CRDT + slide-graph kernel | Per-tenant deck store |
| **Drive** | `oya-drive-*` | S3-compatible + folder/permission graph + sync clients | Per-tenant object store |
| **Meet** | `oya-meet-*` | WebRTC SFU + recording + Foundry transcription | Per-tenant SFU pool |
| **Chat** | `oya-connect-chat-*` | Messenger protocol + DM/group/channel/thread/bot | Per-tenant message store |
| **Forms** | `oya-forms-*` | Ontology-routed | Per-tenant form schema + responses |
| **Sites** | `oya-sites-*` | Static-site generator + Yrs CRDT editor | Per-tenant site store |
| **Tasks** | `oya-tasks-*` | Tasks-graph kernel + Foundry agent task runtime | Per-tenant task store |
| **Notes** | `oya-notes-*` | Yrs CRDT + tag/folder graph | Per-tenant note store |

Adjunct surfaces (ship subsequent-to-M03-completion): **Translate**, **Recordings**, **Address Book**.

### Per-tenant cell isolation

Every app deploys into a per-tenant cell (ADR-0028). Each cell is the unit of data residency, KMS-shred boundary (per ADR-0043), quota + isolation, and audit emission (per ADR-0003).

### Document-format kernel + collab-runtime kernel

```rust
// oya-collab-runtime-kernel
pub struct CollabRuntime {
    pub doc: yrs::Doc,
    pub awareness: yrs::sync::Awareness,
    pub persistence: PersistenceAdapter,
    pub access_control: AccessControlAdapter,
}

// oya-document-format-kernel
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

### Professional — dual-context enforcement (M03)

Per Bominal ADR-0208 / ADR-0215 (inherited):

- Corporate mail context: legal hold + eDiscovery for tenant employees.
- Personal context: separated data boundary; no cross-context data flow.
- Dual-context enforcement is live at M03 GA.

### Mail security

- Per-tenant phishing defense: inbound SPF/DKIM/DMARC validation; URL-rewrite + safe-link; attachment sandboxing in WASM (ADR-0036).
- DLP: per-tenant DLP rules; PHI/PCI/financial-credit data classifications trigger admin-review hold.
- PIPA + KISA-minimum-shippable-tier: per-tenant DPO can issue legal-hold per ADR-0038.

### Migration tooling from incumbent suites

`oya-connect-migrate-cli` ships per source platform: `--from google-workspace`, `--from m365`, `--from naver-works`, `--from kakao-work`. Each adapter is read-only against the source; produces a per-user migration manifest; validates on the destination side. Per-tenant migration sessions are audit-chained.

### Anti-scope

does not ship a database (uses ADR-0045), does not ship its own observability stack (uses ADR-0042), does not ship its own auth (uses ADR-0002).

---

## Consequences

### Concrete crate layout (BNF v4.1)

```
oya-mail-kernel          — mail domain types + ports
oya-mail-domain          — mail business logic
oya-mail-adapter         — SMTP/IMAP4/JMAP impl
oya-mail-worker          — inbound processing, DLP, classification
oya-calendar-kernel
oya-calendar-adapter     — CalDAV + iCalendar impl
oya-docs-kernel
oya-docs-adapter         — Yrs CRDT persistence
oya-sheets-kernel
oya-slides-kernel
oya-drive-kernel
oya-drive-adapter        — S3-compatible object store impl
oya-meet-kernel
oya-meet-worker          — WebRTC SFU + recording
oya-connect-chat-kernel
oya-connect-chat-adapter
oya-forms-kernel
oya-sites-kernel
oya-tasks-kernel
oya-notes-kernel
oya-collab-runtime-kernel  — Yrs CRDT shared runtime
oya-document-format-kernel — export/import (Docx/Hwpx/Odt/...)
oya-connect-migrate-cli            — migration tool from incumbent suites
oya-connect-rest                   — HTTP API surface
oya-connect-grpc                   — gRPC surface
oya-connect-app                    — composition-root binary
```

All crates registered under `connector` in `[workspace.metadata.oya.microservices]`.

### Positive

- Tenant-cell isolation makes the Data Use Boundary mechanically enforceable.
- Foundry agent integration is native — scheduling, summarization, transcription all flow through capability registry (ADR-0011).
- Migration tooling lowers switching cost from incumbent suites.
- KMS-shred per object satisfies PIPA Art 21 (right to erasure).
- Professional with legal hold/eDiscovery satisfies M03 KR group paying tenant requirement.

### Negative

- Twelve apps is a large surface; each must reach feature-parity-enough to keep users.
- Yrs CRDT at scale is non-trivial; `oya-collab-runtime-kernel` becomes a hot reliability surface.
- HWPX export is a specific, fiddly format.
- Mail deliverability is its own discipline.

---

## Related

- ADR-0001 (cohesion — is a µservice in the flat catalog)
- ADR-0002 (tenant + identity)
- ADR-0003 (audit chain)
- ADR-0035 (Workflow engine — calendar/action-card handoffs flow through Workflow)
- ADR-0038 (trust framework + DSR cascade)
- ADR-0043 (HSM + KMS — per-object key-shred)
- ADR-0049 (residency)
- ADR-0055 (Ontology — Forms routed through Ontology)
- ADR-0058 (Flat microservice catalog)
- ADR-0060 (Bominal-inheritance precedence — ADR-0208 / ADR-0215 inherited)
- `[[feedback-flat-product-catalog]]` — Professional ships M03; Workspace name retired
- Bominal ADR-0208 (dual-context, inherited)
- Bominal ADR-0215 (retention / legal hold / dual-context boundary, inherited)
