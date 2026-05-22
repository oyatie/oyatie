# IP-017 Data Pipeline cost budget enforcer

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-017-cost-budget-enforcer.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Enforce budget before connector backfill, transform run, lineage repair, replay, and export.
- Keep budget enforcement separate from cost attribution finalization.
- Prevent runaway provider pulls and replay loops.
- Prevent transform approvals from hiding compute impact.
- Tie licensed connector cost to DealSet decision.
- Treat Fivetran and Airbyte Cloud usage metering as benchmark pressure.
- Treat Hevo and Stitch simple pricing as usability pressure.
- Treat Matillion and Talend Cloud job-cost controls as transform pressure.
- Treat Informatica IICS as enterprise chargeback pressure.
- Treat Estuary Flow as streaming cost pressure.

## Local references
- `microservices/data-pipeline/cost-budget.md` is the budget authority.
- `microservices/data-pipeline/capacity-model.md` defines capacity dimensions.
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml` consumes budget.
- `microservices/data-pipeline/capabilities/connector-run-start.yaml` consumes budget.
- `microservices/data-pipeline/runbooks/transform-job-cost-spike.md` defines cost incident response.
- `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json` observes spend.
- `microservices/data-pipeline/dashboards/local-domain-throughput.json` observes throughput.
- `microservices/data-pipeline/policies/local-transform-run-control.cedar` gates transform spend.
- `microservices/data-pipeline/slos/local-transform-latency.openslo.yaml` tracks latency.
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml` tracks ingest freshness.

## Budget scopes
- Tenant budget is enforced.
- Connector budget is enforced.
- Source object budget is enforced.
- Transform job budget is enforced.
- Transform version budget is enforced.
- Replay window budget is enforced.
- Backfill batch budget is enforced.
- Lineage repair budget is enforced.
- Audit export budget is enforced.
- Streaming watermark catch-up budget is enforced.
- DealSet licensed connector budget is enforced.
- Pack overlay premium budget is enforced.

## Budget states
- `within_budget` permits normal execution.
- `approaching_limit` permits execution and emits warning.
- `soft_limit_exceeded` requires reviewer approval.
- `hard_limit_exceeded` blocks execution.
- `unknown_budget` blocks high-cost execution.
- `degraded_estimate` requires explicit degraded approval.
- `quota_reset_pending` delays execution.
- `migration_credit_only` permits migration replay/export only.
- `dealset_over_quota` follows license quota policy.
- `incident_override` requires incident id.
- `manual_review` requires reviewer separation.
- `corrected` records rollback correction.

## Command deltas
- Connector run start requires budget check for high-volume source.
- Schema drift release requires budget check when backfill is required.
- Transform approval requires cost estimate and budget decision.
- Transform worker start requires active budget decision.
- Lineage repair requires graph mutation cost check.
- Replay approval requires replay cost estimate.
- Replay cursor advance requires finalized cost or accepted degraded state.
- Watermark catch-up requires freshness cost check.
- Audit export requires export cost check when large.
- DealSet connector check supplies license quota state.
- Capacity admission consumes budget state.
- Cost correction command records rollback adjustment.

## Event deltas
- `budget.checked` records budget decision.
- `budget.warning` records approaching limit.
- `budget.soft_limit_exceeded` records review requirement.
- `budget.hard_limit_exceeded` records block.
- `budget.override_approved` records reviewer approval.
- `budget.override_denied` records refusal.
- `budget.degraded_estimate_used` records degraded approval.
- `budget.cost_finalized` records actual spend.
- `budget.cost_corrected` records rollback correction.
- `budget.dealset_quota_blocked` records license quota block.
- Events include budget scope.
- Events include cost attribution id.

## Proto deltas
- `BudgetCheckRequest` includes tenant scope.
- `BudgetCheckRequest` includes budget scope.
- `BudgetCheckRequest` includes estimated units.
- `BudgetCheckRequest` includes connector id.
- `BudgetCheckRequest` includes transform job id when applicable.
- `BudgetCheckRequest` includes replay window id when applicable.
- `BudgetCheckResponse` includes budget state.
- `BudgetCheckResponse` includes decision id.
- `BudgetCheckResponse` includes reviewer requirement.
- `BudgetFinalizeRequest` includes actual units.
- `BudgetCorrectionRequest` includes rollback bundle.
- Proto rejects high-cost run without budget decision.

## Cedar facts
- `budget_scope` is a policy fact.
- `budget_state` is a policy fact.
- `estimated_cpu_ms` is a policy fact.
- `estimated_network_bytes` is a policy fact.
- `estimated_storage_bytes` is a policy fact.
- `actual_cost_state` is a policy fact.
- `dealset_quota_state` is a policy fact.
- `pack_overlay_cost_class` is a policy fact.
- `reviewer_separation_satisfied` is a policy fact.
- `incident_override_state` is a policy fact.
- `migration_credit_state` is a policy fact.
- `rollback_correction_state` is a policy fact.

