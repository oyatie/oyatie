---
doc_class: Architecture
microservice: crm
status: Wave-15A-Rewritten
date: 2026-05-21
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
  - ADR-0254
  - ADR-0263
  - ADR-0297
  - ADR-0314
  - ADR-0328
  - ADR-0329
  - ADR-0330
  - ADR-0331
supersedes:
  - microservices/crm/ARCHITECTURE.md@2026-05-20 §H (90 stamped Architecture-trace lines)
companion_docs:
  - microservices/crm/README.md
  - microservices/crm/PRD.md
  - microservices/crm/compliance.md
  - microservices/crm/manifest.json
  - microservices/crm/competitor-parity-matrix.md
  - microservices/crm/REMEDIATION-NOTES-2026-05-21.md
---

# Architecture: Customer Relationship Management

## A. Boundary

This service owns the lifecycle of revenue-bearing customer relationships across the thirteen bounded contexts enumerated in §C. It does not own ERP master data, payment rails, identity, policy runtime, workflow runtime, ontology storage, marketplace settlement, marketing-automation journeys, contact-center channel surfaces, knowledge-base storage, email rendering, calendar scheduling, contract lifecycle management, predictive intelligence, or call recording transcription. Each excluded concern is named in §D Integration Topology with the substrate microservice that owns it and the contract path the handoff uses. The boundary follows ADR-0245 substrate-vs-product layering: `crm` is a product microservice that depends on substrate microservices; the substrate never depends on `crm`.

## B. Layer Map

The ADR-0105 13-layer enum binds module organization under `src/`. The Wave 15A target layer set is:

| ADR-0105 layer | Planned responsibility | Wave 15A status |
|---|---|---|
| api | public command/query DTOs and OpenAPI 3.2.0 contract binding | DEFERRED to Wave 15B (currently embedded in rest/) |
| rest | HTTP transport and idempotency enforcement | PRESENT (`src/adapter/http`) |
| application | orchestration of usecases and transactions | DEFERRED to Wave 15B |
| usecase | command handlers and read models | PRESENT (`src/usecase`) |
| domain | business invariants and aggregate roots | PRESENT (`src/domain`) |
| kernel | pure value objects and deterministic calculations | DEFERRED to Wave 15B (currently embedded in domain/) |
| adapter | source-system, database, and external-system adapters | PRESENT (`src/adapter`) |
| worker | batch migration, reconciliation, and async workflow workers | DEFERRED to Wave 15B |
| governance | policy, compliance, scorecards, and evidence gates | DEFERRED to Wave 15B |
| config | per-context configuration loading | PRESENT (`src/config`) |
| error | typed error envelopes | PRESENT (`src/error`) |
| infrastructure | low-level I/O wrappers | DEFERRED to Wave 15B |
| integration | external-system bidirectional sync | DEFERRED to Wave 15B (currently embedded in adapter/) |

Layer-flow rule: outer layers depend on inner layers; inner layers never depend on outer layers. The rule is enforced via `cargo deny` configuration + a layer-flow CI lane.

## C. Bounded Context Architecture

Wave 15A expands the six-aggregate Wave 3-G surface to thirteen first-class aggregates plus one read-model aggregate (`customer-360`). The expansion reflects the union-coverage bar across Salesforce + HubSpot + Dynamics 365 Sales canonical surfaces.

### account-master
- Aggregate root: `account_master_document`.
- Counterpart mapping: Salesforce Account + AccountHierarchy + AccountTeamMember; HubSpot Company; Dynamics Account + Sales Hierarchy.
- Invariants: tenant scope required, version monotonic, source-system provenance immutable, destructive correction forbidden, hierarchy is acyclic, rollup is recomputed on parent change within 60 seconds.
- Commands: create, amend, approve, reverse, archive, import, export, reparent, set-rollup-rule, add-team-member, remove-team-member.
- Events: created, amended, approved, reversed, archived, imported, exported, reparented, rollup-rule-set, team-member-added, team-member-removed.
- Read model: tenant-scoped projection keyed by account_id, plus an inverted index by territory_id, owner_principal_id, account_team_role.

### contact
- Aggregate root: `contact_document`.
- Counterpart mapping: Salesforce Contact + Person Account + ContactRole; HubSpot Contact (primary record type); Dynamics Contact.
- Invariants: tenant scope, version monotonic, email-uniqueness per tenant (with conflict-merge), Person Account dual-semantic enforced when `is_person_account = true`.
- Commands: create, amend, approve, archive, merge, set-lifecycle-stage, link-to-account, unlink-from-account, set-do-not-contact, log-activity.
- Events: created, amended, approved, archived, merged, lifecycle-stage-set, linked, unlinked, dnc-set, activity-logged.
- Read model: tenant-scoped projection keyed by contact_id; inverted index by email_lower, account_id, lifecycle_stage.

