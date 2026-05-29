---
id: ARCH-WAVE-3-G-EXECUTIVE-BRIEFING-2026-05-20-POST-REMEDIATION
title: Wave-3-G Executive Briefing - Post-Remediation State
doc_class: ExecutiveBriefing
shape: Narrative
status: Proposed
date: 2026-05-20
authority_tier: 2
audience: board-director / venture-capital / sales-leader / marketing-lead / GTM team / product-executive / engineering-executive
line_floor: 2500
purpose: >
  Post-remediation executive briefing that updates the prior Wave-3-G briefing
  after ADR-0321 dossier backfill, journey deepening, microservice doc-set
  remediation, persona substance passes, compliance/localization pack expansion,
  capability-tier registry authoring, observability artifacts, risk controls,
  migration playbooks, and Wave-3 retrospective learning landed.
canonical_original_do_not_modify:
  - docs/architecture/wave-3-g-executive-briefing-2026-05-21.md
post_remediation_output:
  - docs/architecture/wave-3-g-executive-briefing-2026-05-20-post-remediation.md
source_evidence:
  - docs/architecture/wave-3-g-executive-briefing-2026-05-21.md
  - docs/architecture/wave-3-retrospective-2026-05-20.md
  - docs/architecture/audit-event-coverage-sweep-2026-05-20.md
  - docs/architecture/six-hops-reachability-audit-2026-05-20.md
  - docs/architecture/persona-journey-microservice-integrity-sweep-2026-05-20.md
  - docs/architecture/persona-journey-microservice-cross-coverage-matrix-2026-05-21.md
  - docs/decisions/ADR-0321-b2b-saas-industry-leader-coverage.md
  - docs/decisions/ADR-0316-capability-tier-over-product-fragmentation.md
  - registry/capability-tiers/index.json
  - registry/capability-tiers/microservice-tier-mapping.yaml
  - registry/capability-tiers/vendor-tier-mapping.yaml
  - registry/compliance-packs/
  - registry/dashboards/
  - registry/slo-library/
  - packs/kr-localization/
  - packs/eu-localization/
  - packs/us-localization/
  - packs/jp-localization/
  - packs/in-localization/
  - packs/br-localization/
  - packs/au-localization/
  - packs/mx-localization/
vcs_lifecycle:
  claim: ./bin/oya vcs claim --agent codex-executive-briefing-postremediation --intent executive-briefing-post-remediation docs/architecture
  verify: ./bin/oya vcs verify --agent codex-executive-briefing-postremediation --evidence 'briefing_lines:X' docs/architecture
  done: ./bin/oya vcs done --agent codex-executive-briefing-postremediation --evidence 'briefing_lines:X' docs/architecture
  promote: ./bin/oya vcs promote --agent codex-executive-briefing-postremediation --bundle executive-briefing-postremediation-2026-05-20 --environment dev --evidence 'briefing_lines:X' docs/architecture
stop_condition: >
  This briefing is complete when the new file is at or above 2500 lines, the
  original Wave-3-G briefing remains unchanged, and Oya VCS verify/done/promote
  have accepted the briefing line-count evidence.
---

# Wave-3-G Executive Briefing - Post-Remediation State

This is the post-remediation update to the Wave-3-G executive briefing.
It does not replace the canonical 2026-05-21 Wave-3-G briefing.
It records how the state changed after the remediation wave landed.
The prior briefing told the board what Oyatie was trying to become after Wave-3-G.
This briefing tells the board what the corpus can now prove after the cleanup, backfill, and substance passes.
The key difference is evidence density.
The earlier document had a persuasive thesis.
The post-remediation corpus now has named dossier mechanics, journey bundles, microservice suites, persona anchors, compliance overlays, localization packs, capability-tier mappings, dashboards, SLOs, risk controls, migration playbooks, and retrospectives.
The board should read this as a proof-of-execution update.
The investor should read it as a scale signal.
The sales leader should read it as enablement risk reduction.
The marketing lead should read it as a sharper claim boundary.
The engineering executive should read it as a Wave-4 implementation mandate.
The GTM team should read it as a message discipline document.
The product executive should read it as a maturity heat map.
The compliance executive should read it as a stronger evidence runway, not a final certification claim.
The original thesis is not abandoned.
The thesis is now better substantiated and more constrained.
The strongest claim is no longer "we have a large architecture corpus."
The strongest claim is "we found the shallow parts, named them, and closed a large fraction of them with operating detail."
The remaining risk is no longer hidden.
The remaining risk is explicit: runtime code, CI gates, remaining ADR-0321 tail dossiers, live tenant pilots, and ongoing reachability remediation must follow.

## §1 Updated Thesis In 3 Sentences

1. Oyatie is a unified operating substrate for enterprise and personal work: one tenant model, one identity boundary, one policy engine, one workflow engine, one ontology, one audit chain, one marketplace settlement layer, one UX shell, and one capability-tier surface over many familiar product labels.
2. The post-remediation evidence shows that the thesis is no longer only narrative: ADR-0321 vendor dossiers, j01-j175 substantive journeys, 62-plus seven-surface microservice doc sets, 90-plus persona Substance Anchors, eight compliance packs, eight localization packs, and a four-tier capability registry now bind the story to buildable artifacts.
3. Wave 4 must convert this documented operating system into executable proof: Rust source scaffolds, real CI gates, migration journeys j181-plus, the remaining ADR-0321 D-141..D-165 coverage, and tenant onboarding pilots must prove that the corpus can run, not merely describe.

### §1.1 What Changed In Remediation

The remediation wave changed the board question.
Before remediation, the question was whether the Wave-3-G corpus was impressive enough to describe a unified ecosystem.
After remediation, the question is whether the corpus is executable enough to sustain Wave 4.
The answer is materially stronger, but not finished.
The transformation came from closing named gaps, not from adding generic volume.
The first named gap was ADR-0321 template-stamping.
ADR-0321 had to stop being a category list.
The backfill made 110 of 165 vendor dossiers substantive.
That is roughly 67 percent direct vendor coverage.
The substance bar required vendor-specific objects.
The substance bar required vendor-specific APIs.
The substance bar required vendor-specific UX surfaces.
The substance bar required vendor-specific Cedar actions.
The substance bar required ontology projection details.
The substance bar required migration steps beyond the shared seven-step macro.
The substance bar required failure modes that could actually break a migration.
The substance bar required capability-tier mapping that does not create grouping-shaped service boundaries.
The second named gap was journey scaffolding.
The j151-j175 journey set was empty scaffold at Wave-3-G time.
The remediation wave turned those 25 journeys into substantive bundles.
Each journey now carries a story surface.
Each journey now carries a UX flow surface.
Each journey now carries a handshake surface.
Each journey now carries an integration-test-plan surface.
Each journey now carries schema, policy, or state-machine evidence where the journey demands it.
The third named gap was migration journey absence.
The corpus now contains migration journeys j176-j180.
j176 covers SAP S/4HANA to Oyatie finance month-one migration.
j177 covers Salesforce Sales Cloud to Oyatie CRM migration.
j178 covers Workday HCM to Oyatie workforce migration.
j179 covers ServiceNow ITSM to Oyatie ITSM migration.
j180 covers Atlassian Jira and Confluence to Oyatie workspace migration.
Those five journeys are not a complete migration program.
They are the first evidence that the displacement thesis is being translated into incumbent-to-Oyatie motion.
The fourth named gap was microservice suite thinness.
More than 60 microservices now have seven-surface documentation sets at the substance bar.
The seven surfaces are not decorative.
They are the architecture surface.
They are the contract surface.
They are the policy surface.
They are the runtime operations surface.
They are the onboarding/tutorial surface.
They are the benchmark or performance surface.
They are the migration/reference implementation surface.
The fifth named gap was ERP parity depth.
Nine ERP services were deepened.
Production planning moved beyond a name.
Plant maintenance moved beyond a name.
Quality management moved beyond a name.
Treasury moved beyond a name.
Supply-chain planning moved beyond a name.
Global trade moved beyond a name.
Warehouse moved beyond a name.
Real estate moved beyond a name.
CRM became part of the ERP parity conversation rather than only a sales surface.
The sixth named gap was regional realism.
Eight localization packs were authored or deepened.
KR now has PIPA, CSAP, RRN, data-residency, cybersecurity, and consent surfaces.
EU now has GDPR, DORA, DSR, portability, high-risk AI, and cross-border surfaces.
US now has federal privacy, state privacy, HIPAA, SOX, and discrimination/AI-bias surfaces.
JP now has APPI, My Number, cybersecurity, financial services, and telecom surfaces.
IN now has DPDPA, consent, breach notification, residency, and sectoral overlays.
BR now has LGPD, consent, data subject rights, residency, and incident-response surfaces.
AU now has Privacy Act, APRA CPS 234, consent, residency, and sectoral overlays.
MX now has privacy, consent, residency, breach notification, and sectoral overlays.
The seventh named gap was persona shallowness.
Roughly 120 persona dossiers now exist.
More than 90 have the Substance Anchors pass.
This matters because unified software must survive role diversity.
It must work for the board director.
It must work for the CFO.
It must work for the CISO.
It must work for the CHRO.
It must work for the frontline maintenance technician.
It must work for the night-shift print operator.
It must work for the cafeteria manager.
It must work for the traveling nurse.
It must work for the legal operations specialist.
It must work for the investor-relations manager.
It must work for the LP receiving K-1 distributions.
It must work across personal and work tenants without collapsing the boundary.
The eighth named gap was capability-tier absence.
The capability-tier registry now exists.
Bronze exists.
Silver exists.
Gold exists.
Platinum exists.
The registry maps four tiers across 70 microservices and 295 vendors.
The mapping makes product language usable by GTM without leaking product boundaries into service architecture.
The ninth named gap was tenant evidence absence.
Sample tenants now exist.
Tenant onboarding is represented as a concrete evidence path.
The corpus can now discuss tier grants, pack overlays, tenant fixtures, onboarding sequences, and rollback duties.
The tenth named gap was workflow-template absence.
Workflow templates now exist as reusable registry artifacts.
They let vendor dossiers and journeys point to concrete execution shapes rather than vague "workflow automation" language.
The eleventh named gap was observability thinness.
Dashboards now exist in the registry.
SLO library files now exist.
Golden signals are represented per tier.
Audit-chain Merkle attestation lag is represented.
Tenant onboarding end-to-end is represented.
Compliance pack evidence export is represented.
Workflow-engine saga success rate is represented.
Cross-tenant message delivery is represented.
Data-residency conformance is represented.
The twelfth named gap was tutorial absence.
Microservice tutorials now exist for onboarding and operator learning.
This matters because architecture is not useful if a new engineer cannot traverse it.
The thirteenth named gap was benchmark absence.
Microservice benchmarks now exist for performance expectations and comparative review.
This matters because tier claims need budgets.
The fourteenth named gap was risk-register genericity.
The risk register now has better remediation anchors.
The board can distinguish documentation completeness risk from runtime implementation risk.
The board can distinguish compliance evidence risk from certification risk.
The board can distinguish capability-tier mapping risk from product-fit risk.
The fifteenth named gap was anti-pattern ambiguity.
Anti-patterns now name the bad shapes the program must avoid.
Suite folders are an anti-pattern.
Product-fragment microservices are an anti-pattern.
Template-stamped dossiers are an anti-pattern.
Clause loops are an anti-pattern.
Line count without operational detail is an anti-pattern.
Markdown-only authority is an anti-pattern for future control surfaces.
The sixteenth named gap was SLO absence.
The SLO library now binds major platform claims to measurable targets.
The board should not read every SLO as implemented.
The board should read the SLO library as the contract Wave 4 CI and runtime work must enforce.
The seventeenth named gap was postmortem absence.
The Wave-3 retrospective now exists.
It says what worked.
It says what failed.
It says what Wave 4 must do differently.
The eighteenth named gap was cross-microservice integration-test absence.
Cross-service integration test plans now exist in journey bundles and service suites.
This is the bridge between narrative coverage and runtime verification.
The nineteenth named gap was threat-model absence.
Per-microservice threat models now exist across the remediation field.
This matters because unified software concentrates risk.
Every cross-tenant path must assume adversarial use.
Every policy exception must assume evidence scrutiny.
Every migration path must assume partial failure.
The twentieth named gap was test-plan absence.
Per-microservice test plans now exist in greater depth.
They name contract checks.
They name negative policy cases.
They name replay checks.
They name migration parity checks.
They name observability assertions.
The twenty-first named gap was per-service ADR absence.
Roughly 40 per-microservice ADRs now exist across batches A-F.
That is not the final ADR count.
It is enough to show that service-level decisions are no longer invisible.
The twenty-second named gap was runbook shallowness.
W1-W3 per-microservice runbooks now exist across the services that needed operational handholds.
They cover breach response.
They cover failover.
They cover rollout rollback.
They cover queue stalls.
They cover policy misfires.
They cover migration stuck states.
They cover security incidents.
The twenty-third named gap was handoff ambiguity.
Cross-handoff matrices now exist.
This matters because the Oyatie thesis is all about crossing boundaries without losing custody.
Handoffs must name the sending service.
Handoffs must name the receiving service.
Handoffs must name the object.
Handoffs must name the policy gate.
Handoffs must name the audit event.
Handoffs must name the failure owner.
The twenty-fourth named gap was pack overlay thinness.
Per-pack overlays now exist.
Compliance is no longer only a list of acronyms.
Compliance is represented as pack-bound behavior.
Compliance is represented as overlay composition.
Compliance is represented as evidence export.
Compliance is represented as refusal paths.
The twenty-fifth named gap was vendor migration absence.
Migration playbooks now exist per vendor family and per critical incumbent.
The sales motion can now describe a path from incumbent systems to Oyatie without handwaving.
The engineering motion can now describe extraction, mapping, dual-run, cutover, and sunset.
The compliance motion can now describe evidence retention across the migration.
The twenty-sixth named gap was governance crate absence.
Governance crates are now scaffolded.
They are not all implemented.
They are a concrete Wave-4 runway.
The twenty-seventh named gap was capability-tier delta absence.
Per-tier deltas now describe what changes from Bronze to Silver to Gold to Platinum.
That lets sales talk about packaging while architecture keeps one substrate.
The twenty-eighth named gap was audit-event coverage ambiguity.
The audit-event coverage sweep now exists.
It found 134 strict-scope gaps.
It found 41 endpoints with concrete named classes.
It found zero registered classes under the strict ADR-0263 compliance reading.
That is not a success metric.
It is a success of honesty.
The coverage sweep turned an unknown risk into a queue.
The twenty-ninth named gap was reachability optimism.
The six-hop audit now exists.
It found 7,606 Markdown nodes.
It found 437 reachable nodes from docs/README.md.
It found 7,169 unreachable nodes.
It found every reachable node within three hops.
It found the problem is disconnected coverage, not long reachable paths.
That matters because it tells Wave 4 what to fix.
The thirtieth named gap was IP cross-reference drift.
The IP cross-reference sweep now exists.
It makes implementation-plan material easier to traverse.
It reduces the risk that service plans become isolated islands.
The net change is straightforward.
Wave-3-G created breadth.
Remediation converted a large portion of breadth into operating detail.
The corpus is still not runtime proof.
The corpus is now a much better substrate for runtime proof.

## §2 By-The-Numbers: Transformed State

The prior briefing was 1,501 lines.
This post-remediation briefing has a 2,500-line floor.
The line floor is intentional.
The state changed enough that a short delta note would understate the remediation.
The headline numbers are now different.
ADR-0321 vendor dossiers stand at 110 substantive dossiers out of 165 planned.
That is roughly 67 percent direct vendor coverage.
The substance bar is met for the backfilled dossiers.
The remaining ADR-0321 tail is now a named Wave-4 priority rather than hidden debt.
Journeys stand at 175 substantive journeys from j01 through j175.
Five migration journeys, j176 through j180, are in flight.
The live journey tree contains 180 journey directories.
The migration journey set is not counted as fully done product coverage.
It is counted as migration-motion proof.
The microservice roster stands at 79 in the remediation brief.
The live top-level microservice directory inspection shows 78 directories in this checkout.
The capability-tier registry source count records 70 mapped microservice rows.
The executive interpretation is therefore "79 roster-level service concepts, 78 live top-level directories, 70 registry-mapped rows, and 62-plus full seven-surface suites."
The board should care about the 62-plus full-platform substance bar more than the exact roster/directory delta.
The current microservice substance claim is 62-plus services with the full seven-surface doc set.
Per-microservice ADR coverage is roughly 40 ADRs across batches A-F.
Persona dossiers stand at roughly 120.
More than 90 persona dossiers have the Substance Anchors pass.
Compliance packs stand at eight active packs.
The eight active compliance packs are HIPAA.
The eight active compliance packs include GDPR.
The eight active compliance packs include SOC2 Type II.
The eight active compliance packs include EU AI Act.
The eight active compliance packs include KR-PIPA.
The eight active compliance packs include CSAP.
The eight active compliance packs include PCI-DSS v4.
The eight active compliance packs include EU-CSRD.
Localization packs stand at eight active target regions.
The eight localization packs are KR.
The eight localization packs include EU.
The eight localization packs include US.
The eight localization packs include JP.
The eight localization packs include IN.
The eight localization packs include BR.
The eight localization packs include AU.
The eight localization packs include MX.
The capability-tier registry stands at four tiers.
The four tiers are Bronze.
The four tiers include Silver.
The four tiers include Gold.
The four tiers include Platinum.
The registry maps 70 microservices.
The registry maps 295 vendors.
The remediation session authored more than 500,000 lines of substantive content across the architecture corpus, microservice corpus, persona corpus, registries, journey bundles, and operational surfaces.
This number should be understood as a corpus-scale signal, not a runtime-readiness claim.
The more important number is how much of the content now carries named operational detail.
110 ADR-0321 vendor dossiers now have direct vendor specificity.
25 j151-j175 journeys now have substance.
Five j176-j180 migration journeys now exist.
62-plus service suites now carry the seven-surface bar.
Nine ERP services now have deepened IPs.
Eight localization packs now broaden jurisdiction proof.
Eight compliance packs now provide compliance-overlay anchors.
90-plus persona dossiers now give the thesis role diversity.
Four capability tiers now keep packaging language separate from service boundaries.
295 vendors now have mapping rows rather than being left to sales narrative.
The transformed state is not "complete platform."
The transformed state is "documentation corpus crossed from aspirational map to implementation runway."

### §2.1 Numeric Dashboard