## Workflow decisions
- Budget check runs before capacity admission.
- Budget check runs before transform approval.
- Budget check runs before replay approval.
- Budget check runs before large audit export.
- Budget soft limit opens review workflow.
- Budget hard limit blocks worker enqueue.
- Degraded estimate requires explicit operator marker.
- Actual cost finalization happens before completion claim.
- Rollback cost correction never deletes original actuals.
- DealSet quota and tenant budget both apply.
- Pack premium budget applies to regulated runs.
- Incident override expires with incident closure.

## Failure cases
- Missing budget blocks high-cost work.
- Budget service unavailable blocks high-cost work.
- Estimate unavailable blocks unless degraded approval exists.
- Actual finalization unavailable marks run incomplete.
- Soft limit without reviewer blocks approval.
- Hard limit blocks execution.
- DealSet over-quota blocks licensed connector.
- Pack premium conflict opens compliance review.
- Cost spike opens runbook.
- Replay loop budget exhaustion freezes replay window.
- Transform runaway triggers worker stop.
- Budget correction mismatch opens audit incident.

## Replay cases
- Replay estimate includes source read cost.
- Replay estimate includes target write cost.
- Replay estimate includes transform rerun cost.
- Replay estimate includes lineage repair cost.
- Replay estimate includes watermark catch-up cost.
- Replay actual records retry count.
- Replay hard limit freezes window.
- Replay soft limit requires approval.
- Replay rollback writes correction event.
- Replay migration credit can permit exit replay.
- Replay DealSet quota can block licensed source retry.
- Replay budget evidence attaches to cursor advance.

## Evidence fields
- `budget_decision_id` is mandatory.
- `budget_scope` is mandatory.
- `tenant_id` is mandatory.
- `connector_id` is mandatory when connector applies.
- `transform_job_id` is mandatory when transform applies.
- `replay_window_id` is mandatory when replay applies.
- `estimate_id` is mandatory before execution.
- `actual_id` is mandatory after execution.
- `budget_state` is mandatory.
- `limit_type` is mandatory when exceeded.
- `reviewer_id` is mandatory for override.
- `dealset_quota_state` is mandatory when licensed.
- `pack_cost_class` is mandatory when regulated.
- `rollback_bundle_id` is mandatory for correction.
- `audit_event_id` is mandatory.
- `benchmark_pressure` is mandatory for parity summary.

## SLOs
- Budget check latency is tracked separately.
- Transform cost spike links transform-job-cost-spike runbook.
- Replay budget exhaustion contributes to replay freshness risk.
- Connector budget block contributes to ingest freshness risk.
- Hard-limit count feeds tenant cost dashboard.
- Soft-limit count feeds operator remediation.
- Actual finalization lag feeds audit completeness.
- Budget correction count feeds audit findings.
- DealSet quota block feeds marketplace health.
- Pack premium conflict feeds compliance health.
- Cost estimate degradation feeds quality dashboard.
- Budget override age feeds review SLA.

## Test cases
- Transform approval rejects missing estimate.
- High-cost connector run rejects missing budget decision.
- Replay approval rejects hard-limit state.
- Soft-limit override requires reviewer separation.
- DealSet over-quota blocks licensed connector.
- Pack premium conflict opens review.
- Degraded estimate requires explicit approval.
- Actual finalization required before completion.
- Cost correction preserves original actual.
- Budget service unavailable fails closed for high-cost work.
- Migration credit permits only migration replay.
- Incident override expires after incident closure.

## Rollback
- Rollback writes cost correction.
- Rollback preserves original actuals.
- Rollback recomputes tenant budget burn.
- Rollback recomputes connector budget burn.
- Rollback recomputes replay budget burn.
- Rollback recomputes transform budget burn.
- Rollback preserves DealSet quota decisions.
- Rollback preserves pack premium decisions.
- Rollback emits budget correction event.
- Rollback links cost-spike runbook.
- Rollback requires audit event.
- Rollback verifies dashboard recalculation.

## Acceptance criteria
- High-cost work cannot run without budget decision.
- Transform approval cannot bypass estimate.
- Replay cannot bypass cost check.
- DealSet quota participates in budget enforcement.
- Pack overlay participates in budget enforcement.
- Actual costs finalize before completion claim.
- Cost corrections preserve history.
- Budget override requires reviewer separation.
- Every benchmark reference is comparative.
- Cost budget enforcer remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/cost-budget.md`
- `microservices/data-pipeline/capacity-model.md`
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml`
- `microservices/data-pipeline/capabilities/connector-run-start.yaml`
- `microservices/data-pipeline/runbooks/transform-job-cost-spike.md`
- `microservices/data-pipeline/dashboards/tenant-cost-and-capacity.json`
- `microservices/data-pipeline/dashboards/local-domain-throughput.json`
- `microservices/data-pipeline/policies/local-transform-run-control.cedar`
- `microservices/data-pipeline/slos/local-transform-latency.openslo.yaml`
- `microservices/data-pipeline/slos/local-ingest-freshness.openslo.yaml`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-017-cost-budget-enforcer.md:176` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-017-cost-budget-enforcer.md:1` - # IP-017 Data Pipeline cost budget enforcer; `microservices/data-pipeline/IP-017-cost-budget-enforcer.md:4` - ChangeSet scope: microservices/data-pipeline/IP-017-cost-budget-enforcer.md.
