---
runbook_id: finops-portal/quarterly-regulator-emit-miss
authored: 2026-05-18
status: ready
oncall: ops-finops + compliance
adr_authority: ADR-0162
alert: QuarterlyRegulatorEmitMiss (emit slipped > 5d past quarter-close)
severity: SEV-2
---

# Runbook — Quarterly regulator-emit miss

## When this fires

`QuarterlyRegulatorEmitMiss` fires when the quarterly emit job
has not produced a sealed `FinOpsQuarterlyReport` event on the
audit-chain within `quarter-close + 5 days`. The corresponding SLO
is `slos/regulator-emit-availability.openslo.yaml`.

This is a **compliance event**: regulators in each pack expect the
emit within the cure window. A miss may trigger a regulatory
finding.

## First five minutes

1. **Acknowledge** + page ops-finops manager + compliance lead.
2. **Open** the quarterly-emit job logs in the central log store.
3. **Check** the last successful seal class
   `FinOpsQuarterlyKeyPublished` for the quarter — did the key
   publish succeed?

## Response paths

### Path A — Key publish failed

1. The signing key for the quarter never published.
2. Re-publish via the secrets µservice quarterly-key rotation
   endpoint.
3. Re-trigger the emit job; verify seal lands.

### Path B — Per-tenant invoice missing

1. The job aborted because one tenant's invoice was not yet
   finalized for the last quarter.
2. Identify the tenant; cross-check why finalization stalled.
3. Manually trigger finalization via the
   `POST /v1/tenants/{id}/invoices/{period}/finalize` admin
   endpoint (ops-finops role required).
4. Re-trigger quarterly emit.

### Path C — Audit-chain unavailable

1. Audit-chain endpoint was unreachable during the emit window.
2. Cross-check audit-chain µservice health.
3. Once recovered, re-trigger emit; the audit-chain dedups
   on `quarter` key.

### Path D — Partial emit (sealed but no Parquet copy)

1. The audit-chain seal succeeded but the Parquet copy to
   SeaweedFS failed.
2. Re-run the Parquet-copy phase only (the job is idempotent).

## Escalation

- If miss is unrecoverable within 30 days: file a compliance
  exception ticket with the affected pack's compliance officer.
- If miss is for KR / EU pack (regulator-notice obligations):
  initiate regulator-facing communication within 7 days of
  detection.

## Regulator-facing communication

- KR pack (FSS): use the FSS evidence-portal upload mechanism;
  attach the late-seal envelope + a one-page explanation.
- EU pack (GDPR DPA): the late-seal triggers an Article 33
  notification if the miss involved personal data; coordinate
  with DPO.
- US-healthcare (HIPAA): coordinate with privacy officer;
  determine if breach-notification thresholds apply.

## Post-incident

- Post-mortem within 7 days; root-cause classified into the four
  paths above.
- The post-mortem outcome AMENDS the
  `slos/regulator-emit-availability.openslo.yaml` if a sustained
  pattern emerges.

## Evidence

- Audit-chain class `QuarterlyEmitMissInvestigation` records the
  root-cause + remediation.

## References

- ADR-0162 — per-tenant audit-log slicing + signing.
- ADR-0174 — chargeback formula + quarterly emit cadence.
- `slos/regulator-emit-availability.openslo.yaml`.
- `compliance-matrix.md` for per-pack regulator obligations.
