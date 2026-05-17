---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-workflow-studio
microservice: workflow-studio
status: Accepted
sales_segment: hero-product
tier: external-facing
milestone_first_ship: M03-studio-preview
bominal_source:
  - ADR-0164   # Workflow canonical spec format
  - ADR-0103   # Workflow hexagonal migration
  - ADR-0037   # Plugin substrate (node-library scaffolding)
related_adrs: [ADR-0056, ADR-0065, ADR-0103, ADR-0105, ADR-0106, ADR-0110, ADR-0123, ADR-0130, ADR-0131, ADR-0132, ADR-0133, ADR-0140]
related_specs: [/specs/products/workflow-studio.json, /specs/products/workflow.json, /specs/per-microservice-flat-layout.json]
related_unbundle_adr: ADR-0131
unbundle_sibling: microservices/workflow-engine/
date: 2026-05-17
owner_team: axis-workflow + council-design-system
doc_status: published
---

# PRD-workflow-studio: Workflow Studio — Visual Editor + DSL Frontend

## Purpose

The `workflow-studio` µservice is oyatie's **visual workflow authoring product** — the n8n-class first hero product per `feedback_workflow_studio_scope.md`. Studio is the **end-user surface** of the workflow product unbundle (ADR-0131); its sibling `workflow-engine` µservice owns durable execution. Studio owns: the drag-drop visual canvas, the canonical JSON DSL (workflow_spec.v1.json) round-trip, collaborative multi-user editing, per-pack node libraries, jurisdiction-overlay view-switching, the live debugger frontend, LLM-assist authoring, per-seat license-gate enforcement, and the editor session state.

Studio is **NOT a substrate**. It is a tenant-facing product surface with five distinct user personas (business power user, developer, vertical specialist, agentic developer role, external customer). The visual canvas is the largest Leptos application in oyatie (per ADR-0065 Rust-WASM SSR + browser-WASM hybrid). The canonical source of truth is the `workflow_spec.v1.json` document; the visual canvas derives from the spec, never vice-versa (per Bominal ADR-0164 inherited verbatim).

This µservice operates at the **application** layer of the 12-layer Workflow + Ontology architecture (per `feedback_workflow_objectgraph_adapter_layer.md`): Studio consumes ontology object-type descriptors for typed node configuration; emits workflow_spec.v1 documents to the engine; bridges to foundry-providers for LLM-assist; routes through tenancy for per-seat licensing; runs in the application µservice's hosting shell.

This µservice inherits Bominal ADR-0164 (canonical spec format) verbatim. Studio binds to the same spec format the engine consumes — round-trip byte-equality is the load-bearing invariant. Visual edits emit the spec; spec loads produce the same visual. Anti-pattern `visual_model_above_spec_model` is detected by the `oya-foundry-fitness-workflow-spec-roundtrip` CI lane.

This µservice is **shared substrate AND hero product** simultaneously per `feedback_workflow_studio_scope.md`: the visual canvas + DSL emitter/loader are shared substrate consumed by every workflow-aware oyatie product (Connect, Foundry-eval, healthcare workflows, supply-chain workflows); the editor shell is end-user product packaged as the Studio brand.

## Tenant Value

