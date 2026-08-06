---
id: ADR-0319
title: Front Office / Middle Office / Back Office Information-Barrier Doctrine
status: Rejected
date: 2026-05-20
decision_type: architecture-doctrine
scope: tenant-office-information-barriers
owner: Oyatie governance and compliance architecture
decision_owner: council-architecture
related:
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md
  - ADR-0316-capability-tier-over-product-fragmentation.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
depends_on:
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0263-observability-emission-contract
  - ADR-0313-conglomerate-tenant-hierarchy-sovereign-children
  - ADR-0316-capability-tier-over-product-fragmentation
supersedes: []
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Keep Rejected: FO/MO/BO information barrier — keep as Rejected until regulated-finance pack prioritizes

# ADR-0319: Front Office / Middle Office / Back Office Information-Barrier Doctrine

## Status
Proposed.

This ADR is a binding doctrine for every regulated-finance tenant pack and every general-tenant pack that can process market-sensitive, customer-sensitive, advisory, trading, research, wealth, asset-management, insurance, or bank-operations data.

## Decision Summary

Oyatie will model Front Office, Middle Office, and Back Office as first-class Cedar authorization entities inside each tenant.
Oyatie will model banking and capital-markets information walls as first-class Cedar office boundaries inside each tenant.
The required office-scope entity is `Tenant::OfficeScope` with values `FRONT`, `MIDDLE`, and `BACK`.
The required office-boundary entity is `Tenant::OfficeBoundary` with values `IB`, `Trading`, `Research`, `AssetMgmt`, and `WealthMgmt`.
Cedar remains the universal gate under ADR-0243, so every office-scope and office-boundary decision is evaluated in Cedar before data, workflow, export, search, model, or event access occurs.
The tenant remains the universal scoping primitive under ADR-0244, so office scope never replaces tenant scope; it refines tenant scope.
The observability and audit-emission contract in ADR-0263 applies to every assignment, clearance, and boundary-crossing attempt.
The conglomerate and sovereign-child rules in ADR-0313 apply when an office barrier spans parent, child, advisor, or affiliate relationships.
ADR-0316 is not present in this checkout at authoring time; this ADR records a bounded forward cross-reference only and does not invent ADR-0316 content.

## Section A: Context

### A-1. Why this doctrine exists

Regulated finance does not treat all employees inside one legal customer as equivalent readers.
A universal tenant boundary is necessary, but it is not sufficient for investment banking, brokerage, research, trading, asset-management, wealth-management, insurance, treasury, and bank-operations workloads.
A single tenant can contain employees who are prohibited from sharing material nonpublic information with other employees of the same tenant.
A single tenant can contain employees who may supervise trading activity but may not receive investment-banking deal materials.
A single tenant can contain employees who may process account operations but may not consume research embargoes, M&A drafts, order-flow details, or restricted-list rationales.
A single tenant can contain service principals that are allowed to calculate controls while being barred from returning raw content to user-facing workspaces.
This ADR therefore adds office-level policy entities below the tenant level and above role-specific permissions.

### A-2. Regulatory anchor: FINRA front-running and research controls

FINRA Rule 5280 is an explicit anchor for trading ahead of research reports.
Rule 5280 matters because a research boundary cannot be treated as a simple folder permission.
Rule 5280 requires a system to know when research material exists, which principals belong to research, and which trading principals must be blocked from seeing or acting on that information before public dissemination.
Oyatie maps that requirement to `Tenant::OfficeBoundary::Research` and default-denies cross-boundary research-to-trading access without an auditable clearance.
FINRA Rule 5290 is an explicit anchor for order-entry and execution practices including front-running risk.
Rule 5290 matters because order-flow access must not leak from trading or execution surfaces into advisory, wealth, or research surfaces unless a narrow operational purpose is proven.
Oyatie maps that requirement to `Tenant::OfficeBoundary::Trading` and applies taint to order-flow derived artifacts, embeddings, dashboards, and export jobs.
FINRA Rule 3110 is an explicit anchor for supervision.
Rule 3110 matters because a compliance or supervisory function can need visibility into controls without inheriting ordinary front-office read privileges.
Oyatie maps that requirement to `MIDDLE` office supervisory personas and scoped Cedar permits that are purpose-bound, time-bound, and audit-sealed.
FINRA Rule 4530 is an explicit anchor for reportable events and supervisory reporting.
Rule 4530 matters because denied and allowed boundary attempts can become evidence in a reportable supervisory record.
Oyatie maps that requirement to dual-sealed audit events that can be reported without revealing restricted payloads.

### A-3. Regulatory anchor: FINRA trusted-contact correction

The brief references FINRA Rule 4514 as a trusted-contact anchor.
The current FINRA trusted-contact obligation is anchored in FINRA Rule 4512(a)(1)(F), which requires reasonable efforts to obtain the name and contact information for a trusted contact person for a non-institutional customer account.
FINRA Rule 4514 concerns authorization records for negotiable instruments drawn from a customer account.
This ADR cites both anchors because record authority and trusted-contact handling can both appear in wealth, branch, and account-operation workflows.
Policy packs must use Rule 4512(a)(1)(F) for trusted-contact access and Rule 4514 for negotiable-instrument authorization records.
A wealth-management user may need trusted-contact fields while a trading user does not.
A back-office operations user may need negotiable-instrument authorization metadata while an investment-banking user does not.
This distinction is a doctrine-level reason to avoid one broad tenant-internal role such as `employee`.

### A-4. Regulatory anchor: EU MAR and MiFID II

Regulation (EU) No 596/2014, the Market Abuse Regulation, defines the market-abuse control environment for inside information.
MAR Article 9 is the anchor for legitimate behaviour around inside information.
MAR Article 14 is the anchor for prohibitions on insider dealing and unlawful disclosure of inside information.
MAR Article 16 is the anchor for preventing and detecting market abuse and for reporting suspicious orders and transactions.
MAR Article 17 is the anchor for public disclosure of inside information by issuers.
MAR Article 18 is the anchor for insider lists.
Oyatie maps these MAR articles to restricted-deal labels, insider-list labels, research labels, and default-deny trading separation.
MiFID II Article 16(3) is an organizational and risk-control anchor.
MiFID II Article 16(8) is a record, security, authentication, reliability, confidentiality, and integrity anchor.
MiFID II Article 23 is a conflicts-of-interest anchor.
Oyatie maps MiFID II to office-scope assignments, boundary clearances, and controlled disclosure reports.

### A-5. Regulatory anchor: Korea FSC capital-markets controls

Korea Financial Investment Services and Capital Markets Act Articles 174 through 178 are the Korean capital-markets market-abuse anchor set for this ADR.
Article 174 is treated by the policy pack as the material-nonpublic-information anchor.
Article 175 is treated by the policy pack as a short-swing or insider-profit control anchor when applicable to the current translated text and product pack.
Article 176 is treated by the policy pack as a market-manipulation anchor.
Article 177 is treated by the policy pack as an unfair-trading or loss-compensation related anchor when applicable to the active translated text and product pack.
Article 178 is treated by the policy pack as a fraudulent, deceptive, or unfair-trading anchor when applicable to the active translated text and product pack.
The pack compiler must bind exact statutory labels from the active official or authoritative Korean legal text before enabling jurisdiction-specific legal wording in customer-facing evidence.
The architecture requirement remains stable even when translations or statutory paragraph labels are updated: inside-information, market-manipulation, and unfair-trading controls require office barriers.

### A-6. Regulatory anchor: UK, Singapore, and Australia

UK FCA SYSC 10.2 is an explicit conflicts-of-interest and information-barrier anchor.
Singapore Securities and Futures Act sections 218, 219, and 220 are explicit insider-trading and connected-person anchors for Singapore policy packs.
Singapore MAS is the regulator anchor for capital-markets and financial-advisory conduct pack enablement.
MAS published conduct sources, including the Code of Conduct for Credit Rating Agencies section 9, are Singapore information-control precedents for confidential information handling, selective disclosure controls, and capital-markets product dealing controls.
Australia Corporations Act 2001 section 912A is an explicit financial-services-licensee obligation anchor.
Australia Corporations Act 2001 section 1043A is an explicit insider-trading anchor.
ASIC Regulatory Guide 181 is an explicit conflicts-of-interest management anchor.
These anchors require the same architectural shape: know the office scope, know the boundary, deny by default, approve narrowly, and preserve evidence.

### A-7. Historical anchor: Glass-Steagall and the Volcker Rule

Glass-Steagall is a historical anchor for structural separation between commercial banking and securities activity.
The Volcker Rule, enacted through Dodd-Frank Act section 619 and implemented by US banking and markets regulators, is a modern anchor for proprietary-trading and covered-fund separation.
Oyatie does not recreate historical banking statutes as product logic.
Oyatie uses the historical pattern as an architectural primitive: regulated duties sometimes require durable separation inside or across a financial group.
ADR-0313 already applies this idea to conglomerate sovereign-child tenants and cross-child information barriers.
ADR-0319 applies the same idea inside one tenant through office scopes and office boundaries.

### A-8. Chinese Wall as an architectural primitive

This ADR uses the term Chinese Wall as a historical financial-services term for an information barrier.
The implementation term is `information barrier`.
The doctrine-level primitive is a policy-enforced boundary between business functions that can otherwise share the same tenant, identity provider, data lake, workflow engine, and audit platform.
The barrier is not only a UI filter.
The barrier is not only an application-level convention.
The barrier is not only a data-classification tag.
The barrier is a Cedar-evaluated authorization and audit primitive applied at read, write, export, search, model-ingestion, model-retrieval, workflow transition, and event-subscription time.

## Section B: Decision

### B-1. Cedar entity-type extensions