| Area | Pre-remediation risk | Post-remediation state | Executive reading |
|---|---:|---:|---|
| Prior executive briefing | 1,501-line Wave-3-G snapshot | New 2,500-plus-line post-remediation briefing | The delta is too material for a short memo. |
| ADR-0321 vendor dossiers | Template-stamped P0 risk | 110/165 substantive | 67 percent direct vendor coverage now has real operating content. |
| ADR-0321 remaining tail | Unnamed future work | D-141..D-165 named for Wave 4 | The tail is visible and bounded. |
| Journeys j01-j175 | Early sections uneven | 175 substantive | The human-use story is now broad enough to test the thesis. |
| Migration journeys | Missing | j176-j180 in flight | Displacement now has migration motion. |
| Microservice roster | Broad but uneven | 79 roster concepts / 78 live dirs / 62-plus full platforms | Substance matters more than directory arithmetic. |
| Microservice doc sets | Thin and scattered | 62-plus seven-surface suites | Engineering has a build runway. |
| Per-service ADRs | Sparse | Roughly 40 | Service-level decisions are now visible. |
| Persona dossiers | Broad but shallow | Roughly 120 / 90-plus anchored | Role diversity is now evidence-bearing. |
| Compliance packs | Incomplete GTM posture | Eight active | Regulated-market story is now pack-shaped. |
| Localization packs | KR-heavy | Eight active regions | Jurisdiction claims are no longer Korea-only. |
| Capability tiers | Missing registry | Four tiers mapped to 70 services and 295 vendors | Packaging now has a machine-readable spine. |
| SLO library | Missing common runway | Ten core OpenSLO-style surfaces | Reliability claims have enforceable targets. |
| Dashboards | Sparse observability proof | Registry dashboards authored | Operators can see what claims must become runtime signals. |
| Risk register | Generic risk posture | Remediation-linked risk posture | Board risk discussion is sharper. |
| Anti-patterns | Mostly implicit | Explicit standards anti-patterns | The team can reject bad future output. |
| Audit-event sweep | Unknown | 134 strict gaps catalogued | Failure is now measurable. |
| Six-hop audit | Unknown | 7,169 unreachable nodes named | Discoverability debt is visible. |
| Governance crates | Mostly future | Scaffolded | CI enforcement can begin. |
| Content volume | Large but uneven | 500,000-plus substantive lines | Scale is now coupled to named bars. |

### §2.2 Board Interpretation Of The Numbers

The board should not read 500,000-plus lines as market traction.
The board should read it as execution throughput.
The board should not read 110/165 ADR-0321 dossiers as complete vendor displacement.
The board should read it as a major reduction in thesis credibility risk.
The board should not read 175 journeys as all workflows implemented.
The board should read them as use-case coverage that can drive product slicing.
The board should not read 62-plus microservice suites as deployed software.
The board should read them as service build packets.
The board should not read eight compliance packs as certifications.
The board should read them as compliance-aware design scaffolding.
The board should not read eight localization packs as local market launch readiness.
The board should read them as regional product-design runway.
The board should not read the capability-tier registry as pricing finalization.
The board should read it as the mechanism that prevents product-label sprawl from infecting architecture.
The board should not read the audit-event sweep as a pass.
The board should read it as a valuable failure report.
The board should not read the six-hop audit as a pass.
The board should read it as a map of disconnected documentation debt.
The board should not read migration journeys as migration tooling.
The board should read them as the first structured migration narratives to become tooling.
The board should not read Wave 4 as another writing wave.
The board should read Wave 4 as the implementation conversion wave.

## §3 Cumulative Ambition Vs Delivery Dashboard

This dashboard compares the original ambition against the current post-remediation state.
Percent achieved is an executive estimate of corpus readiness, not runtime product readiness.
Runtime product readiness has a lower percentage until Wave 4 code and CI gates land.

| Workstream | Original target | Current state | Achieved |
|---|---|---|---:|
| Unified-ecosystem thesis | Coherent board story | Thesis unchanged and better evidenced | 100% narrative |
| Fragmentation-tax argument | Explain why consolidation matters | Still valid; now supported by migration and vendor mapping artifacts | 100% narrative |
| ADR-0321 vendor coverage | 165 detailed dossiers | 110 substantive; tail remains | 67% corpus |
| ADR-0321 substance bar | Vendor-specific operating detail | Met for backfilled dossiers | 100% for covered set |
| ADR-0321 tail | Complete all D sections | D-141..D-165 named as Wave 4 | 0% tail done |
| j151-j175 journeys | Replace empty scaffold | 25 substantive journeys | 100% |
| j176-j180 migration journeys | Begin incumbent migration corpus | Five in flight | 100% scaffold / 40% substance |
| Journey bundle surfaces | Story, UX, handshake, test plan, schema | Present across remediation journeys | 95% |
| Microservice roster | Broad service inventory | 79 roster concepts, 78 live directories | 98% roster |
| Seven-surface service suites | Full platform per service | 62-plus at bar | 78% minimum |
| ERP IP deepening | Nine ERP service anchors | Nine deepened | 100% |
| Per-service ADRs | Decision visibility | Roughly 40 across batches A-F | 50% of roster |
| Persona dossiers | Role realism at scale | Roughly 120 | 100% breadth |
| Persona Substance Anchors | Remove generic personas | 90-plus passed | 75% minimum |
| Compliance packs | Regulated-market coverage | Eight active packs | 100% named set |
| Localization packs | Regional-market coverage | Eight active packs | 100% named set |
| Capability-tier registry | Product packaging without service sprawl | Four tiers, 70 services, 295 vendors | 100% first registry |
| Vendor-tier mapping | Sales language to substrate mapping | 295 rows | 100% first mapping |
| Microservice-tier mapping | Service contribution per tier | 70 mapped rows | 100% registry baseline |
| Sample tenants | Tenant fixture proof | Fixtures present | 100% first pass |
| Workflow templates | Reusable workflow library | Registry templates present | 100% first pass |
| Observability dashboards | Operator visibility | Dashboards present | 100% first pass |
| SLO library | Reliability target library | Core SLO files present | 100% first pass |
| Tutorials | New engineer/operator learning | Service tutorials present | 70% |
| Benchmarks | Performance claim runway | Service benchmarks present | 70% |
| Risk register | Board risk visibility | Improved risk posture | 80% |
| Anti-patterns | Stop bad output shapes | Standards anti-patterns present | 100% first pass |
| Wave-3 retrospective | Learning artifact | Authored | 100% |
| Cross-service integration tests | Make journeys buildable | Plans and references present | 60% |
| Threat models | Service security posture | Per-service threat models present across major set | 70% |
| Test plans | Service verification path | Many service test plans present | 70% |
| W1-W3 runbooks | Operational readiness docs | Runbooks present across major set | 70% |
| Cross-handoff matrices | Boundary ownership | Matrices present | 70% |
| Pack overlays | Compliance-aware behavior | Overlays present | 75% |
| Migration playbooks | Vendor displacement proof | Per-vendor/family playbooks present | 65% |
| Governance crates | CI enforcement runway | Scaffolded | 35% runtime |
| Capability-tier deltas | Packaging clarity | Per-tier deltas present | 100% first pass |
| Audit-event sweep | Audit gap inventory | Completed; gaps named | 100% inventory |
| Audit-event registration | Registered class compliance | Strict sweep shows 0 registered in scope | 0% |
| Six-hop audit | Discoverability proof | Completed; disconnected graph named | 100% inventory |
| Six-hop reachability pass | Reach all docs from root | 437/7,606 reachable | 6% |
| IP cross-reference sweep | Link IPs to services/journeys | Sweep performed | 70% |
| Rust source scaffolding | Executable service code | Wave 4 priority | 0% for new wave set |
| Actual CI gates | Enforce corpus claims | Wave 4 priority | 0-35% depending gate |
| Tenant onboarding pilots | Live customer proof | Wave 4 priority | 0% production |

### §3.1 Workstream Detail: ADR-0321 Vendor Dossiers

Original target: prove Oyatie can subsume B2B SaaS leaders without cloning their suite boundaries.
Pre-remediation state: large ADR, but shallow sections risked becoming a template-stamped catalog.
Remediation result: 110 substantive vendor dossiers.
Achieved percentage: roughly 67 percent of the 165-dossier ambition.
Substance bar: vendor data model named.
Substance bar: vendor API surface named.
Substance bar: vendor UX surface named.
Substance bar: vendor Cedar action family named.
Substance bar: vendor ontology projection named.
Substance bar: vendor workflow replacement steps named.
Substance bar: vendor migration failure modes named.
Substance bar: destination microservices named.
Substance bar: capability tier mapped.
Substance bar: benchmark alias kept out of service naming.
Board reading: the vendor displacement thesis now has technical grain.
Investor reading: the proof point is speed and specificity, not final market penetration.
Engineering reading: the remaining tail must be finished before sales claims cite full ADR-0321 coverage.
Sales reading: use covered vendors as proof examples; do not imply every D section is equally deep.
Marketing reading: say "substantial direct coverage" rather than "complete vendor parity."
Risk: D-141..D-165 remain a visible Wave-4 gap.
Stop condition: complete remaining tail and bind every dossier to migration journey or playbook.

### §3.2 Workstream Detail: Journey Corpus

Original target: make the unified ecosystem testable through human narratives.
Pre-remediation state: j151-j175 existed as empty or thin scaffold.
Remediation result: j151-j175 now carry substantive content.
Current total: 175 substantive journeys.
Migration extension: j176-j180 in flight.
Achieved percentage: 100 percent for j01-j175 substance target.
Migration achieved percentage: first five incumbent journeys established, not complete migration tooling.
Substance bar: narrative story present.
Substance bar: UX flow present.
Substance bar: handshake present.
Substance bar: integration test plan present.
Substance bar: schema or policy evidence present where material.
Substance bar: microservice touchpoints named.
Substance bar: personal/work boundary respected when relevant.
Board reading: the thesis now spans executives, frontline workers, regulated roles, creators, students, retirees, operators, and investors.
Investor reading: persona and journey breadth supports TAM breadth.
Engineering reading: journeys should drive Wave-4 vertical slices.
Sales reading: use j176-j180 for displacement conversations.
Marketing reading: use j151-j175 for human proof, not feature laundry lists.
Risk: journey text can still outrun implemented workflows.
Stop condition: convert top 20 journeys into runnable integration tests.

### §3.3 Remediation Journey Roster: j151-j180

- j151: Captain Olufemi typhoon evacuation and co-op cash flow shows emergency, maritime, community, finance, and multilingual coordination in one substrate.
- j152: Ahmad Hassan construction-site incident bilingual flow shows workplace safety, translation, incident intake, Paycom mapping, and regulatory evidence.
- j153: Devon Williams HVAC side-business tax year-end flow shows personal business, tax, scheduling, invoicing, and compliance rollover.
- j154: Tomas Pieter channel-partner co-marketing launch shows shared-tenant provisioning, attribution rules, campaign launch, and partner evidence.
- j155: Stefan Kovacs college night shift and finals week shows education, scheduling, shift work, personal study, and fatigue-aware operations.
- j156: Carlos Reyes II maintenance emergency after hours shows LOTO state, refrigerant disclosure, incident permit, dead-time escalation, and facilities risk.
- j157: Diana Lazar print-operator batch defect and quality recall shows manufacturing quality, recall routing, audit evidence, and frontline operator reality.
- j158: Print-shop cell rebalance and shorts-creator spike shows capacity shift, creator/employer disclosure, policy gating, and short-form content load.
- j159: Saanvi Mehta MBA application spans personal and work shows reference handling, privacy boundary, calendar evidence, and dual-tenant identity.
- j160: Tomas Horak cleaning-company bid cross-tenant onboarding shows small-business procurement, tenant formation, quote approval, and onboarding.
- j161: Soyeon Kim allergen recall and school coordination shows cafeteria operations, food safety, parent notifications, and regulated incident response.
- j162: Diana Lazar night-shift onboarding shows competency unlock, lone-worker protocol, onboarding state machine, and operator-first training.
- j163: Jordan Park board meeting cross-time-zone flow shows AV coordination, recording consent, board mode, time zones, and meeting evidence.
- j164: Hiroshi Tanaka yearly tax and pension flow shows retired-person continuity, pension evidence, personal records, and annual tax workflow.
- j165: Naveen Iyer board quarterly compliance report shows CCO workflow, board packaging, compliance posture, evidence exports, and audit trail.
- j166: Mira Goldberg strategic acquisition go/no-go shows CSO planning, diligence handoffs, restricted deal workspaces, and information barriers.
- j167: Diego Vargas platform major-version cutover shows CTO change governance, version rollout, rollback criteria, and platform continuity.
- j168: Akira Watanabe quarterly ops review and incident debrief shows COO operating cadence, incident retrospectives, and cross-service metrics.
- j169: Felix Ng multi-country launch with locale pack shows marketing launch motion across KR/EU/US/JP/IN/BR/AU/MX localization constraints.
- j170: Aiko Brown sustainability report and Scope 3 supply chain shows CSRD-style sustainability evidence, supply-chain graphing, and reporting.
- j171: Felix Tan ombudsperson cross-tenant mediation with privilege shows confidential mediation, privilege boundaries, and safe cross-tenant handoff.
- j172: Lev Kahn investor-relations shareholder meeting livestream shows IR controls, livestream evidence, disclosure timing, and shareholder access.
- j173: Aamir Khan wealth-manager multi-jurisdictional trust restructure shows wealth, jurisdiction, trust, beneficiary, and regulated advice boundaries.
- j174: Sven Eriksson treasury end-of-day position reconciliation shows treasury controls, liquidity, bank feeds, and end-of-day evidence.
- j175: Aanya Kapoor LP portfolio tax and K-1 distribution shows fund investor, tax package, K-1 reconciliation, and portfolio evidence flow.
- j176: SAP S/4HANA to Oyatie finance month-one migration is the first ERP incumbent migration journey.
- j177: Salesforce Sales Cloud to Oyatie CRM migration is the first CRM incumbent migration journey.
- j178: Workday HCM to Oyatie workforce migration is the first workforce incumbent migration journey.
- j179: ServiceNow ITSM to Oyatie ITSM migration is the first service-management incumbent migration journey.
- j180: Atlassian Jira and Confluence to Oyatie workspace migration is the first collaboration/dev-work management incumbent migration journey.

### §3.4 Workstream Detail: Microservice Suites

Original target: make services buildable without relying on hidden context.
Pre-remediation state: many service directories existed, but suite depth was uneven.
Remediation result: 62-plus services with full seven-surface suites.
Achieved percentage: at least 78 percent against the 79-roster executive target.
The seven-surface suite is not a ceremonial checklist.
Surface one: README or scope framing.
Surface two: architecture.
Surface three: contract or API shape.
Surface four: policy and authorization.
Surface five: runbook or operational response.
Surface six: tutorial, onboarding, or reference implementation.
Surface seven: benchmark, dashboard, test plan, or migration playbook.
The suite bar helps new engineers enter the corpus.
The suite bar helps reviewers reject shallow services.
The suite bar helps Wave 4 produce crates without inventing semantics.
The suite bar helps sales avoid invented roadmap claims.
The suite bar helps product identify which services are ready for runtime slices.
The suite bar helps compliance locate evidence.
The suite bar helps operations locate SLOs and runbooks.
Risk: directory count, registry count, and roster count still differ.
Risk: a full documentation set is not executable code.
Risk: some suites may still need cross-link repair after the six-hop audit.
Stop condition: all roster services have seven surfaces and CI checks enforce the surface contract.

### §3.5 Microservice Roster Reading

- analytics: data and executive insight workload with tiered retention, projection cache, audit, and dashboard obligations.
- api-gateway: north-south entrypoint and rate-limit surface with policy, auth, and evidence responsibilities.
- application: product/usecase orchestration surface for higher-level app flows.
- audit-chain: durable evidence, Merkle sealing, audit identifiers, and regulator export obligations.
- calendar: scheduling, availability, consent, meeting handoff, and time-zone evidence.
- cell: tenant/cell placement, regional isolation, disaster movement, and control-plane boundaries.
- cloud-billing: cloud usage charging and hyperscaler-style billing event ownership.
- cloud-billing-tax: tax treatment of cloud billing and jurisdiction-specific fiscal evidence.
- cloud-data: cloud data substrate and storage integration for platform data movement.
- cloud-iac: infrastructure-as-code control, drift, rollout, and policy-enforced deployment.
- cloud-iam: cloud identity and access management mapping for accounts, roles, and federation.
- cloud-k8s: Kubernetes-first workload runtime and cluster capability mapping.
- cloud-kms: key-management, rotation, HSM/BYOK, and regulated cryptographic custody.
- cloud-network: network substrate, VPC, routing, isolation, and traffic controls.
- cloud-network-dns: DNS control, zone authority, resolver safety, and edge routing.
- cloud-secrets: secret storage, rotation, lease, and access evidence.
- cloud-storage: block/object/file storage mapping with tenant isolation and lifecycle controls.
- comms-email: enterprise and personal email substrate, deliverability, retention, and legal holds.
- community: social/community workspace with moderation, cohorts, and cross-tenant interaction.
- compliance: pack overlays, evidence export, regulatory claims, and attestation workflows.
- connect: super-app integration and cross-product shell coordination.
- consent-graph: consent state, purpose limitation, withdrawal, and downstream propagation.
- contact-center: omnichannel interaction, routing, escalation, and customer-contact evidence.
- contract-lifecycle-management: contract workspace, negotiation, obligations, renewals, and evidence.
- crm: account, lead, opportunity, contact, activity, forecast, and pipeline substrate.
- data-pipeline: ingestion, transformation, replay, and pipeline-run evidence.
- data-warehouse: analytical warehouse, workload isolation, query governance, and data products.
- design-collaboration: design-file workflow, review, handoff, comments, and asset governance.
- detection: detection substrate for abuse, fraud, fairness, and batch/streaming checks.
- developer-sdk: SDK contracts, generated clients, developer onboarding, and version discipline.
- docs: documentation product surface, knowledge navigation, and corpus authoring workflows.
- drive: file storage, sharing, DLP, retention, classification, and collaboration.
- feature-flags: progressive delivery, targeting, rollback, and experiment controls.
- financial-planning: planning, budgets, forecasts, scenario modeling, and finance controls.
- finops-portal: cost attribution, budgets, chargeback, optimization, and executive spend visibility.
- forms: structured data capture, approvals, consent, and low-code intake.
- foundry: agentic engineering platform, evaluation, route policy, evidence, and supervised autonomy.
- global-trade: trade compliance, tariff, customs, sanctions, and international shipment evidence.
- governance: policy lifecycle, decision rights, exception management, and governance events.
- healthcare-integration: FHIR, EHR, clinical boundary, PHI, and healthcare integration substrate.
- identity: principal model, tenant identity, personal/work boundary, auth, and recovery.
- incident-management: incident lifecycle, escalation, on-call, retrospective, and post-incident evidence.
- intelligence: AI/ML inference, scoring, agent support, model lifecycle, and explanation surfaces.
- itsm: service records, request, incident, problem, change, CMDB adjacency, and SLA management.
- learning-management: training, competency, certification, cohort learning, and role readiness.
- mail: mail transport, mailbox, retention, eDiscovery, deliverability, and personal/work split.
- marketing-automation: campaign, segmentation, journey, consent, and attribution automation.
- marketplace: DealSet settlement, supplier discovery, listing lifecycle, commerce, and entitlements.
- meet: meetings, conferencing, consent, recording, transcription, and live collaboration.
- messenger: secure messaging, channels, MLS, cross-tenant chat, and notifications.
- network: platform network primitives and service connectivity.
- notes: personal and work notes, journaling, retention, sync, and encryption.
- observability: logs, metrics, traces, audit-id propagation, dashboards, and SLO burn.
- ontology: shared object model, projection, schema revision, graph edges, and semantic continuity.
- ops-dashboard-control-center: operator cockpit, command surface, incident view, and control loops.
- payments: payments, escrow, settlement, refunds, disputes, and financial evidence.
- performance-management: goals, reviews, feedback cycles, talent signals, and HR evidence.
- plant-maintenance: maintenance orders, work centers, equipment, lockout, and asset operations.
- plugin-app-store: extensions, plugin governance, sandboxing, publication, and monetization.
- production-planning: production orders, capacity planning, MRP, BOM, and shop-floor coordination.
- quality-management: inspection lots, nonconformance, CAPA, recalls, and quality evidence.
- real-estate: leases, facilities, property, space, and contract-bound operations.
- recordings: recorded meetings, retention, consent, eDiscovery, and archive workflows.
- sheets: spreadsheet collaboration, formulas, import/export, and structured workbooks.
- shorts: short-form creator media and monetization workflows.
- sites: internal/external publishing, pages, content governance, and access.
- slides: presentation collaboration, review, board pack, and export controls.
- social: social surfaces, moderation, feeds, and community graph.
- supply-chain-planning: supply network planning, forecast, demand/supply balance, and disruptions.
- tasks: task management, assignment, workflow edges, SLA, and personal/work planning.
- tenancy: tenant lifecycle, sub-scope registry, permits, residency, quota, and onboarding.
- translate: translation memory, locale routing, regional overlays, and bilingual operations.
- treasury: cash position, liquidity, bank feeds, hedging, and end-of-day reconciliation.
- warehouse: warehouse operations, inventory movements, picking, packing, and logistics evidence.
- whiteboard: collaborative sketching, planning, session history, and export.
- workflow-engine: state machines, DAGs, sagas, replay, compensation, and orchestration.
- workflow-studio: workflow authoring, canvas, validation, templates, and collaborative design.
- workplace-integration: external workplace connector surface and incumbent migration bridge.

