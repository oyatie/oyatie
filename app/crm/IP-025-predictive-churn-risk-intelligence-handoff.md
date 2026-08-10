---
doc_class: ImplementationPlan
ip_id: IP-025
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j100-pack-rollout-from-tenant-onboarding-to-first-action::customer-retention-risk-detection
capability_profile: T2-product-erp-parity
status: Proposed
date: 2026-05-20
owner_team: axis-crm + axis-intelligence + axis-customer-success
---

# IP-025: Predictive churn-risk scoring with intelligence handoff

## 1. Context
This net-new slice exists because Customer 360 is incomplete if it only shows history and never predicts retention risk.
The displaced SAP CRM submodule is SAP CRM Analytics and C4C account intelligence for customer retention.
The displaced Salesforce CRM submodule is Salesforce Einstein Opportunity Insights, Account Insights, and Service Cloud predictive analytics.
The named persona is Maya Chen, customer success director at NorthernWind Wearables.
The named journey leg is j100 first action to customer-retention risk detection.
Maya needs churn risk surfaced from cases, contract renewal, opportunity regression, quote delay, order health, campaign disengagement, and partner signals.
Salesforce Einstein can score accounts, but feature lineage and model handoff are opaque for regulated tenants.
SAP analytics provides scoring surfaces, but Oyatie needs explicit intelligence microservice ownership and CRM-side explainability.
This implementation keeps CRM responsible for feature snapshots, customer-visible risk state, and workflow triggers.
The intelligence microservice owns model training, inference, drift checks, and model registry handoff.

## 2. Data Model Deltas
PostgreSQL DDL:
```sql
CREATE TYPE crm.churn_risk_bucket AS ENUM ('Low','Medium','High','Critical','Unknown');
CREATE TYPE crm.churn_signal_source AS ENUM ('Account','Contact','Lead','Opportunity','Quote','Order','Contract','Case','Campaign','Solution','Forecast','Territory','ChannelPartner','MarketingDocument','EmailTemplate');
CREATE TABLE crm.churn_signal_snapshot (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  customer_id uuid NOT NULL,
  account_id uuid NOT NULL,
  feature_window_start timestamptz NOT NULL,
  feature_window_end timestamptz NOT NULL,
  feature_vector jsonb NOT NULL,
  source_counts jsonb NOT NULL,
  consent_snapshot_id uuid NOT NULL,
  projection_signature text NOT NULL,
  audit_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE crm.churn_score_handoff (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  customer_id uuid NOT NULL,
  snapshot_id uuid NOT NULL,
  intelligence_request_id uuid NOT NULL,
  model_name text NOT NULL,
  model_version text NOT NULL,
  risk_score numeric(7,4) NOT NULL,
  risk_bucket crm.churn_risk_bucket NOT NULL,
  explanation jsonb NOT NULL,
  drift_status text NOT NULL CHECK (drift_status IN ('InFamily','Warning','OutOfFamily','Unknown')),
  cedar_decision_id uuid NOT NULL,
  audit_id uuid NOT NULL,
  scored_at timestamptz NOT NULL
);
ALTER TABLE crm.account ADD COLUMN churn_risk_bucket crm.churn_risk_bucket NOT NULL DEFAULT 'Unknown';
ALTER TABLE crm.account ADD COLUMN churn_risk_score numeric(7,4);
ALTER TABLE crm.account ADD COLUMN churn_score_handoff_id uuid;
CREATE INDEX ix_crm_churn_customer ON crm.churn_score_handoff(tenant_id, customer_id, scored_at DESC);
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub churn_risk_bucket: ChurnRiskBucket, pub renewal_date: Option<Date> }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub engagement_score: Decimal, pub consent_state: ConsentState }
pub struct Lead { pub id: LeadId, pub account_id: Option<AccountId>, pub source_quality_score: i32, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub stage_regression_count: u32, pub stage: StageName }
pub struct Quote { pub id: QuoteId, pub opportunity_id: OpportunityId, pub days_in_pending_approval: u32, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub account_id: AccountId, pub late_fulfillment_count: u32, pub status: OrderStatus }
pub struct Contract { pub id: ContractId, pub account_id: AccountId, pub renewal_date: Date, pub arr_amount: Money }
pub struct Case { pub id: CaseId, pub account_id: AccountId, pub sla_breach_count: u32, pub severity: Severity }
pub struct Campaign { pub id: CampaignId, pub disengagement_score: Decimal, pub attribution_model: AttributionModel, pub status: CampaignStatus }
pub struct Solution { pub id: SolutionId, pub case_id: CaseId, pub deflection_failed: bool, pub article_ref: String }
pub struct Forecast { pub id: ForecastId, pub account_id: AccountId, pub churn_adjusted_amount: Money, pub approved_by: Option<UserId> }
pub struct Territory { pub id: TerritoryId, pub owner_id: UserId, pub churn_capacity_units: i32, pub capacity_units: i32 }
pub struct ChannelPartner { pub id: PartnerId, pub account_id: AccountId, pub partner_health_score: Decimal, pub visibility: VisibilityTier }
pub struct MarketingDocument { pub id: DocumentId, pub engagement_signal: EngagementSignal, pub campaign_id: CampaignId, pub storage_ref: String }
pub struct EmailTemplate { pub id: TemplateId, pub negative_reply_rate: Decimal, pub locale: String, pub body_ref: String }
pub struct ChurnSignalSnapshot { pub id: SnapshotId, pub customer_id: CustomerId, pub feature_vector: FeatureVector, pub projection_signature: String }
pub struct ChurnScoreHandoff { pub id: HandoffId, pub model_version: String, pub risk_score: Decimal, pub risk_bucket: ChurnRiskBucket }
```

