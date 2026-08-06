---
doc_class: DemoScript
target_persona: Fortune 500 CIO, COO, CFO, ERP transformation sponsor, enterprise architecture lead
duration_minutes: 60
related_oyatie_adrs:
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0313
  - ADR-0315
  - ADR-0316
status: Published
date: 2026-05-20
owner: customer-success
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Fortune 500 ERP Replacement Demo

## Pre-Demo Checklist
- Demo objective: prove Oyatie can replace ERP gravity without creating a monolithic ERP clone.
- Prospect profile: Fortune 500 manufacturer with SAP S/4HANA, Oracle Fusion, and plant-local tools.
- Named tenant fixture to use: `tenant-helios-fortune-500-manufacturer`.
- Fixture source: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Primary persona to impersonate: Akira Watanabe, global COO.
- Secondary persona to impersonate: Aiko Brown, sustainability and supplier evidence lead.
- Tertiary persona to impersonate: Felix Tan, ombudsperson and restricted-case mediator.
- Prepare tool: Oya Demo Launcher.
- Prepare tool: Tenant Fixture Loader.
- Prepare tool: Ops Dashboard Control Center.
- Prepare tool: Capability Tier Console.
- Prepare tool: ERP Parity Mapper.
- Prepare tool: Ontology Object Explorer.
- Prepare tool: Workflow Engine Replay.
- Prepare tool: Workflow Studio.
- Prepare tool: Cedar Policy Simulator.
- Prepare tool: Audit Chain Explorer.
- Prepare tool: Compliance Pack Console.
- Prepare tool: FinOps Allocation View.
- Prepare tool: Migration Dry-Run Console.
- Prepare tool: Pack Evidence Exporter.
- Prepare tool: Executive Brief Builder.
- Load tenant fixture before the call.
- Confirm `us-central-gold-a`, `eu-west-gold-a`, `jp-east-gold-a`, `kr-central-gold-a`, and `mx-central-gold-a` cells are visible.
- Confirm active packs include GDPR, CCPA, HIPAA-adjacent manufacturing, KR-PIPA, and EU AI Act posture.
- Confirm intentionally absent packs are visible as absent, especially DORA and FedRAMP.
- Confirm sample data is marked fictional in the fixture header.
- Open screen 1 before the meeting: `Control Center > Tenants > Helios Industries`.
- Open screen 2 in a second tab: `ERP Parity Mapper > SAP S/4HANA`.
- Open screen 3 in a third tab: `Workflow Replay > Plant Incident CAPA`.
- Open screen 4 in a fourth tab: `Audit Chain Explorer > Helios > Last 24h`.
- Open screen 5 in a fifth tab: `FinOps > ERP Replacement Model`.
- Prepare migration source examples: SAP company code `HUS1`, Oracle ledger `NL-MFG`, Workday worker file `WD-47K`.
- Prepare dollar example: $18.4M annual license reduction from retiring redundant ERP add-ons.
- Prepare dollar example: $7.2M working-capital release from inventory hold automation.
- Prepare dollar example: $3.6M audit-prep reduction from evidence automation.
- Prepare dollar example: $28M migration-risk reserve avoided by staged parallel run.
- Prepare one customer-specific hypothesis: "your ERP is the system of record, but not the system of action."
- Prepare one risk boundary: "this is a replacement path, not a rip-and-replace weekend."
- Prepare one non-claim: "demo evidence is synthetic and not a certification."
- Prepare expected CTA: agree to a paid ERP replacement assessment using two business objects and one plant.
- Stop condition: prospect can name the first ERP module to test and the economic owner for the assessment.