Oyatie adds `Tenant::OfficeScope` as a required Cedar entity type.
`Tenant::OfficeScope` has exactly three doctrine values: `FRONT`, `MIDDLE`, and `BACK`.
Oyatie adds `Tenant::OfficeBoundary` as a required Cedar entity type.
`Tenant::OfficeBoundary` has exactly five doctrine values: `IB`, `Trading`, `Research`, `AssetMgmt`, and `WealthMgmt`.
Oyatie adds `Tenant::OfficeScopeAssignment` as the audited relation between principal, role, service principal, office scope, and effective interval.
Oyatie adds `Tenant::OfficeBoundaryClearance` as the audited exception relation between principal, source boundary, target boundary, action, purpose, approver, and expiry.
Oyatie adds `Tenant::InformationBarrierTaint` as the data-origin and derived-artifact label for boundary-sensitive information.
Oyatie adds `Tenant::RestrictedDeal` as the deal, mandate, list, or engagement object that activates a per-deal barrier.
Oyatie adds `Tenant::AdvisorRelationshipBarrier` as the cross-tenant link used when an advisor relationship could transmit restricted information between tenants.

### B-2. Default-deny rule

The default rule is denial across every office boundary.
A principal assigned to `FRONT` cannot read, search, export, summarize, or subscribe to resources tainted with a boundary outside the principal clearance set.
A principal assigned to `MIDDLE` can receive supervisory evidence only when the action has a compliance, risk, legal, or operational-control purpose.
A principal assigned to `BACK` can perform settlement, reconciliation, account, records, technology, or operations work without gaining front-office deal, research, or trading content.
A service principal inherits the most restrictive combination of its declared service role, office scope, boundary set, tenant, product pack, and runtime purpose.
A model, search index, embedding store, export worker, event subscriber, or workflow automation is a principal for barrier purposes.

### B-3. Explicit exception rule

Cross-boundary access requires an explicit `Tenant::OfficeBoundaryClearance`.
The clearance must identify the principal, source boundary, target boundary, action class, data class, purpose, approver, effective interval, and revocation state.
The clearance must be evaluated by Cedar on every request.
The clearance must be dual-sealed into the audit chain before it can be used.
The clearance must expire automatically.
The clearance must not grant wildcard access to all boundaries.
The clearance must not downgrade tenant scope.
The clearance must not bypass ADR-0243, ADR-0244, ADR-0263, or ADR-0313.

## Section C: Consequences

### C-1. Consequence dimension 1: authorization

All tenant-internal regulated-finance access is now a compound authorization question.
The compound question includes tenant, audience type, principal, role, office scope, office boundary, resource taint, product pack, jurisdiction, purpose, and clearance state.
This increases policy specificity and reduces accidental lateral access inside a tenant.
The cost is a larger Cedar entity graph and a stricter policy-authoring discipline.
The acceptance criterion is that default-deny remains easy to prove and narrow clearance remains auditable.

### C-2. Consequence dimension 2: data model

The data model gains durable office-scope assignment records.
The data model gains durable boundary-clearance records.
The data model gains boundary-attempt audit rows and event classes.
The data model gains restricted-deal and advisory-relationship barrier labels.
The cost is migration work across services that previously treated tenant membership as sufficient.
The acceptance criterion is that every service can declare whether each role belongs to FRONT, MIDDLE, BACK, or no-regulated-office by pack.

### C-3. Consequence dimension 3: auditability

Every assignment, clearance, denial, allowance, revocation, policy-pack overlay, and cross-tenant advisor attempt becomes an audit event.
ADR-0263 structured logging applies to these events.
ADR-0313 dual-sealing applies when parent, child, affiliate, or advisor tenants are implicated.
Within one tenant, dual-sealing writes to the principal office stream and the resource office-boundary stream.
The cost is higher event volume.
The acceptance criterion is that denied attempts are preserved with metadata and without restricted payload leakage.

### C-4. Consequence dimension 4: operational controls

Compliance, risk, security, and bank operations receive explicit middle-office and back-office paths instead of broad admin bypasses.
Operational break-glass becomes a clearance record, not an untracked override.
Operations dashboards must distinguish barrier posture from service health.
The cost is more runbook specificity and more access-review work.
The acceptance criterion is that an operator can explain why an allowed cross-boundary request was legitimate without viewing the restricted payload in the audit summary.

### C-5. Consequence dimension 5: product-pack overlays

Industry packs can tighten office mapping without weakening the base doctrine.
Investment-banking packs default to the strictest IB, Trading, Research, and AssetMgmt separation.
Investment-management packs focus on AssetMgmt, Trading, Research, and client-mandate conflicts.
Insurance packs focus on underwriting, claims, actuarial, broker, and investment-account barriers while still using FRONT, MIDDLE, and BACK.
General-tenant packs can keep the entities dormant until a regulated workflow activates them.
The acceptance criterion is that a pack overlay never changes the doctrine enum set and never makes a default-deny path permissive.

### C-6. Consequence dimension 6: migration and developer experience

Every microservice must declare role-to-office mappings during migration.
Every microservice must emit assignment, clearance, attempt, and denial event classes through the audit-chain lane when it touches protected workflows.
Every test harness that previously asserted tenant-only authorization must add office-boundary denial cases for regulated packs.
The cost is a broad but mechanical migration.
The acceptance criterion is a service-by-service declaration plus policy tests that prove tenant membership alone is insufficient for restricted resources.

## Section D: Detailed Mechanics

### D-0. Primitive precedent matrix

#### P01. Tenant::OfficeScope
- Primitive purpose: classifies a principal, role, session, or workload as FRONT, MIDDLE, or BACK office within one tenant.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

#### P02. Tenant::OfficeBoundary
- Primitive purpose: classifies the business information wall as IB, Trading, Research, AssetMgmt, or WealthMgmt.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

#### P03. Tenant::OfficeScopeAssignment
- Primitive purpose: records who assigned an employee or service role to an office scope and why.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

#### P04. Tenant::OfficeBoundaryClearance
- Primitive purpose: records a limited, revocable clearance allowing a named cross-boundary action.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

#### P05. Tenant::InformationBarrierTaint
- Primitive purpose: marks data, events, embeddings, reports, and exports with origin boundary labels.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

#### P06. Tenant::RestrictedDeal
- Primitive purpose: binds a deal, mandate, advisory engagement, or research restriction to a barrier set.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

#### P07. Tenant::OfficePackOverlay
- Primitive purpose: maps industry packs to stricter or lighter office-scope requirements without weakening core denial.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

#### P08. Tenant::BoundaryAttemptAuditEvent
- Primitive purpose: captures allowed and denied cross-boundary attempts with dual-sealed audit evidence.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

#### P09. Tenant::AdvisorRelationshipBarrier
- Primitive purpose: prevents cross-tenant disclosure through advisor, parent, or conglomerate relationships.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

#### P10. Tenant::RegulatoryAnchor
- Primitive purpose: links policy decisions to the applicable jurisdiction and rule anchor at evaluation time.
- Regulatory precedent 1: FINRA Rule 3110 requires a supervisory system; this primitive gives supervision a typed control point instead of a broad administrator bypass.
- Regulatory precedent 2: MiFID II Article 16(8) requires reliable, secure, confidential records; this primitive makes office labels explicit and auditable.
- Regulatory precedent 3: MAR Article 16 requires prevention and detection of market abuse; this primitive gives market-abuse detection a policy-visible boundary.
- Regulatory precedent 4: FCA SYSC 10.2 requires conflicts controls and supports information-barrier mechanics.
- Hyperscaler precedent 1: AWS Verified Permissions and Cedar separate authorization policy from application code, which matches ADR-0243.
- Hyperscaler precedent 2: Google Cloud IAM Conditions demonstrate contextual access decisions based on attributes and request context.
- Hyperscaler precedent 3: Azure Privileged Identity Management demonstrates time-bound privileged activation with approval and audit evidence.
- Hyperscaler precedent 4: AWS CloudTrail demonstrates immutable-style API decision evidence as an operational control reference.
- Oyatie binding: the primitive is modeled in Cedar, persisted in Postgres where durable, and emitted through audit-chain events under ADR-0263.
- Denial binding: lack of this primitive, stale primitive state, revoked primitive state, or missing jurisdiction anchor resolves to denial.
- Pack binding: industry packs may add attributes and stricter checks, but they may not remove this primitive from regulated-finance evaluations.
- Test binding: every primitive receives one allow, one deny, one expiry, one revocation, and one audit-shape test in its owning service.

### D-1. Cedar entity types

The Cedar schema extension is doctrine-owned and pack-applied.
The schema extension is additive to the tenant graph defined by ADR-0244.
The schema extension is enforced through ADR-0243 as the universal authorization gate.
The schema extension is observable through ADR-0263 audit and log events.
The schema extension is compatible with ADR-0313 sovereign-child tenant hierarchy.

```cedar
entity Tenant::OfficeScope = {
  tenant_id: String,
  scope_name: String, // FRONT | MIDDLE | BACK
  jurisdiction_set: Set<String>,
  policy_pack: String,
  effective_at: Long,
  revoked_at: Long,
};

entity Tenant::OfficeBoundary = {
  tenant_id: String,
  boundary_name: String, // IB | Trading | Research | AssetMgmt | WealthMgmt
  restricted: Bool,
  jurisdiction_set: Set<String>,
  policy_pack: String,
  audit_stream_id: String,
};

entity Tenant::OfficeScopeAssignment = {
  tenant_id: String,
  principal_id: String,
  office_scope: Tenant::OfficeScope,
  office_boundaries: Set<Tenant::OfficeBoundary>,
  assigned_by: String,
  purpose: String,
  effective_at: Long,
  expires_at: Long,
  revoked_at: Long,
  audit_event_id: String,
};

entity Tenant::OfficeBoundaryClearance = {
  tenant_id: String,
  principal_id: String,
  source_boundary: Tenant::OfficeBoundary,
  target_boundary: Tenant::OfficeBoundary,
  action_class: String,
  data_class: String,
  purpose: String,
  approved_by: String,
  effective_at: Long,
  expires_at: Long,
  revoked_at: Long,
  dual_seal_event_id: String,
};

entity Tenant::InformationBarrierTaint = {
  tenant_id: String,
  origin_boundary: Tenant::OfficeBoundary,
  restricted_deal_id: String,
  source_event_id: String,
  derived_from_event_ids: Set<String>,
  can_export: Bool,
  can_index: Bool,
  can_train_model: Bool,
};
```

