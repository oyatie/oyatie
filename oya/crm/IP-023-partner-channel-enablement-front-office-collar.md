---
doc_class: ImplementationPlan
ip_id: IP-023
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j112-tenant-to-tenant-rfq-and-bid::partner-channel-deal-registration
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-marketplace + axis-partner
---

# IP-023: Partner-channel enablement and front-office collar

## 1. Context
This slice exists because partner selling cannot be implemented as broad sharing rules without leaking tenant data.
The displaced SAP CRM submodule is SAP CRM Channel Management and Partner Relationship Management.
The displaced Salesforce CRM submodule is Salesforce Partner Community, Channel Sales, Partner Portal, and Deal Registration.
The named persona is Renata Costa, channel manager at Cobrahub Industrial.
The named journey leg is j112 tenant-to-tenant RFQ and bid with partner registration.
Renata shares a lead with NorteSul Distribuidora, and Mateus Almeida registers a protected partner deal within the allowed window.
Salesforce Partner Community depends on org-local sharing rules that are brittle across sovereign tenants.
SAP PRM gives channel structure but not Oyatie external front-office collar enforcement.
This implementation treats partner access as explicit, expiring, cross-tenant resource binding.
It binds partner users to front-office surfaces and blocks back-office workspace actions.

## 2. Data Model Deltas
PostgreSQL DDL:
```sql
CREATE TYPE crm.partner_tier AS ENUM ('Authorized','retired-standard','retired-advanced','retired-sovereign','Strategic');
CREATE TYPE crm.deal_registration_status AS ENUM ('Submitted','Approved','Rejected','Expired','Won','Lost');
CREATE TABLE crm.partner_organization (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  partner_tenant_id uuid NOT NULL,
  partner_tier crm.partner_tier NOT NULL,
  contract_id uuid NOT NULL,
  territory_scope jsonb NOT NULL DEFAULT '{}'::jsonb,
  effective_from date NOT NULL,
  effective_to date,
  UNIQUE (tenant_id, partner_tenant_id)
);
CREATE TABLE crm.partner_principal_binding (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  partner_organization_id uuid NOT NULL,
  external_principal_id uuid NOT NULL,
  collar text NOT NULL DEFAULT 'external-front-office',
  cedar_role_bindings text[] NOT NULL,
  status text NOT NULL CHECK (status IN ('Invited','Active','Suspended','Revoked')),
  audit_id uuid,
  invited_at timestamptz NOT NULL DEFAULT now(),
  activated_at timestamptz
);
CREATE TABLE crm.shared_resource_binding (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  resource_type text NOT NULL CHECK (resource_type IN ('Account','Contact','Lead','Opportunity','Quote','Case','Campaign','MarketingDocument')),
  resource_id uuid NOT NULL,
  shared_with_partner_org_id uuid NOT NULL,
  granted_actions text[] NOT NULL,
  expires_at timestamptz NOT NULL,
  granted_by uuid NOT NULL,
  cedar_decision_id uuid NOT NULL,
  audit_id uuid NOT NULL,
  granted_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, resource_type, resource_id, shared_with_partner_org_id)
);
CREATE TABLE crm.deal_registration (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  partner_organization_id uuid NOT NULL,
  account_id uuid NOT NULL,
  lead_id uuid,
  opportunity_id uuid,
  amount_usd numeric(18,2),
  status crm.deal_registration_status NOT NULL,
  protection_until timestamptz NOT NULL,
  reviewer_principal_id uuid,
  audit_id uuid NOT NULL,
  submitted_at timestamptz NOT NULL DEFAULT now(),
  reviewed_at timestamptz
);
CREATE INDEX ix_crm_shared_binding_resource ON crm.shared_resource_binding(tenant_id, resource_type, resource_id);
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub partner_visible: bool, pub territory_id: TerritoryId }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub partner_visible: bool, pub email_hash: [u8; 32] }
pub struct Lead { pub id: LeadId, pub account_id: Option<AccountId>, pub shared_partner_id: Option<PartnerId>, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub channel_partner_id: Option<PartnerId>, pub stage: StageName }
pub struct Quote { pub id: QuoteId, pub opportunity_id: OpportunityId, pub partner_visible: bool, pub net_total: Money }
pub struct Order { pub id: OrderId, pub quote_id: QuoteId, pub partner_id: Option<PartnerId>, pub booked_amount: Money }
pub struct Contract { pub id: ContractId, pub partner_organization_id: Option<PartnerId>, pub rebate_terms: RebateTerms, pub effective_range: DateRange }
pub struct Case { pub id: CaseId, pub account_id: AccountId, pub partner_visible: bool, pub severity: Severity }
pub struct Campaign { pub id: CampaignId, pub partner_campaign: bool, pub status: CampaignStatus, pub budget: Money }
pub struct Solution { pub id: SolutionId, pub partner_visible: bool, pub case_id: Option<CaseId>, pub article_ref: String }
pub struct Forecast { pub id: ForecastId, pub partner_id: Option<PartnerId>, pub partner_pipeline_amount: Money, pub approved_by: Option<UserId> }
pub struct Territory { pub id: TerritoryId, pub partner_scope: PartnerScope, pub owner_id: UserId, pub capacity_units: i32 }
pub struct ChannelPartner { pub id: PartnerId, pub partner_tenant_id: TenantId, pub tier: PartnerTier, pub collar: WorkspaceCollar }
pub struct MarketingDocument { pub id: DocumentId, pub partner_visible: bool, pub campaign_id: CampaignId, pub storage_ref: String }
pub struct EmailTemplate { pub id: TemplateId, pub partner_visible: bool, pub locale: String, pub body_ref: String }
pub struct SharedResourceBinding { pub id: BindingId, pub resource_type: ResourceType, pub expires_at: DateTime, pub actions: Vec<ActionName> }
pub struct DealRegistration { pub id: DealRegistrationId, pub partner_id: PartnerId, pub protection_until: DateTime, pub status: DealRegistrationStatus }
```

