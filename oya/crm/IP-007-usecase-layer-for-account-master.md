---
doc_class: Implementation-Plan
ip_id: IP-007
microservice: crm
related_adrs: [ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0251, ADR-0263, ADR-0297, ADR-0313, ADR-0314, ADR-0315, ADR-0317, ADR-0319]
journey_ref: docs/user-journeys/j100-pack-rollout-from-tenant-onboarding-to-first-action
capability_profile: T2-product-erp-parity
status: Accepted
date: 2026-05-20
owner_team: axis-crm + axis-application + axis-erp-parity
---
# IP-007: Account Master Usecase Layer

## Context
- This slice orchestrates account commands across domain, Cedar, repository, audit-chain, ontology, and workflow ports.
- SAP benchmark: SAP CRM business partner maintenance transaction flow.
- Salesforce benchmark: Sales Cloud Account save, validation rules, duplicate rules, and flow-triggered projection.
- Persona: Maya Chen, enterprise revenue-operations lead at Northwind Robotics.
- Journey leg: j100 first-action leg from tenant setup to first governed customer record.
- Why now: the domain from IP-001 is inert until usecases wire it to policy, idempotency, and durable events.
- Vendor displacement includes Salesforce flows, SAP C4C account maintenance, Dynamics account form logic, Oracle CX customer master, HubSpot company workflows, Pipedrive organizations, Zendesk Sell companies, Freshsales accounts, and ActiveCampaign account automations.
- The usecase layer is transaction boundary owner.
- The usecase layer must not contain SQL, REST DTOs, or adapter SDK types.
- Every command returns a typed outcome: Applied, Denied, Duplicate, PendingSeal, PendingProjection, or Blocked.
- Account merge-review is invoked through workflow-engine, not in-process manual reconciliation.
- This IP proves clean architecture direction under ADR-0105.

