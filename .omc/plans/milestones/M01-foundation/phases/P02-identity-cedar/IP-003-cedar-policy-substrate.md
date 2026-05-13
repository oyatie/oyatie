---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P02-IP-003
title: Cedar policy substrate (-policy-cedar-*) + publish + supersession
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Ship the Cedar RBAC/ABAC substrate that every capability invocation enforces against.
---

# M01-P02-IP-003 — Cedar policy substrate (-policy-cedar-*) + publish + supersession

## Purpose
Ship the Cedar RBAC/ABAC substrate that every capability invocation enforces against.

## Symbols-to-grit-claim
```
crates/oya-platform-policy-cedar-kernel/src/lib.rs::CedarPolicy
crates/oya-platform-policy-cedar-kernel/src/lib.rs::PolicyVersion
crates/oya-platform-policy-cedar-app/src/lib.rs::publish
crates/oya-platform-policy-cedar-app/src/lib.rs::supersede
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
IP-001 identity kernel merged.

## Acceptance-test-commands
```
cargo test -p oya-platform-policy-cedar-app
cargo run -p oya-foundry-fitness-policy-versioning
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
M01-P03-IP-001 (audit chain)

## Icm-store-payload
```
icm store -t context-oyatie -c 'Cedar policy substrate shipped; versioned + supersession-chained; M01-P02 acceptance gate ready' -i critical -k 'M01,P02,IP-003,cedar-substrate,complete'
```

## Decision-log (Linus good-taste row)
Policy publish is versioned + semver + supersession-chained — eliminates 'which version of the policy was active' debug ambiguity.
