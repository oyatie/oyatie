---
doc_class: Runbook
title: Invoice Generation Timeout
status: Accepted
date: 2026-05-20
microservice: cloud-billing
severity: sev1
audience: sre, billing-operator, finance-controller
owner_team: axis-cloud + finance-operations + ops-sre-reliability
doc_status: published
---

# Runbook: Invoice Generation Timeout

## Operator Contract
- Runbook id: cloud-billing-invoice-generation-timeout.
- Primary namespace: `cloud-billing`.
- Owning rotation: PagerDuty `cloud-billing-primary`.
- Finance secondary: PagerDuty `finance-operations-primary`.
- Incident channel: `#inc-cloud-billing`.
- Customer channel: `#support-invoice-generation`.
- Protected surface: invoice previews, final invoices, tax handoff, FX lock, credit memos, ERP export, revenue recognition.
- SLO authority: DemoTrial month-end plus 24 hours; Paid 12 hours; Paid 4 hours; Paid 1 hour.
- Safety invariant: never issue partial invoice as final.
- Tax invariant: final invoice must include cloud-billing-tax response for every taxable line.
- Stop condition: invoices are generated or held with owner, tax handoff is reconciled, and ERP export is staged.
- Evidence event: `EVT_CLOUD_BILLING_INVOICE_TIMEOUT_INCIDENT`.
- Handoff API: `https://cloud-billing.internal.oyatie.dev/v1/invoice/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/invoicing?orgId=1&var-period=current`.
- Tax dashboard: `https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/tax-handoff?orgId=1&var-period=current`.
- Loki query: `{namespace="cloud-billing",runbook="invoice-generation-timeout"}`.
- Canonical FAQ: `microservices/cloud-billing/faqs/billing-engineer-faq.md`.
- Related test: `crates/cloud-billing-tax-app/tests/cloud_billing_invoice_api.rs`.
- Related API: `crates/cloud-finops-api/tests/cloud_finops_report_api.rs`.

## Trigger Conditions
- Alert `CloudBillingInvoiceGenerationTimeout` fires.
- Alert `CloudBillingInvoiceSloBurn` fires for any tier.
- Alert `CloudBillingTaxHandoffTimeout` fires.
- Alert `CloudBillingFxLockMissing` fires.
- Alert `CloudBillingErpExportTimeout` fires.
- Metric `cloud_billing_invoice_generation_lag_seconds` exceeds tier SLO.
- Metric `cloud_billing_invoice_generation_timeout_total` increases.
- Metric `cloud_billing_invoice_queue_depth` exceeds 1000.
- Metric `cloud_billing_tax_handoff_error_ratio` exceeds 0.01.
- Metric `cloud_billing_fx_lock_missing_total` is non-zero.
- Metric `cloud_billing_invoice_finalization_retry_total` spikes.
- Metric `cloud_billing_erp_export_lag_seconds` exceeds 3600.
- Month-end close monitor reports invoices missing.
- Finance reports revenue recognition control is blocked.
- Tenant admin reports invoice unavailable.
- Cloud-billing-tax reports outage or high latency.
- FOCUS export reconcile is green but invoice still times out.
- Credit memo application deadlocks a tenant invoice.
- Rate card version is missing for invoice period.
- Audit-chain lacks `cloud_billing.invoice.finalized`.

