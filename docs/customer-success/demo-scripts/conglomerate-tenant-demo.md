---
doc_class: DemoScript
target_persona: Holdings company CIO, group COO, corporate development leader, portfolio company president, group compliance officer, enterprise architecture leader
duration_minutes: 45
related_oyatie_adrs:
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0313
  - ADR-0314
  - ADR-0316
status: canonical
date: 2026-05-20
owner: Customer Success Engineering
---

# Conglomerate Tenant Demo

## Pre-Demo Checklist

- Confirm meeting type: holdings-company tenant architecture demo or parent-subsidiary governance discussion.
- Confirm prospect segment: conglomerate, private equity platform, holding company, multinational group, family office operating group, or corporate parent with semi-autonomous subsidiaries.
- Confirm primary sponsor: group CIO, group COO, corporate development lead, group compliance officer, chief architect, or portfolio operating partner.
- Confirm stated pain: parent visibility without overreach, subsidiary autonomy, divestiture readiness, acquisition onboarding, cross-company services, shared compliance evidence, or confusing identity boundaries.
- Confirm deal hypothesis: Oyatie ADR-0313 conglomerate tenant hierarchy as sovereign child tenants linked by explicit Cedar permits.
- Confirm named tenant fixture: `tenant-helios-fortune-500-manufacturer`.
- Confirm fixture mode: synthetic parent group with synthetic operating companies, synthetic permits, synthetic shared services, and synthetic restructuring events.
- Confirm ADR-0313 language: child tenants are sovereign; parent access is explicit, permit-bound, and auditable.
- Confirm demo duration: 45 minutes.
- Confirm agenda allocation: 5 minutes opening, 7 minutes discovery, 27 minutes demo, 6 minutes close and commercial.
- Open Oya Demo Console.
- Open Tenant Hierarchy Console.
- Open Conglomerate Grants Registry.
- Open Cedar Permit Simulator.
- Open Parent Visibility Dashboard.
- Open Child Sovereignty Console.
- Open Shared Services Marketplace.
- Open Compliance Evidence Portal.
- Open Restructuring Workbench.
- Open Audit Replay Console.
- Open Cost and Services Ledger.
- Open Migration Roadmap Builder.
- Load tenant fixture `tenant-helios-fortune-500-manufacturer`.
- Load parent tenant `helios-group-parent`.
- Load child tenant `helios-na-industrial`.
- Load child tenant `helios-kr-components`.
- Load child tenant `helios-eu-aerospace`.
- Load child tenant `helios-health-benefits-services`.
- Load persona `group-coo`.
- Load persona `group-cio`.
- Load persona `parent-compliance-officer`.
- Load persona `child-company-president`.
- Load persona `kr-subsidiary-privacy-manager`.
- Load persona `shared-services-finance-operator`.
- Load persona `corporate-development-lead`.
- Load persona `external-auditor-observer`.
- Enable compliance packs `GDPR`, `KR-PIPA`, `SOX-evidence`, `Supplier-Traceability`, `EU-AI-ACT`.
- Enable demo grant `parent_aggregate_operational_risk_view`.
- Enable demo grant `shared_services_invoice_processing`.
- Enable demo grant `group_compliance_evidence_export`.
- Enable demo workflow `subsidiary-risk-rollup`.
- Enable demo workflow `shared-service-payment-approval`.
- Enable demo workflow `divestiture-revoke-and-regrant`.
- Prepare objection-response card `parent-overreach`.
- Prepare objection-response card `subsidiary-autonomy`.
- Prepare objection-response card `divestiture-readiness`.
- Prepare objection-response card `shared-services`.
- Prepare objection-response card `privacy-boundaries`.
- Prepare pricing card `conglomerate-tenant-architecture`.
- Prepare leave-behind `ADR-0313-Conglomerate-Tenant-Control-Map`.
- Verify browser zoom is 90 percent.
- Verify child tenant names are synthetic.
- Verify parent dashboard masks child personal data.
- Verify divestiture action is simulated only.
- Verify permit changes happen in demo fixture, not production.
- Stop condition: do not alter real tenant hierarchy, do not imply parent gets unrestricted subsidiary access, and do not promise legal sufficiency for restructurings without counsel.

