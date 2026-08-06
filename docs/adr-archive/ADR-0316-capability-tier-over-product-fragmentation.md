---
id: ADR-0316
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-engineering
  - council-privacy
  - council-security
  - council-design-system
  - ops-compliance
  - ops-sre-reliability
  - axis-policy-engine
  - axis-ontology
  - axis-workflow-engine
  - axis-tenancy
  - axis-audit-chain
  - axis-foundry
supersedes: []
amends:
  - ADR-0132-product-platform-and-bundle-dissolution.md (adds capability-tier projection doctrine as the successor to per-product fragmentation)
  - ADR-0245-substrate-vs-product-layering.md (adds capability-tier registry as the product-layer activation primitive)
  - ADR-0249-multi-category-marketplace-doctrine.md (declares marketplace categories as tiers and overlays, not fragmented services)
  - ADR-0257-ontology-object-type-versioning-deprecation-handshake.md (requires tier projections to pin object-type schema revisions)
  - ADR-0315-erp-coverage-doctrine-sap-parity.md (reframes ERP parity modules as capability tiers where a distinct operational concern is absent)
superseded_by: [ADR-0329]
amended_by: [ADR-0329, ADR-0330]
supersession_note: "ADR-0329 (Accepted) supersedes this ADR and retires the capability-tier doctrine; ADR-0330 (Accepted) amends with the tenant-class + composable billing-component replacement model. Cross-microservice retirement migration remains scheduled for Wave 15J; implement authority is ADR-0329/0330/0331, not this file."
related:
  - ADR-0132
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0249
  - ADR-0251
  - ADR-0255
  - ADR-0257
  - ADR-0263
  - ADR-0313
  - ADR-0314
  - ADR-0315
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/markdown-retirement-policy.json
  - /specs/microservices/manifest-schema.json
  - /specs/tenant-model.json
  - /specs/compliance-pack-schema.json
  - /specs/cedar-fragment-schema.json
  - /specs/products/ontology.json
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/decisions/ADR-0132-product-platform-and-bundle-dissolution.md
  - docs/decisions/ADR-0245-substrate-vs-product-layering.md
  - docs/decisions/ADR-0249-multi-category-marketplace-doctrine.md
  - docs/decisions/ADR-0257-ontology-object-type-versioning-deprecation-handshake.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
inbound_citations:
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/decisions/ADR-0315-erp-coverage-doctrine-sap-parity.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 2
purpose: >
  Establish capability tiers as the canonical way to express enterprise software
  product surfaces such as CRM, marketing automation, HR, performance management,
  learning management, contract lifecycle management, financial planning, ITSM,
  ERP modules, and content-management variants over oyatie's shared substrate.
  A named product surface is a tenant activation bundle made from Cedar permit
  sets, ontology projections, workflow templates, UX shell manifests, compliance
  overlays, and observability/cost metadata. Adjacent functional categories do
  not become separate product-fragment microservices unless they introduce a
  distinct operational concern that cannot be owned by an existing flat service.
enforcement_status: advisory-until-capability-tier-registry-lands
enforced_by:
  - oya-governance-capability-tier-registry-shape
  - oya-governance-no-product-fragmentation-microservices
  - oya-governance-capability-tier-cedar-coverage
  - oya-governance-capability-tier-ontology-projection-pin
  - oya-governance-capability-tier-workflow-template-coverage
  - oya-governance-capability-tier-ux-shell-coverage
  - oya-governance-capability-tier-compliance-overlay-coverage
  - oya-governance-capability-tier-migration-declaration
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0316: Capability-Tier Over Product Fragmentation Doctrine

## Status

Proposed - 2026-05-20.

This ADR lands after ADR-0313, ADR-0314, and ADR-0315. ADR-0315 proved that SAP-class ERP
parity can be represented by module-to-capability composition rather than a new ERP platform.
ADR-0316 generalizes that result across the rest of enterprise software: CRM, HR, ITSM,
marketing, CLM, learning, performance, analytics, content, support, and adjacent categories
become capability tiers on shared substrate primitives.
Enforcement is advisory until the shared registry crate, Cedar entity types, Postgres
migration, and per-microservice declaration manifests land. After those prerequisites, no
new product-fragment microservice may be merged without satisfying Section D-10.

## Date

2026-05-20.

## Context

This ADR's canonical context is expanded in Section A. The short form is that
enterprise SaaS fragmentation creates duplicate identity, permission, data-model,
workflow, audit, compliance, integration, training, and cost surfaces that oyatie
must not reproduce as per-product microservices.

## Section A - Context

### Section A.1 - Context driver 1
The brief-supplied Gartner 2023 SaaS-sprawl benchmark says the average enterprise runs more
than 110 SaaS applications. Even where the exact count changes by industry and company size,
the architectural pattern is stable: each department buys a point product, each point
product brings a separate identity boundary, a separate permission model, a separate data
model, a separate workflow engine, a separate audit trail, and a separate compliance
posture.

### Section A.2 - Context driver 2
SaaS fragmentation is not merely procurement overhead. It becomes a data-sovereignty and
operational-risk problem because the organization must prove who could see what, when, under
which policy, in which jurisdiction, across dozens or hundreds of vendors that do not share
one subject model or one object model.

### Section A.3 - Context driver 3
The incumbent status quo turns product categories into hard boundaries. Sales buys CRM.
Marketing buys marketing automation. HR buys HCM, performance, learning, applicant tracking,
payroll, and engagement. Legal buys contract lifecycle. IT buys ITSM. Finance buys ERP,
planning, treasury, procurement, and spend tooling. Each purchase adds another island.

### Section A.4 - Context driver 4
The cost compounds. Every island adds SSO integration, SCIM lifecycle mapping, RBAC
translation, audit ingestion, legal review, data-processing agreements, vendor risk review,
training, migration, change management, integration monitoring, and offboarding burden. The
user experience becomes a tour of inconsistent menus, inconsistent object names,
inconsistent reporting semantics, and inconsistent data-retention rules.

### Section A.5 - Context driver 5
The lock-in cycle is predictable. A vendor begins with one category, then expands by
acquisition or module accretion. Customers accept the expansion because the vendor already
owns identity and data. The suite becomes sticky precisely because integration is hard. The
suite then charges rent for the integration burden it helped create.

### Section A.6 - Context driver 6
Oyatie rejects both extremes: it rejects the fragmented point-product portfolio, and it
rejects one monolithic suite microservice. ADR-0132 already forbids new suite and bundle
microservices. ADR-0245 already separates substrate from product layer. ADR-0249 already
models marketplace categories through shared substrates. ADR-0315 already maps ERP modules
to flat services and composition. ADR-0316 names the activation primitive that makes those
doctrines operational.

### Section A.7 - Context driver 7
A capability tier is the tenant-visible projection of shared substrate primitives. The
tenant does not buy a separate CRM service. The tenant receives a named CRM-class capability
tier that activates Cedar permits, ontology object projections, workflow templates, UX shell
widgets, compliance overlays, analytics dashboards, cost dimensions, and operational
runbooks across existing flat services.

### Section A.8 - Context driver 8
This does not mean every category is fake. Some categories introduce genuinely distinct
operational concerns. Contact-center telephony, data-warehouse OLAP, warehouse execution,
production planning, global trade screening, treasury risk, and real-estate lease accounting
may require flat microservices because their bottlenecks, contracts, data-retention rules,
or operational failure modes are distinct. The decision is not product-name driven; it is
operational-concern driven.

### Section A.9 - Context driver 9
The enterprise benchmark file dated 2026-05-21 in this repository identifies 17 net-new
services across the full enterprise matrix. ADR-0316 narrows future expansion pressure by
requiring each candidate category to prove a new operational concern before it becomes a
service. Otherwise it becomes a capability tier on the closest owning service or composition
of owning services.

### Section A.10 - Context driver 10
The doctrine also resolves the product taxonomy tension introduced by adjacent categories.
CRM, customer service, marketing automation, loyalty, contact center, account intelligence,
and revenue operations are not seven identity systems. They are one tenant-scoped customer
graph with different permit sets, projections, workflows, UX shells, and compliance
overlays.

### Section A.11 - Context driver 11
Human capital follows the same pattern. HCM, performance management, learning management,
employee engagement, recruiting, onboarding, offboarding, payroll adjacency, skills graph,
and workforce planning do not need independent user directories or audit systems. They need
tiered projections over community, workplace integration, workflow engine, ontology,
payments, compliance, identity, and analytics.

### Section A.12 - Context driver 12
IT operations follow the same pattern. ITSM, incident management, change management, CMDB,
service catalog, asset management, on-call, endpoint management, and security-response
workflows do not need separate product islands. They need tiered projections over workflow
engine, ontology, observability, audit-chain, policy-engine, tasks, community, and
messaging.

### Section A.13 - SaaS fragmentation tax decomposition

| Tax | Fragmentation effect | Oyatie replacement |
|---|---|---|
| identity-tax | Every point product adds another account lifecycle, session policy, MFA exception path, break-glass path, and offboarding audit trail. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |
| permission-tax | Every point product maps roles differently, so least privilege requires brittle translation tables and repeated access reviews. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |
| data-model-tax | Every point product defines accounts, contacts, employees, contracts, tickets, projects, cost centers, and files differently. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |
| workflow-tax | Every point product embeds workflow state in proprietary objects that cannot be reasoned about by the platform scheduler. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |
| audit-tax | Every point product emits logs in a different schema, retention class, timestamp model, and export path. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |
| compliance-tax | Every point product must be mapped to every pack: KR-CSAP, EU GDPR, FedRAMP High, HIPAA, PCI, SOX, and regional labor rules. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |
| training-tax | Every department must learn a different UI language, role vocabulary, object vocabulary, and reporting grammar. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |
| integration-tax | Every point product demands connectors, ETL jobs, webhook normalizers, retry policies, and reconciliation runbooks. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |
| procurement-tax | Every point product repeats vendor review, security review, renewal, overage negotiation, and license optimization. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |
| exit-tax | Every point product makes data export and migration a vendor-specific project rather than a tenant-owned capability. | Capability-tier projection over shared tenant, Cedar, ontology, workflow, audit, and compliance substrates. |

### Section A.14 - Hyperscaler precedent synthesis

| Precedent | Doctrine imported | Anti-fragmentation lesson |
|---|---|---|
| Salesforce Platform | multitenant, metadata-driven, API-first, app ecosystem, shared lower-layer capabilities, App Cloud shared identity/data/network services. | Mature platforms expose product surfaces by composing shared primitives rather than duplicating identity, workflow, and data planes. |
| ServiceNow Now Platform | no-code and low-code app development, App Engine, workflow automation, governed custom apps on one platform. | Mature platforms expose product surfaces by composing shared primitives rather than duplicating identity, workflow, and data planes. |
| Microsoft 365 | integrated developer platform spanning Teams, Office add-ins, Graph, SharePoint Framework, Power Apps, and store compliance. | Mature platforms expose product surfaces by composing shared primitives rather than duplicating identity, workflow, and data planes. |
| Atlassian Forge | serverless app platform, managed runtime, storage, events, app APIs, data residency, tenant isolation, and marketplace distribution. | Mature platforms expose product surfaces by composing shared primitives rather than duplicating identity, workflow, and data planes. |
| Notion workspace/API | pages, databases, views, connections, capabilities, and shared workspace objects rather than one product per department. | Mature platforms expose product surfaces by composing shared primitives rather than duplicating identity, workflow, and data planes. |
| Palantir Foundry | ontology projection pattern: users see different operational surfaces through object types, actions, functions, and role-bound views. | Mature platforms expose product surfaces by composing shared primitives rather than duplicating identity, workflow, and data planes. |

## Decision

This ADR's canonical decision is expanded in Section B. Adjacent enterprise product
categories become tenant-granted capability tiers over shared primitives unless Section
D-10 proves that a separate reusable substrate service is required.

## Section B - Decision

Oyatie will treat adjacent enterprise product categories as capability tiers over shared
substrate primitives unless the candidate category proves a new operational concern under
Section D-10. A capability tier is a named, versioned, tenant-granted projection bundle
consisting of Cedar permit sets, ontology projections, workflow template libraries, UX shell
manifests, compliance pack overlays, observability metadata, cost metadata, and lifecycle
state.
This is the capability-tiers-as-projection model: product-language outcomes are projections
over shared primitives, not independent product microservices.

### Section B.1 - Normative rule

- A product name MUST NOT create a microservice by itself.
- A department name MUST NOT create a microservice by itself.
- A vendor benchmark category MUST NOT create a microservice by itself.
- A capability tier MUST be activated per tenant through the capability-tier grant registry.
- A capability tier MUST include at least one Cedar permit set.
- A capability tier MUST include at least one ontology projection or explicitly declare why it is workflow-only.
- A capability tier MUST include at least one workflow template or explicitly declare why it is read-only.
- A capability tier MUST include a UX shell manifest unless it is API-only.
- A capability tier MUST include compliance overlay mapping when it touches regulated data or regulated decisions.
- A capability tier MUST pin ontology object-type schema revisions per ADR-0257.
- A capability tier MUST emit audit-chain evidence per ADR-0263.
- A capability tier MUST publish cost dimensions for FinOps allocation.

### Section B.2 - Product-fragmentation prohibition

B2.01. Prohibited product-fragment shape: crm as a duplicate account/contact/task/workflow/audit island.
B2.02. Prohibited product-fragment shape: marketing-automation as a duplicate customer profile and consent island.
B2.03. Prohibited product-fragment shape: hr as a duplicate employee directory and policy island.
B2.04. Prohibited product-fragment shape: performance-management as a duplicate goals/reviews/workflow island.
B2.05. Prohibited product-fragment shape: learning-management as a duplicate identity/course/compliance island.
B2.06. Prohibited product-fragment shape: contract-lifecycle-management as a duplicate document/workflow/signature island.
B2.07. Prohibited product-fragment shape: financial-planning as a duplicate cost-center/forecast/workflow island.
B2.08. Prohibited product-fragment shape: itsm as a duplicate ticket/asset/workflow/audit island.
B2.09. Prohibited product-fragment shape: customer-service as a duplicate case/community/messenger island.
B2.10. Prohibited product-fragment shape: procurement as a duplicate supplier/deal/approval island.
B2.11. Prohibited product-fragment shape: loyalty as a duplicate account/reward/consent island.
B2.12. Prohibited product-fragment shape: content-management as a duplicate drive/retention/search island.

The prohibition does not erase domain language. Tenants may see labels such as Sales Cloud,
Service Desk, Learning, Performance, Procurement, Contract Lifecycle, or Finance Planning.
Those labels are UX and GTM labels bound to capability-tier ids, not service boundaries.

### Section B.3 - Activation bundle

| Activation field | Required meaning |
|---|---|
| capability_tier_id | Stable id such as sales-cloud-core, marketing-automation-core, itsm-core, learning-management-core. |
| tenant_id | Tenant scope from ADR-0244. |
| grant_id | Stable registry id for the activation record. |
| permit_set_id | Cedar permit set resolving role-to-action allow/forbid policy. |
| role_assignment_policy | Mapping from tenant roles to capability actions. |
| ontology_projection_id | Object-type projection manifest with schema revision pins. |
| workflow_template_library_id | Template family loaded into workflow-engine for this tier. |
| ux_shell_manifest_id | Navigation, widgets, gestures, and command palette entries. |
| compliance_pack_overlay_id | Pack overlay for jurisdiction, data class, and regulated decision constraints. |
| audit_profile_id | Audit event classes, trace spans, log schemas, and retention. |
| cost_profile_id | FinOps attribution dimensions and budget default. |
| lifecycle_state | draft, preview, active, deprecated, sunset_pending, sunset. |

### Section B.4 - Rejected alternatives

