---
doc_class: DemoScript
target_persona: Health system CIO, CMIO, compliance officer, revenue cycle executive, clinical operations leader, integration architect
duration_minutes: 60
related_oyatie_adrs:
  - ADR-0219
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0316
  - ADR-0319
status: canonical
date: 2026-05-20
owner: Customer Success Engineering
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Healthcare System Demo

## Pre-Demo Checklist

- Confirm meeting type: first health-system product demo or clinical-transformation follow-up.
- Confirm prospect segment: academic medical center, integrated delivery network, payer-provider group, specialty hospital network, or regional health system.
- Confirm primary sponsor: CIO, CMIO, compliance officer, integration leader, clinical operations executive, or revenue cycle sponsor.
- Confirm stated pain: fragmented workflows, EHR-adjacent manual work, HIPAA audit pressure, Part 11 signatures, FHIR integration backlog, HL7 interface fragility, or AI governance.
- Confirm deal hypothesis: Oyatie as governed healthcare workflow substrate around clinical, compliance, and integration surfaces.
- Confirm named tenant fixture: `tenant-crescent-health-academic-medical-center`.
- Confirm fixture mode: synthetic patient, synthetic orders, synthetic providers, synthetic interfaces, no real protected health information.
- Confirm data posture: demo PHI is fake, production PHI is never used in a prospect demo.
- Confirm regulatory posture: show compliance support and evidence patterns, not legal advice.
- Confirm demo duration: 60 minutes.
- Confirm agenda allocation: 7 minutes opening, 10 minutes discovery, 35 minutes demo, 8 minutes commercial and next step.
- Open Oya Demo Console.
- Open Healthcare Integration Hub.
- Open FHIR Resource Explorer.
- Open HL7 Message Monitor.
- Open Compliance Pack Activator.
- Open HIPAA Access Review.
- Open Part 11 Signature Console.
- Open Clinical Workflow Studio.
- Open AI Clinical Review Queue.
- Open Evidence Portal.
- Open Cedar Permit Simulator.
- Open Audit Replay Console.
- Open Cost and Throughput Lens.
- Open Migration Mapper.
- Load tenant fixture `tenant-crescent-health-academic-medical-center`.
- Load persona `clinical-operations-lead`.
- Load persona `attending-physician`.
- Load persona `nurse-manager`.
- Load persona `health-information-management`.
- Load persona `privacy-officer`.
- Load persona `integration-architect`.
- Load persona `research-coordinator`.
- Load persona `external-auditor-observer`.
- Enable compliance packs `HIPAA`, `21-CFR-Part-11`, `FHIR-HL7-Integration`, `EU-AI-ACT-Clinical-AI`.
- Enable demo workflow `discharge-care-gap-to-follow-up`.
- Enable demo workflow `lab-result-signature-amendment`.
- Enable demo workflow `break-glass-emergency-access-review`.
- Enable demo workflow `fhir-hl7-interface-reconciliation`.
- Enable demo workflow `clinical-ai-human-review`.
- Prepare objection-response card `ehr-coexistence`.
- Prepare objection-response card `hipaa-minimum-necessary`.
- Prepare objection-response card `part-11-validation`.
- Prepare objection-response card `interface-engine-overlap`.
- Prepare objection-response card `clinician-adoption`.
- Prepare pricing card `healthcare-platform-and-compliance-packs`.
- Prepare leave-behind `Healthcare-Workflow-Evidence-Map`.
- Prepare follow-up artifact `HIPAA-Part11-FHIR-HL7-Workshop-Plan`.
- Verify demo browser zoom is 90 percent.
- Verify patient names are synthetic and visibly tagged.
- Verify no production endpoint is selected in Integration Hub.
- Verify Evidence Portal has generated HIPAA, Part 11, FHIR, and audit replay examples.
- Verify Part 11 signature credentials are demo credentials only.
- Stop condition: do not diagnose patients, prescribe care, submit orders to a real system, or expose real PHI.

## Opening Hook

