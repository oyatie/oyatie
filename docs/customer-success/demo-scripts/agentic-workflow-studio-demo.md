---
doc_class: DemoScript
target_persona: Operations leader, RevOps lead, workflow automation owner, product operations lead, enterprise architect, AI transformation sponsor
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
status: canonical
date: 2026-05-20
owner: Customer Success Engineering
---

# Agentic Workflow Studio Demo

## Pre-Demo Checklist

- Confirm meeting type: Workflow Studio product demo for automation, operations, RevOps, or AI transformation buyer.
- Confirm prospect segment: mid-market company, enterprise operations group, regulated function, or platform team evaluating AI-assisted workflow creation.
- Confirm primary sponsor: COO, RevOps leader, workflow automation owner, product operations lead, enterprise architect, or AI transformation sponsor.
- Confirm stated pain: backlog of workflow requests, brittle no-code tools, uncontrolled AI automation, duplicated SaaS workflows, or compliance concerns.
- Confirm deal hypothesis: Oyatie Workflow Studio as governed workflow creation surface with optional AI assistance and human review.
- Confirm named tenant fixture: `tenant-acme-mid-market-saas`.
- Confirm fallback fixture: `tenant-helios-fortune-500-manufacturer` if buyer is enterprise operations.
- Confirm fixture mode: synthetic accounts, synthetic users, synthetic approvals, no production integrations.
- Confirm design posture: no-code first, AI-assisted draft second, human-owned publish always.
- Confirm compliance posture: AI suggestions are drafts, not automatic changes.
- Confirm demo duration: 45 minutes.
- Confirm agenda allocation: 5 minutes opening, 7 minutes discovery, 27 minutes product flow, 6 minutes close and commercial.
- Open Oya Demo Console.
- Open Workflow Studio.
- Open AI Node Assistant.
- Open Policy Simulator.
- Open Cedar Permit Simulator.
- Open Test Fixture Runner.
- Open Evidence Portal.
- Open Audit Replay Console.
- Open Version Diff.
- Open Cost and Usage Lens.
- Open Integration Catalog.
- Open Publish Gate.
- Open Rollback Console.
- Load tenant fixture `tenant-acme-mid-market-saas`.
- Load persona `revops-admin`.
- Load persona `sales-manager`.
- Load persona `finance-approver`.
- Load persona `legal-approver`.
- Load persona `workflow-governance-reviewer`.
- Load persona `security-architect`.
- Enable workflow template `discount-approval-renewal-risk`.
- Enable demo integration stubs `crm-account`, `billing-subscription`, `contract-repository`, `slack-notification`, `email-notification`.
- Enable compliance packs `GDPR`, `SOC2`, `AI-Draft-Governance`.
- Prepare AI prompt seed `Add a renewal-risk escalation node when discount exceeds 18 percent and ARR exceeds $250,000`.
- Prepare AI prompt seed `Generate a customer success task after finance approval when churn risk is high`.
- Prepare AI prompt seed `Explain why legal review is required for non-standard indemnity`.
- Prepare objection-response card `ai-control`.
- Prepare objection-response card `builder-vs-bespoke`.
- Prepare objection-response card `shadow-automation`.
- Prepare objection-response card `workflow-sprawl`.
- Prepare objection-response card `governance-speed`.
- Prepare pricing card `workflow-studio-platform`.
- Prepare leave-behind `Workflow-Studio-Governed-AI-Builder-Map`.
- Verify browser zoom is 90 percent.
- Verify canvas fits on shared screen.
- Verify AI assistant is in demo mode and cannot publish.
- Verify no production connector is selected.
- Verify test fixtures reset.
- Verify publish gate starts in blocked state until tests and review pass.
- Stop condition: do not connect live CRM, send real Slack messages, or publish to production.

## Opening Hook

