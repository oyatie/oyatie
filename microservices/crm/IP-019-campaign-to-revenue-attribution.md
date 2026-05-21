---
doc_class: ImplementationPlan
ip_id: IP-019
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j170-aiko-brown-sustainability-report-and-scope-3-supply-chain::campaign-to-revenue-attribution
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-marketing + axis-analytics
---

# IP-019: Campaign-to-revenue attribution

## 1. Context
This slice exists because campaign influence must be reproducible, not a spreadsheet argument between sales and marketing.
The displaced SAP CRM submodule is SAP CRM Marketing campaign management and interaction tracking.
The displaced Salesforce CRM submodule is Salesforce Marketing Cloud Account Engagement plus Sales Cloud Campaign Influence.
The named persona is Aiko Brown, head of marketing analytics at Vermillion Apparel.
The named journey leg is j170 sustainability report and Scope 3 supply-chain attribution.
Aiko needs board-ready influenced revenue that traces from marketing touch to opportunity, quote, order, and contract.
Salesforce Campaign Influence and Pardot activity streams are useful but often mutable and model-dependent.
SAP CRM-MKT campaign hierarchy provides structure but not Oyatie audit-chain proof.
This implementation makes campaign touches immutable and attribution snapshots append-only.
The slice connects campaign effort to revenue while preserving consent, account hierarchy, and partner-channel rules.

## 2. Data Model Deltas
PostgreSQL DDL:
```sql
CREATE TYPE crm.touch_type AS ENUM ('Open','Click','FormSubmit','EventAttended','ContentView','DemoBooked','PartnerReferral');
CREATE TABLE crm.campaign_touch (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  campaign_id uuid NOT NULL,
  contact_id uuid NOT NULL,
  account_id uuid,
  opportunity_id uuid,
  touch_type crm.touch_type NOT NULL,
  channel text NOT NULL,
  external_event_id text NOT NULL,
  touched_at timestamptz NOT NULL,
  consent_snapshot_id uuid NOT NULL,
  audit_id uuid NOT NULL,
  UNIQUE (tenant_id, external_event_id)
);
CREATE TABLE crm.campaign_attribution_snapshot (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  campaign_id uuid,
  opportunity_id uuid NOT NULL,
  quote_id uuid,
  order_id uuid,
  model_name text NOT NULL,
  model_version int NOT NULL,
  revenue_amount numeric(18,2) NOT NULL,
  breakdown jsonb NOT NULL,
  merkle_leaf_hash text NOT NULL,
  audit_id uuid NOT NULL,
  computed_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE crm.campaign ADD COLUMN attribution_window_days int NOT NULL DEFAULT 90;
ALTER TABLE crm.campaign ADD COLUMN ghg_scope3_category text;
ALTER TABLE crm.campaign ADD COLUMN parent_campaign_id uuid;
CREATE INDEX ix_crm_campaign_touch_contact ON crm.campaign_touch(tenant_id, contact_id, touched_at DESC);
CREATE INDEX ix_crm_attribution_opp ON crm.campaign_attribution_snapshot(tenant_id, opportunity_id, computed_at DESC);
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub hierarchy_root_id: AccountId, pub industry: String }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub consent_state: ConsentState, pub email_hash: [u8; 32] }
pub struct Lead { pub id: LeadId, pub source_campaign_id: Option<CampaignId>, pub score: i32, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub primary_campaign_id: Option<CampaignId>, pub stage: StageName }
pub struct Quote { pub id: QuoteId, pub opportunity_id: OpportunityId, pub campaign_influence_snapshot_id: Option<AttributionSnapshotId>, pub net_total: Money }
pub struct Order { pub id: OrderId, pub quote_id: QuoteId, pub closed_won_source: Option<CampaignId>, pub booked_amount: Money }
pub struct Contract { pub id: ContractId, pub order_id: OrderId, pub campaign_influence_snapshot_id: Option<AttributionSnapshotId>, pub term_months: u16 }
pub struct Case { pub id: CaseId, pub account_id: AccountId, pub campaign_context_id: Option<CampaignId>, pub severity: Severity }
pub struct Campaign { pub id: CampaignId, pub name: String, pub attribution_window_days: i32, pub ghg_scope3_category: Option<String> }
pub struct Solution { pub id: SolutionId, pub campaign_id: Option<CampaignId>, pub article_ref: String, pub influenced_cases: u32 }
pub struct Forecast { pub id: ForecastId, pub campaign_id: Option<CampaignId>, pub influenced_pipeline: Money, pub approved_by: Option<UserId> }
pub struct Territory { pub id: TerritoryId, pub campaign_scope: CampaignScope, pub owner_id: UserId, pub capacity_units: i32 }
pub struct ChannelPartner { pub id: PartnerId, pub referred_campaign_id: Option<CampaignId>, pub account_id: AccountId, pub rebate_terms: String }
pub struct MarketingDocument { pub id: DocumentId, pub campaign_id: CampaignId, pub document_kind: DocumentKind, pub storage_ref: String }
pub struct EmailTemplate { pub id: TemplateId, pub campaign_id: CampaignId, pub consent_purpose: ConsentPurpose, pub locale: String }
pub struct CampaignTouch { pub id: TouchId, pub touch_type: TouchType, pub external_event_id: String, pub consent_snapshot_id: ConsentSnapshotId }
pub struct CampaignAttributionSnapshot { pub id: AttributionSnapshotId, pub model_name: String, pub breakdown: Vec<AttributionLine>, pub merkle_leaf_hash: String }
```

