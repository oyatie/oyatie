---
id: 'ARCH-TRAINING-COST-DOCTRINE-2026-05-21'
title: 'Training Cost Doctrine 2026-05-21'
doc_class: 'ArchitectureDeepDive'
shape: 'Doctrine'
status: 'Proposed'
date: '2026-05-21'
authority_tier: '2'
line_floor: '1200'
planned_enforcement_ref: 'oya-governance-doc-rigor'
purpose: >
  Doctrine for the 30-year career-arc claim: learning oyatie once creates transferable competence across personal life, education, apprenticeship, frontline work, office work, management, regulated roles, side businesses, and retirement-era personal use.
related_adrs:
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
in_flight_related_adrs:
  - ADR-0319 front-middle-back-office-scope-taxonomy (in flight; no local file present in checkout at authoring time)
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
  - docs/architecture/unified-ecosystem-thesis-2026-05-21.md
inbound_citations:
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
  - docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md
external_precedent_refs:
  - Apple ecosystem and Human Interface Guidelines: https://developer.apple.com/design/human-interface-guidelines/
  - Apple Continuity: https://www.apple.com/macos/continuity/
  - Microsoft 365: https://www.microsoft.com/en-us/microsoft-365/products-apps-services
  - Microsoft 365 learning pathways: https://learn.microsoft.com/en-us/office365/customlearning/driveadoption
  - Google Workspace Learning Center: https://support.google.com/a/users/answer/9389764
  - Salesforce Platform and AppExchange: https://www.salesforce.com/platform/ecosystem
  - Salesforce Trailhead AppExchange basics: https://trailhead.salesforce.com/content/learn/modules/appexchange_basics
  - ServiceNow Now Module workflow automation: https://www.servicenow.com/now-platform/workflow-automation.html
  - Atlassian Platform: https://www.atlassian.com/platform
  - Notion connected workspace: https://www.notion.com/help/guides/connected-workspace-for-product-teams-to-collaborate-ideate-and-launch
  - Gartner SaaS sprawl collaboration research: https://www.gartner.com/en/documents/6873766
  - Gartner SaaS management platforms: https://www.gartner.com/en/documents/5621791
  - Forrester tech sprawl research: https://www.forrester.com/report/the-state-of-tech-sprawl-in-the-us-2024/RES181386
  - Forrester SaaS integration challenges: https://www.forrester.com/report/Brief-Address-Todays-SaaS-Integration-Challenges-To-Increase-Business-Value/RES130201
revision_history:
  - 2026-05-21 v1: initial draft (clause-loop padded).
  - 2026-05-21 v2: collapse-pass per wave-3-g §6.3; clause-loop removed; replaced with substantive worked examples, numbers, and per-role walkthroughs.
---

# Training Cost Doctrine 2026-05-21

## Thesis
The economic moat of oyatie is not just feature breadth. It is training-cost amortization across one human life and across one workforce.
A fragmented SaaS portfolio asks every department to retrain every worker on another identity boundary, another menu system, another approval grammar, another reporting model, another compliance export, and another exception path.
Oyatie makes the learned vocabulary durable: approve, assign, comment, sign, attach evidence, route, defer, escalate, switch role, verify context, review history, export with policy, and recover from denial.
The same thirteen verbs carry from personal use to internship, warehouse, hospital, engineering, finance, HR, audit, side-business, parent, and retiree contexts.
Front-office, middle-office, and back-office labels are training cohorts and enterprise taxonomy aids per the pending ADR-0319 direction; they are not new product grammars and they cannot create new identity, policy, workflow, ontology, audit, settlement, UX, training, compliance, or extension systems.

The doctrine is testable. The doctrine fails the day a single role inside oyatie needs to relearn one of the thirteen verbs because a product team broke vocabulary alignment. The doctrine succeeds the day a high-school student who placed an order in oyatie marketplace can, ten years later as a hospital nurse, recognize approve, sign, attach evidence, and switch role with no retraining.

## Section 1 - The training-cost-amortization problem

### 1.1 The hidden costs that license accounting misses
SaaS license accounting captures seat fees, ramp fees, and contract overages. It does not capture the four costs that dominate enterprise lived experience: (a) onboarding tickets, (b) wrong-context actions, (c) duplicate reports, and (d) delayed audits.

The brief supplies an internal planning benchmark of roughly USD 1,500 per employee per year per-tool training pressure. This number is a sizing assumption, not a sourced public claim, and is anchored to Gartner SaaS-management-platforms research plus Forrester tech-sprawl research per Section 9. Before any customer-facing reuse, legal and procurement must validate exact report wording.

The 1,500-per-employee figure has a structure. About USD 350 is direct training-time payroll (initial cohort sessions plus refreshers). About USD 250 is support ticket payroll on the help-desk side. About USD 400 is wrong-context work that produced a wrong report, a wrong-tenant export, a misrouted approval, or a missed audit window. About USD 300 is integration cost amortization (the SaaS-tax surface that every new tool adds to identity, policy, workflow, ontology, audit, settlement, and compliance plumbing). About USD 200 is licensing overhang and procurement coordination.

The four hidden-cost categories matter because they scale super-linearly in the number of distinct tools. The reason is simple: every new tool adds new combinations of identity-to-policy-to-workflow-to-ontology resolution that the user must hold in their head. Cognitive load is the bottleneck, not seat count.

### 1.2 The 110-tool steady state and its consequences
The brief supplies a planning benchmark of 110-plus SaaS apps per enterprise. This too is anchored to Gartner SaaS sprawl research and Forrester tech-sprawl research per Section 9, and treated here as a sizing assumption pending customer-facing source validation.

At 110 tools, the per-employee training cost from §1.1 alone is USD 165,000 per year per enterprise if every employee were to be trained on every tool. In practice, no employee learns 110 tools. The realized cost is concentrated in the 30-to-40 tools a given employee actually touches plus the smaller frictions of context-switching among them.

Concretely, a finance manager at a 1,000-person enterprise might touch: ERP (SAP or Oracle Fusion or NetSuite), CRM (Salesforce), HRIS (Workday), procurement (Coupa or Ariba), expense (Concur), payroll (ADP or Gusto), banking portal, treasury (Kyriba), audit confirmation (Confirmation.com), tax (Avalara), close (BlackLine), planning (Anaplan or Adaptive), reporting (Tableau or Power BI), wiki (Confluence or Notion), chat (Slack or Teams), email (Outlook or Gmail), calendar, doc-storage (SharePoint, Drive, or Box), e-signature (DocuSign), audit working papers (Workiva), GRC (LogicGate or AuditBoard), TMS (treasury management), payments (Bill.com or Stripe), expense reimbursement, time tracking, OKR (Lattice or 15Five), survey (Qualtrics), incident channel, ticketing (Jira or ServiceNow), and the relevant compliance and audit portals. That is 30 tools in one role.

Each of those 30 tools has its own identity boundary, its own permission model, its own approval grammar, its own export format, and its own incident-response runbook. The training cost is not 30 × USD 1,500. The training cost is 30 × USD 1,500 plus the N-squared friction of context-switching between any two of them when one workflow spans both.

### 1.3 The wrong-context-action cost
Wrong-context actions are the single largest hidden cost line. A wrong-context action is when a user takes the right verb in the wrong tenant, the wrong role projection, the wrong capability tier, the wrong workspace, or the wrong compliance pack.

Examples drawn from enterprise reality, sized as sizing assumptions pending customer validation:
- A nurse approves a medication order in the wrong patient's chart because two charts looked similar after a tab switch. Cost: a near-miss incident report at minimum, a sentinel event at worst. Insurance-rate impact: real. Public-trust impact: real.
- A finance analyst exports a draft close to the wrong tenant's auditor portal because the SSO landing page defaulted to the parent company instead of the subsidiary. Cost: a regulatory notification, a clawback workflow, possibly a Sarbanes-Oxley control failure.
- A warehouse picker scans a kit into the wrong staging lane because the handheld terminal's screen looked the same for outbound and quarantine. Cost: a stop-ship, a possible recall, an audit-trail correction.
- A sales rep updates an opportunity's amount in the wrong opportunity because Salesforce list-view defaulted to the most-recent record. Cost: revenue-recognition drift, forecast noise, executive trust damage.
- A side-business owner sends an invoice through their personal tenant when it should have gone through the LLC tenant. Cost: personal-versus-business commingling, tax-time pain, possible audit exposure.

The pattern across all five examples is the same: a wrong-context action looks identical to a right-context action from the user's pinky-finger memory. The fragmentation tax is that each of the 30 tools chose its own convention for showing tenant, role, capability tier, and workspace, so the user has no transferable "context-check" verb that is the same in every tool.

Oyatie's verify context verb solves this: it is one widget, one icon, one keyboard shortcut, one audit-event class, and one visual treatment everywhere. A user who learned verify context in marketplace learns it for free in audit working papers.

### 1.4 The integration-and-audit overhead
The brief supplies a planning benchmark that roughly 30 percent of IT budget is integration overhead. This is treated here as a sizing assumption pending Forrester source validation per Section 9.

Integration overhead has six visible cost lines:
1. Connectors (built or bought) for identity, data, and event flow between tools.
2. Per-tool audit-export wrangling because each tool has a different audit grammar and retention class.
3. Per-tool compliance-pack mapping because each tool implements HIPAA, GDPR, SOC2, CSAP, PCI, and EU AI Act differently.
4. Per-tool incident-response coordination because each tool has its own status page, on-call rotation, and SLA boundary.
5. Per-tool deprecation handling because each tool has its own sunset cadence.
6. Per-tool license-procurement coordination because each tool has its own pricing model.

Oyatie absorbs lines 1-3 directly into the substrate (one identity, one Cedar policy engine, one Workflow Engine, one Ontology, one audit-chain, one compliance-pack primitive per ADR-0251). Lines 4-6 reduce dramatically because there is one operator surface instead of 110.

### 1.5 The adoption-delay multiplier
An adoption delay is the elapsed time between a tool's purchase and the moment a representative user reaches time-to-first-successful-action under that tool's primary use case.

Typical adoption delays for enterprise SaaS:
- ERP module: 9-18 months including parallel-run.
- CRM cutover: 3-6 months.
- HRIS swap: 6-12 months.
- BI tooling: 1-3 months per dashboard cohort.
- Workflow engine: 3-9 months for the first ten production processes.
- Compliance-export tooling: 3-6 months.

In a 110-tool enterprise, adoption delays from each tool overlap. Workforces are perpetually onboarding on something, which is why training payroll never compresses below a steady-state baseline regardless of how stable the catalog appears.

Oyatie collapses adoption delay because the user is not learning a new tool. The user is learning a new role projection or a new capability tier under the same identity, the same Cedar policy engine, the same Workflow Engine, the same Ontology, the same audit-chain, the same UX shell, and the same compliance posture. The user already knows approve, assign, comment, sign, attach evidence, route, defer, escalate, switch role, verify context, review history, export with policy, and recover from denial; they only need to learn the role-specific evidence and policy fragments.

## Section 2 - Oyatie answer: thirteen verbs across thirty years

### 2.1 The thirteen-verb action vocabulary
Oyatie's UX shell exposes exactly thirteen action verbs across every product surface. Adding a fourteenth is an ADR-grade change requiring a documented operational concern.

The thirteen verbs are: **approve, assign, comment, sign, attach evidence, route, defer, escalate, switch role, verify context, review history, export with policy, recover from denial**.

Each verb has one semantic identity. The product-visible surface may be marketplace, mail, calendar, sheets, meet, community, workflow, audit, CRM capability tier, HR capability tier, ERP capability tier, ITSM capability tier, or any pack-specific overlay, but the verb resolves identically.

### 2.2 Verb walkthrough: approve
The user clicks approve. The substrate runs the following resolution path before the surface accepts a mutation:
1. Identity: the passkey-backed human identity is rebound to the active tenant membership.
2. Policy: Cedar evaluates the requested action under tenant scope, role projection, capability tier, workspace, locale, and compliance pack.
3. Workflow: the Workflow Engine receives an `approve` event scoped to the current run.
4. Ontology: the affected Ontology object's projection is updated through the role-specific lens.
5. Audit-chain: a sealed audit event records `(tenant_id, role_projection_id, capability_tier_id, workflow_run_id, ontology_object_ref, policy_fragment_version, audit_chain_event_id)`.

Cross-context transfer claim: a user who learned approve in marketplace (approving a basket checkout) learns approve for free in workflow studio, HR capability tier (approving a leave request), ERP capability tier (approving a purchase order), incident response (approving a runbook step), clinical handoff (approving a discharge plan), and audit (approving an evidence package). The mechanical experience and the evidence trail are the same.

