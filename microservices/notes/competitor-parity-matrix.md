---
doc_class: CompetitorParityMatrix
title: notes µservice — Competitor Parity Matrix
microservice: notes
status: Accepted
date: 2026-05-17
owner_team: axis-notes + gtm-product-marketing
doc_status: published
---

# Competitor Parity Matrix — notes µservice

## Scope

This matrix benchmarks `oyatie notes` against the major short-form-notes + knowledge-capture incumbents enumerated in the PRD. The goal is to identify parity gaps, sharpen differentiation, and bound minimum-shippable-tier scope.

Legend: ✓ full / ◐ partial / ✗ absent / N/A not-applicable.

## Capture + Edit

| Feature | oyatie notes | Apple Notes | Google Keep | OneNote | Notion | Bear | Obsidian | Standard Notes | Evernote | Roam | Logseq | Joplin | Simplenote | Drafts | Craft | Reflect | Heptabase | Tana | Mem | Saga | NotePlan | Boost |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Markdown body | ✓ | ✗ | ✗ | ◐ | ◐ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ | ◐ | ◐ | ✓ | ✓ | ✓ | ✓ |
| Frontmatter | ✓ | ✗ | ✗ | ✗ | ◐ | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| Plaintext-first | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ | ✗ | ◐ | ✗ | ✗ | ✗ | ◐ | ✓ | ✓ |
| KaTeX / LaTeX math | ✓ | ✗ | ✗ | ◐ | ✓ | ✓ | ✓ | ◐ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ |
| Code blocks with syntax highlighting | ✓ | ✗ | ✗ | ◐ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ |
| Checklist `- [ ]` parse | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Inline image / video embed | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ◐ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

## Organisation

| Feature | oyatie notes | Apple Notes | Google Keep | OneNote | Notion | Bear | Obsidian | Standard Notes | Evernote | Roam | Logseq | Joplin | Simplenote | Drafts | Craft | Reflect | Heptabase | Tana | Mem | Saga | NotePlan | Boost |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Multi-tag | ✓ | ✓ | ✓ | ◐ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Tag-graph | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ | ◐ | ✗ | ✗ |
| Nested tags | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✓ | ✗ |
| Notebook / folder | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ◐ | ✗ | ✓ | ✓ | ✓ |
| Bidirectional `[[wikilinks]]` | ✓ | ✗ | ✗ | ✗ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ◐ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ |
| Daily note auto-create | ✓ | ✗ | ✗ | ✗ | ◐ | ✗ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ |
| Templates | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ |
| Graph view (force-directed) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ | ◐ | ✗ | ◐ | ✗ | ✗ |
| Block-level references | ◐ (note-level + heading anchors) | ✗ | ✗ | ✗ | ✓ | ✗ | ◐ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |

## Privacy + Encryption

| Feature | oyatie notes | Apple Notes | Google Keep | OneNote | Notion | Bear | Obsidian | Standard Notes | Evernote | Roam | Logseq | Joplin | Simplenote | Drafts | Craft | Reflect | Heptabase | Tana | Mem | Saga | NotePlan | Boost |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| E2E encryption (any) | ✓ | ◐ (Lockable subset) | ✗ | ✗ | ✗ | ◐ (subscription) | ✓ (Obsidian Sync) | ✓ (default) | ✗ | ✗ | ✓ (local-first) | ✓ (opt-in) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ (local-first) | ✓ |
| E2E by default (Personal-tier) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ◐ (local) | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ◐ (local) | ✗ |
| Tenant-DEK envelope (Professional-tier) | ✓ | ✗ | ✗ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| AI assist refusal on E2E | ✓ (structural) | N/A | N/A | N/A | N/A | N/A | N/A | ✓ (no AI) | N/A | N/A | N/A | N/A | N/A | N/A | N/A | ✗ (AI on plaintext) | ✗ | ✗ | ✗ (AI on plaintext) | ✗ | N/A | N/A |
| Local-first | ◐ (sync default) | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ | ✓ |

