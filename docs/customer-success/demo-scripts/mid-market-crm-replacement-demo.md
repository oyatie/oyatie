---
doc_class: DemoScript
target_persona: Mid-market CRO, VP Sales, RevOps leader, CCO, CTO, sales operations admin
duration_minutes: 45
related_oyatie_adrs:
  - ADR-0219
  - ADR-0220
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0316
  - ADR-0317
status: Published
date: 2026-05-20
owner: customer-success
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Mid-Market CRM Replacement Demo

## Pre-Demo Checklist
- Demo objective: prove Oyatie can replace Salesforce or HubSpot for a mid-market company without recreating CRM platform sprawl.
- Prospect profile: 300 to 2,000 employee B2B company outgrowing HubSpot or over-customized Salesforce.
- Named tenant fixture to use: `tenant-acme-mid-market-saas`.
- Fixture source: `registry/sample-tenants/acme-mid-market-saas.md`.
- Primary persona to impersonate: Saanvi Mehta, VP Product and revenue workflow sponsor.
- Secondary persona to impersonate: Diego Vargas, CTO and security owner.
- Tertiary persona to impersonate: Naveen Iyer, CCO and evidence owner.
- Prepare tool: Oya Demo Launcher.
- Prepare tool: Tenant Fixture Loader.
- Prepare tool: Sales Capability Tier Console.
- Prepare tool: CRM Migration Mapper.
- Prepare tool: Account Ontology Explorer.
- Prepare tool: Deal Workflow Replay.
- Prepare tool: Campaign Consent Console.
- Prepare tool: Workflow Studio.
- Prepare tool: Cedar Policy Simulator.
- Prepare tool: Audit Chain Explorer.
- Prepare tool: RevOps Dashboard.
- Prepare tool: Pack Evidence Exporter.
- Prepare tool: Pricing and Packaging Model.
- Load Acme fixture before the meeting.
- Confirm active cells are `us-central-silver-a` and `eu-west-silver-a`.
- Confirm active packs are GDPR and SOC2 Type II.
- Confirm intentionally absent packs include HIPAA, DORA, KR-PIPA, and EU AI Act.
- Confirm non-subscribed services are visible as absent, including treasury and regulated healthcare surfaces.
- Open screen 1: `RevOps Dashboard > Acme`.
- Open screen 2: `Sales Capability Tier Console > CRM Core`.
- Open screen 3: `CRM Migration Mapper > Salesforce and HubSpot`.
- Open screen 4: `Workflow Studio > Renewal Risk Playbook`.
- Open screen 5: `Audit Chain Explorer > Acme CRM Events`.
- Prepare competitor framing: Salesforce is powerful but often becomes admin-heavy.
- Prepare competitor framing: HubSpot is fast to start but can fragment under enterprise controls.
- Prepare Oyatie framing: CRM is a capability tier over shared identity, ontology, workflow, policy, audit, mail, messenger, and analytics.
- Prepare dollar example: $420K annual CRM license and add-on reduction.
- Prepare dollar example: $310K annual RevOps admin drag reduction.
- Prepare dollar example: $1.1M pipeline acceleration from renewal-risk routing.
- Prepare dollar example: $180K audit-prep savings from SOC 2 evidence automation.
- Prepare a clean non-claim: "This is not legal advice and not a certification claim."
- Prepare expected CTA: a four-week CRM replacement assessment using account, opportunity, and renewal objects.
- Stop condition: prospect names the first CRM workflow and agrees who owns migration risk.

