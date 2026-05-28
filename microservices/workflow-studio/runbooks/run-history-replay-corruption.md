---
doc_class: Runbook
title: Run History Replay Corruption
status: Accepted
date: 2026-05-20
microservice: workflow-studio
severity: sev2
audience: workflow-studio-on-call
owner_team: axis-workflow-studio + ops-sre-reliability
source_wave: codex-runbooks-substrate-w3
change_scope: substance rewrite of existing thin runbook
doc_status: published
---

# Runbook: Run History Replay Corruption

## Operator Contract
- Runbook id: workflow-studio-run-history-replay-corruption.
- Primary service namespace: `workflow-studio`.
- Owning rotation: PagerDuty oya-workflow-studio-primary; collab-runtime-secondary.
- Incident channel: `#inc-workflow-studio`.
- Operational focus: run history replay panel shows corrupted or missing execution edges.
- Named precedent: this follows the Figma multiplayer canvas plus Google Docs CRDT convergence pattern.
- External dependencies: Cloudflare CDN support; BrowserStack enterprise support; OpenAI enterprise support.
- API authority: `https://workflow-studio.internal.oyatie.dev/v1/workflow-studio/run-history-replay-corruption/incident-handoff`.
- Audit event class: `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `RunHistoryReplayCorruptionCritical` is green, and every Cross-microservice handoff API returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/workflow-studio-run-history-replay-corruption-<incident-id>.md`.

## Trigger Conditions
- Page on alert `RunHistoryReplayCorruptionCritical` when `oya_workflow_studio_run_history_replay_corruption_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `RunHistoryReplayCorruptionSloBurn` when `oya_workflow_studio_run_history_replay_corruption_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open sev2 if `oya_workflow_studio_run_history_corruption_total` exceeds the threshold documented in `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`.
- Open sev2 if `oya_workflow_studio_run_history_replay_corruption_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `workflow-studio.run-history-replay-corruption.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate workflow-studio-run-history-replay-corruption --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/workflow-studio-ops/run-history-replay-corruption?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` backed by `microservices/workflow-studio/dashboards/canvas-perf.json`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/workflow-studio-ops/run-history-replay-corruption?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202` backed by `microservices/workflow-studio/dashboards/collab-health.json`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="workflow-studio",runbook="run-history-replay-corruption"}`.
- Alertmanager route: `oyatie-workflow-studio-run-history-replay-corruption-critical`; silence only with incident commander approval and `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` evidence.
- Synthetic probe: `oya ops probe workflow-studio run-history-replay-corruption --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/workflow-studio/run-history-replay-corruption/expected-state.json` hash differs from live `https://workflow-studio.internal.oyatie.dev/v1/workflow-studio/run-history-replay-corruption/admin/state-hash`.
- Service-specific metric `oya_workflow_studio_run_history_corruption_total` is red while `oya_workflow_studio_run_history_replay_corruption_audit_emit_total{status="sealed"}` is flat.

## Symptoms
- User-facing impact: builders may see stale canvas state, invalid node graphs, or degraded assisted generation output; scenario focus is run history replay panel shows corrupted or missing execution edges.
- Operators see Grafana panel `canvas-perf.json / Run History Replay Corruption burn rate` turn red before the primary alert resolves.
- Loki signature `workflow_studio.run_history_replay_corruption.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=RunHistoryReplayCorruptionDegraded` on deployment `workflow-studio-run-history-replay-corruption-worker` or `workflow-studio-api`.
- Audit-chain shows missing or delayed `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT --since 30m`.
- Metric pattern: `oya_workflow_studio_run_history_replay_corruption_error_ratio` rises before `oya_workflow_studio_run_history_replay_corruption_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_workflow_studio_run_history_replay_corruption_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_workflow_studio_run_history_replay_corruption_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `RunHistoryReplayCorruptionCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=run-history-replay-corruption.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=run-history-replay-corruption.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific pattern: `oya_workflow_studio_run_history_corruption_total` rises while `oya_workflow_studio_run_history_replay_corruption_dependency_error_ratio` is flat; inspect local state before escalating Cloudflare CDN support.
- Service-specific pattern: `oya_workflow_studio_run_history_replay_corruption_dependency_error_ratio` rises while `oya_workflow_studio_run_history_corruption_total` is flat; inspect vendor or adjacent-service dependency health before local rollback.