- Say: "Most companies do not have an automation shortage."
- Say: "They have an automation governance problem."
- Say: "A team builds one workflow in a no-code tool."
- Say: "Another team builds a similar workflow in a CRM."
- Say: "Finance keeps a spreadsheet because the official process is too slow."
- Say: "Then AI enters the picture and everyone wants generated automations by Friday."
- Say: "That creates two bad choices."
- Say: "Either central IT blocks the work and the business waits."
- Say: "Or the business moves quickly and the company inherits shadow automation."
- Say: "Oyatie Workflow Studio is built for the middle path."
- Say: "The business gets a visual builder."
- Say: "AI can help draft nodes, labels, conditions, and test cases."
- Say: "But AI does not silently publish automation."
- Say: "Humans review the diff."
- Say: "Policies are simulated before activation."
- Say: "Tests run against fixtures."
- Say: "Cedar permits define who can view, edit, approve, publish, and rollback."
- Say: "Every version becomes evidence."
- Say: "Today I will use the Acme SaaS fixture."
- Say: "The business scenario is a renewal-risk and discount-approval workflow."
- Say: "The dollar amount is concrete: a 320,000 dollar annual renewal with a requested 22 percent discount."
- Say: "We will start from a simple visual workflow."
- Say: "We will ask the AI assistant to draft a node."
- Say: "We will inspect the generated change."
- Say: "We will run tests."
- Say: "We will simulate policy."
- Say: "We will publish only after the human governance gate clears."
- Say: "The outcome we want from this call is deciding whether a two-week Workflow Studio pilot design is worth scheduling."
- Say: "A good pilot should retire one manual approval path, prevent uncontrolled AI automation, and prove a repeatable governance model."
- Pause.
- Ask: "Is your priority speed of workflow creation, AI governance, integration control, or operational cost reduction?"
- Adjust emphasis based on answer.

## Discovery Questions

1. Which workflow request backlog is most painful right now?
2. Who builds workflows today: business admins, IT, RevOps, developers, or consultants?
3. Where do workflows duplicate logic across CRM, ticketing, spreadsheets, chat, and custom apps?
4. What kinds of changes require review before a workflow can go live?
5. Who is allowed to publish automation that affects revenue, customer commitments, or regulated data?
6. Where are AI-generated automations being requested or already attempted?
7. How do you test no-code or low-code workflow changes before production?
8. Can you reconstruct who changed a workflow, why, and what policy allowed it?
9. Which integrations are safe for a pilot: CRM, billing, contract repository, support, finance, or identity?
10. What is the cost of one broken approval workflow in lost revenue, customer delay, or operational rework?
11. How do you roll back a workflow change today?
12. Which compliance or security team needs evidence before AI-assisted workflow generation is acceptable?
13. What would a successful 30-day pilot prove?
14. Which workflow has enough volume and pain to justify an executive business case?
15. Who would own the workflow library after the pilot?

## Demo Flow

1. Screen: Workflow Studio Home.
- Click `Demo Console`.
- Click `Tenant Switcher`.
- Select `tenant-acme-mid-market-saas`.
- Click `Workflow Studio`.
- Say: "We are inside the Acme SaaS tenant."
- Point to `Templates`.
- Point to `Live Workflows`.
- Point to `Drafts`.
- Point to `Governance Queue`.
- Click `Templates`.
- Select `discount-approval-renewal-risk`.
- Click `Open as Draft`.
- Say: "We start from a governed template, not a blank whiteboard."
- Point to the canvas nodes: Account Context, Discount Request, Manager Review, Finance Approval, Legal Review, Customer Success Task, Renewal Close.
- Dollar example: "The example renewal is 320,000 dollars ARR, with a 22 percent discount request."
- Say: "That dollar amount matters because policy should react to business risk."

2. Screen: Canvas Orientation.
- Click node `Discount Request`.
- Show properties panel.
- Point to fields: requested discount, ARR, renewal date, churn risk, non-standard terms, region.
- Say: "The visual builder is the primary interface."
- Click `Manager Review`.
- Show approver rule.
- Say: "A business admin can read this without parsing code."
- Click `Finance Approval`.
- Show threshold `discount greater than 15 percent or ARR greater than $100,000`.
- Click `Legal Review`.
- Show condition `non-standard indemnity or data-processing terms`.
- Click `Evidence`.
- Show audit and approval events captured per node.
- Say: "Every node has runtime behavior and evidence behavior."
- Click `Permissions`.
- Show who can edit this workflow.
- Say: "Workflow editing is also governed."

3. Screen: AI Node Assistant Prompt.
- Click `AI Assistant`.
- Confirm mode `Draft Only`.
- Paste prompt: `Add a renewal-risk escalation node when discount exceeds 18 percent and ARR exceeds $250,000`.
- Click `Generate Draft`.
- Say: "The assistant is allowed to draft a proposed change."
- Say: "It cannot publish the workflow."
- Wait for generated proposal.
- Point to new node `Renewal Risk Escalation`.
- Point to condition `requested_discount > 18 percent and arr > $250,000`.
- Point to suggested owner `Customer Success Director`.
- Point to evidence tag `risk_escalation.created`.
- Say: "This is useful, but we do not trust it blindly."
- Dollar example: "The node catches the 320,000 dollar renewal because 22 percent discount exceeds threshold."

