---
runbook_id: finops-portal/tenant-budget-headroom-low
authored: 2026-05-18
status: ready
oncall: customer-success
adr_authority: ADR-0199
alert: TenantBudgetHeadroomLow (headroom < 20%, >7d projection)
severity: SEV-3 / proactive
---

# Runbook — Tenant budget headroom low

## When this fires

`TenantBudgetHeadroomLow` Prometheus alert fires when:

- A tenant's current-period spend / budget ratio > 80 %, AND
- The 7-day extrapolated spend would exceed the budget.

This is a **proactive** alert; the tenant has not yet exhausted
the budget. The intent is to intervene before exhaustion turns
into a service-level event.

## First five minutes

1. **Acknowledge** in PagerDuty / Opsgenie; SLA is 1 business day,
   not minutes.
2. **Open** the tenant's drill-down in `finops-portal`.
3. **Confirm** the headroom calculation against the dashboard
   "headroom %" column.
4. **Identify** the growth driver:
   - Recent capability invocation surge?
   - New workload-class on a more expensive tier (gpu)?
   - Storage growth (backups, exports)?

## Response paths

### Path A — Predictable growth (e.g. quarter-end batch)

1. Confirm with tenant via customer-success channel that the
   growth is intentional.
2. Offer a one-period budget bump via the credit-ledger (negative
   credit equivalent OR raise of budget via tenancy µservice).
3. Document the conversation in the tenant's customer-success
   notebook; link to the alert.

### Path B — Unexpected growth (anomaly)

1. Cross-check with `TenantCostAnomalySpike` runbook — is this
   the same event?
2. If yes: investigate per that runbook; this alert is the
   trailing indicator.
3. If no: open a tenant-facing conversation; understand intent.

### Path C — Budget too tight (chronic)

1. The tenant's budget was set conservatively in onboarding.
2. Recommend a budget review with the tenant; raise to a
   sustainable level.

## Escalation

- If headroom drops < 5 % AND no conversation in last 24 h: page
  on-call customer-success.
- If tenant budget exhausts before raise / credit applied:
  fall through to `runbooks/tenant-budget-exhausted.md`.

## Evidence

- Audit-chain class `TenantBudgetHeadroomLowAcknowledged` is
  sealed with the action taken (path A / B / C).

## References

- ADR-0199 — FinOps cost-attribution canonical.
- `slos/tenant-invoice-render-latency.openslo.yaml` (not the SLO
  for this alert; reference for the broader BC).
- `runbooks/tenant-budget-exhausted.md` (next-stage runbook).
