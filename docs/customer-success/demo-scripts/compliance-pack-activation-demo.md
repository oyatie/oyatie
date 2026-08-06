---
doc_class: DemoScript
target_persona: Chief compliance officer, privacy officer, data protection officer, AI governance lead, regulatory operations leader, security GRC owner
duration_minutes: 45
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

# Compliance Pack Activation Demo

## Pre-Demo Checklist

- Confirm meeting type: compliance officer demo or privacy and regulatory governance evaluation.
- Confirm prospect segment: multinational enterprise, healthcare-adjacent employer, financial services firm, AI-enabled operator, or global SaaS company.
- Confirm primary sponsor: chief compliance officer, privacy officer, DPO, AI governance lead, security GRC owner, or regulatory operations leader.
- Confirm stated pain: overlapping regulations, manual evidence collection, inconsistent policy enforcement, AI governance readiness, data residency, or multi-jurisdiction privacy workflows.
- Confirm deal hypothesis: Oyatie compliance packs as executable overlays attached to tenant capabilities and workflow runtime.
- Confirm named tenant fixture: `tenant-helios-fortune-500-manufacturer`.
- Confirm support fixture for KR PIPA examples: `tenant-seoul-edu-tech-pyo` available for reference only.
- Confirm fixture mode: synthetic data, synthetic employees, synthetic health-accommodation records, synthetic EU product-safety AI records, synthetic Korean supplier contacts.
- Confirm regulatory posture: show control mechanics and evidence support, not legal advice.
- Confirm compliance packs in scope: HIPAA, GDPR, KR PIPA, EU AI Act.
- Confirm demo duration: 45 minutes.
- Confirm agenda allocation: 5 minutes opening, 7 minutes discovery, 27 minutes demo, 6 minutes close and next step.
- Open Oya Demo Console.
- Open Compliance Pack Activator.
- Open Pack Overlay Inspector.
- Open Workflow Studio.
- Open Cedar Permit Simulator.
- Open Data Purpose Map.
- Open Consent and Legal Basis Console.
- Open AI Governance Review Queue.
- Open Evidence Portal.
- Open Audit Replay Console.
- Open Regulator Export Preview.
- Open Cost and Risk Lens.
- Load tenant fixture `tenant-helios-fortune-500-manufacturer`.
- Load persona `chief-compliance-officer`.
- Load persona `privacy-officer`.
- Load persona `data-protection-officer-eu`.
- Load persona `kr-privacy-manager`.
- Load persona `ai-governance-reviewer`.
- Load persona `occupational-health-coordinator`.
- Load persona `supplier-operations-manager`.
- Load persona `external-auditor-observer`.
- Enable compliance packs `HIPAA`, `GDPR`, `KR-PIPA`, `EU-AI-ACT`.
- Enable workflow `cross-border-service-incident-with-health-and-ai-context`.
- Enable workflow `employee-accommodation-supplier-delay`.
- Enable workflow `ai-product-safety-risk-review`.
- Prepare pack conflict scenario `EU employee health accommodation processed by US HR and Korean supplier operations`.
- Prepare objection-response card `not-legal-advice`.
- Prepare objection-response card `pack-composition`.
- Prepare objection-response card `most-restrictive-wins`.
- Prepare objection-response card `existing-grc`.
- Prepare objection-response card `regulator-evidence`.
- Prepare pricing card `compliance-pack-activation`.
- Prepare leave-behind `Compliance-Pack-Composition-Control-Map`.
- Verify browser zoom is 90 percent.
- Verify all data labels say synthetic.
- Verify HIPAA scope is presented as a demo healthcare/privacy control pattern for covered workflows, not a universal employer claim.
- Verify KR PIPA examples use synthetic Korean data subjects.
- Verify EU AI Act examples use synthetic high-risk AI workflow context.
- Verify pack activation starts in preview mode.
- Stop condition: do not provide legal interpretation, do not claim automatic compliance, and do not export real regulator filings.

## Opening Hook