| Alternative | Claimed benefit | Rejection reason |
|---|---|---|
| Status quo point-product acquisition | Short-term procurement speed. | Permanent identity, policy, data, workflow, audit, and compliance fragmentation. |
| One tenant-rbac microservice | Single owner and familiar grouping label. | Violates ADR-0132 and creates a monolith with broad blast radius. |
| One microservice per enterprise category | Clear product ownership labels. | Duplicates substrate primitives and recreates SaaS sprawl inside oyatie. |
| Config flags only | Cheap first implementation. | Flags lack Cedar evidence, schema pins, lifecycle state, and auditability. |
| Marketplace plugins for every category | Externalizes build effort. | Cannot satisfy first-party compliance, tenant portability, and hyperscaler-grade audit needs. |
| Ontology projections without Cedar | Simpler read model. | Read visibility without action authorization violates ADR-0243. |
| Cedar permits without ontology pins | Simpler authorization surface. | Schema drift violates ADR-0257 and breaks consumer stability. |
| Workflow templates without UX shells | Backend-first delivery. | Tenants cannot operate product surfaces ergonomically. |

## Consequences

This ADR's canonical consequences are expanded in Section C. The durable effect is
lower service-count pressure, stronger observability consistency, clearer cost allocation,
and stricter review gates for every proposed product-fragment microservice.

## Section C - Consequences

| Dimension | Consequence | Acceptance signal |
|---|---|---|
| Maintainability | Product semantics move into versioned tier manifests rather than hard-coded service forks. | A future engineer changes one projection manifest, one Cedar permit set, or one workflow template without creating another service island. |
| Observability | Every tier emits audit, metric, trace, and log dimensions that include tenant, tier, pack, workflow template, and ontology projection ids. | Production diagnosis can distinguish product-surface behavior without splitting telemetry stacks. |
| Scalability | Shared substrates scale on their actual bottlenecks, while tier grants control tenant-visible activation. | CRM account reads scale in ontology; case workflow scales in workflow-engine; mail sends scale in mail; no CRM monolith scales all at once. |
| Performance | Tier projections add bounded authorization and projection joins but remove cross-SaaS synchronization latency. | Single-tenant tier activation path targets p95 under 250 ms for permit evaluation plus projection lookup on warm cache. |
| Optimization | Cost attribution is per tier and per primitive, so product P&L is visible without duplicate infrastructure. | FinOps can report sales-cloud-core cost by workflow runs, ontology reads, messenger sends, mail sends, and analytics queries. |
| Code quality | Tier registry code becomes a shared crate with property tests, migration tests, Cedar fixture tests, and manifest lint. | No product-specific copy-paste authorization or schema logic lands in adjacent services. |

### Section C.1 - Maintainability
Product semantics move into versioned tier manifests rather than hard-coded service forks.
A future engineer changes one projection manifest, one Cedar permit set, or one workflow
template without creating another service island.
Risk: the registry becomes too permissive if reviewers treat tier activation as a
lightweight flag. Mitigation: each tier grant is Cedar-gated, lifecycle-tracked,
version-pinned, and audit-sealed.

### Section C.2 - Observability
Every tier emits audit, metric, trace, and log dimensions that include tenant, tier, pack,
workflow template, and ontology projection ids.
Production diagnosis can distinguish product-surface behavior without splitting telemetry
stacks.
Risk: the registry becomes too permissive if reviewers treat tier activation as a
lightweight flag. Mitigation: each tier grant is Cedar-gated, lifecycle-tracked,
version-pinned, and audit-sealed.

### Section C.3 - Scalability
Shared substrates scale on their actual bottlenecks, while tier grants control
tenant-visible activation.
CRM account reads scale in ontology; case workflow scales in workflow-engine; mail sends
scale in mail; no CRM monolith scales all at once.
Risk: the registry becomes too permissive if reviewers treat tier activation as a
lightweight flag. Mitigation: each tier grant is Cedar-gated, lifecycle-tracked,
version-pinned, and audit-sealed.

### Section C.4 - Performance
Tier projections add bounded authorization and projection joins but remove cross-SaaS
synchronization latency.
Single-tenant tier activation path targets p95 under 250 ms for permit evaluation plus
projection lookup on warm cache.
Risk: the registry becomes too permissive if reviewers treat tier activation as a
lightweight flag. Mitigation: each tier grant is Cedar-gated, lifecycle-tracked,
version-pinned, and audit-sealed.

### Section C.5 - Optimization
Cost attribution is per tier and per primitive, so product P&L is visible without duplicate
infrastructure.
FinOps can report sales-cloud-core cost by workflow runs, ontology reads, messenger sends,
mail sends, and analytics queries.
Risk: the registry becomes too permissive if reviewers treat tier activation as a
lightweight flag. Mitigation: each tier grant is Cedar-gated, lifecycle-tracked,
version-pinned, and audit-sealed.

### Section C.6 - Code quality
Tier registry code becomes a shared crate with property tests, migration tests, Cedar
fixture tests, and manifest lint.
No product-specific copy-paste authorization or schema logic lands in adjacent services.
Risk: the registry becomes too permissive if reviewers treat tier activation as a
lightweight flag. Mitigation: each tier grant is Cedar-gated, lifecycle-tracked,
version-pinned, and audit-sealed.

## Section D - Detailed mechanics

### Section D-1 - Capability-tier-as-Cedar-permit-set

Every product surface begins as a named Cedar permit set. A permit set is not one role. It
is a bundle of Cedar entity types, action namespaces, role assignment rules, default-deny
forbid rules, pack overlays, and evidence requirements. The same microservice can expose
many permit sets; the same tenant can activate multiple tiers; the same user can receive
different tier grants in different tenant contexts.

```cedar
entity Tenant;
entity Principal;
entity CapabilityTier;
entity CapabilityTierGrant;
entity PermitSet;
entity OntologyProjection;
entity WorkflowTemplateLibrary;
entity UxShell;
entity CompliancePack;
entity Microservice;
action "capability_tier.grant.create";
action "capability_tier.grant.activate";
action "capability_tier.grant.revoke";
action "capability_tier.projection.read";
action "capability_tier.workflow.start";
action "capability_tier.ux.render";
action "capability_tier.compliance.override";
permit (principal, action, resource)
when {
  resource is CapabilityTierGrant &&
  resource.tenant == context.tenant &&
  resource.lifecycle_state == "active" &&
  context.permit_set in resource.permit_sets &&
  context.pack_overlay in resource.compliance_pack_overlays
};
forbid (principal, action, resource)
when {
  resource is CapabilityTierGrant &&
  context.data_class in ["PHI", "PCI", "KR_RRN"] &&
  !(context.pack_overlay in resource.regulated_pack_overlays)
};
```

| Tier id | Example Cedar action bundle |
|---|---|
| sales-cloud-core | crm.account.read, crm.opportunity.write, community.profile.read, marketplace.deal.offer.create |
| service-cloud-core | crm.case.read, crm.case.write, messenger.conversation.reply, community.thread.moderate |
| marketing-automation-core | marketing.segment.read, mail.campaign.send, messenger.broadcast.send, consent.preference.read |
| hr-core | workplace.employee.read, workflow.onboarding.start, payments.payroll.preview, compliance.labor.pack.read |
| performance-management-core | goal.read, goal.write, review.start, calibration.workflow.approve |
| learning-management-core | course.read, course.assign, completion.record, compliance.training.report |
| contract-lifecycle-core | contract.draft, contract.approve, drive.document.read, marketplace.deal.amend |
| financial-planning-core | forecast.read, forecast.write, finops.budget.approve, analytics.model.run |
| itsm-core | incident.create, change.approve, asset.read, observability.signal.read |
| content-classification-core | drive.file.classify, retention.policy.apply, ediscovery.hold.create, audit.export |

### Section D-2 - Per-capability-tier ontology projection

A tier projection declares which ontology object types, relation types, actions, functions,
computed properties, and schema revisions are surfaced to a tenant role. The projection is
the antidote to product-specific data models. It gives each department the objects it
expects without duplicating the canonical object graph.

D2.field.01. projection_id is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.02. capability_tier_id is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.03. tenant_id is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.04. object_type_refs is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.05. relation_type_refs is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.06. action_refs is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.07. function_refs is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.08. schema_revision_pins is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.09. field_visibility_rules is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.10. computed_property_rules is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.11. jurisdiction_filters is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.12. pack_filters is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.13. audit_redaction_rules is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.14. export_policy_ref is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.15. search_index_scope is required in the ontology projection manifest or explicitly denied by the tier registry validator.
D2.field.16. lineage_policy_ref is required in the ontology projection manifest or explicitly denied by the tier registry validator.

| Tier id | Object types surfaced |
|---|---|
| sales-cloud-core | Account, Contact, Opportunity, DealSet, Task, MessageThread, MailMessage, ForecastSnapshot |
| service-cloud-core | Account, Contact, Case, Entitlement, Incident, KnowledgeArticle, MessageThread, SatisfactionSurvey |
| marketing-automation-core | Segment, Campaign, ConsentPreference, Contact, ContentAsset, MailMessage, JourneyRun, AttributionEvent |
| hr-core | Employee, Position, OrgUnit, PayrollPreview, LeaveRequest, OnboardingRun, PolicyAcknowledgement |
| performance-management-core | Goal, ReviewCycle, Feedback, CalibrationGroup, PromotionPacket, CompensationBand |
| learning-management-core | Course, Module, Assignment, Completion, Credential, ComplianceTrainingRequirement |
| contract-lifecycle-core | Contract, Clause, Counterparty, ApprovalRoute, SignatureEnvelope, DealSet, Obligation |
| financial-planning-core | Budget, Forecast, CostCenter, Scenario, Variance, ApprovalRun, AnalyticsWorkbook |
| itsm-core | Incident, Problem, ChangeRequest, Asset, ConfigurationItem, Service, OnCallSchedule |
| content-classification-core | File, Folder, ClassificationLabel, RetentionHold, DsrExport, LegalMatter |

### Section D-3 - Per-capability-tier workflow template library

A tier loads workflow templates as product behavior. This preserves ADR-0132 flat-service
boundaries because the Workflow Engine owns execution semantics while the tier registry owns
which templates are visible and preloaded for the tenant.

D3.sales-cloud-core. Template library includes lead-to-opportunity, quote-approval, deal-desk-review, renewal-risk-escalation.
D3.service-cloud-core. Template library includes case-triage, sla-breach-escalation, refund-approval, knowledge-article-review.
D3.marketing-automation-core. Template library includes campaign-launch-approval, consent-refresh, journey-pause-on-complaint, lead-nurture.
D3.hr-core. Template library includes employee-onboarding, leave-approval, policy-attestation, offboarding-checklist.
D3.performance-management-core. Template library includes review-cycle-open, calibration-session, promotion-packet, performance-improvement-plan.
D3.learning-management-core. Template library includes course-assignment, completion-reminder, certification-renewal, mandatory-training-escalation.
D3.contract-lifecycle-core. Template library includes contract-intake, legal-review, signature-routing, renewal-notice.
D3.financial-planning-core. Template library includes budget-cycle-open, forecast-submit, variance-review, scenario-approval.
D3.itsm-core. Template library includes incident-triage, problem-root-cause, change-advisory-board, post-incident-review.
D3.content-classification-core. Template library includes classification-review, retention-hold, ediscovery-export, dsr-collection.

### Section D-4 - Per-capability-tier UX shell

A UX shell is the user-facing product illusion. It selects navigation, widgets, dashboards,
command palette verbs, gestures, empty states, bulk actions, mobile surfaces, and help
labels. UX shells do not own authorization, data, or workflow execution; they render a
tier-specific projection of those primitives.

D4.shell.01. nav_tree MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.02. home_dashboard MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.03. object_table_presets MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.04. detail_panel_layouts MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.05. command_palette_verbs MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.06. bulk_actions MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.07. mobile_tabs MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.08. notification_preferences MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.09. accessibility_profile MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.10. localization_profile MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.11. analytics_widgets MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.
D4.shell.12. embedded_workflow_cards MUST be declared for interactive tiers or explicitly marked not-applicable for API-only tiers.

### Section D-5 - Per-capability-tier compliance pack overlay

Compliance overlays bind a tier to data class, jurisdiction, tenant residency, sovereign
cell, regulated-decision, retention, export, and evidence rules. The same tier may be active
in one pack and constrained in another. A marketing tier in the EU must enforce consent and
profiling constraints. A HR tier in Korea must enforce labor, resident identifier, and
privacy constraints. A financial tier in the US must enforce SOX, GLBA, PCI, or
money-transmission overlays as applicable.

D5.pack.01. pack-kr-csap-pipa-fss: Korea CSAP, PIPA, FSS, resident identifier, financial-sector residency.
D5.pack.02. pack-eu-gdpr-ai-act: GDPR, EU AI Act, works council, data minimization, profiling limits.
D5.pack.03. pack-us-fedramp-high-il5: FedRAMP High, IL5/IL6, agency boundary, audit retention.
D5.pack.04. pack-us-healthcare-hipaa: HIPAA, minimum necessary, BAA, PHI audit.
D5.pack.05. pack-us-sox-pci-glba: SOX, PCI DSS, GLBA, financial reporting control.
D5.pack.06. pack-cn-pipl-csl-dsl: PIPL, Cybersecurity Law, Data Security Law, cross-border export.
D5.pack.07. pack-br-lgpd: LGPD, data subject request, lawful basis.
D5.pack.08. pack-sg-pdpa-mas: PDPA, MAS technology risk, financial audit.

### Section D-6 - Capability-tier composition

A Sales-Cloud-class capability tier is not one service. It is a composition of
workflow-engine, ontology, community, marketplace, messenger, mail, intelligence, analytics,
policy-engine, tenancy, audit-chain, payments, and finops tiers. Composition makes
enterprise product labels possible without sacrificing flat-service ownership.

| Tier class | Composed owning services |
|---|---|
| sales-cloud-class | workflow-engine, ontology, community, marketplace, messenger, mail, intelligence, analytics |
| service-cloud-class | workflow-engine, ontology, community, messenger, mail, observability, audit-chain, analytics |
| marketing-cloud-class | mail, messenger, ontology, consent-graph, workflow-engine, analytics, intelligence |
| hcm-class | community, workplace-integration, workflow-engine, ontology, payments, compliance, identity |
| lms-class | docs, drive, workflow-engine, community, analytics, compliance, identity |
| clm-class | drive, docs, workflow-engine, marketplace, payments, audit-chain, identity |
| fpna-class | finops-portal, analytics, sheets, ontology, workflow-engine, payments |
| itsm-class | tasks, workflow-engine, observability, ontology, messenger, audit-chain |

### Section D-7 - Per-microservice capability-tier registry shape

Every microservice that exposes product behavior declares the capability tiers it
contributes to. The declaration is per microservice because ownership remains flat. The
shared registry joins tenant grants with per-service tier contributions.

```sql
CREATE TABLE tenant_capability_tier_grants (
    grant_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    capability_tier_id TEXT NOT NULL,
    capability_tier_version TEXT NOT NULL,
    lifecycle_state TEXT NOT NULL CHECK (lifecycle_state IN (
        'draft', 'preview', 'active', 'deprecated', 'sunset_pending', 'sunset'
    )),
    home_cell_id TEXT NOT NULL,
    jurisdiction_pack_id TEXT NOT NULL,
    permit_set_ids TEXT[] NOT NULL,
    ontology_projection_ids TEXT[] NOT NULL,
    workflow_template_library_ids TEXT[] NOT NULL,
    ux_shell_manifest_ids TEXT[] NOT NULL,
    compliance_overlay_ids TEXT[] NOT NULL,
    observability_profile_id TEXT NOT NULL,
    cost_profile_id TEXT NOT NULL,
    activated_by_principal_id TEXT NOT NULL,
    activated_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    sunset_after TIMESTAMPTZ,
    schema_revision_pin JSONB NOT NULL,
    evidence_ref TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, capability_tier_id, capability_tier_version)
);
CREATE INDEX tenant_capability_tier_grants_tenant_idx
    ON tenant_capability_tier_grants (tenant_id, lifecycle_state);
CREATE INDEX tenant_capability_tier_grants_tier_idx
    ON tenant_capability_tier_grants (capability_tier_id, capability_tier_version);
CREATE INDEX tenant_capability_tier_grants_pack_idx
    ON tenant_capability_tier_grants (jurisdiction_pack_id, home_cell_id);
CREATE TABLE microservice_capability_tier_contributions (
    contribution_id UUID PRIMARY KEY,
    microservice_id TEXT NOT NULL,
    capability_tier_id TEXT NOT NULL,
    contribution_kind TEXT NOT NULL CHECK (contribution_kind IN (
        'permit_set', 'ontology_projection', 'workflow_template', 'ux_shell',
        'compliance_overlay', 'analytics_widget', 'cost_dimension', 'runbook'
    )),
    artifact_ref TEXT NOT NULL,
    schema_revision TEXT NOT NULL,
    min_registry_version TEXT NOT NULL,
    owner_team TEXT NOT NULL,
    evidence_ref TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (microservice_id, capability_tier_id, contribution_kind, artifact_ref)
);
```