## Opening Hook
- "Most CRM migrations are framed as Salesforce versus HubSpot versus the next CRM."
- "Oyatie frames the decision differently."
- "Your sales team needs accounts, contacts, opportunities, tasks, emails, meetings, forecasts, renewals, support context, consent, and evidence."
- "Those do not need to live inside one CRM island."
- "In Oyatie, CRM is a capability tier."
- "The tier activates ontology projections for account and opportunity objects."
- "It activates workflow templates for deal review, renewal risk, account handoff, and evidence requests."
- "It activates Cedar permit sets for sales, RevOps, customer success, finance, and compliance."
- "It activates compliance overlays such as GDPR and SOC 2 without making reps think like auditors."
- "It uses mail, messenger, calendar, docs, drive, analytics, and audit-chain as shared primitives."
- "That means a mid-market team can replace CRM sprawl without losing the speed that made HubSpot attractive or the control that made Salesforce necessary."
- "In the next 45 minutes I will show Acme Innovations, a fictional Series D SaaS company, replacing CRM workflows."
- "We will migrate an account, inspect opportunity lineage, run a renewal-risk workflow, block an overbroad export, show evidence, and close with economics."
- "The economic story is concrete."
- "$420K of CRM license and add-on spend can be attacked."
- "$310K of RevOps admin drag can be reduced."
- "$1.1M of renewal pipeline risk can be routed earlier."
- "$180K of SOC 2 audit preparation can become evidence export."
- "The close is a four-week assessment, not a full migration commitment."

## Discovery Questions
- "What is the current CRM: Salesforce, HubSpot, Dynamics, Pipedrive, a custom app, or a mix?"
- "Which team owns CRM changes today: sales ops, RevOps, IT, consultants, or admins inside each department?"
- "How many required fields exist because the CRM cannot infer workflow context?"
- "Where does handoff break most often: marketing to sales, sales to success, success to support, or finance to collections?"
- "Which Salesforce or HubSpot add-ons are now required just to run normal operations?"
- "Which CRM reports do executives distrust during forecast calls?"
- "What is the average time from renewal risk signal to accountable owner assignment?"
- "Which customer data falls under GDPR, SOC 2, HIPAA, CCPA, or contract-specific controls?"
- "How do reps know whether they can export a contact list or customer note?"
- "How many duplicate account records exist in the current CRM?"
- "What is the business cost of a dirty renewal forecast?"
- "What is the business cost of delayed enterprise evidence requests?"
- "Where do sales conversations need support, product, or compliance context without broadening access?"
- "What would make a CRM assessment credible to your CRO and CTO at the same time?"
- "What is the smallest object set that proves migration value: account, contact, opportunity, renewal, ticket, or campaign?"

## Demo Flow
1. Screen: RevOps Dashboard.
- Click `RevOps Dashboard`.
- Click `Tenants`.
- Click `Acme Innovations Inc.`
- Click `CRM Replacement View`.
- Dialogue: "I am starting with revenue motion, not a CRM home page."
- Show active packs: GDPR and SOC2 Type II.
- Show active cells: US central and EU west.
- Show customer count: `1,850`.
- Show ARR baseline: `$96M`.
- Click `Pipeline Risk`.
- Show open renewal risk: `$4.8M`.
- Click `Evidence Requests`.
- Show SOC 2 evidence queue.
- Click `CRM Data Health`.
- Show duplicate accounts, stale contacts, and consent gaps.
- Dollar example: "Acme attributes $1.1M in expansion and renewal risk to delayed owner assignment."
- Dialogue: "This is the CRM problem as the executive experiences it."
- Discovery pivot: "Which current CRM screen gets closest to this without a BI rebuild?"
- Stop cue: buyer agrees the executive view is the right entry point.

2. Screen: Capability Tier Instead of CRM Island.
- Click `Sales Capability Tier Console`.
- Select `CRM Core`.
- Click `Tier Composition`.
- Show owning microservices: identity, tenancy, ontology, workflow-engine, mail, messenger, calendar, docs, drive, analytics, audit-chain.
- Click `Cedar Permit Sets`.
- Show permits for rep, manager, RevOps, finance, compliance, and customer-success roles.
- Click `Ontology Projections`.
- Show account, contact, opportunity, renewal, activity, and consent projections.
- Click `Workflow Template Libraries`.
- Show deal review, renewal risk, account handoff, quote exception, evidence request, and customer escalation.
- Dialogue: "ADR-0316 is the product thesis: the buyer sees CRM; the architecture stays shared and flat."
- Click `Compliance Overlays`.
- Show GDPR sales core and SOC 2 evidence overlay.
- Dollar example: "The buyer avoids paying separate add-ons for workflow, audit, and consent exports when those are shared primitives."
- Stop cue: CTO sees that CRM is not another silo.

