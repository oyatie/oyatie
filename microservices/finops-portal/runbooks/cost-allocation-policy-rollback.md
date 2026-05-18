---
runbook_id: finops-portal/cost-allocation-policy-rollback
authored: 2026-05-18
status: ready
oncall: ops-finops
adr_authority: ADR-0174
alert: CostAllocationPolicyChangedAlert (anomalous fleet-wide cost shift after policy change)
severity: SEV-2
---

# Runbook — Cost-allocation policy rollback

## When this fires

`CostAllocationPolicyChangedAlert` fires when:

- A new `CostAllocationPolicy` was promoted to `active`, AND
- Within 24 h of promotion, fleet-wide tenant cost distribution
  shifted by > 10 % at the cost-center level.

This indicates either an intentional rebalancing OR an unintended
mis-allocation that requires rollback.

## First five minutes

1. **Acknowledge**.
2. **Confirm** the policy that changed:
   `oya finops-portal policy list --status active --since 24h`.
3. **Diff** against the prior version via the policy editor's
   diff view or:
   `oya finops-portal policy diff --policy <name> --from v<n-1> --to v<n>`.

## Response paths

### Path A — Intentional rebalancing

1. Confirm with the policy author (the audit-chain seal records
   `authored_by`).
2. Document in the policy's notes; no rollback.

### Path B — Unintended mis-allocation (small blast radius)

1. The change affected one cost-center; the impact is bounded.
2. Promote a new patch policy that **reverses** the affected
   rule.
3. Per IP-010 lifecycle, the patch goes through draft → review
   (2x) → active; in emergency, use the `emergency-promote` path
   that requires the ops-finops manager's signature.

### Path C — Unintended mis-allocation (fleet-wide impact)

1. **Pause** the affected policy by promoting it to `retired`:
   `oya finops-portal policy retire --policy <name> --reason "<text>"`.
2. The retire causes `EffectivePolicySet` to fall back to the
   prior active policy.
3. Communicate fleet-wide impact to affected tenants within 24 h.
4. Issue platform-fault credits for the misallocated period via
   the credit-ledger.

### Path D — Reviewer-quorum bypass

1. Investigate whether the policy was promoted with the proper
   quorum (per IP-010 domain rules: 2 reviewers required for
   Fleet scope).
2. If quorum was bypassed: this is a controls failure; open
   incident; suspend the offending reviewer account; full
   post-mortem.

## Promotion of a rollback patch

The standard lifecycle path:

```
new draft → reviewed (2 reviewers) → active (calendar-month next)
```

The emergency path (only invoked at SEV-2):

```
emergency-promote → active immediately → reviewed within 48h
  → if quorum miss: revert to prior policy
```

## Escalation

- Any rollback affecting > 10 tenants: page ops-finops VP +
  customer-success VP.
- Reviewer-quorum bypass: page security + compliance leads.

## Evidence

- Audit-chain class `CostAllocationPolicyRollback` records the
  retire event + the reason text.

## References

- ADR-0174 — chargeback formula.
- IP-009 / IP-010 — cost-allocation-policy BC.
- `slos/cost-allocation-policy-change-latency.openslo.yaml`.
