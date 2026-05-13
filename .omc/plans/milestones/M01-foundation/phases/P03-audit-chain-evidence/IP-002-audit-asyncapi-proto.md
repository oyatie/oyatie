---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M01-P03-IP-002
title: Audit event AsyncAPI + Proto contract
status: stub
final_shape_compliance: true
dependency_additions: []
purpose: Publish AsyncAPI + Proto source contracts for audit events.
---

# M01-P03-IP-002 — Audit event AsyncAPI + Proto contract

## Purpose
Publish AsyncAPI + Proto source contracts for audit events.

## Symbols-to-grit-claim
```
contracts/asyncapi/platform/audit-events-v1.yaml::AuditEventEnvelope
contracts/proto/platform/audit/v1/audit-event-v1.proto::AuditEvent
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
IP-001 merged.

## Acceptance-test-commands
```
node scripts/asyncapi-lint.mjs contracts/asyncapi/platform/audit-events-v1.yaml
node scripts/proto-lint.mjs contracts/proto/platform/audit/v1/
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
M01-P03-IP-003 (tamper drill)

## Icm-store-payload
```
icm store -t context-oyatie -c 'audit AsyncAPI + Proto contracts published + linted' -i critical -k 'M01,P03,IP-002,audit-contracts'
```

## Decision-log (Linus good-taste row)
Single source contract for both REST and streaming consumers — eliminates 'two diverged audit schemas' failure class.