3. Screen: Salesforce and HubSpot Migration Mapper.
- Click `CRM Migration Mapper`.
- Select source `Salesforce Sales Cloud`.
- Select source `HubSpot CRM`.
- Click `Import Sample`.
- Choose `acme_accounts_500.csv`.
- Choose `acme_opportunities_1200.csv`.
- Choose `acme_activity_90d.json`.
- Click `Map Objects`.
- Show Salesforce Account to Oyatie Account.
- Show HubSpot Company to Oyatie Account.
- Show Salesforce Opportunity to Oyatie Opportunity.
- Show HubSpot Deal to Oyatie Opportunity.
- Show activity history mapped to tenant-scoped timeline.
- Click `Conflicts`.
- Show duplicate account `Northwind Health`.
- Show missing GDPR lawful basis on 840 contacts.
- Show stale opportunity stage mapping.
- Dialogue: "We preserve the concepts reps know while removing vendor-specific object gravity."
- Dollar example: "Acme expects $310K annual reduction in RevOps admin cleanup."
- Click `Dry Run`.
- Show no production mutation.
- Stop cue: RevOps leader sees migration as measurable.

4. Screen: Account Ontology Explorer.
- Click `Account Ontology Explorer`.
- Search `Northwind Health`.
- Click account record.
- Click `Relationship Graph`.
- Show account, contacts, open opportunities, support tickets, evidence requests, renewal risk, and consent edges.
- Click `Role View`.
- Select `Sales Rep`.
- Show sales-relevant fields.
- Select `Compliance Officer`.
- Show evidence and consent fields.
- Select `Support Manager`.
- Show open incidents and customer health.
- Dialogue: "Salesforce and HubSpot can both collect account data. The differentiator here is role-scoped projection over one ontology."
- Click `Field Provenance`.
- Show field sources from CRM import, support event, mail thread, and customer evidence request.
- Dollar example: "A rep sees the $220K renewal, but not restricted SOC 2 reviewer notes."
- Stop cue: buyer understands account as graph, not CRM row.

5. Screen: Renewal Risk Workflow.
- Click `Workflow Engine`.
- Click `Renewal Risk Playbook`.
- Select account `Northwind Health`.
- Click `Start Demo Run`.
- Dialogue: "Now the CRM becomes a system of action."
- Show trigger: support incident severity.
- Show trigger: executive sponsor changed.
- Show trigger: product usage drop.
- Show trigger: unpaid invoice over 30 days.
- Click `Assign Owner`.
- Owner: customer success manager.
- Watch task creation.
- Watch manager approval.
- Watch finance notification.
- Watch executive sponsor email draft.
- Click `Human Review`.
- Show AI-generated summary as draft only.
- Click `Approve Draft`.
- Dollar example: "The workflow protects a $220K renewal and a $900K expansion discussion."
- Click `Evidence`.
- Show audit events for trigger, assignment, draft, approval, and customer message.
- Dialogue: "The AI helps write context; it does not autonomously change the deal."
- Stop cue: CRO sees sales motion, CTO sees governance.