`Tenant::OfficeScope::FRONT` covers client-facing revenue, advisory, sales, relationship, trading, investment-banking, research, asset-management, and wealth-management activity as mapped by boundary.
`Tenant::OfficeScope::MIDDLE` covers risk, compliance, legal, treasury-control, finance-control, and supervisory review activity.
`Tenant::OfficeScope::BACK` covers operations, settlement, reconciliation, payments operations, account maintenance, records, technology operations, and platform administration that is not entitled to restricted front-office content.
`Tenant::OfficeBoundary::IB` covers investment-banking advisory, M&A, underwriting, financing, pitch, mandate, fairness opinion, and confidential committee material.
`Tenant::OfficeBoundary::Trading` covers trading, execution, order flow, market-making, desk positions, strategy, algorithm, and restricted market activity.
`Tenant::OfficeBoundary::Research` covers research drafts, ratings, target prices, embargoed publications, review notes, analyst models, and pre-publication distribution lists.
`Tenant::OfficeBoundary::AssetMgmt` covers fund positions, portfolio decisions, mandate restrictions, allocation decisions, and asset-management strategy.
`Tenant::OfficeBoundary::WealthMgmt` covers wealth-advisory customer context, trusted-contact information, suitability context, and branch-supervision context.

### D-2. Per-employee office-scope assignment and audit chain

Every employee principal in a regulated pack must have zero or more office-scope assignments.
Zero assignments means the principal has no regulated-office access.
A principal may have multiple assignments only when each assignment is purpose-bound and non-overlapping at the boundary level.
A front-office assignment does not imply all front-office boundaries.
A middle-office assignment does not imply unrestricted content access.
A back-office assignment does not imply operational break-glass.
Assignments are made by a role that already has authority to administer the target pack and tenant.
Assignments require a reason code.
Assignments require an effective timestamp.
Assignments require an expiry or review timestamp for regulated packs.
Assignments require a link to a personnel, HR, directory, service-ownership, or governance source of authority.
Assignments require an `OfficeScopeAssignmentCreated` audit event.
Assignment updates require an `OfficeScopeAssignmentChanged` audit event.
Assignment revocations require an `OfficeScopeAssignmentRevoked` audit event.
Assignment decisions are dual-sealed to the principal assignment stream and the tenant governance stream.
If an assignment crosses an ADR-0313 parent-child boundary, the assignment is also sealed to the child tenant and parent tenant audit streams.
Assignment events must include the Cedar policy version used to validate the assignment authority.
Assignment events must include the product-pack overlay version.
Assignment events must include the jurisdiction anchor set.
Assignment events must not include restricted deal payloads.
Assignment events must include stable identifiers for the principal, approver, tenant, scope, and boundary.
Assignment events must be queryable by compliance without exposing restricted content.

Assignment creation flow:
1. Receive request from identity, HR, governance, or service-owner workflow.
2. Resolve tenant from ADR-0244 tenant scope.
3. Resolve `audience_type` from ADR-0244 and reject incompatible external audience assignments.
4. Resolve office-scope value.
5. Resolve boundary values.
6. Evaluate assignment authority in Cedar under ADR-0243.
7. Persist assignment row only after Cedar allow.
8. Emit audit event through ADR-0263 contract.
9. Dual-seal event to assignment and governance streams.
10. Publish cache-invalidation event to policy-evaluation edges.

### D-3. Cedar permit format for cross-boundary scoped access

A cross-boundary permit must be explicit.
A cross-boundary permit must be narrow.
A cross-boundary permit must be time-boxed.
A cross-boundary permit must be purpose-bound.
A cross-boundary permit must be resource-class bound.
A cross-boundary permit must be auditable before use.
A cross-boundary permit must not rely on job title alone.
A managing director can receive a cross-boundary clearance only when the relevant deal, committee, conflict review, or legal-supervision workflow approves it.
A clearance for one M&A deal does not grant visibility into another M&A deal.
A clearance for an IB committee does not grant trading-desk order-flow access.
A clearance for supervisory review does not grant front-office workflow participation.

```cedar
permit(
  principal,
  action in [Tenant::Action::"ReadRestrictedDealSummary", Tenant::Action::"ReviewConflictMemo"],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  context.office_scope == Tenant::OfficeScope::"MIDDLE" &&
  context.clearance.source_boundary == Tenant::OfficeBoundary::"IB" &&
  context.clearance.target_boundary == resource.office_boundary &&
  context.clearance.purpose in ["legal_review", "compliance_review", "conflict_committee"] &&
  context.clearance.expires_at > context.request_time &&
  context.clearance.revoked_at == 0 &&
  context.clearance.dual_sealed == true &&
  resource.restricted_deal_id == context.clearance.restricted_deal_id
};
```

The permit shape is intentionally dependent on context attributes, assignment rows, and clearance rows.
The permit shape does not allow a role called `ManagingDirector` to bypass the barrier.
The permit shape allows a managing director only when the principal also has a sealed clearance with the correct purpose and expiry.
The permit shape is compatible with AWS Verified Permissions style Cedar policy deployment.
The permit shape is compatible with a hyperscaler control-plane approach where policy is centrally distributed and evaluated at every service edge.

### D-4. Chinese-Wall enforcement and auditable exceptions

The implementation term is information barrier.
The financial-services historical term Chinese Wall appears only as a domain synonym.
The core enforcement pairs are IB to Trading, Trading to Research, IB to AssetMgmt, IB to WealthMgmt when deal-specific information is present, and Research to Trading before public release.
Default-deny applies to every listed pair.
Default-deny also applies to any pack-defined pair that is stricter than this base doctrine.

```cedar
forbid(
  principal,
  action,
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  resource.has_information_barrier_taint == true &&
  !(context.has_valid_office_boundary_clearance == true)
};

forbid(
  principal,
  action in [Tenant::Action::"Read", Tenant::Action::"Search", Tenant::Action::"Export", Tenant::Action::"Subscribe", Tenant::Action::"RetrieveEmbedding"],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  context.principal_boundary == Tenant::OfficeBoundary::"Trading" &&
  resource.office_boundary == Tenant::OfficeBoundary::"IB" &&
  !(context.has_valid_office_boundary_clearance == true)
};

forbid(
  principal,
  action in [Tenant::Action::"Read", Tenant::Action::"Search", Tenant::Action::"Export", Tenant::Action::"Subscribe", Tenant::Action::"RetrieveEmbedding"],
  resource
)
when {
  principal.tenant_id == resource.tenant_id &&
  context.principal_boundary == Tenant::OfficeBoundary::"Trading" &&
  resource.office_boundary == Tenant::OfficeBoundary::"Research" &&
  resource.research_publication_state != "published"
};
```

Auditable exceptions require a clearance row.
Auditable exceptions require an allow event and a boundary-attempt event.
Auditable exceptions require a denial event when any condition fails.
Auditable exceptions require the product pack to state the regulatory purpose.
Auditable exceptions require revocation propagation to policy caches.
Auditable exceptions require a review report that does not disclose restricted payload by default.

### D-5. Per-pack overlays

Per-pack overlays are policy overlays, not new doctrine enums.
The base doctrine enum set remains `FRONT`, `MIDDLE`, `BACK`, `IB`, `Trading`, `Research`, `AssetMgmt`, and `WealthMgmt`.
IS-Banking overlay strengthens IB, Trading, Research, and WealthMgmt barriers.
IS-Investment-Mgmt overlay strengthens AssetMgmt, Trading, Research, and client-mandate barriers.
IS-Insurance overlay maps underwriting, claims, actuarial, distribution, investment-accounting, and reinsurance roles into the same office-scope lattice.
General-tenant overlay keeps the entities available and dormant until a regulated workflow, restricted deal, insider-list workflow, or industry pack activates them.

IS-Banking overlay requirements:
- Investment-banking deal rooms are `FRONT` plus `IB`.
- Trading desks are `FRONT` plus `Trading`.
- Research teams are `FRONT` plus `Research`.
- Bank risk managers are `MIDDLE` with supervisory purpose only.
- Bank compliance officers are `MIDDLE` with compliance purpose only.
- Bank operations officers are `BACK` with operational purpose only.
- M&A deal restricted lists create `Tenant::RestrictedDeal` entities.
- Research embargoes create `Tenant::InformationBarrierTaint` entities.
- Volcker controls can mark proprietary-trading sensitive data as `Trading` taint.
- Glass-Steagall-style separations can map to ADR-0313 child-tenant barriers when structural separation is required.

IS-Investment-Mgmt overlay requirements:
- Portfolio managers are `FRONT` plus `AssetMgmt`.
- Traders are `FRONT` plus `Trading`.
- Research analysts are `FRONT` plus `Research`.
- Compliance analysts are `MIDDLE` with review-only purpose.
- Fund operations are `BACK` with reconciliation and settlement purpose.
- Allocation decisions carry `AssetMgmt` taint.
- Client mandates carry `WealthMgmt` or `AssetMgmt` taint depending on pack context.
- Cross-fund conflicts require a clearance or denial record.
- Best-execution reviews do not grant raw IB deal content.
- Model-training from portfolio data is denied unless the taint explicitly permits it.

IS-Insurance overlay requirements:
- Underwriting can be `FRONT` when it is customer-facing or distribution-facing.
- Claims operations can be `BACK` when it processes records and settlement.
- Actuarial risk can be `MIDDLE` when it performs control, capital, and reserve review.
- Investment-accounting can be `BACK` or `MIDDLE` depending on pack configuration.
- Broker relationships can activate `WealthMgmt` style customer-information separation.
- Reinsurance treaties can activate restricted-deal labels.
- Claims fraud investigation requires purpose-bound clearance.
- Customer health or personal data remains governed by privacy policy in addition to this ADR.
- Insurance pack overlays may add attributes but cannot weaken default-deny.
- Insurance audit events use the same dual-seal mechanics as banking and investment management.