## Opening Hook
- "Most ERP replacement projects fail because they try to replace suite gravity with another suite."
- "Oyatie takes the opposite route."
- "We preserve the business objects your company already understands: company code, supplier, plant, purchase order, quality hold, cash position, and shipment."
- "Then we move the work into tenant-scoped workflows, Cedar authorization, ontology projections, compliance packs, audit evidence, and per-capability cost reporting."
- "The practical result is not a prettier ERP screen."
- "The result is a lower-risk path to retire ERP modules one by one without forcing every subsidiary, plant, supplier, and regulator through one cutover cliff."
- "In the next hour I will show a global manufacturer replacing SAP-class module coverage through composition."
- "We will start with visibility, then map SAP module parity, then run a plant incident, then show policy denial, then show migration and economics."
- "The dollar lens is simple."
- "$18.4M of redundant ERP add-ons can be attacked first."
- "$7.2M of working capital can be released by reducing inventory holds and reconciliation delay."
- "$3.6M of annual audit preparation can move from manual evidence gathering to sealed evidence export."
- "The risk lens is equally simple."
- "No single ERP microservice is introduced."
- "No child subsidiary is flattened into headquarters."
- "No compliance pack is treated as marketing copy."
- "Everything we show is scoped to Helios Industries, a fictional demo fixture."
- "At the end, the ask is not a platform commitment."
- "The ask is a six-week assessment against two ERP objects and one plant workflow."
- "If that assessment cannot prove economic value and migration safety, you should not expand."

## Discovery Questions
- "Which ERP modules are most expensive to operate today: finance, procurement, warehouse, production planning, quality, maintenance, treasury, or analytics?"
- "Where does the current suite force business units to wait on central IT even when the local process is well understood?"
- "Which plants or subsidiaries have the highest reconciliation burden after month end?"
- "What percentage of ERP change requests are workflow or policy changes rather than true transaction-engine changes?"
- "Which data objects are trusted enough for regulatory evidence today, and which require manual reconciliation?"
- "How many source systems feed your current ERP reporting package?"
- "Where do SAP, Oracle, Workday, and plant-local systems disagree on supplier or material identity?"
- "Which controls are tested by auditors every quarter, and which controls are still spreadsheet-backed?"
- "What is the business cost of a quality hold that takes 48 hours to clear?"
- "What is the business cost of a supplier onboarding delay?"
- "Which ERP customizations would you delete if you could preserve the business outcome another way?"
- "Which subsidiaries need sovereignty because of regulation, acquisition history, or future divestiture risk?"
- "Who owns the first proof point: CIO, COO, CFO, procurement, plant operations, or audit?"
- "What would make a parallel-run assessment credible to your controller?"
- "What is the smallest ERP object set that would convince the transformation steering committee?"

## Demo Flow
1. Screen: Executive Tenant Posture.
- Click `Control Center`.
- Click `Tenants`.
- Click `Helios Industries`.
- Click `Executive ERP Replacement View`.
- Dialogue: "I am starting at tenant posture, not an app launcher, because ERP replacement fails when scope is invisible."
- Show the five active cells.
- Show active packs and absent packs.
- Show `38 plants`, `11,600 suppliers`, and `47,000 employees`.
- Click `Cost Dimensions`.
- Highlight plant, legal entity, pack, workflow, and capability tier dimensions.
- Dialogue: "Every dollar we discuss later can be tied to tenant, plant, supplier, and workflow."
- Click `Risk Summary`.
- Show three synthetic risks: supplier delay, quality hold, and plant incident backlog.
- Dollar example: "At Helios, a two-day quality hold on turbine assemblies ties up $7.2M of inventory."
- Click `Open Evidence`.
- Show audit-chain event list.
- Dialogue: "The executive view is not a BI copy of ERP. It is a governed projection of live tenant facts."
- Discovery pivot: "Does your current executive cockpit know why a metric is visible to one role and denied to another?"
- Expected reaction: CFO asks whether this can reconcile to the general ledger.
- Response: "Yes, but we start with object lineage before we touch financial posting."
- Stop cue: buyer acknowledges tenant and cell posture.