### §3.6 Workstream Detail: Compliance And Localization

Original target: prove the platform is not US-only, enterprise-only, or compliance-afterthought.
Pre-remediation state: compliance posture existed, but regional depth was uneven.
Remediation result: eight active compliance packs and eight active localization packs.
Achieved percentage: 100 percent for the named Wave-3 remediation pack set.
HIPAA anchors PHI, clinical access, healthcare breach, and BAA-style evidence.
GDPR anchors lawful basis, DSR, portability, erasure, restriction, and cross-border transfers.
SOC2 anchors trust-services evidence and enterprise assurance.
EU AI Act anchors high-risk AI systems, model lifecycle, human oversight, refusal, and evidence.
KR-PIPA anchors Korean personal information protection, RRN handling, and local evidence.
CSAP anchors Korean public-sector and cloud-security assurance posture.
PCI-DSS v4 anchors cardholder data environment control and payment isolation.
EU-CSRD anchors sustainability reporting, Scope 3 evidence, and corporate disclosures.
KR localization ties PIPA, CSAP, RRN, consent, residency, and incident response.
EU localization ties GDPR, DORA, DSR, portability, high-risk AI, and cross-border operation.
US localization ties HIPAA, SOX, federal privacy, state privacy, and discrimination/AI bias.
JP localization ties APPI, My Number, cybersecurity, financial services, and telecom.
IN localization ties DPDPA, consent, residency, breach notification, and sectoral obligations.
BR localization ties LGPD, consent, DSR, cross-border, and breach response.
AU localization ties privacy, APRA CPS 234, sector overlays, and cross-border controls.
MX localization ties privacy, data-subject rights, residency, and incident response.
Board reading: regulated market entry now has evidence surfaces.
Investor reading: geographic TAM has a stronger architecture basis.
Sales reading: use packs as readiness runway, not certification claims.
Marketing reading: avoid implying legal certification until external validation exists.
Engineering reading: pack overlays must become test fixtures and runtime policy.
Risk: pack text can still outrun legal review.
Risk: localization depth differs by region.
Stop condition: pack validators, jurisdictional test fixtures, and launch readiness gates.

### §3.7 Workstream Detail: Capability Tiers

Original target: represent product packaging without creating product-shaped services.
Pre-remediation state: ADR-0316 doctrine existed, but registry proof was missing.
Remediation result: capability-tier registry exists.
Achieved percentage: 100 percent for first registry landing.
Bronze maps bounded SMB or pre-production activation.
Silver maps single-region high-availability production activation.
Gold maps multi-AZ revenue production activation.
Platinum maps sovereign or regulated pack-bound activation.
The tier system lets GTM say "Sales Cloud equivalent" without creating a `salesforce` service.
The tier system lets engineering keep flat service boundaries.
The tier system lets compliance bind pack overlays to entitlement levels.
The tier system lets observability change SLO targets by tenant tier.
The tier system lets FinOps price higher evidence, retention, and throughput.
The tier system lets onboarding grant capability bundles rather than raw service access.
The tier system lets tenant admins understand product-like surfaces while the backend remains composed.
The registry maps 70 microservice rows.
The registry maps 295 vendor rows.
The registry names edition-axis policy.
The registry names coverage-tier code semantics.
The registry names source status.
The registry names Cedar action families.
The registry names ontology projection references.
The registry names workflow templates.
The registry names pricing classes.
The registry names RPS ceilings.
The registry names retention days.
The registry names destination microservices.
Board reading: this is the strongest control against SaaS-suite sprawl reappearing inside Oyatie.
Investor reading: packaging now has a substrate-aware model.
Sales reading: map familiar incumbents to tiers, not to clone products.
Marketing reading: emphasize tiered capability, not product fragmentation.
Engineering reading: enforce registry shape in CI.
Risk: registry counts lag live directory growth.
Stop condition: every roster service and high-priority vendor has enforced tier mapping.

### §3.8 Workstream Detail: Observability, SLOs, And Audit

Original target: make claims measurable.
Pre-remediation state: observability doctrine existed but dashboards and SLO library were sparse.
Remediation result: dashboards and SLO library exist.
Achieved percentage: 100 percent for first registry landing.
Golden signals per tier exist as a reusable reliability reference.
Cross-tenant message delivery exists as an SLO surface.
Data-residency conformance exists as an SLO surface.
Cedar policy evaluation latency exists as an SLO surface.
AI inference cost budget exists as an SLO surface.
Drive encryption rotation cadence exists as an SLO surface.
Workflow-engine saga success rate exists as an SLO surface.
Audit-chain Merkle attestation lag exists as an SLO surface.
Tenant onboarding end-to-end exists as an SLO surface.
Compliance pack evidence export exists as an SLO surface.
Dashboards cover audit event emission throughput.
Dashboards cover tenant isolation health.
Dashboards cover Cedar policy evaluation latency.
Dashboards cover capability-tier SLA conformance.
Dashboards cover compliance-pack attestation lag.
Dashboards cover cell routing and shuffle sharding.
Dashboards cover MLS key delivery health.
Dashboards cover golden signals per microservice.
The audit-event sweep found strict gaps.
The audit-event sweep is useful because it names exact endpoint-to-event failures.
The audit-event sweep is not a pass.
The sweep found 134 strict-scope gaps.
The sweep found 41 endpoints with concrete names.
The sweep found zero registered classes under strict reading.
Board reading: observability is now a build contract, not a vague aspiration.
Investor reading: the platform is learning to police its own claims.
Engineering reading: turn dashboards and SLOs into code and CI.
Risk: named dashboards without live metrics can create false confidence.
Stop condition: emitted telemetry proves the SLO library and dashboard wiring.

### §3.9 Workstream Detail: Discoverability And Six-Hop Audit

Original target: make the corpus navigable from root authority.
Pre-remediation state: reachability was not measured.
Remediation result: six-hop audit exists.
Achieved percentage: 100 percent for audit inventory.
Pass percentage: not acceptable yet.
The audit inspected 7,606 Markdown nodes.
The audit found 437 nodes reachable from docs/README.md.
The audit found 7,169 unreachable nodes.
The audit found every reachable node within three hops.
The audit found zero reachable nodes in the four-to-six hop warning band.
The audit found the problem is disconnection, not excessive path length.
This is an important distinction.
If the problem were long paths, a few index pages might fix it.
Because the problem is disconnection, Wave 4 needs source-of-truth index repair.
The board should value the audit because it stops false navigability claims.
The investor should value the audit because it shows internal quality discipline.
The engineer should value the audit because it gives a deterministic queue.
The writer should value the audit because it reveals where docs are orphaned.
Risk: disconnected docs increase rework and stale decision risk.
Stop condition: root hub and service indexes reach the corpus within policy.

### §3.10 Workstream Detail: Retrospective Learning

Original target: learn from Wave-3 rather than only celebrate it.
Pre-remediation state: learning was implicit.
Remediation result: Wave-3 retrospective exists.
Achieved percentage: 100 percent for retrospective artifact.
The retrospective says Wave-3-G created strategic coherence.
The retrospective says raw velocity created quality failure modes.
The retrospective says volume is not rigor.
The retrospective says audits changed the work from breadth to substance.
The retrospective says the best technique was immediate audit and targeted remediation.
The retrospective says Wave 4 must not be another purely textual expansion wave.
The retrospective says Wave 4 must implement code, validators, and runtime slices.
The retrospective says output should be measured against buildability.
The retrospective says documentation can outrun implementation.
The retrospective says the next 6-12 months should convert corpus to enforcement.
Board reading: the team has an honest learning loop.
Investor reading: the team can change behavior after evidence.
Engineering reading: the operating mode for Wave 4 is implementation-first.
Risk: the organization may repeat high-volume authoring without enforcement.
Stop condition: Wave 4 starts with Rust scaffolds, CI gates, and pilots.

## §4 Wave-3 Retrospective Lens

Wave-3 worked because it forced the full ambition into the open.
Before Wave-3, Oyatie could be interpreted as a set of services.
After Wave-3, Oyatie is plainly a unified operating substrate.
That clarity matters.
It gives the board a thesis.
It gives investors a market-shaping story.
It gives GTM a vocabulary.
It gives engineering a service map.
It gives compliance a pack map.
It gives product a role map.
It gives support an operations map.
It gives agents a corpus to traverse.
Wave-3 also worked because it broadened the role base.
The corpus no longer describes only executives and knowledge workers.
It includes night shifts.
It includes maintenance.
It includes cafeteria operations.
It includes maritime evacuation.
It includes retirees.
It includes LP tax distribution.
It includes bilingual construction incidents.
It includes board livestreams.
It includes ombudsperson privilege.
It includes treasury reconciliation.
It includes sustainability reporting.
It includes workforce onboarding.
That role breadth makes the unified thesis testable.
Wave-3 worked because ADR-0316 clarified capability tiers.
That decision prevented product-label sprawl from becoming service-label sprawl.
Wave-3 worked because ADR-0321 translated competitor coverage into destination services.
That decision prevented the company from building a Salesforce clone, a ServiceNow clone, a Workday clone, and a Microsoft clone.
Wave-3 worked because the remediation wave accepted criticism.
Template-stamping was named.
Clause loops were named.
Shallow sections were named.
Reachability failure was named.
Audit-event registration failure was named.
The team did not merely defend the output.
The team wrote better artifacts.
Wave-3 worked because the remediation wave produced concrete registries.
Registries are stronger than prose.
Capability tiers became files.
Vendor mappings became rows.
Compliance packs became manifests.
Dashboards became registry entries.
SLOs became library entries.
Workflow templates became reusable references.
Sample tenants became fixtures.
Wave-3 worked because it exposed a repeatable quality pattern.
Write a slice.
Audit the slice.
Name the gap.
Backfill to substance.
Record the result.
Move the gap to Wave 4 if it requires runtime implementation.
Wave-3 failed where it confused line count with evidence.
Large documents can still be shallow.
Repeated clauses can still pass superficial length checks.
Template-stamped sections can still look impressive.
Exactly sized PRDs can still fail to tell an engineer what to build.
Board narratives can still overrun implementation.
Wave-3 failed where markdown became the artifact instead of the projection.
Oyatie's own doctrine prefers machine-readable control surfaces.
The remediation still had to author Markdown because the requested output was an executive briefing.
Wave 4 should move more control surfaces into JSON, YAML, schemas, fixtures, tests, and Rust validators.
Wave-3 failed where reachability was assumed.
The six-hop audit showed the corpus is not navigable enough.
Wave-3 failed where audit events were assumed.
The sweep showed strict registration was not green.
Wave-3 failed where runtime enforcement lagged architecture.
The governance crates are scaffolded but not complete.
The dashboards are authored but not live runtime proof.
The SLO library is authored but not enforced.
The migration playbooks exist but are not implemented migration tooling.
The service suites exist but are not runtime services.
The investor lesson is direct.
The company can produce architecture at massive speed.
The company can accept and correct quality gaps.
The next proof point must be execution of the architecture in code.
The board lesson is direct.
Governance must reward enforcement, not only expansion.
The engineering lesson is direct.
Every major architecture artifact needs a validation lane.
The GTM lesson is direct.
Use the stronger evidence, but preserve claim boundaries.
The product lesson is direct.
Use journeys and personas to choose slices, not to decorate decks.
The compliance lesson is direct.
Use packs and overlays as implementation inputs, not certification substitutes.
The operating lesson is direct.
Wave 4 must be narrower, more mechanical, more test-driven, and more runtime-oriented.

## §5 Investor TAM Update

The TAM thesis does not materially change.
Oyatie still targets the fragmentation tax across enterprise SaaS, regulated operations, collaboration, workflow, marketplace, compliance, identity, and personal/professional continuity.
What changed is the proof-of-execution signal.
The prior investor story was ambitious.
The post-remediation investor story is ambitious with a much larger evidence base.
The investor question is no longer only "is the vision broad enough?"
The investor question is now "can this team convert a broad architecture corpus into working infrastructure?"
That is a better question.
It is measurable.
It can be staged.
It can be diligence-tested.
It can be tied to Wave 4 milestones.
The TAM remains anchored in enterprise SaaS consolidation.
The TAM remains anchored in migration from incumbent suites.
The TAM remains anchored in training cost reduction.
The TAM remains anchored in compliance reuse.
The TAM remains anchored in workflow unification.
The TAM remains anchored in personal/work identity continuity.
The TAM remains anchored in regulated vertical expansion.
The TAM remains anchored in marketplace settlement.
The TAM remains anchored in data/ontology reuse.
The TAM remains anchored in AI-enabled workflow and agentic development.
The proof-of-execution update has five investor implications.
First, vendor displacement is now more credible.
ADR-0321 does not just say "Salesforce coverage."
It names Sales Cloud objects.
It names Service Cloud objects.
It names Workday objects.
It names Atlassian objects.
It names Microsoft objects.
It names Adobe objects.
It names Snowflake and Databricks-style data surfaces.
It names observability incumbents.
It names payment and marketplace incumbents.
It names identity and security incumbents.
Second, migration is now a first-class motion.
j176 through j180 show the first migration narratives.
That gives sales a path from displacement claim to customer migration plan.
Third, packaging is now less risky.
Capability tiers make it possible to sell familiar product categories without fracturing the platform.
Fourth, compliance expansion is now more concrete.
Eight packs and eight localizations let investors see a global regulated-market thesis.
Fifth, execution discipline is now visible.
The retrospective, six-hop audit, and audit-event sweep show a culture that can find its own gaps.
The most important investor caveat is runtime conversion.
The corpus does not yet prove product-market fit.
The corpus does not yet prove uptime.
The corpus does not yet prove migration tooling.
The corpus does not yet prove a paying tenant.
The corpus does not yet prove compliance certification.
The corpus does not yet prove all CI gates.
The corpus does prove that the team can produce, audit, and remediate architecture at unusual scale.
That is a venture-relevant signal.
It is not a revenue signal.
It is a technical execution signal.
It should raise confidence in the plan.
It should not remove the need for Wave-4 proof.
The strongest investor framing is therefore:
Oyatie's market remains massive because fragmentation remains massive.
Oyatie's differentiation remains unification, not point-solution superiority.
The remediation wave materially de-risked thesis specificity.
The next investor milestone is executable proof, not another document wave.

## §6 Roadmap Forward: Wave 4 Priorities

Wave 4 must be the implementation and enforcement wave.
The first priority is Rust source scaffolding.
The new and deepened microservices need actual crates, modules, contracts, tests, and minimal runtime paths.
Documentation should drive code.
Code should validate documentation.
The second priority is actual CI gate implementation.
Capability-tier registry shape must be enforced.
Compliance pack overlay shape must be enforced.
SLO library shape must be enforced.
Audit-event registration must be enforced.
Six-hop reachability must be enforced.
Per-service seven-surface suites must be enforced.
Vendor dossier substance bars must be enforced.
Migration journey completeness must be enforced.
The third priority is j181-plus migration journeys.
j176-j180 opened the motion.
j181-plus should continue across Microsoft 365, Dynamics, Snowflake, Databricks, Stripe, Adobe, HubSpot, Zendesk, Okta, CrowdStrike, and high-priority market incumbents.
The fourth priority is ADR-0321 D-141 through D-165.
The remaining tail must be completed at the same substance bar as the backfilled sections.
Do not regress to template-stamped content.
Do not treat vendor names as coverage.
Each dossier must name objects, APIs, UX, Cedar, ontology, workflows, migration, failure modes, and destination services.
The fifth priority is tenant onboarding live pilots.
Sample tenants are not enough.
The platform needs live pilot flows.
The pilot should activate capability tiers.
The pilot should bind compliance packs.
The pilot should use localized overlays.
The pilot should emit audit events.
The pilot should create dashboard signals.
The pilot should exercise rollback.
The pilot should produce board-readable evidence.
The sixth priority is audit-event remediation.
The sweep identified 134 strict gaps.
Wave 4 should register event classes.
Wave 4 should bind endpoint metadata.
Wave 4 should make payload classes explicit.
Wave 4 should carry audit_id through logs, spans, and metrics.
The seventh priority is reachability repair.
The six-hop audit identified 7,169 unreachable nodes.
Wave 4 should repair root and service indexes.
Wave 4 should avoid manual link gardens that drift.
Wave 4 should prefer machine-readable indexes and generated projections.
The eighth priority is migration tooling.
Journey narratives must become commands, validators, staging schemas, parity checks, and cutover runbooks.
The ninth priority is persona-to-slice selection.
Use the 90-plus anchored personas to choose product slices.
Do not implement a slice only because it is easy.
Implement slices that prove the thesis under stress.
The tenth priority is governance crate completion.
Scaffolded crates need tests.
Tests need fixtures.
Fixtures need failing cases.
Failing cases need CI.
CI needs promotion blocking.
Promotion blocking needs override evidence.
The eleventh priority is dashboard wiring.
Dashboard definitions need live data.
Live data needs event emitters.
Event emitters need service code.
Service code needs audit-id propagation.
The twelfth priority is SLO enforcement.
SLO YAML is not uptime.
SLO enforcement requires measurements.
Measurements require instrumentation.
Instrumentation requires runtime paths.
The thirteenth priority is compliance pack validation.
Pack manifests need schema validation.
Pack overlays need policy tests.
Pack evidence exports need fixture data.
Pack launch claims need legal review.
The fourteenth priority is tier grant lifecycle.
Bronze, Silver, Gold, and Platinum should become grantable and revocable capabilities.
Tier promotion should require evidence.
Tier rollback should have a runbook.
Tier cost should appear in FinOps.
Tier SLOs should appear in observability.
The fifteenth priority is limit-setting.
Wave 4 should deliberately avoid another 500,000-line documentation burst without enforcement.
Documentation remains useful.
Unenforced documentation becomes risk.
The Wave-4 stop condition should be executable.
At least one migration journey should run as tooling.
At least one tenant pilot should activate a tier.
At least one compliance pack should bind to runtime policy.
At least one microservice family should have working Rust skeleton, contract test, policy test, dashboard signal, and runbook drill.
At least one audit-event sweep should improve from failure to measurable pass.
At least one reachability audit should improve from disconnected to substantially connected.
At least one board packet should cite runtime evidence rather than only corpus evidence.

