---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-slides
microservice: slides
status: Accepted
sales_segment: workspace-product
tier: external-facing
milestone_first_ship: M03-workspace-preview
bominal_source: []
related_adrs: [ADR-0056, ADR-0065, ADR-0105, ADR-0106, ADR-0123, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-0140 (retired per ADR-0145), ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/microservices/workspace.json]
related_unbundle_adr: ADR-0135
unbundle_sibling_set:
  - microservices/docs/
  - microservices/sheets/
  - microservices/drive/
  - microservices/forms/
date: 2026-05-17
owner_team: axis-workspace + council-design-system
doc_status: published
---

# PRD-slides: Slides — Collaborative Presentation Authoring + Live Broadcast

## Purpose

The `slides` µservice is oyatie's **collaborative presentation product** — a Google-Slides / Microsoft PowerPoint Web / Apple Keynote / Pitch / Beautiful.ai / Canva-presentations-class product surface. It owns: deck authoring (slides, layouts, master slides, themes), real-time multi-user collaboration via Loro CRDT (aligning with workflow-studio + docs + sheets per ADR-WS-0001 family), present-mode (single-presenter + audience-broadcast), live audience-engagement (reactions + Q&A + polls), import/export across PPTX / ODP / PDF / Keynote / MP4, AI-design and AI-content-generation under foundry-runtime gating, embed bridges to docs (quote blocks), sheets (live charts), and forms (in-deck polls), and broadcast-mode signaling reusing the messenger µservice's LiveKit infrastructure.

Slides is **net-new** per ADR-0135 (Connect dissolution) — there is no `oya-connect-slides-*` legacy. The µservice is a hero workspace surface alongside `docs`, `sheets`, `drive`, and `forms`.

