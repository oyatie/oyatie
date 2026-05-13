---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P01-IP-002
title: oya-platform-tenant-kernel final-shape contracts
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Ship the tenant kernel with engine-enforced row-level isolation per ADR-0006.
---

# M01-P01-IP-002 — oya-platform-tenant-kernel final-shape contracts

## Purpose
Ship the tenant kernel with engine-enforced row-level isolation per ADR-0006.

## Symbols-to-grit-claim
```
crates/oya-platform-tenant-kernel/src/lib.rs::Tenant
crates/oya-platform-tenant-kernel/src/lib.rs::TenantId
crates/oya-platform-tenant-kernel/src/lib.rs::RegionBinding
crates/oya-platform-tenant-kernel/src/lib.rs::ResidencyClass
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
IP-001 ADR-0008 merged.

## Acceptance-test-commands
```
cargo test -p oya-platform-tenant-kernel
cargo run -p oya-foundry-fitness-cohesion -- crates/oya-platform-tenant-*
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (per MASTERPLAN §2 Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs already depend on, follow the per-crate split unwind documented in ADR-0015 §7.

## Next-IP-pointer
M01-P01-IP-003 (DSR cascade engine)

## Icm-store-payload
```
icm store -t context-oyatie -c 'oya-platform-tenant-kernel shipped with row-level isolation + region binding immutable post-create' -i critical -k 'M01,P01,IP-002,tenant-kernel,complete'
```

## Decision-log (Linus good-taste row)
Tenant ID is globally unique with region binding immutable post-create — eliminates 'tenant relocation' edge cases by structurally disallowing them.
