---
doc_class: Runbook
title: Kyb Kyc Pipeline Stalled
status: Accepted
date: 2026-05-20
microservice: tenancy
severity: sev1
audience: oncall-engineer
owner_team: axis-tenancy + ops-sre-reliability + ops-security
source_wave: codex-runbooks-substrate-w1
change_scope: substance rewrite of thin existing runbook
doc_status: published
---

# Runbook: Kyb Kyc Pipeline Stalled

## Operator Contract
- Runbook id: tenancy-kyb-kyc-pipeline-stalled.
- Primary service namespace: `tenancy`.
- Owning rotation: PagerDuty oya-tenancy-primary; data-boundary security secondary.
- Incident channel: `#inc-tenancy-boundary`.
- External dependencies: Citus Data support; Oracle PostgreSQL support; Cloudflare Zero Trust support.
- API authority: `https://tenancy.internal.oyatie.dev/v1/tenancy/kyb-kyc-pipeline-stalled/incident-handoff`.
- Audit event class: `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT` with ADR-0263 fields `incident_id`, `tenant_id`, `cell_id`, `microservice`, `runbook_id`, `decision_id`, `evidence_hash`, `operator_id`.
- Stop condition: mitigation has held for 30 minutes, TenancyKybKycPipelineStalledCritical is green, and all handoff APIs in Cross-µservice Coordination return `202 accepted`.
- Safety invariant: never clear the incident until `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT` is sealed and the postmortem skeleton exists under `evidence/postmortems/tenancy-kyb-kyc-pipeline-stalled-<incident-id>.md`.

## Trigger Conditions
- Page on alert `TenancyKybKycPipelineStalledCritical` when `oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio > 0.02` for 10 minutes in any production cell.
- Page on alert `TenancyKybKycPipelineStalledSloBurn` when `oya_tenancy_kyb_kyc_pipeline_stalled_lag_seconds > 300` for 2 consecutive evaluator windows.
- Open a sev0 if `oya_tenancy_kyb_kyc_pipeline_stalled_correctness_ratio < 0.9999` and the affected label set includes `tenant_id` or `principal_id`.
- Open a sev1 if `oya_tenancy_kyb_kyc_pipeline_stalled_queue_depth > 5000` for 15 minutes or retry backlog grows by more than 20 percent in one 5 minute window.
- Trigger from customer report when Support tags the case `tenancy.kyb-kyc-pipeline-stalled.customer_visible` in Zendesk.
- Trigger from CI when `cargo run -p oya-dev-cli -- gate validate tenancy-kyb-kyc-pipeline-stalled --production-snapshot` exits non-zero against the latest production evidence bundle.
- Primary dashboard: `https://grafana.dev.oyatie.internal/d/tenancy-substrate/kyb-kyc-pipeline-stalled?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=108`.
- Secondary dashboard: `https://grafana.dev.oyatie.internal/d/tenancy-substrate/kyb-kyc-pipeline-stalled?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213`.
- Loki explorer: `https://grafana.dev.oyatie.internal/explore?query={namespace="tenancy",runbook="kyb-kyc-pipeline-stalled"}`.
- Alertmanager route: `oyatie-tenancy-kyb-kyc-pipeline-stalled-critical`; silence only with incident commander approval and `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT` evidence.
- Synthetic probe: `oya ops probe tenancy kyb-kyc-pipeline-stalled --cell prod-us-east-1 --tenant synthetic-canary` returns `healthy=true`.
- Drift detector: `registry/tenancy/kyb-kyc-pipeline-stalled/expected-state.json` hash differs from live `https://tenancy.internal.oyatie.dev/v1/admin/state-hash`.

