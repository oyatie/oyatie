---
doc_class: Runbook
title: Lane Failure Triage
status: Accepted
date: 2026-05-20
microservice: governance
severity: sev1
audience: sre
owner_team: axis-foundry + council-architecture + ops-sre-reliability
source_wave: codex-runbooks-substrate-w1
change_scope: substance rewrite of thin existing runbook
doc_status: published
---

# Runbook: Lane Failure Triage

## Operator Contract
- Runbook id: governance-lane-failure-triage.
- Primary service namespace: `governance`.
- Owning rotation: PagerDuty oya-governance-primary; council-architecture reviewer-on-call.
- Incident channel: `#inc-governance-gates`.
- External dependencies: Cedar policy runtime maintainers; Wasmtime security list; GitHub Enterprise support.
- API authority: `https://governance.internal.oyatie.dev/v1/governance/lane-failure-triage/incident-handoff`.
- Audit event class: `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, GovernanceLaneFailureTriageCritical is green, and all handoff APIs in Cross-µservice Coordination return `202 accepted`.
- Safety invariant: never clear the incident until `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/governance-lane-failure-triage-<incident-id>.md`.

## Trigger Conditions
- Page on alert `GovernanceLaneFailureTriageCritical` when `oya_governance_lane_failure_triage_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `GovernanceLaneFailureTriageSloBurn` when `oya_governance_lane_failure_triage_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev0 if `oya_governance_lane_failure_triage_correctness_ratio < 0.9999` and the affected label set includes `tenant_id` or `principal_id`.
- Open a sev1 if `oya_governance_lane_failure_triage_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `governance.lane-failure-triage.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate governance-lane-failure-triage --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/governance-substrate/lane-failure-triage?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=103`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/governance-substrate/lane-failure-triage?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="governance",runbook="lane-failure-triage"}`.
- Alertmanager route: `oyatie-governance-lane-failure-triage-critical`; silence only with incident commander approval and `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT` evidence.
- Synthetic probe: `oya ops probe governance lane-failure-triage --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/governance/lane-failure-triage/expected-state.json` hash differs from live `https://governance.internal.oyatie.dev/v1/admin/state-hash`.

