---
doc_class: Runbook
title: Deadlock Resolution
status: Accepted
date: 2026-05-20
microservice: workflow-engine
severity: sev1
audience: workflow-runtime-on-call
owner_team: axis-workflow-engine + ops-sre-reliability
source_wave: codex-runbooks-substrate-w3
change_scope: substance rewrite of existing thin runbook
doc_status: published
---

# Runbook: Deadlock Resolution

## Operator Contract
- Runbook id: workflow-engine-deadlock-resolution.
- Primary service namespace: `workflow-engine`.
- Owning rotation: PagerDuty oya-workflow-engine-primary; workflow-runtime-secondary.
- Incident channel: `#inc-workflow-engine`.
- Operational focus: state-machine lock graph contains a circular tenant scoped wait.
- Named precedent: this follows the Temporal durable execution plus AWS Step Functions state-machine recovery pattern.
- External dependencies: CNCF Temporal support; Valkey commercial support; PostgreSQL Citus support.
- API authority: `https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/deadlock-resolution/incident-handoff`.
- Audit event class: `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `DeadlockResolutionCritical` is green, and every Cross-microservice handoff API returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/workflow-engine-deadlock-resolution-<incident-id>.md`.

## Trigger Conditions
- Page on alert `DeadlockResolutionCritical` when `oya_workflow_engine_deadlock_resolution_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `DeadlockResolutionSloBurn` when `oya_workflow_engine_deadlock_resolution_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open sev1 if `oya_workflow_engine_deadlock_wait_seconds` exceeds the threshold documented in `workflow/observability/slos/workflow-engine/replay-determinism-correctness.openslo.yaml`.
- Open sev1 if `oya_workflow_engine_deadlock_resolution_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `workflow-engine.deadlock-resolution.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate workflow-engine-deadlock-resolution --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/deadlock-resolution?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` backed by `workflow/workflow-engine/dashboards/durable-state-size.json`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/deadlock-resolution?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202` backed by `workflow/workflow-engine/dashboards/step-latency.json`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="workflow-engine",runbook="deadlock-resolution"}`.
- Alertmanager route: `oyatie-workflow-engine-deadlock-resolution-critical`; silence only with incident commander approval and `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` evidence.
- Synthetic probe: `oya ops probe workflow-engine deadlock-resolution --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/workflow-engine/deadlock-resolution/expected-state.json` hash differs from live `https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/deadlock-resolution/admin/state-hash`.
- Service-specific metric `oya_workflow_engine_deadlock_wait_seconds` is red while `oya_workflow_engine_deadlock_resolution_audit_emit_total{status="sealed"}` is flat.

## Symptoms
- User-facing impact: workflow runs may pause, duplicate, or skip durable state transitions for affected tenants; scenario focus is state-machine lock graph contains a circular tenant scoped wait.
- Operators see Grafana panel `durable-state-size.json / Deadlock Resolution burn rate` turn red before the primary alert resolves.
- Loki signature `workflow_engine.deadlock_resolution.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=DeadlockResolutionDegraded` on deployment `workflow-engine-deadlock-resolution-worker` or `workflow-engine-api`.
- Audit-chain shows missing or delayed `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT --since 30m`.
- Metric pattern: `oya_workflow_engine_deadlock_resolution_error_ratio` rises before `oya_workflow_engine_deadlock_resolution_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_workflow_engine_deadlock_resolution_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_workflow_engine_deadlock_resolution_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `DeadlockResolutionCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=deadlock-resolution.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=deadlock-resolution.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific pattern: `oya_workflow_engine_deadlock_wait_seconds` rises while `oya_workflow_engine_deadlock_resolution_dependency_error_ratio` is flat; inspect local state before escalating CNCF Temporal support.
- Service-specific pattern: `oya_workflow_engine_deadlock_resolution_dependency_error_ratio` rises while `oya_workflow_engine_deadlock_wait_seconds` is flat; inspect vendor or adjacent-service dependency health before local rollback.

## Failure Mode Tree
- Failure mode 1: single-tenant WorkflowRun inconsistency; contain with tenant quarantine, preserve all `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` rows, and avoid fleet rollback.
- Failure mode 2: cross-cell WorkflowSpec drift; freeze writes, compare state hash across cells, and use audit-chain replay before accepting new mutations.
- Failure mode 3: byzantine or abusive principal; suspend the principal through identity, keep tenant data scoped, and preserve Cedar explain output.
- Failure mode 4: external dependency outage at CNCF Temporal support; open vendor ticket only after local dashboards and handoff APIs prove the dependency is causal.
- Failure mode 5: operator mitigation made state worse; roll back feature flag `oya.workflow-engine.deadlock_resolution.incident_hold`, close `workflow-engine-deadlock-resolution-circuit-breaker`, and restore the previous deployment revision.
- Failure mode 6: audit emission is delayed; do not close even when customer symptoms improve because ADR-0263 evidence is incomplete.
- Failure mode 7: regional partition; keep prod-us-east-1 as evidence leader and reject cross-region mutation until `oya_workflow_engine_deadlock_resolution_state_hash_match == 1`.
- Failure mode 8: compliance-pack mismatch; require compliance handoff when KR-CSAP, EU-sovereign, FedRAMP-High, IL5, or CN-PIPL labels are present.
- Failure mode 9: stale dashboard data; verify direct Mimir queries before making rollback decisions.
- Failure mode 10: runbook step ambiguity; halt the ambiguous branch, emit `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` with outcome `blocked`, and patch this runbook after recovery.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-workflow-engine-deadlock-resolution-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/alerts?runbook=deadlock-resolution | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n workflow-engine rollout status deploy/workflow-engine-deadlock-resolution-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n workflow-engine get pods -l app=deadlock-resolution -o wide`.
5. Read structured logs: `kubectl -n workflow-engine logs deploy/workflow-engine-deadlock-resolution-worker --since=30m | rg "workflow_engine.deadlock_resolution.incident_state|DeadlockResolutionCritical|EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="workflow-engine",runbook="deadlock-resolution"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_engine_deadlock_resolution_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_engine_deadlock_resolution_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_engine_deadlock_resolution_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_engine_deadlock_wait_seconds{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/workflow-engine-ops/deadlock-resolution?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/workflow-engine-ops/deadlock-resolution?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops workflow-engine deadlock-resolution status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate workflow-engine-deadlock-resolution --production-snapshot --cell $CELL`.
16. Run crate smoke test: `cargo test -p oya-workflow-engine-execution-engine-domain deadlock_resolution -- --nocapture`.
17. Check API contract smoke: `curl -s https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/deadlock-resolution/incident-handoff -H "x-oya-tenant: $TENANT"`.
18. Inspect config: `test -f workflow/workflow-engine/iac/kustomize/base/kustomization.yaml && sed -n '1,180p' workflow/workflow-engine/iac/kustomize/base/kustomization.yaml`.
19. Inspect feature flags: `oya flags get oya.workflow-engine.deadlock_resolution.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status workflow-engine-deadlock-resolution-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n workflow-engine rollout history deploy/workflow-engine-deadlock-resolution-worker | tail -20`.
22. Check policy file: `test -f microservices/workflow-engine/policy/saga-compensation-policy.md || find workflow/workflow-engine/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls workflow/observability/slos/workflow-engine/*.openslo.yaml | sort | rg "replay|worker"`.
24. Check contract binding: `test -f workflow/workflow-engine/contracts/openapi/workflow-engine.yaml && sed -n '1,120p' workflow/workflow-engine/contracts/openapi/workflow-engine.yaml`.
25. Run targeted SQL state query: `psql $OYA_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from workflow_engine_deadlock_resolution_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric oya_workflow_engine_deadlock_resolution_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice workflow-engine --runbook deadlock-resolution --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Deadlock Resolution incident decision tree
1. Is DeadlockResolutionCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-workflow-engine-primary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_workflow_engine_deadlock_resolution_queue_depth grow while oya_workflow_engine_deadlock_resolution_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-workflow-engine, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed WorkflowRun correctness risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT`.
- Branch B (dependency saturation or replay backlog): use the matching mitigation block below and record `decision_branch=B` in `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT`.
- Branch C (policy, permit, or tenant-scope drift): use the matching mitigation block below and record `decision_branch=C` in `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT`.
- Branch D (customer-visible or regulated evidence gap): use the matching mitigation block below and record `decision_branch=D` in `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service workflow-engine --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-workflow-engine --severity sev1`.
3. Freeze risky automation: `oya flags set oya.workflow-engine.deadlock_resolution.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open workflow-engine-deadlock-resolution-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n workflow-engine scale deploy/workflow-engine-deadlock-resolution-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason workflow-engine-deadlock-resolution --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops workflow-engine deadlock-resolution drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops workflow-engine deadlock-resolution drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n workflow-engine rollout undo deploy/workflow-engine-deadlock-resolution-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n workflow-engine patch hpa workflow-engine-deadlock-resolution-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface workflow-engine.deadlock-resolution --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths workflow/workflow-engine/runbooks/deadlock-resolution.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice workflow-engine --incident $INCIDENT_ID --channel #inc-workflow-engine`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "CNCF Temporal support" --incident $INCIDENT_ID --summary workflow-engine-deadlock-resolution`.
18. Confirm breaker effect: `oya ops breaker status workflow-engine-deadlock-resolution-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/deadlock-resolution/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=deadlock-resolution`.

### Mitigation Branch Guidance
- Branch A: confirmed WorkflowRun correctness risk.
  - Required action: keep `workflow-engine-deadlock-resolution-circuit-breaker` open until `oya_workflow_engine_deadlock_resolution_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/deadlock-resolution?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=110` to the incident.
  - Required audit: emit `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: dependency saturation or replay backlog.
  - Required action: keep `workflow-engine-deadlock-resolution-circuit-breaker` open until `oya_workflow_engine_deadlock_resolution_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/deadlock-resolution?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=111` to the incident.
  - Required audit: emit `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: policy, permit, or tenant-scope drift.
  - Required action: keep `workflow-engine-deadlock-resolution-circuit-breaker` open until `oya_workflow_engine_deadlock_resolution_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/deadlock-resolution?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=112` to the incident.
  - Required audit: emit `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer-visible or regulated evidence gap.
  - Required action: keep `workflow-engine-deadlock-resolution-circuit-breaker` open until `oya_workflow_engine_deadlock_resolution_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/deadlock-resolution?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=113` to the incident.
  - Required audit: emit `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "deadlock_resolution|DeadlockResolutionCritical|workflow_engine.deadlock_resolution.incident_state" crates microservices/workflow-engine -g "!workflow/workflow-engine/runbooks/**"`.
