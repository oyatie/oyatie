---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: docs
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-docs + council-architecture
deciders: axis-docs, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0126, ADR-0131, ADR-0132, ADR-0133, ADR-DOCS-0001, ADR-DOCS-0002, ADR-DOCS-0003, ADR-DOCS-0004, ADR-DOCS-0005, ADR-DOCS-0006]
related_artifacts:
  - microservices/docs/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-DOCS gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (docs µservice)

## Purpose

Quantitative + qualitative parity comparison against industry-leading collaborative document products. Drives `oya-governance-hyperscaler-maturity-claims` gate per HG-DOCS (ADR-0123) and constrains what gtm-customer-success can claim in tenant sales. Re-validated bi-annually because the docs landscape moves quickly (Google Docs AI scheduling, Microsoft Copilot in Word, Notion AI, the Coda doc-as-app movement).

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Google Docs | Workspace Docs | OT-based collab; Workspace integration; suggestion mode; export to many formats | `developers.google.com/docs/api` |
| Microsoft Word Web | M365 Word | OOXML round-trip; Copilot; review/track-changes; rich tables | `learn.microsoft.com/graph/api/resources/document` |
| Notion | Notion pages | block-based model; per-block ACL; database integration; embeds | `developers.notion.com` |
| Coda | Coda docs | doc-as-app; programmable tables; integrations | `coda.io/developers` |
| Quip (Salesforce) | Quip | collab; comments; spreadsheet embed; Salesforce-data integration | `quip.com/dev/automation` |
| Dropbox Paper | Paper | lightweight; embed-friendly; Markdown export | `developers.dropbox.com/paper` |
| ONLYOFFICE | ONLYOFFICE Docs | OOXML-first; self-hostable; WebSocket collab | `api.onlyoffice.com` |
| Collabora Online | Collabora | LibreOffice-derived; self-hostable; OOXML/ODF | `sdk.collaboraonline.com` |
| Etherpad | Etherpad | OT-based plain-text collab (legacy reference) | `etherpad.org` |
| HackMD / CodiMD | HackMD | Markdown-first; technical docs | `hackmd.io/api/v1` |
| Confluence | Atlassian | enterprise wiki + docs; permissions; macros | `developer.atlassian.com/cloud/confluence` |
| Obsidian Publish | Obsidian | Markdown-first publishing; bi-directional links | `help.obsidian.md/Obsidian+Publish/Publish` |
| Craft | Craft Docs | block-based; Apple-ecosystem polish | (consumer; no public API) |
| Bear | Bear | Markdown-first writing app | (consumer; no public API) |
| Roam Research | Roam | graph-style; block references | `roamresearch.com/help` |

## Feature Parity Matrix

### Core document model

| Capability | oyatie | Google | MS Word Web | Notion | Coda | Quip | Paper | ONLYOFFICE | Confluence |
|---|---|---|---|---|---|---|---|---|---|
| Block-based authoring | ✅ Notion-class | partial (per-paragraph) | partial | ✅ | ✅ | partial | partial | ❌ (page-based) | partial |
| Per-block ACL | ✅ AC-04 (differentiator) | ❌ (whole-doc) | ❌ | ✅ | partial | ❌ | ❌ | ❌ | partial (page) |
| Rich inline formatting (bold/italic/strikethrough/code/link) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Headings + outline + TOC | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tables (rich + nested) | ✅ | ✅ | ✅ | ✅ | ✅ (programmable) | ✅ | partial | ✅ | ✅ |
| Lists (ordered/unordered/checklist) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Images + alt-text (WCAG 2.2 AA) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Embed (iframe / media) | ✅ via embed-resolver | ✅ | partial | ✅ | ✅ | ✅ | ✅ | partial | ✅ |
| Code blocks (syntax highlight) | ✅ tree-sitter | partial | partial | ✅ | partial | partial | ✅ | partial | ✅ |
| Math (KaTeX / MathJax) | ✅ KaTeX default | partial (Equation Editor) | ✅ | partial | partial | ❌ | ❌ | ✅ | ✅ (plugin) |
| Callouts / blockquotes | ✅ | ✅ | partial | ✅ | ✅ | partial | partial | partial | ✅ |
| Citations (BibTeX / academic) | ✅ M03+1 | partial | partial | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Collaboration

| Capability | oyatie | Google | MS Word Web | Notion | Coda | Quip | Paper | ONLYOFFICE | Etherpad |
|---|---|---|---|---|---|---|---|---|---|
| Real-time co-editing | ✅ Loro CRDT | ✅ OT | ✅ OT | ✅ Yjs CRDT | ✅ | ✅ OT | ✅ | ✅ WebSocket | ✅ OT |
| Cursor + presence | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Zero-silent-loss CRDT (AC-06) | ✅ load-bearing (differentiator) | partial (OT race) | partial | ✅ | partial | partial | partial | partial | partial (OT race) |
| Comments + threads | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| Suggestions (track-changes) | ✅ | ✅ | ✅ | partial | partial | ❌ | ❌ | partial | ❌ |
| Mentions (cross-µservice link) | ✅ via messenger | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | partial | ❌ |
| Version history + revert | ✅ Merkle-chained | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | partial |
| Audit-chain (Ed25519 + Merkle) on every lifecycle | ✅ (differentiator) | partial (Vault) | partial (eDiscovery) | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Protocols + interop

