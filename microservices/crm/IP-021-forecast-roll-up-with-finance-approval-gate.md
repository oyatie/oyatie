---
doc_class: ImplementationPlan
ip_id: IP-021
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j53-invoice-to-cash-recurring-subscription::weekly-forecast-commit
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-financial-planning + axis-governance
---

# IP-021: Forecast roll-up with finance approval gate

## 1. Context
This slice exists because CRM forecast commit is a revenue governance control, not just a sales dashboard.
The displaced SAP CRM submodule is SAP CRM Sales forecasting with territory and opportunity forecast categories.
The displaced Salesforce CRM submodule is Salesforce Sales Cloud Collaborative Forecasts and Forecast Categories.
The named persona is Esperanza Castillo, VP Sales LATAM at Pampas Hardware.
The named journey leg is j53 recurring subscription invoice-to-cash forecast commit.
Esperanza commits the weekly regional forecast, and Bao Linh in finance must counter-sign before the number enters corporate planning.
Salesforce lets a sales forecast become a commit without a finance-side two-key gate.
SAP forecasting models the sales side, but the information-barrier and MNPI evidence need Oyatie-native audit.
This implementation makes forecast snapshots append-only and only finance-accepted commits eligible for planning handoff.
It directly supports ADR-0319 front-office to middle-office separation.

## 2. Data Model Deltas
PostgreSQL DDL:
```sql
CREATE TYPE crm.forecast_cadence AS ENUM ('Week','Month','Quarter');
CREATE TYPE crm.forecast_commit_status AS ENUM ('Draft','Submitted','SalesCommittedAwaitingFinance','FinanceAccepted','FinanceRejected','Withdrawn');
CREATE TABLE crm.forecast_period (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  cadence crm.forecast_cadence NOT NULL,
  period_start date NOT NULL,
  period_end date NOT NULL,
  UNIQUE (tenant_id, cadence, period_start)
);
CREATE TABLE crm.forecast_snapshot (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  forecast_period_id uuid NOT NULL,
  territory_id uuid NOT NULL,
  submitted_by uuid NOT NULL,
  by_category jsonb NOT NULL,
  by_owner jsonb NOT NULL,
  notes text,
  notes_mnpi boolean NOT NULL DEFAULT false,
  source_signature text NOT NULL,
  cedar_decision_id uuid NOT NULL,
  audit_id uuid NOT NULL,
  submitted_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE crm.forecast_commit (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  forecast_period_id uuid NOT NULL,
  territory_id uuid NOT NULL,
  snapshot_id uuid NOT NULL,
  status crm.forecast_commit_status NOT NULL,
  sales_committer_id uuid,
  finance_accepter_id uuid,
  finance_decision_comment text,
  sales_audit_id uuid,
  finance_audit_id uuid,
  committed_at timestamptz,
  finance_decided_at timestamptz
);
ALTER TABLE crm.opportunity ADD COLUMN forecast_category text NOT NULL DEFAULT 'Pipeline';
ALTER TABLE crm.territory ADD COLUMN finance_owner_id uuid;
CREATE INDEX ix_crm_forecast_snapshot_period ON crm.forecast_snapshot(tenant_id, forecast_period_id, territory_id);
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub territory_id: TerritoryId, pub hierarchy_root_id: AccountId }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub role: ContactRole, pub email_hash: [u8; 32] }
pub struct Lead { pub id: LeadId, pub account_id: Option<AccountId>, pub forecast_source: Option<ForecastId>, pub score: i32 }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub stage: StageName, pub forecast_category: ForecastCategory }
pub struct Quote { pub id: QuoteId, pub opportunity_id: OpportunityId, pub approved_net_total: Money, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub opportunity_id: OpportunityId, pub booked_amount: Money, pub status: OrderStatus }
pub struct Contract { pub id: ContractId, pub account_id: AccountId, pub arr_amount: Money, pub renewal_date: Date }
pub struct Case { pub id: CaseId, pub account_id: AccountId, pub renewal_risk_flag: bool, pub severity: Severity }
pub struct Campaign { pub id: CampaignId, pub influenced_forecast_amount: Money, pub attribution_model: AttributionModel, pub status: CampaignStatus }
pub struct Solution { pub id: SolutionId, pub case_id: Option<CaseId>, pub forecast_risk_note: Option<String>, pub article_ref: String }
pub struct Forecast { pub id: ForecastId, pub period_id: ForecastPeriodId, pub territory_id: TerritoryId, pub commit_amount: Money }
pub struct Territory { pub id: TerritoryId, pub parent_id: Option<TerritoryId>, pub owner_id: UserId, pub finance_owner_id: UserId }
pub struct ChannelPartner { pub id: PartnerId, pub account_id: AccountId, pub partner_pipeline_amount: Money, pub visibility: VisibilityTier }
pub struct MarketingDocument { pub id: DocumentId, pub campaign_id: CampaignId, pub forecast_note_ref: Option<String>, pub storage_ref: String }
pub struct EmailTemplate { pub id: TemplateId, pub forecast_status: ForecastCommitStatus, pub locale: String, pub body_ref: String }
pub struct ForecastSnapshot { pub id: SnapshotId, pub by_category: ForecastBuckets, pub notes_mnpi: bool, pub source_signature: String }
pub struct ForecastCommit { pub id: CommitId, pub snapshot_id: SnapshotId, pub status: ForecastCommitStatus, pub finance_accepter_id: Option<UserId> }
```