## Symptoms
- Invoice status remains `generating`.
- Invoice preview exists but final invoice does not.
- Invoice finalization job times out on tax handoff.
- Invoice has subtotal but no tax lines.
- FX lock is missing or uses wrong timestamp.
- Credit memo application repeats.
- ERP export waits on invoice finalization.
- `invoice_generation_status=timeout` appears in worker logs.
- `tax_handoff_status=timeout` appears in worker logs.
- `fx_lock_status=missing` appears in worker logs.
- `credit_memo_apply_status=conflict` appears in worker logs.
- One tenant invoice blocks a partition queue.
- Paid invoices breach one-hour SLO first.
- DemoTrial backlog can hide a Paid SLO breach if queue priority is broken.
- Invoice numbers skip but final invoice row is absent.
- Audit-chain shows preview generated but finalization missing.
- Tenant billing portal shows spinner or stale previous invoice.
- Finance close dashboard is red.
- SOX control owner requests evidence.
- Customer impact is financial operations and trust.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-billing-invoice-timeout-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export PERIOD=2026-05`.
3. Acknowledge page: `pd incident ack --service cloud-billing --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-cloud-billing --severity sev1`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="invoice-generation")'`.
6. Query generation lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_billing_invoice_generation_lag_seconds{tenant_id="'$TENANT'",period="'$PERIOD'"}'`.
7. Query timeout count: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=rate(cloud_billing_invoice_generation_timeout_total[5m])'`.
8. Query queue depth: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_billing_invoice_queue_depth{period="'$PERIOD'"}'`.
9. Query tax errors: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_billing_tax_handoff_error_ratio{period="'$PERIOD'"}'`.
10. Query ERP lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=cloud_billing_erp_export_lag_seconds{period="'$PERIOD'"}'`.
11. Open invoice dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/invoicing?orgId=1&var-tenant=$TENANT&var-period=$PERIOD"`.
12. Open tax dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/tax-handoff?orgId=1&var-tenant=$TENANT&var-period=$PERIOD"`.
13. Read invoice logs: `kubectl -n cloud-billing logs deploy/cloud-billing-invoice-worker --since=60m | rg "invoice|tax_handoff|fx_lock|credit_memo|erp"`.
14. Check rollout: `kubectl -n cloud-billing rollout status deploy/cloud-billing-invoice-worker --timeout=60s`.
15. Inspect invoice state: `oya billing invoice get --tenant $TENANT --period $PERIOD --output yaml`.
16. Inspect invoice preview: `oya billing invoice preview --tenant $TENANT --period $PERIOD --output json`.
17. Inspect invoice queue: `oya billing invoice queue status --period $PERIOD --tenant $TENANT --output table`.
18. Inspect tenant tier: `oya tenancy tier get --tenant $TENANT --output yaml`.
19. Inspect rate card: `oya billing rate-card get --tenant $TENANT --period $PERIOD --output yaml`.
20. Inspect ledger close: `oya billing period status --tenant $TENANT --period $PERIOD --output json`.
21. Inspect tax handoff: `oya billing tax handoff status --tenant $TENANT --period $PERIOD --output json`.
22. Inspect tax response: `oya billing tax handoff get --tenant $TENANT --period $PERIOD --output json`.
23. Inspect FX lock: `oya billing fx-lock get --tenant $TENANT --period $PERIOD --output json`.
24. Inspect credit memos: `oya billing credit-memo list --tenant $TENANT --period $PERIOD --output table`.
25. Inspect debit memos: `oya billing debit-memo list --tenant $TENANT --period $PERIOD --output table`.
26. Inspect ERP export: `oya billing erp export status --tenant $TENANT --period $PERIOD --output json`.
27. Reconcile FOCUS: `oya billing focus reconcile --tenant $TENANT --period $PERIOD --dry-run --output json`.
28. Query invoice preview audit: `oya audit-chain query --event-class cloud_billing.invoice.preview_generated --tenant $TENANT --since 7d`.
29. Query invoice finalized audit: `oya audit-chain query --event-class cloud_billing.invoice.finalized --tenant $TENANT --since 7d`.
30. Query tax audit: `oya audit-chain query --event-class cloud_billing.tax.applied --tenant $TENANT --since 7d`.
31. Check close control: `oya finance close status --period $PERIOD --control invoice-generation --output json`.
32. Check support cases: `oya support cases list --tag invoice-generation --tenant $TENANT --since 7d`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-billing --runbook invoice-generation-timeout --output evidence/incidents/$INCIDENT_ID.json`.
34. Export invoice job: `oya billing invoice job export --tenant $TENANT --period $PERIOD --output evidence/incidents/$INCIDENT_ID-job.json`.
35. Export finalization inputs: `oya billing invoice finalization-inputs --tenant $TENANT --period $PERIOD --output evidence/incidents/$INCIDENT_ID-inputs.json`.

### Diagnostic Decision Tree
```text
1. Is invoice period ledger closed and reconciled?
   |-- no: invoke attribution or metering triage before invoice finalization.
   |-- yes: continue invoice pipeline triage.
