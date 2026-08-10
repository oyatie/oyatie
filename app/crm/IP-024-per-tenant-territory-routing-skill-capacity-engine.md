---
doc_class: ImplementationPlan
ip_id: IP-024
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j112-tenant-to-tenant-rfq-and-bid::territory-routing-and-capacity-assignment
capability_profile: T2-product-erp-parity
status: Proposed
date: 2026-05-20
owner_team: axis-crm + axis-routing + axis-identity
---

# IP-024: Per-tenant territory routing with skill matrix and capacity awareness

## 1. Context
This net-new slice exists because territory assignment cannot be a static postal-code table once CRM serves sellers, service agents, and partners in the same tenant.
The displaced SAP CRM submodule is SAP CRM Organizational Management and Territory Management.
The displaced Salesforce CRM submodule is Salesforce Enterprise Territory Management plus Omni-Channel skill-based routing.
The named persona is Camila Rocha, revenue operations manager at Cobrahub Industrial.
The named journey leg is j112 territory-aware tenant-to-tenant RFQ routing.
Camila needs inbound leads, partner registrations, opportunities, cases, and campaign responses routed to the right owner by tenant, territory, skill, language, segment, and current load.
Salesforce Enterprise Territory Management handles assignment but does not consistently model capacity and skill as first-class deterministic routing inputs.
Salesforce Omni-Channel models capacity for support, but not the whole front-office CRM aggregate set.
SAP territory management handles sales structure, but cross-tenant partner visibility and per-cell capacity require Oyatie-native policy hooks.
This implementation creates a routing decision record for every assignment and makes capacity auditable.

