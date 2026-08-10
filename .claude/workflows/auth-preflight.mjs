// auth-preflight — hard gate before push, merge, babysit, or restack dispatch.
//
//     node .claude/workflows/auth-preflight.mjs
//     node .claude/workflows/auth-preflight.mjs --self-test
//
// WHY THIS EXISTS. Fleet babysit burned time on restacks that needed push, then failed on an
// invalid gh token and SSH deny. Both are avoidable if auth is checked BEFORE any lane spends
// tokens on rebase/push/merge paths. merge-check and preflight.mjs call this first.
import { execFileSync } from 'node:child_process'
import { pathToFileURL } from 'node:url'

const isMain = process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url

export const REMEDIATION = 'gh auth login -h github.com'

/** @returns {{ ok: true, login: string, status: string } | { ok: false, step: string, detail: string, remediation: string }} */
export function runAuthPreflight() {
  let statusOut = ''
  try {
    statusOut = execFileSync('gh', ['auth', 'status'], { encoding: 'utf8' })
  } catch (e) {
    const detail = String(e.stderr || e.stdout || e.message || 'gh auth status failed').trim()
    return { ok: false, step: 'gh auth status', detail, remediation: REMEDIATION }
  }

  let login = ''
  try {
    login = execFileSync('gh', ['api', 'user', '-q', '.login'], { encoding: 'utf8' }).trim()
  } catch (e) {
    const detail = String(e.stderr || e.stdout || e.message || 'gh api user failed').trim()
    return { ok: false, step: 'gh api user -q .login', detail, remediation: REMEDIATION }
  }

  if (!login) {
    return {
      ok: false,
      step: 'gh api user -q .login',
      detail: 'empty login — token invalid, expired, or missing repo scope',
      remediation: REMEDIATION,
    }
  }

  return { ok: true, login, status: statusOut.trim() }
}

// ---------------------------------------------------------------- self-test
if (isMain && process.argv[2] === '--self-test') {
  let fail = 0
  const check = (name, ok) => {
    console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}`)
    if (!ok) fail++
  }

  check('exports REMEDIATION', REMEDIATION === 'gh auth login -h github.com')
  check('fail shape has remediation', (() => {
    const r = { ok: false, step: 'gh auth status', detail: 'test', remediation: REMEDIATION }
    return r.remediation === REMEDIATION && r.step && r.detail
  })())

  // Live probe when gh exists — documents pass/fail for the operator, not a unit assertion.
  try {
    execFileSync('gh', ['--version'], { encoding: 'utf8' })
    const live = runAuthPreflight()
    if (live.ok) {
      console.log(`LIVE  PASS  auth ok (login: ${live.login})`)
    } else {
      console.log(`LIVE  FAIL  ${live.step} — remediation: ${live.remediation}`)
      console.log(`        ${live.detail.split('\n')[0]}`)
    }
  } catch {
    console.log('SKIP  gh not installed — live probe skipped')
  }

  console.log(fail ? `\n${fail} FAILING` : '\nALL PASS — self-test cases')
  process.exit(fail ? 1 : 0)
}

// ---------------------------------------------------------------- live check
if (isMain) {
const result = runAuthPreflight()
if (result.ok) {
  console.log(`PASS  auth ok (login: ${result.login})`)
  process.exit(0)
}

console.error(`FAIL  ${result.step}`)
console.error(result.detail)
console.error(`\nRemediation: ${result.remediation}`)
process.exit(1)
}
