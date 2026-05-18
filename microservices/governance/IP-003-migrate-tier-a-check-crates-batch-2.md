---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-003-migrate-tier-a-check-crates-batch-2
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, cross-ref-validity, per-microservice-layout]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: Migrate Tier-A check crates batch 2 (5 crates)

## Intent

Atomic ChangeSet: migrate 5 more tier-A `oya-check-*` crates per ADR-0131 IP-M01-MIGR-014.

Crates in this batch (tier-A security + content lanes):
1. `oya-check-data-class`
2. `oya-check-supply-chain`
3. `oya-check-license-policy`
4. `oya-check-placeholder-debt`
5. `oya-check-brand-residue`

## ChangeSet boundary

Same shape as IP-002. 5 `git mv`; 5 workspace path updates; 5 catalog rows.

## Concrete File Targets

Per crate `C` ∈ {data-class, supply-chain, license-policy, placeholder-debt, brand-residue}:
- `git mv crates/oya-check-$C → microservices/governance/src/crates/oya-check-$C`.
- Workspace member update.
- `microservices/governance/catalog/oya-check-$C.yaml` (NEW).

## Acceptance Gates

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo run -p oya-dev-cli -- gate validate cross-ref-validity
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice governance
# Self-application
cargo run -p oya-dev-cli -- gate validate data-class --microservice governance
cargo run -p oya-dev-cli -- gate validate supply-chain --microservice governance
cargo run -p oya-dev-cli -- gate validate license-policy --microservice governance
cargo run -p oya-dev-cli -- gate validate placeholder-debt --microservice governance
cargo run -p oya-dev-cli -- gate validate brand-residue --microservice governance
```

## Test Plan

Same as IP-002: per-crate test suite + cross-ref validation + workspace integrity + self-application.

## Halt Conditions

Same as IP-002.

## Next IP

[`IP-004-lane-runtime-kernel-domain.md`](IP-004-lane-runtime-kernel-domain.md)

## References

- ADR-0131 §"Migration DAG → IP-M01-MIGR-014".
- ADR-0132 §"governance umbrella".
- `microservices/governance/runbooks/migration-execution.md` §A.
- Bominal ADR-0028 (data-class taxonomy).
