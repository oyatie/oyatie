---
doc_class: DemoScript
target_persona: Tier-1 bank CIO, COO, CRO, treasury operations leader, payments transformation sponsor, operational resilience officer
duration_minutes: 60
related_oyatie_adrs:
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0304
  - ADR-0316
  - ADR-0319
status: canonical
date: 2026-05-20
owner: Customer Success Engineering
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Financial Services Vertical Demo

## Pre-Demo Checklist

- Confirm meeting type: first executive demo for regulated financial-services buyer.
- Confirm prospect segment: bank, insurer, capital-markets firm, payment institution, or treasury-heavy corporate.
- Confirm primary sponsor: CIO, COO, CRO, CTO, treasurer, payments head, or operational resilience owner.
- Confirm stated pain: DORA evidence, EU AI Act controls, Basel III reporting, treasury fragmentation, payments modernization, or vendor risk.
- Confirm deal hypothesis: Oyatie as regulated operating substrate rather than single-point workflow tool.
- Confirm named tenant fixture: `tenant-nordic-bank-tier-1-finance`.
- Confirm fixture mode: read-only demo tenant with synthetic bank entities, synthetic payments, and synthetic treasury positions.
- Confirm data posture: no real client data, no production credentials, no live payment initiation.
- Confirm compliance posture: show controls and evidence patterns, not legal advice.
- Confirm demo duration: 60 minutes.
- Confirm agenda allocation: 7 minutes opening, 10 minutes discovery, 35 minutes product demo, 8 minutes commercial close.
- Open Oya Demo Console.
- Open Capability Tier Studio.
- Open Compliance Pack Activator.
- Open Operational Resilience Control Room.
- Open Treasury Command Center.
- Open Payments Orchestration Console.
- Open AI Governance Review Queue.
- Open Basel Scenario Workspace.
- Open Cedar Permit Simulator.
- Open Evidence Portal.
- Open Audit Replay Console.
- Open Cost and Margin Lens.
- Open Migration Mapper.
- Load tenant fixture `tenant-nordic-bank-tier-1-finance`.
- Load persona `chief-operating-officer`.
- Load persona `treasury-operations-lead`.
- Load persona `model-risk-officer`.
- Load persona `payments-operations-analyst`.
- Load persona `external-regulator-observer`.
- Enable compliance packs `DORA`, `GDPR`, `EU-AI-ACT`, `PSD2-SCA`, `Basel-III-ops-view`.
- Enable demo workflow `payments-incident-to-regulator-evidence`.
- Enable demo workflow `intraday-liquidity-risk-to-treasury-action`.
- Enable demo workflow `ai-credit-decision-human-oversight`.
- Enable demo workflow `basel-liquidity-scenario-executive-pack`.
- Prepare objection-response card `regulated-core-systems`.
- Prepare objection-response card `mainframe-modernization`.
- Prepare objection-response card `audit-evidence-fatigue`.
- Prepare objection-response card `model-risk-management`.
- Prepare objection-response card `payments-finality`.
- Prepare pricing card `bank-platform-and-compliance-packs`.
- Prepare pricing card `payments-volume-governed-usage`.
- Prepare leave-behind `FSI-Executive-Resilience-Map`.
- Prepare follow-up artifact `DORA-EUAI-Basel-Treasury-Workshop-Plan`.
- Verify browser zoom is 90 percent for screen share.
- Verify no internal sandbox banner is visible except the tenant fixture banner.
- Verify demo reset time is under 90 seconds.
- Verify Evidence Portal contains generated packets for DORA incident, AI oversight, and liquidity scenario.
- Stop condition: do not run live payment submission, production bank connectivity, or real regulator filing.

## Opening Hook