## Symptoms
- User-facing impact: kyb kyc pipeline stalled blocks or corrupts the tenancy control path for affected tenants.
- Operators see Grafana panel `cell-utilization / Kyb Kyc Pipeline Stalled burn rate` turn red before the primary alert resolves.
- Loki signature `tenancy.kyb_kyc_pipeline_stalled.incident_state=failed` appears with fields `incident_id`, `tenant_id`, `cell_id`, `decision_id`, `evidence_hash`.
- Kubernetes events include `reason=TenancyKybKycPipelineStalledDegraded` on deployment `tenancy-kyb-kyc-pipeline-stalled-worker`.
- Audit-chain shows missing or delayed `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT` entries when queried with `oya audit-chain query --event-class EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT --since 30m`.
- Metric pattern: `oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio` rises before `oya_tenancy_kyb_kyc_pipeline_stalled_lag_seconds`; if lag rises first, suspect dependency saturation rather than local regression.
- Metric pattern: `oya_tenancy_kyb_kyc_pipeline_stalled_queue_depth` increases while pod CPU stays below 40 percent; suspect downstream refusal or feature flag deadlock.
- Tenant-specific shape: one `tenant_id` dominates labels in `oya_tenancy_kyb_kyc_pipeline_stalled_queue_depth`; isolate before fleet mitigation.
- Fleet-wide shape: at least three cells report `TenancyKybKycPipelineStalledCritical` in one 15 minute window; switch to sev1 bridge even if individual tenants are low-volume.
- Log signature `decision=deny reason=kyb-kyc-pipeline-stalled.policy_guard` means the guard is working; investigate caller inputs before rollback.
- Log signature `decision=permit reason=kyb-kyc-pipeline-stalled.break_glass` means manual intervention is active; confirm two-person authorization.
- Log signature `audit_emit_status=stalled event_class=EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT` means mitigation cannot be closed until replay succeeds.

## Diagnostic Steps
1. Set incident variables: `export INCIDENT_ID=INC-tenancy-kyb-kyc-pipeline-stalled-$(date -u +%Y%m%dT%H%M%SZ); export CELL=prod-us-east-1; export TENANT=synthetic-canary`.
2. Confirm active alerts: `curl -s https://tenancy.internal.oyatie.dev/v1/alerts?runbook=kyb-kyc-pipeline-stalled | jq .alerts`.
3. Check Kubernetes rollout: `kubectl -n tenancy rollout status deploy/tenancy-kyb-kyc-pipeline-stalled-worker --timeout=60s`.
4. List unhealthy pods: `kubectl -n tenancy get pods -l app=tenancy-kyb-kyc-pipeline-stalled -o wide`.
5. Read structured logs: `kubectl -n tenancy logs deploy/tenancy-kyb-kyc-pipeline-stalled-worker --since=30m | rg "tenancy.kyb_kyc_pipeline_stalled.incident_state|TenancyKybKycPipelineStalledCritical|EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT"`.
6. Query Loki directly: `logcli query '{namespace="tenancy",runbook="kyb-kyc-pipeline-stalled"}' --since=30m --limit=200`.
7. Check Prometheus fast burn: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio{cell="prod-us-east-1"}'`.
8. Check lag: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_tenancy_kyb_kyc_pipeline_stalled_lag_seconds{cell="prod-us-east-1"}'`.
9. Check queue: `curl -G https://mimir.dev.oyatie.internal/prometheus/api/v1/query --data-urlencode 'query=oya_tenancy_kyb_kyc_pipeline_stalled_queue_depth{cell="prod-us-east-1"}'`.
10. Open primary dashboard: `open "https://grafana.dev.oyatie.internal/d/tenancy-substrate/kyb-kyc-pipeline-stalled?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=108&var-incident=$INCIDENT_ID"`.
11. Open secondary dashboard: `open "https://grafana.dev.oyatie.internal/d/tenancy-substrate/kyb-kyc-pipeline-stalled?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=213&var-tenant=$TENANT"`.
12. Verify audit-chain emission: `oya audit-chain query --event-class EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT --since 30m --cell $CELL --tenant $TENANT`.
13. Verify service state: `oya ops tenancy kyb-kyc-pipeline-stalled status --cell $CELL --tenant $TENANT --output json`.
14. Run production snapshot gate: `cargo run -p oya-dev-cli -- gate validate tenancy-kyb-kyc-pipeline-stalled --production-snapshot --cell $CELL`.
15. Check Cargo owner crate: `cargo test -p oya-tenancy-domain kyb_kyc_pipeline_stalled -- --nocapture`.
16. Check API contract smoke: `curl -s https://tenancy.internal.oyatie.dev/v1/tenancy/kyb-kyc-pipeline-stalled/incident-handoff -H "x-oya-tenant: $TENANT"`.
17. Inspect config: `kubectl -n tenancy get configmap tenancy-kyb-kyc-pipeline-stalled-config -o yaml`.
18. Inspect feature flags: `oya flags get oya.tenancy.kyb_kyc_pipeline_stalled.incident_hold --cell $CELL --tenant $TENANT --output yaml`.
19. Inspect circuit breaker: `oya ops breaker status tenancy-kyb-kyc-pipeline-stalled-circuit-breaker --cell $CELL --tenant $TENANT`.
20. Check recent deploy: `kubectl -n tenancy rollout history deploy/tenancy-kyb-kyc-pipeline-stalled-worker | tail -20`.
21. Check policy file: `test -f microservices/tenancy/policy/rls-isolation.cedar || test -f microservices/tenancy/policy/rls-isolation.md`.
22. Check SLO files: `ls microservices/tenancy/slos/*.openslo.yaml | sort`.
23. Check catalog components: `find microservices/tenancy/catalog -maxdepth 1 -type f | sort | rg "tenancy|kyb"`.
24. Confirm no cross-cell spread: `oya ops cells query --metric oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio --window 30m --threshold 0.02`.
25. Snapshot evidence: `oya evidence snapshot --incident $INCIDENT_ID --microservice tenancy --runbook kyb-kyc-pipeline-stalled --output evidence/incidents/$INCIDENT_ID.json`.