## Opening Hook

- Say: "Conglomerates have a different software problem than single operating companies."
- Say: "The parent needs visibility."
- Say: "Subsidiaries need sovereignty."
- Say: "Shared services need enough access to operate."
- Say: "Compliance needs evidence across the group."
- Say: "Corporate development needs acquisition and divestiture paths."
- Say: "Local privacy teams need boundaries."
- Say: "Traditional enterprise software often collapses this into one of two bad models."
- Say: "Either the parent has too much access and local teams do not trust the platform."
- Say: "Or every subsidiary runs separately and the parent cannot see risk, cost, or performance."
- Say: "ADR-0313 defines Oyatie's answer."
- Say: "Child tenants are sovereign."
- Say: "Parent-child relationships are expressed through explicit Cedar permits."
- Say: "The grant is recorded in a conglomerate grants registry."
- Say: "Access is scoped to purpose, role, data, jurisdiction, and time."
- Say: "Restructuring is handled by revoking and granting permits, not by pretending org charts are static."
- Say: "Today I will use the Helios fixture."
- Say: "We will show a parent group with North America, Korea, Europe, and shared-services child tenants."
- Say: "We will show parent aggregate risk without child personal-data overreach."
- Say: "We will show a shared-services payment workflow."
- Say: "We will show a Korean subsidiary privacy boundary."
- Say: "We will simulate divestiture by revoking and regranting access."
- Say: "We will show dual-sealed audit evidence."
- Say: "The dollar example is concrete."
- Say: "A group-level supplier disruption affects 41 million dollars of backlog, but the parent can only inspect child details through grants."
- Say: "The outcome of this call is deciding whether a two-week conglomerate tenant architecture workshop is worth scheduling."
- Pause.
- Ask: "Should we emphasize parent visibility, subsidiary autonomy, shared services, compliance evidence, or restructuring readiness?"
- Adjust sequence if the buyer chooses a priority.

## Discovery Questions

1. How many operating companies or subsidiaries are in scope for the first phase?
2. Which entities require legal, regulatory, or privacy separation from the parent?
3. What does the parent need to see: risk, finance, operations, compliance, talent, supplier exposure, or all of them?
4. Which data must the parent explicitly not see?
5. Which shared services operate across subsidiaries today?
6. How are parent-subsidiary access rights documented and reviewed?
7. What happens to systems and permissions during acquisitions, carve-outs, or divestitures?
8. Which local teams distrust group platforms because of access overreach?
9. How do you prove parent access was permitted for a specific purpose?
10. Which group compliance reports require evidence from multiple subsidiaries?
11. How do you separate operational visibility from personal-data visibility?
12. Which approvals require both child-company and parent-company evidence?
13. What is the cost of maintaining duplicate systems for autonomy?
14. What would make a first tenant-hierarchy pilot credible in 30 to 60 days?
15. Who owns the governance model: corporate, legal, IT, compliance, or operating-company presidents?

## Demo Flow

1. Screen: Tenant Hierarchy Console.
- Click `Demo Console`.
- Select tenant fixture `tenant-helios-fortune-500-manufacturer`.
- Click `Tenant Hierarchy`.
- Say: "This is the synthetic Helios group."
- Point to parent `helios-group-parent`.
- Point to child `helios-na-industrial`.
- Point to child `helios-kr-components`.
- Point to child `helios-eu-aerospace`.
- Point to child `helios-health-benefits-services`.
- Click `ADR-0313 Mode`.
- Show `Child tenants sovereign`.
- Show `Parent access by explicit grants`.
- Show `Dual-sealed audit`.
- Show `Restructuring by revoke and grant`.
- Say: "The parent is not the database owner of every child."
- Say: "The parent is a tenant with explicit relationships."
- Dollar example: "The group wants a 41 million dollar backlog risk view without taking unrestricted child-tenant access."