6. Screen: Workflow Studio AI-Assisted Node Generation.
- Click `Workflow Studio`.
- Open `Renewal Risk Playbook`.
- Click `Add Node`.
- Click `AI Draft Node`.
- Prompt: `When a renewal over $200K has a security incident and unpaid invoice, route to legal and finance before customer email.`
- Click `Generate Draft`.
- Show proposed node: `LegalFinancePreflight`.
- Show inputs: opportunity amount, incident severity, invoice aging, account region.
- Show outputs: approval status, blocker reason, due date.
- Click `Review Diff`.
- Show new branch in visual builder.
- Click `Run Test`.
- Show test passes in staging.
- Dialogue: "ADR-0219 says AI drafts into a deterministic builder and requires human review."
- Click `Do Not Activate`.
- Show draft remains inactive.
- Dollar example: "This saves admin time without creating invisible automation risk."
- Stop cue: admin sees AI boundary.

7. Screen: Consent and Campaign Export.
- Click `Campaign Consent Console`.
- Select list `Q3 Expansion Webinar`.
- Click `Export Contacts`.
- Choose region `EU`.
- Attempt export as sales manager.
- Observe partial denial.
- Click `Explain Denial`.
- Show missing lawful basis for 113 contacts.
- Show expired purpose on 42 contacts.
- Show SOC 2 reviewer-only note excluded.
- Dialogue: "A CRM should not make compliant export depend on a rep remembering a policy memo."
- Click `Create Remediation Workflow`.
- Assign CCO Naveen Iyer.
- Click `Audit Preview`.
- Dollar example: "One blocked bad export can avoid regulatory and customer-trust cost far beyond the CRM license."
- Stop cue: compliance owner understands policy is in path.

8. Screen: Customer Evidence Request.
- Click `Docs`.
- Click `Evidence Requests`.
- Open `Northwind Health SOC 2 Renewal`.
- Click `Generate Evidence Bundle`.
- Select controls: access review, change approval, incident response, backup verification.
- Click `Pull Evidence`.
- Show audit-chain references.
- Show drive artifacts.
- Show workflow approvals.
- Show redacted customer identifiers.
- Dialogue: "In Salesforce or HubSpot, this often lives in a separate GRC workflow. Here it is connected to the account and renewal."
- Dollar example: "Acme estimates $180K annual audit-prep reduction from evidence reuse."
- Click `Share with Customer`.
- Show Cedar review before external share.
- Stop cue: buyer sees revenue and compliance convergence.

9. Screen: Forecast and Pricing Model.
- Click `RevOps Dashboard`.
- Click `Forecast`.
- Show current-quarter forecast.
- Click `Confidence Drivers`.
- Show renewal risk, support incidents, consent status, invoice status, and executive sponsor coverage.
- Click `Pricing Model`.
- Show synthetic savings.
- License and add-ons reduction: `$420K`.
- RevOps admin drag reduction: `$310K`.
- Renewal acceleration: `$1.1M`.
- Audit-prep reduction: `$180K`.
- Dialogue: "We separate hard savings from risk-weighted revenue impact."
- Click `Assessment Scope`.
- Select account, opportunity, renewal, campaign consent.
- Stop cue: buyer agrees on assessment value drivers.

10. Screen: Close Artifact.
- Click `Executive Brief Builder`.
- Select `CRM Replacement Assessment`.
- Include migration map, account graph, renewal workflow, consent export, evidence bundle, and pricing model.
- Click `Generate Follow-Up Draft`.
- Dialogue: "The follow-up is generated from what we actually showed."
- Show four-week plan.
- Week 1: object export and field mapping.
- Week 2: workflow dry run.
- Week 3: consent and evidence export.
- Week 4: migration and economics readout.
- Dollar example: "We are testing whether the first $500K to $1.5M value path is credible."
- Stop cue: buyer agrees to workshop or assessment.

## Objection Handling
- Objection: "Salesforce already does this."
- Response name: "Suite feature versus shared substrate."
- Talk track: "Salesforce can do many CRM tasks. The question is whether CRM should own workflow, evidence, identity, and compliance as separate gravity."
- Proof point: "Capability Tier Console shows shared primitives instead of another CRM island."
- Demo anchor: "Screen 2."
- Commercial anchor: "Assess only the workflows that are costly in Salesforce."
- Close question: "Which Salesforce customization costs the most to change?"

