---
doc_class: ImplementationPlan
ip_id: IP-022
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0210, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0319]
journey_ref: j124-supply-chain-disruption-emergency-coordination::p1-case-escalation
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-contact-center + axis-service
---

# IP-022: Service-case SLA and escalation engine

## 1. Context
This slice exists because support case handling must be contract-aware, time-aware, and audit-reconstructable during incidents.
The displaced SAP CRM submodule is SAP CRM Service and Interaction Center service ticket processing.
The displaced Salesforce CRM submodule is Salesforce Service Cloud Case, Entitlements, Milestones, and Omni-Channel Routing.
The named persona is Olufemi Adeyemi, tier-1 support lead at HarborCom Logistics.
The named journey leg is j124 supply-chain disruption emergency coordination.
Olufemi receives a P1 case during a typhoon and must route it to the right on-call group with a 30 minute response milestone.
Salesforce Service Cloud is strong but often spreads SLA logic across entitlement rules, flows, and milestone automation.
SAP CRM Service has ticket rigor but does not carry Oyatie tenant, Cedar, and audit-chain semantics.
This implementation makes the service case the legal record, with SLA milestones as explicit rows and escalation as data.
It connects service cases to Account, Contract, Solution, Customer 360, partner portal, and incident coordination.

## 2. Data Model Deltas
PostgreSQL DDL:
```sql
CREATE TYPE crm.case_status AS ENUM ('New','InTriage','InProgress','WaitingOnCustomer','WaitingOnInternal','Escalated','Resolved','Closed','Reopened');
CREATE TYPE crm.case_priority AS ENUM ('P1','P2','P3','P4');
CREATE TABLE crm.service_case_milestone (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  case_id uuid NOT NULL,
  milestone_type text NOT NULL CHECK (milestone_type IN ('ResponseTime','UpdateInterval','RestoreTime','ResolveTime')),
  target_at timestamptz NOT NULL,
  paused_total_seconds int NOT NULL DEFAULT 0,
  completed_at timestamptz,
  outcome text CHECK (outcome IN ('Met','Missed','Waived','InFlight')),
  warning_50pct_emitted_at timestamptz,
  warning_80pct_emitted_at timestamptz,
  breach_emitted_at timestamptz,
  audit_id uuid
);
CREATE TABLE crm.service_case_activity (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  case_id uuid NOT NULL,
  actor_principal_id uuid NOT NULL,
  activity_type text NOT NULL,
  payload jsonb NOT NULL,
  audit_id uuid NOT NULL,
  occurred_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE crm.case ADD COLUMN priority crm.case_priority NOT NULL DEFAULT 'P3';
ALTER TABLE crm.case ADD COLUMN status crm.case_status NOT NULL DEFAULT 'New';
ALTER TABLE crm.case ADD COLUMN entitlement_contract_id uuid;
ALTER TABLE crm.case ADD COLUMN escalation_plan jsonb NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE crm.case ADD COLUMN cedar_decision_id_at_create uuid;
CREATE INDEX ix_crm_case_milestone_target ON crm.service_case_milestone(tenant_id, target_at) WHERE completed_at IS NULL;
```
Rust types:
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub support_tier: SupportTier, pub territory_id: TerritoryId }
pub struct Contact { pub id: ContactId, pub account_id: AccountId, pub preferred_channel: Channel, pub consent_state: ConsentState }
pub struct Lead { pub id: LeadId, pub account_id: Option<AccountId>, pub open_case_count: u32, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub account_id: AccountId, pub stage: StageName, pub support_risk: SupportRisk }
pub struct Quote { pub id: QuoteId, pub opportunity_id: OpportunityId, pub case_blocker_id: Option<CaseId>, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub account_id: AccountId, pub case_blocker_id: Option<CaseId>, pub status: OrderStatus }
pub struct Contract { pub id: ContractId, pub account_id: AccountId, pub support_plan: SupportPlan, pub entitlement_terms: EntitlementTerms }
pub struct Case { pub id: CaseId, pub account_id: AccountId, pub priority: CasePriority, pub status: CaseStatus }
pub struct Campaign { pub id: CampaignId, pub service_notification_scope: ServiceAudience, pub status: CampaignStatus, pub budget: Money }
pub struct Solution { pub id: SolutionId, pub case_id: CaseId, pub article_ref: String, pub resolution_confidence: Decimal }
pub struct Forecast { pub id: ForecastId, pub account_id: AccountId, pub service_risk_adjustment: Money, pub approved_by: Option<UserId> }
pub struct Territory { pub id: TerritoryId, pub support_queue_id: QueueId, pub owner_id: UserId, pub capacity_units: i32 }
pub struct ChannelPartner { pub id: PartnerId, pub support_scope: SupportScope, pub account_id: AccountId, pub visibility: VisibilityTier }
pub struct MarketingDocument { pub id: DocumentId, pub incident_campaign_id: Option<CampaignId>, pub storage_ref: String, pub lifecycle: DocumentLifecycle }
pub struct EmailTemplate { pub id: TemplateId, pub case_status: CaseStatus, pub locale: String, pub body_ref: String }
pub struct ServiceCaseMilestone { pub id: MilestoneId, pub case_id: CaseId, pub target_at: DateTime, pub outcome: MilestoneOutcome }
pub struct ServiceCaseActivity { pub id: ActivityId, pub case_id: CaseId, pub activity_type: ActivityType, pub audit_id: AuditId }
```

## 3. API Endpoints
REST create case endpoint:
```http
POST /v1/crm/cases
```
REST create body:
```json
{"tenant_id":"harborcom-nigeria","account_id":"acc_maersk_lagos","contact_id":"con_ops","channel":"API","priority":"P1","subject":"Port outage during typhoon","body":"Crane control telemetry unavailable","source_external_id":"cc_124_884"}
```
REST create response:
```json
{"case_id":"case_022","case_number":"CRM-2026-000022","entitlement_contract_id":"ctr_retired-sovereign","milestones":[{"type":"ResponseTime","target_at":"2026-05-20T03:17:00Z"},{"type":"RestoreTime","target_at":"2026-05-20T06:47:00Z"}],"audit_id":"audit_022"}
```
REST escalation endpoint:
```http
POST /v1/crm/cases/case_022/escalate
```
Escalation body:
```json
{"reason":"80_percent_sla_burn","target_step":2,"include_partner_visibility":true}
```
gRPC contract:
```proto
service CrmServiceCaseService {
  rpc CreateCase(CreateCaseRequest) returns (CreateCaseResponse);
  rpc AppendCaseActivity(AppendCaseActivityRequest) returns (AppendCaseActivityResponse);
  rpc ChangeCaseStatus(ChangeCaseStatusRequest) returns (ChangeCaseStatusResponse);
  rpc EscalateCase(EscalateCaseRequest) returns (EscalateCaseResponse);
}
message CreateCaseRequest { string tenant_id = 1; string account_id = 2; string priority = 3; string channel = 4; string subject = 5; }
```
AsyncAPI message:
```yaml
crm.case.lifecycle.v1:
  publish:
    message:
      name: CaseSlaWarning80
      payload:
        case_id: case_022
        milestone_type: RestoreTime
        target_at: "2026-05-20T06:47:00Z"
