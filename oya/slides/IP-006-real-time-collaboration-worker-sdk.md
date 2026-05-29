---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-006-real-time-collaboration-worker-sdk
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + cloud-iac
acceptance_lanes: [cargo-check, cargo-nextest, ws-load-test]
depends_on: [IP-005]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: real-time-collaboration worker + SDK

## Intent

Author the long-lived WebSocket gateway worker for slides collab + the tenant SDK. Consistent-hash on `deck_id` for fan-out; HMAC verification per op; per-slide ACL refinement at projection.

## ChangeSet boundary

2 crates:
- `oya-slides-real-time-collaboration-{worker,sdk}`

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-slides-real-time-collaboration-worker/{Cargo.toml,src/main.rs,src/dispatcher.rs,tests/dispatch.rs}` | create |
| `src/crates/oya-slides-real-time-collaboration-sdk/{Cargo.toml,src/lib.rs,src/client.rs,tests/client.rs}` | create |
| `iac/helm/collab-worker/...` | create |

## Code Shape

`real-time-collaboration-worker/src/dispatcher.rs`:

```rust
pub struct WsDispatcher {
    crdt_engine: Arc<dyn CrdtMergeEngine>,
    valkey_lease: Arc<dyn EditorSessionLease>,
    hmac_verifier: Arc<dyn HmacVerifier>,
    acl_filter: Arc<dyn PerSlideAclFilter>,  // ADR-SLIDES-0007 refinement at projection
}

impl WsDispatcher {
    pub async fn handle_op(&self, op: &CrdtOpEnvelope, subscriber_oidc_sub: &str) -> Result<(), DispatchError> {
        // 1. Verify HMAC (T-T-01)
        self.hmac_verifier.verify(op)?;
        // 2. Rebind tenant_id from WS-upgrade OIDC (T-S-01)
        let rebound_tenant_id = self.rebind_tenant(op)?;
        // 3. Per-slide ACL filter (ADR-SLIDES-0007)
        if !self.acl_filter.permit(subscriber_oidc_sub, op.slide_id()).await? {
            return Ok(()); // filter; not silent loss
        }
        // 4. Apply CRDT op (delegate to engine; AC-06 invariant holds via ApplyOutcome enum)
        self.crdt_engine.apply_op(op)?;
        // 5. Fan-out to other peers
        self.fanout(op).await?;
        Ok(())
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-real-time-collaboration-worker --test dispatch
cargo nextest run -p oya-slides-real-time-collaboration-sdk --test client
tests/load/ws-50k-connections.sh
```

## Halt Conditions

- WS load test fails to sustain 50k connections per pod — STOP.

## Next IP

IP-007.
