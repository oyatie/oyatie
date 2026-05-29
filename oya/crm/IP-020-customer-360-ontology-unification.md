---
doc_class: ImplementationPlan
ip_id: IP-020
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j100-pack-rollout-from-tenant-onboarding-to-first-action::customer-360-first-action
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-ontology + axis-consent
---

# IP-020: Customer 360 ontology unification

## 1. Context
This slice exists because support, sales, marketing, and partner teams need one truthful Customer entity instead of four stale tabs.
The displaced SAP CRM submodule is SAP C/4HANA customer data and SAP CRM Interaction Center customer overview.
The displaced Salesforce CRM submodule is Salesforce Customer 360 Data Manager and Data Cloud profile unification.
The named persona is Mateusz Kowalski, tier-2 support specialist at NorthernWind Wearables.
The named journey leg is j100 tenant onboarding to first support and sales action.
Mateusz needs one view showing account hierarchy, contacts, open cases, opportunities, consent, campaign touches, contracts, and partner context.
Salesforce Data Cloud can unify profiles, but lag and licensing make it hard to guarantee sub-second consent masking.
SAP C/4HANA central business partner records are strong but do not express Oyatie ontology and Cedar projection hooks.
This implementation makes CRM the emitter of Customer 360 facts and ontology the materialized read owner.
The slice prevents consent-revoked data from leaking into sales, partner, or service views.

## 2. Data Model Deltas
PostgreSQL DDL:
```sql
CREATE TABLE crm.customer_360_emission_log (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  customer_id uuid NOT NULL,
  account_id uuid NOT NULL,
  contact_id uuid,
  trigger_kind text NOT NULL CHECK (trigger_kind IN ('AccountChange','ContactChange','LeadChange','OpportunityChange','QuoteChange','OrderChange','ContractChange','CaseChange','CampaignChange','ConsentChange','PartnerChange','Rebuild')),
  projection_signature text NOT NULL,
  consent_snapshot_id uuid NOT NULL,
  audit_id uuid NOT NULL,
  emitted_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE crm.customer_360_projection_checkpoint (
  tenant_id uuid NOT NULL,
  customer_id uuid NOT NULL,
  last_signature text NOT NULL,
  last_emitted_at timestamptz NOT NULL,
  last_audit_id uuid NOT NULL,
  PRIMARY KEY (tenant_id, customer_id)
);
ALTER TABLE crm.contact ADD COLUMN customer_360_key text;
ALTER TABLE crm.contact ADD COLUMN email_hash bytea;
ALTER TABLE crm.contact ADD COLUMN phone_hash bytea;
ALTER TABLE crm.account ADD COLUMN customer_360_root_id uuid;
CREATE INDEX ix_crm_customer_360_log_customer ON crm.customer_360_emission_log(tenant_id, customer_id, emitted_at DESC);
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub customer_360_root_id: CustomerId, pub legal_name: String }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub customer_360_key: String, pub consent_state: ConsentState }
pub struct Lead { pub id: LeadId, pub converted_customer_id: Option<CustomerId>, pub source: LeadSource, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub customer_id: CustomerId, pub stage: StageName }
pub struct Quote { pub id: QuoteId, pub opportunity_id: OpportunityId, pub customer_id: CustomerId, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub quote_id: QuoteId, pub customer_id: CustomerId, pub booked_amount: Money }
pub struct Contract { pub id: ContractId, pub customer_id: CustomerId, pub account_id: AccountId, pub effective_range: DateRange }
pub struct Case { pub id: CaseId, pub customer_id: CustomerId, pub account_id: AccountId, pub status: CaseStatus }
pub struct Campaign { pub id: CampaignId, pub customer_segment_id: Option<SegmentId>, pub attribution_model: AttributionModel, pub status: CampaignStatus }
pub struct Solution { pub id: SolutionId, pub customer_context_id: CustomerId, pub case_id: CaseId, pub article_ref: String }
pub struct Forecast { pub id: ForecastId, pub customer_scope_id: CustomerId, pub commit_amount: Money, pub approved_by: Option<UserId> }
pub struct Territory { pub id: TerritoryId, pub customer_scope_id: CustomerId, pub owner_id: UserId, pub capacity_units: i32 }
pub struct ChannelPartner { pub id: PartnerId, pub customer_scope_id: CustomerId, pub account_id: AccountId, pub visibility: VisibilityTier }
pub struct MarketingDocument { pub id: DocumentId, pub customer_segment_id: Option<SegmentId>, pub campaign_id: CampaignId, pub storage_ref: String }
pub struct EmailTemplate { pub id: TemplateId, pub customer_segment_id: Option<SegmentId>, pub consent_purpose: ConsentPurpose, pub locale: String }
pub struct Customer360EmissionLog { pub id: EmissionId, pub customer_id: CustomerId, pub trigger_kind: TriggerKind, pub projection_signature: String }
```