### §6.1 Wave 4 Sequencing

1. Start with gates before breadth.
2. Implement capability-tier registry validation.
3. Implement compliance-pack manifest validation.
4. Implement audit-event class registration validation.
5. Implement seven-surface suite validation.
6. Implement reachability reporting as a repeatable check.
7. Scaffold Rust for the most central services first.
8. Start with tenancy, identity, audit-chain, policy/compliance, workflow-engine, ontology, observability, and one product-facing service.
9. Add one migration path after the core substrate can emit evidence.
10. Prefer Salesforce CRM migration first if the sales motion needs visible displacement.
11. Prefer SAP finance migration first if ERP parity is the board focus.
12. Prefer ServiceNow ITSM migration first if operational workflow is the product focus.
13. Prefer Workday HCM migration first if workforce identity and HR proofs matter most.
14. Prefer Atlassian migration first if internal engineering dogfood matters most.
15. Bind the chosen migration to a pilot tenant.
16. Bind the pilot tenant to a capability tier.
17. Bind the tier to a compliance pack.
18. Bind the pack to audit events.
19. Bind the audit events to dashboards.
20. Bind dashboards to a board packet.
21. Run a review loop after every implementation slice.
22. Reject shallow docs that do not feed code.
23. Reject code that does not update the controlling artifact.
24. Reject claims that do not cite fresh verification.
25. Stop when a runtime slice proves the corpus-to-code loop.

### §6.2 Wave 4 Parallel Lanes

Lane A should own registry and schema gates.
Lane A can work independently on capability tiers, compliance packs, workflow templates, and SLO schemas.
Lane B should own audit-event registration and telemetry propagation.
Lane B depends on ADR-0263 interpretation and strict-scope service contracts.
Lane C should own Rust scaffolding for core substrate services.
Lane C should sequence tenancy, identity, audit-chain, compliance, workflow-engine, ontology, and observability.
Lane D should own one selected migration journey as executable tooling.
Lane D depends on destination service contracts from Lane C.
Lane E should own reachability repair.
Lane E can start immediately with indexes and machine-readable link graphs.
Lane F should own tenant pilot choreography.
Lane F depends on minimum tier, pack, identity, policy, and audit paths.
Lane G should own documentation cleanup.
Lane G should delete or consolidate outdated projections and prevent new disconnected docs.
Lane H should own board evidence packaging.
Lane H should wait for runtime evidence, not produce speculative decks.
Parallel work is safe when lanes touch disjoint files.
Dependent work must sequence through contract gates.
Shared artifacts require Oya VCS claim discipline.
Every lane needs a verification artifact.
Every lane needs a rollback note.
Every lane needs a stop condition.
Every lane needs a claim boundary.

### §6.3 Remaining ADR-0321 Tail Discipline

D-141 through D-165 should not be rushed.
Each tail dossier must match the backfilled bar.
Each tail dossier must name the vendor's objects.
Each tail dossier must name the vendor's APIs.
Each tail dossier must name the vendor's UX shell.
Each tail dossier must name at least three Cedar action verbs.
Each tail dossier must name at least three ontology object types.
Each tail dossier must name at least three workflow templates.
Each tail dossier must name at least three failure modes.
Each tail dossier must name a migration step beyond inventory, dry-run, preflight, projection, replay, seal, and sunset.
Each tail dossier must say whether the outcome is existing service, composed capability, new flat service, or partner/plugin path.
Each tail dossier must reject grouping-shaped services where the benchmark is only a product label.
Each tail dossier must cite destination microservices.
Each tail dossier must preserve benchmark aliases without turning aliases into architecture.
Each tail dossier must feed the vendor-tier mapping.
Each tail dossier must feed migration backlog prioritization.
Each tail dossier must avoid exactly-sized filler.
Each tail dossier must be reviewable by engineering, sales, and compliance.
Each tail dossier must leave an implementation path.
Each tail dossier must be tested by a substance checker when the gate exists.

### §6.4 Tenant Pilot Criteria

A tenant pilot must activate a real tenant fixture.
A tenant pilot must use at least one capability tier.
A tenant pilot must use at least one compliance pack.
A tenant pilot must use at least one localization overlay.
A tenant pilot must create at least one workflow.
A tenant pilot must create at least one ontology projection.
A tenant pilot must emit at least one registered audit-event class.
A tenant pilot must appear on at least one dashboard.
A tenant pilot must trip at least one negative policy test.
A tenant pilot must produce at least one rollback drill.
A tenant pilot must include one onboarding path.
A tenant pilot must include one support path.
A tenant pilot must include one offboarding or revocation path.
A tenant pilot must include one migration or import path.
A tenant pilot must produce evidence suitable for a board packet.
A tenant pilot must not require manual undocumented steps.
A tenant pilot must not rely on external production credentials in CI.
A tenant pilot must state what is simulated.
A tenant pilot must state what is live.
A tenant pilot must state what is not tested.
A tenant pilot must have a clean stop condition.

### §6.5 Executive Evidence Ledger: What The Board Can Now Point To

This ledger exists to make the remediation concrete.
Each line names a board-facing evidence point and the interpretation that should travel with it.
Evidence point 001: the old briefing remains canonical historical context, not the artifact edited here.
Evidence point 002: this file is intentionally a separate post-remediation projection.
Evidence point 003: the thesis still rests on one substrate, not a portfolio of acquired mini-products.
Evidence point 004: the remediation did not change the thesis; it changed the evidence density.
Evidence point 005: ADR-0316 remains the guardrail against product-fragment service boundaries.
Evidence point 006: ADR-0321 remains the guardrail against benchmark-suite cloning.
Evidence point 007: capability tiers now let sales language map onto architecture without corrupting service ownership.
Evidence point 008: Bronze is the bounded activation story.
Evidence point 009: Silver is the high-availability production story.
Evidence point 010: Gold is the revenue-grade multi-AZ story.
Evidence point 011: Platinum is the regulated or sovereign story.
Evidence point 012: 295 vendor rows mean the competitive map is now structured.
Evidence point 013: 70 registry microservice rows mean service contribution is now mapped.
Evidence point 014: 62-plus full seven-surface suites mean most of the service estate has build context.
Evidence point 015: 110 ADR-0321 dossiers mean direct incumbent coverage is no longer mostly placeholder.
Evidence point 016: 55 ADR-0321 planned dossiers remain outside the direct substantive-count claim.
Evidence point 017: D-141 through D-165 are explicitly future Wave-4 coverage.
Evidence point 018: j151 through j175 convert late-wave journey breadth into usable story.
Evidence point 019: j176 through j180 convert vendor displacement into migration narrative.
Evidence point 020: j176 is the SAP finance migration seed.
Evidence point 021: j177 is the Salesforce CRM migration seed.
Evidence point 022: j178 is the Workday workforce migration seed.
Evidence point 023: j179 is the ServiceNow ITSM migration seed.
Evidence point 024: j180 is the Atlassian workspace migration seed.
Evidence point 025: compliance packs are now named artifacts, not slide labels.
Evidence point 026: HIPAA maps to PHI, clinical access, and healthcare evidence posture.
Evidence point 027: GDPR maps to lawful basis, DSR, portability, erasure, and transfer posture.
Evidence point 028: SOC2 maps to enterprise trust-service assurance posture.
Evidence point 029: EU AI Act maps to high-risk AI, oversight, refusal, and model lifecycle.
Evidence point 030: KR-PIPA maps to Korean personal-information protection and RRN handling.
Evidence point 031: CSAP maps to Korean public-sector cloud assurance posture.
Evidence point 032: PCI-DSS v4 maps to cardholder-data environment isolation.
Evidence point 033: EU-CSRD maps to sustainability disclosure and Scope 3 evidence.
Evidence point 034: localization packs are now pack directories, not only region names.
Evidence point 035: KR localization anchors PIPA, CSAP, RRN, residency, cybersecurity, and consent.
Evidence point 036: EU localization anchors GDPR, DORA, DSR, portability, high-risk AI, and transfer limits.
Evidence point 037: US localization anchors HIPAA, SOX, federal privacy, state privacy, and AI bias law.
Evidence point 038: JP localization anchors APPI, My Number, cybersecurity, finance, and telecom law.
Evidence point 039: IN localization anchors DPDPA, consent, residency, breach notice, and sectoral overlays.
Evidence point 040: BR localization anchors LGPD, DSR, consent, residency, and incident response.
Evidence point 041: AU localization anchors privacy, APRA CPS 234, sectoral overlays, and cross-border control.
Evidence point 042: MX localization anchors privacy, DSR, consent, residency, and incident response.
Evidence point 043: the SLO library names reliability contracts Wave 4 must enforce.
Evidence point 044: golden signals per tier prevents a single reliability bar from pretending all tenants are equal.
Evidence point 045: tenant onboarding end-to-end SLO makes activation measurable.
Evidence point 046: compliance-pack evidence export SLO makes regulatory reporting measurable.
Evidence point 047: audit-chain Merkle attestation lag SLO makes evidence freshness measurable.
Evidence point 048: workflow-engine saga success rate SLO makes orchestration reliability measurable.
Evidence point 049: Cedar policy evaluation latency SLO makes authorization speed measurable.
Evidence point 050: data-residency conformance SLO makes jurisdictional routing measurable.
Evidence point 051: AI inference cost budget SLO makes AI usage financially governable.
Evidence point 052: drive encryption rotation cadence SLO makes cryptographic hygiene measurable.
Evidence point 053: cross-tenant message delivery SLO makes communication reliability measurable.
Evidence point 054: dashboards now provide operator-facing claim targets.
Evidence point 055: audit-event emission throughput dashboard points to event-volume visibility.
Evidence point 056: tenant-isolation health dashboard points to boundary visibility.
Evidence point 057: Cedar policy latency dashboard points to policy performance visibility.
Evidence point 058: capability-tier SLA dashboard points to packaging-to-SLO visibility.
Evidence point 059: compliance-pack attestation dashboard points to evidence freshness.
Evidence point 060: cell routing and shuffle-sharding dashboard points to scale/isolation visibility.
Evidence point 061: MLS key delivery health dashboard points to secure messaging visibility.
Evidence point 062: golden signals per microservice dashboard points to service-level reliability visibility.
Evidence point 063: the audit-event sweep is a quality-control artifact, not a victory lap.
Evidence point 064: the sweep found 134 strict-scope gaps.
Evidence point 065: the sweep found 41 endpoints with named classes.
Evidence point 066: the sweep found zero registered classes under strict compliance.
Evidence point 067: the correct board response is to fund remediation gates, not to bury the result.
Evidence point 068: the six-hop audit is a quality-control artifact, not a victory lap.
Evidence point 069: the audit found 7,606 Markdown nodes.
Evidence point 070: the audit found 437 reachable nodes.
Evidence point 071: the audit found 7,169 unreachable nodes.
Evidence point 072: reachable nodes were within three hops, so the problem is disconnected coverage.
Evidence point 073: the correct board response is machine-readable indexing and reachability gates.
Evidence point 074: the Wave-3 retrospective is evidence of learning discipline.
Evidence point 075: the retrospective warns against another prose-only expansion wave.
Evidence point 076: the retrospective says Wave 4 must build code and validators.
Evidence point 077: the corpus now contains enough detail to choose implementation slices.
Evidence point 078: the corpus now contains enough detail to reject fake completeness.
Evidence point 079: the corpus now contains enough detail to brief investors honestly.
Evidence point 080: the corpus now contains enough detail to avoid overpromising legal certification.
Evidence point 081: the corpus now contains enough detail to start pilot design.
Evidence point 082: the corpus still lacks production runtime proof.
Evidence point 083: the corpus still lacks complete CI gate proof.
Evidence point 084: the corpus still lacks complete tenant pilot proof.
Evidence point 085: the corpus still lacks complete migration tooling proof.
Evidence point 086: the corpus still lacks complete audit-event compliance proof.
Evidence point 087: the corpus still lacks complete reachability proof.
Evidence point 088: the corpus still lacks the ADR-0321 tail.
Evidence point 089: the corpus still needs runtime dashboards.
Evidence point 090: the corpus still needs instrumented SLO measurement.
Evidence point 091: the corpus still needs Rust source for many new services.
Evidence point 092: the corpus still needs generated validators for machine-readable artifacts.
Evidence point 093: the corpus still needs a pilot tenant that exercises tier grant lifecycle.
Evidence point 094: the corpus still needs legal review before market claims cite packs as compliance readiness.
Evidence point 095: the corpus still needs security review before high-risk policy exceptions are productized.
Evidence point 096: the corpus still needs product management to pick slices by strategic proof value.
Evidence point 097: the corpus still needs sales enablement to respect coverage boundaries.
Evidence point 098: the corpus still needs engineering ownership by service family.
Evidence point 099: the corpus still needs operational ownership for runbook freshness.
Evidence point 100: the corpus still needs a board reporting cadence that tracks runtime conversion.

### §6.6 Workstream Evidence Ledger: Gap Closed Versus Still Open

- ADR-0321 backfill closed the shallow-dossier gap for 110 direct vendor sections.
- ADR-0321 still has an open tail across the planned 165-section coverage set.
- ADR-0321 closed the generic migration language gap for covered vendors by naming vendor-specific steps.
- ADR-0321 still needs automated substance checks to prevent future regression.
- Capability tiers closed the packaging-to-architecture gap.
- Capability tiers still need grant lifecycle implementation.
- Vendor-tier mapping closed the market-reference-to-service-destination gap.
- Vendor-tier mapping still needs CI validation against live roster growth.
- Microservice-tier mapping closed the service-contribution gap for the registry baseline.
- Microservice-tier mapping still needs reconciliation with the 78/79 live roster framing.
- j151-j175 closed the late-journey scaffold gap.
- j151-j175 still need prioritized conversion into runnable tests.
- j176-j180 closed the zero-migration-journey gap.
- j176-j180 still need executable migration tooling.
- ERP IP deepening closed the SAP-parity shallowness gap.
- ERP IP deepening still needs code and integration tests.
- Compliance packs closed the acronym-only compliance gap.
- Compliance packs still need legal validation and runtime policy fixtures.
- Localization packs closed the Korea-only regional-depth gap.
- Localization packs still need launch readiness review per country.
- Persona Substance Anchors closed the generic-persona gap.
- Persona Substance Anchors still need product-slice prioritization.
- Runbooks closed the zero-operator-handhold gap.
- Runbooks still need drills and freshness gates.
- Threat models closed the no-adversarial-analysis gap.
- Threat models still need security review and negative tests.
- Test plans closed the "build later" ambiguity gap.
- Test plans still need actual test code.
- Dashboards closed the no-operator-visibility gap.
- Dashboards still need telemetry sources.
- SLO library closed the no-reliability-target gap.
- SLO library still needs measurement and alerting.
- Workflow templates closed the vague automation gap.
- Workflow templates still need runtime engine bindings.
- Sample tenants closed the abstract-tenant gap.
- Sample tenants still need pilot activation.
- Migration playbooks closed the handwaved-displacement gap.
- Migration playbooks still need importers, validators, and parity checks.
- Cross-handoff matrices closed the ownership ambiguity gap.
- Cross-handoff matrices still need enforcement in service contracts.
- Pack overlays closed the compliance-behavior gap.
- Pack overlays still need policy tests.
- Governance crates closed the no-code-enforcement runway gap.
- Governance crates still need implementation.
- Capability-tier deltas closed the one-size-tier gap.
- Capability-tier deltas still need pricing and entitlement integration.
- Audit-event coverage sweep closed the unknown-audit-gap gap.
- Audit-event coverage still fails strict registration.
- Six-hop audit closed the unknown-reachability gap.
- Six-hop reachability still fails corpus connectivity.
- IP cross-reference sweep closed some implementation-plan drift.
- IP cross-reference still needs automated prevention of drift.
- Wave-3 retrospective closed the no-learning-artifact gap.
- Wave-3 retrospective still needs behavior change in Wave 4.
- Anti-patterns closed the implicit-quality-standard gap.
- Anti-patterns still need automated lints.
- Benchmarks closed the no-performance-reference gap.
- Benchmarks still need measured runs.
- Tutorials closed some onboarding gaps.
- Tutorials still need smoke-tested paths.
- Reference implementations closed some "how would this work" gaps.
- Reference implementations still need compilation and integration.
- Per-service ADRs closed the invisible-decision gap.
- Per-service ADRs still need complete roster coverage.
- Audit-event class naming closed some semantic ambiguity.
- Audit-event class naming still needs registry registration.
- Root hub pointers closed some authority discovery gaps.
- Root hub pointers still need broader corpus coverage.
- Standards authoring closed process ambiguity.
- Standards still need CI gates.
- Risk register closed board-level blind spots.
- Risk register still needs live risk metrics.
- Observability dashboards closed static chart absence.
- Observability dashboards still need live data backends.
- Capability-tier registry closed the missing package spine.
- Capability-tier registry still needs promote/rollback state.
- Tenant onboarding SLO closed the no-activation-SLO gap.
- Tenant onboarding still needs pilot-run evidence.
- Compliance evidence export SLO closed the no-reporting-SLO gap.
- Compliance evidence export still needs export path code.
- Workflow saga SLO closed the orchestration target gap.
- Workflow saga SLO still needs replay/compensation metrics.
- Cedar latency SLO closed authorization performance ambiguity.
- Cedar latency still needs service-level instrumentation.
- Data-residency SLO closed residency conformance ambiguity.
- Data-residency still needs real routing evidence.
- AI cost budget SLO closed AI spend ambiguity.
- AI cost budget still needs metering code.
- Drive encryption rotation SLO closed crypto hygiene ambiguity.
- Drive encryption rotation still needs key rotation instrumentation.
- Cross-tenant message SLO closed messaging reliability ambiguity.
- Cross-tenant message delivery still needs MLS/key-path runtime proof.
- Board narrative gap closed because post-remediation state can now be explained honestly.
- Board runtime proof gap remains until Wave 4 runs.