2. Screen: Parent Visibility Dashboard.
- Switch persona to `group-coo`.
- Click `Parent Visibility`.
- Show group metrics: backlog risk, supplier concentration, cash exposure, compliance packet status, operational incidents.
- Click `Supplier Concentration`.
- Show aggregate exposure: `Taejin Precision Components`, `$41,000,000 backlog at risk`.
- Click `Child Detail`.
- Show masked detail for `helios-kr-components`.
- Say: "The parent sees aggregate risk."
- Say: "The parent does not automatically see local personal data or detailed records."
- Click `Request Detail`.
- Show grant requirement `parent_aggregate_operational_risk_view`.
- Dollar example: "The parent can act on a 41 million dollar risk signal while respecting child boundaries."

3. Screen: Conglomerate Grants Registry.
- Click `Conglomerate Grants`.
- Open grant `parent_aggregate_operational_risk_view`.
- Show grantor `helios-kr-components`.
- Show grantee `helios-group-parent`.
- Show purpose `group operational risk aggregation`.
- Show fields allowed: supplier name, exposure band, backlog amount, risk rating, remediation status.
- Show fields denied: employee names, personal contact details, local HR notes, unrelated supplier contracts.
- Show duration `90 days`.
- Show review owner `kr-subsidiary-privacy-manager`.
- Say: "This is the ADR-0313 mechanism."
- Say: "Relationship is not implied by ownership."
- Say: "Relationship is made explicit through a grant."
- Click `Cedar Policy`.
- Show policy conditions.
- Dollar example: "A 90-day grant can support a crisis response without creating permanent access overreach."

4. Screen: Cedar Permit Simulation for Parent View.
- Click `Cedar Permit Simulator`.
- Select persona `group-coo`.
- Select action `view_child_supplier_risk_detail`.
- Select child tenant `helios-kr-components`.
- Select grant `parent_aggregate_operational_risk_view`.
- Click `Simulate`.
- Show decision `Allowed`.
- Click `Requested Fields`.
- Add field `local_employee_mobile`.
- Click `Simulate`.
- Show decision `Denied`.
- Say: "The same parent role is allowed for operational risk fields and denied for personal data."
- Click `Explain`.
- Show reason `field not included in conglomerate grant and KR PIPA pack restricts purpose`.
- Dollar example: "This avoids the false choice between no visibility and full data copy."

5. Screen: Child Sovereignty Console.
- Switch persona to `child-company-president`.
- Switch tenant to `helios-kr-components`.
- Click `Child Sovereignty`.
- Show active parent grants.
- Show local workflows.
- Show local compliance packs.
- Show local data restrictions.
- Say: "The child company can inspect what the parent can see."
- Click `Grant Review`.
- Open parent risk grant.
- Click `Request Narrowing`.
- Remove field `supplier contact name`.
- Add reason `Business email sufficient for parent aggregation`.
- Click `Submit`.
- Say: "Subsidiary governance is not ceremonial."
- Click `Audit Trail`.
- Show child requested narrowing.
- Dollar example: "Trust is worth money because it reduces local resistance and duplicate-system creation."

6. Screen: KR PIPA Boundary.
- Switch persona to `kr-subsidiary-privacy-manager`.
- Click `KR PIPA Pack`.
- Show local restrictions for Korean personal data.
- Click `Parent Access Review`.
- Show parent requests pending.
- Approve aggregate risk exposure.
- Deny personal mobile export.
- Add note `Aggregate operational view approved; personal mobile not necessary`.
- Say: "Local privacy review is first-class."
- Click `Cedar Trace`.
- Show pack condition included in parent permit.
- Dollar example: "The parent avoids creating a cross-border privacy issue while still getting risk visibility."

7. Screen: Shared Services Marketplace.
- Switch persona to `shared-services-finance-operator`.
- Switch tenant to `helios-group-parent`.
- Click `Shared Services`.
- Open service `invoice processing for subsidiaries`.
- Say: "Shared services are where conglomerate access gets practical."
- Select child `helios-eu-aerospace`.
- Open invoice `INV-EU-88491`.
- Show amount `EUR 780,000`.
- Show allowed fields: vendor, invoice amount, PO match, tax status, approval state.
- Show denied fields: local HR attachment, unrelated contract notes.
- Click `Process`.
- Show required child approval and parent shared-services approval.
- Say: "Shared services can operate without becoming all-seeing."
- Dollar example: "Centralizing invoice operations across 11 subsidiaries might save 3.2 million dollars annually, but only if local controls trust the model."

