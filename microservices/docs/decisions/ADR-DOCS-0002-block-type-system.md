---
id: ADR-DOCS-0002
title: Block-type system — Notion-style block primitives (block-based); Word-style document-tree rejected
microservice: docs
status: Accepted
date: 2026-05-17
owner: axis-docs + council-design-system
deciders: council-architecture, axis-docs, council-design-system, ops-security
supersedes: []
superseded_by: []
related: [ADR-0105, ADR-0131, ADR-DOCS-0001, ADR-DOCS-0004]
related_artifacts:
  - microservices/docs/PRD.md (§Bounded Contexts block-types BC; AC-02)
  - microservices/docs/IP-005-block-types-kernel-domain.md
purpose: |
  Settle the document-model choice: block-based (Notion-style) vs document-tree
  (Word-style) vs paragraph-based (Google-Docs-style). Closes PRD §"Bounded
  Contexts" block-types BC.
doc_status: published
---

# ADR-DOCS-0002: Block-type system — block-based (Notion-style)

## Status

Accepted — 2026-05-17.

## Context

The docs µservice's `block-types` bounded context governs the document's content model. Three industry models exist:

1. **Block-based** (Notion, Coda, Craft, Roam Research, modern Substack). Each unit of content is a typed block (paragraph, heading, list-item, table, image, embed, code, math, callout, divider, page-break). Blocks have explicit IDs and can be referenced, embedded, ACL'd, and rearranged. The document is a tree (or DAG) of blocks.

2. **Document-tree** (Microsoft Word, OOXML, LibreOffice). Document content is a sequence of paragraphs with formatting runs; structural elements (headings, lists) are paragraph properties. The document is a flat sequence with hierarchy implied by heading-level metadata.

3. **Paragraph-based with named ranges** (Google Docs, Quip). Document is a sequence of paragraphs; comments + suggestions anchor via named ranges (byte offsets). Structural elements are paragraph styles; no explicit IDs per block.

Per ADR-DOCS-0004 (per-block ACL), the docs µservice requires per-block addressability for the load-bearing differentiator vs Google Docs. Per ADR-DOCS-0001 (Loro CRDT), the document state is a CRDT tree where each block needs a stable `TreeID`. Per PRD AC-04, the per-block ACL invariant requires per-block grant evaluation. These three constraints favor a block-based model.

## Decision

Adopt the **block-based model** (Notion-class) for the docs µservice block-types BC. Concretely:

1. **Canonical block primitive set** (per `contracts/proto/docs.proto` `BlockKind` enum):
   - `paragraph` — default text block.
   - `heading_1`, `heading_2`, `heading_3` — section headings; WCAG hierarchy enforced.
   - `ordered_list`, `unordered_list`, `checklist` — list blocks.
   - `table` — rich table block with rows/columns of nested blocks.
   - `image` — image block; alt-text required per WCAG 2.2 AA.
   - `embed` — cross-µservice embed block (workflow-studio / sheets / slides) per embed-resolver BC.
   - `code` — syntax-highlighted code block (tree-sitter).
   - `math` — KaTeX-rendered math block (MathJax fallback).
   - `callout` — accent block (note / warning / tip / info).
   - `divider` — horizontal rule.
   - `page_break` — explicit page break for export.

2. **Each block carries**:
   - `block_id`: stable UUID.
   - `kind`: one of the BlockKind enum.
   - `position`: integer ordering within parent.
   - `text`: string content (paragraph / heading / list-item / code; empty for image / embed / divider).
   - `attributes`: typed map (alignment, language, math notation, etc.).
   - `acl`: per-block ACL per ADR-DOCS-0004.
   - `children`: nested blocks (table cells, list nesting, callout body).

3. **Inline runs** within a block carry styling (bold, italic, strikethrough, underline, code, link). Style is a CRDT-mergeable property.

4. **Schema enforcement**:
   - WCAG 2.2 AA: alt-text required on `image`; heading hierarchy enforced (cannot skip from `heading_1` to `heading_3`).
   - Code-block language tag drawn from a registered list (tree-sitter supported languages).
   - Math-block notation: KaTeX-supported subset by default; MathJax fallback for unsupported macros.
   - Embed-block resolves via embed-resolver BC; never inline raw content.

5. **HTML sanitisation**: any HTML import is sanitised by `ammonia` 4.x with strict allowlist; no inline `<script>`, no event handlers, no `javascript:` URIs, no `data:` URIs except for opt-in image blocks.

6. **Canonical form**: per AC-02, the canonical JSON projection of the block tree is deterministically ordered (children by `position`, attributes by lex-sorted key); this is the seam that makes CRDT-merged state byte-equal across replicas.

## Alternatives Considered

### Alternative A — Document-tree (Microsoft Word / OOXML model)

- **Pros**:
  - Direct OOXML import round-trip without semantic translation.
  - Industry-standard model; legal-document workflows expect this shape.
