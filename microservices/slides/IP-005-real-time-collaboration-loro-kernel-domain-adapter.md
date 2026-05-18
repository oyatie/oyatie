---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-005-real-time-collaboration-loro-kernel-domain-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, layer-correctness, no-silent-loss]
depends_on: [IP-002]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: real-time-collaboration with Loro CRDT — kernel + domain + adapter + adapter-redis + adapter-loro

## Intent

Author the Loro 1.x CRDT integration per ADR-SLIDES-0001. AC-06 "never silent loss" invariant is the load-bearing assertion.

## ChangeSet boundary

7 crates:
- `oya-slides-real-time-collaboration-{kernel,domain,usecase,api,adapter,adapter-redis,adapter-loro}`

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-slides-real-time-collaboration-kernel/{src/entities.rs,src/ports.rs}` | create |
| `src/crates/oya-slides-real-time-collaboration-domain/{src/merge.rs,tests/no_silent_overwrite.rs}` | create |
| `src/crates/oya-slides-real-time-collaboration-adapter-loro/{src/loro_impl.rs,tests/integration.rs}` | create |
| `src/crates/oya-slides-real-time-collaboration-adapter-redis/{src/redis_impl.rs,tests/redis_integration.rs}` | create |

## Code Shape

`real-time-collaboration-kernel/src/ports.rs`:

```rust
pub trait CrdtMergeEngine {
    fn apply_op(&mut self, op: &MergeOp) -> Result<ApplyOutcome, MergeError>;
    fn project_to_canonical(&self) -> CanonicalSpec;
    fn snapshot(&self) -> Vec<u8>;
    fn import_snapshot(&mut self, bytes: &[u8]) -> Result<(), MergeError>;
}

pub enum ApplyOutcome {
    Merged,
    ConflictSurfaced(Conflict),
    // Never silent-drop. AC-06 invariant.
}
```

`real-time-collaboration-domain/tests/no_silent_overwrite.rs`:

```rust
use proptest::prelude::*;

proptest! {
    /// AC-06 invariant: for any two CRDT op streams, either the merge succeeds
    /// (all ops applied) OR a Conflict is surfaced — never silent op-drop.
    /// Slides extension: per-slide ACL refinement variants applied; filtered ops
    /// don't count as silent loss (they're filtered at projection layer, not dropped at merge).
    #[test]
    fn test_no_silent_overwrite(
        ops_a in proptest::collection::vec(any::<u32>(), 1..50),
        ops_b in proptest::collection::vec(any::<u32>(), 1..50),
    ) {
        let result = oya_slides_real_time_collaboration_domain::merge::merge_streams(&ops_a, &ops_b);
        match result {
            MergeOutcome::Merged { applied_a, applied_b } => {
                prop_assert_eq!(applied_a, ops_a.len());
                prop_assert_eq!(applied_b, ops_b.len());
            }
            MergeOutcome::Conflict { .. } => { /* correct */ }
        }
    }
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-slides-real-time-collaboration-domain --test no_silent_overwrite
cargo nextest run -p oya-slides-real-time-collaboration-adapter-loro --test integration
cargo nextest run -p oya-slides-real-time-collaboration-adapter-redis --test redis_integration -- --include-ignored
oya gate validate no-silent-loss --microservice slides
```

## Halt Conditions

- `test_no_silent_overwrite` fails — STOP. AC-06 load-bearing.
- Redis lease split-brain detected — STOP. Single-writer guarantee.
- Loro type leaked through kernel port — STOP. ADR-SLIDES-0001 invariant.

## Next IP

IP-006.