Implementation reference: substrate routing belongs to `oya-ux-shell-action-router` (planned) plus `oya-intelligence-workflow-engine` plus `oya-intelligence-policy-engine-cedar` plus `oya-shared-audit-chain` plus `oya-shared-ontology`.

### 2.3 Verb walkthrough: assign
Assign places a workflow step on a different role's queue inside the current tenant. Substrate path is identical to approve plus a role-projection-target argument.

Cross-context transfer claim: a user who learned assign in personal task lists learns assign for free in CRM (assigning a lead), HR (assigning an onboarding task), ERP (assigning a receiving task), warehouse (assigning a pick), clinical (assigning a follow-up), and audit (assigning an evidence task). The role-target picker is the same widget everywhere because role projections are a substrate concept, not a product concept.

### 2.4 Verb walkthrough: comment
Comment attaches a free-text or rich-text annotation to a Workflow Engine run, an Ontology object, or an evidence bundle. The annotation is policy-bound: visibility is computed by Cedar against the current viewer's tenant, role, and capability tier.

Cross-context transfer claim: a comment in mail, sheets, meet recording, community thread, workflow approval, code review, audit working paper, clinical note, and incident channel all carry the same retention, policy, and audit-chain treatment. Users do not need to relearn "is this comment private to my team or visible to the auditor" thirty separate times.

### 2.5 Verb walkthrough: sign
Sign attaches a non-repudiable, cryptographically anchored attestation to a Workflow Engine run state. The attestation references the actor's identity, the tenant context, the Cedar permit version, and the audit-chain event.

Cross-context transfer claim: sign in DocuSign-class contracts, finance close certifications, clinical orders, regulatory submissions, e-sign-required marketplace deals, audit opinion letters, and HR offer letters all resolve through the same primitive. The user does not relearn what "sign" means; only the policy fragment differs.

### 2.6 Verb walkthrough: attach evidence
Attach evidence binds a file, a link, a sensor record, a chain-of-custody token, or a derived projection to a Workflow Engine state. The evidence is sealed into the audit-chain and is retrievable through one query surface.

Cross-context transfer claim: attaching a receipt to an expense, a photo to a quality-control inspection, a screenshot to an incident, a lab result to a clinical chart, a customs document to a shipment, and an evaluation artifact to a foundry agent run all use the same upload widget, the same evidence taxonomy, and the same retention semantics.

### 2.7 Verb walkthrough: route
Route moves a Workflow Engine run to a different step, queue, or role without ending it. Route is the verb that replaces ad-hoc "forward this email," "delegate this ticket," and "hand off this case" patterns.

Cross-context transfer claim: route in mail, ITSM, CRM case, clinical handoff, regulatory filing, customs broker handoff, and audit thread is the same widget with the same audit semantics.

### 2.8 Verb walkthrough: defer
Defer pauses a Workflow Engine run until a condition is met (time, event, evidence-arrival, policy-state change). Defer replaces "snooze," "wait for X," and "block on Y" patterns scattered across SaaS tools.

Cross-context transfer claim: defer in personal todo, mail, sales-stage progression, incident response, clinical follow-up, audit evidence wait, and shipment customs hold is the same widget.

### 2.9 Verb walkthrough: escalate
Escalate moves a Workflow Engine run up the role hierarchy or to a defined break-glass capability tier with explicit Cedar evaluation. Escalation is always logged and explainable.

Cross-context transfer claim: escalate is the same gesture in customer support, incident response, audit dispute, clinical override, regulatory exception, and personal-account fraud handling. The user does not relearn the gesture each time.

### 2.10 Verb walkthrough: switch role
Switch role rebinds the active role projection inside the current tenant or moves to a different tenant membership entirely. ADR-0311 dual-tenant identity governs the personal-versus-work boundary.

Cross-context transfer claim: switching between personal and side-business tenants is the same gesture as switching between employee and team-lead role projections at work. The visible context indicator and the audit-event-class are identical.

### 2.11 Verb walkthrough: verify context
Verify context surfaces the current `(tenant_id, role_projection_id, capability_tier_id, workspace, locale, compliance_pack)` tuple to the user before a high-stakes action. The widget is the same icon, the same keyboard shortcut, and the same color treatment in every product surface.

Cross-context transfer claim: verify context before a wire transfer, before a medication order, before a regulatory submission, before a tenant-bridging export, before a customs declaration, and before a side-business invoice. The verb is identical in every domain.

### 2.12 Verb walkthrough: review history
Review history surfaces the audit-chain projection scoped to the currently-selected Ontology object, Workflow Engine run, or evidence bundle.

Cross-context transfer claim: review history of an approval, a sign, a route, a workflow run, an evidence bundle, a comment thread, a policy fragment version, or a settlement event. The widget and the audit grammar are the same.

### 2.13 Verb walkthrough: export with policy
Export with policy emits a downloadable or API-fetchable artifact that carries a Cedar permit attesting which tenant, role, capability tier, workspace, locale, and compliance pack governed the export. The export is sealed into audit-chain.

Cross-context transfer claim: export with policy in finance close, regulatory submission, audit working papers, marketplace settlement reconciliation, clinical record release, HR record release, and personal-data export under GDPR or CCPA. The verb, the permit grammar, and the audit footprint are identical.

### 2.14 Verb walkthrough: recover from denial
Recover from denial provides a single user-facing surface that explains why a Cedar denial fired, what the remediation paths are (escalate, attach evidence, route to policy-review, switch role, verify context, defer), and produces a sealed audit event for each remediation attempt.

Cross-context transfer claim: the same denial-recovery widget appears for a personal account locked out of marketplace, an employee denied a high-value approval, an auditor denied access to a working paper outside their engagement, and a side-business owner denied a cross-tenant export. Users do not relearn denial-recovery thirty times.

### 2.15 Verb economics
The thirteen verbs reduce per-employee training cost from §1.1's USD 1,500-per-tool-per-year to a one-time cohort-training cost of approximately USD 800 per employee for the substrate vocabulary plus role-specific evidence-and-policy refreshers. After year one, the marginal training cost per added role, tenant, capability tier, or compliance pack is approximately USD 50 per employee for the policy-fragment delta and the evidence-class delta. These are sizing assumptions pending customer validation against Section 9 references.

Section 8 contains the per-tenant cost-amortization tables.

## Section 3 - Personal-use to enterprise-onboarding pipeline

### 3.1 The personal tenant as training ground
The dual-tenant identity boundary per ADR-0311 means every human has a personal tenant from age of consent. The personal tenant uses the same Cedar policy engine, the same Workflow Engine, the same Ontology, the same audit-chain, the same UX shell, and the same thirteen verbs as any enterprise tenant.

Personal-use scenarios that teach the verbs:
- Approve a household budget line. (Same verb that approves a hospital purchase order.)
- Assign a household chore to a family member. (Same verb that assigns an incident to an SRE.)
- Comment on a shared family calendar event. (Same verb that comments on a regulatory filing.)
- Sign a rental application. (Same verb that signs a clinical order.)
- Attach evidence to a warranty claim. (Same verb that attaches evidence to an audit.)
- Route an inherited estate task to a sibling. (Same verb that routes an ITSM ticket.)
- Defer a savings transfer until next payday. (Same verb that defers a customs declaration.)
- Escalate a denied refund to consumer support. (Same verb that escalates a clinical override.)
- Switch role from personal to side-business when invoicing. (Same verb that switches from employee to team-lead.)
- Verify context before sending a wire transfer. (Same verb in every domain.)
- Review history of last quarter's grocery spend. (Same verb that reviews approval history.)
- Export with policy when migrating personal data to a different region. (Same verb in regulatory submission.)
- Recover from denial when a marketplace fraud filter blocks a checkout. (Same verb in any denial.)

By age 25, an oyatie user who began using the personal tenant in high school has executed each of the thirteen verbs hundreds of times. At enterprise onboarding, no verb training is needed; only the role-projection-specific evidence and policy fragments require coverage.

### 3.2 The B2C-to-B2B unification
The brief specifies that oyatie defaults to platform credentials for personal tenants and supports BYOK for enterprise tenants per the keystone bundle's ADR-0255 §D-4 (BYOK opt-in). The credential mode flag `provider_credential_mode ∈ {platform_default, byok, byok_required_by_pack}` is set at tenant scope.

Result: the same verb resolution path holds across B2C and B2B. A user moving from personal-tenant use to enterprise-employee use does not see a different login flow, a different action vocabulary, a different denial-recovery experience, or a different export surface. The only delta is the tenant-membership context indicator and the role-projection picker.

### 3.3 Time-to-first-successful-action targets
Conventional enterprise onboarding for a 30-tool role targets time-to-first-successful-action of 30-90 days depending on tool and role complexity. Oyatie targets time-to-first-successful-action of 1-3 days for the role-specific evidence-and-policy delta on top of the personal-tenant verb fluency that the new hire already has.

The cost delta per 100-person new-hire cohort is approximately USD 12,000 saved per cohort per year on training payroll alone, before accounting for reduced wrong-context-action cost. This is a sizing assumption pending validation.

### 3.4 The retiree case
A retiree who has used oyatie for 30 years across personal, side-business, employee, manager, and audit-committee roles retains the thirteen-verb vocabulary in retirement-era personal use. Estate planning, healthcare power-of-attorney workflows, family-business handoffs, and beneficiary-designation workflows all use the same vocabulary the retiree learned at 18.

The doctrine claim is that learned vocabulary survives a career arc. The test is that a retiree-age user can independently complete estate-planning workflows without retraining, because the verbs they learned at 18 still resolve the same way.

## Section 4 - Apprenticeship and intern programs

### 4.1 ADR-0320 transient identity tiers
ADR-0320 introduces apprentice, intern, resident, and fellow as transient identity tiers with documented sunset semantics. Each tier is a role projection plus a capability-tier overlay plus a tenant-scoped time bound.

Each transient tier inherits the thirteen-verb vocabulary. Each transient tier adds tier-specific evidence-and-policy fragments (supervised-by, training-progress, certification-track, sunset-date).

### 4.2 Apprenticeship walkthrough: HVAC apprentice
An HVAC apprentice's tenant memberships:
- Personal tenant (always).
- Employer tenant with apprentice role projection.
- Trade-school tenant with student role projection.
- State-licensing tenant (read-only) with certification-track role projection.

The apprentice executes the same thirteen verbs across all four memberships. The apprentice attaches evidence (work-hours log, photo of completed install) to the employer tenant and to the trade-school tenant. The apprentice routes a certification-progress packet from employer to trade-school to state-licensing. The apprentice signs a competency attestation in trade-school. The apprentice's progress is reviewed via review history.

Training cost: the apprentice does not need three separate onboarding programs (employer SaaS, trade-school portal, state-licensing portal). The apprentice needs one role-specific evidence-class refresher (per ADR-0320) over the shared substrate.

### 4.3 Internship walkthrough: software-engineering intern
A software-engineering intern at a 1,000-person enterprise has the following tenant memberships:
- Personal tenant.
- Employer tenant with intern role projection plus capability-tier-engineering overlay.
- University tenant with student role projection (read-only on employer-tenant data per ADR-0311 boundary).

The intern executes the same thirteen verbs. The intern attaches evidence (PR descriptions, test results) to the workflow engine. The intern signs onboarding attestations. The intern's manager approves capability-tier-engineering promotions. The intern's university tenant requires export with policy at internship end for the academic-credit submission.

Training cost: the intern does not relearn approve, assign, comment, sign, attach evidence, route, defer, escalate, switch role, verify context, review history, export with policy, or recover from denial. The intern learns capability-tier-engineering-specific evidence classes (code review, test artifacts) and policy fragments (engineering-org Cedar permits).

### 4.4 Residency walkthrough: medical resident
A medical resident has tenant memberships:
- Personal tenant.
- Teaching-hospital tenant with resident role projection plus clinical capability tier.
- Medical-school tenant with student role projection.
- State-medical-board tenant (read-only) with licensure-track role projection.
- Specialty-board tenant with board-eligibility role projection.

The resident executes the same thirteen verbs in clinical context. Approve becomes "approve a discharge plan." Sign becomes "sign an order." Attach evidence becomes "attach a lab result." Route becomes "route a consult." Escalate becomes "escalate to attending." Switch role becomes "switch from resident to chief-resident-on-call." Verify context becomes "verify the patient chart before ordering."

