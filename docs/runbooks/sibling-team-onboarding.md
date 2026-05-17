# Sibling Team Onboarding: Shipping a Product Vertical on M02 Substrate

**Runbook ID:** sibling-team-onboarding
**Phase:** P22-m02-exit-gate
**Milestone:** M02-substrate
**Owner:** council-architecture

---

## Prerequisites

- Rust stable toolchain + cargo nextest installed
- Access to oyatie workspace (`git clone` or worktree)
- No external agent-coordination tooling required (grit/icm/vox retired; plain cargo + gh)

---

## Step 1: Claim your work (plain git)

```bash
git checkout -b feature/<your-team-id>/<feature-slug> origin/dev
```

No grit claim needed — external agent-coordination tooling is retired (deprecated 2026-05-16).

---

## Step 2: Scaffold your product crate

BNF v4.1 name: `oya-<microservice>(-<bc-tokens>)?-<layer>`

Layer options (ADR-0105 13-layer enum):
`kernel` | `domain` | `application` | `adapter` | `rest` | `grpc` | `worker` | `cli` | `sdk`
| `runner` | `shared` | `tests` | `bench`

Rules:
- **Kernel**: declares port traits (`Send + Sync`). No impls.
- **Domain**: business logic. No infra deps.
- **Adapter**: implements port traits. Postgres/HTTP/gRPC allowed here.
- **Application**: orchestrates domain + ports via use-case structs.

Add workspace entry in `Cargo.toml`:

```toml
# under [workspace]
members = [
  # ... existing ...
  "crates/oya-<your-microservice>-kernel",
  "crates/oya-<your-microservice>-domain",
  "crates/oya-<your-microservice>-adapter",
]
```

---

## Step 3: Cross-product actions — use WorkflowBridgePort

All actions that cross product boundaries go through Workflow.

```rust
// In your application crate:
use oya_workflow_engine_kernel::WorkflowBridgePort;

// Call submit_action — NEVER import another product's kernel directly for actions.
workflow_bridge.submit_action(action).await?;
```

---

## Step 4: Cross-product data — use Ontology ObjectStore

All reads/writes of shared entities go through Ontology.

```rust
// In your application crate:
use oya_ontology_entity_kernel::{ObjectStore, LinkStore, ActionStore};

// NEVER query another product's schema directly.
let entity = object_store.get::<MyEntity>(id).await?;
```

---

## Step 5: Run all CI lanes locally before PR

```bash
cargo check --workspace --all-features
cargo nextest run --workspace --all-features
cargo run -p oya-check-architecture -- cross-product-refusal --workspace
cargo run -p oya-check-architecture -- dependency-direction --workspace
cargo run -p oya-check-statelessness -- --workspace
cargo deny check
```

All 14 fitness lanes run in **BLOCKER mode** on CI (P22 flip). They must all exit 0 before merge.

---

## Step 6: Ship

```bash
git push -u origin feature/<your-team-id>/<feature-slug>
gh pr create --base dev --title "<your feature>" --body "..."
```

CI runs all 14 lanes in BLOCKER mode. All must exit 0. PR merges automatically via merge queue.

---

## Architecture constraints (summary)

| Rule | Enforcement lane |
|---|---|
| No cross-product direct imports | `architecture-cross-product-refusal` |
| Inward-only dependency direction | `architecture-dependency-direction` |
| Layer correctness (13-layer enum) | `architecture-layer-correctness` |
| Port traits in kernel only | `architecture-port-location` |
| Composition root in runner/cli only | `architecture-composition-root-only` |
| SDK public surface in kernel only | `architecture-sdk-kernel-only` |
| Canonical-base neutrality (ADR-0064 §8) | `architecture-canonical-base-neutrality` |
| No cross-pack tenant leakage (ADR-0064 §7) | `architecture-cross-pack-refusal` |
| Stateless µservice | `statelessness-check` |
| Shardable schema | `shardability-check` |
| Full doc coverage (LEAN-A5; ADR-0063) | `doc-coverage-check` |
| Performance budget (ADR-0062) | `perf-budget-check` |
| Benchmark regression | `benchmark-check` |

---

## References

- ADR-0056 v4.1: BNF + check-namespace
- ADR-0062: quality/performance bar
- ADR-0063: doc-coverage LEAN-A5
- ADR-0064: canonical-base-neutrality + cross-pack-refusal
- ADR-0105: 13-layer enum (canonical)
- `docs/architecture/m02-exit-checklist.md`: M02 gate evidence