- Say: "For banks, transformation has stopped being a replacement-program problem."
- Say: "It is now an evidence problem."
- Say: "Every new payment rail, AI model, treasury workflow, vendor integration, and customer journey creates a second job: proving control."
- Say: "DORA asks whether critical operations can keep running and whether incidents can be classified, escalated, and evidenced quickly."
- Say: "The EU AI Act asks whether high-risk AI is governed with human oversight, traceability, and documented controls."
- Say: "Basel III asks whether balance-sheet, liquidity, and capital views are explainable under stress."
- Say: "Payments leaders still have to move money safely, settle exceptions, manage sanctions and approvals, and prevent operational loss."
- Say: "Most banks answer these with another workflow tool, another GRC repository, another model registry, and another data mart."
- Say: "That creates the 47-system problem inside a regulated institution."
- Say: "The business flow happens in one place, the authorization happens in another, the evidence is reconstructed somewhere else, and the regulator packet is assembled by people under deadline pressure."
- Say: "Oyatie's thesis is different."
- Say: "The bank should run on a tenant substrate where capability, policy, workflow, evidence, and cost are part of the same operating surface."
- Say: "We are not here to show a dashboard skin over a core."
- Say: "We are showing how a regulated bank can launch treasury and payments workflows while DORA, EU AI Act, Basel III, and audit evidence stay attached to the work itself."
- Say: "In the next hour, I will use the Nordic Bank fixture."
- Say: "It is synthetic, but it is structured like a tier-1 regulated bank."
- Say: "You will see one operational incident move from detection to DORA evidence."
- Say: "You will see one AI-assisted credit-control decision move through human oversight."
- Say: "You will see one Basel liquidity scenario flow into treasury action."
- Say: "You will see one payment exception handled with dual control and explainable permits."
- Say: "I will use dollar amounts and euro amounts so the economics are concrete."
- Say: "The outcome we want from this call is not a generic product tour."
- Say: "The outcome is deciding whether a two-week resilience-and-payments workshop is worth scheduling."
- Say: "If we get there, the workshop should produce one target operating model, one control map, one integration inventory, and one quantified business case."
- Say: "A reasonable first business case for a bank this size is not a five-percent productivity claim."
- Say: "It is reducing 20 to 40 million dollars of annual operational drag across audit assembly, incident response, payment exception handling, and duplicate workflow systems."
- Say: "The larger strategic case is lowering the risk and time required to modernize regulated operating workflows."
- Pause after the value frame.
- Ask the sponsor whether the emphasis should be DORA, AI governance, Basel reporting, treasury operations, or payments first.
- If they choose one, adjust the order of the first four screens.
- If they do not choose, keep the default sequence: DORA, AI, Basel, treasury/payments.

## Discovery Questions

1. Which regulated business service would you most want to defend in a DORA tabletop today?
2. Where does incident classification happen now, and who has authority to change severity?
3. How many systems must be queried before you can assemble a complete regulator evidence packet?
4. Which payment workflows create the most manual exception handling or operational loss exposure?
5. How do you currently prove dual control on payment release, payment repair, or treasury movement?
6. Which AI or analytics workflows are likely to be considered high-risk under EU AI Act governance?
7. Where do model risk, business owners, legal, and operations review AI-assisted decisions today?
8. How often do liquidity, capital, or treasury scenarios require manual spreadsheet reconciliation?
9. What is the cost of one major audit cycle in people time, consulting support, and business interruption?
10. Which vendor platforms are strategic, and which exist because a team needed a workflow quickly?
11. How do you separate parent-bank visibility from subsidiary sovereignty in regulated jurisdictions?
12. Where does cost attribution fail across payments, treasury, risk, compliance, and technology teams?
13. What would make a pilot credible to your risk committee within 30 days?
14. Which integration is non-negotiable for a first pilot: core banking, payment hub, treasury system, data lake, IAM, or GRC?
15. What would your regulator, internal audit, or second-line risk team need to see before endorsing expansion?

## Demo Flow

1. Screen: Tenant Posture and Regulated Service Map.
- Click `Demo Console`.
- Click `Tenant Switcher`.
- Select `tenant-nordic-bank-tier-1-finance`.
- Click `Load Regulated Service Map`.
- Say: "This tenant is not a product SKU."
- Say: "It is a regulated operating boundary with capabilities, policy, evidence, and cost controls attached."
- Point to `Critical Services`.
- Show `Retail Payments`, `Corporate Treasury`, `Credit Decision Support`, `Liquidity Risk`, and `Regulatory Evidence`.
- Click `Retail Payments`.
- Show the right panel `Active Packs`.
- Say: "DORA, GDPR, PSD2-SCA, EU AI Act, and Basel operational overlays are evaluated in the same place the workflow runs."
- Click `Policy Trace`.
- Show Cedar permit traces for `payment_exception_repair.read`, `payment_exception_repair.approve`, and `regulator_packet.export`.
- Say: "The important part is not that we have permissions."
- Say: "The important part is that permission decisions are durable evidence."
- Dollar example: "If your bank spends 8 million dollars annually assembling evidence across critical-service audits, this is the place we attack the waste."
- Click `Cost Lens`.
- Show synthetic annual control-labor estimate `$7,800,000`.
- Say: "We will use this estimate later when we discuss workshop economics."

