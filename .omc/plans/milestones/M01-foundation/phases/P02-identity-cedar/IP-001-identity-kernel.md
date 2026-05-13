---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P02-IP-001
title: oya-platform-identity-kernel user + region + IdP binding
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Ship the per-tenant user upsert kernel with per-region IdP binding.
---

# M01-P02-IP-001 — oya-platform-identity-kernel user + region + IdP binding

## Purpose
Ship the per-tenant user upsert kernel with per-region IdP binding.

## Symbols-to-grit-claim
```
crates/oya-platform-identity-kernel/src/lib.rs::User
crates/oya-platform-identity-kernel/src/lib.rs::UserId
crates/oya-platform-identity-kernel/src/lib.rs::IdpBinding
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
M01-P01-IP-002 (tenant kernel) merged.

## Acceptance-test-commands
```
cargo test -p oya-platform-identity-kernel
cargo run -p oya-foundry-fitness-cohesion -- crates/oya-platform-identity-*
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
M01-P02-IP-002 (STS rotation)

## Icm-store-payload
```
icm store -t context-oyatie -c 'oya-platform-identity-kernel shipped with per-tenant uniqueness + per-region IdP binding' -i critical -k 'M01,P02,IP-001,identity-kernel,complete'
```

## Decision-log (Linus good-taste row)
User-region binding is part of the identity contract — eliminates 'user moved across regions' edge case by structurally disallowing it.
