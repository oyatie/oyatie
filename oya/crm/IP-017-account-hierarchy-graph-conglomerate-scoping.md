---
doc_class: ImplementationPlan
ip_id: IP-017
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j112-tenant-to-tenant-rfq-and-bid::conglomerate-account-scope
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-tenancy + axis-ontology
---

# IP-017: Account hierarchy graph with conglomerate scoping

## 1. Context
This slice exists because enterprise CRM fails when a parent account can see too much or too little about subsidiaries.
The displaced SAP CRM submodule is SAP CRM Master Data business partner relationships.
The displaced Salesforce CRM submodule is Salesforce Sales Cloud Account Hierarchy and Account Teams.
The named persona is Daichi Watanabe, strategic accounts lead for Mizu Beverage Holdings.
The named journey leg is j112 tenant-to-tenant RFQ and bid scoping.
Daichi needs a roll-up view across fourteen subsidiaries while Korean and Japanese sovereign-child tenants keep PII restricted.
Salesforce `ParentId` is too weak for joint ventures, residency boundaries, and multi-tenant authorization.
SAP CRM `BUT050` relationship semantics are stronger, but they do not carry Oyatie Cedar and residency decision lineage.
This implementation makes Account hierarchy a typed DAG with closure projection and policy-scrubbed roll-ups.
It is the graph substrate for forecast roll-ups, territory assignment, partner visibility, and Customer 360 projection.

## 2. Data Model Deltas
PostgreSQL DDL:
```sql
CREATE TYPE crm.account_edge_type AS ENUM ('ACCOUNT_OF','DIVISION_OF','SITE_OF','JOINT_VENTURE_OF','BILL_TO_OF','SHIP_TO_OF');
CREATE TABLE crm.account_hierarchy_edge (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  parent_account_id uuid NOT NULL,
  child_account_id uuid NOT NULL,
  edge_type crm.account_edge_type NOT NULL,
  ownership_pct numeric(5,2),
  effective_from date NOT NULL DEFAULT current_date,
  effective_to date,
  residency_boundary text NOT NULL DEFAULT 'same-tenant',
  cedar_decision_id uuid NOT NULL,
  audit_id uuid NOT NULL,
  UNIQUE (parent_account_id, child_account_id, edge_type, effective_from)
);
CREATE TABLE crm.account_hierarchy_closure (
  tenant_id uuid NOT NULL,
  ancestor_account_id uuid NOT NULL,
  descendant_account_id uuid NOT NULL,
  depth int NOT NULL,
  path_signature text NOT NULL,
  sovereign_boundary_count int NOT NULL DEFAULT 0,
  PRIMARY KEY (tenant_id, ancestor_account_id, descendant_account_id, path_signature)
);
ALTER TABLE crm.account ADD COLUMN account_type text NOT NULL DEFAULT 'Customer';
ALTER TABLE crm.account ADD COLUMN conglomerate_root_account_id uuid;
ALTER TABLE crm.account ADD COLUMN tenant_residency_pack text NOT NULL DEFAULT 'global-default';
CREATE INDEX ix_crm_account_closure_ancestor ON crm.account_hierarchy_closure(tenant_id, ancestor_account_id, depth);
CREATE INDEX ix_crm_account_closure_descendant ON crm.account_hierarchy_closure(tenant_id, descendant_account_id, depth);
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub legal_name: String, pub account_type: AccountType }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub role: ContactRole, pub pii_residency_pack: ResidencyPack }
pub struct Lead { pub id: LeadId, pub matched_account_id: Option<AccountId>, pub source: LeadSource, pub score: i32 }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub rollup_root_id: AccountId, pub stage: StageName }
pub struct Quote { pub id: QuoteId, pub account_id: AccountId, pub bill_to_account_id: AccountId, pub ship_to_account_id: AccountId }
pub struct Order { pub id: OrderId, pub account_id: AccountId, pub fulfillment_account_id: AccountId, pub status: OrderStatus }
pub struct Contract { pub id: ContractId, pub account_id: AccountId, pub party_scope: PartyScope, pub effective_range: DateRange }
pub struct Case { pub id: CaseId, pub account_id: AccountId, pub entitlement_account_id: AccountId, pub severity: Severity }
pub struct Campaign { pub id: CampaignId, pub audience_account_root_id: AccountId, pub status: CampaignStatus, pub budget: Money }
pub struct Solution { pub id: SolutionId, pub account_scope: AccountScope, pub case_id: Option<CaseId>, pub article_ref: String }
pub struct Forecast { pub id: ForecastId, pub root_account_id: AccountId, pub pipeline_amount: Money, pub scrubbed_descendants: u32 }
pub struct Territory { pub id: TerritoryId, pub account_root_id: AccountId, pub owner_id: UserId, pub residency_packs: Vec<ResidencyPack> }
pub struct ChannelPartner { pub id: PartnerId, pub account_id: AccountId, pub managed_account_roots: Vec<AccountId>, pub visibility: VisibilityTier }
pub struct MarketingDocument { pub id: DocumentId, pub account_scope: AccountScope, pub campaign_id: CampaignId, pub storage_ref: String }
pub struct EmailTemplate { pub id: TemplateId, pub account_scope: AccountScope, pub locale: String, pub consent_purpose: ConsentPurpose }
pub struct AccountHierarchyEdge { pub id: EdgeId, pub parent: AccountId, pub child: AccountId, pub edge_type: AccountEdgeType }
```