- Say: "Compliance officers are being asked to govern more systems, more AI, more regions, and more evidence with the same teams."
- Say: "The hard part is not knowing that HIPAA, GDPR, KR PIPA, and the EU AI Act exist."
- Say: "The hard part is making the obligations operational at the moment work happens."
- Say: "A workflow may involve a health-related record, an EU employee, a Korean supplier contact, an AI-generated risk score, and a customer-impacting decision."
- Say: "Most organizations route that reality through separate policy documents, separate ticket queues, separate GRC records, and separate audit folders."
- Say: "By the time evidence is needed, teams reconstruct the story manually."
- Say: "Oyatie's compliance-pack model is designed to reduce that reconstruction burden."
- Say: "A pack is not just a PDF checklist."
- Say: "It is an executable overlay."
- Say: "It can constrain data fields."
- Say: "It can require purpose tags."
- Say: "It can add human review."
- Say: "It can create evidence obligations."
- Say: "It can change retention."
- Say: "It can deny a permit when the context is wrong."
- Say: "Today I will activate HIPAA, GDPR, KR PIPA, and EU AI Act overlays together in the Helios fixture."
- Say: "We will deliberately create a messy cross-border workflow."
- Say: "The workflow includes a synthetic health accommodation, an EU data subject, a Korean supplier contact, and an AI risk flag."
- Say: "Then we will watch the packs compose."
- Say: "The key concept is most restrictive wins where obligations collide."
- Say: "The goal is not to replace your lawyers or compliance program."
- Say: "The goal is to make your compliance program executable and evidenced."
- Say: "A credible first-year value case is often 1 to 4 million dollars from audit evidence labor, privacy review triage, reduced rework, and faster compliant launches."
- Say: "The outcome of this call is deciding whether to run a pack activation workshop on one real workflow."
- Pause.
- Ask: "Which pressure should I emphasize: privacy, AI governance, healthcare data, Korean data protection, or audit evidence?"
- If compliance lead picks one, start with that pack, then show composition.

## Discovery Questions

1. Which regulations create the most operational ambiguity for your teams today?
2. Where do privacy, compliance, legal, and operations currently review the same workflow separately?
3. Which workflows involve health-related data, EU personal data, Korean personal data, and AI-generated decisions?
4. How do you determine legal basis, purpose, retention, and access at runtime?
5. What evidence is hardest to assemble for an audit or regulator request?
6. How do you prove human oversight for AI-assisted workflows?
7. Where do local privacy teams need authority over global process design?
8. Which systems create the highest risk of over-collection or broad access?
9. How do you handle conflicts between jurisdictions or policies?
10. What is the current cost of privacy impact assessments, AI assessments, and audit evidence assembly?
11. Which GRC or privacy tools must remain system of record?
12. What would convince your legal team that pack activation supports their process rather than bypassing it?
13. Which workflow could safely be modeled with synthetic or masked data first?
14. Who would own pack configuration after pilot: compliance, privacy, security, legal, or platform team?
15. What is the smallest evidence packet that would make this credible?

## Demo Flow

1. Screen: Tenant and Pack Posture.
- Click `Demo Console`.
- Select `tenant-helios-fortune-500-manufacturer`.
- Click `Compliance Pack Activator`.
- Say: "We are in the Helios global manufacturing fixture."
- Point to tenant obligations: GDPR, KR PIPA, EU AI Act, supplier traceability, HIPAA-adjacent occupational health control scenario.
- Click `Available Packs`.
- Show HIPAA, GDPR, KR PIPA, EU AI Act.
- Say: "Packs begin in preview mode."
- Click `Preview Simultaneous Activation`.
- Show status `No workflow changed yet`.
- Say: "Compliance teams should see impact before activation."
- Dollar example: "If a global compliance review cycle costs 600,000 dollars in staff and counsel time, previewing impact is not a convenience. It is risk control."