### Section D-8 - Capability-tier promotion and sunset lifecycle

D8.state.draft. Tier exists in registry, not grantable to production tenants.
D8.state.preview. Tier may be granted to allowlisted tenants under explicit preview evidence.
D8.state.active. Tier may be granted by normal tenant admins with required Cedar permits.
D8.state.deprecated. Tier remains usable but emits deprecation evidence and blocks new grants unless exceptioned.
D8.state.sunset_pending. Tier has a dated exit path, export path, replacement tier, and tenant notification evidence.
D8.state.sunset. Tier cannot execute user actions; only read-only export and audit queries remain.

- Promotion from draft to preview requires Cedar fixtures, ontology projection pins, workflow template tests, UX shell accessibility review, and compliance overlay review.
- Promotion from preview to active requires at least one tenant dry-run grant, audit-chain evidence, rollback rehearsal, and cost profile confirmation.
- Promotion from active to deprecated requires replacement path, tenant notice, export semantics, and support runbook.
- Promotion from deprecated to sunset_pending requires freeze of new grants and evidence that every active tenant has an exit decision.
- Promotion from sunset_pending to sunset requires no active workflow runs and no remaining write permits.

### Section D-9 - Cross-jurisdiction capability-tier interaction

A tenant can activate multiple packs and multiple tiers. The tier registry must resolve the
most restrictive applicable overlay at action time. If a Sales tier wants to read a contact
in an EU pack, a Marketing tier wants to profile that contact, and a Service tier wants to
inspect support cases, Cedar receives the tier id, pack id, data class, purpose of use,
schema revision, residency cell, and principal role as context.
D9.interaction.01. eu-sales-marketing: Sales may read account relationship data; Marketing profiling requires consent and purpose-of-use evidence.
D9.interaction.02. kr-hr-payroll: HR may read employee data; payroll preview requires resident identifier protections and finance-pack evidence.
D9.interaction.03. us-health-service: Service may process a case; PHI fields require HIPAA minimum-necessary overlay.
D9.interaction.04. fed-itsm-observability: ITSM may open incident data; classified workloads require IL5/IL6 cell and audit retention.
D9.interaction.05. cn-analytics-export: Analytics may compute local reports; cross-border export requires PIPL export approval.
D9.interaction.06. global-trade-sales: Sales DealSet creation pauses when global-trade screens a sanctioned counterparty.

### Section D-10 - When a new microservice is required

A new microservice is required only when a candidate category introduces a distinct
operational concern that cannot be safely owned by existing flat services or their
composition. The test is intentionally stricter than product naming and stricter than
vendor-module parity.
D10.test.01. distinct-write-authority: The category owns a source-of-truth write model no existing service can own without violating bounded context.
D10.test.02. distinct-scale-axis: The category scales on a bottleneck unrelated to existing services, such as call-center concurrency, OLAP scans, warehouse task dispatch, or market-data ticks.
D10.test.03. distinct-failure-mode: The category fails in ways requiring independent blast-radius controls, such as telephony outage, customs hold, production-line stop, or liquidity breach.
D10.test.04. distinct-regulatory-license: The category needs a license, certification, or regulated operational control not shared by existing owners.
D10.test.05. distinct-contract-surface: The category publishes OpenAPI, AsyncAPI, proto, event, or CLI contracts that would overload an existing service boundary.
D10.test.06. distinct-data-retention-class: The category has retention, legal hold, or export semantics incompatible with existing service stores.
D10.test.07. distinct-runtime: The category needs a runtime class not present in existing services, such as media relay, telephony routing, OLAP warehouse, or industrial integration.
D10.test.08. distinct-oncall-specialty: The category requires specialized on-call runbooks and escalation ownership beyond existing service teams.

| Candidate category | Default decision | Rationale |
|---|---|---|
| crm | capability-tier-first | Customer graph, cases, opportunities, and account workflows can project over ontology, community, marketplace, messenger, mail, workflow, analytics, and intelligence. |
| marketing-automation | capability-tier-first | Segmentation, journeys, campaigns, and attribution compose from consent, mail, messenger, ontology, analytics, and workflow. |
| contact-center | new-service-likely | Voice/SMS routing, telephony carrier integration, recording, and live queue concurrency are distinct runtime and scale concerns. |
| performance-management | capability-tier-first | Goals, reviews, calibration, and feedback compose from community, workflow, ontology, analytics, and HR overlays. |
| learning-management | capability-tier-first | Courses, completions, credentials, and compliance training compose from docs, drive, community, workflow, analytics, and compliance. |
| contract-lifecycle-management | capability-tier-first | Drafting, clause libraries, approvals, signatures, obligations, and renewals compose from drive, docs, workflow, marketplace DealSet, and audit-chain. |
| financial-planning | capability-tier-first | Forecasting and planning compose from finops, analytics, sheets, ontology, and workflow unless OLAP warehouse requirements split out. |
| data-warehouse | new-service-likely | Tenant OLAP, scan scheduling, columnar storage, and query governance are distinct storage/runtime concerns. |
| itsm | capability-tier-first | Incidents, changes, service catalog, and assets compose from tasks, workflow, ontology, observability, messenger, and audit-chain. |
| incident-management | new-service-possible | On-call paging and escalation may require dedicated delivery guarantees distinct from tasks and messenger. |
| warehouse | new-service-required | Warehouse execution has physical task dispatch, inventory movement, scanner/device integration, and yard management failure modes. |
| treasury | new-service-required | Liquidity, FX, debt, hedge, cash position, and bank control semantics are distinct from payments rails. |

## Section E - Implementation footprint

### Section E.1 - Shared crate

Add a new crate named oya-shared-capability-tier-registry. The crate is a shared kernel
crate, not a product service. It owns pure domain types, validation functions, registry
diffing, lifecycle transition rules, and fixture builders used by service-specific adapters.
E1.file. crates/oya-shared-capability-tier-registry/Cargo.toml
E1.file. crates/oya-shared-capability-tier-registry/src/lib.rs
E1.file. crates/oya-shared-capability-tier-registry/src/grant.rs
E1.file. crates/oya-shared-capability-tier-registry/src/contribution.rs
E1.file. crates/oya-shared-capability-tier-registry/src/lifecycle.rs
E1.file. crates/oya-shared-capability-tier-registry/src/validation.rs
E1.file. crates/oya-shared-capability-tier-registry/src/cedar_entities.rs
E1.file. crates/oya-shared-capability-tier-registry/src/ontology_projection.rs
E1.file. crates/oya-shared-capability-tier-registry/src/workflow_templates.rs
E1.file. crates/oya-shared-capability-tier-registry/src/compliance_overlay.rs
E1.file. crates/oya-shared-capability-tier-registry/tests/grant_lifecycle.rs
E1.file. crates/oya-shared-capability-tier-registry/tests/no_product_fragmentation.rs
E1.file. registry/catalog/oya-shared-capability-tier-registry.yaml
E1.file. specs/capability-tier-registry-schema.json
E1.file. specs/capability-tier-grant-schema.json
E1.file. docs/runbooks/capability-tier-grant-rollback.md

```rust
pub struct CapabilityTierId(String);
pub struct TenantId(String);
pub struct CapabilityTierGrant {
    pub grant_id: String,
    pub tenant_id: TenantId,
    pub capability_tier_id: CapabilityTierId,
    pub version: String,
    pub lifecycle_state: CapabilityTierLifecycleState,
    pub permit_set_ids: Vec<String>,
    pub ontology_projection_ids: Vec<String>,
    pub workflow_template_library_ids: Vec<String>,
    pub ux_shell_manifest_ids: Vec<String>,
    pub compliance_overlay_ids: Vec<String>,
    pub evidence_ref: String,
}
pub enum CapabilityTierLifecycleState {
    Draft,
    Preview,
    Active,
    Deprecated,
    SunsetPending,
    Sunset,
}
pub trait CapabilityTierRegistry {
    fn validate_grant(&self, grant: &CapabilityTierGrant) -> Result<(), CapabilityTierError>;
    fn activate(&self, grant_id: &str, evidence_ref: &str) -> Result<(), CapabilityTierError>;
    fn revoke(&self, grant_id: &str, evidence_ref: &str) -> Result<(), CapabilityTierError>;
    fn contributions_for(&self, tier_id: &CapabilityTierId) -> Vec<CapabilityTierContribution>;
}
```

### Section E.2 - Cedar entity types
E2.entity. Tenant MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. Principal MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. CapabilityTier MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. CapabilityTierGrant MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. PermitSet MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. TierRoleAssignment MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. OntologyProjection MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. WorkflowTemplateLibrary MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. UxShell MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. CompliancePack MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. MicroserviceContribution MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. AuditProfile MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. CostProfile MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. JurisdictionPack MUST be declared in policy-engine and included in Cedar fixture tests.
E2.entity. SchemaRevisionPin MUST be declared in policy-engine and included in Cedar fixture tests.

### Section E.3 - Postgres migrations
E3.migration. microservices/tenancy/migrations/00NN_tenant_capability_tier_grants.sql
E3.migration. microservices/policy-engine/migrations/00NN_capability_tier_cedar_entities.sql
E3.migration. microservices/ontology/migrations/00NN_capability_tier_projection_pins.sql
E3.migration. microservices/workflow-engine/migrations/00NN_capability_tier_template_libraries.sql

### Section E.4 - Validator lanes
E4.lane.capability-tier-registry-shape: Validates grant schema, contribution schema, lifecycle enum, and required evidence fields.
E4.lane.no-product-fragmentation-microservices: Blocks new product-named services unless Section D-10 evidence is present.
E4.lane.capability-tier-cedar-coverage: Ensures every tier action has permit and default-deny forbid coverage.
E4.lane.capability-tier-ontology-projection-pin: Ensures every projection pins object-type schema revisions per ADR-0257.
E4.lane.capability-tier-workflow-template-coverage: Ensures every write tier has workflow template declarations.
E4.lane.capability-tier-ux-shell-coverage: Ensures every interactive tier has a UX shell manifest.
E4.lane.capability-tier-compliance-overlay-coverage: Ensures regulated data and regulated decisions map to packs.
E4.lane.capability-tier-migration-declaration: Ensures each existing service declares contributed tiers or no-tier posture.

## Section F - Migration

Migration is declaration-first. No service is restructured by this ADR. Every existing
service that contributes product behavior declares the capability tiers it exposes. Existing
product docs and PRDs are amended incrementally to name tier ids, permit sets, ontology
projections, workflow libraries, UX shells, and pack overlays. Services that do not expose
product behavior declare no-tier posture.

### Section F.1 - Migration waves
F1.wave-0-inventory: List every existing microservice and identify current product-surface behavior.
F1.wave-1-declare: Add microservice capability-tier contribution manifests without changing runtime behavior.
F1.wave-2-cedar: Publish Cedar permit sets and default-deny fragments for each declared tier.
F1.wave-3-ontology: Publish ontology projection manifests and schema revision pins.
F1.wave-4-workflow: Publish workflow template library ids and replay fixtures.
F1.wave-5-ux: Publish UX shell manifests and accessibility evidence.
F1.wave-6-compliance: Publish pack overlays and jurisdiction interaction fixtures.
F1.wave-7-grants: Backfill tenant_capability_tier_grants for preview tenants.
F1.wave-8-read-flip: Flip product shell reads to registry-driven projections.
F1.wave-9-write-flip: Flip product shell writes to registry-driven Cedar and workflow paths.
F1.wave-10-sunset-fragments: Retire duplicate product-specific flags, roles, and projection copies.

### Section F.2 - Existing microservice declaration examples
F2.service.community. Declares contributions to hr-core, performance-management-core, sales-cloud-class, service-cloud-class.
F2.service.marketplace. Declares contributions to sales-cloud-class, procurement-core, contract-lifecycle-core, commerce-core.
F2.service.messenger. Declares contributions to service-cloud-core, marketing-automation-core, itsm-core, sales-cloud-class.
F2.service.mail. Declares contributions to marketing-automation-core, sales-cloud-class, service-cloud-core, learning-management-core.
F2.service.analytics. Declares contributions to sales-cloud-class, marketing-automation-core, financial-planning-core, performance-management-core.
F2.service.intelligence. Declares contributions to sales-cloud-class, service-cloud-core, marketing-automation-core, learning-management-core.
F2.service.workflow-engine. Declares contributions to all-write-tiers.
F2.service.ontology. Declares contributions to all-object-projection-tiers.
F2.service.policy-engine. Declares contributions to all-cedar-permit-tiers.
F2.service.audit-chain. Declares contributions to all-audit-emitting-tiers.
F2.service.drive. Declares contributions to contract-lifecycle-core, content-classification-core, learning-management-core.
F2.service.docs. Declares contributions to contract-lifecycle-core, learning-management-core, content-authoring-core.
F2.service.sheets. Declares contributions to financial-planning-core, sales-forecasting-core, analytics-workbook-core.
F2.service.tasks. Declares contributions to itsm-core, project-management-core, sales-cloud-class.
F2.service.observability. Declares contributions to itsm-core, incident-management-core, service-health-core.
F2.service.payments. Declares contributions to sales-cloud-class, contract-lifecycle-core, hr-core, procurement-core.
F2.service.finops-portal. Declares contributions to financial-planning-core, cost-management-core, erp-controlling-core.
F2.service.compliance. Declares contributions to all-regulated-pack-tiers.
F2.service.tenancy. Declares contributions to all-tenant-grant-tiers.
F2.service.identity. Declares contributions to all-principal-role-assignment-tiers.

### Section F.3 - Migration safety rules
F3.rule.01. No existing tenant grant is inferred without audit-chain evidence.
F3.rule.02. No product-specific role is deleted until the corresponding Cedar permit set is active and tested.
F3.rule.03. No product-specific object projection is deleted until the ontology projection pin is active and consumers pass replay tests.
F3.rule.04. No product-specific workflow template is retired until the tier template library produces equivalent workflow state transitions.
F3.rule.05. No UX shell flips to tier-driven navigation until accessibility and localization checks pass.
F3.rule.06. No regulated tier activates in a pack until compliance overlay fixtures pass.
F3.rule.07. No sunset begins until export path and tenant notification evidence exist.
F3.rule.08. No registry migration may collapse tenant data across sovereign child tenants from ADR-0313.
F3.rule.09. No marketplace DealSet tier may bypass ADR-0314 settlement semantics.
F3.rule.10. No ERP parity tier may contradict ADR-0315 module mapping.

## Section G - References

- ADR-0132: No-grouping forward policy and flat catalog doctrine.
- ADR-0245: Substrate vs product layering and tier classification.
- ADR-0249: Marketplace as multi-category surface over shared substrates.
- ADR-0257: Ontology object-type versioning and deprecation handshake.
- ADR-0313: Conglomerate tenant hierarchy and sovereign child tenants.
- ADR-0314: Marketplace as universal deal-settlement substrate.
- ADR-0315: ERP coverage doctrine and SAP parity by composition.
- Salesforce Platform architecture: https://architect.salesforce.com/docs/architect/fundamentals/guide/platform-transformation.html
- Salesforce App Cloud announcement: https://investor.salesforce.com/news/news-details/2015/Salesforce-Announces-Salesforce-App-Cloud-A-Unified-Platform-for-Building-Connected-Apps-Fast/default.aspx
- ServiceNow app development and low-code docs: https://www.servicenow.com/docs/r/hyperautomation-low-code/hyperautomation-low-code-landing-page.html
- Microsoft 365 developer platform docs: https://learn.microsoft.com/en-us/microsoft-365/developer/
- Microsoft Graph overview: https://learn.microsoft.com/en-us/graph/overview
- Atlassian Forge platform docs: https://developer.atlassian.com/platform/forge/introduction/the-forge-platform/
- Notion API overview: https://developers.notion.com/guides/get-started/overview
- Notion connection capabilities: https://developers.notion.com/reference/capabilities
- Gartner enterprise software market share analysis 2023: https://www.gartner.com/en/documents/5656823

