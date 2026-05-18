---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-011-replay-debugger-frontend-full
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-nextest, layer-correctness]
depends_on: [IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: replay-debugger-frontend — full BC (6 layers)

## Intent

Author the `replay-debugger-frontend` BC: consumes the engine's `replay-debugger-backend` SDK stream and renders timeline-frame view in Studio. Implements tenant-binding filter as defence-in-depth per threat-model T-I-06.

## ChangeSet boundary

Six crates:
- `oya-workflow-studio-replay-debugger-frontend-{kernel,domain,usecase,api,adapter,sdk}`

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-workflow-studio-replay-debugger-frontend-kernel/{Cargo.toml,src/{lib.rs,entities.rs,ports.rs}}` | create |
| `src/crates/oya-workflow-studio-replay-debugger-frontend-domain/{Cargo.toml,src/{lib.rs,timeline.rs,tenant_filter.rs},tests/{tenant_filter.rs,timeline.rs}}` | create |
| `src/crates/oya-workflow-studio-replay-debugger-frontend-usecase/{Cargo.toml,src/lib.rs}` | create |
| `src/crates/oya-workflow-studio-replay-debugger-frontend-api/{Cargo.toml,src/lib.rs}` | create |
| `src/crates/oya-workflow-studio-replay-debugger-frontend-adapter/{Cargo.toml,src/{lib.rs,engine_sdk_consumer.rs}}` | create |
| `src/crates/oya-workflow-studio-replay-debugger-frontend-sdk/{Cargo.toml,src/{lib.rs,client.rs}}` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-replay-debugger-frontend-*.yaml` | create | 6 catalog records |

## Code Shape

`replay-debugger-frontend-domain/src/tenant_filter.rs`:

```rust
use crate::entities::StepSnapshot;

/// Defence-in-depth: even though engine should never emit cross-tenant frames,
/// Studio re-checks every frame's tenant_id against the subscriber's tenant_id.
/// Per threat-model T-I-06.
pub fn filter_frame(
    subscriber_tenant_id: &str,
    frame: &StepSnapshot,
) -> FilterDecision {
    if frame.tenant_id != subscriber_tenant_id {
        // CRITICAL: log + audit; emit Sev-1 metric.
        metrics::counter!("oya_workflow_studio_debugger_cross_tenant_attempt_total",
            "from_tenant" => frame.tenant_id.clone(),
            "to_tenant" => subscriber_tenant_id.to_string()).increment(1);
        return FilterDecision::Drop;
    }
    FilterDecision::Forward
}

pub enum FilterDecision {
    Forward,
    Drop,
}
```

`replay-debugger-frontend-domain/tests/tenant_filter.rs`:

```rust
#[test]
fn test_drop_cross_tenant_frame() {
    let frame = oya_workflow_studio_replay_debugger_frontend_kernel::entities::StepSnapshot {
        tenant_id: "tenant:aaaaaaaaaaaaaaaa".to_string(),
        run_id: "run-x".to_string(),
        step_id: "step-1".to_string(),
        seq: 1,
        outcome: "success".to_string(),
        payload: serde_json::json!({}),
        frame_checksum: "abc".to_string(),
        emitted_at: chrono::Utc::now(),
    };
    let decision = oya_workflow_studio_replay_debugger_frontend_domain::tenant_filter::filter_frame(
        "tenant:bbbbbbbbbbbbbbbb", &frame,
    );
    assert!(matches!(decision, oya_workflow_studio_replay_debugger_frontend_domain::tenant_filter::FilterDecision::Drop));
}

#[test]
fn test_forward_same_tenant_frame() {
    // ... mirror with same tenant_id; expect Forward.
}
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-studio-replay-debugger-frontend-kernel \
  -p oya-workflow-studio-replay-debugger-frontend-domain \
  -p oya-workflow-studio-replay-debugger-frontend-usecase \
  -p oya-workflow-studio-replay-debugger-frontend-api \
  -p oya-workflow-studio-replay-debugger-frontend-adapter \
  -p oya-workflow-studio-replay-debugger-frontend-sdk
cargo nextest run -p oya-workflow-studio-replay-debugger-frontend-domain --test tenant_filter
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_drop_cross_tenant_frame` | T-I-06 defence-in-depth holds |
| `test_forward_same_tenant_frame` | legitimate frames forwarded |
| `test_frame_checksum_mismatch_reject` | frames with bad checksum dropped + audit emit |
| `test_decode_failure_pause_and_audit` | malformed proto bytes pause session + audit |
| `test_resync_from_seq_emits_engine_request` | resync forwards request to engine |

## Halt Conditions

- Cross-tenant frame forwarded — STOP. T-I-06 invariant breach.
- Checksum mismatch silently accepted — STOP.

## Next IP

[`IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md`](IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md)

## References

- threat-model.md T-I-06.
- runbooks/run-history-replay-corruption.md.
- microservices/workflow-engine/contracts/proto/replay-debugger-backend.proto (sibling).