### lead
- Aggregate root: `lead_document`.
- Counterpart mapping: Salesforce Lead + LeadConversion; HubSpot Contact lifecycle_stage in {Subscriber, Lead, MQL, SQL}; Dynamics Lead entity.
- Invariants: tenant scope, version monotonic, conversion is one-way (Lead → {Account, Contact, Opportunity} triple), source attribution required.
- Commands: create, amend, score, qualify, disqualify, assign, convert, web-to-lead-ingest, email-to-lead-ingest.
- Events: created, amended, scored, qualified, disqualified, assigned, converted, ingested.
- Read model: tenant-scoped projection keyed by lead_id; inverted index by lead_source, lead_status, score_bucket, owner_principal_id, territory_id.

### opportunity
- Aggregate root: `opportunity_document`.
- Counterpart mapping: Salesforce Opportunity + OpportunityHistory + OpportunityContactRole + OpportunityCompetitor; HubSpot Deal + Deal Pipelines; Dynamics Opportunity + BPF.
- Invariants: tenant scope, version monotonic, stage transition is Cedar-gated, OpportunityHistory append-only, amount + currency present at Proposal stage or later.
- Commands: create, amend, advance-stage, revert-stage, win, lose, disqualify, archive, set-forecast-category, set-big-deal-alert, link-campaign, set-pipeline.
- Events: created, amended, stage-advanced, stage-reverted, won, lost, disqualified, archived, forecast-category-set, big-deal-alert-tripped, campaign-linked, pipeline-set.
- Read model: tenant-scoped projection keyed by opportunity_id; inverted indices by pipeline_id, stage, close_period, owner_principal_id, forecast_category.

### opportunity-team
- Aggregate root: `opportunity_team_document`.
- Counterpart mapping: Salesforce OpportunityTeamMember; HubSpot (via custom property arrays); Dynamics (via Connections).
- Invariants: tenant scope, members are tenant principals, role is from a closed enum + tenant-extended set, primary owner exactly one.
- Commands: add-member, remove-member, set-role, set-primary-owner.
- Events: member-added, member-removed, role-set, primary-owner-set.
- Read model: per-opportunity flat list of (principal_id, role, access_level).

### opportunity-split
- Aggregate root: `opportunity_split_document`.
- Counterpart mapping: Salesforce OpportunitySplit; HubSpot (via custom calculation); Dynamics (via custom plug-in).
- Invariants: tenant scope, split-type from closed enum {Revenue, Overlay, Custom}, Revenue splits must sum to 100% per opportunity, Overlay splits independent.
- Commands: define-split, update-split-percent, remove-split.
- Events: split-defined, split-updated, split-removed.
- Read model: per-opportunity list of (principal_id, split_type, split_pct).

### sales-cadence
- Aggregate root: `sales_cadence_document` + child `sales_cadence_enrolment`.
- Counterpart mapping: Salesforce Sales Engagement Cadence; HubSpot Sequences; Dynamics Sales Sequences.
- Invariants: tenant scope, step graph is a DAG, enrolment has unique (cadence_id, target_id), target_id ∈ Leads ∪ Contacts.
- Commands: create-cadence, define-step, set-step-conditions, activate, deactivate, enrol-target, advance-step, complete-step, exit-enrolment, log-step-outcome.
- Events: cadence-created, step-defined, conditions-set, activated, deactivated, target-enrolled, step-advanced, step-completed, enrolment-exited, step-outcome-logged.
- Read model: per-tenant cadence catalog; per-cadence enrolment list; per-target current step.

### cpq-quote
- Aggregate root: `cpq_quote_document` + child `cpq_quote_line` + child `cpq_quote_attribute` + child `cpq_approval_step`.
- Counterpart mapping: Salesforce CPQ Quote + QuoteLineItem + Quote Templates + Advanced Approvals; HubSpot Quote + Quote Tool; Dynamics Quote + QuoteDetail.
- Invariants: tenant scope, quote-status ∈ {Draft, Submitted-for-Approval, Approved, Rejected, Sent, Accepted, Expired, Withdrawn}, configuration constraint rules satisfied at quote-line edit time, total price = sum of line totals + tax (delegated to cloud-billing-tax for tax computation), discount approval matrix evaluated at submit.
- Commands: create-quote, configure-line, set-attribute, apply-discount, submit-for-approval, approve-step, reject-step, recall-approval, generate-document, send-to-customer, mark-accepted, withdraw, expire.
- Events: quote-created, line-configured, attribute-set, discount-applied, submitted, step-approved, step-rejected, recalled, document-generated, sent, accepted, withdrawn, expired.
- Read model: per-quote projection with line items, approval state, document URL; per-tenant quote pipeline indexed by status.

### forecast
- Aggregate root: `forecast_document` + child `forecast_snapshot` + child `quota_assignment`.
- Counterpart mapping: Salesforce Collaborative Forecasts + Forecast Categories + Forecast Quotas + Forecast Adjustments + Forecast Hierarchy; HubSpot Forecast Tools; Dynamics Forecast entity + Snapshot.
- Invariants: tenant scope, forecast period is from closed enum (Monthly, Quarterly, Custom), forecast category ∈ {Pipeline, Best Case, Commit, Closed} per Opportunity, manager adjustment is auditable, quota is per-principal per-period.
- Commands: open-forecast-period, set-forecast-category-on-opp, adjust-forecast, take-snapshot, assign-quota, lock-period.
- Events: period-opened, category-set, adjusted, snapshot-taken, quota-assigned, period-locked.
- Read model: per-tenant per-period forecast view; per-principal quota view.