## Section H - Change log and naming justifications

| Date | Change | Rationale |
|---|---|---|
| 2026-05-20 | ADR-0316 authored. | Generalizes ADR-0315 ERP composition into capability-tier doctrine across enterprise categories. |
| 2026-05-20 | Chose capability-tier over product-module. | Tier signals tenant activation and policy projection; product-module implies service fragmentation. |
| 2026-05-20 | Chose tenant_capability_tier_grants as table name. | Tenant scope is primary, and grants are lifecycle-governed records rather than flags. |
| 2026-05-20 | Chose oya-shared-capability-tier-registry as crate name. | Shared crate owns registry invariants without becoming a runtime service. |
| 2026-05-20 | Chose no-product-fragmentation validator name. | The forbidden pattern is product-fragment services, not product language or user-visible labels. |

## Appendix I - Capability-tier example ledger

I.001. Tier sales-cloud-core.
I.001. Label: Account workspace.
I.001. Permit seed: crm.account.read.
I.001. Ontology projection: Account, Contact, Opportunity.
I.001. Workflow template: lead-to-opportunity.
I.001. UX shell: Sales nav with account dashboard.
I.002. Tier sales-forecasting-core.
I.002. Label: Forecast workspace.
I.002. Permit seed: forecast.submit.
I.002. Ontology projection: ForecastSnapshot, Opportunity.
I.002. Workflow template: forecast-submit.
I.002. UX shell: Forecast grid and variance chart.
I.003. Tier service-cloud-core.
I.003. Label: Case workspace.
I.003. Permit seed: crm.case.write.
I.003. Ontology projection: Case, Entitlement, Contact.
I.003. Workflow template: case-triage.
I.003. UX shell: Case queue and SLA banner.
I.004. Tier customer-success-core.
I.004. Label: Health workspace.
I.004. Permit seed: success.health.read.
I.004. Ontology projection: AccountHealth, RenewalRisk.
I.004. Workflow template: renewal-risk-escalation.
I.004. UX shell: Health dashboard.
I.005. Tier marketing-automation-core.
I.005. Label: Campaign workspace.
I.005. Permit seed: campaign.launch.
I.005. Ontology projection: Campaign, Segment, ConsentPreference.
I.005. Workflow template: campaign-launch-approval.
I.005. UX shell: Journey canvas.
I.006. Tier email-marketing-core.
I.006. Label: Mail workspace.
I.006. Permit seed: mail.campaign.send.
I.006. Ontology projection: MailMessage, Segment.
I.006. Workflow template: consent-refresh.
I.006. UX shell: Campaign composer.
I.007. Tier loyalty-core.
I.007. Label: Loyalty workspace.
I.007. Permit seed: loyalty.reward.grant.
I.007. Ontology projection: Account, Reward, DealSet.
I.007. Workflow template: reward-adjustment.
I.007. UX shell: Rewards console.
I.008. Tier contact-center-core.
I.008. Label: Voice workspace.
I.008. Permit seed: contact_center.call.route.
I.008. Ontology projection: CallSession, Case, Contact.
I.008. Workflow template: queue-routing.
I.008. UX shell: Live queue console.
I.009. Tier hr-core.
I.009. Label: Employee workspace.
I.009. Permit seed: employee.profile.read.
I.009. Ontology projection: Employee, Position, OrgUnit.
I.009. Workflow template: employee-onboarding.
I.009. UX shell: People directory.
I.010. Tier recruiting-core.
I.010. Label: Candidate workspace.
I.010. Permit seed: candidate.stage.move.
I.010. Ontology projection: Candidate, InterviewLoop, Offer.
I.010. Workflow template: interview-loop.
I.010. UX shell: Pipeline board.
I.011. Tier performance-management-core.
I.011. Label: Review workspace.
I.011. Permit seed: review.submit.
I.011. Ontology projection: Goal, ReviewCycle, Feedback.
I.011. Workflow template: review-cycle-open.
I.011. UX shell: Review dashboard.
I.012. Tier learning-management-core.
I.012. Label: Learning workspace.
I.012. Permit seed: course.assign.
I.012. Ontology projection: Course, Completion, Credential.
I.012. Workflow template: course-assignment.
I.012. UX shell: Course catalog.
I.013. Tier workforce-planning-core.
I.013. Label: Workforce workspace.
I.013. Permit seed: workforce.plan.write.
I.013. Ontology projection: PositionPlan, HeadcountScenario.
I.013. Workflow template: headcount-plan-approval.
I.013. UX shell: Scenario planner.
I.014. Tier contract-lifecycle-core.
I.014. Label: Contract workspace.
I.014. Permit seed: contract.approve.
I.014. Ontology projection: Contract, Clause, Counterparty.
I.014. Workflow template: legal-review.
I.014. UX shell: Contract queue.
I.015. Tier signature-core.
I.015. Label: Signature workspace.
I.015. Permit seed: signature.envelope.send.
I.015. Ontology projection: SignatureEnvelope, Contract.
I.015. Workflow template: signature-routing.
I.015. UX shell: Signature tracker.
I.016. Tier financial-planning-core.
I.016. Label: FP&A workspace.
I.016. Permit seed: forecast.write.
I.016. Ontology projection: Budget, Forecast, Scenario.
I.016. Workflow template: budget-cycle-open.
I.016. UX shell: Planning workbook.
I.017. Tier controlling-core.
I.017. Label: Cost workspace.
I.017. Permit seed: cost_center.adjust.
I.017. Ontology projection: CostCenter, Variance.
I.017. Workflow template: variance-review.
I.017. UX shell: Cost dashboard.
I.018. Tier procurement-core.
I.018. Label: Procurement workspace.
I.018. Permit seed: purchase.request.approve.
I.018. Ontology projection: DealSet, Supplier, PurchaseRequest.
I.018. Workflow template: purchase-approval.
I.018. UX shell: Procurement queue.
I.019. Tier supplier-management-core.
I.019. Label: Supplier workspace.
I.019. Permit seed: supplier.performance.read.
I.019. Ontology projection: Supplier, Scorecard, DealSet.
I.019. Workflow template: supplier-review.
I.019. UX shell: Supplier dashboard.
I.020. Tier itsm-core.
I.020. Label: Service desk workspace.
I.020. Permit seed: incident.create.
I.020. Ontology projection: Incident, Asset, Service.
I.020. Workflow template: incident-triage.
I.020. UX shell: Service desk queue.
I.021. Tier change-management-core.
I.021. Label: Change workspace.
I.021. Permit seed: change.approve.
I.021. Ontology projection: ChangeRequest, Service, Risk.
I.021. Workflow template: change-advisory-board.
I.021. UX shell: Change calendar.
I.022. Tier incident-management-core.
I.022. Label: On-call workspace.
I.022. Permit seed: page.send.
I.022. Ontology projection: Incident, OnCallSchedule.
I.022. Workflow template: major-incident-declare.
I.022. UX shell: Incident commander view.
I.023. Tier content-classification-core.
I.023. Label: Records workspace.
I.023. Permit seed: file.classify.
I.023. Ontology projection: File, ClassificationLabel.
I.023. Workflow template: classification-review.
I.023. UX shell: Classification panel.
I.024. Tier ediscovery-core.
I.024. Label: Legal hold workspace.
I.024. Permit seed: hold.create.
I.024. Ontology projection: LegalMatter, RetentionHold, File.
I.024. Workflow template: ediscovery-export.
I.024. UX shell: Matter dashboard.
I.025. Tier analytics-workbook-core.
I.025. Label: BI workspace.
I.025. Permit seed: analytics.query.run.
I.025. Ontology projection: Workbook, Dataset, Metric.
I.025. Workflow template: dataset-refresh.
I.025. UX shell: BI canvas.
I.026. Tier data-warehouse-core.
I.026. Label: Warehouse analytics.
I.026. Permit seed: warehouse.query.run.
I.026. Ontology projection: Dataset, QueryJob, Lineage.
I.026. Workflow template: query-approval.
I.026. UX shell: Query console.
I.027. Tier global-trade-core.
I.027. Label: Trade workspace.
I.027. Permit seed: trade.screen.
I.027. Ontology projection: Shipment, Counterparty, SanctionsHit.
I.027. Workflow template: trade-hold-review.
I.027. UX shell: Trade hold queue.
I.028. Tier treasury-core.
I.028. Label: Treasury workspace.
I.028. Permit seed: cash.position.read.
I.028. Ontology projection: CashPosition, Hedge, BankAccount.
I.028. Workflow template: hedge-approval.
I.028. UX shell: Liquidity dashboard.
I.029. Tier warehouse-core.
I.029. Label: Warehouse workspace.
I.029. Permit seed: warehouse.task.dispatch.
I.029. Ontology projection: InventoryItem, PickTask, Shipment.
I.029. Workflow template: wave-release.
I.029. UX shell: Warehouse task board.
I.030. Tier production-planning-core.
I.030. Label: Production workspace.
I.030. Permit seed: mrp.run.
I.030. Ontology projection: Bom, WorkOrder, CapacityPlan.
I.030. Workflow template: mrp-approval.
I.030. UX shell: MRP planner.
I.031. Tier quality-management-core.
I.031. Label: Quality workspace.
I.031. Permit seed: quality.inspection.record.
I.031. Ontology projection: InspectionPlan, QualityNotification.
I.031. Workflow template: quality-hold.
I.031. UX shell: Quality console.
I.032. Tier plant-maintenance-core.
I.032. Label: Maintenance workspace.
I.032. Permit seed: maintenance.work_order.create.
I.032. Ontology projection: Equipment, WorkOrder, SparePart.
I.032. Workflow template: preventive-maintenance.
I.032. UX shell: Maintenance board.
I.033. Tier real-estate-core.
I.033. Label: Lease workspace.
I.033. Permit seed: lease.amend.
I.033. Ontology projection: Lease, Facility, PaymentSchedule.
I.033. Workflow template: lease-renewal.
I.033. UX shell: Lease dashboard.
I.034. Tier design-collaboration-core.
I.034. Label: Design workspace.
I.034. Permit seed: design.file.comment.
I.034. Ontology projection: DesignFile, Comment, Review.
I.034. Workflow template: design-review.
I.034. UX shell: Design review shell.
I.035. Tier whiteboard-core.
I.035. Label: Canvas workspace.
I.035. Permit seed: whiteboard.session.create.
I.035. Ontology projection: Canvas, Sticky, Vote.
I.035. Workflow template: workshop-facilitation.
I.035. UX shell: Canvas shell.

## Appendix J - Primitive precedent ledger

J.001. Primitive: Cedar permit set.
J.001. Precedent A: AWS Verified Permissions.
J.001. Precedent B: Salesforce role/profile/permission-set model.
J.001. Precedent C: ServiceNow roles and ACL model.
J.001. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.002. Primitive: Ontology projection.
J.002. Precedent A: Palantir Foundry Ontology.
J.002. Precedent B: Microsoft Graph resource relationships.
J.002. Precedent C: Salesforce metadata/object model.
J.002. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.003. Primitive: Workflow template library.
J.003. Precedent A: ServiceNow Flow Designer.
J.003. Precedent B: Microsoft Power Automate.
J.003. Precedent C: Salesforce Flow.
J.003. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.004. Primitive: UX shell manifest.
J.004. Precedent A: Microsoft Teams app manifest.
J.004. Precedent B: Atlassian Forge modules.
J.004. Precedent C: Salesforce Lightning App Builder.
J.004. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.005. Primitive: Compliance pack overlay.
J.005. Precedent A: Salesforce Hyperforce operating zones.
J.005. Precedent B: Microsoft 365 compliance program.
J.005. Precedent C: Atlassian Forge data residency.
J.005. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.006. Primitive: Tenant capability grant.
J.006. Precedent A: Microsoft Graph delegated/application permissions.
J.006. Precedent B: Notion connection capabilities.
J.006. Precedent C: Salesforce AppExchange package installation.
J.006. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.007. Primitive: Schema revision pin.
J.007. Precedent A: Stripe API versioning.
J.007. Precedent B: Palantir schema revision pattern.
J.007. Precedent C: Microsoft Graph versioned API surface.
J.007. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.008. Primitive: Audit profile.
J.008. Precedent A: ServiceNow audit/history and workflow telemetry.
J.008. Precedent B: Salesforce Shield audit pattern.
J.008. Precedent C: Microsoft Purview audit.
J.008. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.009. Primitive: Cost profile.
J.009. Precedent A: Salesforce Hyperforce unit cost management.
J.009. Precedent B: AWS Cost Categories.
J.009. Precedent C: Microsoft cost allocation tags.
J.009. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.010. Primitive: Lifecycle state.
J.010. Precedent A: Atlassian Forge environments and deployment promotion.
J.010. Precedent B: Salesforce package version lifecycle.
J.010. Precedent C: Microsoft app certification lifecycle.
J.010. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.011. Primitive: Microservice contribution.
J.011. Precedent A: Atlassian Forge modules.
J.011. Precedent B: Microsoft 365 app capabilities.
J.011. Precedent C: Salesforce metadata components.
J.011. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.
J.012. Primitive: Tier composition.
J.012. Precedent A: Microsoft 365 integrated platform.
J.012. Precedent B: ServiceNow Now Module workflows.
J.012. Precedent C: Salesforce Platform layered capabilities.
J.012. Doctrine: oyatie uses the primitive only with tenant scope, Cedar evidence, and audit-chain emission.

## Appendix K - Service contribution declaration ledger