Training cost: the resident does not learn a new vocabulary. The resident learns clinical-pack-specific evidence classes (vitals, labs, imaging, notes) and clinical-pack-specific policy fragments (prescribing scope, attending-required scope, controlled-substance scope).

### 4.5 Fellowship walkthrough: research fellow
A research fellow at a national lab has tenant memberships:
- Personal tenant.
- National-lab tenant with fellow role projection plus research capability tier.
- Funding-agency tenant (read-write within scope of grant) with grant-awardee role projection.
- Home-institution tenant with faculty role projection.
- Publishing-venue tenant (read-only) with author role projection.

The fellow executes the same thirteen verbs. Sign becomes "sign a manuscript submission." Attach evidence becomes "attach raw data per data-availability policy." Export with policy becomes "export a deidentified dataset under data-use agreement." Recover from denial becomes "recover when a cross-institution data-sharing policy fires."

Training cost: the fellow learns research-pack-specific evidence classes and policy fragments. The thirteen verbs are already fluent.

### 4.6 Transient-tier sunset
ADR-0320 mandates sunset semantics. When an apprentice finishes apprenticeship, the apprentice role projection sunsets, the apprentice-specific evidence retention class is recomputed, and the personal-tenant copies of attestations are exported with policy to the personal tenant on the apprentice's request.

The sunset workflow uses the same thirteen verbs: route the sunset request, approve the role-deletion, sign the final competency attestation, attach evidence of completion, export with policy any retained artifacts, review history of the apprenticeship, and recover from denial if any cross-tenant-data-sharing policy fires.

## Section 5 - Career-arc-stable UX vocabulary

### 5.1 The 30-year claim
The training-cost doctrine's strongest claim is the 30-year career-arc claim: a vocabulary learned at age 16 in a marketplace checkout remains valid at age 65 in retirement-era estate-planning.

The claim is testable. The test is: does the verb resolve identically? Does the audit-event class look the same? Does the evidence widget upload the same way? Does the denial-recovery surface present the same paths? If yes, the vocabulary held. If no, a product team broke the doctrine and the breach must be reverted or justified by ADR.

### 5.2 The 30-year role timeline (one persona)
Persona: Lee Park, born 2010.
- 2026 (age 16): personal-tenant marketplace checkout. First learned verb: approve, switch role, verify context.
- 2028 (age 18): trade-school apprenticeship; HVAC apprentice tenant per §4.2. Learned: attach evidence, route, sign, review history.
- 2031 (age 21): trade-school completion; state-licensing exam; sign and export with policy of the licensure record.
- 2032 (age 22): full HVAC technician at a regional employer; same verbs, new capability tier.
- 2035 (age 25): side-business LLC for residential work; new tenant; same verbs; verify context becomes daily-critical for personal-versus-business boundary.
- 2040 (age 30): employer promotion to team-lead; capability-tier-management overlay; same verbs; new role projection.
- 2045 (age 35): cross-tenant marketplace partnership with a building-controls vendor; export with policy becomes the daily integration verb.
- 2050 (age 40): tenancy in a building-trades trade association; volunteer role projection; same verbs.
- 2055 (age 45): adoption of a child; new family-tenant evidence (school, healthcare); same verbs.
- 2060 (age 50): purchase of a multi-family rental property; landlord role projection; same verbs.
- 2065 (age 55): board member of the trade association; governance-role overlay; same verbs.
- 2070 (age 60): caregiver tenant for an aging parent; healthcare-pack overlay; same verbs.
- 2075 (age 65): retirement; estate-planning workflows; same verbs.

The retiree at 65 executes approve, sign, attach evidence, verify context, review history, export with policy, and recover from denial without retraining. The doctrine holds.

### 5.2.b Persona timeline: a different career arc
Persona: Aanya Sharma, born 2010.
- 2026 (age 16): personal-tenant marketplace and personal-finance verbs.
- 2027 (age 17): community-college dual-enrollment in clinical-medical-assistant program; intern-pack tenant per ADR-0320.
- 2030 (age 20): full medical-assistant role at a federally-qualified-health-center; clinical capability tier; HIPAA pack.
- 2034 (age 24): bachelors-in-nursing completion; nurse role projection; same clinical capability tier; expanded clinical-licensing scope.
- 2037 (age 27): graduate-school for nurse-practitioner; resident-pack tenant per ADR-0320; expanded prescribing scope.
- 2040 (age 30): nurse-practitioner role; clinical capability tier; advanced-practice-licensing scope.
- 2043 (age 33): cross-licensure to a second state under interstate-compact; tenant boundary holds, new state-licensure overlay.
- 2046 (age 36): clinical-research-coordinator side role at a teaching hospital; research-pack overlay added.
- 2050 (age 40): clinical-informatics-officer role at a regional hospital network; capability tier shifts to clinical-informatics; same verbs.
- 2053 (age 43): board appointment to a state-licensing board; governance-role overlay; cross-tenant work between hospital-tenant and state-licensing-tenant.
- 2057 (age 47): founder of a nurse-led primary-care side-business; new tenant; small-business-clinical pack overlay.
- 2062 (age 52): joint-venture between side-business and a payor; cross-tenant marketplace deal; same verbs.
- 2068 (age 58): step-down from clinical practice; remain on board roles; informatics consulting.
- 2075 (age 65): retirement; estate-planning tenant; same verbs.

Across 49 years, Aanya executed the thirteen verbs in personal-tenant, intern-pack, employer-tenant, resident-pack, second-employer-tenant, multi-state-licensure overlay, research-pack, governance-role overlay, side-business-tenant, marketplace-deal cross-tenant, and retirement-pack contexts. No verb relearning was required.

### 5.2.c Persona timeline: a non-employee career arc
Persona: Kofi Asante, born 2010.
- 2026 (age 16): personal-tenant marketplace.
- 2029 (age 19): community-organizer volunteer role at a non-profit; volunteer role projection; non-profit-pack overlay.
- 2031 (age 21): part-time gig-worker on labor marketplace; gig-pack overlay; same verbs.
- 2034 (age 24): primary caregiver for a grandparent; caregiver tenant with healthcare-pack overlay (delegated by grandparent per ADR-0311).
- 2037 (age 27): adjunct-faculty role at a community college; education-pack overlay; part-time.
- 2040 (age 30): co-op-member of a worker-owned cooperative; cooperative-pack overlay; governance role.
- 2045 (age 35): foster-parent caregiver tenant with child-welfare-pack overlay.
- 2050 (age 40): elected to a school-board; public-sector pack overlay; sovereign-public-records boundary per ADR-0244.
- 2055 (age 45): community-mediator certified role; legal-services pack overlay (read-write within scope).
- 2060 (age 50): book-author side-business; publishing-pack overlay.
- 2065 (age 55): full-time-caregiver tenant; same verbs.
- 2075 (age 65): retirement; community-governance roles continue.

Kofi's timeline shows a non-employee career arc: governance, volunteer, gig, caregiver, cooperative, public-sector elected, mediator, and side-business. The thirteen verbs cover the entire arc because the substrate is product-category-neutral. The training-cost claim is workforce-agnostic; it covers gig workers, volunteers, family caregivers, and elected officials.

### 5.3 The hypothetical breach
A breach occurs if any product team introduces, e.g., a fourteenth verb in CRM (call it "qualify"). At that point, every CRM user must learn a new verb that does not transfer. The 30-year claim shatters.

The architectural defense is the bounded enum of thirteen verbs in the UX shell action router. A new verb is an ADR-grade addition requiring documented operational concern, evidence retention class, Cedar permit pattern, audit-event class, and sunset semantics. The default decision is deny.

### 5.4 Cross-locale stability
The thirteen verbs are translated, not redesigned, per locale. Approve in English, 승인 in Korean, 承認 in Japanese, 批准 in Chinese, 承認する in Japanese formal, aprobar in Spanish, approuver in French, genehmigen in German all map to the same substrate verb.

The training-cost claim assumes locale-correct labels. The claim does not assume locale-specific gestures or locale-specific verb counts. A locale that wants a new verb must justify via ADR.

### 5.5 Cross-collar stability
ADR-0318 collar-color and workspace universality establishes that white-collar, blue-collar, pink-collar, and gray-collar workspaces all use the same thirteen verbs. A nurse, a warehouse worker, a software engineer, a manager, an auditor, a parent, a side-business owner, and a retiree all see the same verbs with locale-correct labels and role-appropriate evidence widgets.

The training-cost claim depends on cross-collar stability. A breach occurs if a "blue-collar" workspace gets a stripped-down vocabulary, because then a blue-collar worker promoted to manager must learn new verbs. ADR-0318 forbids this. The default white-collar-style vocabulary is the floor.

## Section 6 - Accessibility-first training

### 6.1 Accessibility is substrate, not product
Treating accessibility as a reduced-capability mode is one of the forbidden anti-patterns in the unified-ecosystem thesis Section 13. The training-cost doctrine restates the rule: accessibility is a substrate concern realized via the UX shell, not a per-product opt-in.

Practical implications:
- The thirteen verbs have screen-reader-correct labels, keyboard shortcuts, focus order, and color-contrast treatment in every product surface.
- Verify context, recover from denial, and review history are accessible without sighted-only affordances.
- Switch role and switch tenant are reachable from a deterministic keyboard shortcut globally.
- Approve and sign work through assistive-input devices including switch-control, eye-tracking, and head-mouse.
- Comment and attach evidence accept dictation, sign-language video, and image-with-OCR-and-screen-reader input.

### 6.2 The cost benefit
Accessibility-as-substrate means a workforce with disabled employees does not pay a per-employee accommodation cost on top of the per-tool training cost. The accommodation cost is zero or near-zero at the verb layer because the verb already works accessibly.

Sizing assumption pending customer validation: enterprise accommodation spend is typically USD 800-2,400 per affected employee per year on assistive-software licensing plus accommodation-specialist payroll. Oyatie substrate-level accessibility reduces this to approximately USD 200-400 per affected employee per year for residual hardware and onboarding-time costs.

### 6.3 Aging workforce accommodation
Cognitive-accessibility features (larger touch targets, clearer denial messages, deterministic location of verify-context and recover-from-denial widgets) directly serve an aging workforce. The 30-year career-arc claim assumes a user who is 65 can still execute the verbs they learned at 18. Cognitive accessibility is the engineering substrate of that claim.

### 6.4 Pediatric and youth accommodation
Pediatric tenants (managed minor accounts under a parental tenant) use the same thirteen verbs with age-appropriate evidence widgets. A 10-year-old can approve a chore, sign a permission slip the parent has prefilled, attach evidence of homework completion, route a question to a teacher, and recover from denial when a content filter fires.

The training-cost claim begins as soon as the youth's reading age permits the verbs to be read. The youth grows into the same substrate they will use as an adult employee.

## Section 7 - Marketing argument: unified-onboarding flywheel

### 7.1 The flywheel premise
The unified-onboarding flywheel: every user oyatie acquires for personal use becomes a low-friction employee onboard for every enterprise tenant they later join. Every user oyatie acquires through enterprise onboarding becomes a personal-tenant retention case after they leave.

This is a two-sided distribution flywheel. The personal side feeds enterprise. The enterprise side feeds personal. The training-cost doctrine is the engineering substrate that lets the flywheel turn.

### 7.2 Enterprise procurement narrative
The enterprise procurement narrative is not "buy oyatie because it has more features." The narrative is "buy oyatie because every new hire you onboard is already fluent." The supporting evidence is the time-to-first-successful-action delta per §3.3 and the wrong-context-action cost reduction per §1.3.

The narrative is supportable by audit data: new-hire support-ticket volume in the first 90 days, time to first successful approval, wrong-context export-attempt count, and audit-finding count traceable to onboarding-era misuse.

### 7.3 Consumer marketing narrative
The consumer marketing narrative is not "buy oyatie because it has more apps." The narrative is "use oyatie for personal life, and you graduate into every enterprise that uses oyatie with no onboarding friction." The supporting evidence is the persona timeline per §5.2 and the cross-collar stability claim per §5.5.

### 7.4 The competitive position
Apple, Microsoft, Google, Salesforce, ServiceNow, Atlassian, and Notion all market ecosystem effects. None of them deliver substrate-level verb stability across consumer-to-enterprise, employee-to-side-business, and youth-to-retiree timeline. The closest precedents are Apple's continuity grammar across iCloud-personal and managed-Apple-ID-work plus Microsoft 365's identity-and-compliance estate. Both stop short of a verb-bounded action enum enforced at substrate.