## 3. API Endpoints
REST rebuild endpoint:
```http
POST /v1/crm/customer-360/rebuild
```
REST rebuild body:
```json
{"tenant_id":"northernwind-poland","customer_ids":["cust_020"],"reason":"consent-revocation-repair","priority":"High"}
```
REST rebuild response:
```json
{"job_id":"job_020","queued_count":1,"expected_trigger":"Rebuild","audit_id":"audit_020"}
```
REST signature endpoint:
```http
GET /v1/crm/customer-360/cust_020/projection-signature
```
REST signature response:
```json
{"customer_id":"cust_020","last_signature":"sha256:c360020","last_emitted_at":"2026-05-20T15:00:00Z","last_audit_id":"audit_020"}
```
gRPC contract:
```proto
service CrmCustomer360ProjectionService {
  rpc RebuildCustomer360(RebuildCustomer360Request) returns (RebuildCustomer360Response);
  rpc EmitCustomer360Delta(EmitCustomer360DeltaRequest) returns (EmitCustomer360DeltaResponse);
  rpc GetCustomer360ProjectionSignature(GetCustomer360ProjectionSignatureRequest) returns (GetCustomer360ProjectionSignatureResponse);
}
message EmitCustomer360DeltaRequest { string tenant_id = 1; string customer_id = 2; string trigger_kind = 3; string projection_signature = 4; }
```
AsyncAPI message:
```yaml
crm.customer360.projection.v1:
  publish:
    message:
      name: Customer360ProjectionEmitted
      payload:
        customer_id: cust_020
        trigger_kind: ConsentChange
        projection_signature: sha256:c360020
```