## Data Model Deltas
```sql
CREATE SCHEMA IF NOT EXISTS crm;
CREATE TABLE IF NOT EXISTS crm.account (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, legal_name TEXT NOT NULL, lifecycle_state TEXT NOT NULL, usecase_state TEXT NOT NULL DEFAULT 'Applied', audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contact (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), usecase_origin TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.lead (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, converted_account_id UUID, conversion_usecase_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.opportunity (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), account_gate_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.quote (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), account_gate_state TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.order_header (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), account_snapshot_version BIGINT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.contract (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), account_snapshot_version BIGINT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.case_record (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), account_snapshot_version BIGINT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.campaign (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_segment TEXT, account_filter_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.solution (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), recommended_bundle_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.forecast (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), account_rollup_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.territory (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, assignment_usecase_ref TEXT, owner_principal_id UUID, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.channel_partner (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), visibility_usecase_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.marketing_document (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_id UUID REFERENCES crm.account(id), generation_usecase_ref TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.email_template (id UUID PRIMARY KEY, tenant_id UUID NOT NULL, account_segment TEXT, usecase_origin TEXT, audit_id TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS crm.idempotency_record (tenant_id UUID NOT NULL, idempotency_key TEXT NOT NULL, command_name TEXT NOT NULL, result_ref TEXT NOT NULL, audit_id TEXT NOT NULL, PRIMARY KEY (tenant_id, idempotency_key, command_name));
CREATE INDEX IF NOT EXISTS crm_account_usecase_state_idx ON crm.account(tenant_id, usecase_state);
```
```rust
pub struct Account { pub id: AccountId, pub tenant_id: TenantId, pub usecase_state: UsecaseState, pub legal_name: String }
pub struct Contact { pub id: ContactId, pub tenant_id: TenantId, pub account_id: AccountId, pub usecase_origin: UsecaseOrigin }
pub struct Lead { pub id: LeadId, pub tenant_id: TenantId, pub converted_account_id: Option<AccountId>, pub conversion_usecase_ref: Option<String> }
pub struct Opportunity { pub id: OpportunityId, pub tenant_id: TenantId, pub account_id: AccountId, pub account_gate_state: GateState }
pub struct Quote { pub id: QuoteId, pub tenant_id: TenantId, pub account_id: AccountId, pub account_gate_state: GateState }
pub struct Order { pub id: OrderId, pub tenant_id: TenantId, pub account_id: AccountId, pub account_snapshot_version: u64 }
pub struct Contract { pub id: ContractId, pub tenant_id: TenantId, pub account_id: AccountId, pub account_snapshot_version: u64 }
pub struct Case { pub id: CaseId, pub tenant_id: TenantId, pub account_id: AccountId, pub account_snapshot_version: u64 }
pub struct Campaign { pub id: CampaignId, pub tenant_id: TenantId, pub account_segment: String, pub account_filter_ref: String }
pub struct Solution { pub id: SolutionId, pub tenant_id: TenantId, pub account_id: AccountId, pub recommended_bundle_ref: String }
pub struct Forecast { pub id: ForecastId, pub tenant_id: TenantId, pub account_id: AccountId, pub account_rollup_ref: String }
pub struct Territory { pub id: TerritoryId, pub tenant_id: TenantId, pub assignment_usecase_ref: String, pub owner_principal_id: PrincipalId }
pub struct ChannelPartner { pub id: ChannelPartnerId, pub tenant_id: TenantId, pub account_id: AccountId, pub visibility_usecase_ref: String }
pub struct MarketingDocument { pub id: MarketingDocumentId, pub tenant_id: TenantId, pub account_id: AccountId, pub generation_usecase_ref: String }
pub struct EmailTemplate { pub id: EmailTemplateId, pub tenant_id: TenantId, pub account_segment: String, pub usecase_origin: UsecaseOrigin }
pub struct AccountUsecasePorts { pub repo: AccountRepoPort, pub policy: CedarPort, pub audit: AuditChainPort, pub ontology: OntologyPort, pub workflow: WorkflowPort }
pub enum AccountUsecaseOutcome { Applied(AccountId), Denied(PolicyDecisionId), Duplicate(AccountId), PendingSeal(AccountId), PendingProjection(AccountId), Blocked(BlockerCode) }
```

## API Endpoints
- REST facade: `POST /v1/crm/accounts` invokes `CreateAccountUsecase`.
- REST body: `{ "tenant_id": "ten_northwind", "principal_id": "usr_maya", "legal_name": "Northwind Robotics GmbH", "territory_id": "terr_dach", "idempotency_key": "acct-usecase-1" }`.
- REST duplicate response: `{ "outcome": "Duplicate", "account_id": "acc_existing", "audit_event_class": "EVT-CRM-ACCOUNT-DUPLICATE-DETECTED" }`.
- REST pending response: `{ "outcome": "PendingProjection", "account_id": "acc_100", "retry_after_ms": 5000 }`.
- gRPC facade: `rpc CreateAccount(CreateAccountUsecaseRequest) returns (AccountUsecaseReply)`.
- gRPC facade: `rpc ArchiveAccount(ArchiveAccountUsecaseRequest) returns (AccountUsecaseReply)`.
- gRPC reply includes outcome, account_id, audit_id, policy_decision_id, workflow_run_id, ontology_projection_id.
- AsyncAPI channel: `crm.account.usecase.events.v1`.
- AsyncAPI message: `AccountUsecaseApplied`.
- AsyncAPI body: `{ "tenant_id": "ten_northwind", "account_id": "acc_100", "outcome": "Applied", "audit_event_class": "EVT-CRM-ACCOUNT-USECASE-APPLIED" }`.
- Usecase emits idempotency event when returning prior result.
- Usecase publishes outbox event inside same transaction as account mutation.
- REST facade translates typed outcomes to HTTP status.
- gRPC facade preserves typed outcome enum.
- AsyncAPI consumers never infer success from HTTP logs.