2. Is tax handoff timing out?
   |-- yes: coordinate cloud-billing-tax and hold final invoice.
   |-- no: inspect FX, credit memo, rate card, and ERP export.
3. Is FX lock missing?
   |-- yes: lock FX rate from approved source and rerun preview.
   |-- no: continue queue and worker triage.
4. Is one tenant blocking partition queue?
   |-- yes: isolate tenant job and drain priority queue.
   |-- no: inspect deployment or database health.
5. Is the tier SLO breached?
   |-- yes: notify finance and support with tenant_class-specific impact.
   |-- no: continue until generation is fresh.
```

## Mitigation
1. Hold final invoice: `oya billing invoice hold --tenant $TENANT --period $PERIOD --reason $INCIDENT_ID`.
2. Mark portal status: `oya billing portal mark-delayed --tenant $TENANT --period $PERIOD --reason $INCIDENT_ID`.
3. Hold invoice worker deploys: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
4. Isolate blocking job: `oya billing invoice queue isolate --tenant $TENANT --period $PERIOD --reason $INCIDENT_ID`.
5. Drain priority queue dry-run: `oya billing invoice queue drain --period $PERIOD --tier paid,paid --dry-run`.
6. Drain priority queue confirmed: `oya billing invoice queue drain --period $PERIOD --tier paid,paid --confirm $INCIDENT_ID`.
7. Retry tax handoff dry-run: `oya billing tax handoff retry --tenant $TENANT --period $PERIOD --dry-run`.
8. Retry tax handoff confirmed: `oya billing tax handoff retry --tenant $TENANT --period $PERIOD --confirm $INCIDENT_ID`.
9. Apply FX lock from approved source: `oya billing fx-lock set --tenant $TENANT --period $PERIOD --source ECB-reference-rates-daily --confirm $INCIDENT_ID`.
10. Retry invoice preview: `oya billing invoice preview --tenant $TENANT --period $PERIOD --refresh --output evidence/incidents/$INCIDENT_ID-preview.json`.
11. Retry finalization: `oya billing invoice finalize --tenant $TENANT --period $PERIOD --confirm $INCIDENT_ID`.
12. Restart stuck worker: `kubectl -n cloud-billing rollout restart deploy/cloud-billing-invoice-worker`.
13. Roll back causal deploy: `kubectl -n cloud-billing rollout undo deploy/cloud-billing-invoice-worker`.
14. Notify finance: `oya notify finance-operations --incident $INCIDENT_ID --category invoice-timeout`.
15. Notify support: `oya notify support --incident $INCIDENT_ID --template invoice-generation-delayed`.
16. Notify tenant admin when portal delay is visible: `oya notify tenant-admin --tenant $TENANT --incident $INCIDENT_ID --template invoice-delay`.
17. Emit mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_BILLING_INVOICE_TIMEOUT_INCIDENT --incident $INCIDENT_ID --field mitigation=invoice-held`.
18. Keep partial invoice out of final state.
19. Keep tax-naive invoice preview labelled preview.
20. Keep ERP export held until final invoice audit event seals.

