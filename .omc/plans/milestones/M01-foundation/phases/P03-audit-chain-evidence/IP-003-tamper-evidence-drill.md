---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P03-IP-003
title: Tamper-evidence Sev-1 drill runbook
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Author and execute the Sev-1 tamper-evidence drill runbook.
---

# M01-P03-IP-003 — Tamper-evidence Sev-1 drill runbook

## Purpose
Author and execute the Sev-1 tamper-evidence drill runbook.

## Symbols-to-grit-claim
```
docs/runbooks/audit-chain-tamper-drill.md::Procedure
crates/oya-platform-audit-chain-worker/src/lib.rs::detect_tampering
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
IP-001 + IP-002 merged.

## Acceptance-test-commands
```
bash docs/runbooks/audit-chain-tamper-drill.sh
cargo test -p oya-platform-audit-chain-worker --test detect_tampering
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
