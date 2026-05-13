---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P02-IP-002
title: oya-platform-identity-app STS issuance + rotation
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Ship STS issuance bound to ≤1h purpose-bound credentials; no long-lived API keys.
---

# M01-P02-IP-002 — oya-platform-identity-app STS issuance + rotation

## Purpose
Ship STS issuance bound to ≤1h purpose-bound credentials; no long-lived API keys.

## Symbols-to-grit-claim
```
crates/oya-platform-identity-app/src/lib.rs::issue_sts_token
crates/oya-platform-identity-app/src/lib.rs::rotate_sts_token
crates/oya-platform-identity-app/src/lib.rs::PurposeScope
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
IP-001 identity kernel merged.

## Acceptance-test-commands
```
cargo test -p oya-platform-identity-app --test sts_max_1h
cargo test -p oya-platform-identity-app --test no_long_lived_keys
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
M01-P02-IP-003 (Cedar policy substrate)

## Icm-store-payload
```
icm store -t context-oyatie -c 'STS issuance ≤1h purpose-bound enforced at type level; long-lived API keys structurally rejected' -i critical -k 'M01,P02,IP-002,sts,complete'
```

## Decision-log (Linus good-taste row)
Long-lived API key data type does not exist — eliminates the 'leaked-key-lives-forever' failure class structurally.
