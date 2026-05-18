---
id: ADR-SLIDES-0001
title: CRDT library selection for slides collaborative editing — Loro 1.x
microservice: slides
status: Accepted
date: 2026-05-17
owner: axis-workspace + council-architecture
deciders: council-architecture, axis-workspace, council-design-system, ops-security
supersedes: []
superseded_by: []
related: [ADR-0065, ADR-0105, ADR-0126, ADR-0131, ADR-WS-0001]
related_specs: [/specs/per-microservice-flat-layout.json]
related_artifacts:
  - microservices/slides/PRD.md (AC-06, Open Question 1)
  - microservices/slides/PHASE-01-SLIDES-FOUNDATION.md (IP-005)
  - microservices/slides/threat-model.md (T-T-01)
  - microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml
  - microservices/workflow-studio/decisions/ADR-WS-0001-crdt-library-selection.md
  - microservices/docs/decisions/ADR-DOCS-0001-crdt-library-selection.md (sibling — docs alignment)
  - microservices/sheets/decisions/ADR-SHEETS-0001-crdt-library-selection.md (sibling — sheets alignment)
purpose: Choose the CRDT library backing the slides `real-time-collaboration` bounded context, with the AC-06 "never silent loss" invariant as the load-bearing constraint, and align with the cross-µservice CRDT family (workflow-studio + docs + sheets) per the architectural rationale already established in ADR-WS-0001.
doc_status: published
---

# ADR-SLIDES-0001: CRDT library selection — Loro 1.x

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The slides µservice `real-time-collaboration` bounded context (PRD §"Bounded Contexts") requires a CRDT substrate for real-time multi-user editing of decks. Per PRD §"Tenant Value" Outcome 2 and AC-06, the load-bearing invariant is **"never silent loss"**: two business users editing the same deck concurrently MUST have their non-conflicting edits merged, and conflicting edits MUST surface an explicit conflict UI — last-writer-wins is forbidden.

Deck content is shaped as a tree of slides, each with placeholders, text-boxes, shapes, images, video/audio embeds, charts, tables, and equations. Animations + transitions add per-object metadata. The deck structure is naturally tree-shaped:

```
Deck
├── Slide[i]
│   ├── Placeholder[j]
│   │   ├── TextBox / Shape / Image / ...
│   │   └── animations + transitions
│   └── speaker_notes
└── theme metadata
```

This shape is structurally similar to workflow-studio's tree-shaped node graph (per ADR-WS-0001) and to docs' tree-shaped block hierarchy and sheets' grid-then-formula-graph hierarchy. The workflow-studio µservice already chose **Loro 1.x** for this class of problem after evaluating yrs (Yjs Rust port), Automerge 2.0, bespoke, and OT. The docs and sheets µservices are concurrently authoring the same choice.

This slides-specific ADR establishes Loro 1.x for slides as well, with slides-specific applications + downstream impact captured. It is NOT a derivative-of-ADR-WS-0001 rubber-stamp — slides has its own constraints (broadcast-mode interaction, per-slide ACL CRDT-side effects, animation timing model, chart-live-link cell-range bind) that this ADR articulates.