- **Tenant Outcome 1 — Time to first valid workflow under 15 minutes.** Business power user opens Studio, drags 3-5 nodes, sees inline validation + policy preview, saves canonical spec. No code; no terminal; no documentation reading required. Time-to-success target per `/specs/products/workflow-studio.json` §user_experience.
- **Tenant Outcome 2 — TTI under 2 seconds cold load (CDN cached).** Editor opens fast even for tenants in single-region packs; CDN edge cache for spec schema, node library, and design-system primitives; WASM bundle split per route; progressive component loading. n8n parity (n8n claims TTI ≤ 3s typical).
- **Tenant Outcome 3 — Round-trip byte-equality 100% at GA.** Visual edit → emit spec → load spec → render visual → emit again produces byte-identical output. Tenant developers can mix visual + git-PR workflows without surprises; spec author can hand-edit JSON, load in Studio, save, and verify their hand-edit survived.
- **Tenant Outcome 4 — Collaborative editing without silent loss.** Two business users editing same workflow definition simultaneously: CRDT merge applies non-conflicting edits; conflicting edits surface explicit conflict UI; no last-writer-wins. Verified by AC-06.
- **Tenant Outcome 5 — Per-pack node libraries; per-jurisdiction overlay views.** Healthcare tenants in pack-us-healthcare see HIPAA-aware nodes + PHI data-class markers; financial tenants in pack-kr-fss see KR-FSS-specific compliance nodes; same canonical spec format, different overlay rendering.
- **Tenant Outcome 6 — LLM-assist authoring (post-stable).** Tenant prose ("when a sales lead exceeds $50k, route to senior AE and notify Finance") drafts a candidate spec; engine validates; opens in editor for human approval. Bridges to foundry-providers; tenant LLM choice respected.
- **Internal Outcome 7 — DSL backbone for every workflow-aware product.** The canonical workflow_spec.v1.json format is shared across Workflow Studio (authoring), Workflow Engine (execution), Foundry-eval (capability orchestration), and per-domain workflow products. Studio is the round-trip authority.
- **Internal Outcome 8 — Replay-debugger frontend.** Live + historical run debugger renders the engine's replay-debugger-backend stream; tenant operators step through state transitions in the same visual canvas they authored in.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | business power user | to drag nodes onto a canvas and connect them with edges | I can author a workflow without writing code | visual-canvas | Must |
| FR-02 | business power user | to click a node and configure its parameters in a side panel | I can specify what each step does | visual-canvas | Must |
| FR-03 | developer | to view the canonical JSON DSL alongside the visual canvas | I can hand-edit when needed | visual-canvas | Must |
| FR-04 | Studio | to emit a `workflow_spec.v1.json` document when the user saves | engine can durably register the spec | dsl-emitter | Must |
| FR-05 | Studio | to load a `workflow_spec.v1.json` document and render the visual canvas semantically identically | spec-first authoring is preserved | dsl-loader | Must |
| FR-06 | Studio | to round-trip: load spec → render → emit → produce byte-identical output | round-trip byte-equality invariant | dsl-emitter + dsl-loader | Must |
| FR-07 | two users | to edit the same workflow definition concurrently with CRDT merge | no silent loss; explicit conflict on overlap | collab-crdt | Must |
| FR-08 | tenant operator | to switch jurisdiction overlay (kr / eu / us-hc) in editor | I see jurisdiction-resolved view | jurisdiction-overlay-renderer | Must |
| FR-09 | tenant operator | to preview the Cedar policy impact of my spec change before save | I'm not surprised by post-deploy policy denial | visual-canvas | Must |
| FR-10 | tenant operator | to start a debugger session and step through a live or historical run | I can diagnose stuck runs visually | replay-debugger-frontend | Must |
| FR-11 | Studio | to load per-pack node libraries (Agentic / Dev / Business / Healthcare / Supply-Chain / Delivery) | tenant sees their relevant domain | node-library-registry | Must |
| FR-12 | tenant developer | to draft a workflow via prose; LLM emits candidate spec; I review | low-floor LLM-assist authoring | LLM-assist-bridge | Should (GA) |
| FR-13 | tenancy | to enforce per-seat licensing via Cedar at editor open | tenant honors their seat count | license-gate-cedar | Must |
| FR-14 | Studio | to persist edit buffer locally during network disconnect | resume without loss on reconnect | visual-canvas | Must |
| FR-15 | Studio | to expose a diff/PR UI for git-backed definitions | developer workflow integration | visual-canvas | Should (stable) |
| FR-16 | Studio | to render data_class markers on PII / PHI / SECRET-class fields | author sees data sensitivity before save | visual-canvas | Must |
| FR-17 | tenant operator | to author signals (mid-run async input) visually | signal-driven workflows are authorable | visual-canvas | Should (GA) |
| FR-18 | tenant developer | to export evidence of authored definitions for audit | compliance posture verifiable | visual-canvas | Should (stable) |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Editor TTI cold (CDN-cached) | 1s | 2s | 5s | GA target; per `/specs/products/workflow-studio.json` §metrics |
| Save round-trip (emit → engine spec-store → ack) | 80ms | 200ms | 500ms | stable; GA target 100ms p99 |
| Round-trip byte-equality rate | — | — | — | 100% at GA per AC-02 |
| Collab CRDT merge | 30ms | 100ms | 250ms | sub-100ms p99 GA |
| LLM-assist response (full draft) | 1.5s | 3s | 8s | depends on foundry-providers LLM choice; GA p99 ≤ 3s |
| Node-library load (per pack) | 200ms | 500ms | 1s | from CDN cache |
| Spec schema validation in browser | 10ms | 50ms | 200ms | client-side; large specs trigger server-fallback |
| Spec diff render | 50ms | 200ms | 500ms | for ≤ 500-line spec diff |
| Replay-debugger step rendering | 20ms | 100ms | 300ms | from live engine WebSocket stream |
| WebSocket gateway round-trip | 10ms | 50ms | 200ms | collab + debugger streaming |
| Active editor sessions per region | — | 100,000 | — | XL tier; horizontal scale via session sharding |
| Concurrent collab merges per definition | — | 10 | — | beyond 10 users on same doc: degraded UX |

### Security

