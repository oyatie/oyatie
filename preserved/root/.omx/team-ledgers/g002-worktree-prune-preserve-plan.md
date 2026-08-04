# G002 Worktree Prune/Preserve Plan

Created: 2026-06-26
Evidence: `.omx/context/g002-intake/worktree-classification-20260626T2250Z.json`

## Rule

This is a **non-destructive** plan. No `git worktree remove`, `rm -rf`, or branch deletion has been run. Destructive cleanup requires explicit authorization or a future leader-owned destructive cleanup lane with per-path readback evidence.

## Summary

| Class | Count | Decision |
| --- | ---: | --- |
| `prune-candidate-clean-merged` | 30 | Candidate for removal after destructive approval; clean and HEAD is ancestor of origin/dev. |
| `preserve-dirty-investigate` | 18 | Preserve; dirty/uncommitted changes need owner/provenance. |
| `detached-clean-not-in-origin-dev` | 11 | Preserve; detached head not proven in origin/dev. |
| `branch-clean-not-in-origin-dev` | 32 | Preserve; branch head not proven in origin/dev. |
| `leader-root-dirty-behind` | 1 | Preserve; current leader checkout. |

## Candidate removal list (not executed)

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
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/execute-team-bound-ul-9c35bb10/worktrees/worker-1` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/execute-team-bound-ul-9c35bb10/worktrees/worker-3` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/execute-team-bound-ul-9c35bb10/worktrees/worker-4` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/execute-team-bound-ul-9c35bb10/worktrees/worker-5` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-1` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-2` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-3` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-4` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-back-fixed-g015-9c35bb10/worktrees/worker-5` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-1` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-2` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-3` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-4` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/wavea-front-fixed-g01-9c35bb10/worktrees/worker-5` — `detached` @ `d5bdd7bfdd96`
- `/Users/jasonlee/oyatie-worktrees/team-leader-waveb-pr-wip-20260626T023204Z` — `team-leader-waveb-pr-wip-20260626T023204Z` @ `980af9bf615d`
- `/Users/jasonlee/oyatie-worktrees/waveA-hr-payroll-20260625173629` — `detached` @ `8ed7c55f16a3`
- `/Users/jasonlee/oyatie-worktrees/waveA-kernel-os-20260625173629` — `waveB-kernel-os-backlog-20260626` @ `8ed7c55f16a3`
- `/Users/jasonlee/oyatie-worktrees/waveA-market-billing-20260625173629` — `detached` @ `8ed7c55f16a3`

## Preserve/investigate list

### preserve-dirty-investigate
- `/private/tmp/claude-501/-Users-jasonlee-Developer-oyatie/0eea760e-659a-43a5-a5fa-31225a3931a5/scratchpad/wt-session` — `cloud-intel-session-pinning` @ `3a90fd1e6bc8`, dirty_count=46
- `/private/tmp/oya-firewall-0f0dPI/worktree` — `detached` @ `bf063080fcc9`, dirty_count=3
- `/private/tmp/oyatie-pr882-clean.m9mG55` — `detached` @ `df4f755fb598`, dirty_count=1
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-2` — `work-headroom-488-2` @ `7fa050225465`, dirty_count=1
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_f66bf3ee-51a-1` — `cloud-intel-g0-billing-canary` @ `3a4a9238bee5`, dirty_count=1
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-1` — `mail-mailbox-organize-usecase-slice` @ `83e6a7066aa2`, dirty_count=13
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-10` — `compliance-control-evidence-usecase-slice` @ `83e6a7066aa2`, dirty_count=7
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-2` — `comms-messenger-stream-ordering-usecase` @ `83e6a7066aa2`, dirty_count=11
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-3` — `comms-contact-center-core-slice` @ `83e6a7066aa2`, dirty_count=19
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-4` — `worktree-wf_5056f9ff-2b6-4` @ `83e6a7066aa2`, dirty_count=12
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-5` — `billing-accounting-journal-usecase` @ `83e6a7066aa2`, dirty_count=9
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-6` — `billing-finops-budget-alert-585` @ `83e6a7066aa2`, dirty_count=14
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-7` — `data-pipeline-core-ports-slice` @ `83e6a7066aa2`, dirty_count=18
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-8` — `data-warehouse-table-retention-core-adr0587` @ `83e6a7066aa2`, dirty_count=13
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_5056f9ff-2b6-9` — `worktree-wf_5056f9ff-2b6-9` @ `83e6a7066aa2`, dirty_count=12
- `/Users/jasonlee/oyatie-worktrees/claude-archive-20260625/wf_6e664cf7-e06-2` — `milestone-view` @ `648c490e488f`, dirty_count=5
- `/Users/jasonlee/oyatie-worktrees/p3-observability` — `p3-observability` @ `f2cf13a792c6`, dirty_count=38
- `/Users/jasonlee/oyatie-worktrees/pr811-catalog-rekey` — `pr811-rekey` @ `cfdbf1b6e5cb`, dirty_count=6