- Objection: "HubSpot is easier for our reps."
- Response name: "Keep simplicity, add governance."
- Talk track: "The goal is not to bury reps in controls. Role projection keeps their workspace focused while controls run in path."
- Proof point: "Sales rep view hid compliance-only fields."
- Demo anchor: "Screen 4."
- Commercial anchor: "Measure rep task count during assessment."
- Close question: "Which HubSpot workflow breaks when enterprise controls arrive?"

- Objection: "We do not want another long migration."
- Response name: "Dry-run first."
- Talk track: "The first assessment imports sample objects without production mutation."
- Proof point: "Migration Mapper dry run showed conflicts before committing."
- Demo anchor: "Screen 3."
- Commercial anchor: "Four-week assessment before migration plan."
- Close question: "Can we test 500 accounts and 1,200 opportunities first?"

- Objection: "Our reps will reject workflow gates."
- Response name: "Gates only on consequential paths."
- Talk track: "Normal activity stays fast. High-risk exports, large renewals, and regulated evidence requests get gates."
- Proof point: "Consent export denied only contacts missing lawful basis."
- Demo anchor: "Screen 7."
- Commercial anchor: "Assessment maps gates to agreed risk thresholds."
- Close question: "What should be gated at $50K, $200K, and $1M?"

- Objection: "AI-generated nodes scare us."
- Response name: "Draft, diff, test, activate."
- Talk track: "AI creates a draft node. Humans review the visual diff, tests pass, and activation is separate."
- Proof point: "The generated node stayed inactive."
- Demo anchor: "Screen 6."
- Commercial anchor: "Assessment can include or exclude AI drafting."
- Close question: "Would you test AI on a non-production workflow first?"

- Objection: "We need marketing automation too."
- Response name: "Adjacent capability tiers."
- Talk track: "Marketing automation can activate as another tier over shared identity, consent, mail, workflow, and analytics."
- Proof point: "Campaign consent is already not CRM-owned."
- Demo anchor: "Screen 7."
- Commercial anchor: "Start with CRM core, add campaign scope only if needed."
- Close question: "Is marketing automation in first value scope or phase two?"

- Objection: "We need a customer support view."
- Response name: "Account graph includes support context."
- Talk track: "Support context appears through role-scoped account projections without forcing support to live inside CRM."
- Proof point: "Northwind Health graph showed support incidents and renewal risk."
- Demo anchor: "Screen 4."
- Commercial anchor: "Include one support-to-renewal handoff in assessment."
- Close question: "Which support signal should affect renewal forecast?"

- Objection: "Our data is too dirty."
- Response name: "Dirty data is the assessment target."
- Talk track: "Duplicate accounts and missing consent are exactly what the migration mapper surfaces."
- Proof point: "Northwind duplicate and missing lawful basis were visible."
- Demo anchor: "Screen 3."
- Commercial anchor: "Assessment output includes data-health remediation list."
- Close question: "Which dirty-data category is most expensive today?"

- Objection: "We need executive forecast accuracy."
- Response name: "Forecast with evidence drivers."
- Talk track: "Forecast confidence comes from renewal, support, consent, invoice, and sponsor signals."
- Proof point: "Forecast drivers were visible in RevOps Dashboard."
- Demo anchor: "Screen 9."
- Commercial anchor: "Assessment compares forecast confidence on selected accounts."
- Close question: "Which forecast field do executives distrust now?"

- Objection: "Compliance is not a sales problem."
- Response name: "Compliance is a revenue blocker."
- Talk track: "Enterprise deals stall when evidence, consent, and audit exports are slow."
- Proof point: "Customer evidence request connected to renewal."
- Demo anchor: "Screen 8."
- Commercial anchor: "Quantify days saved on enterprise evidence response."
- Close question: "How often do evidence requests slow late-stage deals?"