## 3. API Endpoints
REST link endpoint:
```http
POST /v1/crm/accounts/acc_child/parents
```
REST request body:
```json
{"tenant_id":"mizu-holdings","parent_account_id":"acc_parent","edge_type":"JOINT_VENTURE_OF","ownership_pct":"50.00","effective_from":"2026-05-20","reason":"Mizu Korea joint venture mapping"}
```
REST response body:
```json
{"edge_id":"edge_017","closure_rows_added":8,"sovereign_boundary_count":1,"cedar_decision_id":"cedar_017","audit_id":"audit_017"}
```
REST roll-up endpoint:
```http
GET /v1/crm/accounts/acc_parent/descendants?include_pipeline=true&max_depth=8
```
Roll-up response:
```json
{"account_id":"acc_parent","descendant_count":14,"pipeline_amount":{"currency":"USD","value":"9800000.00"},"scrubbed_count":2,"scrub_reason":"sovereign-child-deny"}
```
gRPC contract:
```proto
service CrmAccountHierarchyService {
  rpc LinkAccountHierarchy(LinkAccountHierarchyRequest) returns (LinkAccountHierarchyResponse);
  rpc CloseAccountHierarchyEdge(CloseAccountHierarchyEdgeRequest) returns (CloseAccountHierarchyEdgeResponse);
  rpc GetAccountRollup(GetAccountRollupRequest) returns (GetAccountRollupResponse);
}
message LinkAccountHierarchyRequest { string tenant_id = 1; string parent_account_id = 2; string child_account_id = 3; string edge_type = 4; }
```
AsyncAPI message:
```yaml
crm.account.hierarchy.v1:
  publish:
    message:
      name: AccountHierarchyChanged
      payload:
        parent_account_id: acc_parent
        child_account_id: acc_child
        path_signature: ACCOUNT_OF/JOINT_VENTURE_OF
```