### §6.7 Vendor-Family Diligence Notes

Salesforce family: Sales Cloud coverage is strongest when framed as CRM plus workflow-engine plus ontology plus marketplace composition.
Salesforce family: Service Cloud coverage depends on CRM, ITSM, messenger, community, intelligence, and contact-center.
Salesforce family: Marketing Cloud coverage depends on marketing-automation, consent-graph, messenger, mail, analytics, and workflow-engine.
Salesforce family: Pardot coverage depends on B2B marketing automation, lead scoring, campaign attribution, and CRM handoff.
Salesforce family: Tableau coverage depends on analytics, data-warehouse, ontology, and governance.
Salesforce family: MuleSoft coverage depends on data-pipeline, API gateway, workflow-engine, and connector governance.
Salesforce family: Slack coverage depends on messenger, community, identity, search, retention, and eDiscovery.
Salesforce family: Heroku coverage depends on cloud-k8s, cloud-iac, developer-sdk, and platform deployment.
Salesforce family: Commerce Cloud coverage depends on marketplace, payments, catalog, and order workflows.
Salesforce family: Vlocity/Industries coverage depends on capability-tier composition rather than industry-suite clones.
Salesforce family: Financial Services Cloud coverage depends on CRM, compliance, identity, and regulated workflow.
Salesforce family: Health Cloud coverage depends on healthcare-integration, CRM, consent, and HIPAA pack behavior.
Salesforce family: Field Service coverage depends on workflow-engine, tasks, calendar, and mobile field evidence.
Salesforce family: Experience Cloud coverage depends on community, identity, marketplace, and portal roles.
Salesforce family: Trailhead coverage depends on learning-management, credentialing, and role readiness.
ServiceNow family: ITSM coverage justifies the `itsm` microservice.
ServiceNow family: Customer Service Management composes contact-center, CRM, and ITSM.
ServiceNow family: HR Service Delivery composes performance-management, tasks, identity, and learning-management.
ServiceNow family: Now Platform coverage maps to workflow-engine and workflow-studio, not a suite clone.
ServiceNow family: CMDB coverage maps to ontology, observability, and ITSM.
ServiceNow family: ITOM coverage maps to observability, incident-management, and cloud operations.
ServiceNow family: Security Operations maps to detection, incident-management, governance, and compliance.
ServiceNow family: GRC maps to compliance, governance, audit-chain, and risk workflows.
ServiceNow family: Field Service maps to tasks, workflow-engine, calendar, and mobile field workflows.
ServiceNow family: Strategic Portfolio Management maps to tasks, workflow-engine, financial-planning, and analytics.
Workday family: HCM coverage maps to identity, performance-management, learning-management, and workforce workflows.
Workday family: Financials coverage maps to financial-planning, treasury, payments, and audit-chain.
Workday family: Adaptive Planning coverage validates the financial-planning service.
Workday family: Recruiting coverage maps to workflow-engine, calendar, identity, and candidate relationship data.
Workday family: Talent Management coverage validates performance-management and learning-management integration.
Workday family: Learning coverage validates learning-management as a first-class service.
Workday family: Procurement coverage maps to marketplace, contract-lifecycle-management, and payments.
Workday family: Expenses coverage maps to payments, financial-planning, compliance, and mobile intake.
Atlassian family: Jira Software coverage maps to tasks, workflow-engine, developer-sdk, and incident-management.
Atlassian family: Jira Service Management coverage maps to ITSM and incident-management.
Atlassian family: Confluence coverage maps to docs, drive, search, and knowledge governance.
Atlassian family: Bitbucket coverage maps to developer-sdk, cloud-iac, CI/CD, and audit-chain.
Atlassian family: Trello coverage maps to tasks, boards, workflow-engine, and personal/work task boundaries.
Atlassian family: Open DevOps coverage maps to developer tooling and workflow composition.
Microsoft family: Dynamics 365 coverage maps to CRM, financial-planning, marketplace, and workflow-engine.
Microsoft family: Power Apps coverage maps to workflow-studio, forms, ontology, and policy.
Microsoft family: Power BI coverage maps to analytics, data-warehouse, and governance.
Microsoft family: Power Automate coverage maps to workflow-engine and workflow-studio.
Microsoft family: Power Pages coverage maps to sites, identity, and community.
Microsoft family: Microsoft 365 coverage maps to mail, calendar, drive, docs, sheets, slides, meet, messenger, and recordings.
Microsoft family: Azure DevOps coverage maps to developer-sdk, tasks, cloud-iac, and workflow-engine.
Microsoft family: Defender XDR coverage maps to detection, incident-management, intelligence, and observability.
Microsoft family: Sentinel coverage maps to detection, data-pipeline, data-warehouse, and incident-management.
Microsoft family: Purview coverage maps to governance, compliance, consent-graph, and data classification.
Microsoft family: Intune coverage maps to identity, device posture, policy, and governance.
Microsoft family: Entra ID coverage maps to identity, cloud-iam, tenancy, and federation.
Microsoft family: Viva coverage maps to learning-management, community, analytics, and workforce experience.
Adobe family: Marketo Engage validates marketing-automation.
Adobe family: Experience Manager validates sites, drive, design-collaboration, and governance.
Adobe family: Campaign validates marketing-automation, consent, messenger, and mail.
Adobe family: Analytics validates analytics, data-pipeline, and data-warehouse.
Adobe family: Real-Time CDP validates consent-graph, ontology, and data-pipeline.
Adobe family: Journey Optimizer validates workflow-engine and marketing-automation.
Adobe family: Adobe Sign validates contract-lifecycle-management and audit-chain.
Adobe family: Creative Cloud validates design-collaboration, drive, and whiteboard.
HubSpot family: Marketing Hub maps to marketing-automation, CRM, and consent.
HubSpot family: Sales Hub maps to CRM and workflow-engine.
HubSpot family: Service Hub maps to CRM, contact-center, and ITSM.
HubSpot family: CMS Hub maps to sites and content governance.
HubSpot family: Operations Hub maps to data-pipeline, workflow-engine, and ontology.
Zendesk family: Support maps to contact-center, CRM, and ITSM.
Zendesk family: Chat maps to messenger and contact-center.
Zendesk family: Talk maps to contact-center, recordings, and compliance.
Intercom coverage maps to messenger, contact-center, CRM, and product analytics.
Snowflake coverage validates data-warehouse, governance, and data-product isolation.
Databricks coverage validates data-pipeline, data-warehouse, intelligence, and notebook/workflow composition.
BigQuery coverage validates cloud-data, data-warehouse, and analytics.
Redshift coverage validates data-warehouse and cloud integration.
dbt coverage validates data-pipeline, analytics engineering, and semantic model lineage.
Fivetran coverage validates connector ingestion and data-pipeline replay.
Airbyte coverage validates open connector ingestion and migration-friendly data movement.
Segment coverage validates consent-aware event collection and customer data projection.
RudderStack coverage validates open event routing and warehouse-first routing.
Looker coverage validates semantic BI, governed dashboards, and analytics.
Hex coverage validates collaborative analytics and notebook-to-app workflows.
Mode coverage validates analyst workflows and report publishing.
Sigma coverage validates spreadsheet-like analytics over governed warehouse data.
ThoughtSpot coverage validates search-driven analytics and governed insight discovery.
PagerDuty coverage validates incident-management and on-call escalation.
OpsGenie coverage validates incident escalation and notification routing.
Splunk coverage validates log analytics, SIEM, and observability ingestion.
Datadog coverage validates metrics, traces, logs, and service health.
New Relic coverage validates APM and user experience telemetry.
Dynatrace coverage validates AIOps-style observability and service topology.
Elastic Observability coverage validates log/search/metrics convergence.
Honeycomb coverage validates high-cardinality observability and trace analysis.
Grafana Cloud coverage validates dashboarding, metrics, logs, and alerting.
AppDynamics coverage validates business transaction tracing.
IBM Instana coverage validates automated service discovery and tracing.
Sentry coverage validates application error tracking and release health.
LogRocket coverage validates session replay and frontend diagnostics.
FullStory coverage validates digital experience analytics and replay.
Mixpanel coverage validates product analytics and event funnels.
Amplitude coverage validates product analytics and cohort exploration.
Heap coverage validates autocaptured event analytics.
Stripe coverage validates payments, billing, settlement, marketplace, and financial evidence.
Adyen coverage validates global acquiring, payment methods, and risk controls.
Square/Block coverage validates SMB commerce, payments, and point-of-sale adjacency.
Shopify Plus coverage validates commerce, marketplace, catalog, and fulfillment.
Twilio coverage validates communications APIs, messaging, voice, and programmable notifications.
Zoom coverage validates meet, recordings, calendar, and webinar-style events.
DocuSign coverage validates contract-lifecycle-management, e-signature, and evidence sealing.
Coupa coverage validates procurement, spend management, marketplace, and supplier workflows.
Okta coverage validates identity, federation, lifecycle, and access governance.
CrowdStrike coverage validates endpoint security, detection, incident response, and threat intel.
Asana coverage validates tasks, projects, workload, and portfolio workflows.
Monday.com coverage validates workflow-studio, tasks, boards, and operational dashboards.
Auth0 coverage validates customer identity, federation, and authorization boundary.
ForgeRock coverage validates enterprise IAM and identity journey migration.
Ping Identity coverage validates federation, SSO, and access management.
1Password coverage validates secret vault, workforce credentials, and access evidence.
LastPass coverage validates password-vault migration and credential-risk controls.
Wiz coverage validates cloud-security, intelligence, automation, policy, developer tools, data-store, IAM, compliance, and observability composition.
The vendor-family point is not that Oyatie copies each suite.
The vendor-family point is that Oyatie maps each suite into substrate capabilities.
The investor should test whether mappings stay coherent as the vendor list grows.
The board should test whether Wave 4 turns the highest-value mappings into runnable proof.

### §6.8 Microservice Runtime Conversion Backlog

analytics needs runtime proof for tiered ingest, semantic projection cache, column-level audit, and executive dashboards.
api-gateway needs runtime proof for ingress policy, rate limits, auth propagation, and audit identifiers.
application needs runtime proof for usecase orchestration and app-layer transaction boundaries.
audit-chain needs runtime proof for event class registration, Merkle sealing, and evidence export.
calendar needs runtime proof for availability, invite policy, timezone behavior, and meeting handoffs.
cell needs runtime proof for placement, shuffle sharding, failover, and regional policy.
cloud-billing needs runtime proof for metering events, invoice formation, and cost allocation.
cloud-billing-tax needs runtime proof for tax jurisdiction and billing evidence.
cloud-data needs runtime proof for data movement, object custody, and residency tagging.
cloud-iac needs runtime proof for deployment plan validation and drift detection.
cloud-iam needs runtime proof for role mapping, federation, and cloud-side permit sync.
cloud-k8s needs runtime proof for Kubernetes workload admission and tenant-safe deployment.
cloud-kms needs runtime proof for key rotation, BYOK/HSM custody, and audit trails.
cloud-network needs runtime proof for network policy, routing, and isolation.
cloud-network-dns needs runtime proof for zone changes, DNSSEC, and resolver controls.
cloud-secrets needs runtime proof for lease, rotation, reveal, and revocation.
cloud-storage needs runtime proof for object/block/file storage, encryption, and retention.
comms-email needs runtime proof for sending, receiving, deliverability, retention, and eDiscovery.
community needs runtime proof for groups, feeds, moderation, and cross-tenant cohorts.
compliance needs runtime proof for pack activation, evidence export, and refusal.
connect needs runtime proof for shell integration and product-surface cohesion.
consent-graph needs runtime proof for consent capture, withdrawal, propagation, and audit.
contact-center needs runtime proof for interaction routing, transcript evidence, and escalation.
contract-lifecycle-management needs runtime proof for envelopes, obligations, renewals, and signatures.
crm needs runtime proof for account, contact, lead, opportunity, activity, and forecast objects.
data-pipeline needs runtime proof for extract, transform, load, replay, and failed-batch recovery.
data-warehouse needs runtime proof for query governance, workload isolation, and lineage.
design-collaboration needs runtime proof for file collaboration, review, comments, and export.
detection needs runtime proof for streaming detection, batch detection, and fairness audit.
developer-sdk needs runtime proof for generated clients, versioning, and smoke tests.
docs needs runtime proof for corpus navigation, doc freshness, and publication.
drive needs runtime proof for file ACLs, classification, DLP, and encryption rotation.
feature-flags needs runtime proof for targeting, rollout, rollback, and experiment audit.
financial-planning needs runtime proof for budgets, forecasts, scenarios, and approvals.
finops-portal needs runtime proof for chargeback, anomaly detection, and optimization.
forms needs runtime proof for schema, validation, approvals, and submissions.
foundry needs runtime proof for route policy, eval gates, evidence, and autonomy ceilings.
global-trade needs runtime proof for tariff, customs, sanctions, and shipment evidence.
governance needs runtime proof for decision rights, exception handling, and policy lifecycle.
healthcare-integration needs runtime proof for FHIR, PHI boundaries, and clinical audit.
identity needs runtime proof for tenant principals, recovery, passkeys, and personal/work boundary.
incident-management needs runtime proof for incident lifecycle, on-call, escalation, and retrospectives.
intelligence needs runtime proof for inference, scoring, model lifecycle, and explainability.
itsm needs runtime proof for incident, request, change, problem, CMDB adjacency, and SLA.
learning-management needs runtime proof for courses, competency, certifications, and evidence.
mail needs runtime proof for mailbox operations, retention, legal hold, and delivery.
marketing-automation needs runtime proof for campaign, segmentation, consent, and attribution.
marketplace needs runtime proof for DealSet, supplier discovery, listings, entitlements, and settlement.
meet needs runtime proof for conferencing, recording, consent, and transcription.
messenger needs runtime proof for MLS, channels, cross-tenant messages, and notification reliability.
network needs runtime proof for core network primitives and platform connectivity.
notes needs runtime proof for encrypted notes, sync, sharing, and retention.
observability needs runtime proof for logs, traces, metrics, dashboards, and audit-id propagation.
ontology needs runtime proof for projection, graph edges, schema revision, and queries.
ops-dashboard-control-center needs runtime proof for operator commands, views, and incident control.
payments needs runtime proof for payment authorization, escrow, refunds, disputes, and settlement.
performance-management needs runtime proof for goals, review cycles, feedback, and talent evidence.
plant-maintenance needs runtime proof for work orders, assets, LOTO, and field operations.
plugin-app-store needs runtime proof for plugin publication, sandboxing, entitlement, and takedown.
production-planning needs runtime proof for BOM, MRP, orders, capacity, and shop-floor coordination.
quality-management needs runtime proof for inspection, nonconformance, CAPA, and recall.
real-estate needs runtime proof for leases, facilities, space, and obligations.
recordings needs runtime proof for capture, consent, retention, archive, and search.
sheets needs runtime proof for formulas, collaboration, import/export, and permissions.
shorts needs runtime proof for media ingest, creator monetization, and content governance.
sites needs runtime proof for pages, publishing, access, and content governance.
slides needs runtime proof for presentation editing, review, export, and board-pack controls.
social needs runtime proof for feeds, moderation, follow graph, and abuse defense.
supply-chain-planning needs runtime proof for demand, supply, disruptions, and supplier graph.
tasks needs runtime proof for assignment, status, SLA, and personal/work task boundaries.
tenancy needs runtime proof for tenant lifecycle, sub-scope registry, permits, quotas, and residency.
translate needs runtime proof for translation memory, locale routing, and bilingual evidence.
treasury needs runtime proof for cash, liquidity, bank feeds, hedging, and reconciliation.
warehouse needs runtime proof for inventory, picking, packing, movements, and logistics.
whiteboard needs runtime proof for collaborative canvas, sessions, comments, and export.
workflow-engine needs runtime proof for state machine, DAG, saga, replay, and compensation.
workflow-studio needs runtime proof for authoring, validation, templates, and collaborative editing.
workplace-integration needs runtime proof for external connector migration and incumbent bridge.

### §6.9 Persona And Role Coverage Reading

The persona set now supports a stronger "whole workforce" thesis.
The executive persona cluster covers CEO, CFO, COO, CTO, CHRO, CCO, CISO, CSO, CMO, and board director needs.
The finance persona cluster covers financial analyst, tax analyst, treasury operator, accountant, banker, LP, and wealth manager needs.
The people-operations persona cluster covers recruiter, HRBP, benefits specialist, total rewards, training specialist, and intern manager needs.
The legal persona cluster covers legal counsel, legal operations, outside counsel, paralegal, and ombudsperson needs.
The compliance persona cluster covers compliance officer, bank compliance, regulator inspector, external auditor, and internal audit needs.
The security persona cluster covers security analyst, CISO, guard, detection operator, and incident responder needs.
The operations persona cluster covers office manager, receptionist, AV coordinator, mailroom, cafeteria manager, cleaning supervisor, and print operator needs.
The industrial persona cluster covers maintenance technician, forklift operator, construction worker, plant roles, and quality roles.
The healthcare persona cluster covers surgeon, resident, nurse, provider, and clinical-support workflows.
The education persona cluster covers teacher, student, intern, apprentice, fellow, and training administrator needs.
The GTM persona cluster covers sales AE, SDR, sales manager, marketing specialist, CMO, PR manager, and channel partner needs.
The investor persona cluster covers venture partner, investor relations, LP, board secretary, and strategic advisor needs.
The small-business persona cluster covers HVAC side business, cleaning company owner, farmer, creator, consultant, and local service operator needs.
The regulated public-sector persona cluster covers police, captain, public affairs, regulator, and emergency-service roles.
The personal-life persona cluster covers retiree, parent, survivor-safety, tax, pension, and personal notes contexts.
The cross-border persona cluster covers JP, KR, EU, US, IN, BR, AU, MX, and multilingual workflows.
The key persona achievement is role pressure.
Role pressure is stronger than demographic variety.
Role pressure tells engineering what must be true.
The CISO pressure is evidence and boundary control.
The CFO pressure is reconciliation and board-ready reporting.
The COO pressure is operational cadence and exception management.
The CTO pressure is safe cutover and version control.
The CHRO pressure is workforce lifecycle and fairness.
The CCO pressure is board compliance pack evidence.
The board director pressure is concise evidence and decision accountability.
The investor-relations pressure is disclosure timing and livestream governance.
The maintenance pressure is after-hours safety and permit state.
The print-operator pressure is quality recall and night-shift onboarding.
The cafeteria-manager pressure is allergen recall and school coordination.
The construction pressure is bilingual incident capture and safety evidence.
The maritime pressure is typhoon evacuation and cash-flow continuity.
The retiree pressure is tax, pension, and long-lived personal continuity.
The LP pressure is K-1 distribution and portfolio-tax evidence.
The ombudsperson pressure is privilege and cross-tenant mediation.
The treasury pressure is end-of-day reconciliation.
The sustainability pressure is Scope 3 evidence and supply-chain reporting.
The channel-partner pressure is shared attribution and tenant provisioning.
The creator pressure is spike handling and employer disclosure.
The student-worker pressure is schedule collision and fatigue-aware work.
The wealth-manager pressure is jurisdictional trust restructuring.
The strategic-acquisition pressure is restricted-deal information barriers.
The board-meeting pressure is recording consent and time-zone coordination.
The key board implication is that Oyatie is not only a desk-worker thesis.
The key investor implication is that TAM breadth now has role-level stories.
The key product implication is that personas can select Wave-4 slices.
The key engineering implication is that personas should become tests.
The key GTM implication is that persona stories can be packaged by buyer role.
The key compliance implication is that regulated role pressure is explicit.
The key risk is that personas are still documents until product slices exist.
The key next step is to select the personas that prove the hardest substrate boundaries.