## Cedar Policy Hooks
- Stage-advance gate: usecase checks account lifecycle before opportunity can consume account as active.
- Territory ownership: usecase builds territory context from identity and territory ports.
- Forecast-roll-up approval: territory reassignment emits forecast-impact review event.
- Partner-portal visibility: usecase checks summary visibility before creating channel partner view.
- Usecase passes principal/action/resource/context to Cedar before domain command execution.
- Resource contains tenant_id, account_id, lifecycle_state, territory_id, portal_visibility.
- Context contains source_system, duplicate_score, import_mode, residency_pack, traceparent, idempotency_key.
- Denied outcomes are audited with no account write except idempotency denial record.
- Policy bundle version is captured on every outcome.
- Cedar network failure returns Blocked, not Allow.

## Ontology Projection
- Salesforce Account maps to `Oyatie::Customer.account_profile`.
- Salesforce Contact maps to `Oyatie::Customer.primary_contacts`.
- Salesforce Case maps to `Oyatie::Customer.service_posture`.
- Salesforce Opportunity maps to `Oyatie::Customer.revenue_pipeline`.
- Usecase emits account projection command after audit seal.
- Delta: usecase outcome is included so ontology can mark projection pending or current.
- Delta: duplicate candidate references are not exposed to customer-visible ontology views.
- Delta: tenant residency pack travels with projection command.

## Workflow Steps
- Node `receive_command`: normalize REST/gRPC command into domain command.
- Node `load_idempotency`: return prior result when key exists.
- Node `build_policy_context`: gather principal, territory, source, and duplicate score.
- Node `evaluate_cedar`: return Denied or continue.
- Node `execute_domain_command`: call IP-001 aggregate.
- Node `persist_account`: commit mutation and idempotency record.
- Node `seal_audit`: call audit-chain and record audit_id.
- Node `enqueue_outbox`: write account event inside transaction.
- Node `project_ontology`: call ontology port with retry policy.
- Branch `duplicate_high_confidence`: start workflow merge-review.
- Branch `audit_timeout`: mark PendingSeal and block external read.
- Branch `projection_timeout`: mark PendingProjection and continue retry.

## Audit Events
- `EVT-CRM-ACCOUNT-USECASE-RECEIVED`.
- `EVT-CRM-ACCOUNT-USECASE-IDEMPOTENT-REPLAYED`.
- `EVT-CRM-ACCOUNT-USECASE-POLICY-DENIED`.
- `EVT-CRM-ACCOUNT-USECASE-APPLIED`.
- `EVT-CRM-ACCOUNT-USECASE-PENDING-SEAL`.
- `EVT-CRM-ACCOUNT-USECASE-PENDING-PROJECTION`.
- `EVT-CRM-ACCOUNT-DUPLICATE-DETECTED`.
- `EVT-CRM-ACCOUNT-MERGE-REVIEW-STARTED`.
- ADR-0263 fields include tenant_id, subscope, usecase_name, outcome, audit_id, trace_id, span_id, schema_version.

## SLO Targets
- CreateAccount usecase p50: 70 ms.
- CreateAccount usecase p95: 230 ms.
- CreateAccount usecase p99: 700 ms when audit or ontology retries are needed.
- Idempotent replay p95: 40 ms from idempotency table.
- Outbox publish lag p95: 500 ms.
- Availability: 99.95 percent for account command usecases.
- Rationale: usecase adds ports to domain but must preserve interactive UI feel.

## Failure Modes and Recovery
- Salesforce Bulk API 10K batch ceiling: importer calls usecase per chunk and records idempotency keys.
- Salesforce governor limits: adapter backs off while usecase remains deterministic.
- Lead conversion conflict: usecase returns Blocked with conversion repair workflow id.
- Idempotency table conflict: return prior result after verifying command hash.
- Audit-chain unavailable: PendingSeal blocks external visibility and retries.
- Ontology projection timeout: PendingProjection exposes account only to privileged operator view.