## 4. Cedar Policy Hooks
Stage-advance gate:
```cedar
permit(principal, action == Action::"crm.stage.advance", resource)
when { context.account_scope.allowed == true && resource.account_id in context.visible_account_ids };
```
Territory ownership:
```cedar
permit(principal in Role::"crm-territory-owner", action == Action::"crm.account.rollup.read", resource)
when { resource.territory_id == principal.territory_id && context.sovereign_boundary_count == 0 };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"crm-forecast-approver", action == Action::"crm.forecast.rollup.approve", resource)
when { context.root_account_id == resource.account_id && context.scrubbed_count == 0 };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.account.partner.read", resource)
when { context.partner_id in resource.authorized_partner_ids && context.detail_level == "scrubbed" };
```
Cross-tenant reads default to deny when `sovereign_boundary_count` is non-zero.
Aggregate-only responses must include `scrubbed_count` and cannot silently omit denied descendants.

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile`.
Salesforce Contact maps to `Oyatie::Customer.primary_contact` and visible contact roles.
Salesforce Case maps to `Oyatie::Customer.service_posture` at the account hierarchy leaf.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture` at the closest allowed ancestor.
Field delta: Salesforce `ParentId` becomes multi-edge `AccountHierarchyEdge`.
Field delta: Salesforce `AccountTeamMember` becomes Cedar role binding plus territory ownership.
Field delta: Salesforce `AccountContactRelation` becomes tenant-scoped contact binding.
Field delta: Salesforce roll-up summaries become policy-scrubbed aggregates.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.account_hierarchy","customer_id":"cust_mizu","field_deltas":["account_profile.hierarchy","revenue_posture.rollup_root","service_posture.scrubbed_count"]}
```

## 6. Workflow Steps
Node `validate_edge_payload` rejects self-links and expired dates.
Node `load_parent_child_accounts` fetches residency packs.
Node `cycle_check` runs recursive ancestor detection.
Decision `cycle_detected` returns conflict and emits a rejection audit class.
Node `cedar_link_gate` checks same-tenant or cross-tenant authority.
Node `insert_edge` writes the edge with Cedar decision id.
Node `recompute_closure` materializes affected ancestor and descendant paths.
Node `publish_hierarchy_changed` notifies ontology and forecast.
Decision `sovereign_boundary_crossed` creates compliance evidence.
Node `refresh_rollup_cache` recomputes aggregate pipeline with scrub count.
Node `emit_audit_bundle` seals ADR-0263 events.
Node `complete_workflow` returns edge and closure metadata.

## 7. Audit Events
ADR-0263 registry class `CrmAccountHierarchyLinkRequested`.
ADR-0263 registry class `CrmAccountHierarchyLinkCreated`.
ADR-0263 registry class `CrmAccountHierarchyCycleRejected`.
ADR-0263 registry class `CrmAccountHierarchyCrossTenantDenied`.
ADR-0263 registry class `CrmAccountHierarchyCrossTenantLinked`.
ADR-0263 registry class `CrmAccountHierarchyEdgeClosed`.
ADR-0263 registry class `CrmAccountHierarchyClosureRebuilt`.
ADR-0263 registry class `CrmAccountRollupReadScrubbed`.
ADR-0263 registry class `CrmPartnerAccountVisibilityGranted`.
ADR-0263 registry class `CrmPartnerAccountVisibilityDenied`.

## 8. SLO Targets
p50 edge link latency is 70 ms for same-tenant links.
p95 edge link latency is 220 ms when closure recomputes fewer than 500 rows.
p99 edge link latency is 900 ms when cross-tenant Cedar and compliance evidence are required.
p50 hierarchy read latency is 25 ms from closure table.
p95 hierarchy read latency is 85 ms with pipeline roll-up cache.
p99 hierarchy read latency is 300 ms for thousand-account conglomerates.
Rationale: Account hierarchy is a hot-path CRM read; closure table cost is accepted to avoid runtime recursive traversal in every view.

## 9. Failure Modes and Recovery
Salesforce-style single ParentId collision appears when an account has two legitimate parents; recovery maps both to typed edges and marks source conflict.
Bulk API 10K hierarchy batch ceiling appears during import; recovery chunks edges by parent root and replays closure after each chunk.
Governor-limit parity failure appears when source org triggers recursive flows; recovery imports data only, not flow side effects.
Cycle attempt appears when a subsidiary is linked back to an ancestor; recovery rejects with proof path.
Sovereign boundary violation appears on roll-up read; recovery returns aggregate with scrubbed count and no PII.
Closure recompute timeout appears for huge roots; recovery accepts the edge, queues rebuild, and temporarily serves direct-edge traversal.

## 10. Migration Notes
Salesforce Sales Cloud Account Hierarchy maps to edge and closure rows.
Salesforce Service Cloud AccountContactRelation maps to contact bindings under the hierarchy.
Salesforce Marketing Cloud business units map to campaign audience account scopes.
Salesforce Industries household and business account models map to typed account roles.
SAP CRM business partner relationships map from BUT050 and BUT051.
SAP C4C account hierarchy maps through external account ids.
SAP Service Cloud installed-base account references map to Case entitlement account.
Microsoft Dynamics 365 CE parentaccountid maps to edge rows.
Microsoft Dynamics 365 Customer Service Hub account-contact links map to Contact bindings.
Oracle Fusion CX, Sales Cloud, and Service Cloud account trees map to closure rows.
HubSpot Sales Hub parent company associations map to account edges.
HubSpot Service Hub ticket company links map to Case account ids.
Zendesk Sell company hierarchy maps to Account relationships.
Pipedrive organization relationships map to edges.
Freshsales account hierarchy maps to root account fields.
ActiveCampaign account custom fields map to audience account scope.

## 11. Cross-Service Handoffs
Marketplace receives conglomerate account root for deal settlement grouping.
Payments receives bill-to and ship-to account lineage for invoice routing.
Community receives scrubbed partner account scopes for partner-channel content.
Marketing-automation receives allowed audience account roots for campaigns.
Intelligence receives hierarchy graph features for scoring and churn models.
Ontology receives closure projection deltas.
Compliance receives cross-tenant and sovereign-boundary audit events.
Forecast receives roll-up root and scrubbed descendant counts.
Audit-chain seals every hierarchy mutation.

## 12. Acceptance
Cycles are rejected at write time.
Joint venture multi-parent relationships are supported without overwriting another parent.
Cross-tenant reads carry explicit scrubbed counts.
Every hierarchy link has Cedar and audit identifiers.
The closure table supports account, contact, case, opportunity, forecast, and partner visibility reads.
The IP is bespoke to conglomerate account scoping rather than a generic account CRUD plan.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-017-account-hierarchy-graph-conglomerate-scoping.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-017-account-hierarchy-graph-conglomerate-scoping.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-017-account-hierarchy-graph-conglomerate-scoping.md` matched [`cost`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-017-account-hierarchy-graph-conglomerate-scoping.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