- Say: "Healthcare systems do not need another portal."
- Say: "They need a safer way to run the work that happens between portals."
- Say: "EHRs hold the clinical record."
- Say: "Interface engines move messages."
- Say: "GRC tools track controls."
- Say: "Workflow tools coordinate teams."
- Say: "But the real operational burden lives between them."
- Say: "A discharge plan depends on clinical context, payer rules, patient outreach, order status, privacy controls, and evidence."
- Say: "A lab amendment may require Part 11 signature evidence, patient-record linkage, provider attestation, and audit review."
- Say: "A break-glass access event requires care urgency, access justification, minimum necessary review, and privacy follow-up."
- Say: "An AI-assisted clinical workflow requires human oversight, traceability, and clear separation between support and autonomous decision."
- Say: "Most health systems solve this by adding a workflow layer that is not truly tied to compliance or integration evidence."
- Say: "Oyatie's thesis is that the workflow, policy, integration state, and evidence should be one operating surface."
- Say: "Today I will use the Crescent Health fixture."
- Say: "It is a synthetic academic medical center tenant with HIPAA, 21 CFR Part 11, FHIR, HL7, and clinical AI governance controls."
- Say: "You will see a discharge follow-up workflow that reads healthcare integration context."
- Say: "You will see FHIR and HL7 reconciliation without pretending the EHR disappears."
- Say: "You will see a Part 11 signature event that keeps audit evidence attached."
- Say: "You will see a break-glass event reviewed under HIPAA."
- Say: "You will see a clinical AI suggestion routed through human review."
- Say: "The value case is not replacing the EHR."
- Say: "The value case is reducing the expensive, risky, manual work around the EHR."
- Say: "For a system your size, a credible first-year case might be 4 to 9 million dollars across discharge leakage, integration triage, audit preparation, and duplicated workflow tools."
- Say: "The point of today's demo is to decide whether a two-week healthcare workflow evidence workshop is worth scheduling."
- Pause.
- Ask: "Should I emphasize clinical operations, compliance evidence, integration architecture, or AI governance first?"
- If CMIO leads, start with discharge workflow.
- If compliance leads, start with HIPAA and Part 11.
- If integration architecture leads, start with FHIR and HL7.
- If CIO leads, keep the default sequence.

## Discovery Questions

1. Which workflow creates the most operational burden outside the EHR today?
2. Where do staff leave the EHR and enter spreadsheets, email, ticketing, or local workflow tools?
3. Which HIPAA access reviews create the most manual evidence work?
4. How do you prove minimum necessary access for exception workflows?
5. Which processes require 21 CFR Part 11 signature evidence or equivalent electronic-signature controls?
6. How many FHIR APIs and HL7 interfaces support the workflow you would pilot first?
7. Where does interface reconciliation happen when the EHR, lab system, and downstream workflow disagree?
8. How do clinical leaders review AI-assisted recommendations today?
9. Which patient-safety or care-continuity process has the strongest business case for improvement?
10. What is the annual cost of audit preparation, privacy review, or integration triage in staff time?
11. Which EHR integration patterns are approved by your architecture and security teams?
12. How do you separate clinical, research, billing, and privacy roles in operational workflows?
13. What would legal, privacy, and clinical governance require before a pilot touches production metadata?
14. Which outcome would make a 30-day pilot worth expanding?
15. Who must sponsor the work: CIO, CMIO, compliance, nursing operations, revenue cycle, or integration leadership?

## Demo Flow

1. Screen: Crescent Health Tenant Overview.
- Click `Demo Console`.
- Click `Tenant Switcher`.
- Select `tenant-crescent-health-academic-medical-center`.
- Click `Load Healthcare Operating View`.
- Say: "This tenant is a healthcare operating boundary, not an EHR replacement."
- Point to `Active Domains`.
- Show `Clinical Operations`, `Integration`, `Compliance`, `Research`, `Revenue Cycle`, and `Evidence`.
- Click `Active Packs`.
- Show `HIPAA`, `21 CFR Part 11`, `FHIR-HL7 Integration`, and `Clinical AI Governance`.
- Say: "Packs are evaluated in the workflow where work happens."
- Click `Synthetic Patient Guardrail`.
- Show label `Synthetic PHI Only`.
- Say: "We are using fake patient data."
- Click `Cedar Permit Trace`.
- Show sample permits for provider, nurse manager, privacy officer, and auditor.
- Dollar example: "If your health system spends 6 million dollars a year on audit assembly and privacy review labor, the first value target is not theoretical."

