---
doc_class: Runbook
title: Reservation Recommendation Engine Stall
status: Accepted
date: 2026-05-20
microservice: cloud-billing
severity: sev2
audience: sre, finops-engineer, product-operator
owner_team: axis-cloud + finance-operations + ops-sre-reliability
doc_status: published
---

# Runbook: Reservation Recommendation Engine Stall

## Operator Contract
- Runbook id: cloud-billing-reservation-recommendation-engine-stall.
- Primary namespace: `cloud-billing`.
- Owning rotation: PagerDuty `oya-cloud-billing-primary`.
- FinOps secondary: PagerDuty `oya-finops-primary`.
- Incident channel: `#inc-cloud-billing`.
- Customer channel: `#support-reservation-recommendations`.
- Protected surface: reservation recommender, utilization forecasts, savings plans, convertible reservation guidance, tenant commitments.
- Safety invariant: never auto-purchase a reservation during this incident.
- Finance invariant: stale recommendations must be marked stale in tenant UI and exports.
- Stop condition: recommender backlog is drained, forecasts are fresh, and tenant-visible recommendations are no older than SLO.
- Evidence event: `EVT_CLOUD_BILLING_RESERVATION_RECOMMENDER_STALL_INCIDENT`.
- Handoff API: `https://cloud-billing.internal.oyatie.dev/v1/reservations/incidents/$INCIDENT_ID/handoff`.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/reservations?orgId=1&var-cell=prod-us-east-1`.
- Forecast dashboard: `https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/forecasting?orgId=1&var-period=current`.
- Loki query: `{namespace="cloud-billing",runbook="reservation-recommendation-engine-stall"}`.
- Canonical FAQ: `microservices/cloud-billing/faqs/billing-engineer-faq.md`.
- Related benchmark: `microservices/cloud-billing/benchmarks/cloud-billing-vs-aws-cur-vs-gcp-billing-vs-azure-cost-management.md`.
- Related action: `cloud_billing::Action::ConvertReservation`.

## Trigger Conditions
- Alert `CloudBillingReservationRecommendationStale` fires.
- Alert `CloudBillingReservationForecastLagHigh` fires.
- Alert `CloudBillingReservationSavingsModelError` fires.
- Alert `CloudBillingReservationExportStale` fires.
- Metric `oya_cloud_billing_reservation_recommendation_age_seconds` exceeds 86400.
- Metric `oya_cloud_billing_reservation_forecast_lag_seconds` exceeds 7200.
- Metric `oya_cloud_billing_reservation_recommender_queue_depth` exceeds 10000.
- Metric `oya_cloud_billing_reservation_model_error_ratio` exceeds 0.02.
- Metric `oya_cloud_billing_reservation_candidate_count` drops to zero for active tenants.
- Metric `oya_cloud_billing_reservation_commitment_delta_usd` spikes.
- Metric `oya_cloud_billing_reservation_utilization_input_stale_total` increases.
- Metric `oya_cloud_billing_reservation_recommendation_export_error_total` increases.
- Tenant UI shows recommendation generated more than 24 hours ago.
- Finance asks why savings recommendations disappeared.
- A tenant contract renewal depends on fresh reservation recommendations.
- Vendor CUR ingest lag blocks utilization inputs.
- Cloud-compute usage aggregates are stale.
- Cloud-billing rate card changed and recommendation model did not refresh.
- Model deployment occurred inside the stale window.
- Audit-chain lacks `cloud_billing.reservation.recommendation.generated`.

## Symptoms
- Reservation dashboard shows stale timestamp.
- Recommendation list is empty for tenants with obvious steady usage.
- Savings estimate differs sharply from prior daily baseline.
- Convertible reservation options are missing.
- Recommendation export file is not generated.
- Tenant UI labels recommendations as current despite stale backend timestamp.
- Recommender worker has growing queue and low throughput.
- Forecast worker is healthy but model errors rise.
- Utilization input table has no rows for recent hours.
- `reservation_model_status=failed` appears in worker logs.
- `utilization_input_status=stale` appears in worker logs.
- `rate_card_version_mismatch=true` appears in recommendations.
- `forecast_horizon_days=0` appears for active tenants.
- Recommender rejects tenant tier because tenancy projection is stale.
- Savings recommendations ignore reservation already purchased.
- Recommendation export to tenant warehouse is missing.
- No invoice generation impact exists yet, but savings guidance is wrong.
- Customer impact is financial optimization, not immediate billing correctness.
- Severity rises if automated purchase pipeline was enabled.
- Severity rises if stale recommendation was sent to customer success or renewal.

## Diagnostic Steps
1. Set scope: `export INCIDENT_ID=INC-cloud-billing-reservation-stall-$(date -u +%Y%m%dT%H%M%SZ)`.
2. Set defaults: `export CELL=prod-us-east-1; export TENANT=synthetic-canary; export PERIOD=2026-05`.
3. Acknowledge page: `pd incident ack --service cloud-billing --incident $INCIDENT_ID`.
4. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-cloud-billing --severity sev2`.
5. Query active alerts: `curl -s https://alertmanager.dev.oyatie.internal/api/v2/alerts | jq '.[] | select(.labels.surface=="reservation-recommender")'`.
6. Query recommendation age: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_billing_reservation_recommendation_age_seconds{tenant_id="'$TENANT'"}'`.
7. Query forecast lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_billing_reservation_forecast_lag_seconds{tenant_id="'$TENANT'"}'`.
8. Query queue depth: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_billing_reservation_recommender_queue_depth{cell="'$CELL'"}'`.
9. Query model errors: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_billing_reservation_model_error_ratio{cell="'$CELL'"}'`.
10. Query candidate count: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_cloud_billing_reservation_candidate_count{tenant_id="'$TENANT'"}'`.
11. Open reservations dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/reservations?orgId=1&var-tenant=$TENANT&var-period=$PERIOD"`.
12. Open forecast dashboard: `open "https://grafana.dev.oyatie.internal/d/cloud-billing-substrate/forecasting?orgId=1&var-tenant=$TENANT&var-period=$PERIOD"`.
13. Read worker logs: `kubectl -n cloud-billing logs deploy/cloud-billing-reservation-recommender --since=60m | rg "reservation|forecast|model|utilization|rate_card"`.
14. Check rollout: `kubectl -n cloud-billing rollout status deploy/cloud-billing-reservation-recommender --timeout=60s`.
15. Inspect recommender status: `oya billing reservations recommender status --tenant $TENANT --period $PERIOD --output json`.
16. Inspect stale inputs: `oya billing reservations inputs status --tenant $TENANT --period $PERIOD --output table`.
17. Inspect utilization: `oya billing reservations utilization --tenant $TENANT --period $PERIOD --output json`.
18. Inspect forecast: `oya billing reservations forecast --tenant $TENANT --period $PERIOD --output json`.
19. Inspect candidates: `oya billing reservations candidates --tenant $TENANT --period $PERIOD --output table`.
20. Inspect existing reservations: `oya billing reservations list --tenant $TENANT --period $PERIOD --output table`.
21. Inspect convertible options: `oya billing reservations convertible-options --tenant $TENANT --period $PERIOD --output json`.
22. Inspect rate card: `oya billing rate-card get --tenant $TENANT --period $PERIOD --output yaml`.
23. Inspect tenant tier: `oya tenancy tier get --tenant $TENANT --output yaml`.
24. Inspect vendor ingest: `oya billing vendor ingest status --period $PERIOD --source all --output table`.
25. Inspect cloud-compute aggregate: `oya billing usage aggregate status --tenant $TENANT --resource-family cloud_compute --period $PERIOD --output json`.
26. Check export status: `oya billing reservations export status --tenant $TENANT --period $PERIOD --output json`.
27. Query generation events: `oya audit-chain query --event-class cloud_billing.reservation.recommendation.generated --tenant $TENANT --since 7d`.
28. Query stale marker events: `oya audit-chain query --event-class cloud_billing.reservation.recommendation.marked_stale --tenant $TENANT --since 7d`.
29. Check purchase automation flag: `oya flags get oya.cloud_billing.reservation.auto_purchase --tenant $TENANT --output yaml`.
30. Check UI stale marker: `oya billing ui state --tenant $TENANT --surface reservation-recommendations --output json`.
31. Check renewal dependency: `oya crm renewals list --tenant $TENANT --fields reservation_recommendation_status --output table`.
32. Check support cases: `oya support cases list --tag reservation-recommendations --tenant $TENANT --since 7d`.
33. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice cloud-billing --runbook reservation-recommendation-engine-stall --output evidence/incidents/$INCIDENT_ID.json`.
34. Export recommendation state: `oya billing reservations recommender export --tenant $TENANT --period $PERIOD --output evidence/incidents/$INCIDENT_ID-recommender.json`.
35. Export input freshness: `oya billing reservations inputs export --tenant $TENANT --period $PERIOD --output evidence/incidents/$INCIDENT_ID-inputs.json`.

### Diagnostic Decision Tree
```text
1. Is auto-purchase enabled anywhere?
   |-- yes: disable immediately and page finance operations.
   |-- no: continue freshness triage.