2. Screen: ERP Parity Mapper.
- Click `ERP Parity Mapper`.
- Select source benchmark `SAP S/4HANA`.
- Click module group `FI`.
- Click destination owners.
- Show `accounting`, `payments`, `finops-portal`, and `treasury`.
- Dialogue: "ADR-0315 says no monolithic ERP microservice. Financial accounting composes across ownership boundaries."
- Click module group `MM`.
- Show `marketplace`, `workflow-engine`, `connector`, and `warehouse`.
- Click module group `PP`.
- Show `production-planning`.
- Click module group `QM`.
- Show `quality-management`.
- Click module group `PM`.
- Show `plant-maintenance`.
- Click module group `TRM`.
- Show `treasury`, `payments`, and `finops-portal`.
- Dialogue: "This is replacement by module parity, not replacement by suite branding."
- Click `Coverage Gaps`.
- Show gap label: `partial-existing-plus-new-warehouse`.
- Dialogue: "Gaps are visible. We do not hide them in a services slide."
- Dollar example: "Helios spends $4.8M per year maintaining custom MM and QM extensions that are workflow and evidence problems."
- Click `Export Assessment Scope`.
- Select `purchase-order`, `quality-hold`, `supplier-master`.
- Dialogue: "These three objects form a credible first assessment."
- Stop cue: buyer names equivalent first module set.

3. Screen: Supplier Master Object Lineage.
- Click `Ontology Object Explorer`.
- Search `Supplier: EastLake Precision`.
- Click `Supplier Profile`.
- Click `Lineage`.
- Show source fields from SAP vendor master, Oracle procurement, and local plant spreadsheet.
- Dialogue: "We do not ask you to trust a magical golden record. We show how the record was assembled."
- Click `Conflicts`.
- Show payment terms mismatch.
- Show tax identifier confidence score.
- Show KR plant residency flag.
- Click `Resolve via Workflow`.
- Select `Supplier Master Reconciliation`.
- Dialogue: "This is where ERP replacement becomes operational. Data quality is a governed workflow, not a one-time migration report."
- Click approver `Procurement Lead`.
- Click approver `Controller`.
- Click approver `Regional Privacy`.
- Dollar example: "Helios estimated $2.1M annual leakage from duplicate supplier records and payment-term mismatch."
- Click `Run Dry Approval`.
- Show no mutation occurs in production.
- Click `Audit Preview`.
- Show pre-commit evidence.
- Dialogue: "Controllers get the same object trail auditors will later inspect."
- Stop cue: buyer sees lineage and reconciliation.

4. Screen: Purchase-to-Pay Workflow.
- Click `Workflow Engine`.
- Click template `Supply Chain Purchase to Pay`.
- Click `Start Demo Run`.
- Select supplier `EastLake Precision`.
- Select plant `Seoul Robotics Line`.
- Enter purchase amount `$1,250,000`.
- Click `Create Purchase Request`.
- Dialogue: "A classic ERP path becomes a workflow with explicit policy checks."
- Watch step `Budget Check`.
- Watch step `Supplier Risk`.
- Watch step `KR-PIPA Scope`.
- Watch step `Quality Prerequisite`.
- Click `Cedar Decision`.
- Show permit for procurement role.
- Click `Payment Hold`.
- Show payment cannot execute until goods receipt and inspection pass.
- Dollar example: "The system prevents a $1.25M early payment without waiting for a nightly ERP control report."
- Click `Evidence`.
- Show audit event classes for request, approval, hold, and release.
- Dialogue: "This is not an integration afterthought. Evidence is born with the transaction."
- Stop cue: procurement stakeholder understands side-effect boundary.