2. Screen: Healthcare Integration Hub.
- Click `Healthcare Integration Hub`.
- Click `Interfaces`.
- Show FHIR APIs: Patient, Encounter, Observation, MedicationRequest, ServiceRequest, DocumentReference.
- Show HL7 feeds: ADT, ORU, ORM, SIU.
- Say: "This screen is for integration architects."
- Say: "It shows how workflow depends on healthcare data movement."
- Click `Interface Health`.
- Show `ADT feed healthy`, `ORU feed delayed`, `FHIR Observation API healthy`, `Scheduling feed warning`.
- Click `ORU delayed`.
- Show delayed lab-result messages.
- Say: "We do not hide interface problems behind a workflow status."
- Click `Affected Workflows`.
- Show `Lab Result Amendment`, `Discharge Follow-up`, `Research Eligibility`.
- Dollar example: "If integration triage consumes 1,500 hours per month at 120 dollars loaded cost, that is 180,000 dollars monthly before clinical delay impact."
- Click `Create Reconciliation Task`.
- Assign to `integration-architect`.
- Say: "The reconciliation task inherits the integration evidence."

3. Screen: FHIR Resource Explorer.
- Click `FHIR Resource Explorer`.
- Search synthetic patient `CRES-PT-10429`.
- Open `Patient`.
- Say: "The patient resource is synthetic, but the interaction pattern is realistic."
- Click `Encounter`.
- Show inpatient encounter `ENC-2026-0518-44`.
- Click `Observation`.
- Show lab observation with abnormal flag.
- Click `DocumentReference`.
- Show discharge summary draft.
- Click `Consent`.
- Show consent state for care management outreach.
- Say: "The workflow can see the clinical context it is authorized to use."
- Click `Cedar Trace`.
- Show permit `clinical_operations_lead.read_care_coordination_context`.
- Say: "The permit is scoped to care coordination."
- Click `Attempt Billing Data`.
- Show decision `Denied`.
- Say: "Minimum necessary is not a slogan here."
- Dollar example: "A privacy incident investigation can cost 50,000 to 250,000 dollars in internal and external effort before any reputational impact."

4. Screen: HL7 Message Monitor.
- Click `HL7 Message Monitor`.
- Select message `ORU-R01-889412`.
- Say: "HL7 is still very real in hospitals."
- Point to synthetic segments: MSH, PID, OBR, OBX.
- Click `Parse`.
- Show mapped Observation resource.
- Click `Mismatch`.
- Show issue `Specimen timestamp differs from EHR encounter timestamp by 74 minutes`.
- Click `Reconcile`.
- Choose action `Hold workflow until lab verification`.
- Say: "A workflow action can be gated by interface reconciliation."
- Click `Notify Lab Supervisor`.
- Show task assigned with message evidence.
- Dollar example: "Preventing one erroneous downstream clinical task can be worth far more than the minutes saved."
- Say: "The platform is not claiming to be the interface engine."
- Say: "It uses integration state to govern operational work."

5. Screen: Discharge Care Gap Workflow.
- Click `Clinical Workflow Studio`.
- Open workflow `Discharge Care Gap to Follow-up`.
- Say: "This is the operational workflow around a discharge."
- Point to nodes: patient context, medication reconciliation, follow-up scheduling, care-gap check, outreach, evidence capture.
- Click `Run Demo Case`.
- Select patient `CRES-PT-10429`.
- Click `Start`.
- Show first task `Confirm discharge summary ready`.
- Switch persona to `nurse-manager`.
- Click `Complete`.
- Show next task `Schedule cardiology follow-up within 7 days`.
- Click `Scheduling Context`.
- Show HL7 SIU warning.
- Say: "The scheduling feed warning is visible inside the workflow."
- Click `Escalate Scheduling Gap`.
- Assign to `care-coordination-team`.
- Dollar example: "If avoidable readmissions cost 14,500 dollars each and better follow-up prevents 40 readmissions annually, the gross value is 580,000 dollars."
- Say: "That is before patient experience and quality metrics."

