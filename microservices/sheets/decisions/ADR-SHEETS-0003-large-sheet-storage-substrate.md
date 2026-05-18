---
id: ADR-SHEETS-0003
title: Large-sheet storage substrate — postgres + Apache Arrow / Parquet hybrid
microservice: sheets
status: Accepted
date: 2026-05-17
owner: axis-sheets + council-architecture
deciders: council-architecture, axis-sheets, ops-sre-reliability, ops-finops
related: [ADR-0056, ADR-0105, ADR-0117, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/sheets/PRD.md (FR-04, AC-07, AC-08)
  - microservices/sheets/capacity-model.md
  - microservices/sheets/IP-006-large-sheet-storage-postgres-arrow-parquet-hybrid.md
purpose: Resolve PRD Open Question 3 — choose the storage substrate for workbooks exceeding the hot OLTP threshold (>100k cells).
doc_status: published
---

# ADR-SHEETS-0003: Large-sheet storage substrate — Postgres (hot OLTP) + Apache Arrow / Parquet on OCI Object Storage (cold analytical)

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Sheets supports workbooks ranging from 10-cell quick lists to 1M-cell financial models. The cell storage substrate must serve two workloads:

1. **Hot OLTP**: per-cell edit, per-cell read, per-range read; latency-sensitive (cell-edit-render p99 ≤ 50ms; sheet-open cold p95 ≤ 400ms).
2. **Cold analytical**: recalc-engine traversal of large dep-graphs; XLSX export streaming whole-workbook; pivot-table aggregation over wide ranges; AI-formula anomaly detection.

The two workloads have opposite optimisation criteria:
- OLTP wants row-oriented storage + indexed point lookups.
- Analytical wants columnar storage + vectorised scan.

Per PRD §"Performance":
- Recalc 100k-cell sheet p95 ≤ 1s.
- Recalc 1M-cell workbook p95 ≤ 10s.
- Cell-edit-render p99 ≤ 50ms.
- XLSX export 100k-cell p95 ≤ 5s.

A single substrate (postgres-only OR columnar-only) cannot simultaneously meet both edit-path latency AND analytical-throughput targets.

Capacity profile per `capacity-model.md`:
- Median workbook: ~5,000 cells.
- 95th percentile workbook: ~100,000 cells.
- 99th percentile workbook: ~1,000,000 cells.
- Cluster XL tier: 100,000+ workbooks; tail accumulates to terabytes of cold cell data.

## Decision

Adopt a **hybrid substrate**:

### Hot tier — Postgres + Citus (workbooks ≤ 100k cells; OR hot-range subsets of larger workbooks)

- Postgres 16 LTS + Citus 12.x distributed table.
- Cells stored row-per-cell in `cells_hot` table; partition by `tenant_id`.
- Per-(workbook, sheet) materialized-view caches for frequently-accessed ranges.
- Cell-level row-level security (RLS) enforced.
- All edit-path reads + writes go through the hot tier.

### Cold tier — Apache Arrow 18.x + Parquet 18.x on OCI Object Storage (workbooks > 100k cells; OR cold ranges of any workbook)

- Cells stored as columnar Arrow blocks (1000 cells per block).
- Blocks serialised to Parquet for cold storage on OCI Object Storage (per-pack bucket, per-(tenant, workbook, sheet) key).
- Recalc-engine + XLSX export + pivot-table can stream Arrow blocks directly without round-tripping through Postgres.
- KMS-SSE encryption at rest.

### Hot ↔ Cold boundary

- **Threshold**: 100,000 cells per workbook (configurable per-pack; pack-us-healthcare lower threshold given PHI sensitivity).
- **Hot → Cold migration**: workbook idle > 24h AND total cells > threshold → migrate to Parquet cold tier. Run by `oya-sheets-large-sheet-storage-worker` background process.
- **Cold → Hot promotion**: first-edit on a cold range materialises the touched range back to Postgres hot for the editor session lifetime (TTL ≥ 1h post-last-edit).
- Migration is **transparent** to cell-grid usecase layer; ports `WorkbookStore` + `ColumnarStore` are composed behind a `HybridStore` facade.

### Snapshots

- Workbook snapshots (version-history) live separately on S3 (per ADR-0028 retention).
- Version-history is independent of hot/cold tier; both materialisation paths participate in version-history.

## Alternatives Considered

### Alternative A — Postgres-only

- **Pros**
  - Single substrate; minimal ops surface; simpler operator mental model.
  - Citus scales horizontally on tenant_id.
- **Cons**
  - Row-storage scan of 1M cells is documented to take > 30s; misses 1M-cell ≤ 10s recalc budget.
  - XLSX export 100k-cell workbook would saturate Postgres connection pool.
  - Pivot-table aggregation over 1M cells via SQL GROUP BY is slow + thrashes Postgres buffer cache.
  - Per-cell row overhead (24 bytes Postgres metadata) is significant at 1M cells per workbook.
- **Rejected reason**: cannot meet PRD §"Performance" budgets at the 99th-percentile workbook scale.

### Alternative B — Columnar-only (Apache Arrow + Parquet; no Postgres for cells)

- **Pros**
  - Optimal analytical performance.
  - Bounded storage cost with Parquet compression.
- **Cons**
  - Single-cell edit requires rewriting the containing block (1000 cells minimum write amplification).
  - Cell-edit-render p99 ≤ 50ms unachievable with block-write amplification.
  - No row-level RLS; tenant isolation would need to be enforced at the application layer only.
  - Columnar-format does not support OLTP-class per-cell point lookups efficiently.
- **Rejected reason**: cannot meet edit-path latency budget; loses defense-in-depth tenant isolation (RLS).

### Alternative C — Custom binary format on OCI Object Storage (no postgres + no Arrow)

A sheets-specific binary cell format authored by axis-sheets.

- **Pros**
  - Full control over format.
- **Cons**
  - Engineering cost.
  - No external tooling (cannot use DuckDB / DataFusion / Polars to query directly).
  - Format-migration tax over time.
- **Rejected reason**: build-vs-buy ratio unfavourable. Arrow + Parquet are the columnar industry standard.

### Alternative D — Postgres + Postgres ColumnStore extension (Citus columnar tables)

Use Citus columnar tables (within Postgres) instead of separate Arrow/Parquet.

- **Pros**
  - Single substrate (still Postgres).
  - Citus columnar tables exist and offer columnar compression within Postgres.
- **Cons**
  - Citus columnar tables are append-only; in-place cell updates require full-row rewrite OR delete-and-insert.
  - Compression ratio of Citus columnar (~50%) less than Parquet (~70%).
  - Cannot stream blocks to external tools (XLSX export, AI-formula anomaly detection) without serialising through Postgres.
  - Citus columnar performance is good but not Arrow-vectorised.
- **Rejected reason**: append-only constraint conflicts with workbook edit semantics; lower compression ratio + lack of external tooling integration vs Arrow/Parquet.

### Alternative E — Postgres + DuckDB (DuckDB as cold-tier query engine over Postgres FDW)

- **Pros**
  - DuckDB is embeddable; analytical query performance.
- **Cons**
  - DuckDB-over-FDW pulls data through Postgres connection at query time; no cold-tier independence.
  - Adds another substrate to monitor + patch.
- **Rejected reason**: doesn't solve the underlying scan-cost problem; just adds a query engine.

## Consequences

### Architectural

- `oya-sheets-large-sheet-storage-*` BC introduces:
  - `adapter-arrow` for in-memory columnar blocks.
  - `adapter-parquet` for cold-tier serialisation.
  - `adapter-s3` for OCI Object Storage I/O.
  - `domain` for hot↔cold migration logic.
- `oya-sheets-cell-grid-usecase` composes `WorkbookStore` (Postgres hot tier) + `ColumnarStore` (Arrow cold tier) behind a hybrid facade.

### Downstream impact

1. **IP-006** authors the hybrid substrate.
2. **IP-004 (recalc-engine)** consumes Arrow blocks directly for large-workbook recalc; meets 1M-cell ≤ 10s budget via vectorised dep-graph traversal.
3. **IP-009 (import-export)** streams Arrow blocks to XLSX export pipeline; meets 100k-cell ≤ 5s budget.
4. **multi-region.md** — Arrow/Parquet blocks replicate intra-pack only.
5. **capacity-model.md** — Postgres sizing + OCI Object Storage sizing both tracked.
6. **cost-budget.md** — Object storage line item ($30-50/mo at XS tier; scales with cold-tier growth).

### Operational

- Hot↔cold migration runs in `oya-sheets-large-sheet-storage-worker`; observed via `dashboards/recalc-engine-health.json` migration panel.
- Migration failures: tenant-notified; workbook continues to function from hot tier (cold migration is a best-effort optimisation).

### Risk register

- **Risk**: Cold-to-hot promotion latency on first edit (block load from OCI Object Storage). **Mitigation**: pre-warm policy when tenant opens workbook (load all blocks for active sheet); progressive load for other sheets.
- **Risk**: Postgres connection pool saturation on first-edit thunderstorm (many tenants editing cold workbooks simultaneously). **Mitigation**: per-tenant connection pool slot; HPA on recalc-worker.
- **Risk**: Arrow / Parquet upstream version bump introduces breaking change. **Mitigation**: LTS pin (18.x); upgrade gated on round-trip corpus.

## References

- PRD `microservices/sheets/PRD.md` §FR-04, AC-07, AC-08.
- `microservices/sheets/capacity-model.md`.
- `microservices/sheets/IP-006-large-sheet-storage-postgres-arrow-parquet-hybrid.md`.
- Apache Arrow 18.x — `arrow.apache.org/docs/`.
- Apache Parquet 18.x — `parquet.apache.org/`.
- Citus distributed Postgres — `docs.citusdata.com/`.
- "Spreadsheets and Calculation" — Joel Spolsky on Excel storage internals.
- ADR-0056 — BNF v4.1.
- ADR-0105 — 13-layer enum + adapter-* backend-qualified.
- ADR-0117 — Cloud-native infrastructure.
- ADR-0126 — Sheets net-new µservice.
- ADR-0131 — Per-microservice flat layout.
