export const meta = {
  name: 'prelane-0.5-truthing-manifests',
  description: 'WIP pre-lane 0.5 (READ-ONLY): produce the source/merge-surface manifests that gate consolidation — tools/ gate-target set (G2), per-lane source inventory + deny-globs, k8s/containerd 139-crate split + cloud-k8s relationship (G4), merge-surface diffs (codex-adapter, managed-k8s), the 12-kernel-subtree exclude list (for 0.6), oya-ci-required characterization + DOC-CATALOG location. Surfaces G2/G4 for the founder. NO source mutation, no build dry-run.',
  phases: [
    { title: 'Manifests', detail: '6 parallel read-only manifest lanes' },
    { title: 'Synthesis', detail: '0.5 manifest bundle + G2/G4 decisions surfaced (opus)' },
  ],
}

const SRC = '/Users/jasonlee/Developer/source'
const DEV = '/Users/jasonlee/Developer'
const OUT = '/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/_execution/prelane-0.5'
const RULE = 'READ-ONLY (no edits, no builds/dry-runs). Use `ls`/grep/read + `gh api` for live dev state. Report against evidence (paths, crate names, counts). Write ONLY your one artifact.'

phase('Manifests')
const lanes = [
  { key: 'tools-targets', body: 'G2 — enumerate every `//tools/...` and `//services/...` target the live merge-gate workflow builds, so retirement EXCLUDES them (standing canonical-homes exception). Read ' + SRC + '/.github/workflows/*.yml (esp. the github-lane-unlocker workflow) + grep for `//tools/`/`//services/` build targets + the WIP plan\'s named ones (oya-doc-staleness-inventory-app, oya-adr-index-regenerator-app). Confirm which gate-load-bearing tools/ targets must stay. WRITE ' + OUT + '/10-tools-targets.md (the keep-set + any others found). RETURN the tools/ standing-exception set.' },
  { key: 'source-inventory', body: 'Per-migration-lane SOURCE INVENTORY: for each of office (' + DEV + '/office), oyago (' + DEV + '/oyago), oyapy (' + DEV + '/oyapy), claude (' + DEV + '/claude), codex (' + DEV + '/codex), linux-stack (' + DEV + '/linux/stack), record (source path, first-party crate allowlist [names], per-tree DENY-GLOBS [_upstream*/third-party/vendor/target/buck-out/prelude/toolchains/__pycache__/.omc/.omx/.claude/legacy-*/talos-reference]). WRITE ' + OUT + '/10-source-inventory.md. RETURN per-repo (path, #first-party-crates, deny-globs).' },
  { key: 'k8s-containerd-split', body: 'G4 — k8s/containerd 139-crate SPLIT + cloud-k8s relationship. In ' + DEV + '/linux/stack/kubernetes/crates: classify each crate as k8s-MERGE (~95) vs containerd-CREATE (~44 `ctrd_*`) vs vendored-exclude. Then resolve the `cloud/cloud-k8s` relationship: `ls ' + SRC + '/cloud/cloud-k8s` + the 4 `cloud/managed-k8s-*` services — is cloud-k8s a 6th merge target / docs-only / out-of-scope? WRITE ' + OUT + '/10-k8s-split.md (the per-crate split count + the cloud-k8s verdict-options). RETURN the 95/44/vendored counts + the cloud-k8s question for the founder.' },
  { key: 'merge-surfaces', body: 'MERGE-surface diffs for the MERGE lanes. (a) codex-adapter: does `' + SRC + '/cloud/cloud-intelligence/crates/oya-cloud-intelligence-codex-adapter` exist + what\'s in it (the L5 merge target)? compare to ' + DEV + '/codex (pkg openai-codex-sdk). (b) managed-k8s: the 4 `' + SRC + '/cloud/managed-k8s-*` services — what crates exist to merge the linux k8s into (L6)? WRITE ' + OUT + '/10-merge-surfaces.md. RETURN what exists to merge into + the surface deltas.' },
  { key: 'kernel-exclude', body: 'For pre-lane 0.6 — the no_std EXCLUDE set. Confirm the 12 kernel-subtree `[workspace]` manifests in ' + DEV + '/linux/stack/kernel (framekernel + the 9 user-*-src ELF test targets + fsbase-worker-src + tests-host) are the full no_std exclude list (per D-CONFORM). List each + confirm it builds on the kernel\'s own nightly/custom-target (not the STD root). WRITE ' + OUT + '/10-kernel-exclude.md (the 12-entry exclude list for the root Cargo.toml [workspace] exclude key). RETURN the 12 exclude paths.' },
  { key: 'gate-characterize', body: 'Characterize the FLIP target + doc location. (a) `oya-ci-required`: find its producer crate (' + SRC + '/oya/ci-controller/... or cloud-ci) + what it posts/checks (for flip-readiness). (b) Does the live gate read ROOT vs `docs/` DOC-CATALOG.md/CHANGELOG.md (where do amendment lanes add rows)? Use `gh api` for the live dev required-contexts + read .github/branch-protection.yaml. WRITE ' + OUT + '/10-gate-characterize.md. RETURN the oya-ci-required producer + the DOC-CATALOG/CHANGELOG location.' },
]
const res = await parallel(lanes.map(function (l) {
  return function () { return agent(RULE + '\n\nLANE: ' + l.body, { label: '0.5:' + l.key, phase: 'Manifests', model: 'opus' }) }
}))
const dig = res.map(function (x, i) { return '--- ' + lanes[i].key + ' ---\n' + (x || '(failed)') }).join('\n\n')

phase('Synthesis')
const synth = await agent(
  RULE.replace('Write ONLY your one artifact.', '') + '\nSynthesize the pre-lane 0.5 manifest bundle. Inputs (open the 6 ' + OUT + '/10-*.md):\n' + dig + '\n\n' +
  'WRITE ' + OUT + '/00-PRELANE-0.5-MANIFESTS.md: (1) the tools/ standing-exception set (G2); (2) per-lane source inventory + deny-globs; (3) k8s/containerd split + cloud-k8s relationship (G4); (4) merge-surfaces (codex/managed-k8s); (5) the 12-kernel-subtree exclude list (feeds 0.6); (6) oya-ci-required characterization + DOC-CATALOG location; (7) the **G2/G4 DECISIONS SURFACED FOR THE FOUNDER** (tools/-exception ratify; db-engine confirm/drop-L8; cloud-k8s 6th-surface disposition; any codename/surface ambiguity) + which are now answered by the manifests vs still need founder input. ' +
  'RETURN the manifest headlines + the explicit G2/G4 founder decisions still open.',
  { label: '0.5:synthesis', phase: 'Synthesis', model: 'opus' }
)

return { artifacts: OUT, summary: synth }