8. Screen: Shared-Service Payment Approval.
- Click `Workflow Studio`.
- Open workflow `Shared Service Payment Approval`.
- Run case `INV-EU-88491`.
- Show steps: child PO match, child business approval, parent shared-service validation, treasury release, evidence packet.
- Switch persona to `child-company-president`.
- Approve business context.
- Switch persona to `shared-services-finance-operator`.
- Validate invoice.
- Click `Release Payment`.
- Show decision `Denied: treasury release requires parent treasury role`.
- Switch persona to `group-coo`.
- Show also denied because COO lacks treasury release.
- Say: "Authority is not inherited from seniority."
- Dollar example: "The payment is 780,000 euros, so the wrong shortcut would be expensive."

9. Screen: Dual-Sealed Audit Evidence.
- Click `Evidence Portal`.
- Open packet `SHARED-SERVICE-PAYMENT-INV-EU-88491`.
- Show parent seal and child seal.
- Say: "Dual-sealed audit means both tenant perspectives are preserved."
- Click `Child Evidence`.
- Show child approval, PO match, local tax note.
- Click `Parent Evidence`.
- Show shared-service validation and treasury routing.
- Click `Permit Evidence`.
- Show conglomerate grants and Cedar decisions.
- Click `Export`.
- Select `Parent Internal Audit`.
- Show parent-visible fields.
- Select `Child Local Audit`.
- Show child-visible fields.
- Say: "Different audit views, same underlying trace."
- Dollar example: "If each shared-service audit takes 40 hours manually and the group has 300 sampled items, that is 12,000 hours of audit effort."

10. Screen: Parent Compliance Rollup.
- Switch persona to `parent-compliance-officer`.
- Click `Compliance Rollup`.
- Show subsidiaries and packet status.
- Click `EU Aerospace`.
- Show GDPR pack status.
- Click `KR Components`.
- Show KR PIPA pack status.
- Click `Health Benefits Services`.
- Show HIPAA-style privacy control status.
- Click `Group Export`.
- Show aggregate evidence without child restricted fields.
- Say: "The parent compliance officer gets group-level assurance."
- Say: "Local restricted data remains local unless a grant allows it."
- Dollar example: "Group compliance reporting can move from monthly manual chase to current evidence status."

11. Screen: Acquisition Onboarding.
- Click `Restructuring Workbench`.
- Select action `Acquire subsidiary`.
- Enter synthetic target `Helios-MX-Assembly`.
- Click `Create Child Tenant Plan`.
- Show steps: legal entity setup, IAM boundary, initial packs, parent grants, shared-services grants, evidence retention, migration schedule.
- Say: "Acquisition onboarding is not just account creation."
- Say: "It is governance design."
- Click `Initial Parent Grants`.
- Show default grants are minimal.
- Say: "ADR-0313 starts from sovereignty, then adds explicit relationship."
- Dollar example: "If an acquisition integration budget is 12 million dollars, avoiding a chaotic access model is a real cost lever."

12. Screen: Divestiture Revoke and Regrant.
- Click `Restructuring Workbench`.
- Select action `Divest child`.
- Select child `helios-eu-aerospace`.
- Click `Simulate Divestiture`.
- Show grants to revoke: parent risk view, shared-services invoice processing, group compliance export, treasury observer.
- Show grants to retain temporarily: transition services agreement for payroll and tax support.
- Click `Generate Revoke Plan`.
- Say: "Divestiture is where implicit access models fail."
- Click `Preview Effective Access After Close`.
- Show parent loses operational detail.
- Show transition services retain narrow access until date.
- Dollar example: "A divestiture with a 500 million dollar sale price should not depend on manual permission cleanup."