### Diagnostic Decision Tree
```text
Kyb Kyc Pipeline Stalled incident decision tree
1. Is TenancyKybKycPipelineStalledCritical firing in more than one cell?
   |-- yes: declare fleet incident, page PagerDuty oya-tenancy-primary; data-boundary security secondary, and run cross-cell containment.
   |-- no: keep scope to the affected cell and continue tenant isolation checks.
2. Does oya_tenancy_kyb_kyc_pipeline_stalled_queue_depth grow while oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio is flat?
   |-- yes: downstream dependency or replay backlog; choose mitigation branch B.
   |-- no: local regression or bad input; continue branch selection.
3. Does audit-chain show EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT gaps?
   |-- yes: do not close; run evidence replay before resolution.
   |-- no: mitigation can proceed after state is green.
4. Is customer or regulator impact confirmed?
   |-- yes: promote severity, open #inc-tenancy-boundary, and notify compliance handoff.
   |-- no: keep internal incident and collect evidence.
```
- Branch A (backlog with healthy dependencies): use the matching mitigation block below and record `decision_branch=A` in `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT`.
- Branch B (downstream dependency degraded): use the matching mitigation block below and record `decision_branch=B` in `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT`.
- Branch C (bad deploy introduced regression): use the matching mitigation block below and record `decision_branch=C` in `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT`.
- Branch D (data replay or rebuild required): use the matching mitigation block below and record `decision_branch=D` in `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT`.

## Mitigation Steps
1. Acknowledge page: `pd incident ack --service tenancy --incident $INCIDENT_ID`.
2. Create bridge: `oya incident bridge create --incident $INCIDENT_ID --channel #inc-tenancy-boundary --severity sev1`.
3. Freeze risky automation: `oya flags set oya.tenancy.kyb_kyc_pipeline_stalled.incident_hold=true --cell $CELL --tenant $TENANT --reason $INCIDENT_ID`.
4. Enable circuit breaker: `oya ops breaker open tenancy-kyb-kyc-pipeline-stalled-circuit-breaker --cell $CELL --tenant $TENANT --ttl 30m --reason $INCIDENT_ID`.
5. Reduce blast radius: `kubectl -n tenancy scale deploy/tenancy-kyb-kyc-pipeline-stalled-worker --replicas=1`.
6. Protect tenant boundary: `oya tenancy quarantine --tenant $TENANT --reason tenancy-kyb-kyc-pipeline-stalled --ttl 60m`.
7. Pause promotion: incident hold PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
8. Drain queue safely: `oya ops tenancy kyb-kyc-pipeline-stalled drain --cell $CELL --tenant $TENANT --max-items 500 --dry-run`.
9. Execute bounded drain: `oya ops tenancy kyb-kyc-pipeline-stalled drain --cell $CELL --tenant $TENANT --max-items 500 --confirm $INCIDENT_ID`.
10. Replay missing audit events: `oya audit-chain replay --event-class EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT --incident $INCIDENT_ID --from evidence/incidents/$INCIDENT_ID.json`.
11. Rollback last deploy if causal: `kubectl -n tenancy rollout undo deploy/tenancy-kyb-kyc-pipeline-stalled-worker`.
12. Raise HPA cap if saturation: `kubectl -n tenancy patch hpa tenancy-kyb-kyc-pipeline-stalled-worker --type merge -p '{"spec":{"maxReplicas":12}}'`.
13. Throttle hot tenant: `oya ops rate-limit set --tenant $TENANT --surface tenancy.kyb-kyc-pipeline-stalled --rps 25 --ttl 30m`.
14. Block abusive principal: `oya identity principal suspend --principal suspected-abuse --tenant $TENANT --reason $INCIDENT_ID`.
15. Protect evidence: `oya evidence freeze --incident $INCIDENT_ID --paths microservices/tenancy/runbooks/kyb-kyc-pipeline-stalled.md,evidence/incidents/$INCIDENT_ID.json`.
16. Notify service owners: `oya notify service-owner --microservice tenancy --incident $INCIDENT_ID --channel #inc-tenancy-boundary`.
17. Open external vendor ticket: `oya vendor ticket open --vendor primary-tenancy --incident $INCIDENT_ID --summary kyb-kyc-pipeline-stalled`.
18. Confirm breaker effect: `oya ops breaker status tenancy-kyb-kyc-pipeline-stalled-circuit-breaker --cell $CELL --tenant $TENANT --expect open`.
19. Confirm user impact reduced: `curl -s https://tenancy.internal.oyatie.dev/v1/tenancy/kyb-kyc-pipeline-stalled/incident-handoff/health -H "x-oya-tenant: $TENANT"`.
20. Emit mitigation audit: `oya audit-chain emit --event-class EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT --incident $INCIDENT_ID --field mitigation=active --field runbook=kyb-kyc-pipeline-stalled`.

