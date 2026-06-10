export const meta = {
  name: 'bominal-oyatie-lost-context-reconciliation',
  description: 'oyatie WAS bominal (renamed/migrated, context churned). Diff bominal (the past, cloned + GitHub issues/milestones) against oyatie/source (the present) to find what the migration LOST or CHURNED — product scope, verticals, roadmap/sequencing, decisions, naming continuity. Read-only. Output: a lost-context register + a FOUNDER INTERVIEW AGENDA (decisions on what to restore). NOT a blind recovery.',
  phases: [
    { title: 'Analyze', detail: 'parallel: bominal product/roadmap, bominal decisions, bominal strategy/issues, oyatie present' },
    { title: 'Diff', detail: 'bominal-past vs oyatie-present → lost/churned-context register (opus)' },
    { title: 'Agenda', detail: 'turn losses into a deduped founder interview agenda (opus)' },
  ],
}

const B = '/Users/jasonlee/Developer/_recover-bominal'
const SRC = '/Users/jasonlee/Developer/source'
const OUT = '/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/bominal-reconciliation'
const PRIOR = 'CONTEXT: oyatie WAS bominal — renamed + migrated; the migration CHURNED/LOST context. We are recovering oyatie\'s OWN history, not adopting a foreign repo. Already-ruled founder decisions live in /Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md (masterplan-generated, own-everything-ratchet, forge=GitHub-now/bespoke-later, data-tier-own-all, identity=oya-identity+Zitadel-bridge, Cedar-contract, framekernel-host-committed, maximal-vertical-scope incl defense+powergrid, etc.) — do NOT re-surface those. The .Trash legacy recovery already found 7 items (KR HR/payroll packs, First Proof Slice, 4-lane closure model) at /Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/legacy-recovery/00-RECOVERY-REGISTER.md — read it, do not duplicate.'

phase('Analyze')
const a1 = function () {
  return agent(
    'You map BOMINAL\'s full PRODUCT + VERTICAL + ROADMAP picture (bominal = oyatie\'s earlier self). ' + PRIOR + '\n' +
    'READ from the clone: `ls ' + B + '/modules/` then each ' + B + '/modules/<m>/README.md (community, connect, emergency, hr, insurance, logistics, mail, manufacturing, medical, messenger, patient, payments, pharmacy, records, security, workflow); ' + B + '/portfolio/strategy/*.md (product-arm-theses, platform-moat-strategy, sequencing-rationale, dependency-map); ' + B + '/.planning/{ROADMAP,MILESTONES,REQUIREMENTS,PROJECT}.md; ' + B + '/product-control/ (capabilities/entitlements/lifecycle/metering/topology READMEs). ' +
    'ALSO fetch the roadmap from GitHub: `gh api repos/jason931225/bominal/milestones --paginate -f state=all` (54 milestones — the multi-year vertical roadmap incl. health/CDSS, logistics/transport/manufacturing/warehouse, marketplace, CCTV-vision, AMR/facility, infrastructure-sovereignty 2027-2028, conglomerate-tier, and the 2029-2030 far-future: facilities+data-center, agriculture+food, public-sector, civil-infra+utilities[POWERGRID], public-safety+drones+DEFENSE). ' +
    'PRODUCE ' + OUT + '/10-bominal-product-roadmap.md: (a) the product/module inventory (each: name, purpose, vertical); (b) the FULL vertical list incl. the far-future ones; (c) the roadmap sequencing/timeline; (d) the infrastructure-sovereignty ratchet (which substrates owned when). RETURN a tight digest.',
    { label: 'bom:product-roadmap', phase: 'Analyze', model: 'opus' }
  )
}
const a2 = function () {
  return agent(
    'You extract BOMINAL\'s key DECISIONS (bominal = oyatie\'s earlier self). ' + PRIOR + '\n' +
    'READ `ls ' + B + '/decisions/` (132 ADRs) and read the product/vertical/architecture-bearing ones fully (e.g. clinical-canonical-record, hr-payroll-vertical, contract-bid-pricing, object-graph, property-types, isolation-operating-model, multi-runtime, tenancy, data-tier, event-streaming, cloud-native-infra, plus any vertical/product ADRs). Skim the rest by title. ' +
    'PRODUCE ' + OUT + '/11-bominal-decisions.md: the key bominal decisions, grouped (product/vertical, architecture, platform, governance), each with a one-line decision atom + whether it looks PRESENT / WEAKER / ABSENT in oyatie (you may be approximate; the diff phase confirms). Flag decisions that encode product/vertical INTENT especially. RETURN a tight digest.',
    { label: 'bom:decisions', phase: 'Analyze', model: 'opus' }
  )
}
const a3 = function () {
  return agent(
    'You extract BOMINAL\'s STRATEGIC INTENT + backlog themes. ' + PRIOR + '\n' +
    'READ ' + B + '/CONSTITUTION.md, ' + B + '/CONTEXT.md, ' + B + '/README.md, ' + B + '/portfolio/strategy/{product-arm-theses,platform-moat-strategy,sequencing-rationale}.md, ' + B + '/portfolio/{kill-criteria,maturity,launch-readiness,commercial-risks}/ (skim). ' +
    'ALSO fetch strategic issues: `gh issue list --repo jason931225/bominal --state all --limit 250 --json number,title,labels,milestone` and focus on the EPICs / "High-value area #1-10" / "Move #1-8" / strategic + planning-labeled issues (not chore/ci noise). ' +
    'PRODUCE ' + OUT + '/12-bominal-strategy.md: the mission/thesis, the moat strategy, the sequencing rationale, the high-value-areas + strategic Moves, and the kill-criteria/maturity framing. RETURN a tight digest.',
    { label: 'bom:strategy', phase: 'Analyze', model: 'opus' }
  )
}
const a4 = function () {
  return agent(
    'You map OYATIE\'s PRESENT state so it can be diffed against bominal (its past). ' + PRIOR + '\n' +
    'READ `ls ' + SRC + '/oya/` and `ls ' + SRC + '/cloud/` (the current service surfaces); skim ' + SRC + '/docs/decisions/ titles (`ls`); read the already-ruled decision record (path in CONTEXT). Build the bominal->oyatie RENAME/MAPPING hypothesis: bominal modules (medical/patient/pharmacy/emergency/records/insurance/logistics/manufacturing/payments/hr/...) -> oyatie services (emr/healthcare-integration/imaging/.../crm/.../oya-billing/...). ' +
    'PRODUCE ' + OUT + '/13-oyatie-present.md: oyatie current products/verticals + the best-guess module->service mapping + which bominal verticals clearly have NO oyatie home yet. RETURN a tight digest.',
    { label: 'oya:present', phase: 'Analyze', model: 'opus' }
  )
}
const analyzed = await parallel([a1, a2, a3, a4])
const dig = analyzed.map(function (x, i) { return '--- analyze[' + i + '] ---\n' + (x || '(failed)') }).join('\n\n')

