---
doc_class: Runbook
title: Held Promotion Recovery
status: Accepted
date: 2026-05-20
microservice: observability
severity: sev3
audience: oncall-engineer
owner_team: axis-observability + ops-sre-reliability
source_wave: codex-runbooks-substrate-w1
change_scope: substance rewrite of thin existing runbook
doc_status: published
---

# Runbook: Held Promotion Recovery

## Operator Contract
- Runbook id: observability-held-promotion-recovery.
- Primary service namespace: `observability`.
- Owning rotation: PagerDuty oya-observability-primary; Opsgenie ops-sre-reliability-secondary.
- Incident channel: `#inc-observability-live`.
- External dependencies: Grafana Enterprise Support; ClickHouse support; Oracle OCI object storage status desk.
- API authority: `https://observability.internal.oyatie.dev/v1/observability/held-promotion-recovery/incident-handoff`.
- Audit event class: `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, ObservabilityHeldPromotionRecoveryCritical is green, and all handoff APIs in Cross-µservice Coordination return `202 accepted`.
- Safety invariant: never clear the incident until `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/observability-held-promotion-recovery-<incident-id>.md`.

## Trigger Conditions
- Page on alert `ObservabilityHeldPromotionRecoveryCritical` when `oya_observability_held_promotion_recovery_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `ObservabilityHeldPromotionRecoverySloBurn` when `oya_observability_held_promotion_recovery_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev0 if `oya_observability_held_promotion_recovery_correctness_ratio < 0.9999` and the affected label set includes `tenant_id` or `principal_id`.
- Open a sev1 if `oya_observability_held_promotion_recovery_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `observability.held-promotion-recovery.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate observability-held-promotion-recovery --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/observability-substrate/held-promotion-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=107`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/observability-substrate/held-promotion-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="observability",runbook="held-promotion-recovery"}`.
- Alertmanager route: `oyatie-observability-held-promotion-recovery-critical`; silence only with incident commander approval and `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT` evidence.
- Synthetic probe: `oya ops probe observability held-promotion-recovery --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/observability/held-promotion-recovery/expected-state.json` hash differs from live `https://observability.internal.oyatie.dev/v1/admin/state-hash`.