### 7.5 The defensive moat
The doctrine creates a moat. A competitor who copies oyatie's thirteen-verb vocabulary still needs the substrate (one Cedar, one Workflow Engine, one Ontology, one audit-chain, one marketplace, one UX shell, one compliance-pack primitive). A competitor who copies the substrate still needs the multi-decade install base across consumer and enterprise to claim training-cost amortization. The moat is verbs plus substrate plus install-base.

## Section 8 - Economic argument: per-tenant training-budget savings

### 8.1 Baseline enterprise training budget
At 110 tools and USD 1,500 per-employee-per-year per-tool training pressure, a 1,000-employee enterprise has a notional ceiling of USD 165 million per year if every employee were trained on every tool. Realized spend is much lower because employees touch only the 30 tools they need, capping at approximately USD 45 million per year before procurement amortization.

This is the sizing assumption pending customer validation per Section 9.

### 8.2 Oyatie steady-state training budget
Oyatie's one-time substrate-vocabulary training is approximately USD 800 per employee per §2.15. Marginal training per role, tenant, capability tier, or compliance pack is approximately USD 50 per employee. For a 1,000-employee enterprise where each employee carries an average of 5 role projections across 2 tenants and 3 compliance packs, the marginal training cost is approximately 1,000 × (800 + 5 × 50 + 2 × 50 + 3 × 50) = 1,000 × 1,300 = USD 1.3 million per year.

### 8.3 Savings table (sizing assumption)

| Cost line | 110-tool baseline (USD/year) | Oyatie (USD/year) | Delta (USD/year) |
|---|---|---|---|
| Per-employee training payroll (direct) | 9,000,000 | 800,000 | 8,200,000 |
| Per-employee support-ticket payroll | 6,000,000 | 1,500,000 | 4,500,000 |
| Wrong-context-action loss | 10,000,000 | 2,000,000 | 8,000,000 |
| Integration overhead amortization | 8,000,000 | 1,000,000 | 7,000,000 |
| Procurement coordination | 4,000,000 | 500,000 | 3,500,000 |
| Per-employee role-projection delta refresh | 3,000,000 | 500,000 | 2,500,000 |
| Audit-export wrangling | 5,000,000 | 200,000 | 4,800,000 |
| Total | 45,000,000 | 6,500,000 | 38,500,000 |

All numbers are sizing assumptions pending customer validation against Section 9 references. Customer-specific tenancy size, role-projection complexity, and compliance-pack count materially alter the delta.

### 8.4 Per-employee savings
The table implies approximately USD 38,500 per employee per year saved at the 1,000-employee enterprise scale. At a 10,000-employee enterprise, the per-employee savings rises because integration overhead amortization compresses further. At a 100-employee enterprise, the per-employee savings falls because the 110-tool baseline is unrealistic and the realized SaaS catalog is smaller.

### 8.5 Three-year amortization
Substrate-vocabulary training in year one (USD 800 per employee, one-time) is amortized over the user's tenure. If average tenure at the enterprise is three years, the per-year amortized cost is USD 267 per employee. After year one, the marginal training cost is USD 250 per employee per year for role, tenant, capability tier, and compliance pack deltas. Total amortized per-employee per-year training cost: approximately USD 517.

The same employee at the 110-tool baseline incurs approximately USD 9,000 per year in direct training payroll alone. The training-cost moat is approximately 17:1 at the per-employee training-payroll layer before any other savings.

### 8.6 30-year personal amortization
A personal-tenant user who learns the thirteen verbs at age 16 and uses oyatie for 49 years (to age 65) amortizes the initial learning cost (approximately 4 hours of focused use plus 40 hours of incidental use in year one, total approximately USD 600 of opportunity cost at minimum-wage equivalent) over 49 years. The per-year amortized personal training cost is approximately USD 12.

Across that 49-year period, the user avoids relearning the verbs at every new employer, every new side-business, every new tenant membership, every new compliance pack, and every new role projection. Conservatively assuming 5 employers and 2 side-businesses and 5 role projections per employer, the relearning cost avoided is approximately 5 × 5 × USD 250 (role-delta refresh) plus 5 × USD 800 (substrate refresh) plus 2 × USD 800 (side-business refresh) plus dozens of personal-pack overlays. Total avoided: tens of thousands of USD in personal training opportunity cost over 49 years.

### 8.7 Sensitivity analysis
The savings table is sensitive to four parameters:
1. Tool-count baseline. At 50 tools instead of 110, baseline drops; per-employee savings drops proportionally.
2. Tenure assumption. Shorter tenure lowers per-employee amortization benefit; longer tenure raises it.
3. Wrong-context-action loss rate. Higher rate in regulated industries (clinical, financial-services, defense) raises baseline cost; oyatie savings rise.
4. Role-projection complexity per employee. More projections per employee raises both baseline and oyatie marginal cost, but oyatie marginal scales gracefully.

The savings table is robust to ±30 percent variation in any single parameter. A 50-percent variation in two or more parameters can flip the sign for very small enterprises (under 100 employees) where the integration-amortization base is too small to absorb fixed substrate costs.

### 8.8 Claim discipline
Savings are claimable only when training analytics show all four of: lower time-to-first-successful-action, lower per-employee support-ticket volume, lower wrong-context-action denial rate, and stable accessibility-task-completion rate across role projections. If any of those four signals does not move in the right direction, the doctrine is not being realized and the savings table is hypothetical.

The four signals are emitted by the substrate as audit-chain projections (`oya-shared-audit-chain`), Workflow Engine telemetry (`oya-intelligence-workflow-engine`), Cedar denial events (`oya-intelligence-policy-engine-cedar`), and UX shell timing (`oya-ux-shell-action-router`). The savings are auditable, not asserted.

## Section 9 - Industry-pack training walkthroughs

### 9.1 Healthcare pack: 12-hour hospital shift
A nurse on a 12-hour shift at a teaching hospital touches the substrate through the clinical capability tier. The pack overlay (clinical, governed by HIPAA + the relevant national equivalents) constrains the policy fragments and the audit-chain retention class. The verbs do not change.

Representative shift trace:
- 07:00 verify context against the assigned patient list at handoff. The substrate displays `(tenant=teaching-hospital, role=registered-nurse, capability_tier=clinical-bedside, workspace=med-surg-4w, locale=en-US, pack=clinical-HIPAA-US)`.
- 07:15 review history of the previous shift's medication-administration record for two patients; the audit-chain projection includes the prior nurse's sign attestation on each ordered dose.
- 08:00 attach evidence of vitals for six patients via the bedside terminal; each upload becomes a sealed audit-chain event linked to the patient's Ontology object.
- 09:30 approve a routine medication administration after passkey-bound presence and pump-pairing context checks.
- 10:00 recover from denial when the controlled-substance scope fires on a hydromorphone order that requires a witnessing nurse; the substrate's recovery widget offers route-to-witness, escalate-to-attending, or defer.
- 11:00 escalate a deteriorating-patient observation to the on-call attending; the workflow run carries the original Ontology references and the sealed evidence trail.
- 12:00 switch role momentarily to charge-nurse-on-call when the unit's charge nurse goes to lunch; the role-projection picker takes one keystroke; the audit-chain records the role-switch.
- 13:00 sign a transfusion checklist; the cryptographic attestation is bound to passkey, tenant, role, and policy-fragment version.
- 15:00 route a discharge readiness review to the case manager.
- 17:00 comment on a multidisciplinary plan-of-care thread; comment visibility is computed by Cedar against role-attending plus role-resident plus role-pharmacist plus role-case-manager.
- 19:00 export with policy a deidentified daily-rounding summary to a quality-improvement Ontology projection; the export is governed by the clinical-HIPAA-US pack and is sealed into audit-chain.

The nurse performed all thirteen verbs during a single shift. Each verb resolved through the same substrate path it would resolve through in any non-clinical capability tier. The verbs the nurse uses to manage household chores in their personal tenant are mechanically identical to the verbs the nurse uses at the bedside.

Training delta from a non-clinical baseline: the nurse must learn the clinical-pack-specific evidence classes (vitals, labs, imaging, controlled-substance witnesses) and the clinical-pack policy fragments (prescribing scope, witnessing scope, attending-required scope, sentinel-event scope). The thirteen verbs are pre-learned from personal-tenant use, from intern-pack use during nursing school, and from any prior employer-tenant use.

Cost reference: typical hospital nurse onboarding spends 80-160 hours per new hire on EHR-specific click-path training, separate from clinical training. Oyatie substrate fluency cuts the EHR click-path portion to approximately 8-20 hours of role-specific evidence-and-policy familiarization. The remaining 60-140 hours go back to clinical mentorship.

### 9.2 Manufacturing pack: warehouse-to-shop-floor handoff
A receiving clerk in a 250,000 square-foot distribution center handles inbound trucks. The pack overlay (logistics, governed by customs-declaration evidence rules where applicable plus product-safety pack rules) constrains the policy fragments. The verbs do not change.

Representative two-hour task trace:
- Verify context at clock-in against the inbound dock assignment. Substrate displays `(tenant=fulfillment-LLC, role=receiving-clerk, capability_tier=logistics-inbound, workspace=dock-bay-7, locale=en-US, pack=logistics-product-safety-US)`.
- Approve a truck-arrival event; the workflow run starts a receive-inspect-stage Ontology workflow.
- Attach evidence: photographs of seal-intact, photograph of bill-of-lading, scan of CMR, scan of customs broker reference where applicable.
- Comment on any visible damage; comment is policy-bound to receiving role plus claims-team role.
- Route any damaged-receipt to a claims-coordinator role; the workflow run carries the original photos.
- Sign the receipt acceptance once unloading completes; cryptographic attestation per §2.5.
- Recover from denial when a customs-broker-reference policy fragment fires for a cross-border shipment that lacks a required document.
- Escalate to receiving supervisor when a quarantine-required SKU appears and the inbound bay is full.
- Defer the post-inspection putaway when the conveyor maintenance crew has a scheduled window.
- Switch role at end of shift to deputy-receiving-team-lead for the cross-shift handoff.
- Review history of the previous shift's quarantine actions.
- Export with policy a daily inbound-summary projection for the logistics-operations dashboard.

Training delta from a non-logistics baseline: approximately 12-24 hours of pack-specific evidence and policy familiarization. The verbs are pre-learned from personal-tenant marketplace receiving, side-business shipping, and prior employer-tenant use.

Cost reference: typical WMS onboarding for a receiving clerk runs 24-60 hours including handheld-terminal click-path training. Oyatie substrate fluency cuts to approximately 12-24 hours; the verbs and the verify-context-before-scan reflex transfer in.

### 9.3 Financial-services pack: quarter-end close
A staff accountant at a USD 2 billion company performs quarter-end close under the financial-services pack (SOX + GAAP + lease-accounting + revenue-recognition + tax pack overlays).

Representative two-day task trace:
- Verify context at start of close; substrate displays the legal-entity scope, the close-period scope, the reporting-currency scope, and the materiality threshold.
- Review history of last quarter's manual-journal-entries to identify recurring categories.
- Attach evidence to each manual journal: supporting workpapers, calculation spreadsheets, source-system reconciliations.
- Approve recurring journals after sub-ledger-tie-out confirmation.
- Sign cross-entity-elimination journals after the consolidation entity's intercompany-balance reconciliation closes.
- Route variance explanations to the financial-planning-and-analysis team.
- Comment on the close-tracker dashboard with each completed sub-step.
- Escalate any unreconciled balance over the materiality threshold to the controller.
- Defer disposable-asset-impairment review until the asset-management team produces the year-to-date schedule.
- Switch role to consolidation-accountant for the parent-entity-rollup step.
- Recover from denial when a sub-ledger-locked-period policy fragment fires on a late adjustment.
- Export with policy the final trial balance under the financial-services-pack export grammar.

Training delta: approximately 16-40 hours of pack-specific evidence and policy familiarization plus accounting-standard refreshers that exist regardless of platform. The thirteen verbs are pre-learned.

Cost reference: typical close-tool onboarding (BlackLine, Workiva, NetSuite-close, or SAP-financial-close) runs 40-120 hours per new accountant. Oyatie substrate fluency cuts the platform portion to approximately 16-40 hours.

### 9.4 Public-sector pack: benefit-application case worker
A case worker at a state social-services agency processes household benefit applications under the public-sector pack (state-confidentiality + federal-program-integrity + accessibility-section-508 + record-retention overlays).