2. Screen: DORA Incident Control Room.
- Click `Operational Resilience`.
- Click `Incidents`.
- Select incident `PAY-2026-05-18-014`.
- Say: "A payment queue degradation has crossed the materiality threshold configured for this tenant."
- Point to `Affected Service: Retail Payments`.
- Point to `Customer Impact: 18,400 delayed SEPA credit transfers`.
- Point to `Estimated Financial Exposure: EUR 4,200,000`.
- Click `Classification`.
- Show severity `Major ICT-related incident candidate`.
- Say: "In many banks, classification begins in an incident tool and evidence gets reconstructed later."
- Click `Evidence Boundaries`.
- Show data sources: payment queue metrics, incident bridge notes, Cedar approvals, customer notification workflow, vendor status, recovery actions.
- Click `Generate DORA Timeline`.
- Say: "Oyatie builds the regulator timeline from the events already emitted by the operating surface."
- Point to timestamps for detect, classify, escalate, mitigate, recover, and executive sign-off.
- Click `Missing Evidence`.
- Show missing item `Vendor root cause attestation`.
- Click `Assign`.
- Select owner `Third Party Risk`.
- Say: "The system does not pretend the packet is complete."
- Say: "It shows the gap, owner, deadline, and audit trail."
- Click `Regulator Preview`.
- Show export watermark `Draft - synthetic fixture`.
- Dollar example: "If a major incident pulls 35 people for 10 days at a blended loaded cost of 1,400 dollars per day, the manual response cost is 490,000 dollars before remediation."
- Say: "This is where automation has immediate financial value without weakening accountability."

3. Screen: DORA Evidence Packet Drilldown.
- Click `Evidence Portal`.
- Click packet `DORA-PAY-014`.
- Click `Control Coverage`.
- Show control rows: incident detection, classification, escalation, business impact, communication, recovery, third-party dependency, post-incident review.
- Say: "The packet is mapped to obligations, but the evidence is operational."
- Click `Detection Evidence`.
- Show metric snapshot with queue latency breach.
- Click `Escalation Evidence`.
- Show Cedar trace for `resilience_manager.escalate_major_incident`.
- Click `Communication Evidence`.
- Show customer notification approval and legal review.
- Click `Recovery Evidence`.
- Show rollback action and payment replay validation.
- Click `Export`.
- Choose `Regulator Draft Bundle`.
- Say: "We do not advise sending a regulator bundle from a demo."
- Say: "The point is that the export can be reviewed because the source trace is complete."
- Click `Reviewer Notes`.
- Add note: "Confirm external provider attestation before final classification."
- Say: "This preserves human judgment."
- Dollar example: "A bank that reduces two major evidence drills from 600 staff-hours to 180 staff-hours saves roughly 420 hours per drill. At 150 dollars per hour, that is 63,000 dollars per drill, before risk reduction."
- Click `Back to Incident`.

4. Screen: EU AI Act Governance Queue.
- Click `AI Governance`.
- Click `High-Risk Review Queue`.
- Select item `CRE-UNDERWRITING-ASSIST-221`.
- Say: "Now we switch from resilience to AI governance."
- Point to `Use Case: Credit exposure recommendation for SME lending`.
- Point to `AI Role: Decision support, not autonomous approval`.
- Point to `Human Oversight Required: Yes`.
- Click `Risk Controls`.
- Show controls: purpose limitation, training data lineage, input constraints, output explanation, human override, post-decision monitoring.
- Say: "The workflow makes the human reviewer visible."
- Click `Recommendation`.
- Show synthetic recommendation: "Reduce approved line from EUR 12,000,000 to EUR 9,500,000 due to concentration and liquidity signal."
- Click `Explain`.
- Show top factors: delayed receivables, sector exposure, liquidity volatility, prior covenant exceptions.
- Click `Human Review`.
- Select reviewer `Credit Risk Director`.
- Click `Override`.
- Enter reason: "Approved EUR 10,500,000 due to new parent guarantee; attach treasury confirmation."
- Say: "The override is not hidden."
- Say: "It becomes part of the AI governance evidence chain."
- Click `Cedar Trace`.
- Show permit `credit_risk_director.ai_recommendation.override`.
- Dollar example: "One wrong 12 million euro credit exposure is not a software productivity issue. It is a risk appetite and governance issue."
- Say: "Oyatie's job is to make AI adoption auditable enough for the bank to use it."

5. Screen: AI Control Evidence.
- Click `Evidence`.
- Click `EU AI Act Control Card`.
- Show fields: system purpose, risk classification, human oversight, data governance, logging, transparency, accuracy monitoring.
- Say: "This view is designed for second-line risk and model governance."
- Click `Data Lineage`.
- Show source systems: credit file, treasury exposure, customer consent, sector-risk table, payment-behavior aggregates.
- Click `Excluded Inputs`.
- Show protected attributes excluded by policy.
- Click `Monitoring`.
- Show drift alert status `No breach`.
- Click `Human Action Log`.
- Show original recommendation, explanation view, override, attached guarantee, final decision.
- Say: "The system does not claim the model was right."
- Say: "It proves the bank used the model inside a governed process."
- Dollar example: "If your model governance team spends 12,000 hours a year assembling evidence at 175 dollars loaded cost, the annual labor base is 2.1 million dollars."
- Say: "The savings matter, but the bigger value is allowing compliant AI usage instead of freezing it."

