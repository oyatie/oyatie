---
doc_class: ADRIndex
microservice: notes
date: 2026-05-17
owner_team: axis-notes + council-privacy + council-architecture
doc_status: published
---

# notes µservice — service-scoped ADRs

This directory holds ADRs that govern the `notes` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or load-bearing decision) surfaced in `microservices/notes/PRD.md`, in `microservices/notes/PHASE-01-NOTES-FOUNDATION.md`, or in a policy / runbook / threat-model artifact under `microservices/notes/`.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-NOTES-0001](./ADR-NOTES-0001-e2e-encryption-default-personal-tier.md) | Personal-tier E2E ON by default; Professional-tier tenant-DEK envelope ON by default with admin policy override | Accepted | 2026-05-17 | PRD NFR §Security + DCI-03 + paired with ADR-MSGR-0002 |
| [ADR-NOTES-0002](./ADR-NOTES-0002-bidirectional-link-and-graph-storage.md) | Postgres adjacency + materialised backlink table on server; client-side WebGL force-directed render | Accepted | 2026-05-17 | PRD FR-04 + FR-14 |
| [ADR-NOTES-0003](./ADR-NOTES-0003-crdt-library-for-optional-collab.md) | Loro 1.x as canonical CRDT for opt-in Professional-tier collab; E2E-tier collab structurally refused | Accepted | 2026-05-17 | PRD FR-17; sibling-aligned with ADR-WS-0001 |
| [ADR-NOTES-0004](./ADR-NOTES-0004-search-architecture-respecting-e2e.md) | Meilisearch 0.10.0 LTS Professional-tier server-side; client-side encrypted-inverted-index in IndexedDB / SQLite with per-note token-bloom-filters for Personal-tier | Accepted | 2026-05-17 | PRD Open Question #1 + FR-13 |
| [ADR-NOTES-0005](./ADR-NOTES-0005-ai-assist-bounds-and-e2e-invariant.md) | AI assist (T0/T1/T2) over Personal-tier E2E STRUCTURALLY IMPOSSIBLE; over Professional-tier requires tenant-admin opt-in + transparency + audit-chain seal | Accepted | 2026-05-17 | PRD NFR §Security + FR-21 + AC-03 |
| [ADR-NOTES-0006](./ADR-NOTES-0006-portable-export-and-import-format.md) | Markdown + YAML frontmatter canonical export; JSON Canonical (RFC 8785) byte-identical roundtrip; six source-format imports at minimum-shippable-tier (Apple Notes, Evernote ENEX, OneNote, Notion, Bear, Obsidian vault) | Accepted | 2026-05-17 | PRD FR-15 + FR-16 + AC-07 + AC-08 + AC-16 |

## Authoring conventions

- ADR ID format: `ADR-NOTES-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at repo root) and sibling µservice ADRs (e.g., `ADR-MSGR-0002` referenced from `ADR-NOTES-0001` as paired privacy posture). Cross-µservice citations are encouraged where decisions are genuinely paired.
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-NOTES-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #1 client-side encrypted-search design | Closed | ADR-NOTES-0004 (encrypted-inverted-index + token-bloom-filters) |
| #2 AI tag-suggest semantics (multi-label classification vs retrieval) | Open | successor-IP capability ADR |
| #3 web-clipper extension distribution (all browsers day-1 vs staged) | Open | successor-IP IP |
| #4 daily-note timezone authority (user-local vs tenant-default) | Open | UX research successor-IP |
| #5 public-share-link OG metadata leakage policy | Open | successor-IP policy decision |
| #6 mobile-platform offline-edit conflict semantics (LWW vs CRDT-merge) | Open | successor-IP ADR |

All PRD Open Questions are either closed by an ADR in this directory or scheduled for successor-IP. Future ADRs land here with sequential `ADR-NOTES-XXXX` IDs.