## 2. Data Model Deltas
PostgreSQL DDL:
```sql
CREATE TYPE crm.routing_subject_type AS ENUM ('Account','Contact','Lead','Opportunity','Quote','Order','Contract','Case','Campaign','Solution','Forecast','ChannelPartner','MarketingDocument','EmailTemplate');
CREATE TYPE crm.routing_decision_status AS ENUM ('Assigned','Deferred','Rejected','Escalated','ManualOverride');
CREATE TABLE crm.territory_skill_matrix (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  territory_id uuid NOT NULL,
  principal_id uuid NOT NULL,
  skill_code text NOT NULL,
  skill_level int NOT NULL CHECK (skill_level BETWEEN 1 AND 5),
  language_code text,
  industry_code text,
  effective_from timestamptz NOT NULL,
  effective_to timestamptz,
  audit_id uuid NOT NULL,
  UNIQUE (tenant_id, territory_id, principal_id, skill_code, effective_from)
);
CREATE TABLE crm.territory_capacity_window (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  territory_id uuid NOT NULL,
  principal_id uuid NOT NULL,
  window_start timestamptz NOT NULL,
  window_end timestamptz NOT NULL,
  capacity_units_total int NOT NULL,
  capacity_units_reserved int NOT NULL DEFAULT 0,
  capacity_units_available int NOT NULL,
  source text NOT NULL CHECK (source IN ('Calendar','Presence','Manual','OnCall','PartnerCommitment')),
  audit_id uuid NOT NULL
);
CREATE TABLE crm.routing_decision (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  subject_type crm.routing_subject_type NOT NULL,
  subject_id uuid NOT NULL,
  territory_id uuid NOT NULL,
  assigned_principal_id uuid,
  decision_status crm.routing_decision_status NOT NULL,
  required_skills text[] NOT NULL,
  capacity_units_reserved int NOT NULL DEFAULT 0,
  routing_score numeric(9,4) NOT NULL,
  cedar_decision_id uuid NOT NULL,
  audit_id uuid NOT NULL,
  rationale jsonb NOT NULL,
  decided_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE crm.territory ADD COLUMN routing_policy jsonb NOT NULL DEFAULT '{}'::jsonb;
ALTER TABLE crm.territory ADD COLUMN default_capacity_units int NOT NULL DEFAULT 10;
CREATE INDEX ix_crm_routing_subject ON crm.routing_decision(tenant_id, subject_type, subject_id, decided_at DESC);
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub territory_id: TerritoryId, pub industry: String }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub language_code: String, pub preferred_channel: Channel }
pub struct Lead { pub id: LeadId, pub account_id: Option<AccountId>, pub required_skills: Vec<SkillCode>, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub territory_id: TerritoryId, pub stage: StageName }
pub struct Quote { pub id: QuoteId, pub opportunity_id: OpportunityId, pub approval_skill: Option<SkillCode>, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub account_id: AccountId, pub routing_owner_id: UserId, pub status: OrderStatus }
pub struct Contract { pub id: ContractId, pub account_id: AccountId, pub renewal_owner_skill: SkillCode, pub effective_range: DateRange }
pub struct Case { pub id: CaseId, pub account_id: AccountId, pub required_skill: SkillCode, pub priority: CasePriority }
pub struct Campaign { pub id: CampaignId, pub territory_id: TerritoryId, pub response_routing_skill: SkillCode, pub status: CampaignStatus }
pub struct Solution { pub id: SolutionId, pub skill_code: SkillCode, pub case_id: Option<CaseId>, pub article_ref: String }
pub struct Forecast { pub id: ForecastId, pub territory_id: TerritoryId, pub capacity_adjusted_amount: Money, pub approved_by: Option<UserId> }
pub struct Territory { pub id: TerritoryId, pub parent_id: Option<TerritoryId>, pub routing_policy: RoutingPolicy, pub default_capacity_units: i32 }
pub struct ChannelPartner { pub id: PartnerId, pub territory_scope: PartnerTerritoryScope, pub capacity_commitment: CapacityCommitment, pub visibility: VisibilityTier }
pub struct MarketingDocument { pub id: DocumentId, pub territory_id: TerritoryId, pub skill_target: Option<SkillCode>, pub storage_ref: String }
pub struct EmailTemplate { pub id: TemplateId, pub territory_id: TerritoryId, pub language_code: String, pub body_ref: String }
pub struct TerritorySkillMatrix { pub id: SkillMatrixId, pub principal_id: UserId, pub skill_code: SkillCode, pub skill_level: u8 }
pub struct RoutingDecision { pub id: RoutingDecisionId, pub subject_type: RoutingSubjectType, pub assigned_principal_id: Option<UserId>, pub rationale: RoutingRationale }
```

## 3. API Endpoints
REST skill matrix endpoint:
```http
POST /v1/crm/territories/terr_brazil/skills
```
REST skill body:
```json
{"tenant_id":"cobrahub-brazil","principal_id":"usr_camila_rep_7","skill_code":"industrial-rfq-portuguese","skill_level":4,"language_code":"pt-BR","industry_code":"industrial","effective_from":"2026-05-20T00:00:00Z"}
```
REST capacity endpoint:
```http
POST /v1/crm/territories/terr_brazil/capacity-windows
```
REST capacity body:
```json
{"principal_id":"usr_camila_rep_7","window_start":"2026-05-20T13:00:00Z","window_end":"2026-05-20T21:00:00Z","capacity_units_total":12,"source":"Presence"}
```
REST routing endpoint:
```http
POST /v1/crm/routing/decisions
```
REST routing body:
```json
{"tenant_id":"cobrahub-brazil","subject_type":"Lead","subject_id":"lead_024","territory_id":"terr_brazil","required_skills":["industrial-rfq-portuguese","partner-channel"],"capacity_units_required":2}
```
REST routing response:
```json
{"routing_decision_id":"rd_024","decision_status":"Assigned","assigned_principal_id":"usr_camila_rep_7","routing_score":"94.5000","cedar_decision_id":"cedar_024","audit_id":"audit_024"}
```
gRPC contract:
```proto
service CrmTerritoryRoutingService {
  rpc UpsertTerritorySkill(UpsertTerritorySkillRequest) returns (UpsertTerritorySkillResponse);
  rpc PublishCapacityWindow(PublishCapacityWindowRequest) returns (PublishCapacityWindowResponse);
  rpc RouteCrmSubject(RouteCrmSubjectRequest) returns (RouteCrmSubjectResponse);
  rpc RebalanceTerritoryLoad(RebalanceTerritoryLoadRequest) returns (RebalanceTerritoryLoadResponse);
}
message RouteCrmSubjectRequest { string tenant_id = 1; string subject_type = 2; string subject_id = 3; string territory_id = 4; repeated string required_skills = 5; int32 capacity_units_required = 6; }
```
AsyncAPI message:
```yaml
crm.routing.decision.v1:
  publish:
    message:
      name: CrmSubjectRouted
      payload:
        subject_type: Lead
        subject_id: lead_024
        assigned_principal_id: usr_camila_rep_7
```