## 4. Cedar Policy Hooks
Stage-advance gate:
```cedar
permit(principal, action == Action::"crm.stage.advance", resource)
when { context.customer_360_visible == true && context.consent_state.profile_unification != "revoked" };
```
Territory ownership:
```cedar
permit(principal in Role::"crm-territory-owner", action == Action::"crm.customer360.read", resource)
when { resource.territory_id == principal.territory_id && resource.tenant_id == principal.tenant_id };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"crm-forecast-approver", action == Action::"crm.forecast.rollup.approve", resource)
when { context.customer_projection_signature != "" && context.customer_projection_staleness_ms < 2000 };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.customer360.partner_read", resource)
when { resource.partner_visible == true && context.partner_payload_class == "scrubbed" && context.partner_id == resource.channel_partner_id };
```
Consent hard forbid:
```cedar
forbid(principal, action == Action::"crm.customer360.read", resource)
when { resource.consent_state.profile_unification == "revoked" };
```

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile`.
Salesforce Contact maps to `Oyatie::Customer.primary_contact` and contact methods.
Salesforce Case maps to `Oyatie::Customer.service_posture`.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture`.
Field delta: Salesforce `UnifiedIndividual` becomes `Oyatie::Customer.customer_id`.
Field delta: Salesforce `Individual` consent flags become consent-graph snapshots.
Field delta: Salesforce `Account.AnnualRevenue` is not trusted for lifetime value; billing and order facts provide it.
Field delta: Salesforce `Case.Status` becomes service posture with SLA and escalation data.
Field delta: Salesforce `Opportunity.StageName` becomes stage enum plus decision lineage.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.customer360","customer_id":"cust_020","field_deltas":["account_profile","primary_contact","service_posture","revenue_posture","consent_state","partner_visibility"]}
```
Ontology owns materialized storage.
CRM owns the emission contract.
Consent-graph owns revocation truth.

## 6. Workflow Steps
Node `collect_account` reads account profile and hierarchy root.
Node `collect_contacts` reads primary and related contacts.
Node `collect_leads` reads active and converted lead history.
Node `collect_opportunities` reads revenue posture.
Node `collect_quotes_orders_contracts` reads commercial lifecycle facts.
Node `collect_cases_solutions` reads service posture and knowledge state.
Node `collect_campaigns_documents_templates` reads marketing posture.
Node `collect_partner_context` reads partner visibility scope.
Node `fetch_consent_snapshot` reads consent-graph.
Decision `profile_unification_revoked` emits deny projection and no rich profile.
Node `apply_consent_masks` removes disallowed channel data.
Node `compute_projection_signature` canonicalizes fields.
Decision `signature_unchanged` suppresses duplicate emission.
Node `emit_customer_projection` publishes AsyncAPI.
Node `write_checkpoint` updates checkpoint and audit id.

## 7. Audit Events
ADR-0263 registry class `CrmCustomer360RebuildRequested`.
ADR-0263 registry class `CrmCustomer360InputCollected`.
ADR-0263 registry class `CrmCustomer360ConsentMasked`.
ADR-0263 registry class `CrmCustomer360ProjectionSuppressedUnchanged`.
ADR-0263 registry class `CrmCustomer360ProjectionEmitted`.
ADR-0263 registry class `CrmCustomer360ReadDeniedConsent`.
ADR-0263 registry class `CrmCustomer360PartnerProjectionScrubbed`.
ADR-0263 registry class `CrmCustomer360ProjectionCheckpointWritten`.
ADR-0263 registry class `CrmCustomer360ProjectionRepairQueued`.
ADR-0263 registry class `CrmCustomer360ProjectionDriftDetected`.

## 8. SLO Targets
p50 projection emission latency is 90 ms with warm Account, Contact, and Case reads.
p95 projection emission latency is 250 ms for ordinary customer changes.
p99 projection emission latency is 800 ms during consent and partner context fan-out.
p50 signature lookup latency is 15 ms.
p95 signature lookup latency is 50 ms.
p99 consent revocation propagation is 1000 ms.
Rationale: consent revocation is legally sensitive and must beat batch-oriented Salesforce Data Cloud unification delays.
Freshness target is 99 percent of Customer 360 reads using data less than two seconds old.

## 9. Failure Modes and Recovery
Salesforce Data Cloud batch-lag parity appears during migration; recovery marks imported profiles as stale until Oyatie emits a fresh signature.
Bulk API 10K contact batch ceiling appears on profile seeding; recovery chunks by Account root and resumes by external id.
Governor-limit parity failure appears when source calculated fields diverge; recovery treats source values as annotations and recomputes canonical fields.
Consent-graph lag exceeds threshold; recovery fail-closes rich Customer 360 reads until fresh consent arrives.
Partial service read failure appears when Case service is unavailable; recovery emits partial projection with explicit `partial=true`.
Partner visibility leak risk appears when partner-linked customer data includes PII; recovery publishes scrubbed partner projection only.

## 10. Migration Notes
Salesforce Sales Cloud Account, Contact, Lead, and Opportunity map to Customer profile and revenue posture.
Salesforce Service Cloud Case maps to service posture.
Salesforce Marketing Cloud profile and engagement map to consent-safe marketing posture.
Salesforce Industries customer roles map to industry-specific account extensions.
SAP CRM business partner and SAP C4HANA customer data map to Account and Contact roots.
SAP C4C customer views map to Customer projections.
SAP Service Cloud tickets map to Case posture.
Microsoft Dynamics 365 CE accounts and contacts map to Customer identity.
Microsoft Dynamics 365 Customer Service Hub cases map to service posture.
Oracle Fusion CX, Sales Cloud, and Service Cloud profiles map to Customer projection components.
HubSpot Sales Hub companies and deals map to account and revenue posture.
HubSpot Service Hub tickets map to service posture.
Zendesk Sell leads and contacts map to Lead and Contact.
Pipedrive organizations and persons map to Account and Contact.
Freshsales account profiles map to Account and Contact.
ActiveCampaign contact timelines map to marketing posture and EmailTemplate usage.

## 11. Cross-Service Handoffs
Marketplace receives customer identity and account root for deal settlement context.
Payments receives customer id for invoice and receipt linkage.
Community receives partner-safe customer context for partner-channel content.
Marketing-automation receives consent-safe profile deltas and suppression state.
Intelligence receives Customer 360 features for churn and next-best-action models.
Ontology receives projection events and serves read API.
Consent-graph provides consent snapshots and revocation events.
Audit-chain seals projection emissions and denial events.
Compliance uses projection logs for DSAR and retention evidence.
Workflow-engine schedules rebuild and repair jobs.

## 12. Acceptance
CRM does not store the materialized Customer 360 read model.
Every projection has a signature and consent snapshot.
Consent revocation propagates within the stated SLO.
Partner portal payloads are scrubbed and Cedar-filtered.
Projection duplicate suppression is signature-based and auditable.
The IP is bespoke to Customer 360 ontology unification and meets the substance bar.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-020-customer-360-ontology-unification.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-020-customer-360-ontology-unification.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-020-customer-360-ontology-unification.md` matched [`emission`, `attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-020-customer-360-ontology-unification.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