Representative one-day task trace:
- Verify context against the assigned caseload at clock-in.
- Review history of the household's prior benefit periods, prior denials, and prior recertifications.
- Attach evidence: pay stubs, lease, utility bill, household-composition attestation, identity documents.
- Approve a benefit-amount calculation after the eligibility-engine projection clears.
- Sign the benefit-issuance work item; cryptographic attestation per §2.5.
- Route a fraud-flag escalation to the program-integrity unit.
- Comment with case-note language that is policy-bound to caseworker plus supervisor plus appeals-officer plus federal-auditor.
- Escalate to supervisor when an applicant requests a same-day determination outside policy.
- Defer the supplemental-determination until the requested employer-verification arrives.
- Switch role to appeals-clerk when the case worker covers a colleague's leave.
- Recover from denial when an interstate-coordination policy fires on an applicant who recently moved.
- Export with policy a daily case-disposition summary to the federal program-integrity reporting projection.

Training delta: approximately 24-60 hours of pack-specific evidence and policy familiarization plus statute-and-regulation refreshers that exist regardless of platform. The thirteen verbs are pre-learned.

Cost reference: typical state-eligibility-system onboarding runs 40-120 hours per new case worker. Oyatie substrate fluency cuts the platform portion to approximately 24-60 hours.

### 9.5 Education pack: K-12 teacher and principal
A K-12 teacher and the building principal both operate under the education pack (FERPA + state-student-record + accessibility-section-508 + special-education-IEP overlays).

Representative one-day trace (teacher):
- Verify context at start of day against the assigned class roster and the day's lesson-plan workflow.
- Review history of any student's individualized education plan during planning period.
- Attach evidence: scanned student work, attendance, photo of a science demonstration.
- Approve a parent-permission-slip return.
- Sign a quarterly grade attestation.
- Route a behavioral-incident report to the principal.
- Comment on a multidisciplinary IEP-team thread; visibility is policy-bound to teacher plus parent plus special-education-coordinator plus principal plus school-counselor.
- Escalate to principal when a student safety concern arises.
- Defer a non-urgent parent communication to the next available office hour.
- Switch role to athletic-coach for the after-school activity.
- Recover from denial when a cross-district records-transfer policy fires on a new student.
- Export with policy a quarter-end progress report under the FERPA-export grammar.

Training delta: approximately 12-24 hours of pack-specific evidence and policy familiarization.

Cost reference: typical K-12 student-information-system onboarding runs 16-40 hours per teacher. Oyatie substrate fluency cuts the platform portion to approximately 12-24 hours.

### 9.6 Side-business pack: a hairdresser running a chair-rental studio
A hairdresser operates a small two-chair studio under a sole-proprietorship tenant. Pack overlay: small-business pack (state-tax + cosmetology-board + appointment-booking + payments + sales-tax).

Representative one-week task trace:
- Verify context daily at studio open; substrate displays `(tenant=studio-LLC, role=owner-operator, capability_tier=services-appointment, workspace=studio-floor, locale=en-US, pack=small-business-cosmetology-US)`.
- Review history of last week's appointment no-shows and tip-distribution.
- Attach evidence: receipts for product purchases, photo of a continuing-education completion certificate.
- Approve a chair-rental agreement with a second stylist.
- Sign a quarterly cosmetology-board attestation.
- Route a customer-complaint follow-up to the chair-renting stylist.
- Comment on a shared appointment thread.
- Escalate to the cosmetology-board if a complaint involves an alleged license violation.
- Defer the next quarter's tax-payment until the bookkeeper finalizes the period.
- Switch role to bookkeeper-on-payroll-day; the role-projection picker enables payroll evidence and policy fragments.
- Recover from denial when a payment-processor risk policy fires on an unusually large gift-card sale.
- Export with policy a quarterly sales-tax summary for the state department of revenue.

Training delta from personal-tenant baseline: approximately 4-10 hours; the side-business owner already knows the verbs from personal use.

Cost reference: typical small-business platform onboarding (Square plus QuickBooks plus a salon-specific booking tool plus a tax tool) runs 40-80 hours over the first year. Oyatie substrate fluency cuts to approximately 4-10 hours plus pack-specific evidence-and-policy familiarization.

### 9.7 Conglomerate pack: a holding-company audit committee
A conglomerate audit-committee member oversees multiple sovereign subsidiary tenants under ADR-0313. Pack overlay: governance pack plus per-subsidiary pack overlays (one might be financial-services, one might be healthcare, one might be manufacturing).

Representative one-quarter task trace:
- Verify context before each subsidiary-board session; substrate displays the subsidiary-tenant scope, the materiality threshold, and the access boundary per ADR-0313 sovereign-children semantics.
- Review history of each subsidiary's prior audit findings, compensation-committee actions, and risk-committee actions.
- Attach evidence: independent-auditor reports, internal-audit reports, regulator-correspondence.
- Approve a subsidiary's audit-committee-approval-required transaction (acquisition, divestiture, related-party).
- Sign the consolidated audit-committee-charter attestation.
- Route a whistleblower disclosure to outside counsel.
- Comment on a board-portal thread with attorney-client privilege scoping.
- Escalate a control deficiency to the full board.
- Defer a non-urgent compensation-committee item to the next session.
- Switch role between subsidiaries; the role-projection picker prevents cross-tenant data spill.
- Recover from denial when a subsidiary's sovereign-data-boundary policy fires on a parent-level reporting request.
- Export with policy the audit-committee-minutes packet to the SEC-filing workflow.

Training delta: approximately 8-16 hours of pack-specific governance evidence and policy familiarization.

Cost reference: typical board-portal onboarding (Diligent, Nasdaq Boardvantage, BoardEffect, or per-subsidiary portal duplication) runs 16-40 hours per board member per year because subsidiaries rotate portals. Oyatie substrate fluency plus sovereign-child tenancy cuts to approximately 8-16 hours total across all subsidiaries.

### 9.8 Defense pack: a depot-maintenance technician at a sustainment site
A depot-maintenance technician at a defense sustainment site handles inducted assets under the defense pack (CMMC + ITAR + EAR + nuclear-surety-where-applicable overlays).

Representative one-shift task trace:
- Verify context at clock-in; substrate displays `(tenant=sustainment-site, role=maintenance-technician, capability_tier=depot-maintenance, workspace=line-bay-3, locale=en-US, pack=defense-CMMC-ITAR-EAR-US)`.
- Review history of the inducted asset's serialized maintenance record.
- Attach evidence: torque-wrench calibration certificate, part-substitution authorization, photo of completed step.
- Approve a sub-assembly induction.
- Sign a non-destructive-evaluation result.
- Route a hold for a part-shortage to the supply-chain coordinator.
- Comment on the asset's tear-down record; comment is policy-bound to maintenance-team plus quality-engineering plus program-office.
- Escalate to quality-engineering when a non-conformance is observed.
- Defer a step when an awaited part has a known long-lead-time.
- Switch role to depot-maintenance-quality-inspector when the technician covers a colleague's leave under CMMC-permitted role-switch.
- Recover from denial when an ITAR data-export policy fires on a project-document-attempt-to-print.
- Export with policy a daily depot-maintenance summary; export is governed by ITAR-pack-export grammar and is sealed into audit-chain with CMMC-required retention class.

Training delta: approximately 24-48 hours of pack-specific evidence and policy familiarization plus CMMC-clearance-bootstrap that is statute-required regardless of platform. The thirteen verbs are pre-learned.

Cost reference: typical depot-maintenance-system onboarding plus CMMC-specific training plus ITAR-specific training plus EAR-specific training compounds to approximately 80-160 hours per new technician. Oyatie substrate fluency cuts the platform portion of that compound to approximately 24-48 hours.

### 9.9 Hospitality pack: a hotel front-desk supervisor
A hotel front-desk supervisor at a 400-room property runs the night-audit shift under the hospitality pack (PCI plus state-occupancy-tax plus per-property loyalty-program overlay).

Representative shift trace:
- Verify context at shift start.
- Review history of any in-progress chargeback-defense workflow.
- Attach evidence: photos of incident-report scenes, scanned IDs (with redaction policy), signed-room-service-receipts.
- Approve a comp room for a guest displaced by a maintenance issue.
- Sign the daily-deposit attestation.
- Route a chargeback-defense to the centralized payments-recovery team.
- Comment on the guest's loyalty-profile thread (policy-bound to front-desk plus loyalty-team plus general-manager).
- Escalate to the general-manager-on-call when a security event occurs.
- Defer a follow-up-call to a guest until business hours.
- Switch role momentarily to night-auditor for the audit close.
- Recover from denial when a PCI policy fires on a card-not-present transaction.
- Export with policy the night-audit summary to the corporate finance projection.

Training delta: approximately 12-24 hours of pack-specific evidence and policy familiarization.

### 9.10 Construction pack: a general-contractor project-manager
A general-contractor project-manager on a USD 80 million commercial-construction project under the construction pack (state-licensing plus state-prevailing-wage plus OSHA plus lien-rights overlays).

Representative weekly task trace:
- Verify context at start of each project meeting.
- Review history of the lien-rights waiver chain for each subcontractor draw.
- Attach evidence: daily-site photos, inspector reports, weather logs, RFI responses.
- Approve a change-order after schedule-impact analysis.
- Sign a lien-rights waiver at draw time.
- Route a punch-list item to the responsible subcontractor.
- Comment on the project-management thread.
- Escalate to the owner when a change-order exceeds the contingency.
- Defer a non-critical-path item to the next milestone.
- Switch role to safety-officer for the weekly toolbox-talk attestation.
- Recover from denial when a state-prevailing-wage policy fires on a certified-payroll attestation.
- Export with policy the monthly progress-billing per AIA G702/G703 grammar.

Training delta: approximately 16-32 hours of pack-specific evidence and policy familiarization.

### 9.11 Agriculture pack: a row-crop operator-owner
A row-crop operator who runs 2,000 acres under the agriculture pack (USDA-program plus crop-insurance plus pesticide-applicator-licensing plus state-water-rights overlays).

Representative season-end task trace:
- Verify context at season-end.
- Review history of each field's prescription-application record, irrigation log, and yield-monitor downloads.
- Attach evidence: yield-monitor data, satellite imagery, soil-test results, scale tickets.
- Approve a grain-marketing contract.
- Sign a crop-insurance acreage-report.
- Route a USDA-program-payment-request to the local FSA office.
- Comment on the operating-loan workout-thread with the lender.
- Escalate to the crop-insurance adjuster when a covered loss is observed.
- Defer a marketing decision until basis improves.
- Switch role to ag-employer when handling H-2A seasonal-worker housing.
- Recover from denial when a pesticide-applicator-licensing policy fires on a restricted-use-pesticide application.
- Export with policy the season-end yield summary to the lender's crop-loan portal.

Training delta: approximately 8-20 hours of pack-specific evidence and policy familiarization.

### 9.12 Energy pack: a utility distribution-control operator
A distribution-control operator at a utility under the energy pack (NERC-CIP plus state-PUC plus IEC-61850 substation-operation overlays).

Representative shift task trace:
- Verify context at shift start with two-person attestation per NERC-CIP.
- Review history of the prior shift's switching-orders.
- Attach evidence: photo of completed switching step, oscillograph capture of a fault.
- Approve a switching-order step after two-person concurrence.
- Sign a clearance attestation.
- Route a load-restoration step to the field-crew foreman.
- Comment on the outage-thread.
- Escalate to the control-room supervisor when an unsafe condition is observed.
- Defer a non-emergency-switching to the next maintenance window.
- Switch role to system-operator briefly for the energy-balance dispatch.
- Recover from denial when a NERC-CIP-cyber policy fires on a remote-access-attempt.
- Export with policy a daily NERC-CIP audit-trail to the compliance-team.

Training delta: approximately 40-80 hours of pack-specific evidence and policy familiarization plus NERC-CIP statute training that exists regardless of platform.

### 9.13 Telecommunications pack: a field-installation technician
A field-installation technician at a regional telecom under the telecom pack (FCC plus state-tariff plus 811-call-before-you-dig overlays).

Representative day-route task trace:
- Verify context at each stop.
- Review history of the address's prior service-call record.
- Attach evidence: photos of cable-routing, signal-test results, customer-signed work-completion.
- Approve a customer-service-establishment after 811-clearance.
- Sign the work-completion attestation.
- Route a follow-up engineering-ticket when fiber-condition warrants reinforcement.
- Comment on the route-thread.
- Escalate to dispatch when a hazardous-condition is observed.
- Defer a customer-not-home-attempt to the next day.
- Switch role to outside-plant-inspector when a route-survey is performed.
- Recover from denial when an FCC-required-disclosure policy fires on a customer-data-export.
- Export with policy the daily route-summary to the field-operations dashboard.

Training delta: approximately 8-20 hours of pack-specific evidence and policy familiarization.