- OIDC tenant-scoped at every REST/WebSocket entry; Studio refuses opens without resolvable tenant identity.
- Per-seat license-gate Cedar fragment enforced at editor open; refusal emits `studio_per_seat_license_denied` audit row.
- Strict CSP (`default-src 'self' https://cdn-<pack>.oyatie.dev; script-src 'self' 'wasm-unsafe-eval' 'nonce-<random>'`) — no inline scripts except WASM bootstrap nonce; no eval.
- XSS-free architecture: spec body fields rendered via virtual-DOM text nodes; never `innerHTML`. Anti-pattern `per_tenant_branding_mid_render` is forbidden per `/specs/products/workflow-studio.json` §anti_patterns.
- Per-tenant CDN cache key: CDN partitions cache by `(tenant_hash, pack, version)`; no cross-tenant cache pollution.
- WebSocket auth: OIDC token validated at WS upgrade; tenant binding rebound at WS message dispatch (server cannot trust client-supplied tenant_id mid-stream).
- LLM-assist content never trusted: spec emitted by LLM-assist passes through full schema + policy + signature pipeline before save. Anti-pattern: bypassing the validation pipeline for LLM-assist drafts.
- Node-library supply-chain: per-pack node libraries are signed (Ed25519); Studio refuses to load tampered libraries; revocation propagation ≤ 60s.
- Per-tenant editor session isolation: session state in Postgres scoped by tenant_id (Citus partition); cross-tenant session access forbidden by RLS + Cedar.
- WASM bundle integrity: subresource-integrity (SRI) hashes for every WASM chunk in HTML; mismatch triggers refuse-to-load with audit row.

### Audit + Compliance

- Every save emits a `definition_saved` audit-chain seal (Ed25519); seal includes `(tenant_id, spec_id, version_sha, author_identity, parent_version_sha, timestamp)`. Per Bominal ADR-0028.
- Editor session events (open, save, conflict, license-gate, jurisdiction switch) emitted to engine event-bus as typed events per `contracts/asyncapi/workflow-studio-events.yaml`.
- Per-tenant `jurisdiction_code` enforced via overlay renderer; cross-pack collab forbidden.
- LLM-assist call: foundry-providers tenant binding inherited; LLM choice + prompt + completion archived for 90d for audit.
- Per-seat license events emitted: `license_gate_emitted{tenant, principal, seat_count_used, seat_count_limit, decision}`.

### Availability + SLO

- Editor REST availability target: 99.95% monthly (GA); 99.9% (stable); 99.5% (preview).
- WebSocket gateway availability: 99.9% monthly (collab + debugger streams).
- LLM-assist availability: 99.5% monthly (acceptable degradation; Studio works without LLM-assist).
- RTO ≤ 30s for editor-rest; RPO ≤ 1s for editor session state.
- Self-observability: Studio emits its own SLO via observability µservice; burn-rate alarms feed Grafana OnCall.

### Data residency

- Editor session state, spec drafts, collab CRDT state, and per-seat license attribution inherit the tenant's `jurisdiction_code` per ADR-0117. Postgres + Redis are per-pack region-pinned.
- CDN static assets are global (no PII; spec schema + node library descriptors + WASM bundles); per-pack CDN edge keys segregate tenant-rendered content where applicable.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), Studio uses: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-redis`, `adapter-cdn`, `adapter-leptos-wasm`, `rest`, `worker` (collab-server WebSocket gateway), `sdk`, `app`. Browser-WASM artifacts compiled from the `app` layer per ADR-0065.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `visual-canvas` | `oya-workflow-studio-visual-canvas-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm,rest,sdk,app}` | Drag-drop canvas, node/edge primitives, config panel, diff viewer, replay timeline | `Canvas`, `Node`, `Edge`, `Selection`, `ViewportState` |
| `dsl-emitter` | `oya-workflow-studio-dsl-emitter-{kernel,domain,usecase,api,adapter,sdk}` | Visual → canonical workflow_spec.v1.json emitter | `EmitContext`, `EmittedSpec`, `EmitDiagnostic` |
| `dsl-loader` | `oya-workflow-studio-dsl-loader-{kernel,domain,usecase,api,adapter,sdk}` | workflow_spec.v1.json → visual canvas loader | `LoadContext`, `LoadedDefinition`, `LoadDiagnostic` |
| `collab-crdt` | `oya-workflow-studio-collab-crdt-{kernel,domain,usecase,api,adapter,adapter-redis,worker,sdk}` | CRDT state, merge logic, conflict surfacer; WebSocket-driven collab server | `CrdtState`, `MergeOp`, `Conflict`, `EditorSession` |
| `node-library-registry` | `oya-workflow-studio-node-library-registry-{kernel,domain,usecase,api,adapter,adapter-cdn,rest,sdk,app}` | Per-pack node library catalog; signed library distribution | `NodeLibrary`, `NodeDescriptor`, `LibrarySignature` |
| `jurisdiction-overlay-renderer` | `oya-workflow-studio-jurisdiction-overlay-renderer-{kernel,domain,usecase,api,adapter}` | Jurisdiction-aware visual diff + overlay resolver | `Jurisdiction`, `Overlay`, `ResolvedView` |
| `replay-debugger-frontend` | `oya-workflow-studio-replay-debugger-frontend-{kernel,domain,usecase,api,adapter,sdk}` | Renders engine replay-debugger-backend stream | `DebuggerSession`, `StepSnapshot`, `TimelineFrame` |
| `license-gate-cedar` | `oya-workflow-studio-license-gate-cedar-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Per-seat Cedar enforcement at editor open + per-action | `SeatLicense`, `LicenseDecision`, `EntitlementClaim` |