## Symptoms
- User-facing impact: held promotion recovery blocks or corrupts the observability control path for affected tenants.
- Operators see Grafana panel `clickhouse-ingest-throughput / Held Promotion Recovery burn rate` turn red before the primary alert resolves.
- Loki signature `observability.held_promotion_recovery.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=ObservabilityHeldPromotionRecoveryDegraded` on deployment `observability-held-promotion-recovery-worker`.
- Audit-chain shows missing or delayed `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT` entries when queried with `oya audit-chain query --event-class EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT --since 30m`.
- Metric pattern: `oya_observability_held_promotion_recovery_error_ratio` rises before `oya_observability_held_promotion_recovery_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_observability_held_promotion_recovery_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_observability_held_promotion_recovery_queue_depth`; isolate before fleet mitigation.
- Fleet-wide shape: at least three cells report `ObservabilityHeldPromotionRecoveryCritical` in one 15 minute window; switch to sev1 bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=held-promotion-recovery.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=held-promotion-recovery.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT` means mitigation cannot be closed until replay succeeds.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-observability-held-promotion-recovery-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://observability.internal.oyatie.dev/v1/alerts?runbook=held-promotion-recovery | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n observability rollout status deploy/observability-held-promotion-recovery-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n observability get pods -l app=observability-held-promotion-recovery -o wide`.
5. Read structured logs: `kubectl -n observability logs deploy/observability-held-promotion-recovery-worker --since=30m | rg "observability.held_promotion_recovery.incident_state|ObservabilityHeldPromotionRecoveryCritical|EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="observability",runbook="held-promotion-recovery"}' --since=30m --limit=200`.
7. Check Prometheus fast burn: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_observability_held_promotion_recovery_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_observability_held_promotion_recovery_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_observability_held_promotion_recovery_queue_depth{cell="prod-us-east-1"}'`.
10. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/observability-substrate/held-promotion-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=107&var-incident=$INCIDENT_ID"`.
11. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/observability-substrate/held-promotion-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=210&var-tenant=$TENANT"`.
12. Verify audit-chain emission: `oya audit-chain query --event-class EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
13. Verify service state: `oya ops observability held-promotion-recovery status --cell $CELL --tenant $TENANT --output json`.
14. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate observability-held-promotion-recovery --production-snapshot --cell $CELL`.
15. Check Cargo owner crate: `cargo test -p oya-observability-domain held_promotion_recovery -- --nocapture`.
16. Check API contract smoke: `curl -s https://observability.internal.oyatie.dev/v1/observability/held-promotion-recovery/incident-handoff -H "x-oya-tenant: $TENANT"`.
17. Inspect config: `kubectl -n observability get configmap observability-held-promotion-recovery-config -o yaml`.
18. Inspect feature flags: `oya flags get oya.observability.held_promotion_recovery.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
19. Inspect circuit breaker: `oya ops breaker status observability-held-promotion-recovery-circuit-breaker --cell $CELL --tenant $TENANT`.
20. Check recent deploy: `kubectl -n observability rollout history deploy/observability-held-promotion-recovery-worker | tail -20`.
21. Check policy file: `test -f microservices/observability/policy/tenant-isolation.cedar || test -f microservices/observability/policy/tenant-isolation.md`.
22. Check SLO files: `ls observability/observability/slos/*.openslo.yaml | sort`.
23. Check catalog components: `find observability/catalog -maxdepth 1 -type f | sort | rg "observability|held"`.
24. Confirm no cross-cell spread: `oya ops cells query --metric oya_observability_held_promotion_recovery_error_ratio --window 30m --threshold 0.02`.
25. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice observability --runbook held-promotion-recovery --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Held Promotion Recovery incident decision tree
1. Is ObservabilityHeldPromotionRecoveryCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-observability-primary; Opsgenie ops-sre-reliability-secondary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_observability_held_promotion_recovery_queue_depth grow while oya_observability_held_promotion_recovery_error_ratio is flat?
   |-- yes: downstream dependency or replay backlog; choose mitigation branch B.
   |-- no: local regression or bad input; continue branch selection.
3. Does audit-chain show EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer or regulator impact confirmed?
   |-- yes: promote severity, open #inc-observability-live, and notify compliance handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (single tenant isolated): use the matching mitigation block below and record `decision_branch=A` in `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT`.
- Branch B (fleet-wide propagation): use the matching mitigation block below and record `decision_branch=B` in `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT`.
- Branch C (dependency regression): use the matching mitigation block below and record `decision_branch=C` in `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT`.
- Branch D (operator ceremony incomplete): use the matching mitigation block below and record `decision_branch=D` in `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service observability --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-observability-live --severity sev3`.
3. Freeze risky automation: `oya flags set oya.observability.held_promotion_recovery.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open observability-held-promotion-recovery-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n observability scale deploy/observability-held-promotion-recovery-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason observability-held-promotion-recovery --ttl 60m`.
7. Pause promotion: keep the incident rollback/fix PR unmerged and require Jenkins promotion checks to remain held/failing for `$INCIDENT_ID` (runbook: held-promotion-recovery).
8. Drain queue safely: `oya ops observability held-promotion-recovery drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops observability held-promotion-recovery drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n observability rollout undo deploy/observability-held-promotion-recovery-worker`.
12. Raise HPA cap if saturation: `kubectl -n observability patch hpa observability-held-promotion-recovery-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface observability.held-promotion-recovery --rps 25 --ttl 30m`.
14. Block abusive principal: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths observability/runbooks/held-promotion-recovery.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice observability --incident $INCIDENT_ID --channel #inc-observability-live`.
17. Open external vendor ticket: `oya vendor ticket open --vendor primary-observability --incident $INCIDENT_ID --summary held-promotion-recovery`.
18. Confirm breaker effect: `oya ops breaker status observability-held-promotion-recovery-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://observability.internal.oyatie.dev/v1/observability/held-promotion-recovery/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=held-promotion-recovery`.

### Mitigation Branch Guidance
- Branch A: single tenant isolated.
  - Required action: keep `observability-held-promotion-recovery-circuit-breaker` open until `oya_observability_held_promotion_recovery_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/observability-substrate/held-promotion-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=107` to the incident.
  - Required audit: emit `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: fleet-wide propagation.
  - Required action: keep `observability-held-promotion-recovery-circuit-breaker` open until `oya_observability_held_promotion_recovery_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/observability-substrate/held-promotion-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=108` to the incident.
  - Required audit: emit `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: dependency regression.
  - Required action: keep `observability-held-promotion-recovery-circuit-breaker` open until `oya_observability_held_promotion_recovery_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/observability-substrate/held-promotion-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=109` to the incident.
  - Required audit: emit `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: operator ceremony incomplete.
  - Required action: keep `observability-held-promotion-recovery-circuit-breaker` open until `oya_observability_held_promotion_recovery_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/observability-substrate/held-promotion-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=110` to the incident.
  - Required audit: emit `EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "held_promotion_recovery|ObservabilityHeldPromotionRecoveryCritical|observability.held_promotion_recovery.incident_state" crates microservices/observability -g "!observability/runbooks/**"`.
2. Patch domain invariant: `edit oya-observability-domain where held_promotion_recovery state transition is validated`.
3. Patch API guard: `edit observability/diagnostics/contracts/openapi.yaml or catalog REST binding if the failing path is north-south`.
4. Patch policy: `edit microservices/observability/policy/tenant-isolation.cedar or .md with explicit deny/permit branch`.
5. Patch runtime config: `edit microservices/observability/iac/k8s-deployment.yaml or secret-bindings.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-observability-domain held_promotion_recovery_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate observability-held-promotion-recovery --fixture incident-held-promotion-recovery.json`.
8. Add SLO assertion: `update observability/observability/slos/* with alert ObservabilityHeldPromotionRecoveryCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/observability/dashboards/clickhouse-ingest-throughput.json with oya_observability_held_promotion_recovery_error_ratio, oya_observability_held_promotion_recovery_lag_seconds, and oya_observability_held_promotion_recovery_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-observability-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-observability-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate observability-policy --microservice observability`.
13. Deploy canary: `oya deploy canary --microservice observability --component held-promotion-recovery-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_observability_held_promotion_recovery_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close observability-held-promotion-recovery-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.observability.held_promotion_recovery.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: merge only after reviewer approval plus green Jenkins CI and `oya gate run-all --ci-required`; record `resolved-$INCIDENT_ID` in the incident evidence.
18. Seal resolution audit: `oya audit-chain emit --event-class EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=held-promotion-recovery`.
19. Verify seal: `oya audit-chain verify --event-class EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-observability-domain`: inspect for held_promotion_recovery invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 1.
- `oya-cloud-observability-api`: inspect for held_promotion_recovery invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 2.
- `oya-dev-cli`: inspect for held_promotion_recovery invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 3.
- `observability/contracts/`: verify this surface only when the incident evidence points there.
- `microservices/observability/dashboards/clickhouse-ingest-throughput.json`: verify this surface only when the incident evidence points there.
- `observability/observability/slos/`: verify this surface only when the incident evidence points there.
- `observability/policy/tenant-isolation.*`: verify this surface only when the incident evidence points there.

## Verification Checklist
- ObservabilityHeldPromotionRecoveryCritical and ObservabilityHeldPromotionRecoverySloBurn are both resolved in Alertmanager for 30 minutes.
- oya_observability_held_promotion_recovery_error_ratio < 0.005 for 3 consecutive 10 minute windows.
- oya_observability_held_promotion_recovery_lag_seconds < 120 for all production cells.
- oya_observability_held_promotion_recovery_queue_depth is draining and not growing for the affected tenant.
- dashboard https://grafana.dev.oyatie.internal/d/observability-substrate/held-promotion-recovery?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=107 shows green panels for the affected cell.
- audit-chain query for EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT returns mitigation and resolution events.
- circuit breaker observability-held-promotion-recovery-circuit-breaker is closed after rollback window.
- feature flag oya.observability.held_promotion_recovery.incident_hold is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to evidence/incidents/$INCIDENT_ID.json.
- service owner acknowledged final handoff in #inc-observability-live.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: observability-held-promotion-recovery
microservice: observability
event_class: EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT
incident_id: <INC-...>
severity: sev3
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Held Promotion Recovery postmortem

## Summary
- What happened in observability/held-promotion-recovery.
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
- Emit EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-observability-primary; Opsgenie ops-sre-reliability-secondary.
- Incident SLA: ack 5m for sev1, 15m for sev2, checkpoint every 20m.
- Incident commander: first responder from axis-observability + ops-sre-reliability; transfer only by explicit message in #inc-observability-live.
- Security escalation: page `ops-security-primary` immediately for sev0, data-boundary, credential, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, or breach clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Grafana Enterprise Support; ClickHouse support; Oracle OCI object storage status desk. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-observability-held-promotion-recovery` and keep private details in the incident channel.
- Regulatory clock: if any tenant data exposure is possible, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source observability --runbook held-promotion-recovery --incident $INCIDENT_ID --severity sev3 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `tenancy`: `oya incident handoff --target tenancy --source observability --runbook held-promotion-recovery --incident $INCIDENT_ID --severity sev3 --branch B`; expect `202 accepted`.
- Require `tenancy` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `governance`: `oya incident handoff --target governance --source observability --runbook held-promotion-recovery --incident $INCIDENT_ID --severity sev3 --branch C`; expect `202 accepted`.
- Require `governance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source observability --runbook held-promotion-recovery --incident $INCIDENT_ID --severity sev3 --branch D`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source observability --runbook held-promotion-recovery --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source observability --runbook held-promotion-recovery --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source observability --runbook held-promotion-recovery --incident $INCIDENT_ID`.
- Identity handoff API: `oya incident handoff --target identity --source observability --runbook held-promotion-recovery --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source observability --runbook held-promotion-recovery --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include oya_observability_held_promotion_recovery_error_ratio, oya_observability_held_promotion_recovery_lag_seconds, oya_observability_held_promotion_recovery_queue_depth, current breaker state, and audit seal status.
- Keep observability-held-promotion-recovery-circuit-breaker owner as axis-observability + ops-sre-reliability until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after EVT-OBSERVABILITY-HELD_PROMOTION_RECOVERY-INCIDENT has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `observability/dashboards/` for dashboard names and operational panels.
- `observability/observability/slos/` for OpenSLO alert vocabulary and threshold alignment.
- `observability/policy/` for named policy and authorization surfaces.
- `observability/catalog/` for component and owner vocabulary.
- Existing thin runbook topic `held-promotion-recovery` was preserved as the scenario anchor while replacing generic steps with concrete commands.
