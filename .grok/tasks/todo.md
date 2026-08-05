# mm-delivery-kit task checklist

See session plan (first principles + gap close). No gjc/omc/omx/hermes.

## Single pipeline (do not spawn new workflows)

- SSOT: `harness/pipeline.json` + `mm-pipeline` + `workflows/lens-delivery-plan.rhai`
- Improve by editing `harness/*.v1.json` / pipeline stages → score/grade/learn
- Anti-pattern: new workflow file per lane/program (archived prior sprawl under `workflows/archive/`)
- Workstreams (same pipeline): cas-g039-1558, k8s-w0a-1561, kit-self — see `pipeline.json#workstreams`

## Inherited programs (context, not separate pipelines)

- [x] #1560 CAS handoff → `programs/cas-fabric/`
- [x] #1561 k8s W0-A handoff → `programs/k8s-port/`
- [x] G002 diagnose #1558 complete
- [ ] G003 fix #1558 (OWNERS+reachability+gates) keep draft
- [ ] G001 k8s #1561 CI/review lifecycle keep draft
- [ ] #1559 post-merge CI completion packet when green
- [ ] #1541 awareness only

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
- [x] D4 briefs path-overlap (mm-drive briefs/tick/status; fail-closed)
- [x] 2.2 parallel critic spawn metrics
- [ ] 2.3 optional multi-shard EXECUTOR

## Drive outer loop (D3–D4)
- [x] D3 checkpoint-check + mm-goals wiring (fail-closed before goals mutate)
- [x] D3 terminal-goal / placeholder evidence heuristics
- [x] D4 path-overlap among parallel resolvable lanes
- [x] static evaluate dual-critic packet contract signal

## Phase 3 — Learn promotion
- [x] 3.1 KPI repeat detector (`mm-learn kpi-repeat` + from-run hook; config in learning-loop.v1.json)
- [ ] 3.2 Human-gated promote suggest (detector emits drafts; pack PR still human)

## Phase 4 — Real delivery
- [ ] 4.1 Revalidate false-green findings on base_ref
- [ ] 4.2 Live lens-delivery-plan
- [ ] 4.3 One execute slice + PR packet

## Phase 5
- [x] 5.1 mm-pipeline close-run
- [ ] 5.2 Docs consolidation