13. Screen: Regrant Under Transition Services Agreement.
- Click `Transition Services`.
- Create grant `tsa_invoice_support_90_days`.
- Grantor `helios-eu-aerospace`.
- Grantee `helios-group-parent`.
- Purpose `post-close invoice support`.
- Duration `90 days`.
- Fields `invoice id`, `vendor`, `amount`, `tax status`, `approval state`.
- Deny fields `customer contracts`, `employee personal data`, `product roadmap`.
- Click `Simulate`.
- Show allowed narrow access.
- Say: "The relationship changes, so the grants change."
- Click `Audit`.
- Show revoke and regrant events.
- Dollar example: "This reduces stranded access risk after close."

14. Screen: Corporate Development View.
- Switch persona to `corporate-development-lead`.
- Click `Portfolio Events`.
- Show acquisition, merger, divestiture, joint venture templates.
- Click `Joint Venture`.
- Show proposed JV between `helios-na-industrial` and external partner.
- Show default access `none`.
- Click `Request JV Reporting Grant`.
- Show required approvals: child president, parent compliance, legal, partner admin.
- Say: "New entity relationships start from no access."
- Dollar example: "A joint venture with 80 million dollars annual revenue can still run on narrow reporting grants."

15. Screen: Personal and Work Data Separation.
- Click `Data Boundary`.
- Select child `helios-health-benefits-services`.
- Select parent persona `group-cfo`.
- Attempt access `employee health accommodation details`.
- Click `Simulate`.
- Show denied.
- Attempt access `aggregate benefits cost trend`.
- Click `Simulate`.
- Show allowed.
- Say: "Parent finance can see aggregate cost trend."
- Say: "Parent finance cannot see personal health details."
- Click `Evidence`.
- Show permit traces for both decisions.
- Dollar example: "Aggregate benefit trend can inform a 12 million dollar budget decision without exposing individual records."

16. Screen: Cost and Services Ledger.
- Click `Cost and Services Ledger`.
- Show shared services costs by child.
- Show parent platform allocation.
- Show subsidiary usage.
- Show compliance evidence cost.
- Click `Allocation Rule`.
- Show invoice processing allocated by volume, compliance evidence by pack activation, platform base by tenant tier.
- Say: "Conglomerates need commercial transparency inside the group."
- Dollar example: "If shared services costs 18 million dollars annually, a transparent allocation model reduces subsidiary disputes."
- Click `Chargeback Preview`.
- Show `helios-eu-aerospace: EUR 1,240,000`.
- Show `helios-kr-components: KRW equivalent synthetic`.

17. Screen: Audit Replay of Parent Access.
- Click `Audit Replay`.
- Select event `PARENT-RISK-VIEW-KR-2026-05`.
- Click `Replay`.
- Show parent request, child grant, KR privacy review, Cedar allow, field mask, parent dashboard view, child narrowing request.
- Say: "This is the evidence that parent access was permitted, scoped, and reviewed."
- Click `What Parent Saw`.
- Show aggregate risk fields.
- Click `What Parent Could Not See`.
- Show denied fields.
- Say: "This is stronger than relying on policy documents."
- Dollar example: "During an internal investigation, replay can save days of interviews and exports."

18. Screen: Architecture Summary.
- Click `Architecture`.
- Show parent tenant, child tenants, grants registry, Cedar policy, pack overlays, evidence store, marketplace services, restructuring workbench.
- Say: "The architecture is built for changing corporate structure."
- Click `Grant Lifecycle`.
- Show create, review, narrow, revoke, expire, regrant.
- Click `Tenant Sovereignty`.
- Show each child has independent policy, packs, workflows, evidence.
- Click `Parent Aggregation`.
- Show parent views created from permitted projections.
- Dollar example: "This is how a group can standardize platform mechanics while preserving legal and operating boundaries."

19. Screen: Executive Value Model.
- Click `Cost and Value`.
- Set subsidiaries `18`.
- Set shared-service workflows `7`.
- Set annual shared-service transactions `220,000`.
- Set audit sample items `300`.
- Set restructuring events per year `4`.
- Click `Calculate`.
- Show value levers:
- Line `shared-service workflow efficiency: $2,400,000`.
- Line `audit evidence reduction: $1,100,000`.
- Line `duplicate system avoidance: $1,800,000`.
- Line `restructuring access cleanup reduction: $650,000`.
- Line `privacy incident risk reduction: qualitative plus risk-adjusted`.
- Show first-year target `$5,950,000`.
- Say: "The numbers depend on group structure."
- Say: "But the value levers are usually visible quickly."