### Mitigation Branch Guidance
- Branch A: backlog with healthy dependencies.
  - Required action: keep `tenancy-kyb-kyc-pipeline-stalled-circuit-breaker` open until `oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/tenancy-substrate/kyb-kyc-pipeline-stalled?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=108` to the incident.
  - Required audit: emit `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT` with `branch=A`, `operator_id`, and `evidence_hash`.
- Branch B: downstream dependency degraded.
  - Required action: keep `tenancy-kyb-kyc-pipeline-stalled-circuit-breaker` open until `oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/tenancy-substrate/kyb-kyc-pipeline-stalled?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=109` to the incident.
  - Required audit: emit `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT` with `branch=B`, `operator_id`, and `evidence_hash`.
- Branch C: bad deploy introduced regression.
  - Required action: keep `tenancy-kyb-kyc-pipeline-stalled-circuit-breaker` open until `oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/tenancy-substrate/kyb-kyc-pipeline-stalled?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=110` to the incident.
  - Required audit: emit `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT` with `branch=C`, `operator_id`, and `evidence_hash`.
- Branch D: data replay or rebuild required.
  - Required action: keep `tenancy-kyb-kyc-pipeline-stalled-circuit-breaker` open until `oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio` is below 0.005 for 3 windows.
  - Required evidence: attach dashboard panel `https://grafana.dev.oyatie.internal/d/tenancy-substrate/kyb-kyc-pipeline-stalled?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=111` to the incident.
  - Required audit: emit `EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT` with `branch=D`, `operator_id`, and `evidence_hash`.

