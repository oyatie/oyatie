# G002 Intake Collision Map — Oyatie Ultragoal

Created: 2026-06-26  
Goal: `G002-m0-trunk-and-active-pr-intake`  
Aggregate goal context: active Codex goal for `.omx/ultragoal/goals.json` / `.omx/ultragoal/ledger.jsonl`  
Plan source: `.omx/plans/ralplan-complete-ultragoal-through-team-ledger-full-replan.md`  
Raw evidence: `.omx/context/g002-intake/raw-inventory-20260626T2247Z.txt`  
Inventory summary: `.omx/context/g002-intake/inventory-summary-20260626T2248Z.json`  
Codex goal snapshot: `.omx/ultragoal/checkpoints/get-goal-active-20260626T2246Z.json`

## Executive decision

Do **not** launch broad Team fanout yet.

The approved next execution unit is G002 intake/reconciliation. Current evidence shows:

- root checkout is dirty and behind `origin/dev` by 202 commits;
- no open PRs target `dev`;
- no local live `.omx/state/team` directories exist;
- 92 registered git worktrees exist, including many stale-looking detached/private-tmp or prior-lane worktrees;
- stale worktree deletion is potentially destructive and must not be done blindly from this intake artifact.

## Fresh evidence snapshot

```text
HEAD:       d705932d4
origin/dev: 490311b9f
ahead/behind HEAD...origin/dev: 0 202
open PRs targeting dev: 0
registered worktrees: 92
local .omx/state/team dirs: 0
```

## Main checkout dirt

```text
## dev...origin/dev [behind 202]
 M .codex/hooks.json
 D goal.json
 D slice06-backfill-results.log
 D slice06-backfill-test.log
 D slice06-buck-test.log
 D slice06-generator.log
 D slice06-progress.log
 D slice06-retest-results.log
 D slice06-retest.log
?? cloud/cloud-intelligence/.omc/
?? specs/capability-registry.json
```

Classification:

| Path/surface | Classification | Action |
| --- | --- | --- |
| `.codex/hooks.json` | pre-existing local config drift; potentially user/runtime-owned | Do not overwrite in Team fanout. Inspect only in a dedicated hygiene lane. |
| `goal.json`, `slice06-*.log` deletions | pre-existing scratch/log cleanup state | Treat as Lane 1 hygiene; verify references before deletion is committed. |
| `cloud/cloud-intelligence/.omc/` | untracked runtime/tool state under source tree | Fence or ignore only after owner check; do not let workers touch by default. |
| `specs/capability-registry.json` | untracked spec-like file; possible authoritative surface | Hot until provenance is verified. Leader-only. |

## Worktree collision inventory

Summary from `git worktree list --porcelain`:

| Category | Count | Collision risk |
| --- | ---: | --- |
| Total registered worktrees | 92 | High until mapped. |
| Detached worktrees | 33 | Likely review/probe leftovers; removal still requires dirty-state check. |
| Branch worktrees | 59 | Must not delete without branch/PR/merge verification. |
| Under repo `.claude/worktrees` | 22 | Local agent/runtime lanes; high collision risk with root checkout. |
| `/private/tmp` worktrees | 8 | Likely stale review/probe worktrees; verify clean before pruning. |
| `/Users/jasonlee/oyatie-worktrees` | 60 | Prior OMX/agent lanes; treat as stale until each lane is proven closed. |
| Developer sibling worktrees | 2 | Review worktrees; verify before prune. |

## Open PR inventory

`gh pr list --base dev --state open --limit 100` returned `[]`.

Implication: the next wave can start from fresh `origin/dev` after intake, but existing local worktrees may still contain unmerged local work and must be classified before destructive cleanup.

## Hot/serial surfaces

These remain leader-owned or single-writer only:

- `.omx/ultragoal/**`
- `.omx/state/team/**`
- `.omx/team-ledgers/**`
- `*.generated.json`
- `.github/workflows/**`
- `oya-ci.toml`
- `specs/root-hub-pointers.json`
- `docs/AGENTS.md`
- `specs/master-plan-sequencing.json`
- `specs/markdown-retirement-policy.json`
- `specs/capability-registry.json` until provenance is resolved
- branch-protection / required-status configuration
- root build/toolchain policy surfaces

## Next-wave lane admission rules

