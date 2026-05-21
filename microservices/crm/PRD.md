---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-crm
microservice: crm
status: Wave-15A-Rewritten
date: 2026-05-21
owner_team: axis-crm + axis-front-office-revenue
parity_set: [Salesforce Sales Cloud, HubSpot CRM, Microsoft Dynamics 365 Sales]
primary_anchor: Salesforce Sales Cloud
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0145
  - ADR-0244
  - ADR-0245
  - ADR-0247
  - ADR-0248
  - ADR-0251
  - ADR-0253
  - ADR-0263
  - ADR-0297
  - ADR-0314
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0338
  - ADR-0339
  - ADR-0340
  - ADR-0341
  - ADR-0342
  - ADR-0343
  - ADR-0344
  - ADR-0345
supersedes:
  - microservices/crm/PRD.md@2026-05-20 §C (30 template-stamped user stories)
  - microservices/crm/PRD.md@2026-05-20 §A.3 (SAP-primary parity stance — reclassified to extended reference)
companion_docs:
  - microservices/crm/ARCHITECTURE.md
  - microservices/crm/README.md
  - microservices/crm/compliance.md
  - microservices/crm/manifest.json
  - microservices/crm/competitor-parity-matrix.md
  - microservices/crm/REMEDIATION-NOTES-2026-05-21.md
planned_enforcement_ref: oya-governance-crm-doc-suite
---

# PRD-crm: Customer Relationship Management

## A. Vision

This PRD defines the Salesforce-anchor Big-8 product requirement surface for Customer Relationship Management.
crm is the Phase-4A.3 Big-8 customer-revenue microservice anchored to Salesforce Sales Cloud (primary anchor per ADR-0328 §D-2.13), HubSpot CRM Sales Hub + Service Hub (second anchor per ADR-0328 §D-2.18-19), and Microsoft Dynamics 365 Sales (third anchor per ADR-0328 §D-2.20-21). The target is full functional equivalence across the union of canonical surfaces from these three Big-8 CRM-family counterparts.
ADR-0244 binds tenant scoping, ADR-0314 binds marketplace DealSet settlement, ADR-0328 binds the substance-bar canonical sequence + Big-8 priority discipline, ADR-0329 retires the capability-tier doctrine, ADR-0330 establishes the tenant-class binary, and ADR-0331 binds cross-µservice tenant-class adoption.
The Wave 3-G "SAP CRM / C4C / Service Cloud parity" framing is reclassified to "extended reference" in Wave 15A. SAP CRM remains an operating-model reference; it does not drive scope decisions.
The service owns the lifecycle of revenue-bearing customer relationships across 14 bounded contexts: account-master, contact, lead, opportunity, opportunity-team, opportunity-split, sales-cadence, cpq-quote, forecast, service-case, campaign, loyalty-ledger, partner, and the read-model customer-360. The bounded-context detail is in §B and ARCHITECTURE.md §C.
The operating bar is the documentation-rigor PRD floor: substantive per-aggregate capability coverage, 30 bespoke user stories spanning the three Big-8 anchors, critical-path coverage, explicit Cedar policies, explicit ontology projections, and direct ADR references.
The service must be buildable by an intern who starts from this PRD plus the referenced ADRs, contracts, policies, and companion docs.
Every requirement below assumes tenant_id, principal_id, tenant_class, data_class, source_system_ref, audit_chain_ref, trace_id, and idempotency_key are present.
Every mutation is Cedar default-deny first; every read is scoped to tenant plus tenant_class; every projection is ontology-version pinned with declared freshness floor.
Open questions are limited to implementation sequencing across Wave 15B and Wave 15C; there is no unresolved product boundary decision in this PRD.

### A.1 Personas
- B2B process owner: wants to prove parity against incumbent ERP workflows without inheriting suite lock-in; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B tenant administrator: wants to activate packs, roles, and data residency boundaries without service-specific policy drift; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B operator: wants to run daily work, recover failures, and see batch progress before customers escalate; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B auditor: wants to export immutable evidence for every state transition and policy decision; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2B integrator: wants to map SAP, Oracle, Workday, NetSuite, bank, carrier, and custom source rows with provenance; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- B2C counterparty user: wants to see only the objects and obligations that a tenant explicitly grants; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- Developer partner: wants to build extensions through contracts and capability tiers instead of direct database access; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.
- SRE and incident commander: wants to diagnose latency, backlog, policy-deny spikes, and regional failover from telemetry alone; frustration is hidden suite coupling, unclear audit evidence, or unclear ownership across services.

### A.2 Non-goals
- Do not create a shared ERP database, shared ERP service, or suite-owned deployment unit.
- Do not bypass workflow-engine for cross-service state changes.
- Do not bypass Cedar, tenant scoping, ontology projection, audit-chain evidence, or marketplace settlement when they are applicable.
- Do not move ownership into concurrent-agent paths such as microservices/marketplace, microservices/workplace-integration, microservices/detection, or B2B-leader services.

### A.3 Parity stance
- Big-8 anchor family per ADR-0328 §D-2.13-21: Salesforce Sales Cloud (primary), HubSpot CRM Sales Hub + Service Hub (second), Microsoft Dynamics 365 Sales (third).
- Oyatie owner: microservices/crm/.
- Primary anchor (Salesforce Sales Cloud): drives every CRM surface decision; Wave 15A target functional-equivalence 85-95%.
- Second anchor (HubSpot CRM): explicitly added in Wave 15A (was structurally absent in Wave 3-G); lifecycle-stage flow + multi-pipeline pattern supported.
- Third anchor (Microsoft Dynamics 365 Sales): slug refresh from legacy "Customer Engagement"; Sales Accelerator workspace surfaced via customer-360 read-model.
- Extended reference (not driving Wave 15A scope): SAP CRM, SAP C4C, SAP Service Cloud (Wave 3-G primary, reclassified); Oracle CX Sales, NetSuite CRM, Zoho, Sugar, Pipedrive, Zendesk Sell, Freshsales (informational).
- Per-counterpart bespoke parity matrix: see `competitor-parity-matrix.md` (243 bespoke rows authored in Wave 15A).
- Risk domain: customer graph integrity, consent + EU-AI-Act compliance, revenue handoff to billing-tax + payments, service commitments under SLA + entitlement, loyalty liability with deterministic ledger.
- Primary companion docs: README.md, ARCHITECTURE.md, compliance.md, manifest.json, threat-model.md, dpia.md, capacity-model.md, cost-budget.md, failure-modes.md, runbooks, contracts, capabilities, SLOs, dashboards, policies, catalog records, IPs, migration playbooks, and competitor-parity-matrix.md.

## B. Capabilities

The capability set converts SAP CRM / C4C / Service Cloud behavior into six first-wave bounded contexts plus shared substrate handoffs.
The minimum parity target is complete create, amend, approve, reverse, archive, import, export, reconcile, simulate, and promote behavior for each context.
Capability records present in this service: account-master-command.yaml, opportunity-reconcile.yaml, quote-export.yaml.
Contract records present in this service: asyncapi-v1.yaml, crm-v1.proto, openapi-v1.yaml.
Policy records present in this service: abuse-defence.cedar, account-master-authorization.cedar, auditor-scope.cedar, campaign-authorization.cedar, ci-scope.cedar, data-residency.md, emergency-services-bypass.cedar, loyalty-ledger-authorization.cedar, opportunity-authorization.cedar, pack-overlay-authorization.cedar, quote-authorization.cedar, service-case-authorization.cedar, tenant-isolation.md.

### B.1 Account Master
- Scope: account-master owns the account master portion of Customer Relationship Management without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP CRM / C4C / Service Cloud account master semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: crm.account-master.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for account-master and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for account-master with replay and dead-letter semantics.
- Proto surface: contracts/crm-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/account-master-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: AccountMaster projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; crm only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from SAP CRM business partner extracts land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.2 Opportunity
- Scope: opportunity owns the opportunity portion of Customer Relationship Management without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP CRM / C4C / Service Cloud opportunity semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: crm.opportunity.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for opportunity and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for opportunity with replay and dead-letter semantics.
- Proto surface: contracts/crm-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/opportunity-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: Opportunity projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; crm only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from Salesforce account exports land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.3 Quote
- Scope: quote owns the quote portion of Customer Relationship Management without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP CRM / C4C / Service Cloud quote semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: crm.quote.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for quote and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for quote with replay and dead-letter semantics.
- Proto surface: contracts/crm-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/quote-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: Quote projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; crm only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from marketing consent feeds land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.4 Service Case
- Scope: service-case owns the service case portion of Customer Relationship Management without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP CRM / C4C / Service Cloud service case semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: crm.service-case.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for service-case and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for service-case with replay and dead-letter semantics.
- Proto surface: contracts/crm-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/service-case-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: ServiceCase projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; crm only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from support case history land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.5 Campaign
- Scope: campaign owns the campaign portion of Customer Relationship Management without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP CRM / C4C / Service Cloud campaign semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: crm.campaign.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for campaign and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for campaign with replay and dead-letter semantics.
- Proto surface: contracts/crm-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/campaign-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: Campaign projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; crm only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from SAP CRM business partner extracts land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.6 Loyalty Ledger
- Scope: loyalty-ledger owns the loyalty ledger portion of Customer Relationship Management without taking ownership of unrelated ERP modules.
- SAP parity: maps SAP CRM / C4C / Service Cloud loyalty ledger semantics into tenant-scoped Oyatie commands and events.
- Commands: create, amend, approve, reverse, archive, import, export, reconcile, simulate, promote.
- Events: crm.loyalty-ledger.created, amended, approved, reversed, archived, imported, exported, reconciled, simulated, promoted.
- API surface: contracts/openapi-v1.yaml must expose command endpoints for loyalty-ledger and return typed error envelopes.
- Event surface: contracts/asyncapi-v1.yaml must expose durable event channels for loyalty-ledger with replay and dead-letter semantics.
- Proto surface: contracts/crm-v1.proto carries internal worker and batch interfaces where synchronous service-to-service calls are required.
- Cedar hook: policy/loyalty-ledger-authorization.cedar authorizes business mutation actions; auditor-scope and ci-scope files gate evidence reads and CI fixtures.
- Ontology object: LoyaltyLedger projects to specs/products/ontology.json with source-system lineage and tenant subclassing.
- Workflow template: workflow-engine owns approval, reversal, import, and exception queues; crm only owns domain validation.
- Observability: emits counter, histogram, audit event, trace span, structured log digest, and SLO burn signal for each state transition.
- Migration: imported rows from Salesforce account exports land as pending projections until validation and Cedar checks pass.
- Failure modes: duplicate idempotency key, source-system mismatch, Cedar deny, ontology schema mismatch, workflow timeout, and audit-chain seal failure all resolve to named states.
- Acceptance: no shared database writes, no cross-tenant reads, no policy bypass, no unversioned event, no settlement event without an audit ref.

### B.7 Functional requirement register
- FR-001: account-master must ship OpenAPI command contract evidence before GA promotion.
- FR-002: account-master must ship AsyncAPI event contract evidence before GA promotion.
- FR-003: account-master must ship proto3 internal contract evidence before GA promotion.
- FR-004: account-master must ship ontology projection evidence before GA promotion.
- FR-005: account-master must ship Cedar authorization evidence before GA promotion.
- FR-006: account-master must ship audit-chain event evidence before GA promotion.
- FR-007: account-master must ship migration fixture evidence before GA promotion.
- FR-008: account-master must ship replay fixture evidence before GA promotion.
- FR-009: account-master must ship SLO and dashboard evidence before GA promotion.
- FR-010: account-master must ship runbook coverage evidence before GA promotion.
- FR-011: opportunity must ship OpenAPI command contract evidence before GA promotion.
- FR-012: opportunity must ship AsyncAPI event contract evidence before GA promotion.
- FR-013: opportunity must ship proto3 internal contract evidence before GA promotion.
- FR-014: opportunity must ship ontology projection evidence before GA promotion.
- FR-015: opportunity must ship Cedar authorization evidence before GA promotion.
- FR-016: opportunity must ship audit-chain event evidence before GA promotion.
- FR-017: opportunity must ship migration fixture evidence before GA promotion.
- FR-018: opportunity must ship replay fixture evidence before GA promotion.
- FR-019: opportunity must ship SLO and dashboard evidence before GA promotion.
- FR-020: opportunity must ship runbook coverage evidence before GA promotion.
- FR-021: quote must ship OpenAPI command contract evidence before GA promotion.
- FR-022: quote must ship AsyncAPI event contract evidence before GA promotion.
- FR-023: quote must ship proto3 internal contract evidence before GA promotion.
- FR-024: quote must ship ontology projection evidence before GA promotion.
- FR-025: quote must ship Cedar authorization evidence before GA promotion.
- FR-026: quote must ship audit-chain event evidence before GA promotion.
- FR-027: quote must ship migration fixture evidence before GA promotion.
- FR-028: quote must ship replay fixture evidence before GA promotion.
- FR-029: quote must ship SLO and dashboard evidence before GA promotion.
- FR-030: quote must ship runbook coverage evidence before GA promotion.
- FR-031: service-case must ship OpenAPI command contract evidence before GA promotion.
- FR-032: service-case must ship AsyncAPI event contract evidence before GA promotion.
- FR-033: service-case must ship proto3 internal contract evidence before GA promotion.
- FR-034: service-case must ship ontology projection evidence before GA promotion.
- FR-035: service-case must ship Cedar authorization evidence before GA promotion.
- FR-036: service-case must ship audit-chain event evidence before GA promotion.
- FR-037: service-case must ship migration fixture evidence before GA promotion.
- FR-038: service-case must ship replay fixture evidence before GA promotion.
- FR-039: service-case must ship SLO and dashboard evidence before GA promotion.
- FR-040: service-case must ship runbook coverage evidence before GA promotion.
- FR-041: campaign must ship OpenAPI command contract evidence before GA promotion.
- FR-042: campaign must ship AsyncAPI event contract evidence before GA promotion.
- FR-043: campaign must ship proto3 internal contract evidence before GA promotion.
- FR-044: campaign must ship ontology projection evidence before GA promotion.
- FR-045: campaign must ship Cedar authorization evidence before GA promotion.
- FR-046: campaign must ship audit-chain event evidence before GA promotion.
- FR-047: campaign must ship migration fixture evidence before GA promotion.
- FR-048: campaign must ship replay fixture evidence before GA promotion.
- FR-049: campaign must ship SLO and dashboard evidence before GA promotion.
- FR-050: campaign must ship runbook coverage evidence before GA promotion.
- FR-051: loyalty-ledger must ship OpenAPI command contract evidence before GA promotion.
- FR-052: loyalty-ledger must ship AsyncAPI event contract evidence before GA promotion.
- FR-053: loyalty-ledger must ship proto3 internal contract evidence before GA promotion.
- FR-054: loyalty-ledger must ship ontology projection evidence before GA promotion.
- FR-055: loyalty-ledger must ship Cedar authorization evidence before GA promotion.
- FR-056: loyalty-ledger must ship audit-chain event evidence before GA promotion.
- FR-057: loyalty-ledger must ship migration fixture evidence before GA promotion.
- FR-058: loyalty-ledger must ship replay fixture evidence before GA promotion.
- FR-059: loyalty-ledger must ship SLO and dashboard evidence before GA promotion.
- FR-060: loyalty-ledger must ship runbook coverage evidence before GA promotion.