20. Screen: Workshop Plan.
- Click `Migration Roadmap`.
- Show phase 0: tenant hierarchy workshop.
- Show phase 1: one parent rollup and one child grant.
- Show phase 2: one shared-service workflow.
- Show phase 3: compliance evidence rollup.
- Show phase 4: acquisition or divestiture simulation.
- Show phase 5: production pilot recommendation.
- Click `Exit Criteria`.
- Show grant model approved, child sovereignty validated, evidence packet approved, value model accepted.
- Say: "The first workshop should prove the governance model before broad rollout."
- Click `Generate Executive Summary`.
- Show recommendation `Proceed to tenant hierarchy workshop`.

## Objection Handling

1. Objection: "The parent owns the company, so why not give full access?"
- Response name: Ownership-Is-Not-Access.
- Say: "Legal ownership does not mean every user should access every child record."
- Say: "Privacy, regulatory, operational, and trust boundaries still matter."
- Say: "ADR-0313 makes access explicit and evidenced."

2. Objection: "Subsidiaries will resist a group platform."
- Response name: Sovereignty-First.
- Say: "The model starts from child sovereignty."
- Say: "Subsidiaries can inspect and review parent grants."
- Say: "That is how the platform earns trust."

3. Objection: "This will slow shared services."
- Response name: Scoped-Speed.
- Say: "Shared services get the fields and actions they need."
- Say: "They do not need unrestricted child access to move fast."
- Say: "The permit model removes ambiguity."

4. Objection: "We already have group reporting."
- Response name: Reporting-Is-Not-Permission.
- Say: "Reporting shows numbers."
- Say: "This model proves why the parent was allowed to see the numbers and what it did not see."
- Say: "That matters in audits and restructurings."

5. Objection: "Cedar policy will be too complex."
- Response name: Grants-First-Abstraction.
- Say: "Business users work with grants, purposes, fields, duration, and owners."
- Say: "Cedar enforces the decision."
- Say: "The workshop defines reusable patterns."

6. Objection: "Divestitures are handled by legal teams."
- Response name: Legal-Needs-System-Execution.
- Say: "Legal defines the transaction and transition services."
- Say: "Systems must execute access changes accurately."
- Say: "Revoke and regrant plans reduce stranded access."

7. Objection: "Our subsidiaries have different ERPs."
- Response name: Tenant-Projection.
- Say: "Different systems can remain behind child tenants."
- Say: "The parent consumes permitted projections."
- Say: "ERP sameness is not required for the first pilot."

8. Objection: "What about local regulation?"
- Response name: Pack-Locality.
- Say: "Local packs, like KR PIPA or GDPR, apply at the child tenant boundary."
- Say: "Parent grants must satisfy those pack conditions."
- Say: "Local review can be required."

9. Objection: "This sounds like matrix bureaucracy."
- Response name: Explicit-Beats-Informal.
- Say: "The bureaucracy already exists in email, spreadsheets, and exceptions."
- Say: "The model makes it explicit, reviewable, and automatable."
- Say: "That usually speeds the approved path."

10. Objection: "Can the parent override a child?"
- Response name: Override-As-Explicit-Grant.
- Say: "Any override must be modeled explicitly."
- Say: "Emergency or reserved rights can be configured, reviewed, and evidenced."
- Say: "They should not be invisible superuser access."

11. Objection: "What if a child refuses a needed grant?"
- Response name: Governance-Escalation.
- Say: "The operating model should define escalation."
- Say: "Oyatie can enforce the approved governance decision."
- Say: "The workshop identifies conflict paths."

12. Objection: "Audit exports will be hard across tenants."
- Response name: Dual-Sealed-Packets.
- Say: "Dual-sealed evidence preserves parent and child views."
- Say: "Exports are role-specific."
- Say: "The underlying trace links grants, permits, and actions."

13. Objection: "Our portfolio changes constantly."
- Response name: Lifecycle-Native.
- Say: "That is why grant lifecycle matters."
- Say: "Acquire, merge, divest, expire, and transition-service states are modeled."
- Say: "Static org assumptions are the risk."