6. Screen: HIPAA Minimum Necessary Review.
- Click `HIPAA Access Review`.
- Select event `ACCESS-2026-0518-BG-17`.
- Say: "This is a break-glass event."
- Point to actor `Emergency Department Attending`.
- Point to reason `Unconscious patient, immediate treatment`.
- Point to accessed resources: allergies, medications, problem list, recent labs.
- Click `Minimum Necessary`.
- Show evaluation `Allowed under emergency treatment context; review required`.
- Click `Post-Event Review`.
- Assign reviewer `privacy-officer`.
- Say: "Emergency access is allowed, but it is not invisible."
- Click `Cedar Trace`.
- Show permit with break-glass condition and review obligation.
- Click `Denied Follow-On`.
- Show denied access to unrelated research dataset.
- Dollar example: "If privacy teams manually review 4,000 exception events per year at 20 minutes each, that is about 1,333 hours of review time."

7. Screen: HIPAA Evidence Packet.
- Click `Evidence Portal`.
- Open packet `HIPAA-BREAKGLASS-17`.
- Show packet sections: access reason, actor, patient context, resources viewed, review owner, review deadline, outcome, retention rule.
- Say: "The packet is designed for privacy and audit teams."
- Click `Actor Evidence`.
- Show provider credential and care-team context.
- Click `Resource Evidence`.
- Show resource-level access list.
- Click `Review Evidence`.
- Show privacy officer task and attestation field.
- Click `Export Draft`.
- Say: "Export is draft until the privacy officer completes review."
- Click `Add Reviewer Note`.
- Add note: "Confirm emergency context in ED record before final close."
- Dollar example: "Reducing average review time from 20 minutes to 8 minutes across 4,000 events saves 800 hours annually."

8. Screen: Part 11 Signature Console.
- Click `Part 11 Signature`.
- Open event `LAB-AMEND-2026-0518-02`.
- Say: "Now we move to electronic-signature control."
- Point to `Action: Amend released lab interpretation`.
- Point to `Reason: Corrected reference range after analyzer calibration issue`.
- Point to `Requires Signature: Attending Pathologist`.
- Switch persona to `attending-physician`.
- Click `Review Amendment`.
- Show original result, amended result, reason, affected patient, interface source.
- Click `Sign`.
- Enter demo signature reason `I approve corrected interpretation after calibration review`.
- Click `Apply Demo Signature`.
- Show signature record: signer, timestamp, meaning, linked record, system check.
- Say: "The signature meaning is explicit."
- Click `Audit Trail`.
- Show before value, after value, reason, signature, reviewer.
- Dollar example: "A Part 11 validation finding can cost months of remediation. The pilot should prove evidence shape early."

9. Screen: Part 11 Audit Replay.
- Click `Audit Replay`.
- Select `LAB-AMEND-2026-0518-02`.
- Click `Replay`.
- Show timeline: lab interface delay, calibration issue, amendment draft, physician review, signature, patient-record update, downstream notification.
- Say: "Audit replay is where compliance teams see the whole story."
- Click `Tamper Check`.
- Show demo status `No evidence mutation detected`.
- Click `Retention`.
- Show retention rule mapped to tenant policy.
- Click `Export`.
- Choose `Audit Review Bundle`.
- Say: "For regulated healthcare workflows, the audit story must be available without heroic reconstruction."
- Dollar example: "If each validation audit requires 300 hours of evidence assembly, cutting 60 percent is 180 hours back per audit."

10. Screen: Clinical AI Review Queue.
- Click `AI Clinical Review`.
- Select item `READMISSION-RISK-ASSIST-332`.
- Say: "This screen is for clinical AI governance."
- Point to `AI Role: Assistive risk flag for discharge planning`.
- Point to `Autonomous Decision: No`.
- Point to `Human Review Required: Nurse Manager`.
- Click `Recommendation`.
- Show suggestion: "High follow-up priority due to prior admission, medication change, and missed appointment history."
- Click `Explanation`.
- Show factors.
- Click `Clinical Review`.
- Switch persona to `nurse-manager`.
- Choose decision `Accept follow-up priority`.
- Add note: "Patient has transportation barrier; route to social work."
- Click `Save`.
- Say: "The system records human action, not just model output."
- Click `EU AI Act Control Card`.
- Show purpose, oversight, logging, drift monitoring, and human override controls.
- Dollar example: "If AI helps identify 200 additional high-risk follow-ups but the system prevents unmanaged use, that is clinical and compliance value together."

