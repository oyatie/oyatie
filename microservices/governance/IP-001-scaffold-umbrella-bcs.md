---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-001-scaffold-umbrella-bcs
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, oya-governance-per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: Scaffold governance umbrella BCs (4 BCs × 9 layers = 36 crates)

## Intent

Create the 36 new umbrella crates that compose the governance µservice's four bounded contexts (`lane-runtime`, `policy-engine`, `evidence-emitter`, `aggregation-indexer`), one per ADR-0105 13-layer enum (using 9 layers per BC: kernel, domain, usecase, api, adapter, rest, worker, sdk, app). Workspace registration; catalog rows.

## ChangeSet boundary

36 new Rust crates under `microservices/governance/src/crates/`. Workspace `[workspace.members]` updated. 36 catalog rows at `microservices/governance/catalog/`. No downstream code dependencies in this IP; only structural scaffold. Subsequent IPs (IP-004, IP-006, IP-008, IP-010) fill in kernel + domain types per BC.

## Concrete File Targets

For each BC ∈ {lane-runtime, policy-engine, evidence-emitter, aggregation-indexer}:
- 9 crates created under `microservices/governance/src/crates/oya-governance-<BC>-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}/`
- Each crate: `Cargo.toml` + `src/lib.rs` (module declarations only) + minimal types.

Plus:
- `Cargo.toml` (workspace) — add 36 paths to `[workspace.members]`.
- `microservices/governance/catalog/oya-governance-<BC>-<LAYER>.yaml` × 36.

## Crate naming

```
NAME: oya-governance-<bc>-<layer>
JUSTIFICATION:
- microservice = governance (microservices/governance/)
- bc-tokens ∈ {lane-runtime, policy-engine, evidence-emitter, aggregation-indexer} (per PRD §"Bounded Contexts")
- layer ∈ {kernel, domain, usecase, api, adapter, rest, worker, sdk, app} (ADR-0105 13-value enum; using 9 of them)
- exemptions claimed: none
```

## Acceptance Gates

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice governance
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice governance
cargo run -p oya-dev-cli -- gate validate naming-bnf-v41 --microservice governance
```

## Test Plan

Per PHASE-01 §"Per-IP Test Coverage Threshold". M01-A tier crates: 80% line coverage minimum. Scaffold-only IPs (IP-001) carry placeholder smoke tests.

| Test | Verifies |
|---|---|
| `test_module_declarations_compile` | every crate's lib.rs compiles |
| `test_workspace_members_resolved` | `cargo metadata` lists all 36 members |
| `test_catalog_row_per_crate` | catalog/ has 36 rows |

## Halt Conditions

- Workspace member path collision — refactor naming.
- BNF v4.1 naming violation — refer to `feedback_naming_justification.md`.

## Next IP

[`IP-002-migrate-tier-a-check-crates-batch-1.md`](IP-002-migrate-tier-a-check-crates-batch-1.md)

## References

- ADR-0056 BNF v4.1; ADR-0105 13-layer enum; ADR-0106 application → usecase rename.
- ADR-0131 §"per-microservice flat layout".
- ADR-0132 §"no-suite forward-policy".
- `microservices/governance/PRD.md` §"Bounded Contexts".