2. Screen: Pack Anatomy.
- Click pack `GDPR`.
- Show controls: legal basis, purpose limitation, data minimization, data subject rights evidence, retention, transfer review.
- Click pack `KR PIPA`.
- Show controls: personal-data purpose, consent or statutory basis, transfer constraints, local manager review, retention.
- Click pack `HIPAA`.
- Show controls: minimum necessary, access logging, disclosure accounting, breach review, safeguard evidence.
- Click pack `EU AI Act`.
- Show controls: risk classification, human oversight, logging, transparency, data governance, post-market monitoring.
- Say: "Each pack has policy effects, workflow effects, evidence effects, and retention effects."
- Click `Overlay JSON`.
- Show high-level overlay fields without editing.
- Say: "The demo is not asking compliance to write code."
- Say: "But the control surface is machine-readable."
- Dollar example: "Machine-readable overlays reduce the cost of repeating the same interpretation across tools."

3. Screen: Cross-Border Workflow Setup.
- Click `Workflow Studio`.
- Open workflow `Employee Accommodation Supplier Delay`.
- Say: "This synthetic workflow is deliberately messy."
- Point to `Employee: EU data subject`.
- Point to `Health-related accommodation context`.
- Point to `Korean supplier contact`.
- Point to `AI delay-risk suggestion`.
- Point to `US operations owner`.
- Click `Run Case`.
- Select case `HELIOS-CROSSBORDER-044`.
- Show order value `$2,400,000`.
- Show potential line delay cost `$180,000`.
- Show health accommodation record needed for shift reassignment.
- Say: "The business wants to reassign work to prevent a production delay."
- Say: "Compliance wants to ensure the minimum data, purpose, local constraints, and AI review are handled."

4. Screen: Data Purpose Map.
- Click `Data Purpose Map`.
- Show data objects: employee accommodation note, EU employee ID, Korean supplier contact, AI risk score, production order, customer commitment.
- Click `Employee Accommodation`.
- Show required purpose `workforce_accommodation`.
- Click `EU Employee ID`.
- Show GDPR legal basis review required.
- Click `Korean Supplier Contact`.
- Show KR PIPA local-purpose review required.
- Click `AI Risk Score`.
- Show EU AI Act oversight required.
- Click `Production Order`.
- Show business operations purpose.
- Say: "This map tells the workflow what it is allowed to know."
- Dollar example: "Over-collecting data to solve a 180,000 dollar delay can create a far larger privacy problem."

5. Screen: Pack Conflict Preview.
- Click `Pack Overlay Inspector`.
- Click `Conflict View`.
- Show conflict: operations wants full accommodation note; HIPAA-style minimum necessary allows only accommodation status and restriction category.
- Show conflict: global manager wants Korean supplier personal phone; KR PIPA restricts to business contact channel.
- Show conflict: AI score wants automatic escalation; EU AI Act pack requires human review.
- Show conflict: GDPR retention default differs from production-order retention.
- Say: "This is the point of simultaneous activation."
- Say: "The system reveals conflicts before the workflow goes live."
- Click `Resolution Strategy`.
- Show `most restrictive field access`, `explicit purpose tag`, `human review`, `shorter retention where lawful`, `local privacy owner review`.
- Dollar example: "Finding this before launch can avoid weeks of remediation."

6. Screen: HIPAA-Style Minimum Necessary Control.
- Click `HIPAA`.
- Click `Minimum Necessary Simulation`.
- Select persona `supplier-operations-manager`.
- Attempt access `full accommodation note`.
- Click `Simulate`.
- Show decision `Denied`.
- Click access `restriction category and accommodation status`.
- Click `Simulate`.
- Show decision `Allowed with occupational-health review evidence`.
- Say: "The operations manager does not need the diagnosis or full note."
- Say: "They need the work restriction needed to staff the shift."
- Click `Evidence Obligation`.
- Show `privacy_review_required`.
- Dollar example: "Reducing unnecessary access lowers breach investigation and audit exposure."

