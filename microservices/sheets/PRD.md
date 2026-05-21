---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-sheets
microservice: sheets
status: Accepted
sales_segment: hero-product
tier: external-facing
milestone_first_ship: M03-sheets-preview
bominal_source: []
net_new: true
related_adrs: [ADR-0056, ADR-0065, ADR-0103, ADR-0105, ADR-0106, ADR-0110, ADR-0123, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0140 (retired per ADR-0145), ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/microservices/sheets.json, /specs/per-microservice-flat-layout.json]
related_unbundle_adr: ADR-0135
sibling_products:
  - microservices/workflow-studio/ (CRDT collab pattern; Loro alignment)
  - microservices/cell/ (per-workbook cell substrate)
  - microservices/ontology/ (Workbook/Sheet/Cell/Range/Chart object types)
  - microservices/foundry/ (AI-formula + smart-fill bridge)
date: 2026-05-17
owner_team: axis-sheets + council-design-system
doc_status: published
---

# PRD-sheets: Sheets — Spreadsheet + Structured-Data Editor

## Purpose

The `sheets` µservice is oyatie's **spreadsheet + structured-data authoring product** — a Google-Sheets / Microsoft-Excel-Web / Airtable-grid / Notion-database / Coda-table-class hero product per ADR-0135. Sheets is the **end-user surface** of the spreadsheet/grid product class; it is NET-NEW per ADR-0135 (no `oya-connect-sheets-*` legacy crates exist; no migration-from-connect). Sheets owns: the cell-grid editor canvas, the workbook/sheet/cell/range/formula data model, the formula recalculation engine (dependency-graph + parallel-safe), pivot tables, charts, conditional formatting, data validation, real-time collaborative editing (CRDT-based, aligned with workflow-studio Loro per ADR-WS-0001 + this ADR-SHEETS-0001), comments + notes, version history, sharing + per-range ACL, XLSX/ODS/CSV/TSV/JSON import/export, AI-formula and AI-fill (T1/T2 tiers; EU AI Act-bounded), and connected-sheets queries.