5. Screen: Quality Hold and CAPA.
- Click `Workflow Replay`.
- Click `Plant Incident CAPA`.
- Select incident `bearing-defect-monterrey-042`.
- Click `Replay From Intake`.
- Dialogue: "Now we move beyond finance into operations, where ERP platforms often need heavy customization."
- Click `Inspect Quality Notification`.
- Show lot, line, material, supplier, and customer packet impact.
- Click `Inventory Hold`.
- Show warehouse locations affected.
- Click `Create CAPA`.
- Assign owner `Akira Watanabe`.
- Assign witness `Seoul plant QA`.
- Click `Customer Packet`.
- Show redacted packet preview.
- Dollar example: "The hold contains $7.2M of inventory and $840K of shipment penalties if not cleared by Friday."
- Click `Release Criteria`.
- Show inspection result, supplier attestation, controller approval, and customer notice.
- Dialogue: "The win is not only faster CAPA. It is that the release decision is traceable and role-scoped."
- Click `Generate Board Summary`.
- Show short executive summary with evidence links.
- Stop cue: COO sees a concrete operational replacement path.

6. Screen: Cedar Denial as Product Value.
- Click `Switch Persona`.
- Select `Investor Relations Analyst`.
- Click `Customer Packet`.
- Click `Open Raw Quality Photos`.
- Observe denial.
- Dialogue: "I want to show a denial because ERP demos usually avoid what should not happen."
- Click `Explain Denial`.
- Show reason: role lacks quality-case scope and KR-PIPA pack purpose.
- Click `Request Access`.
- Show workflow requires case owner, legal, and plant QA.
- Dollar example: "One blocked over-share can avoid a seven-figure customer dispute or regulatory disclosure problem."
- Click `Audit Chain`.
- Show denied access event.
- Dialogue: "The denial is not a broken screen. It is a control with an audit trail."
- Discovery pivot: "How many current ERP or BI denials can your users understand without opening a ticket?"
- Expected reaction: security lead asks if policy is too complex.
- Response: "Policy complexity already exists. Oyatie makes it explicit, testable, and simulated before activation."
- Stop cue: security lead accepts denial as a demo moment.

7. Screen: Compliance Pack Overlay.
- Click `Compliance Pack Console`.
- Click `Helios Active Packs`.
- Select `GDPR`.
- Select `KR-PIPA`.
- Select `HIPAA-adjacent manufacturing`.
- Select `EU AI Act`.
- Click `Overlap View`.
- Dialogue: "Compliance packs are evaluated during request handling, not bolted on after reporting."
- Click `Quality Packet Export`.
- Show GDPR lawful basis.
- Show KR plant residency.
- Show HIPAA-adjacent medical-device customer restriction.
- Show AI Act human-review marker for predictive maintenance recommendation.
- Click `Most Restrictive Wins`.
- Show field-level redaction.
- Dollar example: "Helios spends $3.6M annually preparing audit evidence. This reduces evidence assembly, not legal accountability."
- Click `Export Evidence Pack`.
- Show pack IDs in export manifest.
- Dialogue: "Auditors get the evidence relevant to their pack, not a warehouse of unrelated raw data."
- Stop cue: compliance owner sees pack overlap behavior.

8. Screen: Migration Dry Run.
- Click `Migration Dry-Run Console`.
- Click source `SAP S/4HANA`.
- Upload sample extract `HUS1_supplier_vendor_master_sample`.
- Upload sample extract `HUS1_purchase_order_sample`.
- Upload sample extract `HUS1_quality_notification_sample`.
- Click `Map Objects`.
- Show supplier, purchase order, quality notification, material, plant, and cost center.
- Click `Dry Run`.
- Dialogue: "This is where we make migration risk measurable."
- Show accepted records.
- Show rejected records.
- Show policy warnings.
- Show reconciliation exceptions.
- Click `Rollback Envelope`.
- Show no production mutation and reversible import bundle.
- Dollar example: "For Helios, the assessment reserves $28M for migration risk; our goal is to prove which part of that reserve is unnecessary."
- Click `Parallel Run Plan`.
- Show week 1 object mapping, week 2 read-only projection, week 3 workflow dry run, week 4 controller review, week 5 parallel run, week 6 go/no-go.
- Dialogue: "The first paid step is deliberately narrow."
- Stop cue: transformation lead understands assessment path.