4. Screen: AI Explanation and Diff.
- Click `Review Draft`.
- Click `Version Diff`.
- Show added node.
- Show added edge from `Finance Approval` to `Renewal Risk Escalation`.
- Show added evidence field.
- Show no changes to legal review node.
- Say: "The diff is the governance moment."
- Click `Why Suggested`.
- Show assistant explanation.
- Say: "The assistant explains its proposed logic in business terms."
- Click `Risk Flags`.
- Show warning `No escalation SLA specified`.
- Say: "This is why AI assistance needs review."
- Click `Add SLA`.
- Set SLA `1 business day`.
- Add description `CS Director must approve concession plan before quote is sent`.
- Say: "The human completes the business rule."
- Dollar example: "A one-day delay on a 320,000 dollar renewal is acceptable if it prevents an unmanaged 70,400 dollar discount concession."

5. Screen: Manual Node Editing.
- Click node `Renewal Risk Escalation`.
- Open `Properties`.
- Rename display label `Executive Renewal Risk Review`.
- Set owner `Customer Success Director`.
- Set backup owner `VP Customer Success`.
- Add required field `concession_plan`.
- Add required field `exec_summary`.
- Set notification channel `Slack demo stub`.
- Say: "The business user edits the AI draft visually."
- Click `Save Draft`.
- Show status `Draft saved, not published`.
- Say: "Saving is not publishing."
- Click `History`.
- Show entry: AI draft created, human edited label, human added SLA, human added fields.
- Say: "The history separates machine draft from human decision."

6. Screen: Policy Simulator.
- Click `Policy Simulator`.
- Select workflow `Discount Approval Renewal Risk`.
- Select persona `revops-admin`.
- Action `edit_workflow`.
- Click `Simulate`.
- Show `Allowed`.
- Change action to `publish_workflow`.
- Click `Simulate`.
- Show `Denied: governance reviewer required`.
- Switch persona to `workflow-governance-reviewer`.
- Action `approve_workflow`.
- Click `Simulate`.
- Show `Allowed after tests pass`.
- Say: "Policy simulation makes governance visible before runtime."
- Click `Scenario`.
- Select account region `EU`.
- Click `Simulate Data Access`.
- Show GDPR pack requiring purpose and retention tag.
- Dollar example: "This is how you avoid a 70,000 dollar consulting exercise every time a workflow crosses a data boundary."

7. Screen: Test Fixture Runner.
- Click `Test Fixtures`.
- Select fixture `High ARR High Discount`.
- Show inputs: ARR `$320,000`, requested discount `22 percent`, churn risk `High`, region `EU`.
- Click `Run`.
- Show path: Discount Request to Manager Review to Finance Approval to Executive Renewal Risk Review to Legal Review to Renewal Close.
- Say: "The new node is hit."
- Click `Evidence`.
- Show expected evidence events.
- Select fixture `Low ARR Low Discount`.
- Show inputs: ARR `$24,000`, requested discount `8 percent`, churn risk `Low`, region `US`.
- Click `Run`.
- Show path bypasses executive review.
- Say: "We test that the workflow does not over-escalate."
- Select fixture `High Discount Missing Concession Plan`.
- Click `Run`.
- Show failure `concession_plan required`.
- Say: "The fixture catches incomplete business process."

8. Screen: AI-Generated Test Suggestion.
- Click `AI Assistant`.
- Choose `Suggest Tests`.
- Say: "AI can help with test coverage, again as a draft."
- Show suggested tests: threshold boundary at 18 percent, ARR boundary at $250,000, EU data purpose required, legal terms override, missing concession plan.
- Click `Accept as Draft`.
- Click `Review Test Diff`.
- Approve three tests.
- Reject test `auto-approve if customer is strategic`.
- Say: "We reject this because strategic status should not bypass finance."
- Click `Add Rejection Reason`.
- Enter `Strategic customer status cannot bypass discount governance`.
- Say: "The rejection is evidence for future reviewers."
- Dollar example: "Rejecting one bad test assumption protects every renewal above the threshold."