## Migration Notes
- Salesforce CRM: Account import commands call CreateAccountUsecase with source_system_ref.
- SAP CRM: business partner import calls same usecase with BP role map.
- Microsoft Dynamics 365 CE: account import maps owningbusinessunit to territory context.
- HubSpot Sales Hub: company import carries lifecycle stage as non-authoritative hint.
- Pipedrive: organization import uses source owner only after identity bridge.
- Zendesk Sell: company/person imports split into account and contact usecases.

## Cross-Service Handoffs
- Marketplace receives only applied account IDs with audit_id.
- Payments receives account billing reference through order/contract, not account usecase.
- Community receives partner-visible account summary after policy permit.
- Marketing-automation receives segment signal after ontology projection.
- Intelligence receives account features after projection is current.
- Ontology receives account projection command and retry metadata.
- Workflow-engine receives duplicate merge-review work.
- Audit-chain seals every usecase outcome.

## Build Checklist
- Implement CreateAccountUsecase.
- Implement AmendAccountUsecase.
- Implement ArchiveAccountUsecase.
- Define AccountUsecasePorts trait bundle.
- Define AccountUsecaseOutcome enum.
- Implement idempotency command hash.
- Implement policy context builder.
- Implement audit seal retry state.
- Implement projection retry state.
- Add unit tests for each outcome.
- Add integration test with fake Cedar deny.
- Add integration test with fake audit timeout.
- Add integration test with fake ontology timeout.
- Add REST fixture for Applied.
- Add REST fixture for Duplicate.
- Add gRPC fixture for Blocked.
- Add AsyncAPI AccountUsecaseApplied fixture.
- Add Salesforce import chunk fixture.
- Add SAP BP import fixture.
- Add Dynamics import fixture.
- Add ADR-0263 audit fixture.
- Add metric `oya:crm:account_usecase:latency_ms:histogram`.
- Add outbox lag metric.
- Stop when all typed outcomes have deterministic tests.

## Acceptance
- Usecase has inward dependency on domain and outward dependency only through ports.
- All 15 CRM entities are present in DDL and Rust roster.
- REST, gRPC, and AsyncAPI examples include bodies.
- Cedar hooks cover stage advance, territory ownership, forecast approval, and partner visibility.
- Ontology projection maps Salesforce Account, Contact, Case, and Opportunity into `Oyatie::Customer`.
- Failure modes include Bulk API ceiling, governor limits, lead conversion conflict, idempotency conflict, audit outage, and projection timeout.
- Migration notes cover Salesforce, SAP, Dynamics 365 CE, HubSpot, Pipedrive, and Zendesk Sell.
- Handoffs include marketplace, payments, community, marketing-automation, intelligence, ontology, workflow-engine, and audit-chain.

## DR posture (per ADR-0343)

- ha_trigger_evidence: `microservices/crm/IP-007-usecase-layer-for-account-master.md` matched [`SLO`, `p99`, `payment`].
- applicable_compliance_pack_floor: [`SOX-404`, `SOC2-T2`, `ISO27001-2022`, `KR-PIPA-2023-amendment`] from `specs/compliance-pack-floors.json`; `manifest.json#dr` has no D-2 numeric override in this checkout.
- rto_p99_seconds_target: `14400`; rpo_p99_seconds_target: `900`.
- multi_region_active_active: `true`; floor_requires_active_active: `false`.
- backup_substrate: [`postgres_wal_g`, `object_storage_versioned`, `iceberg_snapshot`, `audit_chain_merkle_seal`].
- evidence_paths: [`microservices/crm/IP-007-usecase-layer-for-account-master.md`, `microservices/crm/manifest.json`, `microservices/crm/ARCHITECTURE.md`, `microservices/crm/PRD.md`, `microservices/crm/multi-region.md`, `microservices/crm/capacity-model.md`].