Slides operates at the **application** layer of the 12-layer Workflow + Ontology architecture (per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`): it consumes ontology object-type descriptors for embed-bridge typing; emits authoring + broadcast events to the workflow event-bus; routes cross-µservice flows (sheets chart links, docs quote-embeds, forms polls, messenger broadcast-signaling, drive storage) through SDKs only; runs in the application µservice's hosting shell.

Slides inherits the **Loro 1.x CRDT decision** structurally from workflow-studio's ADR-WS-0001 (and parallels docs + sheets) — see ADR-SLIDES-0001 for the slides-specific application. The visual canvas adopts the **Leptos WASM substrate** from ADR-WS-0003 — see ADR-SLIDES-0002 for the slides-specific rendering tier choice.

The load-bearing invariants are: **never silent loss** under concurrent edit (CRDT correctness — AC-06 mirrors workflow-studio); **60-fps present-mode** transitions (AC-09); **PPTX round-trip fidelity** for the round-trippable subset (AC-02); **per-slide ACL granularity** when configured (AC-15 via ADR-SLIDES-0007); **EU AI Act risk classification** on all AI-content-generation paths (AC-16 via ADR-SLIDES-0006).

## Tenant Value

- **Tenant Outcome 1 — Deck open in under 400ms (cold) / 150ms (warm), 50 slides p95.** Google Slides parity (~600ms typical cold); Keynote parity (warm only on macOS). CDN-cached WASM bundle + spec schema; progressive slide hydration.
- **Tenant Outcome 2 — Live collaborative editing without silent loss.** Two business users editing same deck concurrently: Loro CRDT merge applies non-conflicting edits; conflicting edits surface explicit conflict UI; no last-writer-wins (AC-06). Cursor sync p99 ≤ 150ms (parity with workflow-studio collab budget).
- **Tenant Outcome 3 — Present-mode at 60fps with audience engagement.** Slide-transition p95 ≤ 50ms; broadcast-mode reuses messenger LiveKit infra; audience polls + reactions + Q&A overlay. Reduced-motion fallback (WCAG 2.2 SC 2.3.3) for accessibility.
- **Tenant Outcome 4 — Export parity across PPTX, PDF, MP4.** PPTX best-effort round-trip for OOXML PresentationML (ECMA-376) round-trippable subset; PDF/A-1b + PDF/A-2u archival via WeasyPrint or Chromium-headless; deterministic MP4 export via ffmpeg in gVisor sandbox.
- **Tenant Outcome 5 — Live chart bridges to sheets µservice.** Author embeds chart from sheet cell range; chart auto-refreshes when sheet data changes; access revocation flows from sheets ACL to slides chart (ADR-SLIDES-0008).
- **Tenant Outcome 6 — AI-design + AI-content-generation under risk-class enforcement.** T1 capabilities (design-assist, layout-suggest, copy-refine, auto-alt-text, slide-summary) baseline low/medium-risk; T2 (full-deck-from-prompt) flagged high-risk when context triggers EU AI Act Annex III (employment, credit, legal, medical). Foundry-runtime mediates.
- **Tenant Outcome 7 — Per-pack residency + jurisdictional defaults.** 11 packs (kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa); residency, retention, and AI-policy defaults are pack-driven.
- **Internal Outcome 8 — Embed-bridge backbone for cross-workspace flows.** Slides is a primary consumer of the embed-bridge pattern; canonical example of cross-µservice SDK-only integration with workflow-studio + docs + sheets + forms + drive + messenger + social + mail.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | author | drag-drop placeholders onto a slide | I can author slides without code | slide |
| FR-02 | author | apply a slide-layout (title, content, two-column, blank) | I get structural defaults | slide-layout |
| FR-03 | author | edit a master slide and have all derived slides cascade | I get consistent theming | slide-layout |
| FR-04 | author | rich-text edits in a text-box with inline formatting + animations | I get DTP-grade text | text-box |
| FR-05 | author | insert shapes (rect, ellipse, line, polygon, freeform) with vector editing | I get illustration capability | shape |
| FR-06 | author | insert images with crop + filter (brightness/contrast/saturation/sepia) | I get DTP-grade image control | image |
| FR-07 | author | embed a video with playback controls + frame poster | I get rich media | video-embed |
| FR-08 | author | embed audio with looping + autoplay | I get audio media | audio-embed |
| FR-09 | author | embed a live chart linked to a sheets cell range | I get auto-refreshing data viz | chart |
| FR-10 | author | insert tables with cell merge + per-cell styling | I get tabular layouts | table |
| FR-11 | author | typeset equations via KaTeX / MathJax | I get academic-grade math | equation |
| FR-12 | two authors | edit the same deck concurrently with Loro CRDT merge | no silent loss; explicit conflict | real-time-collaboration |
| FR-13 | author | add comments + suggestion-mode edits | reviewers can propose changes | comments |
| FR-14 | author | browse version history + restore prior version | I get auditable revisions | version-history |
| FR-15 | author | configure entrance/emphasis/exit/path animations per object | I get motion design | animations |
| FR-16 | author | configure slide transitions (fade, slide, push, morph, none) | I get pacing | transitions |
| FR-17 | presenter | open presenter-view with timer + speaker-notes + audience-camera | I can pace + read notes | presenter-view |
| FR-18 | audience | reactions, polls, and Q&A during a presentation | I engage live | audience-view |
| FR-19 | presenter | broadcast a presentation to large audience via LiveKit | I reach >100 attendees | broadcast-mode |
| FR-20 | author | apply a theme from gallery or upload custom | I get design-system | themes |
| FR-21 | author | save current deck as a template | I get reusable scaffolds | templates |
| FR-22 | author | reorder slides via slide-sorter drag | I get deck-level reorg | slide-sorter |
| FR-23 | author | edit master-slide + custom-layouts | I get advanced theming | master-slide-editor |
| FR-24 | author | auto-align + auto-distribute selected objects | I get layout-engine assist | layout-engine |
| FR-25 | author | import PPTX / ODP / PDF / Keynote into a deck | I migrate from competitors | import |
| FR-26 | author | export deck to PPTX / ODP / PDF (PDF/A-1b + PDF/A-2u) / MP4 / PNG-per-slide | I publish anywhere | export |
| FR-27 | author | get auto-generated alt-text suggestions per image (T1 AI-assist) | accessibility compliance | accessibility-alt-text |
| FR-28 | author | validate color contrast against WCAG 2.2 AA + color-blind-safe palette | inclusive design | accessibility |
| FR-29 | author | prompt foundry-runtime to design-assist (T1) or generate-full-deck (T2) | I get AI-accelerated authoring | ai-design + ai-content-generation |
| FR-30 | tenant | configure per-slide ACL (named-block-level read/comment/edit) | fine-grained sharing | acl |
| FR-31 | tenant | configure deck-level ACL with link-sharing (public-read, link-anyone-with-link) | broad sharing | acl |
| FR-32 | author | use prefers-reduced-motion fallback when presenting to accessible audiences | inclusive present-mode | animations + accessibility |
| FR-33 | tenant | publish deck-as-shorts to social µservice | cross-product publishing | embed-bridge |
| FR-34 | author | share via mail µservice (with rendered preview attachment) | external distribution | embed-bridge |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | p999 | Notes |
|---|---|---|---|---|---|
| Deck open cold (50 slides) | 250ms | 400ms | 600ms | 1.2s | CDN-cached WASM + progressive slide hydration |
| Deck open warm | 80ms | 150ms | 250ms | 500ms | service-worker cache + warm session |
| Slide render (single slide) | 50ms | 100ms | 150ms | 300ms | per-slide signal-driven render |
| Cell-edit-render (e.g., text-box keystroke) | 20ms | 40ms | 50ms | 100ms | p99 = 50ms invariant |
| Collab cursor sync | 60ms | 120ms | 150ms | 250ms | align workflow-studio cursor budget |
| Save (delta) | 50ms | 100ms | 200ms | 400ms | spec-store ack |
| Export PDF (50-slide) | 1.5s | 3s | 5s | 10s | WeasyPrint or Chromium-headless |
| Export PPTX (50-slide) | 2s | 5s | 8s | 15s | OOXML serialization |
| Export MP4 (per slide) | 1s | slide_count × 1s + 5s | — | — | ffmpeg deterministic mode |
| Chart-render (live-link from sheets) | 100ms | 200ms | 350ms | 700ms | per-chart per-refresh |
| Present-mode slide transition | 16ms | 33ms | 50ms | 100ms | 60fps invariant |
| Broadcast-mode signaling round-trip | 80ms | 150ms | 250ms | 500ms | reuses messenger LiveKit |
| Active editor sessions per region | — | 50,000 | 200,000 | — | XL tier; horizontal scale via session sharding |
| Concurrent collab edits per deck | — | 10 | 20 | — | beyond 20 → degraded UX |
| Broadcast-mode concurrent viewers per deck | — | 500 | 5,000 | — | LiveKit SFU cascade for >500 |

### Security

- OIDC tenant-scoped at every REST + WebSocket entry; slides refuses opens without resolvable tenant identity.
- Per-deck ACL evaluated via Cedar v4.2 LTS; per-slide ACL is a Cedar refinement (ADR-SLIDES-0007).
- Strict CSP (`default-src 'self' https://cdn-<pack>.oyatie.dev; script-src 'self' 'wasm-unsafe-eval' 'nonce-<random>'`); no inline scripts except WASM bootstrap nonce.
- XSS-free render: rich-text rendered via virtual-DOM text nodes; never `innerHTML`. Embed-bridge content (charts, polls, docs-quotes) passes through sanitization at the bridge boundary.
- Upload virus-scan: ClamAV + OPSWAT for any uploaded asset (image, video, audio, PPTX/ODP/Keynote import); quarantine on detection.
- Per-tenant CDN cache key partitioning; no cross-tenant cache pollution.
- WebSocket auth: OIDC token validated at WS upgrade; tenant binding rebound at WS message dispatch.
- AI-content-generation content never trusted: T2 full-deck-from-prompt drafts pass through schema + policy + signature pipeline before save; EU AI Act risk-class evaluated at generation request.
- WASM bundle subresource-integrity (SRI) hashes per chunk; mismatch refuses load.
- Per-tenant editor-session isolation: session state in Postgres scoped by tenant_id (Citus partition); cross-tenant access forbidden by RLS + Cedar.
- gVisor sandbox for PPTX import + MP4 export + media transcode workers (untrusted file parsing isolated).
- Asset signing: per-pack node-library + theme-gallery + template-gallery signed Ed25519; tampered assets refused.

### Audit + Compliance

- Every save emits a `slides_deck_saved` audit-chain seal (Ed25519); seal includes `(tenant_id, deck_id, version_sha, author_identity, parent_version_sha, timestamp, pack)`.
- Every share-ACL change emits `slides_acl_changed` audit row.
- Every present-broadcast session emits start + end audit rows including attendee count + duration.
- LLM-assist + AI-content-generation invocations: prompt + completion + risk-class archived 90d for audit; per ADR-SLIDES-0006.
- Per-pack `jurisdiction_code` enforced via overlay (kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa); cross-pack collab forbidden.
- Per-slide ACL changes emit `slides_per_slide_acl_changed` audit row (ADR-SLIDES-0007).
- Chart-live-link revocation cascade audited end-to-end (sheets ACL change → slides chart access revoke → audit).

### Availability + SLO

- Editor REST availability: 99.95% monthly (GA); 99.9% (stable); 99.5% (preview).
- WebSocket collab + broadcast-mode signaling availability: 99.9% monthly.
- Present-mode core (offline-capable after deck-open): 99.99% (network-independent once hydrated).
- LiveKit broadcast-bridge availability: inherited from messenger SLO (99.9%).
- RTO ≤ 1800s for editor-rest per manifest `dr.rto_p99_seconds=1800`.
- RPO ≤ 120s for editor session state and deck-spec writes per manifest `dr.rpo_p99_seconds=120`.

### Data residency

- Editor session state, deck content (Postgres metadata, S3 snapshots, Valkey CRDT cache), per-deck ACL: pack-pinned per tenant `jurisdiction_code` (ADR-0117 inheritance).
- CDN static assets (WASM, theme gallery, template gallery): global with per-pack edge cache keys.
- Broadcast-mode signaling: LiveKit nodes pack-pinned via messenger µservice.
- AI-generation: foundry-runtime handles cross-pack residency; slides inherits.

### DR Posture (ADR-0343)

- RTO/RPO target: manifest `dr` declares `rto_p99_seconds=1800` and `rpo_p99_seconds=120`. HIPAA-2024 (3600s/300s), PCI-DSS-L1-v4 (86400s/3600s), SOC2-T2 (14400s/900s), ISO27001-2022 (14400s/3600s), and LGPD-aligned residency packs leave the effective slides bound at 1800s RTO and 120s RPO.
- failover_runbook: `runbooks/dr-failover.md`; manifest backup substrate is `postgres_wal_g`, `object_storage_versioned`, and `valkey`.
- multi_region_active_active: true, with manifest replication shape `active-active-multi-az-cross-region-warm`; present-mode remains offline-capable once the deck is hydrated.
- WHY: presenters can survive regional control-plane failure without losing committed deck state or ending an already-hydrated presentation.

### Capacity Model (ADR-0340)

- Per-tenant baseline: manifest `capacity_model` declares 0.15 vCPU, 512Mi RAM, 12Gi storage, 3 Valkey connections, 2 Postgres connections, and 6 outbound HTTP connections per tenant.
- Scaling dimension: `per_user`; editing sessions, presenter/audience fan-out, deck rendering, and export workers scale with active users and broadcast participation.
- Cell placement class: Tier-3, matching manifest `capacity_model.cell_placement_class`, because slides is a high-throughput authoring/broadcast application surface rather than tenant-customer code execution.
- Autoscaling boundaries: editor REST min 4 / max 50, real-time collaboration min 3 / max 100, broadcast worker min 2 / max 50, and export worker min 4 / max 100 before tenant throttles engage.
- WHY: slides has two different peaks--authoring bursts and live broadcast fan-out--so capacity must isolate editor/collab, broadcast, and export/AI pools.

### Sustainability + Cost Attribution (ADR-0344)

- Every audit-chain row emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` for saves, ACL changes, broadcast start/end, per-slide ACL, chart revocation, AI generation, import/export, render, and media transcode events.
- Provider routing affected by carbon: no for live broadcast/presenter path, PCI/HIPAA contexts, ACL enforcement, or high-risk AI review; yes for PPTX/PDF/MP4 export, render backfill, theme/template processing, and AI design queues when tenant policy allows.
- Per-tenant transparency surface: FinOps portal shows deck storage, editor sessions, broadcast viewer-minutes, export/render CPU, MP4 transcode time, AI generation calls, and LiveKit bridge cost by tenant/capability/provider/cell/compliance_pack.
- WHY: live presentations cannot be delayed for greener routing, but exports, renders, and AI-design workloads are visible enough to support CSRD, SB-253, and SEC climate reporting.

### API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet via `Oyatie-Version` header, `/v/<YYYY-MM-DD>` URL prefix, and proto3 `oyatie_version` field for deck, slide, ACL, broadcast, import/export, AI-design, and embed-bridge contracts.
- SDK semver model: major.minor.patch for browser-WASM editor SDKs, broadcast SDKs, import/export clients, and embed SDKs.
- Support window: last N=3 public API versions for at least 180 days, including export and broadcast schemas used in customer validation.
- Per-tenant pinning supported: yes, especially for presentation templates, regulated broadcast flows, and deck-export validation suites.
- Internal-mesh exemption: yes; direct gRPC to drive, sheets, docs, forms, messenger, social, mail, and foundry remains exempt under ADR-0145 when internal-only.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), slides uses: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-valkey`, `adapter-s3`, `adapter-loro`, `adapter-pandoc`, `adapter-weasyprint`, `adapter-chromium-headless`, `adapter-ffmpeg`, `adapter-livekit`, `adapter-imagemagick`, `adapter-clamav`, `adapter-opswat`, `adapter-leptos-wasm`, `rest`, `worker`, `sdk`, `app`. Browser-WASM artifacts compile from the `app` layer per ADR-0065.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `presentation` | `oya-slides-presentation-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,rest,sdk,app}` | Deck metadata, deck-level ACL, deck CRUD | `Deck`, `DeckMetadata`, `DeckAcl` |
| `slide` | `oya-slides-slide-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-leptos-wasm,sdk}` | Individual slide structure, content, ordering | `Slide`, `SlideContent`, `SlideOrder` |
| `slide-layout` | `oya-slides-slide-layout-{kernel,domain,usecase,api,adapter,sdk}` | Layout templates (title, content, two-column, blank, master-layout, custom-layout) | `Layout`, `Placeholder`, `MasterLayout` |
| `text-box` | `oya-slides-text-box-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm}` | Rich-text with inline formatting + animations | `TextBox`, `RichRun`, `TextAnimation` |
| `shape` | `oya-slides-shape-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm}` | Vector shapes + freeform paths | `Shape`, `Path`, `VectorStyle` |
| `image` | `oya-slides-image-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-imagemagick,adapter-clamav}` | Image embed with crop + filter | `Image`, `Crop`, `Filter` |
| `video-embed` | `oya-slides-video-embed-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-ffmpeg,adapter-clamav}` | Video embed with transcode + playback | `VideoEmbed`, `Poster`, `PlaybackConfig` |
| `audio-embed` | `oya-slides-audio-embed-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-ffmpeg}` | Audio embed | `AudioEmbed`, `LoopConfig` |
| `chart` | `oya-slides-chart-{kernel,domain,usecase,api,adapter,sdk}` | Live-link chart bridge to sheets | `ChartLink`, `CellRange`, `RefreshPolicy` |
| `table` | `oya-slides-table-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm}` | Tables with merge + per-cell style | `Table`, `Cell`, `CellStyle` |
| `equation` | `oya-slides-equation-{kernel,domain,usecase,api,adapter}` | KaTeX / MathJax equation typeset | `Equation`, `MathExpression` |
| `real-time-collaboration` | `oya-slides-real-time-collaboration-{kernel,domain,usecase,api,adapter,adapter-valkey,adapter-loro,worker,sdk}` | Loro-based CRDT collab; WebSocket dispatcher | `CrdtState`, `MergeOp`, `Conflict`, `EditorSession` |
| `comments` | `oya-slides-comments-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Threaded comments + suggestion-mode | `Comment`, `Suggestion`, `Thread` |
| `version-history` | `oya-slides-version-history-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,sdk}` | Versioning + restore | `Version`, `Diff`, `RestoreRequest` |
| `animations` | `oya-slides-animations-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm}` | Per-object animations + reduced-motion | `Animation`, `Timing`, `ReducedMotionFallback` |
| `transitions` | `oya-slides-transitions-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm}` | Slide-to-slide transitions | `Transition`, `Duration`, `Easing` |
| `speaker-notes` | `oya-slides-speaker-notes-{kernel,domain,usecase,api,adapter}` | Per-slide speaker notes | `SpeakerNote` |
| `presenter-view` | `oya-slides-presenter-view-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm}` | Presenter dashboard with timer + notes + cam | `PresenterSession`, `Timer`, `AudienceCam` |
| `audience-view` | `oya-slides-audience-view-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm,sdk}` | Audience overlay (reactions, polls, Q&A) | `AudienceSession`, `Reaction`, `Poll`, `Question` |
| `broadcast-mode` | `oya-slides-broadcast-mode-{kernel,domain,usecase,api,adapter,adapter-livekit,worker,sdk}` | LiveKit-bridged broadcast streaming | `BroadcastSession`, `SignalRoute`, `ViewerLease` |
| `themes` | `oya-slides-themes-{kernel,domain,usecase,api,adapter,adapter-s3,sdk}` | Theme gallery + custom themes | `Theme`, `Palette`, `Typography` |
| `templates` | `oya-slides-templates-{kernel,domain,usecase,api,adapter,adapter-s3,sdk}` | Template gallery + tenant templates | `Template`, `TemplateMetadata` |
| `slide-sorter` | `oya-slides-slide-sorter-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm}` | Deck-level slide reorder UI | `SlideSorterState`, `ReorderOp` |
| `master-slide-editor` | `oya-slides-master-slide-editor-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm}` | Master + custom-layout authoring | `MasterEditState`, `LayoutEditOp` |
| `layout-engine` | `oya-slides-layout-engine-{kernel,domain,usecase,api,adapter}` | Auto-align + auto-distribute + smart-arrange | `AlignSpec`, `DistributeSpec`, `SmartArrange` |
| `import-export` | `oya-slides-import-export-{kernel,domain,usecase,api,adapter,adapter-pandoc,adapter-weasyprint,adapter-chromium-headless,adapter-ffmpeg,worker,sdk}` | PPTX / ODP / PDF / Keynote / MP4 / PNG pipelines | `ImportJob`, `ExportJob`, `Format` |
| `accessibility` | `oya-slides-accessibility-{kernel,domain,usecase,api,adapter}` | Alt-text suggest, contrast check, reduced-motion policy, color-blind-safe palette | `AltText`, `ContrastReport`, `MotionPolicy` |
| `ai-design` | `oya-slides-ai-design-{kernel,domain,usecase,api,adapter,sdk}` | T0/T1 design suggestions via foundry-runtime | `DesignSuggestion`, `LayoutHint`, `CopyRefine` |
| `ai-content-generation` | `oya-slides-ai-content-generation-{kernel,domain,usecase,api,adapter,sdk}` | T2 full-deck-from-prompt + risk-class enforcement | `DeckGenRequest`, `RiskClassification`, `GeneratedDeck` |
| `embed-bridge` | `oya-slides-embed-bridge-{kernel,domain,usecase,api,adapter,sdk}` | Cross-µservice embed (docs quotes, sheets charts, forms polls) | `EmbedRef`, `BridgeBinding`, `RevocationFlow` |
| `acl` | `oya-slides-acl-{kernel,domain,usecase,api,adapter,adapter-postgres}` | Deck + per-slide + named-block Cedar enforcement | `AclEntry`, `ScopedDecision`, `PolicyImpact` |

Naming justification — `presentation`:

```
NAME: oya-slides-presentation-<layer>
JUSTIFICATION:
- microservice = slides: hero workspace µservice (per-microservice flat layout, ADR-0131).
  Net-new per ADR-0135; no legacy connect-slides-* heritage.