7. Screen: GDPR Legal Basis and Data Subject Rights.
- Click `GDPR`.
- Click `Legal Basis`.
- Show current basis `employment obligation and legitimate operations review`.
- Click `Purpose Limitation`.
- Show allowed purposes `workforce_accommodation`, `production_continuity`.
- Attempt purpose `marketing_analysis`.
- Show denied.
- Click `Data Subject Rights`.
- Show evidence fields for access request, correction request, deletion restriction, legal hold.
- Say: "The pack does not make a legal decision alone."
- Say: "It enforces the configured legal basis and records the evidence."
- Dollar example: "If a data subject request costs 900 dollars in internal effort and evidence search reduces 40 percent, volume matters quickly."

8. Screen: KR PIPA Local Manager Review.
- Click `KR PIPA`.
- Select data object `Korean supplier contact`.
- Show fields: name, business email, business phone, supplier role, local transfer purpose.
- Click `Transfer Review`.
- Show requirement `KR privacy manager review for cross-border use`.
- Switch persona to `kr-privacy-manager`.
- Click `Review`.
- Approve `business email only`.
- Deny `personal mobile`.
- Add reason `Business contact channel sufficient for production delay notification`.
- Say: "Local privacy authority is visible in the workflow."
- Click `Cedar Trace`.
- Show permit with local manager review.
- Dollar example: "The business still moves, but the local data boundary is respected."

9. Screen: EU AI Act Human Oversight.
- Click `AI Governance`.
- Open item `DELAY-RISK-AI-778`.
- Show AI suggestion `Supplier delay likely; recommend shift reassignment and alternate supplier outreach`.
- Show risk classification `operational decision support with workforce impact`.
- Click `Human Oversight`.
- Switch persona to `ai-governance-reviewer`.
- Review explanation: supplier lateness, quality hold history, staffing constraint.
- Click `Approve for assistive use`.
- Add note `AI recommendation may inform escalation but cannot automatically reassign employee`.
- Say: "The pack preserves human oversight and prevents auto-execution."
- Click `Monitoring`.
- Show drift and decision log.
- Dollar example: "A 180,000 dollar delay can justify AI assistance, but not unmanaged AI authority over workforce decisions."

10. Screen: Combined Cedar Permit Simulation.
- Click `Cedar Permit Simulator`.
- Select action `resolve_supplier_delay_with_accommodation_context`.
- Select persona `supplier-operations-manager`.
- Select packs `HIPAA`, `GDPR`, `KR PIPA`, `EU AI Act`.
- Click `Simulate`.
- Show decision `Partial allow`.
- Show allowed: production order, restriction category, supplier business email, AI explanation summary.
- Show denied: diagnosis note, EU personal notes outside purpose, Korean personal mobile, automatic reassignment.
- Say: "This is the most important screen."
- Say: "The answer is not yes or no."
- Say: "The answer is context-scoped allow with explicit denials and obligations."
- Dollar example: "This lets the business protect a 2.4 million dollar order without expanding access beyond need."

11. Screen: Workflow Execution Under Packs.
- Click `Workflow Studio`.
- Click `Continue Case`.
- Show task `Review staffing impact`.
- Switch persona to `occupational-health-coordinator`.
- Confirm restriction category only.
- Click `Approve limited disclosure`.
- Show task `Contact Korean supplier via approved business email`.
- Switch persona to `supplier-operations-manager`.
- Click `Send Demo Notification`.
- Show demo notification preview.
- Show task `AI-assisted delay escalation`.
- Switch persona to `ai-governance-reviewer`.
- Complete human review.
- Say: "The workflow moves because the packs specify acceptable paths."
- Click `Blocked Actions`.
- Show four denied actions from prior simulation.
- Dollar example: "This is faster than routing the entire case through email while waiting for a general legal opinion."

