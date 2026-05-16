---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P03-IP-001
title: oya-audit-chain-domain Merkle + Ed25519
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions:
  - ed25519-dalek@2.2.0
  - sha2@0.10.9
purpose: Implement the audit-chain domain Merkle root and Ed25519 signing/verification kernel.
---

# M01-P03-IP-001 — oya-audit-chain-domain Merkle + Ed25519

## Purpose
Implement the audit-chain domain kernel for append-only, per-tenant-shard event chains with Merkle prefix roots and Ed25519 signatures.

## Symbols-to-grit-claim
```
crates/oya-audit-chain-domain/src/lib.rs::AuditEvent
crates/oya-audit-chain-domain/src/lib.rs::MerkleRoot
crates/oya-audit-chain-domain/src/lib.rs::Ed25519Signature
crates/oya-audit-chain-domain/src/lib.rs::append
crates/oya-audit-chain-domain/src/lib.rs::verify_chain
crates/oya-audit-chain-domain/tests/merkle_chain.rs
crates/oya-audit-chain-domain/tests/ed25519_sign_verify.rs
```
(Scaffold-claim fallback recorded because `grit claim` returned the known FK failure for mixed current/new symbol scope.)

## Agent-prerequisites
M01-P02 complete.

## Acceptance-test-commands
```
cargo test -p oya-audit-chain-domain --test merkle_chain
cargo test -p oya-audit-chain-domain --test ed25519_sign_verify
```

## Done-criteria
- All acceptance-test commands return 0.
- No deployed binary/image ships in this IP.
- No provider-specific deps outside adapter crates.
- Direct dependency additions are limited to `ed25519-dalek` 2.x stable for real Ed25519 sign/verify and `sha2` 0.10.x for collision-resistant SHA-256 event/Merkle digests; ed25519-dalek 3.x is still prerelease as of the checked upstream docs.
- Good-taste audit: append-only event replay remains a read-only vector; the only production mutation path is append, and the chain refuses cross-tenant-shard append.

## Verification
- `rustfmt --edition 2024 --check` on audit-chain domain/application/file-adapter touched files: pass.
- `cargo test -p oya-audit-chain-domain`: pass, 5 tests.
- `cargo test -p oya-audit-chain-application -p oya-audit-chain-file-adapter`: pass, 8 tests.
- `PATH="$(dirname "$(rustup which cargo-clippy)"):$PATH" cargo clippy -p oya-audit-chain-domain -p oya-audit-chain-application -p oya-audit-chain-file-adapter --all-targets -- -D warnings`: pass.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs already depend on, follow the per-crate split unwind documented in ADR-0015 §7.

## Next-IP-pointer
M01-P03-IP-002 (AsyncAPI + Proto)

## Icm-store-payload
```
icm store -t context-oyatie -c 'audit-chain-domain Merkle + Ed25519 kernel shipped; append + verify_chain proven' -i critical -k 'M01,P03,IP-001,audit-chain,merkle,ed25519'
```

## Decision-log (Linus good-taste row)
Audit chain is append-only + SHA-256 hash-chained + Merkle-rooted at the type level, and signed verification requires a trusted Ed25519 key set as a separate explicit proof gate — eliminates 'rewrite audit log' as an expressible operation while keeping unsigned legacy append compatibility visible.
