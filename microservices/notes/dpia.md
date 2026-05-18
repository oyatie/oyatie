---
doc_class: DPIA
title: notes µservice — Data Protection Impact Assessment
microservice: notes
status: Accepted
classification: CONFIDENTIAL
date: 2026-05-17
owner_team: council-privacy + axis-notes + ops-legal
deciders: council-privacy, ops-legal, axis-notes, gtm-customer-success
related_adrs: [ADR-0008, ADR-0028, ADR-0117, ADR-0126, ADR-0131, ADR-NOTES-0001, ADR-NOTES-0005]
review_cadence: annually + on every BC change touching PII
references:
  - GDPR Arts. 5, 6, 9, 17, 22, 25, 32, 35; Recital 26
  - KR PIPA Arts. 15, 17, 22-2, 23, 28, 29
  - HIPAA 45 CFR §164
  - APPI (JP); PDPA (SG, AU); DPDPA 2023 (IN); LGPD (BR); UAE PDPL; KSA PDPL
  - EU AI Act Art. 50 (transparency)
  - ePrivacy Directive 2002/58/EC Art. 5
  - WP29 DPIA guidance (WP248 rev.01)
doc_status: published
---

# DPIA — notes µservice

## Section 1 — Description of Processing

### 1.1 Purpose

The notes µservice processes personal data for the purpose of providing short-form personal-note + knowledge-capture functionality to end-users, including:

- Capture, edit, organise short-form Markdown notes.
- Bidirectional `[[wikilink]]` knowledge graph + tag-graph.
- Web-clipper-based capture from arbitrary web pages (URL + selected text + page metadata).
- Daily-note timeline (one note auto-created per user-local day).
- Share-link emission for read-only external sharing.
- Optional AI-assist (summarize, tag-suggest, link-suggest) on Professional-tier notes only.
- Linear version-history per note.
- Optional Loro CRDT-based real-time collaboration on Professional-tier notes.
- Cross-µservice integration via Workflow events + Ontology object writes.

### 1.2 Categories of data subjects

- End-users of oyatie tenants (Personal + Professional pillar).
- People mentioned within end-user notes (third-party data subjects).
- Web-page subjects whose content is clipped.

### 1.3 Categories of personal data

| Category | Examples | Tier |
|---|---|---|
| Identifying (basic) | name, email, user_id, OIDC sub | both |
| Behavioral | note-create timestamps, tag adjacency, backlink graph, last-opened | both |
| Content (notes body) | Markdown text, attachments, embed refs | both; Personal-tier E2E-protected |
| Special category (Art. 9 GDPR) | health, sexuality, religion, biometric (if user chooses to write) | both; Personal-tier E2E-protected makes server-side processing structurally impossible |
| Third-party identifiers | people @-mentioned in notes (data not authored by mentioned subject) | both |
| Browsing metadata (web-clipper) | clipped URL, clipped HTML, page metadata | both; Personal-tier E2E-protected |

### 1.4 Recipients

- The end-user themselves (primary subject + controller of own personal-pillar notes).
- The tenant (controller of Professional-tier notes; processor for user-content within tenant scope).
- oyatie (processor).
- foundry-runtime µservice (processor for AI assist; never on E2E notes).
- drive µservice (processor for attachment storage).
- tasks µservice (processor for checklist emissions).
- Cross-pack recipients: forbidden by default (see `policy/data-residency.md`).

### 1.5 Retention

Per `policy/data-residency.md` — pack-aware retention bounds; Personal-tier follows per-user policy (default = no retention floor); Professional-tier follows tenant + pack overlay.

### 1.6 Cross-border transfer

Forbidden by default per pack pinning. Allowed only with SCC (GDPR) + tenant-of-tenant consent.

## Section 2 — Necessity + Proportionality

### 2.1 Lawful basis

| Tier | Basis | Article |
|---|---|---|
| Personal-tier (B2C) | Consent (Art. 6(1)(a)); explicit at signup | GDPR Art. 6(1)(a); KR PIPA Art. 15(1)(1); ePrivacy Art. 5 |
| Professional-tier (B2B) | Legitimate interest of tenant (Art. 6(1)(f)) | GDPR Art. 6(1)(f); KR PIPA Art. 15(1)(2) |
| Special category content (E2E-protected) | Art. 9(2)(a) explicit consent BY THE USER; server has no plaintext access | GDPR Art. 9(2)(a); Recital 51 |

### 2.2 Data minimisation

- Personal-tier Workflow events carry opaque `note_id` only — no title, no body, no tag.
- Personal-tier Ontology writes are MINIMAL (no title, no body, no tag).
- Professional-tier events carry titles + tags but never plaintext body in the event payload (body fetched server-side under Cedar).
- Tags are user-authored; no automated tagging without user consent (T1 capability is opt-in per tenant).

### 2.3 Purpose limitation

The µservice MUST NOT process personal data for any purpose beyond user-facing note-capture + organisation + (consented) AI assist. Cross-product use of note content is forbidden absent explicit consent.

### 2.4 Subject rights

| Right | Implementation |
|---|---|
| Access (Art. 15) | Per-user export pipeline (Markdown + frontmatter + JSON Canonical) |
| Rectification (Art. 16) | Inline edit by user; admin edit forbidden on Personal-tier |
| Erasure (Art. 17) | Per-user DSR cascade runner; tombstone + redact identifiers; cross-pack erasure not needed (pack pinning) |
| Restriction (Art. 18) | Per-tenant suspend; per-user export hold |
| Portability (Art. 20) | Markdown + JSON Canonical export; Obsidian-vault export sub-format |
| Objection (Art. 21) | AI-assist tenant-admin disable; per-user opt-out flag |
| Automated-decision (Art. 22) | T2 auto-organize tier disabled at MVP; if enabled, requires explicit per-user opt-in + audit; human review available |