2. Are utilization inputs stale?
   |-- yes: repair vendor or cloud-compute aggregates first.
   |-- no: inspect model and rate-card version.
3. Is model error ratio high?
   |-- yes: roll back recommender model or disable generated recommendations.
   |-- no: inspect queue and export path.
4. Are recommendations generated but export/UI stale?
   |-- yes: repair export or UI projection.
   |-- no: drain recommender backlog.
5. Is customer renewal using stale recommendations?
   |-- yes: mark stale and notify customer success.
   |-- no: close after freshness SLO holds.
```

## Mitigation
1. Disable auto-purchase: `oya flags set oya.cloud_billing.reservation.auto_purchase=false --global --reason $INCIDENT_ID`.
2. Mark stale in tenant UI: `oya billing reservations mark-stale --tenant $TENANT --period $PERIOD --reason $INCIDENT_ID`.
3. Hold recommendation exports: `oya billing reservations export hold --tenant $TENANT --period $PERIOD --reason $INCIDENT_ID`.
4. Hold recommender deploys: incident hold PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
5. Refresh utilization inputs: `oya billing reservations inputs refresh --tenant $TENANT --period $PERIOD --dry-run`.
6. Confirm input refresh: `oya billing reservations inputs refresh --tenant $TENANT --period $PERIOD --confirm $INCIDENT_ID`.
7. Drain queue dry-run: `oya billing reservations recommender drain --tenant $TENANT --period $PERIOD --limit 100 --dry-run`.
8. Drain queue confirmed: `oya billing reservations recommender drain --tenant $TENANT --period $PERIOD --limit 100 --confirm $INCIDENT_ID`.
9. Roll back model if causal: `kubectl -n cloud-billing rollout undo deploy/cloud-billing-reservation-recommender`.
10. Pin prior model: `oya billing reservations model pin --version previous-stable --reason $INCIDENT_ID`.
11. Regenerate recommendations: `oya billing reservations generate --tenant $TENANT --period $PERIOD --confirm $INCIDENT_ID`.
12. Regenerate export: `oya billing reservations export --tenant $TENANT --period $PERIOD --output evidence/incidents/$INCIDENT_ID-recommendations.parquet`.
13. Notify finance: `oya notify finance-operations --incident $INCIDENT_ID --category reservation-recommender`.
14. Notify customer success: `oya notify customer-success --tenant $TENANT --incident $INCIDENT_ID --template reservation-recommendation-stale`.
15. Notify support: `oya notify support --incident $INCIDENT_ID --template reservation-recommendation-stale`.
16. Emit mitigation audit: `oya audit-chain emit --event-class EVT_CLOUD_BILLING_RESERVATION_RECOMMENDER_STALL_INCIDENT --incident $INCIDENT_ID --field mitigation=stale-marked`.
17. Keep stale label visible until generation event is fresh.
18. Keep automated purchase disabled until finance signs off.
19. Keep renewal teams from using stale recommendations.
20. Keep every generated file attached to incident evidence.

## Resolution
1. Patch vendor ingest dependency if utilization was stale.
2. Patch cloud-compute aggregate dependency if usage family was missing.
3. Patch recommender model input validation if candidates dropped to zero.
4. Patch rate-card version compatibility if model read wrong pricing.
5. Patch UI stale marker if backend marked stale but UI did not.
6. Patch export worker if recommendations generated but warehouse export failed.
7. Add regression fixture for stale utilization input.
8. Add regression fixture for rate-card change.
9. Run domain tests: `cargo test -p oya-cloud-billing-domain reservation -- --nocapture`.
10. Run API tests: `cargo test -p oya-cloud-finops-api cloud_finops_report_api -- --nocapture`.
11. Run production gate: `cargo run -p oya-dev-cli -- gate validate cloud-billing-reservations --production-snapshot --period $PERIOD`.
12. Verify fresh recommendations: `oya billing reservations recommender status --tenant $TENANT --period $PERIOD --expect fresh`.
13. Release export hold: `oya billing reservations export unhold --tenant $TENANT --period $PERIOD --reason resolved-$INCIDENT_ID`.
14. Unhold deploys: recovery PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
15. Seal audit: `oya audit-chain emit --event-class EVT_CLOUD_BILLING_RESERVATION_RECOMMENDER_STALL_INCIDENT --incident $INCIDENT_ID --field resolution=complete`.

## Verification Checklist
- `CloudBillingReservationRecommendationStale` is green.
- `oya_cloud_billing_reservation_recommendation_age_seconds` is below 86400.
- `oya_cloud_billing_reservation_forecast_lag_seconds` is below 7200.
- `oya_cloud_billing_reservation_recommender_queue_depth` returns to baseline.
- Recommendation export is fresh.
- Tenant UI shows fresh timestamp.
- Auto-purchase remains disabled unless separately approved.
- Finance confirms recommendations are usable.
- Customer success is notified if renewal material used stale output.
- Audit-chain contains stale marker, generation, mitigation, and resolution events.

## Postmortem Template
```markdown
---
doc_class: IncidentPostmortem
runbook_id: cloud-billing-reservation-recommendation-engine-stall
microservice: cloud-billing
event_class: EVT_CLOUD_BILLING_RESERVATION_RECOMMENDER_STALL_INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Reservation Recommendation Engine Stall postmortem