### quote
- Aggregate root: same as `cpq-quote` — Wave 15A unifies the legacy `quote` bounded context with the CPQ-grade `cpq-quote` bounded context. The legacy name is preserved for compatibility but the canonical aggregate is `cpq_quote_document`.

### service-case
- Aggregate root: `service_case_document` + child `case_comment` + child `case_milestone` + child `entitlement_clock`.
- Counterpart mapping: Salesforce Case + CaseTeamMember + CaseComment + CaseMilestone + Entitlement + ServiceContract; HubSpot Ticket + Ticket Pipelines + Service SLA; Dynamics Case + Customer Service entity.
- Invariants: tenant scope, status ∈ {New, Open, In-Progress, Pending-Customer, Resolved, Closed, Reopened}, SLA clock pauses on Pending-Customer status, milestone evaluation is monotonic, hierarchy is acyclic.
- Commands: create, route, assign, set-severity, add-comment, attach-entitlement, start-milestone, complete-milestone, pause-clock, resume-clock, escalate, resolve, close, reopen.
- Events: created, routed, assigned, severity-set, comment-added, entitlement-attached, milestone-started, milestone-completed, clock-paused, clock-resumed, escalated, resolved, closed, reopened.
- Read model: per-case projection; per-tenant case pipeline by status + severity + assignee; SLA-clock at-risk index.

### campaign
- Aggregate root: `campaign_document` + child `campaign_member`.
- Counterpart mapping: Salesforce Campaign + CampaignMember + CampaignHierarchy + CampaignInfluence; HubSpot Campaign; Dynamics Campaign + MarketingList.
- Invariants: tenant scope, hierarchy is acyclic, member status from per-campaign-type enum, ROI calculation deterministic at close.
- Commands: create, set-hierarchy-parent, add-member, update-member-status, close-campaign, compute-influence, link-opportunity-attribution.
- Events: created, hierarchy-set, member-added, member-status-updated, closed, influence-computed, attribution-linked.
- Read model: per-campaign member list; per-tenant campaign catalog; per-opportunity attribution rollup.

### loyalty-ledger
- Aggregate root: `loyalty_ledger_document` + append-only `loyalty_journal_entry`.
- Counterpart mapping: Salesforce Loyalty Management (Loyalty Cloud); HubSpot (via subscription + custom-property modelling); Dynamics (via Customer Insights).
- Invariants: tenant scope, journal is append-only, balance = sum(journal entries) monotonically reconcilable, certificate liability decremented atomically on redemption.
- Commands: open-account, accrue, redeem, expire, transfer, set-tier, freeze.
- Events: opened, accrued, redeemed, expired, transferred, tier-set, frozen.
- Read model: per-member balance + tier; per-tenant aggregated liability.

### partner
- Aggregate root: `partner_document` + child `deal_registration` + child `partner_user`.
- Counterpart mapping: Salesforce Experience Cloud Partner Community + Deal Registration + Partner Portal; HubSpot Partner-as-Contact-with-permissions; Dynamics Channel Partner + Partner Center.
- Invariants: tenant scope, deal-registration is unique per (partner, account, opportunity), partner-user is scoped by partner.
- Commands: onboard-partner, register-deal, approve-registration, reject-registration, invite-partner-user, revoke-partner-user, record-co-sell.
- Events: onboarded, registered, approved, rejected, user-invited, user-revoked, co-sell-recorded.
- Read model: per-partner deal pipeline; per-tenant partner catalog.

### customer-360
- Aggregate root: `customer_360_projection` (read-model only; no write commands).
- Counterpart mapping: Salesforce Customer 360 + Customer Data Platform; HubSpot Companies-with-everything-attached view; Dynamics Customer Insights.
- Invariants: tenant scope, projection is denormalised from the eleven write-aggregates above, freshness floor 5 seconds per ADR-0145 freshness floor pattern, stale-read banner exposed when freshness exceeded.
- Commands: none — this is a read-model projection only. Source events trigger projection refresh.
- Events: none on the write side; emits `customer-360.refreshed` for downstream readers.
- Read model: full denormalised customer view per (tenant, account_id) — joins Account, Contacts, Opportunities, Quotes, Cases, Campaigns, Loyalty, Partner-relationships.

## D. Integration Topology

The substrate handoffs `crm` performs are named explicitly with the contract path. Calls are gRPC-over-HTTP/3 per ADR-0145 + ADR-0253. Every call carries `tenant_id`, `principal_id`, `trace_context`, `idempotency_key`, `audit_chain_ref`. The handoff invariants are:

- I1. Idempotency: every cross-µservice command carries an idempotency key; replay returns the prior result.
- I2. Audit-chain seal: every state transition emits an audit-chain event before the response is returned.
- I3. Cedar gate: every operation flows through a Cedar default-deny evaluation before substrate dispatch.

