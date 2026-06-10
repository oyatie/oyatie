export const meta = {
  name: 'ralplan-amendment-phase-consensus',
  description: 'Consensus planning (Planner -> Architect -> Critic, deliberate mode) for the oyatie amendment phase. Produces AMENDMENT-PLAN.md (staged parallel lanes, verification gates, source-regimen PR/sign-off points, ralph loop structure) marked pending-approval. NO execution.',
  phases: [
    { title: 'Plan', detail: 'Planner drafts the staged parallel-lanes amendment plan + RALPLAN-DR + pre-mortem' },
    { title: 'Consensus', detail: 'Architect steelman -> Critic verdict, loop until APPROVE (max 3)' },
  ],
}

const WS = '/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06'
const DR = WS + '/synthesis/decision-record-oyatie-canon.md'
const PLAN = WS + '/AMENDMENT-PLAN.md'
const REGS = 'Registers (open as needed): ' + WS + '/synthesis/01-ADR-DISPOSITION-TABLE.md, ' + WS + '/synthesis/03-PROPOSED-RESOLUTION-LEDGER.md, ' + WS + '/synthesis/04-DOMAIN-TAXONOMY.md, ' + WS + '/docs-sweep/00-REST-OF-DOCS-REGISTER.md, ' + WS + '/bominal-reconciliation/00b-INTERVIEW-AGENDA-CORRECTED.md, ' + WS + '/legacy-recovery/00-RECOVERY-REGISTER.md'

const CONTEXT = 'AMENDMENT PHASE = apply the ~30 ruled founder decisions (SSOT: ' + DR + ') to the company monorepo source (GitHub jason931225/oyatie), NOT the linux pilot. Source regimen: ADR-0365 automated-lifecycle (research+consensus -> generative ADR -> oya gen propagate -> gates -> PR to dev -> door:one-way founder sign-off); masterplan GENERATED (oya gen masterplan + drift gate); supersede-never-edit ADR immutability; signed commits + linear history; DOC-CATALOG/CHANGELOG rows; one-doc-per-PR; glossary cascade. TOOLING REALITY: oya gen masterplan + gates exist only in .claude/worktrees (in-flight, not on dev) — flag aspirational-vs-enforced per step. Build-first-cutover-later (Jenkins/Argo stay operative). Verify-at-each-step (separate verifier lane vs primary sources, no phantom findings). D-LANES: everything is parallel lanes; within a lane tasks may be sequential or parallel. WORK: (L1) ADR re-foundation 372->clean ADR-0000+ generative series + 132 Proposed ratify/drop + renumber linux 0001-0026->0515+ + dedup 0377; (L2) foundry per-file rename 831 files sense-routed (intelligence vs governance) + HARD carve-outs (Palantir-Foundry 43, Marlboro-Forge, retirement-record); (L3) integrity sweep (KCMVP/KISA, tautologies, dangling refs); (L4) 13 canon-contradiction fixes CC-1..CC-13; (L5) new/reshaped ADRs (oya-ci reshape-0513, unified safety-gate, KR employment enum, infra-sovereignty schedule, domain-cohesion meta-ADR, masterplan-generated wiring); (L6) vocabulary namespacing tier->*; (L7) owed-depth capture + restorations (KR payroll packs, First-Proof-Slice+M3, released-view, Connect DEK; drop Train; Law/Finance deferred) + vertical-coverage map.'