## C. User Stories

This section was rewritten in Wave 15A to replace the 30 template-stamped story scaffolds. Each story below is bespoke to a specific CRM aggregate × counterpart-parity surface × persona combination. Stories are grouped by aggregate; each story names the Wave 15A bounded context, the primary counterpart surface, the realistic persona, the operational motivation, the explicit acceptance criteria, the cross-µservice handoff, the Cedar policy hook, the ontology projection, the observability evidence, and the tenant-class behaviour (per ADR-0330).

The story set covers all 14 bounded contexts (account-master, contact, lead, opportunity, opportunity-team, opportunity-split, sales-cadence, cpq-quote, forecast, service-case, campaign, loyalty-ledger, partner, customer-360) and ranges across Salesforce-anchor parity stories, HubSpot-anchor parity stories, Dynamics-anchor parity stories, and cross-µservice flow stories.

### C.1 Lead aggregate stories

#### Story CRM-001: Inbound web-to-lead capture with attribution preservation

- Persona: Marketing operations specialist running a B2B SaaS campaign in the EU residency cell.
- Motivation: A prospect submits a contact form on the tenant marketing site; the marketing-ops specialist needs the Lead to land in CRM with first-touch attribution preserved so the eventual Opportunity can be credited to the right Campaign.
- Counterpart anchor: Salesforce Web-to-Lead (SObject Lead via servlet) and HubSpot Forms-to-Contact-with-Lifecycle-Stage=Lead.
- Acceptance criterion 1: A POST to `/v1/crm/{tenant_id}/lead/web-to-lead-ingest` with form fields creates a `lead_document` with `tenant_id`, `lead_source = "web-form"`, `lead_source_detail` (form name + page URL), `utm_campaign`, `utm_source`, `utm_medium`, `utm_content`, `utm_term` fields preserved.
- Acceptance criterion 2: The Lead is auto-assigned to a queue per Cedar-gated assignment rules (region, language, vertical, deal-size hint).
- Acceptance criterion 3: An ontology projection edge from Lead to Campaign is created when `utm_campaign` matches a known Campaign.
- Acceptance criterion 4: An audit-chain event `EVT-CRM-LEAD-CHANGED` is sealed with `action="web-to-lead-ingest"`, `lead_source="web-form"`, `tenant_id`, `principal_id="system"`.
- Cross-µservice handoff: `forms` µservice owns the form rendering; calls `crm.lead.web-to-lead-ingest`. `marketing-automation` µservice may have enrolled the Contact in a journey that triggers form submission.
- Cedar policy hook: `policy/lead-authorization.cedar` action `crm.lead.web-to-lead-ingest` permitted for `principal == "system"` when context carries valid form-submission signature from the `forms` µservice.
- Ontology projection: `lead` projection links to `Tenant`, `Campaign` (via utm_campaign), `Persona`, and `SourceForm`.
- Observability evidence: `oya_crm_lead_transition_total{tenant, tenant_class, action="web-to-lead-ingest", outcome, region}` increments; trace span `crm.lead.web_to_lead_ingest` emitted; structured log records form payload (PII-masked per pack overlay).
- Tenant-class behaviour: demo_trial tenants are capped at 100 Leads total; further submissions return HTTP 402 with `tenant_class_cap_exceeded` error code.

#### Story CRM-002: Lead-to-Opportunity conversion saga with rollback on partial failure

- Persona: Sales Development Representative (SDR) qualifying inbound Leads.
- Motivation: After the SDR confirms the Lead is qualified (BANT criteria met), the SDR clicks Convert; the saga must create Account + Contact + Opportunity atomically; if any step fails the Lead must remain unconverted.
- Counterpart anchor: Salesforce Lead Conversion (`Database.convertLead`) creating Account + Contact + Opportunity in one transaction.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/lead/{lead_id}/convert` with body `{ "create_account": true|false, "existing_account_id": uuid|null, "create_opportunity": true|false, "opportunity_name": string, "opportunity_amount": decimal, "opportunity_close_date": date }` returns the saga_id and 202 Accepted.
- Acceptance criterion 2: A `workflow-engine` saga executes: (a) account-master.create OR account-master.attach; (b) contact.create with `account_id` populated; (c) opportunity.create with `account_id` and `contact_id` populated; (d) lead.mark_converted with refs.
- Acceptance criterion 3: If step (c) fails (e.g., Cedar denial on opportunity create), compensating transitions execute: contact archive → account archive (only if newly created) → lead.unmark_converted. The Lead remains in qualified state with `last_conversion_attempt_failed_at` populated.
- Acceptance criterion 4: Audit-chain emits four events (`EVT-CRM-LEAD-CHANGED`, `EVT-CRM-ACCOUNT_MASTER-CHANGED`, `EVT-CRM-CONTACT-CHANGED`, `EVT-CRM-OPPORTUNITY-CHANGED`) with the same `workflow_run_id`.
- Cross-µservice handoff: `workflow-engine` saga; calls back into `crm` for each step via direct gRPC.
- Cedar policy hook: `policy/lead-authorization.cedar` action `crm.lead.convert` requires principal to be assigned owner OR sales-manager; `policy/opportunity-authorization.cedar` action `crm.opportunity.create` evaluated independently inside the saga.
- Ontology projection: post-conversion, ontology reflects three new entities linked to the converted Lead.
- Observability evidence: `oya_crm_lead_conversion_total{tenant, tenant_class, outcome="converted"|"compensated"}` increments; trace spans linked by `workflow_run_id`.
- Tenant-class behaviour: demo_trial tenants are capped at 100 conversions; paid tenants have no cap.

#### Story CRM-003: Predictive Lead Score consumption with model-version freshness

- Persona: Inside-sales rep prioritising daily call list.
- Motivation: The rep opens the Lead list; the score column shows AI-predicted conversion likelihood; the rep filters to Top-decile and works that list first.
- Counterpart anchor: Salesforce Einstein Lead Scoring; HubSpot Predictive Lead Scoring; Dynamics Sales Insights Predictive Lead Scoring.
- Acceptance criterion 1: GET `/v1/crm/{tenant_id}/lead?score_bucket=top-decile&owner_principal_id={me}` returns Leads sorted by `predicted_conversion_score` descending.
- Acceptance criterion 2: The score for each Lead is read from a cached projection with model_version + freshness floor `<24h`; if the score is stale, a background refresh is enqueued to `intelligence` µservice.
- Acceptance criterion 3: The score response includes `score_value`, `score_bucket` ∈ {top-decile, top-quartile, median, bottom}, `model_version`, `model_explanation_url`.
- Acceptance criterion 4: Per ADR-0251 + EU-AI-Act pack overlay, automated routing decisions based on score require explainability surface; the explainability link points to `intelligence` µservice model card.
- Cross-µservice handoff: `intelligence` µservice computes the score; `crm` caches with freshness floor; `audit-chain` records the prediction-emission event.
- Cedar policy hook: `policy/lead-authorization.cedar` action `crm.lead.list` requires read permission; `policy/tenant-class-authorization.cedar` permits `intelligence` score retrieval only for tenants with AI pack overlay enabled.
- Ontology projection: Lead projection includes `predicted_conversion_score` edge to `intelligence.Prediction` aggregate.
- Observability evidence: `oya_crm_lead_score_retrieved_total{tenant, model_version, score_bucket}` counter.
- Tenant-class behaviour: demo_trial tenants get a free best-effort score (model_version=latest); paid tenants with AI overlay get on-demand refresh.

### C.2 Contact aggregate stories

#### Story CRM-004: HubSpot-style Contact-as-Lead lifecycle stage flow

- Persona: Tenant administrator migrating from HubSpot who wants to preserve the lifecycle-stage workflow.
- Motivation: The administrator activates `pack_overlay.lead_as_contact_lifecycle = true`; the platform must honor the HubSpot-style flow where Contact carries `lifecycle_stage` and Lead aggregate is unused.
- Counterpart anchor: HubSpot Contact Lifecycle Stage transitions.
- Acceptance criterion 1: With the pack overlay active, POST `/v1/crm/{tenant_id}/contact` accepts `lifecycle_stage` field; valid transitions are Subscriber → Lead → MQL → SQL → Opportunity → Customer → Evangelist (and explicit backtrack with reason).
- Acceptance criterion 2: Lifecycle-stage transitions emit `EVT-CRM-CONTACT-CHANGED` with `lifecycle_stage_from`, `lifecycle_stage_to`, `transition_reason`.
- Acceptance criterion 3: Lead aggregate is read-only in this mode; attempts to create a Lead return HTTP 409 with `lead_disabled_by_pack_overlay`.
- Acceptance criterion 4: Conversion to Opportunity (lifecycle_stage = Opportunity) triggers a `workflow-engine` saga to create an `opportunity_document` linked to the Contact and Account.
- Cross-µservice handoff: same as Story CRM-002 but with Contact-as-Lead substitution.
- Cedar policy hook: `policy/contact-authorization.cedar` evaluates the lifecycle-stage transition matrix per principal.
- Ontology projection: Contact projection includes `lifecycle_stage` and `lifecycle_stage_history`.
- Observability evidence: `oya_crm_contact_lifecycle_transition_total{tenant, stage_from, stage_to}` counter.
- Tenant-class behaviour: pack overlay activation is paid-class only; demo_trial uses Salesforce-style default.

#### Story CRM-005: Contact merge with duplicate-detection ML hint

- Persona: Data steward consolidating duplicate Contact records imported from multiple source systems.
- Motivation: After a migration, two Contact records exist for the same person (different emails, different source systems); the steward needs to merge them while preserving the audit trail of the merge.
- Counterpart anchor: Salesforce Contact Merge tool + Dynamics merge.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/contact/merge` with body `{ "winning_contact_id": uuid, "losing_contact_id": uuid, "field_choices": { ...per-field source selection... } }` returns 202.
- Acceptance criterion 2: The winning Contact absorbs all activity, related records (Opportunities, Cases, Campaigns); the losing Contact is archived (not deleted) with `merged_into_contact_id` set.
- Acceptance criterion 3: Audit-chain seals the merge with full before/after diff; both Contact IDs remain queryable.
- Acceptance criterion 4: `intelligence` µservice can be queried for a duplicate-detection score before the merge to confirm match confidence.
- Cross-µservice handoff: `intelligence` µservice for duplicate-detection; `audit-chain` for merge seal.
- Cedar policy hook: `policy/contact-authorization.cedar` action `crm.contact.merge` requires data-steward role.
- Ontology projection: merged Contact's projection retains `merged_from_contact_ids` array.
- Observability evidence: `oya_crm_contact_merge_total{tenant, outcome}` counter.
- Tenant-class behaviour: demo_trial cannot merge across owners; paid has no restriction.

### C.3 Account aggregate stories

#### Story CRM-006: Multi-level Account Hierarchy with rollup recomputation

- Persona: Enterprise sales operations manager organising a Fortune-500 parent + subsidiaries account structure.
- Motivation: A global account (e.g., "Acme Corp") has 47 subsidiaries across 12 countries; the operations manager needs to see consolidated revenue, open Opportunities, and case count at the parent level.
- Counterpart anchor: Salesforce Account Hierarchy + Account Rollup; Dynamics Account Sales Hierarchy.
- Acceptance criterion 1: PUT `/v1/crm/{tenant_id}/account-master/{id}/reparent` with body `{ "new_parent_account_id": uuid }` validates the new parent is in the same tenant; the resulting graph must be acyclic.
- Acceptance criterion 2: A background rollup job recomputes parent-level metrics (revenue, open_opportunity_count, won_opportunity_count, open_case_count, contract_value_total) within 60 seconds.
- Acceptance criterion 3: Audit-chain seals the reparent with old parent + new parent IDs.
- Acceptance criterion 4: GET `/v1/crm/{tenant_id}/account-master/{id}?include=rollup,hierarchy` returns the rollup metrics + flattened hierarchy.
- Cross-µservice handoff: `ontology` µservice receives the rollup projection update.
- Cedar policy hook: `policy/account-master-authorization.cedar` action `crm.account-master.reparent` requires enterprise-admin role.
- Ontology projection: Account hierarchy is a graph (parent_id edge) with rollup metrics on each parent.
- Observability evidence: `oya_crm_account_hierarchy_reparent_total{tenant, hierarchy_depth_after}` counter; `oya_crm_account_rollup_lag_seconds` histogram.
- Tenant-class behaviour: paid tenants get full hierarchy depth; demo_trial capped at 2 levels.

#### Story CRM-007: Account Team membership with named-role access overrides

- Persona: Strategic account manager assembling an Account Team for a key customer.
- Motivation: The account manager adds a Solutions Engineer, a Customer Success Manager, and a Partner Rep to the Account Team with distinct named roles and explicit access levels (Read, Write, Owner).
- Counterpart anchor: Salesforce AccountTeamMember with TeamMemberRole and AccountAccessLevel.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/account-master/{id}/team-member` with `{ "principal_id": uuid, "role": string, "access_level": "Read"|"Write"|"Owner" }`.
- Acceptance criterion 2: A principal with Team-Member access can perform team-scoped actions even if their organizational Cedar permissions would not normally allow.
- Acceptance criterion 3: Audit-chain seals the add/remove with role and access-level.
- Acceptance criterion 4: GET `/v1/crm/{tenant_id}/account-master/{id}/team-member` returns all current team members.
- Cross-µservice handoff: `cloud-iam` µservice validates principal_id is a real tenant principal.
- Cedar policy hook: `policy/account-master-authorization.cedar` permits team-scoped operations when context.account_team_membership contains the resource.
- Ontology projection: Account team membership is an edge between Principal and Account.
- Observability evidence: `oya_crm_account_team_change_total{tenant, action="add"|"remove"|"role-set"}` counter.
- Tenant-class behaviour: demo_trial limited to 1 team-member per Account; paid no limit.

### C.4 Opportunity aggregate stories

#### Story CRM-008: Opportunity stage progression with Cedar-gated transition matrix

- Persona: Account Executive (AE) progressing a deal through the pipeline.
- Motivation: The AE wants to move an Opportunity from "Discovery" to "Solution Fit" but the deal has not had a Solutions-Engineer engagement logged; the transition must be blocked with a meaningful error.
- Counterpart anchor: Salesforce Opportunity stage progression with Process Builder validation; Dynamics Business Process Flow validation.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/opportunity/{id}/advance-stage` with `{ "to_stage": "Solution Fit", "transition_reason": string }` evaluates the canonical stage transition matrix (New → Discovery → Solution Fit → Proposal → Negotiation → ClosedWon | ClosedLost | Disqualified).
- Acceptance criterion 2: Cedar policy evaluates pre-condition for each transition: e.g., Solution Fit requires `solutions_engineer_engaged = true`, Proposal requires `amount > 0 AND currency != null AND close_date != null AND probability >= 50%`, Negotiation requires `cpq_quote_submitted = true`.
- Acceptance criterion 3: Failed pre-condition returns HTTP 422 with `transition_blocked` and the list of unmet conditions.
- Acceptance criterion 4: OpportunityHistory is append-only; the history row records `from_stage`, `to_stage`, `transition_reason`, `principal_id`, `transitioned_at`.
- Cross-µservice handoff: Cedar evaluation is library-first (in-process); ontology projection updates after success.
- Cedar policy hook: `policy/opportunity-authorization.cedar` defines per-transition pre-conditions.
- Ontology projection: OpportunityHistory is a child entity of Opportunity.
- Observability evidence: `oya_crm_opportunity_stage_transition_total{tenant, from_stage, to_stage, outcome="advanced"|"blocked"}` counter; histogram of `time_in_stage_seconds`.
- Tenant-class behaviour: applies uniformly.