11. Screen: Research Consent Boundary.
- Click `Research Coordination`.
- Select study `CARDIO-OBS-2026`.
- Search patient `CRES-PT-10429`.
- Show status `Not eligible for outreach: consent boundary not satisfied`.
- Say: "Clinical care and research are separate purposes."
- Click `Permit Explanation`.
- Show denied reason `research_coordinator lacks consent-backed outreach permit`.
- Switch persona to `attending-physician`.
- Click `View Care Context`.
- Show allowed clinical context.
- Say: "The same person can be visible for care and unavailable for research outreach."
- Click `Evidence`.
- Show consent resource and policy decision.
- Dollar example: "One consent-boundary failure can create legal, reputational, and IRB consequences that dwarf workflow savings."

12. Screen: Revenue Cycle Coordination.
- Click `Revenue Cycle`.
- Open workflow `Prior Authorization Follow-up`.
- Say: "Healthcare buyers often ask whether this applies beyond clinical ops."
- Point to payer request, missing documentation, provider attestation, patient appointment date.
- Click `Generate Task`.
- Assign to `precertification-specialist`.
- Click `Attach Clinical Evidence`.
- Show allowed documents only.
- Click `Attempt Full Chart`.
- Show denied decision.
- Say: "Revenue cycle gets the evidence it is permitted to use, not the entire chart."
- Dollar example: "If preventable authorization delays create 2 million dollars of annual cash acceleration opportunity, a narrow pilot can pay for itself quickly."

13. Screen: Compliance Pack Activator.
- Click `Compliance Pack Activator`.
- Select packs `HIPAA`, `21 CFR Part 11`, `FHIR-HL7 Integration`, `Clinical AI Governance`.
- Say: "This screen shows how controls are activated together."
- Click `HIPAA`.
- Show controls: minimum necessary, access logging, breach workflow, privacy review, retention.
- Click `21 CFR Part 11`.
- Show controls: signature meaning, signer identity, audit trail, record linkage, system checks.
- Click `FHIR-HL7 Integration`.
- Show controls: resource scope, message lineage, reconciliation, interface health.
- Click `Clinical AI Governance`.
- Show controls: human oversight, explanation, logging, monitoring, override.
- Click `Conflict View`.
- Show rule: most restrictive purpose and data-access policy wins.
- Say: "The packs compose without making compliance officers manually compare every workflow."
- Dollar example: "Four parallel control programs can become one governed workflow review."

14. Screen: Migration Mapper.
- Click `Migration Mapper`.
- Select current systems: `Epic`, `interface engine`, `ServiceNow`, `SharePoint audit folders`, `local REDCap workflows`, `custom discharge tracker`.
- Say: "We map around your EHR, not against it."
- Click `Generate Healthcare Pilot Map`.
- Show wave 1: read-only workflow and evidence design.
- Show wave 2: integration metadata and FHIR read scope.
- Show wave 3: one clinical or compliance workflow pilot.
- Show wave 4: audit review and expansion recommendation.
- Click `Discharge Pilot`.
- Show proposed duration `6 to 8 weeks`.
- Show data access: synthetic first, masked test, production metadata after approval.
- Dollar example: "A discharge follow-up pilot might target 600,000 to 1.5 million dollars in annual value depending on readmission and labor assumptions."
- Say: "The pilot must satisfy your clinical governance, privacy, security, and integration gates."

15. Screen: Cost and Throughput Lens.
- Click `Cost and Throughput`.
- Set annual discharges `68,000`.
- Set avoidable follow-up leakage `4 percent`.
- Set targeted improvement `0.6 percent`.
- Set cost per avoidable readmission `$14,500`.
- Click `Calculate`.
- Show synthetic value `Annual readmission value: $591,600`.
- Add privacy review savings `$96,000`.
- Add integration triage savings `$720,000`.
- Add duplicate workflow rationalization `$480,000`.
- Show total `First-year target value: $1,887,600`.
- Say: "This is a conservative narrow-workflow model."
- Click `Expand Scope`.
- Add Part 11 evidence and revenue-cycle authorization.
- Show expanded value `$4,800,000`.
- Say: "We will not ask you to accept demo math."
- Say: "We use your volumes in the workshop."