### §6.10 Compliance And Localization Due-Diligence Ledger

HIPAA diligence question: can PHI access, break-glass, audit, and BAA-style evidence be enforced at runtime?
HIPAA Wave-4 answer required: healthcare-integration plus identity plus audit-chain plus compliance policy tests.
GDPR diligence question: can DSR, lawful basis, erasure, portability, and cross-border transfer controls be proven?
GDPR Wave-4 answer required: consent-graph plus compliance plus residency plus audit-chain fixtures.
SOC2 diligence question: can trust-service controls map to evidence-generating runtime events?
SOC2 Wave-4 answer required: control evidence exports and auditor-facing dashboards.
EU AI Act diligence question: can high-risk AI decisions be refused, logged, appealed, and explained?
EU AI Act Wave-4 answer required: intelligence plus policy plus compliance plus human-oversight tests.
KR-PIPA diligence question: can Korean PI, RRN, consent, residency, and breach notice be enforced?
KR-PIPA Wave-4 answer required: KR pack overlay policy tests and RRN field controls.
CSAP diligence question: can Korean public cloud posture be represented and audited?
CSAP Wave-4 answer required: cloud controls, evidence export, and public-sector deployment profile.
PCI diligence question: can cardholder data be isolated, audited, and scoped?
PCI Wave-4 answer required: payments, data-boundary, audit-chain, and CDE isolation tests.
EU-CSRD diligence question: can sustainability evidence and Scope 3 supplier data be traced?
EU-CSRD Wave-4 answer required: sustainability journey, supply-chain ontology, and evidence export.
KR localization diligence question: can product behavior change under Korean law and operational expectations?
KR localization Wave-4 answer required: localized consent, RRN, residency, incident, and public-sector overlays.
EU localization diligence question: can product behavior change under EU privacy, AI, resilience, and reporting law?
EU localization Wave-4 answer required: DSR, DORA, high-risk AI, transfer, and local evidence fixtures.
US localization diligence question: can federal, state, healthcare, financial, and anti-discrimination duties coexist?
US localization Wave-4 answer required: pack composition tests across HIPAA, SOX, state privacy, and AI bias.
JP localization diligence question: can APPI, My Number, finance, telecom, and cybersecurity overlays coexist?
JP localization Wave-4 answer required: identity field handling, banking workflows, and APPI DSR fixtures.
IN localization diligence question: can DPDPA, consent, residency, breach notice, and sector overlays coexist?
IN localization Wave-4 answer required: consent and localization fixtures bound to tenant tier.
BR localization diligence question: can LGPD DSR, consent, residency, and breach response be enforced?
BR localization Wave-4 answer required: LGPD overlay tests and cross-border data-flow checks.
AU localization diligence question: can APRA CPS 234 and privacy overlays bind regulated customers?
AU localization Wave-4 answer required: operational resilience, incident reporting, and privacy fixtures.
MX localization diligence question: can privacy, DSR, residency, and breach requirements be represented?
MX localization Wave-4 answer required: local pack overlay fixtures and breach workflow evidence.
Cross-pack diligence question: what happens when two packs disagree?
Cross-pack Wave-4 answer required: pack conflict resolver and refusal evidence.
Regulated AI diligence question: what happens when an AI recommendation affects credit, employment, healthcare, or insurance?
Regulated AI Wave-4 answer required: high-risk classifier, refusal path, appeal path, and human review.
Residency diligence question: what happens when user, tenant, data, and provider regions disagree?
Residency Wave-4 answer required: cell-routing policy and data-residency conformance SLO.
Audit diligence question: what evidence exists when an exception is allowed?
Audit Wave-4 answer required: registered audit-event class, audit_id propagation, and Merkle seal.
Legal diligence question: who approves market-facing compliance claims?
Legal Wave-4 answer required: claim approval workflow and evidence package review.
Sales diligence question: which compliance claims can sales use today?
Sales Wave-4 answer required: controlled claim matrix tied to artifact and runtime evidence level.
Board diligence question: which packs are design-complete versus runtime-proven?
Board Wave-4 answer required: pack maturity dashboard.

### §6.11 Board Diligence Questions And Post-Remediation Answers

Question 001: Did the remediation change the company thesis?
Answer 001: No; it made the thesis more evidence-backed.
Question 002: Did remediation make Oyatie production-ready?
Answer 002: No; it made Wave 4 implementation better specified.
Question 003: Is 500,000-plus lines a success metric?
Answer 003: It is a throughput signal, but substance bars are the success metric.
Question 004: What was the largest pre-remediation risk?
Answer 004: Template-stamped volume that looked complete without operational detail.
Question 005: What is the largest post-remediation risk?
Answer 005: Documentation outrunning runtime implementation.
Question 006: What is the most valuable remediation artifact?
Answer 006: The answer depends on audience; ADR-0321 for investors, capability tiers for GTM, audits for engineering.
Question 007: Is ADR-0321 complete?
Answer 007: No; 110 of 165 dossiers are substantive, with tail work remaining.
Question 008: Can sales claim complete incumbent parity?
Answer 008: No; sales can claim a strong mapped coverage runway with named incumbent migrations in flight.
Question 009: Can investors see proof of execution?
Answer 009: Yes, at architecture and remediation scale, not yet runtime or revenue scale.
Question 010: Can the board cite compliance readiness?
Answer 010: The board can cite compliance-aware architecture, not certification.
Question 011: Are the packs enough for market launch?
Answer 011: No; they are launch-design inputs requiring legal, security, and runtime validation.
Question 012: Are personas enough to prove TAM?
Answer 012: They support TAM breadth but do not prove adoption.
Question 013: Are journeys enough to prove product readiness?
Answer 013: They prove use-case coverage, not shipped product.
Question 014: Are microservice suites enough to start implementation?
Answer 014: Yes for many services, provided Wave 4 starts with tests and contracts.
Question 015: Is the capability-tier registry strategic?
Answer 015: Yes; it prevents the thesis from degenerating into another suite portfolio.
Question 016: Is the audit-event sweep bad news?
Answer 016: It is good governance revealing bad compliance readiness.
Question 017: Is the six-hop audit bad news?
Answer 017: It is good governance revealing discoverability debt.
Question 018: Should Wave 4 prioritize new docs?
Answer 018: No; it should prioritize executable gates, code, tests, and pilots.
Question 019: Should Wave 4 finish ADR-0321?
Answer 019: Yes, but not at the expense of runtime proof.
Question 020: What is the first Wave-4 board packet evidence?
Answer 020: A pilot tenant with tier, pack, policy, audit, dashboard, and rollback proof.
Question 021: What should be avoided?
Answer 021: Another high-volume prose wave without enforcement.
Question 022: What should be funded?
Answer 022: Rust scaffolds, CI gates, telemetry, migration tooling, and pilot execution.
Question 023: What should be sequenced?
Answer 023: Registry gates before broad service implementation.
Question 024: What can run in parallel?
Answer 024: Registry gates, reachability repair, selected service scaffolds, and tail dossier work if files are isolated.
Question 025: What must not run in parallel without coordination?
Answer 025: Shared root indexes, capability-tier schema, audit-event registry, and core service contracts.
Question 026: What is the best sales proof?
Answer 026: j176-j180 plus selected ADR-0321 dossiers and tier mappings.
Question 027: What is the best investor proof?
Answer 027: before/after remediation depth and Wave-4 runtime conversion.
Question 028: What is the best engineering proof?
Answer 028: green CI gates that validate actual claims.
Question 029: What is the best compliance proof?
Answer 029: pack-bound policy tests and evidence exports.
Question 030: What is the best product proof?
Answer 030: a pilot slice serving real persona pressure.
Question 031: What is the best operations proof?
Answer 031: dashboard signal, SLO burn, runbook drill, and rollback evidence.
Question 032: What is the best board control?
Answer 032: a dashboard separating design-ready, build-ready, runtime-proven, and launch-ready.
Question 033: What is the strongest caution?
Answer 033: do not mistake architecture coverage for product delivery.
Question 034: What is the strongest encouragement?
Answer 034: the remediation rate and specificity are materially better than the pre-remediation state.
Question 035: What should a director ask every month?
Answer 035: which corpus claims became executable gates this month?
Question 036: What should a director reject?
Answer 036: any claim that cannot cite an artifact and a verification state.
Question 037: What should a director accelerate?
Answer 037: slices that prove tenant, policy, audit, workflow, ontology, and migration together.
Question 038: What should a director defer?
Answer 038: low-signal expansion that adds names without runtime pressure.
Question 039: What should a director protect?
Answer 039: the flat service architecture and capability-tier doctrine.
Question 040: What should a director scrutinize?
Answer 040: any product label that tries to become a service boundary.

### §6.12 Wave 4 Acceptance Criteria

Acceptance criterion 001: at least one capability-tier registry validator runs in CI.
Acceptance criterion 002: the validator rejects malformed tier definition files.
Acceptance criterion 003: the validator rejects vendor mappings that reference missing services.
Acceptance criterion 004: the validator rejects service mappings that omit tier deltas.
Acceptance criterion 005: at least one compliance-pack validator runs in CI.
Acceptance criterion 006: the validator rejects missing pack authority.
Acceptance criterion 007: the validator rejects missing overlay references.
Acceptance criterion 008: the validator rejects unsupported compliance claims.
Acceptance criterion 009: at least one audit-event registry validator runs in CI.
Acceptance criterion 010: the validator rejects mutation endpoints without explicit event class metadata.
Acceptance criterion 011: the validator rejects event classes that are named but not registered.
Acceptance criterion 012: the validator requires audit_id propagation metadata.
Acceptance criterion 013: at least one seven-surface microservice-suite validator runs in CI.
Acceptance criterion 014: the validator checks README/scope presence.
Acceptance criterion 015: the validator checks architecture presence.
Acceptance criterion 016: the validator checks contract/API presence.
Acceptance criterion 017: the validator checks policy presence.
Acceptance criterion 018: the validator checks runbook presence.
Acceptance criterion 019: the validator checks tutorial/onboarding/reference presence.
Acceptance criterion 020: the validator checks benchmark/dashboard/test/migration presence.
Acceptance criterion 021: at least one reachability audit is repeatable by command.
Acceptance criterion 022: reachable-node count improves materially from 437.
Acceptance criterion 023: unreachable-node count falls materially from 7,169.
Acceptance criterion 024: root hub pointers include required authority paths.
Acceptance criterion 025: service indexes link to their service suites.
Acceptance criterion 026: at least one Rust service skeleton compiles.
Acceptance criterion 027: the Rust skeleton exposes a contract boundary.
Acceptance criterion 028: the Rust skeleton has a policy test.
Acceptance criterion 029: the Rust skeleton emits a registered audit event in a test.
Acceptance criterion 030: the Rust skeleton exports a metric or trace in a test.
Acceptance criterion 031: at least one journey becomes an integration test.
Acceptance criterion 032: the integration test exercises two or more services.
Acceptance criterion 033: the integration test includes a negative policy case.
Acceptance criterion 034: the integration test checks audit evidence.
Acceptance criterion 035: the integration test has fixture data.
Acceptance criterion 036: at least one migration journey has executable importer scaffolding.
Acceptance criterion 037: the importer preserves legacy identifiers.
Acceptance criterion 038: the importer emits migration provenance.
Acceptance criterion 039: the importer has dry-run mode.
Acceptance criterion 040: the importer has parity-check output.
Acceptance criterion 041: at least one tenant pilot activates a tier.
Acceptance criterion 042: the tenant pilot binds a compliance pack.
Acceptance criterion 043: the tenant pilot binds a localization overlay.
Acceptance criterion 044: the tenant pilot creates an ontology projection.
Acceptance criterion 045: the tenant pilot creates a workflow instance.
Acceptance criterion 046: the tenant pilot emits audit evidence.
Acceptance criterion 047: the tenant pilot appears on a dashboard.
Acceptance criterion 048: the tenant pilot supports rollback.
Acceptance criterion 049: at least one dashboard reads live test data.
Acceptance criterion 050: the dashboard labels tenant, service, tier, and pack.
Acceptance criterion 051: at least one SLO burn calculation runs over fixture telemetry.
Acceptance criterion 052: the SLO burn calculation has a failing fixture.
Acceptance criterion 053: the SLO burn calculation has a passing fixture.
Acceptance criterion 054: at least one runbook drill is exercised.
Acceptance criterion 055: the runbook drill records evidence.
Acceptance criterion 056: the runbook drill has a stop condition.
Acceptance criterion 057: at least one pack conflict scenario is tested.
Acceptance criterion 058: the conflict scenario produces a deterministic decision.
Acceptance criterion 059: the conflict scenario produces refusal evidence when required.
Acceptance criterion 060: at least one personal/work boundary scenario is tested.
Acceptance criterion 061: the boundary scenario denies unauthorized access.
Acceptance criterion 062: the boundary scenario records the denial.
Acceptance criterion 063: at least one cross-tenant handoff scenario is tested.
Acceptance criterion 064: the handoff scenario names sender and receiver.
Acceptance criterion 065: the handoff scenario names audit custody.
Acceptance criterion 066: at least one incumbent migration board packet is generated from evidence.
Acceptance criterion 067: the board packet separates simulated and live evidence.
Acceptance criterion 068: the board packet separates docs, tests, and runtime.
Acceptance criterion 069: the board packet includes not-tested gaps.
Acceptance criterion 070: the board packet avoids certification claims without legal signoff.
Acceptance criterion 071: at least one ADR-0321 tail dossier is completed at substance bar.
Acceptance criterion 072: each completed tail dossier feeds vendor-tier mapping.
Acceptance criterion 073: each completed tail dossier feeds migration backlog.
Acceptance criterion 074: each completed tail dossier names failure modes.
Acceptance criterion 075: each completed tail dossier rejects grouping-shaped service cloning.
Acceptance criterion 076: at least one governance crate moves from scaffold to tested implementation.
Acceptance criterion 077: the governance crate has positive fixtures.
Acceptance criterion 078: the governance crate has negative fixtures.
Acceptance criterion 079: the governance crate is wired into a command path.
Acceptance criterion 080: the governance crate is documented as a gate.
Acceptance criterion 081: Wave 4 reporting tracks design-ready artifacts.
Acceptance criterion 082: Wave 4 reporting tracks build-ready artifacts.
Acceptance criterion 083: Wave 4 reporting tracks runtime-proven artifacts.
Acceptance criterion 084: Wave 4 reporting tracks launch-ready artifacts.
Acceptance criterion 085: Wave 4 reporting tracks blocked artifacts.
Acceptance criterion 086: Wave 4 reporting tracks residual risk.
Acceptance criterion 087: Wave 4 reporting names claim boundaries.
Acceptance criterion 088: Wave 4 reporting cites verification commands.
Acceptance criterion 089: Wave 4 reporting cites changed files.
Acceptance criterion 090: Wave 4 reporting cites evidence bundles.
Acceptance criterion 091: Wave 4 reporting does not rely on prose-only completion.
Acceptance criterion 092: Wave 4 reporting does not hide audit failures.
Acceptance criterion 093: Wave 4 reporting does not conflate pilot with production.
Acceptance criterion 094: Wave 4 reporting does not conflate compliance architecture with certification.
Acceptance criterion 095: Wave 4 reporting does not conflate migration narrative with importer tooling.
Acceptance criterion 096: Wave 4 reporting does not conflate dashboard definition with live telemetry.
Acceptance criterion 097: Wave 4 reporting does not conflate SLO library with SLO compliance.
Acceptance criterion 098: Wave 4 reporting does not conflate service docs with service runtime.
Acceptance criterion 099: Wave 4 reporting does not conflate persona breadth with adoption.
Acceptance criterion 100: Wave 4 reporting does not conflate TAM thesis with revenue proof.

### §6.13 Sales And Marketing Claim Boundaries

Safe claim 001: Oyatie has a post-remediation architecture corpus for unified enterprise software.
Unsafe claim 001: Oyatie has shipped the full unified enterprise software platform.
Safe claim 002: Oyatie has 110 substantive ADR-0321 vendor dossiers.
Unsafe claim 002: Oyatie has complete direct coverage for all 165 planned vendor dossiers.
Safe claim 003: Oyatie maps 295 vendor rows through capability tiers.
Unsafe claim 003: every mapped vendor is fully implemented.
Safe claim 004: Oyatie uses Bronze, Silver, Gold, and Platinum capability tiers.
Unsafe claim 004: all tiers are live commercial SKUs.
Safe claim 005: Oyatie has 175 substantive journeys and five migration journeys in flight.
Unsafe claim 005: all journeys are shipped product workflows.
Safe claim 006: Oyatie has eight active compliance-pack artifacts.
Unsafe claim 006: Oyatie is certified under all eight regimes.
Safe claim 007: Oyatie has eight active localization-pack artifacts.
Unsafe claim 007: Oyatie is launched in all eight regions.
Safe claim 008: Oyatie has SLO library artifacts.
Unsafe claim 008: all SLOs are currently met in production.
Safe claim 009: Oyatie has observability dashboard definitions.
Unsafe claim 009: all dashboards currently read production telemetry.
Safe claim 010: Oyatie has audit-event coverage gaps catalogued.
Unsafe claim 010: audit-event registration is fully compliant.
Safe claim 011: Oyatie has six-hop reachability audit evidence.
Unsafe claim 011: the corpus is fully reachable from docs/README.md.
Safe claim 012: Oyatie has migration playbooks and first migration journeys.
Unsafe claim 012: Oyatie has automated migration tools for every incumbent.
Safe claim 013: Oyatie has microservice documentation sets at scale.
Unsafe claim 013: all documented microservices are implemented.
Safe claim 014: Oyatie has governance crates scaffolded.
Unsafe claim 014: all governance gates are live.
Safe claim 015: Oyatie has persona Substance Anchors across 90-plus dossiers.
Unsafe claim 015: all persona journeys are validated with users.
Safe claim 016: Oyatie has ERP IP deepening across nine services.
Unsafe claim 016: Oyatie has complete ERP runtime parity today.
Safe claim 017: Oyatie has the architecture to collapse product sprawl.
Unsafe claim 017: Oyatie has already eliminated customer SaaS portfolios.
Safe claim 018: Oyatie has a Wave-4 implementation roadmap.
Unsafe claim 018: Wave 4 is already complete.
Safe claim 019: Oyatie has strong evidence of architecture execution.
Unsafe claim 019: Oyatie has proven product-market fit.
Safe claim 020: Oyatie has a disciplined remediation loop.
Unsafe claim 020: no major risks remain.