12. Screen: Evidence Packet Composition.
- Click `Evidence Portal`.
- Open packet `COMPLIANCE-PACK-COMPOSE-044`.
- Show sections: HIPAA minimum necessary, GDPR legal basis, KR PIPA transfer review, EU AI Act oversight, workflow actions, Cedar trace, retained fields.
- Click `HIPAA Evidence`.
- Show limited disclosure and review owner.
- Click `GDPR Evidence`.
- Show purpose and legal basis.
- Click `KR PIPA Evidence`.
- Show local manager decision.
- Click `EU AI Act Evidence`.
- Show human oversight and AI explanation.
- Click `Workflow Evidence`.
- Show production order and business impact.
- Say: "This packet is the cross-regulatory story."
- Dollar example: "If a cross-regulatory audit takes 80 hours to assemble manually, cutting half the effort across 40 audits is 1,600 hours saved."

13. Screen: Regulator Export Preview.
- Click `Regulator Export`.
- Select export profile `Privacy and AI Internal Audit`.
- Show included evidence.
- Select export profile `EU DPO Review`.
- Show GDPR and EU AI sections emphasized.
- Select export profile `KR Privacy Review`.
- Show KR PIPA transfer and local decision emphasized.
- Select export profile `HIPAA Privacy Review`.
- Show minimum necessary and access log emphasized.
- Say: "Different reviewers need different views of the same underlying trace."
- Click `Masked Fields`.
- Show masked diagnosis, personal mobile, unrelated personal notes.
- Dollar example: "Reviewer-specific exports reduce over-disclosure during compliance work."

14. Screen: Audit Replay.
- Click `Audit Replay`.
- Select case `HELIOS-CROSSBORDER-044`.
- Click `Replay`.
- Show timeline: pack preview, conflict detection, limited disclosure, local KR review, AI human oversight, workflow completion, evidence packet.
- Say: "Audit replay is what prevents the team from reconstructing the story from Slack messages later."
- Click `Policy Version`.
- Show active pack versions.
- Click `Who Changed Pack`.
- Show pack configuration owner and approval history.
- Say: "Pack governance is itself auditable."
- Dollar example: "When a control changes, you can show when it changed and which workflow versions were affected."

15. Screen: Cost and Risk Lens.
- Click `Cost and Risk Lens`.
- Enter annual privacy reviews `3,200`.
- Enter annual AI governance reviews `900`.
- Enter annual cross-border workflow reviews `450`.
- Enter average manual evidence hours `5`.
- Enter loaded hourly cost `$145`.
- Click `Calculate`.
- Show baseline labor `$3,298,750`.
- Set expected reduction `35 percent`.
- Show target annual labor capacity `$1,154,563`.
- Add avoided rework assumption `$600,000`.
- Add duplicate GRC workflow reduction `$350,000`.
- Show total first-year target `$2,104,563`.
- Say: "These numbers are not universal."
- Say: "The workshop replaces them with your volumes."
- Click `Sensitivity`.
- Show low case `$700,000`, base `$2,100,000`, high `$4,200,000`.

16. Screen: Activation Plan.
- Click `Activation Plan`.
- Show phase 0: pack impact preview on synthetic workflow.
- Show phase 1: map one real workflow with legal and compliance.
- Show phase 2: configure pack overlays in sandbox.
- Show phase 3: run evidence review with privacy, AI governance, and audit.
- Show phase 4: controlled pilot with masked data or approved production metadata.
- Show phase 5: expand to adjacent workflows.
- Say: "The activation path is deliberately staged."
- Click `Owners`.
- Show compliance owner, privacy owner, AI governance owner, platform owner, business owner.
- Click `Exit Criteria`.
- Show pack conflict resolution, evidence packet approved, policy owner named, pilot business case.
- Dollar example: "A 100,000 dollar activation workshop should identify at least 500,000 dollars of credible annual value or a critical risk-reduction case."

17. Screen: Executive Summary.
- Click `Executive Summary`.
- Select sections: pack anatomy, conflict resolution, runtime permit, evidence packet, value model, activation plan.
- Click `Generate Readout`.
- Say: "This is the compliance leadership artifact."
- Point to `Not Legal Advice`.
- Point to `Configured Pack Controls`.
- Point to `Most Restrictive Wins`.
- Point to `Evidence From Runtime`.
- Point to `Workshop Recommendation`.
- Say: "The decision is whether one workflow deserves pack activation design."