6. Screen: Basel III Liquidity Scenario Workspace.
- Click `Basel Scenario Workspace`.
- Click `Liquidity Stress Scenario`.
- Select scenario `Wholesale Funding Outflow - Moderate Stress`.
- Say: "Basel reporting often becomes disconnected from operational action."
- Point to `30-day net cash outflow`.
- Point to `High-quality liquid assets`.
- Point to `Synthetic LCR: 119 percent`.
- Click `Scenario Input`.
- Change outflow from `EUR 4.0B` to `EUR 5.2B`.
- Click `Run Scenario`.
- Show new synthetic LCR `105 percent`.
- Say: "The demonstration is not a Basel calculation engine replacement."
- Say: "It is showing how governed workflow links scenario insight to treasury action."
- Click `Create Treasury Action`.
- Select action `Increase overnight secured funding by EUR 600,000,000`.
- Click `Require Approvals`.
- Show approvals: Treasurer, Risk, Treasury Ops, CFO observer.
- Dollar example: "A 600 million euro funding action can change daily carry cost by roughly 45,000 euros depending on spread."
- Say: "That is why the approval chain and scenario evidence belong together."

7. Screen: Treasury Command Center.
- Click `Treasury Command Center`.
- Select `Intraday Liquidity`.
- Say: "Treasury sees real operational positions, not a disconnected report."
- Point to `Opening Cash: EUR 2.8B`.
- Point to `Expected Outflows: EUR 1.9B`.
- Point to `Expected Inflows: EUR 1.2B`.
- Point to `Projected Buffer: EUR 2.1B`.
- Click `Funding Plan`.
- Show actions: central-bank facility standby, secured repo, internal sweep, FX conversion.
- Click `Create Funding Instruction`.
- Enter amount `EUR 600,000,000`.
- Select funding source `Secured repo desk`.
- Select reason `Basel scenario mitigation`.
- Click `Simulate Approval`.
- Show Cedar decision `Allowed with CFO observer`.
- Say: "The permit is context-aware."
- Say: "The action is allowed because amount, role, scenario link, and observer are aligned."
- Click `Remove Scenario Link`.
- Click `Simulate Approval`.
- Show decision `Denied`.
- Say: "This is the kind of denial that protects treasury from orphaned actions."
- Dollar example: "If a wrong-day treasury movement costs 12 basis points on 600 million euros for one day, the cost is about 20,000 euros, and the audit issue can be worse."

8. Screen: Payments Exception Queue.
- Click `Payments Orchestration`.
- Click `Exceptions`.
- Select exception `SEPA-REPAIR-88319`.
- Say: "Now we move to payments operations."
- Point to `Amount: EUR 2,750,000`.
- Point to `Reason: Beneficiary bank identifier mismatch`.
- Point to `SLA: 42 minutes remaining`.
- Click `Repair Suggestions`.
- Show suggested BIC from directory match.
- Click `Evidence`.
- Show source: original instruction, directory lookup, sanctions status, customer approval, dual-control requirement.
- Click `Approve Repair`.
- Select persona `payments-operations-analyst`.
- Show decision `Denied: second approver required for amount above EUR 1,000,000`.
- Switch persona to `payments-supervisor`.
- Click `Approve Repair`.
- Show decision `Allowed pending release control`.
- Say: "Payment repair and payment release are separate permits."
- Click `Release`.
- Show second decision requiring treasury observer because amount exceeds EUR 2,500,000.
- Dollar example: "If this payment misses a same-day settlement window, the corporate client penalty exposure is EUR 35,000, plus relationship damage."
- Say: "The workflow is fast, but speed never bypasses control."

9. Screen: PSD2 and Strong Customer Authentication Evidence.
- Click `Payment Evidence`.
- Click `Customer Authorization`.
- Show synthetic authorization evidence.
- Say: "For retail or corporate payment flows, authorization evidence needs to travel with the exception."
- Click `SCA Trace`.
- Show status `Satisfied by corporate mandate plus delegated authority`.
- Click `Mandate`.
- Show mandate ID, authorized signers, expiry date, delegated limit.
- Click `Cedar Trace`.
- Show policy checks for role, amount, mandate status, jurisdiction, and payment type.
- Say: "The SE does not need to over-explain PSD2 here."
- Say: "The point is that the payment team can answer who approved what, why it was allowed, and what evidence supported it."
- Dollar example: "Ten thousand repairs per month at 12 minutes of avoidable handling is 2,000 hours. At 85 dollars loaded cost, that is 170,000 dollars per month of labor capacity."

