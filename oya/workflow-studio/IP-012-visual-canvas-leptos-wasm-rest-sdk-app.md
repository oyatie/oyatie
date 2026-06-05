---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-012-visual-canvas-leptos-wasm-rest-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + council-design-system
acceptance_lanes: [cargo-check, cargo-nextest, helm-lint, oya-governance-wasm-bundle-sri, oya-governance-xss-vector-scan]
depends_on: [IP-007, IP-009, IP-010, IP-011]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-012: visual-canvas — usecase + api + adapter + adapter-leptos-wasm + rest + sdk + app

## Intent

Complete the `visual-canvas` BC: usecase orchestrators, api contracts, protocol-neutral adapter, Leptos browser-WASM components (the user-facing UI), editor REST handlers, tenant SDK, and composition-root app binary. This is the largest IP — produces the user-facing Studio surface.

## ChangeSet boundary

Seven crates:
- `oya-workflow-studio-visual-canvas-{usecase,api,adapter,adapter-leptos-wasm,rest,sdk,app}`

The `app` composition-root wires all 8 BCs (visual-canvas + dsl-emitter + dsl-loader + collab-crdt + node-library-registry + jurisdiction-overlay-renderer + replay-debugger-frontend + license-gate-cedar) into the Studio binary; emits SSR + WASM per ADR-0065.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-workflow-studio-visual-canvas-usecase/{Cargo.toml,src/{lib.rs,save_orchestrator.rs,load_orchestrator.rs,session_orchestrator.rs}}` | create |
| `src/crates/oya-workflow-studio-visual-canvas-api/{Cargo.toml,src/lib.rs}` | create |
| `src/crates/oya-workflow-studio-visual-canvas-adapter/{Cargo.toml,src/{lib.rs,impl.rs,llm_assist.rs}}` | create (llm_assist from IP-008) |
| `src/crates/oya-workflow-studio-visual-canvas-adapter-leptos-wasm/{Cargo.toml,src/{lib.rs,components/{canvas.rs,node_config_panel.rs,spec_diff_viewer.rs,replay_timeline.rs,policy_preview.rs,policy_disclosure_banner.rs}.rs,sri.rs},tests/{sri.rs,canvas_render.rs}}` | create |
| `src/crates/oya-workflow-studio-visual-canvas-rest/{Cargo.toml,src/{lib.rs,routes.rs,main.rs,middleware/{oidc.rs,tenant_binding.rs,csp.rs}.rs},tests/{routes.rs,csp.rs}}` | create |
| `src/crates/oya-workflow-studio-visual-canvas-sdk/{Cargo.toml,src/{lib.rs,client.rs}}` | create |
| `src/crates/oya-workflow-studio-visual-canvas-app/{Cargo.toml,src/main.rs,src/wire.rs}` | create | composition-root |
| `microservices/workflow-studio/iac/helm/visual-canvas-rest/templates/deployment.yaml` | update | wire to this binary |
| `microservices/workflow-studio/catalog/oya-workflow-studio-visual-canvas-*.yaml` | create | 7 catalog records |
| `microservices/workflow-studio/tests/load/{tti-budget.js,save-roundtrip.js}` | create |

## Code Shape

`visual-canvas-rest/src/middleware/csp.rs`:

```rust
use axum::http::header::{HeaderValue, CONTENT_SECURITY_POLICY};

pub fn strict_csp() -> HeaderValue {
    // Strict CSP per threat-model T-I-02 + T-E-01:
    // - no inline scripts except WASM bootstrap nonce
    // - no eval
    // - Trusted Types enforced
    HeaderValue::from_static(
        "default-src 'self' https://cdn-*.oyatie.dev; \
         script-src 'self' 'wasm-unsafe-eval' 'nonce-{NONCE}'; \
         style-src 'self' 'unsafe-inline'; \
         img-src 'self' data: https://cdn-*.oyatie.dev; \
         connect-src 'self' wss://collab-*.oyatie.dev https://api-*.oyatie.dev; \
         font-src 'self' https://cdn-*.oyatie.dev; \
         object-src 'none'; \
         frame-ancestors 'none'; \
         require-trusted-types-for 'script'; \
         upgrade-insecure-requests"
    )
}
```

`visual-canvas-adapter-leptos-wasm/src/sri.rs`:

```rust
use sha2::{Digest, Sha384};

