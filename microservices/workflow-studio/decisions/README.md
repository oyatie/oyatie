---
doc_class: AdrIndex
microservice: workflow-studio
status: Accepted
date: 2026-05-17
owner: axis-workflow + council-architecture
doc_status: published
---

# workflow-studio service-scoped ADRs

This directory holds **service-scoped** Architecture Decision Records owned by the `workflow-studio` µservice per ADR-0131 §"Canonical folder shape". Repo-wide ADRs continue to live at `/Users/jasonlee/oyatie/docs/decisions/` (e.g., ADR-0105, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145)).

Service-scoped ADRs are numbered `ADR-WS-####` (four-digit, sequential within this directory). The `WS` prefix prevents collision with the repo-wide `ADR-####` series and matches the convention adopted by sibling µservices migrating to ADR-0131 per-microservice flat layout.

## Index

| ADR | Title | Status | Closes PRD Open Question |
|---|---|---|---|
| [ADR-WS-0001](ADR-WS-0001-crdt-library-selection.md) | CRDT library selection — Loro 1.x for native Rust tree-CRDT, WASM-first, AC-06 never-silent-loss invariant defensible | Accepted | Q1 (CRDT library) |
| [ADR-WS-0002](ADR-WS-0002-dsl-canonical-form.md) | DSL canonical form — RFC 8785 JSON Canonicalization Scheme + workflow-studio profile; AC-02 round-trip byte-equality enforced structurally | Accepted | (implicit — AC-02 enforcement) |
| [ADR-WS-0003](ADR-WS-0003-leptos-wasm-substrate.md) | Canvas rendering tier — Leptos 0.7+ browser-WASM with signal-driven reactivity; pure Leptos, no JS-canvas fallback | Accepted | Q2 (WASM canvas) |
| [ADR-WS-0004](ADR-WS-0004-jurisdiction-overlay-renderer.md) | Jurisdiction overlay renderer — hybrid server pre-evaluation + client rendering with Ed25519-signed overlay-decision-bundles | Accepted | (implicit — FR-08/FR-09/FR-16 architecture) |
| [ADR-WS-0005](ADR-WS-0005-ai-copilot-node-generation-bounds.md) | AI-copilot capability tier bounds — T0/T1 intra-µservice by default; T2-cross gated by Cedar + ChangeSet review + 2-person rule | Accepted | Q4 (LLM-assist scope) — broader than the transport question PRD Q4 asks; this ADR also resolves T2 cross-µservice scope |

## Cross-reference policy

- Every service-scoped ADR in this directory MUST reference the repo-wide ADRs it inherits from (e.g., ADR-0131 layout, ADR-0140 Cedar, ADR-0105 layer enum).
- Repo-wide ADRs in `/Users/jasonlee/oyatie/docs/decisions/` MUST NOT depend on service-scoped ADRs in this directory; the dependency direction is one-way (service-scoped depends on repo-wide).
- Service-scoped ADRs may reference each other freely within this directory.
- Supersession of a service-scoped ADR is recorded by adding `superseded_by:` to the old ADR's frontmatter and `supersedes:` to the new ADR's frontmatter; old ADRs are **never deleted** (per the documentation-and-adrs skill).

## Sibling µservice ADR directories

- `microservices/workflow-engine/decisions/` — workflow-engine service-scoped ADRs (executor / spec-store / replay-debugger-backend).
- (Other µservices acquire their own `decisions/` directory at the time they author their first service-scoped ADR.)

## Open Questions still tracked in PRD

After this batch of ADRs the PRD's `Open Questions` table is closed for entries 1, 2, and 4. Remaining entries in `microservices/workflow-studio/PRD.md` §"Open Questions":

| # | Question | Resolution status |
|---|---|---|
| 1 | CRDT library | **Closed** — ADR-WS-0001 |
| 2 | WASM canvas substrate | **Closed** — ADR-WS-0003 |
| 3 | WebSocket gateway substrate | resolved inline (bespoke axum-WS); no separate ADR required |
| 4 | LLM-assist invocation transport (WS stream vs server-side full-draft) | scope: **partially closed** by ADR-WS-0005 (which addresses the broader T0/T1/T2 scope question); transport-only choice resolved inline in IP-008 |
| 5 | Per-pack node-library hot-reload (full vs delta) | resolved inline |
| 6 | Multi-tab editing (single CRDT session vs two) | resolved inline |

## Author + reviewer protocol

Per the documentation-and-adrs skill and ADR-0131:

1. Author a draft ADR under this directory using the structure: Status / Date / Context / Decision / Alternatives Considered (≥3 alternatives) / Consequences.
2. Decision must be concrete (no TODO comments; no deferral within scope).
3. Consequences must list ≥3 downstream impacts.
4. ADR must cross-reference (a) the repo-wide ADRs it inherits from, (b) named industry sources where applicable (RFCs, regulations, standards).
5. ChangeSet review per ADR-0110 with reviewer-agent APPROVE before merge to `dev`.