- **Cons**:
  - **Per-block ACL is impossible**: paragraph properties are not addressable for ACL. A whole-doc ACL is the only granularity, which loses the ADR-DOCS-0004 differentiator vs Google Docs.
  - **CRDT mapping is awkward**: Loro tree CRDT prefers stable per-node IDs; document-tree's paragraph-as-flat-sequence does not naturally provide them.
  - **Embed-resolver targeting is awkward**: an embed must reference a "paragraph offset" which moves with edits; addressing fragility.
  - **Comment + suggestion anchor stability** suffers the same fragility (Hyrum surface #6 in `migration-from-connect.md` documents the legacy connect-docs offset-based anchors that broke on every edit).
- **Rejected reason**: incompatible with per-block ACL (PRD AC-04 differentiator) + stable embed addressing + CRDT-aware anchor stability.

### Alternative B — Paragraph-based with named ranges (Google Docs model)

- **Pros**:
  - Direct Google Docs import compatibility.
  - Comments + suggestions anchor by named ranges; well-understood model.
- **Cons**:
  - **Per-block ACL still requires implicit block boundaries**: would need an in-band marker convention, which is fragile.
  - **Embed-as-typed-block is not natively expressible**: embeds become "object placeholders" anchored to a paragraph offset, which moves with edits.
  - **Math + code blocks** are second-class citizens (typically rendered via Workspace add-ons rather than first-class block kinds).
  - **Notion-class differentiator lost**: tenants who chose oyatie over Google Docs typically wanted Notion-like authoring; paragraph-based forfeits that positioning.
- **Rejected reason**: loses per-block ACL differentiator + Notion-class positioning.

### Alternative C — Flat-text + Markdown semantics (HackMD / Obsidian model)

- **Pros**:
  - Lowest implementation cost.
  - Markdown-first model; technical-writer audience well-served.
  - Direct Markdown round-trip without translation.
- **Cons**:
  - **Cannot natively express**: embeds, callouts, math (without GFM extension), per-block ACL, comments + suggestions.
  - **Loses Notion-class differentiator entirely**.
  - **Targets a smaller audience** (technical writers only).
- **Rejected reason**: insufficiently expressive for the hero-product positioning.

## Consequences

### Architectural

- `oya-docs-block-types-kernel` declares the `Block`, `BlockKind`, `BlockTree`, `InlineStyle`, `BlockAcl` entity types + `BlockSchemaRegistry` port trait.
- `oya-docs-block-types-domain` enforces schema invariants (WCAG hierarchy, alt-text, language tags, math subset).
- `oya-docs-block-types-adapter` wraps `ammonia` HTML sanitiser + KaTeX renderer + tree-sitter syntax highlighter.
- The canonical JSON form is the cross-µservice contract surface (embed-resolver consumes; export-import pipeline emits).

### Downstream impact

1. **ADR-DOCS-0001 CRDT** — block tree maps onto Loro tree; `TreeID` = `block_id` UUID.
2. **ADR-DOCS-0004 ACL** — per-block ACL operates on `block_id`.
3. **ADR-DOCS-0006 DOCX import fidelity** — OOXML→block translation is per `block-types-adapter-pandoc`; fidelity matrix tracks per-OOXML-feature support.
4. **embed-resolver** — embeds are first-class block kind.
5. **export-import pipeline** — Pandoc writers per block kind; PDF rendering per block layout.
6. **SDK** — every SDK exposes `Block` + `BlockKind` types.

### SLOs gaining new dimensions

- `docs.block_schema_validation_failure_total` — Sev-2 alert if non-zero (regression on schema enforcement).
- `docs.wcag_alt_text_missing_total` — Sev-3 alert if non-zero (accessibility regression).

### Risk register

- **Risk**: Block-primitive set proves insufficient post-GA (e.g., users want video blocks, audio blocks, interactive widgets). **Mitigation**: extensible registry; new block kinds added per follow-up ADR-DOCS-XXXX.
- **Risk**: Block-based model is unfamiliar to Word-trained authors. **Mitigation**: per-keystroke conversion shortcuts; sample docs; council-design-system onboarding.

## References

- ADR-0105 (13-layer enum); ADR-0131.
- ADR-DOCS-0001 (Loro CRDT — tree mapping).
- ADR-DOCS-0004 (per-block ACL — addressability requirement).
- ADR-DOCS-0006 (DOCX import fidelity — translation per block kind).
- WCAG 2.2 AA — `w3.org/TR/WCAG22/`.
- ECMA-376 (OOXML).
- CommonMark + GFM (Markdown).
- Notion API + block model — `developers.notion.com/reference/block`.
- ammonia HTML sanitiser — `crates.io/crates/ammonia`.
- KaTeX — `katex.org`.
- tree-sitter — `tree-sitter.github.io`.