| Substrate | Direction | Contract path | Purpose |
|---|---|---|---|
| workflow-engine | crm → workflow-engine | microservices/workflow-engine/contracts/workflow-v1.proto | Lead conversion saga; Opportunity stage progression; CPQ approval chain; Service Case escalation; Campaign-to-Order handoff. |
| ontology | crm → ontology | microservices/ontology/contracts/ontology-v1.proto | Version-pinned projections for all 14 aggregates. |
| audit-chain | crm → audit-chain | microservices/audit-chain/contracts/audit-chain-v1.proto | Seal-event emission for every state transition. |
| marketplace | crm → marketplace | microservices/marketplace/contracts/marketplace-v1.proto | Tenant deal settlement; crm records `marketplace_settlement_ref` on Order handoff. |
| intelligence | crm → intelligence | microservices/intelligence/contracts/intelligence-v1.proto | Lead Scoring, Opportunity Scoring, Conversation Intelligence, Next-Best-Action, Email Generation, Predictive Forecasting. |
| cloud-iam | crm → cloud-iam | microservices/cloud-iam/contracts/iam-v1.proto | Principal claim resolution (tenant_class, scopes, pack overlays). |
| cloud-billing-tax | crm → cloud-billing-tax | microservices/cloud-billing-tax/contracts/billing-v1.proto | Quote → Order → Invoice + Tax handoff. |
| payments | crm → payments | microservices/payments/contracts/payments-v1.proto | Invoice → Payment handoff. |
| marketing-automation | crm ↔ marketing-automation | microservices/marketing-automation/contracts/marketing-v1.proto | Campaign → Journey activation; Journey → CampaignMember status updates. |
| consent-graph | crm → consent-graph | microservices/consent-graph/contracts/consent-v1.proto | Per-purpose subscription + channel consent reads. |
| contract-lifecycle-management | crm → contract-lifecycle-management | microservices/contract-lifecycle-management/contracts/clm-v1.proto | Contract creation from CPQ Quote; CLM workflow + clause library. |
| contact-center | crm → contact-center | microservices/contact-center/contracts/contact-center-v1.proto | Live channel surfaces; case creation from chat / voice / SMS. |
| community | crm → community | microservices/community/contracts/community-v1.proto | Knowledge Base read + linking; Customer Portal binding. |
| mail | crm → mail | microservices/mail/contracts/mail-v1.proto | Email rendering, delivery, tracking. |
| calendar | crm → calendar | microservices/calendar/contracts/calendar-v1.proto | Meeting Scheduler; calendar sync. |
| analytics | crm → analytics | microservices/analytics/contracts/analytics-v1.proto | Deep-cut reporting + embedded dashboards. |
| workplace-integration | crm ↔ workplace-integration | microservices/workplace-integration/contracts/wpi-v1.proto | LinkedIn Sales Navigator, Microsoft Teams, Slack. |
| forms | crm → forms | microservices/forms/contracts/forms-v1.proto | Web-to-Lead, Web-to-Case, surveys, NPS, CSAT. |
| recordings | crm → recordings | microservices/recordings/contracts/recordings-v1.proto | Call recording storage. |
| data-sync | crm ↔ data-sync | microservices/data-sync/contracts/data-sync-v1.proto | Bidirectional sync with external systems (Salesforce, HubSpot, Dynamics, NetSuite, custom). |
| search | crm → search | microservices/search/contracts/search-v1.proto | Full-text + semantic search across crm aggregates. |

## E. Failure Modes

- Source-system import drift: dry-run evidence identifies row, table, transform, and rejection reason. Per IP-013 adapter integrations.
- Cross-tenant reference attempt: Cedar denies before domain command execution and emits refusal evidence. Per `policy/*.cedar`.
- Duplicate command submission: idempotency key returns previous result and increments duplicate metric.
- Regional outage: writes queue in the tenant home cell and reads expose stale-region metadata. Per `multi-region.md`.
- Audit-chain outage: critical state transitions pause; non-critical queries continue with degraded banner. Per `runbooks/source-import-stalled.md`.
- Cedar policy soak failure: deployments halt during the 60-second soak before enforcement. Per ADR-0243.
- Workflow-engine outage: cross-µservice sagas (Lead conversion, CPQ approval, Service Case escalation) pause; in-flight sagas resume on recovery via durable state.
- Intelligence µservice outage: predictive scoring degrades; manual scoring continues; AI-suggestion UI shows fallback banner.
- Marketplace settlement outage: Order handoff queues; Quote operations continue.
- Cell evacuation: per ADR-0248 cellular architecture; tenant traffic shifts to a different cell with shuffle sharding preserving blast radius limits.

## F. Data Integrity

Commands own local transactions. Cross-service work uses workflow-engine sagas with compensating transitions. Financial, inventory, trade, quality, and compliance documents reverse through explicit domain events (`reverse`, `archive`) rather than row deletion. Audit-chain seal events provide append-only evidence per ADR-0263.