9. Screen: FinOps Replacement Model.
- Click `FinOps`.
- Click `ERP Replacement Model`.
- Filter by `Helios`.
- Click `Current Annual Run Rate`.
- Show `ERP license`, `custom extension maintenance`, `audit prep`, `integration support`, and `plant downtime risk`.
- Click `Target Savings`.
- Show `$18.4M license and add-on reduction`.
- Show `$7.2M working capital release`.
- Show `$3.6M audit prep reduction`.
- Show `$2.1M supplier data leakage reduction`.
- Click `Confidence`.
- Show assumptions tagged as buyer-validated or synthetic.
- Dialogue: "We are not asking you to believe a generic ROI calculator. We are showing which assumptions need your data."
- Click `Assessment Inputs`.
- Show objects needed from buyer.
- Click `Export CFO Brief`.
- Show one-page model.
- Stop cue: CFO sees controllable input list.

10. Screen: Executive Close.
- Click `Executive Brief Builder`.
- Select `ERP Replacement Assessment`.
- Include screens `Tenant Posture`, `ERP Parity`, `Supplier Lineage`, `CAPA`, `Migration`, and `FinOps`.
- Click `Generate Draft`.
- Dialogue: "The close is a decision artifact, not a slide deck detached from the demo."
- Show assessment scope.
- Show required buyer data.
- Show success criteria.
- Show risk gates.
- Show named owners.
- Show date proposal.
- Dollar example: "We will test whether the first $2M to $5M of annualized value is credible before expanding."
- Click `Send Follow-Up Draft`.
- Stop cue: buyer agrees to schedule technical and economic validation.

## Objection Handling
- Objection: "We cannot replace SAP."
- Response name: "Module-first replacement."
- Talk track: "We are not proposing a weekend SAP exit."
- Proof point: "ADR-0315 maps SAP module surfaces to owned Oyatie destinations."
- Demo anchor: "Use the ERP Parity Mapper screen."
- Commercial anchor: "Start with two objects and one plant."
- Close question: "Which module is painful enough to assess but bounded enough to parallel-run?"

- Objection: "This looks like another integration layer."
- Response name: "System of action, not middleware."
- Talk track: "Middleware moves messages. Oyatie owns workflow state, policy decisions, object lineage, and evidence."
- Proof point: "Purchase-to-pay emitted authorization and audit evidence during the transaction."
- Demo anchor: "Use the Purchase-to-Pay Workflow screen."
- Commercial anchor: "Price the first assessment on business-object proof, not connector count."
- Close question: "Which current integration creates the most reconciliation work?"

- Objection: "Our custom ERP logic is too unique."
- Response name: "Custom logic decomposition."
- Talk track: "Most custom logic falls into policy, workflow, data mapping, evidence, or true transaction semantics."
- Proof point: "Supplier reconciliation showed policy and workflow without a transaction-engine fork."
- Demo anchor: "Use Supplier Master Object Lineage."
- Commercial anchor: "Assessment classifies customizations before implementation."
- Close question: "Can we sample the top 20 customizations by support cost?"

- Objection: "We cannot risk financial posting."
- Response name: "Read-only then controlled mutation."
- Talk track: "The first path is read-only projection, dry-run workflow, controller review, then bounded write."
- Proof point: "Migration dry run had no production mutation."
- Demo anchor: "Use Migration Dry Run."
- Commercial anchor: "Go/no-go gate after week six."
- Close question: "Which posting-adjacent process can be tested without touching ledger close?"

- Objection: "Auditors will not accept synthetic evidence."
- Response name: "Synthetic demo, real evidence pattern."
- Talk track: "Demo data is synthetic. The evidence model is the pattern we test with your controls."
- Proof point: "Audit-chain event classes are emitted at action time."
- Demo anchor: "Use Audit Chain Explorer."
- Commercial anchor: "Assessment deliverable includes auditor-facing evidence package."
- Close question: "Which audit control should we map first?"

