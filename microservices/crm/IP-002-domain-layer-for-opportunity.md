---
doc_class: Implementation-Plan
ip_id: IP-002
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j36-b2b-workflow-engine-approval-cascade
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-sales + axis-erp-parity
---
# IP-002: Opportunity Domain Layer

## Context
- This slice builds the opportunity aggregate and its deterministic stage model.
- SAP benchmark: SAP CRM-SLS Opportunity Management.
- Salesforce benchmark: Sales Cloud Opportunities, Forecast Categories, and Path stage governance.
- Persona: Rafael Okafor, regional sales director at Meridian Pumps.
- Journey leg: j36 approval-cascade leg where a high-value deal crosses legal, finance, and executive review.
- Why now: quotes, forecasts, campaign attribution, and churn-risk scoring depend on reliable opportunity state.
- The aggregate displaces Salesforce Sales Cloud, SAP C4C Sales, Dynamics 365 Sales, Oracle Sales Cloud, HubSpot Deals, Zendesk Sell deals, Pipedrive deals, Freshsales deals, and ActiveCampaign CRM deals.
- Stage changes are domain decisions, not controller flags.
- The domain rejects stage advancement without required evidence, account status, territory ownership, and policy result.
- Opportunity amount is versioned because quote and forecast consumers require historical stage snapshots.
- Probability is a derived field from stage plus tenant playbook, not a freeform sales-rep number.
- This IP does not build quote pricing, forecast roll-up, or campaign influence; it emits stable deltas for those IPs.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, legal_name TEXT NOT NULL, lifecycle_state TEXT NOT NULL, territory_id UUID, owner_principal_id UUID NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), role TEXT, buying_committee_rank INT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, converted_opportunity_id UUID, qualification_score NUMERIC(8,4), status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID NOT NULL REFERENCES crm.account(id), name TEXT NOT NULL, stage TEXT NOT NULL, amount NUMERIC(18,2) NOT NULL, currency TEXT NOT NULL, close_date DATE NOT NULL, probability_basis TEXT NOT NULL, audit_id TEXT NOT NULL, version BIGINT NOT NULL DEFAULT 1);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), quote_total NUMERIC(18,2), status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), contract_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), opportunity_id UUID REFERENCES crm.opportunity(id), escalation_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_code TEXT NOT NULL, attribution_model TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), proposed_solution_ref TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), forecast_category TEXT NOT NULL, forecast_amount NUMERIC(18,2), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, owner_principal_id UUID NOT NULL, stage_authority JSONB NOT NULL DEFAULT '{}'::jsonb, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), referral_role TEXT, portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_id UUID REFERENCES crm.opportunity(id), document_uri TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, opportunity_stage TEXT NOT NULL, locale TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS crm_opportunity_stage_idx ON crm.opportunity(tenant_id, stage, close_date);
CREATE INDEX IF NOT EXISTS crm_opportunity_account_idx ON crm.opportunity(tenant_id, account_id);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub lifecycle_state: AccountState, pub territory_id: TerritoryId }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub buying_committee_rank: Option<u8> }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub converted_opportunity_id: Option<OpportunityId>, pub qualification_score: Decimal }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub account_id: AccountId, pub stage: OpportunityStage, pub amount: Money, pub close_date: Date }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub quote_total: Money }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub status: OrderState }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub contract_state: ContractState }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub opportunity_id: Option<OpportunityId>, pub escalation_state: EscalationState }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub campaign_code: String, pub attribution_model: AttributionModel }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub proposed_solution_ref: String }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub forecast_category: ForecastCategory }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub owner_principal_id: PrincipalId, pub stage_authority: StageAuthority }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub portal_visibility: PortalVisibility }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub opportunity_id: OpportunityId, pub document_uri: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub opportunity_stage: OpportunityStage, pub locale: Locale }
pub enum OpportunityStage { Prospecting, Qualified, Proposal, Negotiation, Commit, ClosedWon, ClosedLost }
pub enum OpportunityCommand { Create, AdvanceStage, RegressStage, ReassignTerritory, CloseWon, CloseLost }
```

## API Endpoints
- REST command: `POST /v1/crm/opportunities`.
- REST create body: `{ "tenant_id": "ten_meridian", "account_id": "acc_77", "name": "Brazil plant pump retrofit", "amount": "840000.00", "currency": "USD", "close_date": "2026-09-30" }`.
- REST stage body: `{ "target_stage": "Commit", "evidence_refs": ["quote:q_22", "security-review:sr_9"], "forecast_category": "Commit" }`.
- REST response: `{ "opportunity_id": "opp_77", "stage": "Commit", "audit_event_class": "EVT-CRM-OPPORTUNITY-STAGE-ADVANCED" }`.
- REST query: `GET /v1/crm/opportunities?account_id=acc_77&stage=Commit`.
- gRPC service: `rpc AdvanceOpportunityStage(AdvanceOpportunityStageRequest) returns (OpportunityMutationResult)`.
- gRPC service: `rpc RecomputeOpportunityProbability(RecomputeOpportunityProbabilityRequest) returns (OpportunityProbabilityResult)`.
- gRPC request carries tenant_id, principal_id, opportunity_id, from_stage, to_stage, evidence_refs, traceparent, idempotency_key.
- AsyncAPI channel: `crm.opportunity.events.v1`.
- AsyncAPI message: `OpportunityStageAdvanced`.
- AsyncAPI body: `{ "tenant_id": "ten_meridian", "opportunity_id": "opp_77", "from_stage": "Proposal", "to_stage": "Commit", "audit_event_class": "EVT-CRM-OPPORTUNITY-STAGE-ADVANCED" }`.
- AsyncAPI consumers: forecast, campaign attribution, quote, intelligence, ontology, audit-chain.
- API idempotency key expires only after stage state and audit event both commit.
- Stage command returns typed denial, not boolean false.
- All public payloads are SemVer v1 and additive-only.
- Batch import uses async worker, not REST hot path.

## Cedar Policy Hooks
- Stage-advance gate: principal `Role::"crm-sales-manager"` action `Action::"crm.opportunity.advance_stage"` resource Opportunity with target_stage in context.
- Territory ownership: resource.territory_id must be within principal.territory_closure.
- Forecast-roll-up approval: target_stage `Commit` requires context.forecast_period_open and finance_barrier_classification.
- Partner-portal visibility: partner can read opportunity summary only when resource.partner_visibility is `shared_pipeline`.
- Closed-won gate requires accepted quote, no active legal hold, and marketplace settlement route.
- Closed-lost gate requires loss_reason and competitor code for analytics.
- High-risk advance requires abuse-defence spoof score below tenant threshold.
- Context includes source_system, pack_overlay, amount_band, currency, close_date, traceparent, and policy_bundle_version.
- Permit emits policy_decision_id and denial reason into ADR-0263 structured log.
- Forbid branch masks amount for partner principals lacking deal_financials grant.

## Ontology Projection
- Salesforce Account maps through opportunity.account_id to `Oyatie::Customer.account_profile`.
- Salesforce ContactRole maps to `Oyatie::Customer.buying_committee`.
- Salesforce Case maps to `Oyatie::Customer.service_risk` when open cases block stage advance.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline`.
- Delta: Oyatie stores stage evidence refs and Cedar policy decision per transition.
- Delta: Forecast category is event-sourced, not overwritten on the opportunity row alone.
- Delta: Partner visibility is explicit and not inferred from opportunity team membership.
- Delta: Campaign influence remains a separate projection consumed by IP-019.

