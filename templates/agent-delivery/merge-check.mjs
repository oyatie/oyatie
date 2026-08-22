// merge-check — the five conditions that must ALL hold before merging a PR.
//
//     node templates/agent-delivery/merge-check.mjs <pr>      check one PR
//     node templates/agent-delivery/merge-check.mjs --self-test
//
// WHY THIS EXISTS AS A SCRIPT RATHER THAN A REMEMBERED CHECKLIST. Two failures on 2026-08-09, in
// OPPOSITE directions, on the same signal:
//
//   1. `mergeStateStatus: CLEAN` and `reviewDecision` are BOTH blind to a blocking review here.
//      GitHub refuses --request-changes on your own PR, so a genuine BLOCKING verdict is filed as
//      COMMENTED and reviewDecision stays empty — indistinguishable from "nobody reviewed". PR
//      #1627 carried a real BLOCKING verdict and read as unreviewed.
//   2. The obvious fix — grep review bodies for "blocking" — reported FIVE blocking reviews on
//      #1623, whose reviews all said "Looks safe to merge ... NO BLOCKING ISSUES". The pattern
//      matched the negation and inverted its meaning, and would have blocked a clean PR forever.
//
// So BLOCKING_RE matches VERDICT LINES and imperatives, never the bare word — and the self-test at
// the bottom PROVES it separates the two cases in both directions. A check nobody has seen
// distinguish its cases is the false green it exists to prevent, which is the whole lesson of the
// day this file was written.
import { execFileSync } from 'node:child_process'
import { runAuthPreflight } from './auth-preflight.mjs'

const REPO = 'jason931225/oyatie'

// Matches a stated verdict or an imperative. Deliberately NOT the bare word "blocking":
// "no blocking issues", "not blocking" and "non-blocking" are approvals and must not match.
export const BLOCKING_RE =
  /VERDICT:\s*BLOCKING|REQUEST(ING)? CHANGES|must not (be )?merge|do not merge|BLOCKER:/i

const gh = args => execFileSync('gh', args, { encoding: 'utf8', maxBuffer: 1 << 26 }).trim()
const q = (n, f) => gh(['pr', 'view', String(n), '--repo', REPO, '--json', f, '--jq', arguments])

// ---------------------------------------------------------------- self-test
if (process.argv[2] === '--self-test') {
  const cases = [
    // must MATCH — real blocking verdicts
    ['real verdict line', 'VERDICT: BLOCKING', true],
    ['requesting changes', 'I am REQUESTING CHANGES on this PR', true],
    ['imperative', 'This must not merge until the bypass is closed', true],
    ['BLOCKER prefix', 'BLOCKER: the guard counts occurrences', true],
    // must NOT match — the negations that produced the false positive on #1623
    ['no blocking issues', 'Looks safe to merge — no coupling regressions and no blocking issues', false],
    ['not blocking', 'This is a nit and is not blocking', false],
    ['non-blocking advisory', 'Filed as a non-blocking advisory finding', false],
    ['plain approval', 'LGTM, no concerns', false],
  ]
  let fail = 0
  for (const [desc, text, want] of cases) {
    const got = BLOCKING_RE.test(text)
    console.log(`${got === want ? 'PASS' : 'FAIL'}  ${desc}`)
    if (got !== want) fail++
  }
  console.log(fail ? `\n${fail} FAILING` : `\nALL PASS — ${cases.length} cases, both directions`)
  process.exit(fail ? 1 : 0)
}

// ---------------------------------------------------------------- live check
const auth = runAuthPreflight()
if (!auth.ok) {
  console.error(`FAIL  ${auth.step}`)
  console.error(auth.detail)
  console.error(`\nRemediation: ${auth.remediation}`)
  process.exit(1)
}

const pr = process.argv[2]
if (!pr) { console.error('usage: merge-check.mjs <pr> | --self-test'); process.exit(2) }

const json = (n, field) => JSON.parse(gh(['pr', 'view', String(n), '--repo', REPO, '--json', field]))
const say = (k, v, ok) => console.log(`${ok ? 'ok  ' : 'FAIL'}  ${k.padEnd(28)} ${v}`)
let bad = 0
const check = (k, v, ok) => { say(k, v, ok); if (!ok) bad++ }

// 1. The required context resolved against the EXACT head — a rollup lags a fresh push.
const head = json(pr, 'headRefOid').headRefOid
const runs = JSON.parse(gh(['api', `repos/${REPO}/commits/${head}/check-runs`]))
const req = (runs.check_runs || []).find(r => r.name === 'presubmit')
check('required context', req ? `${req.status}/${req.conclusion}` : 'NOT RUN YET', req?.conclusion === 'success')

// 2. No failing check anywhere in the rollup.
const roll = json(pr, 'statusCheckRollup').statusCheckRollup || []
const failing = roll.filter(c => (c.conclusion || c.state) === 'FAILURE')
check('failing checks', String(failing.length), failing.length === 0)

// 3. required_conversation_resolution is on for dev, so ANY unresolved thread blocks.
const threads = JSON.parse(gh(['api', 'graphql', '-f',
  `query={repository(owner:"jason931225",name:"oyatie"){pullRequest(number:${pr}){reviewThreads(first:100){nodes{isResolved}}}}}`]))
const open = threads.data.repository.pullRequest.reviewThreads.nodes.filter(n => !n.isResolved).length
check('unresolved threads', String(open), open === 0)

// 4. A blocking VERDICT in a review body — the signal neither CLEAN nor reviewDecision can see.
const reviews = json(pr, 'reviews').reviews || []
const blocking = reviews.filter(r => BLOCKING_RE.test(r.body || ''))
check('blocking review verdicts', String(blocking.length), blocking.length === 0)
for (const b of blocking) console.log(`        └─ ${(b.body || '').replace(/\s+/g, ' ').slice(0, 120)}`)

// 5. dev requires signatures; an unverified commit blocks with no useful forge message.
const commits = JSON.parse(gh(['api', `repos/${REPO}/pulls/${pr}/commits`, '--paginate']))
const unsigned = commits.filter(c => c.commit.verification.verified === false)
check('unverified commits', String(unsigned.length), unsigned.length === 0)

console.log(bad ? `\nPR #${pr}: NOT MERGEABLE` : `\nPR #${pr}: all five conditions hold — safe to merge`)
process.exit(bad ? 1 : 0)
