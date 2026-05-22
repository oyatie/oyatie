---
doc_class: Implementation-Plan
ip_id: IP-013
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j154-tomas-pieter-channel-partner-co-marketing-launch
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-integrations + axis-erp-parity
---
# IP-013: Adapter Integrations For CRM

## Context
- This slice builds source-system adapters for Salesforce CRM, SAP CRM/C4C, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- SAP benchmark: SAP CRM/C4C OData and IDoc-like export/import surfaces.
- Salesforce benchmark: Bulk API 2.0, REST Composite, Change Data Capture, and governor-limit handling.
- Persona: Tomas Pieter, channel partner at PartnerLift B.V., whose shared campaign sync spans HubSpot and Salesforce.
- Journey leg: j154 shared lead-pool synchronization between PartnerLift HubSpot and Glacier Salesforce.
- Why now: domain/usecase layers need real import/export ports without embedding vendor SDKs inward.
- Vendor displacement covers Salesforce Sales/Service/Marketing Cloud, SAP CRM/C4C/Service Cloud, Dynamics 365 CE, Oracle Fusion CX, HubSpot, Zendesk Sell, Pipedrive, Freshsales, and ActiveCampaign.
- Adapter code depends inward on usecase ports; usecases never depend on adapter packages.
- Every adapter emits source cursor, batch checkpoint, row rejection, and audit evidence.
- Import is replayable and tenant-scoped.
- Export is policy-gated and never bulk dumps cross-tenant data.
- Adapter failures must leave deterministic checkpoint files for recovery.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), source_system TEXT, source_system_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, conversion_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, stage TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, marketplace_settlement_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, signature_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, status TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, knowledge_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, period_key TEXT NOT NULL, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, owner_principal_id UUID, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, portal_visibility TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, document_uri TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT, source_system_ref TEXT, locale TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.source_sync_checkpoint (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, source_system TEXT NOT NULL, object_name TEXT NOT NULL, cursor_value TEXT NOT NULL, batch_ceiling INT NOT NULL, audit_id TEXT NOT NULL);
CREATE UNIQUE INDEX IF NOT EXISTS crm_source_ref_unique ON crm.source_sync_checkpoint(tenant_id, source_system, object_name);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub audit_id: AuditId }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub source_system_ref: SourceSystemRef }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub conversion_state: ConversionState }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub stage: OpportunityStage }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub status: QuoteStatus }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub marketplace_settlement_ref: Option<String> }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub signature_state: SignatureState }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub status: CaseStatus }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub status: CampaignStatus }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub knowledge_ref: String }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub period_key: String }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub owner_principal_id: PrincipalId }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub portal_visibility: PortalVisibility }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub document_uri: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub source_system_ref: SourceSystemRef, pub locale: Locale }
pub trait CrmSourceAdapter { fn pull_batch(&self, cursor: SourceCursor, ceiling: BatchCeiling) -> AdapterBatch; fn map_row(&self, row: SourceRow) -> CrmUsecaseCommand; }
pub enum AdapterOutcome { BatchApplied, PartialRejected, Throttled, CursorAdvanced, Deadlettered }
```

## API Endpoints
- REST admin command: `POST /v1/crm/source-sync/jobs`.
- REST body: `{ "tenant_id": "glacier-partnerlift-q1-2027-mfg-de-nl-be", "source_system": "salesforce", "objects": ["Account", "Contact", "CampaignMember"], "batch_ceiling": 10000 }`.
- REST checkpoint query: `GET /v1/crm/source-sync/jobs/{job_id}/checkpoints`.
- REST response: `{ "job_id": "sync_154", "status": "Throttled", "next_cursor": "2026-05-20T00:00:00Z", "audit_event_class": "EVT-CRM-ADAPTER-SOURCE-THROTTLED" }`.
- gRPC worker: `rpc PullSourceBatch(PullSourceBatchRequest) returns (PullSourceBatchReply)`.
- gRPC worker: `rpc ApplyMappedCrmCommand(ApplyMappedCrmCommandRequest) returns (ApplyMappedCrmCommandReply)`.
- gRPC reply carries accepted_count, rejected_count, next_cursor, audit_id, policy_decision_id.
- AsyncAPI channel: `crm.source-sync.events.v1`.
- AsyncAPI message: `CrmSourceBatchApplied`.
- AsyncAPI body: `{ "tenant_id": "ten_partnerlift", "source_system": "hubspot", "object_name": "Company", "accepted": 9980, "rejected": 20, "audit_event_class": "EVT-CRM-ADAPTER-BATCH-APPLIED" }`.
- Adapter outputs usecase commands, not SQL writes.
- API forbids source-sync job without tenant migration permit.
- AsyncAPI deadletter channel records row-level rejection class.
- External secrets come from OpenBao adapter, never config files.

## Cedar Policy Hooks
- Stage-advance gate: imported stage changes pass through opportunity usecase policy.
- Territory ownership: imported owner or territory mapping must resolve to tenant principal or default queue.
- Forecast-roll-up approval: imported forecast categories require forecast period and manager approval.
- Partner-portal visibility: imported partner records default to hidden until partner contract policy permits.
- Source-sync principal/action/resource/context: principal is integration service, action is `crm.source.import`, resource is source object, context has source_system and batch ceiling.
- Export action `crm.source.export` requires auditor or migration role.
- Context includes source tenant, target tenant, DPA state, credential mode, source cursor, and row count.
- Denied rows write rejection evidence and do not call usecase.
- Governor-limit throttles emit policy-neutral throttle event.
- ADR-0263 audit_id links every checkpoint.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile` through account usecase command.
- Salesforce Contact maps to `Oyatie::Customer.primary_contacts` after consent remap.
- Salesforce Case maps to `Oyatie::Customer.service_posture` after narrative scrub.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline` after stage playbook mapping.
- Delta: adapter stores source cursor and source row hash for replay.
- Delta: source_system_ref is provenance only, never canonical identity.
- Delta: rejected rows are observable but not projected.
- Delta: cross-tenant shared campaign rows carry source_tenant and target_tenant.

## Workflow Steps
- Node `create_sync_job`: validate source, tenant, credential, and object roster.
- Node `open_source_cursor`: load source checkpoint or initial cursor.
- Node `pull_source_batch`: call vendor adapter under batch ceiling.
- Node `map_rows`: convert rows to usecase commands.
- Node `apply_usecases`: call account/contact/lead/opportunity/campaign/case usecases.
- Decision `governor_limit_hit`: branch to throttle and checkpoint.
- Node `record_rejections`: persist row-level rejection class.
- Node `advance_checkpoint`: commit cursor after accepted commands.
- Node `seal_batch_audit`: create ADR-0263 batch event.
- Node `publish_batch_event`: notify monitoring and ontology consumers.
- Branch `credential_expired`: block and request OpenBao rotation.
- Branch `schema_drift`: deadletter rows and continue safe objects.

## Audit Events
- `EVT-CRM-ADAPTER-SYNC-JOB-CREATED`.
- `EVT-CRM-ADAPTER-BATCH-PULLED`.
- `EVT-CRM-ADAPTER-BATCH-APPLIED`.
- `EVT-CRM-ADAPTER-BATCH-PARTIAL-REJECTED`.
- `EVT-CRM-ADAPTER-SOURCE-THROTTLED`.
- `EVT-CRM-ADAPTER-CHECKPOINT-ADVANCED`.
- `EVT-CRM-ADAPTER-CREDENTIAL-BLOCKED`.
- `EVT-CRM-ADAPTER-SCHEMA-DRIFT-DEADLETTERED`.
- ADR-0263 fields include audit_id, tenant_id, source_system, object_name, cursor_value, accepted, rejected, trace_id, span_id.

## SLO Targets
- Batch pull p95: 10K source rows in 120 s for Salesforce Bulk API.
- Row mapping p95: 10K rows in 30 s per worker.
- Checkpoint commit p95: 100 ms.
- Source throttle detection p95: 5 s from vendor error.
- Deadletter publish p95: 1 s from row rejection.
- Availability: 99.9 percent for sync control API.
- Rationale: adapter work is batch-oriented; correctness and checkpoint determinism outrank sub-second completion.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: enforce batch_ceiling and split large jobs.
- Salesforce governor limits: throttle and resume from SystemModstamp checkpoint.
- Lead conversion conflict: map row to conversion repair workflow rather than direct write.
- SAP OData delta token expiry: restart object from last sealed checkpoint with duplicate idempotency keys.
- HubSpot pagination drift: freeze after cursor and reconcile by updatedAt.
- Zendesk Sell schema drift: deadletter unknown fields and continue known object mapping.

## Migration Notes
- Salesforce CRM: Bulk API objects map to usecase commands with SystemModstamp cursor.
- SAP CRM/C4C: OData entity sets map to usecase commands with delta token checkpoint.
- Microsoft Dynamics 365 CE: Dataverse change tracking maps to source_sync_checkpoint.
- HubSpot Sales Hub: Companies, Contacts, Deals, Tickets, Lists map through updatedAt cursor.
- Pipedrive: organizations, persons, deals, activities map through pagination cursor.
- Zendesk Sell: companies, people, deals, tasks map with schema-drift guard.

## Cross-Service Handoffs
- Marketplace validates deal/order settlement refs imported from source systems.
- Payments validates invoice refs before financial handoff.
- Community receives partner-channel records only after policy permit.
- Marketing-automation receives campaign/member deltas after consent mapping.
- Intelligence receives source quality and rejection-rate features.
- Ontology receives projected Customer deltas after usecase application.
- OpenBao provides vendor credentials and rotation evidence.
- Audit-chain seals sync jobs, batches, checkpoints, and deadletters.

## Build Checklist
- Implement Salesforce adapter.
- Implement SAP CRM/C4C adapter.
- Implement Dynamics 365 CE adapter.
- Implement HubSpot adapter.
- Implement Pipedrive adapter.
- Implement Zendesk Sell adapter.
- Implement source checkpoint repository.
- Implement row rejection classifier.
- Add Salesforce 10K batch test.
- Add Salesforce governor-limit throttle test.
- Add SAP delta token expiry test.
- Add HubSpot pagination drift test.
- Add Zendesk schema drift test.
- Add REST sync job fixture.
- Add gRPC PullSourceBatch fixture.
- Add AsyncAPI BatchApplied fixture.
- Add Cedar import permit fixture.
- Add source credential expired fixture.
- Add lead conversion conflict fixture.
- Add ADR-0263 audit fixture.
- Add metric `oya:crm:adapter:batch_rows_total:counter`.
- Add metric `oya:crm:adapter:source_throttle_total:counter`.
- Add deadletter replay fixture.
- Stop when all six vendor adapters can dry-run and checkpoint deterministically.

## Acceptance
- Adapters call usecases and never write domain tables directly.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, delta expiry, pagination drift, and schema drift.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, OpenBao, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-013-adapter-integrations-for-crm.md` matched [`SLO`, `financial`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `iceberg_snapshot`].
- evidence_paths: [`microservices/crm/IP-013-adapter-integrations-for-crm.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