## Failure Mode Tree
- Failure mode 1: single-tenant WorkflowCanvas inconsistency; contain with tenant quarantine, preserve all `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` rows, and avoid fleet rollback.
- Failure mode 2: cross-cell NodeGraph drift; freeze writes, compare state hash across cells, and use audit-chain replay before accepting new mutations.
- Failure mode 3: byzantine or abusive principal; suspend the principal through identity, keep tenant data scoped, and preserve Cedar explain output.
- Failure mode 4: external dependency outage at Cloudflare CDN support; open vendor ticket only after local dashboards and handoff APIs prove the dependency is causal.
- Failure mode 5: operator mitigation made state worse; roll back feature flag `oya.workflow-studio.run_history_replay_corruption.incident_hold`, close `workflow-studio-run-history-replay-corruption-circuit-breaker`, and restore the previous deployment revision.
- Failure mode 6: audit emission is delayed; do not close even when customer symptoms improve because ADR-0263 evidence is incomplete.
- Failure mode 7: regional partition; keep prod-us-east-1 as evidence leader and reject cross-region mutation until `oya_workflow_studio_run_history_replay_corruption_state_hash_match == 1`.
- Failure mode 8: compliance-pack mismatch; require compliance handoff when KR-CSAP, EU-sovereign, FedRAMP-High, IL5, or CN-PIPL labels are present.
- Failure mode 9: stale dashboard data; verify direct Mimir queries before making rollback decisions.
- Failure mode 10: runbook step ambiguity; halt the ambiguous branch, emit `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` with outcome `blocked`, and patch this runbook after recovery.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-workflow-studio-run-history-replay-corruption-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://workflow-studio.internal.oyatie.dev/v1/workflow-studio/alerts?runbook=run-history-replay-corruption | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n workflow-studio rollout status deploy/workflow-studio-run-history-replay-corruption-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n workflow-studio get pods -l app=run-history-replay-corruption -o wide`.
5. Read structured logs: `kubectl -n workflow-studio logs deploy/workflow-studio-run-history-replay-corruption-worker --since=30m | rg "workflow_studio.run_history_replay_corruption.incident_state|RunHistoryReplayCorruptionCritical|EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="workflow-studio",runbook="run-history-replay-corruption"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_studio_run_history_replay_corruption_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_studio_run_history_replay_corruption_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_studio_run_history_replay_corruption_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_studio_run_history_corruption_total{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/workflow-studio-ops/run-history-replay-corruption?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/workflow-studio-ops/run-history-replay-corruption?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops workflow-studio run-history-replay-corruption status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate workflow-studio-run-history-replay-corruption --production-snapshot --cell $CELL`.
16. Run crate smoke test: `cargo test -p oya-workflow-studio-visual-canvas-kernel run_history_replay_corruption -- --nocapture`.
17. Check API contract smoke: `curl -s https://workflow-studio.internal.oyatie.dev/v1/workflow-studio/run-history-replay-corruption/incident-handoff -H "x-oya-tenant: $TENANT"`.
18. Inspect config: `test -f microservices/workflow-studio/iac/kustomize/base/kustomization.yaml && sed -n '1,180p' microservices/workflow-studio/iac/kustomize/base/kustomization.yaml`.
19. Inspect feature flags: `oya flags get oya.workflow-studio.run_history_replay_corruption.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status workflow-studio-run-history-replay-corruption-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n workflow-studio rollout history deploy/workflow-studio-run-history-replay-corruption-worker | tail -20`.
22. Check policy file: `test -f microservices/workflow-studio/policy/editor-isolation.md || find microservices/workflow-studio/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls microservices/workflow-studio/slos/*.openslo.yaml | sort | rg "canvas|collab"`.
24. Check contract binding: `test -f microservices/workflow-studio/contracts/openapi/workflow-studio.yaml && sed -n '1,120p' microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`.
25. Run targeted SQL state query: `psql $OYA_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from workflow_studio_run_history_replay_corruption_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric oya_workflow_studio_run_history_replay_corruption_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice workflow-studio --runbook run-history-replay-corruption --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Run History Replay Corruption incident decision tree
1. Is RunHistoryReplayCorruptionCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-workflow-studio-primary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_workflow_studio_run_history_replay_corruption_queue_depth grow while oya_workflow_studio_run_history_replay_corruption_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-workflow-studio, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed WorkflowCanvas correctness risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT`.
- Branch B (dependency saturation or replay backlog): use the matching mitigation block below and record `decision_branch=B` in `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT`.
- Branch C (policy, permit, or tenant-scope drift): use the matching mitigation block below and record `decision_branch=C` in `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT`.
- Branch D (customer-visible or regulated evidence gap): use the matching mitigation block below and record `decision_branch=D` in `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service workflow-studio --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-workflow-studio --severity sev2`.
3. Freeze risky automation: `oya flags set oya.workflow-studio.run_history_replay_corruption.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open workflow-studio-run-history-replay-corruption-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n workflow-studio scale deploy/workflow-studio-run-history-replay-corruption-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason workflow-studio-run-history-replay-corruption --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops workflow-studio run-history-replay-corruption drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops workflow-studio run-history-replay-corruption drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n workflow-studio rollout undo deploy/workflow-studio-run-history-replay-corruption-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n workflow-studio patch hpa workflow-studio-run-history-replay-corruption-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface workflow-studio.run-history-replay-corruption --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/workflow-studio/runbooks/run-history-replay-corruption.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice workflow-studio --incident $INCIDENT_ID --channel #inc-workflow-studio`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "Cloudflare CDN support" --incident $INCIDENT_ID --summary workflow-studio-run-history-replay-corruption`.
18. Confirm breaker effect: `oya ops breaker status workflow-studio-run-history-replay-corruption-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://workflow-studio.internal.oyatie.dev/v1/workflow-studio/run-history-replay-corruption/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=run-history-replay-corruption`.

