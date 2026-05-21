---
doc_class: Implementation-Plan
ip_id: IP-011
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0272, ADR-0273, ADR-0297, ADR-0313, ADR-0314, ADR-0315]
journey_ref: docs/user-journeys/j154-tomas-pieter-channel-partner-co-marketing-launch
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-marketing + axis-erp-parity
---
# IP-011: Campaign Usecase Layer

## Context
- This slice orchestrates campaign create, member import, asset approval, activation, pause, completion, and archive.
- SAP benchmark: SAP CRM-MKT campaign execution handoff and target-group governance.
- Salesforce benchmark: Sales Cloud Campaigns, Campaign Members, Marketing Cloud Account Engagement, and Campaign Influence prerequisites.
- Persona: Tomas Pieter, channel partner at PartnerLift B.V.
- Journey leg: j154 shared-tenant co-marketing launch where CRM routes shared lead pools.
- Why now: campaign domain from IP-005 needs usecase coordination with consent, shared tenants, marketing-automation, comms-email, community, marketplace, and audit-chain.
- Vendor displacement includes Salesforce Marketing Cloud, SAP C/4HANA Marketing, Dynamics 365 Marketing, Oracle CX Marketing, HubSpot Marketing Hub, ActiveCampaign, Freshsales journeys, and Pipedrive campaigns.
- Usecase owns transaction order, idempotency, and consent-denial evidence.
- Usecase never sends email; marketing-automation owns send execution.
- Usecase never stores partner contract documents; connect owns signed contract artifacts.
- Usecase emits settlement-basis events for marketplace after campaign activation.
- Usecase must handle bulk member imports without hiding per-row consent failures.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_segment TEXT, lifecycle_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), consent_state TEXT NOT NULL, email TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_campaign_id UUID, routing_state TEXT NOT NULL, consent_basis TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, primary_campaign_id UUID, campaign_influence_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, influenced_campaign_id UUID, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_attribution_ref TEXT, marketplace_settlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID, co_marketing_terms_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), campaign_suppression_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, status TEXT NOT NULL, consent_purpose TEXT NOT NULL, usecase_state TEXT NOT NULL, partner_visibility TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), offer_solution_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), influenced_pipeline_amount NUMERIC(18,2), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_region TEXT, owner_principal_id UUID, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), shared_tenant_role TEXT, portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), approval_state TEXT NOT NULL, document_uri TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), locale TEXT NOT NULL, consent_purpose TEXT NOT NULL, approval_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign_usecase_event (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), outcome TEXT NOT NULL, rejected_rows INT NOT NULL DEFAULT 0, audit_id TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS crm_campaign_usecase_idx ON crm.campaign(tenant_id, status, usecase_state);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub account_segment: String, pub lifecycle_state: AccountState }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub consent_state: ConsentState }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub source_campaign_id: Option<CampaignId>, pub routing_state: RoutingState }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub primary_campaign_id: Option<CampaignId>, pub campaign_influence_state: InfluenceState }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub influenced_campaign_id: Option<CampaignId>, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub campaign_attribution_ref: Option<String>, pub marketplace_settlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub campaign_id: Option<CampaignId>, pub co_marketing_terms_ref: Option<String> }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub campaign_suppression_ref: Option<String>, pub account_id: AccountId }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub status: CampaignStatus, pub usecase_state: UsecaseState, pub partner_visibility: PortalVisibility }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub offer_solution_ref: String }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub influenced_pipeline_amount: Money }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub campaign_region: RegionCode, pub owner_principal_id: PrincipalId }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub shared_tenant_role: SharedTenantRole }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub approval_state: ApprovalState }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub approval_state: ApprovalState }
pub struct CampaignUsecasePorts { pub repo: CampaignRepoPort, pub consent: ConsentPort, pub policy: CedarPort, pub marketing: MarketingAutomationPort, pub audit: AuditChainPort }
pub enum CampaignUsecaseOutcome { Applied(CampaignId), Denied(PolicyDecisionId), PartialImport { accepted: u32, rejected: u32 }, PendingDispatch(CampaignId), Blocked(BlockerCode) }
```

## API Endpoints
- REST facade: `POST /v1/crm/campaigns/{id}/members:import`.
- REST import body: `{ "tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be", "campaign_id": "camp_154", "source_tenant": "partnerlift_nl", "contacts": ["con_1", "con_2"] }`.
- REST activate body: `{ "campaign_id": "camp_154", "asset_refs": ["doc_1", "email_de_1"], "dispatch_mode": "marketing_automation" }`.
- REST partial response: `{ "outcome": "PartialImport", "accepted": 9880, "rejected": 120, "audit_event_class": "EVT-CRM-CAMPAIGN-USECASE-PARTIAL-IMPORT" }`.
- REST activate response: `{ "outcome": "PendingDispatch", "campaign_id": "camp_154", "audit_event_class": "EVT-CRM-CAMPAIGN-USECASE-ACTIVATED" }`.
- gRPC facade: `rpc ImportCampaignMembers(ImportCampaignMembersRequest) returns (CampaignUsecaseReply)`.
- gRPC facade: `rpc ActivateCampaign(ActivateCampaignUsecaseRequest) returns (CampaignUsecaseReply)`.
- gRPC reply carries outcome, campaign_id, accepted, rejected, audit_id, policy_decision_id, dispatch_ref.
- AsyncAPI channel: `crm.campaign.usecase.events.v1`.
- AsyncAPI message: `CampaignUsecaseActivated`.
- AsyncAPI body: `{ "tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be", "campaign_id": "camp_154", "outcome": "PendingDispatch", "audit_event_class": "EVT-CRM-CAMPAIGN-USECASE-ACTIVATED" }`.
- Usecase emits marketing-automation dispatch event after audit seal.
- Usecase emits community partner-channel event after partner visibility permit.
- REST maps row-level consent failures to partial import.
- AsyncAPI event never includes raw email addresses.

## Cedar Policy Hooks
- Stage-advance gate: campaign moves to Active only after compliance review, asset approval, and deliverability checks.
- Territory ownership: principal must own campaign_region or hold shared-tenant campaign grant.
- Forecast-roll-up approval: influenced pipeline snapshots require campaign attribution approval.
- Partner-portal visibility: partner sees only shared-tenant campaign rows covered by contract.
- Import context includes source_tenant, lawful_basis, consent_purpose, DPA state, and row count.
- Activation context includes dkim_aligned, dmarc_aligned, email_template_approved, and marketing_document_approved.
- Resource includes campaign_id, tenant_id, status, campaign_type, partner_visibility.
- Denied rows emit row-level denial counters without leaking PII.
- Cedar timeout blocks activation and returns Blocked.
- ADR-0263 audit_id links each aggregate and partial-import event.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.campaign_memberships`.
- Salesforce Case maps to `Oyatie::Customer.service_suppression`.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline`.
- Delta: usecase projection includes import outcome and consent purpose.
- Delta: rejected contacts are counted but not projected.
- Delta: partner shared-tenant visibility is explicit.
- Delta: activation dispatch ref links to marketing-automation, not CRM send ownership.

## Workflow Steps
- Node `receive_import`: normalize contact batch and source tenant.
- Node `load_campaign`: verify campaign status and tenant.
- Node `evaluate_import_policy`: run consent, partner, and source-tenant gates.
- Node `process_rows`: accept or reject each contact deterministically.
- Decision `row_rejections_present`: return PartialImport with counts and evidence.
- Node `persist_membership_events`: write usecase event and outbox rows.
- Node `seal_audit`: seal import and activation evidence.
- Node `activate_campaign`: run asset and deliverability checks.
- Node `emit_dispatch_event`: notify marketing-automation.
- Node `emit_partner_channel_event`: notify community when allowed.
- Branch `dpa_missing`: block import and notify connect.
- Branch `dispatch_unavailable`: return PendingDispatch and retry.

## Audit Events
- `EVT-CRM-CAMPAIGN-USECASE-RECEIVED`.
- `EVT-CRM-CAMPAIGN-USECASE-PARTIAL-IMPORT`.
- `EVT-CRM-CAMPAIGN-USECASE-IMPORT-APPLIED`.
- `EVT-CRM-CAMPAIGN-USECASE-POLICY-DENIED`.
- `EVT-CRM-CAMPAIGN-USECASE-ACTIVATED`.
- `EVT-CRM-CAMPAIGN-USECASE-PENDING-DISPATCH`.
- `EVT-CRM-CAMPAIGN-USECASE-PARTNER-CHANNEL-EMITTED`.
- `EVT-CRM-CAMPAIGN-USECASE-PII-MASKED`.
- ADR-0263 fields include audit_id, tenant_id, campaign_id, row_count, rejected_rows, trace_id, span_id, schema_version.

## SLO Targets
- Member import p50: 1K rows in 6 s.
- Member import p95: 10K rows in 90 s async.
- Activation p95: 400 ms with policy and deliverability gates.
- Partial import response p99: 120 s for 10K rows.
- Dispatch event publish p95: 600 ms.
- Availability: 99.9 percent for campaign usecases.
- Rationale: batch import can be async, but activation must tell users quickly why a launch is blocked.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: import CampaignMember in exact 10K-or-smaller chunks.
- Salesforce governor limits: throttle campaign member pull and preserve checkpoint by campaign id.
- Lead conversion conflict: route member to lead until contact conversion repairs.
- Consent mismatch: reject row, seal consent-denied event, continue batch.
- Shared DPA missing: block import and call connect contract attestation.
- Marketing-automation dispatch outage: return PendingDispatch and replay outbox.

## Migration Notes
- Salesforce CRM/Marketing Cloud: CampaignMember imports keep status and response date.
- SAP CRM-MKT: target group members become campaign members after consent map.
- Microsoft Dynamics 365 CE: marketing list members import with purpose remap.
- HubSpot Sales Hub/Marketing Hub: list membership maps to campaign membership only with consent purpose.
- Pipedrive: campaign plugins import as MarketingDocument plus optional member rows.
- Zendesk Sell: sequence contacts import as campaign members with provenance warning.

## Cross-Service Handoffs
- Marketplace receives campaign settlement-basis event for co-marketing.
- Payments receives escrow release through marketplace, not CRM.
- Community receives partner campaign channel signal.
- Marketing-automation receives dispatch event and member deltas.
- Intelligence receives response and rejection features.
- Ontology receives Customer campaign membership projection.
- Connect receives DPA-missing and contract-attestation requests.
- Audit-chain seals all import, activation, and denial evidence.

## Build Checklist
- Implement ImportCampaignMembersUsecase.
- Implement ActivateCampaignUsecase.
- Define CampaignUsecasePorts.
- Implement row-level consent evaluator.
- Implement shared-tenant DPA gate.
- Implement partial import outcome.
- Implement dispatch outbox.
- Add 10K import test.
- Add consent mismatch row test.
- Add DPA missing test.
- Add dispatch outage test.
- Add REST import fixture.
- Add REST activate fixture.
- Add gRPC import fixture.
- Add AsyncAPI activated fixture.
- Add Cedar partner visibility fixture.
- Add Salesforce CampaignMember fixture.
- Add SAP target group fixture.
- Add HubSpot list fixture.
- Add audit fixture with ADR-0263 fields.
- Add metric `oya:crm:campaign_usecase:import_rows_total:counter`.
- Add metric `oya:crm:campaign_usecase:activation_latency_ms:histogram`.
- Add PII masking fixture.
- Stop when import, partial rejection, activation, dispatch, and handoff tests pass.

## Acceptance
- Usecase does not send email or store signed partner contracts.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, consent mismatch, DPA missing, and dispatch outage.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, connect, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-011-usecase-layer-for-campaign.md` matched [`SLO`, `p99`, `escrow`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-011-usecase-layer-for-campaign.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-011-usecase-layer-for-campaign.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-011-usecase-layer-for-campaign.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`plugin`].
- surface_evidence_paths: [`microservices/crm/IP-011-usecase-layer-for-campaign.md`, `microservices/crm/manifest.json`, `microservices/workflow-studio/manifest.json`, `microservices/crm/contracts/openapi-v1.yaml`].