K.001. Service api-gateway MUST declare capability-tier contributions or no-tier posture.
K.001. Service api-gateway MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.001. Service api-gateway MUST NOT create product-specific authorization outside Cedar permit sets.
K.001. Service api-gateway MUST NOT create product-specific object copies outside ontology projection rules.
K.002. Service audit-chain MUST declare capability-tier contributions or no-tier posture.
K.002. Service audit-chain MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.002. Service audit-chain MUST NOT create product-specific authorization outside Cedar permit sets.
K.002. Service audit-chain MUST NOT create product-specific object copies outside ontology projection rules.
K.003. Service analytics MUST declare capability-tier contributions or no-tier posture.
K.003. Service analytics MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.003. Service analytics MUST NOT create product-specific authorization outside Cedar permit sets.
K.003. Service analytics MUST NOT create product-specific object copies outside ontology projection rules.
K.004. Service calendar MUST declare capability-tier contributions or no-tier posture.
K.004. Service calendar MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.004. Service calendar MUST NOT create product-specific authorization outside Cedar permit sets.
K.004. Service calendar MUST NOT create product-specific object copies outside ontology projection rules.
K.005. Service cell MUST declare capability-tier contributions or no-tier posture.
K.005. Service cell MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.005. Service cell MUST NOT create product-specific authorization outside Cedar permit sets.
K.005. Service cell MUST NOT create product-specific object copies outside ontology projection rules.
K.006. Service cloud-iac MUST declare capability-tier contributions or no-tier posture.
K.006. Service cloud-iac MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.006. Service cloud-iac MUST NOT create product-specific authorization outside Cedar permit sets.
K.006. Service cloud-iac MUST NOT create product-specific object copies outside ontology projection rules.
K.007. Service community MUST declare capability-tier contributions or no-tier posture.
K.007. Service community MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.007. Service community MUST NOT create product-specific authorization outside Cedar permit sets.
K.007. Service community MUST NOT create product-specific object copies outside ontology projection rules.
K.008. Service compliance MUST declare capability-tier contributions or no-tier posture.
K.008. Service compliance MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.008. Service compliance MUST NOT create product-specific authorization outside Cedar permit sets.
K.008. Service compliance MUST NOT create product-specific object copies outside ontology projection rules.
K.009. Service connector MUST declare capability-tier contributions or no-tier posture.
K.009. Service connector MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.009. Service connector MUST NOT create product-specific authorization outside Cedar permit sets.
K.009. Service connector MUST NOT create product-specific object copies outside ontology projection rules.
K.010. Service consent-graph MUST declare capability-tier contributions or no-tier posture.
K.010. Service consent-graph MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.010. Service consent-graph MUST NOT create product-specific authorization outside Cedar permit sets.
K.010. Service consent-graph MUST NOT create product-specific object copies outside ontology projection rules.
K.011. Service developer-sdk MUST declare capability-tier contributions or no-tier posture.
K.011. Service developer-sdk MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.011. Service developer-sdk MUST NOT create product-specific authorization outside Cedar permit sets.
K.011. Service developer-sdk MUST NOT create product-specific object copies outside ontology projection rules.
K.012. Service docs MUST declare capability-tier contributions or no-tier posture.
K.012. Service docs MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.012. Service docs MUST NOT create product-specific authorization outside Cedar permit sets.
K.012. Service docs MUST NOT create product-specific object copies outside ontology projection rules.
K.013. Service drive MUST declare capability-tier contributions or no-tier posture.
K.013. Service drive MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.013. Service drive MUST NOT create product-specific authorization outside Cedar permit sets.
K.013. Service drive MUST NOT create product-specific object copies outside ontology projection rules.
K.014. Service finops-portal MUST declare capability-tier contributions or no-tier posture.
K.014. Service finops-portal MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.014. Service finops-portal MUST NOT create product-specific authorization outside Cedar permit sets.
K.014. Service finops-portal MUST NOT create product-specific object copies outside ontology projection rules.
K.015. Service forms MUST declare capability-tier contributions or no-tier posture.
K.015. Service forms MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.015. Service forms MUST NOT create product-specific authorization outside Cedar permit sets.
K.015. Service forms MUST NOT create product-specific object copies outside ontology projection rules.
K.016. Service foundry MUST declare capability-tier contributions or no-tier posture.
K.016. Service foundry MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.016. Service foundry MUST NOT create product-specific authorization outside Cedar permit sets.
K.016. Service foundry MUST NOT create product-specific object copies outside ontology projection rules.
K.017. Service governance MUST declare capability-tier contributions or no-tier posture.
K.017. Service governance MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.017. Service governance MUST NOT create product-specific authorization outside Cedar permit sets.
K.017. Service governance MUST NOT create product-specific object copies outside ontology projection rules.
K.018. Service identity MUST declare capability-tier contributions or no-tier posture.
K.018. Service identity MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.018. Service identity MUST NOT create product-specific authorization outside Cedar permit sets.
K.018. Service identity MUST NOT create product-specific object copies outside ontology projection rules.
K.019. Service intelligence MUST declare capability-tier contributions or no-tier posture.
K.019. Service intelligence MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.019. Service intelligence MUST NOT create product-specific authorization outside Cedar permit sets.
K.019. Service intelligence MUST NOT create product-specific object copies outside ontology projection rules.
K.020. Service mail MUST declare capability-tier contributions or no-tier posture.
K.020. Service mail MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.020. Service mail MUST NOT create product-specific authorization outside Cedar permit sets.
K.020. Service mail MUST NOT create product-specific object copies outside ontology projection rules.
K.021. Service marketplace MUST declare capability-tier contributions or no-tier posture.
K.021. Service marketplace MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.021. Service marketplace MUST NOT create product-specific authorization outside Cedar permit sets.
K.021. Service marketplace MUST NOT create product-specific object copies outside ontology projection rules.
K.022. Service meet MUST declare capability-tier contributions or no-tier posture.
K.022. Service meet MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.022. Service meet MUST NOT create product-specific authorization outside Cedar permit sets.
K.022. Service meet MUST NOT create product-specific object copies outside ontology projection rules.
K.023. Service messenger MUST declare capability-tier contributions or no-tier posture.
K.023. Service messenger MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.023. Service messenger MUST NOT create product-specific authorization outside Cedar permit sets.
K.023. Service messenger MUST NOT create product-specific object copies outside ontology projection rules.
K.024. Service notes MUST declare capability-tier contributions or no-tier posture.
K.024. Service notes MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.024. Service notes MUST NOT create product-specific authorization outside Cedar permit sets.
K.024. Service notes MUST NOT create product-specific object copies outside ontology projection rules.
K.025. Service observability MUST declare capability-tier contributions or no-tier posture.
K.025. Service observability MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.025. Service observability MUST NOT create product-specific authorization outside Cedar permit sets.
K.025. Service observability MUST NOT create product-specific object copies outside ontology projection rules.
K.026. Service payments MUST declare capability-tier contributions or no-tier posture.
K.026. Service payments MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.026. Service payments MUST NOT create product-specific authorization outside Cedar permit sets.
K.026. Service payments MUST NOT create product-specific object copies outside ontology projection rules.
K.027. Service plugin-app-store MUST declare capability-tier contributions or no-tier posture.
K.027. Service plugin-app-store MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.027. Service plugin-app-store MUST NOT create product-specific authorization outside Cedar permit sets.
K.027. Service plugin-app-store MUST NOT create product-specific object copies outside ontology projection rules.
K.028. Service policy-engine MUST declare capability-tier contributions or no-tier posture.
K.028. Service policy-engine MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.028. Service policy-engine MUST NOT create product-specific authorization outside Cedar permit sets.
K.028. Service policy-engine MUST NOT create product-specific object copies outside ontology projection rules.
K.029. Service regional-pack MUST declare capability-tier contributions or no-tier posture.
K.029. Service regional-pack MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.029. Service regional-pack MUST NOT create product-specific authorization outside Cedar permit sets.
K.029. Service regional-pack MUST NOT create product-specific object copies outside ontology projection rules.
K.030. Service sheets MUST declare capability-tier contributions or no-tier posture.
K.030. Service sheets MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.030. Service sheets MUST NOT create product-specific authorization outside Cedar permit sets.
K.030. Service sheets MUST NOT create product-specific object copies outside ontology projection rules.
K.031. Service sites MUST declare capability-tier contributions or no-tier posture.
K.031. Service sites MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.031. Service sites MUST NOT create product-specific authorization outside Cedar permit sets.
K.031. Service sites MUST NOT create product-specific object copies outside ontology projection rules.
K.032. Service slides MUST declare capability-tier contributions or no-tier posture.
K.032. Service slides MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.032. Service slides MUST NOT create product-specific authorization outside Cedar permit sets.
K.032. Service slides MUST NOT create product-specific object copies outside ontology projection rules.
K.033. Service social MUST declare capability-tier contributions or no-tier posture.
K.033. Service social MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.033. Service social MUST NOT create product-specific authorization outside Cedar permit sets.
K.033. Service social MUST NOT create product-specific object copies outside ontology projection rules.
K.034. Service tasks MUST declare capability-tier contributions or no-tier posture.
K.034. Service tasks MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.034. Service tasks MUST NOT create product-specific authorization outside Cedar permit sets.
K.034. Service tasks MUST NOT create product-specific object copies outside ontology projection rules.
K.035. Service tenancy MUST declare capability-tier contributions or no-tier posture.
K.035. Service tenancy MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.035. Service tenancy MUST NOT create product-specific authorization outside Cedar permit sets.
K.035. Service tenancy MUST NOT create product-specific object copies outside ontology projection rules.
K.036. Service translate MUST declare capability-tier contributions or no-tier posture.
K.036. Service translate MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.036. Service translate MUST NOT create product-specific authorization outside Cedar permit sets.
K.036. Service translate MUST NOT create product-specific object copies outside ontology projection rules.
K.037. Service workflow-engine MUST declare capability-tier contributions or no-tier posture.
K.037. Service workflow-engine MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.037. Service workflow-engine MUST NOT create product-specific authorization outside Cedar permit sets.
K.037. Service workflow-engine MUST NOT create product-specific object copies outside ontology projection rules.
K.038. Service workflow-studio MUST declare capability-tier contributions or no-tier posture.
K.038. Service workflow-studio MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.038. Service workflow-studio MUST NOT create product-specific authorization outside Cedar permit sets.
K.038. Service workflow-studio MUST NOT create product-specific object copies outside ontology projection rules.
K.039. Service workplace-integration MUST declare capability-tier contributions or no-tier posture.
K.039. Service workplace-integration MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.039. Service workplace-integration MUST NOT create product-specific authorization outside Cedar permit sets.
K.039. Service workplace-integration MUST NOT create product-specific object copies outside ontology projection rules.
K.040. Service crm MUST declare capability-tier contributions or no-tier posture.
K.040. Service crm MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.040. Service crm MUST NOT create product-specific authorization outside Cedar permit sets.
K.040. Service crm MUST NOT create product-specific object copies outside ontology projection rules.
K.041. Service marketing-automation MUST declare capability-tier contributions or no-tier posture.
K.041. Service marketing-automation MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.041. Service marketing-automation MUST NOT create product-specific authorization outside Cedar permit sets.
K.041. Service marketing-automation MUST NOT create product-specific object copies outside ontology projection rules.
K.042. Service contact-center MUST declare capability-tier contributions or no-tier posture.
K.042. Service contact-center MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.042. Service contact-center MUST NOT create product-specific authorization outside Cedar permit sets.
K.042. Service contact-center MUST NOT create product-specific object copies outside ontology projection rules.
K.043. Service performance-management MUST declare capability-tier contributions or no-tier posture.
K.043. Service performance-management MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.043. Service performance-management MUST NOT create product-specific authorization outside Cedar permit sets.
K.043. Service performance-management MUST NOT create product-specific object copies outside ontology projection rules.
K.044. Service learning-management MUST declare capability-tier contributions or no-tier posture.
K.044. Service learning-management MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.044. Service learning-management MUST NOT create product-specific authorization outside Cedar permit sets.
K.044. Service learning-management MUST NOT create product-specific object copies outside ontology projection rules.
K.045. Service contract-lifecycle-management MUST declare capability-tier contributions or no-tier posture.
K.045. Service contract-lifecycle-management MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.045. Service contract-lifecycle-management MUST NOT create product-specific authorization outside Cedar permit sets.
K.045. Service contract-lifecycle-management MUST NOT create product-specific object copies outside ontology projection rules.
K.046. Service financial-planning MUST declare capability-tier contributions or no-tier posture.
K.046. Service financial-planning MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.046. Service financial-planning MUST NOT create product-specific authorization outside Cedar permit sets.
K.046. Service financial-planning MUST NOT create product-specific object copies outside ontology projection rules.
K.047. Service data-warehouse MUST declare capability-tier contributions or no-tier posture.
K.047. Service data-warehouse MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.047. Service data-warehouse MUST NOT create product-specific authorization outside Cedar permit sets.
K.047. Service data-warehouse MUST NOT create product-specific object copies outside ontology projection rules.
K.048. Service itsm MUST declare capability-tier contributions or no-tier posture.
K.048. Service itsm MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.048. Service itsm MUST NOT create product-specific authorization outside Cedar permit sets.
K.048. Service itsm MUST NOT create product-specific object copies outside ontology projection rules.
K.049. Service incident-management MUST declare capability-tier contributions or no-tier posture.
K.049. Service incident-management MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.049. Service incident-management MUST NOT create product-specific authorization outside Cedar permit sets.
K.049. Service incident-management MUST NOT create product-specific object copies outside ontology projection rules.
K.050. Service production-planning MUST declare capability-tier contributions or no-tier posture.
K.050. Service production-planning MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.050. Service production-planning MUST NOT create product-specific authorization outside Cedar permit sets.
K.050. Service production-planning MUST NOT create product-specific object copies outside ontology projection rules.
K.051. Service quality-management MUST declare capability-tier contributions or no-tier posture.
K.051. Service quality-management MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.051. Service quality-management MUST NOT create product-specific authorization outside Cedar permit sets.
K.051. Service quality-management MUST NOT create product-specific object copies outside ontology projection rules.
K.052. Service plant-maintenance MUST declare capability-tier contributions or no-tier posture.
K.052. Service plant-maintenance MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.052. Service plant-maintenance MUST NOT create product-specific authorization outside Cedar permit sets.
K.052. Service plant-maintenance MUST NOT create product-specific object copies outside ontology projection rules.
K.053. Service warehouse MUST declare capability-tier contributions or no-tier posture.
K.053. Service warehouse MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.053. Service warehouse MUST NOT create product-specific authorization outside Cedar permit sets.
K.053. Service warehouse MUST NOT create product-specific object copies outside ontology projection rules.
K.054. Service treasury MUST declare capability-tier contributions or no-tier posture.
K.054. Service treasury MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.054. Service treasury MUST NOT create product-specific authorization outside Cedar permit sets.
K.054. Service treasury MUST NOT create product-specific object copies outside ontology projection rules.
K.055. Service global-trade MUST declare capability-tier contributions or no-tier posture.
K.055. Service global-trade MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.055. Service global-trade MUST NOT create product-specific authorization outside Cedar permit sets.
K.055. Service global-trade MUST NOT create product-specific object copies outside ontology projection rules.
K.056. Service real-estate MUST declare capability-tier contributions or no-tier posture.
K.056. Service real-estate MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.056. Service real-estate MUST NOT create product-specific authorization outside Cedar permit sets.
K.056. Service real-estate MUST NOT create product-specific object copies outside ontology projection rules.
K.057. Service whiteboard MUST declare capability-tier contributions or no-tier posture.
K.057. Service whiteboard MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.057. Service whiteboard MUST NOT create product-specific authorization outside Cedar permit sets.
K.057. Service whiteboard MUST NOT create product-specific object copies outside ontology projection rules.
K.058. Service design-collaboration MUST declare capability-tier contributions or no-tier posture.
K.058. Service design-collaboration MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.058. Service design-collaboration MUST NOT create product-specific authorization outside Cedar permit sets.
K.058. Service design-collaboration MUST NOT create product-specific object copies outside ontology projection rules.
K.059. Service supply-chain-planning MUST declare capability-tier contributions or no-tier posture.
K.059. Service supply-chain-planning MUST bind each contribution to owner team, artifact ref, schema revision, and evidence ref.
K.059. Service supply-chain-planning MUST NOT create product-specific authorization outside Cedar permit sets.
K.059. Service supply-chain-planning MUST NOT create product-specific object copies outside ontology projection rules.

## Appendix L - Product-fragmentation review checklist

L.001. Review question: Does the candidate service name match a department, suite, module, or vendor category rather than an operational concern?
L.001. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.002. Review question: Can existing ontology object types represent the category with additional projection rules?
L.002. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.003. Review question: Can existing workflow templates represent the lifecycle with a new template library?
L.003. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.004. Review question: Can existing Cedar action namespaces represent the permissions with a new permit set?
L.004. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.005. Review question: Can the UX be expressed as a new shell rather than a runtime service?
L.005. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.006. Review question: Can compliance constraints be expressed as pack overlays?
L.006. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.007. Review question: Can telemetry distinguish the tier by dimensions rather than a new telemetry stack?
L.007. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.008. Review question: Can FinOps allocate cost by tier profile rather than by new infrastructure?
L.008. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.009. Review question: Does the category need a new write authority?
L.009. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.010. Review question: Does the category need a new runtime class?
L.010. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.011. Review question: Does the category need a new regulated license or certification boundary?
L.011. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.012. Review question: Does the category need a new on-call specialty with distinct failure modes?
L.012. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.013. Review question: Would adding a service duplicate tenant identity?
L.013. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.014. Review question: Would adding a service duplicate role and permission management?
L.014. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.015. Review question: Would adding a service duplicate customer, employee, supplier, contract, file, or ticket data?
L.015. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.016. Review question: Would adding a service duplicate workflow state?
L.016. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.017. Review question: Would adding a service duplicate audit export?
L.017. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.018. Review question: Would adding a service force tenants through another migration during sunset?
L.018. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.019. Review question: Does the category map to ADR-0315 ERP module composition?
L.019. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.020. Review question: Does the category interact with ADR-0314 DealSet settlement?
L.020. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.021. Review question: Does the category cross a sovereign child tenant boundary from ADR-0313?
L.021. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.022. Review question: Does the category pin ontology schema revisions per ADR-0257?
L.022. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.023. Review question: Does the category preserve ADR-0249 marketplace substrate ownership?
L.023. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.024. Review question: Does the category preserve ADR-0245 tier classification?
L.024. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.
L.025. Review question: Does the category preserve ADR-0132 no-grouping doctrine?
L.025. Required answer: capability-tier-first unless Section D-10 evidence proves a new operational concern.

## Appendix M - Validation evidence contract

