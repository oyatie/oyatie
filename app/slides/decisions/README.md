---
doc_class: AdrIndex
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + council-architecture
doc_status: published
---

# Slides ADR index

Per-µservice ADRs for the slides product (per ADR-0131 per-microservice flat layout + ADR-0135 dissolution + ADR-0132 single-concern microservices). Each ADR carries ≥ 3 alternatives + ≥ 3 Consequences per `documentation-and-adrs` skill lens and ADR-0133 axis-4 industry-best-practice conformance.

| ID | Title | Status | Date | Resolves |
|---|---|---|---|---|
| [ADR-SLIDES-0001](ADR-SLIDES-0001-crdt-library-selection.md) | CRDT library selection — Loro 1.x (aligned with workflow-studio + docs + sheets) | Accepted | 2026-05-17 | PRD AC-06 never-silent-loss |
| [ADR-SLIDES-0002](ADR-SLIDES-0002-rendering-canvas-substrate.md) | Rendering canvas substrate — Leptos WASM + SVG baseline + canvas-2d/WebGL fallback | Accepted | 2026-05-17 | PRD AC-09 60-fps present-mode |
| [ADR-SLIDES-0003](ADR-SLIDES-0003-export-pipeline-fidelity.md) | Export pipeline fidelity — PPTX round-trippable subset + PDF/A + deterministic MP4 | Accepted | 2026-05-17 | PRD AC-02 round-trip + AC-15 |
| [ADR-SLIDES-0004](ADR-SLIDES-0004-animation-engine-and-reduced-motion.md) | Animation engine + reduced-motion fallback — WCAG 2.2 SC 2.3.3 | Accepted | 2026-05-17 | PRD AC-17 |
| [ADR-SLIDES-0005](ADR-SLIDES-0005-broadcast-mode-and-livekit-reuse.md) | Broadcast-mode + LiveKit reuse via messenger | Accepted | 2026-05-17 | PRD FR-19 + AC-18 |
| [ADR-SLIDES-0006](ADR-SLIDES-0006-ai-design-and-content-generation-bounds.md) | AI-design + AI-content-generation bounds — EU AI Act risk-class | Accepted | 2026-05-17 | PRD AC-16 |
| [ADR-SLIDES-0007](ADR-SLIDES-0007-per-slide-acl-granularity.md) | Per-slide ACL granularity — Cedar named-block refinement | Accepted | 2026-05-17 | PRD AC-08 |
| [ADR-SLIDES-0008](ADR-SLIDES-0008-chart-live-link-to-sheets.md) | Chart-live-link to sheets — eventual consistency + revocation cascade | Accepted | 2026-05-17 | PRD AC-11 + AC-19 |

## Cross-µservice CRDT family

ADR-SLIDES-0001 aligns with:
- `microservices/workflow-studio/decisions/ADR-WS-0001-crdt-library-selection.md` (parent — establishes Loro 1.x choice for canvas-CRDT class µservices)
- `microservices/docs/decisions/ADR-DOCS-0001-*` (sibling — docs collab; Loro alignment)
- `microservices/sheets/decisions/ADR-SHEETS-0001-*` (sibling — sheets collab; Loro alignment)

## Cross-µservice Leptos canvas family

ADR-SLIDES-0002 aligns with:
- `microservices/workflow-studio/decisions/ADR-WS-0003-leptos-wasm-substrate.md` (parent — establishes Leptos WASM for canvas-class µservices)

## Cross-µservice LiveKit reuse

ADR-SLIDES-0005 aligns with:
- `microservices/messenger/decisions/ADR-MSGR-*-huddles-placement.md` (parent — establishes LiveKit deployment in messenger; slides reuses)