Per-aggregate write-ahead-log strategy: PostgreSQL logical replication to ontology projection; replay-fixture per command for canonicalen-test parity.

## G. Contracts

- REST: OpenAPI 3.2.0 — `contracts/openapi-v1.yaml`.
- Events: AsyncAPI 3.1.0 — `contracts/asyncapi-v1.yaml`.
- Internal RPC: proto3 — `contracts/crm-v1.proto`.
- Naming: BNF v4.1.
- Layers: ADR-0105 13-layer enum.

## H. Wave-15A architecture trace

This section replaces the Wave-3-G stamped 90-line trace block. Each subsection is a substantive architectural decision recorded for the Wave 15A rewrite. Decisions are numbered H-1 through H-25.

### H-1. Why thirteen first-class aggregates plus one read-model aggregate

The Wave 3-G surface declared six aggregates (account-master, opportunity, quote, service-case, campaign, loyalty-ledger). The audit's UNION-coverage analysis across Salesforce + HubSpot + Dynamics established that Lead, Contact, OpportunityTeam, OpportunitySplit, Sales Cadence, Forecast, and Partner are first-class differentiators across all three counterparts. Wave 15A promotes these to first-class bounded contexts because:

- Each requires its own Cedar policy file (different action verbs, different attribute set).
- Each has its own command/event surface in the OpenAPI/AsyncAPI/proto contract.
- Each has its own SLO (e.g., Lead-conversion-latency vs Opportunity-stage-progression-latency).
- Each has its own ontology projection (joins are forbidden across aggregate roots).

The `customer-360` aggregate is a read-model only because it has no write commands; it is a denormalised projection refreshed from the eleven write-aggregates.

### H-2. Why the legacy `quote` bounded context unifies with `cpq-quote`

Salesforce's product surface separates Quote (basic) from Salesforce CPQ Quote (CPQ-grade). The audit found that Oyatie's Wave 3-G `quote` aggregate covered only the basic shape. The Wave 15A decision is to unify under the CPQ-grade aggregate (`cpq_quote_document`) because:

- A non-CPQ Quote is a degenerate CPQ Quote (no line-level configuration attributes, no constraint rules, single approval step).
- Two parallel quote aggregates would duplicate the line-item model, the approval-step model, and the document-template model.
- The Salesforce migration playbook needs to map Salesforce Quote + QuoteLineItem and Salesforce CPQ Quote + Quote Line into the same Oyatie aggregate.

The Wave 15A `cpq-quote` aggregate has a `complexity` enum {basic, configured, bundled-configured} so the simple cases pay no overhead.

### H-3. Why `customer-360` is a read-model not a write-aggregate

Treating Customer 360 as a write-aggregate would require synchronous joins across eleven write-aggregates on every customer page load. The Wave 15A decision is to materialise `customer-360` as a denormalised read-model with freshness floor 5 seconds. This pattern follows:

- Salesforce Customer 360 uses Data Cloud (Customer Data Platform) as a denormalised projection.
- HubSpot's "Company" view denormalises Contacts, Deals, Tickets at read time.
- Dynamics Customer Insights uses a Customer Data Platform substrate.

The Oyatie pattern delegates the projection refresh to ontology µservice (read-pattern projection per ADR-0145 freshness floor); `crm` reads through ontology rather than re-deriving the projection.

### H-4. Why Sales Cadence is a CRM-internal bounded context and not delegated to workflow-engine

Sales Cadence steps are tightly coupled to CRM aggregates (Lead, Contact, Opportunity). Modelling cadence steps as workflow-engine generic workflows would lose:

- Domain-specific step types (email-send, call-task, LinkedIn-touch, custom-CRM-action).
- Domain-specific exit conditions (Lead converted, Opportunity stage advanced, Contact replied).
- Domain-specific analytics (cadence step completion rate, reply rate, conversion attribution).

Wave 15A keeps cadence as a CRM-internal aggregate. Step actions that require cross-µservice orchestration (e.g., email send) delegate to substrate µservices (`mail`); the cadence engine itself remains in `crm`.

### H-5. Why Lead and Contact are separate bounded contexts despite HubSpot's unification

HubSpot's data model unifies Lead and Contact as a single Contact with `lifecycle_stage` ∈ {Subscriber, Lead, MQL, SQL, Opportunity, Customer, Evangelist}. Salesforce separates Lead SObject from Contact SObject, with explicit Lead → {Account, Contact, Opportunity} conversion. Dynamics has both Lead entity and Contact entity.

Wave 15A keeps the Salesforce-style separation as the default Oyatie shape because:

- Lead is operationally distinct (different lifecycle, different scoring, different routing).
- Lead has a one-way conversion event that is auditable and reportable.
- Lead has a different default policy set (more permissive on read, more restrictive on write).

The HubSpot-style flow is supported by setting tenant-level configuration `lead_as_contact_lifecycle = true` (per ADR-0251 pack overlay applied to operating model). In this mode, Lead aggregate is unused and Contact carries `lifecycle_stage`.