### detached-clean-not-in-origin-dev
- `/private/tmp/oya-pr882-probe-90412` — `detached` @ `e17ac9a24b66`, dirty_count=0
- `/private/tmp/oyatie-pr881-review.WFwSdx` — `detached` @ `f3ff09b1458a`, dirty_count=0
- `/private/tmp/oyatie-pr882-clean-review` — `detached` @ `e829bab48d50`, dirty_count=0
- `/private/tmp/oyatie-pr884-review` — `detached` @ `38eded923375`, dirty_count=0
- `/private/tmp/oyatie-pr893-wt` — `detached` @ `6fca8ea580c8`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie-pr883-review` — `detached` @ `016aa1f35d81`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/pr-repair-python-mjs-rust/pr907` — `detached` @ `b7a22b72c5ea`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/pr-repair-python-mjs-rust/pr908` — `detached` @ `370fc4703716`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/pr-repair-python-mjs-rust/pr909` — `detached` @ `cc8ba3c8d4f6`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/pr-repair-python-mjs-rust/pr910` — `detached` @ `cfcad19eb844`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/team-leader-wave-a-20260625/.omx/team/execute-team-bound-ul-9c35bb10/worktrees/worker-2` — `detached` @ `173ff8c8deb3`, dirty_count=0

### branch-clean-not-in-origin-dev
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-1` — `cloud-intel-gemini-oauth` @ `47b7a8f0935a`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-10` — `cloud-intel-principal-verifier` @ `923631b60c80`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-2` — `cloud-intel-codex-class-headers` @ `d5d4e8174335`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-3` — `cloud-intel-overage-guard` @ `dd44f147d818`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-4` — `cloud-intel-headroom-cooldown` @ `7fa050225465`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-5` — `cloud-intel-session-pinning` @ `3a90fd1e6bc8`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-6` — `cloud-intel-cost-pricebook` @ `253926c9aa1e`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-7` — `cloud-intel-retry-fallback` @ `ecbdd644ce1b`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-8` — `cloud-intel-capability-registry` @ `8b1fe4f20cf8`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_10e3e92b-3cd-9` — `cloud-intel-guardrail-port` @ `6c54cad86da2`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-1` — `work-class-headers` @ `ca7ef3e9e6ac`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-3` — `work` @ `30fbc35150da`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-4` — `worktree-wf_a6c6939e-488-4` @ `760a97bff885`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-5` — `worktree-wf_a6c6939e-488-5` @ `6c54cad86da2`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_a6c6939e-488-6` — `work-principal-verifier` @ `b6b131e2286b`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_b306f25c-be3-1` — `cloud-intel-codex-oauth-wiring` @ `41c592b59ffa`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_c4dfe460-b55-1` — `cloud-intel-t2a-wire-types` @ `902ac1e1d0d8`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_c4dfe460-b55-2` — `cloud-intel-t2b-error-class` @ `22806d4383e4`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_c4dfe460-b55-3` — `cloud-intel-t1-tls-reachability` @ `55e68d819e1b`, dirty_count=0
- `/Users/jasonlee/Developer/oyatie/.claude/worktrees/wf_c4dfe460-b55-4` — `cloud-intel-t4-metrics` @ `fc234e2e4484`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/ci-pr-cancel-20260626T025326Z` — `ci-pr-cancel-stale-runs-20260626T025326Z` @ `b745953530d9`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/python-mjs-rust-20260626T105334Z` — `python-mjs-rust-20260626T105334Z` @ `73bf7214ed27`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/team-leader-ci-velocity-20260626T043322Z` — `team-leader-ci-velocity-20260626T043322Z` @ `2123b9ee4d13`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/team-python-mjs-rust-leader-20260626T105958Z` — `team-python-mjs-rust-leader-20260626T105958Z` @ `f93ceb80b89a`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/ultragoal-wave-a-20260626T203951Z` — `ultragoal-wave-a-20260626T203951Z` @ `268e170c3591`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/waveA-ast-transpiler-20260625173629` — `waveB-ast-transpiler-followup-202606260000` @ `643156e505ca`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/waveA-cloud-ci-20260625173629` — `waveB-cloud-ci-873-20260626` @ `e17ac9a24b66`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/waveA-collab-office-20260625173629` — `waveA-collab-office-issue864-202606260018` @ `3b88d0806e60`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/waveA-crm-marketing-20260625173629` — `waveB-crm-descriptor-metadata-878` @ `05a492ca3a80`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/waveA-erp-20260625173629` — `waveA-erp-blockers-20260625` @ `998819e5cf68`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/waveA-iac-k8s-20260625173629` — `waveB-iac-transport-773-20260626` @ `312debb2910e`, dirty_count=0
- `/Users/jasonlee/oyatie-worktrees/waveA-kms-iam-20260625173629` — `waveB-pdp-same-tenant-879` @ `0658568895b0`, dirty_count=0

### leader-root-dirty-behind
- `/Users/jasonlee/Developer/oyatie` — `dev` @ `d705932d4731`, dirty_count=11

## If destructive cleanup is later authorized

For each candidate:

```sh
git -C <worktree> status --short --branch
git -C <worktree> merge-base --is-ancestor HEAD refs/remotes/origin/dev
git worktree remove <worktree>
git worktree prune --verbose
git worktree list --porcelain
```

Do not remove entries outside `prune-candidate-clean-merged` without a separate provenance decision.