| Protocol | oyatie | Google | MS Word Web | Notion | Coda | Quip | ONLYOFFICE | Confluence |
|---|---|---|---|---|---|---|---|---|
| OOXML (DOCX) import | ✅ ECMA-376 + ADR-DOCS-0006 best-effort | ✅ | ✅ (native) | partial | partial | partial | ✅ | partial |
| OOXML (DOCX) export | ✅ | ✅ | ✅ (native) | partial | partial | partial | ✅ | partial |
| OOXML round-trip fidelity ≥ 95% | ✅ AC-03 (differentiator) | partial | ✅ | partial | partial | partial | partial | partial |
| Markdown import | ✅ CommonMark + GFM | partial | partial | ✅ | partial | ✅ | partial | partial |
| Markdown export | ✅ | partial | partial | ✅ | partial | ✅ | partial | partial |
| HTML import | ✅ ammonia sanitiser | ✅ | ✅ | partial | partial | partial | ✅ | partial |
| HTML export | ✅ | ✅ | ✅ | partial | partial | partial | ✅ | partial |
| PDF export | ✅ WeasyPrint + Chromium opt-in | ✅ | ✅ | partial | partial | partial | ✅ | partial |
| PDF/A-1b + PDF/A-2u archival export | ✅ AC-10 (differentiator) | ❌ | ✅ | ❌ | ❌ | ❌ | partial | ❌ |
| EPUB 3 export | ✅ M03+1 | ❌ | ❌ | ❌ | ❌ | ❌ | partial | ❌ |
| LaTeX export | ✅ via Pandoc | partial (3rd-party) | ❌ | ❌ | ❌ | ❌ | partial | ❌ |
| WebSocket collab protocol | ✅ M03 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | partial |
| CRDT op envelope (cross-µservice consistent) | ✅ shared with workflow-studio | ❌ | ❌ | ✅ (Yjs internal) | ❌ | ❌ | ❌ | ❌ |
| Google Docs API compat shim | ✅ M04 (read-only) | n/a | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Microsoft Graph (Docs) compat shim | ✅ M04 (read-only) | ❌ | n/a | ❌ | ❌ | ❌ | ❌ | ❌ |

### Accessibility + privacy + isolation

| Capability | oyatie | Google | MS Word Web | Notion | Coda | Quip | Confluence |
|---|---|---|---|---|---|---|---|
| WCAG 2.2 AA compliance (editor + export) | ✅ AC-11 | ✅ | ✅ | partial | partial | partial | partial |
| Dual-context (Personal/Professional) structural isolation | ✅ (differentiator) | ❌ (acct switching) | ❌ (acct switching) | partial | ❌ | ❌ | ❌ |
| Cross-org sharing with policy-bounded disclosure (Cedar) | ✅ (differentiator) | partial (link permissions) | partial | partial | partial | partial | partial |
| E2E encryption at rest (Tenant-DEK) | ✅ professional context | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Per-jurisdiction retention (11 packs) | ✅ M03 | partial | partial | ❌ | ❌ | ❌ | partial |
| Legal hold on documents | ✅ | ✅ (Vault) | ✅ (eDiscovery) | ❌ | ❌ | ❌ | partial |
| eIDAS PAdES signed PDF export | ✅ M03+1 (pack-eu) | ❌ | partial | ❌ | ❌ | ❌ | ❌ |

### AI + assist (autonomy tiers)

| Capability | oyatie | Google | MS Word Web | Notion | Coda | Quip |
|---|---|---|---|---|---|---|
| T0 grammar suggestion | ✅ M03 | ✅ Smart Compose | ✅ Editor | ✅ AI | ❌ | ❌ |
| T0 title / TOC suggestion | ✅ M03 | partial | ✅ Copilot | ✅ AI | ❌ | ❌ |
| T1 auto-summary | ✅ M03+1 | partial (Duet) | ✅ Copilot | ✅ AI | ❌ | ❌ |
| T1 expand / rewrite suggestion | ✅ M03+1 | partial | ✅ Copilot | ✅ AI | ❌ | ❌ |
| T1 citation suggestion | ✅ M03+1 | partial | partial | ❌ | ❌ | ❌ |
| T2 auto-translate | ✅ M04 | partial | ✅ Copilot | ✅ AI | ❌ | ❌ |
| T2 auto-format | ✅ M04 | partial | ✅ Copilot | partial | ❌ | ❌ |
| T2 auto-cite | ✅ M04 | partial | partial | ❌ | ❌ | ❌ |
| EU AI Act Annex III §3 conformity (HR overlay) | ✅ refused at Cedar layer until ADR-DOCS-0005 | unclear | unclear | unclear | unclear | unclear |