Sheets is **NOT a substrate**. It is a tenant-facing product surface with five distinct user personas (business power user, business analyst, financial-modelling specialist, vertical specialist, agentic developer role). The cell grid is the second-largest Leptos application in oyatie (sibling to workflow-studio's visual canvas, per ADR-0065 Rust-WASM SSR + browser-WASM hybrid). The canonical source of truth is the workbook's structured cell graph; the visual grid derives from the graph, never vice-versa.

This µservice operates at the **application** layer of the 12-layer Workflow + Ontology architecture (per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`): Sheets consumes ontology object-type descriptors for typed-column configuration (Airtable / Notion-database parity); emits cell-edit events to the workflow-engine event-bus (for sheet-edit-triggers-workflow); bridges to foundry-runtime for AI-formulas + smart-fill; routes through tenancy for per-seat licensing; bridges to drive for workbook storage hierarchy; embeds into docs (live cell ranges) and slides (charts).

This µservice is **shared substrate AND hero product** simultaneously: the cell-grid + formula-engine + recalc-engine are shared substrate consumed by every oyatie product that needs grid-class structured data (forms responses, workflow-engine state observation, cost-budget dashboards); the editor shell is end-user product packaged as the Sheets brand.

## Tenant Value

- **Tenant Outcome 1 — Time to first valid workbook under 5 minutes.** Business power user opens Sheets, enters values into a 10×10 grid, applies a SUM formula, sees recalc, saves. No code; no terminal; no documentation reading required.
- **Tenant Outcome 2 — Sheet-open p95 ≤ 400ms cold (10k cells) / 150ms warm; cell-edit-render p99 ≤ 50ms.** Editor opens fast even on cold-cache; per-cell render budget hit at 60fps for live editing. Competitive with Google Sheets + Microsoft Excel Web.
- **Tenant Outcome 3 — Recalc 100k-cell sheet p95 ≤ 1s; 1M-cell workbook p95 ≤ 10s.** Formula engine parallel-safe; dependency graph incremental; tenant can build financial models that exceed Excel-Web's responsiveness threshold.
- **Tenant Outcome 4 — Collaborative editing without silent loss.** Two business users editing same workbook simultaneously: CRDT merge applies non-conflicting edits; conflicting edits surface explicit conflict UI; no last-writer-wins. Cursor sync p99 ≤ 150ms aligned with workflow-studio collab budget.
- **Tenant Outcome 5 — XLSX round-trip fidelity (best-effort fidelity per ADR-SHEETS-0007).** Tenant imports a Microsoft Excel workbook, edits in Sheets, exports back to XLSX; per the named-limit list (no VBA, image fidelity downgrade tolerance), the round-trip preserves formulas, formatting, charts, pivot tables, data validation, named ranges, and comments.
- **Tenant Outcome 6 — Per-pack data-class markers + per-range ACL.** Healthcare tenants in pack-us-healthcare see PHI markers on patient-id columns; financial tenants see SECRET markers on PII columns; per-range ACL (named-ACL granularity per ADR-SHEETS-0006) allows column-level read/edit permission.
- **Tenant Outcome 7 — AI-formulas + smart-fill (T1 advisory; T2 EU-AI-Act-bounded).** Tenant prose ("calculate average revenue per region for Q3") drafts a candidate formula; smart-fill infers column patterns from 3 cells. Bridges to foundry-runtime; T1 advisory (human accepts); T2 auto-apply gated by Cedar + ChangeSet review.
- **Tenant Outcome 8 — Connected-sheets (external-source queries).** Tenant runs SQL-class queries against external databases (Postgres / BigQuery-equivalent / Snowflake-equivalent) via foundry-runtime; results materialize as cell ranges with refresh policy. Competitive with Google Sheets Connected Sheets.
- **Internal Outcome 9 — Grid substrate for every structured-data oyatie product.** Forms response-data, workflow-engine cost-budget dashboards, foundry-eval metric grids, observability metric panels — all materialize via the sheets cell-grid substrate via per-product SDKs.
- **Internal Outcome 10 — Embed bridge into docs + slides.** Live cell-range embeds in docs (per docs ADR family); chart embeds in slides; embed-source-of-truth is sheets canonical cell graph.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | business power user | to enter values into a cell grid and apply formulas | I can author a spreadsheet without writing code | cell-grid | Must |
| FR-02 | business power user | to drag-fill a formula across a range with relative+absolute reference semantics | I can model patterns at scale | formula-engine | Must |
| FR-03 | financial analyst | to use ≥400 functions covering math/logical/lookup/statistical/financial/text/date/array | I can match Excel modelling | formula-engine | Must |
| FR-04 | Sheets | to recalculate dependent cells incrementally on each edit (dep-graph topo-sort; parallel-safe) | tenant sees results within 50ms p99 | recalc-engine | Must |
| FR-05 | Sheets | to load + save workbook as canonical JSON-Sheet AND XLSX (best-effort fidelity per ADR-SHEETS-0007) | tenant mixes oyatie + Excel-Web workflows | import-export | Must |
| FR-06 | two users | to edit the same workbook concurrently with CRDT merge (Loro 1.x per ADR-SHEETS-0001) | no silent loss; explicit conflict on overlap | collab-crdt | Must |
| FR-07 | tenant analyst | to author conditional-formatting rules on cell ranges | I surface anomalies visually | formatting | Must |
| FR-08 | tenant analyst | to author pivot tables over data ranges | I summarise large datasets | pivot-tables | Must |
| FR-09 | tenant analyst | to render charts (bar/line/pie/scatter/area/combo/sparkline) over data ranges | I visualise trends | charts | Must |
| FR-10 | tenant analyst | to apply data-validation rules (dropdown/range/custom-formula) to cells | I enforce input quality | data-validation | Must |
| FR-11 | tenant operator | to share workbook with view/comment/edit permissions and per-range named-ACL | I control access per column/range | sharing-acl | Must |
| FR-12 | tenant analyst | to author cell comments + threaded notes | I collaborate asynchronously | comments | Must |
| FR-13 | tenant analyst | to navigate workbook version history and restore prior versions | I recover from mistakes | version-history | Must |
| FR-14 | tenant developer | to draft formulas via AI from natural-language prose; review before accept | low-floor AI-assist | ai-formula | Should (GA) |
| FR-15 | tenant developer | to smart-fill column patterns from N seed examples (3-5 cells) | repetitive data entry automated | ai-fill | Should (GA) |
| FR-16 | tenant analyst | to query an external data source (SQL-class) and materialize as a cell range | I integrate live database data | connected-sheets | Should (stable) |
| FR-17 | Sheets | to render data-class markers (PII/PHI/SECRET) on annotated cells/columns | author sees data sensitivity before share | cell-grid | Must |
| FR-18 | tenant operator | to author named ranges across workbook scope | I write reusable formulas | named-ranges | Must |
| FR-19 | tenant operator | to author sheet-edit triggers (cross-µservice to workflow-engine) | sheet changes drive automation | trigger-bridge | Should (GA) |
| FR-20 | tenancy | to enforce per-seat licensing via Cedar at workbook open | tenant honors seat count | license-gate-cedar | Must |
| FR-21 | Sheets | to persist edit buffer locally during network disconnect | resume without loss on reconnect | cell-grid | Must |
| FR-22 | Sheets | to expose embed-bridge for docs (cell ranges) and slides (charts) | cross-product live embedding | embed-bridge | Should (stable) |
| FR-23 | tenant developer | to export evidence of authored workbooks for audit | compliance posture verifiable | cell-grid | Should (stable) |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | p999 | Notes |
|---|---|---|---|---|---|
| Sheet-open cold (10k cells, CDN-cached) | 200ms | 400ms | 600ms | 1.5s | GA target per scope |
| Sheet-open warm | 80ms | 150ms | 250ms | 500ms | GA target |
| Cell-edit-render | 16ms | 30ms | 50ms | 100ms | 60fps cap |
| Recalc 100k-cell sheet | 400ms | 1s | 1.5s | 3s | GA target |
| Recalc 1M-cell workbook | 4s | 10s | 15s | 30s | GA target |
| Save round-trip (emit → cell µservice → ack) | 50ms | 80ms | 100ms | 250ms | GA target |
| Collab CRDT merge (cursor sync) | 50ms | 100ms | 150ms | 300ms | aligned workflow-studio collab |
| XLSX export (100k-cell workbook) | 2s | 4s | 5s | 10s | GA target |
| XLSX import (100k-cell workbook) | 2s | 4s | 5s | 10s | GA target |
| Chart render | 80ms | 150ms | 200ms | 400ms | GA target |
| Formula function-library call (single fn) | 50µs | 150µs | 500µs | 2ms | for typical SUM/AVG/VLOOKUP |
| AI-formula draft (LLM-assist) | 1.5s | 2.5s | 3s | 8s | depends on foundry-runtime |
| Smart-fill inference (3-cell seed) | 200ms | 500ms | 800ms | 2s | depends on foundry-runtime T1 |
| Connected-sheets refresh (10k rows) | 1s | 3s | 5s | 15s | depends on external source |
| WebSocket gateway round-trip | 10ms | 30ms | 50ms | 200ms | collab + recalc-progress streaming |
| Active editor sessions per region | — | — | 100,000 | — | XL tier; horizontal scale via session sharding |
| Concurrent collab editors per workbook | — | — | 10 | — | beyond 10 users: degraded UX |

### Security

- OIDC tenant-scoped at every REST/WebSocket entry; Sheets refuses opens without resolvable tenant identity.
- Per-seat license-gate Cedar fragment enforced at workbook open; refusal emits `sheets_per_seat_license_denied` audit row.
- Strict CSP (`default-src 'self' https://cdn-<pack>.oyatie.dev; script-src 'self' 'wasm-unsafe-eval' 'nonce-<random>'`) — no inline scripts except WASM bootstrap nonce; no eval.
- XSS-free architecture: cell text rendered via virtual-DOM text nodes; never `innerHTML`. Anti-pattern `per_tenant_branding_mid_render` is forbidden.
- Per-tenant CDN cache key: CDN partitions cache by `(tenant_hash, pack, version)`; no cross-tenant cache pollution.
- WebSocket auth: OIDC token validated at WS upgrade; tenant binding rebound at WS message dispatch.
- AI-formula content never trusted: formula emitted by AI-formula passes through full schema + Cedar policy preview before save. Anti-pattern: bypassing the validation pipeline for AI-formula drafts.
- XLSX import sandboxing: import pipeline runs in gVisor user-mode sandbox; ClamAV + OPSWAT scan every uploaded XLSX file; embedded VBA scripts excluded per ADR-SHEETS-0007.
- Per-tenant workbook isolation: workbook + cell state in Postgres scoped by tenant_id (Citus partition); cross-tenant access forbidden by RLS + Cedar.
- WASM bundle integrity: subresource-integrity (SRI) hashes for every WASM chunk in HTML; mismatch triggers refuse-to-load with audit row.
- Per-range ACL granularity (ADR-SHEETS-0006): named-ACL Cedar policy enforces column/range-level read/edit permission.

### Audit + Compliance

- Every cell edit emits a `cell_edit` audit-chain seal (Ed25519); seal includes `(tenant_id, workbook_id, sheet_id, cell_ref, old_value_hash, new_value_hash, author_identity, parent_seal_sha, timestamp)`. Per Bominal ADR-0028.
- Every share-permission change emits `share_acl_changed` seal.
- Every formula-engine version upgrade emits `formula_engine_version_changed` seal.
- Editor session events (open, save, conflict, license-gate, sharing change) emitted to engine event-bus as typed events per `contracts/asyncapi/sheets-events.yaml`.
- Per-tenant `jurisdiction_code` enforced via Cedar policy; cross-pack collab forbidden.
- AI-formula call: foundry-runtime tenant binding inherited; LLM choice + prompt + completion archived for 90d for audit.
- Per-seat license events emitted: `license_gate_emitted{tenant, principal, seat_count_used, seat_count_limit, decision}`.

### Availability + SLO

- Editor REST availability target: 99.95% monthly (GA); 99.9% (stable); 99.5% (preview).
- WebSocket gateway availability: 99.9% monthly (collab + recalc-progress streams).
- XLSX export pipeline availability: 99.5% monthly (acceptable degradation; manual CSV fallback).
- AI-formula availability: 99.5% monthly (acceptable degradation; Sheets works without AI-formula).
- RTO ≤ 1800s for editor-rest per manifest `dr.rto_p99_seconds=1800`; hot editor failover may recover faster when cell capacity is healthy.
- RPO ≤ 120s for cell-edit state per manifest `dr.rpo_p99_seconds=120`.
- Self-observability: Sheets emits its own SLO via observability µservice; burn-rate alarms feed Grafana OnCall.

### Data residency

- Workbook metadata, cell storage, edit buffer, collab CRDT state, AI-formula prompts, and per-seat license attribution inherit the tenant's `jurisdiction_code` per ADR-0117. Postgres + Valkey + Arrow/Parquet large-sheet storage + S3 snapshots are per-pack region-pinned.
- CDN static assets are global (no PII; spec schema + design-system primitives + WASM bundles); per-pack CDN edge keys segregate tenant-rendered content where applicable.

### DR Posture (ADR-0343)

- RTO/RPO target: manifest `dr` declares `rto_p99_seconds=1800` and `rpo_p99_seconds=120`. EU-AI-ACT-2024-HIGH-RISK (1800s/300s), HIPAA-2024 (3600s/300s), SOC2-T2 (14400s/900s), DORA continuity expectations, and ISO27001-2022 (14400s/3600s) leave the effective sheets bound at 1800s RTO and 120s RPO.
- failover_runbook: `runbooks/dr-failover.md`; manifest backup substrate is `postgres_wal_g`, `object_storage_versioned`, and `valkey`.
- multi_region_active_active: true, with manifest replication shape `active-active-multi-az-cross-region-warm`; global static WASM stays PII-free and workbook content stays pack-pinned.
- WHY: spreadsheet edits can drive financial, healthcare, and AI-assisted decisions, so tenants need recoverable cell state and visible conflict handling after a cell fault.

### Capacity Model (ADR-0340)

- Per-tenant baseline: manifest `capacity_model` declares 0.18 vCPU, 512Mi RAM, 40Gi storage, 3 Valkey connections, 3 Postgres connections, and 6 outbound HTTP connections per tenant.
- Scaling dimension: `per_query`; formula recalculation, large-sheet reads, chart queries, and export paths dominate resource use.
- Cell placement class: Tier-3, matching manifest `capacity_model.cell_placement_class`, because sheets is a high-throughput application/query surface rather than tenant-customer code execution.
- Autoscaling boundaries: visual-grid REST min 4 / max 50, WS gateway and recalc workers follow `capacity-model.md` replica formulas, and recalc/export queues back-pressure before tenant workloads can exhaust shared cell capacity.
- WHY: sheets load is driven by active workbooks, cell count, and recalc fan-out, not just document count, so scaling is tied to session, formula, and import/export queues.

### Sustainability + Cost Attribution (ADR-0344)

- Every audit-chain row emits `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, and `region` for cell edits, share ACL changes, formula-engine upgrades, session events, AI-formula calls, imports/exports, and connected-sheets refresh.
- Provider routing affected by carbon: no for EU-AI-Act Annex III/high-risk, HIPAA, formula-correctness, license-gate, or realtime collab paths; yes for scheduled recalc, connected-sheets refresh, XLSX import/export, and AI advisory queues when tenant policy allows.
- Per-tenant transparency surface: FinOps portal shows workbook sessions, hot/cold cell storage, recalc CPU, XLSX worker time, AI-formula calls, connected refreshes, and CDN/WASM egress by tenant/capability/provider/cell/compliance_pack.
- WHY: sheets mixes low-latency collaborative state with compute-heavy recalc and AI, so CSRD, SB-253, and SEC climate reporting need attribution that does not alter regulated calculation behavior.

### API Versioning Posture (ADR-0342)

- Public API version model: YYYY-MM-DD carrier triplet via `Oyatie-Version` header, `/v/<YYYY-MM-DD>` URL prefix, and proto3 `oyatie_version` field for workbook, sheet, formula, recalc, import/export, sharing, embed, and event contracts.
- SDK semver model: major.minor.patch for browser-WASM editor SDKs, automation SDKs, and cross-product embed SDKs.
- Support window: last N=3 public API versions for at least 180 days, with formula-engine and XLSX schema compatibility called out in deprecation notices.
- Per-tenant pinning supported: yes, especially for regulated workbook validation and formula-engine certification windows.
- Internal-mesh exemption: yes; direct gRPC to drive, docs, slides, workflow, foundry, and cell remains exempt under ADR-0145 when internal-only.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates), Sheets uses: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-valkey`, `adapter-arrow`, `adapter-parquet`, `adapter-s3`, `adapter-loro`, `adapter-calamine`, `adapter-rust-xlsxwriter`, `adapter-clamav`, `adapter-opswat`, `adapter-cdn`, `adapter-leptos-wasm`, `rest`, `worker`, `sdk`, `app`. Browser-WASM artifacts compiled from the `app` layer per ADR-0065.

| BC | Crate family (BNF v4.1 + ADR-0105) | Purpose | Key entities |
|---|---|---|---|
| `cell-grid` | `oya-sheets-cell-grid-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-leptos-wasm,rest,sdk,app}` | Drag-fill cell-grid canvas, viewport, selection, headers, freeze panes | `Workbook`, `Sheet`, `Cell`, `Range`, `Selection`, `ViewportState` |
| `formula-engine` | `oya-sheets-formula-engine-{kernel,domain,usecase,api,adapter,sdk}` | ≥400-function library (math/logical/lookup/statistical/financial/text/date/array); pure parser + evaluator | `Formula`, `FunctionCall`, `EvalContext`, `EvalResult`, `FormulaError` |
| `recalc-engine` | `oya-sheets-recalc-engine-{kernel,domain,usecase,api,adapter,worker,sdk}` | Dependency-graph builder; topological recalc; parallel-safe; incremental | `DepGraph`, `DirtySet`, `RecalcPlan`, `RecalcFrame` |
| `formatting` | `oya-sheets-formatting-{kernel,domain,usecase,api,adapter}` | Number/date/currency/percent/custom formats; conditional formatting rules | `Format`, `ConditionalRule`, `FormatPalette` |
| `pivot-tables` | `oya-sheets-pivot-tables-{kernel,domain,usecase,api,adapter}` | Pivot config + aggregation evaluator | `Pivot`, `PivotAxis`, `PivotAggregator` |
| `charts` | `oya-sheets-charts-{kernel,domain,usecase,api,adapter,adapter-leptos-wasm,sdk}` | Bar/line/pie/scatter/area/combo/sparkline; custom Leptos canvas renderer | `Chart`, `ChartSeries`, `ChartAxis`, `RenderedChart` |
| `data-validation` | `oya-sheets-data-validation-{kernel,domain,usecase,api,adapter}` | Dropdown/range/custom-formula validation | `ValidationRule`, `ValidationError` |
| `collab-crdt` | `oya-sheets-collab-crdt-{kernel,domain,usecase,api,adapter,adapter-valkey,adapter-loro,worker,sdk}` | Loro 1.x CRDT merge engine for cell ops; conflict surfacer; WebSocket gateway | `CrdtState`, `MergeOp`, `Conflict`, `EditorSession` |
| `import-export` | `oya-sheets-import-export-{kernel,domain,usecase,api,adapter,adapter-calamine,adapter-rust-xlsxwriter,adapter-clamav,adapter-opswat,worker,sdk}` | XLSX/ODS/CSV/TSV/JSON-Sheet import/export; sandboxed; AV-scanned | `ImportContext`, `ExportContext`, `ExportFidelityTier` |
| `large-sheet-storage` | `oya-sheets-large-sheet-storage-{kernel,domain,usecase,api,adapter,adapter-arrow,adapter-parquet,adapter-s3}` | Hybrid postgres+Arrow-Parquet substrate for >100k cells; S3 snapshot | `ColumnarBlock`, `SheetSnapshot`, `HotColdBoundary` |
| `sharing-acl` | `oya-sheets-sharing-acl-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | View/comment/edit + per-range named-ACL | `Share`, `RangeAcl`, `AclDecision` |
| `comments` | `oya-sheets-comments-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Cell comments + threaded notes; mention bridge | `Comment`, `Thread`, `Mention` |
| `version-history` | `oya-sheets-version-history-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-s3,sdk}` | Workbook snapshots + named-version pointers; restore | `Snapshot`, `VersionPointer`, `RestoreContext` |
| `named-ranges` | `oya-sheets-named-ranges-{kernel,domain,usecase,api,adapter}` | Workbook-scope + sheet-scope named-range registry | `NamedRange`, `RangeScope` |
| `ai-formula` | `oya-sheets-ai-formula-{kernel,domain,usecase,api,adapter,sdk}` | Bridge to foundry-runtime; prose→formula; smart-fill | `AiFormulaDraft`, `SmartFillInference` |
| `connected-sheets` | `oya-sheets-connected-sheets-{kernel,domain,usecase,api,adapter,worker,sdk}` | External SQL-source query + materialize as cell range | `ConnectedQuery`, `RefreshPolicy`, `MaterializedRange` |
| `trigger-bridge` | `oya-sheets-trigger-bridge-{kernel,domain,usecase,api,adapter,sdk}` | Sheet-edit-triggers-workflow event bridge | `EditTrigger`, `WorkflowDispatch` |
| `embed-bridge` | `oya-sheets-embed-bridge-{kernel,domain,usecase,api,adapter,sdk}` | Live cell-range embed in docs; chart embed in slides | `EmbedDescriptor`, `EmbedRefreshPolicy` |
| `license-gate-cedar` | `oya-sheets-license-gate-cedar-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` | Per-seat Cedar enforcement at workbook open + per-action | `SeatLicense`, `LicenseDecision`, `EntitlementClaim` |

Naming justification — `cell-grid`:

```
NAME: oya-sheets-cell-grid-<layer>
JUSTIFICATION:
- microservice = sheets: hero product µservice (per-microservice flat layout, ADR-0131).
  Net-new per ADR-0135; no legacy oya-connect-sheets-* crates.
- bc-tokens = cell-grid: primary BC for the cell-grid canvas, viewport, selection, headers,
  freeze panes, drag-fill. ADR-0056 v4.1 BC-optionality honoured (18 sibling BCs exist).
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical enum.
  - kernel: port-trait + entity types (Workbook, Sheet, Cell, Range, Selection, ViewportState). Zero I/O.
  - domain: pure cell-graph algebra; deterministic dirty-marking math.
  - usecase (per ADR-0106): orchestrators driving cell state transitions.
  - api: protocol-neutral typed I/O contracts.
  - adapter: protocol-neutral implementations.
  - adapter-postgres: workbook metadata + cell storage with materialized-view caches; backend-qualified per ADR-0105 Amendment 3.
  - adapter-leptos-wasm: Leptos-component implementations (browser-WASM target); backend-qualified.
  - rest: HTTP handler/route layer (workbook session CRUD).
  - sdk: client library for tenant-side Sheets embed.
  - app: composition-root binary; SSR + WASM emit per ADR-0065.
- exemptions claimed: none.
```

Naming justification — `formula-engine`:

```
NAME: oya-sheets-formula-engine-<layer>
JUSTIFICATION:
- microservice = sheets.
- bc-tokens = formula-engine: BC for the ≥400-function library + parser + evaluator.
  Library-only; consumed in-process by recalc-engine + cell-grid + ai-formula.
- layer = <layer>: trimmed crate set.
  - kernel: port-trait + entities (Formula, FunctionCall, EvalContext, EvalResult, FormulaError). Zero I/O.
  - domain: pure parsing + evaluation; deterministic per ADR-SHEETS-0002 Excel-conformance subset.
  - usecase: orchestrators driving evaluation + diagnostic emission.
  - api: typed contracts.
  - adapter: protocol-neutral impls (e.g., function-library bindings).
  - sdk: client library (tenant-side formula construction).
- exemptions claimed: rest/worker/app/adapter-postgres — formula-engine is library-only.
```

Naming justification — `recalc-engine`:

```
NAME: oya-sheets-recalc-engine-<layer>
JUSTIFICATION:
- microservice = sheets.
- bc-tokens = recalc-engine: BC for dep-graph + topo-sort + parallel recalc; load-bearing
  for the 100k-cell ≤ 1s p95 + 1M-cell ≤ 10s p95 targets.
- layer = <layer>: full set; worker for large-sheet background recalc.
  - kernel: port-trait + entities (DepGraph, DirtySet, RecalcPlan, RecalcFrame). Zero I/O.
  - domain: pure topo-sort + dep-marking algebra; deterministic per ADR-SHEETS-0004.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
  - worker: long-lived process for large-workbook background recalc.
  - sdk: client library (recalc progress streaming).
- exemptions claimed: rest/app/adapter-postgres — recalc-engine is library + worker only.
```

Naming justification — `collab-crdt`:

```
NAME: oya-sheets-collab-crdt-<layer>
JUSTIFICATION:
- microservice = sheets.
- bc-tokens = collab-crdt: BC for CRDT state + merge logic + conflict surfacer + WebSocket
  collab server. Aligns with workflow-studio collab-crdt per ADR-SHEETS-0001 (same Loro 1.x).
- layer = <layer>: full set; WebSocket worker is long-lived.
  - kernel: port-trait + entities (CrdtState, MergeOp, Conflict, EditorSession). Zero I/O.
  - domain: pure CRDT merge algebra; deterministic.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
  - adapter-loro: Loro 1.x adapter; backend-qualified per ADR-0105 Amendment 3.
  - adapter-valkey: ephemeral CRDT state cache.
  - worker: WebSocket gateway long-lived process.
  - sdk: client library.
- exemptions claimed: app — collab-crdt rolls into cell-grid-app composition root.
```

Naming justification — `import-export`:

```
NAME: oya-sheets-import-export-<layer>
JUSTIFICATION:
- microservice = sheets.
- bc-tokens = import-export: BC for XLSX/ODS/CSV/TSV/JSON-Sheet import/export with the
  best-effort fidelity tier per ADR-SHEETS-0007; sandboxed gVisor + AV-scanned.
- layer = <layer>: full set; worker for XLSX export background pipeline.
  - kernel: port-trait + entities (ImportContext, ExportContext, ExportFidelityTier). Zero I/O.
  - domain: pure XLSX↔canonical-sheet mapping; deterministic.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
  - adapter-calamine: XLSX read via calamine 0.26; backend-qualified.
  - adapter-rust-xlsxwriter: XLSX write via rust_xlsxwriter 0.79; backend-qualified.
  - adapter-clamav: AV scan adapter; backend-qualified.
  - adapter-opswat: OPSWAT MetaDefender alternative scan adapter; backend-qualified.
  - worker: long-lived XLSX export pipeline process (gVisor sandboxed).
  - sdk: client library.
- exemptions claimed: rest/app — import-export is library + worker only.
```

Naming justification — `large-sheet-storage`:

```
NAME: oya-sheets-large-sheet-storage-<layer>
JUSTIFICATION:
- microservice = sheets.
- bc-tokens = large-sheet-storage: BC for the hybrid postgres+Arrow-Parquet substrate per
  ADR-SHEETS-0003 for sheets >100k cells. Postgres for hot OLTP edit-path; Arrow for
  analytical recalc + export.
- layer = <layer>:
  - kernel: port-trait + entities (ColumnarBlock, SheetSnapshot, HotColdBoundary). Zero I/O.
  - domain: pure hot↔cold mapping logic; deterministic.
  - usecase: orchestrators.
  - api: typed contracts.
  - adapter: protocol-neutral impls.
  - adapter-arrow: Apache Arrow 18.x columnar adapter; backend-qualified.
  - adapter-parquet: Parquet 18.x snapshot serialisation; backend-qualified.
  - adapter-s3: S3 snapshot adapter; backend-qualified.
- exemptions claimed: rest/worker/sdk/app — library-only; consumed by cell-grid-app.
```

Naming justifications for the remaining BCs (formatting / pivot-tables / charts / data-validation / sharing-acl / comments / version-history / named-ranges / ai-formula / connected-sheets / trigger-bridge / embed-bridge / license-gate-cedar) follow the same shape; recorded in companion artifacts under `microservices/sheets/specs/naming-justifications.md` to keep this PRD focused.

Layer mapping per BC (13-layer canonical enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-valkey | adapter-arrow | adapter-parquet | adapter-s3 | adapter-loro | adapter-calamine | adapter-rust-xlsxwriter | adapter-clamav | adapter-opswat | adapter-leptos-wasm | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `cell-grid` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | ✓ | ✓ | — | ✓ | ✓ |
| `formula-engine` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| `recalc-engine` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | ✓ | — |
| `formatting` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| `pivot-tables` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| `charts` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | ✓ | — | — | ✓ | — |
| `data-validation` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| `collab-crdt` | ✓ | ✓ | ✓ | ✓ | ✓ | — | ✓ | — | — | — | ✓ | — | — | — | — | — | — | ✓ | ✓ | — |
| `import-export` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | — |
| `large-sheet-storage` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — |
| `sharing-acl` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| `comments` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| `version-history` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | — | — | — | — | — | — | — | — | ✓ | — |
| `named-ranges` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
| `ai-formula` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| `connected-sheets` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | ✓ | — |
| `trigger-bridge` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| `embed-bridge` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |
| `license-gate-cedar` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | ✓ | — |

Total crates introduced by this µservice: approximately **115** (counting one crate per BC × layer cell ticked above). The cell µservice substrate (per-workbook cell boundary, ADR-0135) is a sibling; sheets does NOT re-implement cell storage but consumes the cell SDK.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `WorkbookStore` | `oya-sheets-cell-grid-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `CellGraphRender` | `oya-sheets-cell-grid-kernel` | `-adapter-leptos-wasm` | `BEHAVIORAL_TENANT_PRODUCT` |
| `FormulaParser` | `oya-sheets-formula-engine-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `FormulaEvaluator` | `oya-sheets-formula-engine-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `DepGraphBuilder` | `oya-sheets-recalc-engine-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |
| `RecalcScheduler` | `oya-sheets-recalc-engine-kernel` | `-worker` | `INTERNAL_ONLY` |
| `CrdtMergeEngine` | `oya-sheets-collab-crdt-kernel` | `-adapter-loro` (Loro 1.x) | `INTERNAL_ONLY` |
| `EditorSessionStore` | `oya-sheets-collab-crdt-kernel` | `-adapter-valkey` | `BEHAVIORAL_TENANT_PRODUCT` |
| `WebSocketGatewayDispatcher` | `oya-sheets-collab-crdt-kernel` | `-worker` | `BEHAVIORAL_TENANT_PRODUCT` |
| `XlsxImporter` | `oya-sheets-import-export-kernel` | `-adapter-calamine` (calamine 0.26) | `BEHAVIORAL_TENANT_PRODUCT` + `PII_QUASI_IDENTIFIER` |
| `XlsxExporter` | `oya-sheets-import-export-kernel` | `-adapter-rust-xlsxwriter` (rust_xlsxwriter 0.79) | `BEHAVIORAL_TENANT_PRODUCT` |
| `UploadAvScan` | `oya-sheets-import-export-kernel` | `-adapter-clamav` + `-adapter-opswat` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ColumnarStore` | `oya-sheets-large-sheet-storage-kernel` | `-adapter-arrow` + `-adapter-parquet` | `BEHAVIORAL_TENANT_PRODUCT` |
| `SnapshotStore` | `oya-sheets-large-sheet-storage-kernel` | `-adapter-s3` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ShareAclEvaluator` | `oya-sheets-sharing-acl-kernel` | `-domain` (Cedar-driven) | `AUDIT` |
| `RangeAclStore` | `oya-sheets-sharing-acl-kernel` | `-adapter-postgres` | `AUDIT` |
| `ConnectedQueryDispatcher` | `oya-sheets-connected-sheets-kernel` | `-worker` | `BEHAVIORAL_TENANT_PRODUCT` |
| `AiFormulaBridge` | `oya-sheets-ai-formula-kernel` | `-adapter` (consumes foundry-runtime SDK) | `BEHAVIORAL_TENANT_PRODUCT` + `AUDIT` |
| `SeatLicenseStore` | `oya-sheets-license-gate-cedar-kernel` | `-adapter-postgres` | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` |
| `CedarPolicyEvaluator` | `oya-sheets-license-gate-cedar-kernel` | `-domain` (pure) | `INTERNAL_ONLY` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields at PR-time.

Cross-product rule: `sheets` MUST NOT import any other product µservice crate directly. Sheets consumes:
- `cell` via its SDK (per-workbook cell boundary storage substrate).
- `ontology` via its SDK (object-type descriptors for typed-column configuration; Workbook/Sheet/Cell/Range/Chart entity bindings).
- `foundry-runtime` via its SDK (T1 formula suggest; T1 smart-fill; T2 AI-formula generation; T2 anomaly detection).
- `tenancy` via its SDK (per-seat licensing + tenant resolution).
- `audit-chain` via its SDK (cell-edit + sharing + formula-engine-upgrade seals).
- `workflow-engine` via its SDK (trigger-bridge: sheet-edit-triggers-workflow).
- `docs` / `slides` via their SDKs (embed-bridge: live cell-range + chart embedding).
- `drive` via its SDK (workbook storage hierarchy).
- `forms` via its SDK (response data flow → sheets cell ranges).
- `mail` via its SDK (export-to-mail).
- `community` via its SDK (shared template marketplace).

All cross-µservice flows go through SDK boundaries; no direct kernel imports across µservices. LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice sheets` — dependency-direction
- `oya gate validate lean-a2 --microservice sheets` — cross-product-refusal
- `oya gate validate port-location --microservice sheets`
- `oya gate validate layer-correctness --microservice sheets`
- `oya gate validate per-microservice-layout --microservice sheets`
- `oya gate validate statelessness --microservice sheets`
- `oya gate validate shardability --microservice sheets`
- `oya gate validate sheets-crdt-no-silent-loss --microservice sheets`
- `oya gate validate sheets-formula-engine-correctness --microservice sheets` — Excel-reference conformance on named test corpus
- `oya gate validate sheets-recalc-determinism --microservice sheets`
- `oya gate validate sheets-xlsx-roundtrip-best-effort --microservice sheets`
- `oya gate validate sheets-range-acl-cedar-required --microservice sheets`
- `oya gate validate sheets-import-sandboxed-and-avscan-required --microservice sheets`
- `oya gate validate cedar-preview-required --microservice sheets`
- `oya gate validate editor-execution-forbidden --microservice sheets`
- `oya gate validate wasm-bundle-sri --microservice sheets`

## Integration via Workflow + Ontology

Sheets is a product µservice; cross-product flows route through the engine's event-bus (per `feedback_workflow_objectgraph_adapter_layer.md`) and through Ontology reads.

### Workflow events produced

| Event type | Trigger | Consumed by | Purpose |
|---|---|---|---|
| `WorkbookOpened` | editor open | engine (audit), tenancy (seat tracking), observability | per-seat license attribution + audit |
| `WorkbookSaved` | save action | engine (audit), audit-chain, ontology (workbook link), drive (storage hierarchy) | round-trip ack |
| `CellEdited` | cell value change | audit-chain, observability, trigger-bridge (workflow-engine dispatch) | audit + sheet-edit-triggers-workflow |
| `RangeAclChanged` | share/permission change | audit-chain, tenancy, observability | per-range ACL audit |
| `CollabMerged` | CRDT merge applied | observability | collab-merge-rate SLI |
| `CollabConflictSurfaced` | conflict UI shown | observability, audit | conflict-rate SLI |
| `LicenseGateEmitted` | Cedar evaluation | tenancy (billing), audit-chain | seat enforcement |
| `FormulaEngineVersionChanged` | engine upgrade | audit-chain, observability | formula-engine rollback support per ADR-SHEETS-0002 |
| `XlsxImported` | XLSX import complete | audit-chain, observability | import-pipeline audit |
| `XlsxExported` | XLSX export complete | audit-chain, observability, mail (if export-to-mail) | export audit |
| `AiFormulaDraftRequested` | tenant prose → AI-formula | foundry-runtime, audit-chain | AI-formula usage + audit |
| `AiFormulaDraftAccepted` | tenant accepts AI-formula draft | audit-chain | AI-formula quality SLI |
| `SmartFillInferred` | smart-fill applied | audit-chain, observability | smart-fill audit |
| `ConnectedQueryRefreshed` | external-source materialize | audit-chain, observability | connected-sheets audit |
| `WorkbookEmbedded` | embed-bridge created | docs / slides, audit-chain | cross-product embed lifecycle |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `OntologyTypeDescriptorUpdated` | ontology | cell-grid + data-validation | hot-reload typed-column descriptors |
| `TenantSeatLimitUpdated` | tenancy | license-gate-cedar | refresh entitlement claim |
| `WorkflowDefinitionUpdated` | workflow-engine | trigger-bridge | refresh edit-trigger bindings |
| `FormsResponseAppended` | forms | cell-grid | append response row |
| `DriveWorkbookMoved` | drive | cell-grid | hierarchy path refresh |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `Workbook{tenant, workbook_id, version_sha, author, saved_at, parent_version_sha}` | `authored_by→TenantUser`, `derived_from→Workbook` (prior version), `lives_in→DriveFolder` | cell-grid → audit-chain | Ed25519 |
| `Sheet{tenant, sheet_id, workbook_id, position, name}` | `member_of→Workbook` | cell-grid | Ed25519 |
| `Cell{tenant, sheet_id, ref, value_hash, formula_hash, data_class}` | `member_of→Sheet` | cell-grid (sampled; per-cell-edit too noisy for unsampled emission) | Ed25519 (sampled) |
| `Range{tenant, sheet_id, start, end, name?}` | `member_of→Sheet` | named-ranges + cell-grid | Ed25519 |
| `Chart{tenant, chart_id, sheet_id, type, source_range}` | `member_of→Sheet`, `derives→Range` | charts | Ed25519 |
| `RangeAcl{tenant, range_id, principal, decision}` | `applies_to→Range` | sharing-acl | Ed25519 |
| `AiFormulaDraft{tenant, draft_id, prompt_hash, completion_hash, accepted_at}` | `drafted_by→Foundry-runtime-llm`, `accepted_into→Cell` | ai-formula | Ed25519 |

### Ontology reads

| Object Type / Function | Read by BC | Query shape |
|---|---|---|
| `ObjectTypeDescriptor` (catalog) | cell-grid + data-validation | `where(domain in tenant.packs).descriptors()` for typed-column config (Airtable-class) |
| `Tenant` (catalog) | license-gate-cedar | `where(tenant_id=...).seat_limit` |
| `Pack` (catalog) | sharing-acl + license-gate-cedar | `where(pack_id=...).overlays()` |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Google Sheets | Workbook + grid + collab + Connected Sheets + Apps Script | full reference parity | `support.google.com/docs/topic/9054603` |
| Microsoft Excel Web (M365) | Workbook + grid + collab + Power Query | full reference parity | `learn.microsoft.com/en-us/office/dev/scripts/` |
| Airtable | Grid + typed-column + collab + automations | typed-column + Airtable formula language | `support.airtable.com` |
| Notion databases | Database views + relations + formulas | typed-column + properties + relations | `notion.so/help/category/databases` |
| Coda tables | Coda tables + Coda formula language | typed-column + Coda formula + actions | `help.coda.io` |
| Smartsheet | Smartsheet grid + Gantt + automations | sheet-class grid + automations | `help.smartsheet.com` |
| Quip Spreadsheets | Quip workbook + collab | collab + embed-in-docs | `quip.com/api/reference` |
| Zoho Sheet | Workbook + grid + Zoho-flavoured formulas | parity baseline | `help.zoho.com/portal/en/kb/sheet` |
| OnlyOffice Spreadsheet | OOXML-fidelity grid editor | strict-OOXML round-trip benchmark | `api.onlyoffice.com/editors` |
| LibreOffice Online Calc | Open-source Calc | LibreOffice Calc behaviour matrix (ADR-SHEETS-0002 reference corpus) | `documentation.libreoffice.org` |
| NocoDB | OSS Airtable alternative | typed-column + API | `docs.nocodb.com` |
| Baserow | OSS Airtable alternative | typed-column + API | `baserow.io/docs` |
| Rows | Spreadsheet + integrations | integration richness | `rows.com/docs` |
| Equals | Analytics spreadsheet | SQL-class connected sheets | `equals.com/docs` |
| Causal | Modeling spreadsheet | financial modelling DSL | `causal.app/docs` |
| Anyleaf | Open-source spreadsheet | parity baseline | `anyleaf.org` |

Key parity gaps to close (ordered by priority for M03 preview milestone):

1. **Function-library coverage ≥ 400 functions** — Excel ships ~500 functions; Google Sheets ~470; oyatie M03 target ≥ 400 across math/logical/lookup/statistical/financial/text/date/array categories. Per ADR-SHEETS-0002.
2. **XLSX best-effort fidelity** — OnlyOffice and LibreOffice claim strict OOXML round-trip; oyatie ships best-effort fidelity per ADR-SHEETS-0007 with named limit list (no VBA, image fidelity downgrade tolerance). Strict-round-trip scheduled-for-distinct-tracked-work to subsequent-to-M03-completion phase.
3. **Recalc performance** — 1M-cell recalc p95 ≤ 10s (oyatie target); Google Sheets caps at 10M cells per workbook; Excel-Web caps at 1M cells. oyatie matches Excel-Web.
4. **Real-time collab** — Google Sheets + Microsoft Excel Web + Quip have real-time collab; oyatie matches with Loro CRDT (no silent loss invariant; competitors use OT-class with last-writer-wins fallback).
5. **Per-range ACL granularity** — Google Sheets has protected ranges; oyatie matches via named-ACL per ADR-SHEETS-0006.
6. **Typed-column / database-grid** — Airtable + Notion-database + Coda offer typed columns; oyatie matches via ontology object-type descriptors at the cell-grid layer.
7. **AI-formula + smart-fill** — Excel has Copilot; Google Sheets has Gemini; oyatie matches via foundry-runtime T1/T2 bridge with EU-AI-Act-bounded scope.
8. **Connected-sheets** — Google Sheets Connected Sheets + Excel Power Query; oyatie matches via foundry-runtime external-source query bridge.

Detailed quantitative comparison in `competitor-parity-matrix.md`.

## Performance Targets

(Duplicated from §"Non-Functional Requirements" for ease of citation by downstream consumers.)

| Metric | p50 | p95 | p99 | p999 |
|---|---|---|---|---|
| Sheet-open cold | 200ms | 400ms | 600ms | 1.5s |
| Sheet-open warm | 80ms | 150ms | 250ms | 500ms |
| Cell-edit-render | 16ms | 30ms | 50ms | 100ms |
| Recalc 100k-cell | 400ms | 1s | 1.5s | 3s |
| Recalc 1M-cell | 4s | 10s | 15s | 30s |
| Save round-trip | 50ms | 80ms | 100ms | 250ms |
| Collab cursor sync | 50ms | 100ms | 150ms | 300ms |
| XLSX export 100k-cell | 2s | 4s | 5s | 10s |
| Chart render | 80ms | 150ms | 200ms | 400ms |

Error budget:
- Monthly error budget for editor REST: 0.05% (≈22 min/month).
- Monthly error budget for WebSocket collab gateway: 0.1%.
- Burn-rate alarms: 14.4× burn over 1h for editor REST; 6× burn over 6h for collab gateway.

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Rationale:
- Workbook metadata, cell storage (small), per-seat license attribution, sharing/ACL, comments, version-history pointers: `postgres` (Citus-distributed by tenant_id).
- Large-sheet cell storage (>100k cells): hybrid `postgres+arrow-parquet` per ADR-SHEETS-0003; hot OLTP rows in Postgres; cold analytical blocks in Arrow/Parquet on S3.
- Ephemeral collab CRDT state, recalc-progress streams: `valkey` (per-cell cluster; reconstructable from Postgres on cold-start).
- Workbook snapshots + version-history binaries: `s3` (object storage; per-pack bucket).
- Static assets (WASM bundles + design-system primitives): `cdn` (global edge cache; per-pack key partitioning).

**Active-active compatibility**: `stateless-compatible` for REST/SDK; `single-writer-compatible` for collab CRDT (one WebSocket gateway pod owns active sessions for a given workbook; lease-coordinated via Valkey).

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active editor sessions | 10,000 | 100,000 | WebSocket connection count > 70k OR collab merge wait p99 > 100ms |
| Workbooks per tenant | 1,000 | 100,000 | Postgres shard fill > 80% |
| Concurrent collab WS connections | 50,000 | 500,000 | WS gateway pod CPU > 70% |
| Recalc invocations/sec (cluster-wide) | 1,000 | 100,000 | Recalc worker queue depth > threshold |
| XLSX export jobs/sec | 5 | 100 | Export worker pool saturation |
| AI-formula requests/sec | 10 | 1,000 | foundry-runtime backpressure signal |

Scale-out policy:
- Editor REST: stateless HPA on CPU > 70%; min 2 replicas; max 50.
- WebSocket gateway: stateful per active editor session; lease-coordinated via Valkey; HPA on WS connection count; min 3 replicas; max 100.
- Recalc worker: HPA on queue depth; min 2 replicas; max 50.
- XLSX export worker: HPA on queue depth + gVisor sandbox capacity; min 2 replicas; max 20.
- Postgres + Citus: tenant_id shard key; linear shard addition.
- Valkey: per-cell cluster; HA via Sentinel.
- CDN: global; per-pack edge nodes; OCI CDN service.

Cross-region story:
- M03 preview launch: single KR region.
- Post-M03: per-pack residency activation; CDN edges per pack; editor session state pinned to pack.
- AI-formula: foundry-runtime handles cross-pack residency; Sheets inherits.

Sharding:
- Postgres + Citus on `tenant_id`; cell-edit log append-only; Citus distributed table.
- Arrow/Parquet large-sheet blocks: per-(tenant_id, workbook_id, sheet_id) key.
- Valkey per-cell cluster; cell-local CRDT state.
- WebSocket gateway: consistent-hash on `workbook_id` ensures collab participants land on same gateway pod.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | Cell value entry + SUM formula → recalc → display correct sum on a 100×100 grid | `cargo nextest run -p oya-sheets-formula-engine-domain --test test_sum_basic` |
| AC-02 | Load XLSX (100-workbook reference corpus) → render → export → byte-compatible (best-effort fidelity) per ADR-SHEETS-0007 | `cargo nextest run -p oya-sheets-import-export-domain --test test_xlsx_best_effort_roundtrip` |
| AC-03 | Editor open with pending changes; network disconnect; local buffer persists; resume without loss on reconnect | `tests/e2e/offline-buffer-resume.rs` |
| AC-04 | Workbook with PII column; share with per-range ACL hiding PII column; recipient sees workbook without PII range | `cargo nextest run -p oya-sheets-sharing-acl-domain --test test_per_range_acl_hides_pii` |
| AC-05 | AI-formula prose → candidate formula via API; valid: opens in editor; invalid: precise per-line error | `tests/e2e/ai-formula-validation.rs` |
| AC-06 | Two users edit same workbook concurrently; CRDT merge applies non-conflicting; conflict UI for overlap; never silent loss | `cargo nextest run -p oya-sheets-collab-crdt-domain --test test_no_silent_overwrite` |
| AC-07 | Recalc 100k-cell sheet completes p95 ≤ 1s (synthetic harness) | `tests/load/recalc-100k-budget.js` |
| AC-08 | Recalc 1M-cell workbook completes p95 ≤ 10s (synthetic harness) | `tests/load/recalc-1m-budget.js` |
| AC-09 | Sheet-open cold p95 ≤ 400ms; warm p95 ≤ 150ms | `tests/load/sheet-open-budget.js` |
| AC-10 | Cell-edit-render p99 ≤ 50ms (60fps cap) | `tests/load/cell-edit-render-budget.js` |
| AC-11 | Formula-engine ≥ 400 functions; conformance against named Excel-reference test corpus per ADR-SHEETS-0002 | `cargo nextest run -p oya-sheets-formula-engine-domain --test test_excel_reference_corpus` |
| AC-12 | XLSX export 100k-cell workbook p95 ≤ 5s | `tests/load/xlsx-export-budget.js` |
| AC-13 | Chart render p95 ≤ 200ms (bar/line/pie/scatter/area/combo) | `tests/load/chart-render-budget.js` |
| AC-14 | Per-seat license check on workbook open; Cedar policy gate enforces seat; audit row emitted | `cargo nextest run -p oya-sheets-license-gate-cedar-domain --test test_per_seat_cedar` |
| AC-15 | XLSX import passes ClamAV + OPSWAT scans; embedded VBA stripped; sandboxed in gVisor | `cargo nextest run -p oya-sheets-import-export-adapter-clamav --test test_avscan_required` |
| AC-16 | Smart-fill 3-cell seed infers column pattern with ≥ 80% accuracy on named corpus | `cargo nextest run -p oya-sheets-ai-formula-domain --test test_smart_fill_corpus` |
| AC-17 | Connected-sheets refresh materializes external-source 10k-row range p95 ≤ 5s | `tests/load/connected-sheets-budget.js` |
| AC-18 | `oya gate validate per-microservice-layout --microservice sheets` exit 0 | ADR-0131 lane |
| AC-19 | `oya gate validate authority-cohesion` exit 0 | ADR-0123 lane; HG-SHEETS registered |
| AC-20 | `oya gate validate sheets-crdt-no-silent-loss --microservice sheets` exit 0 | new lane spec'd in PHASE-01 |
| AC-21 | `oya gate validate sheets-formula-engine-correctness --microservice sheets` exit 0 | new lane spec'd in PHASE-01 |
| AC-22 | `oya gate validate sheets-recalc-determinism --microservice sheets` exit 0 | new lane |
| AC-23 | `oya gate validate cedar-preview-required --microservice sheets` exit 0 | new lane |
| AC-24 | `oya gate validate editor-execution-forbidden --microservice sheets` exit 0 | new lane; Sheets never executes; only emits |
| AC-25 | `oya gate validate wasm-bundle-sri --microservice sheets` exit 0 | new lane |

## Open Questions

| # | Question | Owner | Resolution |
|---|---|---|---|
| 1 | CRDT library (yrs vs Loro vs bespoke)? | council-architecture | **Closed** by ADR-SHEETS-0001 — Loro 1.x (align ADR-WS-0001) |
| 2 | Formula-engine conformance target (full-Excel vs core-subset)? | council-architecture + axis-sheets | **Closed** by ADR-SHEETS-0002 — core-subset ≥ 400 fns; LibreOffice Calc reference corpus |
| 3 | Large-sheet storage substrate (postgres-only vs hybrid)? | council-architecture | **Closed** by ADR-SHEETS-0003 — postgres + Arrow/Parquet hybrid |
| 4 | Recalc engine architecture (single-thread vs parallel-task-graph)? | council-architecture + axis-sheets | **Closed** by ADR-SHEETS-0004 — dep-graph + topo + parallel-task-graph |
| 5 | AI-formula scope bounds (T0/T1/T2)? | council-architecture + council-legal-compliance | **Closed** by ADR-SHEETS-0005 — T0+T1 intra; T2 gated |
| 6 | Per-range ACL granularity (per-cell vs per-range vs whole-sheet)? | council-architecture + council-design-system | **Closed** by ADR-SHEETS-0006 — per-range named-ACL |
| 7 | XLSX export fidelity tier (best-effort vs strict-round-trip)? | council-architecture + axis-sheets | **Closed** by ADR-SHEETS-0007 — best-effort fidelity with named limit list |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0065 | Docs-as-Leptos webapp | Sheets is second-largest Leptos app |
| ADR-0103 (Bominal) | Workflow hexagonal migration | inherited; clean-arch placement |
| ADR-0105 | 13-layer enum + adapter-* backend-qualified | layer authority |
| ADR-0106 | Application → usecase rename | applied for new crates |
| ADR-0110 | ChangeSet state machine | each IP is one ChangeSet |
| ADR-0123 | Hyperscaler maturity claim gate | HG-SHEETS registers here |
| ADR-0135 | Sheets net-new µservice (no legacy connect-sheets) | this µservice's existence rationale |
| ADR-0139 | Agentic SLO-gated promotion | Sheets SLO promotion gates this µservice |
| ADR-0131 | Per-microservice flat layout | this µservice authored natively under it |
| ADR-0132 | Product-suite-and-bundle dissolution | Sheets is a hero product, not a suite |
| ADR-0133 | Industry-best-practice conformance | Sheets competitor parity tracked here |
| ADR-0140 | Cedar policy enforcement | license-gate-cedar + sharing-acl built on this |
| ADR-SHEETS-0001 | Loro 1.x CRDT — aligns ADR-WS-0001 | local |
| ADR-SHEETS-0002 | Formula-engine conformance target | local |
| ADR-SHEETS-0003 | Large-sheet storage substrate | local |
| ADR-SHEETS-0004 | Recalc engine architecture | local |
| ADR-SHEETS-0005 | AI-formula + smart-fill bounds | local |
| ADR-SHEETS-0006 | Per-range ACL granularity | local |
| ADR-SHEETS-0007 | XLSX export fidelity policy | local |
| oyatie override | Workflow + Ontology = ecosystem adapter | `feedback_workflow_objectgraph_adapter_layer.md` |