9. Screen: Integration Catalog.
- Click `Integration Catalog`.
- Show stubs: CRM Account, Billing Subscription, Contract Repository, Slack Notification, Email Notification.
- Click `CRM Account`.
- Show read scope: account name, ARR, owner, renewal date, region.
- Click `Billing Subscription`.
- Show read scope: subscription amount, renewal status, invoice state.
- Click `Contract Repository`.
- Show read scope: terms flags, DPA status, indemnity flag.
- Click `Slack Notification`.
- Show demo stub only.
- Say: "The pilot can use stubs, then approved connectors."
- Click `Attempt Write Scope`.
- Show blocked because workflow is draft.
- Say: "Draft workflows cannot write to production connectors."
- Dollar example: "This is how you prevent a demo experiment from sending a real quote or notification."

10. Screen: Data Purpose and Retention.
- Click `Compliance`.
- Select `GDPR`.
- Show purpose tag `renewal_operations`.
- Show retention `contract_lifecycle_plus_legal_hold_policy`.
- Show data minimization: account fields only, no contact personal notes.
- Say: "The workflow does not get broad CRM access by default."
- Click `Field Access`.
- Attempt to add `customer_personal_notes`.
- Show warning `Purpose review required`.
- Click `Deny Field`.
- Say: "This is a business-friendly governance moment."
- Say: "The admin can see why the field is not appropriate."
- Dollar example: "Avoiding one inappropriate data exposure can save far more than the workflow labor savings."

11. Screen: Publish Gate.
- Click `Publish Gate`.
- Show checklist: tests passed, policy simulation passed, governance reviewer approved, integration scope approved, rollback point created, evidence preview generated.
- Point to blocked item `Governance reviewer approval missing`.
- Switch persona to `workflow-governance-reviewer`.
- Click `Review`.
- Show diff summary.
- Show test results.
- Show policy results.
- Show AI draft provenance.
- Click `Approve`.
- Add note `Approved for sandbox activation; production requires connector review`.
- Click `Approve`.
- Show gate item complete.
- Say: "The reviewer approves the change, not the AI."
- Click `Publish to Sandbox`.
- Show success.
- Dollar example: "A controlled sandbox publish is cheaper than a broken production renewal workflow."

12. Screen: Runtime Case.
- Click `Run Sandbox Case`.
- Select account `Northstar Manufacturing`.
- Show ARR `$320,000`.
- Show requested discount `22 percent`.
- Show churn risk `High`.
- Click `Start Workflow`.
- Complete manager review.
- Complete finance approval.
- Show new task `Executive Renewal Risk Review`.
- Switch persona to `Customer Success Director`.
- Open task.
- Add concession plan `Two-year term, executive sponsor call, phased services credit capped at $18,000`.
- Click `Approve`.
- Say: "The workflow now forces the concession plan before quote release."
- Click `Renewal Close`.
- Show final quote requires legal review because non-standard indemnity is flagged.
- Dollar example: "The workflow protects roughly 70,400 dollars of requested discount exposure while preserving deal speed."

13. Screen: Audit Replay.
- Click `Audit Replay`.
- Select workflow version `v12-sandbox`.
- Click `Replay Change`.
- Show sequence: template opened, AI draft generated, human edits, test suggestions, rejected AI test, policy simulation, governance approval, sandbox publish.
- Say: "This is the answer to shadow automation."
- Click `Runtime Replay`.
- Show case path for Northstar Manufacturing.
- Click `Export Evidence`.
- Show draft evidence bundle.
- Say: "The evidence is useful for security, compliance, operations, and future maintainers."
- Dollar example: "If one workflow incident consumes 80 hours across RevOps, IT, legal, and finance, evidence replay cuts the investigation cost."

14. Screen: Rollback Console.
- Click `Rollback`.
- Show current sandbox version.
- Select previous version `v11`.
- Click `Compare`.
- Show removed node if rollback applied.
- Click `Simulate Rollback`.
- Show affected open cases: `3`.
- Say: "Rollback is also governed."
- Click `Policy Check`.
- Show governance reviewer can rollback sandbox, production rollback requires operations owner.
- Say: "We do not make rollback a hidden admin shortcut."
- Dollar example: "A bad approval workflow during quarter close can delay hundreds of thousands of dollars of renewals."
- Click `Cancel Rollback`.