### Mitigation Branch Guidance
- Branch A: confirmed WorkflowCanvas correctness risk.
  - Required action: keep `workflow-studio-run-history-replay-corruption-circuit-breaker` open until `oya_workflow_studio_run_history_replay_corruption_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-studio-ops/run-history-replay-corruption?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=110` to the incident.
  - Required audit: emit `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: dependency saturation or replay backlog.
  - Required action: keep `workflow-studio-run-history-replay-corruption-circuit-breaker` open until `oya_workflow_studio_run_history_replay_corruption_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-studio-ops/run-history-replay-corruption?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=111` to the incident.
  - Required audit: emit `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: policy, permit, or tenant-scope drift.
  - Required action: keep `workflow-studio-run-history-replay-corruption-circuit-breaker` open until `oya_workflow_studio_run_history_replay_corruption_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-studio-ops/run-history-replay-corruption?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=112` to the incident.
  - Required audit: emit `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer-visible or regulated evidence gap.
  - Required action: keep `workflow-studio-run-history-replay-corruption-circuit-breaker` open until `oya_workflow_studio_run_history_replay_corruption_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-studio-ops/run-history-replay-corruption?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=113` to the incident.
  - Required audit: emit `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "run_history_replay_corruption|RunHistoryReplayCorruptionCritical|workflow_studio.run_history_replay_corruption.incident_state" crates microservices/workflow-studio -g "!microservices/workflow-studio/runbooks/**"`.
2. Patch domain invariant: `edit oya-workflow-studio-visual-canvas-kernel where run_history_replay_corruption state transition is validated`.
3. Patch API guard: `edit microservices/workflow-studio/contracts/openapi/workflow-studio.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit microservices/workflow-studio/policy/editor-isolation.md with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit microservices/workflow-studio/iac/kustomize/base/kustomization.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-workflow-studio-visual-canvas-kernel run_history_replay_corruption_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate workflow-studio-run-history-replay-corruption --fixture incident-run-history-replay-corruption.json`.
8. Add SLO assertion: `update microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml with alert RunHistoryReplayCorruptionCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/workflow-studio/dashboards/canvas-perf.json with oya_workflow_studio_run_history_replay_corruption_error_ratio, oya_workflow_studio_run_history_replay_corruption_lag_seconds, and oya_workflow_studio_run_history_corruption_total`.
10. Rebuild affected crate: `cargo check -p oya-workflow-studio-visual-canvas-kernel --all-targets`.
11. Run targeted tests: `cargo test -p oya-workflow-studio-visual-canvas-kernel --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate workflow-studio-policy --microservice workflow-studio`.
13. Deploy canary: `oya deploy canary --microservice workflow-studio --component workflow-studio-run-history-replay-corruption-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_workflow_studio_run_history_replay_corruption_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close workflow-studio-run-history-replay-corruption-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.workflow-studio.run_history_replay_corruption.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=run-history-replay-corruption`.
19. Verify seal: `oya audit-chain verify --event-class EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-workflow-studio-visual-canvas-kernel`: inspect for `run_history_replay_corruption` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-workflow-studio-collab-crdt-domain`: inspect for `run_history_replay_corruption` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-workflow-studio-dsl-emitter-domain`: inspect for `run_history_replay_corruption` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-workflow-studio-node-library-registry-domain`: inspect for `run_history_replay_corruption` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/workflow-studio/contracts/openapi/workflow-studio.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/workflow-studio/contracts/asyncapi/workflow-studio-events.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/workflow-studio/contracts/proto/workflow-studio.proto`: verify request/response or event contract only when incident evidence points there.
- `microservices/workflow-studio/dashboards/canvas-perf.json`: verify panel coverage for `oya_workflow_studio_run_history_replay_corruption_error_ratio`, `oya_workflow_studio_run_history_replay_corruption_lag_seconds`, and `oya_workflow_studio_run_history_corruption_total`.
- `microservices/workflow-studio/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `microservices/workflow-studio/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `RunHistoryReplayCorruptionCritical` and `RunHistoryReplayCorruptionSloBurn` are both resolved in Alertmanager for 30 minutes.
- `oya_workflow_studio_run_history_replay_corruption_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `oya_workflow_studio_run_history_replay_corruption_lag_seconds < 120` for all production cells.
- `oya_workflow_studio_run_history_replay_corruption_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `oya_workflow_studio_run_history_corruption_total` is below the threshold documented in `microservices/workflow-studio/slos/canvas-frame-time-p99.openslo.yaml`.
- Dashboard `https://grafana.dev.oyatie.internal/d/workflow-studio-ops/run-history-replay-corruption?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` shows green panels for the affected cell.
- Audit-chain query for `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` returns mitigation and resolution events.
- Circuit breaker `workflow-studio-run-history-replay-corruption-circuit-breaker` is closed after rollback window.
- Feature flag `oya.workflow-studio.run_history_replay_corruption.incident_hold` is false for the affected tenant unless long-term hold is approved.
- Runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- Service owner acknowledged final handoff in `#inc-workflow-studio`.