## Objection Handling

1. Objection: "Compliance cannot be productized."
- Response name: Productized-Control-Not-Productized-Law.
- Say: "The legal interpretation remains yours."
- Say: "Oyatie productizes the control surface, workflow obligations, and evidence pattern."
- Say: "The pack is configured with your compliance owners."

2. Objection: "This sounds like legal advice."
- Response name: No-Legal-Advice.
- Say: "We are not providing legal advice."
- Say: "We show how configured obligations become runtime controls and evidence."
- Say: "Your legal and compliance teams approve the configuration."

3. Objection: "HIPAA may not apply to this exact employer scenario."
- Response name: Scope-Validation.
- Say: "Correct, scope must be validated."
- Say: "The demo uses HIPAA-style health privacy controls to show pack mechanics."
- Say: "A workshop would confirm exact applicability before activation."

4. Objection: "Regulations conflict."
- Response name: Conflict-Visible.
- Say: "Conflicts are why preview mode exists."
- Say: "The system shows field, purpose, retention, review, and permit conflicts."
- Say: "Compliance owners choose the resolution strategy."

5. Objection: "Most restrictive wins may block the business."
- Response name: Scoped-Allowed-Path.
- Say: "The system does not only block."
- Say: "It finds scoped allowed paths, limited disclosure, local review, and human oversight."
- Say: "The goal is compliant movement, not paralysis."

6. Objection: "We already have OneTrust or GRC tooling."
- Response name: GRC-Coexistence.
- Say: "Those tools can remain systems of record for assessments or records."
- Say: "Oyatie focuses on runtime workflow enforcement and evidence capture."
- Say: "We can export or synchronize where needed."

7. Objection: "Our policies change often."
- Response name: Versioned-Pack-Governance.
- Say: "Pack versions are governed."
- Say: "Policy changes show affected workflows and evidence impact."
- Say: "That is safer than undocumented spreadsheet logic."

8. Objection: "AI governance is not mature enough."
- Response name: Start-With-Human-Oversight.
- Say: "Then start with human oversight, logging, explanation, and approved use cases."
- Say: "The pack does not require broad AI rollout."
- Say: "It creates controlled conditions for use."

9. Objection: "Local privacy teams need control."
- Response name: Local-Owner-In-The-Workflow.
- Say: "The KR PIPA example showed local manager review."
- Say: "Regional owners can have explicit approval authority."
- Say: "Global workflows do not have to erase local governance."

10. Objection: "Evidence exports could over-disclose."
- Response name: Reviewer-Specific-Exports.
- Say: "Exports are profile-based and field-masked."
- Say: "The underlying trace remains governed."
- Say: "Reviewers see what their role permits."

11. Objection: "This will slow product launches."
- Response name: Faster-Approved-Path.
- Say: "Manual compliance review already slows launches."
- Say: "Executable packs can make the approved path clearer and faster."
- Say: "The pilot should measure cycle time."

12. Objection: "We need retention by jurisdiction."
- Response name: Retention-As-Control.
- Say: "Retention is part of the overlay."
- Say: "Conflicts are previewed and resolved by policy owners."
- Say: "Evidence shows which retention rule applied."

13. Objection: "Who maintains pack content?"
- Response name: Joint-Pack-Ownership.
- Say: "Compliance owns interpretation."
- Say: "Platform owns implementation."
- Say: "Business owns workflow intent."
- Say: "Changes are versioned and reviewed."

14. Objection: "We cannot expose sensitive data in a pilot."
- Response name: Synthetic-And-Masked-First.
- Say: "The pilot can start with synthetic data and masked metadata."
- Say: "Production use requires approved data boundaries."
- Say: "The control model can be validated without broad access."

