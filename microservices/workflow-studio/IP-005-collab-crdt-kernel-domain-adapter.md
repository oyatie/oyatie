---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-005-collab-crdt-kernel-domain-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness]
depends_on: [IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: collab-crdt — kernel + domain + usecase + api + adapter + adapter-redis

## Intent

Author the `collab-crdt` BC's first six layers: CRDT merge engine (loro-based tree CRDT), conflict surfacer, editor-session store port, and Valkey adapter (Redis wire-compat) for ephemeral CRDT state. The "never silent loss" invariant (AC-06) is the load-bearing assertion of this IP.

## ChangeSet boundary

Six crates:
- `oya-workflow-studio-collab-crdt-{kernel,domain,usecase,api,adapter,adapter-redis}`

Per ADR-0105 Amendment 3 backend-qualified naming for `adapter-redis`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-workflow-studio-collab-crdt-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}` | create |
| `src/crates/oya-workflow-studio-collab-crdt-domain/{Cargo.toml,src/lib.rs,src/merge.rs,src/conflict.rs,tests/merge.rs,tests/no_silent_overwrite.rs}` | create |
| `src/crates/oya-workflow-studio-collab-crdt-usecase/{Cargo.toml,src/lib.rs,src/orchestrator.rs}` | create |
| `src/crates/oya-workflow-studio-collab-crdt-api/{Cargo.toml,src/lib.rs,src/contracts.rs}` | create |
| `src/crates/oya-workflow-studio-collab-crdt-adapter/{Cargo.toml,src/lib.rs,src/impl.rs}` | create |
| `src/crates/oya-workflow-studio-collab-crdt-adapter-redis/{Cargo.toml,src/lib.rs,src/redis_impl.rs,tests/redis_integration.rs}` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-collab-crdt-{kernel,domain,usecase,api,adapter,adapter-redis}.yaml` | create |

## Code Shape

`collab-crdt-kernel/src/entities.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrdtState {
    pub definition_id: String,
    pub tenant_id: String,
    pub sequence_num: u64,
    /// loro-encoded CRDT snapshot bytes.
    pub snapshot: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MergeOp {
    pub author_oidc_sub: String,
    pub sequence_num: u64,
    pub op_payload: Vec<u8>,
    pub hmac: String,
    pub emitted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Conflict {
    pub conflict_id: String,
    pub branches: Vec<ConflictBranch>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConflictBranch {
    pub branch_id: String,
    pub author_oidc_sub: String,
    pub ops_summary: String,
}
```

`collab-crdt-domain/tests/no_silent_overwrite.rs`:

```rust
use proptest::prelude::*;

proptest! {
    /// AC-06 invariant: for any two CRDT op streams, either the merge succeeds
    /// (all ops applied OR all ops applied with auto-merge) OR a Conflict is
    /// surfaced — never silent op-drop.
    #[test]
    fn test_no_silent_overwrite(
        ops_a in proptest::collection::vec(any::<u32>(), 1..50),
        ops_b in proptest::collection::vec(any::<u32>(), 1..50),
    ) {
        let result = oya_workflow_studio_collab_crdt_domain::merge::merge_streams(&ops_a, &ops_b);
        // Either fully merged OR conflict; never silent drop.
        match result {
            oya_workflow_studio_collab_crdt_domain::merge::MergeOutcome::Merged { applied_a, applied_b } => {
                prop_assert_eq!(applied_a, ops_a.len());
                prop_assert_eq!(applied_b, ops_b.len());
            }
            oya_workflow_studio_collab_crdt_domain::merge::MergeOutcome::Conflict { .. } => {
                // Conflict surfaced; correct behavior.
            }
        }
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-studio-collab-crdt-kernel -p oya-workflow-studio-collab-crdt-domain \
  -p oya-workflow-studio-collab-crdt-usecase -p oya-workflow-studio-collab-crdt-api \
  -p oya-workflow-studio-collab-crdt-adapter -p oya-workflow-studio-collab-crdt-adapter-redis
cargo nextest run -p oya-workflow-studio-collab-crdt-domain --test no_silent_overwrite
cargo nextest run -p oya-workflow-studio-collab-crdt-adapter-redis --test redis_integration -- --include-ignored
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice workflow-studio
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_no_silent_overwrite` (property) | AC-06; 1000 random op-stream pairs; never silent loss |
| `test_merge_commutativity` (property) | merge(a, b) == merge(b, a) for commuting ops |
| `test_conflict_surfaced_when_needed` | overlapping field-edits produce Conflict, not silent merge |
| `test_redis_lease_single_writer` | only one WS pod holds the lease at a time |
| `test_redis_ttl_expiry` | abandoned lease expires after TTL |

## Halt Conditions

- `test_no_silent_overwrite` fails — STOP. This is the load-bearing AC-06 invariant.
- Valkey lease test reveals split-brain — STOP. Cannot ship without single-writer guarantee.

## Next IP

[`IP-006-collab-crdt-worker-sdk.md`](IP-006-collab-crdt-worker-sdk.md)

## References

- PRD AC-06 + FR-07.
- threat-model.md T-T-01, T-T-02.
- loro CRDT docs — `loro.dev/docs`.
- yrs (Yjs Rust port) — `github.com/y-crdt/y-crdt`.
- Shapiro et al. — "Conflict-free Replicated Data Types" (Inria 2011).