- Objection: "Policy will slow operators down."
- Response name: "Faster permitted path."
- Talk track: "A good policy gate speeds the permitted path by removing ambiguity."
- Proof point: "The denial explained itself and offered a request workflow."
- Demo anchor: "Use Cedar Denial as Product Value."
- Commercial anchor: "Measure cycle time before and after on one workflow."
- Close question: "Which approval exists today only because the system cannot prove scope?"

- Objection: "Subsidiaries need independence."
- Response name: "Sovereign child tenant."
- Talk track: "ADR-0313 preserves child tenant sovereignty and grants parent access through Cedar permits."
- Proof point: "The tenant view keeps plants and legal entities scoped."
- Demo anchor: "Use Executive Tenant Posture."
- Commercial anchor: "Assessment includes one subsidiary boundary."
- Close question: "Which subsidiary would be harmed by a flat global tenant?"

- Objection: "We are already consolidating in S/4HANA."
- Response name: "Consolidate facts, not lock-in."
- Talk track: "S/4HANA can remain a source during parallel run while Oyatie proves the next operating model."
- Proof point: "SAP extracts mapped into ontology objects without production mutation."
- Demo anchor: "Use Migration Dry Run."
- Commercial anchor: "Use consolidation program data as assessment input."
- Close question: "Which consolidation workstream would benefit from better evidence?"

- Objection: "We use Oracle and Workday too."
- Response name: "Multi-source object mapping."
- Talk track: "The object model accepts SAP, Oracle, Workday, NetSuite, and custom exports."
- Proof point: "Supplier lineage showed SAP, Oracle, and plant spreadsheet sources."
- Demo anchor: "Use Supplier Master Object Lineage."
- Commercial anchor: "Assessment includes up to three source systems."
- Close question: "Which cross-system object causes the most dispute?"

- Objection: "This will become a giant services project."
- Response name: "Assessment gate."
- Talk track: "We do not expand until object mapping, workflow, evidence, and economics are proven."
- Proof point: "The close screen names success criteria and stop gates."
- Demo anchor: "Use Executive Close."
- Commercial anchor: "Six-week paid assessment with no open-ended implementation commitment."
- Close question: "Who signs off on the go/no-go gate?"

- Objection: "We need global performance."
- Response name: "Cell-based replacement."
- Talk track: "Tenant cells keep work near operators while preserving global posture."
- Proof point: "Helios runs five active regional cells."
- Demo anchor: "Use Executive Tenant Posture."
- Commercial anchor: "Assessment measures latency on selected workflows."
- Close question: "Which regions must be included in the first test?"

- Objection: "ERP is our compliance backbone."
- Response name: "Compliance at request time."
- Talk track: "Compliance packs evaluate requests and emit evidence at action time."
- Proof point: "The pack overlay redacted fields before export."
- Demo anchor: "Use Compliance Pack Overlay."
- Commercial anchor: "Assessment maps one control family to pack evidence."
- Close question: "Which compliance pack is most painful today?"

- Objection: "We cannot retrain 40,000 employees."
- Response name: "Role projection."
- Talk track: "Operators see role-specific workflows, not a generic ERP transaction catalog."
- Proof point: "Akira, Aiko, and Felix each saw different surfaces."
- Demo anchor: "Use persona switch from screens 1 and 6."
- Commercial anchor: "Assessment includes one operator workflow and one executive workflow."
- Close question: "Which user group has the highest current training burden?"

- Objection: "AI recommendations create regulatory risk."
- Response name: "Human-reviewed intelligence."
- Talk track: "AI assist produces recommendations that route through review, policy, and evidence."
- Proof point: "Predictive maintenance carried an EU AI Act human-review marker."
- Demo anchor: "Use Compliance Pack Overlay."
- Commercial anchor: "Assessment can exclude AI if the first proof should be deterministic."
- Close question: "Should the first scope include AI or defer it?"