### H-6. Why OpportunityTeam and OpportunitySplit are separate aggregates

Salesforce has separate SObjects (OpportunityTeamMember, OpportunitySplit) because their lifecycle, security model, and revenue-attribution semantics differ from the parent Opportunity. Wave 15A mirrors this:

- OpportunityTeam: who has access (named-role membership; access-level on read/write).
- OpportunitySplit: who gets credit (revenue-attribution percentage).

These are separate aggregates with separate Cedar policies because team-membership changes are independent of credit-allocation changes.

### H-7. Why CPQ Configure, Price, Document, Approval are CPQ-Quote child aggregates not separate bounded contexts

Salesforce CPQ separates Configure (Product2 + Bundle), Price (Pricebook2 + Discount Schedule), Document (Quote Template), and Approval (Advanced Approvals) into separate SObjects. Wave 15A models these as children of the `cpq_quote_document` aggregate because:

- Product catalog (Salesforce Product2 + Pricebook2) is delegated to the `marketplace` + `cloud-billing-tax` substrate.
- Configuration attributes are quote-line-level data; constraint rules are quote-level policy.
- Approval chains are quote-level state.
- Document templates are tenant-level configuration with quote-level rendering.

The child-aggregate pattern keeps the write boundary tight and avoids cross-aggregate transactional joins.

### H-8. Why `forecast` is a CRM-internal bounded context

Forecast roll-up requires reading Opportunity amount + forecast_category + close_period across the entire pipeline. Modelling Forecast as an `analytics` µservice projection would force every forecast adjustment (a manager override) to round-trip through analytics. Wave 15A keeps `forecast` in `crm` because:

- Forecast adjustments are write operations with audit-chain emission, not read-only projections.
- Forecast quotas are tenant-principal-period writes with Cedar gating.
- Forecast snapshots are point-in-time captures with audit value for renewal-conversation evidence.

The deep-cut forecast analytics (trend, predictive forecast, AI overlay) delegate to `analytics` + `intelligence`.

### H-9. Why `partner` is a CRM-internal bounded context not a separate µservice

Salesforce Experience Cloud Partner Community is a separate licensed product. Dynamics has Channel Partner module. HubSpot models partners as Contacts with permissions. Wave 15A keeps `partner` in `crm` because:

- Deal Registration is a Lead/Opportunity write with partner attribution; tight coupling to CRM aggregates.
- Co-sell motion is an OpportunityTeam membership with partner role.
- Partner-user IAM scoping is delegated to `cloud-iam`.
- Partner Portal UX is delegated to the application frontend + `community` µservice.

A dedicated `partner-relationship-management` µservice would require an asymmetric read path back into `crm`; the cost of the µservice boundary exceeds its isolation benefit at Wave 15A scale.

### H-10. Why Order, Contract, Product Catalog are NOT crm bounded contexts

The audit identified `crm.order_header` DDL and `crm.contract` DDL in IP-001. Wave 15A removes these from `crm` ownership:

- Order: delegated to `cloud-billing-tax` (per ADR-0314 marketplace settlement + per ADR-0328 §D-1.85 cloud-billing service). Quote-to-Order handoff is a workflow-engine saga.
- Contract: delegated to `contract-lifecycle-management` (per ADR-0328 §D-1.88). CPQ-Quote-to-Contract handoff is a workflow-engine saga.
- Product Catalog: delegated to `marketplace` (per ADR-0314) for catalog and `cloud-billing-tax` for price-book. The Wave 15A Cedar policy + workflow saga binds the cross-µservice handoff.

This matches Salesforce's product-line split (Sales Cloud has Quote; Revenue Cloud has Order + Invoice; Contract Cloud has CLM; AppExchange has Product distribution) while keeping each Oyatie substrate single-concern per ADR-0131 + ADR-0132.

### H-11. Cedar policy organization

Per-aggregate Cedar policies live under `policy/<aggregate>-authorization.cedar`. The Wave 15A policy set is:

- `account-master-authorization.cedar`
- `contact-authorization.cedar` (new in Wave 15A)
- `lead-authorization.cedar` (new in Wave 15A)
- `opportunity-authorization.cedar`
- `opportunity-team-authorization.cedar` (new in Wave 15A)
- `opportunity-split-authorization.cedar` (new in Wave 15A)
- `sales-cadence-authorization.cedar` (new in Wave 15A)
- `cpq-quote-authorization.cedar` (supersedes `quote-authorization.cedar`)
- `forecast-authorization.cedar` (new in Wave 15A)
- `service-case-authorization.cedar`
- `campaign-authorization.cedar`
- `loyalty-ledger-authorization.cedar`
- `partner-authorization.cedar` (new in Wave 15A)
- `customer-360-authorization.cedar` (new in Wave 15A, read-only)
- `auditor-scope.cedar`
- `ci-scope.cedar`
- `abuse-defence.cedar`
- `emergency-services-bypass.cedar`
- `pack-overlay-authorization.cedar`
- `tenant-class-authorization.cedar` (new in Wave 15A — gates by tenant_class)