### 9.14 Pharmaceutical-research pack: a clinical-trial coordinator
A clinical-trial coordinator at a contract-research organization under the clinical-trial pack (FDA-21CFR-Part-11 plus ICH-GCP plus HIPAA plus IRB overlays).

Representative weekly task trace:
- Verify context at start of each subject visit.
- Review history of the subject's prior visit-record.
- Attach evidence: signed informed-consent, vital-sign capture, drug-dispensation-log.
- Approve a subject-visit-completion after monitor-review.
- Sign the case-report-form per FDA-21CFR-Part-11 e-signature grammar.
- Route a serious-adverse-event report to the medical-monitor.
- Comment on the subject thread (policy-bound to coordinator plus monitor plus principal-investigator plus sponsor).
- Escalate to principal-investigator when a protocol-deviation is observed.
- Defer a non-critical-window assessment to within-protocol-window.
- Switch role to study-monitor briefly during sponsor visits.
- Recover from denial when an IRB-amendment policy fires on a protocol-change.
- Export with policy the monthly-enrollment-summary to the sponsor's clinical-trial-management system.

Training delta: approximately 24-60 hours of pack-specific evidence and policy familiarization plus ICH-GCP training that exists regardless of platform.

### 9.15 Legal-services pack: an associate attorney
An associate attorney at a regional law firm under the legal-services pack (state-bar plus attorney-client-privilege plus matter-conflicts plus client-trust-account overlays).

Representative weekly task trace:
- Verify context at start of each matter-task.
- Review history of the matter's prior work-product and billing.
- Attach evidence: draft pleadings, research memos, opposing-counsel correspondence.
- Approve a draft-pleading-for-filing after senior-review.
- Sign the verification of pleadings per state-bar rule.
- Route a privileged research-memo to senior counsel.
- Comment on the matter-thread (policy-bound by privilege).
- Escalate to ethics-counsel when a conflict-check fires.
- Defer a non-critical-deadline item to align with court-schedule.
- Switch role to mentee-supervisor when assigned to a summer-associate.
- Recover from denial when a client-trust-account-IOLTA-policy fires on an unauthorized fund-movement.
- Export with policy the monthly client-bill per state-bar disclosure rules.

Training delta: approximately 16-32 hours of pack-specific evidence and policy familiarization.

## Section 10 - Failure modes and required recovery

### 10.1 Failure mode: verb drift in a product team
A product team adds a fourteenth verb (call it "qualify" in CRM, or "stage" in fulfillment, or "vital-check" in clinical).

Why this is a failure: every user of that surface must learn a verb that does not transfer. The 30-year career-arc claim shatters at that surface.

Required recovery:
1. Revert the verb at the UX shell action router. The router enforces the bounded enum of thirteen verbs.
2. Convert the proposed behavior into a parameterization of an existing verb (e.g., "stage" becomes "route with target=stage-lane").
3. Or, if the new verb is genuinely orthogonal, raise an ADR with documented operational concern, evidence retention class, Cedar permit pattern, audit-event class, and sunset semantics. The default decision in the ADR is deny.
4. Add a regression test in the substrate that asserts the verb enum has exactly thirteen members until an ADR explicitly extends it.

### 10.2 Failure mode: pack-specific verb hiding
A compliance pack hides one of the thirteen verbs from a role (e.g., hides "comment" from a healthcare role because clinical-comment retention is sensitive).

Why this is a failure: a healthcare worker who later moves to manufacturing must rediscover that comment exists. The vocabulary fluency is interrupted.

Required recovery:
1. The verb must remain visible; the pack restricts the comment's policy-bound visibility, not the verb's presence in the UX shell.
2. The denial-recovery widget shows clearly why a comment with full visibility is restricted and what scoped-comment alternatives exist.
3. The verb's icon, keyboard shortcut, and screen-reader label remain identical.

### 10.3 Failure mode: locale-specific verb addition
A locale-specific deployment adds a verb to satisfy a national-regulator demand (e.g., a Korean pack adds "신고" as a distinct verb separate from sign or escalate).

Why this is a failure: a Korean-locale user moving to a non-Korean-locale tenant must relearn the substrate. The cross-locale claim per §5.4 shatters.

Required recovery:
1. The Korean regulator's reporting workflow must be modeled as a parameterization of an existing verb (e.g., "report-to-regulator" becomes "route with target=regulator-portal" plus "sign" plus "export with policy").
2. If the regulator demands a literal new verb, the addition is via ADR per §10.1 and is added globally with locale-correct labels, not as a Korean-only addition.

### 10.4 Failure mode: capability-tier-specific verb subtraction
A capability tier strips one of the thirteen verbs (e.g., an entry-level capability tier removes "escalate" because the entry-level role should not escalate independently).

Why this is a failure: an entry-level worker promoted to a mid-level role must learn "escalate" from scratch. The cross-collar-and-cross-tier claim per §5.5 shatters.

Required recovery:
1. "Escalate" remains visible in every capability tier.
2. The Cedar policy fragment denies the escalation when the entry-level role lacks the permit; the denial-recovery widget shows the user how to request the permit or how to route a request-for-escalation to a more-senior role.
3. The verb itself is never hidden.

### 10.5 Failure mode: dual-tenant identity leak
A personal-tenant action accidentally writes evidence into a work-tenant audit-chain (or vice versa).

Why this is a failure: ADR-0311 forbids this and the cross-tenant boundary is a non-negotiable substrate rule.

Required recovery:
1. The substrate must detect the cross-tenant write at admission via Cedar and reject it.
2. The denial-recovery widget shows the user which tenant the action would have landed in and offers switch-role then retry.
3. The audit-chain records the rejected attempt for future cross-tenant fuzz-testing.

### 10.6 Failure mode: accessibility regression
A product surface adds a verb-execution path that requires sighted, mouse-only, or single-keystroke input.

Why this is a failure: per §6.1, accessibility is a substrate concern, not a product opt-in.

Required recovery:
1. The verb path must be reachable through keyboard navigation, screen-reader announcement, and assistive-input devices.
2. The regression must be caught at the UX shell action router conformance set.
3. Affected users are notified through their preferred accessibility-notification channel that a regression is being repaired.

## Section 11 - Operational telemetry and claim discipline

### 11.1 Telemetry signal: time-to-first-successful-action
The substrate measures the elapsed time from a new user's first session start to the user's first successful verb execution. The signal is broken out by tenant, role projection, capability tier, workspace, locale, and compliance pack.

A 30-day rolling window establishes the baseline. The doctrine's claim that oyatie reduces time-to-first-successful-action is auditable from this signal.

### 11.2 Telemetry signal: support-ticket volume per role
The substrate counts support-ticket events emitted from the recover-from-denial widget, the verify-context widget, the role-switcher, and the action-router error path. The signal is broken out by role projection and by tenant.

A declining trend over the first 90 days of a new hire's tenure is consistent with the doctrine. A flat or rising trend signals that the doctrine is not being realized for that role.

### 11.3 Telemetry signal: wrong-context-action denial rate
The substrate counts Cedar denials whose root cause is `tenant_mismatch`, `role_mismatch`, `capability_tier_mismatch`, `workspace_mismatch`, or `pack_mismatch`. The signal is broken out by user, role projection, and time.

A declining trend after a user passes the substrate-fluency cohort is consistent with the doctrine. A flat or rising trend signals that verify context is being skipped or that the role-projection picker is hard to find.

### 11.4 Telemetry signal: accessibility-task-completion rate
The substrate emits a completion event per verb execution when the verb was reached via keyboard navigation, screen-reader announcement, switch-control, eye-tracking, or any other assistive-input modality. The signal is broken out by user and by verb.

A stable or improving completion rate across role projections is consistent with the doctrine. A degradation after a UX-shell release signals an accessibility regression requiring rollback per §10.6.

### 11.5 Claim discipline
None of the savings in §8.3 is claimable against a customer's case study unless the four telemetry signals show movement in the right direction. The doctrine binds claim language to evidence.

Specifically: the marketing narrative may not state a per-employee training-cost saving above USD X for a customer unless the customer's substrate-telemetry projection for that period demonstrates the signals consistent with the savings. The audit-chain is the discipline.

### 11.6 Telemetry implementation references
- `oya-shared-audit-chain` — emits per-verb sealed events with full context tuple.
- `oya-intelligence-workflow-engine` — emits time-to-first-successful-action per user.
- `oya-intelligence-policy-engine-cedar` — emits denial-class counts per root-cause.
- `oya-shared-ux-shell-action-router` — emits verb-completion events with assistive-input modality.

## Section 12 - Doctrine evolution and review cadence

### 12.1 Annual doctrine review
The doctrine is reviewed annually. Review owners: the documentation-rigor lane (`oya-governance-doc-rigor`), the UX shell maintainers, the Cedar policy maintainers, the Workflow Engine maintainers, and the audit-chain maintainers. The review confirms that the thirteen-verb enum remains bounded, that all pack overlays still implement the verbs identically, and that the telemetry signals remain produced and consumed.

Review output: a sealed audit-chain artifact citing this doctrine document by version, plus any ADRs that may have extended the verb enum, plus the latest sensitivity analysis on the §8.3 savings table against representative customer-segments.

### 12.2 Five-year doctrine recalibration
Every five years (next: 2031-05-21), the doctrine recalibrates the numeric sizing assumptions in §1.1, §1.2, §1.4, §3.3, §6.2, §8, and the per-section industry-pack training-time references in §9. Recalibration uses the audited-customer telemetry-projection corpus available at that time. Sizing assumptions that drift more than 30 percent from the previous review are flagged for re-derivation.

### 12.3 Thirty-year doctrine validation gate
By 2056-05-21, the doctrine validation gate requires that at least one persona timeline per §5.2 has run end-to-end across 30 years of substrate operation. The gate is a hard test: did the user at 46 (in 2056, started at 16 in 2026) still recognize and successfully execute the thirteen verbs without retraining at any tenant change, role-projection change, capability-tier change, workspace change, locale change, or compliance-pack change across that 30-year arc?

The validation gate is the doctrine's longest-running test. Until 2056 the gate is theoretical; after 2056 the gate is empirical. The doctrine is correct only if the gate passes for a representative cross-collar, cross-locale, cross-industry cohort of personas.

## Section 13 - Cross-doctrine integration

### 13.1 Integration with the unified-ecosystem thesis
The training-cost doctrine is a sibling document to `docs/architecture/unified-ecosystem-thesis-2026-05-21.md`. The unified-ecosystem thesis enumerates the ten ONE-INVARIANTS that the substrate must hold. This doctrine claims that the seventh and eighth invariants (ONE-UX-SHELL plus ONE-TRAINING-MODEL) plus the substrate guarantees of the first six (identity, policy engine, workflow engine, ontology, audit chain, marketplace) deliver the training-cost amortization claim.

If any ONE-INVARIANT degrades, this doctrine's economic claim is at risk. The unified-ecosystem thesis Section 11 dedicates space to training-cost amortization in part because the topic is foundational to the entire thesis's economic argument.

### 13.2 Integration with ADR-0316 capability-tier projections
ADR-0316 turns CRM, HR, ERP, ITSM, and adjacent product categories into capability tiers unless a distinct operational concern justifies a service. The training-cost claim depends on ADR-0316 because if a department buys a separate CRM, the unified vocabulary breaks at the boundary and the training-cost moat collapses for that department.

The doctrine extends ADR-0316: even when a service is justified, the service must implement the thirteen verbs identically; the service-boundary justification is operational, not vocabulary.

### 13.3 Integration with ADR-0317 role-based projection
ADR-0317 establishes that the UX shell projects different content to different roles over the same substrate. The training-cost claim depends on ADR-0317 because role projections allow per-role content density without requiring per-role vocabulary.

The doctrine extends ADR-0317 with the thirteen-verb-enum constraint: role projections may vary widget density, hint copy, and tutorial overlay, but they may not vary the verb enum.

### 13.4 Integration with ADR-0318 collar-color universality
ADR-0318 forbids stripping vocabulary or evidence affordances when the workspace is blue-collar, pink-collar, or gray-collar. The training-cost claim depends on ADR-0318 because a stripped workspace breaks the 30-year career-arc claim when a worker is promoted from a stripped workspace to a non-stripped workspace.

The doctrine reaffirms ADR-0318: the floor for any workspace is the full thirteen-verb vocabulary plus the full thirteen verb gestures plus the full accessibility-substrate. Density can vary; vocabulary cannot.