15. Objection: "Our auditors require source evidence."
- Response name: Source-Linked-Evidence.
- Say: "Every packet item links to source events, permits, reviews, and pack versions."
- Say: "The export is not the source of truth."
- Say: "Audit replay exposes the path."

16. Objection: "This seems hard to configure."
- Response name: Workshop-Scope.
- Say: "That is why we begin with one workflow and four packs."
- Say: "We configure only the controls needed for that scope."
- Say: "Expansion follows proven patterns."

17. Objection: "Business teams will dislike denials."
- Response name: Explainable-Denial.
- Say: "Denied actions include reasons and allowed alternatives."
- Say: "People accept controls better when the path forward is visible."
- Say: "The demo showed partial allow, not blind rejection."

18. Objection: "What if a pack is wrong?"
- Response name: Version-And-Review.
- Say: "Pack configuration is reviewed and versioned."
- Say: "Impacted workflows can be identified."
- Say: "Rollback or correction becomes auditable."

19. Objection: "Can this cover more regulations?"
- Response name: Pack-Expansion.
- Say: "The model is designed for additional overlays."
- Say: "We start with the most important packs for the pilot."
- Say: "Do not activate everything before you validate one workflow."

20. Objection: "Procurement will ask what category this is."
- Response name: Executable-Compliance-Overlay.
- Say: "The category is executable compliance overlay for governed workflow."
- Say: "It complements privacy, GRC, IAM, and workflow systems."
- Say: "The pilot defines the integration posture."

## Closing Call to Action

- Say: "The demo showed simultaneous compliance-pack activation."
- Say: "HIPAA-style minimum necessary limited health-context access."
- Say: "GDPR constrained purpose, legal basis, and data subject evidence."
- Say: "KR PIPA required local review and limited Korean contact data."
- Say: "EU AI Act governance required human oversight for AI-assisted workflow action."
- Say: "The Cedar decision produced partial allow, explicit denials, and obligations."
- Say: "The evidence packet assembled the cross-regulatory story."
- Say: "The recommended next step is a Compliance Pack Activation Workshop."
- Propose day 1: select one real workflow and data objects.
- Propose day 2: map regulatory applicability with legal and compliance.
- Propose day 3: pack control preview and conflict analysis.
- Propose day 4: Cedar permit and data-purpose model.
- Propose day 5: evidence packet design.
- Propose week 2 day 1: sandbox workflow configuration.
- Propose week 2 day 2: reviewer export profiles.
- Propose week 2 day 3: audit replay and pack-version review.
- Propose week 2 day 4: value and risk model.
- Propose week 2 day 5: pilot recommendation.
- Ask: "Which workflow has the clearest overlapping regulatory burden?"
- Ask: "Who owns legal interpretation for each pack?"
- Ask: "Can we schedule the workflow selection and pack-scope call this week?"
- If they hesitate, offer a 90-minute pack conflict preview using synthetic data.
- If they agree, capture workflow, packs, data types, owners, and success criteria.

## Pricing Conversation Anchors

- Anchor around compliance operating leverage and evidence quality.
- Explain that pack activation pricing has four parts.
- Part 1: base tenant platform subscription.
- Part 2: compliance-pack activation for selected obligations.
- Part 3: evidence retention and export volume.
- Part 4: advisory and implementation services for workflow-specific configuration.
- Suggested pack activation workshop anchor: $75,000 to $150,000.
- Suggested first workflow pilot anchor: $250,000 to $650,000 depending on pack count, integration scope, evidence review, and data-boundary complexity.
- Enterprise expansion depends on number of workflows, jurisdictions, pack overlays, retention volume, and regulator export needs.
- Value lever: privacy review hours reduced.
- Value lever: AI governance review hours reduced.
- Value lever: audit evidence assembly reduced.
- Value lever: launch rework avoided.
- Value lever: duplicate control workflow rationalization.
- Value lever: reduced over-disclosure risk.
- Dollar anchor: "The demo base case showed 2.1 million dollars of first-year target value."
- Dollar anchor: "Even the low case of 700,000 dollars can justify a tightly scoped pilot."
- Dollar anchor: "A single avoided privacy remediation can exceed the workshop cost."
- Do not sell packs as legal guarantees.
- Do not offer unlimited regulation coverage without scoping.
- If buyer asks for a fixed enterprise number, request workflow count, pack count, jurisdiction count, evidence retention, and export volume.
- If legal asks for accountability, state that customer legal approves configuration.
- If procurement asks category, use `executable compliance overlay and evidence automation`.
- If GRC owner objects, position as runtime evidence source rather than replacement by default.