M.001. sales-cloud-core validation kind cedar-fixture: crm.account.read permit and forbid fixtures pass.
M.002. sales-cloud-core validation kind ontology-pin: Account, Contact, Opportunity projection pins schema revisions.
M.003. sales-cloud-core validation kind workflow-replay: lead-to-opportunity replay fixture passes.
M.004. sales-cloud-core validation kind ux-shell: Sales nav with account dashboard manifest passes accessibility and localization checks.
M.005. sales-cloud-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.006. sales-forecasting-core validation kind cedar-fixture: forecast.submit permit and forbid fixtures pass.
M.007. sales-forecasting-core validation kind ontology-pin: ForecastSnapshot, Opportunity projection pins schema revisions.
M.008. sales-forecasting-core validation kind workflow-replay: forecast-submit replay fixture passes.
M.009. sales-forecasting-core validation kind ux-shell: Forecast grid and variance chart manifest passes accessibility and localization checks.
M.010. sales-forecasting-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.011. service-cloud-core validation kind cedar-fixture: crm.case.write permit and forbid fixtures pass.
M.012. service-cloud-core validation kind ontology-pin: Case, Entitlement, Contact projection pins schema revisions.
M.013. service-cloud-core validation kind workflow-replay: case-triage replay fixture passes.
M.014. service-cloud-core validation kind ux-shell: Case queue and SLA banner manifest passes accessibility and localization checks.
M.015. service-cloud-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.016. customer-success-core validation kind cedar-fixture: success.health.read permit and forbid fixtures pass.
M.017. customer-success-core validation kind ontology-pin: AccountHealth, RenewalRisk projection pins schema revisions.
M.018. customer-success-core validation kind workflow-replay: renewal-risk-escalation replay fixture passes.
M.019. customer-success-core validation kind ux-shell: Health dashboard manifest passes accessibility and localization checks.
M.020. customer-success-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.021. marketing-automation-core validation kind cedar-fixture: campaign.launch permit and forbid fixtures pass.
M.022. marketing-automation-core validation kind ontology-pin: Campaign, Segment, ConsentPreference projection pins schema revisions.
M.023. marketing-automation-core validation kind workflow-replay: campaign-launch-approval replay fixture passes.
M.024. marketing-automation-core validation kind ux-shell: Journey canvas manifest passes accessibility and localization checks.
M.025. marketing-automation-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.026. email-marketing-core validation kind cedar-fixture: mail.campaign.send permit and forbid fixtures pass.
M.027. email-marketing-core validation kind ontology-pin: MailMessage, Segment projection pins schema revisions.
M.028. email-marketing-core validation kind workflow-replay: consent-refresh replay fixture passes.
M.029. email-marketing-core validation kind ux-shell: Campaign composer manifest passes accessibility and localization checks.
M.030. email-marketing-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.031. loyalty-core validation kind cedar-fixture: loyalty.reward.grant permit and forbid fixtures pass.
M.032. loyalty-core validation kind ontology-pin: Account, Reward, DealSet projection pins schema revisions.
M.033. loyalty-core validation kind workflow-replay: reward-adjustment replay fixture passes.
M.034. loyalty-core validation kind ux-shell: Rewards console manifest passes accessibility and localization checks.
M.035. loyalty-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.036. contact-center-core validation kind cedar-fixture: contact_center.call.route permit and forbid fixtures pass.
M.037. contact-center-core validation kind ontology-pin: CallSession, Case, Contact projection pins schema revisions.
M.038. contact-center-core validation kind workflow-replay: queue-routing replay fixture passes.
M.039. contact-center-core validation kind ux-shell: Live queue console manifest passes accessibility and localization checks.
M.040. contact-center-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.041. hr-core validation kind cedar-fixture: employee.profile.read permit and forbid fixtures pass.
M.042. hr-core validation kind ontology-pin: Employee, Position, OrgUnit projection pins schema revisions.
M.043. hr-core validation kind workflow-replay: employee-onboarding replay fixture passes.
M.044. hr-core validation kind ux-shell: People directory manifest passes accessibility and localization checks.
M.045. hr-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.046. recruiting-core validation kind cedar-fixture: candidate.stage.move permit and forbid fixtures pass.
M.047. recruiting-core validation kind ontology-pin: Candidate, InterviewLoop, Offer projection pins schema revisions.
M.048. recruiting-core validation kind workflow-replay: interview-loop replay fixture passes.
M.049. recruiting-core validation kind ux-shell: Pipeline board manifest passes accessibility and localization checks.
M.050. recruiting-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.051. performance-management-core validation kind cedar-fixture: review.submit permit and forbid fixtures pass.
M.052. performance-management-core validation kind ontology-pin: Goal, ReviewCycle, Feedback projection pins schema revisions.
M.053. performance-management-core validation kind workflow-replay: review-cycle-open replay fixture passes.
M.054. performance-management-core validation kind ux-shell: Review dashboard manifest passes accessibility and localization checks.
M.055. performance-management-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.056. learning-management-core validation kind cedar-fixture: course.assign permit and forbid fixtures pass.
M.057. learning-management-core validation kind ontology-pin: Course, Completion, Credential projection pins schema revisions.
M.058. learning-management-core validation kind workflow-replay: course-assignment replay fixture passes.
M.059. learning-management-core validation kind ux-shell: Course catalog manifest passes accessibility and localization checks.
M.060. learning-management-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.061. workforce-planning-core validation kind cedar-fixture: workforce.plan.write permit and forbid fixtures pass.
M.062. workforce-planning-core validation kind ontology-pin: PositionPlan, HeadcountScenario projection pins schema revisions.
M.063. workforce-planning-core validation kind workflow-replay: headcount-plan-approval replay fixture passes.
M.064. workforce-planning-core validation kind ux-shell: Scenario planner manifest passes accessibility and localization checks.
M.065. workforce-planning-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.066. contract-lifecycle-core validation kind cedar-fixture: contract.approve permit and forbid fixtures pass.
M.067. contract-lifecycle-core validation kind ontology-pin: Contract, Clause, Counterparty projection pins schema revisions.
M.068. contract-lifecycle-core validation kind workflow-replay: legal-review replay fixture passes.
M.069. contract-lifecycle-core validation kind ux-shell: Contract queue manifest passes accessibility and localization checks.
M.070. contract-lifecycle-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.071. signature-core validation kind cedar-fixture: signature.envelope.send permit and forbid fixtures pass.
M.072. signature-core validation kind ontology-pin: SignatureEnvelope, Contract projection pins schema revisions.
M.073. signature-core validation kind workflow-replay: signature-routing replay fixture passes.
M.074. signature-core validation kind ux-shell: Signature tracker manifest passes accessibility and localization checks.
M.075. signature-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.076. financial-planning-core validation kind cedar-fixture: forecast.write permit and forbid fixtures pass.
M.077. financial-planning-core validation kind ontology-pin: Budget, Forecast, Scenario projection pins schema revisions.
M.078. financial-planning-core validation kind workflow-replay: budget-cycle-open replay fixture passes.
M.079. financial-planning-core validation kind ux-shell: Planning workbook manifest passes accessibility and localization checks.
M.080. financial-planning-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.081. controlling-core validation kind cedar-fixture: cost_center.adjust permit and forbid fixtures pass.
M.082. controlling-core validation kind ontology-pin: CostCenter, Variance projection pins schema revisions.
M.083. controlling-core validation kind workflow-replay: variance-review replay fixture passes.
M.084. controlling-core validation kind ux-shell: Cost dashboard manifest passes accessibility and localization checks.
M.085. controlling-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.086. procurement-core validation kind cedar-fixture: purchase.request.approve permit and forbid fixtures pass.
M.087. procurement-core validation kind ontology-pin: DealSet, Supplier, PurchaseRequest projection pins schema revisions.
M.088. procurement-core validation kind workflow-replay: purchase-approval replay fixture passes.
M.089. procurement-core validation kind ux-shell: Procurement queue manifest passes accessibility and localization checks.
M.090. procurement-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.091. supplier-management-core validation kind cedar-fixture: supplier.performance.read permit and forbid fixtures pass.
M.092. supplier-management-core validation kind ontology-pin: Supplier, Scorecard, DealSet projection pins schema revisions.
M.093. supplier-management-core validation kind workflow-replay: supplier-review replay fixture passes.
M.094. supplier-management-core validation kind ux-shell: Supplier dashboard manifest passes accessibility and localization checks.
M.095. supplier-management-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.096. itsm-core validation kind cedar-fixture: incident.create permit and forbid fixtures pass.
M.097. itsm-core validation kind ontology-pin: Incident, Asset, Service projection pins schema revisions.
M.098. itsm-core validation kind workflow-replay: incident-triage replay fixture passes.
M.099. itsm-core validation kind ux-shell: Service desk queue manifest passes accessibility and localization checks.
M.100. itsm-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.101. change-management-core validation kind cedar-fixture: change.approve permit and forbid fixtures pass.
M.102. change-management-core validation kind ontology-pin: ChangeRequest, Service, Risk projection pins schema revisions.
M.103. change-management-core validation kind workflow-replay: change-advisory-board replay fixture passes.
M.104. change-management-core validation kind ux-shell: Change calendar manifest passes accessibility and localization checks.
M.105. change-management-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.106. incident-management-core validation kind cedar-fixture: page.send permit and forbid fixtures pass.
M.107. incident-management-core validation kind ontology-pin: Incident, OnCallSchedule projection pins schema revisions.
M.108. incident-management-core validation kind workflow-replay: major-incident-declare replay fixture passes.
M.109. incident-management-core validation kind ux-shell: Incident commander view manifest passes accessibility and localization checks.
M.110. incident-management-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.111. content-classification-core validation kind cedar-fixture: file.classify permit and forbid fixtures pass.
M.112. content-classification-core validation kind ontology-pin: File, ClassificationLabel projection pins schema revisions.
M.113. content-classification-core validation kind workflow-replay: classification-review replay fixture passes.
M.114. content-classification-core validation kind ux-shell: Classification panel manifest passes accessibility and localization checks.
M.115. content-classification-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.116. ediscovery-core validation kind cedar-fixture: hold.create permit and forbid fixtures pass.
M.117. ediscovery-core validation kind ontology-pin: LegalMatter, RetentionHold, File projection pins schema revisions.
M.118. ediscovery-core validation kind workflow-replay: ediscovery-export replay fixture passes.
M.119. ediscovery-core validation kind ux-shell: Matter dashboard manifest passes accessibility and localization checks.
M.120. ediscovery-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.121. analytics-workbook-core validation kind cedar-fixture: analytics.query.run permit and forbid fixtures pass.
M.122. analytics-workbook-core validation kind ontology-pin: Workbook, Dataset, Metric projection pins schema revisions.
M.123. analytics-workbook-core validation kind workflow-replay: dataset-refresh replay fixture passes.
M.124. analytics-workbook-core validation kind ux-shell: BI canvas manifest passes accessibility and localization checks.
M.125. analytics-workbook-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.126. data-warehouse-core validation kind cedar-fixture: warehouse.query.run permit and forbid fixtures pass.
M.127. data-warehouse-core validation kind ontology-pin: Dataset, QueryJob, Lineage projection pins schema revisions.
M.128. data-warehouse-core validation kind workflow-replay: query-approval replay fixture passes.
M.129. data-warehouse-core validation kind ux-shell: Query console manifest passes accessibility and localization checks.
M.130. data-warehouse-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.131. global-trade-core validation kind cedar-fixture: trade.screen permit and forbid fixtures pass.
M.132. global-trade-core validation kind ontology-pin: Shipment, Counterparty, SanctionsHit projection pins schema revisions.
M.133. global-trade-core validation kind workflow-replay: trade-hold-review replay fixture passes.
M.134. global-trade-core validation kind ux-shell: Trade hold queue manifest passes accessibility and localization checks.
M.135. global-trade-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.136. treasury-core validation kind cedar-fixture: cash.position.read permit and forbid fixtures pass.
M.137. treasury-core validation kind ontology-pin: CashPosition, Hedge, BankAccount projection pins schema revisions.
M.138. treasury-core validation kind workflow-replay: hedge-approval replay fixture passes.
M.139. treasury-core validation kind ux-shell: Liquidity dashboard manifest passes accessibility and localization checks.
M.140. treasury-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.141. warehouse-core validation kind cedar-fixture: warehouse.task.dispatch permit and forbid fixtures pass.
M.142. warehouse-core validation kind ontology-pin: InventoryItem, PickTask, Shipment projection pins schema revisions.
M.143. warehouse-core validation kind workflow-replay: wave-release replay fixture passes.
M.144. warehouse-core validation kind ux-shell: Warehouse task board manifest passes accessibility and localization checks.
M.145. warehouse-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.146. production-planning-core validation kind cedar-fixture: mrp.run permit and forbid fixtures pass.
M.147. production-planning-core validation kind ontology-pin: Bom, WorkOrder, CapacityPlan projection pins schema revisions.
M.148. production-planning-core validation kind workflow-replay: mrp-approval replay fixture passes.
M.149. production-planning-core validation kind ux-shell: MRP planner manifest passes accessibility and localization checks.
M.150. production-planning-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.151. quality-management-core validation kind cedar-fixture: quality.inspection.record permit and forbid fixtures pass.
M.152. quality-management-core validation kind ontology-pin: InspectionPlan, QualityNotification projection pins schema revisions.
M.153. quality-management-core validation kind workflow-replay: quality-hold replay fixture passes.
M.154. quality-management-core validation kind ux-shell: Quality console manifest passes accessibility and localization checks.
M.155. quality-management-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.156. plant-maintenance-core validation kind cedar-fixture: maintenance.work_order.create permit and forbid fixtures pass.
M.157. plant-maintenance-core validation kind ontology-pin: Equipment, WorkOrder, SparePart projection pins schema revisions.
M.158. plant-maintenance-core validation kind workflow-replay: preventive-maintenance replay fixture passes.
M.159. plant-maintenance-core validation kind ux-shell: Maintenance board manifest passes accessibility and localization checks.
M.160. plant-maintenance-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.161. real-estate-core validation kind cedar-fixture: lease.amend permit and forbid fixtures pass.
M.162. real-estate-core validation kind ontology-pin: Lease, Facility, PaymentSchedule projection pins schema revisions.
M.163. real-estate-core validation kind workflow-replay: lease-renewal replay fixture passes.
M.164. real-estate-core validation kind ux-shell: Lease dashboard manifest passes accessibility and localization checks.
M.165. real-estate-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.166. design-collaboration-core validation kind cedar-fixture: design.file.comment permit and forbid fixtures pass.
M.167. design-collaboration-core validation kind ontology-pin: DesignFile, Comment, Review projection pins schema revisions.
M.168. design-collaboration-core validation kind workflow-replay: design-review replay fixture passes.
M.169. design-collaboration-core validation kind ux-shell: Design review shell manifest passes accessibility and localization checks.
M.170. design-collaboration-core validation kind audit-cost: audit profile and cost profile dimensions are present.
M.171. whiteboard-core validation kind cedar-fixture: whiteboard.session.create permit and forbid fixtures pass.
M.172. whiteboard-core validation kind ontology-pin: Canvas, Sticky, Vote projection pins schema revisions.
M.173. whiteboard-core validation kind workflow-replay: workshop-facilitation replay fixture passes.
M.174. whiteboard-core validation kind ux-shell: Canvas shell manifest passes accessibility and localization checks.
M.175. whiteboard-core validation kind audit-cost: audit profile and cost profile dimensions are present.

## Appendix N - Capability-tier catalog seed

