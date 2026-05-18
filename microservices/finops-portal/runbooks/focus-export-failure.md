---
runbook_id: finops-portal/focus-export-failure
authored: 2026-05-18
status: ready
oncall: ops-finops
adr_authority: ADR-0199
alert: FocusExportFailureRate (>1% over 10m window)
severity: SEV-3
---

# Runbook — FOCUS export failure

## When this fires

`FocusExportFailureRate` Prometheus alert fires when the
`finops_portal_focus_export_total{outcome!="success"}` ratio
exceeds 1 % over a 10-minute window. The corresponding SLO is
`slos/focus-export-availability.openslo.yaml`.

## First five minutes

1. **Acknowledge** the alert.
2. **Open** the recent failure samples via the
   `anomaly-investigation` dashboard.
3. **Classify** the failure mode:
   - `TranslateError::UnknownCostCenter` — translator can't map a
     cost-center to FOCUS column.
   - `WriteError::BucketUnavailable` — SeaweedFS down.
   - `IssueError::SignerKeyExpired` — signed-URL HMAC key expired.
   - HTTP 5xx with no specific class — upstream Mimir.

## Response paths

### Path A — Unknown cost-center

1. The translator hit a cost-center that has no mapping. Either:
   - A new cost-center was deployed without updating the mapping.
   - A typo in the cost-center label.
2. Fix: add the mapping in
   `crates/oya-finops-portal-focus-export-kernel/src/schema.rs`
   and ship a patch.
3. Verify via re-run.

### Path B — SeaweedFS bucket unavailable

1. Cross-check with cloud-iac SeaweedFS dashboard.
2. If SeaweedFS is fully down: defer the export; the export
   re-tries with exponential backoff for up to 24 h.
3. If a specific bucket is mis-configured: check the bucket
   provisioning runbook in cloud-iac.

### Path C — Signer key expired

1. The HMAC key for signed-URL issuance has expired.
2. Rotate the key via the secrets µservice rotation endpoint.
3. Re-deploy `finops-portal` to pick up the rotated key.

### Path D — Upstream Mimir 5xx

1. Cross-check Mimir dashboard.
2. If Mimir is degraded: pause export pipeline (via feature flag);
   notify ops-finops of the pause.
3. Resume when Mimir recovers.

## Escalation

- If failure rate sustains > 5 % for > 30 min: escalate to SEV-2;
  page ops-finops manager.
- If the SLO error budget is exhausted: trigger the SLO-gated
  promotion freeze per ADR-0130 (no new deploys until budget
  recovers).

## Tenant communication

- For tenant-impacting failures (path A/D): post status to the
  tenant status page.
- For platform-fault failures lasting > 1 h: issue a one-time
  credit per the SLA addendum.

## Evidence

- Audit-chain class `FocusExportFailureInvestigation` records the
  path + remediation.

## References

- ADR-0199 — FinOps + FOCUS canonical.
- `slos/focus-export-availability.openslo.yaml`.
- `runbooks/finops-portal-deploy-rollback.md` (if a recent deploy
  introduced the failure).
