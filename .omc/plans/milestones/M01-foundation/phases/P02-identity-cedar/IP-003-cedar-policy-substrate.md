---
purpose: Ship the Cedar RBAC/ABAC substrate that every capability invocation enforces against.
---

---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P02-IP-003
title: Cedar policy substrate (`oya-policy-cedar-domain` + `oya-platform-policy-cedar-api`) + publish + supersession
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Ship the Cedar RBAC/ABAC substrate that every capability invocation enforces against.
evidence_ref: ../../../../../evidence/foundation/m01-p02-ip-003-cedar-policy-substrate.json
---

# M01-P02-IP-003 — Cedar policy substrate + publish + supersession

## Purpose
Ship the Cedar-shaped RBAC/ABAC substrate that every capability invocation enforces against.

## Symbols-to-grit-claim
```
crates/oya-policy-cedar-domain/src/lib.rs::PolicyVersion
crates/oya-policy-cedar-domain/src/lib.rs::PolicySet
crates/oya-policy-cedar-domain/src/lib.rs::PublishedPolicy
crates/oya-platform-policy-cedar-api/src/lib.rs::publish_cedar_policy_from_api
crates/oya-platform-policy-cedar-api/tests/cedar_policy_publish_api.rs
```
(Scaffold-claim used per ADR-0054 after `grit claim` returned the known FK failure.)

## Agent-prerequisites
M01-P02-IP-001 identity kernel complete.

## Acceptance-test-commands
```
rustfmt --check crates/oya-policy-cedar-domain/src/lib.rs crates/oya-platform-policy-cedar-api/src/lib.rs crates/oya-platform-policy-cedar-api/tests/cedar_policy_publish_api.rs
cargo test -p oya-policy-cedar-domain --all-features
cargo test -p oya-platform-policy-cedar-api --all-features
cargo clippy -p oya-policy-cedar-domain --all-features --all-targets -- -D warnings
cargo clippy -p oya-platform-policy-cedar-api --all-features --all-targets -- -D warnings
```

## Done-criteria
- All acceptance-test commands return 0.
- `cedar.policy.publish` is versioned, semver-validated, idempotent, and supports tenant/global scopes.
- Supersession requires an existing older same-scope version and exposes a resolvable supersession chain.
- Authorization evaluates active unsuperseded policy versions only.
- No provider-specific deps outside adapter crates (per MASTERPLAN §2 Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Evidence recorded at `/evidence/foundation/m01-p02-ip-003-cedar-policy-substrate.json`.

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