phase('Diff')
const diff = await agent(
  'You are the LOST-CONTEXT DIFF lead. oyatie WAS bominal (renamed/migrated, context churned). Diff bominal (past) vs oyatie (present) to find what the migration LOST or CHURNED. ' + PRIOR + '\n' +
  'Open the four analysis artifacts in ' + OUT + ' (10/11/12/13) + the .Trash recovery register. Analysis digests:\n' + dig + '\n\n' +
  'PRODUCE ' + OUT + '/20-LOST-CONTEXT-REGISTER.md — a categorized register of migration losses: (A) PRODUCT/MODULE scope lost or fuzzed (bominal module with no clear oyatie owner, or renamed so the lineage is unclear); (B) VERTICAL scope lost (esp. the far-future verticals — utilities/powergrid, defense, agriculture, public-sector, data-center, conglomerate — present in bominal milestones, absent in oyatie ADRs); (C) ROADMAP/SEQUENCING lost (the multi-year milestone sequencing + infra-sovereignty ratchet); (D) DECISION/RATIONALE lost (bominal ADRs encoding intent not carried into oyatie); (E) NAMING-CONTINUITY broken (rename map unclear/ambiguous). Each row: item | what it was (bominal ref) | oyatie status (absent/weaker/renamed-unclear/present) | severity (HIGH/MED/LOW) | restore-recommendation. Lead with HIGH-severity genuine losses; be honest where oyatie is actually STRONGER (migration improved it). RETURN the HIGH-severity losses + counts per category verbatim.',
  { label: 'bom:diff', phase: 'Diff', model: 'opus' }
)

phase('Agenda')
const agenda = await agent(
  'You are the INTERVIEW-AGENDA author. The founder explicitly wants to be INTERVIEWED on what to restore from bominal (NOT a blind recovery). ' + PRIOR + '\n' +
  'Read ' + OUT + '/20-LOST-CONTEXT-REGISTER.md and the already-ruled decision record. Turn the losses into a CRISP, DEDUPED, PRIORITIZED founder interview agenda — skip anything already ruled in the decision record. ' +
  'PRODUCE ' + OUT + '/00-INTERVIEW-AGENDA.md: grouped decision themes (e.g. vertical-scope-restoration, module->service lineage, roadmap/sequencing adoption, specific high-value modules like medical/clinical-record/CDSS, the infra-sovereignty ratchet mapping, naming-continuity), each as 1-3 crisp FOUNDER QUESTIONS with 2-4 concrete options + a recommendation. Order by leverage. Mark which are door:one-way. Aim for a tight agenda (the highest-value decisions first), not an exhaustive list. RETURN the full agenda verbatim so it can be asked.',
  { label: 'bom:agenda', phase: 'Agenda', model: 'opus' }
)

return {
  lost_context_register: OUT + '/20-LOST-CONTEXT-REGISTER.md',
  interview_agenda: OUT + '/00-INTERVIEW-AGENDA.md',
  agenda: agenda,
}
