---
doc_class: ADRIndex
title: translate µservice — Architecture Decision Records
microservice: translate
status: Accepted
date: 2026-05-17
owner_team: axis-translate + council-architecture + council-privacy
doc_status: published
---

# translate µservice ADR Index

Per ADR-0131 §"per-microservice flat layout", each µservice maintains its own `decisions/` folder for ADRs scoped to the µservice. Cross-µservice or repo-wide ADRs continue to live under `docs/decisions/`.

## In-Scope ADRs (this folder)

| ADR | Title | Status | Date |
|---|---|---|---|
| [ADR-TRANSLATE-0001](ADR-TRANSLATE-0001-mt-engine-routing-and-fallback.md) | MT engine routing + fallback | Accepted | 2026-05-17 |
| [ADR-TRANSLATE-0002](ADR-TRANSLATE-0002-translation-memory-and-leverage-model.md) | Translation Memory + leverage model | Accepted | 2026-05-17 |
| [ADR-TRANSLATE-0003](ADR-TRANSLATE-0003-quality-estimation-and-eu-ai-act-bounds.md) | Quality Estimation + EU AI Act bounds | Accepted | 2026-05-17 |
| [ADR-TRANSLATE-0004](ADR-TRANSLATE-0004-data-residency-bound-inference.md) | Data-residency-bound inference | Accepted | 2026-05-17 |
| [ADR-TRANSLATE-0005](ADR-TRANSLATE-0005-document-round-trip-fidelity.md) | Document round-trip fidelity | Accepted | 2026-05-17 |
| [ADR-TRANSLATE-0006](ADR-TRANSLATE-0006-real-time-translation-stream-architecture.md) | Real-time translation stream architecture | Accepted | 2026-05-17 |

## Out-of-Scope ADRs (live under `docs/decisions/`)

- ADR-0056 (Rust clean-architecture BNF v4.1).
- ADR-0105 (13-layer enum).
- ADR-0106 (`application` → `usecase` rename).
- ADR-0117 (pack residency model).
- ADR-0135 (connect super-app expansion — parent ADR for translate's existence).
- ADR-0139 (agentic SLO-gated promotion).
- ADR-0131 (per-microservice flat layout).
- ADR-0132 (product platform + bundle dissolution).
- ADR-0133 (industry-best-practice conformance program).

## Authoring Convention

Per `docs/STANDARDS-AND-TEMPLATES.md` + ADR-0131 + the `documentation-and-adrs` skill:

1. Every ADR has: Context · Decision · Alternatives (≥ 3) · Consequences (≥ 3) · References (named industry sources + legal citations).
2. ADRs use plain markdown with frontmatter (doc_class: AdrSpec).
3. ADRs cite competitor practice + standards (OASIS / LISA / ISO / OWASP / NIST / SLSA).
4. ADRs label each Consequence as `positive` | `negative` | `neutral`.
5. Supersede + amend cycles recorded inline.
6. ADRs name decision deciders + date.

## Cross-µservice References

These ADRs cross-reference:
- `microservices/intelligence/decisions/` (when published) for provider-router patterns.
- `microservices/intelligence/decisions/` (when published) for capability-execution patterns.
- `microservices/observability/decisions/` (when published) for SLO-promotion patterns.

## Lifecycle

- Quarterly council-architecture + council-privacy review.
- New ADR triggered by: new engine vendor adapter; new pack activation; new content-class regulatory classification; new document format; new real-time-stream protocol.
- Supersede via standard "Supersedes ADR-TRANSLATE-####" frontmatter + reciprocal "Superseded by" on the old ADR.
