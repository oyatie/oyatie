---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-tasks-foundation
impl_plan_id: IP-002-cargo-workspace-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-tasks
acceptance_lanes: [cargo-build, oya-governance-per-microservice-layout, oya-governance-naming-justification, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Cargo workspace bootstrap — 57 crate stubs per BNF v4.1

## Intent

Scaffold the Cargo workspace for `microservices/tasks/src/` per ADR-0131
flat layout + ADR-0056 BNF v4.1 + ADR-0105 13-layer enum + ADR-0106
`application→usecase` rename. Author 57 crate stubs (per PRD layer-
mapping table) with `Cargo.toml`, `lib.rs` skeleton, and matching
catalog entry under `microservices/tasks/catalog/`.

Each crate skeleton: zero-business-logic stub returning `unimplemented!()`
for ports; `#[data_class(...)]` placeholders on all kernel structs;
naming justification one-liner referencing ADR-0056 v4.1 BNF + ADR-0105
13-layer-enum + Amendment 3 backend-qualification.

## ChangeSet boundary

57 crate stubs (≈ `task-store` × 10 layers + `project-list` × 8 + `view-
engine` × 8 + `dependency-graph` × 7 + `recurrence` × 7 + `search-index`
× 8 + `importers` × 14 — rounding per PRD §Bounded Contexts table).
Plus root workspace `Cargo.toml`.

## Crate Naming

Per PRD §"Bounded Contexts" layer mapping. Crate prefix
`oya-tasks-<bc>-<layer>` per ADR-0056 v4.1. Backend-qualified per
ADR-0105 Amendment 3: `-adapter-postgres`, `-adapter-valkey`,
`-adapter-meilisearch`, `-adapter-csv`, `-adapter-jira`, `-adapter-
asana`, `-adapter-trello`, `-adapter-linear`, `-adapter-todoist`.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/tasks/src/Cargo.toml` | created | workspace root |
| `microservices/tasks/src/oya-tasks-task-store-kernel/Cargo.toml + src/lib.rs` | created | kernel port traits |
| `microservices/tasks/src/oya-tasks-task-store-domain/Cargo.toml + src/lib.rs` | created | invariant math |
| `microservices/tasks/src/oya-tasks-task-store-usecase/Cargo.toml + src/lib.rs` | created | orchestrators |
| `microservices/tasks/src/oya-tasks-task-store-{api,adapter,adapter-postgres,rest,worker,sdk,app}/...` | created | rest of stack |
| ...remaining BCs (project-list, view-engine, dependency-graph, recurrence, search-index, importers)... | created | per PRD table |
| `microservices/tasks/catalog/oya-tasks-*.yaml` | created | catalog entry per crate (handled in IPs that own each BC) |

## Acceptance Gates

```bash
cargo build --workspace --manifest-path microservices/tasks/src/Cargo.toml
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice tasks
cargo run -p oya-dev-cli -- gate validate naming-justification --microservice tasks
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice tasks
```

## Test Plan

- `cargo build --workspace` exits 0 with all 57 stubs compiling.
- naming-justification lane parses one-liner per crate.
- layer-correctness lane enforces port-in-kernel + inward-only-flow.

## Halt Conditions

- Any crate fails BNF v4.1 parse — root-cause; do not bypass.
- layer-correctness lane red — refuse to bundle; fix at root.

## Next IP

[`IP-003-task-store-kernel-domain.md`](IP-003-task-store-kernel-domain.md)

## References

- ADR-0056 (BNF v4.1); ADR-0105 (13-layer); ADR-0106 (usecase rename); ADR-0131 (flat layout).
- `microservices/tasks/specs/naming-justification.md`.