## 3. API Endpoints
REST snapshot endpoint:
```http
POST /v1/crm/churn-risk/snapshots
```
REST snapshot body:
```json
{"tenant_id":"northernwind-poland","customer_id":"cust_025","account_id":"acc_025","feature_window_days":180,"reason":"renewal-90-day-check"}
```
REST snapshot response:
```json
{"snapshot_id":"snap_025","source_counts":{"Case":14,"Opportunity":3,"Quote":2,"Order":7,"Campaign":21},"projection_signature":"sha256:cust025","audit_id":"audit_snapshot_025"}
```
REST score endpoint:
```http
POST /v1/crm/churn-risk/snapshots/snap_025:score
```
REST score response:
```json
{"handoff_id":"handoff_025","intelligence_request_id":"intel_req_025","model_name":"crm-churn-risk","model_version":"2026.05.20","risk_score":"0.8730","risk_bucket":"High","drift_status":"InFamily","audit_id":"audit_score_025"}
```
gRPC contract:
```proto
service CrmChurnRiskService {
  rpc BuildChurnSignalSnapshot(BuildChurnSignalSnapshotRequest) returns (BuildChurnSignalSnapshotResponse);
  rpc RequestChurnScore(RequestChurnScoreRequest) returns (RequestChurnScoreResponse);
  rpc ApplyChurnScore(ApplyChurnScoreRequest) returns (ApplyChurnScoreResponse);
  rpc GetCustomerChurnRisk(GetCustomerChurnRiskRequest) returns (GetCustomerChurnRiskResponse);
}
message RequestChurnScoreRequest { string tenant_id = 1; string snapshot_id = 2; string model_name = 3; string purpose = 4; }
```
AsyncAPI message:
```yaml
crm.churn.risk.v1:
  publish:
    message:
      name: ChurnRiskScoreApplied
      payload:
        customer_id: cust_025
        risk_bucket: High
        model_version: "2026.05.20"
```