Naming justification — `visual-canvas`:

```
NAME: oya-workflow-studio-visual-canvas-<layer>
JUSTIFICATION:
- microservice = workflow-studio: hero product µservice (per-microservice flat layout, ADR-0131).
  Studio half of the workflow unbundle; engine is sibling at microservices/workflow-engine/.
- bc-tokens = visual-canvas: primary BC for drag-drop canvas + node/edge primitives + config panel +
  diff viewer + replay timeline UI. ADR-0056 v4.1 BC-optionality honoured (7 sibling BCs exist).
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + entity types (Canvas, Node, Edge, Selection, ViewportState). Zero I/O.
  - domain: pure visual layout algebra; deterministic node-placement math.
  - usecase (per ADR-0106): orchestrators driving canvas state transitions.
  - api: protocol-neutral typed I/O contracts.
  - adapter: protocol-neutral implementations.
  - adapter-leptos-wasm: Leptos-component implementations (browser-WASM target);
    backend-qualified per ADR-0105 Amendment 3.
  - rest: HTTP handler/route layer (editor session CRUD).
  - sdk: client library for tenant-side Studio embed (post-GA marketplace).
  - app: composition-root binary; SSR + WASM emit per ADR-0065.
- exemptions claimed: none.
```

Naming justification — `dsl-emitter`:

```
NAME: oya-workflow-studio-dsl-emitter-<layer>
JUSTIFICATION:
- microservice = workflow-studio.
- bc-tokens = dsl-emitter: BC for visual → canonical workflow_spec.v1.json emission.
- layer = <layer>: trimmed crate set; no rest/worker because emitter is consumed in-process by
  visual-canvas + usecase orchestrators.
  - kernel: port-trait + entities (EmitContext, EmittedSpec, EmitDiagnostic). Zero I/O.
  - domain: pure visual-to-spec mapping logic; deterministic.
  - usecase: orchestrators driving emit + validation.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
  - sdk: client library (tenant-side spec construction).
- exemptions claimed: rest/worker/app — emitter is library-only, consumed by visual-canvas layer.
```

Naming justification — `dsl-loader`:

```
NAME: oya-workflow-studio-dsl-loader-<layer>
JUSTIFICATION:
- microservice = workflow-studio.
- bc-tokens = dsl-loader: BC for workflow_spec.v1.json → visual rendering load.
- layer = <layer>: mirrors dsl-emitter (paired BCs).
  - kernel: port-trait + entities (LoadContext, LoadedDefinition, LoadDiagnostic). Zero I/O.
  - domain: pure spec-to-visual mapping logic; deterministic; load(emit(visual)) == visual invariant.
  - usecase: orchestrators driving load + validation.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
  - sdk: client library.
- exemptions claimed: rest/worker/app — loader is library-only.
```

Naming justification — `collab-crdt`:

```
NAME: oya-workflow-studio-collab-crdt-<layer>
JUSTIFICATION:
- microservice = workflow-studio.
- bc-tokens = collab-crdt: BC for CRDT state + merge logic + conflict surfacer + WebSocket collab
  server (separate worker binary).
- layer = <layer>: full set; WebSocket worker is a long-lived process.
  - kernel: port-trait + entities (CrdtState, MergeOp, Conflict, EditorSession). Zero I/O.
  - domain: pure CRDT merge algebra (yrs-style or loro-style); deterministic.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
  - adapter-redis: ephemeral CRDT state cache (active editor sessions only).
  - worker: WebSocket gateway long-lived process; handles fan-out of CRDT ops.
  - sdk: client library (tenant-side CRDT op submission).
- exemptions claimed: app — collab-crdt rolls into visual-canvas-app composition root.
```

Naming justification — `node-library-registry`:

```
NAME: oya-workflow-studio-node-library-registry-<layer>
JUSTIFICATION:
- microservice = workflow-studio.
- bc-tokens = node-library-registry: BC for per-pack signed node library distribution.
- layer = <layer>: full set; CDN-backed.
  - kernel: port-trait + entities (NodeLibrary, NodeDescriptor, LibrarySignature). Zero I/O.
  - domain: pure node-descriptor validation + signature verification logic.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
  - adapter-cdn: CDN edge-cache for signed library distribution; backend-qualified.
  - rest: HTTP surface for tenant-side library list + download.
  - sdk: client library.
  - app: composition-root.
- exemptions claimed: none.
```

Naming justification — `jurisdiction-overlay-renderer`:

```
NAME: oya-workflow-studio-jurisdiction-overlay-renderer-<layer>
JUSTIFICATION:
- microservice = workflow-studio.
- bc-tokens = jurisdiction-overlay-renderer: BC for jurisdiction-aware overlay resolution +
  visual diff over jurisdictions.
- layer = <layer>: trimmed; renderer is in-process consumed by visual-canvas.
  - kernel: port-trait + entities (Jurisdiction, Overlay, ResolvedView). Zero I/O.
  - domain: pure overlay resolution + diff algebra.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
- exemptions claimed: rest/worker/sdk/app — renderer is library-only; composed by visual-canvas-app.
```