## Capacity and Rollback Guardrails
- Capacity math: if `oya_workflow_studio_run_history_replay_corruption_queue_depth` is 5000 and the worker drains 25 items/second, the best-case drain is 200 seconds before retries; page earlier when drain time exceeds 300 seconds.
- Capacity math: with 12 replicas at 25 items/second each, the hard ceiling is 300 items/second; keep tenant throttle below 25 RPS until error ratio stays below 0.005.
- Rollback checkpoint 1: before changing `oya.workflow-studio.run_history_replay_corruption.incident_hold`, snapshot current value with `oya flags get oya.workflow-studio.run_history_replay_corruption.incident_hold --output json`.
- Rollback checkpoint 2: before opening `workflow-studio-run-history-replay-corruption-circuit-breaker`, capture `oya_workflow_studio_run_history_replay_corruption_request_rate` and `oya_workflow_studio_run_history_replay_corruption_success_ratio` from Mimir.
- Rollback checkpoint 3: before scaling deployments, capture `kubectl -n workflow-studio get deploy workflow-studio-run-history-replay-corruption-worker -o yaml`.
- Rollback command for flag: `oya flags set oya.workflow-studio.run_history_replay_corruption.incident_hold=false --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for breaker: `oya ops breaker close workflow-studio-run-history-replay-corruption-circuit-breaker --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for deployment: `kubectl -n workflow-studio rollout undo deploy/workflow-studio-run-history-replay-corruption-worker`.
- Rollback command for tenant throttle: `oya ops rate-limit clear --tenant $TENANT --surface workflow-studio.run-history-replay-corruption --reason rollback-$INCIDENT_ID`.
- Stop rollback if `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` cannot be emitted; preserve the current state and escalate to audit-chain before additional mutation.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: workflow-studio-run-history-replay-corruption
microservice: workflow-studio
event_class: EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT
incident_id: <INC-...>
severity: sev2
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Run History Replay Corruption postmortem

