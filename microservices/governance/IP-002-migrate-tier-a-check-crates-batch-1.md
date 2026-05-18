---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-002-migrate-tier-a-check-crates-batch-1
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, cross-ref-validity, per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-002: Migrate Tier-A check crates batch 1 (5 crates)

## Intent

Atomic per-ADR-0110 ChangeSet: migrate 5 tier-A `oya-check-*` crates from `crates/` to `microservices/governance/src/crates/` per ADR-0131 §"Migration DAG → IP-M01-MIGR-014" + ADR-0132. Per ADR-0131 §"Crate naming inside each `microservices/<ms>/crates/` subtree is unchanged", crate names retained.

Crates in this batch (tier-A clean-arch foundation lanes):
1. `oya-check-lean-a1` (dependency-direction)
2. `oya-check-lean-a2` (cross-product-refusal)
3. `oya-check-port-location`
4. `oya-check-layer-correctness`
5. `oya-check-naming-bnf-v41`

## ChangeSet boundary

5 directory moves via `git mv`; 5 workspace member paths updated; 5 catalog rows authored at `microservices/governance/catalog/`. All cross-refs to old paths updated atomically (per `runbooks/migration-execution.md` §A).

## Concrete File Targets

Per crate `C`:
- `git mv crates/$C → microservices/governance/src/crates/$C` (history preserved).
- `Cargo.toml` (workspace): `crates/$C` → `microservices/governance/src/crates/$C`.
- `microservices/governance/catalog/$C.yaml` (NEW).
- Any cross-ref in docs / Rust deps / CI workflows updated via `ast-grep` (NOT raw sed, per ADR-0131 §"No-blanket-sed").

## Acceptance Gates

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo run -p oya-dev-cli -- gate validate cross-ref-validity
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice governance
# Self-application: each migrated lane runs on the governance µservice itself
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice governance
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice governance
cargo run -p oya-dev-cli -- gate validate port-location --microservice governance
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice governance
cargo run -p oya-dev-cli -- gate validate naming-bnf-v41 --microservice governance
```

## Test Plan

| Test | Verifies |
|---|---|
| Per-crate existing test suite | unchanged behaviour post-migration |
| Cross-ref validation across moved paths | no broken links |
| `cargo nextest run --workspace` | workspace integrity |
| Self-application | each lane passes on `microservices/governance/` itself |

## Halt Conditions

- Self-application fails on the governance µservice itself → use synthetic-probe fallback per `runbooks/lane-failure-triage.md` §C2; halt and fix in same IP.
- Broken cross-ref → halt; structured fix via `ast-grep`.
- `git mv` history loss → halt; redo move.

## Next IP

[`IP-003-migrate-tier-a-check-crates-batch-2.md`](IP-003-migrate-tier-a-check-crates-batch-2.md)

## References

- ADR-0131 §"Migration DAG → IP-M01-MIGR-014" + §"completion gate".
- ADR-0132 §"governance umbrella".
- `microservices/governance/runbooks/migration-execution.md` §A.
- `feedback_clean_architecture_requirements.md` (LEAN-A1 + LEAN-A2 authority).
