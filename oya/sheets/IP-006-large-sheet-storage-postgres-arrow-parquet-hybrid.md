---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-006-large-sheet-storage-postgres-arrow-parquet-hybrid
status: pending
owner: axis-sheets
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1]
depends_on: [IP-001, IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: large-sheet-storage — postgres + Arrow/Parquet hybrid substrate (per ADR-SHEETS-0003)

## Intent

Author `large-sheet-storage` BC: hot-tier OLTP cell rows in Postgres + cold-tier Arrow/Parquet columnar blocks in OCI Object Storage. Threshold 100k cells per workbook triggers hot↔cold migration. Analytical recalc + XLSX export operate on columnar tier.

## ChangeSet boundary

Eight crates:
- `oya-sheets-large-sheet-storage-{kernel,domain,usecase,api,adapter,adapter-arrow,adapter-parquet,adapter-s3}`

## Code Shape

`large-sheet-storage-kernel/src/entities.rs`:

```rust
#[derive(Clone, Debug)]
pub struct ColumnarBlock {
    pub tenant_id: TenantId,
    pub workbook_id: WorkbookId,
    pub sheet_id: SheetId,
    pub start_row: u32,
    pub end_row: u32,
    pub arrow_buffer: Vec<u8>,  // Apache Arrow 18.x IPC format
}

pub struct HotColdBoundary {
    pub threshold_cells: u32,  // default 100_000 per ADR-SHEETS-0003
    pub hot_to_cold_idle_hours: u32,  // default 24
}
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-large-sheet-storage-kernel ... -p oya-sheets-large-sheet-storage-adapter-s3
cargo nextest run -p oya-sheets-large-sheet-storage-domain
cargo nextest run -p oya-sheets-large-sheet-storage-adapter-arrow --test arrow_block_io -- --include-ignored
cargo nextest run -p oya-sheets-large-sheet-storage-adapter-parquet --test parquet_snapshot_io -- --include-ignored
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_hot_cold_threshold` | 100k-cell threshold triggers migration |
| `test_arrow_block_io` | round-trip Arrow blocks to OCI Object Storage |
| `test_parquet_snapshot_io` | round-trip Parquet snapshots |
| `test_cold_to_hot_materialize` | first-edit on cold tier materializes back to Postgres hot for touched range |
| `test_compression_ratio` | Parquet compression ≥ 70% on cell-dense workbooks |

## Halt Conditions

- Hot↔cold migration test fails — STOP. ADR-SHEETS-0003 load-bearing.

## Next IP

[`IP-007-cell-grid-adapter-postgres-and-materialized-views.md`](IP-007-cell-grid-adapter-postgres-and-materialized-views.md)

## References

- ADR-SHEETS-0003 (large-sheet storage substrate).
- Apache Arrow 18.x — `arrow.apache.org/docs/`.
- Apache Parquet 18.x — `parquet.apache.org/`.
- OCI Object Storage — `oracle.com/cloud/storage/object-storage/`.