## 3. API Endpoints
REST snapshot endpoint:
```http
POST /v1/crm/forecast/snapshots
```
REST snapshot body:
```json
{"tenant_id":"pampas-hardware-argentina","period_id":"fp_2026_w21","territory_id":"terr_latam","by_category":{"Pipeline":"9200000.00","BestCase":"5200000.00","Commit":"3100000.00","Closed":"1100000.00"},"notes":"LATAM renewal exposure improving","notes_mnpi":true}
```
REST commit endpoint:
```http
POST /v1/crm/forecast/commits
```
REST commit response:
```json
{"commit_id":"fc_021","status":"SalesCommittedAwaitingFinance","snapshot_id":"fs_021","sales_audit_id":"audit_sales_021"}
```
REST finance accept endpoint:
```http
POST /v1/crm/forecast/commits/fc_021/finance-accept
```
REST finance response:
```json
{"commit_id":"fc_021","status":"FinanceAccepted","finance_accepter_id":"usr_bao_linh","finance_audit_id":"audit_fin_021"}
```
gRPC contract:
```proto
service CrmForecastService {
  rpc SubmitForecastSnapshot(SubmitForecastSnapshotRequest) returns (SubmitForecastSnapshotResponse);
  rpc CommitForecast(CommitForecastRequest) returns (CommitForecastResponse);
  rpc FinanceDecideForecast(FinanceDecideForecastRequest) returns (FinanceDecideForecastResponse);
  rpc GetForecastRollup(GetForecastRollupRequest) returns (GetForecastRollupResponse);
}
message FinanceDecideForecastRequest { string tenant_id = 1; string commit_id = 2; string decision = 3; string comment = 4; }
```
AsyncAPI message:
```yaml
crm.forecast.commit.v1:
  publish:
    message:
      name: ForecastCommitFinanceAccepted
      payload:
        commit_id: fc_021
        period_id: fp_2026_w21
        territory_id: terr_latam
```

