---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P02-IP-002
title: oya-identity-application STS issuance + rotation
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship STS issuance bound to ≤1h purpose-bound credentials; no long-lived API keys.
evidence_ref: ../../../../../evidence/foundation/m01-p02-ip-002-sts-rotation.json
---

# M01-P02-IP-002 — oya-identity-application STS issuance + rotation

## Purpose
Ship STS issuance and rotation bound to ≤1h purpose-bound credentials; no long-lived API keys.

## Symbols-to-grit-claim
```
crates/oya-identity-application/src/lib.rs::issue_identity_token_from_app
crates/oya-identity-application/src/lib.rs::rotate_identity_token_from_app
crates/oya-identity-application/src/lib.rs::PurposeScope
crates/oya-identity-application/tests/identity_token_issue.rs
```
(Scaffold-claim used per ADR-0054 after `grit claim` returned the known FK failure for the mixed current/new symbol scope.)

## Agent-prerequisites
M01-P02-IP-001 identity kernel complete.

## Acceptance-test-commands
```
cargo test -p oya-identity-application
cargo clippy -p oya-identity-application -- -D warnings
rustfmt --check crates/oya-identity-application/src/lib.rs crates/oya-identity-application/tests/identity_token_issue.rs
```

## Done-criteria
- All acceptance-test commands return 0.
- STS issuance remains idempotent, purpose-bound, scope-bound, and ≤1h.
- STS rotation accepts only active prior STS records and preserves tenant/subject/credential binding plus purpose/scope.
- Long-lived API-key material is rejected at the application parser before typed issuance.
- No provider-specific deps outside adapter crates (per MASTERPLAN §2 Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Evidence recorded at `.omc/evidence/foundation/m01-p02-ip-002-sts-rotation.json`.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs already depend on, follow the per-crate split unwind documented in ADR-0015 §7.

## Next-IP-pointer
M01-P02-IP-003 (Cedar policy substrate)

## Icm-store-payload
```
icm store -t context-oyatie -c 'STS issuance and rotation are ≤1h purpose-bound; long-lived API-key requests are rejected before typed issuance' -i critical -k 'M01,P02,IP-002,sts,rotation,complete'
```

## Decision-log (Linus good-taste row)
Rotation is a narrow re-issue path that preserves tenant, subject, purpose, and scope — eliminates the 'rotated into broader authority' failure class structurally.