2. Patch domain invariant: `edit oya-workflow-engine-execution-engine-domain where deadlock_resolution state transition is validated`.
3. Patch API guard: `edit workflow/workflow-engine/contracts/openapi/workflow-engine.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit microservices/workflow-engine/policy/saga-compensation-policy.md with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit workflow/workflow-engine/iac/kustomize/base/kustomization.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-workflow-engine-execution-engine-domain deadlock_resolution_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate workflow-engine-deadlock-resolution --fixture incident-deadlock-resolution.json`.
8. Add SLO assertion: `update workflow/observability/slos/workflow-engine/replay-determinism-correctness.openslo.yaml with alert DeadlockResolutionCritical when this was a missing alert`.
9. Add dashboard panel: `update workflow/workflow-engine/dashboards/durable-state-size.json with oya_workflow_engine_deadlock_resolution_error_ratio, oya_workflow_engine_deadlock_resolution_lag_seconds, and oya_workflow_engine_deadlock_wait_seconds`.
10. Rebuild affected crate: `cargo check -p oya-workflow-engine-execution-engine-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-workflow-engine-execution-engine-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate workflow-engine-policy --microservice workflow-engine`.
13. Deploy canary: `oya deploy canary --microservice workflow-engine --component workflow-engine-deadlock-resolution-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_workflow_engine_deadlock_resolution_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close workflow-engine-deadlock-resolution-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.workflow-engine.deadlock_resolution.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=deadlock-resolution`.
19. Verify seal: `oya audit-chain verify --event-class EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-workflow-engine-execution-engine-domain`: inspect for `deadlock_resolution` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-workflow-engine-state-machine-kernel`: inspect for `deadlock_resolution` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-workflow-engine-event-bus-worker`: inspect for `deadlock_resolution` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-workflow-engine-replay-debugger-backend-domain`: inspect for `deadlock_resolution` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `workflow/workflow-engine/contracts/openapi/workflow-engine.yaml`: verify request/response or event contract only when incident evidence points there.
- `workflow/workflow-engine/contracts/asyncapi/workflow-events.yaml`: verify request/response or event contract only when incident evidence points there.
- `workflow/workflow-engine/contracts/proto/workflow-engine.proto`: verify request/response or event contract only when incident evidence points there.
- `workflow/workflow-engine/dashboards/durable-state-size.json`: verify panel coverage for `oya_workflow_engine_deadlock_resolution_error_ratio`, `oya_workflow_engine_deadlock_resolution_lag_seconds`, and `oya_workflow_engine_deadlock_wait_seconds`.
- `workflow/observability/slos/workflow-engine/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `workflow/workflow-engine/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `DeadlockResolutionCritical` and `DeadlockResolutionSloBurn` are both resolved in Alertmanager for 30 minutes.
- `oya_workflow_engine_deadlock_resolution_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `oya_workflow_engine_deadlock_resolution_lag_seconds < 120` for all production cells.
- `oya_workflow_engine_deadlock_resolution_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `oya_workflow_engine_deadlock_wait_seconds` is below the threshold documented in `workflow/observability/slos/workflow-engine/replay-determinism-correctness.openslo.yaml`.
- Dashboard `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/deadlock-resolution?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` shows green panels for the affected cell.
- Audit-chain query for `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` returns mitigation and resolution events.
- Circuit breaker `workflow-engine-deadlock-resolution-circuit-breaker` is closed after rollback window.
- Feature flag `oya.workflow-engine.deadlock_resolution.incident_hold` is false for the affected tenant unless long-term hold is approved.
- Runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- Service owner acknowledged final handoff in `#inc-workflow-engine`.