N.001. Seed category account-management defaults to capability-tier review before service creation.
N.001. Seed category account-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.001. Seed category account-management may request new-service status only by passing every Section D-10 test with evidence.
N.002. Seed category opportunity-management defaults to capability-tier review before service creation.
N.002. Seed category opportunity-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.002. Seed category opportunity-management may request new-service status only by passing every Section D-10 test with evidence.
N.003. Seed category quote-management defaults to capability-tier review before service creation.
N.003. Seed category quote-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.003. Seed category quote-management may request new-service status only by passing every Section D-10 test with evidence.
N.004. Seed category renewal-management defaults to capability-tier review before service creation.
N.004. Seed category renewal-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.004. Seed category renewal-management may request new-service status only by passing every Section D-10 test with evidence.
N.005. Seed category case-management defaults to capability-tier review before service creation.
N.005. Seed category case-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.005. Seed category case-management may request new-service status only by passing every Section D-10 test with evidence.
N.006. Seed category knowledge-management defaults to capability-tier review before service creation.
N.006. Seed category knowledge-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.006. Seed category knowledge-management may request new-service status only by passing every Section D-10 test with evidence.
N.007. Seed category field-service defaults to capability-tier review before service creation.
N.007. Seed category field-service requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.007. Seed category field-service may request new-service status only by passing every Section D-10 test with evidence.
N.008. Seed category customer-community defaults to capability-tier review before service creation.
N.008. Seed category customer-community requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.008. Seed category customer-community may request new-service status only by passing every Section D-10 test with evidence.
N.009. Seed category campaign-management defaults to capability-tier review before service creation.
N.009. Seed category campaign-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.009. Seed category campaign-management may request new-service status only by passing every Section D-10 test with evidence.
N.010. Seed category journey-orchestration defaults to capability-tier review before service creation.
N.010. Seed category journey-orchestration requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.010. Seed category journey-orchestration may request new-service status only by passing every Section D-10 test with evidence.
N.011. Seed category consent-management defaults to capability-tier review before service creation.
N.011. Seed category consent-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.011. Seed category consent-management may request new-service status only by passing every Section D-10 test with evidence.
N.012. Seed category attribution defaults to capability-tier review before service creation.
N.012. Seed category attribution requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.012. Seed category attribution may request new-service status only by passing every Section D-10 test with evidence.
N.013. Seed category employee-profile defaults to capability-tier review before service creation.
N.013. Seed category employee-profile requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.013. Seed category employee-profile may request new-service status only by passing every Section D-10 test with evidence.
N.014. Seed category position-management defaults to capability-tier review before service creation.
N.014. Seed category position-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.014. Seed category position-management may request new-service status only by passing every Section D-10 test with evidence.
N.015. Seed category onboarding defaults to capability-tier review before service creation.
N.015. Seed category onboarding requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.015. Seed category onboarding may request new-service status only by passing every Section D-10 test with evidence.
N.016. Seed category offboarding defaults to capability-tier review before service creation.
N.016. Seed category offboarding requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.016. Seed category offboarding may request new-service status only by passing every Section D-10 test with evidence.
N.017. Seed category goals defaults to capability-tier review before service creation.
N.017. Seed category goals requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.017. Seed category goals may request new-service status only by passing every Section D-10 test with evidence.
N.018. Seed category reviews defaults to capability-tier review before service creation.
N.018. Seed category reviews requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.018. Seed category reviews may request new-service status only by passing every Section D-10 test with evidence.
N.019. Seed category calibration defaults to capability-tier review before service creation.
N.019. Seed category calibration requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.019. Seed category calibration may request new-service status only by passing every Section D-10 test with evidence.
N.020. Seed category skills defaults to capability-tier review before service creation.
N.020. Seed category skills requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.020. Seed category skills may request new-service status only by passing every Section D-10 test with evidence.
N.021. Seed category course-catalog defaults to capability-tier review before service creation.
N.021. Seed category course-catalog requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.021. Seed category course-catalog may request new-service status only by passing every Section D-10 test with evidence.
N.022. Seed category certification defaults to capability-tier review before service creation.
N.022. Seed category certification requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.022. Seed category certification may request new-service status only by passing every Section D-10 test with evidence.
N.023. Seed category mandatory-training defaults to capability-tier review before service creation.
N.023. Seed category mandatory-training requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.023. Seed category mandatory-training may request new-service status only by passing every Section D-10 test with evidence.
N.024. Seed category credentialing defaults to capability-tier review before service creation.
N.024. Seed category credentialing requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.024. Seed category credentialing may request new-service status only by passing every Section D-10 test with evidence.
N.025. Seed category contract-intake defaults to capability-tier review before service creation.
N.025. Seed category contract-intake requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.025. Seed category contract-intake may request new-service status only by passing every Section D-10 test with evidence.
N.026. Seed category clause-library defaults to capability-tier review before service creation.
N.026. Seed category clause-library requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.026. Seed category clause-library may request new-service status only by passing every Section D-10 test with evidence.
N.027. Seed category signature-routing defaults to capability-tier review before service creation.
N.027. Seed category signature-routing requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.027. Seed category signature-routing may request new-service status only by passing every Section D-10 test with evidence.
N.028. Seed category obligation-tracking defaults to capability-tier review before service creation.
N.028. Seed category obligation-tracking requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.028. Seed category obligation-tracking may request new-service status only by passing every Section D-10 test with evidence.
N.029. Seed category budgeting defaults to capability-tier review before service creation.
N.029. Seed category budgeting requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.029. Seed category budgeting may request new-service status only by passing every Section D-10 test with evidence.
N.030. Seed category forecasting defaults to capability-tier review before service creation.
N.030. Seed category forecasting requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.030. Seed category forecasting may request new-service status only by passing every Section D-10 test with evidence.
N.031. Seed category scenario-planning defaults to capability-tier review before service creation.
N.031. Seed category scenario-planning requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.031. Seed category scenario-planning may request new-service status only by passing every Section D-10 test with evidence.
N.032. Seed category variance-analysis defaults to capability-tier review before service creation.
N.032. Seed category variance-analysis requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.032. Seed category variance-analysis may request new-service status only by passing every Section D-10 test with evidence.
N.033. Seed category incident defaults to capability-tier review before service creation.
N.033. Seed category incident requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.033. Seed category incident may request new-service status only by passing every Section D-10 test with evidence.
N.034. Seed category problem defaults to capability-tier review before service creation.
N.034. Seed category problem requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.034. Seed category problem may request new-service status only by passing every Section D-10 test with evidence.
N.035. Seed category change defaults to capability-tier review before service creation.
N.035. Seed category change requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.035. Seed category change may request new-service status only by passing every Section D-10 test with evidence.
N.036. Seed category cmdb defaults to capability-tier review before service creation.
N.036. Seed category cmdb requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.036. Seed category cmdb may request new-service status only by passing every Section D-10 test with evidence.
N.037. Seed category procurement defaults to capability-tier review before service creation.
N.037. Seed category procurement requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.037. Seed category procurement may request new-service status only by passing every Section D-10 test with evidence.
N.038. Seed category supplier-risk defaults to capability-tier review before service creation.
N.038. Seed category supplier-risk requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.038. Seed category supplier-risk may request new-service status only by passing every Section D-10 test with evidence.
N.039. Seed category purchase-approval defaults to capability-tier review before service creation.
N.039. Seed category purchase-approval requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.039. Seed category purchase-approval may request new-service status only by passing every Section D-10 test with evidence.
N.040. Seed category goods-receipt defaults to capability-tier review before service creation.
N.040. Seed category goods-receipt requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.040. Seed category goods-receipt may request new-service status only by passing every Section D-10 test with evidence.
N.041. Seed category trade-screening defaults to capability-tier review before service creation.
N.041. Seed category trade-screening requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.041. Seed category trade-screening may request new-service status only by passing every Section D-10 test with evidence.
N.042. Seed category customs defaults to capability-tier review before service creation.
N.042. Seed category customs requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.042. Seed category customs may request new-service status only by passing every Section D-10 test with evidence.
N.043. Seed category export-control defaults to capability-tier review before service creation.
N.043. Seed category export-control requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.043. Seed category export-control may request new-service status only by passing every Section D-10 test with evidence.
N.044. Seed category sanctions defaults to capability-tier review before service creation.
N.044. Seed category sanctions requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.044. Seed category sanctions may request new-service status only by passing every Section D-10 test with evidence.
N.045. Seed category cash-position defaults to capability-tier review before service creation.
N.045. Seed category cash-position requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.045. Seed category cash-position may request new-service status only by passing every Section D-10 test with evidence.
N.046. Seed category fx-hedge defaults to capability-tier review before service creation.
N.046. Seed category fx-hedge requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.046. Seed category fx-hedge may request new-service status only by passing every Section D-10 test with evidence.
N.047. Seed category debt-management defaults to capability-tier review before service creation.
N.047. Seed category debt-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.047. Seed category debt-management may request new-service status only by passing every Section D-10 test with evidence.
N.048. Seed category bank-account-control defaults to capability-tier review before service creation.
N.048. Seed category bank-account-control requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.048. Seed category bank-account-control may request new-service status only by passing every Section D-10 test with evidence.
N.049. Seed category warehouse-task defaults to capability-tier review before service creation.
N.049. Seed category warehouse-task requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.049. Seed category warehouse-task may request new-service status only by passing every Section D-10 test with evidence.
N.050. Seed category inventory-movement defaults to capability-tier review before service creation.
N.050. Seed category inventory-movement requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.050. Seed category inventory-movement may request new-service status only by passing every Section D-10 test with evidence.
N.051. Seed category yard-management defaults to capability-tier review before service creation.
N.051. Seed category yard-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.051. Seed category yard-management may request new-service status only by passing every Section D-10 test with evidence.
N.052. Seed category slotting defaults to capability-tier review before service creation.
N.052. Seed category slotting requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.052. Seed category slotting may request new-service status only by passing every Section D-10 test with evidence.
N.053. Seed category production-order defaults to capability-tier review before service creation.
N.053. Seed category production-order requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.053. Seed category production-order may request new-service status only by passing every Section D-10 test with evidence.
N.054. Seed category mrp defaults to capability-tier review before service creation.
N.054. Seed category mrp requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.054. Seed category mrp may request new-service status only by passing every Section D-10 test with evidence.
N.055. Seed category capacity-plan defaults to capability-tier review before service creation.
N.055. Seed category capacity-plan requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.055. Seed category capacity-plan may request new-service status only by passing every Section D-10 test with evidence.
N.056. Seed category shop-floor defaults to capability-tier review before service creation.
N.056. Seed category shop-floor requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.056. Seed category shop-floor may request new-service status only by passing every Section D-10 test with evidence.
N.057. Seed category quality-inspection defaults to capability-tier review before service creation.
N.057. Seed category quality-inspection requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.057. Seed category quality-inspection may request new-service status only by passing every Section D-10 test with evidence.
N.058. Seed category nonconformance defaults to capability-tier review before service creation.
N.058. Seed category nonconformance requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.058. Seed category nonconformance may request new-service status only by passing every Section D-10 test with evidence.
N.059. Seed category certificate-analysis defaults to capability-tier review before service creation.
N.059. Seed category certificate-analysis requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.059. Seed category certificate-analysis may request new-service status only by passing every Section D-10 test with evidence.
N.060. Seed category quality-audit defaults to capability-tier review before service creation.
N.060. Seed category quality-audit requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.060. Seed category quality-audit may request new-service status only by passing every Section D-10 test with evidence.
N.061. Seed category plant-work-order defaults to capability-tier review before service creation.
N.061. Seed category plant-work-order requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.061. Seed category plant-work-order may request new-service status only by passing every Section D-10 test with evidence.
N.062. Seed category preventive-maintenance defaults to capability-tier review before service creation.
N.062. Seed category preventive-maintenance requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.062. Seed category preventive-maintenance may request new-service status only by passing every Section D-10 test with evidence.
N.063. Seed category spare-part defaults to capability-tier review before service creation.
N.063. Seed category spare-part requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.063. Seed category spare-part may request new-service status only by passing every Section D-10 test with evidence.
N.064. Seed category equipment-master defaults to capability-tier review before service creation.
N.064. Seed category equipment-master requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.064. Seed category equipment-master may request new-service status only by passing every Section D-10 test with evidence.
N.065. Seed category lease-management defaults to capability-tier review before service creation.
N.065. Seed category lease-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.065. Seed category lease-management may request new-service status only by passing every Section D-10 test with evidence.
N.066. Seed category facility-management defaults to capability-tier review before service creation.
N.066. Seed category facility-management requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.066. Seed category facility-management may request new-service status only by passing every Section D-10 test with evidence.
N.067. Seed category real-estate-payment defaults to capability-tier review before service creation.
N.067. Seed category real-estate-payment requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.067. Seed category real-estate-payment may request new-service status only by passing every Section D-10 test with evidence.
N.068. Seed category occupancy-plan defaults to capability-tier review before service creation.
N.068. Seed category occupancy-plan requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.068. Seed category occupancy-plan may request new-service status only by passing every Section D-10 test with evidence.
N.069. Seed category whiteboard-session defaults to capability-tier review before service creation.
N.069. Seed category whiteboard-session requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.069. Seed category whiteboard-session may request new-service status only by passing every Section D-10 test with evidence.
N.070. Seed category design-review defaults to capability-tier review before service creation.
N.070. Seed category design-review requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.070. Seed category design-review may request new-service status only by passing every Section D-10 test with evidence.
N.071. Seed category content-classification defaults to capability-tier review before service creation.
N.071. Seed category content-classification requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.071. Seed category content-classification may request new-service status only by passing every Section D-10 test with evidence.
N.072. Seed category retention-hold defaults to capability-tier review before service creation.
N.072. Seed category retention-hold requires Cedar permit set, ontology projection, workflow template, UX shell, compliance overlay, audit profile, and cost profile.
N.072. Seed category retention-hold may request new-service status only by passing every Section D-10 test with evidence.

## Appendix O - Implementation acceptance criteria

O.001. Acceptance criterion: The ADR file exists at docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md.
O.002. Acceptance criterion: The ADR line count is at least 2000 lines.
O.003. Acceptance criterion: Section A covers SaaS fragmentation tax, vendor lock-in, per-department training, integration debt, and named platform precedents.
O.004. Acceptance criterion: Section B declares capability tiers as projection bundles and forbids adjacent product-fragment microservices.
O.005. Acceptance criterion: Section C covers maintainability, observability, scalability, performance, optimization, and code quality.
O.006. Acceptance criterion: Section D-1 covers Cedar permit sets.
O.007. Acceptance criterion: Section D-2 covers ontology projections.
O.008. Acceptance criterion: Section D-3 covers workflow template libraries.
O.009. Acceptance criterion: Section D-4 covers UX shells.
O.010. Acceptance criterion: Section D-5 covers compliance overlays.
O.011. Acceptance criterion: Section D-6 covers composition including Sales-Cloud-class capability tier.
O.012. Acceptance criterion: Section D-7 includes Postgres DDL for tenant_capability_tier_grants.
O.013. Acceptance criterion: Section D-8 covers promotion and sunset lifecycle.
O.014. Acceptance criterion: Section D-9 covers cross-jurisdiction interactions.
O.015. Acceptance criterion: Section D-10 covers when a new microservice is required.
O.016. Acceptance criterion: Section E names oya-shared-capability-tier-registry and Cedar entity types.
O.017. Acceptance criterion: Section F declares migration waves and per-existing-service declarations.
O.018. Acceptance criterion: Section G cites internal ADRs and external platform documentation.
O.019. Acceptance criterion: Section H records change log and naming justifications.
O.020. Acceptance criterion: Appendix I enumerates capability-tier examples.
O.021. Acceptance criterion: Appendix J supplies at least two hyperscaler precedents per primitive.
O.022. Acceptance criterion: Appendix K supplies per-service declaration obligations.
O.023. Acceptance criterion: Appendix L supplies the product-fragmentation review checklist.
O.024. Acceptance criterion: Appendix M supplies validation evidence contract rows.
O.025. Acceptance criterion: Appendix N supplies catalog seed rows.

## Appendix P - Threat and failure-mode ledger

