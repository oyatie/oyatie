export const meta = {
  name: 'foundry-routing-service-investigation',
  description: 'READ-ONLY investigation of the real source services to ground the foundry-rename routing + governance-validity question before L2. Inspect what oya/intelligence, cloud/cloud-intelligence, and oya/governance ACTUALLY are and do (real impls vs stale shells), determine where the absorbed foundry AI-agent-platform belongs (oya-intelligence vs cloud-intelligence), and whether governance is still a valid live service. Output: evidence + a founder interview agenda.',
  phases: [
    { title: 'Inspect', detail: '3 parallel read-only service deep-dives' },
    { title: 'Verdict', detail: 'head-to-head routing + governance-validity + interview agenda (opus)' },
  ],
}

const SRC = '/Users/jasonlee/Developer/source'
const OUT = '/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/_execution/routing-investigation'
const RULE = 'READ-ONLY. Never edit any source file. Report what the service ACTUALLY is and does, grounded in real files (ls the dir, read README/manifest/PRD, SAMPLE crate sources — lib.rs + key modules — to judge real-implementation vs stale-shell). Cite file paths. Be honest where a crate is an empty/placeholder shell vs a real impl. Write ONLY your one artifact.'

phase('Inspect')
const lanes = [
  { key: 'oya-intelligence', dir: SRC + '/oya/intelligence', what: 'oya/intelligence — the PRODUCT AI substrate (ADR-0255 two-layer, ADR-0220 consumer-intelligence). Does it hold the AI-agent-PLATFORM primitives the foundry had (model gateway, provider/account adapters, capability registry, MCP gateway, eval harness, guardrails, RAG, per-tenant context, autonomy ceiling)? List its crates + what each does + real-vs-shell. Note the oya-intelligence-* crate families (adapters: anthropic/codex/etc.; account; etc.).' },
  { key: 'cloud-intelligence', dir: SRC + '/cloud/cloud-intelligence', what: 'cloud/cloud-intelligence — the CLOUD AI layer (ADR-0389 bedrock-pattern cloud-primitive, ADR-0390 request-pipeline+proof-layer). Is this an INFERENCE GATEWAY (LLM serving / request pipeline) distinct from the product substrate? Does it hold agent-platform primitives, or just inference/gateway? List crates + real-vs-shell. Note oya-cloud-intelligence-* families (codex-adapter, authz-cedar, eventsink-clickhouse, app).' },
  { key: 'oya-governance', dir: SRC + '/oya/governance', what: 'oya/governance — the policy/quality/fitness AUTHORITY (ADR-0363 "stays its own service"; ADR-0347 foundry-fitness->governance). Are the oya-governance-* crates (abuse-defence, acl-enforcement, admission, ...) REAL working implementations or stale shells? What do they actually DO? Is governance a genuinely-valid live service, or decision-debt? How does it relate to oya-ci execution (does governance DEFINE gates that oya-ci RUNS, or is it redundant with oya-ci)? Count real-vs-shell crates.' },
]
const found = await parallel(lanes.map(function (l) {
  return function () {
    return agent(
      RULE + '\n\nSERVICE: ' + l.what + '\nSTART: `ls -R ' + l.dir + ' | head -80`, then read ' + l.dir + '/README.md + manifest.json/PRD if present, then SAMPLE 4-6 representative crate lib.rs/modules to judge real-vs-shell. ' +
      'WRITE ' + OUT + '/10-' + l.key + '.md (what it is, what it does, crate inventory with real-vs-shell flag, its actual primitives/responsibilities, and — for the foundry question — whether the AI-agent-platform primitives live here). RETURN a tight digest (purpose + real-vs-shell verdict + does-it-hold-agent-platform-primitives).',
      { label: 'inv:' + l.key, phase: 'Inspect', model: 'opus' }
    )
  }
}))
const dig = found.map(function (r, i) { return '--- ' + lanes[i].key + ' ---\n' + (r || '(failed)') }).join('\n\n')

phase('Verdict')
const verdict = await agent(
  RULE.replace('Write ONLY your one artifact.', '') + '\nYou are the ROUTING + GOVERNANCE-VALIDITY synthesizer. Inputs (open the 3 artifacts in ' + OUT + ' for detail):\n' + dig + '\n\n' +
  'Answer, with evidence: (1) WHERE does the absorbed foundry AI-agent-PLATFORM belong — `oya/intelligence` (product substrate) or `cloud/cloud-intelligence` (cloud inference layer)? Compare the two head-to-head on which actually holds/should-hold the agent-platform primitives (gateway, provider adapters, capability registry, MCP, eval, autonomy, RAG). The founder leans cloud-intelligence — test that against the evidence: is cloud-intelligence the right home, or is it the inference gateway with the product substrate in oya/intelligence? State the boundary crisply. (2) Is `oya/governance` a VALID live service or stale debt — what do its crates actually do, and is it distinct-and-needed vs redundant-with-oya-ci? (3) What is the corrected foundry-rename routing given all this? ' +
  'WRITE ' + OUT + '/00-ROUTING-VERDICT.md (the evidence-based routing + governance verdict) + ' + OUT + '/01-INTERVIEW-AGENDA.md (the crisp founder questions this raises, with options + a recommendation each). RETURN the routing recommendation + governance verdict + the interview questions verbatim.',
  { label: 'inv:verdict', phase: 'Verdict', model: 'opus' }
)

return { artifacts: OUT, summary: verdict }