- bc-tokens = presentation: top-level container BC; one Presentation = one Deck.
  ADR-0056 v4.1 BC-optionality honoured (30 sibling BCs exist; presentation is the
  composition root).
- layer = <layer>: per ADR-0105 13-value canonical enum.
  - kernel: port-trait + entity types (Deck, DeckMetadata, DeckAcl). Zero I/O.
  - domain: pure deck-lifecycle algebra.
  - usecase (per ADR-0106): orchestrators driving deck CRUD + ACL transitions.
  - api: protocol-neutral typed contracts.
  - adapter: protocol-neutral impls.
  - adapter-postgres: deck metadata + ACL persistence (backend-qualified per ADR-0105 Amd.3).
  - adapter-s3: deck snapshot blob storage (backend-qualified).
  - rest: HTTP surface (deck CRUD, ACL CRUD).
  - sdk: client library (tenant + cross-µservice consumers).
  - app: composition-root binary; SSR + WASM emit per ADR-0065.
- exemptions claimed: none.
```

Naming justification — `real-time-collaboration`:

```
NAME: oya-slides-real-time-collaboration-<layer>
JUSTIFICATION:
- microservice = slides.
- bc-tokens = real-time-collaboration: BC for Loro CRDT state + merge + conflict surfacer + WS
  dispatcher. Mirrors workflow-studio collab-crdt BC; slides-side bound; per-BC isolation
  preserved (collab-crdt is workflow-studio's BC, real-time-collaboration is slides'; the Loro
  library substrate is shared per ADR-SLIDES-0001).