General-tenant overlay requirements:
- Office entities may exist without granting any regulated-finance privilege.
- Non-regulated workflows can receive no-office assignments.
- A tenant can enable office barriers before enabling an industry pack.
- A tenant can use office barriers to separate legal, HR, finance, and security data when policy demands it.
- General packs cannot use office barriers to bypass ADR-0244 tenant scope.
- General packs cannot use office barriers to bypass ADR-0243 Cedar evaluation.
- General packs emit the same audit event classes when barriers activate.
- General packs can map external auditors to `MIDDLE` review-only scopes with expiry.
- General packs deny model training from tainted data by default.
- General packs preserve an upgrade path to regulated finance packs.

### D-6. Audit-chain dual sealing on every boundary crossing

Every boundary crossing attempt emits evidence.
The attempt can be allowed.
The attempt can be denied.
The attempt can be blocked before payload retrieval.
The attempt can be blocked before model retrieval.
The attempt can be blocked before export generation.
All cases are audit-significant.

Dual sealing inside one tenant means two audit commitments are written.
The first commitment is written to the principal office-scope stream.
The second commitment is written to the resource office-boundary stream.
If the resource also belongs to a restricted deal, a third pointer is written to the restricted-deal evidence index.
If the event crosses parent, child, affiliate, or advisor tenants, ADR-0313 dual-sealing rules add tenant-side commitments.

The boundary attempt event has these mandatory fields:
- `event_class`.
- `event_id`.
- `tenant_id`.
- `principal_id`.
- `principal_office_scope`.
- `principal_office_boundary_set`.
- `resource_id`.
- `resource_office_boundary`.
- `restricted_deal_id`.
- `action_class`.
- `purpose`.
- `decision`.
- `cedar_policy_version`.
- `cedar_schema_version`.
- `policy_pack_version`.
- `jurisdiction_anchor_set`.
- `clearance_id`.
- `clearance_expiry`.
- `denial_reason_code`.
- `principal_stream_commitment`.
- `boundary_stream_commitment`.
- `payload_hash`.
- `redaction_profile`.
- `trace_id`.
- `request_time`.

Audit events must not log raw restricted payload.
Audit events may log payload hash, schema hash, row hash, and redacted descriptors.
Audit events must preserve enough data for FINRA Rule 4530 reporting, MAR Article 16 review, MiFID II Article 16 record obligations, FCA SYSC 10.2 review, and ASIC conflicts review.
Audit events must be reconstructable without granting the investigator unrestricted access to the underlying deal, trade, research, wealth, or asset-management content.

### D-7. Per-jurisdiction anchors

- Anchor `US-FINRA-5280`: FINRA Rule 5280.
  - Control purpose: trading ahead of research reports.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `US-FINRA-5290`: FINRA Rule 5290.
  - Control purpose: order entry and execution practices including front-running risk.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `US-FINRA-3110`: FINRA Rule 3110.
  - Control purpose: supervision and control system requirements.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `US-FINRA-4512`: FINRA Rule 4512(a)(1)(F).
  - Control purpose: trusted-contact reasonable-efforts requirement for non-institutional accounts.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `US-FINRA-4514`: FINRA Rule 4514.
  - Control purpose: authorization records for negotiable instruments; cited here to correct the common trusted-contact mismatch.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `US-FINRA-4530`: FINRA Rule 4530.
  - Control purpose: reporting requirements for complaints, violations, and findings.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `US-SEC-619`: Dodd-Frank Act section 619 and Volcker implementing rules.
  - Control purpose: covered-fund and proprietary-trading separation.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `EU-MAR-9`: Regulation (EU) No 596/2014 Article 9.
  - Control purpose: legitimate behaviour around inside information.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `EU-MAR-14`: Regulation (EU) No 596/2014 Article 14.
  - Control purpose: prohibition of insider dealing and unlawful disclosure.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `EU-MAR-16`: Regulation (EU) No 596/2014 Article 16.
  - Control purpose: prevention and detection of market abuse.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `EU-MAR-18`: Regulation (EU) No 596/2014 Article 18.
  - Control purpose: insider lists.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `EU-MIFID-16`: MiFID II Article 16(3) and Article 16(8).
  - Control purpose: organizational, security, record, and confidentiality controls.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `EU-MIFID-23`: MiFID II Article 23.
  - Control purpose: conflicts of interest.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `KR-FSCMA-174`: Korea FSCMA Articles 174 through 178.
  - Control purpose: capital-markets market-abuse and unfair-trading anchor set.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `UK-FCA-SYSC-10`: FCA SYSC 10.2.
  - Control purpose: conflicts of interest and Chinese-wall style information barriers.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `SG-SFA-218`: Singapore Securities and Futures Act sections 218, 219, and 220.
  - Control purpose: insider trading and connected-person controls.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `SG-MAS-CRA-9`: Monetary Authority of Singapore Code of Conduct for Credit Rating Agencies section 9.
  - Control purpose: confidential information handling, selective disclosure control, and capital-markets product dealing controls.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when a Singapore capital-markets pack uses MAS conduct guidance as a confidential-information control reference.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why Singapore MAS conduct controls evaluated.

- Anchor `AU-CA-912A`: Australia Corporations Act 2001 section 912A.
  - Control purpose: financial-services licensee obligations.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `AU-CA-1043A`: Australia Corporations Act 2001 section 1043A.
  - Control purpose: insider trading prohibition.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

- Anchor `AU-ASIC-RG181`: ASIC Regulatory Guide 181.
  - Control purpose: conflicts-of-interest management.
  - Cedar implication: include this anchor in `context.jurisdiction_anchor_set` when the pack, tenant, customer, trade, account, or resource jurisdiction requires it.
  - Audit implication: copy the anchor code into the boundary-attempt event so compliance can prove why the barrier evaluated.

### D-8. Bank persona office assignments

The ADR persona roster uses these regulated-bank assignments.
Bank Operations Officer is `BACK`.
Bank Operations Officer default purpose is operations, settlement, reconciliation, account maintenance, records, and exception processing.
Bank Operations Officer default boundary access is none for `IB`, none for `Trading`, none for `Research`, none for `AssetMgmt`, and purpose-limited for `WealthMgmt` only when branch or account operations require it.
Bank Operations Officer can see workflow state and redacted control evidence without seeing restricted deal, trade, or research payload.
Bank Risk Manager is `MIDDLE`.
Bank Risk Manager default purpose is risk review, credit risk, market risk, liquidity risk, operational risk, control monitoring, and supervisory escalation.
Bank Risk Manager default boundary access is review-only through clearance; no raw IB, Trading, Research, AssetMgmt, or WealthMgmt payload is allowed by title alone.
Bank Risk Manager can receive risk metrics, aggregated exposure, and redacted exception evidence when the pack allows it.
Bank Compliance Officer is `MIDDLE`.
Bank Compliance Officer default purpose is compliance review, surveillance, reporting, conduct review, suspicious activity review, conflicts review, and regulatory evidence preparation.
Bank Compliance Officer default boundary access is purpose-bound clearance with dual-sealed audit on every attempt.
Bank Compliance Officer can prepare FINRA Rule 4530, MAR Article 16, FCA SYSC 10.2, MAS, or ASIC evidence without unbounded payload access.
A person holding more than one persona must receive separate assignment rows.
Persona conflicts are resolved by denial until a specific clearance permits the requested action.

### D-9. Per-deal barrier

A restricted M&A deal is represented as `Tenant::RestrictedDeal`.
The restricted deal has a tenant id.
The restricted deal has a deal id.
The restricted deal has an originating office boundary, usually `IB`.
The restricted deal has a restricted participant set.
The restricted deal has a senior-leadership exception set.
The restricted deal has a compliance and legal review set.
The restricted deal has effective and release timestamps.
The restricted deal has a restricted-list reason code.
The restricted deal has a jurisdiction anchor set.
The restricted deal has an audit stream id.

Per-deal rule: M&A deal materials are visible to approved IB deal team members and approved senior leadership only.
Per-deal rule: Trading does not receive M&A deal materials.
Per-deal rule: Research does not receive M&A deal materials before the restricted release policy permits it.
Per-deal rule: AssetMgmt does not receive M&A deal materials through portfolio tooling.
Per-deal rule: WealthMgmt does not receive M&A deal materials through customer-advice tooling.
Per-deal rule: compliance and legal can receive redacted review evidence or explicit clearance-based payload access.
Per-deal rule: senior leadership clearance must name the specific deal and purpose.
Per-deal rule: a committee pack can allow a managing director to see a conflict memo without granting the underlying data room.
Per-deal rule: all denied attempts from Trading are retained as boundary-attempt audit events.
Per-deal rule: all model retrieval, search, export, and subscription attempts are subject to the same deal barrier.

### D-10. Cross-tenant barrier and advisor relationships

ADR-0313 Section D-6 governs cross-child information barriers for conglomerate and sovereign-child tenant structures.
ADR-0319 extends the same evidence pattern to advisor relationships that can transmit restricted information across otherwise separate tenants.
An advisor tenant can be given a clearance for a restricted engagement.
The advisor clearance must identify the client tenant, advisor tenant, restricted deal or engagement, office scope, office boundary, purpose, effective interval, and revocation state.
The advisor clearance must not become a broad cross-tenant data grant.
The advisor clearance must not allow one client tenant to receive another client tenant restricted material.
The advisor clearance must not allow a parent tenant to pierce a sovereign child barrier without ADR-0313 authority.
The advisor clearance must not allow one advisory team to leak IB content into Trading, Research, AssetMgmt, or WealthMgmt.
The advisor clearance must be dual-sealed to both tenant audit chains when cross-tenant access occurs.
The advisor clearance must include `origin_tenant_id`, `advisor_tenant_id`, and `information_barrier_set` in audit evidence.

Cross-tenant default-deny cases:
- Advisor A serving Client X cannot use Client X M&A materials in Client Y work.
- Parent P cannot inspect Child C investment-banking deal materials through corporate administration APIs.
- Retail banking child cannot access investment-banking child restricted-list rationales unless ADR-0313 and this ADR both allow the specific purpose.
- Trading affiliate cannot subscribe to IB advisory room events through shared event infrastructure.
- Research affiliate cannot retrieve embargoed research drafts through shared search infrastructure.
- Asset-management affiliate cannot train a model on IB restricted-deal data without explicit taint permission.
- Wealth-management affiliate cannot use trusted-contact data outside customer-protection purposes.
- Platform operations cannot export restricted cross-tenant evidence without redaction profile and clearance.

