---
doc_class: Runbook
title: Stuck Workflow Recovery
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

# Runbook: Stuck Workflow Recovery

## Operator Contract
- Runbook id: workflow-engine-stuck-workflow-recovery.
- Primary service namespace: `workflow-engine`.
- Owning rotation: PagerDuty oya-workflow-engine-primary; workflow-runtime-secondary.
- Incident channel: `#inc-workflow-engine`.
- Operational focus: workflow remains WAITING beyond the SLA timer and needs operator intervention.
- Named precedent: this follows the Temporal durable execution plus AWS Step Functions state-machine recovery pattern.
- External dependencies: CNCF Temporal support; Valkey commercial support; PostgreSQL Citus support.
- API authority: `https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/stuck-workflow-recovery/incident-handoff`.
- Audit event class: `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, `StuckWorkflowRecoveryCritical` is green, and every Cross-microservice handoff API returns `202 accepted`.
- Safety invariant: never clear the incident until `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/workflow-engine-stuck-workflow-recovery-<incident-id>.md`.

## Trigger Conditions
- Page on alert `StuckWorkflowRecoveryCritical` when `oya_workflow_engine_stuck_workflow_recovery_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `StuckWorkflowRecoverySloBurn` when `oya_workflow_engine_stuck_workflow_recovery_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open sev1 if `oya_workflow_engine_run_stuck_seconds` exceeds the threshold documented in `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`.
- Open sev1 if `oya_workflow_engine_stuck_workflow_recovery_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `workflow-engine.stuck-workflow-recovery.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate workflow-engine-stuck-workflow-recovery --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/stuck-workflow-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` backed by `microservices/workflow-engine/dashboards/durable-state-size.json`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/stuck-workflow-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202` backed by `microservices/workflow-engine/dashboards/step-latency.json`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="workflow-engine",runbook="stuck-workflow-recovery"}`.
- Alertmanager route: `oyatie-workflow-engine-stuck-workflow-recovery-critical`; silence only with incident commander approval and `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` evidence.
- Synthetic probe: `oya ops probe workflow-engine stuck-workflow-recovery --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/workflow-engine/stuck-workflow-recovery/expected-state.json` hash differs from live `https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/stuck-workflow-recovery/admin/state-hash`.
- Service-specific metric `oya_workflow_engine_run_stuck_seconds` is red while `oya_workflow_engine_stuck_workflow_recovery_audit_emit_total{status="sealed"}` is flat.