```

## 4. Cedar Policy Hooks
Stage-advance gate:
```cedar
forbid(principal, action == Action::"crm.stage.advance", resource)
when { context.open_p1_case_count > 0 && context.to_stage == "ClosedWon" && !(principal in Role::"crm-service-risk-override") };
```
Territory ownership:
```cedar
permit(principal in Role::"crm-service-agent", action == Action::"crm.case.update", resource)
when { resource.support_queue_id in principal.assigned_queue_ids && resource.tenant_id == principal.tenant_id };
```
Forecast roll-up approval:
```cedar
permit(principal in Role::"crm-forecast-approver", action == Action::"crm.forecast.rollup.approve", resource)
when { context.p1_case_exposure_amount <= principal.service_risk_ceiling_usd };
```
Partner portal visibility:
```cedar
permit(principal in Role::"crm-partner-portal-user", action == Action::"crm.case.read.partner", resource)
when { resource.partner_visible == true && context.payload_class == "case_summary" && context.partner_id == resource.channel_partner_id };
```
SLA waiver:
```cedar
permit(principal in Role::"crm-service-supervisor", action == Action::"crm.case.sla.waive", resource)
when { context.waive_justification.length >= 20 && resource.status != "Closed" };
```

## 5. Ontology Projection
Salesforce Account maps to `Oyatie::Customer.account_profile` with support tier.
Salesforce Contact maps to `Oyatie::Customer.primary_contact` with preferred support channel.
Salesforce Case maps to `Oyatie::Customer.service_posture`.
Salesforce Opportunity maps to `Oyatie::Customer.revenue_posture` with service risk blockers.
Field delta: Salesforce Case becomes typed Case with entitlement contract id.
Field delta: Salesforce CaseMilestone becomes explicit warning and breach rows.
Field delta: Salesforce Entitlement becomes Contract-derived support plan.
Field delta: Salesforce KnowledgeArticle solution becomes `Solution` with resolution confidence.
Projection event:
```json
{"entity":"Oyatie::Customer","source":"crm.case_sla","customer_id":"cust_harbor","field_deltas":["service_posture.open_cases","service_posture.sla_risk","revenue_posture.service_blockers"]}
```

## 6. Workflow Steps
Node `idempotency_check` rejects duplicate source external ids.
Node `load_account_contact_contract` collects Account, Contact, and Contract.
Node `cedar_create_case_gate` evaluates support authority and abuse controls.
Node `instantiate_case` writes Case.
Node `materialize_milestones` creates ResponseTime, UpdateInterval, RestoreTime, and ResolveTime rows.
Node `route_to_support_queue` calls contact-center with skill and presence.
Decision `no_agent_available` assigns supervisor queue and starts SLA clock anyway.
Node `append_initial_activity` records inbound payload.
Node `sla_burn_watcher` emits 50 percent, 80 percent, and breach events.
Decision `eighty_percent_burn` advances escalation plan.
Node `publish_partner_summary` sends scrubbed case if partner visible.
Node `resolve_with_solution` links Solution.
Node `close_case` verifies customer grace window and writes status.

## 7. Audit Events
ADR-0263 registry class `CrmCaseOpened`.
ADR-0263 registry class `CrmCaseActivityAppended`.
ADR-0263 registry class `CrmCaseStatusChanged`.
ADR-0263 registry class `CrmCaseSlaWarning50`.
ADR-0263 registry class `CrmCaseSlaWarning80`.
ADR-0263 registry class `CrmCaseSlaBreached`.
ADR-0263 registry class `CrmCaseSlaWaived`.
ADR-0263 registry class `CrmCaseEscalated`.
ADR-0263 registry class `CrmCaseResolved`.
ADR-0263 registry class `CrmCaseReopened`.

## 8. SLO Targets
p50 case create latency is 80 ms.
p95 case create latency is 200 ms including entitlement lookup.
p99 case create latency is 600 ms under contract cache miss.
p95 P1 routing latency is 1500 ms end to end.
p95 SLA warning emission accuracy is within two seconds of target.
p99 case read with last fifty activities is 140 ms.
Rationale: incident response depends on predictable routing and exact milestone emission more than broad dashboard throughput.
Activity append throughput target is 5000 inserts per second per cell.

## 9. Failure Modes and Recovery
Salesforce Service Cloud Bulk API 10K case import ceiling appears during migration; recovery chunks by account and source case number.
Governor-limit parity appears when legacy entitlement flow had hidden side effects; recovery models every side effect as explicit workflow node.
Entitlement lookup miss appears for a paying customer; recovery opens case in pending entitlement confirmation and notifies supervisor.
SLA watcher restarts after leader failover; recovery reconstructs pending milestones from `target_at` and idempotent emitted columns.
Partner visibility leakage risk appears on partner-linked cases; recovery publishes summary-only partner payload.
Lead conversion conflict appears when an open P1 should block ClosedWon; recovery denies stage advance until service risk override is present.

## 10. Migration Notes
Salesforce Sales Cloud opportunity service blockers map to Opportunity support risk.
Salesforce Service Cloud Case, Entitlement, Milestone, Omni-Channel, and Knowledge map to Case, Contract, Milestone, queue routing, and Solution.
Salesforce Marketing Cloud incident notification journeys map to Campaign and EmailTemplate.
Salesforce Industries service entitlements map to Contract support plans.
SAP CRM Service tickets and Interaction Center records map to Case and Activity.
SAP C4C Service tickets map to Case.
SAP Service Cloud entitlements map to Contract support plan.
Microsoft Dynamics 365 CE account cases map to Case.
Microsoft Dynamics 365 Customer Service Hub cases and SLAs map to Case and Milestone.
Oracle Fusion CX, Sales Cloud, and Service Cloud service requests map to Case.
HubSpot Sales Hub deal blockers map to Opportunity support risk.
HubSpot Service Hub tickets map to Case.
Zendesk Sell customer records and Zendesk Support tickets map to Account and Case.
Pipedrive activities map to service activities where used for support.
Freshsales support tickets map to Case.
ActiveCampaign incident emails map to EmailTemplate and Campaign.

## 11. Cross-Service Handoffs
Marketplace receives service-blocker flags for deal settlement readiness.
Payments receives case-linked invoice dispute context.
Community receives partner-safe case summaries and status updates.
Marketing-automation receives incident communication triggers.
Intelligence receives case severity, SLA burn, and solution outcome features.
Contact-center receives routing requests and returns agent assignment.
Contract lifecycle supplies entitlement support plan.
ITSM receives mirrored incidents when escalation plan requires engineering.
Ontology receives Customer service posture deltas.
Audit-chain seals all case lifecycle and SLA events.

## 12. Acceptance
P1 case create materializes response and restore milestones.
SLA warnings are emitted at 50 percent, 80 percent, and breach idempotently.
Open P1 cases can block ClosedWon without service-risk override.
Partner case visibility is summary-only and Cedar-filtered.
Every status change and activity has an ADR-0263 audit class.
The IP is bespoke to service-case SLA and escalation behavior.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-022-service-case-sla-and-escalation-engine.md` matched [`SLO`, `p99`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-022-service-case-sla-and-escalation-engine.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-022-service-case-sla-and-escalation-engine.md` matched [`emission`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-022-service-case-sla-and-escalation-engine.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
