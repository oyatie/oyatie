---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P01-IP-003
title: tenant.dsr.cascade ≤30d cascade engine
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Ship the DSR cascade engine that satisfies SPEC §2 'tenant.dsr.cascade' with proof-of-erasure per affected store.
---

# M01-P01-IP-003 — tenant.dsr.cascade ≤30d cascade engine

## Purpose
Ship the DSR cascade engine that satisfies SPEC §2 'tenant.dsr.cascade' with proof-of-erasure per affected store.

## Symbols-to-grit-claim
```
crates/oya-platform-dsr-app/src/lib.rs::DsrCascade
crates/oya-platform-dsr-app/src/lib.rs::execute
crates/oya-platform-dsr-app/src/lib.rs::ProofOfErasure
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
IP-002 tenant kernel merged.

## Acceptance-test-commands
```
cargo test -p oya-platform-dsr-app --test cascade_30d
cargo test -p oya-platform-dsr-app --test proof_of_erasure_per_store
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
M01-P02-IP-001 (identity kernel)

## Icm-store-payload
```
icm store -t context-oyatie -c 'DSR cascade ≤30d demonstrated; proof-of-erasure per affected store green; M01-P01 acceptance gate ready' -i critical -k 'M01,P01,IP-003,dsr-cascade,complete'
```

## Decision-log (Linus good-taste row)
DSR cascade is a single engine traversing the per-tenant ER graph — eliminates N per-axis DSR re-implementations.