## Section E: Implementation Footprint

### E-1. Cedar action classes

The following action classes are doctrine-owned and can be specialized by product packs.
`Tenant::Action::ReadOfficeScopedResource` gates ordinary read access.
`Tenant::Action::WriteOfficeScopedResource` gates ordinary write access.
`Tenant::Action::SearchOfficeScopedResource` gates search and discovery.
`Tenant::Action::ExportOfficeScopedResource` gates file, report, and data export.
`Tenant::Action::SubscribeOfficeScopedEvent` gates event subscription.
`Tenant::Action::RetrieveOfficeScopedEmbedding` gates vector, RAG, model, and search retrieval.
`Tenant::Action::TrainFromOfficeScopedResource` gates model-training and fine-tuning inputs.
`Tenant::Action::RequestBoundaryCrossing` gates request creation.
`Tenant::Action::ApproveBoundaryCrossing` gates approval authority.
`Tenant::Action::RevokeBoundaryCrossing` gates revocation authority.
`Tenant::Action::AttachDealRestriction` gates restricted-deal activation.
`Tenant::Action::ReleaseDealRestriction` gates restricted-deal release.
`Tenant::Action::ReviewBoundaryAttemptAudit` gates compliance review of boundary events.

### E-2. Cedar evaluation sequence

1. Resolve tenant scope under ADR-0244.
2. Resolve audience type under ADR-0244.
3. Resolve principal identity, role, service role, and session purpose.
4. Resolve principal office-scope assignments.
5. Resolve resource office-boundary taints.
6. Resolve restricted-deal labels.
7. Resolve cross-tenant advisor barriers under ADR-0313.
8. Resolve product-pack overlay.
9. Resolve jurisdiction anchor set.
10. Resolve active clearances.
11. Evaluate Cedar under ADR-0243.
12. Deny when any required context is missing.
13. Emit `OfficeBoundaryAttemptEvaluated`.
14. Dual-seal the event.
15. Return only the allowed payload shape.

### E-3. Postgres DDL

The DDL below is the doctrine-level shape.
Services may add generated columns, partitioning, indexes, or pack-specific reference tables without changing the semantics.

```sql
CREATE TYPE office_scope_name AS ENUM ('FRONT', 'MIDDLE', 'BACK');

CREATE TYPE office_boundary_name AS ENUM ('IB', 'Trading', 'Research', 'AssetMgmt', 'WealthMgmt');

CREATE TYPE office_boundary_decision AS ENUM ('ALLOW', 'DENY', 'ERROR');

CREATE TABLE office_scope_assignments (
    id uuid PRIMARY KEY,
    tenant_id uuid NOT NULL,
    principal_id uuid NOT NULL,
    principal_kind text NOT NULL,
    office_scope office_scope_name NOT NULL,
    office_boundaries office_boundary_name[] NOT NULL DEFAULT ARRAY[]::office_boundary_name[],
    audience_type text NOT NULL,
    policy_pack text NOT NULL,
    jurisdiction_anchor_set text[] NOT NULL DEFAULT ARRAY[]::text[],
    assignment_reason_code text NOT NULL,
    source_authority_ref text NOT NULL,
    assigned_by uuid NOT NULL,
    approved_by uuid,
    effective_at timestamptz NOT NULL,
    review_after timestamptz NOT NULL,
    expires_at timestamptz,
    revoked_at timestamptz,
    revocation_reason_code text,
    cedar_policy_version text NOT NULL,
    cedar_schema_version text NOT NULL,
    audit_chain_id uuid NOT NULL,
    assignment_event_id uuid NOT NULL,
    dual_seal_event_id uuid NOT NULL,
    row_hash bytea NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (review_after > effective_at),
    CHECK (expires_at IS NULL OR expires_at > effective_at),
    CHECK (revoked_at IS NULL OR revoked_at >= effective_at)
);

CREATE UNIQUE INDEX office_scope_assignments_active_uniq
    ON office_scope_assignments (tenant_id, principal_id, office_scope, policy_pack)
    WHERE revoked_at IS NULL;

CREATE INDEX office_scope_assignments_principal_idx
    ON office_scope_assignments (tenant_id, principal_id, effective_at DESC);

CREATE INDEX office_scope_assignments_review_idx
    ON office_scope_assignments (tenant_id, review_after)
    WHERE revoked_at IS NULL;

CREATE TABLE office_boundary_clearances (
    id uuid PRIMARY KEY,
    tenant_id uuid NOT NULL,
    principal_id uuid NOT NULL,
    source_boundary office_boundary_name NOT NULL,
    target_boundary office_boundary_name NOT NULL,
    action_class text NOT NULL,
    data_class text NOT NULL,
    purpose text NOT NULL,
    restricted_deal_id uuid,
    advisor_relationship_id uuid,
    requested_by uuid NOT NULL,
    approved_by uuid NOT NULL,
    approval_ticket text NOT NULL,
    effective_at timestamptz NOT NULL,
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    revocation_reason_code text,
    jurisdiction_anchor_set text[] NOT NULL DEFAULT ARRAY[]::text[],
    cedar_policy_version text NOT NULL,
    cedar_schema_version text NOT NULL,
    clearance_event_id uuid NOT NULL,
    dual_seal_event_id uuid NOT NULL,
    row_hash bytea NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (source_boundary <> target_boundary),
    CHECK (expires_at > effective_at),
    CHECK (revoked_at IS NULL OR revoked_at >= effective_at)
);

CREATE INDEX office_boundary_clearances_active_idx
    ON office_boundary_clearances (tenant_id, principal_id, target_boundary, expires_at)
    WHERE revoked_at IS NULL;

CREATE INDEX office_boundary_clearances_deal_idx
    ON office_boundary_clearances (tenant_id, restricted_deal_id)
    WHERE restricted_deal_id IS NOT NULL;

CREATE TABLE office_boundary_attempts (
    id uuid PRIMARY KEY,
    tenant_id uuid NOT NULL,
    principal_id uuid NOT NULL,
    principal_office_scope office_scope_name,
    principal_boundary_set office_boundary_name[] NOT NULL DEFAULT ARRAY[]::office_boundary_name[],
    resource_id text NOT NULL,
    resource_kind text NOT NULL,
    resource_boundary office_boundary_name NOT NULL,
    restricted_deal_id uuid,
    advisor_relationship_id uuid,
    action_class text NOT NULL,
    purpose text,
    decision office_boundary_decision NOT NULL,
    denial_reason_code text,
    clearance_id uuid,
    cedar_policy_version text NOT NULL,
    cedar_schema_version text NOT NULL,
    policy_pack text NOT NULL,
    jurisdiction_anchor_set text[] NOT NULL DEFAULT ARRAY[]::text[],
    principal_stream_commitment text NOT NULL,
    boundary_stream_commitment text NOT NULL,
    tenant_stream_commitment text,
    payload_hash bytea,
    redaction_profile text NOT NULL,
    trace_id text NOT NULL,
    request_time timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (decision <> 'DENY' OR denial_reason_code IS NOT NULL)
);

CREATE INDEX office_boundary_attempts_principal_idx
    ON office_boundary_attempts (tenant_id, principal_id, request_time DESC);

CREATE INDEX office_boundary_attempts_resource_idx
    ON office_boundary_attempts (tenant_id, resource_boundary, request_time DESC);

CREATE INDEX office_boundary_attempts_deal_idx
    ON office_boundary_attempts (tenant_id, restricted_deal_id, request_time DESC)
    WHERE restricted_deal_id IS NOT NULL;
```

### E-4. Audit event classes