A lane may start only if it has:

1. a fresh isolated worktree from current `origin/dev`;
2. a single owner;
3. owned paths and avoid paths written before work begins;
4. no writes to hot/serial surfaces unless explicitly rebound by the leader;
5. a verification command list before implementation;
6. a PR target of `dev` with `oya-ci-required` as merge authority;
7. no generated-file hand edits.

## G002 remaining cleanup decisions

Non-destructive intake is complete enough to plan the next lane, but G002 is **not fully complete** until stale worktrees/branches are either:

- proven clean and pruned, or
- proven live and preserved, or
- documented as blocked with owner/provenance.

Destructive cleanup candidates must be checked with at least:

```sh
git -C <worktree> status --short --branch
git -C <worktree> log --oneline --decorate -5
git branch --contains <worktree-head> || true
gh pr list --head <branch> --state all --json number,state,mergedAt,url || true
```

No automated deletion was performed by this artifact.

## Admission verdict

- **G002 intake/reconciliation:** in progress; collision map established.
- **G015 Team fanout:** blocked until `.omx/team-ledgers/g015-wave-a-team-ledger.md` is accepted and stale worktree cleanup/provenance is resolved enough to avoid collision.
- **Safe immediate next action:** classify/prune stale worktrees in a leader-owned hygiene pass, or launch only a read-only verifier lane to classify worktrees with no deletions.

## Read-only worktree classification (2026-06-26T22:50Z)

Classification evidence: `.omx/context/g002-intake/worktree-classification-20260626T2250Z.json`

| Class | Count | Meaning |
| --- | ---: | --- |
| `leader-root-dirty-behind` | 1 | Current leader checkout; dirty and behind origin/dev. |
| `preserve-dirty-investigate` | 18 | Has uncommitted changes; never prune without owner/provenance check. |
| `detached-clean-not-in-origin-dev` | 11 | Detached and clean, but head is not proven in origin/dev; preserve until provenance is checked. |
| `branch-clean-not-in-origin-dev` | 32 | Branch worktree clean, but head is not proven in origin/dev; preserve until branch/PR/merge provenance is checked. |
| `prune-candidate-clean-merged` | 30 | Clean and HEAD is ancestor of origin/dev; candidate for destructive git worktree remove/prune after approval. |

### Prune-candidate examples (not removed)

- `/Users/jasonlee/oyatie-worktrees/advisory-claude-reviewers-20260625T223854Z` — `advisory-claude-reviewers-20260625T223854Z` @ `d705932d4731`
- `/Users/jasonlee/oyatie-worktrees/python-mjs-rust-20260626T105028Z` — `python-mjs-rust-20260626T105028Z` @ `9fd3ac9c28a4`
- `/Users/jasonlee/oyatie-worktrees/python-mjs-rust-worker4-anchor-sweep-20260626T112327Z` — `worker-4-anchor-sweep-rust-20260626T112327Z` @ `859926a97107`
- `/Users/jasonlee/oyatie-worktrees/team-leader-ci12-20260626T044133Z` — `team-leader-ci12-20260626T044133Z` @ `a148ac3dce58`
- `/Users/jasonlee/oyatie-worktrees/team-leader-ci12-b-20260626T044133Z` — `team-leader-ci12-b-20260626T044133Z` @ `a148ac3dce58`
- `/Users/jasonlee/oyatie-worktrees/team-leader-ci12-c-20260626T044133Z` — `team-leader-ci12-c-20260626T044133Z` @ `a148ac3dce58`
- `/Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z` — `team-leader-ci12-c2-20260626T045700Z` @ `a148ac3dce58`
- `/Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z/.omx/team/group-c2-opus-xhigh-c-9f982c4a/worktrees/worker-1` — `detached` @ `a148ac3dce58`
- `/Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z/.omx/team/group-c2-opus-xhigh-c-9f982c4a/worktrees/worker-2` — `detached` @ `a148ac3dce58`
- `/Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z/.omx/team/group-c2-opus-xhigh-c-9f982c4a/worktrees/worker-3` — `detached` @ `a148ac3dce58`
- `/Users/jasonlee/oyatie-worktrees/team-leader-ci12-c2-20260626T045700Z/.omx/team/group-c2-opus-xhigh-c-9f982c4a/worktrees/worker-4` — `detached` @ `a148ac3dce58`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625` — `team-leader-wave-a-20260625` @ `d5bdd7bfdd96`

### Preserve/investigate examples

#### preserve-dirty-investigate
- `/private/tmp/claude-501/-Users-jasonlee-Developer-oyatie/0eea760e-659a-43a5-a5fa-31225a3931a5/scratchpad/wt-session` — `cloud-intel-session-pinning` @ `3a90fd1e6bc8`, dirty_count=46
- `/private/tmp/oya-firewall-0f0dPI/worktree` — `detached` @ `bf063080fcc9`, dirty_count=3
- `/private/tmp/oyatie-pr882-clean.m9mG55` — `detached` @ `df4f755fb598`, dirty_count=1
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-2` — `work-headroom-488-2` @ `7fa050225465`, dirty_count=1
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_f66bf3ee-51a-1` — `cloud-intel-g0-billing-canary` @ `3a4a9238bee5`, dirty_count=1
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-1` — `mail-mailbox-organize-usecase-slice` @ `83e6a7066aa2`, dirty_count=13
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-10` — `compliance-control-evidence-usecase-slice` @ `83e6a7066aa2`, dirty_count=7
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-2` — `comms-messenger-stream-ordering-usecase` @ `83e6a7066aa2`, dirty_count=11

