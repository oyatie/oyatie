# Drive status — 2026-08-05 (agent dual-critic, no human APPROVE)

## Policy
- `harness/drive.v1.json` `merge_policy`: `require_agent_dual_critic=true`, `require_independent_approve=false`, `auto_undraft=true`
- Human GH APPROVE is **not** mandatory

## PRs

| PR | Title | State | Head | Dual-critic | CI | Next |
|----|-------|-------|------|-------------|-----|------|
| **#1558** | CAS proof-cell prerequisites | **MERGED** → `a1bd1f14a` | squash | APPROVE | post-merge run `30999838837` **queued** | Fill G039 packet when trunk green |
| **#1561** | k8s W0-A admit Go→Rust port | OPEN MERGEABLE | `c744a2f45` | **APPROVE** | run `31000488083` **queued** (postgres legs already SUCCESS earlier) | `mm-drive merge` when green |
| **#1562** | R2 path-filter live-postgres | OPEN draft MERGEABLE | `adfad9eaa` (rebased on #1558) | **APPROVE** | run `31000739951` pending/queued | undraft+merge when green |

## Parallel lanes

| Lane | Status |
|------|--------|
| R1 runners | **Queue-bound** — multiple `oya-ci-required` runs queued, **zero in_progress**. Human ops scale (runbook landed). |
| R2 #1562 | Dual-critic done; CI queue |
| R3 packets | Template + 1559 example; G039 DRAFT; G001 DRAFT |
| R5/G039 | Merged; not ultragoal-complete until trunk green + packet |
| R7/G001 | Dual-critic done; merge when CI green |
| W0-B/G002 | **Plan only** — `W0-B-ADMISSION-PLAN.md` + `W0-B-ready-gate.json`; product code hard-stopped until G001 packet |

## Evidence paths
- `.grok/programs/k8s-port/evidence/pr-1561-dual-critic.json`
- `.grok/programs/hyperscaler-delivery-lanes/evidence/pr-1562-dual-critic.json`
- `.grok/programs/cas-fabric/evidence/G039-post-merge-completion-packet-DRAFT.json`
- `.grok/programs/k8s-port/W0-B-ADMISSION-PLAN.md`

## Monitor
Background merge poller armed: when merge-check ok → `mm-drive merge --pr`.