## Capacity and Rollback Guardrails
- Capacity math: if `oya_workflow_engine_deadlock_resolution_queue_depth` is 5000 and the worker drains 25 items/second, the best-case drain is 200 seconds before retries; page earlier when drain time exceeds 300 seconds.
- Capacity math: with 12 replicas at 25 items/second each, the hard ceiling is 300 items/second; keep tenant throttle below 25 RPS until error ratio stays below 0.005.
- Rollback checkpoint 1: before changing `oya.workflow-engine.deadlock_resolution.incident_hold`, snapshot current value with `oya flags get oya.workflow-engine.deadlock_resolution.incident_hold --output json`.
- Rollback checkpoint 2: before opening `workflow-engine-deadlock-resolution-circuit-breaker`, capture `oya_workflow_engine_deadlock_resolution_request_rate` and `oya_workflow_engine_deadlock_resolution_success_ratio` from Mimir.
- Rollback checkpoint 3: before scaling deployments, capture `kubectl -n workflow-engine get deploy workflow-engine-deadlock-resolution-worker -o yaml`.
- Rollback command for flag: `oya flags set oya.workflow-engine.deadlock_resolution.incident_hold=false --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for breaker: `oya ops breaker close workflow-engine-deadlock-resolution-circuit-breaker --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for deployment: `kubectl -n workflow-engine rollout undo deploy/workflow-engine-deadlock-resolution-worker`.
- Rollback command for tenant throttle: `oya ops rate-limit clear --tenant $TENANT --surface workflow-engine.deadlock-resolution --reason rollback-$INCIDENT_ID`.
- Stop rollback if `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` cannot be emitted; preserve the current state and escalate to audit-chain before additional mutation.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: workflow-engine-deadlock-resolution
microservice: workflow-engine
event_class: EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Deadlock Resolution postmortem

