---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P03-IP-003
title: Tamper-evidence verification drill runbook
status: complete
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Bind the Sev-1 tamper-evidence drill to the live audit-chain verification surfaces.
---

# M01-P03-IP-003 — Tamper-evidence verification drill runbook

## Purpose
Bind the Sev-1 tamper-evidence drill to the live audit-chain verification surfaces.

## Symbols-to-grit-claim
```
docs/runbooks/cross-axis/audit-chain-integrity-failure.md::Verify-recovery
crates/oya-audit-chain-domain/src/lib.rs::verify_chain
crates/oya-audit-chain-domain/tests/merkle_chain.rs::merkle_root_advances_with_each_append_and_detects_payload_tamper
crates/oya-audit-chain-file-adapter/tests/file_ledger.rs::file_audit_ledger_rejects_divergent_history_and_tampered_records
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
IP-001 + IP-002 merged.

## Acceptance-test-commands
```
cargo test -p oya-audit-chain-domain --test merkle_chain merkle_root_advances_with_each_append_and_detects_payload_tamper -- --exact
cargo test -p oya-audit-chain-file-adapter --test file_ledger file_audit_ledger_rejects_divergent_history_and_tampered_records -- --exact
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
M01-P04-IP-001 (outbox + topic registry)

## Icm-store-payload
```
icm store -t context-oyatie -c 'audit-chain tamper-drill green; detection within one verification cycle; M01-P03 acceptance gate ready' -i critical -k 'M01,P03,IP-003,tamper-drill,complete'
```

## Decision-log (Linus good-taste row)
Tamper detection is a single verify_chain pass — eliminates per-store integrity-check special cases.

## Completion-evidence
- Sev-1 audit-chain integrity runbook is active and names the live one-cycle `verify_chain`/file-ledger replay drill.
- Domain tamper drill passes: `cargo test -p oya-audit-chain-domain --test merkle_chain merkle_root_advances_with_each_append_and_detects_payload_tamper -- --exact`.
- File-ledger tamper drill passes: `cargo test -p oya-audit-chain-file-adapter --test file_ledger file_audit_ledger_rejects_divergent_history_and_tampered_records -- --exact`.
- M01-P03 phase acceptance is complete: stable audit event SPEC/contract, AsyncAPI/Proto source, and tamper-evidence drill evidence.