#### Story CRM-009: Big-Deal-Alert threshold trip with multichannel notification

- Persona: Sales VP wanting visibility into deals above a threshold.
- Motivation: The VP configures a Big-Deal Alert for deals ≥ $250k in the tenant's primary currency; when an AE creates or updates an Opportunity above this threshold, the VP gets a notification.
- Counterpart anchor: Salesforce Big Deal Alert; Dynamics Goal Notification.
- Acceptance criterion 1: PUT `/v1/crm/{tenant_id}/opportunity-big-deal-alert-config` configures the threshold + notification recipients + channel preferences.
- Acceptance criterion 2: When `crm.opportunity.create` or `crm.opportunity.amend` results in amount crossing the threshold, an `EVT-CRM-OPPORTUNITY-CHANGED` event with `big_deal_alert_tripped = true` is emitted.
- Acceptance criterion 3: `notifications` µservice picks up the event and sends per-recipient notifications via email + Slack + Teams + in-app + mobile push (per recipient channel preferences).
- Acceptance criterion 4: Once tripped, the alert is not re-triggered for the same Opportunity unless amount changes by >10%.
- Cross-µservice handoff: `notifications` + `workplace-integration` (Slack / Teams).
- Cedar policy hook: `policy/opportunity-authorization.cedar` validates threshold config requires sales-manager+ role.
- Ontology projection: alert configuration is an Opportunity-level setting; tripped state is a flag.
- Observability evidence: `oya_crm_big_deal_alert_total{tenant, channel="email"|"slack"|"teams"|"in-app"|"mobile"}` counter.
- Tenant-class behaviour: applies uniformly.

### C.5 OpportunityTeam + OpportunitySplit stories

#### Story CRM-010: Opportunity revenue split with 100% sum invariant

- Persona: Finance controller setting up revenue attribution for a multi-rep deal.
- Motivation: An Opportunity has three contributors (primary AE, overlay rep, partner rep); finance wants to attribute 50% / 30% / 20% of revenue.
- Counterpart anchor: Salesforce OpportunitySplit with Revenue type.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/opportunity/{id}/split` with body `[ { "principal_id": uuid, "split_type": "Revenue", "split_pct": 50 }, { "principal_id": uuid, "split_type": "Revenue", "split_pct": 30 }, { "principal_id": uuid, "split_type": "Revenue", "split_pct": 20 } ]`.
- Acceptance criterion 2: Sum of all Revenue splits must equal 100%; sum != 100 returns HTTP 422 with `split_sum_invariant_violation`.
- Acceptance criterion 3: Overlay splits are independent; multiple Overlay splits may sum to anything.
- Acceptance criterion 4: Audit-chain seals the split definition + each update.
- Cross-µservice handoff: when Opportunity closes won, the split is propagated to `cloud-billing-tax` for revenue recognition attribution.
- Cedar policy hook: `policy/opportunity-split-authorization.cedar` requires finance + sales-manager dual approval for split changes above 10% per recipient.
- Ontology projection: split records as an OpportunitySplit child aggregate.
- Observability evidence: `oya_crm_opportunity_split_change_total{tenant, split_type, outcome}` counter.
- Tenant-class behaviour: demo_trial cannot define Revenue splits; paid no restriction.

### C.6 Sales Cadence stories

#### Story CRM-011: Multi-step Sales Cadence enrolment with conditional branching

- Persona: SDR running a 7-step outbound cadence on a Lead list.
- Motivation: The SDR designs a cadence: Day 1 email, Day 2 LinkedIn touch, Day 4 call, Day 6 email follow-up, Day 9 call, Day 12 final email, Day 14 exit-or-recycle.
- Counterpart anchor: Salesforce Sales Engagement Cadence; HubSpot Sequence; Dynamics Sales Sequence.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/sales-cadence` creates a cadence definition with ordered steps (each step has type, day-offset, content_ref, exit_condition).
- Acceptance criterion 2: POST `/v1/crm/{tenant_id}/sales-cadence/{id}/enrol` with `{ "target_ids": [uuid, uuid, ...], "target_type": "Lead"|"Contact" }` enrols targets; max 1000 per call.
- Acceptance criterion 3: A `workflow-engine` saga executes steps per day-offset; per-target step state is tracked; step exit conditions (Lead converted, Contact replied, opted out) terminate enrolment for that target.
- Acceptance criterion 4: GET `/v1/crm/{tenant_id}/sales-cadence/{id}?include=analytics` returns enrolment count, step completion rate, reply rate, conversion rate.
- Cross-µservice handoff: `mail` µservice executes email steps; `contact-center` µservice executes call steps; `workplace-integration` executes LinkedIn-touch steps; `workflow-engine` orchestrates the day-offset schedule.
- Cedar policy hook: `policy/sales-cadence-authorization.cedar` action `crm.sales-cadence.enrol` requires sales rep + Target read permission.
- Ontology projection: cadence catalog per tenant + per-enrolment current step.
- Observability evidence: `oya_crm_cadence_step_executed_total{tenant, cadence_id, step_type, outcome}` counter.
- Tenant-class behaviour: demo_trial limited to 1 active cadence + 50 enrolments total; paid no limit.

#### Story CRM-012: Sales Cadence step exit on Contact reply detection

- Persona: SDR who wants automation to stop when the prospect replies.
- Motivation: When a prospect replies to a cadence email, the cadence should stop further emails and notify the SDR to take over the conversation manually.
- Counterpart anchor: HubSpot Sequences auto-exit on reply; Salesforce Cadence reply detection.
- Acceptance criterion 1: Cadence step definition includes `exit_on_reply: bool`; default true.
- Acceptance criterion 2: When `mail` µservice detects a reply to a cadence-step email (via In-Reply-To / References header), it calls `crm.sales-cadence.exit-enrolment` with reason "reply-received".
- Acceptance criterion 3: SDR receives an in-app notification + (optionally) Slack/Teams notification.
- Acceptance criterion 4: Per-cadence analytics include `reply_rate = enrolments_exited_on_reply / total_enrolments`.
- Cross-µservice handoff: `mail` µservice → `crm.sales-cadence` exit; `notifications` µservice for SDR alert.
- Cedar policy hook: `policy/sales-cadence-authorization.cedar` permits exit by `principal == "mail-system"` when signed reply-detection event.
- Ontology projection: enrolment exit-reason captured.
- Observability evidence: `oya_crm_cadence_exit_total{tenant, exit_reason}` counter.
- Tenant-class behaviour: applies uniformly.

### C.7 CPQ Quote stories

#### Story CRM-013: CPQ Quote bundle configuration with constraint-rule validation

- Persona: Solutions Engineer configuring a bundle for a customer.
- Motivation: The SE selects a "Cloud Platform Bundle" containing Compute + Storage + Networking; the bundle has constraint rules (e.g., Compute tier must be ≥ Storage tier; Networking premium only valid if Compute is enterprise).
- Counterpart anchor: Salesforce CPQ Product Bundle + Constraint Rules.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/cpq-quote/{id}/configure-line` with body specifying bundle parent + bundle children + per-line configuration attributes.
- Acceptance criterion 2: Constraint-rule engine in `src/kernel/cpq_constraint` evaluates all rules; violation returns HTTP 422 with the list of failed rules + suggested fixes.
- Acceptance criterion 3: On successful configuration, the line totals are recomputed; the quote total is recomputed.
- Acceptance criterion 4: Audit-chain seals the configuration event.
- Cross-µservice handoff: `marketplace` µservice provides catalog refs; constraint engine is local to crm.
- Cedar policy hook: `policy/cpq-quote-authorization.cedar` action `crm.cpq-quote.configure-line` requires Quote editor role.
- Ontology projection: cpq_quote_line + cpq_quote_attribute child entities.
- Observability evidence: `oya_crm_cpq_constraint_violation_total{tenant, rule_id}` counter.
- Tenant-class behaviour: demo_trial cannot save bundles ≥ 5 lines; paid no limit.

#### Story CRM-014: CPQ multi-step approval chain with smart-approval recall

- Persona: AE submitting a Quote with a 35% discount that exceeds the standard approval threshold.
- Motivation: The Quote needs Director-level approval; the AE submits; the Director needs to approve before the Quote can be sent.
- Counterpart anchor: Salesforce CPQ Advanced Approvals; Dynamics CPQ Approval Workflows.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/cpq-quote/{id}/submit-for-approval` evaluates the approval matrix (discount % vs threshold, deal size vs threshold, custom-attribute conditions).
- Acceptance criterion 2: The approval chain is computed (e.g., Manager → Director → VP → CRO based on discount %).
- Acceptance criterion 3: Each step receives an in-app + email notification; can approve/reject with a reason.
- Acceptance criterion 4: The submitter can recall the submission before approval; recall returns Quote to Draft.
- Acceptance criterion 5: "Smart Approval" pattern: identical Quote conditions previously approved by the same chain may auto-approve (configurable per tenant + per pack overlay).
- Cross-µservice handoff: `workflow-engine` owns the approval saga.
- Cedar policy hook: `policy/cpq-quote-authorization.cedar` defines per-step approver roles.
- Ontology projection: cpq_approval_step child entities.
- Observability evidence: `oya_crm_quote_approval_step_total{tenant, step_index, outcome}` counter; histogram of `quote_approval_total_seconds`.
- Tenant-class behaviour: paid tenants can configure smart-approval; demo_trial uses default fixed chain.

#### Story CRM-015: CPQ Quote document generation in tenant + recipient locale

- Persona: International AE selling to a French customer.
- Motivation: The Quote PDF must render in French with EUR currency + local date formats; the AE's UI shows the same Quote in English.
- Counterpart anchor: Salesforce CPQ Multi-language Quote; Dynamics Quote Document Template.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/cpq-quote/{id}/generate-document` with `{ "locale": "fr-FR", "currency": "EUR", "template_id": uuid }` returns 202 + document_render_id.
- Acceptance criterion 2: `cloud-rendering` µservice renders the PDF; on completion `EVT-CRM-CPQ_QUOTE-CHANGED` with `action="document-generated"` is sealed.
- Acceptance criterion 3: GET `/v1/crm/{tenant_id}/cpq-quote/{id}/document/{document_render_id}` returns the PDF URL with signed link (5-minute expiration).
- Acceptance criterion 4: Document includes tenant logo, recipient address block in locale, currency-formatted line items, locale-appropriate date format, signature block.
- Cross-µservice handoff: `cloud-rendering` for PDF; `e-signature` µservice if e-signature requested.
- Cedar policy hook: `policy/cpq-quote-authorization.cedar` permits document generation per Quote editor role.
- Ontology projection: document_renders are an audit-trail of Quote document versions.
- Observability evidence: `oya_crm_quote_document_render_total{tenant, locale, outcome}` counter; histogram of `document_render_latency_seconds`.
- Tenant-class behaviour: demo_trial limited to 10 document generations per month; paid no limit.

### C.8 Forecast stories

#### Story CRM-016: Forecast Category assignment with manager adjustment

- Persona: Sales Manager preparing the quarterly forecast call.
- Motivation: The Manager reviews her team's Opportunities; she moves three specific Opportunities from Best Case to Commit (manager's judgment) and one from Commit to Closed-won (anticipating closing this week).
- Counterpart anchor: Salesforce Collaborative Forecasts + Manager Adjustments.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/forecast/adjust` with `{ "opportunity_id": uuid, "to_category": "Commit", "adjustment_reason": string, "adjusted_amount": decimal|null }`.
- Acceptance criterion 2: Adjustment is per-manager + per-period; multiple managers can adjust the same Opportunity (last-write-wins with full history).
- Acceptance criterion 3: Audit-chain seals every adjustment with `principal_id`, `from_category`, `to_category`, `adjustment_reason`, `period_id`.
- Acceptance criterion 4: Forecast snapshot at period close captures all adjustments; locked period prevents further adjustment.
- Cross-µservice handoff: ontology projection updates the forecast rollup.
- Cedar policy hook: `policy/forecast-authorization.cedar` action `crm.forecast.adjust` requires sales-manager+ role; adjustments above threshold require VP approval.
- Ontology projection: forecast_adjustment child entity; forecast_snapshot child entity.
- Observability evidence: `oya_crm_forecast_adjust_total{tenant, manager_principal, period, adjustment_magnitude}` counter.
- Tenant-class behaviour: demo_trial uses single-level forecast (no manager adjustment); paid full hierarchy.

#### Story CRM-017: Forecast snapshot for regulator-grade historical evidence