- Objection: "We have Salesforce consultants already."
- Response name: "Use them as source-system experts."
- Talk track: "Existing admins and consultants can help classify fields, workflows, and customizations."
- Proof point: "Migration mapping accepts Salesforce object knowledge."
- Demo anchor: "Screen 3."
- Commercial anchor: "Assessment includes RevOps and admin workshops."
- Close question: "Who knows the current field history best?"

- Objection: "We need mobile reps."
- Response name: "Role workspace, not CRM clone."
- Talk track: "Mobile experience should expose tasks, account context, meeting prep, and follow-up, not the entire schema."
- Proof point: "Role projection narrows the view."
- Demo anchor: "Screen 4."
- Commercial anchor: "Mobile UX can be a phase-two pilot if required."
- Close question: "What must a field rep do on mobile in under two minutes?"

- Objection: "We are not ready to leave HubSpot."
- Response name: "Parallel run."
- Talk track: "Use HubSpot as a source during the assessment and compare workflow and evidence outcomes."
- Proof point: "HubSpot Company and Deal mapped to Oyatie objects."
- Demo anchor: "Screen 3."
- Commercial anchor: "No production replacement in the first four weeks."
- Close question: "Which HubSpot report can we parallel-run?"

- Objection: "Our sales process changes every quarter."
- Response name: "Visual workflow changes."
- Talk track: "Workflow Studio lets admins edit, test, and activate changes without code-first releases."
- Proof point: "The legal-finance node was drafted and tested."
- Demo anchor: "Screen 6."
- Commercial anchor: "Assessment includes one live change simulation."
- Close question: "What sales-process change is already pending?"

- Objection: "Pricing sounds like another enterprise-market platform."
- Response name: "Assess before subscription."
- Talk track: "We price the first assessment to prove value before subscription expansion."
- Proof point: "Close artifact showed a four-week plan."
- Demo anchor: "Screen 10."
- Commercial anchor: "Assessment price is separate from production ARR."
- Close question: "Who needs to approve a four-week assessment?"

- Objection: "We need integrations with Gmail, Slack, and finance."
- Response name: "Shared primitives and connectors."
- Talk track: "Mail, messenger, calendar, finance, and workflow are platform primitives or connector-backed flows, not CRM plug-ins."
- Proof point: "Renewal workflow touched finance and customer message drafts."
- Demo anchor: "Screen 5."
- Commercial anchor: "Include one integration path in assessment."
- Close question: "Which integration causes most CRM admin work?"

- Objection: "We cannot lose historical activity."
- Response name: "Timeline preservation."
- Talk track: "Activity history maps to a tenant-scoped timeline with provenance."
- Proof point: "Migration Mapper imported 90-day activity."
- Demo anchor: "Screen 3."
- Commercial anchor: "Assessment tests representative historical import."
- Close question: "How many years of activity must be searchable on day one?"

- Objection: "We need customer-facing portal context."
- Response name: "Account projection, not portal silo."
- Talk track: "Customer portal events become account graph edges and workflow triggers."
- Proof point: "Support incidents and evidence requests appeared on the account graph."
- Demo anchor: "Screen 4."
- Commercial anchor: "Portal context can be included after CRM core proof."
- Close question: "Which customer portal event should trigger sales action?"

## Closing Call to Action
- "The recommended next step is a four-week CRM replacement assessment."
- "Scope account, opportunity, renewal, and campaign consent."
- "Use one source export from Salesforce or HubSpot."
- "Use one renewal-risk workflow."
- "Use one evidence-request workflow."
- "Use one consent export gate."
- "Keep production untouched."
- "Deliver a migration map."
- "Deliver a data-health report."
- "Deliver a workflow dry run."
- "Deliver a Cedar policy simulation."
- "Deliver a SOC 2 or GDPR evidence export."
- "Deliver a CFO or CRO value model."
- "Success means the team agrees whether Oyatie can replace the first CRM workflow with less admin drag and stronger controls."
- "Failure means the team keeps the map and stops before migration spend."
- "Decision requested now: source CRM, first object set, RevOps owner, compliance owner, and workshop date."
- "If those are not known, schedule discovery with RevOps and sales leadership."
- "If those are known, start procurement for the assessment."