## Workflow Steps
- Node `load_opportunity_snapshot`: lock current version and stage.
- Node `check_account_active`: reject when account lifecycle is Draft, Suspended, or Archived.
- Node `evaluate_stage_evidence`: verify quote, solution, campaign, legal, and finance evidence refs.
- Node `evaluate_cedar_stage`: run stage, territory, partner, and forecast gates.
- Decision `requires_finance_barrier`: branch for Commit and ClosedWon above configured threshold.
- Node `commit_stage_event`: append immutable transition row.
- Node `update_forecast_projection`: emit forecast delta for IP-021.
- Node `update_customer_ontology`: emit Customer pipeline delta.
- Node `publish_opportunity_event`: publish stage event.
- Branch `lead_conversion_conflict`: pause and require conversion repair.
- Branch `quote_missing`: return actionable blocked state.
- Branch `policy_denied`: seal denial event and return 403 typed reason.

## Audit Events
- `EVT-CRM-OPPORTUNITY-CREATE-REQUESTED`.
- `EVT-CRM-OPPORTUNITY-CREATED`.
- `EVT-CRM-OPPORTUNITY-STAGE-ADVANCE-REQUESTED`.
- `EVT-CRM-OPPORTUNITY-STAGE-ADVANCED`.
- `EVT-CRM-OPPORTUNITY-STAGE-DENIED`.
- `EVT-CRM-OPPORTUNITY-CLOSED-WON`.
- `EVT-CRM-OPPORTUNITY-CLOSED-LOST`.
- `EVT-CRM-OPPORTUNITY-PARTNER-VISIBILITY-MASKED`.
- ADR-0263 fields: audit_id, tenant_id, subscope, trace_id, span_id, metric exemplar, schema_version, source_microservice.

