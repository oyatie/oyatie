---
runbook_id: finops-portal/tenant-cost-anomaly-spike
authored: 2026-05-18
status: seed
oncall: ops-finops
adr_authority: ADR-0199
slo_breach: tenant-cost-anomaly threshold (1.5x rolling 7d)
---

# Runbook — Tenant cost anomaly spike

## When this fires

`TenantCostAnomalySpike` Prometheus alert: per-tenant cost > 1.5× rolling
7-day average over a 1 h window. Severity warning. Routes to
ops-finops.

## First five minutes

1. **Acknowledge** the alert in PagerDuty / Opsgenie.
2. **Open** the tenant's cost drill-down in `finops-portal` (or
   OpenCost UI today during Phase 0).
3. **Filter** by the 1 h window centered on the alert fire time.
4. **Identify** the contributing dimension:
   - Did one cost-center spike? (capability invocations? storage growth?
     backup retention churn?)
   - Did one workload-class spike? (gpu? batch?)
   - Did one cell spike?
5. **Check** if a deploy in the last 1 h could explain it (correlate with
   ArgoCD / argo-rollouts deploy timeline).

## Likely root causes (per past incidents)

| Cause                                     | Confirm via                                | Action                              |
|-------------------------------------------|--------------------------------------------|-------------------------------------|
| Tenant ran a large foundry batch job      | foundry-eval invocation count metric       | confirm with tenant; usually no-op  |
| GPU workload ran longer than expected     | karpenter_gpu_node_active_seconds          | confirm; check budget headroom      |
| Backup retention bump just landed         | velero_backup_size_bytes by tenant         | expected; document                  |
| Cost-allocation policy changed            | audit-chain class=CostAllocationPolicyChanged | review change; revert if accidental |
| OpenCost custom-pricing config changed    | configmap diff in last 1 h                 | confirm intended                    |

## Escalation

- If spend > $5,000 / hr extrapolated AND tenant budget headroom < 10 %:
  page on-call ops-finops manager.
- If anomaly persists > 4 h: open incident in incident-management µservice;
  open tenant-facing comms via the customer-success rotation.

## Evidence

- Audit-chain class `TenantCostAnomalyInvestigation` is sealed with the
  investigation outcome.
- The audit event references the original alert and links to the
  drill-down dashboard URL.

## References

- ADR-0199 — FinOps cost-attribution canonical.
- ADR-0174 — chargeback formula + anomaly thresholds.
- `docs/standards/finops-cost-attribution-canonical.md`.
