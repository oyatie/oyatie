export const meta = {
  name: 'wf2-rest-of-docs-sweep',
  description: 'Sweep all ~2000 non-ADR source docs + linux non-ADR docs against the now-ruled canon. Parallel lanes: deep-read high/med-signal clusters (top-level canonical, architecture/standards/specs, products/localization, governance/process, runbooks) + grep-driven scan of high-volume user-journeys/personas + a mechanical stale-term footprint (foundry 830, etc.). Each lane emits an artifact. Synthesis = rest-of-docs findings register (canon-contradictions / stale / slop / refinement / reachability) feeding the amendment phase. READ-ONLY.',
  phases: [
    { title: 'Footprint', detail: 'mechanical stale-term canon-conflict inventory (scripted)' },
    { title: 'Lanes', detail: 'parallel deep-read + scan lanes per cluster' },
    { title: 'Synthesis', detail: 'rest-of-docs findings register (opus)' },
  ],
}

const SRC = '/Users/jasonlee/Developer/source/docs'
const LNX = '/Users/jasonlee/Developer/linux/docs'
const OUT = '/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/docs-sweep'

const CANON = 'RULED CANON (check every doc against this — flag contradictions/stale framing): ' +
  '(1) masterplan is GENERATED from ADRs (NOT hand-authored authority); ADRs=SSOT. ' +
  '(2) "foundry" brand RETIRED → cloud-intelligence (the AI/agent substrate) or governance (the fitness/policy lane), per context. ' +
  '(3) Forge: GitHub NOW → bespoke VCS later; Forgejo DROPPED (mirror at most). ' +
  '(4) CI: unified oya-ci (Run+graph, Prow+Tekton+Argo); Jenkins/Argo OPERATIVE-until-cutover then retire (build-first-cutover-later); Jenkins/Argo are NOT the canonical endpoint. ' +
  '(5) Data: OWN the whole tier (endpoint); Postgres/Citus/Milvus/ClickHouse/etc. are TRANSITIONAL bridges; Redis→Valkey, Kafka→Pulsar. ' +
  '(6) Identity: oya-identity owned, Zitadel BRIDGE (not canonical). Policy: Cedar = CONTRACT, owned PARC = engine. ' +
  '(7) Isolation: framekernel-host COMMITTED endpoint; assume-breach microVM DEFAULT (NOT native-default/secure-by-default-native). ' +
  '(8) Autonomy ceiling: runtime-enforced hard gate, owned by governance. ' +
  '(9) Vocabulary: tenant-CLASS (not tenant-tier/tier-system); namespace "tier" (autonomy_tier/eu_ai_act_risk_tier/dr_tier/storage_tier); M0-M3/MVP wave-vocab RETIRED (gate-defined waves). ' +
  '(10) cloud/ DOGFOODS oya/ products (products = tenant workloads). GLOBAL-CANONICAL core + localization packs; KR = FIRST pack to market (not KR-core). ' +
  '(11) Maximal vertical scope, SEQUENCED via M0-gated parallel lanes; build-first-cutover-later; own-endpoint/vendor-bridge/ratchet-when-proven. ' +
  '(12) Unified governance-owned safety-gate set (HITL/no-actuation/biometric-off/no-lethal). ' +
  'Also flag: AI-slop (filler/fabricated-precision/internal-contradiction/hedging), stale/garbage/plain-wrong content, and reachability (DECISION→ADR / INSTRUCTION→session-context-bundle / GENERATED-REFERENCE / ORPHAN→not-needed).'

// ---------- Footprint (scripted, one agent driving bash) ----------
phase('Footprint')
const footprint = await agent(
  'You produce a MECHANICAL stale-term canon-conflict FOOTPRINT (no judgment, just inventory) for the amendment phase. ' + CANON + '\n' +
  'Using bash grep over ' + SRC + ' (exclude /decisions/ADR-*), for EACH stale term produce: the file COUNT + the top dirs affected. Terms: foundry, Jenkins, Forgejo, "tenant-tier|tier-system", "\\bM0\\b|\\bM1\\b|\\bM2\\b|\\bM3\\b", "\\bMVP\\b", Kafka, Redis, "native.?default|secure-by-default.*native", "Cedar.*engine", "eliminate.*Postgres", "M0/M1/M2/M3 milestone". ' +
  'For the BIG ones (foundry 830, tenant-tier 152, M0-M3 103, Redis 98, Kafka 85): sample ~5 files each and classify the fix as MECHANICAL-RENAME (simple term swap) vs CONTENT-CHANGE-NEEDED (the framing itself is stale, not just the word). ' +
  'WRITE ' + OUT + '/10-stale-term-footprint.md (per-term: count, top dirs, mechanical-vs-content, the rename target per context e.g. foundry→intelligence-vs-governance). RETURN a tight digest (the footprint totals + which terms need content-change not just rename).',
  { label: 'sweep:footprint', phase: 'Footprint', model: 'opus' }
)