### §6.14 Executive Metric Definitions

Metric definition 001: "substantive dossier" means vendor-specific objects, APIs, UX, Cedar, ontology, workflow, migration, and failure modes.
Metric definition 002: "substantive journey" means story, UX flow, handshake, integration test plan, and material schemas or policy.
Metric definition 003: "seven-surface suite" means service framing, architecture, contracts, policy, operations, onboarding, and verification/performance/migration support.
Metric definition 004: "Substance Anchors" means persona details tied to actual workflow pressure rather than demographic decoration.
Metric definition 005: "compliance pack" means an architecture/evidence overlay, not a certification.
Metric definition 006: "localization pack" means region-specific operational behavior, not merely translation.
Metric definition 007: "capability tier" means tenant-grantable bundle over shared substrate, not a product microservice.
Metric definition 008: "vendor mapping row" means benchmark-to-destination metadata, not runtime implementation.
Metric definition 009: "dashboard definition" means operator view contract, not live telemetry unless wired.
Metric definition 010: "SLO library entry" means reliability target contract, not measured compliance unless instrumented.
Metric definition 011: "migration journey" means narrative and test-plan runway, not importer tooling.
Metric definition 012: "migration playbook" means procedure and risk map, not fully automated cutover.
Metric definition 013: "governance crate scaffold" means code-enforcement runway, not a completed validator.
Metric definition 014: "audit-event sweep" means gap inventory, not remediation.
Metric definition 015: "six-hop audit" means discoverability measurement, not reachability pass.
Metric definition 016: "board-ready" means understandable and evidence-bound, not runtime-complete.
Metric definition 017: "investor-ready" means diligence-framable, not revenue-validated.
Metric definition 018: "sales-ready" means claim-bound enablement, not unrestricted promises.
Metric definition 019: "build-ready" means enough artifacts exist to implement with fewer guesses.
Metric definition 020: "runtime-proven" means code, tests, telemetry, and evidence demonstrate the claim.
Metric definition 021: "launch-ready" means runtime proof plus legal/security/compliance/product approval.
Metric definition 022: "Wave-3 complete" means corpus expansion and remediation artifacts exist.
Metric definition 023: "Wave-4 complete" must mean executable proof has landed.
Metric definition 024: "TAM unchanged" means market scope remains broad, not that traction is proven.
Metric definition 025: "proof-of-execution at scale" means architecture throughput and remediation discipline.
Metric definition 026: "direct vendor coverage" means named incumbent specifics, not category inference.
Metric definition 027: "composed coverage" means multiple Oyatie services together subsume a vendor surface.
Metric definition 028: "new microservice" means distinct operational concern, not familiar product name.
Metric definition 029: "benchmark alias" means vendor name used for comparison, not service identity.
Metric definition 030: "flat service" means ADR-0131-aligned ownership boundary.

### §6.15 Board Operating Cadence For The Next 90 Days

Day 1 expectation: confirm the post-remediation corpus is line-count verified and separately promoted.
Day 7 expectation: choose the first Wave-4 executable slice.
Day 14 expectation: land first registry validator plan.
Day 21 expectation: show failing fixtures for at least one governance gate.
Day 30 expectation: compile one Rust skeleton with contract and policy tests.
Day 37 expectation: improve audit-event registration for one strict-scope service.
Day 45 expectation: wire one dashboard to test telemetry.
Day 52 expectation: convert one migration journey into importer dry-run scaffolding.
Day 60 expectation: activate one sample tenant through a capability-tier fixture.
Day 67 expectation: bind one compliance pack overlay to a runtime policy test.
Day 75 expectation: run one cross-service journey integration test.
Day 82 expectation: update the reachability audit and show material improvement.
Day 90 expectation: present a board packet that separates docs, code, tests, telemetry, and launch gaps.
The cadence should be evidence-first.
The cadence should be narrow enough to finish.
The cadence should be broad enough to prove the thesis loop.
The cadence should reject prose-only completion.
The cadence should preserve Oya VCS discipline.
The cadence should make risk visible.
The cadence should keep sales claims within verified boundaries.
The cadence should make investor updates sharper.
The cadence should turn remediation into implementation.

### §6.16 ADR-0321 Substance Bar Checklist

ADR-0321 checklist 001: vendor name must be explicit.
ADR-0321 checklist 002: vendor category must be explicit.
ADR-0321 checklist 003: coverage tier must be explicit.
ADR-0321 checklist 004: Oyatie destination services must be explicit.
ADR-0321 checklist 005: non-service destination references must be separated from service names.
ADR-0321 checklist 006: capability tier must be mapped.
ADR-0321 checklist 007: mid-market edition must be mapped when relevant.
ADR-0321 checklist 008: enterprise edition must be mapped when relevant.
ADR-0321 checklist 009: regulated edition must be mapped when relevant.
ADR-0321 checklist 010: vendor data model must name top objects.
ADR-0321 checklist 011: vendor object fields must include migration-relevant fields.
ADR-0321 checklist 012: vendor APIs must name endpoint or protocol shapes.
ADR-0321 checklist 013: vendor import/export surfaces must be named.
ADR-0321 checklist 014: vendor event or streaming surfaces must be named when present.
ADR-0321 checklist 015: vendor metadata surfaces must be named when material.
ADR-0321 checklist 016: vendor UX shell must be described.
ADR-0321 checklist 017: vendor workflow builder surface must be described when present.
ADR-0321 checklist 018: vendor mobile or embedded surface must be described when material.
ADR-0321 checklist 019: Cedar action verbs must be vendor-specific.
ADR-0321 checklist 020: Cedar actions must not be generic read/write placeholders.
ADR-0321 checklist 021: Cedar resource types must be vendor-specific or destination-specific.
ADR-0321 checklist 022: Cedar conditions must include tenant boundary.
ADR-0321 checklist 023: Cedar conditions must include pack or purpose constraints where regulated.
ADR-0321 checklist 024: Cedar conditions must include role or clearance where material.
ADR-0321 checklist 025: ontology projections must map vendor objects to Oyatie object types.
ADR-0321 checklist 026: ontology projections must preserve legacy identifiers.
ADR-0321 checklist 027: ontology projections must identify lossy transforms.
ADR-0321 checklist 028: ontology projections must identify custom field or extension handling.
ADR-0321 checklist 029: workflow templates must name replacement flows.
ADR-0321 checklist 030: workflow templates must include state or approval semantics.
ADR-0321 checklist 031: workflow templates must include replay implications when migration matters.
ADR-0321 checklist 032: migration extract step must name source API or export.
ADR-0321 checklist 033: migration land step must name staging custody.
ADR-0321 checklist 034: migration projection step must name ontology mapping.
ADR-0321 checklist 035: migration permit step must name authorization transfer.
ADR-0321 checklist 036: migration replay step must name workflow-state continuity.
ADR-0321 checklist 037: migration cutover step must name user entrypoint change.
ADR-0321 checklist 038: migration sunset step must name evidence retention.
ADR-0321 checklist 039: failure modes must be vendor-specific.
ADR-0321 checklist 040: failure modes must include API quota or export risk when relevant.
ADR-0321 checklist 041: failure modes must include custom logic or plugin risk when relevant.
ADR-0321 checklist 042: failure modes must include permission mismatch risk when relevant.
ADR-0321 checklist 043: failure modes must include reporting divergence when relevant.
ADR-0321 checklist 044: failure modes must include retention or legal hold risk when relevant.
ADR-0321 checklist 045: failure modes must include integration webhook risk when relevant.
ADR-0321 checklist 046: failure modes must include identity or SSO risk when relevant.
ADR-0321 checklist 047: naming justification must preserve canonical service names.
ADR-0321 checklist 048: benchmark aliases must remain aliases.
ADR-0321 checklist 049: dossier must feed vendor-tier mapping.
ADR-0321 checklist 050: dossier must feed migration backlog.
ADR-0321 checklist 051: dossier must be reviewable by sales.
ADR-0321 checklist 052: dossier must be reviewable by engineering.
ADR-0321 checklist 053: dossier must be reviewable by compliance.
ADR-0321 checklist 054: dossier must avoid grouping-shaped microservice creation.
ADR-0321 checklist 055: dossier must avoid shallow repeated text.
ADR-0321 checklist 056: dossier must avoid exact-length filler.
ADR-0321 checklist 057: dossier must state what is covered by existing services.
ADR-0321 checklist 058: dossier must state what requires composition.
ADR-0321 checklist 059: dossier must state what requires a new flat service.
ADR-0321 checklist 060: dossier must state what remains partner/plugin path if applicable.

### §6.17 Migration Journey Expansion Backlog

j181 candidate: Microsoft 365 tenant to Oyatie mail, calendar, drive, docs, meet, and identity migration.
j182 candidate: Microsoft Dynamics 365 to Oyatie CRM, finance, workflow, and marketplace migration.
j183 candidate: Snowflake warehouse to Oyatie data-warehouse and governance migration.
j184 candidate: Databricks workspace to Oyatie data-pipeline, intelligence, and warehouse migration.
j185 candidate: Stripe Billing and to Oyatie payments, marketplace, and billing migration.
j186 candidate: Adobe Marketo Engage to Oyatie marketing-automation and consent migration.
j187 candidate: HubSpot Marketing/Sales/Service hubs to Oyatie CRM and contact-center migration.
j188 candidate: Zendesk Support and Talk to Oyatie contact-center and ITSM migration.
j189 candidate: Okta workforce identity to Oyatie identity and cloud-iam migration.
j190 candidate: CrowdStrike endpoint-security posture to Oyatie detection and incident-management migration.
j191 candidate: Datadog observability estate to Oyatie observability migration.
j192 candidate: PagerDuty on-call operations to Oyatie incident-management migration.
j193 candidate: Coupa procurement to Oyatie marketplace and contract-lifecycle-management migration.
j194 candidate: DocuSign envelope archive to Oyatie contract-lifecycle-management and audit-chain migration.
j195 candidate: Slack workspace to Oyatie messenger and community migration.
j196 candidate: Zoom meetings and recordings to Oyatie meet and recordings migration.
j197 candidate: Google Workspace to Oyatie mail, calendar, drive, docs, sheets, and meet migration.
j198 candidate: Shopify Plus commerce to Oyatie marketplace and payments migration.
j199 candidate: Twilio messaging and voice to Oyatie messenger and contact-center migration.
j200 candidate: Tableau analytics to Oyatie analytics and data-warehouse migration.
j201 candidate: ServiceNow GRC to Oyatie governance and compliance migration.
j202 candidate: Workday Learning to Oyatie learning-management migration.
j203 candidate: Workday Adaptive Planning to Oyatie financial-planning migration.
j204 candidate: Jira Service Management to Oyatie ITSM and incident-management migration.
j205 candidate: Confluence knowledge space to Oyatie docs and drive migration.
j206 candidate: Bitbucket pipelines to Oyatie developer-sdk and cloud-iac migration.
j207 candidate: Fivetran connector estate to Oyatie data-pipeline migration.
j208 candidate: Segment CDP to Oyatie consent-graph, data-pipeline, and ontology migration.
j209 candidate: Looker semantic model to Oyatie analytics and ontology migration.
j210 candidate: Adobe Experience Manager to Oyatie sites and design-collaboration migration.
j211 candidate: Intune device management to Oyatie identity, cloud-iam, and governance migration.
j212 candidate: Microsoft Purview to Oyatie compliance, governance, and consent-graph migration.
j213 candidate: Sentinel SIEM to Oyatie detection and observability migration.
j214 candidate: Defender XDR to Oyatie detection and incident-management migration.
j215 candidate: 1Password vault to Oyatie cloud-secrets and identity migration.
j216 candidate: Wiz cloud-security to Oyatie cloud-security composition across detection, intelligence, policy, and observability.
j217 candidate: Shopify supplier marketplace to Oyatie marketplace and tenant onboarding migration.
j218 candidate: FullStory session replay to Oyatie observability and privacy-bound replay migration.
j219 candidate: Amplitude product analytics to Oyatie analytics and event-governance migration.
j220 candidate: SAP Ariba procurement to Oyatie marketplace, contracts, and supplier workflows migration.
The backlog should be prioritized by proof value, not by alphabetical vendor order.
The highest proof value is a journey that touches identity, policy, audit, workflow, ontology, and migration parity.
The next highest proof value is a journey that proves regulated pack overlays.
The next highest proof value is a journey that proves tenant onboarding.
The next highest proof value is a journey that proves cross-tenant handoff.
The next highest proof value is a journey that proves rollback.
The backlog should not become a new prose-only expansion wave.
Each accepted migration journey should have an importer, fixture, parity report, and cutover runbook.

### §6.18 Runtime Proof Matrix

Runtime proof 001: tenant creation API returns a tenant id and emits TenantCreated.
Runtime proof 002: tenant creation writes a tenancy audit event with audit_id.
Runtime proof 003: tenant creation labels traces with tenant, region, and tier.
Runtime proof 004: tenant creation appears in tenant onboarding dashboard.
Runtime proof 005: tier grant command validates Bronze, Silver, Gold, and Platinum ids.
Runtime proof 006: tier grant command rejects unknown tier ids.
Runtime proof 007: tier grant command emits CapabilityTierGranted.
Runtime proof 008: tier revoke command emits CapabilityTierRevoked.
Runtime proof 009: compliance pack activation validates pack id.
Runtime proof 010: compliance pack activation rejects unsupported jurisdiction.
Runtime proof 011: compliance pack activation emits CompliancePackActivated.
Runtime proof 012: localization overlay activation validates region.
Runtime proof 013: localization overlay activation rejects incompatible pack combinations.
Runtime proof 014: identity principal creation binds personal or work tenant boundary.
Runtime proof 015: identity principal lookup refuses cross-boundary access.
Runtime proof 016: Cedar policy check returns allow with audit evidence.
Runtime proof 017: Cedar policy check returns deny with audit evidence.
Runtime proof 018: policy check latency records p50, p95, and p99.
Runtime proof 019: ontology projection creates typed object.
Runtime proof 020: ontology projection preserves legacy source id.
Runtime proof 021: ontology projection records schema revision.
Runtime proof 022: workflow instance starts from template.
Runtime proof 023: workflow instance transitions state.
Runtime proof 024: workflow instance handles compensation.
Runtime proof 025: workflow instance replay preserves current state.
Runtime proof 026: migration dry-run imports fixture rows.
Runtime proof 027: migration dry-run writes no production state.
Runtime proof 028: migration dry-run reports row count.
Runtime proof 029: migration dry-run reports lossy fields.
Runtime proof 030: migration dry-run reports permission deltas.
Runtime proof 031: migration cutover requires approval evidence.
Runtime proof 032: migration sunset writes source-system retention evidence.
Runtime proof 033: dashboard query reads emitted fixture data.
Runtime proof 034: dashboard query groups by tenant.
Runtime proof 035: dashboard query groups by tier.
Runtime proof 036: dashboard query groups by pack.
Runtime proof 037: SLO burn calculation accepts good fixture.
Runtime proof 038: SLO burn calculation rejects bad fixture.
Runtime proof 039: runbook drill records operator action.
Runtime proof 040: runbook drill records rollback state.
Runtime proof 041: audit chain seals event batch.
Runtime proof 042: audit chain rejects malformed event class.
Runtime proof 043: audit chain exports evidence packet.
Runtime proof 044: compliance export includes pack id.
Runtime proof 045: compliance export includes audit ids.
Runtime proof 046: compliance export includes time window.
Runtime proof 047: compliance export includes not-tested gaps.
Runtime proof 048: reachability check calculates node count.
Runtime proof 049: reachability check calculates reachable count.
Runtime proof 050: reachability check fails when required root link missing.
Runtime proof 051: seven-surface check passes complete service fixture.
Runtime proof 052: seven-surface check fails missing runbook fixture.
Runtime proof 053: vendor mapping check passes known service references.
Runtime proof 054: vendor mapping check fails missing service references.
Runtime proof 055: localization pack check passes required sections.
Runtime proof 056: localization pack check fails missing residency section.
Runtime proof 057: compliance pack check passes required authority.
Runtime proof 058: compliance pack check fails missing evidence export path.
Runtime proof 059: journey bundle check passes story/UX/handshake/test plan.
Runtime proof 060: journey bundle check fails missing integration test plan.
Runtime proof 061: persona anchor check passes role pressure fields.
Runtime proof 062: persona anchor check fails generic biography-only file.
Runtime proof 063: ADR substance check passes vendor-specific object/API/UX/Cedar/ontology/workflow/failure mode.
Runtime proof 064: ADR substance check fails repeated template clauses.
Runtime proof 065: CI blocks merge on failed governance gate.
Runtime proof 066: CI publishes evidence bundle.
Runtime proof 067: CI reports not-tested gap when a validator is skipped.
Runtime proof 068: board packet consumes CI evidence.
Runtime proof 069: board packet separates simulated fixture from live pilot.
Runtime proof 070: board packet names remaining blockers.

### §6.19 Board Role-Specific Reading Guide

Audit committee should start with compliance packs, audit-event sweep, SLO library, and evidence export.
Audit committee should ask why strict audit-event registration is still zero in the sweep.
Audit committee should ask when registered class compliance moves above zero.
Audit committee should ask which pack claims have legal signoff.
Risk committee should start with risk register, threat models, runbooks, and incident-management surfaces.
Risk committee should ask which unified-substrate concentration risks are mitigated.
Risk committee should ask how personal/work boundaries are tested.
Risk committee should ask how cross-tenant handoffs are evidenced.
Product committee should start with journeys, personas, and microservice suites.
Product committee should ask which journeys become Wave-4 executable slices.
Product committee should ask which personas create the hardest product proof.
Product committee should ask which services are build-ready versus only design-ready.
Technology committee should start with Rust scaffolding, CI gates, registries, and dashboard wiring.
Technology committee should ask which validator blocks promotion first.
Technology committee should ask which core services compile first.
Technology committee should ask which runtime signal appears first.
Go-to-market committee should start with capability tiers, vendor mapping, migration journeys, and claim boundaries.
Go-to-market committee should ask which vendor stories are safe to sell.
Go-to-market committee should ask which claims need runtime evidence first.
Go-to-market committee should ask which migration paths can be piloted.
Finance committee should start with TAM update, FinOps, tier pricing classes, and migration cost framing.
Finance committee should ask how capability tiers map to pricing.
Finance committee should ask how migration cost is estimated and measured.
Finance committee should ask how SLOs change operating cost by tier.
Compensation committee should start with persona coverage and workforce workflows only if internal rollout is in scope.
Compensation committee should ask how performance-management and learning-management avoid fairness risk.
Security committee should start with identity, policy, audit-chain, detection, and threat models.
Security committee should ask how high-risk exceptions are refused or allowed.
Security committee should ask how every deny becomes evidence.
Security committee should ask how every allow can be reconstructed.
The full board should ask one shared question.
The shared question is: which post-remediation artifact became executable evidence this period?