## 4. Cedar Policy Hooks
Stage-advance gate:
```cedar
permit(principal, action == Action::"crm.stage.advance", resource)
when { context.forecast_period_open == true && resource.territory_id in principal.visible_territory_ids };
```
Territory ownership:
```cedar
permit(principal in Role::"crm-sales-manager", action == Action::"crm.forecast.snapshot.submit", resource)
when { resource.manager_principal_id == principal.id || resource.id in principal.delegated_territory_ids };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"finance-forecast-acceptor", action == Action::"crm.forecast.rollup.approve", resource)
when { resource.status == "SalesCommittedAwaitingFinance" && resource.tenant_id == principal.tenant_id && context.source_signature != "" };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.forecast.partner_pipeline.read", resource)
when { context.partner_id == resource.partner_id && context.payload_class == "aggregate_only" };
```
MNPI read forbid:
```cedar
forbid(principal, action == Action::"crm.forecast.notes.read", resource)
when { resource.notes_mnpi == true && !(principal in Role::"finance-side-mnpi-cleared") };
```

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile` with territory and account hierarchy root.
Salesforce Contact maps to `Oyatie::Customer.primary_contact` only when forecast drill-down is authorized.
Salesforce Case maps to `Oyatie::Customer.service_posture` as renewal risk.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture` with forecast category and amount.
Field delta: Salesforce `ForecastingItem` becomes append-only ForecastSnapshot.
Field delta: Salesforce `ForecastingFact` becomes derived roll-up view.
Field delta: Salesforce manager override becomes snapshot override lineage.
Field delta: Salesforce commit flag becomes two-key ForecastCommit.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.forecast","customer_id":"cust_pampas","field_deltas":["revenue_posture.forecast_category","revenue_posture.commit_status","service_posture.renewal_risk"]}
```

## 6. Workflow Steps
Node `open_forecast_period` creates cadence and period.
Node `collect_opportunities` reads active pipeline by territory.
Node `collect_quotes_orders_contracts` adds approved quote, booked order, and ARR context.
Node `collect_case_risk` adds renewal blockers.
Node `bottom_up_aggregate` computes buckets by territory closure.
Node `submit_snapshot` writes append-only snapshot.
Decision `notes_mnpi_true` marks notes as finance-cleared only.
Node `sales_commit` changes commit state to awaiting finance.
Node `notify_finance_acceptor` creates finance task.
Decision `finance_rejects` keeps planning handoff blocked.
Node `finance_accept` writes second audit id.
Node `publish_financial_planning_handoff` emits only accepted commit.
Node `publish_intelligence_training_event` sends forecast variance features.

## 7. Audit Events
ADR-0263 registry class `CrmForecastPeriodOpened`.
ADR-0263 registry class `CrmForecastSnapshotSubmitted`.
ADR-0263 registry class `CrmForecastManagerOverrideApplied`.
ADR-0263 registry class `CrmForecastCommitSalesSigned`.
ADR-0263 registry class `CrmForecastCommitFinanceAccepted`.
ADR-0263 registry class `CrmForecastCommitFinanceRejected`.
ADR-0263 registry class `CrmForecastCommitWithdrawn`.
ADR-0263 registry class `CrmForecastMnpiNotesRedacted`.
ADR-0263 registry class `CrmForecastPlanningHandoffPublished`.
ADR-0263 registry class `CrmForecastPartnerAggregateRead`.

## 8. SLO Targets
p50 snapshot submit latency is 85 ms.
p95 snapshot submit latency is 250 ms for territories with fewer than 200 reports.
p99 snapshot submit latency is 1200 ms for large territory closure reads.
p50 roll-up read latency is 90 ms from cached aggregates.
p95 roll-up read latency is 400 ms for 500 descendant territories.
p99 async roll-up completion is 6000 ms for 5000 descendant territories.
Rationale: forecast commit is not a typing interaction, but Monday close cannot wait on minutes-long batch jobs.
Finance acceptance has a business SLA of 24 working hours and is tracked separately from system latency.

## 9. Failure Modes and Recovery
Salesforce Collaborative Forecast import hits Bulk API 10K ceiling; recovery chunks by territory and period.
Governor-limit parity appears when source forecast formulas depend on Apex; recovery stores source output and recomputes Oyatie buckets.
Manager override exceeds subordinate sum by more than 100 percent; recovery accepts only with high-risk audit and compliance alert.
Finance acceptance races with sales withdrawal; recovery allows withdrawal only inside configured grace and emits withdrawal audit.
MNPI leak risk appears when sales reads notes; recovery masks notes and emits redaction event.
Partner aggregate overexposure appears when partner pipeline drill-down includes direct deals; recovery returns aggregate-only partner payload.

## 10. Migration Notes
Salesforce Sales Cloud Collaborative Forecasts map to ForecastSnapshot and ForecastCommit.
Salesforce Service Cloud renewal risk cases map to Case risk inputs.
Salesforce Marketing Cloud campaign influence maps to forecast source annotations.
Salesforce Industries revenue schedules map to Opportunity and Contract forecast inputs.
SAP CRM-SLS forecast cubes map to snapshots.
SAP C4C sales forecasts map through external forecast period ids.
SAP Service Cloud renewal blockers map to Case risk flags.
Microsoft Dynamics 365 CE forecasts map to forecast periods and snapshots.
Microsoft Dynamics 365 Customer Service Hub cases map to renewal risk.
Oracle Fusion CX, Sales Cloud, and Service Cloud forecasts map to commit snapshots.
HubSpot Sales Hub forecast categories map to Opportunity forecast category.
HubSpot Service Hub tickets map to forecast risk context.
Zendesk Sell forecast reports map to snapshots.
Pipedrive weighted pipeline maps to Pipeline and BestCase buckets.
Freshsales forecast reports map to snapshots.
ActiveCampaign influenced opportunities map to campaign forecast annotations.

## 11. Cross-Service Handoffs
Marketplace receives finance-accepted forecast context for settlement capacity planning.
Payments receives accepted commit context for cash planning analytics, not invoice authority.
Community receives partner aggregate forecast content for partner channels.
Marketing-automation receives influenced-pipeline feedback.
Intelligence receives forecast variance and manager override features.
Financial-planning consumes only FinanceAccepted commits.
Compliance receives MNPI redaction and information-barrier evidence.
Ontology receives Customer revenue posture deltas.
Audit-chain seals sales and finance sides separately.
Workflow-engine manages close calendar and approval tasks.

## 12. Acceptance
Financial-planning cannot consume SalesCommittedAwaitingFinance commits.
Sales users cannot read MNPI notes unless explicitly cleared.
Every commit has sales and finance audit evidence when accepted.
Partner reads are aggregate-only.
Forecast roll-ups are deterministic for a frozen period and source signature.
The IP is bespoke to forecast roll-up and finance approval gating.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-021-forecast-roll-up-with-finance-approval-gate.md` matched [`financial`, `SLO`, `p99`].
- applicable_compliance_pack_floor: [`EU-AI-ACT-2024-HIGH-RISK`, `SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-021-forecast-roll-up-with-finance-approval-gate.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-021-forecast-roll-up-with-finance-approval-gate.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-021-forecast-roll-up-with-finance-approval-gate.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