## Summary
- What happened in workflow-studio/run-history-replay-corruption.
- Who was affected: tenant_id list, cell_id list, user-facing surface list.
- Current status: mitigated, resolved, or monitoring.

## Timeline
- T0 detection: alert/customer/audit source.
- T1 acknowledgement: operator handle and channel.
- T2 mitigation: feature flag, breaker, rollback, or throttle.
- T3 resolution: code/config/policy fix.
- T4 verification: dashboard, metric, audit seal, customer confirmation.

## Root Cause
- Direct trigger.
- Contributing factors.
- Why existing controls did not catch it earlier.

## ADR-0263 Audit Emission Requirements
- Emit EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-workflow-studio-primary; collab-runtime-secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until the critical alert clears.
- Incident commander: first responder from axis-workflow-studio + ops-sre-reliability; transfer only by explicit message in `#inc-workflow-studio`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Cloudflare CDN support; BrowserStack enterprise support; OpenAI enterprise support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-workflow-studio-run-history-replay-corruption` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `workflow-engine`: `oya incident handoff --target workflow-engine --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `workflow-engine` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `marketplace`: `oya incident handoff --target marketplace --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID --severity sev2 --branch B`; expect `202 accepted`.
- Require `marketplace` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `identity`: `oya incident handoff --target identity --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID --severity sev2 --branch C`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID --severity sev2 --branch D`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID --severity sev2 --branch A`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source workflow-studio --runbook run-history-replay-corruption --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `oya_workflow_studio_run_history_replay_corruption_error_ratio`, `oya_workflow_studio_run_history_replay_corruption_lag_seconds`, `oya_workflow_studio_run_history_replay_corruption_queue_depth`, `oya_workflow_studio_run_history_corruption_total`, current breaker state, and audit seal status.
- Keep `workflow-studio-run-history-replay-corruption-circuit-breaker` owner as axis-workflow-studio + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_WORKFLOW_STUDIO_RUN_HISTORY_REPLAY_CORRUPTION_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/workflow-studio/dashboards/` for dashboard names and operational panels: canvas-perf.json, collab-health.json, copilot-quality.json, editor-experience.json.
- `microservices/workflow-studio/slos/` for OpenSLO alert vocabulary and threshold alignment: canvas-frame-time-p99.openslo.yaml, collab-crdt-merge-latency.openslo.yaml, collab-crdt-no-silent-loss.openslo.yaml, editor-rest-availability.openslo.yaml, editor-rest-latency.openslo.yaml, license-gate-cedar-availability.openslo.yaml.
- `microservices/workflow-studio/policy/` for named policy and authorization surfaces: editor-isolation.md, tenant-scope.cedar, data-residency.md, auditor-scope.cedar.
- `microservices/workflow-studio/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi/workflow-studio.yaml, contracts/asyncapi/workflow-studio-events.yaml, contracts/proto/workflow-studio.proto.
- `microservices/workflow-studio/manifest.json` for owner, dependency, capability, and bounded-context vocabulary; topic `run-history-replay-corruption` is the scenario anchor.

## Checkpoint Closure Criteria
- The runbook remains current when `RunHistoryReplayCorruptionCritical`, `RunHistoryReplayCorruptionSloBurn`, `oya_workflow_studio_run_history_corruption_total`, `oya.workflow-studio.run_history_replay_corruption.incident_hold`, and `workflow-studio-run-history-replay-corruption-circuit-breaker` all resolve to live telemetry, flag, or breaker records.
- The incident is cleanly halted if required authority is missing for tenant quarantine, policy rollback, or vendor escalation; do not improvise outside the named commands.
- The checkpoint is complete when `./bin/oya vcs verify --agent codex-runbooks-substrate-w3 --evidence 'runbooks_substance:X new_runbooks:Y' ...` accepts the five target scopes.