The Wave 15A scope authors only the high-level structure of new Cedar files; full Cedar implementation is tracked in Wave 15B per audit dimension §3.7.

### H-12. Tenant-class gating

Per ADR-0330 + the tenant-class memory, `crm` Cedar policies read `tenant_class` from the principal claim:

```
permit (
    principal,
    action,
    resource
) when {
    context.tenant_class == "paid"
} unless {
    context.tenant_class == "demo_trial" &&
    resource.aggregate_kind == "opportunity" &&
    resource.amount > 50000
};
```

The demo_trial cap (5 named principals + 100 Leads + 100 Opportunities + 1 active Cadence) is enforced as Cedar `forbid` rules per `tenant-class-authorization.cedar`.

### H-13. Per-counterpart migration adapter

Wave 15A formalizes the migration adapter layer under `src/adapter/external/`:

- `src/adapter/external/salesforce/` — Salesforce SOAP/REST/Bulk API client; SObject mapping; Person Account dual-semantic resolver; Territory2 mapper; CurrencyIsoCode mapper; QueryAll soft-deleted handler; formula-field recomputer; Shield encrypted-field-masking accepter.
- `src/adapter/external/hubspot/` — HubSpot CRM v3 API client; Contact-Company-Deal-Ticket mapper; Custom Properties extractor; Lifecycle Stage mapper.
- `src/adapter/external/dynamics/` — Dynamics Web API v9.2 client; Dataverse entity mapper; BPF stage extractor; Sales Insights score importer.

Each adapter has bidirectional sync support: read-from-counterpart for migration, write-to-counterpart for hybrid coexistence (a tenant running both Salesforce and Oyatie during cutover).

### H-14. Audit-chain seal event catalog

Wave 15A expands the seal-event catalog from six events (per the Wave 3-G manifest) to fourteen events (per the thirteen first-class aggregates plus customer-360 refresh):

- `EVT-CRM-ACCOUNT_MASTER-CHANGED`
- `EVT-CRM-CONTACT-CHANGED`
- `EVT-CRM-LEAD-CHANGED`
- `EVT-CRM-OPPORTUNITY-CHANGED`
- `EVT-CRM-OPPORTUNITY_TEAM-CHANGED`
- `EVT-CRM-OPPORTUNITY_SPLIT-CHANGED`
- `EVT-CRM-SALES_CADENCE-CHANGED`
- `EVT-CRM-CPQ_QUOTE-CHANGED`
- `EVT-CRM-FORECAST-CHANGED`
- `EVT-CRM-SERVICE_CASE-CHANGED`
- `EVT-CRM-CAMPAIGN-CHANGED`
- `EVT-CRM-LOYALTY_LEDGER-CHANGED`
- `EVT-CRM-PARTNER-CHANGED`
- `EVT-CRM-CUSTOMER_360-REFRESHED`

### H-15. Per-counterpart-migration dashboard

The `dashboards/` directory currently holds three operational dashboards. Wave 15A scope adds a per-counterpart migration dashboard variant under `dashboards/migration-from-<counterpart>.json` (deferred to Wave 15B implementation) that surfaces:

- Records migrated per aggregate.
- Records rejected per rejection reason.
- Mapping coverage (% of source-system records with a successful Oyatie projection).
- Cutover progress (% of source-system traffic moved to Oyatie).
- Per-aggregate sync drift (Oyatie vs counterpart row count).

### H-16. AI overlay handoff

The Wave 15A architecture decision is to keep all predictive scoring in the `intelligence` µservice with `crm` calling intelligence via gRPC. The handoff invariants:

- I1. Score request includes tenant_id + tenant_class + aggregate_id + model_version.
- I2. Score response includes score_value + score_bucket + feature_importance + model_explanation.
- I3. Score response is cached in `crm` with freshness floor per model_version (typically 24h for lead-score, 1h for opportunity-score).
- I4. Cedar gates the score request (a tenant can disable AI per pack overlay or per principal-level setting).
- I5. EU-AI-Act high-risk system classification triggers when score outcome enters automated routing or refusal (audited per ADR-0251).

### H-17. Per-tenant Custom Objects extensibility

Wave 15A authors the architectural shape of the Custom Object primitive (full implementation deferred to Wave 15C). The shape is:

- Tenant administrator defines a Custom Object name + Custom Field definitions via the `crm.custom_object_definition` aggregate (managed in `crm` for CRM-scoped Custom Objects; cross-µservice Custom Objects delegated to `ontology` µservice).
- Custom Object records are stored in a tenant-scoped table `crm.custom_object_record_<object_id>` (one table per Custom Object Definition) with `tenant_id` + `record_id` + JSONB `field_values`.
- Cedar policies are generated per Custom Object Definition at creation time.
- OpenAPI surface exposes dynamic endpoints `/v1/crm/{tenant_id}/custom-object/{object_name}` with field-set discovered from the definition.
- Audit-chain emits `EVT-CRM-CUSTOM_OBJECT_RECORD-CHANGED` with object_name + field_diff.

### H-18. Mobile native frontend

