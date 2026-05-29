---
doc_class: Implementation-Plan
ip_id: IP-005
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0272, ADR-0273, ADR-0297, ADR-0313, ADR-0314, ADR-0315]
journey_ref: docs/user-journeys/j154-tomas-pieter-channel-partner-co-marketing-launch
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-marketing + axis-erp-parity
---
# IP-005: Campaign Domain Layer

## Context
- This slice builds the CRM-owned campaign aggregate for membership, influence, compliance boundaries, and partner campaign visibility.
- SAP benchmark: SAP CRM-MKT Campaign Management and Target Group handling.
- Salesforce benchmark: Marketing Cloud Account Engagement plus Sales Cloud Campaigns and Campaign Members.
- Persona: Tomas Pieter, channel partner at PartnerLift B.V.
- Journey leg: j154 co-marketing launch leg where PartnerLift and Glacier share a tenant-scoped campaign pool.
- Why now: lead routing, attribution, email templates, marketing documents, and partner settlement require a governed campaign root.
- This IP displaces Salesforce Marketing Cloud, SAP C/4HANA Marketing, Dynamics 365 Marketing, Oracle CX Marketing, HubSpot Marketing/Sales Hub, ActiveCampaign, Freshsales campaigns, and Pipedrive campaign add-ons.
- CRM owns campaign membership and CRM-side influence; marketing-automation owns send execution.
- Campaign consent is explicit per-purpose and per-channel; no campaign can infer consent from account membership alone.
- Partner co-marketing data is visible only through contract-scoped tenant grants.
- IP-019 later builds revenue attribution math; this slice creates immutable campaign atoms.
- The campaign aggregate must keep audit evidence sufficient for GDPR, DSA ad transparency, and partner settlement.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, legal_name TEXT NOT NULL, account_segment TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), email TEXT, consent_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_campaign_id UUID, status TEXT NOT NULL, consent_basis TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), primary_campaign_id UUID, stage TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), influenced_campaign_id UUID, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, quote_id UUID REFERENCES crm.quote(id), campaign_attribution_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID, co_marketing_terms_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), campaign_source_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, name TEXT NOT NULL, campaign_type TEXT NOT NULL, status TEXT NOT NULL, consent_purpose TEXT NOT NULL, partner_visibility TEXT NOT NULL, audit_id TEXT NOT NULL, version BIGINT NOT NULL DEFAULT 1);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), offer_solution_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), influenced_pipeline_amount NUMERIC(18,2), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_region TEXT, capacity_score NUMERIC(8,4), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), partner_role TEXT, portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), document_uri TEXT NOT NULL, approval_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), locale TEXT NOT NULL, subject TEXT NOT NULL, consent_purpose TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS crm_campaign_status_idx ON crm.campaign(tenant_id, status, campaign_type);
CREATE INDEX IF NOT EXISTS crm_campaign_partner_idx ON crm.channel_partner(tenant_id, campaign_id, portal_visibility);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub account_segment: String, pub legal_name: String }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub consent_state: ConsentState }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub source_campaign_id: Option<CampaignId>, pub consent_basis: ConsentBasis }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub primary_campaign_id: Option<CampaignId>, pub stage: OpportunityStage }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub influenced_campaign_id: Option<CampaignId>, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub campaign_attribution_ref: Option<String>, pub audit_id: AuditId }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub campaign_id: Option<CampaignId>, pub co_marketing_terms_ref: Option<String> }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub campaign_source_ref: Option<String>, pub account_id: AccountId }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub name: String, pub status: CampaignStatus, pub consent_purpose: ConsentPurpose }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub offer_solution_ref: String }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub influenced_pipeline_amount: Money }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub campaign_region: RegionCode, pub capacity_score: Decimal }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub partner_role: PartnerRole }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub approval_state: ApprovalState }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub consent_purpose: ConsentPurpose }
pub enum CampaignStatus { Draft, ComplianceReview, Active, Paused, Completed, Archived }
pub enum CampaignCommand { Create, AddMember, ApproveAsset, Activate, Pause, Complete, Archive }
```

## API Endpoints
- REST command: `POST /v1/crm/campaigns`.
- REST create body: `{ "tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be", "name": "Q1 Manufacturing NL-DE-BE", "campaign_type": "co_marketing", "consent_purpose": "b2b_direct_marketing" }`.
- REST member body: `{ "campaign_id": "camp_154", "contact_id": "con_154", "source_tenant": "partnerlift_nl", "attribution_split": "60_40" }`.
- REST activate body: `{ "campaign_id": "camp_154", "asset_refs": ["doc_1", "email_nl_1"], "gdpr_lawful_basis": "consent" }`.
- REST response: `{ "campaign_id": "camp_154", "status": "Active", "audit_event_class": "EVT-CRM-CAMPAIGN-ACTIVATED" }`.
- gRPC service: `rpc AddCampaignMember(AddCampaignMemberRequest) returns (CampaignMutationResult)`.
- gRPC service: `rpc ActivateCampaign(ActivateCampaignRequest) returns (CampaignMutationResult)`.
- gRPC request carries tenant_id, principal_id, campaign_id, contact_id, consent_purpose, source_tenant, traceparent, idempotency_key.
- AsyncAPI channel: `crm.campaign.events.v1`.
- AsyncAPI message: `CampaignMemberAdded`.
- AsyncAPI body: `{ "tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be", "campaign_id": "camp_154", "contact_id": "con_154", "audit_event_class": "EVT-CRM-CAMPAIGN-MEMBER-ADDED" }`.
- AsyncAPI consumers: marketing-automation, comms-email, community, analytics, marketplace, audit-chain.
- API forbids activation when email templates lack consent purpose.
- Campaign member import uses replayable async worker.
- Public contracts are SemVer v1 with additive-only evolution.

## Cedar Policy Hooks
- Stage-advance gate: campaign cannot move to Active until compliance review, consent basis, and asset approval pass.
- Territory ownership: campaign region must be inside principal marketing territory or shared tenant grant.
- Forecast-roll-up approval: influenced pipeline roll-up requires campaign attribution snapshot approval.
- Partner-portal visibility: partners see campaign member rows only for shared tenant scope and contract role.
- Add-member gate checks contact consent_state and purpose match.
- Asset approval gate checks marketing_document.approval_state.
- Email template gate checks ADR-0273 sender-domain alignment before activation.
- Context includes source_tenant, target_tenant, consent_purpose, dpa_signed, lawful_basis, dkim_aligned, traceparent.
- Denial returns policy_decision_id and masked contact fields.
- Every activation decision carries ADR-0263 audit_id.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.campaign_memberships`.
- Salesforce Case maps to `Oyatie::Customer.service_posture` for suppression rules.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline` for influence.
- Delta: Oyatie adds consent_purpose, source_tenant, partner_visibility, and attribution_split.
- Delta: Campaign member identity is tenant-scoped and cannot be globally deduped without consent.
- Delta: Marketing documents and email templates project as approved campaign assets only.
- Delta: DSA transparency refs stay on campaign events for ad reporting.

## Workflow Steps
- Node `create_campaign`: validate tenant, type, consent purpose, and partner topology.
- Node `add_campaign_member`: check contact, consent, source tenant, and duplicate membership.
- Node `approve_assets`: confirm marketing_document and email_template approval.
- Node `evaluate_activation_policy`: run Cedar with GDPR, DSA, partner, and deliverability context.
- Decision `shared_tenant_campaign`: branch to trinity grant validation.
- Node `activate_campaign`: commit Active status.
- Node `emit_member_events`: publish member and activation events.
- Node `notify_marketing_automation`: hand off send execution.
- Node `notify_marketplace`: record co-marketing settlement attribution basis.
- Branch `consent_missing`: reject member and emit consent-denied event.
- Branch `partner_scope_invalid`: block activation and seal denial.
- Branch `deliverability_not_ready`: keep ComplianceReview and return missing DNS refs.

## Audit Events
- `EVT-CRM-CAMPAIGN-CREATE-REQUESTED`.
- `EVT-CRM-CAMPAIGN-CREATED`.
- `EVT-CRM-CAMPAIGN-MEMBER-ADD-REQUESTED`.
- `EVT-CRM-CAMPAIGN-MEMBER-ADDED`.
- `EVT-CRM-CAMPAIGN-ASSET-APPROVED`.
- `EVT-CRM-CAMPAIGN-ACTIVATED`.
- `EVT-CRM-CAMPAIGN-PAUSED`.
- `EVT-CRM-CAMPAIGN-PARTNER-VISIBILITY-DENIED`.
- `EVT-CRM-CAMPAIGN-CONSENT-DENIED`.
- Events carry ADR-0263 audit_id, tenant_id, subscope, trace_id, span_id, schema_version, and source_microservice.

## SLO Targets
- Campaign create p50: 50 ms.
- Add member p95: 150 ms for single contact consent check.
- Activation p95: 350 ms with asset, consent, partner, and deliverability gates.
- Bulk member import p95: 10K contacts in 90 s async.
- Event publish p99: 1 s from activation commit.
- Availability: 99.9 percent for campaign mutation surface.
- Rationale: campaign activation can wait for policy gates, but member consent checks must stay fast enough for imports.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: chunk campaign members by campaign and contact cursor.
- Salesforce governor limits: throttle CampaignMember pulls and preserve campaign shell before members.
- Lead conversion conflict: keep campaign member linked to lead until conversion maps to contact/account.
- Consent revoked mid-import: skip row, seal consent-denied event, and continue batch.
- Partner shared tenant grant expires: pause activation and notify connect/identity.
- Marketing-automation unavailable: campaign remains ActivePendingDispatch with replayable event.

## Migration Notes
- Salesforce CRM/Marketing Cloud: Campaign.Id and CampaignMember.Id map to source_system_ref with consent remap.
- SAP CRM-MKT: campaign GUID maps to source_system_ref; target group maps to campaign member import set.
- Microsoft Dynamics 365 CE: campaignid and listmember map to campaign/member with lossy consent note.
- HubSpot Sales Hub/Marketing Hub: list and workflow enrollment map to campaign membership only with purpose.
- Pipedrive: campaign add-ons map to MarketingDocument plus member import when contact consent exists.
- Zendesk Sell: sequences map to EmailTemplate plus campaign member provenance.

## Cross-Service Handoffs
- Marketplace receives co-marketing attribution and settlement basis.
- Payments receives escrow release basis after marketplace settlement, not from CRM directly.
- Community receives partner campaign channel creation signal.
- Marketing-automation receives campaign activation and member deltas for send execution.
- Intelligence receives campaign response features for propensity scoring.
- Ontology receives Customer campaign membership projection.
- Comms-email receives sender-domain and template refs through marketing-automation.
- Audit-chain seals campaign activation, member, consent, and partner events.

## Build Checklist
- Implement Campaign aggregate with status transition table.
- Implement CampaignMember identity and uniqueness.
- Implement ConsentPurpose value object.
- Implement PartnerVisibility value object.
- Implement shared-tenant grant validator port.
- Implement asset approval state guard.
- Add test for activation without consent denied.
- Add test for shared tenant partner scope.
- Add test for deliverability not ready.
- Add test for consent revoked during import.
- Add REST create fixture.
- Add REST member fixture.
- Add REST activate fixture.
- Add gRPC AddCampaignMember fixture.
- Add AsyncAPI CampaignMemberAdded fixture.
- Add Cedar consent denial fixture.
- Add Cedar partner scope fixture.
- Add Salesforce Campaign migration fixture.
- Add SAP CRM-MKT target group fixture.
- Add HubSpot list import fixture.
- Add audit fixture with ADR-0263 fields.
- Add metric `oya:crm:campaign:add_member_latency_ms:histogram`.
- Add import metric for 10K chunk duration.
- Stop when member, activation, consent, partner, and handoff fixtures pass.

## Acceptance
- Campaign domain delegates send execution to marketing-automation.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, consent revocation, shared-tenant expiry, and marketing-automation outage.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, comms-email, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-005-domain-layer-for-campaign.md` matched [`SLO`, `p99`, `escrow`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-005-domain-layer-for-campaign.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-005-domain-layer-for-campaign.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-005-domain-layer-for-campaign.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