Performance envelope (PRD §"Performance"):
- Collab cursor sync p99 ≤ 150ms (aligning workflow-studio's 100ms cursor + add latency budget for slide-specific projection).
- CRDT merge p99 ≤ 100ms.
- Up to 10 concurrent collaborators per deck (graceful degradation beyond 20).
- 200,000 active editor sessions per region at XL tier.
- WASM bundle must stay within Editor TTI ≤ 400ms cold p95 budget.

Substrate constraints:
- Rust kernel + domain layers (ADR-0105 13-layer enum); CRDT merge engine lives in `oya-slides-real-time-collaboration-domain` (pure, deterministic).
- Browser-WASM target (per ADR-0065 Leptos WASM frontend, applied to slides per ADR-SLIDES-0002).
- Ephemeral CRDT state in Redis; reconstructable from Postgres on cold-start.
- WebSocket gateway long-lived worker (`-worker` crate) fans out CRDT ops via consistent-hash on `deck_id`.

Slides-specific constraints (different from workflow-studio):
- Per-slide ACL refinement (ADR-SLIDES-0007) means CRDT ops carry `slide_id` and the gateway must evaluate per-slide Cedar permission before applying a delivered op to other peers' views.
- Animation + transition objects have per-frame-budget rendering requirements (60fps invariant per ADR-SLIDES-0002); CRDT projection latency p99 ≤ 30ms is required.
- Chart-live-link binding (ADR-SLIDES-0008) means a CRDT op may include a reference to a sheets cell-range; the projection must NOT inline sheet values (they live in a separate refresh-out-of-band stream).
- Broadcast-mode (ADR-SLIDES-0005) means CRDT state is read-only for audience-viewers during a broadcast session; the gateway must hold a presenter-only-write lease.
- Speaker-notes are part of the CRDT state but MUST NOT cross the broadcast frame boundary (T-I-07).

Operational constraints:
- The CRDT library is supply-chain critical: a vulnerability in merge logic can corrupt every tenant's deck content.
- License must be permissive (MIT / Apache-2.0); copyleft (GPL/AGPL) is incompatible with the closed-source SDK distribution model per `sdk-plan.md`.

## Decision

Adopt **Loro 1.x** (`crates.io/crates/loro`) as the slides CRDT library, with the following constraints:

1. Loro types are wrapped in slides' own `oya-slides-real-time-collaboration-kernel` port traits (`CrdtMergeEngine`, `CrdtState`, `MergeOp`); the library is an implementation detail of the `-domain` crate + backend-qualified `-adapter-loro` crate. Public APIs across BCs and across µservices MUST NOT leak Loro types — this preserves the option to swap the library without breaking the canvas, presentation rest, SDK, or cross-µservice consumers.
2. The Loro tree CRDT (`LoroTree`) backs the deck → slide → placeholder hierarchy; Loro maps (`LoroMap`) back placeholder/text-box/shape attribute dictionaries; Loro lists (`LoroList`) back slide-ordering + content-array structures.
3. CRDT-to-canonical-spec projection deterministically orders Loro nodes by stable `TreeID`s, with map keys lex-sorted at the projection boundary. Per-slide ACL filter applied AT PROJECTION TIME so each subscriber receives only ops + state for slides they're permitted to read/edit.
4. Loro snapshot encoding for Redis persistence (`snapshot()` + `import_snapshot()`); canonical JSON projection ONLY for the deck-spec emission + import/export pipelines (per ADR-SLIDES-0003).
5. Loro version pinning (`^1.0`); major-version upgrade gated by (a) AC-06 property test green, (b) PPTX round-trip subset fidelity drill green, (c) WASM bundle size delta ≤ +50 KB gzip.
6. The slides-side CI lane `oya-governance-collab-no-silent-loss` runs Loro upstream's example test suite + slides' own AC-06 property test (10 concurrent editors, randomized op interleaving + slide-ACL-refinement variants, assertion that every accepted op is reachable from final state OR surfaced as conflict — never silently dropped).
7. Per-session HMAC-SHA-256 over each op envelope (T-T-01 mitigation); HMAC keys held in Redis per-session; rotate on suspected tampering (per `runbooks/collab-conflict-resolution-crdt.md` Step 3a).
8. Animation + transition op-class: ops touching animation/transition objects are CRDT-merged with the same semantic as other ops; replay during present-mode must produce byte-identical frame timing (deterministic Loro projection ordering).
9. Speaker-notes carrying a per-field `data_class = SPEAKER_NOTE` annotation; gateway projection filter MUST exclude speaker-notes from any subscriber whose stream is the broadcast-frame subscriber (audience-view). Enforced at the kernel port boundary.

## Alternatives Considered

### Alternative A — Yjs (via yrs, the Rust port)

Most-deployed CRDT library in production (Notion, Figma, Linear all use Yjs-class systems).

- **Pros**
  - Largest production install base; battle-tested merge semantics over ~7 years.
  - Mature ecosystem: yjs-codemirror, yjs-prosemirror, y-websocket reference server.
  - Native browser-WASM target via yrs.
- **Cons**
  - Yjs's tree representation is built on top of Yjs Maps; tree-rebalance semantics are emulated, not native. Deck reorganization (drag a slide subtree across ordinals) hits this corner case.
  - yrs lags Yjs feature parity by ~2 minor versions historically.
  - Yjs's awareness protocol (cursor/presence) and ops protocol are coupled; for slides we need cursor + presenter-cursor + audience-reaction signals as separate channels — Yjs would need adapter work.
  - Snapshot format is binary; debugging requires Yjs tooling.
- **Rejected reason**: tree-rebalance is a primary slide-reorder + per-slide acl-refine gesture; Yjs's emulated tree adds a class of subtle reparenting bugs. Production-install-base advantage does not outweigh structural-fitness disadvantage. Choosing Yjs would also diverge slides from the cross-µservice CRDT family (workflow-studio + docs + sheets all Loro), creating a maintenance + advisory-monitoring tax.

### Alternative B — Automerge 2.0 (`crates.io/crates/automerge`)

Rewrite-in-Rust of the historic JavaScript-first Automerge.

- **Pros**
  - Academic provenance (Kleppmann et al.).
  - JSON-document fit natively.
  - Byte-canonical encoded changes (`save()` is deterministic for a given history).
  - Apache-2.0 licensed.
- **Cons**
  - No native tree CRDT.
  - WASM bundle size ~2-3x larger than Loro at evaluation time.
  - History-preservation-first design retains every op forever unless explicitly compacted; multiplies storage for long-lived decks. Decks routinely live > 1y and accrete hundreds of edits.
  - Performance for high-frequency op interleaving documented as O(n log n) in op count; Loro's RGA-tree variant is closer to O(log n).
- **Rejected reason**: WASM bundle hits TTI budget; storage cost for long-lived decks (and Postgres deck-spec growth) at slides-scale is prohibitive; cross-µservice family divergence.

### Alternative C — Bespoke Rust CRDT

A slides-authored CRDT targeting exactly the deck/slide/placeholder shape.

- **Pros**
  - Zero external dependency.
  - Could make AC-06 byte-equality structural rather than projection-layer.
  - No third-party version-pinning + upgrade tax.
- **Cons**
  - CRDT correctness proofs are research-grade; bespoke implementations historically take years to reach production stability.
  - No external review surface.
  - Owns the WASM build target end-to-end; loses Loro's WASM-first engineering.
  - Recruiting + onboarding cost across slides' BC fleet.
  - Diverges from the cross-µservice CRDT family — each µservice would own their own bespoke implementation with different bug surfaces.
- **Rejected reason**: build-vs-buy unfavourable. Slides does not differentiate by reinventing CRDT correctness. Differentiation lives in the per-slide ACL, broadcast-mode reuse, present-mode 60fps, AI-content-generation risk-class enforcement.

### Alternative D — Last-writer-wins (no CRDT)

The simplest collaborative-edit pattern.

- **Pros**
  - Trivial implementation.
  - No CRDT library dependency.
  - Familiar to developers (most CMS/admin UIs).
- **Cons**
  - Directly violates AC-06 "never silent loss". Last-writer-wins IS silent loss by definition.
  - Industry consensus (post-2018) is that LWW is unacceptable for collab editing of value-bearing tenant content.
- **Rejected reason**: violates the load-bearing PRD invariant.

### Alternative E — Adopt workflow-studio's collab-crdt crates as a shared library

Re-use `oya-workflow-studio-collab-crdt-{kernel,domain,...}` directly.

- **Pros**
  - Zero new code.
  - Cross-µservice consistency by construction.
- **Cons**
  - Violates the ADR-0131 per-microservice flat layout invariant (slides MUST own its kernel + domain crates).
  - Violates the LEAN-A2 cross-product refusal lane (slides cannot import workflow-studio crates).
  - Slides' CRDT entities (Slide, Placeholder, animation, broadcast-mode lease, per-slide ACL) are semantically different from workflow-studio's (Node, Edge, Parameter, jurisdiction overlay).
- **Rejected reason**: per-µservice ownership is a load-bearing architectural invariant; the Loro library is the shared substrate, not the kernel/domain types built on top of it.

## Consequences

### Architectural

- The slides `real-time-collaboration-domain` crate depends on Loro 1.x via the backend-qualified `-adapter-loro` (per ADR-0105 Amendment 3); the Loro types do NOT leak past the adapter boundary into kernel ports.
- `oya-slides-real-time-collaboration-kernel` declares `CrdtMergeEngine`, `CrdtState`, `MergeOp`, `Conflict`, `EditorSessionStore`, `WebSocketGatewayDispatcher` port traits with no Loro types in the signature.
- Per-slide ACL filter at projection boundary: the gateway dispatcher checks Cedar permission for each subscriber × each delivered op against `slide_id`; ops to non-permitted slides are filtered before fan-out. Defined in `domain` layer + tested via property test.
- Speaker-notes data-class enforcement: kernel port forces `data_class = SPEAKER_NOTE` annotation; projection filter excludes from broadcast-subscriber stream. Tested via T-I-07 property test.
- WASM bundle adds ~250 KB gzip from Loro (same as workflow-studio observation); TTI ≤ 400ms cold p95 budget verified at IP-014 measurement.

### Downstream impact on other µservices and IPs

1. **IP-005 (real-time-collaboration kernel/domain/adapter/adapter-redis/adapter-loro)** — adopts Loro; property tests + per-slide ACL refinement variants + HMAC verification.
2. **IP-006 (real-time-collaboration worker + SDK)** — WebSocket gateway with consistent-hash on `deck_id`; tenant SDKs MUST NOT expose Loro types directly — `CrdtOp` envelopes only.
3. **IP-010 (broadcast-mode)** — uses CRDT-state read-only stream for audience-view; speaker-notes filter enforced at projection.
4. **IP-013 (acl + comments + version-history)** — per-slide ACL refinement applied at CRDT projection boundary; restore from version-history rebuilds Loro snapshot deterministically.
5. **sibling docs + sheets + workflow-studio µservices** — share the Loro 1.x version pin and advisory feed.
6. **observability µservice** — slides-specific SLIs (`oya_slides_collab_op_published_total`, `oya_slides_collab_silent_loss_attempt_total`, `oya_slides_collab_conflict_rate`) tagged `crdt_library=loro`.

### SLOs gaining new dimensions

- `slides.collab_merge_latency_p99` — tagged with `crdt_library=loro`.
- `slides.collab_no_silent_loss_count` — Sev-1 alert if non-zero in any 24h window.
- `slides.crdt_wasm_bundle_size_bytes` — release-gated; alarms if Loro upgrade bumps it past the budget set in IP-014.
- `slides.crdt_projection_latency_p99` — slides-specific (60fps invariant); alarm if > 30ms.

### Supply-chain + security

- Loro added to `cargo deny` allowlist with explicit version pin (aligned with workflow-studio + docs + sheets).
- GitHub Security Advisories + RustSec advisory database monitoring.
- Major-version Loro upgrade gated on: (a) AC-06 property test green; (b) PPTX round-trip subset fidelity drill green; (c) WASM bundle size delta ≤ +50 KB gzip; (d) cross-µservice family consensus (workflow-studio + docs + sheets must also upgrade in coordinated rolls).
- Per-session HMAC over op envelope; rotation on Sev-1 alarm per `runbooks/collab-conflict-resolution-crdt.md` Step 3a.

### Risk register

- **Risk**: Loro 1.x breaking-change release. **Mitigation**: pin `^1.0`; coordinated cross-µservice upgrade.
- **Risk**: Loro maintainer-attrition. **Mitigation**: kernel port-trait wrapper makes library swap a contained refactor; this ADR will be superseded rather than allowed to rot.
- **Risk**: WASM bundle creep across Loro minor versions. **Mitigation**: bundle-size SLO + release gate.
- **Risk**: Per-slide ACL refinement filter performance at large deck size. **Mitigation**: pre-compute per-(subscriber × slide) permission cache; invalidate on ACL change event.
- **Risk**: Speaker-notes leak via incorrect projection filter. **Mitigation**: T-I-07 property test; dedicated `oya-governance-broadcast-speaker-notes-isolation` lane.

## References

- PRD `microservices/slides/PRD.md` §"Tenant Outcome 2", AC-06.
- `microservices/slides/PHASE-01-SLIDES-FOUNDATION.md` IP-005, IP-006.
- `microservices/slides/threat-model.md` T-T-01.
- `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`.
- ADR-WS-0001 (parent — workflow-studio CRDT decision).
- ADR-0105 (13-layer + backend-qualified adapters Amendment 3).
- ADR-0126 (Connect dissolution).
- Loro — `loro.dev`, `github.com/loro-dev/loro`.
- yrs (Yjs Rust port) — `github.com/y-crdt/y-crdt`.
- Automerge 2.0 — `automerge.org`.
- Shapiro, M. et al. (2011), "Conflict-free Replicated Data Types," INRIA RR-7687.
- Preguiça, N. (2018), "Conflict-free Replicated Data Types: An Overview," arXiv:1806.10254.