14. Objection: "We need one master data model."
- Response name: Projection-Before-Uniformity.
- Say: "A single master model may be a long-term goal."
- Say: "The first step is permitted projection and evidence."
- Say: "Do not block governance improvement on perfect master data."

15. Objection: "Finance needs more detail."
- Response name: Purpose-Bound-Finance.
- Say: "Finance can get the detail required for approved finance purposes."
- Say: "Personal or unrelated operational data remains restricted."
- Say: "The grant defines that line."

16. Objection: "What happens during an emergency?"
- Response name: Emergency-Grant-Pattern.
- Say: "Emergency grants can be modeled with short duration, specific purpose, elevated approval, and mandatory review."
- Say: "Break-glass should be evidenced."
- Say: "It should not be permanent access."

17. Objection: "We use shared identity already."
- Response name: Identity-Is-Not-Authorization.
- Say: "Shared identity tells us who the actor is."
- Say: "Conglomerate grants tell us why that actor can cross a tenant boundary."
- Say: "Both are needed."

18. Objection: "This may be hard to explain to executives."
- Response name: Parent-Sees-Risk-Child-Keeps-Sovereignty.
- Say: "The executive message is simple."
- Say: "Parent sees permitted risk and performance."
- Say: "Child keeps sovereign control of restricted data."
- Say: "Every crossing is evidenced."

19. Objection: "We need to start small."
- Response name: One-Grant-One-Workflow.
- Say: "Start with one parent rollup, one child tenant, and one shared-service workflow."
- Say: "That is enough to validate the model."
- Say: "Expansion should be earned."

20. Objection: "Procurement will ask what this is."
- Response name: Conglomerate-Tenant-Architecture.
- Say: "The category is conglomerate tenant architecture for governed parent-child operations."
- Say: "It includes tenant hierarchy, grants, policy enforcement, evidence, and shared-service workflows."
- Say: "The pilot scope defines the commercial package."

## Closing Call to Action

- Say: "The demo showed the ADR-0313 pattern."
- Say: "Child tenants are sovereign."
- Say: "Parent access is explicit, scoped, and evidenced."
- Say: "Conglomerate grants define purpose, fields, duration, and owner."
- Say: "Cedar permits enforce the grant at runtime."
- Say: "Local packs such as KR PIPA can constrain parent access."
- Say: "Shared services can operate without unrestricted visibility."
- Say: "Divestiture is revoke and regrant, not manual cleanup."
- Say: "Dual-sealed audit preserves both parent and child perspectives."
- Say: "The recommended next step is a Conglomerate Tenant Architecture Workshop."
- Propose day 1: entity map and sovereignty requirements.
- Propose day 2: parent visibility needs and forbidden data.
- Propose day 3: first child grant design.
- Propose day 4: Cedar permit and pack constraints.
- Propose day 5: shared-service workflow map.
- Propose week 2 day 1: evidence packet design.
- Propose week 2 day 2: acquisition or divestiture simulation.
- Propose week 2 day 3: value model.
- Propose week 2 day 4: architecture, legal, and compliance review.
- Propose week 2 day 5: pilot recommendation.
- Ask: "Which child tenant and parent rollup should anchor the workshop?"
- Ask: "Which data must be explicitly forbidden from parent view?"
- Ask: "Can we schedule the entity-boundary session with corporate, legal, compliance, architecture, and one operating-company leader?"
- If they hesitate, offer a 60-minute grant-model whiteboard using synthetic tenants.
- If they agree, capture parent view, child entity, shared-service workflow, local compliance owner, and decision date.

## Pricing Conversation Anchors