## 4. Cedar Policy Hooks
Stage-advance gate:
```cedar
permit(principal, action == Action::"crm.stage.advance", resource)
when { resource.assigned_principal_id == principal.id && context.routing_decision_status == "Assigned" };
```
Territory ownership:
```cedar
permit(principal in Role::"crm-territory-router", action == Action::"crm.routing.assign", resource)
when { resource.territory_id in principal.managed_territory_ids && resource.tenant_id == principal.tenant_id };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"crm-forecast-approver", action == Action::"crm.forecast.rollup.approve", resource)
when { context.capacity_adjustment_explained == true && context.routing_decision_count > 0 };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.routing.partner_status.read", resource)
when { context.partner_id == resource.channel_partner_id && context.payload_class == "status_only" };
```
Capacity reservation:
```cedar
forbid(principal, action == Action::"crm.routing.assign", resource)
when { context.capacity_units_required > context.capacity_units_available && !(principal in Role::"crm-routing-override") };
```

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile` with routing territory.
Salesforce Contact maps to `Oyatie::Customer.primary_contact` with language and channel preference.
Salesforce Case maps to `Oyatie::Customer.service_posture` with assigned skill queue.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture` with assigned territory owner.
Field delta: Salesforce Enterprise Territory rules become typed Territory plus RoutingDecision.
Field delta: Salesforce Omni-Channel capacity becomes `territory_capacity_window`.
Field delta: Salesforce skills become `territory_skill_matrix`.
Field delta: Salesforce assignment history becomes audit-linked routing decisions.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.territory_routing","customer_id":"cust_024","field_deltas":["account_profile.territory","service_posture.assigned_skill","revenue_posture.routing_owner"]}
```

## 6. Workflow Steps
Node `load_subject` reads the subject aggregate by type.
Node `derive_required_skills` uses subject, Account, Campaign, Case, and partner context.
Node `load_territory_policy` reads tenant-specific routing policy.
Node `load_skill_matrix` finds eligible principals.
Node `load_capacity_windows` filters principals with available capacity.
Decision `no_capacity_available` defers or escalates to override role.
Node `score_candidates` ranks by skill level, account history, language, partner scope, and load.
Node `cedar_assignment_gate` evaluates territory and capacity authority.
Node `reserve_capacity` decrements availability atomically.
Node `write_routing_decision` stores rationale and audit ids.
Node `publish_assignment` emits AsyncAPI.
Node `update_customer_projection` sends routing delta to ontology.
Decision `partner_subject` publishes status-only partner portal update.
Node `rebalance_if_over_capacity` queues load rebalance when needed.

## 7. Audit Events
ADR-0263 registry class `CrmTerritorySkillUpserted`.
ADR-0263 registry class `CrmTerritoryCapacityWindowPublished`.
ADR-0263 registry class `CrmRoutingDecisionRequested`.
ADR-0263 registry class `CrmRoutingDecisionAssigned`.
ADR-0263 registry class `CrmRoutingDecisionDeferredNoCapacity`.
ADR-0263 registry class `CrmRoutingDecisionRejectedPolicy`.
ADR-0263 registry class `CrmRoutingDecisionManualOverride`.
ADR-0263 registry class `CrmRoutingCapacityReserved`.
ADR-0263 registry class `CrmRoutingCapacityReleased`.
ADR-0263 registry class `CrmRoutingPartnerStatusPublished`.

## 8. SLO Targets
p50 routing decision latency is 45 ms for warm skill and capacity caches.
p95 routing decision latency is 180 ms for territories with up to 1000 eligible principals.
p99 routing decision latency is 700 ms when identity presence and partner scope require remote reads.
p50 skill matrix upsert latency is 40 ms.
p95 capacity reservation latency is 120 ms.
p99 rebalance job completion is 5000 ms for 10,000 active subjects.
Rationale: assignment is an interactive path for leads and cases, but rebalancing can run asynchronously.
Correctness target is zero over-capacity assignments unless an override audit event exists.

## 9. Failure Modes and Recovery
Salesforce Enterprise Territory import hits Bulk API 10K assignment-rule ceiling; recovery chunks by territory and source rule id.
Salesforce Omni-Channel governor limits hide skill-capacity behavior; recovery stores source routing outcome and models explicit skill rows.
Capacity window stale after presence outage; recovery switches to default capacity with high-risk audit flag.
Concurrent routing race reserves the last capacity unit twice; recovery uses atomic update and one request receives deferred status.
Partner territory scope mismatch appears on shared lead; recovery denies partner route and emits partner status rejected.
Lead conversion conflict appears after rerouting; recovery preserves original routing decision and writes manual override if changed.

## 10. Migration Notes
Salesforce Sales Cloud Enterprise Territory Management maps to Territory and routing policy.
Salesforce Service Cloud Omni-Channel skills and capacity map to skill matrix and capacity windows.
Salesforce Marketing Cloud lead assignment automations map to routing decisions.
Salesforce Industries industry-specific territories map to territory policy extensions.
SAP CRM Organizational Management maps to Territory.
SAP CRM Territory Management maps to territory hierarchy and capacity defaults.
SAP C4C sales territory rules map to routing policy.
SAP Service Cloud routing rules map to Case skill routing.
Microsoft Dynamics 365 CE assignment rules map to RoutingDecision.
Microsoft Dynamics 365 Customer Service Hub unified routing maps to capacity windows.
Oracle Fusion CX, Sales Cloud, and Service Cloud assignment rules map to routing policy and skill matrix.
HubSpot Sales Hub round-robin routing maps to simple policy mode.
HubSpot Service Hub ticket routing maps to Case required skill.
Zendesk Sell lead distribution maps to Lead routing decisions.
Pipedrive owner assignment maps to Opportunity routing decisions.
Freshsales territory assignment maps to Territory.
ActiveCampaign assignment automations map to routing workflow triggers.

## 11. Cross-Service Handoffs
Marketplace receives territory assignment for deal settlement ownership.
Payments receives assigned account owner context for invoice escalation.
Community receives partner-safe routing status for partner-channel content.
Marketing-automation receives lead owner and campaign response owner updates.
Intelligence receives routing features and outcome labels for assignment optimization.
Identity provides skills, presence, roles, and delegation.
Contact-center provides service capacity when subject type is Case.
Ontology receives Customer assignment deltas.
Forecast receives capacity-adjusted territory pipeline.
Audit-chain seals every assignment and capacity override.

## 12. Acceptance
Routing decisions are persisted for every routed CRM subject.
Capacity cannot be over-reserved without an override audit event.
Skills, language, industry, territory, and partner scope influence candidate scoring.
Partner portal only sees status-only routing payloads.
Forecast receives capacity-adjusted territory inputs.
The IP is net-new and does not duplicate the existing 23 CRM IP topics.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-024-per-tenant-territory-routing-skill-capacity-engine.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`EU-AI-ACT-2024-HIGH-RISK`, `SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-024-per-tenant-territory-routing-skill-capacity-engine.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