- `OfficeScopeAssignmentCreated` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficeScopeAssignmentChanged` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficeScopeAssignmentRevoked` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficeBoundaryClearanceRequested` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficeBoundaryClearanceApproved` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficeBoundaryClearanceDenied` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficeBoundaryClearanceRevoked` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficeBoundaryAttemptEvaluated` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficeBoundaryAttemptDenied` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficeBoundaryAttemptAllowed` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `InformationBarrierTaintAttached` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `InformationBarrierTaintDerived` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `InformationBarrierTaintReleased` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `RestrictedDealCreated` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `RestrictedDealParticipantAdded` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `RestrictedDealParticipantRemoved` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `RestrictedDealReleased` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `AdvisorRelationshipBarrierCreated` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `AdvisorRelationshipBarrierAttemptEvaluated` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficePackOverlayActivated` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficePackOverlayChanged` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

- `OfficePackOverlayRetired` is a mandatory audit-chain event class for ADR-0319 implementations.
  - The event must carry `tenant_id`, `event_id`, `trace_id`, `cedar_policy_version`, `policy_pack`, and `jurisdiction_anchor_set`.
  - The event must carry redacted descriptors and stable identifiers instead of raw restricted payload.
  - The event must follow ADR-0263 structured emission requirements.

### E-5. Service integration contract

Every service that handles protected data must call Cedar before access.
Every service that indexes protected data must preserve the taint on derived index rows.
Every service that exports protected data must evaluate `ExportOfficeScopedResource`.
Every service that emits protected events must evaluate `SubscribeOfficeScopedEvent` for subscribers.
Every service that stores embeddings must evaluate `RetrieveOfficeScopedEmbedding` before returning vectors or generated text.
Every service that trains models must evaluate `TrainFromOfficeScopedResource` before accepting training data.
Every service that manages office assignments must emit assignment audit events.
Every service that manages clearances must emit clearance audit events.
Every service that denies or allows a boundary attempt must emit a boundary-attempt event.
Every service must preserve tenant scoping and audience type from ADR-0244.
Every service must preserve cross-tenant barrier state from ADR-0313.

## Section F: Migration

### F-1. Migration principle

Migration is service-by-service and pack-by-pack.
No service can claim regulated-finance readiness until its role-to-office mapping is declared.
No service can claim regulated-finance readiness until its boundary-attempt audit events are implemented or explicitly declared not applicable by pack.
No service can claim regulated-finance readiness until tenant-only access tests include office-boundary denial cases.
No service can claim regulated-finance readiness until derived data, search, export, and model paths preserve information-barrier taint.

### F-2. Per-service office-scope declaration roster

#### F-2.1. `analytics`
- Migration owner: `analytics` service owner.
- FRONT mapping: `analytics_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `analytics_risk_reviewer`, `analytics_compliance_reviewer`, and `analytics_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `analytics_ops_operator`, `analytics_records_operator`, and `analytics_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.2. `api-gateway`
- Migration owner: `api-gateway` service owner.
- FRONT mapping: `api_gateway_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `api_gateway_risk_reviewer`, `api_gateway_compliance_reviewer`, and `api_gateway_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `api_gateway_ops_operator`, `api_gateway_records_operator`, and `api_gateway_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.3. `application`
- Migration owner: `application` service owner.
- FRONT mapping: `application_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `application_risk_reviewer`, `application_compliance_reviewer`, and `application_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `application_ops_operator`, `application_records_operator`, and `application_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.4. `audit-chain`
- Migration owner: `audit-chain` service owner.
- FRONT mapping: `audit_chain_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `audit_chain_risk_reviewer`, `audit_chain_compliance_reviewer`, and `audit_chain_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `audit_chain_ops_operator`, `audit_chain_records_operator`, and `audit_chain_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.5. `calendar`
- Migration owner: `calendar` service owner.
- FRONT mapping: `calendar_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `calendar_risk_reviewer`, `calendar_compliance_reviewer`, and `calendar_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `calendar_ops_operator`, `calendar_records_operator`, and `calendar_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.6. `cell`
- Migration owner: `cell` service owner.
- FRONT mapping: `cell_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `cell_risk_reviewer`, `cell_compliance_reviewer`, and `cell_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `cell_ops_operator`, `cell_records_operator`, and `cell_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.7. `cloud-iac`
- Migration owner: `cloud-iac` service owner.
- FRONT mapping: `cloud_iac_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `cloud_iac_risk_reviewer`, `cloud_iac_compliance_reviewer`, and `cloud_iac_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `cloud_iac_ops_operator`, `cloud_iac_records_operator`, and `cloud_iac_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.8. `cloud-k8s`
- Migration owner: `cloud-k8s` service owner.
- FRONT mapping: `cloud_k8s_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `cloud_k8s_risk_reviewer`, `cloud_k8s_compliance_reviewer`, and `cloud_k8s_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `cloud_k8s_ops_operator`, `cloud_k8s_records_operator`, and `cloud_k8s_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.9. `cloud-secrets`
- Migration owner: `cloud-secrets` service owner.
- FRONT mapping: `cloud_secrets_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `cloud_secrets_risk_reviewer`, `cloud_secrets_compliance_reviewer`, and `cloud_secrets_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `cloud_secrets_ops_operator`, `cloud_secrets_records_operator`, and `cloud_secrets_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.10. `comms-email`
- Migration owner: `comms-email` service owner.
- FRONT mapping: `comms_email_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `comms_email_risk_reviewer`, `comms_email_compliance_reviewer`, and `comms_email_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `comms_email_ops_operator`, `comms_email_records_operator`, and `comms_email_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.11. `community`
- Migration owner: `community` service owner.
- FRONT mapping: `community_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `community_risk_reviewer`, `community_compliance_reviewer`, and `community_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `community_ops_operator`, `community_records_operator`, and `community_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.12. `compliance`
- Migration owner: `compliance` service owner.
- FRONT mapping: `compliance_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `compliance_risk_reviewer`, `compliance_compliance_reviewer`, and `compliance_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `compliance_ops_operator`, `compliance_records_operator`, and `compliance_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.13. `connector`
- Migration owner: `connector` service owner.
- FRONT mapping: `connect_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `connect_risk_reviewer`, `connect_compliance_reviewer`, and `connect_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `connect_ops_operator`, `connect_records_operator`, and `tenant_rbac_packaging_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.14. `consent-graph`
- Migration owner: `consent-graph` service owner.
- FRONT mapping: `consent_graph_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `consent_graph_risk_reviewer`, `consent_graph_compliance_reviewer`, and `consent_graph_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `consent_graph_ops_operator`, `consent_graph_records_operator`, and `consent_graph_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.15. `crm`
- Migration owner: `crm` service owner.
- FRONT mapping: `crm_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `crm_risk_reviewer`, `crm_compliance_reviewer`, and `crm_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `crm_ops_operator`, `crm_records_operator`, and `crm_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.16. `developer-sdk`
- Migration owner: `developer-sdk` service owner.
- FRONT mapping: `developer_sdk_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `developer_sdk_risk_reviewer`, `developer_sdk_compliance_reviewer`, and `developer_sdk_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `developer_sdk_ops_operator`, `developer_sdk_records_operator`, and `developer_sdk_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.17. `docs`
- Migration owner: `docs` service owner.
- FRONT mapping: `docs_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `docs_risk_reviewer`, `docs_compliance_reviewer`, and `docs_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `docs_ops_operator`, `docs_records_operator`, and `docs_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.18. `drive`
- Migration owner: `drive` service owner.
- FRONT mapping: `drive_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `drive_risk_reviewer`, `drive_compliance_reviewer`, and `drive_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `drive_ops_operator`, `drive_records_operator`, and `drive_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.19. `feature-flags`
- Migration owner: `feature-flags` service owner.
- FRONT mapping: `feature_flags_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `feature_flags_risk_reviewer`, `feature_flags_compliance_reviewer`, and `feature_flags_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `feature_flags_ops_operator`, `feature_flags_records_operator`, and `feature_flags_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.20. `finops-portal`
- Migration owner: `finops-portal` service owner.
- FRONT mapping: `finops_portal_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `finops_portal_risk_reviewer`, `finops_portal_compliance_reviewer`, and `finops_portal_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `finops_portal_ops_operator`, `finops_portal_records_operator`, and `finops_portal_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.21. `forms`
- Migration owner: `forms` service owner.
- FRONT mapping: `forms_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `forms_risk_reviewer`, `forms_compliance_reviewer`, and `forms_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `forms_ops_operator`, `forms_records_operator`, and `forms_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.22. `foundry`
- Migration owner: `foundry` service owner.
- FRONT mapping: `foundry_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `foundry_risk_reviewer`, `foundry_compliance_reviewer`, and `foundry_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `foundry_ops_operator`, `foundry_records_operator`, and `foundry_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.23. `global-trade`
- Migration owner: `global-trade` service owner.
- FRONT mapping: `global_trade_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `global_trade_risk_reviewer`, `global_trade_compliance_reviewer`, and `global_trade_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `global_trade_ops_operator`, `global_trade_records_operator`, and `global_trade_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.24. `governance`
- Migration owner: `governance` service owner.
- FRONT mapping: `governance_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `governance_risk_reviewer`, `governance_compliance_reviewer`, and `governance_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `governance_ops_operator`, `governance_records_operator`, and `governance_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.25. `identity`
- Migration owner: `identity` service owner.
- FRONT mapping: `identity_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `identity_risk_reviewer`, `identity_compliance_reviewer`, and `identity_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `identity_ops_operator`, `identity_records_operator`, and `identity_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.26. `intelligence`
- Migration owner: `intelligence` service owner.
- FRONT mapping: `intelligence_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `intelligence_risk_reviewer`, `intelligence_compliance_reviewer`, and `intelligence_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `intelligence_ops_operator`, `intelligence_records_operator`, and `intelligence_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.27. `mail`
- Migration owner: `mail` service owner.
- FRONT mapping: `mail_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `mail_risk_reviewer`, `mail_compliance_reviewer`, and `mail_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `mail_ops_operator`, `mail_records_operator`, and `mail_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.28. `marketplace`
- Migration owner: `marketplace` service owner.
- FRONT mapping: `marketplace_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `marketplace_risk_reviewer`, `marketplace_compliance_reviewer`, and `marketplace_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `marketplace_ops_operator`, `marketplace_records_operator`, and `marketplace_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.29. `meet`
- Migration owner: `meet` service owner.
- FRONT mapping: `meet_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `meet_risk_reviewer`, `meet_compliance_reviewer`, and `meet_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `meet_ops_operator`, `meet_records_operator`, and `meet_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.30. `messenger`
- Migration owner: `messenger` service owner.
- FRONT mapping: `messenger_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `messenger_risk_reviewer`, `messenger_compliance_reviewer`, and `messenger_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `messenger_ops_operator`, `messenger_records_operator`, and `messenger_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.31. `network`
- Migration owner: `network` service owner.
- FRONT mapping: `network_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `network_risk_reviewer`, `network_compliance_reviewer`, and `network_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `network_ops_operator`, `network_records_operator`, and `network_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.32. `notes`
- Migration owner: `notes` service owner.
- FRONT mapping: `notes_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `notes_risk_reviewer`, `notes_compliance_reviewer`, and `notes_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `notes_ops_operator`, `notes_records_operator`, and `notes_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.33. `observability`
- Migration owner: `observability` service owner.
- FRONT mapping: `observability_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `observability_risk_reviewer`, `observability_compliance_reviewer`, and `observability_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `observability_ops_operator`, `observability_records_operator`, and `observability_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.34. `ontology`
- Migration owner: `ontology` service owner.
- FRONT mapping: `ontology_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `ontology_risk_reviewer`, `ontology_compliance_reviewer`, and `ontology_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `ontology_ops_operator`, `ontology_records_operator`, and `ontology_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.35. `ops-dashboard-control-center`
- Migration owner: `ops-dashboard-control-center` service owner.
- FRONT mapping: `ops_dashboard_control_center_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `ops_dashboard_control_center_risk_reviewer`, `ops_dashboard_control_center_compliance_reviewer`, and `ops_dashboard_control_center_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `ops_dashboard_control_center_ops_operator`, `ops_dashboard_control_center_records_operator`, and `ops_dashboard_control_center_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.36. `payments`
- Migration owner: `payments` service owner.
- FRONT mapping: `payments_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `payments_risk_reviewer`, `payments_compliance_reviewer`, and `payments_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `payments_ops_operator`, `payments_records_operator`, and `payments_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.37. `plant-maintenance`
- Migration owner: `plant-maintenance` service owner.
- FRONT mapping: `plant_maintenance_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `plant_maintenance_risk_reviewer`, `plant_maintenance_compliance_reviewer`, and `plant_maintenance_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `plant_maintenance_ops_operator`, `plant_maintenance_records_operator`, and `plant_maintenance_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.38. `plugin-app-store`
- Migration owner: `plugin-app-store` service owner.
- FRONT mapping: `plugin_app_store_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `plugin_app_store_risk_reviewer`, `plugin_app_store_compliance_reviewer`, and `plugin_app_store_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `plugin_app_store_ops_operator`, `plugin_app_store_records_operator`, and `plugin_app_store_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.39. `production-planning`
- Migration owner: `production-planning` service owner.
- FRONT mapping: `production_planning_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `production_planning_risk_reviewer`, `production_planning_compliance_reviewer`, and `production_planning_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `production_planning_ops_operator`, `production_planning_records_operator`, and `production_planning_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.40. `quality-management`
- Migration owner: `quality-management` service owner.
- FRONT mapping: `quality_management_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `quality_management_risk_reviewer`, `quality_management_compliance_reviewer`, and `quality_management_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `quality_management_ops_operator`, `quality_management_records_operator`, and `quality_management_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.41. `real-estate`
- Migration owner: `real-estate` service owner.
- FRONT mapping: `real_estate_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `real_estate_risk_reviewer`, `real_estate_compliance_reviewer`, and `real_estate_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `real_estate_ops_operator`, `real_estate_records_operator`, and `real_estate_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.42. `recordings`
- Migration owner: `recordings` service owner.
- FRONT mapping: `recordings_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `recordings_risk_reviewer`, `recordings_compliance_reviewer`, and `recordings_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `recordings_ops_operator`, `recordings_records_operator`, and `recordings_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.43. `sheets`
- Migration owner: `sheets` service owner.
- FRONT mapping: `sheets_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `sheets_risk_reviewer`, `sheets_compliance_reviewer`, and `sheets_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `sheets_ops_operator`, `sheets_records_operator`, and `sheets_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.44. `shorts`
- Migration owner: `shorts` service owner.
- FRONT mapping: `shorts_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `shorts_risk_reviewer`, `shorts_compliance_reviewer`, and `shorts_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `shorts_ops_operator`, `shorts_records_operator`, and `shorts_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.45. `sites`
- Migration owner: `sites` service owner.
- FRONT mapping: `sites_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `sites_risk_reviewer`, `sites_compliance_reviewer`, and `sites_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `sites_ops_operator`, `sites_records_operator`, and `sites_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.46. `slides`
- Migration owner: `slides` service owner.
- FRONT mapping: `slides_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `slides_risk_reviewer`, `slides_compliance_reviewer`, and `slides_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `slides_ops_operator`, `slides_records_operator`, and `slides_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.47. `social`
- Migration owner: `social` service owner.
- FRONT mapping: `social_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `social_risk_reviewer`, `social_compliance_reviewer`, and `social_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `social_ops_operator`, `social_records_operator`, and `social_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.48. `supply-chain-planning`
- Migration owner: `supply-chain-planning` service owner.
- FRONT mapping: `supply_chain_planning_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `supply_chain_planning_risk_reviewer`, `supply_chain_planning_compliance_reviewer`, and `supply_chain_planning_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `supply_chain_planning_ops_operator`, `supply_chain_planning_records_operator`, and `supply_chain_planning_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.49. `tasks`
- Migration owner: `tasks` service owner.
- FRONT mapping: `tasks_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `tasks_risk_reviewer`, `tasks_compliance_reviewer`, and `tasks_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `tasks_ops_operator`, `tasks_records_operator`, and `tasks_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.50. `tenancy`
- Migration owner: `tenancy` service owner.
- FRONT mapping: `tenancy_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `tenancy_risk_reviewer`, `tenancy_compliance_reviewer`, and `tenancy_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `tenancy_ops_operator`, `tenancy_records_operator`, and `tenancy_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.51. `translate`
- Migration owner: `translate` service owner.
- FRONT mapping: `translate_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `translate_risk_reviewer`, `translate_compliance_reviewer`, and `translate_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `translate_ops_operator`, `translate_records_operator`, and `translate_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.52. `treasury`
- Migration owner: `treasury` service owner.
- FRONT mapping: `treasury_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `treasury_risk_reviewer`, `treasury_compliance_reviewer`, and `treasury_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `treasury_ops_operator`, `treasury_records_operator`, and `treasury_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.53. `warehouse`
- Migration owner: `warehouse` service owner.
- FRONT mapping: `warehouse_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `warehouse_risk_reviewer`, `warehouse_compliance_reviewer`, and `warehouse_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `warehouse_ops_operator`, `warehouse_records_operator`, and `warehouse_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.54. `workflow-engine`
- Migration owner: `workflow-engine` service owner.
- FRONT mapping: `workflow_engine_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `workflow_engine_risk_reviewer`, `workflow_engine_compliance_reviewer`, and `workflow_engine_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `workflow_engine_ops_operator`, `workflow_engine_records_operator`, and `workflow_engine_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.55. `workflow-studio`
- Migration owner: `workflow-studio` service owner.
- FRONT mapping: `workflow_studio_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `workflow_studio_risk_reviewer`, `workflow_studio_compliance_reviewer`, and `workflow_studio_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `workflow_studio_ops_operator`, `workflow_studio_records_operator`, and `workflow_studio_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

