---
purpose: Ship the tenancy kernel with immutable region binding and engine-enforced row-level isolation contracts per ADR-0002, ADR-0006, ADR-0049, and PRD-tenancy.
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P01-IP-002
title: oya-tenancy-kernel final-shape contracts
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship the tenancy kernel with immutable region binding and engine-enforced row-level isolation contracts per ADR-0002, ADR-0006, ADR-0049, and PRD-tenancy.
---

# M01-P01-IP-002 — oya-tenancy-kernel final-shape contracts

## Purpose
Ship the tenancy kernel with immutable region binding and engine-enforced row-level isolation contracts per ADR-0002, ADR-0006, ADR-0049, and PRD-tenancy.

## Symbols-to-grit-claim
```
crates/oya-tenancy-kernel/src/lib.rs::Tenant
crates/oya-tenancy-kernel/src/lib.rs::TenantId
crates/oya-tenancy-kernel/src/lib.rs::RegionBinding
crates/oya-tenancy-kernel/src/lib.rs::ResidencyClass
crates/oya-tenancy-kernel/src/lib.rs::TenantContext
crates/oya-tenancy-kernel/src/lib.rs::TenantScopedRecord
Cargo.toml::workspace.members
```

`grit claim` returned the known FK failure for new symbols, so ADR-0054 scaffold locks were used:
- `01KRKFAT7DMF4KBKTAK5HR78NX` — `oya-tenancy-kernel` + workspace/IP/evidence/masterplan scope.
- `01KRKFEZXPT8ETDRRK8AYGZDEG` — data-boundary `FINANCIAL_KR` canonical label alignment scope extension.

## Agent-prerequisites
IP-001 ADR-0008 Accepted and code-review APPROVE/CLEAR.

## Acceptance-test-commands
```
rustfmt --check crates/oya-tenancy-kernel/src/lib.rs crates/oya-data-boundary-kernel/src/lib.rs
cargo test -p oya-tenancy-kernel
cargo check -p oya-tenancy-kernel
cargo test -p oya-data-boundary-kernel
cargo check -p oya-data-boundary-kernel
cargo test -p oya-tenancy-domain
cargo metadata --no-deps --format-version 1
```

## Done-criteria
- All acceptance-test commands return 0.
- `crates/oya-tenancy-kernel` is a workspace member and inherits Rust 1.95.0 / edition 2024 from `[workspace.package]`.
- `TenantId`, `Tenant`, `RegionBinding`, and `ResidencyClass` are pure kernel contracts with no external dependencies.
- `RegionBinding` is immutable post-create by construction (private fields + accessors only).
- `TenantScopedRecord::require_tenant` denies cross-tenant reads before adapter/database-specific RLS.
- No provider-specific deps outside adapter crates (per MASTERPLAN §2 Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8); this IP adds no external direct deps.
- Good-taste audit section non-empty.

## Rollback-procedure
`grit done` is atomic per-symbol when a native claim exists; this IP used ADR-0054 scaffold-lock fallback because the native new-symbol claim failed FK validation. If a subsequent IP regresses, revert the merge commit. For crates that other IPs already depend on, follow the per-crate split unwind documented in ADR-0015 §7.

## Next-IP-pointer
M01-P01-IP-003 (DSR cascade engine)

## Icm-store-payload
```
icm store -t context-oyatie -c 'M01-P01-IP-002 complete: oya-tenancy-kernel shipped with TenantId, immutable RegionBinding, ResidencyClass, TenantContext, TenantScopedRecord row-level guard; Rust 1.95/edition 2024 gates green' -i critical -k 'M01,P01,IP-002,tenant-kernel,complete'
```

## Decision-log (Linus good-taste row)
Tenant ID and region binding are private newtypes with no mutation setters; the weird case (tenant relocation) is not represented as a field update. Cross-region/residency change must become a new tenant + DSR cascade path instead of a special-case mutable transition.

## Completion evidence

Completed on 2026-05-14. Evidence bundle: [`../../../../../../evidence/foundation/m01-p01-ip-002-tenant-kernel-contracts.json`](../../../../../../evidence/foundation/m01-p01-ip-002-tenant-kernel-contracts.json).

Fresh gates:
- `rustfmt --check crates/oya-tenancy-kernel/src/lib.rs crates/oya-data-boundary-kernel/src/lib.rs` → pass.
- `cargo test -p oya-tenancy-kernel` → 6 passed.
- `cargo check -p oya-tenancy-kernel` → pass.
- `cargo test -p oya-data-boundary-kernel` → 13 passed.
- `cargo check -p oya-data-boundary-kernel` → pass.
- `cargo test -p oya-tenancy-domain` → 2 passed (existing compatibility crate unaffected).
- `cargo metadata --no-deps --format-version 1` → `packages=163 workspace_members=163`; `oya-tenancy-kernel` present.

Known pre-existing blocker:
- `scripts/check.sh` remains blocked by missing `scripts/check-stage0-application-shell-prereqs.py`.