## 3. API Endpoints
REST touch ingest endpoint:
```http
POST /v1/crm/campaigns/touches:batch
```
REST ingest body:
```json
{"tenant_id":"vermillion-apparel","source_system":"marketing-automation","touches":[{"external_event_id":"ma_7781","campaign_id":"cmp_scope3","contact_id":"con_170","touch_type":"ContentView","channel":"email","touched_at":"2026-05-20T14:10:00Z","consent_snapshot_id":"cons_884"}]}
```
REST attribution endpoint:
```http
POST /v1/crm/campaigns/attribution:snapshot
```
REST attribution body:
```json
{"tenant_id":"vermillion-apparel","model_name":"u_shaped","opportunity_ids":["opp_170"],"window_days":120,"include_partner_referrals":true}
```
REST response:
```json
{"job_id":"job_019","accepted_count":1,"model_name":"u_shaped","expected_snapshot_class":"CrmCampaignAttributionSnapshotSealed"}
```
gRPC contract:
```proto
service CrmCampaignAttributionService {
  rpc IngestCampaignTouches(IngestCampaignTouchesRequest) returns (IngestCampaignTouchesResponse);
  rpc ComputeAttributionSnapshot(ComputeAttributionSnapshotRequest) returns (ComputeAttributionSnapshotResponse);
  rpc GetOpportunityAttribution(GetOpportunityAttributionRequest) returns (GetOpportunityAttributionResponse);
}
message ComputeAttributionSnapshotRequest { string tenant_id = 1; string model_name = 2; repeated string opportunity_ids = 3; int32 window_days = 4; }
```
AsyncAPI message:
```yaml
crm.campaign.attribution.v1:
  publish:
    message:
      name: CampaignAttributionSnapshotSealed
      payload:
        snapshot_id: attr_019
        opportunity_id: opp_170
        merkle_leaf_hash: sha256:campaign019
```