10. Screen: Third-Party Dependency and DORA Vendor Evidence.
- Click `Vendor Dependencies`.
- Select vendor `Payment Gateway Provider B`.
- Say: "DORA also sharpens third-party accountability."
- Point to `Supports: Retail Payments, Corporate Payments`.
- Point to `Criticality: High`.
- Click `Incident Linkage`.
- Show related incident `PAY-2026-05-18-014`.
- Click `Contract Evidence`.
- Show exit plan, SLA, concentration risk, last resilience test.
- Click `Attestation Request`.
- Show pending attestation assigned to vendor manager.
- Say: "This turns vendor evidence into a workflow obligation."
- Click `Escalate Missing Attestation`.
- Show Cedar permit `third_party_risk.escalate_vendor_gap`.
- Dollar example: "A missed vendor attestation can turn a 300,000 dollar incident into a board-level control finding."
- Say: "The value is not a prettier vendor inventory."
- Say: "The value is a current dependency map tied to incidents and obligations."

11. Screen: Regulator Observer View.
- Click `Persona Switcher`.
- Select `external-regulator-observer`.
- Click `Evidence Portal`.
- Say: "This is a controlled observer view."
- Point to masked commercial fields.
- Point to visible incident timeline.
- Point to visible control evidence.
- Click `Denied Areas`.
- Show denied access to customer PII and internal pricing.
- Click `Cedar Explanation`.
- Show reason `observer role can view packet evidence, not underlying personal data`.
- Say: "For regulated demos, this screen lands well with audit and second-line teams."
- Say: "They can see that transparency does not mean uncontrolled access."
- Dollar example: "If regulator preparation pulls 15 senior people for a week, that is easily 90,000 dollars of internal cost per request."
- Say: "More importantly, it reduces the chance that rushed access creates a privacy incident."

12. Screen: Integrated Evidence Search.
- Switch persona to `chief-operating-officer`.
- Click `Evidence Search`.
- Search `PAY-2026-05-18-014 AI override treasury action`.
- Say: "This search crosses operational domains while preserving permissions."
- Open result `DORA incident timeline`.
- Open result `Treasury funding instruction`.
- Open result `AI credit override`.
- Say: "The COO sees the connected operating story."
- Click `Compare Controls`.
- Show shared controls: human approval, Cedar permit trace, evidence export, retention rule.
- Say: "This is the platform pattern."
- Say: "Each domain is different, but the evidence and policy mechanics are common."
- Dollar example: "This is how one platform replaces three or four narrow governance add-ons without pretending all financial workflows are identical."

13. Screen: Migration Mapper.
- Click `Migration Mapper`.
- Select current systems: `ServiceNow GRC`, `Archer`, `Payment Hub Workflow`, `Treasury Spreadsheet Pack`, `Model Registry`.
- Say: "We do not start by ripping out the core."
- Say: "We start by mapping operating flows and evidence gaps."
- Click `Generate Migration Heatmap`.
- Show waves: evidence overlay, workflow pilot, payments exception pilot, treasury scenario pilot, controlled expansion.
- Click `Wave 1`.
- Show duration `2 weeks`.
- Show outputs: control map, integration inventory, evidence packet, economic case.
- Click `Wave 2`.
- Show duration `6 to 8 weeks`.
- Show outputs: pilot workflow, IAM integration, audit review, production go/no-go.
- Dollar example: "A conservative first-year value case might target 3.5 million dollars from audit automation, 2.8 million from payment exception productivity, and 1.2 million from duplicate workflow rationalization."
- Say: "That is 7.5 million dollars before counting risk avoidance."

14. Screen: Cost and Margin Lens.
- Click `Cost and Margin`.
- Select `Financial Services Operating View`.
- Say: "We make cost visible because platform adoption has to survive CFO review."
- Show costs: platform subscription, regulated pack activation, integration services, usage-based evidence exports, payment workflow volume.
- Show benefits: audit hours reduced, incident response hours reduced, payment repair hours reduced, duplicate system retirements, faster AI approvals.
- Click `Scenario`.
- Set bank employee count `65,000`.
- Set regulated workflows in scope `12`.
- Set annual payment exceptions `180,000`.
- Click `Calculate`.
- Show synthetic business case:
- Line `Year 1 Gross Benefit: $8,700,000`.
- Line `Year 1 Oyatie Program Cost: $3,250,000`.
- Line `Net Year 1 Impact: $5,450,000`.
- Line `Payback: 6.1 months`.
- Say: "We will not ask you to accept these numbers today."
- Say: "The workshop tests them with your real volumes."