- layer = <layer>:
  - kernel + domain + usecase + api + adapter: standard layers.
  - adapter-valkey: ephemeral CRDT cache.
  - adapter-loro: backend-qualified per ADR-0105 Amd.3 — Loro library binding lives here, kept
    out of -domain to preserve the "kernel ports never leak Loro types" invariant from ADR-SLIDES-0001.
  - worker: WS gateway long-lived process.
  - sdk: client library.
- exemptions claimed: app — composition rolls up into presentation-app.
```

Naming justification — `broadcast-mode`:

```
NAME: oya-slides-broadcast-mode-<layer>
JUSTIFICATION:
- microservice = slides.
- bc-tokens = broadcast-mode: BC for large-audience broadcast presenting via LiveKit-bridged
  signaling. Reuses messenger µservice's LiveKit infrastructure (per ADR-SLIDES-0005) — slides
  does NOT host LiveKit; consumes the SDK.
- layer = <layer>:
  - kernel + domain + usecase + api + adapter: standard layers.
  - adapter-livekit: backend-qualified per ADR-0105 Amd.3; LiveKit client binding.
  - worker: long-lived broadcast-session manager.
  - sdk: client library for cross-µservice (e.g., audience-view consumers).
- exemptions claimed: app.
```

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106; backend-qualified adapters per Amendment 3):

| BC | kernel | domain | usecase | api | adapter | -postgres | -valkey | -s3 | -loro | -pandoc | -weasyprint | -chromium-headless | -ffmpeg | -livekit | -imagemagick | -clamav | -opswat | -leptos-wasm | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| presentation | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | — | — | — | — | — | — | — | — | — | ✓ | — | ✓ | ✓ |
| slide | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | ✓ | — |
| slide-layout | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| text-box | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | — | — |
| shape | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | — | — |
| image | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | — | — | — | ✓ | ✓ | — | — | — | — | — | — |
| video-embed | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | — | ✓ | — | — | ✓ | — | — | — | — | — | — |
| audio-embed | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | — | ✓ | — | — | — | — | — | — | — | — | — |
| chart | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| table | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | — | — |
| equation | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| real-time-collaboration | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | ✓ | — | — | — | — | — | — | — | — | — | — | ✓ | ✓ | — |
| comments | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| version-history | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| animations | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | — | — |
| transitions | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | — | — |
| speaker-notes | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| presenter-view | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | — | — |
| audience-view | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | ✓ | — |
| broadcast-mode | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | ✓ | — | — | — | — | — | ✓ | ✓ | — |
| themes | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| templates | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| slide-sorter | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | — | — |
| master-slide-editor | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | — | — |
| layout-engine | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| import-export | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | ✓ | ✓ | — |
| accessibility | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| ai-design | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| ai-content-generation | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| embed-bridge | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| acl | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |

Total crates introduced by this µservice: **~210** across 31 BCs. Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `slides` MUST NOT import any other product µservice crate directly. Slides consumes:
- `sheets` via its SDK (live chart cell-range bind).
- `docs` via its SDK (doc-quote embed).
- `forms` via its SDK (in-deck poll embed).
- `drive` via its SDK (asset storage hierarchy).
- `messenger` via its SDK (broadcast-mode LiveKit signaling reuse).
- `social` via its SDK (publish-as-shorts).
- `mail` via its SDK (share-via-email).
- `ontology` via its SDK (object-type descriptors).
- `foundry-runtime` via its SDK (AI-design + AI-content-generation; T1/T2 gating).
- `tenancy` via its SDK (per-deck residency + ACL).
- `audit-chain` via its SDK (Ed25519 seal).
- `observability` via its SDK (SLI emission).
- `application` via composition (runs in app's hosting shell).

All cross-µservice flows go through SDK boundaries; no direct kernel imports across µservices. LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice slides` — dependency-direction
- `oya gate validate lean-a2 --microservice slides` — cross-product-refusal
- `oya gate validate port-location --microservice slides`
- `oya gate validate layer-correctness --microservice slides`
- `oya gate validate per-microservice-layout --microservice slides`
- `oya gate validate statelessness --microservice slides` — REST stateless (state in Postgres + Valkey + S3)
- `oya gate validate shardability --microservice slides` — editor sessions sharded by tenant_id
- `oya gate validate slides-pptx-roundtrip-subset --microservice slides` — round-trippable OOXML subset preserved
- `oya gate validate cedar-preview-required --microservice slides` — every save path exercises Cedar policy preview
- `oya gate validate wasm-bundle-sri --microservice slides` — every WASM chunk has SRI hash
- `oya gate validate reduced-motion-fallback-mandatory --microservice slides` — animations BC honors `prefers-reduced-motion`
- `oya gate validate ai-act-risk-class-stamp --microservice slides` — every ai-content-generation invocation carries a risk-class

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | Purpose |
|---|---|---|---|
| `DeckOpened` | editor open | observability, audit-chain | session tracking |
| `DeckSaved` | save action | audit-chain, ontology (Deck object link) | spec-store ack |
| `CollabMerged` | CRDT merge applied | observability | collab-merge-rate SLI |
| `CollabConflictSurfaced` | conflict UI shown | observability, audit-chain | conflict-rate SLI |
| `PresentModeStarted` | presenter starts present | observability, audit-chain | usage metric |
| `PresentModeEnded` | presenter ends present | observability, audit-chain | duration + attendee-count audit |
| `BroadcastStarted` | broadcast-mode opens | messenger (LiveKit room create), audit-chain | broadcast bridge |
| `BroadcastEnded` | broadcast-mode closes | messenger, audit-chain | session close |
| `AclChanged` | per-deck or per-slide ACL change | audit-chain, ontology | sharing audit |
| `ChartLinkBound` | chart-live-link added | sheets (read-grant), ontology | live-link audit |
| `ChartLinkRevoked` | sheets ACL revoke → cascade | audit-chain, observability | revocation audit |
| `AiDesignSuggested` | T0/T1 AI-design invocation | foundry-runtime, audit-chain | AI-assist usage |
| `AiContentGenerated` | T2 full-deck-from-prompt | foundry-runtime, audit-chain | AI-gen audit + risk-class |
| `ExportJobCompleted` | PPTX/PDF/MP4 export complete | observability, audit-chain | export SLI |
| `ImportJobCompleted` | PPTX/ODP/Keynote import complete | observability, audit-chain | import SLI |
| `AltTextSuggested` | T1 alt-text suggest invoked | foundry-runtime, audit-chain | accessibility metric |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `SheetsCellRangeUpdated` | sheets | chart | live-refresh chart |
| `SheetsAclRevoked` | sheets | chart | revoke chart access + cascade-audit |
| `DocsQuoteUpdated` | docs | embed-bridge | refresh quote text |
| `FormsPollResponseReceived` | forms | audience-view | render poll result |
| `MessengerLivekitRoomTerminated` | messenger | broadcast-mode | end broadcast session |
| `TenantSeatLimitUpdated` | tenancy | acl | refresh per-seat entitlement |
| `OntologyTypeDescriptorUpdated` | ontology | embed-bridge | hot-reload typed descriptors |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Presentation{tenant, deck_id, version_sha, author, saved_at, parent_version_sha, pack}` | `authored_by→TenantUser`, `derived_from→Presentation` | presentation → version-history | Ed25519 |
| `Slide{tenant, deck_id, slide_id, ordinal, layout_id}` | `belongs_to→Presentation`, `derived_from→Layout` | slide | Ed25519 |
| `Theme{tenant, theme_id, palette, typography}` | `used_by→Presentation` | themes | Ed25519 |
| `Asset{tenant, asset_id, kind, sha256, scan_verdict}` | `used_by→Presentation` | image / video-embed / audio-embed | Ed25519 |
| `BroadcastSession{tenant, session_id, deck_id, started_at, ended_at, attendee_count}` | `presents→Presentation` | broadcast-mode | Ed25519 |
| `AiContentGenerationEvent{tenant, request_id, prompt_hash, risk_class, decision}` | `generated_into→Presentation` | ai-content-generation | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `ObjectTypeDescriptor` (catalog) | embed-bridge | `where(domain in tenant.packs).descriptors()` for typed embed config |
| `Tenant` (catalog) | acl + presentation | `where(tenant_id=...).seat_limit, residency_pack` |
| `Pack` (catalog) | accessibility + ai-content-generation | `where(pack_id=...).ai_policy, reduced_motion_default` |
| `Asset` (catalog) | image / video-embed | `where(asset_id=...).scan_verdict` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Google Slides | Slides (Workspace) | drag-drop canvas; collab; chart-live-link to Sheets; present-mode; broadcast | `support.google.com/docs#slides` |
| Microsoft PowerPoint Web | PowerPoint for the web (M365) | co-authoring; PPTX fidelity round-trip; PowerPoint Live (broadcast) | `support.microsoft.com/powerpoint` |
| Apple Keynote | Keynote (iCloud) | animation engine + Magic Move; broadcast (Keynote Live) | `apple.com/keynote` |
| Pitch | Pitch.com | collab-first; templates; analytics | `pitch.com/help` |
| Beautiful.ai | Beautiful.ai | AI-design + smart-layout | `help.beautiful.ai` |
| Canva Presentations | Canva | template gallery + brand kit + remote-control present-mode | `canva.com/help/presentations` |
| Prezi | Prezi Next | non-linear / zooming present-mode | `support.prezi.com` |
| Gamma | Gamma.app | AI-content-generation T2 (full-deck-from-prompt) | `help.gamma.app` |
| Tome | Tome | AI narrative slides + storytelling | `help.tome.app` |
| Decktopus | Decktopus | AI-deck-builder | `help.decktopus.com` |
| ONLYOFFICE Presentation | ONLYOFFICE | self-host; PPTX fidelity | `helpcenter.onlyoffice.com/presentation` |
| LibreOffice Online Impress | Impress | ODF native; PPTX fidelity | `documentation.libreoffice.org/impress` |
| Slidebean | Slidebean | startup-pitch templates; analytics | `slidebean.com/help` |
| Visme | Visme | infographic-class slides | `help.visme.co` |
| Mentimeter | Mentimeter | audience-engagement (polls, Q&A) subset | `help.mentimeter.com` |
| Slido | Slido | Q&A + polling subset | `slido.com/help` |