## Resolution Steps
1. Identify code owner path: `rg "kyb_kyc_pipeline_stalled|TenancyKybKycPipelineStalledCritical|tenancy.kyb_kyc_pipeline_stalled.incident_state" crates microservices/tenancy -g "!microservices/tenancy/runbooks/**"`.
2. Patch domain invariant: `edit oya-tenancy-domain where kyb_kyc_pipeline_stalled state transition is validated`.
3. Patch API guard: `edit microservices/tenancy/contracts/openapi.yaml or catalog REST binding if the failing path is north-south`.
4. Patch policy: `edit microservices/tenancy/policy/rls-isolation.cedar or .md with explicit deny/permit branch`.
5. Patch runtime config: `edit microservices/tenancy/iac/k8s-deployment.yaml or secret-bindings.yaml if deploy/config drift caused the incident`.
6. Add regression test: `cargo test -p oya-tenancy-domain kyb_kyc_pipeline_stalled_incident_regression -- --nocapture`.
7. Add gate evidence: `cargo run -p oya-dev-cli -- gate validate tenancy-kyb-kyc-pipeline-stalled --fixture incident-kyb-kyc-pipeline-stalled.json`.
8. Add SLO assertion: `update microservices/tenancy/slos/* with alert TenancyKybKycPipelineStalledCritical when this was a missing alert`.
9. Add dashboard panel: `update microservices/tenancy/dashboards/cell-utilization.json with oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio, oya_tenancy_kyb_kyc_pipeline_stalled_lag_seconds, and oya_tenancy_kyb_kyc_pipeline_stalled_queue_depth`.
10. Rebuild affected crate: `cargo check -p oya-tenancy-domain --all-targets`.
11. Run targeted tests: `cargo test -p oya-tenancy-domain --all-features`.
12. Run policy validation: `cargo run -p oya-dev-cli -- gate validate tenancy-policy --microservice tenancy`.
13. Deploy canary: `oya deploy canary --microservice tenancy --component kyb-kyc-pipeline-stalled-worker --cell $CELL --weight 1`.
14. Watch burn rate: `oya ops watch --metric oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio --threshold 0.005 --window 30m --cell $CELL`.
15. Close circuit breaker: `oya ops breaker close tenancy-kyb-kyc-pipeline-stalled-circuit-breaker --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
16. Unfreeze automation: `oya flags set oya.tenancy.kyb_kyc_pipeline_stalled.incident_hold=false --cell $CELL --tenant $TENANT --reason resolved-$INCIDENT_ID`.
17. Resume promotion: recovery PR against `dev` (plain `git`; Jenkins + `oya gate run-all --ci-required` required).
18. Seal resolution audit: `oya audit-chain emit --event-class EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT --incident $INCIDENT_ID --field resolution=complete --field runbook=kyb-kyc-pipeline-stalled`.
19. Verify seal: `oya audit-chain verify --event-class EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT --incident $INCIDENT_ID`.
20. Attach final evidence: `oya evidence attach --incident $INCIDENT_ID --file evidence/incidents/$INCIDENT_ID.json --kind final-resolution`.

### Code Paths To Inspect First
- `oya-tenancy-domain`: inspect for kyb_kyc_pipeline_stalled invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 1.
- `oya-tenancy-kernel`: inspect for kyb_kyc_pipeline_stalled invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 2.
- `oya-tenancy-api`: inspect for kyb_kyc_pipeline_stalled invariants, alert emission, and ADR-0263 evidence fields before touching adjacent code path 3.
- `microservices/tenancy/contracts/`: verify this surface only when the incident evidence points there.
- `microservices/tenancy/dashboards/cell-utilization.json`: verify this surface only when the incident evidence points there.
- `microservices/tenancy/slos/`: verify this surface only when the incident evidence points there.
- `microservices/tenancy/policy/rls-isolation.*`: verify this surface only when the incident evidence points there.

## Verification Checklist
- TenancyKybKycPipelineStalledCritical and TenancyKybKycPipelineStalledSloBurn are both resolved in Alertmanager for 30 minutes.
- oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio < 0.005 for 3 consecutive 10 minute windows.
- oya_tenancy_kyb_kyc_pipeline_stalled_lag_seconds < 120 for all production cells.
- oya_tenancy_kyb_kyc_pipeline_stalled_queue_depth is draining and not growing for the affected tenant.
- dashboard https://grafana.dev.oyatie.internal/d/tenancy-substrate/kyb-kyc-pipeline-stalled?orgId=1&var-cell=prod-us-east-1&var-pack=canonical-base&viewPanel=108 shows green panels for the affected cell.
- audit-chain query for EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT returns mitigation and resolution events.
- circuit breaker tenancy-kyb-kyc-pipeline-stalled-circuit-breaker is closed after rollback window.
- feature flag oya.tenancy.kyb_kyc_pipeline_stalled.incident_hold is false for the affected tenant unless long-term hold is approved.
- runbook invocation evidence is attached to evidence/incidents/$INCIDENT_ID.json.
- service owner acknowledged final handoff in #inc-tenancy-boundary.

## Postmortem Template
Use this exact skeleton for the incident document. The field names are intentionally stable for ADR-0263 audit emission extraction.
```markdown
---
doc_class: IncidentPostmortem
runbook_id: tenancy-kyb-kyc-pipeline-stalled
microservice: tenancy
event_class: EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT
incident_id: <INC-...>
severity: sev1
status: draft
detected_at: <UTC>
mitigated_at: <UTC>
resolved_at: <UTC>
commander: <handle>
evidence_hash: <sha256>
---

# Kyb Kyc Pipeline Stalled postmortem

## Summary
- What happened in tenancy/kyb-kyc-pipeline-stalled.
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
- Emit EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT with incident_id, tenant_id, cell_id, principal_id, decision_id, evidence_hash, operator_id, runbook_id.
- Attach dashboard snapshot URLs and command transcripts.
- Seal mitigation and resolution events before closure.