## Summary
- What happened in workflow-engine/deadlock-resolution.
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
- Emit EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-workflow-engine-primary; workflow-runtime-secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, 30m for sev3; status update every 10m until the critical alert clears.
- Incident commander: first responder from axis-workflow-engine + ops-sre-reliability; transfer only by explicit message in `#inc-workflow-engine`.
- Security escalation: page `ops-security-primary` immediately for sev0, credential, cross-tenant, fraud, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, money movement, or breach-clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: CNCF Temporal support; Valkey commercial support; PostgreSQL Citus support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-workflow-engine-deadlock-resolution` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `audit-chain`: `oya incident handoff --target audit-chain --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `audit-chain` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID --severity sev1 --branch B`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `identity`: `oya incident handoff --target identity --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID --severity sev1 --branch C`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID --severity sev1 --branch D`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `workflow-studio`: `oya incident handoff --target workflow-studio --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `workflow-studio` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source workflow-engine --runbook deadlock-resolution --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `oya_workflow_engine_deadlock_resolution_error_ratio`, `oya_workflow_engine_deadlock_resolution_lag_seconds`, `oya_workflow_engine_deadlock_resolution_queue_depth`, `oya_workflow_engine_deadlock_wait_seconds`, current breaker state, and audit seal status.
- Keep `workflow-engine-deadlock-resolution-circuit-breaker` owner as axis-workflow-engine + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_WORKFLOW_ENGINE_DEADLOCK_RESOLUTION_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `workflow/workflow-engine/dashboards/` for dashboard names and operational panels: durable-state-size.json, step-latency.json, workflow-execution-rate.json.
- `workflow/observability/slos/workflow-engine/` for OpenSLO alert vocabulary and threshold alignment: replay-determinism-correctness.openslo.yaml, worker-poll-availability.openslo.yaml, workflow-completion-availability.openslo.yaml, workflow-start-latency.openslo.yaml, workflow-step-execute-latency.openslo.yaml.
- `workflow/workflow-engine/policy/` for named policy and authorization surfaces: saga-compensation-policy.md, spec-integrity.md, tenant-scope.cedar, data-residency.md.
- `workflow/workflow-engine/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi/workflow-engine.yaml, contracts/asyncapi/workflow-events.yaml, contracts/proto/workflow-engine.proto.
- `workflow/workflow-engine/manifest.json` for owner, dependency, capability, and bounded-context vocabulary; topic `deadlock-resolution` is the scenario anchor.

## Checkpoint Closure Criteria
- The runbook remains current when `DeadlockResolutionCritical`, `DeadlockResolutionSloBurn`, `oya_workflow_engine_deadlock_wait_seconds`, `oya.workflow-engine.deadlock_resolution.incident_hold`, and `workflow-engine-deadlock-resolution-circuit-breaker` all resolve to live telemetry, flag, or breaker records.
- The incident is cleanly halted if required authority is missing for tenant quarantine, policy rollback, or vendor escalation; do not improvise outside the named commands.
- The checkpoint is complete when `./bin/oya vcs verify --agent codex-runbooks-substrate-w3 --evidence 'runbooks_substance:X new_runbooks:Y' ...` accepts the five target scopes.