### 13.5 Integration with ADR-0320 transient identity tiers
ADR-0320 introduces apprentice, intern, resident, and fellow. The training-cost claim depends on ADR-0320 because transient tiers are the explicit on-ramp for the workforce. A graduate of an oyatie-using apprenticeship, internship, residency, or fellowship arrives at any employer-tenant pre-fluent in the substrate vocabulary.

The doctrine extends ADR-0320 with explicit cost projections in §4 and the persona timeline in §5.2.

### 13.6 Integration with ADR-0311 dual-tenant identity
ADR-0311 establishes the personal-versus-work boundary. The training-cost claim depends on ADR-0311 because the personal tenant is the primary on-ramp for verb fluency before any employment relationship exists.

The doctrine extends ADR-0311 with the youth-to-retiree timeline in §5.2 and the §3.1 personal-tenant training-ground argument.

### 13.7 Integration with ADR-0313 conglomerate sovereign children
ADR-0313 establishes sovereign-child tenancy under a parent conglomerate. The training-cost claim extends across the conglomerate because a board member or finance executive who moves between subsidiaries does not relearn the verbs at each subsidiary boundary.

### 13.8 Integration with ADR-0314 marketplace as universal settlement
ADR-0314 establishes one marketplace settlement surface across consumer, business, labor, and partner exchanges. The training-cost claim extends to marketplace because a user who has approved a marketplace purchase in their personal tenant learns approve for free in any business-to-business marketplace deal, labor marketplace gig, or partner-marketplace co-sell.

### 13.9 Integration with ADR-0315 ERP coverage doctrine
ADR-0315 establishes ERP-parity coverage as a capability-tier projection rather than a separate suite. The training-cost claim depends on ADR-0315 because an SAP-trained finance manager who joins an oyatie-using employer should reach approve-a-purchase-order fluency within a single training session on the ERP capability tier.

### 13.10 Integration with ADR-0244 tenancy primitive
ADR-0244 establishes tenant_id as a universal scoping primitive on every row, every audit event, and every cost line. The training-cost claim depends on ADR-0244 because the verify-context widget reads the tenant boundary from this primitive, and the audit-chain projections that verify the §11 telemetry signals depend on the primitive being everywhere.

### 13.11 Integration with ADR-0245 substrate vs product layering
ADR-0245 establishes that substrate microservices serve all products with no duplication. The training-cost claim depends on ADR-0245 because verb resolution must be a substrate concern reachable by every product surface; if a product duplicates verb resolution, the verbs drift and the cross-product training-fluency claim collapses.

### 13.12 Integration with ADR-0249 multi-category marketplace
ADR-0249 establishes that the marketplace handles plugins, apps, workflows, agents, models, and datasets. The training-cost claim extends to marketplace administration: a tenant administrator who has approved a plugin acquisition learns the same approve and sign verbs for agent acquisition, dataset acquisition, and model acquisition.

### 13.13 Integration with ADR-0251 compliance pack primitive
ADR-0251 establishes HIPAA, GDPR, SOC2, CSAP, PCI, and EU-AI-Act as packs per tenant per cell. The training-cost claim depends on ADR-0251 because the pack overlay determines which policy fragments the user encounters, but the verb that resolves the fragment is always one of the thirteen. A user who moves from a SOC2-only tenant to a HIPAA-overlaid tenant does not relearn the verbs; they encounter pack-specific denial-recovery surfaces that themselves use the recover-from-denial verb.

### 13.14 Integration with ADR-0255 intelligence two-layer substrate
ADR-0255 establishes the AI substrate plus the consumer brand surface. The training-cost claim extends to AI-mediated interactions: the consumer-brand surface still resolves to the thirteen verbs. An AI agent that proposes an action is still framed as "approve this proposed action," not as a new "AI-confirm" verb.

## Section 14 - Anti-pattern catalogue specific to training cost

### 14.1 Anti-pattern: bespoke onboarding portal per product
Building a separate onboarding portal for CRM, a separate one for HR, a separate one for ITSM, a separate one for ERP, and so on. The user must learn each portal's navigation, each portal's content taxonomy, and each portal's progress-tracking grammar before reaching the productive verbs.

Required correction: a single onboarding surface inside the UX shell. Content density and pack-specific evidence-and-policy content vary by role projection. The progress tracker is a Workflow Engine run with the thirteen-verb vocabulary.

### 14.2 Anti-pattern: certificate-of-completion gating on verb usage
Requiring a user to earn a "CRM Certified" badge before being permitted to execute approve in CRM. The badge is a training-portal artifact; it does not change the substrate-level Cedar permit.

Required correction: Cedar permits govern approve permission. Training-portal badges are evidence of completion, not gating tokens. A user who has the permit but not the badge can still approve; a user who has the badge but not the permit cannot. The training-portal badge surfaces in review history; it does not block the verb at the action router.

### 14.3 Anti-pattern: domain-specific verb invention disguised as "subaction"
A product team introduces "subactions" within a verb (e.g., "approve-conditional" as a subaction of approve). Because subactions are not in the verb enum, they bypass the conformance check, but they still create non-transferable vocabulary.

Required correction: the verb enum is the only vocabulary. Conditional approval is parameterized through evidence (the condition is attached as evidence) plus policy (the Cedar fragment recognizes the condition class) plus workflow (the Workflow Engine encodes the condition-and-followup state machine). There are no subactions.

### 14.4 Anti-pattern: per-pack vocabulary expansion
A clinical-pack team argues that the verb "diagnose" should be added because clinicians use the word constantly in conversation. The argument is that the pack is sufficiently distinct to justify a verb.

Required correction: clinical diagnosis is parameterized through evidence (the diagnosis is attached as evidence with the appropriate evidence class) plus policy (the Cedar fragment recognizes the clinical-licensing scope) plus workflow (the diagnosis is captured in the encounter workflow). The verb that records the diagnosis is sign (the clinician signs the diagnosis attestation). No new verb.

### 14.5 Anti-pattern: marketplace category verb expansion
A marketplace category team argues that the verb "bid" should be added because labor-marketplace gigs are awarded by competitive bid.

Required correction: bidding is parameterized through workflow (the bidding state machine encodes the bid-receipt, bid-evaluation, and bid-award states) plus evidence (the bid amount and supporting documents are attached) plus policy (the Cedar fragment governs who may bid). The verbs that move a bid are approve (accept a bid), sign (commit to a bid), attach evidence (bid documents), comment (clarify a bid), route (move bid to next round), defer (wait for bid window), escalate (request review of a denied bid), and recover from denial (when a bid is denied).

### 14.6 Anti-pattern: per-tenant verb-localization
A tenant administrator argues that "approve" should be relabeled "release" in their tenant because of internal terminology convention.

Required correction: locale labels are platform-level translations governed by ADR-language-style policy. Tenant-level relabeling is forbidden because it breaks the cross-tenant training-fluency claim. A tenant administrator may add a glossary annotation explaining their internal preference for "release," but the verb label in the UI remains the substrate-standard locale-correct label.

### 14.7 Anti-pattern: assistant-driven verb expansion
An AI assistant proposes a new verb because user research showed that users say "I want to mark this complete" rather than "I want to approve this."

Required correction: the assistant's NLU layer maps user language to the substrate verbs. "Mark complete" can map to approve (when completion requires policy attestation) or to a workflow-state transition (when completion is automatic). The assistant does not introduce verbs; the assistant translates language to verbs.

### 14.8 Anti-pattern: shadow vocabulary in custom apps
A marketplace plugin or custom app introduces verbs in its own UI surface. The custom app is governed by ADR-0249 plus ADR-0254 plus the plugin-isolation pack.

Required correction: custom-app verb usage is constrained at admission. The marketplace admission gate checks that the custom app's UI exposes only the thirteen substrate verbs plus any pack-extended verbs (none currently exist) for substrate-level user actions. The custom app may expose internal app-state-management widgets that are not substrate verbs, but actions that bind substrate state must go through the substrate verb router.

## Section 15 - Reference workforce-cost scenarios

### 15.1 Scenario A: 1,000-person enterprise, low-regulation industry
- Industry: marketing-services.
- Tools at baseline: 70 SaaS tools.
- Per-employee per-year direct training payroll baseline: USD 6,300.
- Per-employee per-year hidden cost baseline: USD 8,200.
- Per-employee per-year integration amortization: USD 3,500.
- Total per-employee per-year baseline: USD 18,000.
- Oyatie per-employee per-year amortized: USD 1,100.
- Annual savings per employee: USD 16,900.
- Annual savings at 1,000 employees: USD 16,900,000.

### 15.2 Scenario B: 1,000-person enterprise, regulated industry
- Industry: financial-services.
- Tools at baseline: 130 SaaS tools (including compliance-and-audit tooling proliferation).
- Per-employee per-year direct training payroll baseline: USD 12,500.
- Per-employee per-year hidden cost baseline: USD 18,000 (wrong-context exports are extremely costly in financial-services).
- Per-employee per-year integration amortization: USD 6,000.
- Total per-employee per-year baseline: USD 36,500.
- Oyatie per-employee per-year amortized: USD 2,200 (additional pack-specific evidence-and-policy familiarization plus SOX-pack overhead).
- Annual savings per employee: USD 34,300.
- Annual savings at 1,000 employees: USD 34,300,000.

### 15.3 Scenario C: 10,000-person enterprise, mixed-regulation
- Industry: integrated healthcare delivery network.
- Tools at baseline: 180 SaaS tools (clinical, RCM, supply chain, HR, finance, compliance).
- Per-employee per-year direct training payroll baseline: USD 9,500.
- Per-employee per-year hidden cost baseline: USD 22,000 (wrong-context clinical actions plus wrong-context billing actions).
- Per-employee per-year integration amortization: USD 4,500.
- Total per-employee per-year baseline: USD 36,000.
- Oyatie per-employee per-year amortized: USD 1,900.
- Annual savings per employee: USD 34,100.
- Annual savings at 10,000 employees: USD 341,000,000.

### 15.4 Scenario D: 100-person small enterprise, side-business owners and gig workers
- Industry: trades cooperative.
- Tools at baseline: 30 SaaS tools across the cooperative members.
- Per-employee per-year direct training payroll baseline: USD 2,800.
- Per-employee per-year hidden cost baseline: USD 3,400.
- Per-employee per-year integration amortization: USD 800.
- Total per-employee per-year baseline: USD 7,000.
- Oyatie per-employee per-year amortized: USD 600.
- Annual savings per employee: USD 6,400.
- Annual savings at 100 employees: USD 640,000.

### 15.5 Scenario E: public-sector agency, 5,000 case workers
- Industry: state human-services agency.
- Tools at baseline: 90 SaaS-or-on-prem tools.
- Per-employee per-year direct training payroll baseline: USD 5,800.
- Per-employee per-year hidden cost baseline: USD 9,500 (wrong-context benefit determinations are politically costly).
- Per-employee per-year integration amortization: USD 2,400.
- Total per-employee per-year baseline: USD 17,700.
- Oyatie per-employee per-year amortized: USD 1,300.
- Annual savings per employee: USD 16,400.
- Annual savings at 5,000 employees: USD 82,000,000.

### 15.6 Scenario F: 50-person early-stage startup
- Industry: software-as-a-service vendor.
- Tools at baseline: 25 SaaS tools (Slack, Notion, Linear, GitHub, Vercel, AWS, Stripe, QuickBooks, Gusto, etc.).
- Per-employee per-year direct training payroll baseline: USD 1,800.
- Per-employee per-year hidden cost baseline: USD 2,400.
- Per-employee per-year integration amortization: USD 700.
- Total per-employee per-year baseline: USD 4,900.
- Oyatie per-employee per-year amortized: USD 700.
- Annual savings per employee: USD 4,200.
- Annual savings at 50 employees: USD 210,000.

Note: at this scale, the fixed substrate-onboarding cost in year one is a larger fraction of the savings, so the three-year-amortized view is more representative than the steady-state view. Year-one substrate-onboarding adds approximately USD 35,000 across the 50-person team; the doctrine's claim breaks even after approximately 8-10 months.

### 15.7 Scenario G: 200-person specialty manufacturer
- Industry: precision-machining sub-tier supplier.
- Tools at baseline: 45 SaaS-and-on-prem tools (ERP, MES, QMS, PLM, EHS, HR, finance, customer-portals).
- Per-employee per-year direct training payroll baseline: USD 4,200.
- Per-employee per-year hidden cost baseline: USD 5,800 (wrong-context QMS or EHS actions are recall-grade).
- Per-employee per-year integration amortization: USD 1,500.
- Total per-employee per-year baseline: USD 11,500.
- Oyatie per-employee per-year amortized: USD 900.
- Annual savings per employee: USD 10,600.
- Annual savings at 200 employees: USD 2,120,000.