#### F-2.56. `workplace-integration`
- Migration owner: `workplace-integration` service owner.
- FRONT mapping: `workplace_integration_front_user` is denied by default until the active product pack binds it to `IB`, `Trading`, `Research`, `AssetMgmt`, or `WealthMgmt`.
- MIDDLE mapping: `workplace_integration_risk_reviewer`, `workplace_integration_compliance_reviewer`, and `workplace_integration_legal_reviewer` are review-only and purpose-bound.
- BACK mapping: `workplace_integration_ops_operator`, `workplace_integration_records_operator`, and `workplace_integration_platform_operator` are operational and payload-limited.
- Boundary exposure: tenant membership alone is insufficient for restricted data in this service.
- Cedar requirement: evaluate `Tenant::OfficeScopeAssignment` and `Tenant::OfficeBoundaryClearance` for protected read, write, search, export, event, embedding, and model actions.
- Audit requirement: emit `OfficeBoundaryAttemptEvaluated` for every protected decision and the relevant assignment or clearance event when state changes.
- Taint requirement: preserve `Tenant::InformationBarrierTaint` on derived records, reports, indexes, embeddings, and workflow events.
- Denial test: prove a same-tenant principal without the target boundary cannot access a tainted resource.
- Clearance test: prove a same-tenant principal with a valid clearance can perform only the named action until expiry.
- Revocation test: prove revocation invalidates cached access and creates audit evidence.
- Pack test: prove the service honors IS-Banking, IS-Investment-Mgmt, IS-Insurance, and general-tenant overlays when they apply.

### F-3. Migration waves

Wave 1 migrates identity, tenancy, governance, compliance, audit-chain, observability, and api-gateway because they define the control plane.
Wave 2 migrates global-trade, treasury, payments, crm, wealth-adjacent customer systems, marketplace, and workflow-engine because they are direct regulated-data surfaces.
Wave 3 migrates analytics, intelligence, ontology, docs, drive, sheets, slides, mail, messenger, meet, recordings, and search-adjacent services because they can leak or derive restricted content.
Wave 4 migrates operations, workplace, supply-chain, production, quality, plant, warehouse, real-estate, and remaining general services because office barriers can be activated by tenant packs.
Wave 5 migrates developer-sdk, plugin-app-store, feature-flags, cloud infrastructure, and service templates because extension surfaces must inherit this doctrine automatically.

### F-4. Migration acceptance gates

- Gate `G01`: Cedar schema contains `Tenant::OfficeScope` and `Tenant::OfficeBoundary`.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G02`: Cedar schema contains assignment, clearance, taint, restricted-deal, and advisor-barrier entities.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G03`: Postgres migration creates `office_scope_assignments`.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G04`: Postgres migration creates `office_boundary_clearances`.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G05`: Postgres migration creates `office_boundary_attempts`.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G06`: Assignment creation emits `OfficeScopeAssignmentCreated`.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G07`: Assignment update emits `OfficeScopeAssignmentChanged`.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G08`: Assignment revocation emits `OfficeScopeAssignmentRevoked`.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G09`: Clearance approval emits `OfficeBoundaryClearanceApproved`.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G10`: Clearance revocation emits `OfficeBoundaryClearanceRevoked`.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G11`: Every allowed attempt emits `OfficeBoundaryAttemptAllowed` or `OfficeBoundaryAttemptEvaluated` with decision ALLOW.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G12`: Every denied attempt emits `OfficeBoundaryAttemptDenied` or `OfficeBoundaryAttemptEvaluated` with decision DENY.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G13`: Search preserves taint.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G14`: Export preserves taint.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G15`: Embedding retrieval preserves taint.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G16`: Model training denies tainted data by default.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G17`: M&A deal room restriction blocks Trading by default.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G18`: Research embargo blocks Trading before publication.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G19`: Asset-management portfolio data blocks IB by default when mandate conflict applies.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G20`: Wealth trusted-contact data maps to Rule 4512(a)(1)(F) and not Rule 4514.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G21`: Negotiable-instrument authorization records map to Rule 4514.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G22`: FINRA Rule 4530 evidence can be produced without restricted payload exposure.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G23`: MAR Article 18 insider-list evidence can be produced for restricted deals.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G24`: MiFID II Article 16(8) record controls are represented in audit evidence.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G25`: KR FSCMA Articles 174 through 178 anchors can be attached by pack.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G26`: FCA SYSC 10.2 conflicts evidence can be produced.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G27`: Singapore SFA sections 218 through 220 anchors can be attached by pack.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G28`: Australia Corporations Act sections 912A and 1043A anchors can be attached by pack.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G29`: ASIC RG 181 conflicts evidence can be produced.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G30`: Volcker-sensitive Trading taint can be enforced.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G31`: Glass-Steagall-style structural separation delegates to ADR-0313 when tenant hierarchy requires it.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G32`: ADR-0244 audience_type remains visible in assignment evaluation.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G33`: ADR-0243 remains the only authorization gate for allow or deny decisions.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G34`: ADR-0263 remains the emission contract for audit events.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G35`: ADR-0313 cross-child dual sealing remains active for parent-child crossings.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