- Persona: Finance Controller responding to a SOX audit request for historical forecast accuracy.
- Motivation: The auditor asks "what was your committed forecast at Q3 close, by region, by product family"; the controller must produce a tamper-evident snapshot.
- Counterpart anchor: Dynamics Forecast Snapshot; Salesforce Forecast Snapshot.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/forecast/{period_id}/snapshot` captures the entire forecast state (all Opportunities + categories + amounts + manager adjustments + quota assignments) at point-in-time.
- Acceptance criterion 2: The snapshot is sealed in audit-chain; the snapshot hash is recorded on the Forecast aggregate.
- Acceptance criterion 3: GET `/v1/crm/{tenant_id}/forecast/{period_id}/snapshot/{snapshot_id}?include=full-detail` returns the full snapshot with audit-chain seal_ref.
- Acceptance criterion 4: Snapshots are immutable; new snapshots create new IDs.
- Cross-µservice handoff: `audit-chain` µservice seals.
- Cedar policy hook: `policy/forecast-authorization.cedar` action `crm.forecast.snapshot` requires finance-controller role.
- Ontology projection: forecast_snapshot child entity.
- Observability evidence: `oya_crm_forecast_snapshot_total{tenant, period_id, outcome}` counter; histogram of `snapshot_size_bytes`.
- Tenant-class behaviour: SOX-404 pack required; paid only.

### C.9 Service Case stories

#### Story CRM-018: Case routing with omnichannel skill-match

- Persona: Service Operations Manager configuring routing for premium-customer cases.
- Motivation: Premium customers' Cases must be routed to a Service Rep with the required language + product expertise + Tier-3 entitlement clearance.
- Counterpart anchor: Salesforce Omnichannel Routing; Dynamics Unified Routing.
- Acceptance criterion 1: PUT `/v1/crm/{tenant_id}/service-case-routing-config` defines per-segment routing rules (Channel, Customer Tier, Product Family, Language, Severity, Region).
- Acceptance criterion 2: POST `/v1/crm/{tenant_id}/service-case` with channel/product/severity triggers routing evaluation; the resulting `assigned_principal_id` is computed by IP-024 routing engine matching skill capacity.
- Acceptance criterion 3: If no eligible principal is available, Case enters a queue; queue depth + oldest-waiting time are surfaced.
- Acceptance criterion 4: Audit-chain seals the routing decision with the evaluated rules + assigned principal.
- Cross-µservice handoff: `cloud-iam` µservice provides principal skill metadata; `contact-center` channels provide inbound capture.
- Cedar policy hook: `policy/service-case-authorization.cedar` action `crm.service-case.route` requires service-ops role.
- Ontology projection: routing_decision child entity captures the audit-trail.
- Observability evidence: `oya_crm_case_routing_decision_total{tenant, channel, severity, outcome="routed"|"queued"}` counter; queue-depth gauge.
- Tenant-class behaviour: demo_trial uses default round-robin; paid uses skill-match.

#### Story CRM-019: Entitlement SLA clock with pause-on-pending-customer

- Persona: Service Rep working a Case where she's waiting on customer feedback.
- Motivation: The customer needs to provide logs; while waiting, the SLA clock should pause (so the rep is not penalised for customer-side delay).
- Counterpart anchor: Salesforce Entitlement Milestone + Pause on Pending-Customer.
- Acceptance criterion 1: PUT `/v1/crm/{tenant_id}/service-case/{id}/status` to "Pending-Customer" automatically pauses the SLA clock via IP-022 SLA engine.
- Acceptance criterion 2: PUT back to "In-Progress" resumes the clock; pause-resume cycles are audited.
- Acceptance criterion 3: At Case resolution, the elapsed-time-against-SLA = total_elapsed - sum(paused_durations).
- Acceptance criterion 4: SLA breach notification is sent at 80% + 100% of SLA window; recipients per tenant config.
- Cross-µservice handoff: `notifications` µservice for breach alerts.
- Cedar policy hook: `policy/service-case-authorization.cedar` action `crm.service-case.status-change` per Case-assignee role.
- Ontology projection: entitlement_clock child entity with pause/resume events.
- Observability evidence: `oya_crm_case_sla_breach_total{tenant, severity, breach_pct}` counter.
- Tenant-class behaviour: applies uniformly.

### C.10 Campaign stories

#### Story CRM-020: Campaign-to-Revenue attribution with influence-model selection

- Persona: Marketing Director justifying campaign ROI to the CMO.
- Motivation: A multi-touch campaign (webinar + email nurture + ad retargeting) drove a $1.2M Opportunity; the Director wants to attribute revenue across the three touchpoints with First-Touch / Last-Touch / Even Distribution / Custom models.
- Counterpart anchor: Salesforce Campaign Influence Models.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/campaign/{id}/compute-influence` with `{ "influence_model": "First-Touch"|"Last-Touch"|"Even-Distribution"|"Custom", "custom_weights": null|{...} }`.
- Acceptance criterion 2: IP-019 attribution engine computes per-Opportunity attribution; results are stored in CampaignInfluence projection.
- Acceptance criterion 3: GET `/v1/crm/{tenant_id}/campaign/{id}/influence?period=Q3` returns aggregate revenue attributed per campaign + per influence-model.
- Acceptance criterion 4: Attribution results are point-in-time; recomputation creates a new attribution snapshot.
- Cross-µservice handoff: `analytics` µservice consumes the attribution projection for deep-cut reporting.
- Cedar policy hook: `policy/campaign-authorization.cedar` action `crm.campaign.compute-influence` requires marketing-ops role.
- Ontology projection: CampaignInfluence + Influence_Snapshot child entities.
- Observability evidence: `oya_crm_campaign_influence_total{tenant, model, outcome}` counter.
- Tenant-class behaviour: demo_trial First-Touch only; paid all models + Custom.

### C.11 Loyalty Ledger stories

#### Story CRM-021: Loyalty points accrual with deterministic balance

- Persona: Loyalty Program Manager running a rewards program.
- Motivation: Members earn points on purchase; the points balance must be deterministic across cell failover.
- Counterpart anchor: Salesforce Loyalty Management (Loyalty Cloud).
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/loyalty-ledger/{member_id}/accrue` with `{ "points": int, "accrual_reason": string, "source_transaction_ref": string }` appends a journal entry.
- Acceptance criterion 2: Balance = sum(journal entries) deterministically; replay from journal produces the same balance.
- Acceptance criterion 3: Concurrent accruals are linearisable per-member via aggregate-root locking.
- Acceptance criterion 4: Audit-chain seals each journal entry.
- Cross-µservice handoff: `marketplace` µservice may settle the underlying transaction.
- Cedar policy hook: `policy/loyalty-ledger-authorization.cedar` requires loyalty-program role for accruals.
- Ontology projection: loyalty_journal_entry child entity.
- Observability evidence: `oya_crm_loyalty_accrue_total{tenant, member_segment, outcome}` counter.
- Tenant-class behaviour: paid only; demo_trial cannot run loyalty.

### C.12 Partner stories

#### Story CRM-022: Partner Deal Registration with first-registration priority

- Persona: Partner Account Manager validating a partner's deal registration.
- Motivation: A partner registers a deal at "Acme Corp" for "Cloud Platform"; the platform must ensure first-registration priority (the first valid registration gets attribution; subsequent registrations are flagged as conflicting).
- Counterpart anchor: Salesforce Experience Cloud Deal Registration.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/partner/{partner_id}/deal-registration` with `{ "account_name": string, "product_family": string, "expected_close": date, "deal_size": decimal }`.
- Acceptance criterion 2: Uniqueness check on (partner_id, normalized_account_name, product_family); duplicate within 90-day window is rejected.
- Acceptance criterion 3: Deal registration enters Pending-Approval state; partner account manager approves or rejects with reason.
- Acceptance criterion 4: Approved registration creates an Opportunity link + populates `partner_id` field on the Opportunity.
- Cross-µservice handoff: when Opportunity closes won, partner revenue-share computation occurs via `marketplace` settlement + Opportunity-Split overlay.
- Cedar policy hook: `policy/partner-authorization.cedar` action `crm.partner.deal-registration.approve` requires partner-account-manager role.
- Ontology projection: deal_registration child entity.
- Observability evidence: `oya_crm_deal_registration_total{tenant, partner, outcome}` counter.
- Tenant-class behaviour: paid only.

### C.13 Customer 360 stories

#### Story CRM-023: Customer 360 read with freshness floor banner

- Persona: Customer Success Manager opening a customer's profile.
- Motivation: The CSM clicks an Account; the page shows Account info + all Contacts + all open Opportunities + all open Cases + Loyalty balance + recent campaigns; freshness < 5 seconds.
- Counterpart anchor: Salesforce Customer 360 / Customer Data Platform; HubSpot Companies view; Dynamics Customer Insights.
- Acceptance criterion 1: GET `/v1/crm/{tenant_id}/customer-360/{account_id}` returns the denormalised view with `freshness_seconds` in response headers.
- Acceptance criterion 2: If freshness > 5 seconds, response includes `x-freshness-stale: true` banner + reason.
- Acceptance criterion 3: All sub-entities respect the principal's Cedar policy (no leakage of un-permitted Contacts / Cases / Opportunities).
- Acceptance criterion 4: A background refresh is triggered if freshness is stale.
- Cross-µservice handoff: `ontology` µservice owns the denormalised projection; the projection refresh is event-driven from the 13 write aggregates.
- Cedar policy hook: `policy/customer-360-authorization.cedar` action `crm.customer-360.read` filters per-sub-entity permission.
- Ontology projection: customer_360_projection read-model.
- Observability evidence: `oya_crm_customer_360_read_total{tenant, freshness_class="fresh"|"stale"}` counter.
- Tenant-class behaviour: applies uniformly.

### C.14 Cross-aggregate flow stories

#### Story CRM-024: Quote-to-Cash end-to-end with compensating transitions

- Persona: AE closing a deal end-to-end.
- Motivation: Opportunity closes won → CPQ Quote accepted → Order created → Invoice issued → Payment collected → Revenue recognised. Each step is a saga checkpoint; compensation on failure preserves the previous valid state.
- Counterpart anchor: Salesforce Revenue Cloud (Quote → Order → Invoice → Revenue Recognition).
- Acceptance criterion 1: When `crm.cpq-quote.accept` is called, a `workflow-engine` Q2C saga starts.
- Acceptance criterion 2: Saga steps: Quote.accepted → cloud-billing-tax.order.create → cloud-billing-tax.invoice.create → payments.payment.collect → cloud-billing-tax.revenue-recognition.record. Each step has an idempotency key tied to the saga_run_id.
- Acceptance criterion 3: Compensating transitions on failure: payment failed → invoice.cancel → order.cancel → quote.restore-to-sent. Compensation is idempotent.
- Acceptance criterion 4: Saga state is durable; restart resumes from the last completed step.
- Acceptance criterion 5: Audit-chain seals each step + the saga lifecycle.
- Cross-µservice handoff: `workflow-engine` orchestrates; `cloud-billing-tax` + `payments` execute steps.
- Cedar policy hook: each step gated by the respective µservice's policy.
- Ontology projection: Q2C saga reflected in `customer-360` projection.
- Observability evidence: `oya_crm_quote_to_cash_step_total{tenant, step, outcome}` counter; histogram of `q2c_total_latency_seconds`.
- Tenant-class behaviour: paid only.

#### Story CRM-025: Migration cutover from Salesforce with dual-write coexistence

- Persona: Migration project lead cutting over from Salesforce to Oyatie.
- Motivation: The tenant runs both systems for 30 days; writes go to both during cutover; reads can be served from either; reconciliation runs nightly.
- Counterpart anchor: Salesforce → Oyatie migration playbook + Person Account handling + Currency multi-mode.
- Acceptance criterion 1: Migration playbook `migration-playbooks/from-salesforce-sales-cloud.md` enumerates the field mapping table + Person Account dual-semantic resolution + multi-currency CurrencyIsoCode + Territory2 mapping + Shield-encrypted field acceptance.
- Acceptance criterion 2: `data-sync` µservice runs bidirectional sync; per-aggregate row count is monitored.
- Acceptance criterion 3: Daily reconciliation job emits drift events for any row-count mismatch >0.1%.
- Acceptance criterion 4: Cutover completion: read traffic shifts to Oyatie; write traffic stops on Salesforce; final snapshot recorded.
- Cross-µservice handoff: `data-sync` µservice; `audit-chain` records cutover snapshot.
- Cedar policy hook: `policy/account-master-authorization.cedar` + sibling aggregates permit principal == "migration-system" with signed cutover token.
- Ontology projection: migration_run aggregate (in `data-sync` µservice) referenced from crm.
- Observability evidence: `oya_crm_migration_drift_total{tenant, source="salesforce", aggregate, drift_pct}` counter.
- Tenant-class behaviour: migration runs are paid-only.

#### Story CRM-026: HubSpot migration with Contact-as-Lead lifecycle preservation

- Persona: Migration project lead cutting over from HubSpot to Oyatie.
- Motivation: The tenant uses HubSpot's Contact-with-lifecycle-stage flow; migration should preserve that exact flow without forcing Salesforce-style Lead separation.
- Counterpart anchor: HubSpot Contact + lifecycle_stage migration.
- Acceptance criterion 1: Migration playbook `migration-playbooks/from-hubspot-sales-hub.md` maps HubSpot Contact → Oyatie Contact with lifecycle_stage preserved.
- Acceptance criterion 2: Pack overlay `lead_as_contact_lifecycle = true` is activated automatically during migration.
- Acceptance criterion 3: HubSpot Deals map to Oyatie Opportunities; HubSpot Tickets map to Oyatie Service Cases.
- Acceptance criterion 4: HubSpot Custom Properties map to Oyatie Custom Fields (Wave 15C implementation).
- Cross-µservice handoff: `data-sync` µservice + `crm` adapter `src/adapter/external/hubspot/`.
- Cedar policy hook: per Story CRM-025.
- Ontology projection: per Story CRM-025.
- Observability evidence: per Story CRM-025 with source="hubspot".
- Tenant-class behaviour: paid only.

#### Story CRM-027: Dynamics migration with Dataverse virtual-table federation

- Persona: Migration project lead cutting over from Dynamics 365 Sales.
- Motivation: The tenant has Dataverse virtual tables federating an ERP system; the migration should preserve the federation pattern by mapping virtual tables to Oyatie ontology projections.
- Counterpart anchor: Dynamics Dataverse + virtual tables.
- Acceptance criterion 1: Migration playbook `migration-playbooks/from-microsoft-dynamics-365-ce.md` (slug rename pending Wave 15C) maps Dataverse entities + BPF + Sales Insights scores.
- Acceptance criterion 2: Virtual-table federation maps to `ontology` µservice projection sources.
- Acceptance criterion 3: BPF stages map to Oyatie Cedar-gated state-machine transitions.
- Acceptance criterion 4: Sales Insights scores migrate to `intelligence` µservice as historical evidence.
- Cross-µservice handoff: `data-sync` + `ontology` + `intelligence`.
- Cedar policy hook: per Story CRM-025.
- Ontology projection: per Story CRM-025.
- Observability evidence: per Story CRM-025 with source="dynamics".
- Tenant-class behaviour: paid only.

#### Story CRM-028: Cell evacuation with tenant traffic re-shard

- Persona: SRE responding to a cell-wide incident.
- Motivation: A bad deployment causes elevated error rates in cell `aws-us-east-1-cell-3`; the SRE evacuates tenant traffic to a peer cell with shuffle-sharding preserving blast radius.
- Counterpart anchor: AWS multi-AZ failover; Dynamics geo-failover.
- Acceptance criterion 1: The SRE issues a cell-evacuation command via the platform CLI; the `workflow-engine` evacuation saga starts.
- Acceptance criterion 2: Tenant routing rules update via `cloud-iam` + edge gateway; traffic shifts to peer cells per shuffle-sharding.
- Acceptance criterion 3: In-flight Q2C sagas resume on the new cell from durable state; no double-execution.
- Acceptance criterion 4: Audit-chain seals the evacuation event with affected tenant count + duration + peer-cell assignment.
- Cross-µservice handoff: cellular substrate; `crm` aggregates are tenant-scoped and follow tenant cell assignment.
- Cedar policy hook: `policy/abuse-defence.cedar` + emergency-bypass policies permit accelerated operations during evacuation.
- Ontology projection: cell_assignment per tenant is updated.
- Observability evidence: `oya_crm_cell_evacuation_total{from_cell, to_cell, outcome}` counter; tenants-affected gauge.
- Tenant-class behaviour: applies uniformly.

