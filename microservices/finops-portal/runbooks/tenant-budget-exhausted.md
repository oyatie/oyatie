---
runbook_id: finops-portal/tenant-budget-exhausted
authored: 2026-05-18
status: ready
oncall: customer-success + ops-finops
adr_authority: ADR-0199
alert: TenantBudgetExhausted (period spend ≥ budget)
severity: SEV-2 / reactive
---

# Runbook — Tenant budget exhausted

## When this fires

`TenantBudgetExhausted` Prometheus alert fires when current-period
tenant spend ≥ budget. This is a **reactive** alert; the tenant has
crossed their pre-agreed financial ceiling.

The platform does NOT automatically throttle on this alert. Throttle
decisions are tenant-contract-specific and require human review.

## First five minutes

1. **Acknowledge** the alert; this is SEV-2 — page customer-success
   on-call.
2. **Pull** the tenant's contract from the tenancy µservice via the
   contract-view endpoint.
3. **Check** contract clause: hard-cap vs soft-cap on budget
   exceedance.
4. **Open** the drill-down; identify what is driving exceedance.

## Response paths

### Path A — Soft-cap (warn-only)

1. Notify the tenant via the channel specified in the contract
   (email/Slack/PagerDuty).
2. Apply a one-time credit if the exceedance is platform-fault
   (e.g. backup retention bump that wasn't communicated).
3. Document.

### Path B — Hard-cap with throttle clause

1. **Confirm** with customer-success the customer-facing message.
2. **Trigger** the throttle workflow in workflow-studio:
   the tenant's foundry-eval invocations switch to a degraded tier
   (slower; cheaper).
3. **Notify** the tenant immediately; provide path to unblock
   (raise budget, add committed-use commitment, etc.).
4. **Apply** the throttle within the regulatory-pack rules: KR
   pack requires 24 h tenant-notice before throttle for in-progress
   workflow runs; throttle applies only to new runs in that window.

### Path C — Hard-cap with billing-suspend clause

1. This is the last-resort path; rare.
2. Escalate to ops-finops manager + customer-success VP.
3. Coordinate with legal on tenant-facing communication.
4. Suspend new authentications via the auth-portal µservice
   feature flag.

## Escalation

- If tenant is mission-critical (per tenancy.criticality_class
  metadata) AND budget exhausted: page ops-finops VP regardless of
  contract path.
- If exceedance is > 50 % above budget: this is a control gap;
  open an incident in incident-management µservice to investigate
  the controls failure.

## Post-incident

- Issue a post-mortem within 5 business days.
- Update the tenant's budget OR add committed-use commitment OR
  contract amendment.

## Evidence

- Audit-chain seal class `TenantBudgetExhaustedAction` records the
  path taken (A / B / C) + decision rationale.

## References

- ADR-0199 — FinOps canonical.
- `runbooks/tenant-budget-headroom-low.md` (preceding alert).
- `runbooks/credit-application-reconciliation.md`.