Key parity gaps to close (ordered by priority for M03 preview milestone):

1. **PPTX round-trip fidelity (round-trippable subset)** — PowerPoint Web and ONLYOFFICE are reference standard; oyatie target = 95% of round-trippable OOXML PresentationML subset preserved byte-for-byte on import → export → reimport.
2. **AI-content-generation under EU AI Act risk-class** — Gamma + Tome ship T2 generation broadly; oyatie differentiates with risk-class enforcement at generation request (ADR-SLIDES-0006).
3. **Per-slide ACL granularity** — Google Slides offers deck-level ACL only; oyatie target = per-slide named-block ACL (ADR-SLIDES-0007) → unique differentiator.
4. **Chart-live-link to sheets with revocation cascade** — Google Slides ↔ Sheets has live-link but inconsistent revocation; oyatie target = end-to-end audited revocation (ADR-SLIDES-0008).
5. **Broadcast-mode reusing messenger LiveKit** — Keynote Live + PowerPoint Live + Google Meet-bridged are competitor options; oyatie reuses messenger LiveKit (ADR-SLIDES-0005).
6. **Reduced-motion + color-blind-safe** baseline-on; competitors mostly accessibility-opt-in.

Detailed quantitative comparison in `competitor-parity-matrix.md`.

## Performance Targets