16. Screen: Executive Readout.
- Click `Executive Summary`.
- Select sections: integration, discharge workflow, HIPAA break-glass, Part 11 signature, AI review, economics.
- Click `Generate Readout`.
- Say: "This is the artifact for CIO, CMIO, compliance, and operations."
- Point to `EHR Coexistence`.
- Point to `Compliance Evidence`.
- Point to `Integration Boundary`.
- Point to `Pilot Economics`.
- Click `Workshop Plan`.
- Show two-week plan.
- Say: "The decision today is not EHR replacement."
- Say: "The decision is whether Oyatie should be evaluated as the governed workflow layer around selected healthcare operations."

## Objection Handling

1. Objection: "We already have an EHR."
- Response name: EHR-Coexistence.
- Say: "Oyatie is not positioned as the EHR of record."
- Say: "It coordinates governed work around the EHR where teams currently use spreadsheets, email, and disconnected tools."
- Say: "The pilot defines clear EHR read, write, and event boundaries."

2. Objection: "Our clinicians will not adopt another tool."
- Response name: Clinician-Minutes-Matter.
- Say: "The pilot must remove work from clinicians, not add clicks."
- Say: "We start with a workflow where handoffs, evidence, or follow-up are already painful."
- Say: "Clinician-facing steps should be minimal and role-specific."

3. Objection: "HIPAA makes this too risky."
- Response name: HIPAA-By-Workflow.
- Say: "That is exactly why policy and evidence must be built into the workflow."
- Say: "We can start with synthetic data and then masked or metadata-only pilots."
- Say: "Production PHI access requires your privacy and security gates."

4. Objection: "Part 11 requires validation."
- Response name: Validation-Artifact-First.
- Say: "We should not treat Part 11 as a checkbox."
- Say: "The workshop defines signature meaning, audit trail, record linkage, and validation evidence."
- Say: "A pilot should include compliance review before production use."

5. Objection: "Our interface engine already handles HL7."
- Response name: Interface-State-Into-Workflow.
- Say: "Oyatie does not need to replace the interface engine."
- Say: "It uses interface state to govern downstream operational work."
- Say: "The value is connecting integration evidence to tasks, permits, and audits."

6. Objection: "FHIR coverage is inconsistent."
- Response name: Mixed-Integration-Reality.
- Say: "That is normal."
- Say: "The demo explicitly includes both FHIR resources and HL7 messages."
- Say: "The pilot inventory identifies which integration pattern is safe for the selected workflow."

7. Objection: "We cannot use AI for clinical decisions."
- Response name: Assistive-Only-Control.
- Say: "The demo uses AI as assistive prioritization, not autonomous care."
- Say: "Human review, explanation, override, and audit are required."
- Say: "We can avoid AI in the first pilot if governance is not ready."

8. Objection: "This looks like a custom build."
- Response name: Configured-Substrate.
- Say: "The tenant, packs, permits, and workflows are configured on a shared substrate."
- Say: "Healthcare-specific content is necessary, but the operating mechanics are reusable."
- Say: "That is different from bespoke departmental software."

9. Objection: "Security will block this."
- Response name: Security-First-Pilot.
- Say: "Security should be in the first workshop."
- Say: "We can start with synthetic data, read-only integration maps, and explicit IAM boundaries."
- Say: "The pilot should not bypass your security review."

10. Objection: "Audit teams still need documents."
- Response name: Evidence-Export-Plus-Trace.
- Say: "Documents are generated from traceable source evidence."
- Say: "Auditors can inspect the underlying events, permits, signatures, and reviews."
- Say: "The export is a convenience, not the evidence source."

11. Objection: "Our workflows vary by hospital."
- Response name: Tenant-And-Site-Variation.
- Say: "Site variation can be modeled as tenant or site-specific configuration."
- Say: "The pilot should pick one service line or facility first."
- Say: "Expansion comes after we validate the pattern."

12. Objection: "We have Epic or Oracle Health workflows."
- Response name: Workflow-Boundary-Definition.
- Say: "Some workflows should stay in the EHR."
- Say: "Oyatie is strongest for cross-system, cross-role, compliance-heavy work."
- Say: "The workshop defines where the EHR remains primary."

13. Objection: "Revenue cycle and clinical operations should not mix."
- Response name: Purpose-Boundary.
- Say: "They do not have to share broad access."
- Say: "Purpose-specific permits and evidence boundaries control what each team sees."
- Say: "The demo showed denied full-chart access for revenue-cycle work."