15. Screen: Executive Decision Summary.
- Click `Executive Summary`.
- Select sections: DORA incident, EU AI Act control, Basel treasury action, payments exception, economics.
- Click `Generate Readout`.
- Say: "This is the artifact we want your steering group to see."
- Show one-page summary with control map and business case.
- Point to `Decision Required`.
- Say: "The decision is not whether to replace every system tomorrow."
- Say: "The decision is whether these operating patterns are worth a focused workshop."
- Click `Workshop Plan`.
- Show proposed attendees: COO delegate, treasury ops, payments ops, risk, compliance, architecture, security, IAM, data, procurement.
- Show proposed output: prioritized pilot backlog and business case.
- Dollar example: "If the two-week workshop costs 85,000 dollars in services and internal time, the hurdle is identifying at least 850,000 dollars of credible first-year value."
- Say: "For this profile, that hurdle is usually modest."

## Objection Handling

1. Objection: "We already have a GRC platform."
- Response name: GRC-Is-Not-Operations.
- Say: "We are not trying to replace every GRC record on day one."
- Say: "The gap we target is the space between operating action and reconstructed evidence."
- Say: "Oyatie can feed or coexist with a GRC platform while making the source evidence stronger."

2. Objection: "Our core banking systems cannot be disturbed."
- Response name: Core-Safe-Overlay.
- Say: "The first wave does not require replacing the core."
- Say: "We begin with governed workflow, evidence, and exception orchestration around selected flows."
- Say: "Core writes stay behind explicit integration controls."

3. Objection: "Regulators will not accept generated evidence."
- Response name: Trace-First-Evidence.
- Say: "The packet is generated, but the evidence is not invented."
- Say: "Every item links back to operational events, permits, approvals, and controls."
- Say: "Your legal and regulatory teams remain final reviewers."

4. Objection: "EU AI Act classification is still evolving."
- Response name: Configurable-AI-Governance.
- Say: "That is why governance should be policy-driven and adaptable."
- Say: "The workflow captures purpose, oversight, data lineage, logging, and human review even as interpretations mature."
- Say: "We configure the pack with your risk and legal teams."

5. Objection: "Basel III calculations are handled elsewhere."
- Response name: Scenario-To-Action.
- Say: "Oyatie is not positioned as the regulatory calculation engine in this demo."
- Say: "We connect scenario outputs to governed treasury actions and evidence."
- Say: "That closes the actionability gap."

6. Objection: "Payment operations require very low latency."
- Response name: Policy-At-The-Right-Step.
- Say: "Low-latency rails stay low latency."
- Say: "Oyatie focuses on governed exception, approval, repair, evidence, and human-control surfaces."
- Say: "Where runtime latency matters, we design for asynchronous evidence capture or precomputed permits."

7. Objection: "Our bank has too many jurisdictions for one platform."
- Response name: Jurisdictional-Pack-Composition.
- Say: "The point is not one universal rule."
- Say: "It is composable overlays per tenant, service, workflow, and jurisdiction."
- Say: "The demo shows DORA, EU AI Act, GDPR, PSD2, and Basel overlays coexisting without flattening their differences."

8. Objection: "Model risk will not trust business-built AI workflows."
- Response name: Human-Reviewed-AI-Drafts.
- Say: "AI-assisted work is not auto-approved."
- Say: "Human review, lineage, override reason, and monitoring are first-class artifacts."
- Say: "Model risk gets a governed review surface rather than unmanaged shadow AI."

9. Objection: "This seems broad."
- Response name: Narrow-Pilot-Entry.
- Say: "The platform is broad, but the first pilot should be narrow."
- Say: "For a bank, we usually pick one regulated service and one measurable workflow."
- Say: "DORA incident evidence or payments exception repair are strong candidates."

10. Objection: "We cannot share sensitive production data."
- Response name: Synthetic-First-Then-Controlled-Data.
- Say: "The workshop can start with synthetic and masked data."
- Say: "Production integration requires your security, privacy, and architecture gates."
- Say: "We do not need raw production data to validate workflow shape and control design."

11. Objection: "Our auditors need immutable logs."
- Response name: Audit-Replay-And-Retention.
- Say: "The evidence chain includes event trace, permit decision, actor, timestamp, control mapping, and retention policy."
- Say: "We can map that to your existing audit-retention requirements."
- Say: "The pilot should include audit review before expansion."

12. Objection: "This duplicates ServiceNow."
- Response name: Workflow-Of-Record-Clarity.
- Say: "Some incident or ticket workflows may remain in ServiceNow."
- Say: "Oyatie becomes valuable where regulated operational work, policy, and evidence need to run together."
- Say: "We define system-of-record boundaries in the workshop."