(Duplicated from §"Non-Functional Requirements" for ease of citation.)

Error budget:
- Monthly error budget for editor REST: 0.05% (≈22 min/month).
- Monthly error budget for WS collab + broadcast-mode signaling: 0.1%.
- Burn-rate alarms: 14.4× burn over 1h for editor REST; 6× burn over 6h for WS gateway.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Rationale:
- Editor session state, deck spec metadata, ACL: `postgres` (Citus-distributed by tenant_id).
- Deck content snapshots + assets (images, video, audio): `s3`.
- Ephemeral collab CRDT state, presenter-view session, broadcast-mode lease: `valkey` (per-cell cluster; reconstructable from Postgres on cold-start).
- Static assets (WASM bundles + theme gallery + template gallery): `cdn` (global edge cache; per-tenant key partitioning).
- Live broadcast media: LiveKit SFU nodes (consumed via messenger µservice; NOT hosted in slides).

**Active-active compatibility**: `stateless-compatible` for REST/SDK; `single-writer-compatible` for collab CRDT (one WS gateway pod owns active sessions for a given deck; lease-coordinated via Valkey); `single-writer-compatible` for broadcast-mode lease.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active editor sessions | 10,000 | 200,000 | WS connection count > 70k OR cursor-sync p99 > 150ms |
| Decks per tenant | 1,000 | 100,000 | Postgres shard fill > 80% |
| Concurrent collab-server WS connections | 50,000 | 500,000 | WS gateway pod CPU > 70% |
| Save round-trips/sec (cluster-wide) | 1,000 | 100,000 | Postgres write rate > 80% |
| Concurrent broadcast viewers (per deck) | 500 | 5,000 | LiveKit SFU cascade |
| Concurrent broadcast sessions (cluster) | 100 | 5,000 | messenger LiveKit cluster scale-out |
| Export jobs/sec (PPTX+PDF+MP4) | 10 | 200 | gVisor export-worker pod CPU > 70% |
| AI-content-generation requests/sec | 5 | 200 | foundry-runtime backpressure |