## 3. API Endpoints
REST partner organization endpoint:
```http
POST /v1/crm/partner-orgs
```
REST partner body:
```json
{"tenant_id":"cobrahub-brazil","partner_tenant_id":"nortesul-distribuidora","partner_tier":"retired-advanced","contract_id":"ctr_partner_023","territory_scope":{"countries":["BR"],"industries":["industrial"]}}
```
REST lead share endpoint:
```http
POST /v1/crm/leads/lead_023/share
```
REST share body:
```json
{"partner_organization_id":"partner_023","expires_at":"2026-06-20T00:00:00Z","granted_actions":["read","update","convert"],"reason":"registered channel opportunity"}
```
REST deal registration response:
```json
{"deal_registration_id":"dr_023","status":"Submitted","protection_until":"2026-06-19T00:00:00Z","audit_id":"audit_023"}
```
gRPC contract:
```proto
service CrmPartnerChannelService {
  rpc OnboardPartnerOrganization(OnboardPartnerOrganizationRequest) returns (OnboardPartnerOrganizationResponse);
  rpc ShareCrmResource(ShareCrmResourceRequest) returns (ShareCrmResourceResponse);
  rpc SubmitDealRegistration(SubmitDealRegistrationRequest) returns (SubmitDealRegistrationResponse);
  rpc DecideDealRegistration(DecideDealRegistrationRequest) returns (DecideDealRegistrationResponse);
}
message ShareCrmResourceRequest { string tenant_id = 1; string resource_type = 2; string resource_id = 3; string partner_organization_id = 4; repeated string actions = 5; }
```
AsyncAPI message:
```yaml
crm.partner.channel.v1:
  publish:
    message:
      name: DealRegistrationSubmitted
      payload:
        deal_registration_id: dr_023
        partner_organization_id: partner_023
        protection_until: "2026-06-19T00:00:00Z"
```