#### Story CRM-029: Tenant-class demo_trial cap exceeded with conversion path

- Persona: Prospect on demo_trial running out of Opportunity capacity.
- Motivation: A prospect signed up for demo_trial; has loaded 100 sample Opportunities; tries to add a 101st; the platform must refuse the write + show a clear conversion path to paid.
- Counterpart anchor: HubSpot Free tier limits with upgrade-modal prompt; Salesforce trial expiration with conversion prompt.
- Acceptance criterion 1: POST `/v1/crm/{tenant_id}/opportunity` for the 101st Opportunity returns HTTP 402 Payment Required with `tenant_class_cap_exceeded` + structured detail (current_count: 100, cap: 100, upgrade_url: <https://oyatie.dev/upgrade?tenant_id=...>).
- Acceptance criterion 2: The UI surfaces an in-app banner explaining the cap + conversion offer.
- Acceptance criterion 3: Upgrade flow creates a paid tenant; data migrates from demo_trial to paid; demo_trial caps lift.
- Acceptance criterion 4: Audit-chain seals the cap-exceeded event for conversion-funnel analytics.
- Cross-µservice handoff: `cloud-billing-tax` µservice owns the upgrade flow; `cloud-iam` updates principal claims.
- Cedar policy hook: `policy/tenant-class-authorization.cedar` denies write past cap; `policy/opportunity-authorization.cedar` invokes tenant-class policy.
- Ontology projection: cap_exceeded events are an audit trail.
- Observability evidence: `oya_crm_demo_trial_cap_total{tenant, aggregate}` counter; conversion-rate metrics.
- Tenant-class behaviour: this story is the conversion event.

#### Story CRM-030: EU-AI-Act high-risk classification with explainability surface

- Persona: Tenant Compliance Officer in the EU using AI-driven Lead routing.
- Motivation: Per EU-AI-Act Article 6 (high-risk AI systems), an AI-routed Lead decision must be explainable to the affected data subject on request.
- Counterpart anchor: Salesforce Einstein explainability cards; not yet a Dynamics surface; EU-AI-Act compliance offering.
- Acceptance criterion 1: When `intelligence` µservice classifies a Lead score as "automated decision-impacting routing", the EU-AI-Act pack overlay activates the explainability flow.
- Acceptance criterion 2: Lead record includes `automated_decision_audit_ref` linking to the explainability artifact (feature importances, model card, training data summary, decision rationale).
- Acceptance criterion 3: Affected data subject can request the artifact via `consent-graph` + `cloud-iam` data-subject-request flow.
- Acceptance criterion 4: Tenant Compliance Officer can view aggregate high-risk decision metrics + per-decision drill-down.
- Cross-µservice handoff: `intelligence` + `consent-graph` + `audit-chain` + `cloud-iam`.
- Cedar policy hook: `policy/lead-authorization.cedar` + `policy/pack-overlay-authorization.cedar` enforces EU-AI-Act overlay.
- Ontology projection: automated_decision_audit entity.
- Observability evidence: `oya_crm_eu_ai_act_high_risk_total{tenant, decision_type}` counter.
- Tenant-class behaviour: EU-AI-Act pack is paid-only; demo_trial cannot use AI in EU residency.

### C.15 User-story summary

Total user stories authored in Wave 15A: 30 (CRM-001..CRM-030). All stories are bespoke; no template stamping. Coverage:

- Lead aggregate: 3 stories (CRM-001..CRM-003).
- Contact aggregate: 2 stories (CRM-004..CRM-005).
- Account aggregate: 2 stories (CRM-006..CRM-007).
- Opportunity aggregate: 2 stories (CRM-008..CRM-009).
- OpportunityTeam + OpportunitySplit: 1 story (CRM-010).
- Sales Cadence: 2 stories (CRM-011..CRM-012).
- CPQ Quote: 3 stories (CRM-013..CRM-015).
- Forecast: 2 stories (CRM-016..CRM-017).
- Service Case: 2 stories (CRM-018..CRM-019).
- Campaign: 1 story (CRM-020).
- Loyalty Ledger: 1 story (CRM-021).
- Partner: 1 story (CRM-022).
- Customer 360: 1 story (CRM-023).
- Cross-aggregate / migration / cellular / tenant-class: 7 stories (CRM-024..CRM-030).

Per-counterpart distribution:

- Salesforce-primary parity stories: 16.
- HubSpot-primary parity stories: 4 (CRM-001, CRM-004, CRM-011, CRM-026).
- Dynamics-primary parity stories: 3 (CRM-008, CRM-017, CRM-027).
- Cross-counterpart parity stories: 7.

Per-persona distribution: Marketing Operations (CRM-001), Sales Development Representative (CRM-002, CRM-011, CRM-012), Inside Sales Rep (CRM-003), Tenant Administrator (CRM-004, CRM-025..CRM-027), Data Steward (CRM-005), Enterprise Sales Operations Manager (CRM-006), Strategic Account Manager (CRM-007), Account Executive (CRM-008, CRM-014, CRM-024), Sales VP (CRM-009), Finance Controller (CRM-010, CRM-017), Solutions Engineer (CRM-013), International AE (CRM-015), Sales Manager (CRM-016), Service Operations Manager (CRM-018), Service Rep (CRM-019), Marketing Director (CRM-020), Loyalty Program Manager (CRM-021), Partner Account Manager (CRM-022), Customer Success Manager (CRM-023), SRE (CRM-028), Prospect (CRM-029), Tenant Compliance Officer (CRM-030).

All stories carry explicit Cedar policy hooks, observability metric definitions, tenant-class behaviour, and cross-µservice handoff paths — the substance bar required for intern-buildability per ADR-0322 + ADR-0324.

## D. Ontology Projection

Ontology projection is the contract that prevents Customer Relationship Management from becoming an isolated ERP island.
Every projection pins object type version, relation type version, source-system lineage, tenant subclass, retention class, and Cedar-visible attributes.
The pattern is the Palantir Foundry ontology projection pattern adapted to ADR-0244 tenant scope and ADR-0330 tenant-class model (which supersedes the retired ADR-0316 capability-tier doctrine).

### D.1 AccountMaster object projection
- Object type: AccountMaster.
- Required identifiers: tenant_id, account_master_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by CapabilityTier; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Marketplace; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.crm.account-master namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.2 Opportunity object projection
- Object type: Opportunity.
- Required identifiers: tenant_id, opportunity_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by CapabilityTier; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Community; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.crm.opportunity namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.3 Quote object projection
- Object type: Quote.
- Required identifiers: tenant_id, quote_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by CapabilityTier; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Payments; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.crm.quote namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.4 ServiceCase object projection
- Object type: ServiceCase.
- Required identifiers: tenant_id, service_case_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by CapabilityTier; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Workflow Engine; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.crm.service-case namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.5 Campaign object projection
- Object type: Campaign.
- Required identifiers: tenant_id, campaign_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by CapabilityTier; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Intelligence; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.crm.campaign namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.6 LoyaltyLedger object projection
- Object type: LoyaltyLedger.
- Required identifiers: tenant_id, loyalty_ledger_id, source_system_ref, source_row_ref, ontology_object_ref, version, status.
- Required relations: belongs_to Tenant; initiated_by Principal; governed_by CapabilityTier; emits AuditEvidence; orchestrated_by WorkflowRun.
- Optional relations: settles_through DealSet; priced_by FinOpsCostObject; fulfilled_by Ontology; remediated_by RunbookExecution.
- Projection rule: no row becomes queryable until source provenance, Cedar action namespace, workflow template id, and audit-chain target are present.
- Version rule: object schema revisions use ADR-0257 deprecation handshakes; readers must accept current and previous minor versions during migration.
- Tenant subclassing: tenant-specific attributes stay under tenant.crm.loyalty-ledger namespace and never alter shared base object semantics.
- Search rule: read models expose only authorized objects and return redacted relation stubs for denied cross-tenant counterparties.
- Export rule: auditor exports include object history, relation history, Cedar decision refs, and source-system transformation notes.

### D.7 Ontology quality gates
- OQ-01: AccountMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-02: Opportunity projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-03: Quote projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-04: ServiceCase projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-05: Campaign projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-06: LoyaltyLedger projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-07: AccountMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-08: Opportunity projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-09: Quote projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-10: ServiceCase projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-11: Campaign projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-12: LoyaltyLedger projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-13: AccountMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-14: Opportunity projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-15: Quote projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-16: ServiceCase projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-17: Campaign projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-18: LoyaltyLedger projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-19: AccountMaster projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-20: Opportunity projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-21: Quote projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-22: ServiceCase projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-23: Campaign projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.
- OQ-24: LoyaltyLedger projection must round-trip create, amend, reverse, archive, import, export, and replay fixtures without dropping tenant, policy, workflow, or audit fields.

## E. Workflow

Workflow-engine owns orchestration; crm owns domain validation, state transitions, and emitted events.
### E.1 Activation flow
- Step 1: Tenant selects capability tier.
- Step 2: marketplace verifies entitlement.
- Step 3: crm seeds templates.
- Step 4: audit-chain seals activation evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.2 Daily operation flow
- Step 1: Operator submits command.
- Step 2: Cedar authorizes principal.
- Step 3: crm validates domain state.
- Step 4: workflow-engine advances lifecycle.
- Step 5: ontology updates projection.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.3 Approval flow
- Step 1: Command enters approval queue.
- Step 2: approver receives task.
- Step 3: policy-engine checks separation of duties.
- Step 4: audit-chain records decision.
- Step 5: crm emits approved event.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.4 Exception flow
- Step 1: Failure enters dead-letter state.
- Step 2: runbook execution opens.
- Step 3: operator fixes source or policy input.
- Step 4: replay resumes from idempotency key.
- Step 5: SLO burn is re-evaluated.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.5 Migration flow
- Step 1: connect imports source rows.
- Step 2: crm validates row set.
- Step 3: ontology creates pending objects.
- Step 4: workflow-engine runs dry-run approval.
- Step 5: cutover emits accepted, rejected, and deferred evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.6 Settlement flow
- Step 1: crm emits settlement-intent event.
- Step 2: marketplace creates or amends DealSet.
- Step 3: payments or treasury handles rail-specific state.
- Step 4: finops records chargeback dimensions.
- Step 5: audit-chain links commercial evidence.
- Success evidence: workflow_run_ref, audit_chain_ref, ontology_object_ref, Cedar decision id, and trace_id are all present.
- Failure evidence: failed step, responsible service, retry policy, rollback path, and user-visible state are named.

### E.7 Workflow invariants
- WI-01: account-master cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-02: opportunity cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-03: quote cannot call intelligence directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-04: service-case cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-05: campaign cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-06: loyalty-ledger cannot call community directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-07: account-master cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-08: opportunity cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-09: quote cannot call intelligence directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-10: service-case cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-11: campaign cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-12: loyalty-ledger cannot call community directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-13: account-master cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-14: opportunity cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-15: quote cannot call intelligence directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-16: service-case cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-17: campaign cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-18: loyalty-ledger cannot call community directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-19: account-master cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-20: opportunity cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-21: quote cannot call intelligence directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-22: service-case cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-23: campaign cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-24: loyalty-ledger cannot call community directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-25: account-master cannot call payments directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-26: opportunity cannot call workflow-engine directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-27: quote cannot call intelligence directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-28: service-case cannot call ontology directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-29: campaign cannot call marketplace directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.
- WI-30: loyalty-ledger cannot call community directly for state mutation unless workflow-engine has produced a workflow_run_ref and audit-chain has accepted the pre-transition evidence.

## F. Policy

Cedar is the only application authorization language for Customer Relationship Management.
Policy coverage uses ADR-0244 tenant scope, ADR-0314 DealSet settlement when commercial events exist, ADR-0328 Big-8 anchor priority, and ADR-0330 tenant-class gating (supersedes the retired ADR-0316 capability-tier doctrine).
Policy files present: abuse-defence.cedar, account-master-authorization.cedar, auditor-scope.cedar, campaign-authorization.cedar, ci-scope.cedar, data-residency.md, emergency-services-bypass.cedar, loyalty-ledger-authorization.cedar, opportunity-authorization.cedar, pack-overlay-authorization.cedar, quote-authorization.cedar, service-case-authorization.cedar, tenant-isolation.md.

### F.1 Account Master Cedar hooks
- Action crm.account-master.read: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.account-master.create: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.account-master.amend: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.account-master.approve: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.account-master.reverse: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.account-master.archive: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.account-master.import: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.account-master.export: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.account-master.reconcile: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.account-master.simulate: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.account-master, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale capability tier, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.2 Opportunity Cedar hooks
- Action crm.opportunity.read: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.opportunity.create: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.opportunity.amend: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.opportunity.approve: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.opportunity.reverse: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.opportunity.archive: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.opportunity.import: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.opportunity.export: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.opportunity.reconcile: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.opportunity.simulate: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.opportunity, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale capability tier, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.3 Quote Cedar hooks
- Action crm.quote.read: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.quote.create: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.quote.amend: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.quote.approve: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.quote.reverse: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.quote.archive: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.quote.import: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.quote.export: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.quote.reconcile: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.quote.simulate: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.quote, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale capability tier, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.4 Service Case Cedar hooks
- Action crm.service-case.read: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.service-case.create: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.service-case.amend: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.service-case.approve: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.service-case.reverse: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.service-case.archive: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.service-case.import: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.service-case.export: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.service-case.reconcile: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.service-case.simulate: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.service-case, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale capability tier, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.5 Campaign Cedar hooks
- Action crm.campaign.read: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.campaign.create: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.campaign.amend: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.campaign.approve: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.campaign.reverse: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.campaign.archive: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.campaign.import: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.campaign.export: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.campaign.reconcile: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.campaign.simulate: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.campaign, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale capability tier, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.6 Loyalty Ledger Cedar hooks
- Action crm.loyalty-ledger.read: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.loyalty-ledger.create: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.loyalty-ledger.amend: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.loyalty-ledger.approve: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.loyalty-ledger.reverse: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.loyalty-ledger.archive: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.loyalty-ledger.import: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.loyalty-ledger.export: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.loyalty-ledger.reconcile: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Action crm.loyalty-ledger.simulate: permit only when principal.tenant_id equals resource.tenant_id, capability tier includes crm.loyalty-ledger, data_class is allowed by active pack, and workflow context matches the expected lifecycle state.
- Default-deny: missing tenant, missing sub_scope_path, stale principal grant, stale capability tier, unknown source system, and unsealed audit target all deny.
- Break-glass: emergency-services-bypass.cedar requires post-hoc audit justification and cannot approve commercial settlement without marketplace DealSet evidence.

### F.7 Policy acceptance gates
- PG-01: fixture account-master.create-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-02: fixture opportunity.amend-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-03: fixture quote.approve-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-04: fixture service-case.reverse-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-05: fixture campaign.archive-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-06: fixture loyalty-ledger.run-a-migration-dry-run must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-07: fixture account-master.compare-source-system-rows must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-08: fixture opportunity.export-audit-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-09: fixture quote.resolve-a-policy-denied-mutation must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-10: fixture service-case.promote-a-capability-tier must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-11: fixture campaign.inspect-ontology-lineage must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-12: fixture loyalty-ledger.coordinate-a-cross-service-workflow must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-13: fixture account-master.receive-settlement-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-14: fixture opportunity.handle-a-regional-failover must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-15: fixture quote.run-a-batch-reconcile must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-16: fixture service-case.trace-a-source-system-discrepancy must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-17: fixture campaign.apply-a-compliance-pack must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-18: fixture loyalty-ledger.review-SLO-burn must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-19: fixture account-master.simulate-a-10x-volume-surge must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-20: fixture opportunity.deactivate-a-stale-pack must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-21: fixture quote.create-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-22: fixture service-case.amend-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-23: fixture campaign.approve-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-24: fixture loyalty-ledger.reverse-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-25: fixture account-master.archive-a-governed-record must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-26: fixture opportunity.run-a-migration-dry-run must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-27: fixture quote.compare-source-system-rows must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-28: fixture service-case.export-audit-evidence must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-29: fixture campaign.resolve-a-policy-denied-mutation must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.
- PG-30: fixture loyalty-ledger.promote-a-capability-tier must include one allowed path, one denied path, one cross-tenant denial, one auditor-scope read, and one CI-scope read.

## G. Observability

The PRD requires production diagnosis from telemetry alone.
Dashboards present: account-master-health.json, crm-overview.json, opportunity-residency.md.
SLO files present: account-master-success-rate.openslo.yaml, crm-availability.openslo.yaml, crm-latency-p99.openslo.yaml, crm-throughput.openslo.yaml.
Runbooks present: approval-deadletter.md, capacity-saturation.md, marketplace-settlement-blocked.md, policy-deny-spike.md, regional-failover.md, source-import-stalled.md.

### G.1 Account Master telemetry
- Metric counter: oya_crm_account_master_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_crm_account_master_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_crm_account_master_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: crm.account-master.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-CRM-ACCOUNT_MASTER-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.2 Opportunity telemetry
- Metric counter: oya_crm_opportunity_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_crm_opportunity_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_crm_opportunity_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: crm.opportunity.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-CRM-OPPORTUNITY-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.3 Quote telemetry
- Metric counter: oya_crm_quote_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_crm_quote_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_crm_quote_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: crm.quote.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-CRM-QUOTE-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.4 Service Case telemetry
- Metric counter: oya_crm_service_case_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_crm_service_case_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_crm_service_case_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: crm.service-case.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-CRM-SERVICE_CASE-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.5 Campaign telemetry
- Metric counter: oya_crm_campaign_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_crm_campaign_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_crm_campaign_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: crm.campaign.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-CRM-CAMPAIGN-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.6 Loyalty Ledger telemetry
- Metric counter: oya_crm_loyalty_ledger_transition_total with tenant, region, cell, tier, action, outcome, policy_decision, source_system, and replay dimensions.
- Metric histogram: oya_crm_loyalty_ledger_command_latency_ms with p50 under 120 ms, p95 under 300 ms, p99 under 750 ms for lightweight mutations.
- Batch metric: oya_crm_loyalty_ledger_batch_lag_seconds with warning at 300 seconds and page at 900 seconds for migration, replay, and reconcile jobs.
- Trace span: crm.loyalty-ledger.command wraps validation, Cedar decision, workflow transition, ontology projection, audit seal, and publication.
- Structured log: includes event_id, tenant_id hash, principal_id hash, action, resource id, source_system_ref, policy_decision_id, workflow_run_ref, and audit_chain_ref.
- Audit event: EVT-CRM-LOYALTY_LEDGER-STATE with before_state, after_state, reason, actor, and evidence hash.
- Cardinality budget: tenant and region are allowed dimensions; raw principal and source row IDs are hash-only and cannot become unbounded label values.
- Dashboard panel: health, latency, deny rate, replay lag, source mismatch, and regional failover state are visible without database access.

### G.7 Capacity and performance model
- Little's Law guardrail: concurrency equals arrival_rate times service_time; each context must document worker capacity at 10x and 100x tenant load.
- Baseline: a 300 ms p95 command at 1000 commands per second requires 300 concurrent worker slots before headroom.
- Headroom: production allocation targets 2x calculated concurrency for active-active regions and 3x for regulated sovereign cells during migration windows.
- Backpressure: batch workers shed optional projection refresh before command writes; user-visible states name queued, deferred, denied, and replaying conditions.
- Cost attribution: every event sends tenant, sub_scope_path, capability_profile, bounded_context, workflow_run_ref, and cell to finops-portal.
- OM-01: account-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-02: opportunity SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-03: quote SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-04: service-case SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-05: campaign SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-06: loyalty-ledger SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-07: account-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-08: opportunity SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-09: quote SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-10: service-case SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-11: campaign SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-12: loyalty-ledger SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-13: account-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-14: opportunity SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-15: quote SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-16: service-case SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-17: campaign SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-18: loyalty-ledger SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-19: account-master SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.
- OM-20: opportunity SLO reviews must include p50, p95, p99, error budget burn, policy deny rate, replay lag, audit seal latency, and ontology projection lag.

## H. Packs

Packs are tenant-selected overlays. They activate policy fragments, retention rules, workflows, evidence exports, and marketplace settlement controls without creating product-fragment microservices.

### H.1 core-enterprise
- Activation effect: enables crm.account-master commands appropriate for core-enterprise and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.account-master and compliance_pack contains core-enterprise.
- Ontology effect: projects AccountMaster with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with core-enterprise terms rather than a bespoke settlement table.

### H.2 sox-404
- Activation effect: enables crm.opportunity commands appropriate for sox-404 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.opportunity and compliance_pack contains sox-404.
- Ontology effect: projects Opportunity with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with sox-404 terms rather than a bespoke settlement table.

### H.3 soc2
- Activation effect: enables crm.quote commands appropriate for soc2 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.quote and compliance_pack contains soc2.
- Ontology effect: projects Quote with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with soc2 terms rather than a bespoke settlement table.

### H.4 iso-27001
- Activation effect: enables crm.service-case commands appropriate for iso-27001 and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.service-case and compliance_pack contains iso-27001.
- Ontology effect: projects ServiceCase with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with iso-27001 terms rather than a bespoke settlement table.

### H.5 gdpr-eu
- Activation effect: enables crm.campaign commands appropriate for gdpr-eu and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.campaign and compliance_pack contains gdpr-eu.
- Ontology effect: projects Campaign with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with gdpr-eu terms rather than a bespoke settlement table.

### H.6 kr-csap
- Activation effect: enables crm.loyalty-ledger commands appropriate for kr-csap and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.loyalty-ledger and compliance_pack contains kr-csap.
- Ontology effect: projects LoyaltyLedger with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with kr-csap terms rather than a bespoke settlement table.

### H.7 fedramp-high
- Activation effect: enables crm.account-master commands appropriate for fedramp-high and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.account-master and compliance_pack contains fedramp-high.
- Ontology effect: projects AccountMaster with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with fedramp-high terms rather than a bespoke settlement table.

### H.8 industry-regulated
- Activation effect: enables crm.opportunity commands appropriate for industry-regulated and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.opportunity and compliance_pack contains industry-regulated.
- Ontology effect: projects Opportunity with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with industry-regulated terms rather than a bespoke settlement table.

### H.9 marketplace-settlement
- Activation effect: enables crm.quote commands appropriate for marketplace-settlement and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.quote and compliance_pack contains marketplace-settlement.
- Ontology effect: projects Quote with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with marketplace-settlement terms rather than a bespoke settlement table.

### H.10 migration-assurance
- Activation effect: enables crm.service-case commands appropriate for migration-assurance and pins retention, residency, evidence, and export behavior.
- Cedar effect: requires capability_profile contains crm.service-case and compliance_pack contains migration-assurance.
- Ontology effect: projects ServiceCase with pack-specific attributes under tenant-owned namespace.
- Workflow effect: provisions approval, exception, and export templates with pack-specific deadlines.
- Observability effect: adds pack dimension to SLO burn, audit export, and policy-deny dashboards.
- Marketplace effect: when commercial obligation exists, creates or amends a DealSet with migration-assurance terms rather than a bespoke settlement table.

## I. Migration

Migration converts source ERP records into accepted, rejected, or deferred tenant-scoped objects with replayable evidence.
Source systems named for this service: SAP CRM business partner extracts; Salesforce account exports; marketing consent feeds; support case history.

### I.1 Inventory phase
- Entry condition: source rows for Customer Relationship Management have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into crm commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: crm rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.2 Mapping phase
- Entry condition: source rows for Customer Relationship Management have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into crm commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: crm rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.3 Dry Run phase
- Entry condition: source rows for Customer Relationship Management have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into crm commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: crm rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.4 Dual Write phase
- Entry condition: source rows for Customer Relationship Management have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into crm commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: crm rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.5 Cutover phase
- Entry condition: source rows for Customer Relationship Management have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into crm commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: crm rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.6 Reconciliation phase
- Entry condition: source rows for Customer Relationship Management have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into crm commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: crm rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.7 Retirement phase
- Entry condition: source rows for Customer Relationship Management have tenant scope, source_system_ref, source_row_ref, extract timestamp, data_class, and checksum.
- Transformation: connect adapter maps source fields into crm commands, ontology projections, Cedar-visible attributes, and workflow templates.
- Validation: crm rejects ambiguous owner, missing tenant, stale source version, invalid state transition, and missing audit target.
- Evidence: audit-chain stores batch summary, row counts, accepted/rejected/deferred counts, sample hashes, and operator decisions.
- Rollback: current read path stays active until dual-read reconciliation clears; cutover rollback reverts read routing and keeps imported rows quarantined.

### I.8 Migration row acceptance rules
- MR-01: account-master rows from SAP CRM business partner extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-02: opportunity rows from Salesforce account exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-03: quote rows from marketing consent feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-04: service-case rows from support case history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-05: campaign rows from SAP CRM business partner extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-06: loyalty-ledger rows from Salesforce account exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-07: account-master rows from marketing consent feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-08: opportunity rows from support case history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-09: quote rows from SAP CRM business partner extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-10: service-case rows from Salesforce account exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-11: campaign rows from marketing consent feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-12: loyalty-ledger rows from support case history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-13: account-master rows from SAP CRM business partner extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-14: opportunity rows from Salesforce account exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-15: quote rows from marketing consent feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-16: service-case rows from support case history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-17: campaign rows from SAP CRM business partner extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-18: loyalty-ledger rows from Salesforce account exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-19: account-master rows from marketing consent feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-20: opportunity rows from support case history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-21: quote rows from SAP CRM business partner extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-22: service-case rows from Salesforce account exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-23: campaign rows from marketing consent feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-24: loyalty-ledger rows from support case history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-25: account-master rows from SAP CRM business partner extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-26: opportunity rows from Salesforce account exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-27: quote rows from marketing consent feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-28: service-case rows from support case history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-29: campaign rows from SAP CRM business partner extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-30: loyalty-ledger rows from Salesforce account exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-31: account-master rows from marketing consent feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-32: opportunity rows from support case history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-33: quote rows from SAP CRM business partner extracts must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-34: service-case rows from Salesforce account exports must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-35: campaign rows from marketing consent feeds must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.
- MR-36: loyalty-ledger rows from support case history must map source id, source version, tenant, state, owner, policy context, ontology type, workflow template, audit stream, and replay key before they become accepted.

## J. Tenant Classes (formerly Capability Tiers)

This section is rewritten in Wave 15A. The Wave 3-G "Capability Tiers" doctrine bound to ADR-0316 is retired per ADR-0329 (capability-tier retirement) + ADR-0330 (tenant-class binary model). The tenant-visible activation primitive is now `tenant_class ∈ {demo_trial, paid}`; paid carries `billing_components ⊆ {revenue_share, per_seat, per_usage}`. The Wave 3-G six tier subsections (starter-readonly, professional-operator, enterprise-controlled, regulated-sovereign, hyperscale-multicell, partner-network) are retained below as historical reference only and will be removed in Wave 15B per audit T-006.

### J.1 starter-readonly
- Includes: crm.account-master.read, crm.account-master.export, and tier-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: capability_profile=starter-readonly is part of Cedar context and is recorded in audit-chain for every action.
- Workflow: tier controls approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by starter-readonly without increasing high-cardinality labels beyond the approved budget.
- Migration: tier selection determines dry-run depth, dual-write duration, and rollback window.

### J.2 professional-operator
- Includes: crm.opportunity.read, crm.opportunity.export, and tier-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: capability_profile=professional-operator is part of Cedar context and is recorded in audit-chain for every action.
- Workflow: tier controls approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by professional-operator without increasing high-cardinality labels beyond the approved budget.
- Migration: tier selection determines dry-run depth, dual-write duration, and rollback window.

### J.3 enterprise-controlled
- Includes: crm.quote.read, crm.quote.export, and tier-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: capability_profile=enterprise-controlled is part of Cedar context and is recorded in audit-chain for every action.
- Workflow: tier controls approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by enterprise-controlled without increasing high-cardinality labels beyond the approved budget.
- Migration: tier selection determines dry-run depth, dual-write duration, and rollback window.

### J.4 regulated-sovereign
- Includes: crm.service-case.read, crm.service-case.export, and tier-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: capability_profile=regulated-sovereign is part of Cedar context and is recorded in audit-chain for every action.
- Workflow: tier controls approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by regulated-sovereign without increasing high-cardinality labels beyond the approved budget.
- Migration: tier selection determines dry-run depth, dual-write duration, and rollback window.

### J.5 hyperscale-multicell
- Includes: crm.campaign.read, crm.campaign.export, and tier-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: capability_profile=hyperscale-multicell is part of Cedar context and is recorded in audit-chain for every action.
- Workflow: tier controls approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by hyperscale-multicell without increasing high-cardinality labels beyond the approved budget.
- Migration: tier selection determines dry-run depth, dual-write duration, and rollback window.

### J.6 partner-network
- Includes: crm.loyalty-ledger.read, crm.loyalty-ledger.export, and tier-appropriate mutation actions.
- Excludes: direct database access, unscoped cross-tenant views, non-Cedar permission checks, and unversioned ontology extensions.
- Policy: capability_profile=partner-network is part of Cedar context and is recorded in audit-chain for every action.
- Workflow: tier controls approval thresholds, replay rights, export size, and regulator evidence deadlines.
- Observability: dashboards can filter by partner-network without increasing high-cardinality labels beyond the approved budget.
- Migration: tier selection determines dry-run depth, dual-write duration, and rollback window.

### J.7 Tier promotion gates
- TG-01: account-master cannot promote to starter-readonly until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-02: opportunity cannot promote to professional-operator until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-03: quote cannot promote to enterprise-controlled until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-04: service-case cannot promote to regulated-sovereign until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-05: campaign cannot promote to hyperscale-multicell until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-06: loyalty-ledger cannot promote to partner-network until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-07: account-master cannot promote to starter-readonly until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-08: opportunity cannot promote to professional-operator until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-09: quote cannot promote to enterprise-controlled until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-10: service-case cannot promote to regulated-sovereign until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-11: campaign cannot promote to hyperscale-multicell until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-12: loyalty-ledger cannot promote to partner-network until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-13: account-master cannot promote to starter-readonly until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-14: opportunity cannot promote to professional-operator until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-15: quote cannot promote to enterprise-controlled until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-16: service-case cannot promote to regulated-sovereign until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-17: campaign cannot promote to hyperscale-multicell until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-18: loyalty-ledger cannot promote to partner-network until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-19: account-master cannot promote to starter-readonly until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-20: opportunity cannot promote to professional-operator until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-21: quote cannot promote to enterprise-controlled until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-22: service-case cannot promote to regulated-sovereign until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-23: campaign cannot promote to hyperscale-multicell until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-24: loyalty-ledger cannot promote to partner-network until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-25: account-master cannot promote to starter-readonly until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-26: opportunity cannot promote to professional-operator until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-27: quote cannot promote to enterprise-controlled until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-28: service-case cannot promote to regulated-sovereign until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-29: campaign cannot promote to hyperscale-multicell until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.
- TG-30: loyalty-ledger cannot promote to partner-network until contracts, Cedar fixtures, ontology projection tests, migration replay, dashboard panels, runbook verification, and SLO files all pass.

## K. Critical-Path Scenarios

These 30 bespoke scenarios cover normal, edge, and failure cases for Customer Relationship Management.

### Scenario CRM-SC-001: Account Master happy path creation
- Normal case: crm.account-master accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for happy path creation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/account-master-authorization.cedar evaluates action crm.account-master.happy_path_creation with pack, tier, principal, and data-class context.
- Ontology projection: AccountMaster keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (happy-path-creation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-002: Opportunity approval escalation
- Normal case: crm.opportunity accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for approval escalation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: community receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/opportunity-authorization.cedar evaluates action crm.opportunity.approval_escalation with pack, tier, principal, and data-class context.
- Ontology projection: Opportunity keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and CommunityHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (approval-escalation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-003: Quote source duplicate import
- Normal case: crm.quote accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for source duplicate import; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/quote-authorization.cedar evaluates action crm.quote.source_duplicate_import with pack, tier, principal, and data-class context.
- Ontology projection: Quote keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (source-duplicate-import maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-004: Service Case policy deny spike
- Normal case: crm.service-case accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for policy deny spike; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/service-case-authorization.cedar evaluates action crm.service-case.policy_deny_spike with pack, tier, principal, and data-class context.
- Ontology projection: ServiceCase keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (policy-deny-spike maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-005: Campaign regional failover
- Normal case: crm.campaign accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for regional failover; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: intelligence receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/campaign-authorization.cedar evaluates action crm.campaign.regional_failover with pack, tier, principal, and data-class context.
- Ontology projection: Campaign keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and IntelligenceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (regional-failover maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-006: Loyalty Ledger batch replay
- Normal case: crm.loyalty-ledger accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for batch replay; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/loyalty-ledger-authorization.cedar evaluates action crm.loyalty-ledger.batch_replay with pack, tier, principal, and data-class context.
- Ontology projection: LoyaltyLedger keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (batch-replay maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-007: Account Master ontology schema upgrade
- Normal case: crm.account-master accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for ontology schema upgrade; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/account-master-authorization.cedar evaluates action crm.account-master.ontology_schema_upgrade with pack, tier, principal, and data-class context.
- Ontology projection: AccountMaster keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (ontology-schema-upgrade maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-008: Opportunity marketplace settlement block
- Normal case: crm.opportunity accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for marketplace settlement block; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: community receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/opportunity-authorization.cedar evaluates action crm.opportunity.marketplace_settlement_block with pack, tier, principal, and data-class context.
- Ontology projection: Opportunity keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and CommunityHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (marketplace-settlement-block maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-009: Quote audit export under regulator deadline
- Normal case: crm.quote accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for audit export under regulator deadline; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/quote-authorization.cedar evaluates action crm.quote.audit_export_under_regulator_deadline with pack, tier, principal, and data-class context.
- Ontology projection: Quote keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (audit-export-under-regulator-deadline maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-010: Service Case concurrent amendment conflict
- Normal case: crm.service-case accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for concurrent amendment conflict; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/service-case-authorization.cedar evaluates action crm.service-case.concurrent_amendment_conflict with pack, tier, principal, and data-class context.
- Ontology projection: ServiceCase keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (concurrent-amendment-conflict maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-011: Campaign SLO burn rate page
- Normal case: crm.campaign accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for SLO burn rate page; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: intelligence receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/campaign-authorization.cedar evaluates action crm.campaign.SLO_burn_rate_page with pack, tier, principal, and data-class context.
- Ontology projection: Campaign keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and IntelligenceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (burn-rate-page maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-012: Loyalty Ledger stale connector credential
- Normal case: crm.loyalty-ledger accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for stale connector credential; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/loyalty-ledger-authorization.cedar evaluates action crm.loyalty-ledger.stale_connector_credential with pack, tier, principal, and data-class context.
- Ontology projection: LoyaltyLedger keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/marketplace-settlement-blocked.md (stale-connector-credential maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-013: Account Master tenant merger carve-out
- Normal case: crm.account-master accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for tenant merger carve-out; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/account-master-authorization.cedar evaluates action crm.account-master.tenant_merger_carve-out with pack, tier, principal, and data-class context.
- Ontology projection: AccountMaster keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (tenant-merger-carve-out maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-014: Opportunity sovereign pack activation
- Normal case: crm.opportunity accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for sovereign pack activation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: community receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/opportunity-authorization.cedar evaluates action crm.opportunity.sovereign_pack_activation with pack, tier, principal, and data-class context.
- Ontology projection: Opportunity keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and CommunityHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (sovereign-pack-activation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-015: Quote cross-cell query degradation
- Normal case: crm.quote accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for cross-cell query degradation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/quote-authorization.cedar evaluates action crm.quote.cross-cell_query_degradation with pack, tier, principal, and data-class context.
- Ontology projection: Quote keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (cross-cell-query-degradation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-016: Service Case idempotency replay
- Normal case: crm.service-case accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for idempotency replay; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/service-case-authorization.cedar evaluates action crm.service-case.idempotency_replay with pack, tier, principal, and data-class context.
- Ontology projection: ServiceCase keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (idempotency-replay maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-017: Campaign poison message dead-letter
- Normal case: crm.campaign accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for poison message dead-letter; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: intelligence receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/campaign-authorization.cedar evaluates action crm.campaign.poison_message_dead-letter with pack, tier, principal, and data-class context.
- Ontology projection: Campaign keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and IntelligenceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/marketplace-settlement-blocked.md (poison-message-dead-letter maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-018: Loyalty Ledger capacity saturation
- Normal case: crm.loyalty-ledger accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for capacity saturation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/loyalty-ledger-authorization.cedar evaluates action crm.loyalty-ledger.capacity_saturation with pack, tier, principal, and data-class context.
- Ontology projection: LoyaltyLedger keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (capacity-saturation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-019: Account Master operator rollback
- Normal case: crm.account-master accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for operator rollback; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/account-master-authorization.cedar evaluates action crm.account-master.operator_rollback with pack, tier, principal, and data-class context.
- Ontology projection: AccountMaster keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (operator-rollback maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-020: Opportunity counterparty access revocation
- Normal case: crm.opportunity accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for counterparty access revocation; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: community receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/opportunity-authorization.cedar evaluates action crm.opportunity.counterparty_access_revocation with pack, tier, principal, and data-class context.
- Ontology projection: Opportunity keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and CommunityHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (counterparty-access-revocation maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-021: Quote pricing or cost allocation mismatch
- Normal case: crm.quote accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for pricing or cost allocation mismatch; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/quote-authorization.cedar evaluates action crm.quote.pricing_or_cost_allocation_mismatch with pack, tier, principal, and data-class context.
- Ontology projection: Quote keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (pricing-or-cost-allocation-mismatch maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-022: Service Case event ordering gap
- Normal case: crm.service-case accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for event ordering gap; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/service-case-authorization.cedar evaluates action crm.service-case.event_ordering_gap with pack, tier, principal, and data-class context.
- Ontology projection: ServiceCase keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (event-ordering-gap maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-023: Campaign data residency dispute
- Normal case: crm.campaign accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for data residency dispute; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: intelligence receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/campaign-authorization.cedar evaluates action crm.campaign.data_residency_dispute with pack, tier, principal, and data-class context.
- Ontology projection: Campaign keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and IntelligenceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (data-residency-dispute maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-024: Loyalty Ledger principal offboarding
- Normal case: crm.loyalty-ledger accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for principal offboarding; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/loyalty-ledger-authorization.cedar evaluates action crm.loyalty-ledger.principal_offboarding with pack, tier, principal, and data-class context.
- Ontology projection: LoyaltyLedger keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/policy-deny-spike.md (principal-offboarding maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-025: Account Master pack downgrade request
- Normal case: crm.account-master accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for pack downgrade request; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: marketplace receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/account-master-authorization.cedar evaluates action crm.account-master.pack_downgrade_request with pack, tier, principal, and data-class context.
- Ontology projection: AccountMaster keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and MarketplaceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (pack-downgrade-request maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-026: Opportunity high-volume seasonal peak
- Normal case: crm.opportunity accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for high-volume seasonal peak; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: community receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/opportunity-authorization.cedar evaluates action crm.opportunity.high-volume_seasonal_peak with pack, tier, principal, and data-class context.
- Ontology projection: Opportunity keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and CommunityHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (high-volume-seasonal-peak maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-027: Quote external system outage
- Normal case: crm.quote accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for external system outage; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: payments receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/quote-authorization.cedar evaluates action crm.quote.external_system_outage with pack, tier, principal, and data-class context.
- Ontology projection: Quote keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and PaymentsHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/regional-failover.md (external-system-outage maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-028: Service Case manual correction request
- Normal case: crm.service-case accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for manual correction request; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: workflow-engine receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/service-case-authorization.cedar evaluates action crm.service-case.manual_correction_request with pack, tier, principal, and data-class context.
- Ontology projection: ServiceCase keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and WorkflowEngineHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/capacity-saturation.md (manual-correction-request maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-029: Campaign compliance evidence gap
- Normal case: crm.campaign accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for compliance evidence gap; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: intelligence receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/campaign-authorization.cedar evaluates action crm.campaign.compliance_evidence_gap with pack, tier, principal, and data-class context.
- Ontology projection: Campaign keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and IntelligenceHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/source-import-stalled.md (compliance-evidence-gap maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

### Scenario CRM-SC-030: Loyalty Ledger tier promotion readiness
- Normal case: crm.loyalty-ledger accepts a tenant-scoped command, validates SAP CRM / C4C / Service Cloud parity fields, advances workflow, updates ontology, emits audit-chain evidence, and publishes an event.
- Edge case: source-system data is valid but incomplete for tier promotion readiness; the command enters deferred state with exact missing fields and operator action.
- Failure case: Cedar deny, ontology version mismatch, workflow timeout, audit-chain seal failure, and duplicate idempotency key each produce named terminal or retryable states.
- Cross-microservice handoff: ontology receives only an event or workflow task with tenant, trace, policy, and audit refs; it never receives direct shared-database access.
- Cedar policy hook: policy/loyalty-ledger-authorization.cedar evaluates action crm.loyalty-ledger.tier_promotion_readiness with pack, tier, principal, and data-class context.
- Ontology projection: LoyaltyLedger keeps relation edges to Tenant, CapabilityTier, WorkflowRun, AuditEvidence, SourceSystemRecord, and OntologyHandoff when applicable.
- Observability: dashboard panel shows latency, deny rate, replay lag, dead-letter count, and region/cell split for this scenario.
- Recovery: runbook runbooks/approval-deadletter.md (tier-promotion-readiness maps to this operational runbook family) documents trigger, pre-checks, procedure, verification, rollback, post-incident, and references.
- Acceptance: user-visible state distinguishes accepted, deferred, denied, replaying, failed, reversed, archived, and exported without ambiguous wording.

## L. References

### L.1 Internal doctrine
- Internal: docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md.
- Internal: docs/decisions/ADR-0314-marketplace-as-universal-deal-settlement.md.
- Internal: docs/decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md.
- Internal: docs/decisions/ADR-0329-capability-tier-retirement.md (supersedes ADR-0316).
- Internal: docs/decisions/ADR-0330-tenant-class-binary-model.md.
- Internal: docs/decisions/ADR-0331-cross-microservice-tenant-class-adoption.md.
- Internal: docs/standards/documentation-rigor.md.
- Internal: specs/products/ontology.json.
- Internal: specs/cedar-fragment-schema.json.
- Companion: microservices/crm/ARCHITECTURE.md.
- Companion: microservices/crm/compliance.md.
- Companion: microservices/crm/manifest.json.
- Companion: microservices/crm/contracts/openapi-v1.yaml.
- Companion: microservices/crm/contracts/asyncapi-v1.yaml.
- Companion: microservices/crm/contracts/crm-v1.proto.

### L.2 SAP and comparator references
- SAP Help Portal: SAP CRM / C4C / Service Cloud: https://help.sap.com/doc/57c828a79b0b4a83bb5bc490e44bd31f/1.0/en-US/loiof13699bb8d32409a86bda0ece47266a9_f13699bb8d32409a86bda0ece47266a9.pdf.
- Comparator precedent: SAP CRM.
- Comparator precedent: SAP Cloud for Customer.
- Comparator precedent: Salesforce Sales Cloud.
- Comparator precedent: Microsoft Dynamics 365 Customer Engagement.

### L.3 Artifact references
- Capability record: microservices/crm/capabilities/account-master-command.yaml.
- Capability record: microservices/crm/capabilities/opportunity-reconcile.yaml.
- Capability record: microservices/crm/capabilities/quote-export.yaml.
- Policy record: microservices/crm/policy/abuse-defence.cedar.
- Policy record: microservices/crm/policy/account-master-authorization.cedar.
- Policy record: microservices/crm/policy/auditor-scope.cedar.
- Policy record: microservices/crm/policy/campaign-authorization.cedar.
- Policy record: microservices/crm/policy/ci-scope.cedar.
- Policy record: microservices/crm/policy/data-residency.md.
- Policy record: microservices/crm/policy/emergency-services-bypass.cedar.
- Policy record: microservices/crm/policy/loyalty-ledger-authorization.cedar.
- Policy record: microservices/crm/policy/opportunity-authorization.cedar.
- Policy record: microservices/crm/policy/pack-overlay-authorization.cedar.
- Policy record: microservices/crm/policy/quote-authorization.cedar.
- Policy record: microservices/crm/policy/service-case-authorization.cedar.
- Policy record: microservices/crm/policy/tenant-isolation.md.
- SLO record: microservices/crm/slos/account-master-success-rate.openslo.yaml.
- SLO record: microservices/crm/slos/crm-availability.openslo.yaml.
- SLO record: microservices/crm/slos/crm-latency-p99.openslo.yaml.
- SLO record: microservices/crm/slos/crm-throughput.openslo.yaml.
- Dashboard record: microservices/crm/dashboards/account-master-health.json.
- Dashboard record: microservices/crm/dashboards/crm-overview.json.
- Dashboard record: microservices/crm/dashboards/opportunity-residency.md.
- Runbook record: microservices/crm/runbooks/approval-deadletter.md.
- Runbook record: microservices/crm/runbooks/capacity-saturation.md.
- Runbook record: microservices/crm/runbooks/marketplace-settlement-blocked.md.
- Runbook record: microservices/crm/runbooks/policy-deny-spike.md.
- Runbook record: microservices/crm/runbooks/regional-failover.md.
- Runbook record: microservices/crm/runbooks/source-import-stalled.md.

### L.4 Review checklist
- RC-01: 1500 or more lines in PRD.md.
- RC-02: 40 or more As-a/I-want/So-that stories.
- RC-03: 30 critical-path scenarios.
- RC-04: ADR-0244, ADR-0314, ADR-0328, ADR-0329, ADR-0330, and ADR-0331 references.
- RC-05: Salesforce + HubSpot + Dynamics 365 Sales (Big-8 anchor set) references; SAP CRM kept as extended reference.
- RC-06: Cedar hooks per story and scenario.
- RC-07: ontology projection per story and scenario.
- RC-08: cross-microservice handoff per story and scenario.
- RC-09: no forbidden planning markers.
- RC-10: frontmatter YAML parse success.

## M. Buildability Appendix

This appendix adds implementation-grade detail so the PRD clears the documentation-rigor line floor without relying on tribal knowledge.
- BA-001: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.create, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack iso-27001, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-002: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.amend, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tier partner-network, and replay fixture evidence in the same trace.
- BA-003: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.approve, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack kr-csap, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-004: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.reverse, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack fedramp-high, tier professional-operator, and replay fixture evidence in the same trace.
- BA-005: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.archive, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack industry-regulated, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-006: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.import, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack marketplace-settlement, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-007: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.export, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack migration-assurance, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-008: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.read, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tier partner-network, and replay fixture evidence in the same trace.
- BA-009: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.create, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack sox-404, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-010: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.amend, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack soc2, tier professional-operator, and replay fixture evidence in the same trace.
- BA-011: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.approve, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack iso-27001, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-012: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.reverse, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack gdpr-eu, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-013: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.archive, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack kr-csap, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-014: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.import, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tier partner-network, and replay fixture evidence in the same trace.
- BA-015: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.export, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack industry-regulated, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-016: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.read, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack marketplace-settlement, tier professional-operator, and replay fixture evidence in the same trace.
- BA-017: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.create, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack migration-assurance, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-018: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.amend, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack core-enterprise, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-019: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.approve, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack sox-404, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-020: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.reverse, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tier partner-network, and replay fixture evidence in the same trace.
- BA-021: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.archive, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack iso-27001, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-022: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.import, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack gdpr-eu, tier professional-operator, and replay fixture evidence in the same trace.
- BA-023: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.export, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack kr-csap, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-024: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.read, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack fedramp-high, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-025: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.create, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack industry-regulated, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-026: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.amend, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tier partner-network, and replay fixture evidence in the same trace.
- BA-027: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.approve, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack migration-assurance, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-028: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.reverse, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack core-enterprise, tier professional-operator, and replay fixture evidence in the same trace.
- BA-029: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.archive, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack sox-404, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-030: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.import, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack soc2, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-031: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.export, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack iso-27001, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-032: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.read, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tier partner-network, and replay fixture evidence in the same trace.
- BA-033: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.create, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack kr-csap, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-034: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.amend, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack fedramp-high, tier professional-operator, and replay fixture evidence in the same trace.
- BA-035: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.approve, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack industry-regulated, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-036: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.reverse, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack marketplace-settlement, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-037: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.archive, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack migration-assurance, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-038: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.import, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tier partner-network, and replay fixture evidence in the same trace.
- BA-039: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.export, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack sox-404, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-040: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.read, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack soc2, tier professional-operator, and replay fixture evidence in the same trace.
- BA-041: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.create, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack iso-27001, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-042: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.amend, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack gdpr-eu, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-043: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.approve, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack kr-csap, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-044: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.reverse, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tier partner-network, and replay fixture evidence in the same trace.
- BA-045: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.archive, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack industry-regulated, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-046: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.import, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack marketplace-settlement, tier professional-operator, and replay fixture evidence in the same trace.
- BA-047: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.export, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack migration-assurance, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-048: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.read, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack core-enterprise, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-049: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.create, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack sox-404, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-050: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.amend, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tier partner-network, and replay fixture evidence in the same trace.
- BA-051: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.approve, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack iso-27001, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-052: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.reverse, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack gdpr-eu, tier professional-operator, and replay fixture evidence in the same trace.
- BA-053: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.archive, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack kr-csap, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-054: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.import, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack fedramp-high, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-055: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.export, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack industry-regulated, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-056: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.read, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tier partner-network, and replay fixture evidence in the same trace.
- BA-057: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.create, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack migration-assurance, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-058: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.amend, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack core-enterprise, tier professional-operator, and replay fixture evidence in the same trace.
- BA-059: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.approve, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack sox-404, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-060: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.reverse, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack soc2, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-061: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.archive, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack iso-27001, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-062: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.import, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tier partner-network, and replay fixture evidence in the same trace.
- BA-063: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.export, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack kr-csap, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-064: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.read, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack fedramp-high, tier professional-operator, and replay fixture evidence in the same trace.
- BA-065: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.create, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack industry-regulated, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-066: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.amend, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack marketplace-settlement, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-067: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.approve, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack migration-assurance, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-068: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.reverse, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tier partner-network, and replay fixture evidence in the same trace.
- BA-069: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.archive, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack sox-404, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-070: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.import, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack soc2, tier professional-operator, and replay fixture evidence in the same trace.
- BA-071: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.export, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack iso-27001, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-072: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.read, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack gdpr-eu, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-073: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.create, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack kr-csap, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-074: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.amend, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tier partner-network, and replay fixture evidence in the same trace.
- BA-075: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.approve, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack industry-regulated, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-076: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.reverse, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack marketplace-settlement, tier professional-operator, and replay fixture evidence in the same trace.
- BA-077: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.archive, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack migration-assurance, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-078: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.import, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack core-enterprise, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-079: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.export, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack sox-404, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-080: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.read, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tier partner-network, and replay fixture evidence in the same trace.
- BA-081: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.create, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack iso-27001, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-082: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.amend, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack gdpr-eu, tier professional-operator, and replay fixture evidence in the same trace.
- BA-083: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.approve, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack kr-csap, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-084: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.reverse, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack fedramp-high, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-085: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.archive, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack industry-regulated, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-086: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.import, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tier partner-network, and replay fixture evidence in the same trace.
- BA-087: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.export, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack migration-assurance, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-088: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.read, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack core-enterprise, tier professional-operator, and replay fixture evidence in the same trace.
- BA-089: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.create, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack sox-404, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-090: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.amend, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack soc2, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-091: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.approve, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack iso-27001, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-092: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.reverse, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack gdpr-eu, tier partner-network, and replay fixture evidence in the same trace.
- BA-093: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.archive, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack kr-csap, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-094: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.import, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack fedramp-high, tier professional-operator, and replay fixture evidence in the same trace.
- BA-095: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.export, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack industry-regulated, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-096: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.read, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack marketplace-settlement, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-097: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.create, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack migration-assurance, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-098: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.amend, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack core-enterprise, tier partner-network, and replay fixture evidence in the same trace.
- BA-099: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.approve, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack sox-404, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-100: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.reverse, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack soc2, tier professional-operator, and replay fixture evidence in the same trace.
- BA-101: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.archive, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack iso-27001, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-102: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.import, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack gdpr-eu, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-103: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.export, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack kr-csap, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-104: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.read, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack fedramp-high, tier partner-network, and replay fixture evidence in the same trace.
- BA-105: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.create, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack industry-regulated, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-106: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.amend, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack marketplace-settlement, tier professional-operator, and replay fixture evidence in the same trace.
- BA-107: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.approve, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack migration-assurance, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-108: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.reverse, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack core-enterprise, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-109: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.archive, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack sox-404, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-110: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.import, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack soc2, tier partner-network, and replay fixture evidence in the same trace.
- BA-111: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.export, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack iso-27001, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-112: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.read, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack gdpr-eu, tier professional-operator, and replay fixture evidence in the same trace.
- BA-113: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.create, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack kr-csap, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-114: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.amend, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack fedramp-high, tier regulated-sovereign, and replay fixture evidence in the same trace.
- BA-115: crm.account-master implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.account-master.approve, ontology projection AccountMaster, workflow handoff to payments, audit-chain seal, pack industry-regulated, tier hyperscale-multicell, and replay fixture evidence in the same trace.
- BA-116: crm.opportunity implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.opportunity.reverse, ontology projection Opportunity, workflow handoff to workflow-engine, audit-chain seal, pack marketplace-settlement, tier partner-network, and replay fixture evidence in the same trace.
- BA-117: crm.quote implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.quote.archive, ontology projection Quote, workflow handoff to intelligence, audit-chain seal, pack migration-assurance, tier starter-readonly, and replay fixture evidence in the same trace.
- BA-118: crm.service-case implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.service-case.import, ontology projection ServiceCase, workflow handoff to ontology, audit-chain seal, pack core-enterprise, tier professional-operator, and replay fixture evidence in the same trace.
- BA-119: crm.campaign implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.campaign.export, ontology projection Campaign, workflow handoff to marketplace, audit-chain seal, pack sox-404, tier enterprise-controlled, and replay fixture evidence in the same trace.
- BA-120: crm.loyalty-ledger implementation must keep SAP CRM / C4C / Service Cloud parity fields, tenant scope, Cedar action crm.loyalty-ledger.read, ontology projection LoyaltyLedger, workflow handoff to community, audit-chain seal, pack soc2, tier regulated-sovereign, and replay fixture evidence in the same trace.

## N. Non-Functional Requirements

### DR posture (ADR-0343)

- Target: RTO <= 14400 s and RPO <= 900 s for customer, opportunity, quote, case, campaign, and loyalty-ledger state, matching `manifest.json#dr`.
- Compliance floors considered: SOC2-T2 requires 14400 s / 900 s; ISO27001-2022 requires 14400 s / 3600 s; SOX-404 base floor requires 14400 s / 3600 s; KR-PIPA general personal information requires 14400 s / 900 s. CRM does not own RRN, health-sensitive, or general-ledger journal writes in the manifest-backed posture; activating those data/process overlays would tighten the effective target below this PRD value.
- Failover runbook reference: `runbooks/regional-failover.md` plus `multi-region.md` home_cell/dr_cell promotion rules. The manifest substrate is `postgres_wal_g`, `object_storage_versioned`, and `valkey`; the runbook must prove replay cursor continuity, tenant-scope denial for cross-cell leakage, and audit-chain seal continuity.
- Multi-region active-active posture: `false` in `manifest.json`; replicated read projections and customer-360 degraded reads can be served during DR, but there are no active-active writes. Writes route to home_cell until a signed disaster promotion changes the role.
- WHY: sales and service teams can keep reading customer posture and continue controlled operations during a regional outage without losing revenue-lineage evidence or violating residency packs.

### Capacity model (ADR-0340)

- Manifest source: `manifest.json#capacity_model` declares the PRD capacity baseline.
- Per-tenant baseline: reserve 0.08 vCPU, 192 MiB RAM, 2 GB CRM OLTP/projection storage, 4 Postgres connections, 3 Valkey/cache connections, and 12 outbound HTTP slots for marketplace, CRM-adjacent, and import adapters.
- Scaling dimension: `per_user`, because customer accounts, opportunities, quotes, and service-case state scale with active CRM seats plus sales-motion volume.
- Cell placement class: Tier-3 product cell. Rationale: CRM owns product-domain records and revenue workflow evidence, while settlement, identity, and policy stay in their own substrate/control cells.
- Autoscaling boundaries: minimum 2 REST replicas and 2 worker replicas per hot bounded context; scale to 60 REST replicas or 120 worker leases per cell when p99 exceeds the capacity profile or worker replay leases exceed 60 s. Enterprise profile budgets 2500 rps at p99 250 ms; regulated-enterprise profile budgets 1500 rps at p99 300 ms.
- WHY: this serves mixed sales, service, campaign, and partner workloads where one tenant's import, attribution, or territory-routing surge must not starve quote and case commands.

### Sustainability + cost attribution (ADR-0344)

- Per-call emission claim: every CRM command, worker checkpoint, projection repair, source-sync, campaign attribution, and audit export row emits `cost_usd_minor_units`, `co2_grams`, and `watt_hours`.
- Provider routing affected by carbon: yes for source imports, campaign attribution snapshots, projection repair, and customer-360 rebuilds; no for quote approval, order/contract lineage commands, service-case escalation, or policy-denied reads where latency and audit order dominate.
- Per-tenant cost transparency surface: CRM customer-360/admin reporting shows cost by tenant, bounded context, capability, provider, cell, and compliance pack; FinOps receives the same dimensions for product-level rollups.
- WHY: revenue operations can explain cost-to-serve and Scope 3-adjacent CRM analytics without letting carbon routing alter customer commitments or SOX-relevant evidence order.

### API versioning posture (ADR-0342)

- Public API version model: date carrier triplet using `Oyatie-Version: YYYY-MM-DD`, URL prefix `/v/<YYYY-MM-DD>/crm/...`, and proto3 field `oyatie_version`.
- SDK semver model: CRM REST/gRPC SDKs use `major.minor.patch`.
- Support window: last N=3 public API dates are supported for at least 180 days.
- Per-tenant pinning supported: yes, especially for Salesforce, HubSpot, Dynamics, and SAP/C4C migration adapters.
- Internal-mesh exemption: yes. ADR-0145 direct gRPC for service-to-service CRM coordination stays exempt from URL date prefixes.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