15. Screen: Cost and Usage Lens.
- Click `Cost and Usage`.
- Show workflow runs per month `1,250`.
- Show average manual touches before `5`.
- Show average manual touches after `3`.
- Show minutes saved per run `9`.
- Show monthly hours saved `187.5`.
- Show loaded cost per hour `$85`.
- Show monthly labor value `$15,938`.
- Show annual labor value `$191,256`.
- Add avoided discount leakage assumption `0.5 percent of $18,000,000 renewal pool`.
- Show value `$90,000`.
- Add retired tool spend `$48,000`.
- Show first-year target value `$329,256`.
- Say: "For one mid-market workflow, a 300,000 dollar value case is credible."
- Say: "For enterprise workflow libraries, the economics compound."

16. Screen: Workflow Library Governance.
- Click `Workflow Library`.
- Show tags: revenue, finance, legal, customer-success, GDPR, AI-assisted, sandbox-only.
- Click `Ownership`.
- Show owner `RevOps`.
- Show governance reviewer `Business Systems`.
- Show security reviewer `Security Architecture`.
- Click `Promotion Rules`.
- Show sandbox to production requires integration review and production test run.
- Say: "The library is the asset."
- Say: "Over time, this becomes how companies stop re-solving workflow governance in every department."
- Dollar example: "Retiring five separate workflow tools at 60,000 dollars each is 300,000 dollars of subscription rationalization before productivity."

17. Screen: Executive Summary.
- Click `Executive Summary`.
- Select sections: AI draft, human diff, test results, policy simulation, sandbox run, value model.
- Click `Generate Readout`.
- Say: "This is what your AI governance sponsor and COO need to see."
- Point to `AI did not publish`.
- Point to `Policy simulated before activation`.
- Point to `Tests captured threshold behavior`.
- Point to `Business value estimated from real workflow volume`.
- Click `Pilot Plan`.
- Show two-week design followed by four-to-six-week sandbox pilot.
- Say: "The decision today is whether one real workflow is worth mapping in detail."

## Objection Handling

1. Objection: "AI-generated workflows are too risky."
- Response name: AI-Draft-Only.
- Say: "The assistant drafts, explains, and suggests tests."
- Say: "It does not publish."
- Say: "Humans approve diffs, tests, policies, and promotion gates."

2. Objection: "Our business users will create chaos."
- Response name: Governed-Builder.
- Say: "Users can build inside assigned permissions."
- Say: "Publish requires tests, policy checks, and governance approval."
- Say: "The system is designed to prevent shadow automation."

3. Objection: "We already have no-code tools."
- Response name: Evidence-And-Policy-Differentiator.
- Say: "Many no-code tools create workflow."
- Say: "The differentiator here is policy simulation, AI provenance, evidence, and publish governance."
- Say: "We coexist or migrate based on workflow value."

4. Objection: "This is too much governance for simple workflows."
- Response name: Risk-Based-Gates.
- Say: "Governance can be risk-based."
- Say: "A low-risk notification workflow should not require the same review as a revenue-impacting discount approval."
- Say: "The tenant policy sets that threshold."

5. Objection: "Developers could build this faster."
- Response name: Developer-Time-Is-Scarce.
- Say: "Developers can build one workflow quickly."
- Say: "The question is who maintains hundreds of changes, tests, permissions, and evidence over time."
- Say: "Workflow Studio moves appropriate change closer to the business while preserving control."

6. Objection: "The AI assistant may hallucinate rules."
- Response name: Diff-Test-Policy-Loop.
- Say: "That is why every suggestion is reviewed as a diff."
- Say: "Tests and policy simulation catch bad assumptions."
- Say: "Rejected suggestions are recorded."

7. Objection: "We cannot connect to production systems."
- Response name: Stub-First-Pilot.
- Say: "The pilot can start with stubs and synthetic fixtures."
- Say: "Production connectors require your normal security process."
- Say: "The governance model can be validated before live writes."

8. Objection: "Business logic belongs in CRM."
- Response name: CRM-Boundary.
- Say: "Some CRM-native logic should stay there."
- Say: "Cross-system approvals, legal review, finance review, AI provenance, and evidence often outgrow the CRM."
- Say: "The workshop defines the boundary."

9. Objection: "Rollback sounds dangerous."
- Response name: Governed-Rollback.
- Say: "Rollback is simulated and permissioned."
- Say: "Open-case impact is shown before action."
- Say: "Production rollback requires the right owner."

