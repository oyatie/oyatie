export const meta = {
  name: 'monorepo-conformance-audit',
  description: 'AUDIT (read-only) whether the assumed-merged single monorepo conforms to source policies/shape/protocols. Extract source monorepo policies, then audit each sibling repo (linux stack, oyago, oyapy, office, claude, codex) in current state against them → per-repo conformance + reshape/rename-needed → conformance register feeding the consolidation ralplan. NOT execution; verifies the migration FITS before consolidation.',
  phases: [
    { title: 'Policies', detail: 'extract source monorepo policy checklist (read-only)' },
    { title: 'Audit', detail: '6 parallel per-repo conformance audits vs the checklist' },
    { title: 'Register', detail: 'conformance register + consolidation-gap synthesis (opus)' },
  ],
}

const SRC = '/Users/jasonlee/Developer/source'
const DEV = '/Users/jasonlee/Developer'
const OUT = '/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/monorepo-conformance'
const RULE = 'READ-ONLY. Never edit any file. Report against EVIDENCE (file paths, crate names, Cargo.toml contents). Cite. Write ONLY your one artifact.'

phase('Policies')
const policies = await agent(
  RULE + '\nExtract source\'s MONOREPO POLICIES / SHAPE / PROTOCOLS into a single conformance CHECKLIST every migrated repo must satisfy. Read the governing ADRs + invariants in ' + SRC + '/docs/decisions/ (ADR-0131 flat-microservice-layout / per-microservice-flat, ADR-0017 brand-naming + repo-layout, ADR-0056 rust-clean-architecture-bnf, ADR-0105 13-layer-enum + check-family, ADR-0092 workspace-dependency-seam, ADR-0212 buildability-doctrine, ADR-0392/0408 Buck2 canonical build, ADR-0211 in-house-tech-stack, ADR-0119 specs-flat-root) + ' + SRC + '/AGENTS.md + the root Cargo.toml shape. PRODUCE a numbered checklist: canonical homes ({oya,cloud}/<service>/crates/<crate> + libs/), oya-* prefix + package.name==basename, brand-residue FORBIDDEN list (oyaoffice/oyago/oyapy/kuberos/foundry-*/oyatie-*/talos-* codenames), one-root-workspace/no-nested-[workspace]/one-version, hexagonal kernel/adapter/app + 13-layer enum, no_std-kernels/std-adapters, Buck2 per-crate BUCK + reindeer, data_class on kernel fields, deny.toml license policy, doctest=false/workspace-inherited-version, vendored-excluded. WRITE ' + OUT + '/00-policy-checklist.md. RETURN the numbered checklist (terse).',
  { label: 'conf:policies', phase: 'Policies', model: 'opus' }
)

phase('Audit')
const repos = [
  { key: 'linux-stack', path: DEV + '/linux/stack', note: 'the pilot: kernel (no_std framekernel→cloud/cloud-kernel), operating-system (talos-* STD→cloud/cloud-node-os), kubernetes+containerd (139 crates: 44 ctrd_ + 95 k8s)' },
  { key: 'oyago', path: DEV + '/oyago', note: 'Go→Rust transpiler → oya/transpiler-go-to-rust; codename oyago-* FORBIDDEN' },
  { key: 'oyapy', path: DEV + '/oyapy', note: 'Py→Rust transpiler → oya/transpiler-python-to-rust; codename oyapy-* FORBIDDEN' },
  { key: 'office', path: DEV + '/office', note: 'OyaOffice → oya/office; crates oyaoffice-* FORBIDDEN → oya-office-*' },
  { key: 'claude', path: DEV + '/claude', note: 'Claude SDK → cloud/cloud-intelligence/...anthropic-claude-adapter; pkg claude-agent-sdk' },
  { key: 'codex', path: DEV + '/codex', note: 'Codex SDK → MERGE into cloud/cloud-intelligence/...codex-adapter; pkg openai-codex-sdk' },
]
const audits = await parallel(repos.map(function (r) {
  return function () {
    return agent(
      RULE + '\nAUDIT this sibling repo against the source monorepo policy checklist (' + OUT + '/00-policy-checklist.md — read it FIRST). REPO: ' + r.path + ' — ' + r.note +
      '\nStart: `ls ' + r.path + '` + find its Cargo.toml(s) + crate names. For EACH checklist item, judge CONFORMS / NEEDS-RESHAPE / NEEDS-RENAME / VIOLATES, with evidence (crate names, layout, package names, no_std-vs-std, nested-[workspace]?, brand residue, vendored dirs present). Identify the reshape/rename WORK to make it fit (e.g. oyaoffice-*→oya-office-*, target home {oya,cloud}/<service>/crates, hexagonal split, BUCK files, data_class). WRITE ' + OUT + '/10-' + r.key + '.md (per-checklist-item verdict + the fit-work needed). RETURN a tight digest: top conformance gaps + the rename/reshape needed + any blocker (e.g. missing source).',
      { label: 'conf:' + r.key, phase: 'Audit', model: 'opus' }
    )
  }
}))
const dig = audits.map(function (x, i) { return '--- ' + repos[i].key + ' ---\n' + (x || '(failed)') }).join('\n\n')

phase('Register')
const reg = await agent(
  RULE.replace('Write ONLY your one artifact.', '') + '\nYou are the CONFORMANCE-REGISTER synthesizer. Inputs (open ' + OUT + '/00-policy-checklist.md + the 6 ' + OUT + '/10-*.md):\n' + dig + '\n\n' +
  'PRODUCE ' + OUT + '/00-CONFORMANCE-REGISTER.md: (1) a MATRIX (repo × policy → CONFORMS/RESHAPE/RENAME/VIOLATES/BLOCKER); (2) the per-repo reshape+rename work to FIT the monorepo (the conformance debt each migration lane must clear); (3) does the WIP migration plan\'s conformance gates (brand-residue scan, canonical-homes, package.name==basename, one-workspace, Buck2) COVER all the gaps, or are there gaps the ralplan must add?; (4) any hard BLOCKERS (db-engine missing, no_std framekernel workspace-exclusion, nested-workspace conflicts, vendored-tree stripping); (5) the verdict — is the assumed-merged monorepo conformance achievable by the existing unified/WIP plan, or does it need ralplan revision before consolidation? ' +
  'RETURN the matrix summary + the top conformance gaps + whether the WIP plan covers them + the verdict.',
  { label: 'conf:register', phase: 'Register', model: 'opus' }
)

return { artifacts: OUT, summary: reg }
