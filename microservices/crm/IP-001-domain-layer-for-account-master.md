---
doc_class: Implementation-Plan
ip_id: IP-001
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-erp-parity
---
# IP-001: Account Master Domain Layer

## Context
- This slice builds the account master aggregate that every later CRM slice relies on.
- SAP benchmark: SAP CRM-MD Business Partner master data and SAP Cloud for Customer Account Management.
- Salesforce benchmark: Sales Cloud Account Management with Customer 360 account identity.
- Persona: Maya Chen, enterprise revenue-operations lead at Northwind Robotics.
- Journey leg: j100 tenant onboarding first-action leg where the first governed account must exist before any sales motion.
- Why now: account data is the join root for contacts, leads, cases, opportunities, quotes, contracts, campaigns, and partner visibility.
- This IP displaces account-sprawl behavior from Salesforce, Dynamics 365 CE, Oracle Fusion CX, HubSpot, Zendesk Sell, Pipedrive, Freshsales, and ActiveCampaign.
- The account aggregate is append-only for material changes; destructive correction is rejected.
- The domain layer owns invariants only; REST, gRPC, workers, ontology storage, and external imports are downstream slices.
- Account hierarchy is not solved here; this slice exposes parent pointers and guardrails for IP-017.
- Marketplace settlement is not owned here; this slice records stable settlement references for later deal handoff.
- The account master must be buildable from this file plus cited contracts without asking for tribal rules.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, legal_name TEXT NOT NULL, lifecycle_state TEXT NOT NULL, territory_id UUID, owner_principal_id UUID NOT NULL, source_system TEXT, audit_id TEXT NOT NULL, version BIGINT NOT NULL DEFAULT 1);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), email TEXT, consent_state TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, converted_account_id UUID REFERENCES crm.account(id), source_campaign_id UUID, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), stage TEXT NOT NULL, amount NUMERIC(18,2), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), opportunity_id UUID REFERENCES crm.opportunity(id), status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), quote_id UUID REFERENCES crm.quote(id), marketplace_settlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), order_id UUID REFERENCES crm.order_header(id), effective_from TIMESTAMPTZ, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), contact_id UUID REFERENCES crm.contact(id), priority TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_segment TEXT, attribution_model TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, case_id UUID REFERENCES crm.case_record(id), knowledge_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, territory_id UUID, period_key TEXT NOT NULL, committed_amount NUMERIC(18,2), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, parent_territory_id UUID, capacity_score NUMERIC(8,4), audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), partner_tier TEXT, portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), document_uri TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, campaign_id UUID REFERENCES crm.campaign(id), locale TEXT NOT NULL, subject TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE INDEX IF NOT EXISTS crm_account_tenant_state_idx ON crm.account(tenant_id, lifecycle_state);
CREATE UNIQUE INDEX IF NOT EXISTS crm_account_source_unique ON crm.account(tenant_id, source_system, legal_name) WHERE source_system IS NOT NULL;
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub legal_name: String, pub lifecycle_state: AccountState, pub owner_principal_id: PrincipalId }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub consent_state: ConsentState }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub converted_account_id: Option<AccountId>, pub status: LeadStatus }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub account_id: AccountId, pub stage: OpportunityStage }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub account_id: AccountId, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub account_id: AccountId, pub marketplace_settlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub account_id: AccountId, pub effective_from: Time }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub account_id: AccountId, pub priority: CasePriority }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub account_segment: String, pub attribution_model: AttributionModel }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub case_id: CaseId, pub knowledge_ref: String }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub territory_id: TerritoryId, pub period_key: String }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub parent_territory_id: Option<TerritoryId>, pub capacity_score: Decimal }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub account_id: AccountId, pub portal_visibility: PortalVisibility }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub document_uri: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub campaign_id: CampaignId, pub locale: Locale }
pub enum AccountState { Draft, Active, Suspended, Archived }
pub enum AccountCommand { CreateAccount, AmendAccount, SuspendAccount, ArchiveAccount }
```

## API Endpoints
- REST command: `POST /v1/crm/accounts`.
- REST body: `{ "tenant_id": "ten_northwind", "legal_name": "Northwind Robotics GmbH", "owner_principal_id": "usr_maya", "source_system": "salesforce-account", "idempotency_key": "crm-acct-001" }`.
- REST success: `202 Accepted` with `{ "account_id": "acc_01", "audit_event_class": "EVT-CRM-ACCOUNT-CREATED", "state": "Active" }`.
- REST query: `GET /v1/crm/accounts/{account_id}?include=contacts,cases,opportunities`.
- REST patch: `PATCH /v1/crm/accounts/{account_id}` accepts legal name, owner, territory, and lifecycle transitions.
- REST reject: `409` when source-system duplicate maps to a different tenant account.
- gRPC service: `rpc CreateAccount(CreateAccountRequest) returns (AccountMutationResult)`.
- gRPC service: `rpc AmendAccount(AmendAccountRequest) returns (AccountMutationResult)`.
- gRPC request fields: tenant_id, principal_id, legal_name, owner_principal_id, source_system, traceparent, idempotency_key.
- gRPC response fields: account_id, state, audit_id, policy_decision_id, ontology_projection_version.
- AsyncAPI channel: `crm.account.events.v1`.
- AsyncAPI message: `AccountCreated`.
- AsyncAPI body: `{ "tenant_id": "ten_northwind", "account_id": "acc_01", "state": "Active", "audit_event_class": "EVT-CRM-ACCOUNT-CREATED", "occurred_at": "2026-05-20T00:00:00Z" }`.
- AsyncAPI replay key: tenant_id plus account_id plus version.
- Idempotency scope: tenant_id plus idempotency_key plus command name.
- Versioning: additive-only v1 messages; removals require ADR-0263 schema-evolution handshake.

## Cedar Policy Hooks
- Stage-advance gate: principal `User::"usr_maya"` action `Action::"crm.account.activate"` resource `Account::"acc_01"` context includes source_system_verified and onboarding_pack_ready.
- Territory ownership: principal must own or manage `territory_id`; context includes territory_path and delegated_until.
- Forecast-roll-up approval: account owner changes that affect forecast territory emit a finance-visible review event.
- Partner-portal visibility: channel partners can read account summary only when resource.portal_visibility is `partner_summary`.
- Create rule denies when context.tenant_id differs from resource.tenant_id.
- Amend rule denies when account is Archived unless action is `crm.account.restore`.
- Suspend rule requires abuse-defence signal when reason is fraud or spoofing.
- Archive rule requires no open case, no active contract, and no unsettled marketplace deal.
- Context fields: tenant_id, principal_id, purpose, traceparent, policy_bundle_version, source_system, residency_pack.
- Policy decision is logged with ADR-0263 `audit_id` linkage before command commit.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.primary_contacts`.
- Salesforce Case maps to `Oyatie::Customer.service_posture`.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline`.
- Delta: Oyatie adds tenant_id, data_class, consent_state, residency_pack, policy_decision_id, audit_id, and ontology_projection_version.
- Delta: Salesforce OwnerId becomes Oyatie owner_principal_id with tenant-scoped principal continuity.
- Delta: SAP business partner number becomes source_system_ref, never the primary key.
- Delta: Dynamics account category maps to account_segment, not lifecycle_state.
- Projection is emitted to ontology; CRM does not own the materialized read store.

## Workflow Steps
- Node `validate_account_identity`: normalize legal name, jurisdiction, source ref, and duplicate candidates.
- Decision `duplicate_candidate_found`: branch to merge-review when confidence is above 0.92.
- Node `evaluate_cedar_create`: run tenant, owner, and territory policies.
- Node `reserve_account_id`: allocate UUID and source-system unique guard.
- Node `commit_domain_event`: persist account with version 1.
- Node `seal_audit_event`: create ADR-0263 linked audit-chain event.
- Node `emit_account_created`: publish AsyncAPI event.
- Node `project_customer_ontology`: call ontology writer contract.
- Node `notify_workflow_completion`: mark j100 first-action gate satisfied.
- Branch `policy_denied`: return typed denial with policy decision id.
- Branch `audit_chain_unavailable`: hold mutation in pending-seal queue.
- Branch `ontology_projection_failed`: commit account but mark projection_retry_required.

## Audit Events
- `EVT-CRM-ACCOUNT-CREATE-REQUESTED`.
- `EVT-CRM-ACCOUNT-CREATED`.
- `EVT-CRM-ACCOUNT-AMENDED`.
- `EVT-CRM-ACCOUNT-SUSPENDED`.
- `EVT-CRM-ACCOUNT-ARCHIVED`.
- `EVT-CRM-ACCOUNT-DUPLICATE-MERGE-REQUESTED`.
- `EVT-CRM-ACCOUNT-TERRITORY-OWNER-CHANGED`.
- `EVT-CRM-ACCOUNT-PARTNER-VISIBILITY-CHANGED`.
- Every event carries tenant_id, account_id, principal_id, trace_id, span_id, audit_id, policy_decision_id, schema_version, and source_microservice.

## SLO Targets
- Create account p50: 45 ms because domain validation is in-memory plus one PostgreSQL insert.
- Create account p95: 180 ms because Cedar, audit-chain seal, and ontology enqueue are on the hot path.
- Create account p99: 450 ms with tail sampling at 100 percent for errors and p99 outliers.
- Account query p95: 90 ms for single-account read with contacts suppressed.
- Duplicate-detection async completion p95: 3 s for 10K candidate tenant corpus.
- Availability: 99.95 percent monthly for command surface.
- Error budget burn triggers rollback when p95 exceeds 250 ms for 15 minutes.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: split imports into tenant-scoped chunks and resume from source cursor.
- Salesforce governor limits: back off with per-source token bucket and emit `EVT-CRM-ACCOUNT-SOURCE-THROTTLED`.
- Lead conversion conflict: hold account activation until lead, contact, and opportunity references agree.
- Duplicate legal entity collision: branch to merge-review workflow with no automatic destructive merge.
- Audit-chain seal timeout: persist pending mutation in sealed-later queue and block external visibility.
- Territory owner missing: create account in Draft and assign to tenant default queue.

## Migration Notes
- Salesforce CRM: Account.Id and OwnerId map to source_system_ref and owner_principal_id; record type maps to account_segment.
- SAP CRM: BUT000 partner number maps to source_system_ref; BP roles map to capability flags.
- Microsoft Dynamics 365 CE: accountid maps to source_system_ref; owningbusinessunit maps to territory path only after Cedar evaluation.
- HubSpot Sales Hub: companyId maps to source_system_ref; lifecycle stage maps to lifecycle_state with lossy-stage warning.
- Pipedrive: organization id maps to source_system_ref; owner user maps only when identity bridge exists.
- Zendesk Sell: company and person split maps to Account plus Contact with import provenance.

## Cross-Service Handoffs
- Marketplace receives only settlement-ready account refs, never raw account mutation authority.
- Payments receives invoice account identifiers only after contract/order acceptance.
- Community receives partner-channel account summary when partner visibility policy permits.
- Marketing-automation receives segment membership, not unrestricted account PII.
- Intelligence receives account scoring features through feature contracts with data_class labels.
- Ontology receives Account, Contact, Case, and Opportunity deltas for `Oyatie::Customer`.
- Workflow-engine owns merge-review, archive approval, and first-action gate closure.
- Audit-chain seals all state-changing events before external reads are enabled.

## Build Checklist
- Create domain value objects for AccountId, AccountName, AccountState, and SourceSystemRef.
- Enforce tenant_id on constructor, repository queries, events, and projections.
- Implement duplicate candidate detection as a domain service with deterministic scoring.
- Keep SQL adapter outside domain; domain accepts repository ports only.
- Add property tests for legal-name normalization and idempotent amendment.
- Add unit tests for forbidden Archived transitions.
- Add contract fixture for REST create body.
- Add contract fixture for gRPC CreateAccountRequest.
- Add AsyncAPI fixture for AccountCreated.
- Add Cedar fixture for denied cross-tenant create.
- Add Cedar fixture for territory owner delegation.
- Add audit fixture with ADR-0263 audit_id, trace_id, and schema_version.
- Add ontology fixture proving Customer projection fields.
- Add import fixture for Salesforce Account.
- Add import fixture for SAP Business Partner.
- Add import fixture for Dynamics Account.
- Add source replay test for 10K batch split.
- Add failure test for audit-chain unavailable.
- Add failure test for ontology projection retry.
- Add SLO dashboard row for account create latency.
- Add metric `oya:crm:account:create_latency_ms:histogram`.
- Add log field scrubber for legal_name and email-bearing contact previews.
- Add migration rollback note: drop indexes only after source replay snapshot is sealed.
- Stop when account create, amend, suspend, archive, import, and projection fixtures pass.

## Acceptance
- Account aggregate compiles without adapter dependencies.
- All 15 CRM entity types are represented in DDL and Rust type roster.
- REST, gRPC, and AsyncAPI examples validate against the command shape.
- Cedar hooks cover stage-advance, territory ownership, forecast approval impact, and partner visibility.
- Ontology projection names Salesforce Account, Contact, Case, and Opportunity deltas.
- Four-plus failure scenarios include Salesforce batch and governor-limit behavior.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Cross-service handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-001-domain-layer-for-account-master.md` matched [`SLO`, `p99`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/crm/IP-001-domain-layer-for-account-master.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/crm/IP-001-domain-layer-for-account-master.md` matched [`attribution`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/crm/IP-001-domain-layer-for-account-master.md`, `microservices/crm/manifest.json`, `microservices/crm/capacity-model.md`, `microservices/crm/compliance.md`, `microservices/crm/ARCHITECTURE.md`].
