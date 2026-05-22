---
doc_class: Implementation-Plan
ip_id: IP-008
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j36-b2b-workflow-engine-approval-cascade
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-sales + axis-erp-parity
---
# IP-008: Opportunity Usecase Layer

## Context
- This slice orchestrates opportunity creation, stage advancement, territory reassignment, close-won, and close-lost commands.
- SAP benchmark: SAP CRM-SLS guided selling and opportunity phase transitions.
- Salesforce benchmark: Sales Cloud Opportunity Path, validation rules, approvals, and Flow.
- Persona: Rafael Okafor, regional sales director at Meridian Pumps.
- Journey leg: j36 high-value approval cascade for a regulated opportunity.
- Why now: domain stage invariants from IP-002 need transaction, policy, workflow, forecast, and ontology coordination.
- Vendor displacement covers Salesforce Sales Cloud automation, SAP C4C opportunity workflow, Dynamics 365 Business Process Flow, Oracle Sales Cloud opportunities, HubSpot deal workflows, Pipedrive automations, Zendesk Sell pipelines, Freshsales workflows, and ActiveCampaign deal automation.
- Usecase owns idempotency and expected-version checks.
- Usecase is the only layer allowed to start workflow approvals for stage exceptions.
- Usecase emits forecast and campaign attribution deltas after domain commit.
- Usecase must keep finance barrier semantics explicit for Commit and ClosedWon transitions.
- This IP does not implement forecast roll-up; it emits the evidence IP-021 consumes.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, lifecycle_state TEXT NOT NULL, territory_id UUID, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), buying_role TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, converted_opportunity_id UUID, conversion_lock TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), stage TEXT NOT NULL, amount NUMERIC(18,2), expected_version BIGINT NOT NULL DEFAULT 1, usecase_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), close_won_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), legal_review_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), opportunity_blocker_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), influence_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), solution_review_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), delta_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, owner_principal_id UUID NOT NULL, reassignment_workflow_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), partner_deal_ref TEXT, portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), proposal_asset_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_stage TEXT NOT NULL, playbook_template_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity_stage_event (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), from_stage TEXT, to_stage TEXT NOT NULL, policy_decision_id TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS crm_opp_usecase_stage_idx ON crm.opportunity(tenant_id, stage, usecase_state);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub lifecycle_state: AccountState, pub territory_id: TerritoryId }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub buying_role: BuyingRole }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub converted_opportunity_id: Option<OpportunityId>, pub conversion_lock: Option<String> }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub stage: OpportunityStage, pub expected_version: u64, pub usecase_state: UsecaseState }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub close_won_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub legal_review_state: LegalReviewState }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub account_id: AccountId, pub opportunity_blocker_ref: Option<String> }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub influence_ref: Option<String> }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub solution_review_state: ReviewState }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub delta_state: DeltaState }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub owner_principal_id: PrincipalId, pub reassignment_workflow_ref: Option<String> }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub partner_deal_ref: String }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub proposal_asset_ref: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub opportunity_stage: OpportunityStage, pub playbook_template_ref: String }
pub struct OpportunityUsecasePorts { pub repo: OpportunityRepoPort, pub policy: CedarPort, pub workflow: WorkflowPort, pub forecast: ForecastPort, pub audit: AuditChainPort }
pub enum OpportunityUsecaseOutcome { Applied(OpportunityId), Denied(PolicyDecisionId), VersionConflict, ApprovalStarted(WorkflowRunId), PendingForecast(OpportunityId), Blocked(BlockerCode) }
```

## API Endpoints
- REST facade: `POST /v1/crm/opportunities/{id}/stage-advancements`.
- REST body: `{ "tenant_id": "ten_meridian", "principal_id": "usr_rafael", "from_stage": "Proposal", "to_stage": "Commit", "expected_version": 7, "evidence_refs": ["quote:quo_88"] }`.
- REST response: `{ "outcome": "Applied", "opportunity_id": "opp_88", "audit_event_class": "EVT-CRM-OPPORTUNITY-USECASE-STAGE-APPLIED" }`.
- REST approval response: `{ "outcome": "ApprovalStarted", "workflow_run_id": "wf_36", "required_step": "finance_barrier" }`.
- gRPC facade: `rpc AdvanceOpportunityStage(AdvanceOpportunityStageUsecaseRequest) returns (OpportunityUsecaseReply)`.
- gRPC facade: `rpc CloseOpportunity(CloseOpportunityUsecaseRequest) returns (OpportunityUsecaseReply)`.
- gRPC reply carries outcome, opportunity_id, audit_id, policy_decision_id, workflow_run_id, forecast_delta_id.
- AsyncAPI channel: `crm.opportunity.usecase.events.v1`.
- AsyncAPI message: `OpportunityUsecaseStageApplied`.
- AsyncAPI body: `{ "tenant_id": "ten_meridian", "opportunity_id": "opp_88", "to_stage": "Commit", "audit_event_class": "EVT-CRM-OPPORTUNITY-USECASE-STAGE-APPLIED" }`.
- Usecase emits forecast delta and ontology delta after stage commit.
- REST maps VersionConflict to 409.
- REST maps ApprovalStarted to 202 with workflow pointer.
- gRPC preserves enum outcome for workers.
- AsyncAPI never publishes stage applied before audit seal.

## Cedar Policy Hooks
- Stage-advance gate: usecase action is `crm.opportunity.advance_stage`.
- Territory ownership: usecase reads territory closure from identity/territory port.
- Forecast-roll-up approval: Commit and ClosedWon above threshold require finance-barrier context.
- Partner-portal visibility: partner deal rooms receive only policy-approved opportunity summary.
- Context includes from_stage, to_stage, amount, currency, close_date, account_risk, quote_status, open_case_count.
- Resource includes opportunity_id, tenant_id, account_id, stage, territory_id, partner_visibility.
- Principal includes role, territory path, delegated authority, and partner relationship.
- Denial seals `EVT-CRM-OPPORTUNITY-USECASE-POLICY-DENIED`.
- Cedar failure returns Blocked, never Allow.
- Policy decision id is included in stage event row.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.buying_committee`.
- Salesforce Case maps to `Oyatie::Customer.service_risk`.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline`.
- Delta: usecase outcome is represented as stage_transition_evidence.
- Delta: finance barrier workflow id becomes Customer revenue governance metadata.
- Delta: partner-visible summary is separate from internal opportunity view.
- Delta: forecast delta id is included for downstream predictive scoring.

## Workflow Steps
- Node `normalize_stage_command`: convert REST/gRPC input to usecase command.
- Node `check_idempotency`: replay prior stage result when command hash matches.
- Node `load_snapshot`: fetch opportunity, account, open cases, quotes, and territory.
- Node `build_cedar_context`: include amount, account risk, quote state, and finance barrier.
- Decision `requires_workflow_approval`: branch when Cedar permits only with workflow approval.
- Node `execute_domain_stage`: call IP-002 aggregate.
- Node `persist_stage_event`: append event and expected version increment.
- Node `seal_audit`: seal ADR-0263 linked event.
- Node `emit_forecast_delta`: notify forecast consumers.
- Node `emit_ontology_delta`: notify ontology.
- Branch `version_conflict`: return current version and no mutation.
- Branch `lead_conversion_conflict`: start repair workflow.

## Audit Events
- `EVT-CRM-OPPORTUNITY-USECASE-RECEIVED`.
- `EVT-CRM-OPPORTUNITY-USECASE-IDEMPOTENT-REPLAYED`.
- `EVT-CRM-OPPORTUNITY-USECASE-POLICY-DENIED`.
- `EVT-CRM-OPPORTUNITY-USECASE-APPROVAL-STARTED`.
- `EVT-CRM-OPPORTUNITY-USECASE-STAGE-APPLIED`.
- `EVT-CRM-OPPORTUNITY-USECASE-VERSION-CONFLICT`.
- `EVT-CRM-OPPORTUNITY-USECASE-FORECAST-PENDING`.
- `EVT-CRM-OPPORTUNITY-USECASE-PARTNER-SUMMARY-MASKED`.
- ADR-0263 fields include audit_id, policy_decision_id, trace_id, span_id, tenant_id, opportunity_id, stage pair, and schema_version.

## SLO Targets
- Stage usecase p50: 80 ms.
- Stage usecase p95: 260 ms.
- Stage usecase p99: 850 ms with finance barrier lookup.
- Version conflict p95: 45 ms.
- Forecast delta publish p95: 500 ms.
- Availability: 99.95 percent for stage command.
- Rationale: interactive sales stage updates must remain responsive while preserving workflow approval path.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: migration invokes usecase through chunks with replayable idempotency.
- Salesforce governor limits: adapter throttles; usecase returns deterministic retry metadata.
- Lead conversion conflict: usecase blocks stage and starts conversion repair.
- Version conflict: return current version and let caller re-read.
- Workflow-engine unavailable: return ApprovalStartedPending and retry workflow start.
- Forecast outbox stalled: keep stage applied and expose PendingForecast until replay catches up.

## Migration Notes
- Salesforce CRM: Opportunity stage imports call usecase in source order with SystemModstamp cursor.
- SAP CRM: sales phase imports map to usecase stage transitions, not direct row updates.
- Microsoft Dynamics 365 CE: business process flow stage maps to stage command with expected_version.
- HubSpot Sales Hub: deal pipeline changes replay as stage commands.
- Pipedrive: deal stage movement imports as stage commands with lossy probability note.
- Zendesk Sell: deal status imports as close or stage commands with source warnings.

## Cross-Service Handoffs
- Marketplace receives closed-won settlement intent through usecase event.
- Payments receives invoice readiness only after order handoff.
- Community receives partner deal-room updates after masking.
- Marketing-automation receives stage-trigger campaign signals.
- Intelligence receives stage transition features.
- Ontology receives Customer revenue-pipeline deltas.
- Workflow-engine receives high-value approvals and conversion repairs.
- Audit-chain seals every usecase outcome.

## Build Checklist
- Implement AdvanceOpportunityStageUsecase.
- Implement CloseOpportunityUsecase.
- Define OpportunityUsecasePorts.
- Implement expected-version guard.
- Implement idempotency hash for stage commands.
- Implement finance barrier context builder.
- Implement forecast delta outbox.
- Implement ontology delta outbox.
- Add VersionConflict unit test.
- Add ApprovalStarted branch test.
- Add lead conversion conflict test.
- Add forecast outbox stalled test.
- Add REST stage fixture.
- Add REST approval-started fixture.
- Add gRPC stage fixture.
- Add AsyncAPI stage applied fixture.
- Add Cedar cross-territory denial fixture.
- Add Salesforce stage replay fixture.
- Add SAP phase replay fixture.
- Add Dynamics BPF replay fixture.
- Add ADR-0263 audit fixture.
- Add metric `oya:crm:opportunity_usecase:stage_latency_ms:histogram`.
- Add metric `oya:crm:opportunity_usecase:version_conflict_total:counter`.
- Stop when outcome, policy, workflow, forecast, and ontology tests pass.

## Acceptance
- Usecase depends on domain and ports only.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, version conflict, workflow outage, and forecast stall.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-008-usecase-layer-for-opportunity.md` matched [`SLO`, `p99`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/crm/IP-008-usecase-layer-for-opportunity.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-008-usecase-layer-for-opportunity.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-008-usecase-layer-for-opportunity.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