- Gate `G36`: ADR-0316 forward reference is tracked without fabricated requirements.
  - Proof shape: automated policy test, migration test, or audit-shape test.
  - Failure state: regulated-finance pack cannot claim office-barrier readiness.
  - Evidence sink: audit-chain evidence plus service-local test output.

## Section G: References

### G-1. Internal references

- ADR-0243: `docs/decisions/ADR-0243-cedar-as-universal-gate.md`.
- ADR-0244: `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`.
- ADR-0263: `docs/decisions/ADR-0263-observability-emission-contract.md`.
- ADR-0313: `docs/decisions/ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md`.
- ADR-0313 Section D-6: joint venture and cross-child information barrier worked example.
- ADR-0316: forward cross-reference reserved by the requested ADR bundle; no local file existed at authoring time.
- Documentation rigor standard: `docs/standards/documentation-rigor.md`.

### G-2. Official regulatory and source references

- FINRA Rule 5280: https://www.finra.org/rules-guidance/rulebooks/finra-rules/5280
- FINRA Rule 5290: https://www.finra.org/rules-guidance/rulebooks/finra-rules/5290
- FINRA Rule 3110: https://www.finra.org/rules-guidance/rulebooks/finra-rules/3110
- FINRA trusted-contact report and Rule 4512(a)(1)(F) discussion: https://www.finra.org/rules-guidance/guidance/reports/2024-finra-annual-regulatory-oversight-report/trusted-contact-persons
- FINRA Rule 4514: https://www.finra.org/rules-guidance/rulebooks/finra-rules/4514
- FINRA Rule 4530: https://www.finra.org/rules-guidance/rulebooks/finra-rules/4530
- EUR-Lex Regulation (EU) No 596/2014 Market Abuse Regulation: https://eur-lex.europa.eu/legal-content/en/ALL/?qid=1674279978096&uri=CELEX%3A32014R0596
- ESMA MiFID II Article 16 organizational requirements: https://www.esma.europa.eu/fr/publications-and-data/interactive-single-rulebook/mifid-ii/article-16-organisational-requirements
- EUR-Lex Directive 2014/65/EU MiFID II: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX%3A32014L0065
- Korea Financial Investment Services and Capital Markets Act English translation: https://elaw.klri.re.kr/eng_mobile/ganadaDetail.do?hseq=69898&key=FINANCIAL+INVESTMENT+SERVICES+AND+CAPITAL+MARKETS+ACT&param=F&type=abc
- Federal Reserve Volcker Rule page: https://www.federalreserve.gov/supervisionreg/volcker-rule.htm
- eCFR 12 CFR Part 248 Volcker Rule implementing regulation: https://www.ecfr.gov/current/title-12/chapter-II/subchapter-A/part-248
- Federal Reserve History Glass-Steagall Act essay: https://www.federalreservehistory.org/essays/glass-steagall-act
- FCA Handbook SYSC 10.2: https://handbook.fca.org.uk/handbook/SYSC/10/2.html
- MAS Code of Conduct for Credit Rating Agencies: https://www.mas.gov.sg/-/media/mas/about-mas/code-of-conduct-for-credit-rating-agencies--8oct2018.pdf
- MAS Financial Institutions Directory: https://eservices.mas.gov.sg/fid
- Singapore Securities and Futures Act 2001: https://sso.agc.gov.sg/Act/SFA2001
- Australia Corporations Act 2001: https://www.legislation.gov.au/C2004A00818/latest/text
- ASIC Regulatory Guide 181: https://asic.gov.au/regulatory-resources/find-a-document/regulatory-guides/rg-181-licensing-managing-conflicts-of-interest/
- Cedar documentation: https://docs.cedarpolicy.com/
- AWS Verified Permissions user guide: https://docs.aws.amazon.com/verifiedpermissions/latest/userguide/what-is-avp.html
- Google Cloud IAM Conditions overview: https://cloud.google.com/iam/docs/conditions-overview
- Microsoft Entra Privileged Identity Management documentation: https://learn.microsoft.com/en-us/entra/id-governance/privileged-identity-management/pim-configure
- AWS CloudTrail user guide: https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-user-guide.html

### G-3. Source-use notes

- FINRA Rule 4512(a)(1)(F), not FINRA Rule 4514, is the trusted-contact anchor used by this ADR.
- FINRA Rule 4514 remains a record-authority anchor for negotiable-instrument authorization records.
- EU MAR Article 9 is cited for legitimate behaviour; Articles 14, 16, 17, and 18 are cited for prohibitions, detection, disclosure, and insider-list mechanics.
- MiFID II Article 16(3), Article 16(8), and Article 23 are cited for organizational controls, record security, and conflicts of interest.
- Korea FSCMA Articles 174 through 178 are cited as the Korean market-abuse and unfair-trading anchor set; pack compilers bind exact statutory labels from active legal text.
- Glass-Steagall and the Volcker Rule are used as structural-separation precedents, not as one-to-one software requirements.
- Chinese Wall is treated as a historical synonym; implementation language should prefer information barrier.

## Section H: Change Log and Naming Justifications

### H-1. Change log

- 2026-05-20: Created ADR-0319 as the doctrine for Front, Middle, and Back Office information barriers inside a tenant.
- 2026-05-20: Added `Tenant::OfficeScope` and `Tenant::OfficeBoundary` as required Cedar entity types.
- 2026-05-20: Added assignment, clearance, taint, restricted-deal, advisor-barrier, pack-overlay, and audit-event doctrine.
- 2026-05-20: Bound the doctrine to ADR-0243, ADR-0244, ADR-0263, ADR-0313, and a bounded forward reference to ADR-0316.
- 2026-05-20: Corrected the FINRA trusted-contact anchor by distinguishing Rule 4512(a)(1)(F) from Rule 4514.
- 2026-05-20: Included official regulatory references for US FINRA and SEC/Volcker, EU MAR and MiFID II, Korea FSCMA, UK FCA, Singapore SFA, Australia Corporations Act, ASIC RG 181, and hyperscaler policy precedents.
- 2026-05-20: Added service-by-service migration declarations for the current microservice roster.

### H-2. Naming justification: `Tenant::OfficeScope`

`OfficeScope` is used instead of `OfficeRole` because it scopes authorization and data handling, not only job function.
`OfficeScope` is used instead of `Department` because tenants may have many departments with different local names.
`OfficeScope` is used instead of `RegulatedRole` because service principals, workflows, and derived-data processors also need the label.
`FRONT`, `MIDDLE`, and `BACK` are uppercase enum values because they are doctrine constants and map cleanly to Cedar attributes, SQL enums, and policy-pack manifests.

### H-3. Naming justification: `Tenant::OfficeBoundary`

`OfficeBoundary` is used instead of `ChineseWall` because implementation language should describe the control directly.
`OfficeBoundary` is used instead of `BusinessLine` because the same business line can have multiple barriers and the same barrier can span multiple departments.
`IB`, `Trading`, `Research`, `AssetMgmt`, and `WealthMgmt` are selected because they are the minimum capital-markets and wealth-management boundaries needed by the brief and by the regulatory anchors.
The enum can be extended only by a later ADR or by a stricter pack-specific attribute that does not weaken the base doctrine.

### H-4. Naming justification: `InformationBarrierTaint`

`InformationBarrierTaint` is used because the control must survive derivation.
A report derived from restricted deal data remains tainted.
A search index derived from embargoed research remains tainted.
An embedding derived from order flow remains tainted.
A model-training set derived from wealth customer data remains tainted.
Taint release requires a policy event, not a silent data copy.

### H-5. Naming justification: `OfficeBoundaryClearance`

`OfficeBoundaryClearance` is used instead of `Exception` because the entity is affirmative, bounded, approved, expiring, and auditable.
The clearance name prevents broad bypass thinking.
The clearance name makes approval, expiry, revocation, and dual sealing part of the domain model.
The clearance name maps naturally to compliance and legal review workflows.

### H-6. Index and catalog handling

This ADR file is the requested deliverable.
ADR indexes, document catalogs, and changelogs are not modified by this ADR authoring step because this checkout contains broad unrelated in-flight changes in documentation surfaces.
A later indexing step may add this ADR to repository catalogs after the concurrent documentation work stabilizes.
The doctrine itself is complete in this file and does not depend on index mutation for binding force.

### H-7. Completion criteria

- The file contains more than 1500 lines.
- The file introduces `Tenant::OfficeScope`.
- The file introduces `Tenant::OfficeBoundary`.
- The file includes `FRONT`, `MIDDLE`, and `BACK`.
- The file includes `IB`, `Trading`, `Research`, `AssetMgmt`, and `WealthMgmt`.
- The file covers per-employee assignment and audit-chain assignment events.
- The file covers cross-boundary Cedar permits.
- The file covers default-deny information-barrier enforcement.
- The file covers per-pack overlays.
- The file covers audit-chain dual sealing on every boundary attempt.
- The file covers US FINRA and SEC/Volcker anchors.
- The file covers EU MAR and MiFID II anchors.
- The file covers Korea FSCMA Articles 174 through 178.
- The file covers UK FCA, Singapore MAS/SFA, and Australia ASIC anchors.
- The file covers Bank Operations Officer, Bank Risk Manager, and Bank Compliance Officer assignments.
- The file covers per-deal M&A barriers.
- The file covers cross-tenant advisor barriers.
- The file includes Postgres DDL for `office_scope_assignments`.
- The file includes Postgres DDL for `office_boundary_clearances`.
- The file includes Postgres DDL for `office_boundary_attempts`.
- The file includes audit-event classes.
- The file includes per-service migration declarations.
- The file includes official references.
- The file includes change log and naming justifications.

End of ADR-0319.