/// Subresource Integrity (SRI) hash per HTML5 spec.
/// AC-12: every WASM chunk has SRI; mismatch refuses load.
pub fn compute_sri_sha384(bytes: &[u8]) -> String {
    let digest = Sha384::digest(bytes);
    format!("sha384-{}", base64::encode(&digest))
}
```

`visual-canvas-app/src/wire.rs`:

```rust
use anyhow::Result;

pub fn wire_dependencies() -> Result<AppState> {
    let pg = oya_workflow_studio_license_gate_cedar_adapter_postgres::PostgresStore::from_env()?;
    let valkey = oya_workflow_studio_collab_crdt_adapter_valkey::ValkeyStore::from_env()?;
    let cdn = oya_workflow_studio_node_library_registry_adapter_cdn::CdnStore::from_env()?;
    let engine_sdk = workflow_engine_sdk::Client::from_env()?;
    let ontology_sdk = ontology_sdk::Client::from_env()?;
    let foundry_sdk = foundry_providers_sdk::Client::from_env()?;
    let tenancy_sdk = tenancy_sdk::Client::from_env()?;
    Ok(AppState { pg, valkey, cdn, engine_sdk, ontology_sdk, foundry_sdk, tenancy_sdk })
}
```

## Acceptance Gates

```bash
cargo check --workspace --all-features
cargo build --workspace --target wasm32-unknown-unknown \
  -p oya-workflow-studio-visual-canvas-adapter-leptos-wasm
cargo build --release -p oya-workflow-studio-visual-canvas-app
cargo nextest run --workspace --tests
buck2 build //:quality-lane-registry-authority-check # lane=wasm-bundle-sri --microservice workflow-studio
buck2 build //:quality-lane-registry-authority-check # lane=xss-vector-scan --microservice workflow-studio
helm lint microservices/workflow-studio/iac/helm/visual-canvas-rest
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_sri` | AC-12; every WASM chunk has SRI; tampered chunk refused |
| `test_csp_header_present` | strict CSP set on every editor response |
| `test_innerHTML_never_called` | static scan greps `innerHTML` / `outerHTML` / `dangerouslySetInnerHTML` → 0 |
| `test_canvas_renders_5k_nodes_under_3s` | cold-load 5k-node graph ≤ 3s p99 |
| `test_save_round_trip_p99_under_200ms` | stable budget |
| `test_offline_buffer_resume` | edit during disconnect; persists; restored on reconnect (AC-03) |
| `tests/load/tti-budget.js` | Lighthouse-style TTI p99 ≤ 2s GA |

## Halt Conditions

- SRI test fails — STOP. AC-12 + T-T-06 breach.
- CSP missing on any response — STOP. T-I-02 breach.
- `innerHTML` found in any Studio crate — STOP. XSS-free architecture broken.
- TTI > 2s — Sev-2 per `runbooks/canvas-perf-regression.md`; do not promote without remediation.

## Next IP

[`IP-013-observability-slo-manifests.md`](IP-013-observability-slo-manifests.md)

## References

- ADR-0065 Leptos for browser UI.
- ADR-0105 layer enum (rest, sdk, app, adapter-leptos-wasm).
- threat-model.md T-I-02, T-T-06, T-E-01.
- PRD AC-09, AC-12.
- W3C Subresource Integrity — `w3.org/TR/SRI/`.
- CSP3 spec — `w3.org/TR/CSP3/`.
- Trusted Types — `w3.org/TR/trusted-types/`.
- Leptos book — `book.leptos.dev`.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/workflow-studio/IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md` matched [`p99`].
- applicable_compliance_pack_floor: [`HIPAA-2024`, `SOC2-T2`, `ISO27001-2022`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `3600`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/workflow-studio/IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/ARCHITECTURE.md`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/multi-region.md`, `microservices/workflow-studio/capacity-model.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