// ---------- Lanes (parallel) ----------
phase('Lanes')
function lane(key, instruction) {
  return function () {
    return agent(
      'You are a REST-OF-DOCS REVIEWER (lane: ' + key + '). ' + CANON + '\n' + instruction +
      '\nFor each doc reviewed, note: canon-CONTRADICTIONS (cite the canon item # + file:line), stale framing, AI-slop, refinement opportunities, and reachability class. ' +
      'WRITE ' + OUT + '/20-' + key + '.md (findings grouped by doc; lead with the genuine canon-contradictions). RETURN a tight digest (top contradictions + counts).',
      { label: 'sweep:' + key, phase: 'Lanes', model: 'opus' }
    )
  }
}
const lanes = [
  lane('toplevel-canonical', 'DEEP-READ each top-level canonical doc and check vs canon: ' + SRC + '/DESIGN.md, MASTERPLAN.md, PRD.md, PRD-OYATIE-FROM-SCRATCH-CANONICAL.md, GLOSSARY.md (skim for stale terms), DOC-CATALOG.md, CONTRADICTION-LEDGER.md, COMPETITIVE-GAP-ANALYSIS.md, MISTAKES-LEDGER.md, DESIGN/architecture overview. These are the highest-signal; contradictions here are the most important.'),
  lane('architecture', 'DEEP-READ the architecture cluster: `ls ' + SRC + '/architecture/` (43 docs) — read each (or chunk by topic). Flag every architectural claim that contradicts the ruled canon (forge/CI/data/identity/isolation/masterplan/foundry).'),
  lane('standards-specs', 'Review standards/ (103) + specs/ (116): `ls ' + SRC + '/standards/` and `ls ' + SRC + '/specs/`. Read the architecturally-load-bearing ones fully, scan the rest. Flag canon-contradictions + stale framing (esp. data-tier, CI, policy, isolation standards).'),
  lane('products-localization', 'Review products/ (34) + localization-packs/ (14) + prds/: vs the product/vertical/KR-pack rulings (canon #10/#11). Confirm global-canonical+localization-pack framing; flag any product doc that contradicts the vertical scope, the cloud-dogfood layering, or KR-first-pack.'),
  lane('governance-process', 'Review governance-lanes/ (65) + checklists/ (31) + templates/ (29) + release/ (28) + teams/ (40) + advanced-cicd/ (39) + automation/ (19) + agents/ (11) + onboarding/ (12): vs canon (esp. forge/CI/Jenkins/Forgejo, foundry→governance rename, the lanes/wave framing, masterplan-generated). High foundry+Jenkins density expected here.'),
  lane('runbooks', 'Scan runbooks/ (207): `ls ' + SRC + '/runbooks/`. These are operational; grep-driven for canon-conflicts (foundry/Jenkins/Forgejo/tenant-tier/Redis/Kafka) + sample-read the infra/CI/data runbooks. Flag stale operational procedures that assume the wrong canonical tech.'),
  lane('journeys-personas-scan', 'HIGH-VOLUME lane: user-journeys/ (913) + personas/ (131) = 1044 docs — do NOT read each. Use bash grep to find every one mentioning a stale-canon term (foundry/Jenkins/Forgejo/tenant-tier/tier-system/M0-M3/MVP/Redis/Kafka/native-default). Report the counts + a sampled deep-read of ~8 representative hits to judge whether these are mechanical-rename or genuine-contradiction. BE HONEST: this is a scan, not a full read; state coverage explicitly (no silent truncation).'),
  lane('linux-nonadr', 'Review the LINUX pilot non-ADR docs: ' + LNX + '/context/*.md (cloud-native-stack, component-boundaries, conformance-gates, engineering-conventions, rust-engineering-guardrails, roadmap, testing-strategy, phase2/3-context, source-parity, migration-*), ' + LNX + '/research/*, ' + LNX + '/migration/*. Flag contradictions vs the ruled canon + stale pilot framing (these migrate into source, so they must match canon).'),
]
const laneResults = (await parallel(lanes)).filter(Boolean)
const laneDigests = laneResults.map(function (r, i) { return '--- lane[' + i + '] ---\n' + r }).join('\n\n')

// ---------- Synthesis ----------
phase('Synthesis')
const synth = await agent(
  'You are the WF2 SYNTHESIS lead. ' + CANON + '\n' +
  'Merge the footprint + all lane artifacts (open ' + OUT + '/10-stale-term-footprint.md and ' + OUT + '/20-*.md) into ONE rest-of-docs findings register for the amendment phase.\n' +
  'Footprint digest:\n' + footprint + '\n\nLane digests:\n' + laneDigests + '\n\n' +
  'WRITE ' + OUT + '/00-REST-OF-DOCS-REGISTER.md with: (1) CANON-CONTRADICTIONS (ranked; each: doc, the contradicting claim, the canon item it violates, the fix); (2) the MECHANICAL STALE-TERM sweep plan (foundry→intelligence/governance split rule, tenant-tier→tenant-class, M0-M3→gate-waves, Redis→Valkey, Kafka→Pulsar, Jenkins/Forgejo handling — with counts + the per-context rename rule); (3) AI-SLOP / stale / plain-wrong docs (candidates for delete/rewrite); (4) REFINEMENT opportunities; (5) REACHABILITY classification summary (which docs are ORPHAN→not-needed); (6) coverage honesty (what was deep-read vs scanned vs counted-only — no silent truncation). ' +
  'RETURN the ranked canon-contradictions + the mechanical-sweep totals + the coverage-honesty statement verbatim.',
  { label: 'sweep:synthesis', phase: 'Synthesis', model: 'opus' }
)

return {
  footprint: OUT + '/10-stale-term-footprint.md',
  lanes_done: laneResults.length,
  register: OUT + '/00-REST-OF-DOCS-REGISTER.md',
  summary: synth,
}