## SLO Targets
- Stage advance p50: 60 ms for current snapshot, Cedar, and append.
- Stage advance p95: 220 ms with account, quote, forecast, and ontology checks.
- Stage advance p99: 650 ms for high-value deals with finance barrier lookup.
- Opportunity list p95: 120 ms for tenant-stage-close-date indexed reads.
- Forecast delta publish p95: 500 ms from domain commit.
- Error rate target: below 0.25 percent excluding policy denials.
- Rationale: sales users tolerate sub-second stage action but not silent delayed forecast propagation.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: import opportunities in deterministic cursor windows and seal each checkpoint.
- Salesforce governor limits: throttle source reads and resume by SystemModstamp.
- Lead conversion conflict: lock lead, account, contact, and opportunity conversion IDs until a repair workflow resolves.
- Stage regression by stale client: reject when expected_version mismatches current version.
- Forecast consumer unavailable: persist event in outbox and expose forecast_projection_lag.
- Partner reads unmasked financial fields: Cedar forbid wins and emits partner visibility masked event.

## Migration Notes
- Salesforce CRM: Opportunity.Id maps to source_system_ref; StageName maps through tenant stage playbook.
- SAP CRM: CRM-SLS opportunity GUID maps to source_system_ref; sales phase maps to stage with confidence note.
- Microsoft Dynamics 365 CE: opportunityid maps to source_system_ref; estimatedclosedate maps to close_date.
- HubSpot Sales Hub: dealId maps to source_system_ref; pipeline/stage maps through tenant stage dictionary.
- Pipedrive: deal stage maps to stage; probability ignored unless tenant accepts source probability.
- Zendesk Sell: deal status maps to lifecycle transition with lossy-status warning.

## Cross-Service Handoffs
- Marketplace receives closed-won settlement intent only after quote acceptance.
- Payments receives invoice intent through order/contract, never directly from open opportunity.
- Community receives partner deal-room reference after partner visibility permit.
- Marketing-automation receives campaign influence outcome and stage-change triggers.
- Intelligence receives stage history for win-propensity and churn-risk model features.
- Ontology receives Customer revenue-pipeline projection.
- Workflow-engine owns high-value approval cascade.
- Audit-chain seals every stage transition and denial.

## Build Checklist
- Implement Opportunity aggregate with versioned stage transitions.
- Implement tenant playbook mapping for stage order.
- Implement amount and currency value objects.
- Implement probability derivation from stage plus playbook.
- Implement stage evidence validator.
- Implement territory ownership port.
- Implement forecast delta port.
- Implement partner visibility value object.
- Add property tests for monotonic stage advance.
- Add test for allowed regression only through manager-approved correction.
- Add test for ClosedWon requiring quote acceptance.
- Add test for Commit requiring forecast period openness.
- Add Cedar fixture for cross-territory denial.
- Add Cedar fixture for partner amount masking.
- Add REST fixture for create.
- Add REST fixture for stage advance.
- Add gRPC fixture for stage advance.
- Add AsyncAPI fixture for stage advanced.
- Add import fixture for Salesforce Opportunity.
- Add migration fixture for HubSpot Deal.
- Add replay fixture for 10K opportunity batch.
- Add audit fixture with ADR-0263 fields.
- Add SLO metric `oya:crm:opportunity:stage_advance_latency_ms:histogram`.
- Stop when stage, forecast, ontology, and partner fixtures pass.

## Acceptance
- Opportunity domain has no direct database, REST, or external SDK dependency.
- All 15 CRM entities are present in DDL and Rust roster.
- API examples include REST, gRPC, and AsyncAPI bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, and lead conversion conflicts.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-002-domain-layer-for-opportunity.md` matched [`financial`, `SLO`, `p99`, `payment`].
- applicable_compliance_pack_floor: [`EU-AI-ACT-2024-HIGH-RISK`, `SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `1800`; rpo_p99_seconds_target: `300`.
- multi_region_active_active: `true`; floor_requires_active_active: `true`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-002-domain-layer-for-opportunity.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-002-domain-layer-for-opportunity.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-002-domain-layer-for-opportunity.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