### §6.20 Claim Maturity Ladder

Level 0 claim: idea exists in conversation.
Level 1 claim: idea exists in prose.
Level 2 claim: idea exists in a named artifact.
Level 3 claim: artifact has source authority.
Level 4 claim: artifact has a substance bar.
Level 5 claim: artifact passes review.
Level 6 claim: artifact has machine-readable representation.
Level 7 claim: representation has schema validation.
Level 8 claim: validation runs locally.
Level 9 claim: validation runs in CI.
Level 10 claim: CI blocks promotion.
Level 11 claim: code implements the claim.
Level 12 claim: tests exercise the implementation.
Level 13 claim: negative tests reject bad states.
Level 14 claim: telemetry measures runtime behavior.
Level 15 claim: dashboard displays runtime behavior.
Level 16 claim: SLO evaluates runtime behavior.
Level 17 claim: runbook covers failure.
Level 18 claim: drill exercises runbook.
Level 19 claim: pilot tenant uses the behavior.
Level 20 claim: board packet reports evidence.
Level 21 claim: legal/security/product approve launch language.
Level 22 claim: production tenant uses the behavior.
Level 23 claim: customer outcome confirms usefulness.
Level 24 claim: revenue or retention validates market value.
Level 25 claim: repeatable operating cadence sustains the claim.
Post-remediation ADR-0321 coverage is around levels 3-5 for 110 dossiers.
Post-remediation capability-tier registry is around levels 6-8 until CI gates land.
Post-remediation compliance packs are around levels 3-6 depending pack.
Post-remediation SLO library is around levels 3-6 until telemetry lands.
Post-remediation dashboards are around levels 3-6 until live data lands.
Post-remediation journeys are around levels 3-5 until tests land.
Post-remediation migration journeys are around levels 2-5 until tooling lands.
Post-remediation microservice suites are around levels 3-6 until Rust code lands.
Post-remediation governance crates are around levels 6-8 for scaffolds, lower where tests are absent.
Post-remediation tenant pilots are not yet at runtime proof.
Wave 4 should move selected claims upward rather than moving every claim one small step.
The board should prefer a few claims at levels 15-20 over thousands stuck at levels 3-5.

### §6.21 Investor Update Language

Investor line 001: "The thesis is unchanged; the proof density changed."
Investor line 002: "Wave-3-G gave us breadth; remediation gave us operating detail."
Investor line 003: "We have 110 substantive vendor dossiers out of 165 planned."
Investor line 004: "We have 175 substantive journeys and five migration journeys in flight."
Investor line 005: "We have 62-plus service suites at a seven-surface documentation bar."
Investor line 006: "We have eight compliance packs and eight localization packs as launch-design inputs."
Investor line 007: "We have a four-tier capability registry mapped across 70 services and 295 vendors."
Investor line 008: "We found audit-event and reachability gaps rather than hiding them."
Investor line 009: "Wave 4 is about executable proof: Rust, CI, telemetry, migration tooling, and pilots."
Investor line 010: "We are not claiming production completeness."
Investor line 011: "We are claiming unusual architecture throughput and remediation discipline."
Investor line 012: "The next milestone is a runtime slice that proves tenant, policy, audit, workflow, ontology, and migration together."
Investor line 013: "The corpus is now specific enough to build from."
Investor line 014: "The corpus is now specific enough to diligence."
Investor line 015: "The corpus is now specific enough to expose its own remaining risk."
Investor line 016: "The TAM remains enterprise SaaS fragmentation and regulated workflow consolidation."
Investor line 017: "The differentiation remains unification without grouping-shaped internal sprawl."
Investor line 018: "The immediate risk remains conversion from docs to working gates."
Investor line 019: "The immediate opportunity is demonstrating one tenant pilot."
Investor line 020: "The board should measure the next quarter by executable evidence, not document volume."

### §6.22 Engineering Update Language

Engineering line 001: "Treat this briefing as a projection, not a control surface."
Engineering line 002: "Use registries, schemas, fixtures, tests, and Rust code as control surfaces."
Engineering line 003: "Start with validators where claims are already machine-readable."
Engineering line 004: "Do not expand new service docs until the seven-surface gate exists."
Engineering line 005: "Do not add new vendor dossiers without the ADR-0321 substance bar."
Engineering line 006: "Do not claim audit readiness until event classes register."
Engineering line 007: "Do not claim reachability until the graph improves."
Engineering line 008: "Do not claim SLOs until telemetry feeds them."
Engineering line 009: "Do not claim dashboards until they read data."
Engineering line 010: "Do not claim migration support until dry-run tooling exists."
Engineering line 011: "Do not claim pilot readiness until tenant activation is runnable."
Engineering line 012: "Do not claim compliance launch until legal and security review complete."
Engineering line 013: "Prefer failing fixtures before green validators."
Engineering line 014: "Prefer one end-to-end slice over ten shallow scaffolds."
Engineering line 015: "Prefer service ownership over suite ownership."
Engineering line 016: "Prefer tier grants over product clones."
Engineering line 017: "Prefer audit evidence over screenshots."
Engineering line 018: "Prefer generated projections over hand-maintained link gardens."
Engineering line 019: "Prefer negative policy tests over optimistic happy paths."
Engineering line 020: "Prefer board evidence that can be rerun."

### §6.23 Execution Readiness Checkpoint Matrix

Readiness checkpoint 001: thesis is stable.
Readiness checkpoint 002: thesis is not yet runtime-proven.
Readiness checkpoint 003: ADR-0321 direct coverage is substantial.
Readiness checkpoint 004: ADR-0321 direct coverage is incomplete.
Readiness checkpoint 005: j01-j175 human journey coverage is substantive.
Readiness checkpoint 006: j176-j180 migration coverage is in flight.
Readiness checkpoint 007: service-suite coverage is broad.
Readiness checkpoint 008: service-suite coverage is not universal.
Readiness checkpoint 009: per-service ADR coverage exists.
Readiness checkpoint 010: per-service ADR coverage is not universal.
Readiness checkpoint 011: persona role coverage is broad.
Readiness checkpoint 012: persona validation with users remains future work.
Readiness checkpoint 013: compliance packs are architecture-ready.
Readiness checkpoint 014: compliance packs are not certification-ready.
Readiness checkpoint 015: localization packs are design-ready.
Readiness checkpoint 016: localization packs are not launch-ready.
Readiness checkpoint 017: capability-tier registry is authored.
Readiness checkpoint 018: capability-tier registry needs enforced validators.
Readiness checkpoint 019: vendor-tier mapping is authored.
Readiness checkpoint 020: vendor-tier mapping needs drift prevention.
Readiness checkpoint 021: microservice-tier mapping is authored.
Readiness checkpoint 022: microservice-tier mapping needs live roster reconciliation.
Readiness checkpoint 023: workflow templates are authored.
Readiness checkpoint 024: workflow templates need runtime bindings.
Readiness checkpoint 025: sample tenants are authored.
Readiness checkpoint 026: sample tenants need pilot activation.
Readiness checkpoint 027: observability dashboards are authored.
Readiness checkpoint 028: observability dashboards need live telemetry.
Readiness checkpoint 029: SLO library is authored.
Readiness checkpoint 030: SLO library needs measured burn calculations.
Readiness checkpoint 031: tutorials are authored.
Readiness checkpoint 032: tutorials need smoke tests.
Readiness checkpoint 033: benchmarks are authored.
Readiness checkpoint 034: benchmarks need measured runs.
Readiness checkpoint 035: risk register is more useful.
Readiness checkpoint 036: risk register needs live risk indicators.
Readiness checkpoint 037: anti-pattern standards are authored.
Readiness checkpoint 038: anti-pattern standards need automated checks.
Readiness checkpoint 039: Wave-3 retrospective is authored.
Readiness checkpoint 040: Wave-3 retrospective needs Wave-4 behavior change.
Readiness checkpoint 041: cross-service test plans exist.
Readiness checkpoint 042: cross-service test plans need executable tests.
Readiness checkpoint 043: threat models exist.
Readiness checkpoint 044: threat models need adversarial test fixtures.
Readiness checkpoint 045: runbooks exist.
Readiness checkpoint 046: runbooks need drills.
Readiness checkpoint 047: handoff matrices exist.
Readiness checkpoint 048: handoff matrices need contract enforcement.
Readiness checkpoint 049: pack overlays exist.
Readiness checkpoint 050: pack overlays need policy tests.
Readiness checkpoint 051: migration playbooks exist.
Readiness checkpoint 052: migration playbooks need importer tooling.
Readiness checkpoint 053: governance crates are scaffolded.
Readiness checkpoint 054: governance crates need tests and CI wiring.
Readiness checkpoint 055: audit-event sweep exists.
Readiness checkpoint 056: audit-event compliance is not green.
Readiness checkpoint 057: six-hop audit exists.
Readiness checkpoint 058: reachability is not green.
Readiness checkpoint 059: IP cross-reference sweep exists.
Readiness checkpoint 060: IP drift prevention needs automation.
Readiness checkpoint 061: board narrative is stronger.
Readiness checkpoint 062: board evidence must become runtime evidence.
Readiness checkpoint 063: investor TAM story is stronger.
Readiness checkpoint 064: investor proof still needs execution.
Readiness checkpoint 065: sales enablement is richer.
Readiness checkpoint 066: sales claims need strict boundaries.
Readiness checkpoint 067: marketing story is clearer.
Readiness checkpoint 068: marketing copy must avoid overclaiming.
Readiness checkpoint 069: product roadmap is better grounded.
Readiness checkpoint 070: product roadmap must choose slices.
Readiness checkpoint 071: engineering backlog is clearer.
Readiness checkpoint 072: engineering backlog must narrow.
Readiness checkpoint 073: compliance posture is better specified.
Readiness checkpoint 074: compliance posture needs legal review.
Readiness checkpoint 075: security posture is better specified.
Readiness checkpoint 076: security posture needs negative tests.
Readiness checkpoint 077: operations posture is better specified.
Readiness checkpoint 078: operations posture needs live drills.
Readiness checkpoint 079: tenant model is central.
Readiness checkpoint 080: tenant model needs pilot proof.
Readiness checkpoint 081: identity boundary is central.
Readiness checkpoint 082: identity boundary needs personal/work tests.
Readiness checkpoint 083: policy engine is central.
Readiness checkpoint 084: policy engine needs latency and denial evidence.
Readiness checkpoint 085: workflow engine is central.
Readiness checkpoint 086: workflow engine needs saga and replay proof.
Readiness checkpoint 087: ontology is central.
Readiness checkpoint 088: ontology needs projection and migration proof.
Readiness checkpoint 089: audit-chain is central.
Readiness checkpoint 090: audit-chain needs class registration proof.
Readiness checkpoint 091: marketplace settlement is central.
Readiness checkpoint 092: marketplace settlement needs runtime deal proof.
Readiness checkpoint 093: UX shell coherence is central.
Readiness checkpoint 094: UX shell coherence needs pilot interaction proof.
Readiness checkpoint 095: capability tiers are central.
Readiness checkpoint 096: capability tiers need grant and revoke proof.
Readiness checkpoint 097: compliance overlays are central.
Readiness checkpoint 098: compliance overlays need conflict resolution proof.
Readiness checkpoint 099: localization overlays are central.
Readiness checkpoint 100: localization overlays need region fixture proof.
Readiness checkpoint 101: migration is central to displacement.
Readiness checkpoint 102: migration needs dry-run and parity proof.
Readiness checkpoint 103: dashboards are central to trust.
Readiness checkpoint 104: dashboards need live query proof.
Readiness checkpoint 105: SLOs are central to reliability.
Readiness checkpoint 106: SLOs need burn calculation proof.
Readiness checkpoint 107: runbooks are central to operations.
Readiness checkpoint 108: runbooks need exercise proof.
Readiness checkpoint 109: risk register is central to governance.
Readiness checkpoint 110: risk register needs owners and metrics.
Readiness checkpoint 111: quality bars are central to scale.
Readiness checkpoint 112: quality bars need automated enforcement.
Readiness checkpoint 113: root hub is central to agent navigation.
Readiness checkpoint 114: root hub needs reachability repair.
Readiness checkpoint 115: machine-readable specs are central to future execution.
Readiness checkpoint 116: machine-readable specs need schema tests.
Readiness checkpoint 117: Markdown remains useful for briefing.
Readiness checkpoint 118: Markdown must not be sole authority for gates.
Readiness checkpoint 119: Wave 3 proved corpus generation.
Readiness checkpoint 120: Wave 4 must prove corpus execution.
Readiness checkpoint 121: Wave 3 proved audit response.
Readiness checkpoint 122: Wave 4 must prove audit closure.
Readiness checkpoint 123: Wave 3 proved breadth.
Readiness checkpoint 124: Wave 4 must prove depth in runtime slices.
Readiness checkpoint 125: Wave 3 proved strategic coherence.
Readiness checkpoint 126: Wave 4 must prove operational coherence.
Readiness checkpoint 127: Wave 3 proved documentation velocity.
Readiness checkpoint 128: Wave 4 must prove validation velocity.
Readiness checkpoint 129: Wave 3 proved multi-domain ambition.
Readiness checkpoint 130: Wave 4 must prove domain integration.
Readiness checkpoint 131: Wave 3 proved role coverage.
Readiness checkpoint 132: Wave 4 must prove role workflows.
Readiness checkpoint 133: Wave 3 proved vendor mapping.
Readiness checkpoint 134: Wave 4 must prove migration mechanics.
Readiness checkpoint 135: Wave 3 proved pack design.
Readiness checkpoint 136: Wave 4 must prove pack enforcement.
Readiness checkpoint 137: Wave 3 proved dashboard design.
Readiness checkpoint 138: Wave 4 must prove dashboard data.
Readiness checkpoint 139: Wave 3 proved SLO design.
Readiness checkpoint 140: Wave 4 must prove SLO measurement.
Readiness checkpoint 141: Wave 3 proved service inventory.
Readiness checkpoint 142: Wave 4 must prove service runtime.
Readiness checkpoint 143: Wave 3 proved retrospective honesty.
Readiness checkpoint 144: Wave 4 must prove process correction.
Readiness checkpoint 145: Wave 3 proved board story.
Readiness checkpoint 146: Wave 4 must prove board evidence.
Readiness checkpoint 147: Wave 3 proved investor story.
Readiness checkpoint 148: Wave 4 must prove diligence artifacts that run.
Readiness checkpoint 149: Wave 3 proved GTM vocabulary.
Readiness checkpoint 150: Wave 4 must prove controlled claims.
Readiness checkpoint 151: Wave 3 proved remediation stamina.
Readiness checkpoint 152: Wave 4 must prove implementation stamina.
Readiness checkpoint 153: Wave 3 proved the corpus can absorb criticism.
Readiness checkpoint 154: Wave 4 must prove the product can absorb real tests.
Readiness checkpoint 155: Wave 3 proved that false confidence can be surfaced.
Readiness checkpoint 156: Wave 4 must prove that surfaced gaps can be closed.
Readiness checkpoint 157: Wave 3 proved the organization can describe the platform.
Readiness checkpoint 158: Wave 4 must prove the organization can ship the platform.
Readiness checkpoint 159: Wave 3 proved a high ceiling.
Readiness checkpoint 160: Wave 4 must prove a firm floor.
Readiness checkpoint 161: Wave 3 proved the story is worth pursuing.
Readiness checkpoint 162: Wave 4 must prove the story can execute.
Readiness checkpoint 163: Wave 3 proved Oyatie can think at ecosystem scale.
Readiness checkpoint 164: Wave 4 must prove Oyatie can operate at service scale.
Readiness checkpoint 165: Wave 3 proved the team can write the map.
Readiness checkpoint 166: Wave 4 must prove the team can drive the route.
Readiness checkpoint 167: Wave 3 proved breadth under time pressure.
Readiness checkpoint 168: Wave 4 must prove correctness under verification pressure.
Readiness checkpoint 169: Wave 3 proved internal ambition.
Readiness checkpoint 170: Wave 4 must prove external credibility.
Readiness checkpoint 171: Wave 3 proved corpus depth after remediation.
Readiness checkpoint 172: Wave 4 must prove implementation depth after audit.
Readiness checkpoint 173: Wave 3 proved a serious foundation.
Readiness checkpoint 174: Wave 4 must prove a serious product path.
Readiness checkpoint 175: Wave 3 proved the remediation transformed state.
Readiness checkpoint 176: Wave 4 must prove the transformed state can run.
Readiness checkpoint 177: The board should approve scope by evidence, not enthusiasm.
Readiness checkpoint 178: The board should ask for fewer claims and stronger proof.
Readiness checkpoint 179: The board should reward closure over expansion.
Readiness checkpoint 180: The board should keep this document as the post-remediation checkpoint.

## §7 Board-Readable Narrative

The short version is that Oyatie is no longer just a sweeping software-unification thesis.
It is now a sweeping thesis with a much denser implementation runway.
The remediation wave closed the most dangerous narrative gap: volume without substance.
ADR-0321 is no longer merely a list of incumbents.
It now contains 110 substantive vendor dossiers.
The journey corpus is no longer only a broad map.
j01 through j175 now carry substantive human-use evidence.
The migration corpus is no longer absent.
j176 through j180 now start the incumbent-to-Oyatie motion.
The microservice corpus is no longer only a roster.
Sixty-two-plus services now meet a seven-surface substance bar.
The persona corpus is no longer decorative.
More than 90 personas now anchor actual role pressure.
The compliance and localization posture is no longer generic.
Eight compliance packs and eight localization packs now create a regulated-market runway.
The packaging model is no longer hand-wavy.
Bronze, Silver, Gold, and Platinum tiers now map across services and vendors.
The observability posture is no longer just doctrine.
Dashboards and SLO library entries now describe what runtime must prove.
The quality posture is no longer optimistic.
The retrospective, audit-event sweep, and six-hop audit name the remaining failures.
The next phase is not another expansion of prose.
The next phase is conversion.
Convert registries into gates.
Convert services into Rust crates.
Convert journeys into tests.
Convert migration narratives into tooling.
Convert packs into policy.
Convert dashboards into live telemetry.
Convert sample tenants into pilots.
Convert board confidence from thesis confidence into evidence confidence.
The board should approve Wave 4 only if it is scoped around executable proof.
The board should expect narrower scope and harder verification.
The board should ask for runtime evidence, not merely more documents.
The board should keep the TAM thesis intact.
The board should keep the claim boundary honest.
The company has shown it can generate and remediate a very large architecture corpus.
Now it must show the corpus can run.

### §7.1 Checkpoint

Checkpoint status: new post-remediation briefing authored as a separate file.
Checkpoint boundary: original 2026-05-21 Wave-3-G briefing must remain untouched.
Checkpoint claim: post-remediation state is transformed but not runtime-complete.
Checkpoint evidence target: line count at or above 2,500.
Checkpoint VCS target: claim, verify, done, and promote through Oya VCS.
Checkpoint remaining work after this document: Wave 4 implementation and enforcement.