10. Objection: "We need version control."
- Response name: Workflow-Version-Evidence.
- Say: "Each workflow version has a diff, author history, AI provenance, tests, policy results, and publish evidence."
- Say: "The goal is version control that business reviewers can understand."

11. Objection: "This could become another platform to administer."
- Response name: Library-Rationalization.
- Say: "The target is fewer disconnected workflow surfaces."
- Say: "Workflow Studio becomes valuable when it replaces scattered tools and informal processes."
- Say: "We measure rationalization in the pilot."

12. Objection: "Security needs code review."
- Response name: Policy-And-Connector-Review.
- Say: "Security reviews integration scopes, connector permissions, data access, and promotion rules."
- Say: "For custom code nodes, code review can be required."
- Say: "For no-code nodes, policy and test evidence are the review surface."

13. Objection: "We cannot quantify value."
- Response name: Volume-Times-Minutes.
- Say: "Start with workflow volume, manual touches, minutes saved, exception cost, and tool spend."
- Say: "Then add risk and leakage assumptions separately."
- Say: "The demo math should be replaced with your numbers."

14. Objection: "Our approval rules change constantly."
- Response name: Change-Governance.
- Say: "Frequent change is exactly why visual diff, tests, and policy simulation matter."
- Say: "You can change faster without hiding the change."
- Say: "The history remains inspectable."

15. Objection: "AI governance will slow adoption."
- Response name: Trust-Accelerates-Adoption.
- Say: "Untrusted AI slows adoption more."
- Say: "A clear draft-review-publish pattern helps governance teams say yes."
- Say: "The first pilot should prove speed with control."

16. Objection: "We have too many workflow variants."
- Response name: Template-And-Variant-Model.
- Say: "Templates handle shared structure."
- Say: "Variants handle local rules."
- Say: "The library shows ownership and divergence explicitly."

17. Objection: "What happens if the workflow creates customer-facing errors?"
- Response name: Sandbox-And-Connector-Gates.
- Say: "Customer-facing actions require connector approval and sandbox validation."
- Say: "High-risk outputs can require preview and human send."
- Say: "Production write scopes are not granted by default."

18. Objection: "This seems like an IT project."
- Response name: Joint-Ownership.
- Say: "The strongest pilots are jointly owned."
- Say: "Business owns process intent."
- Say: "IT owns integration, security, and tenant RBAC governance."

19. Objection: "Can we just use ChatGPT to generate workflows?"
- Response name: Generated-Text-Is-Not-Governed-Runtime.
- Say: "A generated diagram is not a governed runtime."
- Say: "Oyatie ties generation to permissions, tests, integrations, publish gates, and evidence."
- Say: "That is the difference between inspiration and operational automation."

20. Objection: "We need to prove this without a long implementation."
- Response name: Two-Week-Design.
- Say: "Start with one workflow."
- Say: "Map current state, build a sandbox fixture, run the draft-review-test-publish loop, and produce a value model."
- Say: "That is enough to decide whether to continue."

## Closing Call to Action

- Say: "The demo showed the governed AI workflow loop."
- Say: "Visual builder first."
- Say: "AI draft second."
- Say: "Human diff review."
- Say: "Policy simulation."
- Say: "Fixture tests."
- Say: "Governance approval."
- Say: "Sandbox publish."
- Say: "Audit replay and rollback."
- Say: "The recommended next step is a two-week Workflow Studio Pilot Design."
- Propose day 1: workflow selection and current-state map.
- Propose day 2: integration and data-scope inventory.
- Propose day 3: policy and role model.
- Propose day 4: test fixture design.
- Propose day 5: AI usage and governance boundary.
- Propose week 2 day 1: workflow draft in sandbox.
- Propose week 2 day 2: policy simulation and test run.
- Propose week 2 day 3: value model.
- Propose week 2 day 4: security and governance review.
- Propose week 2 day 5: pilot recommendation.
- Ask: "Which workflow has enough pain and enough volume to be the pilot anchor?"
- Ask: "Who owns publish authority today?"
- Ask: "Can we schedule the workflow-selection call this week?"
- If they hesitate, offer a 60-minute workflow backlog triage.
- If they agree, capture workflow name, owner, systems touched, and target metric before ending the call.

## Pricing Conversation Anchors