13. Objection: "We need proof this can integrate with payment hubs."
- Response name: Integration-Inventory-First.
- Say: "The first workshop output is an integration inventory and risk map."
- Say: "We identify read, write, event, and approval touchpoints separately."
- Say: "A pilot can begin with read-only evidence capture before controlled writes."

14. Objection: "The business case is hard to measure."
- Response name: Volume-Based-Economics.
- Say: "We anchor economics in concrete volumes."
- Say: "Payment exceptions, audit requests, incident drills, model reviews, and duplicate workflow licenses are measurable."
- Say: "The workshop replaces guesses with your numbers."

15. Objection: "We cannot add another platform."
- Response name: Platform-Rationalization.
- Say: "That is the reason to evaluate Oyatie."
- Say: "The target is fewer disconnected workflow, evidence, and compliance tools over time."
- Say: "The first pilot should prove consolidation potential, not add permanent sprawl."

16. Objection: "Treasury will not let workflow slow them down."
- Response name: Treasury-Control-Without-Drag.
- Say: "Treasury actions need fast paths and strong exceptions."
- Say: "The demo shows contextual permits and scenario-linked approvals to avoid unnecessary review."
- Say: "We design thresholds with treasury, risk, and finance together."

17. Objection: "Who owns the policy?"
- Response name: Three-Line-Policy-Ownership.
- Say: "Business owners define workflow intent."
- Say: "Risk and compliance define control obligations."
- Say: "Technology implements and verifies the policy-as-control surface."

18. Objection: "What happens during an outage?"
- Response name: Resilience-By-Design.
- Say: "DORA-aligned operation requires fallback paths, evidence continuity, and recovery procedures."
- Say: "The pilot should include degraded-mode behavior and evidence replay."
- Say: "We test that explicitly before production."

19. Objection: "Our subsidiaries have separate controls."
- Response name: Tenant-Sovereignty.
- Say: "Oyatie supports tenant-specific control overlays."
- Say: "Parent visibility and subsidiary sovereignty are separated by permits."
- Say: "That matters for regulated groups."

20. Objection: "We need procurement clarity."
- Response name: Workshop-To-Commercial-Pack.
- Say: "We do not need a full enterprise commitment to start."
- Say: "The next step can be a bounded workshop with clear deliverables."
- Say: "Commercial expansion follows validated value and governance approval."

## Closing Call to Action

- Say: "The demo showed four connected patterns."
- Say: "DORA evidence can be produced from the incident workflow rather than reconstructed afterward."
- Say: "EU AI Act governance can be embedded into human-reviewed AI decisions."
- Say: "Basel scenario insight can move into treasury action with approval context attached."
- Say: "Payments exceptions can move faster without losing dual control or evidence."
- Say: "The recommended next step is a two-week Financial Services Operating Evidence Workshop."
- Propose day 1: regulated service selection and stakeholder alignment.
- Propose day 2: integration inventory and data boundary review.
- Propose day 3: DORA evidence map and incident workflow review.
- Propose day 4: payments exception or treasury workflow map.
- Propose day 5: AI governance and model-risk review.
- Propose week 2 day 1: Cedar permit model and IAM mapping.
- Propose week 2 day 2: evidence packet prototype.
- Propose week 2 day 3: economic model with bank volumes.
- Propose week 2 day 4: architecture and security review.
- Propose week 2 day 5: executive readout with pilot recommendation.
- Ask: "Can we schedule the workshop planning call with operations, risk, treasury, payments, architecture, and compliance this week?"
- Ask: "Who besides you must sign off on the pilot scope?"
- Ask: "Which regulated service should we use as the workshop anchor?"
- If they hesitate, offer a smaller entry: one 90-minute DORA evidence design session.
- If they are interested, secure calendar owners before the call ends.

## Pricing Conversation Anchors

