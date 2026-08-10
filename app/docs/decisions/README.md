---
doc_class: ADRIndex
microservice: docs
date: 2026-05-17
owner_team: axis-docs + council-privacy
doc_status: published
---

# docs µservice — service-scoped ADRs

This directory holds ADRs that govern the `docs` µservice exclusively, per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs that govern multiple µservices remain at `docs/decisions/` at the repo root.

Each ADR closes one Open Question (or derived gap) surfaced in `microservices/docs/PRD.md`, in `microservices/docs/PHASE-01-DOCS-FOUNDATION.md`, or in a capability / runbook / threat-model / DPIA artifact under `app/docs/`.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-DOCS-0001](./ADR-DOCS-0001-crdt-library-selection.md) | CRDT library selection — Loro 1.x (aligned with workflow-studio ADR-WS-0001); Yjs/Automerge/hand-rolled rejected | Accepted | 2026-05-17 | PRD §"Open Questions" — CRDT pick (derived from cross-µservice consistency requirement with workflow-studio) |
| [ADR-DOCS-0002](./ADR-DOCS-0002-block-type-system.md) | Block-type system — block-based (Notion-style) primitive set; Word-style document-tree rejected | Accepted | 2026-05-17 | PRD §"Bounded Contexts" block-types BC + AC-02 round-trip byte-equality |
| [ADR-DOCS-0003](./ADR-DOCS-0003-export-pipeline-architecture.md) | Export pipeline architecture — Pandoc 3.x + WeasyPrint default + Chromium-headless opt-in inside gVisor sandbox | Accepted | 2026-05-17 | PRD AC-09 (gVisor sandbox) + AC-10 (PDF/A) + AC-03 (OOXML round-trip) + FR-09 (export) + FR-10 (import) |
| [ADR-DOCS-0004](./ADR-DOCS-0004-acl-granularity-per-block.md) | ACL granularity — per-block (Notion-style); whole-doc + named-range comments only (Google-Docs style) rejected | Accepted | 2026-05-17 | PRD AC-04 (per-block ACL) + FR-07 (per-block visibility) + FR-08 (sharing) |
| [ADR-DOCS-0005](./ADR-DOCS-0005-ai-writing-assist-bounds.md) | AI writing-assist EU AI Act bounds — Annex III-exempt by default; tenant-opt-in conformity assessment when HR-context | Accepted | 2026-05-17 | PRD FR-14 (AI writing-assist) + capabilities/T1+T2 EU AI Act trigger surface |
| [ADR-DOCS-0006](./ADR-DOCS-0006-import-fidelity-policy.md) | DOCX import fidelity tier — best-effort with named edge-case test matrix; strict-round-trip rejected | Accepted | 2026-05-17 | PRD AC-03 (OOXML round-trip ≥ 95%) + FR-10 (import); derived from export-import pipeline architecture |

## Authoring conventions

- ADR ID format: `ADR-DOCS-XXXX` (4-digit, scope-prefixed) per ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision, Alternatives Considered (≥3 per decision; each with Pros/Cons/Rejected reason), Consequences (≥3 downstream impacts), References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at repo root) and sibling µservice ADRs. Cross-µservice citations encouraged where decisions are genuinely paired (e.g., ADR-DOCS-0001 ↔ ADR-WS-0001).
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted → (Superseded by ADR-DOCS-NNNN | Deprecated)`. Never delete; supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #5 Federation with external Google Docs / Word source (coexistence vs migration-only) | Open | subsequent-to-M04-completion ADR; will pair with workflow-engine federation posture |
| #6 Public-read URL publishing | Open | subsequent-to-M04-completion ADR; will pair with content-distribution µservice |

These remain in `microservices/docs/PRD.md` §"Open Questions"; future ADRs land here with sequential IDs.

## References

- ADR-0131 (per-microservice flat layout + service-scoped ADR convention).
- agent-skills documentation-and-adrs SKILL.md — ADR template authority.
- ADR-WS-0001 (workflow-studio CRDT library selection — primary cross-µservice authority for CRDT).
- `microservices/calendar/decisions/README.md` — sibling µservice ADR index pattern.
- `microservices/workflow-studio/decisions/README.md` — sibling µservice ADR index pattern.
- `microservices/mail/decisions/README.md` — sibling µservice ADR index pattern.