## Resolution
1. Patch invoice queue partitioning if one tenant blocked others.
2. Patch tax handoff timeout handling if tax call hung finalization.
3. Patch FX lock creation if missing lock blocked finalization.
4. Patch credit memo conflict handling if memo application deadlocked.
5. Patch rate-card lookup if invoice period had missing rate.
6. Patch ERP export if finalization succeeded but export timed out.
7. Add regression fixture for tax handoff timeout.
8. Add regression fixture for missing FX lock.
9. Run invoice tests: `cargo test -p cloud-billing-tax-app cloud_billing_invoice_api -- --nocapture`.
10. Run FinOps report tests: `cargo test -p cloud-finops-api cloud_finops_report_api -- --nocapture`.
11. Run production gate: `cargo run -p dev-cli -- gate validate cloud-billing-invoicing --production-snapshot --period $PERIOD`.
12. Verify final invoice: `oya billing invoice get --tenant $TENANT --period $PERIOD --expect finalized`.
13. Release portal delay: `oya billing portal clear-delayed --tenant $TENANT --period $PERIOD --reason resolved-$INCIDENT_ID`.
14. Release deploy hold: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_BILLING_INVOICE_TIMEOUT_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudBillingInvoiceGenerationTimeout` is green.
- `CloudBillingInvoiceSloBurn` is green for affected tier.
- `cloud_billing_invoice_generation_lag_seconds` is inside tier SLO.
- Invoice state is `finalized`.
- Tax lines exist for every taxable line item.
- FX lock has source and timestamp.
- Credit and debit memos are applied exactly once.
- ERP export is staged or complete.
- Audit-chain contains preview, tax applied, finalized, mitigation, and resolution events.
- Finance close control is green.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-billing-invoice-generation-timeout
microservice: cloud-billing
event_class: EVT_CLOUD_BILLING_INVOICE_TIMEOUT_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Invoice Generation Timeout postmortem

## Summary
- Which tenant, period, tier, and invoice job timed out.
- Which stage failed: ledger, tax, FX, memo, finalization, ERP.
- Whether invoice SLO or revenue recognition control was breached.

## Timeline
- Timeout detected:
- Invoice held:
- Failing stage identified:
- Invoice finalized:
- ERP export completed:

## Financial Impact
- Invoice amount:
- Tax amount:
- Delay duration:
- SLA credit:

## Root Cause
- Queue:
- Tax handoff:
- FX lock:
- Credit memo:
- ERP export:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Finance control:
```

## Escalation Path
- Page `cloud-billing-primary` for invoice generation timeout.
- Page `finance-operations-primary` when close or revenue recognition is blocked.
- Page `cloud-billing-tax-primary` when tax handoff is failing.
- Page `audit-chain-primary` when finalization events fail to seal.
- Notify `#inc-cloud-billing` with tenant, period, tier, and failing stage.
- Notify `#support-invoice-generation` before tenant messaging.
- Notify `#sox-controls` when month-end close is blocked.
- Escalate to executive incident commander when Paid SLO is breached for multiple tenants.
- Engage external tax provider only through cloud-billing-tax owner.
- Keep all external messages clear that preview is not final.

## Cross-µservice Coordination
- `cloud-billing-tax`: compute and verify tax lines.
- `audit-chain`: seal preview, tax, finalization, mitigation, and resolution events.
- `tenancy`: verify tenant tier and invoice contact routing.
- `cloud-iam`: verify billing worker principal and ERP export permissions.
- `cloud-kms`: verify invoice signing key if signed invoice fails.
- `comms-email`: send invoice delay and all-clear notices.
- `support`: manage tenant invoice cases.
- `finance-operations`: own close control and revenue recognition.
- `cloud-billing`: own ledger, rate card, FX lock, memo, and finalization.
- `observability`: attach invoicing and tax dashboards.
- `foundry`: pause invoicing deploys until resolved.
- `compliance`: review SOX and statutory invoice obligations.

## Runbook Maintenance
- Keep tier SLO thresholds aligned with billing FAQ.
- Add every new invoice stage to Diagnostic Steps.
- Keep tax handoff commands synced with cloud-billing-tax API.
- Review this runbook before every month-end close.
- Keep partial invoice warning explicit.