phase('Plan')
const plan = await agent(
  'You are the PLANNER (deliberate mode, high-risk). ' + CONTEXT + '\n' + REGS + '\n' +
  'Read the decision record (' + DR + ') fully + skim the registers. WRITE ' + PLAN + ' — a staged, parallel-lanes, ralph-ready amendment execution plan with: (a) RALPLAN-DR summary at top (3-5 Principles, top-3 Decision Drivers, >=2 Viable Options for the overall sequencing strategy with bounded pros/cons + invalidation rationale if one wins); (b) the LANE DECOMPOSITION (L1-L7 above as parallel lanes) with per-lane sequential-vs-parallel task breakdown; (c) DEPENDENCY EDGES between lanes (e.g. L1 re-foundation gates the domain-field; L2 foundry-rename must precede ADR-0000+ re-author to avoid double-work; integrity sweep L3 before backfill); (d) VERIFICATION GATES per lane (separate verifier vs primary sources); (e) SOURCE-REGIMEN PR/sign-off points (one-doc-per-PR, door:one-way founder sign-off, where aspirational tooling means a step is manual); (f) where FOUNDER CREDENTIALS / sign-off / GitHub auth are required; (g) the RALPH LOOP structure (what each loop iteration does, the exit criteria); (h) DELIBERATE-MODE: a pre-mortem (3 failure scenarios + mitigations — e.g. blind foundry-swap breaks Palantir refs; ADR renumber breaks cross-refs; masterplan-generator-not-on-dev blocks backfill) + an expanded VERIFICATION plan (per-lane: what evidence proves done). Mark the doc "STATUS: pending approval (door:one-way founder sign-off)". RETURN a tight digest of the lane plan + the RALPLAN-DR.',
  { label: 'ralplan:planner', phase: 'Plan', model: 'opus' }
)

phase('Consensus')
let verdict = 'ITERATE'
let lastCritic = ''
let lastArch = ''
for (let round = 1; round <= 3; round++) {
  const arch = await agent(
    'You are the ARCHITECT reviewing the amendment plan at ' + PLAN + ' (round ' + round + '). ' + CONTEXT + '\n' +
    'Read the plan fully. Provide: the STRONGEST steelman ANTITHESIS (the best argument the plan is wrong/risky), at least one real TRADEOFF TENSION, principle-violation flags, and (where possible) a SYNTHESIS that improves it. Focus on: the lane dependency edges (is the ordering sound? does L2 foundry-rename really need to precede L1 re-foundation, or fight it?), the build-first-cutover-later sequencing, the aspirational-tooling risk (generator not on dev), and the verify-at-each-step adequacy. WRITE ' + WS + '/_arch-round' + round + '.md. RETURN your review digest.',
    { label: 'ralplan:architect-r' + round, phase: 'Consensus', model: 'opus' }
  )
  lastArch = arch
  const critic = await agent(
    'You are the CRITIC evaluating the amendment plan at ' + PLAN + ' against the Architect review (' + WS + '/_arch-round' + round + '.md), round ' + round + '. ' + CONTEXT + '\n' +
    'Enforce: principle-option consistency, fair alternatives, risk-mitigation clarity, TESTABLE acceptance criteria, concrete verification steps, and (deliberate mode) a real pre-mortem + expanded verification plan — REJECT if missing/weak. Read the plan + the arch review. WRITE ' + WS + '/_critic-round' + round + '.md with your findings. End your RETURN with a final line exactly: "VERDICT: APPROVE" or "VERDICT: ITERATE" or "VERDICT: REJECT".',
    { label: 'ralplan:critic-r' + round, phase: 'Consensus', model: 'opus' }
  )
  lastCritic = critic
  if (critic.indexOf('VERDICT: APPROVE') !== -1) { verdict = 'APPROVE'; break }
  verdict = critic.indexOf('VERDICT: REJECT') !== -1 ? 'REJECT' : 'ITERATE'
  if (round < 3) {
    await agent(
      'You are the PLANNER revising the amendment plan at ' + PLAN + ' (round ' + round + ' feedback). ' + CONTEXT + '\n' +
      'Read the current plan + the Architect review (' + WS + '/_arch-round' + round + '.md) + the Critic findings (' + WS + '/_critic-round' + round + '.md). REVISE ' + PLAN + ' in place to address every blocking point (incorporate the synthesis, fix the dependency ordering, strengthen the pre-mortem/verification). Keep "STATUS: pending approval". RETURN a digest of what you changed.',
      { label: 'ralplan:revise-r' + round, phase: 'Consensus', model: 'opus' }
    )
  }
}

return {
  plan_path: PLAN,
  final_verdict: verdict,
  planner_digest: plan,
  architect_last: lastArch,
  critic_last: lastCritic,
}
