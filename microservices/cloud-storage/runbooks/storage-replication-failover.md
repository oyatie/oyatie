# cloud-storage DR Failover Runbook

## Purpose
Restore the cloud-storage service to the manifest-declared RTO/RPO without changing tenant isolation, compliance-pack placement, or audit-chain evidence.

## Doctrine
- ADR-0343 drives the RTO/RPO declaration in manifest.json.
- ADR-0338 and ADR-0340 keep runtime tier and cell placement unchanged during failover.
- ADR-0345 requires patched upstream pins before a failed region is reintroduced.

## Trigger
Use this runbook when the active cell or region cannot satisfy the p99 SLO and the manifest DR block points to this file.

## Steps
1. Freeze writes or route them to the healthy active region according to the service controller.
2. Verify the latest audit-chain seal and backup substrate listed in manifest.json.
3. Promote the warm region/cell and keep tenant_id, compliance_pack, and deployment_context bindings unchanged.
4. Rehydrate Valkey/Postgres/object or specialized substrate state from the manifest-declared backup list.
5. Run the service smoke check and record the evidence id in audit-chain.
6. Keep the failed region read-only until drift and CVE/pin checks pass.

## Exit Criteria
- RTO <= 3600s and RPO <= 300s are met or an incident exception is opened.
- Manifest runtime tier 1 and cell placement Tier-1 remain unchanged.
- A follow-up restore drill records the next last_drill_evidence_id.
