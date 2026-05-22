---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-005-collab-crdt-loro-aligned-ws-0001
status: pending
owner: axis-sheets
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, oya-governance-sheets-crdt-no-silent-loss]
depends_on: [IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: collab-crdt — kernel + domain + usecase + api + adapter + adapter-loro + adapter-valkey + worker + sdk (Loro 1.x aligned with workflow-studio ADR-WS-0001)

## Intent

Author the `collab-crdt` BC's full crate set with Loro 1.x as CRDT merge engine per ADR-SHEETS-0001 (aligned with workflow-studio ADR-WS-0001). WebSocket gateway worker fans out CRDT ops by consistent-hash on workbook_id. The "never silent loss" AC-06 invariant is load-bearing.

## ChangeSet boundary

Nine crates:
- `oya-sheets-collab-crdt-{kernel,domain,usecase,api,adapter,adapter-loro,adapter-valkey,worker,sdk}`

## Code Shape

`collab-crdt-domain/tests/no_silent_overwrite.rs`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_no_silent_overwrite(
        ops_a in proptest::collection::vec(any::<CellOp>(), 1..50),
        ops_b in proptest::collection::vec(any::<CellOp>(), 1..50),
    ) {
        let result = oya_sheets_collab_crdt_domain::merge::merge_streams(&ops_a, &ops_b);
        match result {
            MergeOutcome::Merged { applied_a, applied_b } => {
                prop_assert_eq!(applied_a, ops_a.len());
                prop_assert_eq!(applied_b, ops_b.len());
            }
            MergeOutcome::Conflict { .. } => {
                // Conflict surfaced; correct behavior.
            }
        }
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-collab-crdt-kernel ... -p oya-sheets-collab-crdt-worker
cargo nextest run -p oya-sheets-collab-crdt-domain --test no_silent_overwrite
cargo nextest run -p oya-sheets-collab-crdt-adapter-valkey --test valkey_integration -- --include-ignored
cargo run -p oya-dev-cli -- gate validate sheets-crdt-no-silent-loss --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_no_silent_overwrite` (property) | AC-06; 1000 random op-stream pairs; never silent loss |
| `test_merge_commutativity` (property) | merge(a, b) == merge(b, a) for commuting ops |
| `test_conflict_surfaced_when_needed` | overlapping cell-value edits produce Conflict |
| `test_loro_alignment_with_workflow_studio` | Loro 1.x version pinned identically to workflow-studio |
| `test_valkey_lease_single_writer` | only one WS pod holds the lease at a time |
| `test_valkey_ttl_expiry` | abandoned lease expires after TTL |
| `test_cross_tenant_collab_forbidden` | WS subscriber on tenant-A cannot receive tenant-B ops |
| `test_cross_workbook_routing_filter` | server-side filter enforces (subscriber.workbook_id == op.workbook_id) |

## Halt Conditions

- `test_no_silent_overwrite` fails — STOP. AC-06 load-bearing.
- Valkey lease test reveals split-brain — STOP.

## Next IP

[`IP-006-large-sheet-storage-postgres-arrow-parquet-hybrid.md`](IP-006-large-sheet-storage-postgres-arrow-parquet-hybrid.md)

## References

- PRD AC-06 + FR-06.
- threat-model.md T-T-01, T-T-02, T-I-04.
- ADR-SHEETS-0001 (Loro CRDT — aligned with workflow-studio ADR-WS-0001).
- microservices/workflow-studio/decisions/ADR-WS-0001.
- Loro CRDT docs — `loro.dev/docs`.
- Shapiro et al. — "Conflict-free Replicated Data Types" (Inria 2011).
