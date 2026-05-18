---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-002-cargo-workspace-cell-grid-kernel-domain
status: pending
owner: axis-sheets + council-design-system
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness, port-location]
depends_on: []
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: cell-grid — kernel + domain (entities + pure cell-graph algebra)

## Intent

Register the sheets cargo workspace and author the `cell-grid` BC's kernel + domain layers: Workbook, Sheet, Cell, Range, Selection, ViewportState entities + pure cell-graph algebra (dirty-marking math, range arithmetic, A1-notation parser). Pure Rust; zero I/O.

## ChangeSet boundary

Two crates plus workspace registration:
- `oya-sheets-cell-grid-kernel`
- `oya-sheets-cell-grid-domain`
- Workspace `Cargo.toml` updated to include `microservices/sheets/src/crates/oya-sheets-cell-grid-*`.

## Concrete File Targets

| Path | Action |
|---|---|
| `Cargo.toml` (workspace) | update — add sheets crates members |
| `microservices/sheets/src/crates/oya-sheets-cell-grid-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}` | create |
| `microservices/sheets/src/crates/oya-sheets-cell-grid-domain/{Cargo.toml,src/lib.rs,src/cell_graph.rs,src/a1_notation.rs,src/dirty_marking.rs,tests/cell_graph.rs,tests/a1_parser.rs}` | create |
| `microservices/sheets/catalog/oya-sheets-cell-grid-{kernel,domain}.yaml` | create (already authored in catalog/) |

## Code Shape

`cell-grid-kernel/src/entities.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[data_class("BEHAVIORAL_TENANT_PRODUCT")]
pub struct Workbook {
    pub workbook_id: WorkbookId,
    pub tenant_id: TenantId,
    pub version_sha: VersionSha,
    pub parent_version_sha: Option<VersionSha>,
    pub author_oidc_sub: OidcSub,
    pub title: String,
    pub sheets: Vec<SheetId>,
    pub named_ranges: Vec<NamedRangeId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[data_class("BEHAVIORAL_TENANT_PRODUCT")]
pub struct Cell {
    pub cell_ref: CellRef,
    pub value: CellValue,
    pub formula: Option<Formula>,
    pub format: Option<FormatId>,
    pub data_class: DataClass,
    pub formula_error: Option<FormulaError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Range {
    pub sheet_id: SheetId,
    pub start: CellRef,
    pub end: CellRef,
    pub range_id: Option<NamedRangeId>,
}
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-cell-grid-kernel -p oya-sheets-cell-grid-domain
cargo nextest run -p oya-sheets-cell-grid-domain
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice sheets
cargo run -p oya-dev-cli -- gate validate port-location --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_a1_parser_basic` | parses A1, AA10, Z100 etc. |
| `test_a1_parser_qualified` | parses Sheet1!A1:B10 etc. |
| `test_range_arithmetic` | union, intersect, contains |
| `test_dirty_marking_propagation` | edits propagate to dependents |
| `test_cell_value_type_coercion` | string/number/bool/null transitions |

## Halt Conditions

- A1-notation parser regression — STOP.
- Dirty-marking property test reveals non-determinism — STOP.

## Next IP

[`IP-003-formula-engine-kernel-domain-400-functions.md`](IP-003-formula-engine-kernel-domain-400-functions.md)

## References

- PRD §"Bounded Contexts" §cell-grid.
- ADR-0056, ADR-0105.
- A1-notation spec (Microsoft OOXML).