- Objection: "The economics are speculative."
- Response name: "Buyer-validated assumptions."
- Talk track: "Every economic line is tagged synthetic until your team validates the input."
- Proof point: "FinOps confidence separated buyer-validated from synthetic assumptions."
- Demo anchor: "Use FinOps Replacement Model."
- Commercial anchor: "Assessment deliverable includes CFO-reviewed assumptions."
- Close question: "Who owns license, working capital, and audit-prep numbers?"

- Objection: "Our procurement process is already automated."
- Response name: "Automation plus evidence."
- Talk track: "The gap is usually not automation. It is explainable authorization, object lineage, and audit evidence."
- Proof point: "Purchase-to-pay showed side-effect boundaries."
- Demo anchor: "Use Purchase-to-Pay Workflow."
- Commercial anchor: "Compare one automated process against audit evidence cost."
- Close question: "Which automated process still needs manual proof during audit?"

- Objection: "What happens during divestiture?"
- Response name: "Grant revocation instead of data migration."
- Talk track: "A sovereign child tenant can leave by revoking parent grants rather than splitting a database."
- Proof point: "ADR-0313 is referenced in the related ADR set."
- Demo anchor: "Use tenant posture and legal entity dimensions."
- Commercial anchor: "Assessment can model one divestiture-sensitive subsidiary."
- Close question: "Which entity is most likely to be acquired, sold, or carved out?"

- Objection: "We need board-ready proof."
- Response name: "Decision artifact close."
- Talk track: "The demo ends in a decision artifact with scope, risks, economics, and evidence."
- Proof point: "Executive Brief Builder generated the follow-up package."
- Demo anchor: "Use Executive Close."
- Commercial anchor: "Assessment kickoff includes board or steering committee criteria."
- Close question: "What must the steering committee see to approve phase two?"

## Closing Call to Action
- "The right next step is not a platform-wide replacement plan."
- "The right next step is a paid six-week ERP replacement assessment."
- "Scope one plant."
- "Scope two or three ERP business objects."
- "Include one read-only projection."
- "Include one workflow dry run."
- "Include one compliance evidence export."
- "Include one migration rollback envelope."
- "Include one CFO-reviewed value model."
- "Buyer provides sample extracts, current process map, license-cost baseline, audit control owner, and plant stakeholder."
- "Oyatie provides object mapping, workflow replay, Cedar policy simulation, pack evidence export, and assessment readout."
- "Success means the steering committee sees a credible path to the first $2M to $5M in annualized value."
- "Failure means we stop with a useful map and no production mutation."
- "Proposed first workshop: 90 minutes with CIO, controller, procurement, plant ops, security, and audit."
- "Proposed assessment start: within 15 business days after data-sharing approval."
- "Decision requested in call: name the first plant, name the first module, and name the executive sponsor."
- "If the buyer cannot name those three, schedule a discovery workshop instead of pushing for commercial paperwork."
- "If the buyer can name those three, ask for procurement path and security intake owner."

