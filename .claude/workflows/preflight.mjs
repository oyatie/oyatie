// Preflight for the workflow suite. Run before dispatching anything:
//     node .claude/workflows/preflight.mjs
//
// Runs auth-preflight first (gh token must work before push/merge/babysit/restack paths).
//
// WHY THIS EXISTS. An unescaped backtick inside the shared CTX template literal does NOT throw.
// It silently ends the literal and turns the remainder into a TAGGED TEMPLATE — valid JavaScript,
// so `node --check` passes, while everything after the backtick quietly stops being part of the
// prompt the agents receive. The doctrine is still in the file and no longer in the message.
//
// That has now cost three separate dispatches. `node --check` cannot catch it because the file is
// syntactically fine; only the MEANING changed. So this checks meaning: it extracts each CTX body
// and asserts the text that should be in it actually is.
import fs from 'node:fs'
import path from 'node:path'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const HERE = path.dirname(fileURLToPath(import.meta.url))

const auth = spawnSync(process.execPath, [path.join(HERE, 'auth-preflight.mjs')], { stdio: 'inherit' })
if (auth.status !== 0) {
  console.error('\nauth-preflight FAILED — fix gh auth before dispatching any workflow')
  process.exit(auth.status || 1)
}
const FILES = fs.readdirSync(HERE).filter(f => f.endsWith('.js'))

let fail = 0
const check = (name, ok, detail) => {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${ok ? '' : ' :: ' + detail}`)
  if (!ok) fail++
}

// Strip ${...} interpolations, which legitimately contain their own nested backticks.
const stripInterpolations = body => {
  let out = '', depth = 0
  for (let i = 0; i < body.length; i++) {
    if (body[i] === '$' && body[i + 1] === '{') { depth++; i++; continue }
    if (depth > 0) { if (body[i] === '{') depth++; else if (body[i] === '}') depth--; continue }
    out += body[i]
  }
  return out
}

for (const f of FILES) {
  const src = fs.readFileSync(path.join(HERE, f), 'utf8')

  const i = src.indexOf('const CTX = `')
  if (i === -1) { check(`${f}: has a CTX block`, false, 'no `const CTX = \\`` found'); continue }
  const start = i + 'const CTX = `'.length
  const end = src.indexOf('\n`\n', start)
  check(`${f}: CTX literal is terminated`, end !== -1, 'no closing backtick on its own line')
  if (end === -1) continue

  const body = stripInterpolations(src.slice(start, end))
  const stray = [...body.matchAll(/(?<!\\)`/g)].length
  check(`${f}: no unescaped backtick inside CTX`, stray === 0,
    `${stray} stray backtick(s) — everything after the first one silently leaves the prompt`)

  // The doctrine paragraphs are appended at the END of CTX, so if a stray backtick truncated the
  // literal these markers fall outside it. Checking for them is the meaning-level assertion that
  // a syntax check cannot make.
  for (const marker of ['TRUST BOUNDARY', 'ONE BUCK2 CLIENT PER PROJECT ROOT', 'PROVE ZERO REGRESSIONS'])
    check(`${f}: CTX still carries "${marker}"`, body.includes(marker), 'marker fell outside the literal')

  // Process hardenings encoded 2026-08-10 — only workflows that enforce them carry these markers.
  if (f === 'deliver.js' || f === 'restack.js') {
    for (const marker of [
      'AUTH PREFLIGHT',
      'EQUALITY-PINNED CENSUS MERGE PROTOCOL',
      'TWO-ROUND RULE',
      'BEAD COUNTERS ARE NOT LIVE STATE',
    ])
      check(`${f}: CTX still carries "${marker}"`, body.includes(marker), 'marker fell outside the literal')
  }

  // meta must be a pure literal the host can read without evaluating the script.
  check(`${f}: declares meta.name`, /export const meta = \{[\s\S]{0,400}?name: '/.test(src), 'missing or non-literal meta.name')
  check(`${f}: tolerates string args`, src.includes("typeof raw === 'string'"),
    'args can arrive JSON-encoded; refusing then looks like a policy refusal, not a parse failure')
}

console.log(fail ? `\n${fail} FAILING — do not dispatch` : `\nALL PASS — ${FILES.length} workflow(s) safe to dispatch`)
process.exit(fail ? 1 : 0)
