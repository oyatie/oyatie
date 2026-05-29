---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-013-cell-grid-rest-leptos-wasm-app-license-gate
status: pending
owner: axis-sheets + council-design-system + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-cedar-preview-required, oya-governance-editor-execution-forbidden, oya-governance-wasm-bundle-sri]
depends_on: [IP-008, IP-010, IP-011, IP-012]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: cell-grid — adapter-leptos-wasm + rest + sdk + app + license-gate-cedar full BC

## Intent

Author the Leptos browser-WASM cell-grid editor surface (adapter-leptos-wasm), the editor REST surface, the SDK, the composition-root app binary, and the full license-gate-cedar BC (per-seat Cedar enforcement at workbook open + per-action). This IP wires together everything authored in IP-001..IP-012.

## ChangeSet boundary

~14 crates.

## Code Shape

`cell-grid-app/src/main.rs` (composition root):

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let postgres = adapter_postgres::Client::new(env::POSTGRES_CONN).await?;
    let valkey = adapter_valkey::Client::new(env::VALKEY_AUTH).await?;
    let cell_sdk = cell_sdk::Client::new(env::CELL_SVC_URL, env::SPIFFE_IDENTITY).await?;
    let cedar = cedar_evaluator::Client::new(env::CEDAR_ENDPOINT, /*fail_closed*/ true).await?;
    // ... wire all adapters + ports
    let app = rest::router()
        .with_state(AppState { postgres, valkey, cell_sdk, cedar, /* ... */ });
    axum::serve(listener, app).await?;
    Ok(())
}
```

## Acceptance Gates

```bash
cargo build --target wasm32-unknown-unknown -p oya-sheets-cell-grid-adapter-leptos-wasm
cargo check -p oya-sheets-cell-grid-rest -p oya-sheets-cell-grid-sdk -p oya-sheets-cell-grid-app \
  -p oya-sheets-license-gate-cedar-kernel ... -p oya-sheets-license-gate-cedar-sdk
cargo nextest run -p oya-sheets-license-gate-cedar-domain --test test_per_seat_cedar
cargo run -p oya-dev-cli -- gate validate cedar-preview-required --microservice sheets
cargo run -p oya-dev-cli -- gate validate editor-execution-forbidden --microservice sheets
cargo run -p oya-dev-cli -- gate validate wasm-bundle-sri --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_per_seat_cedar` | AC-14; seat-overage refuses workbook open |
| `test_oidc_required_on_every_route` | LEAN check |
| `test_server_side_tenant_id_stamping` | client-supplied tenant_id overridden by OIDC claim |
| `test_wasm_sri_per_chunk` | every WASM chunk has SRI hash |
| `test_xss_vector_scan` | LEAN check; no `innerHTML` / `eval` |
| `test_no_tenant_branding_mid_render` | anti-pattern blocked |
| `test_editor_execution_forbidden` | LEAN check; no exec primitives outside import-export sandbox |
| `test_offline_buffer_resume` | AC-03 |
| `test_cell_edit_render_p99_50ms` | AC-10 budget |
| `test_sheet_open_cold_p95_400ms` | AC-09 budget |

## Halt Conditions

- Any LEAN check fails — STOP.
- Cell-edit-render budget breached — STOP.

## Next IP

[`IP-014-observability-slo-manifests-9-openslo.md`](IP-014-observability-slo-manifests-9-openslo.md)

## References

- PRD AC-09 + AC-10 + AC-14 + AC-03.
- threat-model.md T-I-02 + T-T-06 + T-E-04 + T-S-01.
- ADR-0065 (Leptos WASM).
- ADR-0140 (retired per ADR-0145) (Cedar policy enforcement).