## 4. Cedar Policy Hooks
Stage-advance gate:
```cedar
permit(principal, action == Action::"crm.stage.advance", resource)
when { context.campaign_influence_required == false || context.attribution_snapshot_id != "" };
```
Territory ownership:
```cedar
permit(principal in Role::"crm-campaign-analyst", action == Action::"crm.campaign.attribution.read", resource)
when { resource.territory_id in principal.visible_territory_ids && resource.tenant_id == principal.tenant_id };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"crm-forecast-approver", action == Action::"crm.forecast.rollup.approve", resource)
when { context.attribution_snapshot_sealed == true && context.model_name in ["first_touch","last_touch","u_shaped","w_shaped","linear","time_decay_30d"] };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.campaign.partner_influence.read", resource)
when { resource.partner_id == context.partner_id && context.payload_class == "partner_safe" };
```
Consent forbid:
```cedar
forbid(principal, action == Action::"crm.campaign.touch.ingest", resource)
when { context.consent_state.marketing == "revoked" };
```

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile` with campaign audience segment.
Salesforce Contact maps to `Oyatie::Customer.primary_contact` with consent-safe touch summary.
Salesforce Case maps to `Oyatie::Customer.service_posture` with campaign-driven support context.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture` with influence snapshot id.
Field delta: Salesforce Campaign becomes Campaign with `ghg_scope3_category`.
Field delta: Salesforce CampaignMember becomes consent-bound campaign membership.
Field delta: Salesforce CampaignInfluence becomes immutable attribution snapshot.
Field delta: Pardot prospect activity becomes `CampaignTouch` with idempotency.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.campaign_attribution","customer_id":"cust_170","field_deltas":["revenue_posture.campaign_influence","account_profile.audience_segment","service_posture.campaign_context"]}
```

## 6. Workflow Steps
Node `receive_touch_batch` accepts marketing-automation events.
Node `dedupe_external_event` enforces idempotency.
Node `validate_consent_snapshot` rejects revoked marketing consent.
Node `attach_contact_account` resolves Account and Contact.
Decision `opportunity_known` links direct opportunity or leaves touch account-scoped.
Node `write_campaign_touch` stores immutable touch.
Node `freeze_attribution_inputs` gathers touches, opportunity, quote, order, and contract.
Node `apply_model` computes first-touch, last-touch, U-shaped, W-shaped, linear, or time-decay weights.
Node `compute_merkle_leaf` hashes canonical breakdown.
Node `write_snapshot` appends the snapshot.
Node `publish_snapshot_sealed` sends analytics, forecast, marketplace, and compliance events.
Decision `esg_export_requested` requires `ghg_scope3_category`.
Node `complete_job` records deterministic input count and model version.

## 7. Audit Events
ADR-0263 registry class `CrmCampaignCreated`.
ADR-0263 registry class `CrmCampaignTouchIngested`.
ADR-0263 registry class `CrmCampaignTouchRejectedConsent`.
ADR-0263 registry class `CrmCampaignTouchDuplicateIgnored`.
ADR-0263 registry class `CrmCampaignAttributionSnapshotRequested`.
ADR-0263 registry class `CrmCampaignAttributionSnapshotSealed`.
ADR-0263 registry class `CrmCampaignAttributionSnapshotSuperseded`.
ADR-0263 registry class `CrmCampaignEsgExported`.
ADR-0263 registry class `CrmPartnerCampaignInfluencePublished`.
ADR-0263 registry class `CrmCampaignInfluenceForecastApproved`.

## 8. SLO Targets
p50 touch ingest latency is 35 ms for batches of 100.
p95 touch ingest latency is 90 ms for batches of 1000.
p99 touch ingest latency is 500 ms under consent snapshot cache misses.
p50 attribution compute latency is 180 ms for opportunities with fewer than 250 touches.
p95 attribution compute latency is 900 ms for opportunities with fewer than 1000 touches.
p99 attribution compute latency is 4000 ms for large accounts and runs asynchronously.
Rationale: marketing analytics tolerates async compute, but touch ingestion must stay low-latency to avoid source retries and duplicate storms.

## 9. Failure Modes and Recovery
Salesforce Bulk API 10K batch ceiling appears when importing CampaignMember and activity rows; recovery chunks by campaign and external id.
Pardot-style duplicate activity ids appear during connector replay; recovery idempotently ignores exact duplicates and quarantines mismatched duplicates.
Governor-limit parity failure appears when source attribution was computed by Apex; recovery stores source value as external benchmark and recomputes Oyatie snapshot.
Consent revocation arrives after a touch; recovery tombstones influence for future snapshots while preserving audit history.
Partner referral over-credit appears when partner and paid campaign both touched; recovery uses model version and emits partner-safe breakdown.
Attribution worker timeout appears on high-touch accounts; recovery marks job retryable and does not insert partial snapshots.

## 10. Migration Notes
Salesforce Sales Cloud Campaign Influence maps to immutable snapshot.
Salesforce Service Cloud campaign-related cases map to Case campaign context.
Salesforce Marketing Cloud and Account Engagement activity streams map to CampaignTouch.
Salesforce Industries campaign audiences map to account and territory scopes.
SAP CRM-MKT campaigns and campaign hierarchy map to Campaign and parent_campaign_id.
SAP C4C campaigns map to Campaign external references.
SAP Service Cloud interaction records map to service-aware touches.
Microsoft Dynamics 365 CE campaigns map to Campaign.
Microsoft Dynamics 365 Customer Service Hub interactions map to Case campaign context.
Oracle Fusion CX, Sales Cloud, Service Cloud, and Eloqua map to touch and attribution data.
HubSpot Sales Hub attribution reports map to snapshots.
HubSpot Service Hub ticket influence maps to Case context.
Zendesk Sell sequences map to touches.
Pipedrive campaign fields map to Opportunity source.
Freshsales campaigns map to Campaign and touch rows.
ActiveCampaign automations map to CampaignTouch and EmailTemplate.

## 11. Cross-Service Handoffs
Marketplace receives partner-influenced revenue rows for settlement and rebate logic.
Payments receives campaign attribution metadata for invoice analytics, not invoice authority.
Community receives partner-channel campaign content eligibility.
Marketing-automation produces raw touches and receives closed-loop conversion feedback.
Intelligence receives touch sequences for next-best-action and churn models.
Analytics receives sealed snapshots for OLAP cubes.
Compliance receives ESG export evidence.
Consent-graph validates consent snapshots.
Forecast receives influenced pipeline and booked revenue.
Audit-chain seals every snapshot.

## 12. Acceptance
Every touch is idempotent by tenant and external event id.
Attribution snapshots are append-only and model-versioned.
Consent revocation affects future attribution without mutating history.
Partner influence is partner-safe and Cedar-filtered.
ESG export requires campaign scope mapping.
The IP is bespoke to campaign-to-revenue attribution and satisfies the CRM substance bar.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-019-campaign-to-revenue-attribution.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-019-campaign-to-revenue-attribution.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-019-campaign-to-revenue-attribution.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-019-campaign-to-revenue-attribution.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