## Web Clipper

| Feature | oyatie notes | Apple Notes | Google Keep | OneNote | Notion | Bear | Obsidian | Standard Notes | Evernote | Roam | Logseq | Joplin | Simplenote | Drafts | Craft | Reflect | Heptabase | Tana | Mem | Saga | NotePlan | Boost |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Chrome MV3 extension | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |
| Firefox AMO | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ | ✓ | ✗ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ |
| Safari Web Extensions | ✓ | ✗ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Microsoft Edge Add-ons | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ | ✓ | ✗ | ✓ | ✗ | ◐ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |

## Collaboration + Sharing

| Feature | oyatie notes | Apple Notes | Google Keep | OneNote | Notion | Bear | Obsidian | Standard Notes | Evernote | Roam | Logseq | Joplin | Simplenote | Drafts | Craft | Reflect | Heptabase | Tana | Mem | Saga | NotePlan | Boost |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Read-only share-link | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ◐ (publish) | ✗ | ✓ | ✓ | ✓ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ |
| Passphrase-gated share | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Real-time collab (CRDT) | ✓ (Loro 1.x; Professional only) | ◐ | ◐ | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ | ✓ | ◐ | ✗ | ✗ | ✗ | ✓ | ✗ | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |

## Import / Export

| Feature | oyatie notes |
|---|---|
| Apple Notes archive (.notes) | ✓ |
| Evernote ENEX | ✓ |
| OneNote .one + .onepkg | ✓ |
| Notion Markdown export | ✓ |
| Bear .bearbk | ✓ |
| Obsidian vault | ✓ |
| Markdown + frontmatter export | ✓ |
| JSON Canonical (RFC 8785) export | ✓ |
| PDF export | ✓ |

## AI Assist

| Feature | oyatie notes | Notion AI | Mem | Reflect | Tana | Saga | Heptabase |
|---|---|---|---|---|---|---|---|
| Summarize | ✓ (T1; Professional only) | ✓ | ✓ | ✓ | ◐ | ✓ | ✗ |
| Tag-suggest | ✓ (T1; Professional only) | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ |
| Link-suggest | ✓ (T1; Professional only) | ✗ | ✓ | ✓ | ✓ | ✗ | ✗ |
| Auto-organise (T2) | ◐ (declared; disabled minimum-shippable-tier) | ✗ | ✓ | ◐ | ✓ | ✗ | ✗ |
| **Refuse AI on E2E content** | ✓ structural | N/A | ✗ | ✗ | N/A | N/A | N/A |

## Mobile + Desktop Native

| Platform | oyatie notes |
|---|---|
| iOS native (Swift + WebKit) | subsequent-to-M02-completion |
| Android native (Kotlin + Compose) | subsequent-to-M02-completion |
| macOS native (Swift + Tauri) | M02 (Tauri); subsequent-to-M02-completion (Swift) |
| Windows / Linux (Tauri) | M02 |
| Web | M02 |

## Differentiation Summary

1. **E2E-default on Personal-tier as a structural property** — only Standard Notes matches; oyatie additionally bridges this with Professional-tier four-eyes admin disclosure for regulated tenants.
2. **AI-refusal on E2E content as a CI-enforced invariant** — unique to oyatie.
3. **Native Workflow + Ontology integration with typed events + entity writes** — unique to oyatie.
4. **OpenSLO-gated rollouts + per-pack regulatory overlays** — unique to oyatie.
5. **Loro CRDT collab + bidirectional `[[wikilinks]]` + graph view + daily-note + template gallery** all in one product — closest is Obsidian Sync + plugin ecosystem but plugin-author dependent; oyatie ships first-class.
6. **Cross-µservice checklist → tasks** — closest is Notion + Reminders integration; oyatie ships as typed Workflow event.

## References

- PRD §Competitive Benchmark.
- ADR-NOTES-0001..0006.