## Key differentiators (ordered)

1. **Per-block ACL with Cedar-gated cross-µservice embed passthrough** — Notion has per-block but no cross-µservice embed ACL passthrough; Google Docs has no per-block.
2. **Zero-silent-loss CRDT (AC-06) with cross-µservice library alignment** — Loro shared with workflow-studio via published port-trait re-export; competitor solutions are OT (Google/MS/Etherpad) or Yjs (Notion).
3. **Dual-context (Personal / Professional) structural isolation enforced in code** — competitor solutions are policy-only or account-switching.
4. **Audit-chain (Ed25519 + Merkle) on every document lifecycle** — beyond what enterprise competitors offer; eIDAS-compliant.
5. **OOXML round-trip fidelity ≥ 95% on Microsoft corpus** — best-effort tier per ADR-DOCS-0006; Microsoft Word Web is the bar; Notion + Coda + Quip lag.
6. **PDF/A-1b + PDF/A-2u archival-grade export** — Google Docs has no PDF/A; Microsoft Word has PDF/A; LibreOffice/Collabora have PDF/A.
7. **11-pack regulatory overlay** — per-jurisdiction retention, Hijri/imperial calendar in PDF metadata, Sharia retention extension.
8. **eIDAS PAdES B-LT signed PDF export** for legal-evidence tenants (pack-eu) — Microsoft Word has partial; competitors lag.
9. **WCAG 2.2 AA compliance** — Google Docs is the bar; Notion + Coda + Quip + Confluence partial.

## Gap closing plan (M03 → M05)

| Gap | Current state | Plan | Target |
|---|---|---|---|
| EPUB 3 export | M03+1 | Pandoc EPUB writer + post-process | M03+1 |
| LaTeX export | M03+1 | Pandoc LaTeX writer | M03+1 |
| Citations (BibTeX) | M03+1 | adapter-bibtex crate | M03+1 |
| Google Docs API compat shim (read-only) | M04 | adapter behind feature flag | M04 |
| Microsoft Graph compat shim (read-only) | M04 | adapter behind feature flag | M04 |
| EU AI Act conformity assessment (HR overlay) | refused at Cedar layer | dedicated tenant-opt-in flow per ADR-DOCS-0005 | M04+ |
| Federation with external Google Docs / Word source | M04+ (coexistence mode) | resolver adapter; long-poll changes | M04+ |
| Public-read URL publishing (post-Notion-style) | M04+ | review at M05 | M05+ |
| Native iOS / Android editor parity (CalDAV-like protocol independence) | M05+ | dependent on Swift / Kotlin SDK CRDT bindings | M05+ |

## Verification

- HG-DOCS gate validates this matrix is consistent with the PRD `§Competitive Benchmark` row.
- gtm-customer-success references this matrix in sales materials; any claim of parity / superiority that diverges is a process violation.
- Bi-annual review re-validates each row against current competitor release notes; new competitor entrants get added.

## References

- ADR-0123 — Hyperscaler maturity claim gate.
- ADR-0126; ADR-0131; ADR-0132; ADR-0133.
- ADR-DOCS-0001 (CRDT — Loro 1.x).
- ADR-DOCS-0002 (block-type system).
- ADR-DOCS-0003 (export pipeline architecture).
- ADR-DOCS-0004 (ACL granularity).
- ADR-DOCS-0005 (AI writing-assist bounds).
- ADR-DOCS-0006 (DOCX import fidelity).
- `microservices/docs/PRD.md` §Competitive Benchmark.
- Google Docs API — `developers.google.com/docs/api`.
- Microsoft Graph (Docs) — `learn.microsoft.com/graph/api/resources/document`.
- Notion API — `developers.notion.com`.
- Coda API — `coda.io/developers`.
- Quip Automation — `quip.com/dev/automation`.
- Dropbox Paper — `developers.dropbox.com/paper`.
- ONLYOFFICE API — `api.onlyoffice.com`.
- Collabora SDK — `sdk.collaboraonline.com`.
- Etherpad — `etherpad.org`.
- HackMD API — `hackmd.io/api/v1`.
- Confluence API — `developer.atlassian.com/cloud/confluence`.
- Obsidian Publish — `help.obsidian.md/Obsidian+Publish/Publish`.
- Roam Research — `roamresearch.com/help`.
- `microservices/workflow-studio/competitor-parity-matrix.md` — sibling reference (CRDT alignment).
- `microservices/calendar/competitor-parity-matrix.md` — sibling reference (pattern).