## 4. Cedar Policy Hooks
Stage-advance gate:
```cedar
permit(principal, action == Action::"crm.stage.advance", resource)
when { resource.channel_partner_id == context.partner_id || principal in Role::"crm-channel-manager" };
```
Territory ownership:
```cedar
permit(principal in Role::"crm-channel-manager", action == Action::"crm.partner.resource.share", resource)
when { resource.territory_id in principal.visible_territory_ids && resource.tenant_id == principal.tenant_id };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"crm-forecast-approver", action == Action::"crm.forecast.rollup.approve", resource)
when { context.partner_pipeline_amount <= principal.partner_forecast_ceiling_usd && context.partner_id != "" };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.partner.resource.read", resource)
when { has_active_shared_binding(principal.id, resource.id) && context.now < context.binding_expires_at && principal.collar == "external-front-office" };
```
Back-office forbid:
```cedar
forbid(principal, action == Action::"crm.backoffice.any", resource)
when { principal.collar == "external-front-office" };
```

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile` with partner-safe account flags.
Salesforce Contact maps to `Oyatie::Customer.primary_contact` only when binding permits contact visibility.
Salesforce Case maps to `Oyatie::Customer.service_posture` as partner-safe summary.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture` with channel partner and registration protection.
Field delta: Salesforce `AccountPartner` becomes ChannelPartner with external tenant id.
Field delta: Salesforce partner licensed User becomes PartnerPrincipalBinding with collar.
Field delta: Salesforce share tables become expiring SharedResourceBinding.
Field delta: Salesforce Deal Registration custom object becomes native DealRegistration.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.partner_channel","customer_id":"cust_cobra","field_deltas":["account_profile.partner_visibility","revenue_posture.deal_registration","service_posture.partner_summary"]}
```

## 6. Workflow Steps
Node `validate_partner_contract` verifies partner contract lifecycle.
Node `create_partner_organization` writes partner org.
Node `invite_partner_principal` creates binding in Invited state.
Node `activate_partner_principal` binds external front-office collar.
Node `cedar_share_gate` evaluates resource share authority.
Node `insert_shared_resource_binding` writes explicit expiring binding.
Node `publish_resource_shared` notifies partner portal and ontology.
Node `submit_deal_registration` records proposed deal.
Decision `protection_window_conflict` returns 409 with existing deal id.
Node `channel_manager_review` approves or rejects registration.
Node `open_partner_pipeline_view` publishes partner-safe Opportunity and Quote data.
Node `settlement_handoff` sends won partner registration to marketplace.
Node `rebate_contract_update` updates contract lifecycle when needed.

## 7. Audit Events
ADR-0263 registry class `CrmPartnerOrganizationOnboarded`.
ADR-0263 registry class `CrmPartnerPrincipalInvited`.
ADR-0263 registry class `CrmPartnerPrincipalActivated`.
ADR-0263 registry class `CrmPartnerResourceShared`.
ADR-0263 registry class `CrmPartnerResourceShareExpired`.
ADR-0263 registry class `CrmDealRegistrationSubmitted`.
ADR-0263 registry class `CrmDealRegistrationApproved`.
ADR-0263 registry class `CrmDealRegistrationRejected`.
ADR-0263 registry class `CrmDealRegistrationProtectionBlocked`.
ADR-0263 registry class `CrmPartnerWorkspaceCollarDenied`.

## 8. SLO Targets
p50 partner lead-share latency is 80 ms.
p95 partner lead-share latency is 250 ms with Cedar evaluation and audit outbox.
p99 partner lead-share latency is 700 ms under cross-tenant identity lookup.
p50 deal-registration submit latency is 120 ms.
p95 deal-registration submit latency is 350 ms including protection-window check.
p99 partner pipeline read latency is 500 ms for 2000 visible opportunities.
Rationale: partner selling is interactive, but correctness hinges on explicit binding and protection-window conflict detection.
Revocation propagation target is two seconds p95.

## 9. Failure Modes and Recovery
Salesforce Partner Community migration hits Bulk API 10K user/share ceiling; recovery chunks by partner organization and source user.
Governor-limit parity appears when legacy sharing rules were implicit; recovery emits explicit shared bindings and rejects ambiguous rules.
Partner tenant missing appears during onboarding; recovery creates a pending partner tenant bootstrap task.
Lead-share expires mid-edit; recovery denies the request and returns share-expired with extension workflow link.
Concurrent deal registration race appears on the same account; recovery uses protection conflict and returns the existing registration id.
Partner workspace collar bypass attempt appears; recovery denies back-office action and emits collar-denied audit.

## 10. Migration Notes
Salesforce Sales Cloud Partner Community accounts map to ChannelPartner.
Salesforce Service Cloud partner-visible cases map to partner-safe Case summaries.
Salesforce Marketing Cloud partner campaign content maps to MarketingDocument and EmailTemplate visibility.
Salesforce Industries channel programs map to partner tier and territory scope.
SAP CRM-CHM and SAP PRM partners map to PartnerOrganization.
SAP C4C partner roles map to partner bindings.
SAP Service Cloud partner ticket sharing maps to case shared resource bindings.
Microsoft Dynamics 365 CE partner portal records map to ChannelPartner.
Microsoft Dynamics 365 Customer Service Hub partner cases map to partner-safe cases.
Oracle Fusion CX, Sales Cloud, Service Cloud, and PRM maps to partner orgs and deal registration.
HubSpot Sales Hub partner deals map to DealRegistration.
HubSpot Service Hub shared tickets map to Case bindings.
Zendesk Sell partner pipelines map to partner Opportunity views.
Pipedrive partner pipelines map to DealRegistration.
Freshsales partner sales maps to partner org and opportunity fields.
ActiveCampaign partner nurture maps to Campaign and EmailTemplate visibility.

## 11. Cross-Service Handoffs
Marketplace receives won partner deal registrations for settlement and rebate payout.
Payments receives partner payout references after marketplace settlement, not directly from CRM.
Community receives partner-channel content eligibility.
Marketing-automation receives partner campaign audience and share feedback.
Intelligence receives partner conversion and protection-window data for scoring.
Identity manages external principals and collar binding.
Contract lifecycle owns partner contract and rebate terms.
Ontology receives partner-safe Customer projection deltas.
Compliance receives cross-tenant sharing evidence.
Audit-chain seals every partner share and registration decision.

## 12. Acceptance
Every partner-visible resource has an explicit expiring binding.
Partner principals are forced into the external-front-office collar.
Deal registration protection prevents concurrent duplicate partner claims.
Partner pipeline hides non-shared direct opportunities.
Marketplace gets won partner registration for settlement.
The IP is bespoke to partner-channel enablement and front-office collar behavior.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-023-partner-channel-enablement-front-office-collar.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/crm/IP-023-partner-channel-enablement-front-office-collar.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