Naming justification — `replay-debugger-frontend`:

```
NAME: oya-workflow-studio-replay-debugger-frontend-<layer>
JUSTIFICATION:
- microservice = workflow-studio.
- bc-tokens = replay-debugger-frontend: BC for the Studio-side rendering of engine replay-debugger-backend
  stream; distinct from engine BC because frontend concerns (timeline rendering, step navigation UX) are
  not engine-half concerns.
- layer = <layer>: trimmed; consumed by visual-canvas.
  - kernel: port-trait + entities (DebuggerSession, StepSnapshot, TimelineFrame). Zero I/O.
  - domain: pure step-snapshot → timeline-frame translation.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral impls; consumes engine replay-debugger-backend SDK.
  - sdk: client library.
- exemptions claimed: rest/worker/app — consumed by visual-canvas-app.
```

Naming justification — `license-gate-cedar`:

```
NAME: oya-workflow-studio-license-gate-cedar-<layer>
JUSTIFICATION:
- microservice = workflow-studio.
- bc-tokens = license-gate-cedar: BC for per-seat Cedar enforcement (load-bearing for hero-product
  billing); distinct from other Studio BCs because licensing concerns cross-cut authoring + collab +
  debugger.
- layer = <layer>:
  - kernel: port-trait + entities (SeatLicense, LicenseDecision, EntitlementClaim). Zero I/O.
  - domain: pure Cedar policy evaluation against seat-license entitlement claims.
  - usecase: orchestrators driving license check at editor open + per-action.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
  - adapter-postgres: persistent seat-attribution store.
  - sdk: client library.
- exemptions claimed: rest/worker/app — composed by visual-canvas-app.
```

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-redis | adapter-cdn | adapter-leptos-wasm | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `visual-canvas` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | ✓ | — | ✓ | ✓ |
| `dsl-emitter` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | ✓ | — |
| `dsl-loader` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | ✓ | — |
| `collab-crdt` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | — | — | ✓ | ✓ | — |
| `node-library-registry` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | ✓ | — | ✓ | ✓ |
| `jurisdiction-overlay-renderer` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — |
| `replay-debugger-frontend` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | ✓ | — |
| `license-gate-cedar` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | ✓ | — |