- Anchor around governed workflow throughput and tool rationalization.
- Explain that Workflow Studio pricing usually has three components.
- Component 1: tenant platform subscription.
- Component 2: Workflow Studio capability activation and AI draft governance.
- Component 3: implementation services for selected workflows, integrations, and governance setup.
- Suggested design workshop anchor: $40,000 to $90,000 depending on workflow complexity.
- Suggested sandbox pilot anchor: $175,000 to $450,000 for one to three workflows, fixtures, policy model, test coverage, and integration stubs.
- Enterprise expansion depends on workflow count, integration depth, pack activation, and runtime volume.
- Value lever: manual approval minutes saved.
- Value lever: discount leakage reduction.
- Value lever: faster cycle time.
- Value lever: retired no-code tool spend.
- Value lever: reduced workflow incident investigation time.
- Value lever: AI governance enabling controlled adoption.
- Dollar anchor: "At 1,250 runs per month and 9 minutes saved per run, one workflow saves about 187.5 hours per month."
- Dollar anchor: "At 85 dollars per hour, that is about 191,000 dollars per year in labor capacity."
- Dollar anchor: "A half-percent improvement on an 18 million dollar renewal pool is 90,000 dollars."
- Dollar anchor: "Retiring three small workflow tools at 50,000 dollars each adds 150,000 dollars."
- Do not price AI as magic.
- Do not offer unlimited workflow generation.
- If buyer asks for per-user pricing, translate to builder, reviewer, runtime, and governance roles.
- If buyer asks for consumption pricing, discuss workflow runs, evidence retention, AI draft usage, and integration calls.
- If procurement asks for category, use `governed workflow automation platform`.
- If security asks for proof, offer sandbox-only pilot with no production writes.

## Follow-up Email Template

Subject: Follow-up from Oyatie Workflow Studio demo

Hi {{first_name}},

Thank you for joining the Workflow Studio walkthrough. I heard four core needs:

1. Give the business a faster way to build and modify workflows.
2. Prevent AI-generated automation from bypassing governance.
3. Test and simulate workflow changes before production.
4. Create evidence for who changed what, why, and under which policy.

The demo used the `tenant-acme-mid-market-saas` fixture and showed:

- A visual discount-approval renewal workflow.
- An AI-drafted renewal-risk escalation node.
- Human review of the generated diff.
- Test fixtures for threshold, data, and missing-field behavior.
- Cedar policy simulation for edit, approve, publish, and rollback.
- Sandbox publish with governance approval.
- Audit replay of both the workflow change and runtime case.
- A preliminary value model based on workflow volume, minutes saved, leakage reduction, and tool rationalization.

Recommended next step: a two-week Workflow Studio Pilot Design.

Proposed outputs:

- Pilot workflow selection.
- Current-state workflow map.
- Integration and data-scope inventory.
- Role and publish-authority model.
- AI-assistance governance boundary.
- Test fixture set.
- Sandbox workflow draft.
- Value model.
- Pilot recommendation.

Suggested attendees:

- Business workflow owner.
- Operations or RevOps leader.
- Business systems or IT owner.
- Security architecture.
- Compliance or governance reviewer.
- Finance or procurement observer if ROI is required.

Could we reserve 60 minutes next week to select the pilot workflow and confirm the systems touched?

Best,

{{sender_name}}

## References

- Internal: `registry/sample-tenants/tenant-acme-mid-market-saas.md`.
- Internal: `registry/sample-tenants/tenant-helios-fortune-500-manufacturer.md`.
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/decisions/ADR-0220-*`.
- Internal: `docs/decisions/ADR-0243-*`.
- Internal: `docs/decisions/ADR-0244-*`.
- Internal: `docs/decisions/ADR-0251-*`.
- Internal: `docs/decisions/ADR-0263-*`.
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/decisions/ADR-0317-*`.
- Internal: `specs/capability-tier-schema.json`.
- Internal: `specs/pack-overlay-schema.json`.
- External: Cedar policy language, https://www.cedarpolicy.com/.
- External: GDPR Regulation (EU) 2016/679, https://eur-lex.europa.eu/eli/reg/2016/679/oj.
- External: SOC 2 overview, AICPA, https://www.aicpa-cima.com/topic/audit-assurance/audit-and-assurance-greater-than-soc-2.
- External: NIST AI Risk Management Framework, https://www.nist.gov/itl/ai-risk-management-framework.
- Demo note: all account, renewal, approval, and integration examples in this script are synthetic.