## 4. Cedar Policy Hooks
Stage-advance gate:
```cedar
forbid(principal, action == Action::"crm.stage.advance", resource)
when { context.churn_risk_bucket in ["High","Critical"] && context.to_stage == "ClosedWon" && !(principal in Role::"crm-retention-risk-override") };
```
Territory ownership:
```cedar
permit(principal in Role::"crm-customer-success-owner", action == Action::"crm.churn.snapshot.build", resource)
when { resource.territory_id in principal.visible_territory_ids && resource.tenant_id == principal.tenant_id };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"crm-forecast-approver", action == Action::"crm.forecast.rollup.approve", resource)
when { context.churn_adjusted_amount_explained == true && context.model_drift_status != "OutOfFamily" };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.churn.partner_summary.read", resource)
when { context.partner_id == resource.channel_partner_id && context.payload_class == "risk_bucket_only" };
```
Model handoff:
```cedar
permit(principal in Role::"crm-intelligence-handoff", action == Action::"crm.churn.score.request", resource)
when { context.consent_state.profile_unification != "revoked" && context.purpose == "retention" };
```

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile` with churn risk bucket.
Salesforce Contact maps to `Oyatie::Customer.primary_contact` with engagement and consent features.
Salesforce Case maps to `Oyatie::Customer.service_posture` with SLA breach and reopen counts.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture` with stage regression and renewal pressure.
Field delta: Salesforce Einstein Account Score becomes `ChurnScoreHandoff`.
Field delta: Salesforce Case sentiment becomes service feature vector inputs.
Field delta: Salesforce Campaign engagement becomes consent-safe engagement signal.
Field delta: Salesforce Opportunity risk insights become explainable churn drivers.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.churn_risk","customer_id":"cust_025","field_deltas":["risk_flags.churn_score_bucket","revenue_posture.churn_adjusted_amount","service_posture.retention_drivers"]}
```

## 6. Workflow Steps
Node `load_customer_360_signature` fetches latest ontology signature.
Node `collect_account_contract_features` reads account tenure, ARR, renewal date, and contract changes.
Node `collect_case_solution_features` reads SLA breaches, reopen counts, severity, and failed deflection.
Node `collect_opportunity_quote_order_features` reads stage regression, quote delays, order delays, and churn-adjusted amount.
Node `collect_campaign_engagement_features` reads touch decay and disengagement.
Node `collect_partner_features` reads partner health where partner owns relationship.
Node `validate_consent_for_modeling` enforces retention purpose.
Node `write_signal_snapshot` stores feature vector and source counts.
Node `request_intelligence_score` sends snapshot id and features to intelligence.
Decision `model_drift_out_of_family` prevents applying score and creates review task.
Node `apply_churn_score` writes account risk and handoff row.
Node `publish_customer_projection_delta` emits risk bucket to ontology.
Node `trigger_retention_playbook` starts workflow for customer success.
Node `publish_forecast_adjustment` sends churn-adjusted amount to forecast.

## 7. Audit Events
ADR-0263 registry class `CrmChurnSignalSnapshotRequested`.
ADR-0263 registry class `CrmChurnSignalSnapshotCreated`.
ADR-0263 registry class `CrmChurnScoreRequestedFromIntelligence`.
ADR-0263 registry class `CrmChurnScoreReceived`.
ADR-0263 registry class `CrmChurnScoreApplied`.
ADR-0263 registry class `CrmChurnScoreRejectedDrift`.
ADR-0263 registry class `CrmChurnRetentionPlaybookTriggered`.
ADR-0263 registry class `CrmChurnForecastAdjustmentPublished`.
ADR-0263 registry class `CrmChurnPartnerRiskSummaryPublished`.
ADR-0263 registry class `CrmChurnConsentDenied`.

## 8. SLO Targets
p50 snapshot build latency is 140 ms with warm Customer 360 and case caches.
p95 snapshot build latency is 600 ms for 180 day feature windows.
p99 snapshot build latency is 2500 ms for accounts with high case and campaign volume.
p50 score handoff latency is 80 ms excluding intelligence inference.
p95 intelligence response application latency is 300 ms after inference result arrives.
p99 end-to-end snapshot to applied score is 5000 ms for online scoring.
Rationale: retention workflow can tolerate seconds, but customer success screens need current risk state once inference returns.
Model drift check must complete before score application.

## 9. Failure Modes and Recovery
Salesforce Einstein score import hits Bulk API 10K account ceiling; recovery chunks by account root and marks imported score as vendor annotation.
Governor-limit parity appears when source org computed risk in Apex; recovery records source formula output and recomputes Oyatie features.
Consent revocation blocks modeling; recovery writes ChurnConsentDenied and leaves bucket Unknown.
Model drift is OutOfFamily; recovery rejects score, opens intelligence review task, and keeps previous score.
Feature snapshot staleness exceeds Customer 360 signature; recovery rebuilds snapshot before handoff.
Partner risk summary could leak driver details; recovery sends risk bucket only through partner portal.

## 10. Migration Notes
Salesforce Sales Cloud Einstein Opportunity and Account Insights map to churn score annotations and handoffs.
Salesforce Service Cloud predictive service metrics map to Case and Solution features.
Salesforce Marketing Cloud engagement scores map to Campaign, MarketingDocument, and EmailTemplate features.
Salesforce Industries churn fields map to tenant-specific feature extensions.
SAP CRM Analytics and C4C customer insights map to ChurnSignalSnapshot.
SAP Service Cloud sentiment and ticket metrics map to Case and Solution features.
SAP C4C account intelligence maps to handoff annotations.
Microsoft Dynamics 365 CE relationship analytics maps to churn features.
Microsoft Dynamics 365 Customer Service Hub sentiment maps to Case features.
Oracle Fusion CX, Sales Cloud, and Service Cloud predictive scores map to imported annotations.
HubSpot Sales Hub health score fields map to churn features.
HubSpot Service Hub customer health maps to Case and Contract features.
Zendesk Sell customer health maps to Account and Case features.
Pipedrive deal rotting indicators map to Opportunity stage regression.
Freshsales Freddy AI scores map to imported annotations.
ActiveCampaign engagement scoring maps to campaign disengagement features.

## 11. Cross-Service Handoffs
Marketplace receives churn risk only when it affects partner settlement readiness.
Payments receives churn risk context for collections prioritization, not payment authorization.
Community receives retention playbook content eligibility.
Marketing-automation receives churn segment updates for consent-safe nurture.
Intelligence receives feature snapshots and returns model score, version, drift, and explanation.
Ontology receives Customer risk flag deltas.
Forecast receives churn-adjusted ARR and commit risk.
Workflow-engine starts retention playbooks.
Consent-graph validates modeling purpose and profile unification.
Audit-chain seals snapshot, handoff, application, and denial events.

## 12. Acceptance
CRM stores feature snapshots and applied score handoffs, not model weights.
Intelligence owns inference, model registry, and drift decisions.
Consent revocation blocks scoring and emits denial audit.
Out-of-family drift prevents applying a score.
Partner portal sees only risk bucket, never explanation drivers.
The IP is net-new and does not duplicate the existing 23 CRM IP topics.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-025-predictive-churn-risk-intelligence-handoff.md` matched [`SLO`, `p99`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-025-predictive-churn-risk-intelligence-handoff.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-025-predictive-churn-risk-intelligence-handoff.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-025-predictive-churn-risk-intelligence-handoff.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
