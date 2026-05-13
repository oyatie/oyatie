---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P03-IP-001
title: oya-platform-audit-chain-kernel Merkle + Ed25519
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Implement ADR-0003 Merkle-sealed audit chain with Ed25519 signing.
---

# M01-P03-IP-001 — oya-platform-audit-chain-kernel Merkle + Ed25519

## Purpose
Implement ADR-0003 Merkle-sealed audit chain with Ed25519 signing.

## Symbols-to-grit-claim
```
crates/oya-platform-audit-chain-kernel/src/lib.rs::AuditEvent
crates/oya-platform-audit-chain-kernel/src/lib.rs::MerkleRoot
crates/oya-platform-audit-chain-kernel/src/lib.rs::Ed25519Signature
crates/oya-platform-audit-chain-kernel/src/lib.rs::append
crates/oya-platform-audit-chain-kernel/src/lib.rs::verify_chain
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
M01-P02 merged.

## Acceptance-test-commands
```
cargo test -p oya-platform-audit-chain-kernel --test merkle_chain
cargo test -p oya-platform-audit-chain-kernel --test ed25519_sign_verify
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
M01-P03-IP-002 (AsyncAPI + Proto)

## Icm-store-payload
```
icm store -t context-oyatie -c 'audit-chain-kernel Merkle + Ed25519 shipped; append + verify_chain proven' -i critical -k 'M01,P03,IP-001,audit-chain,merkle'
```

## Decision-log (Linus good-taste row)
Audit chain is append-only + hash-chained at the type level — eliminates 'rewrite audit log' as an expressible operation.