## Symptoms
- User-facing impact: workflow runs may pause, duplicate, or skip durable state transitions for affected tenants; scenario focus is workflow remains WAITING beyond the SLA timer and needs operator intervention.
- Operators see Grafana panel `durable-state-size.json / Stuck Workflow Recovery burn rate` turn red before the primary alert resolves.
- Loki signature `workflow_engine.stuck_workflow_recovery.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=StuckWorkflowRecoveryDegraded` on deployment `workflow-engine-stuck-workflow-recovery-worker` or `workflow-engine-api`.
- Audit-chain shows missing or delayed `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` entries when queried with `oya audit-chain query --event-class EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT --since 30m`.
- Metric pattern: `oya_workflow_engine_stuck_workflow_recovery_error_ratio` rises before `oya_workflow_engine_stuck_workflow_recovery_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_workflow_engine_stuck_workflow_recovery_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal, replay backlog, or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_workflow_engine_stuck_workflow_recovery_queue_depth`; isolate tenant before fleet mitigation.
- Fleet-wide shape: at least three cells report `StuckWorkflowRecoveryCritical` in one 15 minute window; switch to cross-cell bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=stuck-workflow-recovery.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=stuck-workflow-recovery.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` means mitigation cannot be closed until replay succeeds.
- Service-specific pattern: `oya_workflow_engine_run_stuck_seconds` rises while `oya_workflow_engine_stuck_workflow_recovery_dependency_error_ratio` is flat; inspect local state before escalating CNCF Temporal support.
- Service-specific pattern: `oya_workflow_engine_stuck_workflow_recovery_dependency_error_ratio` rises while `oya_workflow_engine_run_stuck_seconds` is flat; inspect vendor or adjacent-service dependency health before local rollback.

## Failure Mode Tree
- Failure mode 1: single-tenant WorkflowRun inconsistency; contain with tenant quarantine, preserve all `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` rows, and avoid fleet rollback.
- Failure mode 2: cross-cell WorkflowSpec drift; freeze writes, compare state hash across cells, and use audit-chain replay before accepting new mutations.
- Failure mode 3: byzantine or abusive principal; suspend the principal through identity, keep tenant data scoped, and preserve Cedar explain output.
- Failure mode 4: external dependency outage at CNCF Temporal support; open vendor ticket only after local dashboards and handoff APIs prove the dependency is causal.
- Failure mode 5: operator mitigation made state worse; roll back feature flag `oya.workflow-engine.stuck_workflow_recovery.incident_hold`, close `workflow-engine-stuck-workflow-recovery-circuit-breaker`, and restore the previous deployment revision.
- Failure mode 6: audit emission is delayed; do not close even when customer symptoms improve because ADR-0263 evidence is incomplete.
- Failure mode 7: regional partition; keep prod-us-east-1 as evidence leader and reject cross-region mutation until `oya_workflow_engine_stuck_workflow_recovery_state_hash_match == 1`.
- Failure mode 8: compliance-pack mismatch; require compliance handoff when KR-CSAP, EU-sovereign, FedRAMP-High, IL5, or CN-PIPL labels are present.
- Failure mode 9: stale dashboard data; verify direct Mimir queries before making rollback decisions.
- Failure mode 10: runbook step ambiguity; halt the ambiguous branch, emit `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` with outcome `blocked`, and patch this runbook after recovery.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-workflow-engine-stuck-workflow-recovery-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/alerts?runbook=stuck-workflow-recovery | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n workflow-engine rollout status deploy/workflow-engine-stuck-workflow-recovery-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n workflow-engine get pods -l app=stuck-workflow-recovery -o wide`.
5. Read structured logs: `kubectl -n workflow-engine logs deploy/workflow-engine-stuck-workflow-recovery-worker --since=30m | rg "workflow_engine.stuck_workflow_recovery.incident_state|StuckWorkflowRecoveryCritical|EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="workflow-engine",runbook="stuck-workflow-recovery"}' --since=30m --limit=200`.
7. Check Prometheus error ratio: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_engine_stuck_workflow_recovery_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_engine_stuck_workflow_recovery_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_engine_stuck_workflow_recovery_queue_depth{cell="prod-us-east-1"}'`.
10. Check service-specific signal: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_workflow_engine_run_stuck_seconds{cell="prod-us-east-1"}'`.
11. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/workflow-engine-ops/stuck-workflow-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101&var-incident=$INCIDENT_ID"`.
12. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/workflow-engine-ops/stuck-workflow-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=202&var-tenant=$TENANT"`.
13. Verify audit-chain emission: `oya audit-chain query --event-class EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
14. Verify service state: `oya ops workflow-engine stuck-workflow-recovery status --cell $CELL --tenant $TENANT --output json`.
15. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate workflow-engine-stuck-workflow-recovery --production-snapshot --cell $CELL`.
16. Run crate smoke test: `cargo test -p oya-workflow-engine-execution-engine-domain stuck_workflow_recovery -- --nocapture`.
17. Check API contract smoke: `curl -s https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/stuck-workflow-recovery/incident-handoff -H "x-oya-tenant: $TENANT"`.
18. Inspect config: `test -f microservices/workflow-engine/iac/kustomize/base/kustomization.yaml && sed -n '1,180p' microservices/workflow-engine/iac/kustomize/base/kustomization.yaml`.
19. Inspect feature flags: `oya flags get oya.workflow-engine.stuck_workflow_recovery.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
20. Inspect circuit breaker: `oya ops breaker status workflow-engine-stuck-workflow-recovery-circuit-breaker --cell $CELL --tenant $TENANT`.
21. Check recent deploy: `kubectl -n workflow-engine rollout history deploy/workflow-engine-stuck-workflow-recovery-worker | tail -20`.
22. Check policy file: `test -f microservices/workflow-engine/policy/saga-compensation-policy.md || find microservices/workflow-engine/policy -maxdepth 2 -type f | sort`.
23. Check SLO files: `ls microservices/workflow-engine/slos/*.openslo.yaml | sort | rg "replay|worker"`.
24. Check contract binding: `test -f microservices/workflow-engine/contracts/openapi/workflow-engine.yaml && sed -n '1,120p' microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`.
25. Run targeted SQL state query: `psql $OYA_PROD_DSN -c "select incident_id, tenant_id, cell_id, state, updated_at from workflow_engine_stuck_workflow_recovery_incidents where updated_at > now() - interval '30 minutes' order by updated_at desc limit 20;"`.
26. Confirm no cross-cell spread: `oya ops cells query --metric oya_workflow_engine_stuck_workflow_recovery_error_ratio --window 30m --threshold 0.02`.
27. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice workflow-engine --runbook stuck-workflow-recovery --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Stuck Workflow Recovery incident decision tree
1. Is StuckWorkflowRecoveryCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-workflow-engine-primary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_workflow_engine_stuck_workflow_recovery_queue_depth grow while oya_workflow_engine_stuck_workflow_recovery_error_ratio is flat?
   |-- yes: downstream dependency, replay backlog, or queue-drain issue; choose mitigation branch B.
   |-- no: local regression, bad input, or policy/config drift; continue branch selection.
3. Does audit-chain show EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer, finance, security, or regulator impact confirmed?
   |-- yes: promote severity, open #inc-workflow-engine, and notify compliance or security handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (confirmed WorkflowRun correctness risk): use the matching mitigation block below and record `decision_branch=A` in `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT`.
- Branch B (dependency saturation or replay backlog): use the matching mitigation block below and record `decision_branch=B` in `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT`.
- Branch C (policy, permit, or tenant-scope drift): use the matching mitigation block below and record `decision_branch=C` in `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT`.
- Branch D (customer-visible or regulated evidence gap): use the matching mitigation block below and record `decision_branch=D` in `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service workflow-engine --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-workflow-engine --severity sev1`.
3. Freeze risky automation: `oya flags set oya.workflow-engine.stuck_workflow_recovery.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open workflow-engine-stuck-workflow-recovery-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n workflow-engine scale deploy/workflow-engine-stuck-workflow-recovery-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason workflow-engine-stuck-workflow-recovery --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
8. Drain queue safely: `oya ops workflow-engine stuck-workflow-recovery drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops workflow-engine stuck-workflow-recovery drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n workflow-engine rollout undo deploy/workflow-engine-stuck-workflow-recovery-worker`.
12. Raise HPA cap if saturation is proven: `kubectl -n workflow-engine patch hpa workflow-engine-stuck-workflow-recovery-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface workflow-engine.stuck-workflow-recovery --rps 25 --ttl 30m`.
14. Block abusive principal when relevant: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/workflow-engine/runbooks/stuck-workflow-recovery.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice workflow-engine --incident $INCIDENT_ID --channel #inc-workflow-engine`.
17. Open external vendor ticket: `oya vendor ticket open --vendor "CNCF Temporal support" --incident $INCIDENT_ID --summary workflow-engine-stuck-workflow-recovery`.
18. Confirm breaker effect: `oya ops breaker status workflow-engine-stuck-workflow-recovery-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://workflow-engine.internal.oyatie.dev/v1/workflow-engine/stuck-workflow-recovery/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=stuck-workflow-recovery`.

### Mitigation Branch Guidance
- Branch A: confirmed WorkflowRun correctness risk.
  - Required action: keep `workflow-engine-stuck-workflow-recovery-circuit-breaker` open until `oya_workflow_engine_stuck_workflow_recovery_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/stuck-workflow-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=110` to the incident.
  - Required audit: emit `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: dependency saturation or replay backlog.
  - Required action: keep `workflow-engine-stuck-workflow-recovery-circuit-breaker` open until `oya_workflow_engine_stuck_workflow_recovery_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/stuck-workflow-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=111` to the incident.
  - Required audit: emit `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: policy, permit, or tenant-scope drift.
  - Required action: keep `workflow-engine-stuck-workflow-recovery-circuit-breaker` open until `oya_workflow_engine_stuck_workflow_recovery_error_ratio` is below 0.005 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/stuck-workflow-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=112` to the incident.
  - Required audit: emit `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: customer-visible or regulated evidence gap.
  - Required action: keep `workflow-engine-stuck-workflow-recovery-circuit-breaker` open until `oya_workflow_engine_stuck_workflow_recovery_error_ratio` is below 0.01 for 3 evaluator windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/stuck-workflow-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=113` to the incident.
  - Required audit: emit `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "stuck_workflow_recovery|StuckWorkflowRecoveryCritical|workflow_engine.stuck_workflow_recovery.incident_state" crates microservices/workflow-engine -g "!microservices/workflow-engine/runbooks/**"`.
2. Patch domain invariant: `edit oya-workflow-engine-execution-engine-domain where stuck_workflow_recovery state transition is validated`.
3. Patch API guard: `edit microservices/workflow-engine/contracts/openapi/workflow-engine.yaml if the failing path is north-south or async handoff`.
4. Patch policy: `edit microservices/workflow-engine/policy/saga-compensation-policy.md with explicit deny/permit branch and tenant/cell scope`.
5. Patch runtime config: `edit microservices/workflow-engine/iac/kustomize/base/kustomization.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-workflow-engine-execution-engine-domain stuck_workflow_recovery_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate workflow-engine-stuck-workflow-recovery --fixture incident-stuck-workflow-recovery.json`.
8. Add SLO assertion: `update microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml with alert StuckWorkflowRecoveryCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/workflow-engine/dashboards/durable-state-size.json with oya_workflow_engine_stuck_workflow_recovery_error_ratio, oya_workflow_engine_stuck_workflow_recovery_lag_seconds, and oya_workflow_engine_run_stuck_seconds`.
10. Rebuild affected crate: `cargo check -p oya-workflow-engine-execution-engine-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-workflow-engine-execution-engine-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate workflow-engine-policy --microservice workflow-engine`.
13. Deploy canary: `oya deploy canary --microservice workflow-engine --component workflow-engine-stuck-workflow-recovery-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_workflow_engine_stuck_workflow_recovery_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close workflow-engine-stuck-workflow-recovery-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.workflow-engine.stuck_workflow_recovery.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=stuck-workflow-recovery`.
19. Verify seal: `oya audit-chain verify --event-class EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-workflow-engine-execution-engine-domain`: inspect for `stuck_workflow_recovery` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-workflow-engine-state-machine-kernel`: inspect for `stuck_workflow_recovery` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-workflow-engine-event-bus-worker`: inspect for `stuck_workflow_recovery` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `oya-workflow-engine-replay-debugger-backend-domain`: inspect for `stuck_workflow_recovery` invariants, alert emission, ADR-0263 evidence fields, and tenant/cell scoping before touching adjacent code.
- `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/workflow-engine/contracts/asyncapi/workflow-events.yaml`: verify request/response or event contract only when incident evidence points there.
- `microservices/workflow-engine/contracts/proto/workflow-engine.proto`: verify request/response or event contract only when incident evidence points there.
- `microservices/workflow-engine/dashboards/durable-state-size.json`: verify panel coverage for `oya_workflow_engine_stuck_workflow_recovery_error_ratio`, `oya_workflow_engine_stuck_workflow_recovery_lag_seconds`, and `oya_workflow_engine_run_stuck_seconds`.
- `microservices/workflow-engine/slos/`: verify alert vocabulary and threshold alignment before changing runtime thresholds.
- `microservices/workflow-engine/policy/`: verify policy branch ownership before relaxing deny rules or emergency bypasses.

## Verification Checklist
- `StuckWorkflowRecoveryCritical` and `StuckWorkflowRecoverySloBurn` are both resolved in Alertmanager for 30 minutes.
- `oya_workflow_engine_stuck_workflow_recovery_error_ratio < 0.005` for 3 consecutive 10 minute windows.
- `oya_workflow_engine_stuck_workflow_recovery_lag_seconds < 120` for all production cells.
- `oya_workflow_engine_stuck_workflow_recovery_queue_depth` is draining and not growing for the affected tenant.
- Service-specific signal `oya_workflow_engine_run_stuck_seconds` is below the threshold documented in `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`.
- Dashboard `https://grafana.dev.oyatie.internal/d/workflow-engine-ops/stuck-workflow-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=101` shows green panels for the affected cell.
- Audit-chain query for `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` returns mitigation and resolution events.
- Circuit breaker `workflow-engine-stuck-workflow-recovery-circuit-breaker` is closed after rollback window.
- Feature flag `oya.workflow-engine.stuck_workflow_recovery.incident_hold` is false for the affected tenant unless long-term hold is approved.
- Runbook invocation evidence is attached to `evidence/incidents/$INCIDENT_ID.json`.
- Service owner acknowledged final handoff in `#inc-workflow-engine`.

## Capacity and Rollback Guardrails
- Capacity math: if `oya_workflow_engine_stuck_workflow_recovery_queue_depth` is 5000 and the worker drains 25 items/second, the best-case drain is 200 seconds before retries; page earlier when drain time exceeds 300 seconds.
- Capacity math: with 12 replicas at 25 items/second each, the hard ceiling is 300 items/second; keep tenant throttle below 25 RPS until error ratio stays below 0.005.
- Rollback checkpoint 1: before changing `oya.workflow-engine.stuck_workflow_recovery.incident_hold`, snapshot current value with `oya flags get oya.workflow-engine.stuck_workflow_recovery.incident_hold --output json`.
- Rollback checkpoint 2: before opening `workflow-engine-stuck-workflow-recovery-circuit-breaker`, capture `oya_workflow_engine_stuck_workflow_recovery_request_rate` and `oya_workflow_engine_stuck_workflow_recovery_success_ratio` from Mimir.
- Rollback checkpoint 3: before scaling deployments, capture `kubectl -n workflow-engine get deploy workflow-engine-stuck-workflow-recovery-worker -o yaml`.
- Rollback command for flag: `oya flags set oya.workflow-engine.stuck_workflow_recovery.incident_hold=false --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for breaker: `oya ops breaker close workflow-engine-stuck-workflow-recovery-circuit-breaker --cell $CELL --tenant $TENANT --reason rollback-$INCIDENT_ID`.
- Rollback command for deployment: `kubectl -n workflow-engine rollout undo deploy/workflow-engine-stuck-workflow-recovery-worker`.
- Rollback command for tenant throttle: `oya ops rate-limit clear --tenant $TENANT --surface workflow-engine.stuck-workflow-recovery --reason rollback-$INCIDENT_ID`.
- Stop rollback if `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` cannot be emitted; preserve the current state and escalate to audit-chain before additional mutation.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: workflow-engine-stuck-workflow-recovery
microservice: workflow-engine
event_class: EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Stuck Workflow Recovery postmortem

## Summary
- What happened in workflow-engine/stuck-workflow-recovery.
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
- Emit EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
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
- Customer communications: use status page component `oyatie-workflow-engine-stuck-workflow-recovery` and keep private details in the incident channel.
- Regulatory clock: if tenant data, financial correctness, or evidence integrity is possibly affected, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `audit-chain`: `oya incident handoff --target audit-chain --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `audit-chain` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID --severity sev1 --branch B`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `identity`: `oya incident handoff --target identity --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID --severity sev1 --branch C`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID --severity sev1 --branch D`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `workflow-studio`: `oya incident handoff --target workflow-studio --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `workflow-studio` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID`.
- Audit-chain handoff API: `oya incident handoff --target audit-chain --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source workflow-engine --runbook stuck-workflow-recovery --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include `oya_workflow_engine_stuck_workflow_recovery_error_ratio`, `oya_workflow_engine_stuck_workflow_recovery_lag_seconds`, `oya_workflow_engine_stuck_workflow_recovery_queue_depth`, `oya_workflow_engine_run_stuck_seconds`, current breaker state, and audit seal status.
- Keep `workflow-engine-stuck-workflow-recovery-circuit-breaker` owner as axis-workflow-engine + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after `EVT_WORKFLOW_ENGINE_STUCK_WORKFLOW_RECOVERY_INCIDENT` has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/workflow-engine/dashboards/` for dashboard names and operational panels: durable-state-size.json, step-latency.json, workflow-execution-rate.json.
- `microservices/workflow-engine/slos/` for OpenSLO alert vocabulary and threshold alignment: replay-determinism-correctness.openslo.yaml, worker-poll-availability.openslo.yaml, workflow-completion-availability.openslo.yaml, workflow-start-latency.openslo.yaml, workflow-step-execute-latency.openslo.yaml.
- `microservices/workflow-engine/policy/` for named policy and authorization surfaces: saga-compensation-policy.md, spec-integrity.md, tenant-scope.cedar, data-residency.md.
- `microservices/workflow-engine/contracts/` for API, AsyncAPI, proto, and adapter surfaces: contracts/openapi/workflow-engine.yaml, contracts/asyncapi/workflow-events.yaml, contracts/proto/workflow-engine.proto.
- `microservices/workflow-engine/manifest.json` for owner, dependency, capability, and bounded-context vocabulary; topic `stuck-workflow-recovery` is the scenario anchor.

## Checkpoint Closure Criteria
- The runbook remains current when `StuckWorkflowRecoveryCritical`, `StuckWorkflowRecoverySloBurn`, `oya_workflow_engine_run_stuck_seconds`, `oya.workflow-engine.stuck_workflow_recovery.incident_hold`, and `workflow-engine-stuck-workflow-recovery-circuit-breaker` all resolve to live telemetry, flag, or breaker records.
- The incident is cleanly halted if required authority is missing for tenant quarantine, policy rollback, or vendor escalation; do not improvise outside the named commands.
- The checkpoint is complete when `./bin/oya vcs verify --agent codex-runbooks-substrate-w3 --evidence 'runbooks_substance:X new_runbooks:Y' ...` accepts the five target scopes.