- Anchor the conversation around regulated operating value, not seat count.
- Explain that financial-services pricing usually has three components.
- Component 1: tenant platform subscription for the regulated operating substrate.
- Component 2: compliance-pack activation for DORA, EU AI Act, GDPR, PSD2, Basel-oriented overlays, and evidence retention.
- Component 3: implementation and integration services for selected workflows.
- For a tier-1 bank pilot, use a bounded workshop anchor before enterprise pricing.
- Suggested workshop anchor: $85,000 to $150,000 depending on integration depth and evidence scope.
- Suggested pilot anchor: $450,000 to $900,000 for one regulated service, one to two workflows, IAM integration, evidence review, and security governance.
- Suggested enterprise expansion anchor: low seven figures annually for platform and packs, before high-volume custom integrations.
- Use concrete value levers.
- Value lever: audit evidence labor reduction.
- Value lever: incident response coordination and evidence speed.
- Value lever: payment exception handling productivity.
- Value lever: duplicate workflow and GRC-adjacent tool rationalization.
- Value lever: AI governance enablement for high-risk workflows.
- Value lever: treasury control and scenario-to-action traceability.
- Dollar anchor: "If we find only 3 million dollars in first-year value, a 750,000 dollar pilot has a 4x gross coverage ratio."
- Dollar anchor: "If payment exception handling alone saves 20,000 hours at 85 dollars per hour, that is 1.7 million dollars annually."
- Dollar anchor: "If audit evidence automation saves 15,000 hours at 150 dollars per hour, that is 2.25 million dollars annually."
- Dollar anchor: "If one major DORA evidence failure is avoided, the risk-adjusted value can exceed the whole first-year program."
- Keep legal/regulatory benefit framed as risk reduction and evidence quality, not guaranteed compliance.
- If procurement asks for per-seat pricing, say: "We can map named users, but the commercial logic is regulated capability plus workflow volume."
- If finance asks for ROI, offer a joint value model using their volumes.
- If they ask for discounting, trade only against scope, term, referenceability, or workshop-to-pilot conversion.
- Do not offer unlimited enterprise rights in the first call.
- Do not imply Oyatie replaces the core banking system in the pilot.
- Do not imply regulator acceptance is automatic.

## Follow-up Email Template

Subject: Follow-up from Oyatie financial services demo

Hi {{first_name}},

Thank you for the discussion today. I heard five priorities:

1. Reduce the manual burden of DORA incident evidence and resilience reporting.
2. Govern AI-assisted financial workflows with human oversight and traceable decisions.
3. liquidity and Basel-style scenario insight to controlled treasury action.
4. Improve payment exception handling without weakening dual control.
5. Build a credible business case without disrupting core banking systems in the first wave.

The demo used the `tenant-nordic-bank-tier-1-finance` fixture and showed:

- A payment incident moving into a DORA evidence packet.
- A high-risk AI credit recommendation moving through human override and governance evidence.
- A liquidity scenario producing a governed treasury funding action.
- A payment repair requiring contextual dual control and permit evidence.
- A preliminary value model using audit, incident, payment exception, and workflow-rationalization levers.

Recommended next step: a two-week Financial Services Operating Evidence Workshop.

Proposed outputs:

- Regulated service selection.
- Current-state integration inventory.
- DORA evidence map.
- AI governance control map.
- Payments or treasury workflow pilot definition.
- Cedar permit and IAM mapping.
- Evidence packet prototype.
- First-year value model.
- Executive pilot recommendation.

Suggested attendees:

- Operations owner.
- Payments operations lead.
- Treasury operations lead.
- Risk or operational resilience owner.
- Compliance lead.
- Model risk or AI governance lead.
- Enterprise architecture.
- Security and IAM.
- Procurement or finance observer.

Proposed scheduling question:

Could we reserve 60 minutes next week to select the regulated service and workshop participants?

Best,

{{sender_name}}

## References

- Internal: `registry/sample-tenants/tenant-nordic-bank-tier-1-finance.md`.
- Internal: `docs/COMPLIANCE-MATRIX.md`.
- Internal: `specs/pack-overlay-schema.json`.
- Internal: `specs/capability-tier-schema.json`.
- Internal: `docs/adr-archive/ADR-0243-cedar-as-universal-gate.md
- Internal: `docs/adr-archive/ADR-0244-tenant-as-universal-scoping-primitive.md
- Internal: `docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md
- Internal: `docs/adr-archive/ADR-0263-observability-emission-contract.md
- Internal: `docs/adr-archive/ADR-0304-cross-jurisdiction-conflict-resolution.md
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/adr-archive/ADR-0319-front-middle-back-office-information-barrier.md
- External: DORA Regulation (EU) 2022/2554, https://eur-lex.europa.eu/eli/reg/2022/2554/oj.
- External: EU AI Act Regulation (EU) 2024/1689, https://eur-lex.europa.eu/eli/reg/2024/1689/oj.
- External: Basel III Framework, Bank for International Settlements, https://www.bis.org/basel_framework/.
- External: GDPR Regulation (EU) 2016/679, https://eur-lex.europa.eu/eli/reg/2016/679/oj.
- External: European Banking Authority DORA materials, https://www.eba.europa.eu/.
- External: European Central Bank supervisory priorities, https://www.bankingsupervision.europa.eu/.
- External: PSD2 Directive (EU) 2015/2366, https://eur-lex.europa.eu/eli/dir/2015/2366/oj.
- External: Cedar policy language, https://www.cedarpolicy.com/.
- Demo note: all payment, treasury, and bank examples in this script are synthetic.