14. Objection: "We cannot quantify patient-safety value."
- Response name: Start-With-Measurable-Proxies.
- Say: "We can start with measurable proxies: follow-up completion, review time, interface triage, and audit hours."
- Say: "Clinical outcome claims should be validated carefully."
- Say: "The first value model can be conservative."

15. Objection: "Integration will take too long."
- Response name: Read-Only-First.
- Say: "A pilot can begin with synthetic data and read-only metadata."
- Say: "Then we add approved FHIR or HL7 connectivity for the selected workflow."
- Say: "We do not need enterprise-wide integration to prove value."

16. Objection: "Compliance packs sound generic."
- Response name: Pack-Plus-Local-Policy.
- Say: "The pack provides the structured control surface."
- Say: "Your privacy, legal, and compliance teams configure local policy."
- Say: "The point is repeatability without ignoring local rules."

17. Objection: "What about downtime?"
- Response name: Healthcare-Degraded-Mode.
- Say: "The production design must include downtime and fallback behavior."
- Say: "We should model degraded-mode evidence in the pilot plan."
- Say: "Healthcare workflows cannot depend on a brittle happy path."

18. Objection: "We need proof clinicians trust this."
- Response name: Clinical-Governance-First.
- Say: "The pilot should have clinical governance from day one."
- Say: "We choose a workflow with a clear pain point and limited cognitive burden."
- Say: "Clinician feedback becomes part of the expansion decision."

19. Objection: "We are not ready for another transformation program."
- Response name: Small-Workflow-Wedge.
- Say: "Do not start with a transformation program."
- Say: "Start with one painful workflow, one compliance surface, and one integration boundary."
- Say: "A narrow pilot should prove whether broader transformation is justified."

20. Objection: "Procurement needs a clear category."
- Response name: Governed-Workflow-Substrate.
- Say: "The category is governed healthcare workflow substrate."
- Say: "It sits around selected EHR-adjacent workflows, compliance evidence, and integration state."
- Say: "We can map it to your internal procurement taxonomy during the workshop."

## Closing Call to Action

- Say: "The demo showed five healthcare patterns."
- Say: "Workflow can run around the EHR without pretending to replace it."
- Say: "FHIR and HL7 integration state can govern downstream tasks."
- Say: "HIPAA access decisions can create review obligations and evidence automatically."
- Say: "Part 11 signature meaning, audit trail, and record linkage can stay attached to the action."
- Say: "Clinical AI can be assistive, human-reviewed, and auditable."
- Say: "The recommended next step is a two-week Healthcare Workflow Evidence Workshop."
- Propose day 1: select workflow and sponsor.
- Propose day 2: EHR and integration inventory.
- Propose day 3: HIPAA access and minimum necessary mapping.
- Propose day 4: Part 11 or signature-control mapping.
- Propose day 5: clinical governance and AI boundary review.
- Propose week 2 day 1: Cedar permit and IAM mapping.
- Propose week 2 day 2: FHIR/HL7 evidence prototype.
- Propose week 2 day 3: value model with system volumes.
- Propose week 2 day 4: security, privacy, and compliance review.
- Propose week 2 day 5: executive pilot recommendation.
- Ask: "Can we schedule a workshop scoping call with CIO, CMIO, privacy, integration, and clinical operations?"
- Ask: "Which workflow should anchor the pilot?"
- Ask: "Which system owner must be in the room before we touch integration details?"
- If they hesitate, offer a 90-minute EHR-adjacent workflow mapping session.
- If they are ready, secure named participants and dates.

## Pricing Conversation Anchors

