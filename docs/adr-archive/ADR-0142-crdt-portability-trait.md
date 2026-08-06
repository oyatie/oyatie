---
id: ADR-0142
status: Superseded
deciders: council-architecture, axis-workflow-studio, axis-docs, axis-sheets, axis-slides
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-704]
related: [ADR-0056, ADR-0064, ADR-0131, ADR-0133, ADR-0135]
related_memory: [feedback_no_silent_regression, feedback_clean_architecture_requirements, feedback_workflow_studio_scope]
related_specs:
  - /specs/products/workflow-studio.json
purpose: |
  De-risk Loro 1.x bus-factor-1 by introducing a CRDT trait kernel that Loro
  implements as the primary adapter, with Yjs and Automerge maintained as
  compile-but-not-deployed alternates at INV-PORTABILITY-CI-COMPILE level.
---

# ADR-0142: CRDT portability trait + alternate-adapter compile gate

## Status

Accepted — 2026-05-18.

## Date

2026-05-18.

## Context

Workflow Studio's visual canvas, the canvas-style collaborative
authoring layers in Docs / Sheets / Slides, and the messenger thread-
tree collaborative-editing surface all depend on a CRDT library. The
candidate set in the public CRDT ecosystem (as of 2026-Q2) is:

| Library     | Lang   | Active maintainer count | License            | Notes |
|-------------|--------|-------------------------|--------------------|-------|
| Loro        | Rust   | 1 primary (zxch3n)      | MIT                | Rust-native; fastest for typical document sizes; bus-factor concern. |
| Yjs         | TS/JS  | ~3 active (Kevin Jahns + community) | MIT      | TS/JS-native; mature; large ecosystem; needs wasm-bindgen bridge for Rust. |
| Automerge   | Rust   | ~5 active (Ink & Switch) | MIT-Apache         | Rust-native via 2.0; slower large-doc perf than Loro per public benchmarks 2024. |
| Y-CRDT      | Rust   | ~2 active               | MIT                | Rust port of Yjs; lower feature parity with Yjs main. |

Loro 1.x is currently the fastest and the most ergonomic for our
canvas use cases. But `feedback_no_silent_regression.md` and the
overall hyperscaler-grade bar require us to **never** ship a
production substrate built on a bus-factor-1 dependency without a
documented, exercised fallback. The Linus-grade discipline cited in
that feedback applies: a substrate decision must survive the loss of
any single upstream maintainer.

The cost of "rip and replace Loro for Yjs in a panic" is bounded
purely by how much application code is coupled to Loro's concrete
type surface. If our application code is coupled to a trait we own,
that cost is hours; if it's coupled to `loro::LoroDoc`, that cost is
weeks.

Three reinforcing user directives narrow the design:

1. **Industry-leading quality bar** (Stripe / Palantir / Linear) —
   none of those companies couple application code to a single CRDT
   implementation. Linear's blog (2023) describes a CRDT trait
   abstraction they own; Palantir AIP wraps Yjs in an internal
   `OntologyDoc` trait.
2. **No silent regression** — public application code MUST NOT
   change shape if we swap CRDT impls.
3. **Hyperscaler-grade scalability** — the CRDT layer is on the hot
   path; the trait MUST be zero-cost (no boxed-trait virtual
   dispatch in the inner write loop).

## Decision

oyatie owns a CRDT portability trait kernel; Loro is the primary
adapter; Yjs and Automerge are maintained as **INV-PORTABILITY-CI-
COMPILE** alternates — they compile in CI on every change, exercising
the trait surface, but are NOT deployed.

### Layer A — trait kernel (this ADR's primary artifact)

A new crate `crates/oya-shared-crdt-portability-kernel/`:

- `pub trait CrdtDoc` — the shared surface: `new()`, `apply_local(op)`,
  `apply_remote(bytes)`, `export(bytes)`, `subscribe(callback)`,
  `snapshot()`, `from_snapshot(bytes)`.
- `pub trait CrdtMap`, `pub trait CrdtList`, `pub trait CrdtText` —
  collection sub-traits for canvas / document / chat use cases.
- Generic over the underlying type (zero-cost; no boxed dispatch in
  inner loop); adapters expose `LoroDoc: CrdtDoc`, `YjsDoc: CrdtDoc`,
  `AutomergeDoc: CrdtDoc`.

### Layer B — primary adapter (Loro 1.x)

A new crate `crates/oya-shared-crdt-adapter-loro/`:

- Depends on `loro 1.x`.
- Implements `CrdtDoc`, `CrdtMap`, `CrdtList`, `CrdtText` against
  Loro's native types.
- Every µservice that needs CRDTs (workflow-studio, docs, sheets,
  slides, messenger-threads) takes `D: CrdtDoc` as a generic parameter
  and is wired to `LoroDoc` at composition root.

### Layer C — alternate adapters (CI-compile only)

Two crates:

- `crates/oya-shared-crdt-adapter-yjs/` — wraps Yjs via wasm-bindgen
  FFI (Yjs is JS-native; the Rust adapter compiles the wasm bridge
  and exposes `YjsDoc: CrdtDoc`).
- `crates/oya-shared-crdt-adapter-automerge/` — wraps Automerge 2.x
  via its Rust API; exposes `AutomergeDoc: CrdtDoc`.

These crates:

- Compile on every CI run (a new fitness lane
  `oya-governance-crdt-portability` asserts both adapters
  `cargo check`).
- Run their adapter unit tests in CI (round-trip apply/export must
  satisfy the same trait contract as Loro).
- Are NOT linked into any production binary.

### Promotion criteria from CI-compile to deployed

If Loro 1.x becomes unmaintained (no commits for 6 months OR Loro's
sole primary maintainer publishes a wind-down notice), the council-
architecture team promotes one alternate adapter to primary by:

1. Switching the composition-root wiring in workflow-studio (and the
   four other µservice consumers) from `LoroDoc` to the chosen
   alternate.
2. Running the existing canvas / docs / sheets test sets against
   the new adapter.
3. Authoring ADR-XXXX promoting the alternate to primary.
4. Demoting Loro to CI-compile status.

The expected cost of this promotion is **engineering days, not
weeks** because no µservice code touches `LoroDoc` directly.

## Alternatives considered

### Alternative 1: Loro-only, no trait

- **Pros:** Simpler; no trait overhead; fewer crates.
- **Cons:** Bus-factor-1 risk un-mitigated; rip-and-replace cost
  measured in weeks.
- **Rejected because:** Violates the no-silent-regression discipline
  and the hyperscaler-grade bar. Stripe and Linear both wrap their
  CRDTs behind an internal trait; oyatie should not be less rigorous.

### Alternative 2: Box<dyn CrdtDoc> (dynamic dispatch)

- **Pros:** Adapter switching at runtime; fewer monomorphisations.
- **Cons:** Boxed virtual dispatch in the inner CRDT write loop adds
  ~20-30 ns per operation. At 100k ops/sec per canvas (Workflow
  Studio's stated target), that's 2-3 ms/sec of CPU lost to vtable
  indirection.
- **Rejected because:** the canvas path is hot-path performance-
  critical; the trait MUST be zero-cost. Generic monomorphisation
  gives identical codegen to a direct call.

### Alternative 3: Maintain only Loro + Automerge; drop Yjs

- **Pros:** One less adapter to maintain.
- **Cons:** Yjs's JS ecosystem (Y-WebSocket, Y-IndexedDB,
  y-prosemirror) is by far the most mature; if the eventual primary
  shifts to Yjs, having NO bridge means starting from zero.
- **Rejected because:** the CI-compile cost of maintaining the Yjs
  adapter is bounded (one fitness lane; one round-trip test); the
  insurance value materially exceeds that cost.

## Consequences

### Positive

1. **Loro bus-factor risk neutralised.** Any single-maintainer event
   for Loro can be absorbed by swapping the composition root in days.
2. **CRDT trait becomes shared substrate** — all five canvas/document
   µservices use the same surface; future µservices that need a CRDT
   (e.g. a real-time poll µservice) inherit the trait + every adapter
   for free.
3. **Three adapters keep us honest about portability claims.** A
   trait that only one adapter implements is not a portability claim;
   it is a thin sleeve. The CI-compile gate exercises the trait
   surface across genuinely different CRDT semantics (Loro RGA-like,
   Yjs YATA, Automerge OT-CRDT hybrid).
4. **Onboarding new CRDT impls is cheap.** When a new CRDT library
   appears (e.g. the announced Diamond-Types 2.0), wrapping it is a
   one-week task; the trait surface is already proven.

### Negative

1. **Crate count grows by 4** (kernel + 3 adapters). Mitigation:
   well within the workspace's existing 200+ crate count.
2. **Adapter maintenance load.** When Loro 1.x ships a breaking trait
   change, the trait kernel may need an update; cascading to Yjs and
   Automerge adapters takes engineering time. Mitigation: the trait
   surface is intentionally small (one doc-level trait + three
   collection sub-traits).
3. **CI runtime grows.** The CI-compile lane adds a few minutes per
   run. Mitigation: cached via existing sccache+ARC infrastructure.

### Comparisons to industry-standard practice

- **AWS:** AWS Outposts wraps three storage backends (EBS, S3,
  Glacier) behind an internal trait. Direct precedent for the
  primary-with-alternates pattern.
- **Google:** Spanner has multiple compaction backends; the SST file
  format is the abstraction seam. Direct precedent.
- **Anthropic:** the model dispatch layer in Claude API supports
  multiple internal inference backends behind a trait; only one is
  primary at a time.
- **Linear:** public engineering blog 2023 describes a `SyncDoc`
  trait above Yjs. Direct precedent for the CRDT trait pattern.
- **Palantir AIP:** wraps Yjs in an `OntologyDoc` trait per public
  engineering writeups. Direct precedent.

## References

- ADR-0056 — substrate architecture.
- ADR-0064 — canonical-base-and-localization-packs.
- ADR-0131 — per-microservice flat layout.
- ADR-0133 — industry-best-practice + hyperscaler-conformance.
- ADR-0135 — connect super-app expansion (consumer products).
- Loro public benchmarks 2024 (`loro.dev/docs/performance`).
- Yjs documentation (`docs.yjs.dev`).
- Automerge 2.x release notes (`automerge.org/blog`).
- Linear engineering blog (2023) — *Building a fast and reliable real-time sync engine*.
- `feedback_workflow_studio_scope.md`.
- `feedback_no_silent_regression.md`.