Scale-out policy:
- Editor REST: stateless HPA on CPU > 70%; min 4 replicas; max 50.
- WS gateway (real-time-collaboration-worker): stateful per active editor session; lease-coordinated via Valkey; HPA on WS connection count; min 3 replicas; max 100.
- Broadcast-mode worker: stateful per broadcast session; lease via Valkey; HPA on session count; min 2 replicas; max 50.
- Export workers (PPTX/PDF/MP4 in gVisor): job-queue HPA; min 4 replicas; max 100; per-job CPU + memory budgets.
- Postgres + Citus on `tenant_id`; linear shard addition.
- Valkey per-cell cluster; cell-local CRDT + lease state.
- CDN global; per-pack edge nodes.

Cross-region story:
- M03 preview launch: single KR region.
- Post-M03: per-pack residency activation; CDN edges per pack; editor session state pinned to pack; LiveKit nodes pack-pinned via messenger.

Sharding:
- Postgres on `tenant_id`; deck content snapshots S3 partitioned by `(tenant_id, deck_id)`.
- Valkey per-cell cluster; cell-local CRDT state.
- WS gateway: consistent-hash on `deck_id` ensures collab participants land on same gateway pod.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Drag-drop placeholder onto slide; save; emit canonical slide-spec.v1.json (semantically identical) | `cargo nextest run -p oya-slides-slide-domain --test test_authoring_roundtrip` |
| AC-02 | Import PPTX (round-trippable subset) → emit → reimport byte-identical for 100 reference decks | `cargo nextest run -p oya-slides-import-export-domain --test test_pptx_roundtrip_subset` |
| AC-03 | Editor open with pending changes; network disconnect; local buffer persists; resume without loss on reconnect | `tests/e2e/offline-buffer-resume.rs` |
| AC-04 | Per-pack jurisdiction switch; overlay-resolved view renders; base reachable | `cargo nextest run -p oya-slides-accessibility-domain --test test_per_pack_overlay` |
| AC-05 | AI-content-generation T2 deck-from-prompt; valid: opens in editor; invalid: precise per-slide error | `tests/e2e/ai-content-validation.rs` |
| AC-06 | Two users edit same deck concurrently; Loro CRDT merge applies non-conflicting; conflict UI for overlap; never silent loss | `cargo nextest run -p oya-slides-real-time-collaboration-domain --test test_no_silent_overwrite` |
| AC-07 | Editor in us-healthcare pack; author touches PHI-class field; visual data_class marker + Cedar policy preview before save | `cargo nextest run -p oya-slides-acl-domain --test test_phi_marker_visible` |
| AC-08 | Per-deck ACL check on editor open; per-slide ACL check on slide-render; audit row emitted | `cargo nextest run -p oya-slides-acl-domain --test test_per_slide_acl` |
| AC-09 | Present-mode 60fps transition; p95 ≤ 50ms over 50-slide deck | `tests/load/present-mode-frame-budget.js` |
| AC-10 | Save round-trip ≤ 100ms p95 | `tests/load/save-roundtrip.js` |
| AC-11 | Chart-live-link refresh: sheet cell change → chart re-render p95 ≤ 200ms | `tests/integration/chart-live-link.rs` |
| AC-12 | WASM bundle SRI: every chunk hash verifies; mismatch refuses load | `cargo nextest run -p oya-slides-slide-adapter-leptos-wasm --test test_sri` |
| AC-13 | `oya gate validate per-microservice-layout --microservice slides` exit 0 | ADR-0131 lane |
| AC-14 | `oya gate validate authority-cohesion` exit 0; HG-SLIDES gate green | ADR-0123 lane |
| AC-15 | `oya gate validate slides-pptx-roundtrip-subset --microservice slides` exit 0 | new lane |
| AC-16 | `oya gate validate ai-act-risk-class-stamp --microservice slides` exit 0 — every ai-content-generation invocation carries a risk-class | new lane |
| AC-17 | `oya gate validate reduced-motion-fallback-mandatory --microservice slides` exit 0 — animations BC honors `prefers-reduced-motion` | new lane |
| AC-18 | Broadcast-mode: LiveKit signaling drop mid-present → graceful degradation to non-broadcast present-mode; audience reconnect on signaling recovery | `tests/e2e/broadcast-degraded.rs` |
| AC-19 | Chart-live-link revocation cascade: sheets ACL revoke → chart access revoke audit row emitted within 5s | `tests/integration/chart-revocation-cascade.rs` |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Vector rendering tier — Skia (via tiny-skia + WASM) vs canvas-2d in Leptos vs SVG vs WebGL? Bias: SVG baseline (accessibility + DOM debugging) + canvas-2d for present-mode 60fps + WebGL fallback if 60fps fails. | council-design-system | ADR-SLIDES-0002 |
| 2 | PPTX library — pure-Rust pptx-rs (preview-grade) vs Pandoc bridge vs calamine-derived approach? Bias: Pandoc bridge for import (best-effort) + bespoke OOXML serializer for export (round-trippable subset). | axis-workspace + council-architecture | ADR-SLIDES-0003 |
| 3 | PDF renderer — WeasyPrint (Python) vs Chromium-headless (Node-less) vs typst-pdf (Rust-native)? Bias: WeasyPrint for PDF/A-1b path + Chromium-headless fallback for complex CSS; typst-pdf evaluated subsequent-to-M03-completion. | council-architecture | ADR-SLIDES-0003 |
| 4 | Broadcast-mode AV transport — reuse messenger LiveKit verbatim vs slides-owned LiveKit cluster? Bias: reuse — see ADR-SLIDES-0005. | axis-workspace + axis-realtime | ADR-SLIDES-0005 |
| 5 | Per-slide ACL granularity — slide-level only vs named-block within slide? Bias: named-block to differentiate from Google Slides. | council-architecture + ops-security | ADR-SLIDES-0007 |
| 6 | AI-content-generation T2 risk-class default — refuse on Annex III high-risk context vs allow-with-watermark vs require-human-review? Bias: refuse high-risk by default; per-pack override. | ops-security + council-architecture | ADR-SLIDES-0006 |
| 7 | Chart-live-link consistency model — strong (block save until refresh confirmed) vs eventual (refresh in background)? Bias: eventual with explicit stale-marker UI. | axis-workspace | ADR-SLIDES-0008 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0065 | Docs-as-Leptos webapp | inherited; slides canvas adopts Leptos |
| ADR-0105 | 13-layer enum + backend-qualified adapters Amd.3 | layer authority |
| ADR-0106 | Application → usecase rename | applied for new crates |
| ADR-0123 | Hyperscaler maturity claim gate | HG-SLIDES registers here |
| ADR-0135 | Connect dissolution | slides is one of 5 net-new µservices unbundled from connect; no legacy |
| ADR-0139 | Agentic SLO-gated promotion | slides SLO promotion gates this µservice |
| ADR-0131 | Per-microservice flat layout | this µservice authored natively under it |
| ADR-0132 | Product-suite-and-bundle dissolution | slides is a single-concern µservice, not a suite |
| ADR-0133 | Industry-best-practice conformance | slides competitor parity tracked here |
| ADR-0134 | Strangler-fig migration | not applicable; slides is net-new |
| ADR-0140 | Cedar policy enforcement | acl BC built on this |
| ADR-WS-0001 | Loro CRDT library | structural parent of ADR-SLIDES-0001 |
| ADR-WS-0003 | Leptos WASM substrate | structural parent of ADR-SLIDES-0002 |
| ADR-SLIDES-0001 | CRDT library — Loro 1.x | slides-specific application |
| ADR-SLIDES-0002 | Rendering canvas substrate | Leptos canvas + SVG baseline + canvas-2d + WebGL fallback |
| ADR-SLIDES-0003 | Export pipeline fidelity | PPTX round-trippable subset; PDF/A; deterministic MP4 |
| ADR-SLIDES-0004 | Animation engine + reduced-motion | WCAG 2.2 SC 2.3.3 |
| ADR-SLIDES-0005 | Broadcast-mode + LiveKit reuse | messenger LiveKit reuse pattern |
| ADR-SLIDES-0006 | AI-design + content-generation bounds | EU AI Act risk-class |
| ADR-SLIDES-0007 | Per-slide ACL granularity | named-block-level Cedar |
| ADR-SLIDES-0008 | Chart-live-link to sheets | eventual consistency + revocation cascade |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `slides` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `slides` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 3 module pin(s) across 1 context(s).
- Scaling input: `per_user` with cell placement `Tier-3` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