## Pricing Conversation Anchors
- Suggested assessment price band: $60K to $140K.
- Use $60K for one source CRM, two object types, and one workflow.
- Use $95K for Salesforce plus HubSpot, four object types, one workflow, and one compliance overlay.
- Use $140K for two source CRMs, activity history, consent export, evidence request, and executive value model.
- Production subscription should be discussed only after assessment scope is accepted.
- Subscription pricing depends on active capability tiers, users, workflow volume, evidence retention, and cells.
- Do not discount below the effort needed for migration mapping and workflow proof.
- Strong savings anchor: CRM license and add-on reduction around $420K in the Acme fixture.
- Strong efficiency anchor: RevOps admin drag around $310K in the Acme fixture.
- Strong revenue anchor: renewal-risk routing around $1.1M risk-weighted value in the Acme fixture.
- Strong compliance anchor: SOC 2 evidence prep around $180K in the Acme fixture.
- Phrase: "We are not pricing a replacement dream; we are pricing a decision-quality assessment."
- Phrase: "Production price should be earned by measured admin and cycle-time reduction."
- Phrase: "Your current CRM can remain the source during the first proof."
- Procurement note: a free workshop can scope the assessment but should not include object import.
- Security note: no production credentials are needed for first dry run.
- Legal note: compliance examples are synthetic and require buyer legal review.
- Stop condition: buyer accepts price band or asks for scoped discovery to narrow it.

## Follow-up Email Template
- Subject: CRM replacement assessment proposal for {{company}}
- Hi {{first_name}},
- Thanks for the discussion today.
- We used the fictional Acme Innovations fixture to show a mid-market CRM replacement path against Salesforce and HubSpot-style workflows.
- The demo focused on CRM as a capability tier over shared identity, ontology, workflow, policy, audit, mail, messenger, analytics, and compliance primitives.
- Screens covered:
- RevOps dashboard.
- Capability tier composition.
- Salesforce and HubSpot migration mapping.
- Account ontology graph.
- Renewal-risk workflow.
- AI-assisted Workflow Studio node generation.
- Consent-protected campaign export.
- Customer evidence request.
- Forecast and pricing model.
- Recommended assessment scope:
- Source CRM: {{source_crm}}
- Object types: {{object_types}}
- Workflow: {{workflow}}
- Compliance overlay: {{compliance_overlay}}
- RevOps owner: {{revops_owner}}
- Compliance owner: {{compliance_owner}}
- Proposed duration: four weeks.
- Proposed deliverables: migration map, data-health report, workflow dry run, Cedar simulation, evidence export, and value model.
- Suggested workshop attendees: CRO, RevOps, sales ops admin, CTO or security lead, compliance owner, and customer success leader.
- Proposed times:
- {{time_option_1}}
- {{time_option_2}}
- {{time_option_3}}
- Regards,
- {{sender_name}}

## References
- Internal: `registry/sample-tenants/acme-mid-market-saas.md`.
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
- Internal: `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- Internal: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- Internal: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`.
- Internal: `docs/decisions/ADR-0706-observability-live-apex.md`.
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `specs/capability-tier-schema.json`.
- Internal: `registry/capability-tiers/vendor-tier-mapping.yaml`.
- Salesforce Sales Cloud product page: https://www.salesforce.com/products/sales-cloud/
- Salesforce Trust documentation: https://trust.salesforce.com/
- HubSpot CRM product page: https://www.hubspot.com/products/crm
- HubSpot Operations Hub product page: https://www.hubspot.com/products/operations
- EU GDPR Regulation 2016/679: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679
- SOC 2 overview from AICPA: https://www.aicpa-cima.com/topic/audit-assurance/audit-and-assurance-greater-than-soc-2
- Cedar policy language: https://www.cedarpolicy.com/