- Anchor around governed group visibility, shared-services efficiency, and restructuring risk reduction.
- Explain that conglomerate tenant pricing usually has four components.
- Component 1: parent tenant platform subscription.
- Component 2: child tenant subscriptions or capability tiers.
- Component 3: conglomerate grants, evidence retention, and compliance pack activation.
- Component 4: implementation services for hierarchy, grant model, shared-service workflows, and restructuring simulations.
- Suggested workshop anchor: $90,000 to $180,000 depending on number of entities and legal/compliance complexity.
- Suggested first pilot anchor: $300,000 to $800,000 for one parent rollup, one to three child tenants, one shared-service workflow, and evidence review.
- Larger group rollout depends on subsidiaries, packs, shared services, and transaction lifecycle needs.
- Value lever: shared-service workflow efficiency.
- Value lever: duplicate subsidiary system avoidance.
- Value lever: group audit evidence reduction.
- Value lever: reduced access cleanup during acquisitions and divestitures.
- Value lever: local privacy incident risk reduction.
- Value lever: faster parent visibility during operational disruptions.
- Dollar anchor: "The demo base case showed 5.95 million dollars of first-year target value."
- Dollar anchor: "A single shared-service workflow can justify the first pilot if transaction volume is high."
- Dollar anchor: "A clean divestiture access model can protect far more value than the software cost."
- Do not price as one flat parent license if child sovereignty requires separate tenant boundaries.
- Do not promise legal sufficiency for restructuring without counsel.
- If CFO asks for chargeback, show the services ledger model.
- If CIO asks for architecture, show grants registry and tenant projection.
- If operating company president objects, emphasize child inspection and grant review.

## Follow-up Email Template

Subject: Follow-up from Oyatie conglomerate tenant demo

Hi {{first_name}},

Thank you for the conversation today. I heard the core need as parent-company visibility without erasing subsidiary sovereignty.

The demo used the `tenant-helios-fortune-500-manufacturer` fixture and showed:

- A parent tenant with sovereign child tenants.
- ADR-0313 parent-child access through explicit conglomerate grants.
- Parent aggregate risk visibility without unrestricted child detail.
- KR PIPA-constrained child access review.
- Shared-service invoice processing across a child tenant.
- Dual-sealed parent and child evidence packets.
- Acquisition onboarding with minimal initial grants.
- Divestiture revoke and regrant under a transition services agreement.
- Personal and work-data separation for parent finance.
- A first-year value model across shared services, audit, duplicate systems, and restructuring cleanup.

Recommended next step: a two-week Conglomerate Tenant Architecture Workshop.

Proposed outputs:

- Entity and tenant hierarchy map.
- Parent visibility requirements.
- Forbidden data list.
- First child grant model.
- Cedar permit model.
- Compliance pack constraints.
- Shared-service workflow design.
- Dual-sealed evidence packet.
- Acquisition or divestiture simulation.
- Value model and pilot recommendation.

Suggested attendees:

- Group CIO or architecture owner.
- Group COO or operating sponsor.
- Group compliance or legal owner.
- One operating-company president or delegate.
- Local privacy owner for an in-scope jurisdiction.
- Shared-services owner.
- Corporate development if restructuring is in scope.
- Finance owner for value and chargeback.

Could we reserve 60 minutes next week to select the first child tenant, parent rollup, and shared-service workflow?

Best,

{{sender_name}}

## References

- Internal: `registry/sample-tenants/tenant-helios-fortune-500-manufacturer.md`.
- Internal: `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- Internal: `docs/adr-archive/ADR-0314-marketplace-as-universal-deal-settlement.md
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/adr-archive/ADR-0243-cedar-as-universal-gate.md
- Internal: `docs/adr-archive/ADR-0244-tenant-as-universal-scoping-primitive.md
- Internal: `docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md
- Internal: `docs/adr-archive/ADR-0263-observability-emission-contract.md
- Internal: `specs/capability-tier-schema.json`.
- Internal: `specs/pack-overlay-schema.json`.
- Internal: `docs/COMPLIANCE-MATRIX.md`.
- External: Cedar policy language, https://www.cedarpolicy.com/.
- External: GDPR Regulation (EU) 2016/679, https://eur-lex.europa.eu/eli/reg/2016/679/oj.
- External: Personal Information Protection Act, Korea Law Translation Center, https://elaw.klri.re.kr/eng_service/lawView.do?hseq=53044&lang=ENG.
- Demo note: all tenant names, subsidiaries, grants, invoices, dollar examples, and restructuring events in this script are synthetic.
