---
doc_class: Runbook
title: Per Tenant Cost Attribution Mismatch
status: Accepted
date: 2026-05-20
microservice: cloud-billing
severity: sev1
audience: sre, finops-engineer, billing-operator
owner_team: axis-cloud + finance-operations + ops-sre-reliability
doc_status: published
---

# Runbook: Per Tenant Cost Attribution Mismatch

## Operator Contract
- Runbook id: cloud-billing-per-tenant-cost-attribution-mismatch.
- Primary namespace: `cloud-billing`.
- Owning rotation: PagerDuty `cloud-billing-primary`.
- Finance secondary: PagerDuty `finance-operations-primary`.
- Incident channel: `#inc-cloud-billing`.
- Customer channel: `#support-billing-attribution`.
- Protected surface: metering bus, FOCUS 1.1 normalization, tenant tags, cost centers, rate cards, invoice line items.
- Source schema: `cloud_billing.metering.v1`.
- External sources: AWS CUR, GCP Billing Export, Azure Cost Management, Oracle usage export.
- Safety invariant: do not mutate ledger rows in place.
- Ledger invariant: corrections must be credit memo, debit memo, or replay with idempotency proof.
- Stop condition: mismatch cohort is identified, ledger correction is staged, FOCUS export reconciles, and invoice impact is documented.
- Evidence event: `EVT_CLOUD_BILLING_ATTRIBUTION_MISMATCH_INCIDENT`.
- Handoff API: `https://cloud-billing.internal.oyatie.dev/v1/attribution/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/attribution?orgId=1&var-cell=prod-us-east-1`.
- FOCUS dashboard: `https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/focus-export?orgId=1&var-period=current`.
- Loki query: `{namespace="cloud-billing",runbook="per-tenant-cost-attribution-mismatch"}`.
- Canonical FAQ: `microservices/cloud-billing/faqs/billing-engineer-faq.md`.
- Related ADR: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
- Related bus: `cloud_billing.metering.v1`.

## Trigger Conditions
- Alert `CloudBillingTenantAttributionMismatchCritical` fires.
- Alert `CloudBillingUntaggedVendorCostSpike` fires.
- Alert `CloudBillingFocusExportReconciliationFailed` fires.
- Alert `CloudBillingLedgerReplayDedupeFailure` fires.
- Metric `cloud_billing_unattributed_cost_ratio` exceeds 0.005.
- Metric `cloud_billing_tenant_cost_delta_usd` exceeds tenant threshold.
- Metric `cloud_billing_vendor_line_unmapped_total` increases by more than 1000 in 15 minutes.
- Metric `cloud_billing_metering_event_signature_invalid_total` is non-zero.
- Metric `cloud_billing_focus_reconciliation_error_total` increases.
- Metric `cloud_billing_cost_center_missing_total` spikes.
- Metric `cloud_billing_shared_overhead_amortization_delta_usd` exceeds baseline.
- Metric `cloud_billing_ledger_replay_duplicate_total` is non-zero.
- Tenant reports showback/chargeback value differs from expected cloud spend.
- Finance reports FOCUS export mismatch with ERP import.
- AWS CUR ingest completes but tenant tag coverage drops.
- GCP Billing Export ingest reports resource without `oyatie.tenant_id`.
- Azure Cost Management ingest maps costs to wrong subscription.
- Rate card change occurs inside the affected usage period.
- Tenancy merge or split changes tenant tree during billing period.
- Audit-chain lacks `cloud_billing.attribution.applied` for a replay batch.

