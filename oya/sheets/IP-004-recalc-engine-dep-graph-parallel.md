---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-sheets-preview
phase: P01-sheets-foundation
impl_plan_id: IP-004-recalc-engine-dep-graph-parallel
status: pending
owner: axis-sheets
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, oya-governance-sheets-recalc-determinism]
depends_on: [IP-003]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: recalc-engine — kernel + domain + usecase + api + adapter + worker + sdk (dep-graph + topological + parallel-task-graph per ADR-SHEETS-0004)

## Intent

Author the `recalc-engine` BC's full crate set: dependency-graph builder + topological sort + parallel-task-graph executor + cycle detection + slow-formula budget enforcement. Achieves 100k-cell ≤ 1s + 1M-cell ≤ 10s p95 targets per ADR-SHEETS-0004 + ADR-SHEETS-0003 hybrid storage path.

## ChangeSet boundary

Seven crates:
- `oya-sheets-recalc-engine-{kernel,domain,usecase,api,adapter,worker,sdk}`

## Code Shape

`recalc-engine-domain/src/dep_graph.rs` (excerpt):

```rust
use rayon::prelude::*;

pub struct DepGraph {
    pub edges: Vec<(CellRef, CellRef)>,  // (dependent, dependency)
}

impl DepGraph {
    /// Topological sort returning recalc levels; cells in same level are parallel-safe.
    pub fn topological_levels(&self) -> Result<Vec<Vec<CellRef>>, CycleDetected> {
        // Kahn's algorithm; refuse on cycle (return Err)
    }

    /// Parallel recalc plan executor; rayon thread pool sized to pod CPU count.
    pub fn execute(&self, evaluator: &FormulaEvaluator, dirty: &DirtySet) -> RecalcResult {
        for level in self.topological_levels()? {
            level.par_iter().for_each(|cell_ref| {
                // 30s slow-formula budget; kill if exceeded → #SLOW! error
                let _ = with_budget(Duration::from_secs(30), || evaluator.eval(cell_ref));
            });
        }
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-sheets-recalc-engine-kernel ... -p oya-sheets-recalc-engine-worker
cargo nextest run -p oya-sheets-recalc-engine-domain
buck2 build //:quality-lane-registry-authority-check # lane=sheets-recalc-determinism --microservice sheets
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_topological_sort` | dep-graph topo-sort correctness on N=1000 random DAGs |
| `test_cycle_detection` | cycles refused (Err); never infinite-loop |
| `test_parallel_recalc_determinism` | same dep-graph + same edits → identical final state regardless of rayon thread interleaving |
| `test_slow_formula_budget` | formulas > 30s killed; `#SLOW!` returned |
| `test_recalc_100k_cell_budget` | 100k-cell sheet recalc p95 ≤ 1s (per AC-07) |
| `test_recalc_1m_cell_budget` | 1M-cell workbook recalc p95 ≤ 10s (per AC-08) |
| `test_hot_cold_arrow_boundary` | recalc transparently spans Postgres hot tier + Arrow cold tier per ADR-SHEETS-0003 |

## Halt Conditions

- Non-determinism in parallel recalc — STOP. ADR-SHEETS-0004 load-bearing.
- 100k or 1M budget breach on reference hardware — STOP; investigate before merge.

## Next IP

[`IP-005-collab-crdt-loro-aligned-ws-0001.md`](IP-005-collab-crdt-loro-aligned-ws-0001.md)

## References

- PRD AC-07 + AC-08.
- ADR-SHEETS-0003 (large-sheet storage).
- ADR-SHEETS-0004 (recalc-engine architecture).
- rayon — `docs.rs/rayon`.
- "Spreadsheets and Calculation" — Joel Spolsky reference on Excel dep-graph.