## Follow-up Email Template

Subject: Follow-up from Oyatie compliance-pack activation demo

Hi {{first_name}},

Thank you for the discussion today. I heard the main challenge as turning compliance obligations into operational controls without slowing the business or reconstructing evidence after the fact.

The demo used the `tenant-helios-fortune-500-manufacturer` fixture and showed:

- Simultaneous activation preview for HIPAA, GDPR, KR PIPA, and EU AI Act overlays.
- A cross-border workflow involving synthetic health context, EU personal data, Korean supplier contact data, and an AI risk suggestion.
- Pack conflict detection and most-restrictive resolution.
- HIPAA-style minimum necessary field controls.
- GDPR purpose and legal-basis evidence.
- KR PIPA local review for Korean personal data.
- EU AI Act human oversight for AI-assisted action.
- A Cedar permit result with partial allow, explicit denials, and review obligations.
- A cross-regulatory evidence packet and reviewer-specific export profiles.

Recommended next step: a two-week Compliance Pack Activation Workshop.

Proposed outputs:

- Workflow and data-object selection.
- Regulatory applicability map.
- Pack conflict preview.
- Permit and purpose model.
- Evidence packet design.
- Reviewer export profiles.
- Value and risk model.
- Pilot recommendation.

Suggested attendees:

- Chief compliance officer or delegate.
- Privacy officer or DPO.
- Local privacy owner for in-scope jurisdiction.
- AI governance lead if AI is in scope.
- Business workflow owner.
- Security or IAM lead.
- Platform or architecture owner.
- Legal counsel as appropriate.

Could we reserve 60 minutes next week to pick the workflow and confirm the pack scope?

Best,

{{sender_name}}

## References

- Internal: `registry/sample-tenants/tenant-helios-fortune-500-manufacturer.md`.
- Internal: `registry/sample-tenants/tenant-seoul-edu-tech-pyo.md`.
- Internal: `docs/COMPLIANCE-MATRIX.md`.
- Internal: `specs/pack-overlay-schema.json`.
- Internal: `specs/capability-tier-schema.json`.
- Internal: `docs/decisions/ADR-0243-*`.
- Internal: `docs/decisions/ADR-0244-*`.
- Internal: `docs/decisions/ADR-0251-*`.
- Internal: `docs/decisions/ADR-0263-*`.
- Internal: `docs/decisions/ADR-0304-*`.
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/decisions/ADR-0319-*`.
- External: HHS HIPAA Privacy Rule, https://www.hhs.gov/hipaa/for-professionals/privacy/index.html.
- External: HHS HIPAA Security Rule, https://www.hhs.gov/hipaa/for-professionals/security/index.html.
- External: GDPR Regulation (EU) 2016/679, https://eur-lex.europa.eu/eli/reg/2016/679/oj.
- External: EU AI Act Regulation (EU) 2024/1689, https://eur-lex.europa.eu/eli/reg/2024/1689/oj.
- External: Personal Information Protection Act, Korea Law Translation Center, https://elaw.klri.re.kr/eng_service/lawView.do?hseq=53044&lang=ENG.
- External: Personal Information Protection Commission, Republic of Korea, https://www.pipc.go.kr/.
- External: Cedar policy language, https://www.cedarpolicy.com/.
- Demo note: all health, employee, supplier, AI, and cross-border examples in this script are synthetic.