## Section 3 — Risks

| ID | Risk | Likelihood | Impact | Mitigation | Residual |
|---|---|---|---|---|---|
| R-01 | Personal-tier server-side plaintext leak | low (structurally impossible) | catastrophic if violated | ADR-NOTES-0001 E2E-default + LEAN lane + Cedar `e2e-ai-refusal.cedar` | residual = 0 by data model |
| R-02 | Tenant-admin reads Personal-tier without authority | low | catastrophic | DCI-03 + Cedar unconditional `forbid` on admin-disclosure of Personal | residual = 0 by policy |
| R-03 | AI provider exfiltrates Personal-tier content via assist call | n/a (refused) | catastrophic | ADR-NOTES-0005 invariant + CI lane `oya-check-e2e-ai-refusal` | residual = 0 |
| R-04 | Cross-tenant search bleed | low | high | per-tenant Meilisearch namespace + Cedar | residual = low |
| R-05 | Web-clipper installation-token leak | medium | high | per-install token + MV3 isolated world + rotation 90d | residual = low |
| R-06 | Share-link enumeration | medium | medium | 128-bit URL-safe token + rate-limit + CAPTCHA | residual = low |
| R-07 | Cross-context invariant violation (Personal → Professional) | low | catastrophic | DCI-01..07 immutable enum + LEAN lane | residual = 0 by data model |
| R-08 | Import pipeline injects malicious content (XSS in note body) | medium | medium | strict Markdown sanitisation + CSP + sandboxed render | residual = low |
| R-09 | Backlink graph reveals private notebook structure to unauthorised viewer | low | medium | graph-view server-side Cedar scope + per-tenant isolation | residual = low |
| R-10 | DSR erasure incomplete (search index residue) | medium | medium | erasure runs purge across Postgres + Meilisearch + S3 + version-history; verification step | residual = low |
| R-11 | Cross-region replication of Personal-tier ciphertext to unauthorised pack | low | high | pack-pinning CI lane; per-pod-label `oyatie/pack` enforcement | residual = low |
| R-12 | Recovery seed loss → permanent data destruction (Personal-tier) | medium | catastrophic for user | accepted tradeoff; documented at onboarding with double-confirmation UX; analogous to Apple iCloud Advanced Data Protection | residual = low (with explicit user acknowledgement) |
| R-13 | Loro CRDT op-log leaks edit-pattern PII | low | medium | Professional-only; per-tenant isolation; op-log compaction | residual = low |
| R-14 | Daily-note auto-create generates note in wrong timezone → wrong-day attribution | medium | low | user-local-tz authoritative; documented in Open Q #4 | residual = low |
| R-15 | T1 AI-assist provider stores prompt for training | medium | high | per-pack tenant gating + DPA with provider; foundry-runtime contract: no-train clause; tenant opt-in | residual = low |
| R-16 | Embed reference reveals drive-µservice attachment to non-member | low | medium | drive-µservice resolves under requester's Cedar scope; embed never bypasses | residual = low |

## Section 4 — Mitigation Detail

### Personal-pillar E2E (DCI-03)

Personal-tier notes use MLS RFC 9420 + openmls 0.6 client-side key derivation. Server stores ciphertext + KeyPackage signing certs + commit messages only. ADR-NOTES-0001 establishes this as canonical posture.

### AI E2E-refusal (ADR-NOTES-0005)

- Cedar `e2e-ai-refusal.cedar` unconditional `forbid` on `Action::ai_call` over Personal+E2E resources.
- Type signature on `AssistInvoker::invoke(ProfessionalNoteRef)` refuses cross-tier construction.
- CI lane `oya-check-e2e-ai-refusal` blocks any path from `PersonalNoteRef` to `AssistInvoker::invoke`.
- Runtime metric `oya_notes_ai_call_blocked_e2e_total` increments on any forbidden attempt; alarms at > 0.

### Pack pinning

Per `policy/data-residency.md` — every Helm release tagged with `oyatie/pack`; CI lane `oya-check-notes-pack-residency` asserts every row carries the cluster's pack; periodic Postgres audit.

### Subject access + portability

- Per-user export job pipeline (`oya-notes-export-pipeline-*`) emits Markdown + frontmatter + JSON Canonical bundle inside SLA per pack.
- Obsidian-vault export sub-format provided for cross-tool portability.

## Section 5 — Approval

| Reviewer | Date | Signature |
|---|---|---|
| Council of Privacy chair | 2026-05-17 | (pending) |
| GDPR-DPO | 2026-05-17 | (pending) |
| KR-PIPC liaison | 2026-05-17 | (pending) |
| HIPAA officer (pack-us-healthcare) | conditional | (post-BAA) |
| Tenant-DPO (per active pack) | per-tenant | (per-deployment) |

## Section 6 — Review

Annual review unless a BC adds PII/PHI category or AI capability tier escalates; in that case the DPIA is re-opened.

## References

- See header `references:` block.
- ADR-NOTES-0001, ADR-NOTES-0005.
- `microservices/notes/policy/dual-context-isolation.md`.
- `microservices/notes/policy/data-residency.md`.
- `microservices/notes/threat-model.md`.
