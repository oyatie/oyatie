---
doc_class: ReferenceImplementation
microservice: sheets
language: Rust
date: 2026-05-20
doc_status: published
---

# Reference implementation — Cell edit + formula + CRDT collab via the sheets Rust SDK

A runnable example that opens a workbook, writes cells, subscribes to CRDT updates, and watches recalc cascade — using `oya-sheets-client` (target API; once IP-003 + IP-005 + IP-007 + IP-013 land).

## Cargo.toml

```toml
[package]
name = "sheets-collab-example"
version = "0.1.0"
edition = "2021"

[dependencies]
oya-sheets-client = { path = "../../crates/oya-sheets-client" }
oya-cedar-client = { path = "../../crates/oya-cedar-client" }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
futures = "0.3"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## src/main.rs

```rust
use anyhow::Result;
use futures::StreamExt;
use oya_cedar_client::CedarPrincipal;
use oya_sheets_client::{
    CellEdit, CellValue, CrdtUpdate, NamedRangeRequest, SheetsClient, SheetsClientConfig,
    WorkbookOpenRequest,
};
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let principal = CedarPrincipal::from_env("SHEETS_PRINCIPAL_JWT")?;
    let config = SheetsClientConfig {
        cell_endpoint: std::env::var("SHEETS_CELL")?,
        tenant_id: std::env::var("OYA_TENANT_ID")?,
        principal,
        request_timeout: std::time::Duration::from_secs(10),
    };
    let client = SheetsClient::connect(config).await?;

    // 1. Open a workbook.
    let workbook = client
        .open_workbook(WorkbookOpenRequest {
            name: "acme-2026-financial-model".into(),
        })
        .await?;
    info!(
        workbook_id = %workbook.workbook_id,
        sheets = workbook.sheets.len(),
        "workbook opened"
    );

    // 2. Set a few cells.
    let edits = vec![
        CellEdit {
            sheet: "assumptions".into(),
            address: "B2".into(),
            value: CellValue::Number(0.12),
        },
        CellEdit {
            sheet: "assumptions".into(),
            address: "B3".into(),
            value: CellValue::Number(0.08),
        },
        CellEdit {
            sheet: "revenue-forecast".into(),
            address: "B4".into(),
            value: CellValue::Formula(
                "=SUMIF('product-sales'!A:A, \"widgets\", 'product-sales'!E:E)".into(),
            ),
        },
    ];
    let edit_receipt = client
        .bulk_cell_edit(&workbook.workbook_id, edits)
        .await?;
    info!(edits_applied = edit_receipt.edits_applied, "edits applied");

    // 3. Define named ranges.
    let _ = client
        .named_range_add(NamedRangeRequest {
            workbook_id: workbook.workbook_id.clone(),
            name: "growth_rate".into(),
            range: "assumptions!B2".into(),
        })
        .await?;

    // 4. Subscribe to CRDT updates (real-time view of other collaborators' edits).
    let mut crdt_stream = client.subscribe_crdt(&workbook.workbook_id).await?;
    info!("subscribed to CRDT stream; press Ctrl-C to stop");

    while let Some(update_result) = crdt_stream.next().await {
        match update_result {
            Ok(update) => match update {
                CrdtUpdate::CellChanged {
                    sheet,
                    address,
                    before,
                    after,
                    actor,
                    actor_kind,
                } => {
                    info!(
                        sheet = %sheet,
                        address = %address,
                        before = ?before,
                        after = ?after,
                        actor = %actor,
                        actor_kind = ?actor_kind,
                        "cell changed by collaborator"
                    );
                }
                CrdtUpdate::CellRecalculated {
                    sheet,
                    address,
                    new_value,
                    triggered_by_edit_id,
                } => {
                    info!(
                        sheet = %sheet,
                        address = %address,
                        new_value = ?new_value,
                        triggered_by = %triggered_by_edit_id,
                        "cell recalculated"
                    );
                }
                CrdtUpdate::ConflictDetected {
                    sheet,
                    address,
                    versions,
                } => {
                    warn!(
                        sheet = %sheet,
                        address = %address,
                        version_count = versions.len(),
                        "conflict detected; user must resolve"
                    );
                }
                CrdtUpdate::NamedRangeAdded { name, range } => {
                    info!(name = %name, range = %range, "named range added");
                }
                CrdtUpdate::SharingChanged { granted_to, role } => {
                    info!(granted_to = %granted_to, role = ?role, "sharing changed");
                }
            },
            Err(e) => {
                warn!(error = ?e, "CRDT stream error");
                break;
            }
        }
    }

    Ok(())
}
```

## Expected log output (when a second collaborator edits)

```
INFO workbook opened workbook_id=wb-7f3a9b2c sheets=5
INFO edits applied edits_applied=3
INFO subscribed to CRDT stream; press Ctrl-C to stop
INFO cell changed by collaborator sheet=revenue-forecast address=C4 before=None after=Formula("=B4 * 1.15") actor=drill-modeller-b actor_kind=User
INFO cell recalculated sheet=revenue-forecast address=B4 new_value=Number(2_345_678.0) triggered_by=edit-abc123
INFO cell recalculated sheet=revenue-forecast address=B8 new_value=Number(8_976_543.0) triggered_by=edit-abc123
INFO cell recalculated sheet=revenue-forecast address=B10 new_value=Number(34_567_890.0) triggered_by=edit-abc123
INFO cell changed by collaborator sheet=revenue-forecast address=B6 before=Formula(...) after=Formula(...) actor=drill-modeller-a actor_kind=User
WARN conflict detected sheet=revenue-forecast address=B6 version_count=2
INFO sharing changed granted_to=drill-modeller-c role=Editor
```

## Audit chain emission

```sh
oya audit query --tenant drill-acme --since 30m --workbook acme-2026-financial-model
```

Expected events:

- `workbook_opened`
- `cell_set` × N (one per bulk-edit)
- `named_range_added`
- `crdt_subscription_started`
- `crdt_conflict_resolved` (when conflicts surface)
- `cell_recalculated` × M (one per cascade)
- `sharing_granted` (if applicable)

## Direct gRPC alternative

```sh
grpcurl -plaintext \
    -H "Authorization: Bearer $JWT" \
    -H "X-Oya-Tenant-Id: drill-acme" \
    -d '{
        "workbook_id": "wb-7f3a9b2c",
        "edits": [
            {"sheet": "assumptions", "address": "B2", "value": {"number": 0.12}},
            {"sheet": "revenue-forecast", "address": "B4", "value": {"formula": "=SUMIF(...)"}}
        ]
    }' \
    sheets.drill-syd-1.oyatie.local:9090 \
    oya.sheets.v1.SheetsService/BulkCellEdit
```

## Error handling

| Error class | Retry? | Action |
|---|---|---|
| `cedar_denied` | No | Principal lacks permission. Fix at IAM. |
| `workbook_locked` | Yes (transient) | Concurrent CRDT merge in progress; SDK retries |
| `formula_invalid` | No | Formula syntax/reference is wrong. Fix at caller. |
| `cell_data_class_violation` | No | The cell value violates the cell's data-class marker (e.g., writing PHI to a SECRET column). |
| `recalc_cycle_detected` | No | The formula edit creates a cycle. Recall the edit; surface to user. |
| `crdt_divergence_detected` | No (specific recovery) | Per `runbooks/loro-crdt-divergence.md`; re-sync from canonical state. |
| `cell_unavailable` | Yes (circuit-breaker) | Cell down; SDK fails after 3 retries; opens for 30 s. |

## Where this file lives

`microservices/sheets/reference-implementations/cell-edit-and-formula-rust-sdk.md` (this file). Runnable Cargo project lands at `microservices/sheets/reference-implementations/collab-example/` once IP-003 + IP-005 + IP-007 + IP-013 land.