- Anchor around governed workflow value, not EHR displacement.
- Explain that healthcare pricing usually has three parts.
- Part 1: healthcare tenant platform subscription.
- Part 2: compliance-pack activation for HIPAA, Part 11, FHIR-HL7 integration controls, and clinical AI governance if used.
- Part 3: implementation services for selected workflow, integration, security, and validation support.
- Suggested workshop anchor: $75,000 to $140,000 depending on integration and validation depth.
- Suggested pilot anchor: $350,000 to $850,000 for one workflow, selected integration scope, compliance evidence, and governance review.
- Enterprise expansion depends on facilities, workflows, packs, evidence retention, and integration depth.
- Value lever: discharge follow-up completion and avoidable readmission reduction.
- Value lever: HIPAA access review labor reduction.
- Value lever: Part 11 evidence assembly reduction.
- Value lever: integration triage and reconciliation savings.
- Value lever: duplicate departmental workflow tool rationalization.
- Value lever: revenue-cycle documentation and authorization throughput.
- Dollar anchor: "If one discharge workflow produces 600,000 dollars of conservative value, it can fund a narrow pilot."
- Dollar anchor: "If integration triage savings are 720,000 dollars annually, the platform does not need heroic clinical claims to justify itself."
- Dollar anchor: "If audit preparation savings are 300,000 dollars annually across two control areas, that is measurable and low-regret."
- Keep clinical outcome claims cautious and evidence-based.
- Do not imply medical advice or autonomous diagnosis.
- Do not imply HIPAA or Part 11 compliance is automatic.
- If procurement asks for departmental licenses, translate to workflow scope and tenant capability.
- If finance asks for ROI, offer a value model using discharge volume, review volume, interface triage hours, and tool spend.
- If security asks about data, reiterate synthetic-first and approved integration boundaries.
- Do not offer production PHI usage during demo or workshop without formal security process.

## Follow-up Email Template

Subject: Follow-up from Oyatie healthcare workflow demo

Hi {{first_name}},

Thank you for the discussion today. I heard five priorities:

1. Reduce operational work that sits outside the EHR.
2. FHIR and HL7 integration state to governed workflows.
3. Improve HIPAA access review and minimum necessary evidence.
4. Support Part 11-style electronic signature evidence where required.
5. Evaluate clinical AI only with human review, traceability, and clear governance.

The demo used the `tenant-crescent-health-academic-medical-center` fixture and showed:

- A discharge care-gap workflow using synthetic FHIR and HL7 context.
- FHIR and HL7 reconciliation driving operational tasks.
- A HIPAA break-glass event with minimum necessary review.
- A Part 11 signature event with audit replay.
- A clinical AI suggestion routed through human review.
- A conservative value model for workflow, integration, privacy review, and duplicate-tool reduction.

Recommended next step: a two-week Healthcare Workflow Evidence Workshop.

Proposed outputs:

- Pilot workflow selection.
- EHR and integration inventory.
- HIPAA access and purpose map.
- Part 11 or signature-control map if in scope.
- FHIR/HL7 evidence prototype.
- Cedar permit and IAM mapping.
- Security and privacy review notes.
- First-year value model.
- Executive pilot recommendation.

Suggested attendees:

- CIO or digital sponsor.
- CMIO or clinical governance lead.
- Privacy officer.
- Compliance or validation lead.
- Integration architect.
- Clinical operations owner.
- Security and IAM.
- Revenue cycle or research lead if in scope.

Could we reserve 60 minutes next week to choose the pilot workflow and required participants?

Best,

{{sender_name}}

## References

- Internal: `registry/sample-tenants/tenant-crescent-health-academic-medical-center.md`.
- Internal: `docs/COMPLIANCE-MATRIX.md`.
- Internal: `specs/pack-overlay-schema.json`.
- Internal: `specs/capability-tier-schema.json`.
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/adr-archive/ADR-0243-cedar-as-universal-gate.md
- Internal: `docs/adr-archive/ADR-0244-tenant-as-universal-scoping-primitive.md
- Internal: `docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md
- Internal: `docs/adr-archive/ADR-0263-observability-emission-contract.md
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/adr-archive/ADR-0319-front-middle-back-office-information-barrier.md
- External: HHS HIPAA Privacy Rule, https://www.hhs.gov/hipaa/for-professionals/privacy/index.html.
- External: HHS HIPAA Security Rule, https://www.hhs.gov/hipaa/for-professionals/security/index.html.
- External: HHS HIPAA Breach Notification Rule, https://www.hhs.gov/hipaa/for-professionals/breach-notification/index.html.
- External: 21 CFR Part 11, https://www.ecfr.gov/current/title-21/chapter-I/subchapter-A/part-11.
- External: HL7 FHIR, https://hl7.org/fhir/.
- External: HL7 Version 2 Product Platform, https://www.hl7.org/implement/standards/product_brief.cfm?product_id=185.
- External: EU AI Act Regulation (EU) 2024/1689, https://eur-lex.europa.eu/eli/reg/2024/1689/oj.
- External: Cedar policy language, https://www.cedarpolicy.com/.
- Demo note: all patient, clinical, provider, and integration examples in this script are synthetic.