## Symptoms
- User-facing impact: lane failure triage blocks or corrupts the governance control path for affected tenants.
- Operators see Grafana panel `envoy-wasm-filter-latency-p99 / Lane Failure Triage burn rate` turn red before the primary alert resolves.
- Loki signature `governance.lane_failure_triage.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=GovernanceLaneFailureTriageDegraded` on deployment `governance-lane-failure-triage-worker`.
- Audit-chain shows missing or delayed `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT` entries when queried with `oya audit-chain query --event-class EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT --since 30m`.
- Metric pattern: `oya_governance_lane_failure_triage_error_ratio` rises before `oya_governance_lane_failure_triage_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_governance_lane_failure_triage_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_governance_lane_failure_triage_queue_depth`; isolate before fleet mitigation.
- Fleet-wide shape: at least three cells report `GovernanceLaneFailureTriageCritical` in one 15 minute window; switch to sev1 bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=lane-failure-triage.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=lane-failure-triage.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT` means mitigation cannot be closed until replay succeeds.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-governance-lane-failure-triage-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://governance.internal.oyatie.dev/v1/alerts?runbook=lane-failure-triage | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n governance rollout status deploy/governance-lane-failure-triage-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n governance get pods -l app=governance-lane-failure-triage -o wide`.
5. Read structured logs: `kubectl -n governance logs deploy/governance-lane-failure-triage-worker --since=30m | rg "governance.lane_failure_triage.incident_state|GovernanceLaneFailureTriageCritical|EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="governance",runbook="lane-failure-triage"}' --since=30m --limit=200`.
7. Check Prometheus fast burn: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_governance_lane_failure_triage_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_governance_lane_failure_triage_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_governance_lane_failure_triage_queue_depth{cell="prod-us-east-1"}'`.
10. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/governance-substrate/lane-failure-triage?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=103&var-incident=$INCIDENT_ID"`.
11. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/governance-substrate/lane-failure-triage?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210&var-tenant=$TENANT"`.
12. Verify audit-chain emission: `oya audit-chain query --event-class EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
13. Verify service state: `oya ops governance lane-failure-triage status --cell $CELL --tenant $TENANT --output json`.
14. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate governance-lane-failure-triage --production-snapshot --cell $CELL`.
15. Check Cargo owner crate: `cargo test -p oya-governance-domain lane_failure_triage -- --nocapture`.
16. Check API contract smoke: `curl -s https://governance.internal.oyatie.dev/v1/governance/lane-failure-triage/incident-handoff -H "x-oya-tenant: $TENANT"`.
17. Inspect config: `kubectl -n governance get configmap governance-lane-failure-triage-config -o yaml`.
18. Inspect feature flags: `oya flags get oya.governance.lane_failure_triage.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
19. Inspect circuit breaker: `oya ops breaker status governance-lane-failure-triage-circuit-breaker --cell $CELL --tenant $TENANT`.
20. Check recent deploy: `kubectl -n governance rollout history deploy/governance-lane-failure-triage-worker | tail -20`.
21. Check policy file: `test -f microservices/governance/policy/lane-execution.cedar || test -f microservices/governance/policy/lane-execution.md`.
22. Check SLO files: `ls microservices/governance/slos/*.openslo.yaml | sort`.
23. Check catalog components: `find microservices/governance/catalog -maxdepth 1 -type f | sort | rg "governance|lane"`.
24. Confirm no cross-cell spread: `oya ops cells query --metric oya_governance_lane_failure_triage_error_ratio --window 30m --threshold 0.02`.
25. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice governance --runbook lane-failure-triage --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Lane Failure Triage incident decision tree
1. Is GovernanceLaneFailureTriageCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-governance-primary; council-architecture reviewer-on-call, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_governance_lane_failure_triage_queue_depth grow while oya_governance_lane_failure_triage_error_ratio is flat?
   |-- yes: downstream dependency or replay backlog; choose mitigation branch B.
   |-- no: local regression or bad input; continue branch selection.
3. Does audit-chain show EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer or regulator impact confirmed?
   |-- yes: promote severity, open #inc-governance-gates, and notify compliance handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (single tenant isolated): use the matching mitigation block below and record `decision_branch=A` in `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT`.
- Branch B (fleet-wide propagation): use the matching mitigation block below and record `decision_branch=B` in `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT`.
- Branch C (dependency regression): use the matching mitigation block below and record `decision_branch=C` in `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT`.
- Branch D (operator ceremony incomplete): use the matching mitigation block below and record `decision_branch=D` in `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service governance --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-governance-gates --severity sev1`.
3. Freeze risky automation: `oya flags set oya.governance.lane_failure_triage.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open governance-lane-failure-triage-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n governance scale deploy/governance-lane-failure-triage-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason governance-lane-failure-triage --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
8. Drain queue safely: `oya ops governance lane-failure-triage drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops governance lane-failure-triage drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n governance rollout undo deploy/governance-lane-failure-triage-worker`.
12. Raise HPA cap if saturation: `kubectl -n governance patch hpa governance-lane-failure-triage-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface governance.lane-failure-triage --rps 25 --ttl 30m`.
14. Block abusive principal: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/governance/runbooks/lane-failure-triage.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice governance --incident $INCIDENT_ID --channel #inc-governance-gates`.
17. Open external vendor ticket: `oya vendor ticket open --vendor primary-governance --incident $INCIDENT_ID --summary lane-failure-triage`.
18. Confirm breaker effect: `oya ops breaker status governance-lane-failure-triage-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://governance.internal.oyatie.dev/v1/governance/lane-failure-triage/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=lane-failure-triage`.

### Mitigation Branch Guidance
- Branch A: single tenant isolated.
  - Required action: keep `governance-lane-failure-triage-circuit-breaker` open until `oya_governance_lane_failure_triage_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/lane-failure-triage?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=103` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: fleet-wide propagation.
  - Required action: keep `governance-lane-failure-triage-circuit-breaker` open until `oya_governance_lane_failure_triage_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/lane-failure-triage?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=104` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: dependency regression.
  - Required action: keep `governance-lane-failure-triage-circuit-breaker` open until `oya_governance_lane_failure_triage_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/lane-failure-triage?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=105` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: operator ceremony incomplete.
  - Required action: keep `governance-lane-failure-triage-circuit-breaker` open until `oya_governance_lane_failure_triage_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/governance-substrate/lane-failure-triage?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=106` to the incident.
  - Required audit: emit `EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "lane_failure_triage|GovernanceLaneFailureTriageCritical|governance.lane_failure_triage.incident_state" crates microservices/governance -g "!microservices/governance/runbooks/**"`.
2. Patch domain invariant: `edit oya-governance-domain where lane_failure_triage state transition is validated`.
3. Patch API guard: `edit microservices/governance/contracts/openapi.yaml or catalog REST binding if the failing path is north-south`.
4. Patch policy: `edit microservices/governance/policy/lane-execution.cedar or .md with explicit deny/permit branch`.
5. Patch runtime config: `edit microservices/governance/iac/k8s-deployment.yaml or secret-bindings.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-governance-domain lane_failure_triage_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate governance-lane-failure-triage --fixture incident-lane-failure-triage.json`.
8. Add SLO assertion: `update microservices/governance/slos/* with alert GovernanceLaneFailureTriageCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/governance/dashboards/envoy-wasm-filter-latency-p99.json with oya_governance_lane_failure_triage_error_ratio, oya_governance_lane_failure_triage_lag_seconds, and oya_governance_lane_failure_triage_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-governance-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-governance-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate governance-policy --microservice governance`.
13. Deploy canary: `oya deploy canary --microservice governance --component lane-failure-triage-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_governance_lane_failure_triage_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close governance-lane-failure-triage-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.governance.lane_failure_triage.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; branch-protected `oya-ci-required` required; legacy `oya gate` output optional local/provenance only).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=lane-failure-triage`.
19. Verify seal: `oya audit-chain verify --event-class EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-governance-domain`: inspect for lane_failure_triage invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 1.
- `oya-dev-cli`: inspect for lane_failure_triage invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 2.
- `oya-policy-cedar-domain`: inspect for lane_failure_triage invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 3.
- `microservices/governance/contracts/`: verify this surface only when the incident evidence points there.
- `microservices/governance/dashboards/envoy-wasm-filter-latency-p99.json`: verify this surface only when the incident evidence points there.
- `microservices/governance/slos/`: verify this surface only when the incident evidence points there.
- `microservices/governance/policy/lane-execution.*`: verify this surface only when the incident evidence points there.

## Verification Checklist
- GovernanceLaneFailureTriageCritical and GovernanceLaneFailureTriageSloBurn are both resolved in Alertmanager for 30 minutes.
- oya_governance_lane_failure_triage_error_ratio < 0.005 for 3 consecutive 10 minute windows.
- oya_governance_lane_failure_triage_lag_seconds < 120 for all production cells.
- oya_governance_lane_failure_triage_queue_depth is draining and not growing for the affected tenant.
- dashboard https://grafana.dev.oyatie.internal/d/governance-substrate/lane-failure-triage?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=103 shows green panels for the affected cell.
- audit-chain query for EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT returns mitigation and resolution events.
- circuit breaker governance-lane-failure-triage-circuit-breaker is closed after rollback window.
- feature flag oya.governance.lane_failure_triage.incident_hold is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to evidence/incidents/$INCIDENT_ID.json.
- service owner acknowledged final handoff in #inc-governance-gates.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: governance-lane-failure-triage
microservice: governance
event_class: EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Lane Failure Triage postmortem

## Summary
- What happened in governance/lane-failure-triage.
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
- Emit EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-governance-primary; council-architecture reviewer-on-call.
- Incident SLA: ack 5m for sev1, 15m for sev2, lane owner checkpoint every 20m.
- Incident commander: first responder from axis-foundry + council-architecture + ops-sre-reliability; transfer only by explicit message in #inc-governance-gates.
- Security escalation: page `ops-security-primary` immediately for sev0, data-boundary, credential, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, or breach clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Cedar policy runtime maintainers; Wasmtime security list; GitHub Enterprise support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-governance-lane-failure-triage` and keep private details in the incident channel.
- Regulatory clock: if any tenant data exposure is possible, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source governance --runbook lane-failure-triage --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source governance --runbook lane-failure-triage --incident $INCIDENT_ID --severity sev1 --branch B`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source governance --runbook lane-failure-triage --incident $INCIDENT_ID --severity sev1 --branch C`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source governance --runbook lane-failure-triage --incident $INCIDENT_ID --severity sev1 --branch D`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source governance --runbook lane-failure-triage --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source governance --runbook lane-failure-triage --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source governance --runbook lane-failure-triage --incident $INCIDENT_ID`.
- Identity handoff API: `oya incident handoff --target identity --source governance --runbook lane-failure-triage --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source governance --runbook lane-failure-triage --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include oya_governance_lane_failure_triage_error_ratio, oya_governance_lane_failure_triage_lag_seconds, oya_governance_lane_failure_triage_queue_depth, current breaker state, and audit seal status.
- Keep governance-lane-failure-triage-circuit-breaker owner as axis-foundry + council-architecture + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after EVT-GOVERNANCE-LANE_FAILURE_TRIAGE-INCIDENT has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/governance/dashboards/` for dashboard names and operational panels.
- `microservices/governance/slos/` for OpenSLO alert vocabulary and threshold alignment.
- `microservices/governance/policy/` for named policy and authorization surfaces.
- `microservices/governance/catalog/` for component and owner vocabulary.
- Existing thin runbook topic `lane-failure-triage` was preserved as the scenario anchor while replacing generic steps with concrete commands.