## Summary
- Which tenant, period, resource family, and model version stalled.
- Whether stale recommendations were visible.
- Whether any renewal or customer communication used stale data.

## Timeline
- Staleness detected:
- UI marked stale:
- Inputs refreshed:
- Recommendations regenerated:
- Export released:

## Financial Impact
- Estimated missed savings:
- Renewal impact:
- Tenant communication:

## Root Cause
- Ingest:
- Model:
- Rate card:
- Export:
- UI stale marker:

## Corrective Actions
- Owner:
- Due date:
- Regression test:
- Finance control:
```

## Escalation Path
- Page `oya-cloud-billing-primary` for recommender staleness.
- Page `oya-finops-primary` when recommendations drive customer savings or renewal.
- Page `oya-cloud-compute-primary` when utilization aggregate is stale.
- Page `oya-audit-chain-primary` when generation events are missing.
- Notify `#inc-cloud-billing` with tenant, period, and model version.
- Notify `#support-reservation-recommendations` for tenant-visible staleness.
- Notify `#customer-success-renewals` when renewal collateral is affected.
- Escalate to finance leadership if stale recommendation changed commitment guidance.
- Escalate to executive incident commander if auto-purchase was enabled.
- Keep recommendations marked stale until verified fresh.

## Cross-µservice Coordination
- `cloud-compute`: verify utilization aggregates for VM, pod, and function usage.
- `cloud-billing`: own rate card, reservation, and recommendation state.
- `cloud-iam`: verify recommender principal permissions.
- `audit-chain`: seal generated, stale, mitigation, and resolution events.
- `tenancy`: confirm tenant tier and contract eligibility.
- `cloud-billing-tax`: confirm no tax impact from recommendation-only changes.
- `support`: manage tenant questions.
- `customer-success`: avoid stale renewal collateral.
- `finance-operations`: validate recommendation finance logic.
- `observability`: attach recommender and forecast dashboards.
- `foundry`: pause recommender model deploys while incident is active.
- `comms-email`: send approved stale/fresh notifications.

## Runbook Maintenance
- Add new resource families to input freshness checks.
- Keep model rollback commands current.
- Keep auto-purchase disablement first.
- Review this runbook before each rate-card launch.
- Add every recommender model version failure to Symptoms.