#### detached-clean-not-in-origin-dev
- `/private/tmp/oya-pr882-probe-90412` — `detached` @ `e17ac9a24b66`, dirty_count=0
- `/private/tmp/oyatie-pr881-review.WFwSdx` — `detached` @ `f3ff09b1458a`, dirty_count=0
- `/private/tmp/oyatie-pr882-clean-review` — `detached` @ `e829bab48d50`, dirty_count=0
- `/private/tmp/oyatie-pr884-review` — `detached` @ `38eded923375`, dirty_count=0
- `/private/tmp/oyatie-pr893-wt` — `detached` @ `6fca8ea580c8`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie-pr883-review` — `detached` @ `016aa1f35d81`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/pr-repair-python-mjs-rust/pr907` — `detached` @ `b7a22b72c5ea`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/pr-repair-python-mjs-rust/pr908` — `detached` @ `370fc4703716`, dirty_count=0

#### branch-clean-not-in-origin-dev
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-1` — `cloud-intel-gemini-oauth` @ `47b7a8f0935a`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-10` — `cloud-intel-principal-verifier` @ `923631b60c80`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-2` — `cloud-intel-codex-class-headers` @ `d5d4e8174335`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-3` — `cloud-intel-overage-guard` @ `dd44f147d818`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-4` — `cloud-intel-headroom-cooldown` @ `7fa050225465`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-5` — `cloud-intel-session-pinning` @ `3a90fd1e6bc8`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-6` — `cloud-intel-cost-pricebook` @ `253926c9aa1e`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-7` — `cloud-intel-retry-fallback` @ `ecbdd644ce1b`, dirty_count=0

### Cleanup hold

No worktree was removed. The 30 `prune-candidate-clean-merged` entries are candidates only; destructive pruning requires an explicit cleanup branch/approval or a narrow leader-owned destructive pass with per-path readback evidence.

## Prune/preserve follow-up plan

A non-destructive worktree cleanup plan was written to `.omx/team-ledgers/g002-worktree-prune-preserve-plan.md`.

Dry-run evidence: `.omx/context/g002-intake/worktree-prune-dry-run-20260626T2252Z.txt`.

`git worktree prune --dry-run --verbose` produced no stale administrative entries to prune. This does not remove or validate the 30 registered clean/merged worktree removal candidates; those still require explicit destructive cleanup authorization or a future leader-owned destructive pass.

## Root dirt classification

Root dirty-state classification was written to `.omx/team-ledgers/g002-root-dirt-classification.md`.

Key decisions:

- `.codex/hooks.json` drift is a dedicated Lane 1 hygiene candidate, not broad fanout material.
- `goal.json` and `slice06-*.log` deletions are cleanup candidates but require reference verification before commit.
- `specs/capability-registry.json` is hot governance/spec-like data and remains leader-only.
- `cloud/cloud-intelligence/.omc/` is runtime/tool state drift and must be fenced or owner-checked before cleanup.