## Symptoms
- One tenant receives costs from another tenant.
- Costs are assigned to `oyatie.platform.shared-overhead` unexpectedly.
- Showback dashboard and invoice preview disagree.
- FOCUS export has rows with missing `SubAccountId`.
- ERP chargeback import rejects cost center fields.
- Vendor pass-through line item has no tenant mapping.
- Metering event has valid tenant but missing cost center dimension.
- Metering event signature fails HMAC verification.
- Deduplication by `event_id` admits duplicate charges.
- Resource tags show new tenant id while ledger shows old tenant id.
- Tenant tree shows parent-child change inside usage window.
- Reservation benefit allocation is applied to wrong tenant.
- FX lock timestamp differs between invoice and attribution export.
- `cloud_billing.attribution.mismatch` appears in worker logs.
- `focus_reconciliation_status=failed` appears for one period.
- `ledger_replay_status=partial` appears after mitigation.
- The dashboard shows source-specific skew, usually AWS, GCP, or Azure.
- Customer impact is financial and reputational even when service is available.
- SOX controls are implicated when invoice period is closed.
- Severity rises to Sev0 if cross-tenant financial data is exposed.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-billing-attribution-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export PERIOD=2026-05`.
3. Acknowledge page: `pd incident ack --service cloud-billing --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-cloud-billing --severity sev1`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.service=="cloud-billing")'`.
6. Query unattributed cost: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_billing_unattributed_cost_ratio{period="'$PERIOD'"}'`.
7. Query tenant delta: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_billing_tenant_cost_delta_usd{tenant_id="'$TENANT'",period="'$PERIOD'"}'`.
8. Query unmapped vendor lines: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(cloud_billing_vendor_line_unmapped_total[15m])'`.
9. Query invalid signatures: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(cloud_billing_metering_event_signature_invalid_total[5m])'`.
10. Query FOCUS errors: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(cloud_billing_focus_reconciliation_error_total[5m])'`.
11. Open attribution dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/attribution?orgId=1&var-period=$PERIOD&var-tenant=$TENANT"`.
12. Open FOCUS dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/focus-export?orgId=1&var-period=$PERIOD&var-tenant=$TENANT"`.
13. Read attribution logs: `kubectl -n cloud-billing logs deploy/cloud-billing-attribution-worker --since=60m | rg "attribution|focus|tenant_id|cost_center|replay"`.
14. Check rollout: `kubectl -n cloud-billing rollout status deploy/cloud-billing-attribution-worker --timeout=60s`.
15. Inspect tenant cost summary: `oya billing attribution summary --tenant $TENANT --period $PERIOD --output yaml`.
16. Inspect source split: `oya billing attribution source-split --tenant $TENANT --period $PERIOD --output table`.
17. Inspect unmatched vendor rows: `oya billing vendor unmatched --period $PERIOD --tenant $TENANT --limit 100 --output json`.
18. Inspect tenant tags: `oya billing tags audit --tenant $TENANT --period $PERIOD --source all --output json`.
19. Inspect tenant tree: `oya tenancy tree history --tenant $TENANT --period $PERIOD --output yaml`.
20. Inspect cost centers: `oya billing cost-center audit --tenant $TENANT --period $PERIOD --output table`.
21. Inspect metering event sample: `oya billing metering sample --tenant $TENANT --period $PERIOD --limit 50 --output json`.
22. Verify event signatures: `oya billing metering verify-signatures --tenant $TENANT --period $PERIOD --output json`.
23. Check dedupe state: `oya billing metering dedupe-status --tenant $TENANT --period $PERIOD --output json`.
24. Check rate card: `oya billing rate-card get --tenant $TENANT --period $PERIOD --output yaml`.
25. Check reservation allocation: `oya billing reservation allocation --tenant $TENANT --period $PERIOD --output table`.
26. Check FOCUS export: `oya billing focus reconcile --tenant $TENANT --period $PERIOD --dry-run --output json`.
27. Check invoice preview: `oya billing invoice preview --tenant $TENANT --period $PERIOD --output json`.
28. Check ERP import: `oya billing erp validate --tenant $TENANT --period $PERIOD --dry-run --output json`.
29. Query attribution audit: `oya audit-chain query --event-class cloud_billing.attribution.applied --tenant $TENANT --since 30d`.
30. Query correction audit: `oya audit-chain query --event-class cloud_billing.ledger.correction --tenant $TENANT --since 30d`.
31. Check closed period: `oya billing period status --tenant $TENANT --period $PERIOD --output json`.
32. Check support cases: `oya support cases list --tag cloud-billing.attribution --tenant $TENANT --since 7d`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-billing --runbook per-tenant-cost-attribution-mismatch --output evidence/incidents/$INCIDENT_ID.json`.
34. Export mismatch rows: `oya billing attribution mismatch export --tenant $TENANT --period $PERIOD --output evidence/incidents/$INCIDENT_ID-mismatch.json`.
35. Export FOCUS rows: `oya billing focus export --tenant $TENANT --period $PERIOD --output evidence/incidents/$INCIDENT_ID-focus.parquet`.

### Diagnostic Decision Tree
```text
1. Is another tenant's cost assigned to this tenant?
   |-- yes: treat as cross-tenant financial data incident and page security.
   |-- no: continue financial reconciliation.