Total crates introduced by this µservice: **52** (9 visual-canvas + 6 dsl-emitter + 6 dsl-loader + 8 collab-crdt + 9 node-library-registry + 5 jurisdiction-overlay-renderer + 6 replay-debugger-frontend + 7 license-gate-cedar; with 4 backend-qualified adapter crates counted).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `CanvasStateStore` | `oya-workflow-studio-visual-canvas-kernel` | `-adapter-postgres` (via license-gate-cedar's session store) + `-adapter-leptos-wasm` (local edit buffer) | `BEHAVIORAL_TENANT_PRODUCT` |
| `SpecEmitter` | `oya-workflow-studio-dsl-emitter-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `SpecLoader` | `oya-workflow-studio-dsl-loader-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `CrdtMergeEngine` | `oya-workflow-studio-collab-crdt-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `EditorSessionStore` | `oya-workflow-studio-collab-crdt-kernel` | `-adapter-redis` (active sessions) | `BEHAVIORAL_TENANT_PRODUCT` |
| `WebSocketGatewayDispatcher` | `oya-workflow-studio-collab-crdt-kernel` | `-worker` | `BEHAVIORAL_TENANT_PRODUCT` |
| `NodeLibraryRepository` | `oya-workflow-studio-node-library-registry-kernel` | `-adapter-cdn` (signed distribution) | `INTERNAL_ONLY` + `AUDIT` (signatures) |
| `LibrarySignatureVerifier` | `oya-workflow-studio-node-library-registry-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `OverlayResolver` | `oya-workflow-studio-jurisdiction-overlay-renderer-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `JurisdictionDiffEngine` | `oya-workflow-studio-jurisdiction-overlay-renderer-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `DebuggerStreamConsumer` | `oya-workflow-studio-replay-debugger-frontend-kernel` | `-adapter` (consumes engine SDK) | `BEHAVIORAL_TENANT_PRODUCT` |
| `SeatLicenseStore` | `oya-workflow-studio-license-gate-cedar-kernel` | `-adapter-postgres` | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` |
| `CedarPolicyEvaluator` | `oya-workflow-studio-license-gate-cedar-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `workflow-studio` MUST NOT import any other product µservice crate directly. Studio consumes:
- `workflow-engine` via its SDK (spec submission + run query + replay-debugger-backend stream).
- `ontology` via its SDK (object-type descriptors for node config).
- `foundry-providers` via its SDK (LLM-assist).
- `application` via composition (Studio runs in the application µservice's hosting shell).
- `tenancy` via its SDK (per-seat licensing + tenant resolution).

All cross-µservice flows go through SDK boundaries; no direct kernel imports across µservices. LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice workflow-studio` — dependency-direction
- `oya gate validate lean-a2 --microservice workflow-studio` — cross-product-refusal
- `oya gate validate port-location --microservice workflow-studio`
- `oya gate validate layer-correctness --microservice workflow-studio`
- `oya gate validate per-microservice-layout --microservice workflow-studio`
- `oya gate validate statelessness --microservice workflow-studio` — Studio REST stateless (state in Postgres + Redis)
- `oya gate validate shardability --microservice workflow-studio` — editor sessions sharded by tenant_id
- `oya gate validate workflow-spec-roundtrip --microservice workflow-studio` — NEW lane asserting load(emit(x)) byte-equal to x for ≥ 100 golden specs
- `oya gate validate cedar-preview-required --microservice workflow-studio` — every save path exercises Cedar policy preview
- `oya gate validate editor-execution-forbidden --microservice workflow-studio` — Studio never executes; only emits
- `oya gate validate node-library-determinism --microservice workflow-studio` — 3x re-load assertion per `/specs/products/workflow-studio.json` §anti_patterns
- `oya gate validate wasm-bundle-sri --microservice workflow-studio` — every WASM chunk has SRI hash

## Integration via Workflow + Ontology

Studio is a product µservice; cross-product flows route through the engine's event-bus (per `feedback_workflow_objectgraph_adapter_layer.md`) and through Ontology reads.

### Workflow events produced

| Event type | Trigger | Consumed by | Purpose |
|---|---|---|---|
| `EditorSessionOpened` | editor open | engine (audit), tenancy (seat tracking), observability | per-seat license attribution + audit |
| `DefinitionSaved` | save action | engine (spec-store submit), audit-chain, ontology (definition link) | round-trip ack |
| `CollabMerged` | CRDT merge applied | observability | collab-merge-rate SLI |
| `CollabConflictSurfaced` | conflict UI shown | observability, audit | conflict-rate SLI |
| `LicenseGateEmitted` | Cedar evaluation | tenancy (billing), audit-chain | seat enforcement |
| `JurisdictionSwitched` | overlay change in editor | observability, audit | per-jurisdiction usage metric |
| `LlmAssistDraftRequested` | tenant prose → LLM-assist | foundry-providers (LLM invocation), audit-chain | LLM-assist usage + audit |
| `LlmAssistDraftAccepted` | tenant accepts LLM-assist output | engine (spec submission), audit | LLM-assist quality SLI |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `WorkflowStarted`, `StepStarted`, `StepCompleted`, `StepFailed` | engine | replay-debugger-frontend | live debugger render |
| `WorkflowCompleted`, `WorkflowFailed` | engine | visual-canvas (replay timeline) | terminal-state render |
| `OntologyTypeDescriptorUpdated` | ontology | node-library-registry | hot-reload node descriptors |
| `TenantSeatLimitUpdated` | tenancy | license-gate-cedar | refresh entitlement claim |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `WorkflowDefinition{tenant, spec_id, version_sha, author, saved_at, parent_version_sha}` | `authored_by→TenantUser`, `derived_from→WorkflowDefinition` (prior version) | visual-canvas → dsl-emitter → engine spec-store | Ed25519 |
| `EditorSession{tenant, session_id, user, opened_at, last_active, definition_id}` | `editing→WorkflowDefinition` | visual-canvas | Ed25519 (audit) |
| `LlmAssistDraft{tenant, draft_id, prompt_hash, completion_hash, accepted_at}` | `drafted_by→Foundry-providers-llm`, `accepted_into→WorkflowDefinition` | visual-canvas + LLM-assist-bridge | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `ObjectTypeDescriptor` (catalog) | node-library-registry + visual-canvas | `where(domain in tenant.packs).descriptors()` for typed node config |
| `Tenant` (catalog) | license-gate-cedar | `where(tenant_id=...).seat_limit` |
| `Pack` (catalog) | jurisdiction-overlay-renderer | `where(pack_id=...).overlays()` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| n8n | n8n Studio (visual editor) | drag-drop canvas; node library; webhook trigger; visual debugger | `docs.n8n.io` |
| Zapier | Zap editor | trigger/action editor; field mapping; AI generation | `help.zapier.com/zap-editor` |
| Workato | Recipe editor + recipe copilot | connector catalog; recipe functions; recipe copilot | `docs.workato.com` |
| Make.com | Scenario editor | scenario settings; error handlers; scenario recovery | `help.make.com` |
| Tray.io | Workflow builder | enterprise integration; visual builder | `tray.io/docs` |
| Microsoft Power Automate | Power Automate cloud + desktop | approvals; Dataverse; ALM; admin governance | `learn.microsoft.com/power-automate` |
| Workflow86 | Visual workflow editor | low-code workflow builder | `workflow86.com/docs` |
| Pipedream | Workflow + code editor | mixed visual+code; serverless | `pipedream.com/docs` |
| Camunda Modeler | BPMN modeler | BPMN 2.0; collaboration; Git sync; play mode; element templates | `docs.camunda.io` |
| Linear (workflow status UX benchmark) | Linear workflow configuration | opinionated state model; keyboard-fast UX | `linear.app/docs/configuring-workflows` |

Key parity gaps to close (ordered by priority for M03 preview milestone):

1. **TTI parity with n8n** — n8n claims ≤ 3s typical; oyatie target ≤ 2s GA via CDN-cached WASM bundle. Hard engineering requirement.
2. **Round-trip byte-equality** — no competitor offers this as a contractual invariant. Most BPMN-first tools mutate spec on visual-edit. oyatie unique.
3. **Per-pack node libraries (6 domains)** — n8n has ~400 nodes; Workato has 1200+ connectors. oyatie's per-pack discipline is the differentiator; raw count not the claim.
4. **Cedar policy preview before save** — none of n8n / Zapier / Workato / Make show policy impact before save. oyatie unique.
5. **Jurisdiction-overlay visual diff** — oyatie unique; competitors do not have multi-jurisdiction overlay UX.
6. **LLM-assist authoring** — Zapier has AI generation (entry-level); Workato has recipe copilot. oyatie M03+ target with foundry-providers backbone.
7. **Per-seat Cedar licensing** — competitors use SaaS-account billing; oyatie's per-seat Cedar enforcement is fine-grained.

Detailed quantitative comparison in `competitor-parity-matrix.md`.

## Performance Targets

(Duplicated from §"Non-Functional Requirements" for ease of citation by downstream PRD consumers.)

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Editor TTI cold | 1s | 2s | 5s | GA target via CDN-cached WASM bundle |
| Save round-trip | 80ms | 200ms | 500ms | engine spec-store ack |
| Collab CRDT merge | 30ms | 100ms | 250ms | sub-100ms p99 GA |
| LLM-assist draft | 1.5s | 3s | 8s | depends on foundry-providers |
| Round-trip byte-equality | — | — | — | 100% at GA |

Error budget:
- Monthly error budget for editor REST: 0.05% (≈22 min/month).
- Monthly error budget for WebSocket collab gateway: 0.1%.
- Burn-rate alarms: 14.4× burn over 1h for editor REST; 6× burn over 6h for collab gateway.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Rationale:
- Editor session state, spec drafts, per-seat license attribution: `postgres` (Citus-distributed by tenant_id).
- Ephemeral collab CRDT state (active editor sessions): `redis` (per-cell cluster; reconstructable from Postgres on cold-start).
- Static assets (WASM bundles + node library descriptors + spec schema): `cdn` (global edge cache; per-tenant key partitioning).
- Object storage for large node library binaries (per-pack signed): OCI Object Storage.

**Active-active compatibility**: `stateless-compatible` for REST/SDK; `single-writer-compatible` for collab CRDT (one WebSocket gateway pod owns active sessions for a given definition; lease-coordinated via Redis).

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active editor sessions | 10,000 | 100,000 | WebSocket connection count > 70k OR collab merge wait p99 > 100ms |
| Definitions per tenant | 1,000 | 100,000 | Postgres shard fill > 80% |
| Concurrent collab-server WS connections | 50,000 | 500,000 | WS gateway pod CPU > 70% |
| Save round-trips/sec (cluster-wide) | 1,000 | 100,000 | Postgres write rate > 80% capacity |
| LLM-assist requests/sec | 10 | 1,000 | foundry-providers backpressure signal |

Scale-out policy:
- Editor REST: stateless HPA on CPU > 70%; min 2 replicas; max 50.
- WebSocket gateway (collab-crdt-worker): stateful per active editor session; lease-coordinated via Redis; HPA on WS connection count; min 3 replicas; max 100.
- Postgres + Citus: tenant_id shard key; linear shard addition.
- Redis: per-cell cluster; HA via Sentinel.
- CDN: global; per-pack edge nodes; OCI CDN service.

Cross-region story:
- M03 preview launch: single KR region.
- Post-M03: per-pack residency activation; CDN edges per pack; editor session state pinned to pack.
- LLM-assist: foundry-providers handles cross-pack residency; Studio inherits.

Sharding:
- Postgres + Citus on `tenant_id`; editor sessions append-only; Citus distributed table.
- Redis per-cell cluster; cell-local CRDT state.
- WebSocket gateway: consistent-hash on `definition_id` ensures collab participants land on same gateway pod.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Visually-built workflow → save → emit canonical workflow_spec.v1.json (semantically identical) | `cargo nextest run -p oya-workflow-studio-dsl-emitter-domain --test test_visual_round_trip` |
| AC-02 | Load workflow_spec.v1.json → render visual → emit → byte-identical to input (100% over corpus 100 specs) | `cargo nextest run -p oya-workflow-studio-dsl-loader-domain --test test_load_emit_byte_equal` |
| AC-03 | Editor open with pending changes; network disconnect; local buffer persists; resume without loss on reconnect | `tests/e2e/offline-buffer-resume.rs` |
| AC-04 | Workflow with jurisdiction overlay; switch jurisdiction; overlay-resolved view renders; base reachable | `cargo nextest run -p oya-workflow-studio-jurisdiction-overlay-renderer-domain --test test_jurisdiction_view_switch` |
| AC-05 | LLM-authored spec via API; valid: opens in editor; invalid: precise per-line error | `tests/e2e/llm-assist-validation.rs` |
| AC-06 | Two users edit same definition concurrently; CRDT merge applies non-conflicting; conflict UI for overlap; never silent loss | `cargo nextest run -p oya-workflow-studio-collab-crdt-domain --test test_no_silent_overwrite` |
| AC-07 | Editor in kr jurisdiction; author touches PII field; visual data_class marker + Cedar policy preview before save | `cargo nextest run -p oya-workflow-studio-visual-canvas-domain --test test_pii_marker_visible` |
| AC-08 | Per-seat license check on editor open; Cedar policy gate enforces seat; audit row emitted | `cargo nextest run -p oya-workflow-studio-license-gate-cedar-domain --test test_per_seat_cedar` |
| AC-09 | TTI ≤ 2s p99 cold (CDN-cached); Lighthouse-style budget assertion | `tests/load/tti-budget.js` |
| AC-10 | Save round-trip ≤ 200ms p99 (stable); ≤ 100ms p99 (GA) | `tests/load/save-roundtrip.js` |
| AC-11 | Node-library determinism: same library version loads byte-identical descriptors 3x | `cargo nextest run -p oya-workflow-studio-node-library-registry-domain --test test_load_determinism` |
| AC-12 | WASM bundle SRI: every chunk hash verifies; mismatch refuses load | `cargo nextest run -p oya-workflow-studio-visual-canvas-adapter-leptos-wasm --test test_sri` |
| AC-13 | `oya gate validate per-microservice-layout --microservice workflow-studio` exit 0 | ADR-0131 lane |
| AC-14 | `oya gate validate authority-cohesion` exit 0 | ADR-0123 lane; HG-WORKFLOW-STUDIO registered |
| AC-15 | `oya gate validate workflow-spec-roundtrip --microservice workflow-studio` exit 0 | new lane spec'd in PHASE-01 |
| AC-16 | `oya gate validate cedar-preview-required --microservice workflow-studio` exit 0 | new lane |
| AC-17 | `oya gate validate editor-execution-forbidden --microservice workflow-studio` exit 0 | new lane; Studio never executes |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | CRDT library: yrs (yjs Rust port) vs loro vs bespoke? Bias: loro for tree-aware CRDT (matches DAG structure). | council-architecture | ADR-XXXX (gates IP-005) |
| 2 | WASM canvas: pure Leptos-Rust-WASM vs Leptos-shell + JS canvas library (reactflow/xyflow)? Bias: pure Leptos for stable+; JS-canvas only for M03-preview if Leptos canvas not ready. | council-design-system + axis-workflow | ADR-XXXX (gates IP-002) |
| 3 | WebSocket gateway substrate: bespoke axum-WS vs NATS-WebSocket-bridge? Bias: bespoke axum for tight tenant-binding control. | council-architecture | resolved inline; see collab-crdt IP |
| 4 | LLM-assist invocation: stream-back-to-browser via WS vs server-side full-draft then send? Bias: stream-back for UX. | axis-workflow + foundry-providers | ADR-XXXX (gates IP-008) |
| 5 | Per-pack node-library hot-reload: full reload vs delta? | axis-workflow | resolved inline |
| 6 | Multi-tab editing: same definition open in 2 tabs by same user — single CRDT session or 2 sessions? Bias: single session, per-tab cursor. | council-design-system | resolved inline |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0164 (Bominal) | Workflow canonical spec format | inherited; Studio binds to this format verbatim |
| ADR-0103 (Bominal) | Workflow hexagonal migration | inherited; clean-arch placement |
| ADR-0037 (Bominal) | Plugin substrate (WASM) | inherited; node-library scaffolding |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0065 | Docs-as-Leptos webapp | inherited; Studio is the largest Leptos app |
| ADR-0105 | 13-layer enum + adapter-cdn | layer authority |
| ADR-0106 | Application → usecase rename | applied for new crates |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0123 | Hyperscaler maturity claim gate | HG-WORKFLOW-STUDIO registers here |
| ADR-0130 | Agentic SLO-gated promotion | Studio SLO promotion gates this µservice |
| ADR-0131 | Per-microservice flat layout + workflow unbundle | this µservice authored natively under it; sibling = workflow-engine |
| ADR-0132 | Product-suite-and-bundle dissolution | Studio is a hero product, not a suite |
| ADR-0133 | Industry-best-practice conformance | Studio competitor parity tracked here |
| ADR-0140 | Cedar policy enforcement | license-gate-cedar BC built on this |
| oyatie override | Workflow Studio scope | `feedback_workflow_studio_scope.md` |
| oyatie override | Workflow + Ontology = ecosystem adapter | `feedback_workflow_objectgraph_adapter_layer.md` |
| oyatie split | engine vs studio unbundle | ADR-0131 §"workflow unbundle" |
