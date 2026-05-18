---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-012-connected-sheets-comments-version-history-trigger-embed-bridge
status: pending
owner: axis-sheets
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1]
depends_on: [IP-007]
---

# IP-012: connected-sheets + comments + version-history + trigger-bridge + embed-bridge — full BC crate sets

## Intent

Author five remaining BCs:
- `connected-sheets`: external SQL-source query (Postgres / BigQuery-equivalent / Snowflake-equivalent / custom via foundry-runtime); refresh policy (manual / on-open / scheduled); materializes as cell range.
- `comments`: cell comments + threaded notes; mention bridge to tenancy.
- `version-history`: workbook snapshots in S3; named-version pointers in Postgres; restore.
- `trigger-bridge`: sheet-edit-triggers-workflow event bridge to workflow-engine.
- `embed-bridge`: live cell-range embed in docs; chart embed in slides; refresh policy.

## ChangeSet boundary

~30 crates across the five BCs.

## Acceptance Gates

```bash
cargo check -p oya-sheets-connected-sheets-{kernel,domain,worker} \
  -p oya-sheets-comments-{kernel,domain,adapter-postgres} \
  -p oya-sheets-version-history-{kernel,domain,adapter-postgres,adapter-s3} \
  -p oya-sheets-trigger-bridge-{kernel,domain,sdk} \
  -p oya-sheets-embed-bridge-{kernel,domain,sdk}
cargo nextest run --workspace --all-features -E 'package(oya-sheets-connected-sheets-* + oya-sheets-comments-* + oya-sheets-version-history-* + oya-sheets-trigger-bridge-* + oya-sheets-embed-bridge-*)'
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_connected_query_external_source` | Postgres external-source materialized as range; refresh works |
| `test_connected_query_credentials_via_openbao` | external credentials never inline; OpenBao reference only (T-I-08) |
| `test_comments_thread` | comments + threaded reply works |
| `test_version_history_restore` | restoreVersion produces a new version pointer + audit seal |
| `test_trigger_bridge_dispatch` | cell-edit triggers workflow-engine workflow dispatch |
| `test_embed_bridge_docs` | docs embed receives live cell-range updates per refresh policy |
| `test_embed_bridge_slides` | slides embed receives chart-render output |

## Halt Conditions

- Connected-sheets credentials leak into cell payload — STOP. T-I-08.
- Version-history restore loses cells — STOP.

## Next IP

[`IP-013-cell-grid-rest-leptos-wasm-app-license-gate.md`](IP-013-cell-grid-rest-leptos-wasm-app-license-gate.md)

## References

- PRD FR-12 + FR-13 + FR-16 + FR-19 + FR-22.
- threat-model.md T-I-08.