2. Is the mismatch from vendor tag ingestion?
   |-- yes: repair tag mapping and replay vendor rows.
   |-- no: inspect metering bus and tenant tree.
3. Is the mismatch from metering event signature or dedupe failure?
   |-- yes: stop affected emitter and replay from trusted ledger.
   |-- no: inspect rate card, reservation, and cost center projection.
4. Is the invoice period already closed?
   |-- yes: use credit memo or debit memo, never direct ledger update.
   |-- no: replay attribution batch with idempotency proof.
5. Does FOCUS export reconcile after dry-run correction?
   |-- yes: stage correction and notify finance.
   |-- no: keep incident open and escalate to finance operations.
```

## Mitigation
1. Hold invoices for affected tenant: `oya billing invoice hold --tenant $TENANT --period $PERIOD --reason $INCIDENT_ID`.
2. Hold FOCUS export: `oya billing focus export hold --tenant $TENANT --period $PERIOD --reason $INCIDENT_ID`.
3. Hold attribution deploys: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
4. Stop suspect emitter: `oya billing metering emitter pause --tenant $TENANT --source <source> --reason $INCIDENT_ID`.
5. Freeze ledger snapshot: `oya billing ledger snapshot --tenant $TENANT --period $PERIOD --output evidence/incidents/$INCIDENT_ID-ledger.json`.
6. Stage replay dry-run: `oya billing attribution replay --tenant $TENANT --period $PERIOD --from trusted-source --dry-run`.
7. Stage correction dry-run: `oya billing ledger correction plan --tenant $TENANT --period $PERIOD --reason $INCIDENT_ID --dry-run`.
8. Apply open-period replay: `oya billing attribution replay --tenant $TENANT --period $PERIOD --from trusted-source --confirm $INCIDENT_ID`.
9. Apply closed-period credit memo: `oya billing credit-memo issue --tenant $TENANT --period $PERIOD --reason attribution-error --confirm $INCIDENT_ID`.
10. Apply closed-period debit memo only with finance approval: `oya billing debit-memo issue --tenant $TENANT --period $PERIOD --reason attribution-error --confirm $INCIDENT_ID`.
11. Refresh tenant tags: `oya billing tags reconcile --tenant $TENANT --period $PERIOD --confirm $INCIDENT_ID`.
12. Refresh tenant tree projection: `oya billing tenant-tree sync --tenant $TENANT --period $PERIOD --confirm $INCIDENT_ID`.
13. Regenerate FOCUS export: `oya billing focus export --tenant $TENANT --period $PERIOD --output evidence/incidents/$INCIDENT_ID-focus-corrected.parquet`.
14. Notify finance: `oya notify finance-operations --incident $INCIDENT_ID --category attribution-mismatch`.
15. Notify support: `oya notify support --incident $INCIDENT_ID --template billing-attribution-hold`.
16. Notify tenant admin when invoice hold is visible: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template billing-attribution-review`.
17. Emit mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_BILLING_ATTRIBUTION_MISMATCH_INCIDENT --incident $INCIDENT_ID --field mitigation=invoice-held`.
18. Preserve every correction plan in evidence.
19. Keep direct SQL updates forbidden.
20. Keep customer-facing estimates labelled preliminary until finance signs off.

## Resolution
1. Patch vendor tag mapping if vendor rows were unmapped.
2. Patch metering schema validation if signatures or dimensions were missing.
3. Patch tenant tree projection if merger or split caused wrong attribution.
4. Patch reservation allocation if benefits landed on wrong tenant.
5. Patch FOCUS export if canonical ledger was correct but export was wrong.
6. Patch ERP adapter if downstream import rejected valid fields.
7. Add regression fixture for affected vendor source.
8. Add regression fixture for tenant tree change inside period.
9. Run domain tests: `cargo test -p cloud-billing-domain attribution -- --nocapture`.
10. Run tax app tests if invoice lines changed: `cargo test -p cloud-billing-tax-app cloud_billing_invoice_api -- --nocapture`.
11. Run production gate: `cargo run -p dev-cli -- gate validate cloud-billing-attribution --production-snapshot --period $PERIOD`.
12. Reconcile corrected FOCUS export: `oya billing focus reconcile --tenant $TENANT --period $PERIOD --expect clean`.
13. Release invoice hold: `oya billing invoice unhold --tenant $TENANT --period $PERIOD --reason resolved-$INCIDENT_ID`.
14. Release export hold: `oya billing focus export unhold --tenant $TENANT --period $PERIOD --reason resolved-$INCIDENT_ID`.
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_BILLING_ATTRIBUTION_MISMATCH_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudBillingTenantAttributionMismatchCritical` is green.
- `cloud_billing_unattributed_cost_ratio` is below 0.001.
- `cloud_billing_tenant_cost_delta_usd` is inside tenant threshold.
- FOCUS reconcile returns clean.
- Invoice preview matches corrected ledger.
- ERP validation passes.
- Audit-chain contains attribution correction or replay evidence.
- Finance owner signs the correction plan.
- Support has tenant-facing explanation when invoice changed.
- No direct ledger mutation occurred.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-billing-per-tenant-cost-attribution-mismatch
microservice: cloud-billing
event_class: EVT_CLOUD_BILLING_ATTRIBUTION_MISMATCH_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Per Tenant Cost Attribution Mismatch postmortem