P.001. Failure mode catalog-sprawl: rejects a new service when a named permit set and projection can express the category.
P.001. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.002. Failure mode permission-drift: requires Cedar policies to reference stable tier identifiers instead of product aliases.
P.002. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.003. Failure mode ontology-fork: requires object-type reuse or versioned extension before a department-specific schema appears.
P.003. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.004. Failure mode workflow-fork: forces template variation into parameterized workflow definitions with provenance.
P.004. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.005. Failure mode ux-fork: keeps navigation, widgets, and gestures declared by tier manifest instead of copied shells.
P.005. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.006. Failure mode audit-gap: requires activation, denial, exception, sunset, and promotion events in the shared audit stream.
P.006. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.007. Failure mode billing-shadow: requires cost attribution to follow tier grants, not local product codes.
P.007. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.008. Failure mode jurisdiction-leak: requires compliance overlays to filter projection visibility before workflow launch.
P.008. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.009. Failure mode integration-rebuild: requires connectors to bind to ontology objects and events instead of category applications.
P.009. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.010. Failure mode tenant-surprise: requires tenant-visible activation evidence, entitlement reason, and sunset date.
P.010. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.011. Failure mode support-fragmentation: requires support runbooks to name the shared primitive and tier manifest.
P.011. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.012. Failure mode metric-incompatibility: requires observability dimensions to include tier, primitive, tenant, region, and pack.
P.012. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.013. Failure mode search-fragmentation: requires search facets to derive from ontology projection labels.
P.013. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.014. Failure mode reporting-fragmentation: requires analytics marts to read shared event and object streams.
P.014. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.015. Failure mode compliance-duplication: requires regulatory mappings to live in pack overlays.
P.015. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.016. Failure mode migration-stall: requires every existing microservice to publish the tiers it exposes.
P.016. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.017. Failure mode admission-bypass: requires new-service proposals to attach Section D-10 evidence.
P.017. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.018. Failure mode ai-policy-gap: requires intelligence features to inherit tier grants and ontology scope.
P.018. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.019. Failure mode marketplace-silo: requires marketplace listings to declare compatible tier manifests.
P.019. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.020. Failure mode community-silo: requires communities to attach to the same tenant objects and grants.
P.020. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.021. Failure mode messenger-silo: requires messages to reference shared subjects, workflows, and retention packs.
P.021. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.022. Failure mode mail-silo: requires mail projections to respect the same consent, retention, and audit overlays.
P.022. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.023. Failure mode analytics-silo: requires dashboards to declare primitive provenance and tenant grant scope.
P.023. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.024. Failure mode performance-blindness: requires hot-path metrics per primitive and tier before optimization.
P.024. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.025. Failure mode code-owner-confusion: requires each tier manifest to name owning primitive services and review groups.
P.025. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.026. Failure mode feature-flag-conflict: requires feature flags to compose with tenant tier grants.
P.026. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.027. Failure mode sunset-limbo: requires every retired tier to define replacement tier, migration script, and audit closure.
P.027. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.028. Failure mode naming-drift: requires category names to remain marketing labels over canonical tier identifiers.
P.028. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.029. Failure mode vendor-lock-in-replay: requires imports to normalize into ontology objects and workflow events.
P.029. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.030. Failure mode training-friction: requires a shared UX shell so role training explains surfaced capabilities, not new apps.
P.030. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.031. Failure mode data-residency-conflict: requires region and jurisdiction overlays to bind before projection materialization.
P.031. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.032. Failure mode tenant-admin-overload: requires admin controls to group tiers by primitive and compliance pack.
P.032. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.033. Failure mode duplicated-permission-ui: requires entitlement UI to render Cedar grants from shared registry state.
P.033. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.034. Failure mode duplicated-settings: requires settings panes to bind to tier manifests and primitive adapters.
P.034. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.035. Failure mode duplicated-notifications: requires notification routing to consume workflow events and tier audience rules.
P.035. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.
P.036. Failure mode duplicated-export: requires export jobs to resolve ontology projections and compliance restrictions.
P.036. Evidence gate: reviewer confirms Cedar grant, ontology scope, workflow template, UX manifest, compliance overlay, and audit event before category launch.

## Appendix Q - Compliance overlay control ledger

Q.001. Compliance control gdpr-export: subject export must traverse ontology projection and redact denied attributes.
Q.001. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.002. Compliance control gdpr-erasure: erasure must bind to object retention state and tier-specific legal holds.
Q.002. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.003. Compliance control ccpa-disclosure: consumer disclosure must include tier activation basis and shared-data processors.
Q.003. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.004. Compliance control hipaa-minimum-necessary: health attributes must be invisible unless the compliance pack grants purpose-bound access.
Q.004. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.005. Compliance control sox-change-control: financial planning workflows must emit immutable approval and configuration-change events.
Q.005. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.006. Compliance control pci-token-boundary: payment references must remain tokenized and inaccessible to non-payment tiers.
Q.006. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.007. Compliance control fedramp-boundary: public-sector tenants must pin region, audit, identity, and incident overlays.
Q.007. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.008. Compliance control data-residency-eu: EU resident data must not project into non-EU workflows without transfer basis.
Q.008. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.009. Compliance control data-residency-kr: Korean-resident regulated data must bind to Korean residency and retention packs.
Q.009. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.010. Compliance control ai-output-review: intelligence tier output must record source objects, grant scope, and reviewer state.
Q.010. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.011. Compliance control model-training-consent: training use must be an explicit tenant grant separate from operational inference.
Q.011. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.012. Compliance control legal-hold: hold state must override tier sunset deletion plans.
Q.012. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.013. Compliance control retention-disposal: disposal jobs must prove no active workflow or hold needs the object.
Q.013. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.014. Compliance control segregation-of-duties: workflow approvals must block conflicting role assignments from the same principal.
Q.014. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.015. Compliance control access-recertification: tenant admins must review active tier grants on a scheduled cadence.
Q.015. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.016. Compliance control cross-border-case: case workflows spanning jurisdictions must compute the stricter overlay before disclosure.
Q.016. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.017. Compliance control third-party-connector: connector scopes must be generated from tier manifest and tenant grant state.
Q.017. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.018. Compliance control audit-export: auditors receive normalized activation, access, workflow, and evidence events.
Q.018. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.019. Compliance control incident-response: incidents name affected tier, primitive, region, tenant, and compliance pack.
Q.019. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.020. Compliance control breach-notice: notification clocks derive from compliance pack and tenant jurisdiction.
Q.020. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.021. Compliance control record-of-processing: processing records cite tier identifier, purpose, object type, and retention rule.
Q.021. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.022. Compliance control field-level-masking: masked fields remain masked in UI shell, API projection, export, and analytics.
Q.022. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.023. Compliance control encryption-key-scope: keys bind to tenant, region, pack, and sensitive object class.
Q.023. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.024. Compliance control privileged-access: break-glass access records purpose, expiration, reviewer, and tier scope.
Q.024. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.025. Compliance control sandbox-copy: sandbox projection excludes restricted attributes unless synthetic replacement is present.
Q.025. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.026. Compliance control analytics-aggregation: analytics tiers cannot expose cohorts below the compliance pack threshold.
Q.026. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.027. Compliance control marketplace-extension: extension manifests must declare requested primitives and compliance claims.
Q.027. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.028. Compliance control community-visibility: external community roles require explicit audience and object-scope grants.
Q.028. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.029. Compliance control mail-retention: mail tier retention follows pack policy even when workflow retention differs.
Q.029. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.030. Compliance control messenger-discovery: messenger discovery must align with legal hold and export overlays.
Q.030. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.031. Compliance control learning-record: learning tiers must separate employee development records from performance decisions.
Q.031. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.
Q.032. Compliance control hr-sensitive-field: HR projections hide protected attributes from unrelated operational workflows.
Q.032. Overlay rule: enforcement is declared once in the pack and consumed by Cedar, ontology projection, workflow launch, UX rendering, analytics, export, and audit.

## Appendix R - Promotion and sunset evidence ledger

R.001. Lifecycle stage draft-tier: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.001. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.002. Lifecycle stage internal-preview: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.002. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.003. Lifecycle stage tenant-preview: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.003. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.004. Lifecycle stage limited-availability: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.004. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.005. Lifecycle stage general-availability: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.005. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.006. Lifecycle stage regulated-availability: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.006. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.007. Lifecycle stage regional-expansion: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.007. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.008. Lifecycle stage marketplace-extension: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.008. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.009. Lifecycle stage template-refresh: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.009. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.010. Lifecycle stage ontology-extension: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.010. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.011. Lifecycle stage workflow-variant: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.011. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.012. Lifecycle stage ux-shell-refresh: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.012. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.013. Lifecycle stage compliance-pack-refresh: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.013. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.014. Lifecycle stage performance-optimization: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.014. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.015. Lifecycle stage cost-optimization: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.015. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.016. Lifecycle stage support-hardening: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.016. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.017. Lifecycle stage audit-hardening: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.017. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.018. Lifecycle stage connector-recertification: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.018. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.019. Lifecycle stage analytics-recertification: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.019. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.020. Lifecycle stage intelligence-recertification: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.020. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.021. Lifecycle stage deprecated: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.021. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.022. Lifecycle stage replacement-offered: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.022. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.023. Lifecycle stage sunset-announced: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.023. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.024. Lifecycle stage grant-freeze: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.024. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.025. Lifecycle stage migration-window: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.025. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.026. Lifecycle stage tenant-cutover: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.026. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.027. Lifecycle stage read-only-period: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.027. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.028. Lifecycle stage deactivation: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.028. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.029. Lifecycle stage archive: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.029. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.
R.030. Lifecycle stage final-audit-close: promotion cannot proceed without manifest version, owner, tenant impact, rollback path, and audit evidence.
R.030. Sunset evidence: replacement tier, affected tenants, data migration, workflow migration, compliance hold review, and support notice must be recorded before closure.

## Appendix S - Cross-jurisdiction interaction ledger

S.001. Jurisdiction interaction eu-to-us-sales-case: Sales tier in the US consuming EU lead records must use transfer basis and EU masking before workflow assignment.
S.001. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.002. Jurisdiction interaction kr-hr-to-global-learning: Korean HR records projected into global learning must expose only training eligibility and completion state.
S.002. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.003. Jurisdiction interaction us-finance-to-eu-contract: US planning forecasts attached to EU contracts must preserve contract-region retention and disclosure rules.
S.003. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.004. Jurisdiction interaction public-sector-itsm: ITSM incidents for public-sector tenants must retain region-pinned audit and incident response overlays.
S.004. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.005. Jurisdiction interaction health-support-case: support workflows that reference health data must inherit minimum-necessary fields and purpose labels.
S.005. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.006. Jurisdiction interaction marketplace-third-party-eu: EU marketplace extensions must declare processors and subprocessor evidence before grant activation.
S.006. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.007. Jurisdiction interaction community-external-party: external community participants must receive audience-scoped projections, not full tenant objects.
S.007. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.008. Jurisdiction interaction mail-cross-border: mail projections crossing regions must preserve retention, discovery, and recipient consent overlays.
S.008. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.009. Jurisdiction interaction messenger-legal-hold: messenger threads under hold remain discoverable even when the adjacent tier sunsets.
S.009. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.010. Jurisdiction interaction analytics-global-dashboard: global dashboards must aggregate under the strictest jurisdiction in the result set.
S.010. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.011. Jurisdiction interaction intelligence-assistant: assistant prompts must exclude denied fields and record source object provenance.
S.011. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.012. Jurisdiction interaction procurement-to-finance: procurement workflow data projected to finance must keep vendor residency and sanctions-screening packs.
S.012. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.013. Jurisdiction interaction employee-performance-to-learning: performance feedback projected to learning must avoid protected attributes and adverse-action leakage.
S.013. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.014. Jurisdiction interaction contract-to-service: contract obligations projected to service workflows must preserve governing law and notice obligations.
S.014. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.015. Jurisdiction interaction field-service-spares: field service spares across regions must keep import, export, and inventory compliance attributes.
S.015. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.016. Jurisdiction interaction real-estate-lease: lease tiers must enforce location, payment, and occupancy data restrictions independently.
S.016. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.017. Jurisdiction interaction customer-success-community: customer success communities must hide internal account health fields from external users.
S.017. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.018. Jurisdiction interaction marketing-consent: marketing automation must consume consent state from shared profiles before campaign enrollment.
S.018. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.019. Jurisdiction interaction invoice-retention: invoice workflows must observe longer of tax-retention and legal-hold requirements.
S.019. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.
S.020. Jurisdiction interaction incident-war-room: war-room tiers must record every external participant and disclosed object projection.
S.020. Required computation: resolve tenant region, subject region, object region, compliance pack, Cedar grant, and workflow purpose before exposing the projection.

## Appendix T - New-service exception examples

T.001. New-service exception cryptographic-key-custody: requires isolated custody, independent threat model, and hardware-backed controls beyond a projection.
T.001. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.002. New-service exception real-time-media-plane: requires latency and packet semantics that the workflow substrate does not own.
T.002. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.003. New-service exception payments-clearing: requires payment-network certification, token boundary ownership, and settlement invariants.
T.003. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.004. New-service exception identity-provider-core: requires authentication protocol ownership and tenant boundary guarantees.
T.004. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.005. New-service exception search-index-engine: requires indexing infrastructure shared across tiers rather than a category product.
T.005. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.006. New-service exception message-delivery-plane: requires delivery reliability and queue semantics shared by mail, messenger, and notification tiers.
T.006. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.007. New-service exception data-warehouse-engine: requires analytical storage contracts that outlive category projections.
T.007. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.008. New-service exception policy-decision-engine: requires Cedar-compatible decision ownership and audit guarantees.
T.008. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.009. New-service exception workflow-runtime-engine: requires execution semantics, retries, timers, and compensation primitives.
T.009. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.010. New-service exception ontology-registry: requires canonical object-type lifecycle and compatibility rules.
T.010. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.011. New-service exception marketplace-runtime: requires extension isolation, installation lifecycle, and billing controls.
T.011. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.012. New-service exception file-storage-engine: requires blob lifecycle, scanning, retention, and encryption primitives.
T.012. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.013. New-service exception observability-pipeline: requires log, metric, trace, and event ingestion contracts.
T.013. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.014. New-service exception region-router: requires placement and routing guarantees for residency and latency.
T.014. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.015. New-service exception billing-ledger: requires immutable money and entitlement accounting.
T.015. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.016. New-service exception audit-ledger: requires append-only evidence semantics across all tiers.
T.016. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.017. New-service exception ai-inference-runtime: requires prompt, model, safety, provenance, and cost controls shared across categories.
T.017. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.018. New-service exception integration-runtime: requires connector execution and secret management beyond one capability tier.
T.018. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.019. New-service exception notification-runtime: requires fanout, retry, preference, and deliverability primitives.
T.019. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.
T.020. New-service exception tenant-admin-core: requires principal, role, grant, and entitlement administration across all tiers.
T.020. Boundary note: the service exists as a primitive when its invariants are reusable substrate capabilities, not because a department wants a branded application.

## Appendix U - Reviewer closeout checklist

U.001. Closeout check: Cedar permit-set identifiers are canonical and product-label aliases are secondary.
U.001. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.
U.002. Closeout check: Ontology projections are versioned and tied to ADR-0257 object-type lifecycle rules.
U.002. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.
U.003. Closeout check: Workflow templates are parameterized and reusable across adjacent categories.
U.003. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.
U.004. Closeout check: UX shells render from tier manifests and do not fork product applications.
U.004. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.
U.005. Closeout check: Compliance overlays execute before projection materialization and export.
U.005. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.
U.006. Closeout check: Sales-Cloud-class composition names workflow, ontology, community, marketplace, messenger, mail, intelligence, and analytics tiers.
U.006. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.
U.007. Closeout check: Postgres grants include tenant, tier, status, source, audit, time, region, and pack fields.
U.007. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.
U.008. Closeout check: Existing microservices declare exposed tiers before marketing new categories.
U.008. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.
U.009. Closeout check: New-service proposals cite Section D-10 and Appendix T evidence.
U.009. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.
U.010. Closeout check: References include internal ADRs and platform precedent documents.
U.010. Reviewer action: mark pass only when the ADR text names the mechanism and not merely the intent.

## Deliverable report

File path: docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md.
Line count target: at least 2000 lines; verified after write with wc -l.
Capability-tier examples enumerated: sales-cloud-core, service-cloud-core, marketing-automation-core, hr-core, performance-management-core, learning-management-core, contract-lifecycle-core, financial-planning-core, itsm-core, content-classification-core, and catalog seed tiers in Appendix N.
Cross-references: ADR-0132, ADR-0245, ADR-0249, ADR-0257, ADR-0313, ADR-0314, ADR-0315.
External precedents cited: Salesforce Platform and App Cloud, ServiceNow Now Platform and low-code/hyperautomation, Microsoft 365 and Microsoft Graph, Atlassian Forge, Notion API and capabilities, and Gartner enterprise software market-share analysis.
