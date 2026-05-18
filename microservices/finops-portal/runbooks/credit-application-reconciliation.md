---
runbook_id: finops-portal/credit-application-reconciliation
authored: 2026-05-18
status: ready
oncall: ops-finops
adr_authority: ADR-0162
alert: CreditApplicationSealMiss (audit-chain seal failed within 30s)
severity: SEV-2
---

# Runbook — Credit-application reconciliation

## When this fires

`CreditApplicationSealMiss` fires when a `CreditApplied` ledger
entry was appended in-process but the corresponding audit-chain
seal did not land within 30 s. The SLO is
`slos/credit-application-correctness.openslo.yaml`.

This is a SEV-2 because the append-only ledger invariant is
intact in our store, but the audit trail is stale until the seal
catches up. Until reconciled, the quarterly emit will report a
delta between ledger state and sealed state.

## First five minutes

1. **Acknowledge** the alert.
2. **Confirm** the ledger entry exists in postgres:
   `SELECT id, tenant_id, amount_cents, issued_at FROM
   credit_ledger WHERE id = '<id>';`.
3. **Confirm** the audit-chain has NO seal for that id:
   `oya audit-chain query --class CreditApplied --id <id>`.

## Response paths

### Path A — Audit-chain transient failure

1. Audit-chain was briefly unreachable during the append.
2. Trigger the reconciler:
   `oya finops-portal reconcile-credit-seals --since <ts>`.
3. The reconciler re-emits seals for any un-sealed entries;
   audit-chain dedups so this is safe to re-run.

### Path B — Sealer key rotation in flight

1. The audit-chain seal key was rotating at the moment of append.
2. Wait for rotation to complete (≤ 5 min); re-run reconciler.

### Path C — Schema mismatch

1. The audit-chain rejected the seal envelope (schema validation
   failure).
2. Inspect the rejection log:
   `oya audit-chain query --class RejectedSeal --id <id>`.
3. Fix the envelope shape in the usecase layer; re-deploy;
   re-run reconciler.

### Path D — Tenant data corruption

1. The ledger entry references a tenant id that does not exist
   in the tenancy µservice.
2. This is a data-integrity event; **DO NOT** force the seal.
3. Open an incident in incident-management; freeze tenant
   ledger writes for that tenant id; investigate.

## Escalation

- If un-sealed entries > 100: page ops-finops manager.
- If quarterly emit is within 7 days: SEV-1 because the emit
  will report a mismatch.

## Verification after reconcile

```sh
oya finops-portal reconcile-credit-seals --since '<ts>' --dry-run
# should report 0 un-sealed entries
```

## Evidence

- Audit-chain class `CreditSealReconciliation` records each
  reconcile run with the count of seals re-emitted.

## References

- ADR-0162 — per-tenant audit-log slicing.
- ADR-0174 — chargeback formula.
- `slos/credit-application-correctness.openslo.yaml`.
- IP-013 (credit-ledger kernel).
