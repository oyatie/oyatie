---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-006-collab-crdt-worker-sdk
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + cloud-iac
acceptance_lanes: [cargo-check, cargo-nextest, helm-lint, lean-a1, layer-correctness]
depends_on: [IP-005]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: collab-crdt — worker + sdk (WebSocket gateway long-lived process + tenant SDK)

## Intent

Author the long-lived `collab-crdt-worker` (axum-WS WebSocket gateway, per-tenant per-definition lease coordinator, CRDT op fan-out) and the `collab-crdt-sdk` (tenant-side client library for CRDT op submission). The gateway enforces tenant-binding rebinding at each WS message dispatch per threat-model T-S-01.

## ChangeSet boundary

Two crates:
- `oya-workflow-studio-collab-crdt-worker` — long-lived WS process; consumes adapter-redis; emits asyncapi events.
- `oya-workflow-studio-collab-crdt-sdk` — tenant-side client library.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-workflow-studio-collab-crdt-worker/{Cargo.toml,src/main.rs,src/lib.rs,src/ws_gateway.rs,src/lease_coordinator.rs,src/fan_out.rs,tests/e2e_10_user_collab.rs}` | create |
| `src/crates/oya-workflow-studio-collab-crdt-sdk/{Cargo.toml,src/lib.rs,src/client.rs,src/ws_client.rs}` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-worker.yaml` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-sdk.yaml` | create |
| `microservices/workflow-studio/iac/helm/collab-crdt-worker/templates/deployment.yaml` | update | (created in IP-001; bind to this binary) |

## Code Shape

`collab-crdt-worker/src/ws_gateway.rs`:

```rust
use axum::{extract::ws::{WebSocket, WebSocketUpgrade}, response::Response, routing::get, Router};
use std::sync::Arc;
use tracing::{info, warn};

pub struct WsGatewayState {
    pub lease: Arc<crate::lease_coordinator::LeaseCoordinator>,
    pub fan_out: Arc<crate::fan_out::FanOut>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    state: axum::extract::State<Arc<WsGatewayState>>,
    // OIDC bearer validated by upstream middleware
    oidc_claims: oidc::TenantClaims,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, oidc_claims))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: axum::extract::State<Arc<WsGatewayState>>,
    oidc_claims: oidc::TenantClaims,
) {
    let session_token = generate_session_token(&oidc_claims);
    while let Some(Ok(msg)) = socket.recv().await {
        // SECURITY: rebind tenant_id from authoritative oidc_claims on every message
        // (per threat-model T-S-01); never trust client-supplied tenant_id mid-stream.
        let crdt_op = match decode_op(&msg, &oidc_claims) {
            Ok(o) => o,
            Err(e) => {
                metrics::counter!("oya_workflow_studio_collab_op_rejected_total",
                    "reason" => format!("{:?}", e)).increment(1);
                continue;
            }
        };
        if crdt_op.author_oidc_sub != oidc_claims.sub {
            metrics::counter!("oya_workflow_studio_collab_op_rejected_total",
                "reason" => "author_mismatch").increment(1);
            warn!(?oidc_claims.sub, "rejected op: author mismatch");
            continue;
        }
        // Lease check: this pod must own (tenant_id, definition_id).
        if !state.lease.we_own(&oidc_claims.tenant_id, &crdt_op.definition_id).await {
            // Route to owner pod.
            state.fan_out.route_to_owner(&crdt_op).await;
            continue;
        }
        // Apply CRDT op + fan out to subscribers.
        state.fan_out.apply_and_broadcast(&crdt_op).await;
    }
}
```

## Acceptance Gates

```bash
cargo build -p oya-workflow-studio-collab-crdt-worker --release
cargo nextest run -p oya-workflow-studio-collab-crdt-worker --test e2e_10_user_collab
helm lint microservices/workflow-studio/iac/helm/collab-crdt-worker
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice workflow-studio
```

## Test Plan

| Test | Verifies |
|---|---|
| `e2e_10_user_collab` | 10 concurrent users editing same definition; CRDT merge applied; conflict UI for overlap; no silent loss |
| `test_lease_handoff_during_rolling_deploy` | restart pod; lease moves cleanly; clients reconnect; CRDT state preserved |
| `test_tenant_binding_rebound_on_every_message` | author-mismatch attempt rejected |
| `test_ws_per_tenant_rate_limit` | per-(tenant, user) 100 ops/sec; excess refused |
| `test_idle_disconnect_5min` | inactive connections drop after 5min |

## Halt Conditions

- e2e 10-user collab silent loss detected — STOP. AC-06 invariant breach.
- Lease split-brain reproducible — STOP.
- Author-mismatch attempt succeeds — STOP. T-S-01 breach.

## Next IP

[`IP-007-node-library-registry-full.md`](IP-007-node-library-registry-full.md)

## References

- threat-model.md T-S-01, T-T-01, T-D-08.
- ADR-0105 worker layer.
- axum docs — `docs.rs/axum`.
- Redis lease pattern — `redis.io/commands/setnx/`.