## Corrective Actions
- Action, owner, due date, validation command, linked issue.
```

## Escalation Path
- Primary on-call: PagerDuty oya-tenancy-primary; data-boundary security secondary.
- Incident SLA: ack 3m for sev0/sev1, 10m for sev2, isolation checkpoint every 10m until contained.
- Incident commander: first responder from axis-tenancy + ops-sre-reliability + ops-security; transfer only by explicit message in #inc-tenancy-boundary.
- Security escalation: page `ops-security-primary` immediately for sev0, data-boundary, credential, or audit-seal symptoms.
- Compliance escalation: page `dpo-office-duty` when tenant data, regulator evidence, or breach clock symptoms are present.
- Architecture escalation: page `council-architecture-reviewer` before manual bypass, policy rollback, or invariant relaxation.
- External vendors: Citus Data support; Oracle PostgreSQL support; Cloudflare Zero Trust support. Open a ticket once local dependency health is proven and vendor dependency remains suspect.
- Customer communications: use status page component `oyatie-tenancy-kyb-kyc-pipeline-stalled` and keep private details in the incident channel.
- Regulatory clock: if any tenant data exposure is possible, start the compliance 72h assessment timer even if exposure is unconfirmed.
- Executive notice: sev0 or fleet-wide sev1 goes to `#exec-incident-readout` within 30 minutes.

## Cross-µservice Coordination
- Notify `identity`: `oya incident handoff --target identity --source tenancy --runbook kyb-kyc-pipeline-stalled --incident $INCIDENT_ID --severity sev1 --branch A`; expect `202 accepted`.
- Require `identity` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `compliance`: `oya incident handoff --target compliance --source tenancy --runbook kyb-kyc-pipeline-stalled --incident $INCIDENT_ID --severity sev1 --branch B`; expect `202 accepted`.
- Require `compliance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `governance`: `oya incident handoff --target governance --source tenancy --runbook kyb-kyc-pipeline-stalled --incident $INCIDENT_ID --severity sev1 --branch C`; expect `202 accepted`.
- Require `governance` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Notify `observability`: `oya incident handoff --target observability --source tenancy --runbook kyb-kyc-pipeline-stalled --incident $INCIDENT_ID --severity sev1 --branch D`; expect `202 accepted`.
- Require `observability` to return `handoff_id` and `owner_rotation`; paste both into the incident timeline.
- Observability handoff API: `oya incident handoff --target observability --source tenancy --runbook kyb-kyc-pipeline-stalled --incident $INCIDENT_ID`.
- Governance handoff API: `oya incident handoff --target governance --source tenancy --runbook kyb-kyc-pipeline-stalled --incident $INCIDENT_ID`.
- Compliance handoff API: `oya incident handoff --target compliance --source tenancy --runbook kyb-kyc-pipeline-stalled --incident $INCIDENT_ID`.
- Identity handoff API: `oya incident handoff --target identity --source tenancy --runbook kyb-kyc-pipeline-stalled --incident $INCIDENT_ID`.
- Tenancy handoff API: `oya incident handoff --target tenancy --source tenancy --runbook kyb-kyc-pipeline-stalled --incident $INCIDENT_ID`.

## Handoff Notes
- Do not hand off with only the alert name; include oya_tenancy_kyb_kyc_pipeline_stalled_error_ratio, oya_tenancy_kyb_kyc_pipeline_stalled_lag_seconds, oya_tenancy_kyb_kyc_pipeline_stalled_queue_depth, current breaker state, and audit seal status.
- Keep tenancy-kyb-kyc-pipeline-stalled-circuit-breaker owner as axis-tenancy + ops-sre-reliability + ops-security until the receiving service explicitly accepts.
- If another runbook owns the downstream fix, link this incident as upstream and keep this runbook open until downstream verification returns green.
- Close only after EVT-TENANCY-KYB_KYC_PIPELINE_STALLED-INCIDENT has a sealed resolution row and every coordination endpoint above has either accepted or explicitly declined scope.

## Sources Checked During This Substance Pass
- `microservices/tenancy/dashboards/` for dashboard names and operational panels.
- `microservices/tenancy/slos/` for OpenSLO alert vocabulary and threshold alignment.
- `microservices/tenancy/policy/` for named policy and authorization surfaces.
- `microservices/tenancy/catalog/` for component and owner vocabulary.
- Existing thin runbook topic `kyb-kyc-pipeline-stalled` was preserved as the scenario anchor while replacing generic steps with concrete commands.