Per the os-support-matrix + rust-strict-only memories, Wave 15A confirms native mobile apps:

- iOS: Swift native, distributed via App Store + TestFlight; supports iOS 17+ on Apple Silicon.
- Android: Kotlin native, distributed via Google Play + Firebase App Distribution; supports Android 13+ on ARMv8.

The mobile apps connect to the same `crm` OpenAPI surface as the web frontend. Offline support uses a per-aggregate sync delta API (`/v1/crm/{tenant_id}/sync-delta`) that streams writes-since-last-sync via QUIC long-poll.

### H-19. Reports & Dashboards customer-facing primitive

Wave 15A delegates customer-facing reporting to the `analytics` µservice. The `crm`-side responsibility is:

- Emit per-aggregate metrics via ADR-0263 observability emission.
- Maintain ontology projections so `analytics` can read denormalised data.
- Provide pre-canned dashboards via `dashboards/customer-facing/<dashboard>.json` (deferred to Wave 15B implementation).

### H-20. Quote-to-Cash flow

Wave 15A defines the end-to-end Q2C saga:

1. Opportunity won → CPQ Quote accepted → `workflow-engine` Q2C saga starts.
2. Q2C saga: crm.quote.accepted → cloud-billing-tax.order.create → cloud-billing-tax.invoice.create → payments.payment.collect → cloud-billing-tax.revenue-recognition.record.
3. Compensating transitions: payment failed → invoice cancel → order cancel → quote restored to Sent state.
4. Audit-chain seals every step.
5. Per-aggregate ontology projections refresh after each step.

The saga state machine lives in `workflow-engine`; `crm` records the saga_id on the cpq-quote aggregate and emits the start command.

### H-21. Cross-µservice freshness contract

Per ADR-0145 freshness floor pattern, `crm` declares its read-path freshness contracts:

- ontology projection freshness floor: 1 second for write-aggregates, 5 seconds for customer-360.
- intelligence score freshness floor: 1 hour for opp-score, 24 hours for lead-score, no-cache for next-best-action.
- marketplace settlement reference freshness floor: 5 seconds.
- consent-graph consent freshness floor: 30 seconds (consent revocation is high-priority).

Each read path exposes the freshness floor in OpenAPI response headers (`x-freshness-floor-seconds`).

### H-22. Cellular deployment shape

Per ADR-0248 Amazon-shape cellular architecture, `crm` is cell-eligible on Tiers 0-4:

- Tier 0: regional global cells (`us-east-1`, `us-west-2`, `eu-west-1`, `ap-southeast-1` + OCI equivalents).
- Tier 1: per-tenant dedicated cells for high-value enterprise customers.
- Tier 2: per-pack regulated cells (FedRAMP-High, KR-CSAP, HIPAA-bound tenants).
- Tier 3: per-cluster shuffle-sharded cells for noisy-neighbor isolation.
- Tier 4: per-pod isolation cells for emergency break-glass operations.

Tenant cell-assignment is durable on the tenant record; cell evacuation is a workflow-engine saga.

### H-23. Mesh layering

Per ADR-0254 + `manifest.json#mesh_layering`:

- Cilium L4 routing: enabled.
- Ambient ztunnel (mTLS): enabled.
- Ambient waypoint (L7 routing): disabled by default; enabled on pack-overlay activation.
- North-south only: false (east-west service-to-service is allowed for crm → substrate handoffs).

### H-24. Bootstrap and credential isolation

Per the manifest.json keystone_adr_field_roster:

- SPIFFE identity per cell; kill-switch for bootstrap-tier surfaces.
- OpenBao dynamic secrets with TTL ≤ 60 seconds OR sidecar isolation.
- ECH (Encrypted Client Hello) advertised on tenant ingress.
- PQC X25519MLKEM768 offered when peer supports.

### H-25. Wave 15A deferral inventory

The Wave 15A rewrite establishes the architecture-of-record but defers concrete implementation in several areas to subsequent waves. Deferrals are tracked in REMEDIATION-NOTES-2026-05-21.md:

- IaC per-context refactor (`iac/<context>/`) — Wave 15B.
- Cargo.toml workspace separation — Wave 15B.
- src/ layer modules (api/, application/, kernel/, worker/, governance/, infrastructure/, integration/) — Wave 15B.
- New Cedar policy file authoring (contact, lead, opportunity-team, opportunity-split, sales-cadence, forecast, partner, customer-360, tenant-class) — Wave 15B.
- New OpenAPI/AsyncAPI/proto contract authoring for new aggregates — Wave 15B.
- New IP-026..IP-035 implementation plans for new aggregates — Wave 15B.
- Per-counterpart migration dashboards — Wave 15B.
- Per-tenant Custom Object implementation — Wave 15C.
- Per-tenant-class SLO threshold split — Wave 15C.
- Migration playbook slug rename for Dynamics — Wave 15C.

The Wave 15A scope rewrites the documentation backbone (README, ARCHITECTURE, PRD §C, competitor parity matrix) so the implementation waves have a clean target to build against.