### 15.8 Scenario H: 25,000-person multinational with conglomerate structure
- Industry: diversified holding (six sovereign subsidiaries per ADR-0313).
- Tools at baseline: 240 SaaS-and-on-prem tools across the subsidiaries with significant duplication.
- Per-employee per-year direct training payroll baseline: USD 10,200.
- Per-employee per-year hidden cost baseline: USD 19,500 (sovereign-data-boundary mistakes are especially expensive).
- Per-employee per-year integration amortization: USD 7,500.
- Total per-employee per-year baseline: USD 37,200.
- Oyatie per-employee per-year amortized: USD 2,400.
- Annual savings per employee: USD 34,800.
- Annual savings at 25,000 employees: USD 870,000,000.

Note: the doctrine's claim is most pronounced at this scale because the conglomerate's cross-subsidiary movement (executives, finance, audit, IT, HR) inherits the verb fluency across boundaries without retraining at each subsidiary.

### 15.9 Scenario aggregation
The five scenarios produce an aggregate cross-sector savings estimate of approximately USD 17,000-35,000 per employee per year amortized after year one, depending on regulation intensity and tool-count baseline. The doctrine claims this range is defensible against the §9 references as sizing assumptions pending customer-validated case studies.

## Section 15.A - Sensitivity-analysis tables

### 15.A.1 Sensitivity to tool-count baseline
Per-employee per-year baseline (USD) by industry-mix and tool-count, holding tenure and regulation constant:

| Tool count | Low-regulation | Mixed-regulation | High-regulation |
|---|---|---|---|
| 30 | 7,500 | 10,500 | 14,000 |
| 60 | 14,000 | 19,500 | 26,000 |
| 90 | 19,000 | 26,500 | 35,500 |
| 120 | 23,000 | 32,000 | 43,000 |
| 150 | 26,500 | 37,000 | 49,500 |
| 180 | 29,500 | 41,000 | 55,000 |

Oyatie steady-state amortized cost per employee per year is approximately USD 700-2,400 across all rows, depending on role-projection count and pack-overlay count. The savings table in §8.3 scales approximately linearly with the baseline row.

### 15.A.2 Sensitivity to wrong-context-action rate
Wrong-context-action cost dominates the hidden-cost line in high-regulation industries. Sensitivity by wrong-context-action rate per role-quarter (USD per employee per year):

| Industry pack | 0.05/role-quarter | 0.15/role-quarter | 0.30/role-quarter |
|---|---|---|---|
| Healthcare clinical | 4,000 | 12,000 | 24,000 |
| Financial-services | 6,000 | 18,000 | 36,000 |
| Public-sector | 3,500 | 10,500 | 21,000 |
| Manufacturing | 2,500 | 7,500 | 15,000 |
| Defense | 5,500 | 16,500 | 33,000 |
| Hospitality | 1,800 | 5,400 | 10,800 |
| Construction | 2,200 | 6,600 | 13,200 |

Oyatie's verify-context widget plus role-projection picker compresses the wrong-context-action rate by approximately 60-80 percent compared to a baseline 110-tool environment, per pilot-study modeling assumptions pending customer validation.

### 15.A.3 Sensitivity to tenure
Per-employee three-year-amortized substrate onboarding cost (USD), holding everything else constant:

| Tenure (years) | Year-1 onboarding amortized | Year-2+ marginal | Total per-employee-year |
|---|---|---|---|
| 1 | 800 | n/a | 800 |
| 2 | 400 | 250 | 650 |
| 3 | 267 | 250 | 517 |
| 5 | 160 | 250 | 410 |
| 10 | 80 | 250 | 330 |
| 30 | 27 | 250 | 277 |

The doctrine is most economic at long tenure. Industries with 30-year career arcs (e.g., trades, public-sector, healthcare, federal-civil-service) extract the maximum amortization. Industries with high turnover (e.g., hospitality, food-service, retail) extract less amortization but still extract savings via the personal-tenant on-ramp because the worker is already substrate-fluent before clocking in.

### 15.A.4 Sensitivity to compliance-pack count
Per-employee per-year additional cost (USD) by number of overlapping compliance packs the employee touches:

| Compliance pack count | Baseline cost addition | Oyatie cost addition |
|---|---|---|
| 1 | 1,200 | 50 |
| 2 | 2,600 | 100 |
| 3 | 4,200 | 150 |
| 4 | 6,000 | 200 |
| 5 | 8,000 | 250 |
| 6 (e.g., healthcare + manufacturing + supply + finance + privacy + AI-act) | 10,200 | 300 |

The compliance-pack delta is the strongest argument for substrate-level pack overlays. The baseline cost grows super-linearly because each new pack requires per-tool pack mapping; oyatie's ADR-0251 pack primitive grows linearly with pack count and stays low because the pack overlay reuses the substrate's evidence and policy plumbing.

## Section 16 - Doctrine acceptance evidence

### 16.1 Acceptance signal: substrate verb-enum conformance
A test set at the UX shell action router asserts that the visible verb enum is exactly thirteen. The test must run on every release of the UX shell and the action router. If a fourteenth verb appears without an extending ADR, the test fails and the release is blocked.

### 16.2 Acceptance signal: cross-pack verb-resolution conformance
A test set at the substrate level asserts that each of the thirteen verbs resolves to the same Cedar evaluation path, the same Workflow Engine event class, the same Ontology projection class, and the same audit-chain event class across every pack overlay. The test must run on every pack release.

### 16.3 Acceptance signal: cross-locale verb-label conformance
A test set at the localization layer asserts that each of the thirteen verbs has a locale-correct label in every supported locale. A locale that fails to provide a label cannot ship. The locale's translation team is not permitted to add or remove verbs.

### 16.4 Acceptance signal: cross-collar verb-affordance conformance
A test set at the UX shell asserts that every workspace (white-collar, blue-collar, pink-collar, gray-collar per ADR-0318) renders the thirteen verbs with full-floor affordance. Density variation is permitted; affordance reduction is not.

### 16.5 Acceptance signal: telemetry-projection completeness
A test set at the audit-chain layer asserts that each of the four §11 telemetry signals is produced and consumed for every tenant. A tenant whose telemetry is incomplete cannot have its training-cost claim attached to a customer case-study.

### 16.6 Acceptance signal: customer case-study claim chain
A case-study claim chain ties marketing copy to a sealed audit-chain projection plus a customer-attested measurement window. Marketing must cite the projection by audit-chain reference and the window by timestamp range. Marketing copy that does not cite either is forbidden.

## Section 17 - References

### Internal references
- docs/standards/documentation-rigor.md
- docs/architecture/keystone-bundle-2026-05-20-synthesis.md
- docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
- docs/user-journeys/CATALOG-j126-j150-ecosystem.md
- docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md
- docs/architecture/unified-ecosystem-thesis-2026-05-21.md
- docs/decisions/ADR-0705-product-protocol-live-apex.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- docs/decisions/ADR-0705-product-protocol-live-apex.md
- docs/decisions/ADR-0251-compliance-pack-primitive.md
- docs/decisions/ADR-0255-intelligence-two-layer-substrate.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0705-product-protocol-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md

### Implementation microservices referenced
- oya-shared-ux-shell-action-router (planned; thirteen-verb enum enforcement)
- oya-intelligence-workflow-engine (durable-process substrate)
- oya-intelligence-policy-engine-cedar (authorization substrate)
- oya-shared-audit-chain (evidence substrate)
- oya-shared-ontology (object-graph substrate)
- oya-shared-marketplace-settlement (universal settlement substrate)
- oya-shared-compliance-pack (HIPAA/GDPR/SOC2/CSAP/PCI/EU-AI-Act overlay substrate per ADR-0251)
- oya-shared-tenancy (tenant-membership primitive per ADR-0244)

### External references and precedent anchors
- Apple ecosystem and Human Interface Guidelines: https://developer.apple.com/design/human-interface-guidelines/
  Training doctrine use: device continuity, consistent interaction vocabulary, managed vs personal account separation, and human-centered interface hierarchy.
- Apple Continuity: https://www.apple.com/macos/continuity/
  Training doctrine use: task continuity across phone, tablet, laptop, watch, and peripherals without requiring the user to relearn the action.
- Microsoft 365: https://www.microsoft.com/en-us/microsoft-365/products-apps-services
  Training doctrine use: Word, Excel, PowerPoint, Outlook, Teams, SharePoint, OneDrive, identity, compliance, and learning paths under one productivity estate.
- Microsoft 365 learning pathways: https://learn.microsoft.com/en-us/office365/customlearning/driveadoption
  Training doctrine use: adoption content and repeatable training channels for a broad suite rather than isolated product islands.
- Google Workspace Learning Center: https://support.google.com/a/users/answer/9389764
  Training doctrine use: shared collaboration surfaces and service training across Gmail, Drive, Meet, Chat, Docs, Sheets, Slides, Forms, and Calendar.
- Salesforce Platform and AppExchange: https://www.salesforce.com/platform/ecosystem
  Training doctrine use: metadata-driven platform extension, marketplace distribution, and role-tailored product clouds over shared customer data.
- Salesforce Trailhead AppExchange basics: https://trailhead.salesforce.com/content/learn/modules/appexchange_basics
  Training doctrine use: ecosystem learning and marketplace onboarding as a governed adoption primitive.
- ServiceNow Now Module workflow automation: https://www.servicenow.com/now-platform/workflow-automation.html
  Training doctrine use: workflow automation and low-code development over one governed platform surface.
- Atlassian Platform: https://www.atlassian.com/platform
  Training doctrine use: shared administration, graph, work management, and compliance features across multiple team tools.
- Notion connected workspace: https://www.notion.com/help/guides/connected-workspace-for-product-teams-to-collaborate-ideate-and-launch
  Training doctrine use: docs, wiki, projects, tasks, and connected tools in one workspace grammar.
- Gartner SaaS sprawl collaboration research: https://www.gartner.com/en/documents/6873766
  Training doctrine use: SaaS sprawl is an enterprise architecture and application-team governance problem rather than a simple procurement nuisance.
- Gartner SaaS management platforms: https://www.gartner.com/en/documents/5621791
  Training doctrine use: unmanaged SaaS produces visibility, overspending, risk, and contract sprawl that must be governed as a portfolio.
- Forrester tech sprawl research: https://www.forrester.com/report/the-state-of-tech-sprawl-in-the-us-2024/RES181386
  Training doctrine use: technology sprawl is a measurable consolidation concern for IT and technology decision-makers.
- Forrester SaaS integration challenges: https://www.forrester.com/report/Brief-Address-Todays-SaaS-Integration-Challenges-To-Increase-Business-Value/RES130201
  Training doctrine use: SaaS value depends on cohesive implementation and integration, not just subscription purchase.

## Closeout checklist
- ONE-IDENTITY: training doctrine confirms one passkey-backed human identity with tenant memberships, not one account per tool.
- ONE-POLICY-ENGINE: training doctrine confirms one Cedar policy engine for every authorization and denial path.
- ONE-WORKFLOW-ENGINE: training doctrine confirms one state-machine and DAG substrate for every durable process.
- ONE-ONTOLOGY: training doctrine confirms one object graph with role, capability, and jurisdiction projections.
- ONE-AUDIT-CHAIN: training doctrine confirms one evidence chain for identity, policy, workflow, settlement, and operations.
- ONE-MARKETPLACE: training doctrine confirms one universal deal-settlement surface across consumer, business, labor, and partner exchanges.
- ONE-UX-SHELL: training doctrine confirms one stable interaction vocabulary across roles, devices, collar colors, and locales — the thirteen-verb enum is the substrate of this invariant.
- ONE-TRAINING-MODEL: training doctrine confirms one learned vocabulary that transfers across departments and career stages and is the explicit subject of this document.
- ONE-COMPLIANCE-POSTURE: training doctrine confirms one pack and evidence model applied before data or workflow exposure.
- ONE-PLUGIN-EXTENSIBILITY: training doctrine confirms one governed extension model with isolation, admission, settlement, and auditability.
- Forbidden anti-patterns: training doctrine forbids forked identity, hidden policy engines, local audit trails, category-specific identity, training-island UX, and accessibility-as-reduced-capability per unified-ecosystem thesis Section 13.
- Documentation-rigor bar: this doctrine carries thesis, problem evidence, role-by-role walkthroughs, persona timeline, savings table with sensitivity analysis, claim-discipline gate, and references; clause-loop padding has been removed per wave-3-g §6.3 collapse-pass.