## Pricing Conversation Anchors
- Anchor the assessment as a risk-reduction purchase, not a license discount.
- Suggested assessment price band: $175K to $350K depending on sources, objects, and regions.
- Use $175K when scope is one source, two objects, one plant, read-only projection, and one workflow dry run.
- Use $250K when scope is two sources, three objects, one plant, policy simulation, and evidence export.
- Use $350K when scope is three sources, three objects, two regions, compliance pack overlap, and CFO model.
- Do not quote production subscription in the first 20 minutes.
- Do not promise full ERP retirement timeline during the demo.
- Tie subscription pricing to active capability tiers, tenant cells, evidence retention, and workflow volume.
- Explain that license displacement depends on module sequence.
- Explain that implementation services depend on connector quality and custom logic decomposition.
- Explain that compliance-pack activation can change infrastructure cost.
- Explain that regulated retention changes storage and audit-chain cost.
- Use the Helios synthetic run-rate as the story, not as a guarantee.
- Pricing phrase: "We price the first step so your team can stop if the evidence is weak."
- Pricing phrase: "Expansion is earned by validated objects, not by slideware."
- Pricing phrase: "We do not ask you to buy the platform before proving the migration thesis."
- Discount guardrail: do not discount the assessment below the internal cost of migration, policy, and evidence work.
- Procurement guardrail: if buyer wants a free proof, offer a two-hour architecture workshop, not data mapping.
- Enterprise anchor: first production phase often lands between $1.2M and $3.5M ARR for global regulated manufacturing, before services.
- Services anchor: production implementation services should be separately scoped by module and region.
- Value anchor: a single avoided quality over-share or faster inventory release can justify assessment cost.
- CFO anchor: every value line must be buyer-validated before board use.
- Legal anchor: demo scripts are not legal advice and do not assert certification.
- Stop condition: buyer agrees to assessment price band or asks for a scoped workshop to define it.

## Follow-up Email Template
- Subject: Helios-style ERP replacement assessment next steps
- Hi {{first_name}},
- Thank you for the time today.
- We walked through a Fortune 500 ERP replacement path using the fictional Helios Industries tenant fixture.
- The key thesis was module-first replacement rather than another monolithic ERP platform.
- The demo covered tenant posture, SAP-class parity mapping, supplier lineage, purchase-to-pay workflow, quality hold CAPA, Cedar denial, compliance pack overlays, migration dry run, and FinOps modeling.
- The strongest candidate assessment scope we discussed was:
- Plant: {{plant_or_region}}
- Source systems: {{source_systems}}
- Business objects: {{business_objects}}
- First workflow: {{workflow_name}}
- Executive owner: {{executive_owner}}
- Control owner: {{control_owner}}
- Economic owner: {{economic_owner}}
- Proposed assessment duration: six weeks.
- Proposed output: object map, dry-run import, workflow replay, Cedar policy simulation, evidence export, rollback envelope, and CFO assumption model.
- Proposed success criteria:
- Prove object lineage for the selected business objects.
- Prove the workflow can run without production mutation.
- Prove the evidence package satisfies the selected control owner.
- Prove a credible first $2M to $5M annualized value path or stop.
- Items requested from your team:
- Sample extracts for selected objects.
- Current process map.
- License and support baseline for the selected module.
- Audit-control owner availability.
- Security intake path.
- I suggest a 90-minute assessment scoping workshop with CIO, controller, plant operations, procurement, security, and audit.
- Proposed times:
- {{time_option_1}}
- {{time_option_2}}
- {{time_option_3}}
- Regards,
- {{sender_name}}

## References
- Internal: `registry/sample-tenants/helios-fortune-500-manufacturer.md`.
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/decisions/ADR-0709-general-live-apex.md`.
- Internal: `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- Internal: `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- Internal: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
- Internal: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`.
- Internal: `docs/decisions/ADR-0706-observability-live-apex.md`.
- Internal: `specs/capability-tier-schema.json`.
- Internal: `specs/pack-overlay-schema.json`.
- Internal: `registry/capability-tiers/vendor-tier-mapping.yaml`.
- SAP S/4HANA documentation: https://help.sap.com/docs/SAP_S4HANA_ON-PREMISE
- Oracle Fusion Cloud ERP documentation: https://docs.oracle.com/en/cloud/saas/erp/
- Microsoft Dynamics 365 documentation: https://learn.microsoft.com/en-us/dynamics365/
- Workday Financial Management product page: https://www.workday.com/en-us/products/financial-management.html
- NetSuite ERP product page: https://www.netsuite.com/portal/products/erp.shtml
- Stripe documentation: https://docs.stripe.com/connect
- EU GDPR Regulation 2016/679: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679
- EU AI Act Regulation 2024/1689: https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1689
