# mm-delivery-kit task checklist

See session plan (first principles + gap close). No gjc/omc/omx/hermes.

## Parallel inherit (#1560 handoff)

- [x] Recover plan + goals/ledger snapshots from recovery ref; plan SHA verified
- [x] Write `programs/cas-fabric/INHERIT.md` + `PROGRAM.json`
- [x] `mm-goals` run `20260805T090132Z-2725e960` (6 goals; ultragoal NOT false-completed)
- [ ] G039: clean worktree from `origin/dev@a4a5ace5+`; diagnose #1558 red lanes
- [ ] #1559 post-merge CI run 30990439972 green → completion packet
- [ ] Do not touch #1561; #1541 security separate

## Phase 0 — Foundation
- [x] 0.1 Kit manifest (`harness/kit.v1.json`) + evaluate kit check
- [x] 0.2 PREFLIGHT reads `project-profile.vcs.base_ref` + profile-aware fetch
- [x] 0.3 Prompt compiler `mm-prompt` (lenses + role + authority)
- [x] Checkpoint 0: `mm-evaluate static` ≥ A; prompt golden A≠B

## Phase 1
- [x] 1.1 mm-sync-workflow-prompts + markers in lens-delivery-plan
- [ ] 1.2 Console bootstrap (**human gate**)
- [x] 1.3 Run journal schema (`run-layout.v1.json`)

## Phase 2 — Throughput
- [x] 2.1 parallelism.v1.json + mm-paths overlap checker
- [ ] 2.2 parallel critic spawn metrics
- [ ] 2.3 optional multi-shard EXECUTOR

## Phase 3 — Learn promotion
- [ ] 3.1 KPI repeat detector
- [ ] 3.2 Human-gated promote suggest

## Phase 4 — Real delivery
- [ ] 4.1 Revalidate false-green findings on base_ref
- [ ] 4.2 Live lens-delivery-plan
- [ ] 4.3 One execute slice + PR packet

## Phase 5
- [x] 5.1 mm-pipeline close-run
- [ ] 5.2 Docs consolidation