## Summary
- Which tenant, period, source, and ledger rows mismatched.
- Whether cross-tenant financial data was exposed.
- Whether invoice, FOCUS export, or ERP import changed.

## Timeline
- Mismatch detected:
- Invoice held:
- Correction staged:
- Finance approved:
- Invoice released:

## Financial Impact
- Original amount:
- Corrected amount:
- Credit or debit memo:
- SLA credit:

## Root Cause
- Vendor tags:
- Metering event:
- Tenant tree:
- Reservation:
- Export:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Finance control:
```

## Escalation Path
- Page `cloud-billing-primary` for all attribution mismatches.
- Page `finance-operations-primary` when invoice or ERP output is affected.
- Page `security-policy-primary` when another tenant's cost appears.
- Page `audit-chain-primary` when correction events fail to seal.
- Notify `#inc-cloud-billing` with tenant, period, and source.
- Notify `#support-billing-attribution` before customer communication.
- Notify `#sox-controls` when a closed invoice period changes.
- Escalate to executive incident commander for material revenue impact.
- Engage cloud provider support only after vendor source is confirmed.
- Keep finance approval explicit before any closed-period memo.

## Cross-µservice Coordination
- `tenancy`: verify tenant tree, parent-child scope, and merger history.
- `cloud-iam`: verify emitter principal and Cedar permission for metering.
- `audit-chain`: seal attribution, replay, correction, and memo events.
- `cloud-kms`: verify metering event HMAC keys if signature failures appear.
- `cloud-network`: verify resource tags for network resources.
- `cloud-compute`: verify resource tags for compute resources.
- `cloud-storage`: verify resource tags for storage resources.
- `comms-email`: send invoice-hold and correction notices.
- `support`: manage tenant cases and billing explanations.
- `finance-operations`: approve correction and ERP handling.
- `compliance`: review SOX and revenue recognition impact.
- `observability`: attach attribution and FOCUS dashboards.

## Runbook Maintenance
- Add new vendor source failure signatures after every incident.
- Keep FOCUS commands aligned with schema version.
- Keep correction paths memo-based for closed periods.
- Add every new cost source to cross-service coordination.
- Review this runbook before month-end close.
