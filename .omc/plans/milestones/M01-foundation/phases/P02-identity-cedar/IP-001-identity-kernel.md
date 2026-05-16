---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P02-IP-001
title: oya-identity-domain user + region + IdP binding
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship the per-tenant user upsert kernel with per-region IdP binding.
evidence_ref: ../../../../../evidence/foundation/m01-p02-ip-001-identity-kernel.json
---

# M01-P02-IP-001 — oya-identity-domain user + region + IdP binding

## Purpose
Ship the per-tenant user upsert kernel with per-region IdP binding.

## Symbols-to-grit-claim
```
crates/oya-identity-domain/src/lib.rs::User
crates/oya-identity-domain/src/lib.rs::UserId
crates/oya-identity-domain/src/lib.rs::IdpBinding
crates/oya-platform-identity-api/src/lib.rs::upsert_identity_user_from_api
```
(Scaffold-claim used per ADR-0054 after `grit claim` returned the known FK failure for the mixed current/stale symbol scope.)

## Agent-prerequisites
M01-P01-IP-002 (tenant kernel) complete.

## Acceptance-test-commands
```
cargo test -p oya-identity-domain
cargo test -p oya-platform-identity-api
cargo test -p oya-identity-application
cargo clippy -p oya-identity-domain -- -D warnings
cargo clippy -p oya-platform-identity-api -- -D warnings
```

## Done-criteria
- All acceptance-test commands return 0.
- Workspace inherits Rust `1.95.0`, edition `2024`, and repo-root rustfmt `style_edition = "2024"`.
- No provider-specific deps outside adapter crates (per MASTERPLAN §2 Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Evidence recorded at `/evidence/foundation/m01-p02-ip-001-identity-kernel.json`.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs already depend on, follow the per-crate split unwind documented in ADR-0015 §7.

## Next-IP-pointer
M01-P02-IP-002 (STS rotation) and M01-P02-IP-003 (Cedar policy substrate) are both unblocked after this identity kernel lands; keep shared doc/Cargo surfaces serialized.

## Icm-store-payload
```
icm store -t context-oyatie -c 'oya-identity-domain shipped with per-tenant user identity + per-region IdP binding; platform identity user API is a workspace member and passes user-upsert regression coverage' -i critical -k 'M01,P02,IP-001,identity-kernel,complete'
```

## Decision-log (Linus good-taste row)
User-region binding is part of the identity contract — eliminates 'user moved across regions' edge case by structurally requiring an IdP binding on every `User`.
